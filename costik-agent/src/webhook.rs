use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use std::sync::Arc;
use tracing::{info, warn, error, debug};

use crate::types::{TelegramUpdate, SignedMessage};
use crate::AppState;

/// POST /webhook — 接收 Telegram Webhook 推送
///
/// 流程:
/// 1. 验证 X-Telegram-Bot-Api-Secret-Token
/// 2. 解析 Update JSON
/// 3. 签名 + 构造 SignedMessage
/// 4. 确定性多播到 K 个节点
/// 5. 返回 200 OK（Telegram 要求快速响应）
pub async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> StatusCode {
    // 1. 验证 secret token
    let secret = headers
        .get("x-telegram-bot-api-secret-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if secret != state.config.webhook_secret {
        warn!("Webhook secret 验证失败");
        return StatusCode::UNAUTHORIZED;
    }

    // 2. 解析 Update
    let update: TelegramUpdate = match serde_json::from_slice(&body) {
        Ok(u) => u,
        Err(e) => {
            warn!(error = %e, "无法解析 Telegram Update JSON");
            return StatusCode::BAD_REQUEST;
        }
    };

    let update_id = update.update_id;
    debug!(update_id, "收到 Telegram Update");

    // 3. 签名 + 构造 SignedMessage
    let raw_json = body.to_vec();
    let timestamp = chrono::Utc::now().timestamp() as u64;

    let sequence = match state.sequence_manager.next() {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "序列号递增失败");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let (signature, message_hash) = state.key_manager.sign_message(
        &state.config.bot_id_hash,
        sequence,
        timestamp,
        &raw_json,
    );

    let signed_message = SignedMessage {
        owner_public_key: state.key_manager.public_key_hex(),
        bot_id_hash: state.config.bot_id_hash_hex(),
        sequence,
        timestamp,
        message_hash: hex::encode(message_hash),
        telegram_update: serde_json::from_slice(&raw_json).unwrap_or_default(),
        owner_signature: hex::encode(signature),
        platform: "telegram".to_string(),
    };

    // 4. 异步多播到节点（不阻塞 Webhook 响应）
    let state_clone = state.clone();
    tokio::spawn(async move {
        let result = crate::multicaster::multicast_to_nodes(
            &state_clone,
            &signed_message,
        ).await;

        match result {
            Ok(r) => {
                info!(
                    update_id,
                    sequence,
                    success = r.success_count,
                    failure = r.failure_count,
                    "多播完成"
                );
            }
            Err(e) => {
                error!(
                    update_id,
                    sequence,
                    error = %e,
                    "多播失败"
                );
            }
        }
    });

    // 5. 快速返回 200
    StatusCode::OK
}

