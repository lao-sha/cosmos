mod config;
pub mod crypto;
mod discord_executor;
mod executor;
pub mod gateway;
pub mod group_config;
mod multicaster;
pub mod platform;
mod rate_limiter;
mod signer;
mod types;
pub mod local_store;
pub mod local_processor;
mod webhook;

use std::sync::Arc;
use std::time::Instant;

use axum::{routing::{get, post}, Router};
use tokio::sync::RwLock;
use tracing::{info, error};

use config::AgentConfig;
use signer::{KeyManager, SequenceManager};
use types::NodeInfo;

/// 全局应用状态
pub struct AppState {
    pub config: AgentConfig,
    pub key_manager: KeyManager,
    pub sequence_manager: SequenceManager,
    pub executor: executor::TelegramExecutor,
    pub discord_executor: Option<discord_executor::DiscordExecutor>,
    pub config_store: group_config::ConfigStore,
    pub local_store: local_store::LocalStore,
    pub http_client: reqwest::Client,
    pub nodes: RwLock<Vec<NodeInfo>>,
    pub start_time: Instant,
    pub webhook_limiter: rate_limiter::RateLimiter,
    pub execute_limiter: rate_limiter::RateLimiter,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nexus_agent=info".into()),
        )
        .init();

    // 加载 .env 文件（如果存在）
    dotenvy::dotenv().ok();

    info!("╔══════════════════════════════════════════╗");
    info!("║     Nexus Agent v0.1.0                  ║");
    info!("║     去中心化多节点验证 · Local Agent      ║");
    info!("╚══════════════════════════════════════════╝");

    // 加载配置
    let config = AgentConfig::from_env()?;
    info!(
        bot_id_hash = config.bot_id_hash_hex(),
        webhook_port = config.webhook_port,
        chain_rpc = config.chain_rpc,
        "配置已加载"
    );

    // 加载/生成 Ed25519 密钥
    let key_manager = KeyManager::load_or_generate(&config.data_dir)?;
    info!(
        public_key = key_manager.public_key_hex(),
        "Ed25519 公钥（请注册到链上）"
    );

    // 加载序列号
    let sequence_manager = SequenceManager::load_or_init(&config.data_dir)?;
    info!(current_sequence = sequence_manager.current(), "序列号已就绪");

    // HTTP 客户端（连接池）
    let http_client = reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // TODO: Sprint 2 Week 3 — 从链上订阅 ActiveNodeList
    // 当前: 从环境变量加载静态节点列表（开发用）
    let nodes = load_initial_nodes();
    info!(count = nodes.len(), "初始节点列表已加载");

    // TG API 执行器（复用共享 HTTP 客户端）
    let tg_executor = executor::TelegramExecutor::new(config.bot_token.clone(), http_client.clone());

    // Discord 执行器（仅在 discord/both 模式下创建）
    let discord_exec = if config.platform.needs_discord() {
        let dc = config.discord.as_ref().expect("Discord 配置缺失");
        Some(discord_executor::DiscordExecutor::new(
            dc.bot_token.clone(),
            dc.application_id.clone(),
            http_client.clone(),
        ))
    } else {
        None
    };

    // 群配置存储
    let config_store = group_config::ConfigStore::new(&config.data_dir);

    // 本地状态存储（防刷屏、警告计数等）
    let local_store = local_store::LocalStore::new();
    info!("LocalStore 已初始化");

    // 限流器: webhook 60s/1000次, execute 60s/200次
    let webhook_limiter = rate_limiter::RateLimiter::new(60, 1000);
    let execute_limiter = rate_limiter::RateLimiter::new(60, 200);

    let state = Arc::new(AppState {
        config: config.clone(),
        key_manager,
        sequence_manager,
        executor: tg_executor,
        discord_executor: discord_exec,
        config_store,
        local_store,
        http_client,
        nodes: RwLock::new(nodes),
        start_time: Instant::now(),
        webhook_limiter,
        execute_limiter,
    });

    // 定时清理 LocalStore 过期数据 (每 60 秒)
    {
        let state_ref = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                state_ref.local_store.cleanup_expired();
            }
        });
    }

    // ═══ 平台启动 ═══
    // Telegram: 注册 Webhook（仅在 telegram/both 模式下）
    if config.platform.needs_telegram() {
        info!("正在注册 Telegram Webhook...");
        match multicaster::register_telegram_webhook(
            &state.http_client,
            &config.bot_token,
            &config.webhook_url,
            &config.webhook_secret,
        ).await {
            Ok(_) => info!("Telegram Webhook 注册成功"),
            Err(e) => {
                error!(error = %e, "Telegram Webhook 注册失败");
            }
        }
    }

    // Discord: 启动 Gateway WebSocket（仅在 discord/both 模式下）
    if config.platform.needs_discord() {
        let dc_config = config.discord.as_ref().expect("Discord 配置缺失");
        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<platform::PlatformEvent>(256);

        // 启动 Gateway 连接（后台任务，自动重连）
        let gw = gateway::discord::DiscordGateway::new(
            dc_config.bot_token.clone(),
            dc_config.intents,
            event_tx,
        );
        tokio::spawn(async move {
            gw.run().await;
            error!("Discord Gateway 已退出");
        });
        info!("Discord Gateway 后台任务已启动");

        // 启动事件处理循环（Discord 事件 → 签名 → 多播到节点）
        let state_for_dc = state.clone();
        let dc_bot_id_hash_bytes = dc_config.bot_id_hash;
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                // 跳过 Bot 自己的消息
                if event.sender_is_bot {
                    continue;
                }

                // 将 PlatformEvent 转为 SignedMessage 并多播
                let raw_json = serde_json::to_vec(&event.raw_event).unwrap_or_default();

                let sequence = match state_for_dc.sequence_manager.next() {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!(error = %e, "Discord 序列号递增失败");
                        continue;
                    }
                };
                let timestamp = chrono::Utc::now().timestamp() as u64;

                let (signature, message_hash) = state_for_dc.key_manager.sign_message(
                    &dc_bot_id_hash_bytes,
                    sequence,
                    timestamp,
                    &raw_json,
                );

                let signed_message = types::SignedMessage {
                    owner_public_key: state_for_dc.key_manager.public_key_hex(),
                    bot_id_hash: hex::encode(dc_bot_id_hash_bytes),
                    sequence,
                    timestamp,
                    message_hash: hex::encode(message_hash),
                    platform_event: serde_json::from_slice(&raw_json).unwrap_or_default(),
                    owner_signature: hex::encode(signature),
                    platform: "discord".to_string(),
                };

                // 异步多播到节点
                let sc = state_for_dc.clone();
                tokio::spawn(async move {
                    let _ = multicaster::multicast_to_nodes(&sc, &signed_message).await;
                });
            }
        });
        info!("Discord 事件处理循环已启动");
    }

    // Axum HTTP 服务器（限制请求体 1MB 防止 OOM 攻击）
    let app = Router::new()
        .route("/webhook", post(webhook::handle_webhook))
        .route("/v1/execute", post(webhook::handle_execute))
        .route("/v1/group-config", get(group_config::handle_get_config))
        .route("/v1/group-config", post(group_config::handle_update_config))
        .route("/health", get(webhook::handle_health))
        .layer(axum::extract::DefaultBodyLimit::max(1024 * 1024))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.webhook_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(addr, "HTTP 服务器已启动，等待 Webhook...");

    axum::serve(listener, app).await?;

    Ok(())
}

/// 从环境变量加载初始节点列表
///
/// 格式: NODES=node_id@http://host:port,node_id2@http://host2:port2,...
fn load_initial_nodes() -> Vec<NodeInfo> {
    let nodes_str = std::env::var("NODES").unwrap_or_default();
    if nodes_str.is_empty() {
        return vec![];
    }

    nodes_str
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|entry| {
            let parts: Vec<&str> = entry.splitn(2, '@').collect();
            if parts.len() == 2 {
                Some(NodeInfo {
                    node_id: parts[0].trim().to_string(),
                    endpoint: parts[1].trim().to_string(),
                    node_public_key: String::new(),
                    status: "Active".to_string(),
                })
            } else {
                tracing::warn!(entry, "无法解析节点条目，期望格式: node_id@http://host:port");
                None
            }
        })
        .collect()
}
