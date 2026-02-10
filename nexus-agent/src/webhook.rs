use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use std::sync::Arc;
use tracing::{info, warn, error, debug};

use crate::executor::PlatformExecutor;
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
    // 0. 限流检查
    if !state.webhook_limiter.check().await {
        warn!("Webhook 限流触发");
        return StatusCode::TOO_MANY_REQUESTS;
    }

    // 1. 验证 secret token
    let secret = headers
        .get("x-telegram-bot-api-secret-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if secret != state.config.webhook_secret {
        warn!("Webhook secret 验证失败");
        return StatusCode::UNAUTHORIZED;
    }

    // 2. 解析 Update（只解析一次 JSON，同时获取结构体和 Value）
    let raw_update: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "无法解析 Telegram Update JSON");
            return StatusCode::BAD_REQUEST;
        }
    };
    let update: TelegramUpdate = match serde_json::from_value(raw_update.clone()) {
        Ok(u) => u,
        Err(e) => {
            warn!(error = %e, "Telegram Update 结构解析失败");
            return StatusCode::BAD_REQUEST;
        }
    };

    let update_id = update.update_id;
    debug!(update_id, "收到 Telegram Update");

    // B3.3: 管理员缓存自动刷新 — 如果缓存过期则异步拉取
    let chat_id = raw_update.pointer("/message/chat/id")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    if chat_id != 0 {
        let cache_miss = state.local_store.is_admin_cached(chat_id, 0).is_none();
        if cache_miss {
            let state_for_admin = state.clone();
            tokio::spawn(async move {
                if let Err(e) = refresh_admin_cache(&state_for_admin, chat_id).await {
                    debug!(error = %e, chat_id, "管理员缓存刷新失败");
                }
            });
        }
    }

    let group_config = state.config_store.get().map(|c| c.config);
    let local_actions = crate::local_processor::LocalProcessor::process(
        &raw_update,
        group_config.as_ref(),
        &state.local_store,
    );

    if !local_actions.is_empty() {
        let state_for_local = state.clone();
        let actions_clone = local_actions.clone();
        tokio::spawn(async move {
            for action in &actions_clone {
                if let Err(e) = execute_local_action(&state_for_local, action).await {
                    warn!(error = %e, reason = action.reason, "本地快速路径执行失败");
                }
            }
            // B3.5: 审计日志 — 本地快速路径执行的动作异步提交到 Node 网络
            submit_audit_log(&state_for_local, &actions_clone).await;
        });
    }

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
        platform_event: serde_json::from_slice(&raw_json).unwrap_or_default(),
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
    headers: HeaderMap,
    Json(action): Json<crate::executor::ExecuteAction>,
) -> (StatusCode, Json<crate::executor::ExecuteResult>) {
    // H6: Bearer Token 认证（如果设置了 EXECUTE_TOKEN）
    if let Some(ref expected_token) = state.config.execute_token {
        let provided = headers.get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .unwrap_or("");
        if provided != expected_token {
            warn!("Execute Bearer Token 认证失败");
            return (
                StatusCode::UNAUTHORIZED,
                Json(crate::executor::ExecuteResult {
                    action_id: action.action_id.clone(),
                    success: false,
                    error: Some("Unauthorized: invalid or missing Bearer token".into()),
                    api_method: None,
                    api_response: None,
                    agent_signature: None,
                }),
            );
        }
    }

    // 限流检查
    if !state.execute_limiter.check().await {
        warn!("Execute 限流触发");
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(crate::executor::ExecuteResult {
                action_id: action.action_id.clone(),
                success: false,
                error: Some("限流: 请求过多".into()),
                api_method: None,
                api_response: None,
                agent_signature: None,
            }),
        );
    }

    info!(
        action_id = action.action_id,
        action_type = ?action.action_type,
        leader = action.leader_node_id,
        consensus_count = action.consensus_nodes.len(),
        "收到 Leader 执行指令"
    );

    // V1 修复: 验证 Leader 签名 + consensus_nodes 数量
    if let Err(e) = verify_leader_action(&state, &action).await {
        warn!(error = %e, "Leader 指令验证失败");
        return (
            StatusCode::FORBIDDEN,
            Json(crate::executor::ExecuteResult {
                action_id: action.action_id.clone(),
                success: false,
                error: Some(format!("验证失败: {}", e)),
                api_method: None,
                api_response: None,
                agent_signature: None,
            }),
        );
    }

    // ═══ ConfigUpdate: 拦截配置更新动作，更新 ConfigStore 后发确认消息 ═══
    if let crate::executor::ActionType::ConfigUpdate(ref cu) = action.action_type {
        let result = handle_config_update(&state, &action, cu).await;
        let status = if result.success { StatusCode::OK } else { StatusCode::INTERNAL_SERVER_ERROR };
        return (status, Json(result));
    }

    // ═══ Warn E2E: 拦截 warn 相关动作，使用 LocalStore 管理计数 ═══
    if is_warn_action(&action) {
        let result = handle_warn_action(&state, &action).await;
        let status = if result.success { StatusCode::OK } else { StatusCode::INTERNAL_SERVER_ERROR };
        return (status, Json(result));
    }

    // 根据 platform 路由到对应执行器
    let result = if action.platform == "discord" {
        if let Some(ref dc_exec) = state.discord_executor {
            dc_exec.execute(&action, &state.key_manager).await
        } else {
            crate::executor::ExecuteResult {
                action_id: action.action_id.clone(),
                success: false,
                error: Some("Discord executor 未配置（PLATFORM 未设置为 discord 或 both）".into()),
                api_method: None,
                api_response: None,
                agent_signature: None,
            }
        }
    } else {
        state.executor.execute(&action, &state.key_manager).await
    };

    let status = if result.success {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    (status, Json(result))
}

