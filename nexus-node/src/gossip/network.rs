use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use tokio::net::TcpListener;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message};
use tracing::{info, warn, debug, error};

use crate::chain_cache::ChainCache;
use crate::types::GossipEnvelope;

/// Gossip WebSocket 网络层
///
/// 职责:
/// 1. 启动 WebSocket 服务端（接收其他节点的连接）
/// 2. 连接到所有对等节点（WebSocket 客户端）
/// 3. 广播 outbound 消息到所有对等节点
/// 4. 接收 inbound 消息转发到 GossipEngine

/// 运行 Gossip 网络层（阻塞）
pub async fn run_gossip_network(
    node_id: String,
    gossip_port: u16,
    chain_cache: Arc<ChainCache>,
    mut outbound_rx: mpsc::UnboundedReceiver<GossipEnvelope>,
) -> anyhow::Result<()> {
    // 对等节点连接池: node_id → sender
    let peers: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    // 1. 启动 WebSocket 服务端
    let server_peers = peers.clone();
    let server_node_id = node_id.clone();
    tokio::spawn(async move {
        if let Err(e) = run_ws_server(gossip_port, server_node_id, server_peers).await {
            error!(error = %e, "Gossip WS 服务端退出");
        }
    });

    // 2. 定时连接到对等节点
    let connect_peers = peers.clone();
    let connect_cache = chain_cache.clone();
    let connect_node_id = node_id.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            discover_and_connect(
                &connect_node_id,
                &connect_cache,
                &connect_peers,
            ).await;
        }
    });

    // 3. 广播 outbound 消息
    info!(port = gossip_port, "Gossip 网络层已启动");
    while let Some(envelope) = outbound_rx.recv().await {
        let msg_bytes = match serde_json::to_vec(&envelope) {
            Ok(b) => b,
            Err(e) => {
                warn!(error = %e, "序列化 GossipEnvelope 失败");
                continue;
            }
        };
        let ws_msg = Message::Binary(msg_bytes);

        let peers_read = peers.read().await;
        for (peer_id, sender) in peers_read.iter() {
            if *peer_id == node_id {
                continue; // 不发给自己
            }
            if let Err(e) = sender.send(ws_msg.clone()) {
                debug!(peer = peer_id, error = %e, "发送到对等节点失败");
            }
        }
    }

    Ok(())
}

/// WebSocket 服务端
async fn run_ws_server(
    port: u16,
    node_id: String,
    peers: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>,
) -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!(addr, "Gossip WebSocket 服务端已启动");

    while let Ok((stream, peer_addr)) = listener.accept().await {
        let peers = peers.clone();
        let _node_id = node_id.clone();

        tokio::spawn(async move {
            let ws_stream = match accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    warn!(peer = %peer_addr, error = %e, "WebSocket 握手失败");
                    return;
                }
            };

            let (mut ws_sender, mut ws_receiver) = ws_stream.split();
            let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

            // 为此连接注册发送通道
            let peer_id = format!("inbound_{}", peer_addr);
            peers.write().await.insert(peer_id.clone(), tx);

            // 发送 → WebSocket
            let send_task = tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    if ws_sender.send(msg).await.is_err() {
                        break;
                    }
                }
            });

            // WebSocket → 接收处理
            while let Some(msg_result) = ws_receiver.next().await {
                match msg_result {
                    Ok(Message::Binary(data)) => {
                        if let Ok(envelope) = serde_json::from_slice::<GossipEnvelope>(&data) {
                            debug!(
                                from = envelope.sender_node_id,
                                msg_type = ?envelope.msg_type,
                                "收到 Gossip 消息"
                            );
                            // TODO: 转发到 GossipEngine 处理
                        }
                    }
                    Ok(Message::Ping(_data)) => {
                        // Pong 由 tungstenite 自动处理
                    }
                    Ok(Message::Close(_)) => break,
                    Err(e) => {
                        debug!(error = %e, "WebSocket 接收错误");
                        break;
                    }
                    _ => {}
                }
            }

            // 清理
            peers.write().await.remove(&peer_id);
            send_task.abort();
            debug!(peer = %peer_addr, "对等节点断开");
        });
    }

    Ok(())
}

/// 发现并连接到对等节点
async fn discover_and_connect(
    my_node_id: &str,
    cache: &ChainCache,
    peers: &Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>,
) {
    let all_nodes = cache.get_all_nodes();
    let connected = peers.read().await;

    for node in &all_nodes {
        if node.node_id == my_node_id {
            continue;
        }
        if connected.contains_key(&node.node_id) {
            continue;
        }

        // 从 HTTP endpoint 推导 WebSocket endpoint
        // http://host:8080 → ws://host:9090
        let ws_url = endpoint_to_ws(&node.endpoint);
        if ws_url.is_empty() {
            continue;
        }

        let node_id = node.node_id.clone();
        let peers = peers.clone();

        tokio::spawn(async move {
            match connect_async(&ws_url).await {
                Ok((ws_stream, _)) => {
                    info!(peer = node_id, url = ws_url, "已连接到对等节点");

                    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
                    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

                    peers.write().await.insert(node_id.clone(), tx);

                    // 发送任务
                    let send_node_id = node_id.clone();
                    let send_task = tokio::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            if ws_sender.send(msg).await.is_err() {
                                break;
                            }
                        }
                    });

                    // 接收处理
                    while let Some(msg_result) = ws_receiver.next().await {
                        match msg_result {
                            Ok(Message::Binary(data)) => {
                                if let Ok(envelope) = serde_json::from_slice::<GossipEnvelope>(&data) {
                                    debug!(
                                        from = envelope.sender_node_id,
                                        msg_type = ?envelope.msg_type,
                                        "收到对等消息"
                                    );
                                    // TODO: 转发到 GossipEngine
                                }
                            }
                            Ok(Message::Close(_)) => break,
                            Err(_) => break,
                            _ => {}
                        }
                    }

                    peers.write().await.remove(&node_id);
                    send_task.abort();
                    debug!(peer = send_node_id, "对等连接断开");
                }
                Err(e) => {
                    debug!(peer = node_id, error = %e, "连接对等节点失败");
                }
            }
        });
    }
}

/// HTTP endpoint → WebSocket endpoint
///
/// http://host:8080 → ws://host:9090
fn endpoint_to_ws(http_endpoint: &str) -> String {
    http_endpoint
        .replace("http://", "ws://")
        .replace("https://", "wss://")
        .replace(":8080", ":9090")
        .replace(":8081", ":9091")
        .replace(":8082", ":9092")
        .replace(":8083", ":9093")
}
