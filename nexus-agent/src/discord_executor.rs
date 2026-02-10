use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::{Client, Method, StatusCode};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::executor::{
    ActionType, AdminAction, ConfigUpdateAction, ExecuteAction, ExecuteResult,
    MessageAction, PlatformExecutor, QueryAction,
};

/// Discord REST API 基地址
const DISCORD_API_BASE: &str = "https://discord.com/api/v10";

/// Discord 权限位
pub mod permissions {
    pub const KICK_MEMBERS: u64    = 1 << 1;
    pub const BAN_MEMBERS: u64     = 1 << 2;
    pub const ADMINISTRATOR: u64   = 1 << 3;
    pub const MANAGE_CHANNELS: u64 = 1 << 4;
    pub const MANAGE_GUILD: u64    = 1 << 5;
    pub const SEND_MESSAGES: u64   = 1 << 11;
    pub const MANAGE_MESSAGES: u64 = 1 << 13;
    pub const MANAGE_ROLES: u64    = 1 << 28;
    pub const MODERATE_MEMBERS: u64 = 1 << 40;
}

/// Rate limit bucket 状态
#[derive(Debug, Clone)]
struct RateLimitBucket {
    remaining: u32,
    reset_at: Instant,
}

/// Discord REST API 执行器
///
/// 实现 PlatformExecutor trait，将 ExecuteAction 映射到 Discord REST API 调用。
/// 包含 per-route rate limit tracking。
pub struct DiscordExecutor {
    bot_token: String,
    application_id: String,
    client: Client,
    rate_limits: Arc<RwLock<HashMap<String, RateLimitBucket>>>,
}