/// 处理 ConfigUpdate 动作 — 更新 ConfigStore + 发确认消息 + 广播
///
/// 流程:
/// 1. 读取当前 GroupConfig
/// 2. 根据 ConfigUpdateAction 类型修改对应字段
/// 3. 签名并保存到 ConfigStore
/// 4. 发确认消息到群内
/// 5. 广播更新后的配置到所有节点
async fn handle_config_update(
    state: &Arc<AppState>,
    action: &crate::executor::ExecuteAction,
    cu: &crate::executor::ConfigUpdateAction,
) -> crate::executor::ExecuteResult {
    use crate::executor::ConfigUpdateAction;

    // 读取当前配置（没有则使用默认值）
    let current_version = state.config_store.get_version();
    let mut config = state.config_store.get()
        .map(|sc| sc.config)
        .unwrap_or_else(|| crate::group_config::GroupConfig {
            version: 0,
            bot_id_hash: state.config.bot_id_hash_hex(),
            ..serde_json::from_value(serde_json::json!({})).unwrap_or_else(|_| {
                // 手动构造最小默认配置
                serde_json::from_str("{}").unwrap()
            })
        });

    // 应用配置变更
    let confirm_text = match cu {
        ConfigUpdateAction::AddBlacklistWord => {
            let word = action.params.get("word")
                .and_then(|v| v.as_str()).unwrap_or("").to_string();
            if !word.is_empty() && !config.blacklist_words.contains(&word) {
                config.blacklist_words.push(word.clone());
            }
            format!("✅ Blacklist word added: `{}`\nTotal: {} words", word, config.blacklist_words.len())
        }
        ConfigUpdateAction::RemoveBlacklistWord => {
            let word = action.params.get("word")
                .and_then(|v| v.as_str()).unwrap_or("").to_string();
            config.blacklist_words.retain(|w| w != &word);
            format!("✅ Blacklist word removed: `{}`\nTotal: {} words", word, config.blacklist_words.len())
        }
        ConfigUpdateAction::LockType => {
            let lock_str = action.params.get("lock_type")
                .and_then(|v| v.as_str()).unwrap_or("");
            match parse_lock_type(lock_str) {
                Some(lt) => {
                    if !config.lock_types.contains(&lt) {
                        config.lock_types.push(lt.clone());
                    }
                    format!("🔒 Locked: {:?}", lt)
                }
                None => format!("⚠️ Unknown lock type: `{}`\nValid: audio, video, photo, document, sticker, gif, url, forward, voice, contact, location, poll, game, inline", lock_str),
            }
        }
        ConfigUpdateAction::UnlockType => {
            let lock_str = action.params.get("lock_type")
                .and_then(|v| v.as_str()).unwrap_or("");
            match parse_lock_type(lock_str) {
                Some(lt) => {
                    config.lock_types.retain(|t| t != &lt);
                    format!("🔓 Unlocked: {:?}", lt)
                }
                None => format!("⚠️ Unknown lock type: `{}`", lock_str),
            }
        }
        ConfigUpdateAction::SetWelcome => {
            let text = action.params.get("text")
                .and_then(|v| v.as_str()).unwrap_or("").to_string();
            if text.is_empty() {
                config.welcome_message.clear();
                "✅ Welcome message cleared.".to_string()
            } else {
                config.welcome_message = text.clone();
                format!("✅ Welcome message set:\n{}", text)
            }
        }
        ConfigUpdateAction::SetFloodLimit => {
            let limit = action.params.get("limit")
                .and_then(|v| v.as_u64()).unwrap_or(0) as u16;
            config.antiflood_limit = limit;
            if limit == 0 {
                "✅ Antiflood disabled.".to_string()
            } else {
                format!("✅ Antiflood set: {} msgs / {}s", limit, config.antiflood_window)
            }
        }
        ConfigUpdateAction::SetWarnLimit => {
            let limit = action.params.get("limit")
                .and_then(|v| v.as_u64()).unwrap_or(3) as u8;
            config.warn_limit = limit;
            format!("✅ Warn limit set to: {}", limit)
        }
        ConfigUpdateAction::SetWarnAction => {
            let action_str = action.params.get("action")
                .and_then(|v| v.as_str()).unwrap_or("ban");
            match action_str {
                "ban" => { config.warn_action = crate::group_config::WarnAction::Ban; }
                "kick" => { config.warn_action = crate::group_config::WarnAction::Kick; }
                "mute" => { config.warn_action = crate::group_config::WarnAction::Mute; }
                _ => {}
            }
            format!("✅ Warn action set to: {:?}", config.warn_action)
        }
    };

    // 版本自增 + 签名 + 保存
    config.version = current_version + 1;
    config.updated_at = chrono::Utc::now().timestamp() as u64;

    let config_json = serde_json::to_vec(&config).unwrap_or_default();
    let sig = state.key_manager.sign(&config_json);
    let signed_config = crate::group_config::SignedGroupConfig {
        config: config.clone(),
        signature: hex::encode(sig),
        signer_public_key: state.key_manager.public_key_hex(),
    };

    if let Err(e) = state.config_store.set(signed_config.clone(), current_version) {
        warn!(error = %e, "ConfigUpdate 保存失败");
        return crate::executor::ExecuteResult {
            action_id: action.action_id.clone(),
            success: false,
            error: Some(format!("Config save failed: {}", e)),
            api_method: None,
            api_response: None,
            agent_signature: None,
        };
    }

    info!(
        action_id = action.action_id,
        config_update = ?cu,
        new_version = config.version,
        "ConfigUpdate 已应用"
    );

    // 发确认消息到群内
    let _ = state.executor.execute(
        &crate::executor::ExecuteAction {
            action_id: format!("{}_confirm", action.action_id),
            action_type: crate::executor::ActionType::Message(crate::executor::MessageAction::Send),
            bot_id_hash: action.bot_id_hash.clone(),
            chat_id: action.chat_id,
            params: serde_json::json!({ "text": confirm_text }),
            leader_signature: String::new(),
            leader_node_id: String::new(),
            consensus_nodes: vec![],
            platform: action.platform.clone(),
        },
        &state.key_manager,
    ).await;

    // 异步广播配置到节点
    let state_clone = state.clone();
    let bot_id_hash = action.bot_id_hash.clone();
    tokio::spawn(async move {
        crate::group_config::broadcast_config_to_nodes(
            &state_clone, &bot_id_hash, signed_config,
        ).await;
    });

    crate::executor::ExecuteResult {
        action_id: action.action_id.clone(),
        success: true,
        error: None,
        api_method: Some("ConfigUpdate".into()),
        api_response: Some(serde_json::json!({
            "ok": true,
            "version": config.version,
        })),
        agent_signature: None,
    }
}

