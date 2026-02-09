mod config;
pub mod crypto;
mod executor;
pub mod group_config;
mod multicaster;
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

    // 注册 Telegram Webhook
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
            // 不退出 — 允许手动注册 Webhook
        }
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
