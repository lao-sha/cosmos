use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};

/// Leader 节点发来的执行指令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteAction {
    pub action_id: String,
    pub action_type: ActionType,
    pub bot_id_hash: String,
    pub chat_id: i64,
    pub params: serde_json::Value,
    pub leader_signature: String,
    pub leader_node_id: String,
    pub consensus_nodes: Vec<String>,
}

/// 动作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    SendMessage,
    DeleteMessage,
    BanUser,
    MuteUser,
    UnmuteUser,
    PinMessage,
    UnpinMessage,
    ApproveChatJoinRequest,
    DeclineChatJoinRequest,
    NoAction,
}

/// 执行结果（含密码学回执）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResult {
    pub action_id: String,
    pub success: bool,
    pub error: Option<String>,
    /// 实际调用的 TG API 方法
    pub tg_api_method: Option<String>,
    /// Telegram API 原始响应 JSON
    pub tg_api_response: Option<serde_json::Value>,
    /// Agent Ed25519 签名回执: sign(action_id + method + response_hash)
    pub agent_signature: Option<String>,
}

/// TG API 执行器
///
/// 接收 Leader 指令 → 调用 Telegram Bot API → 返回结果
pub struct TelegramExecutor {
    bot_token: String,
    client: reqwest::Client,
}

impl TelegramExecutor {
    pub fn new(bot_token: String) -> Self {
        Self {
            bot_token,
            client: reqwest::Client::new(),
        }
    }

    /// 执行动作（含签名回执）
    pub async fn execute(&self, action: &ExecuteAction, key_manager: &crate::signer::KeyManager) -> ExecuteResult {
        let result = match &action.action_type {
            ActionType::SendMessage => {
                let text = action.params.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(empty)");
                self.call_tg_api("sendMessage", serde_json::json!({
                    "chat_id": action.chat_id,
                    "text": text,
                })).await
            }
            ActionType::DeleteMessage => {
                let message_id = action.params.get("message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("deleteMessage", serde_json::json!({
                    "chat_id": action.chat_id,
                    "message_id": message_id,
                })).await
            }
            ActionType::BanUser => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("banChatMember", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                })).await
            }
            ActionType::MuteUser => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let duration = action.params.get("duration_seconds")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3600);
                let until = chrono::Utc::now().timestamp() + duration as i64;
                self.call_tg_api("restrictChatMember", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                    "permissions": {
                        "can_send_messages": false,
                        "can_send_media_messages": false,
                        "can_send_other_messages": false,
                    },
                    "until_date": until,
                })).await
            }
            ActionType::UnmuteUser => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("restrictChatMember", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                    "permissions": {
                        "can_send_messages": true,
                        "can_send_media_messages": true,
                        "can_send_other_messages": true,
                        "can_add_web_page_previews": true,
                    },
                })).await
            }
            ActionType::PinMessage => {
                let message_id = action.params.get("message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("pinChatMessage", serde_json::json!({
                    "chat_id": action.chat_id,
                    "message_id": message_id,
                })).await
            }
            ActionType::UnpinMessage => {
                let message_id = action.params.get("message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("unpinChatMessage", serde_json::json!({
                    "chat_id": action.chat_id,
                    "message_id": message_id,
                })).await
            }
            ActionType::ApproveChatJoinRequest => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("approveChatJoinRequest", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                })).await
            }
            ActionType::DeclineChatJoinRequest => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("declineChatJoinRequest", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                })).await
            }
            ActionType::NoAction => {
                Ok(("NoAction".to_string(), serde_json::json!({"ok": true})))
            }
        };

        match result {
            Ok((method, tg_response)) => {
                info!(action_id = action.action_id, action_type = ?action.action_type, "TG API 执行成功");

                // V2: 签名回执 = sign(action_id + method + SHA256(tg_response))
                let sig = Self::sign_receipt(key_manager, &action.action_id, &method, &tg_response);

                ExecuteResult {
                    action_id: action.action_id.clone(),
                    success: true,
                    error: None,
                    tg_api_method: Some(method),
                    tg_api_response: Some(tg_response),
                    agent_signature: Some(sig),
                }
            }
            Err(e) => {
                warn!(action_id = action.action_id, error = %e, "TG API 执行失败");
                ExecuteResult {
                    action_id: action.action_id.clone(),
                    success: false,
                    error: Some(e.to_string()),
                    tg_api_method: None,
                    tg_api_response: None,
                    agent_signature: None,
                }
            }
        }
    }

    /// 调用 Telegram Bot API — 返回 (method, response_json)
    async fn call_tg_api(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<(String, serde_json::Value)> {
        let url = format!(
            "https://api.telegram.org/bot{}/{}",
            self.bot_token, method
        );

        debug!(method, "调用 TG API");

        let resp = self.client
            .post(&url)
            .json(&params)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        let body: serde_json::Value = resp.json().await?;

        if body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
            Ok((method.to_string(), body))
        } else {
            let desc = body.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            anyhow::bail!("TG API {}: {}", method, desc)
        }
    }

    /// 生成签名回执
    ///
    /// receipt_data = action_id + method + SHA256(tg_response_json)
    /// signature = Ed25519_sign(receipt_data)
    fn sign_receipt(
        key_manager: &crate::signer::KeyManager,
        action_id: &str,
        method: &str,
        tg_response: &serde_json::Value,
    ) -> String {
        use sha2::{Sha256, Digest};

        let response_bytes = serde_json::to_vec(tg_response).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(&response_bytes);
        let response_hash = hasher.finalize();

        let mut receipt_data = Vec::new();
        receipt_data.extend_from_slice(action_id.as_bytes());
        receipt_data.extend_from_slice(method.as_bytes());
        receipt_data.extend_from_slice(&response_hash);

        let sig = key_manager.sign(&receipt_data);
        format!("{}:{}", key_manager.public_key_hex(), hex::encode(sig))
    }
}