/// 解析锁定类型字符串为 LockType 枚举
fn parse_lock_type(s: &str) -> Option<crate::group_config::LockType> {
    use crate::group_config::LockType;
    match s {
        "audio" => Some(LockType::Audio),
        "video" => Some(LockType::Video),
        "photo" => Some(LockType::Photo),
        "document" | "doc" => Some(LockType::Document),
        "sticker" => Some(LockType::Sticker),
        "gif" | "animation" => Some(LockType::Gif),
        "url" | "link" => Some(LockType::Url),
        "forward" | "fwd" => Some(LockType::Forward),
        "voice" => Some(LockType::Voice),
        "contact" => Some(LockType::Contact),
        "location" => Some(LockType::Location),
        "poll" => Some(LockType::Poll),
        "game" => Some(LockType::Game),
        "inline" => Some(LockType::Inline),
        _ => None,
    }
}

/// 判断是否为 warn 相关动作（通过 reason 字段识别）
fn is_warn_action(action: &crate::executor::ExecuteAction) -> bool {
    action.params.get("warn_action").is_some()
}

/// 处理 warn 相关动作 — LocalStore 计数 + 自动升级
///
/// 流程:
/// - warn_action = "add": 增加计数，超限 → 执行 warn_action (ban/kick/mute)
/// - warn_action = "remove": 减少计数
/// - warn_action = "query": 查询计数并回复
/// - warn_action = "reset": 重置计数
async fn handle_warn_action(
    state: &Arc<AppState>,
    action: &crate::executor::ExecuteAction,
) -> crate::executor::ExecuteResult {
    let warn_op = action.params.get("warn_action")
        .and_then(|v| v.as_str()).unwrap_or("");
    let user_id = action.params.get("user_id")
        .and_then(|v| v.as_i64()).unwrap_or(0);
    let chat_id = action.chat_id;

    // 获取 warn 配置
    let (warn_limit, warn_action_cfg) = state.config_store.get()
        .map(|sc| (sc.config.warn_limit, sc.config.warn_action.clone()))
        .unwrap_or((3, crate::group_config::WarnAction::Ban));

    let confirm_text = match warn_op {
        "add" => {
            let reason = action.params.get("reason")
                .and_then(|v| v.as_str()).unwrap_or("");
            let count = state.local_store.add_warn(chat_id, user_id);

            if warn_limit > 0 && count >= warn_limit {
                // 自动升级: 超限 → 执行配置的动作
                let escalation_action = match &warn_action_cfg {
                    crate::group_config::WarnAction::Ban => {
                        crate::executor::ActionType::Admin(crate::executor::AdminAction::Ban)
                    }
                    crate::group_config::WarnAction::Kick => {
                        crate::executor::ActionType::Admin(crate::executor::AdminAction::Kick)
                    }
                    crate::group_config::WarnAction::Mute => {
                        crate::executor::ActionType::Admin(crate::executor::AdminAction::Mute)
                    }
                };

                info!(
                    chat_id, user_id, count, warn_limit,
                    escalation = ?warn_action_cfg,
                    "Warn 自动升级"
                );

                // 重置计数
                state.local_store.reset_warns(chat_id, user_id);

                // 执行升级动作
                let escalation = crate::executor::ExecuteAction {
                    action_id: format!("{}_escalation", action.action_id),
                    action_type: escalation_action,
                    bot_id_hash: action.bot_id_hash.clone(),
                    platform: action.platform.clone(),
                    chat_id,
                    params: serde_json::json!({
                        "user_id": user_id,
                        "duration_seconds": 3600,
                    }),
                    leader_signature: String::new(),
                    leader_node_id: String::new(),
                    consensus_nodes: vec![],
                };
                let _ = state.executor.execute(&escalation, &state.key_manager).await;

                format!(
                    "⚠️ User {} has {}/{} warnings → auto {:?}!",
                    user_id, count, warn_limit, warn_action_cfg
                )
            } else {
                let reason_text = if reason.is_empty() {
                    String::new()
                } else {
                    format!("\nReason: {}", reason)
                };
                format!(
                    "⚠️ User {} warned ({}/{}){}", user_id, count, warn_limit, reason_text
                )
            }
        }
        "remove" => {
            let count = state.local_store.remove_warn(chat_id, user_id);
            format!("✅ Warning removed. User {} now has {}/{} warnings.", user_id, count, warn_limit)
        }
        "query" => {
            let count = state.local_store.get_warns(chat_id, user_id);
            format!("ℹ️ User {} has {}/{} warnings.", user_id, count, warn_limit)
        }
        "reset" => {
            state.local_store.reset_warns(chat_id, user_id);
            format!("✅ Warnings reset for user {}.", user_id)
        }
        _ => {
            return crate::executor::ExecuteResult {
                action_id: action.action_id.clone(),
                success: false,
                error: Some(format!("Unknown warn_action: {}", warn_op)),
                api_method: None,
                api_response: None,
                agent_signature: None,
            };
        }
    };

    // 发确认消息到群内
    let send_action = crate::executor::ExecuteAction {
        action_id: format!("{}_reply", action.action_id),
        action_type: crate::executor::ActionType::Message(crate::executor::MessageAction::Send),
        bot_id_hash: action.bot_id_hash.clone(),
        platform: action.platform.clone(),
        chat_id,
        params: serde_json::json!({ "text": confirm_text }),
        leader_signature: String::new(),
        leader_node_id: String::new(),
        consensus_nodes: vec![],
    };
    let _ = state.executor.execute(&send_action, &state.key_manager).await;

    crate::executor::ExecuteResult {
        action_id: action.action_id.clone(),
        success: true,
        error: None,
        api_method: Some("WarnAction".into()),
        api_response: Some(serde_json::json!({ "ok": true })),
        agent_signature: None,
    }
}

