mod api;
mod chain_cache;
mod chain_client;
mod chain_submitter;
mod config;
// pub mod crypto; // TODO: Sprint 后续实现
mod gossip;
mod leader;
pub mod platform;
#[allow(dead_code)]
mod rule_engine;
#[allow(dead_code)]
mod types;
mod verifier;

use std::sync::Arc;
use std::time::Instant;

use axum::{routing::{get, post}, Router};
use tracing::{info, warn};

use chain_cache::ChainCache;
use chain_submitter::ChainSubmitter;
use config::NodeConfig;
use gossip::engine::GossipEngine;
use gossip::state::GossipState;

/// 全局应用状态
pub struct AppState {
    pub config: NodeConfig,
    pub chain_cache: Arc<ChainCache>,
    pub gossip_engine: GossipEngine,
    pub chain_submitter: Arc<ChainSubmitter>,
    pub start_time: Instant,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nexus_node=info".into()),
        )
        .init();

    dotenvy::dotenv().ok();

    info!("╔══════════════════════════════════════════╗");
    info!("║     Nexus Node v0.1.0                   ║");
    info!("║     去中心化多节点验证 · Project Node     ║");
    info!("╚══════════════════════════════════════════╝");

    let config = NodeConfig::from_env()?;
    info!(
        node_id = config.node_id,
        listen_port = config.listen_port,
        gossip_port = config.gossip_port,
        "配置已加载"
    );

    // 链上缓存 — 先从环境变量加载静态数据
    let chain_cache = Arc::new(ChainCache::new());
    chain_cache.load_from_env();

    // 尝试 subxt 连接链（可选 — 连接失败不阻止启动）
    let _chain_client = match chain_client::ChainClient::connect(
        &config.chain_rpc,
        load_or_generate_signer(&config.data_dir),
    ).await {
        Ok(client) => {
            info!("subxt 链客户端已连接");
            // 从链上刷新缓存
            let client = Arc::new(client);
            refresh_cache_from_chain(&client, &chain_cache).await;
            // 启动事件订阅后台任务
            spawn_chain_watcher(client.clone(), chain_cache.clone());
            Some(client)
        }
        Err(e) => {
            warn!(error = %e, "subxt 连接失败 — 使用静态缓存模式");
            None
        }
    };

    // Gossip 状态 + 引擎
    let gossip_state = Arc::new(GossipState::new());
    let gossip_engine = GossipEngine::new(
        config.node_id.clone(),
        gossip_state.clone(),
        chain_cache.clone(),
    );

    // 链上提交器
    let submitter = Arc::new(ChainSubmitter::new(config.node_id.clone()));
    submitter.clone().spawn_submit_loop();

    let state = Arc::new(AppState {
        config: config.clone(),
        chain_cache: chain_cache.clone(),
        gossip_engine,
        chain_submitter: submitter,
        start_time: Instant::now(),
    });

    // GC 定时任务
    let gc_state = gossip_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            gc_state.gc_expired_messages();
        }
    });

    // Gossip WebSocket 网络层
    let gossip_outbound_rx = state.gossip_engine.subscribe_outbound();
    let ws_cache = chain_cache.clone();
    let ws_node_id = config.node_id.clone();
    let ws_gossip_port = config.gossip_port;
    tokio::spawn(async move {
        if let Err(e) = gossip::network::run_gossip_network(
            ws_node_id,
            ws_gossip_port,
            ws_cache,
            gossip_outbound_rx,
        ).await {
            warn!(error = %e, "Gossip 网络层退出");
        }
    });

    // Axum HTTP API
    let app = Router::new()
        .route("/v1/message", post(api::handle_message))
        .route("/v1/status/{msg_id}", get(api::handle_message_status))
        .route("/health", get(api::handle_health))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.listen_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(addr, "HTTP API 已启动，等待 Agent 消息...");

    axum::serve(listener, app).await?;

    Ok(())
}

/// 从链上刷新缓存
async fn refresh_cache_from_chain(
    client: &chain_client::ChainClient,
    cache: &ChainCache,
) {
    match client.fetch_active_node_list().await {
        Ok(node_ids) => {
            for nid in &node_ids {
                if let Ok(Some(node_info)) = client.fetch_node(nid).await {
                    cache.add_node(node_info);
                }
            }
            info!(count = node_ids.len(), "从链上刷新节点列表完成");
        }
        Err(e) => warn!(error = %e, "获取链上节点列表失败"),
    }
}

/// 启动链上事件订阅
fn spawn_chain_watcher(
    client: Arc<chain_client::ChainClient>,
    cache: Arc<ChainCache>,
) {
    tokio::spawn(async move {
        use futures_util::StreamExt;
        match client.subscribe_finalized_blocks().await {
            Ok(mut stream) => {
                info!("链上区块订阅已启动");
                let mut block_count: u64 = 0;
                while let Some(block_result) = stream.next().await {
                    match block_result {
                        Ok(_block) => {
                            block_count += 1;
                            // 每 100 个区块刷新一次缓存
                            if block_count % 100 == 0 {
                                refresh_cache_from_chain(&client, &cache).await;
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, "区块订阅错误");
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            }
            Err(e) => warn!(error = %e, "区块订阅失败"),
        }
    });
}

/// 加载或生成节点签名密钥
fn load_or_generate_signer(data_dir: &str) -> subxt_signer::sr25519::Keypair {
    let key_path = std::path::Path::new(data_dir).join("node_signer.key");
    if key_path.exists() {
        if let Ok(seed_bytes) = std::fs::read(&key_path) {
            if seed_bytes.len() == 32 {
                let mut seed = [0u8; 32];
                seed.copy_from_slice(&seed_bytes);
                if let Ok(kp) = subxt_signer::sr25519::Keypair::from_secret_key(seed) {
                    info!("节点签名密钥已加载");
                    return kp;
                }
            }
        }
    }

    // 生成随机密钥并持久化
    let mut seed = [0u8; 32];
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(&mut seed);
    let kp = subxt_signer::sr25519::Keypair::from_secret_key(seed)
        .expect("有效的随机种子");

    // 持久化到文件
    std::fs::create_dir_all(data_dir).ok();
    if let Err(e) = std::fs::write(&key_path, &seed) {
        warn!(error = %e, "节点签名密钥持久化失败");
    } else {
        info!("已生成并保存新的节点签名密钥");
    }
    kp
}
