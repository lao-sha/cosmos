use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use tracing::{warn, debug};

use crate::types::SignedMessage;
use crate::AppState;

/// POST /v1/message — 接收 Agent 多播的签名消息
///
/// 流程:
/// 1. 解析 SignedMessage
/// 2. 四层验证
/// 3. 送入 Gossip 引擎
/// 4. 返回 200
pub async fn handle_message(
    State(state): State<Arc<AppState>>,
    Json(message): Json<SignedMessage>,
) -> StatusCode {
    let msg_id = crate::gossip::state::GossipState::make_msg_id(
        &message.bot_id_hash,
        message.sequence,
    );

    debug!(msg_id, sequence = message.sequence, "收到 Agent 消息");

    // 四层验证
    match crate::verifier::verify_signed_message(
        &message,
        &state.config.node_id,
        &state.chain_cache,
    ) {
        Ok(_) => {
            debug!(msg_id, "验证通过，送入 Gossip 引擎");
        }
        Err(e) => {
            warn!(msg_id, error = %e, "消息验证失败");
            return StatusCode::FORBIDDEN;
        }
    }

    // 送入 Gossip 引擎
    state.gossip_engine.on_agent_message(message);

    StatusCode::OK
}

/// GET /health — 健康检查
pub async fn handle_health(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let uptime = state.start_time.elapsed().as_secs();
    let active_msgs = state.gossip_engine.state.active_message_count();
    let processed = state.gossip_engine.state.messages_processed
        .load(std::sync::atomic::Ordering::Relaxed);

    Json(serde_json::json!({
        "status": "ok",
        "node_id": state.config.node_id,
        "uptime_seconds": uptime,
        "active_messages": active_msgs,
        "messages_processed": processed,
    }))
}

/// GET /v1/status/:msg_id — 查询消息状态（调试用）
pub async fn handle_message_status(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(msg_id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let status = state.gossip_engine.state.get_status(&msg_id);

    Json(serde_json::json!({
        "msg_id": msg_id,
        "status": format!("{:?}", status),
    }))
}
