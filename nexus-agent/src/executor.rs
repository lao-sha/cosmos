use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug};

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

/// 消息操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageAction {
    Send,
    Delete,
    DeleteBatch,
    Pin,
    Unpin,
    /// 编辑消息文本 (editMessageText)
    EditText,
    /// 编辑消息 Inline 键盘 (editMessageReplyMarkup)
    EditReplyMarkup,
    /// 发送投票 (sendPoll)
    SendPoll,
    /// 停止投票 (stopPoll)
    StopPoll,
    /// 发送图片 (sendPhoto)
    SendPhoto,
    /// 发送文件 (sendDocument)
    SendDocument,
    /// 发送媒体组 (sendMediaGroup)
    SendMediaGroup,
    /// 发送聊天动作 (sendChatAction)
    SendChatAction,
    /// 取消所有置顶 (unpinAllChatMessages)
    UnpinAll,
}

/// 管理操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdminAction {
    Ban,
    Unban,
    Mute,
    Unmute,
    ApproveJoinRequest,
    DeclineJoinRequest,
    SetPermissions,
    Kick,
    Promote,
    Demote,
    /// 注册 Bot 命令菜单 (setMyCommands)
    SetMyCommands,
    /// 设置管理员自定义头衔 (setChatAdministratorCustomTitle)
    SetCustomTitle,
    /// 设置群标题 (setChatTitle)
    SetChatTitle,
    /// 设置群描述 (setChatDescription)
    SetChatDescription,
    /// 创建邀请链接 (createChatInviteLink)
    CreateInviteLink,
    /// 导出邀请链接 (exportChatInviteLink)
    ExportInviteLink,
    /// 撤销邀请链接 (revokeChatInviteLink)
    RevokeInviteLink,
}

/// 查询操作（只读，不需要共识）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryAction {
    GetChatMember,
    GetAdmins,
    GetChat,
    GetMe,
    /// 回复 Inline 键盘回调 (answerCallbackQuery)
    AnswerCallback,
}

/// 配置更新操作（通过管理员命令修改 GroupConfig）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigUpdateAction {
    AddBlacklistWord,
    RemoveBlacklistWord,
    LockType,
    UnlockType,
    SetWelcome,
    SetFloodLimit,
    SetWarnLimit,
    SetWarnAction,
}