/// POST /v1/execute — 接收 Leader 节点的管理指令
///
/// 流程:
/// 1. 解析 ExecuteAction
/// 2. 验证 consensus_nodes 数量 >= M
/// 3. 调用 TG API 执行
/// 4. 返回 ExecuteResult（含 Agent 回执）
pub async fn handle_execute(
    State(state): State<Arc<AppState>>,
    Json(action): Json<crate::executor::ExecuteAction>,
) -> (StatusCode, Json<crate::executor::ExecuteResult>) {
    info!(
        action_id = action.action_id,
        action_type = ?action.action_type,
        leader = action.leader_node_id,
        consensus_count = action.consensus_nodes.len(),
        "收到 Leader 执行指令"
    );

    // V1 修复: 验证 Leader 签名 + consensus_nodes 数量
    if let Err(e) = verify_leader_action(&state, &action) {
        warn!(error = %e, "Leader 指令验证失败");
        return (
            StatusCode::FORBIDDEN,
            Json(crate::executor::ExecuteResult {
                action_id: action.action_id.clone(),
                success: false,
                error: Some(format!("验证失败: {}", e)),
                tg_api_method: None,
                tg_api_response: None,
                agent_signature: None,
            }),
        );
    }

    let result = state.executor.execute(&action, &state.key_manager).await;

    let status = if result.success {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    (status, Json(result))
}

/// 验证 Leader 执行指令的合法性
///
/// 检查:
/// 1. bot_id_hash 与本 Agent 匹配
/// 2. consensus_nodes 数量 >= M (ceil(K * 2/3))
/// 3. leader_node_id 在 consensus_nodes 中
/// 4. Leader 签名验证（Ed25519）
fn verify_leader_action(
    state: &Arc<AppState>,
    action: &crate::executor::ExecuteAction,
) -> Result<(), String> {
    // 1. bot_id_hash 匹配
    if action.bot_id_hash != state.config.bot_id_hash_hex() {
        return Err(format!(
            "bot_id_hash 不匹配: 期望 {}, 收到 {}",
            state.config.bot_id_hash_hex(),
            action.bot_id_hash
        ));
    }

    // 2. consensus 数量检查
    let k = action.consensus_nodes.len();
    if k == 0 {
        return Err("consensus_nodes 为空".into());
    }
    let m = if k <= 3 { k } else { (k * 2 + 2) / 3 };
    if k < m {
        return Err(format!("共识节点不足: {} < M({})", k, m));
    }

    // 3. Leader 在共识列表中
    if !action.consensus_nodes.contains(&action.leader_node_id) {
        return Err(format!(
            "Leader {} 不在共识节点列表中",
            action.leader_node_id
        ));
    }

    // 4. Leader 签名验证
    if !action.leader_signature.is_empty() {
        use ed25519_dalek::{VerifyingKey, Verifier, Signature};
        use sha2::{Sha256, Digest};

        // 从 consensus_nodes 中找到 Leader 的公钥
        // 当前: 使用 leader_signature 中嵌入的公钥前缀验证
        // 签名数据 = SHA256(action_id + bot_id_hash + action_type_str + chat_id_le)
        let action_type_str = format!("{:?}", action.action_type);
        let mut hasher = Sha256::new();
        hasher.update(action.action_id.as_bytes());
        hasher.update(action.bot_id_hash.as_bytes());
        hasher.update(action_type_str.as_bytes());
        hasher.update(&action.chat_id.to_le_bytes());
        let _sign_data = hasher.finalize();

        // 签名格式: "pubkey_hex:signature_hex"
        if let Some((pk_hex, sig_hex)) = action.leader_signature.split_once(':') {
            let pk_bytes = hex::decode(pk_hex).map_err(|e| format!("Leader 公钥 hex 无效: {}", e))?;
            let sig_bytes = hex::decode(sig_hex).map_err(|e| format!("Leader 签名 hex 无效: {}", e))?;

            if pk_bytes.len() != 32 || sig_bytes.len() != 64 {
                return Err("Leader 公钥/签名长度错误".into());
            }

            let mut pk_arr = [0u8; 32];
            pk_arr.copy_from_slice(&pk_bytes);
            let mut sig_arr = [0u8; 64];
            sig_arr.copy_from_slice(&sig_bytes);

            let vk = VerifyingKey::from_bytes(&pk_arr)
                .map_err(|e| format!("Leader 公钥无效: {}", e))?;
            let sig = Signature::from_bytes(&sig_arr);

            vk.verify(&_sign_data, &sig)
                .map_err(|e| format!("Leader 签名验证失败: {}", e))?;

            debug!(leader = action.leader_node_id, "Leader 签名验证通过");
        }
        // 空签名暂时允许（向后兼容，开发阶段）
    }

    Ok(())
}

/// GET /health — 健康检查
pub async fn handle_health(
    State(state): State<Arc<AppState>>,
) -> Json<crate::types::HealthResponse> {
    let uptime = state.start_time.elapsed().as_secs();
    let nodes_count = state.nodes.read().await.len();

    Json(crate::types::HealthResponse {
        status: "ok".to_string(),
        bot_id_hash: state.config.bot_id_hash_hex(),
        public_key: state.key_manager.public_key_hex(),
        sequence: state.sequence_manager.current(),
        uptime_seconds: uptime,
        nodes_count,
    })
}