/// 验证 Leader 执行指令的合法性
///
/// 检查:
/// 1. bot_id_hash 与本 Agent 匹配
/// 2. consensus_nodes 数量 >= M (ceil(K * 2/3))
/// 3. leader_node_id 在 consensus_nodes 中
/// 4. Leader 签名验证（Ed25519）
async fn verify_leader_action(
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

    // 2. consensus 数量检查 — M 基于活跃节点总数 N 计算
    let consensus_count = action.consensus_nodes.len();
    if consensus_count == 0 {
        return Err("consensus_nodes 为空".into());
    }
    let total_nodes = state.nodes.read().await.len();
    let k = if total_nodes <= 3 { total_nodes } else { (total_nodes * 2 + 2) / 3 };
    let m = if k <= 3 { k } else { (k * 2 + 2) / 3 };
    let m = m.max(1);
    if consensus_count < m {
        return Err(format!(
            "共识节点不足: {} < M({}), 活跃节点总数 N={}, K={}",
            consensus_count, m, total_nodes, k
        ));
    }

    // 3. Leader 在共识列表中
    if !action.consensus_nodes.contains(&action.leader_node_id) {
        return Err(format!(
            "Leader {} 不在共识节点列表中",
            action.leader_node_id
        ));
    }

    // 4. Leader 签名验证（必须提供）
    if action.leader_signature.is_empty() {
        return Err("leader_signature 不能为空".into());
    }

    {
        use ed25519_dalek::{VerifyingKey, Verifier, Signature};
        use sha2::{Sha256, Digest};

        // 签名格式: "pubkey_hex:signature_hex"
        let (pk_hex, sig_hex) = action.leader_signature.split_once(':')
            .ok_or_else(|| "leader_signature 格式错误，期望 pubkey_hex:signature_hex".to_string())?;

        let pk_bytes = hex::decode(pk_hex).map_err(|e| format!("Leader 公钥 hex 无效: {}", e))?;
        let sig_bytes = hex::decode(sig_hex).map_err(|e| format!("Leader 签名 hex 无效: {}", e))?;

        if pk_bytes.len() != 32 || sig_bytes.len() != 64 {
            return Err("Leader 公钥/签名长度错误".into());
        }

        // C3 修复: 验证 Leader 公钥与 nodes 列表中注册的公钥匹配
        let nodes = state.nodes.read().await;
        let leader_node = nodes.iter()
            .find(|n| n.node_id == action.leader_node_id);
        if let Some(node) = leader_node {
            if !node.node_public_key.is_empty() && node.node_public_key != pk_hex {
                return Err(format!(
                    "Leader 公钥不匹配: 节点注册 {}, 签名提供 {}",
                    node.node_public_key, pk_hex
                ));
            }
        }
        drop(nodes);

        let mut pk_arr = [0u8; 32];
        pk_arr.copy_from_slice(&pk_bytes);
        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(&sig_bytes);

        let vk = VerifyingKey::from_bytes(&pk_arr)
            .map_err(|e| format!("Leader 公钥无效: {}", e))?;
        let sig = Signature::from_bytes(&sig_arr);

        // 签名数据 = SHA256(action_id + bot_id_hash + action_type_str + chat_id_le)
        let action_type_str = format!("{:?}", action.action_type);
        let mut hasher = Sha256::new();
        hasher.update(action.action_id.as_bytes());
        hasher.update(action.bot_id_hash.as_bytes());
        hasher.update(action_type_str.as_bytes());
        hasher.update(&action.chat_id.to_le_bytes());
        let sign_data = hasher.finalize();

        vk.verify(&sign_data, &sig)
            .map_err(|e| format!("Leader 签名验证失败: {}", e))?;

        debug!(leader = action.leader_node_id, "Leader 签名验证通过");
    }

    Ok(())
}