/// 动作类型（嵌套枚举）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Message(MessageAction),
    Admin(AdminAction),
    Query(QueryAction),
    ConfigUpdate(ConfigUpdateAction),
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
    pub fn new(bot_token: String, client: reqwest::Client) -> Self {
        Self {
            bot_token,
            client,
        }
    }

    /// 执行动作（含签名回执）
    pub async fn execute(&self, action: &ExecuteAction, key_manager: &crate::signer::KeyManager) -> ExecuteResult {
        let result = match &action.action_type {
            ActionType::Message(MessageAction::Send) => {
                let text = action.params.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(empty)");
                self.call_tg_api("sendMessage", serde_json::json!({
                    "chat_id": action.chat_id,
                    "text": text,
                })).await
            }
            ActionType::Message(MessageAction::Delete) => {
                let message_id = action.params.get("message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("deleteMessage", serde_json::json!({
                    "chat_id": action.chat_id,
                    "message_id": message_id,
                })).await
            }
            ActionType::Message(MessageAction::DeleteBatch) => {
                let message_ids = action.params.get("message_ids")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                if message_ids.is_empty() {
                    return ExecuteResult {
                        action_id: action.action_id.clone(),
                        success: true,
                        error: None,
                        tg_api_method: Some("deleteMessages".into()),
                        tg_api_response: Some(serde_json::json!({"ok": true, "result": true})),
                        agent_signature: None,
                    };
                }
                self.call_tg_api("deleteMessages", serde_json::json!({
                    "chat_id": action.chat_id,
                    "message_ids": message_ids,
                })).await
            }
            ActionType::Message(MessageAction::Pin) => {
                let message_id = action.params.get("message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("pinChatMessage", serde_json::json!({
                    "chat_id": action.chat_id,
                    "message_id": message_id,
                })).await
            }
            ActionType::Message(MessageAction::Unpin) => {
                let message_id = action.params.get("message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("unpinChatMessage", serde_json::json!({
                    "chat_id": action.chat_id,
                    "message_id": message_id,
                })).await
            }
            ActionType::Admin(AdminAction::Ban) => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("banChatMember", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                })).await
            }
            ActionType::Admin(AdminAction::Unban) => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("unbanChatMember", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                    "only_if_banned": true,
                })).await
            }
            ActionType::Admin(AdminAction::Mute) => {
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
            ActionType::Admin(AdminAction::Unmute) => {
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
            ActionType::Admin(AdminAction::ApproveJoinRequest) => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("approveChatJoinRequest", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                })).await
            }
            ActionType::Admin(AdminAction::DeclineJoinRequest) => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("declineChatJoinRequest", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                })).await
            }
            ActionType::Admin(AdminAction::SetPermissions) => {
                let permissions = action.params.get("permissions")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));
                // 如果是针对单个用户 → restrictChatMember
                // 否则 → setChatPermissions (全群)
                if let Some(user_id) = action.params.get("user_id").and_then(|v| v.as_i64()) {
                    let mut body = serde_json::json!({
                        "chat_id": action.chat_id,
                        "user_id": user_id,
                        "permissions": permissions,
                    });
                    if let Some(until) = action.params.get("until_date") {
                        body["until_date"] = until.clone();
                    }
                    self.call_tg_api("restrictChatMember", body).await
                } else {
                    self.call_tg_api("setChatPermissions", serde_json::json!({
                        "chat_id": action.chat_id,
                        "permissions": permissions,
                    })).await
                }
            }
            ActionType::Admin(AdminAction::Kick) => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                // Kick = ban + immediate unban
                let ban_result = self.call_tg_api("banChatMember", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                })).await;
                if ban_result.is_ok() {
                    if let Err(e) = self.call_tg_api("unbanChatMember", serde_json::json!({
                        "chat_id": action.chat_id,
                        "user_id": user_id,
                        "only_if_banned": true,
                    })).await {
                        warn!(user_id, error = %e, "Kick: unban 失败，用户可能被永久封禁");
                    }
                }
                ban_result.map(|(_, resp)| ("kick".to_string(), resp))
            }
            ActionType::Admin(AdminAction::Promote) => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("promoteChatMember", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                    "can_manage_chat": true,
                    "can_delete_messages": true,
                    "can_restrict_members": true,
                    "can_pin_messages": true,
                    "can_invite_users": true,
                })).await
            }
            ActionType::Admin(AdminAction::Demote) => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("promoteChatMember", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                    "can_manage_chat": false,
                    "can_delete_messages": false,
                    "can_restrict_members": false,
                    "can_pin_messages": false,
                    "can_invite_users": false,
                    "can_promote_members": false,
                    "can_change_info": false,
                })).await
            }
            ActionType::Query(QueryAction::GetChatMember) => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("getChatMember", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                })).await
            }
            ActionType::Query(QueryAction::GetAdmins) => {
                self.call_tg_api("getChatAdministrators", serde_json::json!({
                    "chat_id": action.chat_id,
                })).await
            }
            ActionType::Query(QueryAction::GetChat) => {
                self.call_tg_api("getChat", serde_json::json!({
                    "chat_id": action.chat_id,
                })).await
            }
            ActionType::Query(QueryAction::GetMe) => {
                self.call_tg_api("getMe", serde_json::json!({})).await
            }
            ActionType::Query(QueryAction::AnswerCallback) => {
                let callback_query_id = action.params.get("callback_query_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let text = action.params.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let show_alert = action.params.get("show_alert")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                self.call_tg_api("answerCallbackQuery", serde_json::json!({
                    "callback_query_id": callback_query_id,
                    "text": text,
                    "show_alert": show_alert,
                })).await
            }
            ActionType::Message(MessageAction::EditText) => {
                let message_id = action.params.get("message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let text = action.params.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let mut body = serde_json::json!({
                    "chat_id": action.chat_id,
                    "message_id": message_id,
                    "text": text,
                });
                if let Some(parse_mode) = action.params.get("parse_mode") {
                    body["parse_mode"] = parse_mode.clone();
                }
                if let Some(markup) = action.params.get("reply_markup") {
                    body["reply_markup"] = markup.clone();
                }
                self.call_tg_api("editMessageText", body).await
            }
            ActionType::Message(MessageAction::EditReplyMarkup) => {
                let message_id = action.params.get("message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let mut body = serde_json::json!({
                    "chat_id": action.chat_id,
                    "message_id": message_id,
                });
                if let Some(markup) = action.params.get("reply_markup") {
                    body["reply_markup"] = markup.clone();
                }
                self.call_tg_api("editMessageReplyMarkup", body).await
            }
            ActionType::Message(MessageAction::SendPoll) => {
                let question = action.params.get("question")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let options = action.params.get("options")
                    .cloned()
                    .unwrap_or(serde_json::json!([]));
                let mut body = serde_json::json!({
                    "chat_id": action.chat_id,
                    "question": question,
                    "options": options,
                });
                if let Some(is_anonymous) = action.params.get("is_anonymous") {
                    body["is_anonymous"] = is_anonymous.clone();
                }
                if let Some(poll_type) = action.params.get("type") {
                    body["type"] = poll_type.clone();
                }
                self.call_tg_api("sendPoll", body).await
            }
            ActionType::Message(MessageAction::StopPoll) => {
                let message_id = action.params.get("message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                self.call_tg_api("stopPoll", serde_json::json!({
                    "chat_id": action.chat_id,
                    "message_id": message_id,
                })).await
            }
            // ═══ Sprint 12: 媒体 + Bot 初始化 ═══
            ActionType::Message(MessageAction::SendPhoto) => {
                let photo = action.params.get("photo")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let mut body = serde_json::json!({
                    "chat_id": action.chat_id,
                    "photo": photo,
                });
                if let Some(caption) = action.params.get("caption") {
                    body["caption"] = caption.clone();
                }
                if let Some(parse_mode) = action.params.get("parse_mode") {
                    body["parse_mode"] = parse_mode.clone();
                }
                if let Some(markup) = action.params.get("reply_markup") {
                    body["reply_markup"] = markup.clone();
                }
                self.call_tg_api("sendPhoto", body).await
            }
            ActionType::Message(MessageAction::SendDocument) => {
                let document = action.params.get("document")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let mut body = serde_json::json!({
                    "chat_id": action.chat_id,
                    "document": document,
                });
                if let Some(caption) = action.params.get("caption") {
                    body["caption"] = caption.clone();
                }
                if let Some(parse_mode) = action.params.get("parse_mode") {
                    body["parse_mode"] = parse_mode.clone();
                }
                self.call_tg_api("sendDocument", body).await
            }
            ActionType::Message(MessageAction::SendMediaGroup) => {
                let media = action.params.get("media")
                    .cloned()
                    .unwrap_or(serde_json::json!([]));
                self.call_tg_api("sendMediaGroup", serde_json::json!({
                    "chat_id": action.chat_id,
                    "media": media,
                })).await
            }
            ActionType::Message(MessageAction::SendChatAction) => {
                let chat_action = action.params.get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("typing");
                self.call_tg_api("sendChatAction", serde_json::json!({
                    "chat_id": action.chat_id,
                    "action": chat_action,
                })).await
            }
            ActionType::Message(MessageAction::UnpinAll) => {
                self.call_tg_api("unpinAllChatMessages", serde_json::json!({
                    "chat_id": action.chat_id,
                })).await
            }
            ActionType::Admin(AdminAction::SetMyCommands) => {
                let commands = action.params.get("commands")
                    .cloned()
                    .unwrap_or(serde_json::json!([]));
                let mut body = serde_json::json!({
                    "commands": commands,
                });
                if let Some(scope) = action.params.get("scope") {
                    body["scope"] = scope.clone();
                }
                if let Some(lang) = action.params.get("language_code") {
                    body["language_code"] = lang.clone();
                }
                self.call_tg_api("setMyCommands", body).await
            }
            ActionType::Admin(AdminAction::SetCustomTitle) => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let custom_title = action.params.get("custom_title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                self.call_tg_api("setChatAdministratorCustomTitle", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                    "custom_title": custom_title,
                })).await
            }
            ActionType::Admin(AdminAction::SetChatTitle) => {
                let title = action.params.get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                self.call_tg_api("setChatTitle", serde_json::json!({
                    "chat_id": action.chat_id,
                    "title": title,
                })).await
            }
            ActionType::Admin(AdminAction::SetChatDescription) => {
                let description = action.params.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                self.call_tg_api("setChatDescription", serde_json::json!({
                    "chat_id": action.chat_id,
                    "description": description,
                })).await
            }
            // ═══ Sprint 13: 邀请链接 ═══
            ActionType::Admin(AdminAction::CreateInviteLink) => {
                let mut body = serde_json::json!({
                    "chat_id": action.chat_id,
                });
                if let Some(name) = action.params.get("name") {
                    body["name"] = name.clone();
                }
                if let Some(expire) = action.params.get("expire_date") {
                    body["expire_date"] = expire.clone();
                }
                if let Some(limit) = action.params.get("member_limit") {
                    body["member_limit"] = limit.clone();
                }
                if let Some(approval) = action.params.get("creates_join_request") {
                    body["creates_join_request"] = approval.clone();
                }
                self.call_tg_api("createChatInviteLink", body).await
            }
            ActionType::Admin(AdminAction::ExportInviteLink) => {
                self.call_tg_api("exportChatInviteLink", serde_json::json!({
                    "chat_id": action.chat_id,
                })).await
            }
            ActionType::Admin(AdminAction::RevokeInviteLink) => {
                let invite_link = action.params.get("invite_link")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                self.call_tg_api("revokeChatInviteLink", serde_json::json!({
                    "chat_id": action.chat_id,
                    "invite_link": invite_link,
                })).await
            }
            ActionType::ConfigUpdate(_) => {
                // ConfigUpdate 由 handle_execute 在调用 executor 之前/之后处理
                // executor 仅发确认消息到群内
                Ok(("ConfigUpdate".to_string(), serde_json::json!({"ok": true})))
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

    /// 权限前置检查: 目标用户是否为管理员
    ///
    /// 如果是管理员则返回 Err，防止 Bot 尝试封禁/禁言管理员导致 API 失败
    pub async fn check_target_not_admin(
        &self,
        chat_id: i64,
        user_id: i64,
    ) -> Result<(), String> {
        if user_id == 0 { return Ok(()); }

        let result = self.call_tg_api("getChatMember", serde_json::json!({
            "chat_id": chat_id,
            "user_id": user_id,
        })).await;

        if let Ok((_, resp)) = result {
            let status = resp.pointer("/result/status")
                .and_then(|v| v.as_str())
                .unwrap_or("member");
            if status == "administrator" || status == "creator" {
                return Err(format!("目标用户 {} 是管理员 ({}), 无法执行管理操作", user_id, status));
            }
        }
        // 如果 getChatMember 失败，不阻止后续操作 (可能是私聊等场景)
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════
    // ActionType 序列化 / 反序列化
    // ═══════════════════════════════════════════

    #[test]
    fn test_action_type_roundtrip() {
        let cases = vec![
            ActionType::Message(MessageAction::Send),
            ActionType::Message(MessageAction::Delete),
            ActionType::Message(MessageAction::DeleteBatch),
            ActionType::Message(MessageAction::Pin),
            ActionType::Message(MessageAction::Unpin),
            ActionType::Message(MessageAction::EditText),
            ActionType::Message(MessageAction::EditReplyMarkup),
            ActionType::Message(MessageAction::SendPoll),
            ActionType::Message(MessageAction::StopPoll),
            ActionType::Message(MessageAction::SendPhoto),
            ActionType::Message(MessageAction::SendDocument),
            ActionType::Message(MessageAction::SendMediaGroup),
            ActionType::Message(MessageAction::SendChatAction),
            ActionType::Message(MessageAction::UnpinAll),
            ActionType::Admin(AdminAction::Ban),
            ActionType::Admin(AdminAction::Unban),
            ActionType::Admin(AdminAction::Mute),
            ActionType::Admin(AdminAction::Unmute),
            ActionType::Admin(AdminAction::Kick),
            ActionType::Admin(AdminAction::Promote),
            ActionType::Admin(AdminAction::Demote),
            ActionType::Admin(AdminAction::ApproveJoinRequest),
            ActionType::Admin(AdminAction::DeclineJoinRequest),
            ActionType::Admin(AdminAction::SetPermissions),
            ActionType::Admin(AdminAction::SetMyCommands),
            ActionType::Admin(AdminAction::SetCustomTitle),
            ActionType::Admin(AdminAction::SetChatTitle),
            ActionType::Admin(AdminAction::SetChatDescription),
            ActionType::Admin(AdminAction::CreateInviteLink),
            ActionType::Admin(AdminAction::ExportInviteLink),
            ActionType::Admin(AdminAction::RevokeInviteLink),
            ActionType::Query(QueryAction::GetChatMember),
            ActionType::Query(QueryAction::GetAdmins),
            ActionType::Query(QueryAction::GetChat),
            ActionType::Query(QueryAction::GetMe),
            ActionType::Query(QueryAction::AnswerCallback),
            ActionType::NoAction,
        ];

        for action in &cases {
            let json = serde_json::to_string(action).unwrap();
            let back: ActionType = serde_json::from_str(&json).unwrap();
            assert_eq!(
                format!("{:?}", action),
                format!("{:?}", back),
                "ActionType {:?} roundtrip 失败",
                action,
            );
        }
    }

    #[test]
    fn test_execute_action_deserialize() {
        let json = r#"{
            "action_id": "act_001",
            "action_type": {"Admin": "Ban"},
            "bot_id_hash": "abc123",
            "chat_id": -100123456,
            "params": {"user_id": 42},
            "leader_signature": "pk:sig",
            "leader_node_id": "node_001",
            "consensus_nodes": ["node_001", "node_002", "node_003"]
        }"#;

        let action: ExecuteAction = serde_json::from_str(json).unwrap();
        assert_eq!(action.action_id, "act_001");
        assert_eq!(action.chat_id, -100123456);
        assert_eq!(action.consensus_nodes.len(), 3);
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::Ban)));
    }

    #[test]
    fn test_execute_result_serialize() {
        let result = ExecuteResult {
            action_id: "act_001".into(),
            success: true,
            error: None,
            tg_api_method: Some("banChatMember".into()),
            tg_api_response: Some(serde_json::json!({"ok": true})),
            agent_signature: Some("pk:sig".into()),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["tg_api_method"], "banChatMember");
        assert!(json["error"].is_null());
    }

    // ═══════════════════════════════════════════
    // sign_receipt 验证
    // ═══════════════════════════════════════════

    #[test]
    fn test_sign_receipt_format() {
        let dir = tempfile::tempdir().unwrap();
        let km = crate::signer::KeyManager::load_or_generate(
            dir.path().to_str().unwrap(),
        ).unwrap();

        let sig = TelegramExecutor::sign_receipt(
            &km,
            "act_001",
            "banChatMember",
            &serde_json::json!({"ok": true}),
        );

        // 格式: pk_hex:sig_hex
        let parts: Vec<&str> = sig.split(':').collect();
        assert_eq!(parts.len(), 2, "签名格式应为 pk:sig");
        assert_eq!(parts[0].len(), 64, "公钥 hex 应为 64 字符");
        assert_eq!(parts[1].len(), 128, "签名 hex 应为 128 字符");
        assert_eq!(parts[0], km.public_key_hex());
    }

    #[test]
    fn test_sign_receipt_verifiable() {
        use ed25519_dalek::{VerifyingKey, Verifier, Signature};
        use sha2::{Sha256, Digest};

        let dir = tempfile::tempdir().unwrap();
        let km = crate::signer::KeyManager::load_or_generate(
            dir.path().to_str().unwrap(),
        ).unwrap();

        let action_id = "act_verify";
        let method = "sendMessage";
        let response = serde_json::json!({"ok": true, "result": {"message_id": 42}});

        let sig_str = TelegramExecutor::sign_receipt(&km, action_id, method, &response);
        let parts: Vec<&str> = sig_str.split(':').collect();

        // 重建 receipt_data
        let response_bytes = serde_json::to_vec(&response).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&response_bytes);
        let response_hash = hasher.finalize();

        let mut receipt_data = Vec::new();
        receipt_data.extend_from_slice(action_id.as_bytes());
        receipt_data.extend_from_slice(method.as_bytes());
        receipt_data.extend_from_slice(&response_hash);

        let sig_bytes = hex::decode(parts[1]).unwrap();
        let sig = Signature::from_bytes(&sig_bytes.try_into().unwrap());
        let vk = VerifyingKey::from_bytes(&km.public_key_bytes()).unwrap();
        assert!(vk.verify(&receipt_data, &sig).is_ok(), "sign_receipt 签名应可验证");
    }

    #[test]
    fn test_sign_receipt_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        let km = crate::signer::KeyManager::load_or_generate(
            dir.path().to_str().unwrap(),
        ).unwrap();

        let resp = serde_json::json!({"ok": true});
        let s1 = TelegramExecutor::sign_receipt(&km, "a", "m", &resp);
        let s2 = TelegramExecutor::sign_receipt(&km, "a", "m", &resp);
        assert_eq!(s1, s2, "相同输入签名应确定性相同");
    }

    #[test]
    fn test_sign_receipt_different_actions() {
        let dir = tempfile::tempdir().unwrap();
        let km = crate::signer::KeyManager::load_or_generate(
            dir.path().to_str().unwrap(),
        ).unwrap();

        let resp = serde_json::json!({"ok": true});
        let s1 = TelegramExecutor::sign_receipt(&km, "act_1", "ban", &resp);
        let s2 = TelegramExecutor::sign_receipt(&km, "act_2", "ban", &resp);
        assert_ne!(s1, s2, "不同 action_id 签名应不同");
    }
}