impl DiscordExecutor {
    pub fn new(bot_token: String, application_id: String, client: Client) -> Self {
        Self {
            bot_token,
            application_id,
            client,
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 调用 Discord REST API（含 rate limit 处理）
    async fn call_discord_api(
        &self,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<(String, serde_json::Value), String> {
        let route_key = format!("{}:{}", method, path.split('?').next().unwrap_or(path));

        // 检查 rate limit
        {
            let limits = self.rate_limits.read().await;
            if let Some(bucket) = limits.get(&route_key) {
                if bucket.remaining == 0 && bucket.reset_at > Instant::now() {
                    let wait = bucket.reset_at - Instant::now();
                    debug!(route = %route_key, wait_ms = wait.as_millis(), "Rate limit, 等待...");
                    tokio::time::sleep(wait).await;
                }
            }
        }

        let url = format!("{}{}", DISCORD_API_BASE, path);
        let mut request = self.client
            .request(method.clone(), &url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .header("User-Agent", "DiscordBot (nexus-agent, 0.1.0)");

        if let Some(ref b) = body {
            request = request.json(b);
        }

        let response = request.send().await
            .map_err(|e| format!("Discord API 请求失败: {}", e))?;

        // 更新 rate limit
        let status = response.status();
        if let Some(remaining) = response.headers().get("x-ratelimit-remaining") {
            if let Ok(r) = remaining.to_str().unwrap_or("0").parse::<u32>() {
                let reset_after = response.headers()
                    .get("x-ratelimit-reset-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(1.0);

                let mut limits = self.rate_limits.write().await;
                limits.insert(route_key.clone(), RateLimitBucket {
                    remaining: r,
                    reset_at: Instant::now() + Duration::from_secs_f64(reset_after),
                });
            }
        }

        // 429 Too Many Requests → 等待 retry_after 后重试一次
        if status == StatusCode::TOO_MANY_REQUESTS {
            let body_text = response.text().await.unwrap_or_default();
            let retry_after = serde_json::from_str::<serde_json::Value>(&body_text)
                .ok()
                .and_then(|v| v.get("retry_after")?.as_f64())
                .unwrap_or(1.0);

            warn!(retry_after_s = retry_after, route = %route_key, "Discord 429 rate limit");
            tokio::time::sleep(Duration::from_secs_f64(retry_after)).await;

            // 重试一次
            let mut retry_req = self.client
                .request(method, &url)
                .header("Authorization", format!("Bot {}", self.bot_token))
                .header("Content-Type", "application/json")
                .header("User-Agent", "DiscordBot (nexus-agent, 0.1.0)");
            if let Some(ref b) = body {
                retry_req = retry_req.json(b);
            }

            let retry_resp = retry_req.send().await
                .map_err(|e| format!("Discord API 重试失败: {}", e))?;
            let retry_status = retry_resp.status();
            let retry_body: serde_json::Value = retry_resp.json().await
                .unwrap_or(json!({"error": "retry parse failed"}));

            if retry_status.is_success() {
                return Ok((route_key, retry_body));
            }
            return Err(format!("Discord API 重试仍失败: {} {}", retry_status, retry_body));
        }

        if !status.is_success() && status != StatusCode::NO_CONTENT {
            let error_body: serde_json::Value = response.json().await
                .unwrap_or(json!({"error": "unknown"}));
            return Err(format!("Discord API 错误 {}: {}", status, error_body));
        }

        // 204 No Content
        if status == StatusCode::NO_CONTENT {
            return Ok((route_key, json!({"ok": true})));
        }

        let resp_body: serde_json::Value = response.json().await
            .unwrap_or(json!({"ok": true}));
        Ok((route_key, resp_body))
    }

    /// 签名回执（复用 TelegramExecutor 相同逻辑）
    fn sign_receipt(
        key_manager: &crate::signer::KeyManager,
        action_id: &str,
        method: &str,
        response: &serde_json::Value,
    ) -> String {
        use sha2::{Sha256, Digest};

        let response_bytes = serde_json::to_vec(response).unwrap_or_default();
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

    // ═══════════════════════════════════════════════════════════════
    // Discord REST API 方法映射
    // ═══════════════════════════════════════════════════════════════

    /// POST /channels/{channel_id}/messages
    async fn send_message(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let channel_id = action.params.get("channel_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 channel_id")?;
        let text = action.params.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let mut body = json!({"content": text});

        // 支持 embed
        if let Some(embed) = action.params.get("embed") {
            body["embeds"] = json!([embed]);
        }

        self.call_discord_api(
            Method::POST,
            &format!("/channels/{}/messages", channel_id),
            Some(body),
        ).await
    }

    /// DELETE /channels/{channel_id}/messages/{message_id}
    async fn delete_message(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let channel_id = action.params.get("channel_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 channel_id")?;
        let message_id = action.params.get("message_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 message_id")?;

        self.call_discord_api(
            Method::DELETE,
            &format!("/channels/{}/messages/{}", channel_id, message_id),
            None,
        ).await
    }

    /// POST /channels/{channel_id}/messages/bulk-delete
    async fn delete_messages_bulk(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let channel_id = action.params.get("channel_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 channel_id")?;
        let message_ids = action.params.get("message_ids")
            .ok_or("缺少 message_ids")?;

        self.call_discord_api(
            Method::POST,
            &format!("/channels/{}/messages/bulk-delete", channel_id),
            Some(json!({"messages": message_ids})),
        ).await
    }

    /// PUT /channels/{channel_id}/pins/{message_id}
    async fn pin_message(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let channel_id = action.params.get("channel_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 channel_id")?;
        let message_id = action.params.get("message_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 message_id")?;

        self.call_discord_api(
            Method::PUT,
            &format!("/channels/{}/pins/{}", channel_id, message_id),
            None,
        ).await
    }

    /// DELETE /channels/{channel_id}/pins/{message_id}
    async fn unpin_message(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let channel_id = action.params.get("channel_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 channel_id")?;
        let message_id = action.params.get("message_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 message_id")?;

        self.call_discord_api(
            Method::DELETE,
            &format!("/channels/{}/pins/{}", channel_id, message_id),
            None,
        ).await
    }

    /// PUT /guilds/{guild_id}/bans/{user_id}
    async fn ban_user(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let guild_id = self.guild_id_str(action)?;
        let user_id = action.params.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 user_id")?;

        let mut body = json!({});
        if let Some(days) = action.params.get("delete_message_days").and_then(|v| v.as_u64()) {
            body["delete_message_seconds"] = json!(days * 86400);
        }
        if let Some(reason) = action.params.get("reason").and_then(|v| v.as_str()) {
            body["reason"] = json!(reason);
        }

        self.call_discord_api(
            Method::PUT,
            &format!("/guilds/{}/bans/{}", guild_id, user_id),
            Some(body),
        ).await
    }

    /// DELETE /guilds/{guild_id}/bans/{user_id}
    async fn unban_user(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let guild_id = self.guild_id_str(action)?;
        let user_id = action.params.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 user_id")?;

        self.call_discord_api(
            Method::DELETE,
            &format!("/guilds/{}/bans/{}", guild_id, user_id),
            None,
        ).await
    }

    /// PATCH /guilds/{guild_id}/members/{user_id} — communication_disabled_until
    async fn mute_user(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let guild_id = self.guild_id_str(action)?;
        let user_id = action.params.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 user_id")?;
        let duration_seconds = action.params.get("duration_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(3600);

        let timeout_until = chrono::Utc::now()
            + chrono::Duration::seconds(duration_seconds as i64);

        self.call_discord_api(
            Method::PATCH,
            &format!("/guilds/{}/members/{}", guild_id, user_id),
            Some(json!({
                "communication_disabled_until": timeout_until.to_rfc3339()
            })),
        ).await
    }

    /// PATCH /guilds/{guild_id}/members/{user_id} — remove timeout
    async fn unmute_user(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let guild_id = self.guild_id_str(action)?;
        let user_id = action.params.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 user_id")?;

        self.call_discord_api(
            Method::PATCH,
            &format!("/guilds/{}/members/{}", guild_id, user_id),
            Some(json!({"communication_disabled_until": null})),
        ).await
    }

    /// DELETE /guilds/{guild_id}/members/{user_id} — kick
    async fn kick_user(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let guild_id = self.guild_id_str(action)?;
        let user_id = action.params.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 user_id")?;

        self.call_discord_api(
            Method::DELETE,
            &format!("/guilds/{}/members/{}", guild_id, user_id),
            None,
        ).await
    }

    /// PUT /guilds/{guild_id}/members/{user_id}/roles/{role_id}
    async fn add_role(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let guild_id = self.guild_id_str(action)?;
        let user_id = action.params.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 user_id")?;
        let role_id = action.params.get("role_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 role_id")?;

        self.call_discord_api(
            Method::PUT,
            &format!("/guilds/{}/members/{}/roles/{}", guild_id, user_id, role_id),
            None,
        ).await
    }

    /// DELETE /guilds/{guild_id}/members/{user_id}/roles/{role_id}
    async fn remove_role(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let guild_id = self.guild_id_str(action)?;
        let user_id = action.params.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 user_id")?;
        let role_id = action.params.get("role_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 role_id")?;

        self.call_discord_api(
            Method::DELETE,
            &format!("/guilds/{}/members/{}/roles/{}", guild_id, user_id, role_id),
            None,
        ).await
    }

    /// PATCH /channels/{channel_id}/messages/{message_id}
    async fn edit_message(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let channel_id = action.params.get("channel_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 channel_id")?;
        let message_id = action.params.get("message_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 message_id")?;

        let mut body = json!({});
        if let Some(text) = action.params.get("text").and_then(|v| v.as_str()) {
            body["content"] = json!(text);
        }
        if let Some(embed) = action.params.get("embed") {
            body["embeds"] = json!([embed]);
        }

        self.call_discord_api(
            Method::PATCH,
            &format!("/channels/{}/messages/{}", channel_id, message_id),
            Some(body),
        ).await
    }

    /// GET /guilds/{guild_id}/members/{user_id}
    async fn get_member(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let guild_id = self.guild_id_str(action)?;
        let user_id = action.params.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 user_id")?;

        self.call_discord_api(
            Method::GET,
            &format!("/guilds/{}/members/{}", guild_id, user_id),
            None,
        ).await
    }

    /// GET /guilds/{guild_id}
    async fn get_guild(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let guild_id = self.guild_id_str(action)?;

        self.call_discord_api(
            Method::GET,
            &format!("/guilds/{}", guild_id),
            None,
        ).await
    }

    /// POST /interactions/{id}/{token}/callback — ACK interaction
    pub async fn ack_interaction_api(
        &self,
        interaction_id: &str,
        interaction_token: &str,
    ) -> Result<(), String> {
        self.call_discord_api(
            Method::POST,
            &format!("/interactions/{}/{}/callback", interaction_id, interaction_token),
            Some(json!({"type": 5})), // Deferred Channel Message with Source
        ).await?;
        Ok(())
    }

    /// POST /webhooks/{app_id}/{token} — followup message
    pub async fn followup_interaction(
        &self,
        interaction_token: &str,
        content: &str,
    ) -> Result<(String, serde_json::Value), String> {
        self.call_discord_api(
            Method::POST,
            &format!("/webhooks/{}/{}", self.application_id, interaction_token),
            Some(json!({"content": content})),
        ).await
    }

    /// PUT /applications/{app_id}/guilds/{guild_id}/commands — 注册 Slash Commands
    pub async fn register_guild_commands(
        &self,
        guild_id: &str,
        commands: serde_json::Value,
    ) -> Result<(String, serde_json::Value), String> {
        self.call_discord_api(
            Method::PUT,
            &format!("/applications/{}/guilds/{}/commands", self.application_id, guild_id),
            Some(commands),
        ).await
    }

    /// PUT /channels/{channel_id}/permissions/{overwrite_id}
    async fn set_channel_permissions(&self, action: &ExecuteAction) -> Result<(String, serde_json::Value), String> {
        let channel_id = action.params.get("channel_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 channel_id")?;
        let overwrite_id = action.params.get("overwrite_id")
            .and_then(|v| v.as_str())
            .ok_or("缺少 overwrite_id")?;
        let allow = action.params.get("allow").and_then(|v| v.as_str()).unwrap_or("0");
        let deny = action.params.get("deny").and_then(|v| v.as_str()).unwrap_or("0");
        // type 0 = role, 1 = member
        let overwrite_type = action.params.get("type").and_then(|v| v.as_u64()).unwrap_or(0);

        self.call_discord_api(
            Method::PUT,
            &format!("/channels/{}/permissions/{}", channel_id, overwrite_id),
            Some(json!({
                "allow": allow,
                "deny": deny,
                "type": overwrite_type
            })),
        ).await
    }

    /// 辅助: 从 action 中提取 guild_id
    fn guild_id_str(&self, action: &ExecuteAction) -> Result<String, String> {
        // 优先用 params.guild_id, 否则 chat_id 转字符串
        action.params.get("guild_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                if action.chat_id != 0 {
                    Some(action.chat_id.to_string())
                } else {
                    None
                }
            })
            .ok_or_else(|| "缺少 guild_id".to_string())
    }
}

impl PlatformExecutor for DiscordExecutor {
    async fn execute(&self, action: &ExecuteAction, key_manager: &crate::signer::KeyManager) -> ExecuteResult {
        let result = match &action.action_type {
            ActionType::Message(MessageAction::Send) => self.send_message(action).await,
            ActionType::Message(MessageAction::Delete) => self.delete_message(action).await,
            ActionType::Message(MessageAction::DeleteBatch) => self.delete_messages_bulk(action).await,
            ActionType::Message(MessageAction::Pin) => self.pin_message(action).await,
            ActionType::Message(MessageAction::Unpin) => self.unpin_message(action).await,
            ActionType::Message(MessageAction::EditText) => self.edit_message(action).await,

            ActionType::Admin(AdminAction::Ban) => self.ban_user(action).await,
            ActionType::Admin(AdminAction::Unban) => self.unban_user(action).await,
            ActionType::Admin(AdminAction::Mute) => self.mute_user(action).await,
            ActionType::Admin(AdminAction::Unmute) => self.unmute_user(action).await,
            ActionType::Admin(AdminAction::Kick) => self.kick_user(action).await,
            ActionType::Admin(AdminAction::Promote) => self.add_role(action).await,
            ActionType::Admin(AdminAction::Demote) => self.remove_role(action).await,
            ActionType::Admin(AdminAction::SetPermissions) => self.set_channel_permissions(action).await,

            ActionType::Query(QueryAction::GetChatMember) => self.get_member(action).await,
            ActionType::Query(QueryAction::GetChat) => self.get_guild(action).await,

            ActionType::NoAction => {
                return ExecuteResult {
                    action_id: action.action_id.clone(),
                    success: true,
                    error: None,
                    api_method: Some("NoAction".into()),
                    api_response: Some(json!({"ok": true})),
                    agent_signature: None,
                };
            }

            other => {
                return ExecuteResult {
                    action_id: action.action_id.clone(),
                    success: false,
                    error: Some(format!("Discord 不支持的动作类型: {:?}", other)),
                    api_method: None,
                    api_response: None,
                    agent_signature: None,
                };
            }
        };

        match result {
            Ok((method, response)) => {
                info!(action_id = action.action_id, method = %method, "Discord API 执行成功");
                let sig = Self::sign_receipt(key_manager, &action.action_id, &method, &response);
                ExecuteResult {
                    action_id: action.action_id.clone(),
                    success: true,
                    error: None,
                    api_method: Some(method),
                    api_response: Some(response),
                    agent_signature: Some(sig),
                }
            }
            Err(e) => {
                warn!(action_id = action.action_id, error = %e, "Discord API 执行失败");
                ExecuteResult {
                    action_id: action.action_id.clone(),
                    success: false,
                    error: Some(e),
                    api_method: None,
                    api_response: None,
                    agent_signature: None,
                }
            }
        }
    }

    async fn ack_interaction(
        &self,
        interaction_id: &str,
        interaction_token: &str,
    ) -> anyhow::Result<()> {
        self.ack_interaction_api(interaction_id, interaction_token).await
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn register_commands(&self, guild_id: &str) -> anyhow::Result<()> {
        let commands = default_slash_commands();
        self.register_guild_commands(guild_id, commands).await
            .map_err(|e| anyhow::anyhow!(e))?;
        info!(guild_id, "Discord Slash Commands 已注册");
        Ok(())
    }
}

/// 默认 Slash Commands 定义
pub fn default_slash_commands() -> serde_json::Value {
    json!([
        {
            "name": "ban",
            "description": "Ban a user from the server",
            "options": [
                {"name": "user", "type": 6, "description": "Target user", "required": true},
                {"name": "reason", "type": 3, "description": "Ban reason"}
            ]
        },
        {
            "name": "unban",
            "description": "Unban a user",
            "options": [
                {"name": "user", "type": 6, "description": "Target user", "required": true}
            ]
        },
        {
            "name": "kick",
            "description": "Kick a user from the server",
            "options": [
                {"name": "user", "type": 6, "description": "Target user", "required": true}
            ]
        },
        {
            "name": "mute",
            "description": "Timeout a user",
            "options": [
                {"name": "user", "type": 6, "description": "Target user", "required": true},
                {"name": "duration", "type": 4, "description": "Duration in seconds", "required": true}
            ]
        },
        {
            "name": "unmute",
            "description": "Remove timeout from a user",
            "options": [
                {"name": "user", "type": 6, "description": "Target user", "required": true}
            ]
        },
        {
            "name": "warn",
            "description": "Warn a user",
            "options": [
                {"name": "user", "type": 6, "description": "Target user", "required": true},
                {"name": "reason", "type": 3, "description": "Warn reason"}
            ]
        },
        {
            "name": "config",
            "description": "Bot configuration",
            "options": [
                {
                    "name": "blacklist",
                    "type": 1,
                    "description": "Manage word blacklist",
                    "options": [
                        {"name": "action", "type": 3, "description": "add/remove/list", "required": true,
                         "choices": [
                            {"name": "add", "value": "add"},
                            {"name": "remove", "value": "remove"},
                            {"name": "list", "value": "list"}
                         ]},
                        {"name": "word", "type": 3, "description": "Word to add/remove"}
                    ]
                },
                {
                    "name": "flood",
                    "type": 1,
                    "description": "Set flood protection limit",
                    "options": [
                        {"name": "limit", "type": 4, "description": "Max messages per window"},
                        {"name": "window", "type": 4, "description": "Window in seconds"}
                    ]
                },
                {
                    "name": "welcome",
                    "type": 1,
                    "description": "Set welcome message",
                    "options": [
                        {"name": "message", "type": 3, "description": "Welcome text (use {name} for username)", "required": true}
                    ]
                },
                {
                    "name": "warnlimit",
                    "type": 1,
                    "description": "Set warn limit before auto-action",
                    "options": [
                        {"name": "limit", "type": 4, "description": "Warn count limit", "required": true}
                    ]
                },
                {
                    "name": "warnaction",
                    "type": 1,
                    "description": "Set action when warn limit reached",
                    "options": [
                        {"name": "action", "type": 3, "description": "ban/kick/mute", "required": true,
                         "choices": [
                            {"name": "ban", "value": "Ban"},
                            {"name": "kick", "value": "Kick"},
                            {"name": "mute", "value": "Mute"}
                         ]}
                    ]
                }
            ]
        },
        {
            "name": "help",
            "description": "Show bot help and available commands"
        },
        {
            "name": "rules",
            "description": "Show current group rules and configuration"
        }
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_slash_commands_valid_json() {
        let cmds = default_slash_commands();
        assert!(cmds.is_array());
        let arr = cmds.as_array().unwrap();
        assert!(arr.len() >= 8, "应至少有 8 个命令");

        let names: Vec<&str> = arr.iter()
            .filter_map(|c| c.get("name")?.as_str())
            .collect();
        assert!(names.contains(&"ban"));
        assert!(names.contains(&"kick"));
        assert!(names.contains(&"mute"));
        assert!(names.contains(&"warn"));
        assert!(names.contains(&"config"));
        assert!(names.contains(&"help"));
    }

    #[test]
    fn test_permissions_constants() {
        assert_eq!(permissions::KICK_MEMBERS, 2);
        assert_eq!(permissions::BAN_MEMBERS, 4);
        assert_eq!(permissions::ADMINISTRATOR, 8);
        assert_eq!(permissions::MANAGE_MESSAGES, 8192);
        assert_eq!(permissions::MODERATE_MEMBERS, 1 << 40);
    }

    #[test]
    fn test_sign_receipt_format() {
        let dir = tempfile::tempdir().unwrap();
        let km = crate::signer::KeyManager::load_or_generate(
            dir.path().to_str().unwrap(),
        ).unwrap();

        let sig = DiscordExecutor::sign_receipt(
            &km, "act_dc_001", "POST:/channels/123/messages", &json!({"id": "msg_1"}),
        );
        let parts: Vec<&str> = sig.split(':').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].len(), 64);
        assert_eq!(parts[1].len(), 128);
    }

    #[test]
    fn test_guild_id_extraction() {
        let exec = DiscordExecutor::new(
            "test".into(), "app123".into(), Client::new(),
        );

        let action = ExecuteAction {
            action_id: "a1".into(),
            action_type: ActionType::Admin(AdminAction::Ban),
            bot_id_hash: String::new(),
            chat_id: 0,
            params: json!({"guild_id": "guild_999", "user_id": "u1"}),
            leader_signature: String::new(),
            leader_node_id: String::new(),
            consensus_nodes: vec![],
            platform: "discord".into(),
        };
        assert_eq!(exec.guild_id_str(&action).unwrap(), "guild_999");

        // chat_id fallback
        let action2 = ExecuteAction {
            action_id: "a2".into(),
            action_type: ActionType::Admin(AdminAction::Ban),
            bot_id_hash: String::new(),
            chat_id: 123456,
            params: json!({"user_id": "u1"}),
            leader_signature: String::new(),
            leader_node_id: String::new(),
            consensus_nodes: vec![],
            platform: "discord".into(),
        };
        assert_eq!(exec.guild_id_str(&action2).unwrap(), "123456");
    }
}