/// 检查 TG API 响应体中的 ok 字段，记录业务层错误
async fn check_tg_response(resp: reqwest::Response, method: &str, chat_id: i64) {
    if let Ok(body) = resp.json::<serde_json::Value>().await {
        let ok = body.get("ok").and_then(|v| v.as_bool()).unwrap_or(true);
        if !ok {
            let desc = body.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            warn!(method, chat_id, desc, "TG API 业务错误");
        }
    }
}

/// 执行本地快速路径动作（直接调用 TG API，不走共识）
///
/// 支持的动作:
///   - DeleteMessage → deleteMessage
///   - MuteUser → restrictChatMember (can_send_messages: false)
///   - BanUser → banChatMember
///   - KickUser → banChatMember + unbanChatMember
///   - SendMessage → sendMessage
async fn execute_local_action(
    state: &Arc<AppState>,
    action: &crate::local_processor::LocalAction,
) -> Result<(), String> {
    use crate::local_processor::LocalActionType;

    let bot_token = &state.config.bot_token;
    let base = format!("https://api.telegram.org/bot{}", bot_token);

    match &action.action {
        LocalActionType::DeleteMessage => {
            let msg_id = action.params.get("message_id")
                .and_then(|v| v.as_i64()).unwrap_or(0);
            if msg_id == 0 { return Ok(()); }

            let url = format!("{}/deleteMessage", base);
            let resp = state.http_client.post(&url)
                .json(&serde_json::json!({
                    "chat_id": action.chat_id,
                    "message_id": msg_id,
                }))
                .send().await
                .map_err(|e| format!("deleteMessage failed: {}", e))?;
            check_tg_response(resp, "deleteMessage", action.chat_id).await;

            debug!(chat_id = action.chat_id, msg_id, reason = action.reason, "本地删除消息");
        }
        LocalActionType::MuteUser => {
            let user_id = action.params.get("user_id")
                .and_then(|v| v.as_i64()).unwrap_or(0);
            let duration = action.params.get("duration_seconds")
                .and_then(|v| v.as_u64()).unwrap_or(300);
            if user_id == 0 { return Ok(()); }

            let until_date = chrono::Utc::now().timestamp() + duration as i64;
            let url = format!("{}/restrictChatMember", base);
            let resp = state.http_client.post(&url)
                .json(&serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                    "permissions": { "can_send_messages": false },
                    "until_date": until_date,
                }))
                .send().await
                .map_err(|e| format!("restrictChatMember failed: {}", e))?;
            check_tg_response(resp, "restrictChatMember", action.chat_id).await;

            // 如果也需要删除消息
            if let Some(msg_id) = action.params.get("message_id").and_then(|v| v.as_i64()) {
                let del_url = format!("{}/deleteMessage", base);
                let _ = state.http_client.post(&del_url)
                    .json(&serde_json::json!({
                        "chat_id": action.chat_id,
                        "message_id": msg_id,
                    }))
                    .send().await;
            }

            debug!(chat_id = action.chat_id, user_id, duration, reason = action.reason, "本地禁言");
        }
        LocalActionType::BanUser => {
            let user_id = action.params.get("user_id")
                .and_then(|v| v.as_i64()).unwrap_or(0);
            if user_id == 0 { return Ok(()); }

            let url = format!("{}/banChatMember", base);
            let resp = state.http_client.post(&url)
                .json(&serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                }))
                .send().await
                .map_err(|e| format!("banChatMember failed: {}", e))?;
            check_tg_response(resp, "banChatMember", action.chat_id).await;

            // 如果也需要删除消息
            if let Some(msg_id) = action.params.get("message_id").and_then(|v| v.as_i64()) {
                let del_url = format!("{}/deleteMessage", base);
                let _ = state.http_client.post(&del_url)
                    .json(&serde_json::json!({
                        "chat_id": action.chat_id,
                        "message_id": msg_id,
                    }))
                    .send().await;
            }

            debug!(chat_id = action.chat_id, user_id, reason = action.reason, "本地封禁");
        }
        LocalActionType::KickUser => {
            let user_id = action.params.get("user_id")
                .and_then(|v| v.as_i64()).unwrap_or(0);
            if user_id == 0 { return Ok(()); }

            // ban then immediately unban = kick
            let ban_url = format!("{}/banChatMember", base);
            let _ = state.http_client.post(&ban_url)
                .json(&serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                }))
                .send().await
                .map_err(|e| format!("banChatMember(kick) failed: {}", e))?;

            let unban_url = format!("{}/unbanChatMember", base);
            let _ = state.http_client.post(&unban_url)
                .json(&serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                    "only_if_banned": true,
                }))
                .send().await;

            debug!(chat_id = action.chat_id, user_id, reason = action.reason, "本地踢出");
        }
        LocalActionType::SendMessage => {
            let text = action.params.get("text")
                .and_then(|v| v.as_str()).unwrap_or("");
            if text.is_empty() { return Ok(()); }

            let url = format!("{}/sendMessage", base);
            let resp = state.http_client.post(&url)
                .json(&serde_json::json!({
                    "chat_id": action.chat_id,
                    "text": text,
                }))
                .send().await
                .map_err(|e| format!("sendMessage failed: {}", e))?;
            check_tg_response(resp, "sendMessage", action.chat_id).await;

            debug!(chat_id = action.chat_id, reason = action.reason, "本地发送消息");
        }
    }

    Ok(())
}

/// B3.3: 从 Telegram API 拉取管理员列表并缓存到 LocalStore
///
/// 调用 getChatAdministrators → 解析 admin user_id 列表 → 写入 LocalStore
///
/// 参考: FallenRobot/modules/helper_funcs/chat_status.py — TTLCache(maxsize=512, ttl=300)
async fn refresh_admin_cache(
    state: &Arc<AppState>,
    chat_id: i64,
) -> Result<(), String> {
    let bot_token = &state.config.bot_token;
    let url = format!(
        "https://api.telegram.org/bot{}/getChatAdministrators",
        bot_token
    );

    let resp = state.http_client
        .post(&url)
        .json(&serde_json::json!({ "chat_id": chat_id }))
        .send()
        .await
        .map_err(|e| format!("getChatAdministrators request failed: {}", e))?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("getChatAdministrators parse failed: {}", e))?;

    if !body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        let desc = body.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        return Err(format!("getChatAdministrators error: {}", desc));
    }

    let result = body.get("result").and_then(|v| v.as_array());
    let admin_ids: Vec<i64> = result
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.pointer("/user/id").and_then(|v| v.as_i64()))
                .collect()
        })
        .unwrap_or_default();

    let count = admin_ids.len();
    state.local_store.set_admin_cache(chat_id, admin_ids);

    debug!(chat_id, count, "管理员缓存已刷新");
    Ok(())
}

/// B3.5: 审计日志 — 将本地快速路径执行的动作异步提交到 Node 网络
///
/// 格式: 以 SignedMessage 方式发送审计记录到节点，
/// 节点可记录到本地审计日志或提交到链上。
///
/// 当前实现: 通过已有的多播通道发送审计事件。
/// 后续: 可批量合并，减少网络开销。
async fn submit_audit_log(
    state: &Arc<AppState>,
    actions: &[crate::local_processor::LocalAction],
) {
    if actions.is_empty() {
        return;
    }

    // 构造审计事件 JSON
    let audit_entries: Vec<serde_json::Value> = actions.iter().map(|a| {
        serde_json::json!({
            "action": format!("{:?}", a.action),
            "chat_id": a.chat_id,
            "reason": a.reason,
            "timestamp": chrono::Utc::now().timestamp(),
        })
    }).collect();

    let audit_update = serde_json::json!({
        "audit_log": {
            "source": "local_processor",
            "bot_id_hash": state.config.bot_id_hash_hex(),
            "entries": audit_entries,
            "timestamp": chrono::Utc::now().timestamp(),
        }
    });

    // 签名审计记录
    let raw_json = serde_json::to_vec(&audit_update).unwrap_or_default();
    let timestamp = chrono::Utc::now().timestamp() as u64;

    // 使用独立的审计序列号，不消耗主消息序列号
    let sequence = state.sequence_manager.next_audit();

    let (signature, message_hash) = state.key_manager.sign_message(
        &state.config.bot_id_hash,
        sequence,
        timestamp,
        &raw_json,
    );

    let signed_message = crate::types::SignedMessage {
        owner_public_key: state.key_manager.public_key_hex(),
        bot_id_hash: state.config.bot_id_hash_hex(),
        sequence,
        timestamp,
        message_hash: hex::encode(message_hash),
        platform_event: audit_update,
        owner_signature: hex::encode(signature),
        platform: "audit".to_string(),
    };

    // 异步多播（不阻塞，失败静默）
    match crate::multicaster::multicast_to_nodes(state, &signed_message).await {
        Ok(r) => {
            debug!(
                entries = actions.len(),
                success = r.success_count,
                "审计日志已提交"
            );
        }
        Err(e) => {
            debug!(error = %e, "审计日志提交失败（非关键）");
        }
    }
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
