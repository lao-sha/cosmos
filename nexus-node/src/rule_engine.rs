use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::types::{ActionType, MessageAction, AdminAction, QueryAction, ConfigUpdateAction, SignedMessage, GroupConfig, JoinApprovalPolicy};
use crate::chain_cache::ChainCache;

/// 规则引擎
///
/// 根据链上群规则 + 消息内容 → 决定执行什么动作
/// 支持可扩展的规则链: 反垃圾 → 命令解析 → 入群审批 → 默认
pub struct RuleEngine {
    /// 规则列表（按优先级排序）
    rules: Vec<Box<dyn Rule + Send + Sync>>,
}

/// 规则 trait
pub trait Rule: Send + Sync {
    /// 规则名称
    fn name(&self) -> &str;

    /// 匹配: 返回 Some(action) 如果匹配，None 继续下一条规则
    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction>;
}

/// 规则上下文（传入每条规则）
#[derive(Debug, Clone)]
pub struct RuleContext {
    /// 原始消息
    pub message: SignedMessage,
    /// 提取的文本
    pub text: String,
    /// chat_id
    pub chat_id: i64,
    /// 发送者 user_id
    pub sender_id: i64,
    /// 是否为 bot 命令 (以 / 开头)
    pub is_command: bool,
    /// 命令名 (不含 /)
    pub command: Option<String>,
    /// 命令参数
    pub command_args: Option<String>,
    /// reply_to 的消息发送者 ID
    pub reply_to_user_id: Option<i64>,
    /// reply_to 的消息 ID
    pub reply_to_message_id: Option<i64>,
    /// 是否为入群申请
    pub is_join_request: bool,
    /// 入群申请者 ID
    pub join_request_user_id: Option<i64>,
    /// 是否为回调查询
    pub is_callback: bool,
    /// 回调数据
    pub callback_data: Option<String>,
    /// 群配置（从 ChainCache 获取，Sprint 8 同步）
    pub group_config: Option<GroupConfig>,
}

/// 规则匹配后的动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleAction {
    pub action_type: ActionType,
    pub chat_id: i64,
    pub params: serde_json::Value,
    pub reason: String,
}

impl RuleEngine {
    /// 创建默认规则引擎（内置规则链）
    pub fn default_engine() -> Self {
        let rules: Vec<Box<dyn Rule + Send + Sync>> = vec![
            Box::new(JoinRequestRule),
            Box::new(AdminPermissionRule),
            Box::new(CommandRule),
            Box::new(AntifloodRule),
            Box::new(SpamDetectorRule),
            Box::new(BlacklistRule),
            Box::new(LockRule),
            Box::new(WelcomeRule),
            Box::new(LinkFilterRule),
            Box::new(DefaultRule),
        ];

        Self { rules }
    }

    /// 评估消息 → 返回动作
    pub fn evaluate(&self, message: &SignedMessage, chain_cache: Option<&ChainCache>) -> RuleAction {
        let ctx = Self::build_context(message, chain_cache);

        for rule in &self.rules {
            if let Some(action) = rule.evaluate(&ctx) {
                debug!(
                    rule = rule.name(),
                    action_type = ?action.action_type,
                    reason = action.reason,
                    "规则匹配"
                );
                return action;
            }
        }

        // 兜底: 无动作
        RuleAction {
            action_type: ActionType::NoAction,
            chat_id: ctx.chat_id,
            params: serde_json::json!({}),
            reason: "no_rule_matched".into(),
        }
    }

    /// 从 ChainCache 查找 GroupConfig
    fn lookup_group_config(bot_id_hash: &str, chain_cache: Option<&ChainCache>) -> Option<GroupConfig> {
        chain_cache
            .and_then(|cache| cache.get_group_config(bot_id_hash))
            .map(|signed| signed.config)
    }

    /// 从 SignedMessage 构建规则上下文
    fn build_context(message: &SignedMessage, chain_cache: Option<&ChainCache>) -> RuleContext {
        let update = &message.telegram_update;

        let chat_id = update.pointer("/message/chat/id")
            .or_else(|| update.pointer("/callback_query/message/chat/id"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let text = update.pointer("/message/text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let sender_id = update.pointer("/message/from/id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let is_command = text.starts_with('/');
        let (command, command_args) = if is_command {
            let parts: Vec<&str> = text.splitn(2, ' ').collect();
            let cmd = parts[0].trim_start_matches('/').to_string();
            // 去掉 @bot_name
            let cmd = cmd.split('@').next().unwrap_or("").to_string();
            let args = parts.get(1).map(|s| s.to_string());
            (Some(cmd), args)
        } else {
            (None, None)
        };

        let reply_to_user_id = update.pointer("/message/reply_to_message/from/id")
            .and_then(|v| v.as_i64());

        let reply_to_message_id = update.pointer("/message/reply_to_message/message_id")
            .and_then(|v| v.as_i64());

        let is_join_request = update.get("chat_join_request").is_some();
        let join_request_user_id = update.pointer("/chat_join_request/from/id")
            .and_then(|v| v.as_i64());

        let is_callback = update.get("callback_query").is_some();
        let callback_data = update.pointer("/callback_query/data")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        RuleContext {
            message: message.clone(),
            text,
            chat_id,
            sender_id,
            is_command,
            command,
            command_args,
            reply_to_user_id,
            reply_to_message_id,
            is_join_request,
            join_request_user_id,
            is_callback,
            callback_data,
            group_config: Self::lookup_group_config(&message.bot_id_hash, chain_cache),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 内置规则
// ═══════════════════════════════════════════════════════════════

/// 入群申请规则
struct JoinRequestRule;

impl Rule for JoinRequestRule {
    fn name(&self) -> &str { "join_request" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        if !ctx.is_join_request {
            return None;
        }

        let user_id = ctx.join_request_user_id.unwrap_or(0);

        // 根据群配置决定是否自动通过
        let policy = ctx.group_config.as_ref()
            .map(|c| &c.join_policy);

        match policy {
            Some(JoinApprovalPolicy::AutoApprove) | None => Some(RuleAction {
                action_type: ActionType::Admin(AdminAction::ApproveJoinRequest),
                chat_id: ctx.chat_id,
                params: serde_json::json!({ "user_id": user_id }),
                reason: "auto_approve_join".into(),
            }),
            Some(JoinApprovalPolicy::ManualApproval) => None,
            Some(JoinApprovalPolicy::CaptchaRequired) => None, // TODO: 验证码流程未实现
            Some(JoinApprovalPolicy::TokenGating { .. }) => None, // TODO: 链上余额验证未实现
        }
    }
}

/// 命令规则 (/ban, /mute, /unmute, /pin, /del)
struct CommandRule;

impl Rule for CommandRule {
    fn name(&self) -> &str { "command" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        if !ctx.is_command {
            return None;
        }

        let cmd = ctx.command.as_deref()?;

        match cmd {
            "ban" | "kick" => {
                let user_id = ctx.reply_to_user_id.unwrap_or(0);
                if user_id == 0 { return None; }
                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::Ban),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "user_id": user_id }),
                    reason: "command_ban".into(),
                })
            }
            "mute" => {
                let user_id = ctx.reply_to_user_id.unwrap_or(0);
                if user_id == 0 { return None; }
                let duration = ctx.command_args.as_deref()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(3600);
                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::Mute),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "user_id": user_id,
                        "duration_seconds": duration,
                    }),
                    reason: "command_mute".into(),
                })
            }
            "unmute" => {
                let user_id = ctx.reply_to_user_id.unwrap_or(0);
                if user_id == 0 { return None; }
                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::Unmute),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "user_id": user_id }),
                    reason: "command_unmute".into(),
                })
            }
            "pin" => {
                let msg_id = ctx.reply_to_message_id.unwrap_or(0);
                if msg_id == 0 { return None; }
                Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::Pin),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "message_id": msg_id }),
                    reason: "command_pin".into(),
                })
            }
            "del" | "delete" => {
                let msg_id = ctx.reply_to_message_id.unwrap_or(0);
                if msg_id == 0 { return None; }
                Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::Delete),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "message_id": msg_id }),
                    reason: "command_delete".into(),
                })
            }
            "unban" => {
                let user_id = ctx.reply_to_user_id.unwrap_or(0);
                if user_id == 0 { return None; }
                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::Unban),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "user_id": user_id }),
                    reason: "command_unban".into(),
                })
            }
            "warn" => {
                let user_id = ctx.reply_to_user_id.unwrap_or(0);
                if user_id == 0 { return None; }
                let reason = ctx.command_args.clone().unwrap_or_default();
                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::Mute), // placeholder — Agent handles warn logic
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "user_id": user_id,
                        "warn_action": "add",
                        "reason": reason,
                    }),
                    reason: "command_warn".into(),
                })
            }
            "unwarn" => {
                let user_id = ctx.reply_to_user_id.unwrap_or(0);
                if user_id == 0 { return None; }
                Some(RuleAction {
                    action_type: ActionType::NoAction,
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "user_id": user_id,
                        "warn_action": "remove",
                    }),
                    reason: "command_unwarn".into(),
                })
            }
            "warns" => {
                let user_id = ctx.reply_to_user_id.unwrap_or(ctx.sender_id);
                Some(RuleAction {
                    action_type: ActionType::Query(QueryAction::GetChatMember),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "user_id": user_id,
                        "warn_action": "query",
                    }),
                    reason: "command_warns".into(),
                })
            }
            "resetwarns" => {
                let user_id = ctx.reply_to_user_id.unwrap_or(0);
                if user_id == 0 { return None; }
                Some(RuleAction {
                    action_type: ActionType::NoAction,
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "user_id": user_id,
                        "warn_action": "reset",
                    }),
                    reason: "command_resetwarns".into(),
                })
            }
            "help" => {
                Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::Send),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "text": concat!(
                            "📋 *Available commands:*\n\n",
                            "👮 *Admin:*\n",
                            "/ban - Ban user (reply)\n",
                            "/unban - Unban user (reply)\n",
                            "/mute <sec> - Mute user (reply)\n",
                            "/unmute - Unmute user (reply)\n",
                            "/kick - Kick user (reply)\n",
                            "/warn - Warn user (reply)\n",
                            "/unwarn - Remove warning (reply)\n",
                            "/warns - View warnings\n",
                            "/resetwarns - Reset warnings (reply)\n",
                            "/pin - Pin message (reply)\n",
                            "/del - Delete message (reply)\n",
                            "/unpinall - Unpin all messages\n",
                            "/perms <perm> <on|off> - Set permissions\n",
                            "/poll Q | A | B - Create poll\n\n",
                            "🖼 *Media:*\n",
                            "/photo <url> [caption] - Send photo\n\n",
                            "🔗 *Invite & Settings:*\n",
                            "/invite [name] - Create invite link\n",
                            "/revoke <link> - Revoke invite link\n",
                            "/exportlink - Export primary link\n",
                            "/title <text> - Set group title\n",
                            "/desc <text> - Set group description\n",
                            "/admintitle <title> - Set admin title (reply)\n",
                            "/setcommands - Register bot menu\n\n",
                            "ℹ️ *Info:*\n",
                            "/id - Show user/chat ID\n",
                            "/info - User info (reply)\n",
                            "/rules - Group rules\n",
                            "/help - This message",
                        ),
                        "parse_mode": "Markdown",
                    }),
                    reason: "command_help".into(),
                })
            }
            "rules" => {
                let rules_text = ctx.group_config.as_ref()
                    .map(|c| {
                        let mut lines = vec!["📜 *Group Rules:*\n".to_string()];
                        if c.antiflood_limit > 0 {
                            lines.push(format!("• Antiflood: {} msgs / {}s → {:?}",
                                c.antiflood_limit, c.antiflood_window, c.antiflood_action));
                        }
                        if c.warn_limit > 0 {
                            lines.push(format!("• Warn limit: {} → {:?}", c.warn_limit, c.warn_action));
                        }
                        if !c.blacklist_words.is_empty() {
                            lines.push(format!("• Blacklist: {} words ({:?})",
                                c.blacklist_words.len(), c.blacklist_mode));
                        }
                        if !c.lock_types.is_empty() {
                            let types: Vec<String> = c.lock_types.iter().map(|t| format!("{:?}", t)).collect();
                            lines.push(format!("• Locked: {}", types.join(", ")));
                        }
                        if c.filter_links {
                            lines.push("• Links: filtered".to_string());
                        }
                        if c.spam_detection_enabled {
                            lines.push(format!("• Spam detection: ON (max emoji: {})", c.spam_max_emoji));
                        }
                        if !c.welcome_message.is_empty() {
                            lines.push("• Welcome message: configured".to_string());
                        }
                        if lines.len() == 1 {
                            lines.push("No special rules configured.".to_string());
                        }
                        lines.join("\n")
                    })
                    .unwrap_or_else(|| "No group configuration found.".to_string());
                Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::Send),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "text": rules_text,
                        "parse_mode": "Markdown",
                    }),
                    reason: "command_rules".into(),
                })
            }
            "info" => {
                let target_id = ctx.reply_to_user_id.unwrap_or(ctx.sender_id);
                Some(RuleAction {
                    action_type: ActionType::Query(QueryAction::GetChatMember),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "user_id": target_id,
                        "info_action": "user_info",
                    }),
                    reason: "command_info".into(),
                })
            }
            "id" => {
                let target_id = ctx.reply_to_user_id.unwrap_or(ctx.sender_id);
                Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::Send),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "text": format!("🆔 User ID: {}\n💬 Chat ID: {}", target_id, ctx.chat_id),
                    }),
                    reason: "command_id".into(),
                })
            }

            // ═══ 配置修改命令（需要共识 → Agent 更新 GroupConfig）═══

            "blacklist" => {
                let word = ctx.command_args.as_deref()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                if word.is_empty() { return None; }
                Some(RuleAction {
                    action_type: ActionType::ConfigUpdate(ConfigUpdateAction::AddBlacklistWord),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "word": word }),
                    reason: "command_blacklist_add".into(),
                })
            }
            "unblacklist" => {
                let word = ctx.command_args.as_deref()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                if word.is_empty() { return None; }
                Some(RuleAction {
                    action_type: ActionType::ConfigUpdate(ConfigUpdateAction::RemoveBlacklistWord),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "word": word }),
                    reason: "command_blacklist_remove".into(),
                })
            }
            "blacklists" => {
                let words = ctx.group_config.as_ref()
                    .map(|c| {
                        if c.blacklist_words.is_empty() {
                            "📝 Blacklist is empty.".to_string()
                        } else {
                            let list = c.blacklist_words.iter()
                                .enumerate()
                                .map(|(i, w)| format!("{}. `{}`", i + 1, w))
                                .collect::<Vec<_>>()
                                .join("\n");
                            format!("📝 *Blacklist words* ({} total, mode: {:?}):\n{}",
                                c.blacklist_words.len(), c.blacklist_mode, list)
                        }
                    })
                    .unwrap_or_else(|| "No config found.".to_string());
                Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::Send),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "text": words, "parse_mode": "Markdown" }),
                    reason: "command_blacklists".into(),
                })
            }
            "lock" => {
                let lock_str = ctx.command_args.as_deref()
                    .map(|s| s.trim().to_lowercase())
                    .unwrap_or_default();
                if lock_str.is_empty() { return None; }
                Some(RuleAction {
                    action_type: ActionType::ConfigUpdate(ConfigUpdateAction::LockType),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "lock_type": lock_str }),
                    reason: "command_lock".into(),
                })
            }
            "unlock" => {
                let lock_str = ctx.command_args.as_deref()
                    .map(|s| s.trim().to_lowercase())
                    .unwrap_or_default();
                if lock_str.is_empty() { return None; }
                Some(RuleAction {
                    action_type: ActionType::ConfigUpdate(ConfigUpdateAction::UnlockType),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "lock_type": lock_str }),
                    reason: "command_unlock".into(),
                })
            }
            "locks" => {
                let locks_text = ctx.group_config.as_ref()
                    .map(|c| {
                        if c.lock_types.is_empty() {
                            "🔓 No locked message types.".to_string()
                        } else {
                            let types: Vec<String> = c.lock_types.iter()
                                .map(|t| format!("{:?}", t))
                                .collect();
                            format!("🔒 *Locked types*: {}", types.join(", "))
                        }
                    })
                    .unwrap_or_else(|| "No config found.".to_string());
                Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::Send),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "text": locks_text, "parse_mode": "Markdown" }),
                    reason: "command_locks".into(),
                })
            }
            "welcome" => {
                let text = ctx.command_args.as_deref()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                Some(RuleAction {
                    action_type: ActionType::ConfigUpdate(ConfigUpdateAction::SetWelcome),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "text": text }),
                    reason: "command_welcome".into(),
                })
            }
            "flood" => {
                let args = ctx.command_args.as_deref().unwrap_or("").trim();
                let limit = args.parse::<u16>().unwrap_or(0);
                Some(RuleAction {
                    action_type: ActionType::ConfigUpdate(ConfigUpdateAction::SetFloodLimit),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "limit": limit }),
                    reason: "command_flood".into(),
                })
            }
            "setwarnlimit" => {
                let args = ctx.command_args.as_deref().unwrap_or("").trim();
                let limit = args.parse::<u8>().unwrap_or(3);
                Some(RuleAction {
                    action_type: ActionType::ConfigUpdate(ConfigUpdateAction::SetWarnLimit),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "limit": limit }),
                    reason: "command_setwarnlimit".into(),
                })
            }
            "setwarnaction" => {
                let action_str = ctx.command_args.as_deref()
                    .map(|s| s.trim().to_lowercase())
                    .unwrap_or_default();
                if !["ban", "kick", "mute"].contains(&action_str.as_str()) {
                    return Some(RuleAction {
                        action_type: ActionType::Message(MessageAction::Send),
                        chat_id: ctx.chat_id,
                        params: serde_json::json!({
                            "text": "⚠️ Usage: /setwarnaction <ban|kick|mute>"
                        }),
                        reason: "command_setwarnaction_usage".into(),
                    });
                }
                Some(RuleAction {
                    action_type: ActionType::ConfigUpdate(ConfigUpdateAction::SetWarnAction),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "action": action_str }),
                    reason: "command_setwarnaction".into(),
                })
            }
            "perms" => {
                // /perms <permission> <on|off>
                // 例: /perms send_messages off → 全群禁言
                // 例: /perms send_media on → 允许发送媒体
                let args = ctx.command_args.as_deref().unwrap_or("").trim();
                let parts: Vec<&str> = args.splitn(2, ' ').collect();

                if parts.len() < 2 || parts[0].is_empty() {
                    return Some(RuleAction {
                        action_type: ActionType::Message(MessageAction::Send),
                        chat_id: ctx.chat_id,
                        params: serde_json::json!({
                            "text": concat!(
                                "⚠️ Usage: /perms <permission> <on|off>\n\n",
                                "Permissions:\n",
                                "• send_messages\n",
                                "• send_media\n",
                                "• send_polls\n",
                                "• send_other\n",
                                "• add_previews\n",
                                "• change_info\n",
                                "• invite_users\n",
                                "• pin_messages\n\n",
                                "Example: /perms send_messages off",
                            )
                        }),
                        reason: "command_perms_usage".into(),
                    });
                }

                let perm_name = parts[0].to_lowercase();
                let enabled = parts[1].trim().to_lowercase();
                let value = matches!(enabled.as_str(), "on" | "true" | "1" | "yes");

                // 映射到 Telegram ChatPermissions 字段名
                let tg_field = match perm_name.as_str() {
                    "send_messages" => "can_send_messages",
                    "send_media" => "can_send_media_messages",
                    "send_polls" => "can_send_polls",
                    "send_other" => "can_send_other_messages",
                    "add_previews" => "can_add_web_page_previews",
                    "change_info" => "can_change_info",
                    "invite_users" => "can_invite_users",
                    "pin_messages" => "can_pin_messages",
                    _ => {
                        return Some(RuleAction {
                            action_type: ActionType::Message(MessageAction::Send),
                            chat_id: ctx.chat_id,
                            params: serde_json::json!({
                                "text": format!("⚠️ Unknown permission: {}", perm_name),
                            }),
                            reason: "command_perms_unknown".into(),
                        });
                    }
                };

                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::SetPermissions),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "permissions": { tg_field: value },
                    }),
                    reason: "command_perms".into(),
                })
            }
            "poll" => {
                // /poll Question | Option1 | Option2 [| Option3 ...]
                let args = ctx.command_args.as_deref().unwrap_or("").trim();
                let parts: Vec<&str> = args.split('|').map(|s| s.trim()).collect();

                if parts.len() < 3 || parts[0].is_empty() {
                    return Some(RuleAction {
                        action_type: ActionType::Message(MessageAction::Send),
                        chat_id: ctx.chat_id,
                        params: serde_json::json!({
                            "text": "⚠️ Usage: /poll Question | Option1 | Option2 [| Option3 ...]",
                        }),
                        reason: "command_poll_usage".into(),
                    });
                }

                let question = parts[0].to_string();
                let options: Vec<serde_json::Value> = parts[1..].iter()
                    .map(|s| serde_json::json!({"text": s.to_string()}))
                    .collect();

                Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::SendPoll),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "question": question,
                        "options": options,
                        "is_anonymous": false,
                    }),
                    reason: "command_poll".into(),
                })
            }
            // ═══ Sprint 12: Bot 初始化 + 媒体 ═══
            "setcommands" => {
                // /setcommands — 注册 Bot 命令菜单（使用预定义列表）
                let commands = serde_json::json!([
                    {"command": "help", "description": "Show available commands"},
                    {"command": "rules", "description": "Show group rules"},
                    {"command": "info", "description": "User info"},
                    {"command": "id", "description": "Show user/chat ID"},
                    {"command": "ban", "description": "Ban user (reply)"},
                    {"command": "unban", "description": "Unban user (reply)"},
                    {"command": "mute", "description": "Mute user (reply)"},
                    {"command": "unmute", "description": "Unmute user (reply)"},
                    {"command": "kick", "description": "Kick user (reply)"},
                    {"command": "warn", "description": "Warn user (reply)"},
                    {"command": "warns", "description": "View warnings"},
                    {"command": "pin", "description": "Pin message (reply)"},
                    {"command": "del", "description": "Delete message (reply)"},
                    {"command": "perms", "description": "Set group permissions"},
                    {"command": "poll", "description": "Create a poll"},
                    {"command": "invite", "description": "Create invite link"},
                    {"command": "title", "description": "Set group title"},
                    {"command": "desc", "description": "Set group description"},
                ]);
                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::SetMyCommands),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "commands": commands }),
                    reason: "command_setcommands".into(),
                })
            }
            "photo" => {
                // /photo <file_id_or_url> [caption]
                let args = ctx.command_args.as_deref().unwrap_or("").trim();
                let parts: Vec<&str> = args.splitn(2, ' ').collect();

                if parts.is_empty() || parts[0].is_empty() {
                    return Some(RuleAction {
                        action_type: ActionType::Message(MessageAction::Send),
                        chat_id: ctx.chat_id,
                        params: serde_json::json!({
                            "text": "⚠️ Usage: /photo <file_id_or_url> [caption]",
                        }),
                        reason: "command_photo_usage".into(),
                    });
                }

                let photo = parts[0].to_string();
                let caption = parts.get(1).map(|s| s.to_string());

                let mut params = serde_json::json!({ "photo": photo });
                if let Some(cap) = caption {
                    params["caption"] = serde_json::json!(cap);
                }

                Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::SendPhoto),
                    chat_id: ctx.chat_id,
                    params,
                    reason: "command_photo".into(),
                })
            }
            "unpinall" => {
                Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::UnpinAll),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({}),
                    reason: "command_unpinall".into(),
                })
            }
            // ═══ Sprint 13: 邀请链接 + 管理员 ═══
            "invite" => {
                // /invite [name] — 创建邀请链接
                let name = ctx.command_args.as_deref()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                let mut params = serde_json::json!({});
                if let Some(n) = name {
                    params["name"] = serde_json::json!(n);
                }
                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::CreateInviteLink),
                    chat_id: ctx.chat_id,
                    params,
                    reason: "command_invite".into(),
                })
            }
            "revoke" => {
                // /revoke <invite_link> — 撤销邀请链接
                let link = ctx.command_args.as_deref()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                if link.is_empty() {
                    return Some(RuleAction {
                        action_type: ActionType::Message(MessageAction::Send),
                        chat_id: ctx.chat_id,
                        params: serde_json::json!({
                            "text": "⚠️ Usage: /revoke <invite_link>",
                        }),
                        reason: "command_revoke_usage".into(),
                    });
                }

                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::RevokeInviteLink),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "invite_link": link }),
                    reason: "command_revoke".into(),
                })
            }
            "exportlink" => {
                // /exportlink — 导出主邀请链接
                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::ExportInviteLink),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({}),
                    reason: "command_exportlink".into(),
                })
            }
            "title" => {
                // /title <new title>
                let title = ctx.command_args.as_deref()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                if title.is_empty() {
                    return Some(RuleAction {
                        action_type: ActionType::Message(MessageAction::Send),
                        chat_id: ctx.chat_id,
                        params: serde_json::json!({
                            "text": "⚠️ Usage: /title <new title>",
                        }),
                        reason: "command_title_usage".into(),
                    });
                }

                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::SetChatTitle),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "title": title }),
                    reason: "command_title".into(),
                })
            }
            "desc" => {
                // /desc <new description>
                let description = ctx.command_args.as_deref()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                if description.is_empty() {
                    return Some(RuleAction {
                        action_type: ActionType::Message(MessageAction::Send),
                        chat_id: ctx.chat_id,
                        params: serde_json::json!({
                            "text": "⚠️ Usage: /desc <new description>",
                        }),
                        reason: "command_desc_usage".into(),
                    });
                }

                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::SetChatDescription),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "description": description }),
                    reason: "command_desc".into(),
                })
            }
            "admintitle" => {
                // /admintitle <custom_title> — 设置管理员头衔 (回复目标用户)
                let custom_title = ctx.command_args.as_deref()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                if custom_title.is_empty() {
                    return Some(RuleAction {
                        action_type: ActionType::Message(MessageAction::Send),
                        chat_id: ctx.chat_id,
                        params: serde_json::json!({
                            "text": "⚠️ Usage: /admintitle <custom_title> (reply to user)",
                        }),
                        reason: "command_admintitle_usage".into(),
                    });
                }

                let user_id = ctx.reply_to_user_id.unwrap_or(0);
                if user_id == 0 {
                    return Some(RuleAction {
                        action_type: ActionType::Message(MessageAction::Send),
                        chat_id: ctx.chat_id,
                        params: serde_json::json!({
                            "text": "⚠️ Reply to a user to set their admin title.",
                        }),
                        reason: "command_admintitle_no_reply".into(),
                    });
                }

                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::SetCustomTitle),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "user_id": user_id,
                        "custom_title": custom_title,
                    }),
                    reason: "command_admintitle".into(),
                })
            }
            _ => None,
        }
    }
}

/// 链接过滤规则
struct LinkFilterRule;

impl Rule for LinkFilterRule {
    fn name(&self) -> &str { "link_filter" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        let filter_enabled = ctx.group_config.as_ref()
            .map(|c| c.filter_links)
            .unwrap_or(false);

        if !filter_enabled {
            return None;
        }

        // 简单 URL 检测
        let has_link = ctx.text.contains("http://")
            || ctx.text.contains("https://")
            || ctx.text.contains("t.me/");

        if has_link {
            let msg_id = ctx.message.telegram_update
                .pointer("/message/message_id")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            if msg_id > 0 {
                return Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::Delete),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "message_id": msg_id }),
                    reason: "link_filtered".into(),
                });
            }
        }

        None
    }
}

/// 防刷屏规则
///
/// 统计每个用户在时间窗口内的消息数，超过阈值触发动作。
/// 实际执行在 Agent 本地快速路径（LocalProcessor），此处用于 Node 侧验证。
///
/// 参考: FallenRobot/modules/antiflood.py — CHAT_FLOOD 内存字典 + 5 种 flood_type
struct AntifloodRule;

impl Rule for AntifloodRule {
    fn name(&self) -> &str { "antiflood" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        let config = ctx.group_config.as_ref()?;

        if config.antiflood_limit == 0 {
            return None;
        }

        // 跳过命令消息
        if ctx.is_command || ctx.is_join_request || ctx.is_callback {
            return None;
        }

        // 跳过白名单和管理员
        let sender_str = ctx.sender_id.to_string();
        if config.whitelist.contains(&sender_str) || config.admins.contains(&sender_str) {
            return None;
        }

        // Node 侧标记：需要 Agent 检查防刷屏（实际计数在 Agent LocalStore）
        // 这里返回 NoAction + metadata，Agent LocalProcessor 负责实际检查
        None
    }
}

/// 黑名单关键词过滤规则
///
/// 检查消息文本是否包含黑名单关键词。
///
/// 参考: FallenRobot/modules/blacklist.py — SQL blacklist 表 + 正则匹配
/// 参考: tg-spam/lib/tgspam/detector.go — stopWords 精确匹配
struct BlacklistRule;

impl Rule for BlacklistRule {
    fn name(&self) -> &str { "blacklist" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        let config = ctx.group_config.as_ref()?;

        if config.blacklist_words.is_empty() || ctx.text.is_empty() {
            return None;
        }

        // 跳过命令消息
        if ctx.is_command || ctx.is_join_request || ctx.is_callback {
            return None;
        }

        // 跳过白名单和管理员
        let sender_str = ctx.sender_id.to_string();
        if config.whitelist.contains(&sender_str) || config.admins.contains(&sender_str) {
            return None;
        }

        let text_lower = ctx.text.to_lowercase();
        let matched = match &config.blacklist_mode {
            crate::types::BlacklistMode::Exact => {
                config.blacklist_words.iter().any(|w| text_lower == w.to_lowercase())
            }
            crate::types::BlacklistMode::Contains => {
                config.blacklist_words.iter().any(|w| text_lower.contains(&w.to_lowercase()))
            }
            crate::types::BlacklistMode::Regex => {
                config.blacklist_words.iter().any(|pattern| {
                    regex::Regex::new(pattern)
                        .map(|re| re.is_match(&text_lower))
                        .unwrap_or(false)
                })
            }
        };

        if !matched {
            return None;
        }

        let msg_id = ctx.message.telegram_update
            .pointer("/message/message_id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        if msg_id == 0 {
            return None;
        }

        // 根据 blacklist_action 决定动作
        match &config.blacklist_action {
            crate::types::BlacklistAction::Delete => {
                Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::Delete),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({ "message_id": msg_id }),
                    reason: "blacklist_delete".into(),
                })
            }
            crate::types::BlacklistAction::DeleteAndWarn => {
                Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::Delete),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "message_id": msg_id,
                        "user_id": ctx.sender_id,
                        "also_warn": true,
                    }),
                    reason: "blacklist_delete_warn".into(),
                })
            }
            crate::types::BlacklistAction::DeleteAndMute => {
                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::Mute),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "message_id": msg_id,
                        "user_id": ctx.sender_id,
                        "duration_seconds": config.auto_mute_duration,
                        "also_delete": true,
                    }),
                    reason: "blacklist_delete_mute".into(),
                })
            }
            crate::types::BlacklistAction::DeleteAndBan => {
                Some(RuleAction {
                    action_type: ActionType::Admin(AdminAction::Ban),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "message_id": msg_id,
                        "user_id": ctx.sender_id,
                        "also_delete": true,
                    }),
                    reason: "blacklist_delete_ban".into(),
                })
            }
        }
    }
}

/// 消息类型锁定规则
///
/// 检查消息类型是否在锁定列表中。
///
/// 参考: FallenRobot/modules/locks.py — LOCK_TYPES 字典 (25 种)
struct LockRule;

impl Rule for LockRule {
    fn name(&self) -> &str { "lock" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        let config = ctx.group_config.as_ref()?;

        if config.lock_types.is_empty() {
            return None;
        }

        if ctx.is_command || ctx.is_join_request || ctx.is_callback {
            return None;
        }

        // 跳过白名单和管理员
        let sender_str = ctx.sender_id.to_string();
        if config.whitelist.contains(&sender_str) || config.admins.contains(&sender_str) {
            return None;
        }

        let update = &ctx.message.telegram_update;
        let msg = update.get("message")?;

        // 检测消息类型
        let detected_type = if msg.get("photo").is_some() {
            Some(crate::types::LockType::Photo)
        } else if msg.get("video").is_some() {
            Some(crate::types::LockType::Video)
        } else if msg.get("audio").is_some() {
            Some(crate::types::LockType::Audio)
        } else if msg.get("document").is_some() {
            Some(crate::types::LockType::Document)
        } else if msg.get("sticker").is_some() {
            Some(crate::types::LockType::Sticker)
        } else if msg.get("animation").is_some() {
            Some(crate::types::LockType::Gif)
        } else if msg.get("voice").is_some() {
            Some(crate::types::LockType::Voice)
        } else if msg.get("contact").is_some() {
            Some(crate::types::LockType::Contact)
        } else if msg.get("location").is_some() {
            Some(crate::types::LockType::Location)
        } else if msg.get("poll").is_some() {
            Some(crate::types::LockType::Poll)
        } else if msg.get("game").is_some() {
            Some(crate::types::LockType::Game)
        } else if msg.get("forward_from").is_some() || msg.get("forward_from_chat").is_some() {
            Some(crate::types::LockType::Forward)
        } else if msg.get("via_bot").is_some() {
            Some(crate::types::LockType::Inline)
        } else {
            None
        };

        let lock_type = detected_type?;

        if !config.lock_types.contains(&lock_type) {
            return None;
        }

        let msg_id = msg.get("message_id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        if msg_id == 0 {
            return None;
        }

        Some(RuleAction {
            action_type: ActionType::Message(MessageAction::Delete),
            chat_id: ctx.chat_id,
            params: serde_json::json!({ "message_id": msg_id }),
            reason: format!("lock_{:?}", lock_type),
        })
    }
}

/// 欢迎消息规则
///
/// 新成员加入时发送欢迎消息。
///
/// 参考: FallenRobot/modules/welcome.py — 支持变量替换 {first}/{chatname}/{id}
struct WelcomeRule;

impl Rule for WelcomeRule {
    fn name(&self) -> &str { "welcome" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        let config = ctx.group_config.as_ref()?;

        if config.welcome_message.is_empty() {
            return None;
        }

        let update = &ctx.message.telegram_update;
        let new_members = update.pointer("/message/new_chat_members")?;
        let members = new_members.as_array()?;

        if members.is_empty() {
            return None;
        }

        // 取第一个新成员做变量替换
        let member = &members[0];
        let first_name = member.get("first_name")
            .and_then(|v| v.as_str())
            .unwrap_or("User");
        let last_name = member.get("last_name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let user_id = member.get("id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let username = member.get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let chat_name = update.pointer("/message/chat/title")
            .and_then(|v| v.as_str())
            .unwrap_or("Group");

        let fullname = if last_name.is_empty() {
            first_name.to_string()
        } else {
            format!("{} {}", first_name, last_name)
        };

        let text = config.welcome_message
            .replace("{first}", first_name)
            .replace("{last}", last_name)
            .replace("{fullname}", &fullname)
            .replace("{username}", username)
            .replace("{id}", &user_id.to_string())
            .replace("{chatname}", chat_name);

        // 支持媒体欢迎消息: {photo:<file_id>} caption text...
        if text.starts_with("{photo:") {
            if let Some(end) = text.find('}') {
                let file_id = &text[7..end];
                let caption = text[end+1..].trim();
                let mut params = serde_json::json!({ "photo": file_id });
                if !caption.is_empty() {
                    params["caption"] = serde_json::json!(caption);
                }
                return Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::SendPhoto),
                    chat_id: ctx.chat_id,
                    params,
                    reason: "welcome_photo".into(),
                });
            }
        }

        Some(RuleAction {
            action_type: ActionType::Message(MessageAction::Send),
            chat_id: ctx.chat_id,
            params: serde_json::json!({ "text": text }),
            reason: "welcome_message".into(),
        })
    }
}

/// 管理员权限前置检查规则
///
/// 在 CommandRule 之前运行：如果是 admin 命令但发送者不是管理员，
/// 直接拒绝并回复提示。
///
/// 参考: FallenRobot/modules/helper_funcs/chat_status.py — is_user_admin()
struct AdminPermissionRule;

/// 需要管理员权限的命令列表
const ADMIN_COMMANDS: &[&str] = &[
    "ban", "unban", "kick", "mute", "unmute", "warn", "unwarn",
    "resetwarns", "pin", "del", "delete",
    "blacklist", "unblacklist", "lock", "unlock", "welcome",
    "flood", "setwarnlimit", "setwarnaction", "perms", "poll",
    "setcommands", "photo", "unpinall",
    "invite", "revoke", "exportlink", "title", "desc", "admintitle",
];

impl Rule for AdminPermissionRule {
    fn name(&self) -> &str { "admin_permission" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        if !ctx.is_command {
            return None;
        }

        let cmd = ctx.command.as_deref()?;

        // 只拦截需要管理员权限的命令
        if !ADMIN_COMMANDS.contains(&cmd) {
            return None;
        }

        let config = ctx.group_config.as_ref()?;
        let sender_str = ctx.sender_id.to_string();

        // 管理员名单中存在 → 放行（返回 None 继续到 CommandRule）
        if config.admins.contains(&sender_str) {
            return None;
        }

        // 白名单也放行
        if config.whitelist.contains(&sender_str) {
            return None;
        }

        // 非管理员 → 拒绝
        debug!(cmd, sender_id = ctx.sender_id, "非管理员执行 admin 命令，已拒绝");

        Some(RuleAction {
            action_type: ActionType::Message(MessageAction::Send),
            chat_id: ctx.chat_id,
            params: serde_json::json!({
                "text": format!("⛔ You don't have permission to use /{}", cmd),
                "reply_to_message_id": ctx.message.telegram_update
                    .pointer("/message/message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            }),
            reason: "admin_permission_denied".into(),
        })
    }
}

/// 反垃圾检测规则 (Node 侧)
///
/// 检查消息文本是否存在垃圾/spam 特征:
///   - 过多 emoji
///   - 过多大写字母 (全大写刷屏)
///   - 多语言混排 (Latin + Cyrillic + CJK 混合 → 常见 spam 特征)
///   - 过短消息重复发送标记
///
/// 实际计数在 Agent LocalProcessor（重复检测 + emoji）。
/// 此规则做 Node 侧 二次验证 / 标记。
///
/// 参考: tg-spam/lib/tgspam/detector.go — classifier chain
struct SpamDetectorRule;

impl Rule for SpamDetectorRule {
    fn name(&self) -> &str { "spam_detector" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        let config = ctx.group_config.as_ref()?;

        if !config.spam_detection_enabled {
            return None;
        }

        if ctx.is_command || ctx.is_join_request || ctx.is_callback || ctx.text.is_empty() {
            return None;
        }

        // 跳过白名单和管理员
        let sender_str = ctx.sender_id.to_string();
        if config.whitelist.contains(&sender_str) || config.admins.contains(&sender_str) {
            return None;
        }

        let text = &ctx.text;
        let msg_id = ctx.message.telegram_update
            .pointer("/message/message_id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        if msg_id == 0 {
            return None;
        }

        // ═══ 1. Emoji 过多检测 ═══
        if config.spam_max_emoji > 0 {
            let emoji_count = text.chars().filter(|c| is_emoji(*c)).count() as u8;
            if emoji_count > config.spam_max_emoji {
                return Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::Delete),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "message_id": msg_id,
                        "user_id": ctx.sender_id,
                        "spam_type": "emoji",
                        "count": emoji_count,
                    }),
                    reason: format!("spam_emoji_{}", emoji_count),
                });
            }
        }

        // ═══ 2. 全大写刷屏检测 ═══
        let alpha_chars: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
        if alpha_chars.len() >= 10 {
            let upper_count = alpha_chars.iter().filter(|c| c.is_uppercase()).count();
            let upper_ratio = upper_count as f32 / alpha_chars.len() as f32;
            if upper_ratio > 0.8 {
                return Some(RuleAction {
                    action_type: ActionType::Message(MessageAction::Delete),
                    chat_id: ctx.chat_id,
                    params: serde_json::json!({
                        "message_id": msg_id,
                        "user_id": ctx.sender_id,
                        "spam_type": "caps",
                        "ratio": upper_ratio,
                    }),
                    reason: "spam_caps".into(),
                });
            }
        }

        // ═══ 3. 多语言混排检测 (Latin + Cyrillic 混合) ═══
        let has_latin = text.chars().any(|c| matches!(c, 'a'..='z' | 'A'..='Z'));
        let has_cyrillic = text.chars().any(|c| matches!(c as u32, 0x0400..=0x04FF));
        let has_cjk = text.chars().any(|c| matches!(c as u32, 0x4E00..=0x9FFF));

        // Latin + Cyrillic 混排是常见 spam/phishing 特征
        let script_count = [has_latin, has_cyrillic, has_cjk].iter().filter(|&&b| b).count();
        if script_count >= 3 || (has_latin && has_cyrillic && text.len() > 20) {
            return Some(RuleAction {
                action_type: ActionType::Message(MessageAction::Delete),
                chat_id: ctx.chat_id,
                params: serde_json::json!({
                    "message_id": msg_id,
                    "user_id": ctx.sender_id,
                    "spam_type": "mixed_script",
                }),
                reason: "spam_mixed_script".into(),
            });
        }

        None
    }
}

/// 简单 emoji 检测（覆盖常见 Unicode emoji 范围）
fn is_emoji(c: char) -> bool {
    let cp = c as u32;
    (0x1F600..=0x1F64F).contains(&cp) || // Emoticons
    (0x1F300..=0x1F5FF).contains(&cp) || // Misc Symbols & Pictographs
    (0x1F680..=0x1F6FF).contains(&cp) || // Transport & Map
    (0x2600..=0x26FF).contains(&cp) ||   // Misc Symbols
    (0x2700..=0x27BF).contains(&cp) ||   // Dingbats
    (0x1F900..=0x1F9FF).contains(&cp) || // Supplemental Symbols
    (0x1F1E0..=0x1F1FF).contains(&cp)    // Flags
}

/// 默认规则（兜底）
struct DefaultRule;

impl Rule for DefaultRule {
    fn name(&self) -> &str { "default" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        Some(RuleAction {
            action_type: ActionType::NoAction,
            chat_id: ctx.chat_id,
            params: serde_json::json!({}),
            reason: "passthrough".into(),
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// 多平台适配器 trait
// ═══════════════════════════════════════════════════════════════

/// 平台适配器 trait
///
/// 所有平台（Telegram, Discord, Slack, Matrix, Farcaster）实现此 trait
pub trait PlatformAdapter: Send + Sync {
    /// 平台名称
    fn platform_name(&self) -> &str;

    /// 从平台原始 JSON 提取 RuleContext
    fn extract_context(&self, raw_json: &serde_json::Value) -> Option<RuleContext>;

    /// 将通用 RuleAction 转换为平台特定的 API 调用参数
    fn action_to_api_call(&self, action: &RuleAction) -> Option<PlatformApiCall>;
}

/// 平台 API 调用描述
#[derive(Debug, Clone)]
pub struct PlatformApiCall {
    /// API 方法
    pub method: String,
    /// API URL
    pub url: String,
    /// 请求体
    pub body: serde_json::Value,
}

/// Telegram 适配器
pub struct TelegramAdapter;

impl PlatformAdapter for TelegramAdapter {
    fn platform_name(&self) -> &str { "telegram" }

    fn extract_context(&self, _raw_json: &serde_json::Value) -> Option<RuleContext> {
        // Telegram 的上下文提取在 RuleEngine::build_context 中已实现
        // 此处提供统一接口
        None // TODO: 统一 RuleEngine::build_context 到这里
    }

    fn action_to_api_call(&self, action: &RuleAction) -> Option<PlatformApiCall> {
        let (method, body) = match &action.action_type {
            ActionType::Message(MessageAction::Send) => {
                let text = action.params.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                ("sendMessage", serde_json::json!({
                    "chat_id": action.chat_id,
                    "text": text,
                }))
            }
            ActionType::Message(MessageAction::Delete) => {
                let msg_id = action.params.get("message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                ("deleteMessage", serde_json::json!({
                    "chat_id": action.chat_id,
                    "message_id": msg_id,
                }))
            }
            ActionType::Admin(AdminAction::Ban) => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                ("banChatMember", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                }))
            }
            ActionType::Admin(AdminAction::Mute) => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                ("restrictChatMember", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                    "permissions": {"can_send_messages": false},
                }))
            }
            ActionType::Admin(AdminAction::ApproveJoinRequest) => {
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                ("approveChatJoinRequest", serde_json::json!({
                    "chat_id": action.chat_id,
                    "user_id": user_id,
                }))
            }
            ActionType::Message(MessageAction::Pin) => {
                let msg_id = action.params.get("message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                ("pinChatMessage", serde_json::json!({
                    "chat_id": action.chat_id,
                    "message_id": msg_id,
                }))
            }
            ActionType::NoAction => return None,
            _ => return None,
        };

        Some(PlatformApiCall {
            method: method.to_string(),
            url: String::new(), // 由调用方填入 bot_token
            body,
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// Discord 适配器
// ═══════════════════════════════════════════════════════════════

/// Discord 适配器
///
/// Discord Gateway Event → RuleContext
/// RuleAction → Discord REST API 调用
///
/// 支持的事件: MESSAGE_CREATE, GUILD_MEMBER_ADD
/// 支持的动作: SendMessage, DeleteMessage, BanUser, MuteUser (timeout)
pub struct DiscordAdapter;

impl PlatformAdapter for DiscordAdapter {
    fn platform_name(&self) -> &str { "discord" }

    fn extract_context(&self, raw_json: &serde_json::Value) -> Option<RuleContext> {
        // Discord Gateway Event 格式:
        // { "t": "MESSAGE_CREATE", "d": { "id": "...", "channel_id": "...", "content": "...", "author": { "id": "..." } } }
        let event_type = raw_json.get("t")?.as_str()?;
        let data = raw_json.get("d")?;

        match event_type {
            "MESSAGE_CREATE" => {
                let channel_id = data.get("channel_id")?.as_str()?;
                let content = data.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let author_id = data.pointer("/author/id").and_then(|v| v.as_str()).unwrap_or("0");
                let _message_id = data.get("id").and_then(|v| v.as_str()).unwrap_or("0");

                let is_command = content.starts_with('!') || content.starts_with('/');
                let (command, command_args) = if is_command {
                    let trimmed = content.trim_start_matches('!').trim_start_matches('/');
                    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
                    (Some(parts[0].to_string()), parts.get(1).map(|s| s.to_string()))
                } else {
                    (None, None)
                };

                // Discord: reply via message_reference
                let reply_to_user_id = data.pointer("/referenced_message/author/id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<i64>().ok());
                let reply_to_message_id = data.pointer("/referenced_message/id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<i64>().ok());

                // Discord mentions
                let mentions = data.get("mentions").and_then(|v| v.as_array());
                let mentioned_user_id = mentions
                    .and_then(|m| m.first())
                    .and_then(|u| u.get("id"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<i64>().ok());

                // 优先使用 reply，其次 mention
                let target_user = reply_to_user_id.or(mentioned_user_id);

                Some(RuleContext {
                    message: SignedMessage {
                        owner_public_key: String::new(),
                        bot_id_hash: String::new(),
                        sequence: 0,
                        timestamp: 0,
                        message_hash: String::new(),
                        telegram_update: raw_json.clone(),
                        owner_signature: String::new(),
                        platform: "discord".into(),
                    },
                    text: content.to_string(),
                    chat_id: channel_id.parse().unwrap_or(0),
                    sender_id: author_id.parse().unwrap_or(0),
                    is_command,
                    command,
                    command_args,
                    reply_to_user_id: target_user,
                    reply_to_message_id,
                    is_join_request: false,
                    join_request_user_id: None,
                    is_callback: false,
                    callback_data: None,
                    group_config: None,
                })
            }
            "GUILD_MEMBER_ADD" => {
                let user_id = data.pointer("/user/id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0);
                let guild_id = data.get("guild_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0);

                Some(RuleContext {
                    message: SignedMessage {
                        owner_public_key: String::new(),
                        bot_id_hash: String::new(),
                        sequence: 0,
                        timestamp: 0,
                        message_hash: String::new(),
                        telegram_update: raw_json.clone(),
                        owner_signature: String::new(),
                        platform: "discord".into(),
                    },
                    text: String::new(),
                    chat_id: guild_id,
                    sender_id: user_id,
                    is_command: false,
                    command: None,
                    command_args: None,
                    reply_to_user_id: None,
                    reply_to_message_id: None,
                    is_join_request: true,
                    join_request_user_id: Some(user_id),
                    is_callback: false,
                    callback_data: None,
                    group_config: None,
                })
            }
            _ => None,
        }
    }

    fn action_to_api_call(&self, action: &RuleAction) -> Option<PlatformApiCall> {
        // Discord REST API base: https://discord.com/api/v10
        let channel_id = action.chat_id;

        match &action.action_type {
            ActionType::Message(MessageAction::Send) => {
                let text = action.params.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                Some(PlatformApiCall {
                    method: "POST".into(),
                    url: format!("/channels/{}/messages", channel_id),
                    body: serde_json::json!({ "content": text }),
                })
            }
            ActionType::Message(MessageAction::Delete) => {
                let msg_id = action.params.get("message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                Some(PlatformApiCall {
                    method: "DELETE".into(),
                    url: format!("/channels/{}/messages/{}", channel_id, msg_id),
                    body: serde_json::json!({}),
                })
            }
            ActionType::Admin(AdminAction::Ban) => {
                let guild_id = action.params.get("guild_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(channel_id);
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                Some(PlatformApiCall {
                    method: "PUT".into(),
                    url: format!("/guilds/{}/bans/{}", guild_id, user_id),
                    body: serde_json::json!({ "delete_message_seconds": 0 }),
                })
            }
            ActionType::Admin(AdminAction::Mute) => {
                // Discord: timeout via PATCH /guilds/{guild}/members/{user}
                let guild_id = action.params.get("guild_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(channel_id);
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let duration = action.params.get("duration_seconds")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3600);
                let until = chrono::Utc::now() + chrono::Duration::seconds(duration as i64);
                Some(PlatformApiCall {
                    method: "PATCH".into(),
                    url: format!("/guilds/{}/members/{}", guild_id, user_id),
                    body: serde_json::json!({
                        "communication_disabled_until": until.to_rfc3339()
                    }),
                })
            }
            ActionType::Admin(AdminAction::Unmute) => {
                let guild_id = action.params.get("guild_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(channel_id);
                let user_id = action.params.get("user_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                Some(PlatformApiCall {
                    method: "PATCH".into(),
                    url: format!("/guilds/{}/members/{}", guild_id, user_id),
                    body: serde_json::json!({
                        "communication_disabled_until": serde_json::Value::Null
                    }),
                })
            }
            ActionType::Message(MessageAction::Pin) => {
                let msg_id = action.params.get("message_id")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                Some(PlatformApiCall {
                    method: "PUT".into(),
                    url: format!("/channels/{}/pins/{}", channel_id, msg_id),
                    body: serde_json::json!({}),
                })
            }
            ActionType::NoAction => None,
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Slack 适配器
// ═══════════════════════════════════════════════════════════════

/// Slack 适配器
///
/// Slack Events API → RuleContext
/// RuleAction → Slack Web API 调用
///
/// 支持的事件: message, team_join
/// 支持的动作: SendMessage, DeleteMessage, BanUser (kick)
pub struct SlackAdapter;

impl PlatformAdapter for SlackAdapter {
    fn platform_name(&self) -> &str { "slack" }

    fn extract_context(&self, raw_json: &serde_json::Value) -> Option<RuleContext> {
        // Slack Events API 格式:
        // { "type": "event_callback", "event": { "type": "message", "channel": "C...", "user": "U...", "text": "..." } }
        let event = raw_json.get("event")?;
        let event_type = event.get("type")?.as_str()?;

        match event_type {
            "message" => {
                // 过滤 bot 消息和子类型消息
                if event.get("bot_id").is_some() {
                    return None;
                }
                if let Some(subtype) = event.get("subtype") {
                    if subtype.as_str() != Some("file_share") {
                        return None; // 只处理普通消息和文件分享
                    }
                }

                let _channel = event.get("channel")?.as_str()?;
                let _user = event.get("user").and_then(|v| v.as_str()).unwrap_or("unknown");
                let text = event.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let _ts = event.get("ts").and_then(|v| v.as_str()).unwrap_or("0");

                let is_command = text.starts_with('!') || text.starts_with('/');
                let (command, command_args) = if is_command {
                    let trimmed = text.trim_start_matches('!').trim_start_matches('/');
                    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
                    (Some(parts[0].to_string()), parts.get(1).map(|s| s.to_string()))
                } else {
                    (None, None)
                };

                // Slack: user mentions via <@U12345>
                let mentioned_user = extract_slack_mention(text);

                // Slack: thread reply → parent message
                let _thread_ts = event.get("thread_ts").and_then(|v| v.as_str());

                Some(RuleContext {
                    message: SignedMessage {
                        owner_public_key: String::new(),
                        bot_id_hash: String::new(),
                        sequence: 0,
                        timestamp: 0,
                        message_hash: String::new(),
                        telegram_update: raw_json.clone(),
                        owner_signature: String::new(),
                        platform: "slack".into(),
                    },
                    text: text.to_string(),
                    chat_id: 0, // Slack channel IDs are strings, stored in raw_json
                    sender_id: 0, // Slack user IDs are strings
                    is_command,
                    command,
                    command_args,
                    reply_to_user_id: mentioned_user.and_then(|s| s.parse::<i64>().ok()),
                    reply_to_message_id: None,
                    is_join_request: false,
                    join_request_user_id: None,
                    is_callback: false,
                    callback_data: None,
                    group_config: None,
                })
            }
            "team_join" => {
                let user_id = event.pointer("/user/id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0");

                Some(RuleContext {
                    message: SignedMessage {
                        owner_public_key: String::new(),
                        bot_id_hash: String::new(),
                        sequence: 0,
                        timestamp: 0,
                        message_hash: String::new(),
                        telegram_update: raw_json.clone(),
                        owner_signature: String::new(),
                        platform: "slack".into(),
                    },
                    text: String::new(),
                    chat_id: 0,
                    sender_id: 0,
                    is_command: false,
                    command: None,
                    command_args: None,
                    reply_to_user_id: None,
                    reply_to_message_id: None,
                    is_join_request: true,
                    join_request_user_id: user_id.parse::<i64>().ok(),
                    is_callback: false,
                    callback_data: None,
                    group_config: None,
                })
            }
            _ => None,
        }
    }

    fn action_to_api_call(&self, action: &RuleAction) -> Option<PlatformApiCall> {
        // Slack Web API: https://slack.com/api/...
        match &action.action_type {
            ActionType::Message(MessageAction::Send) => {
                let text = action.params.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let channel = action.params.get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                Some(PlatformApiCall {
                    method: "chat.postMessage".into(),
                    url: "https://slack.com/api/chat.postMessage".into(),
                    body: serde_json::json!({
                        "channel": channel,
                        "text": text,
                    }),
                })
            }
            ActionType::Message(MessageAction::Delete) => {
                let channel = action.params.get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let ts = action.params.get("ts")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                Some(PlatformApiCall {
                    method: "chat.delete".into(),
                    url: "https://slack.com/api/chat.delete".into(),
                    body: serde_json::json!({
                        "channel": channel,
                        "ts": ts,
                    }),
                })
            }
            ActionType::Admin(AdminAction::Ban) => {
                // Slack: kick from channel (no permanent ban in Slack API)
                let channel = action.params.get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let user = action.params.get("user_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                Some(PlatformApiCall {
                    method: "conversations.kick".into(),
                    url: "https://slack.com/api/conversations.kick".into(),
                    body: serde_json::json!({
                        "channel": channel,
                        "user": user,
                    }),
                })
            }
            ActionType::Message(MessageAction::Pin) => {
                let channel = action.params.get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let ts = action.params.get("ts")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                Some(PlatformApiCall {
                    method: "pins.add".into(),
                    url: "https://slack.com/api/pins.add".into(),
                    body: serde_json::json!({
                        "channel": channel,
                        "timestamp": ts,
                    }),
                })
            }
            ActionType::NoAction => None,
            _ => None,
        }
    }
}

/// 从 Slack 文本中提取 <@U12345> 格式的 mention
fn extract_slack_mention(text: &str) -> Option<&str> {
    if let Some(start) = text.find("<@") {
        let rest = &text[start + 2..];
        if let Some(end) = rest.find('>') {
            let user_ref = &rest[..end];
            // 可能是 "U12345" 或 "U12345|display_name"
            return Some(user_ref.split('|').next().unwrap_or(user_ref));
        }
    }
    None
}

/// 平台适配器注册表
pub struct AdapterRegistry {
    adapters: Vec<Box<dyn PlatformAdapter>>,
}

impl AdapterRegistry {
    /// 创建包含所有内置适配器的注册表
    pub fn new() -> Self {
        Self {
            adapters: vec![
                Box::new(TelegramAdapter),
                Box::new(DiscordAdapter),
                Box::new(SlackAdapter),
            ],
        }
    }

    /// 根据平台名获取适配器
    pub fn get(&self, platform: &str) -> Option<&dyn PlatformAdapter> {
        self.adapters.iter()
            .find(|a| a.platform_name() == platform)
            .map(|a| a.as_ref())
    }

    /// 所有支持的平台名称
    pub fn platforms(&self) -> Vec<&str> {
        self.adapters.iter().map(|a| a.platform_name()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(update: serde_json::Value) -> SignedMessage {
        SignedMessage {
            owner_public_key: String::new(),
            bot_id_hash: String::new(),
            sequence: 1,
            timestamp: 0,
            message_hash: String::new(),
            telegram_update: update,
            owner_signature: String::new(),
            platform: "telegram".into(),
        }
    }

    #[test]
    fn test_join_request_auto_approve() {
        let engine = RuleEngine::default_engine();
        let msg = make_msg(serde_json::json!({
            "update_id": 1,
            "chat_join_request": {
                "chat": {"id": -100},
                "from": {"id": 456, "is_bot": false}
            }
        }));

        let action = engine.evaluate(&msg, None);
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::ApproveJoinRequest)));
        assert_eq!(action.params["user_id"], 456);
    }

    #[test]
    fn test_ban_command() {
        let engine = RuleEngine::default_engine();
        let msg = make_msg(serde_json::json!({
            "update_id": 1,
            "message": {
                "message_id": 10,
                "chat": {"id": -100, "type": "supergroup"},
                "from": {"id": 1, "is_bot": false},
                "text": "/ban",
                "reply_to_message": {
                    "message_id": 9,
                    "from": {"id": 789, "is_bot": false}
                }
            }
        }));

        let action = engine.evaluate(&msg, None);
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::Ban)));
        assert_eq!(action.params["user_id"], 789);
    }

    #[test]
    fn test_mute_with_duration() {
        let engine = RuleEngine::default_engine();
        let msg = make_msg(serde_json::json!({
            "update_id": 1,
            "message": {
                "message_id": 10,
                "chat": {"id": -100, "type": "supergroup"},
                "from": {"id": 1, "is_bot": false},
                "text": "/mute 7200",
                "reply_to_message": {
                    "message_id": 9,
                    "from": {"id": 789, "is_bot": false}
                }
            }
        }));

        let action = engine.evaluate(&msg, None);
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::Mute)));
        assert_eq!(action.params["duration_seconds"], 7200);
    }

    #[test]
    fn test_plain_message_no_action() {
        let engine = RuleEngine::default_engine();
        let msg = make_msg(serde_json::json!({
            "update_id": 1,
            "message": {
                "message_id": 10,
                "chat": {"id": -100, "type": "supergroup"},
                "from": {"id": 1, "is_bot": false},
                "text": "hello world"
            }
        }));

        let action = engine.evaluate(&msg, None);
        assert!(matches!(action.action_type, ActionType::NoAction));
    }

    #[test]
    fn test_telegram_adapter_ban_api_call() {
        let adapter = TelegramAdapter;
        let action = RuleAction {
            action_type: ActionType::Admin(AdminAction::Ban),
            chat_id: -100,
            params: serde_json::json!({"user_id": 789}),
            reason: "test".into(),
        };

        let call = adapter.action_to_api_call(&action).unwrap();
        assert_eq!(call.method, "banChatMember");
        assert_eq!(call.body["chat_id"], -100);
        assert_eq!(call.body["user_id"], 789);
    }

    #[test]
    fn test_no_action_returns_none() {
        let adapter = TelegramAdapter;
        let action = RuleAction {
            action_type: ActionType::NoAction,
            chat_id: 0,
            params: serde_json::json!({}),
            reason: "test".into(),
        };

        assert!(adapter.action_to_api_call(&action).is_none());
    }

    // ═══════════════════════════════════════════════════════════
    // Discord adapter tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_discord_extract_message() {
        let adapter = DiscordAdapter;
        let event = serde_json::json!({
            "t": "MESSAGE_CREATE",
            "d": {
                "id": "111222333",
                "channel_id": "999888777",
                "content": "!ban",
                "author": { "id": "100200300" },
                "mentions": [{ "id": "400500600" }]
            }
        });

        let ctx = adapter.extract_context(&event).unwrap();
        assert_eq!(ctx.text, "!ban");
        assert!(ctx.is_command);
        assert_eq!(ctx.command.as_deref(), Some("ban"));
        assert_eq!(ctx.reply_to_user_id, Some(400500600));
    }

    #[test]
    fn test_discord_extract_guild_member_add() {
        let adapter = DiscordAdapter;
        let event = serde_json::json!({
            "t": "GUILD_MEMBER_ADD",
            "d": {
                "guild_id": "123456",
                "user": { "id": "789012" }
            }
        });

        let ctx = adapter.extract_context(&event).unwrap();
        assert!(ctx.is_join_request);
        assert_eq!(ctx.join_request_user_id, Some(789012));
        assert_eq!(ctx.chat_id, 123456);
    }

    #[test]
    fn test_discord_ban_api_call() {
        let adapter = DiscordAdapter;
        let action = RuleAction {
            action_type: ActionType::Admin(AdminAction::Ban),
            chat_id: 999,
            params: serde_json::json!({"guild_id": 123, "user_id": 456}),
            reason: "test".into(),
        };

        let call = adapter.action_to_api_call(&action).unwrap();
        assert_eq!(call.method, "PUT");
        assert!(call.url.contains("/guilds/123/bans/456"));
    }

    #[test]
    fn test_discord_delete_message_api_call() {
        let adapter = DiscordAdapter;
        let action = RuleAction {
            action_type: ActionType::Message(MessageAction::Delete),
            chat_id: 999,
            params: serde_json::json!({"message_id": 111}),
            reason: "test".into(),
        };

        let call = adapter.action_to_api_call(&action).unwrap();
        assert_eq!(call.method, "DELETE");
        assert!(call.url.contains("/channels/999/messages/111"));
    }

    #[test]
    fn test_discord_send_message_api_call() {
        let adapter = DiscordAdapter;
        let action = RuleAction {
            action_type: ActionType::Message(MessageAction::Send),
            chat_id: 888,
            params: serde_json::json!({"text": "hello"}),
            reason: "test".into(),
        };

        let call = adapter.action_to_api_call(&action).unwrap();
        assert_eq!(call.method, "POST");
        assert!(call.url.contains("/channels/888/messages"));
        assert_eq!(call.body["content"], "hello");
    }

    // ═══════════════════════════════════════════════════════════
    // Slack adapter tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_slack_extract_message() {
        let adapter = SlackAdapter;
        let event = serde_json::json!({
            "type": "event_callback",
            "event": {
                "type": "message",
                "channel": "C12345",
                "user": "U67890",
                "text": "!ban <@U11111>",
                "ts": "1234567890.123456"
            }
        });

        let ctx = adapter.extract_context(&event).unwrap();
        assert_eq!(ctx.text, "!ban <@U11111>");
        assert!(ctx.is_command);
        assert_eq!(ctx.command.as_deref(), Some("ban"));
    }

    #[test]
    fn test_slack_filters_bot_messages() {
        let adapter = SlackAdapter;
        let event = serde_json::json!({
            "type": "event_callback",
            "event": {
                "type": "message",
                "channel": "C12345",
                "bot_id": "B99999",
                "text": "bot message"
            }
        });

        assert!(adapter.extract_context(&event).is_none());
    }

    #[test]
    fn test_slack_team_join() {
        let adapter = SlackAdapter;
        let event = serde_json::json!({
            "type": "event_callback",
            "event": {
                "type": "team_join",
                "user": { "id": "U55555" }
            }
        });

        let ctx = adapter.extract_context(&event).unwrap();
        assert!(ctx.is_join_request);
    }

    #[test]
    fn test_slack_send_message_api_call() {
        let adapter = SlackAdapter;
        let action = RuleAction {
            action_type: ActionType::Message(MessageAction::Send),
            chat_id: 0,
            params: serde_json::json!({"channel": "C123", "text": "hello"}),
            reason: "test".into(),
        };

        let call = adapter.action_to_api_call(&action).unwrap();
        assert_eq!(call.method, "chat.postMessage");
        assert_eq!(call.body["channel"], "C123");
        assert_eq!(call.body["text"], "hello");
    }

    #[test]
    fn test_slack_kick_api_call() {
        let adapter = SlackAdapter;
        let action = RuleAction {
            action_type: ActionType::Admin(AdminAction::Ban),
            chat_id: 0,
            params: serde_json::json!({"channel": "C123", "user_id": "U456"}),
            reason: "test".into(),
        };

        let call = adapter.action_to_api_call(&action).unwrap();
        assert_eq!(call.method, "conversations.kick");
        assert_eq!(call.body["user"], "U456");
    }

    // ═══════════════════════════════════════════════════════════
    // Utility tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_extract_slack_mention() {
        assert_eq!(extract_slack_mention("hello <@U12345> world"), Some("U12345"));
        assert_eq!(extract_slack_mention("cc <@U99|bob>"), Some("U99"));
        assert_eq!(extract_slack_mention("no mention"), None);
        assert_eq!(extract_slack_mention(""), None);
    }

    #[test]
    fn test_adapter_registry() {
        let registry = AdapterRegistry::new();

        assert!(registry.get("telegram").is_some());
        assert!(registry.get("discord").is_some());
        assert!(registry.get("slack").is_some());
        assert!(registry.get("matrix").is_none());

        let platforms = registry.platforms();
        assert_eq!(platforms.len(), 3);
        assert!(platforms.contains(&"telegram"));
        assert!(platforms.contains(&"discord"));
        assert!(platforms.contains(&"slack"));
    }

    // ═══════════════════════════════════════════════════════════
    // Sprint-Bot3 tests
    // ═══════════════════════════════════════════════════════════

    fn make_bot3_config() -> GroupConfig {
        GroupConfig {
            version: 1,
            bot_id_hash: "test".into(),
            join_policy: JoinApprovalPolicy::AutoApprove,
            filter_links: false,
            restrict_mentions: false,
            rate_limit_per_minute: 0,
            auto_mute_duration: 300,
            new_member_restrict_duration: 0,
            welcome_message: String::new(),
            whitelist: vec![],
            admins: vec!["100".to_string()],
            quiet_hours_start: None,
            quiet_hours_end: None,
            updated_at: 0,
            antiflood_limit: 5,
            antiflood_window: 10,
            antiflood_action: crate::types::FloodAction::default(),
            warn_limit: 3,
            warn_action: crate::types::WarnAction::default(),
            blacklist_words: vec!["spam".into()],
            blacklist_mode: crate::types::BlacklistMode::default(),
            blacklist_action: crate::types::BlacklistAction::default(),
            lock_types: vec![],
            spam_detection_enabled: true,
            spam_max_emoji: 3,
            spam_first_messages_only: 0,
        }
    }

    fn make_bot3_ctx(text: &str, sender_id: i64, config: Option<GroupConfig>) -> RuleContext {
        let update = serde_json::json!({
            "message": {
                "message_id": 100,
                "chat": {"id": -100, "type": "supergroup"},
                "from": {"id": sender_id, "is_bot": false},
                "text": text
            }
        });
        let is_command = text.starts_with('/');
        let (command, command_args) = if is_command {
            let parts: Vec<&str> = text.splitn(2, ' ').collect();
            let cmd = parts[0].trim_start_matches('/').split('@').next().unwrap_or("").to_string();
            let args = parts.get(1).map(|s| s.to_string());
            (Some(cmd), args)
        } else {
            (None, None)
        };

        RuleContext {
            message: SignedMessage {
                owner_public_key: String::new(),
                bot_id_hash: "test".into(),
                sequence: 1,
                timestamp: 1000,
                message_hash: String::new(),
                telegram_update: update,
                owner_signature: String::new(),
                platform: "telegram".into(),
            },
            text: text.to_string(),
            chat_id: -100,
            sender_id,
            is_command,
            command,
            command_args,
            reply_to_user_id: Some(42),
            reply_to_message_id: Some(99),
            is_join_request: false,
            join_request_user_id: None,
            is_callback: false,
            callback_data: None,
            group_config: config,
        }
    }

    #[test]
    fn test_admin_permission_allows_admin() {
        let rule = AdminPermissionRule;
        let config = make_bot3_config();
        // sender_id=100 is in admins
        let ctx = make_bot3_ctx("/ban", 100, Some(config));
        // Should return None (allow through to CommandRule)
        assert!(rule.evaluate(&ctx).is_none());
    }

    #[test]
    fn test_admin_permission_blocks_non_admin() {
        let rule = AdminPermissionRule;
        let config = make_bot3_config();
        // sender_id=42 is NOT in admins
        let ctx = make_bot3_ctx("/ban", 42, Some(config));
        let action = rule.evaluate(&ctx);
        assert!(action.is_some());
        assert_eq!(action.unwrap().reason, "admin_permission_denied");
    }

    #[test]
    fn test_admin_permission_ignores_non_admin_commands() {
        let rule = AdminPermissionRule;
        let config = make_bot3_config();
        // /help is not an admin command
        let ctx = make_bot3_ctx("/help", 42, Some(config));
        assert!(rule.evaluate(&ctx).is_none());
    }

    #[test]
    fn test_spam_emoji_detected() {
        let rule = SpamDetectorRule;
        let config = make_bot3_config();
        let ctx = make_bot3_ctx("Look at this 😀😀😀😀😀!", 42, Some(config));
        let action = rule.evaluate(&ctx);
        assert!(action.is_some());
        assert!(action.unwrap().reason.starts_with("spam_emoji_"));
    }

    #[test]
    fn test_spam_emoji_under_limit() {
        let rule = SpamDetectorRule;
        let config = make_bot3_config(); // max_emoji = 3
        let ctx = make_bot3_ctx("Nice 😀😀", 42, Some(config));
        assert!(rule.evaluate(&ctx).is_none());
    }

    #[test]
    fn test_spam_caps_detected() {
        let rule = SpamDetectorRule;
        let config = make_bot3_config();
        let ctx = make_bot3_ctx("BUY CRYPTO NOW AMAZING DEAL FREE MONEY", 42, Some(config));
        let action = rule.evaluate(&ctx);
        assert!(action.is_some());
        assert_eq!(action.unwrap().reason, "spam_caps");
    }

    #[test]
    fn test_spam_mixed_script_detected() {
        let rule = SpamDetectorRule;
        let config = make_bot3_config();
        // Latin + Cyrillic mixed (common phishing pattern)
        let ctx = make_bot3_ctx("Hеllo frіеnd, chеck this оut now!", 42, Some(config));
        let action = rule.evaluate(&ctx);
        assert!(action.is_some());
        assert_eq!(action.unwrap().reason, "spam_mixed_script");
    }

    #[test]
    fn test_spam_admin_exempt() {
        let rule = SpamDetectorRule;
        let config = make_bot3_config();
        // sender_id=100 is admin
        let ctx = make_bot3_ctx("BUY CRYPTO NOW AMAZING DEAL FREE MONEY 😀😀😀😀😀", 100, Some(config));
        assert!(rule.evaluate(&ctx).is_none());
    }

    #[test]
    fn test_spam_disabled() {
        let rule = SpamDetectorRule;
        let mut config = make_bot3_config();
        config.spam_detection_enabled = false;
        let ctx = make_bot3_ctx("BUY CRYPTO NOW 😀😀😀😀😀", 42, Some(config));
        assert!(rule.evaluate(&ctx).is_none());
    }

    #[test]
    fn test_command_rules() {
        let rule = CommandRule;
        let config = make_bot3_config();
        let ctx = make_bot3_ctx("/rules", 42, Some(config));
        let action = rule.evaluate(&ctx).unwrap();
        assert_eq!(action.reason, "command_rules");
        let text = action.params.get("text").unwrap().as_str().unwrap();
        assert!(text.contains("Antiflood"));
        assert!(text.contains("Blacklist"));
    }

    #[test]
    fn test_command_info() {
        let rule = CommandRule;
        let config = make_bot3_config();
        let ctx = make_bot3_ctx("/info", 42, Some(config));
        let action = rule.evaluate(&ctx).unwrap();
        assert_eq!(action.reason, "command_info");
    }

    // ═══════════════════════════════════════════════════════════
    // B4.1 E2E: 全 RuleEngine 管道测试（含 ChainCache）
    // ═══════════════════════════════════════════════════════════

    fn make_e2e_cache() -> (crate::chain_cache::ChainCache, tempfile::TempDir) {
        use ed25519_dalek::{SigningKey, Signer};
        use crate::chain_cache::ChainCache;
        use crate::types::SignedGroupConfig;

        let tmp = tempfile::TempDir::new().unwrap();
        let cache = ChainCache::with_data_dir(tmp.path().to_path_buf());

        let key = SigningKey::from_bytes(&[42u8; 32]);
        let pk_hex = hex::encode(key.verifying_key().as_bytes());

        // Register the bot first so apply_signed_config can find it
        cache.register_bot(crate::types::BotInfoCache {
            bot_id_hash: "test".into(),
            owner_public_key: key.verifying_key().to_bytes(),
            platform: "telegram".into(),
            is_active: true,
        });

        let config = make_bot3_config();
        let json = serde_json::to_string(&config).unwrap();
        let sig = key.sign(json.as_bytes());

        let signed = SignedGroupConfig {
            config,
            signature: hex::encode(sig.to_bytes()),
            signer_public_key: pk_hex,
        };

        cache.apply_signed_config("test", signed).unwrap();
        (cache, tmp)
    }

    fn make_e2e_msg(text: &str, sender_id: i64) -> SignedMessage {
        SignedMessage {
            owner_public_key: String::new(),
            bot_id_hash: "test".into(),
            sequence: 1,
            timestamp: 1000,
            message_hash: String::new(),
            telegram_update: serde_json::json!({
                "message": {
                    "message_id": 100,
                    "chat": {"id": -100, "type": "supergroup"},
                    "from": {"id": sender_id, "is_bot": false, "first_name": "Test"},
                    "text": text,
                    "reply_to_message": {
                        "message_id": 99,
                        "from": {"id": 42, "is_bot": false}
                    }
                }
            }),
            owner_signature: String::new(),
            platform: "telegram".into(),
        }
    }

    #[test]
    fn e2e_engine_ban_command_with_config() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        // Admin (100) sends /ban
        let msg = make_e2e_msg("/ban", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::Ban)));
        assert_eq!(action.reason, "command_ban");
    }

    #[test]
    fn e2e_engine_ban_blocked_non_admin() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        // Non-admin (42) sends /ban → blocked by AdminPermissionRule
        let msg = make_e2e_msg("/ban", 42);
        let action = engine.evaluate(&msg, Some(&cache));
        assert_eq!(action.reason, "admin_permission_denied");
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::Send)));
    }

    #[test]
    fn e2e_engine_blacklist_with_config() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        // "spam" is in blacklist_words
        let msg = make_e2e_msg("this is spam content", 42);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(action.reason.starts_with("blacklist_"));
    }

    #[test]
    fn e2e_engine_spam_emoji_with_config() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("Look 😀😀😀😀😀 here", 42);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(action.reason.starts_with("spam_emoji_"));
    }

    #[test]
    fn e2e_engine_spam_caps_with_config() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("BUY CRYPTO NOW AMAZING DEAL FREE MONEY", 42);
        let action = engine.evaluate(&msg, Some(&cache));
        assert_eq!(action.reason, "spam_caps");
    }

    #[test]
    fn e2e_engine_help_any_user() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        // /help is allowed for any user
        let msg = make_e2e_msg("/help", 42);
        let action = engine.evaluate(&msg, Some(&cache));
        assert_eq!(action.reason, "command_help");
    }

    #[test]
    fn e2e_engine_rules_shows_config() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/rules", 42);
        let action = engine.evaluate(&msg, Some(&cache));
        assert_eq!(action.reason, "command_rules");
        let text = action.params.get("text").unwrap().as_str().unwrap();
        assert!(text.contains("Antiflood"));
        assert!(text.contains("Blacklist"));
        assert!(text.contains("Spam detection"));
    }

    #[test]
    fn e2e_engine_normal_message_passthrough() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("hello world", 42);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::NoAction));
    }

    #[test]
    fn e2e_engine_admin_exempt_from_spam() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        // Admin (100) sends spam-like text → no action
        let msg = make_e2e_msg("BUY CRYPTO NOW 😀😀😀😀😀", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        // Should be NoAction (admin exempt)
        assert!(matches!(action.action_type, ActionType::NoAction));
    }

    // ═══════════════════════════════════════════════════════════
    // Phase A: 配置修改命令测试
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn e2e_engine_blacklist_add_command() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/blacklist badword", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::ConfigUpdate(ConfigUpdateAction::AddBlacklistWord)));
        assert_eq!(action.params["word"], "badword");
        assert_eq!(action.reason, "command_blacklist_add");
    }

    #[test]
    fn e2e_engine_blacklist_add_empty_ignored() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        // /blacklist without args → NoAction (no word provided)
        let msg = make_e2e_msg("/blacklist", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::NoAction));
    }

    #[test]
    fn e2e_engine_unblacklist_command() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/unblacklist badword", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::ConfigUpdate(ConfigUpdateAction::RemoveBlacklistWord)));
        assert_eq!(action.params["word"], "badword");
    }

    #[test]
    fn e2e_engine_blacklists_query() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/blacklists", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::Send)));
        assert_eq!(action.reason, "command_blacklists");
        let text = action.params["text"].as_str().unwrap();
        assert!(text.contains("spam")); // "spam" is in the default test config
    }

    #[test]
    fn e2e_engine_lock_command() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/lock photo", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::ConfigUpdate(ConfigUpdateAction::LockType)));
        assert_eq!(action.params["lock_type"], "photo");
    }

    #[test]
    fn e2e_engine_unlock_command() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/unlock sticker", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::ConfigUpdate(ConfigUpdateAction::UnlockType)));
        assert_eq!(action.params["lock_type"], "sticker");
    }

    #[test]
    fn e2e_engine_locks_query() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/locks", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::Send)));
        assert_eq!(action.reason, "command_locks");
    }

    #[test]
    fn e2e_engine_welcome_command() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/welcome Hello new member!", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::ConfigUpdate(ConfigUpdateAction::SetWelcome)));
        assert_eq!(action.params["text"], "Hello new member!");
    }

    #[test]
    fn e2e_engine_flood_command() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/flood 20", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::ConfigUpdate(ConfigUpdateAction::SetFloodLimit)));
        assert_eq!(action.params["limit"], 20);
    }

    #[test]
    fn e2e_engine_setwarnlimit_command() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/setwarnlimit 5", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::ConfigUpdate(ConfigUpdateAction::SetWarnLimit)));
        assert_eq!(action.params["limit"], 5);
    }

    #[test]
    fn e2e_engine_setwarnaction_command() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/setwarnaction kick", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::ConfigUpdate(ConfigUpdateAction::SetWarnAction)));
        assert_eq!(action.params["action"], "kick");
    }

    #[test]
    fn e2e_engine_setwarnaction_invalid() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/setwarnaction invalid", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        // Invalid action → usage message
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::Send)));
        assert_eq!(action.reason, "command_setwarnaction_usage");
    }

    #[test]
    fn e2e_engine_config_commands_blocked_non_admin() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        // Non-admin (42) tries config commands → blocked
        for cmd in &["/blacklist word", "/lock photo", "/welcome hi", "/flood 10", "/setwarnlimit 5"] {
            let msg = make_e2e_msg(cmd, 42);
            let action = engine.evaluate(&msg, Some(&cache));
            assert_eq!(action.reason, "admin_permission_denied",
                "Expected admin_permission_denied for cmd: {}", cmd);
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Phase 4: /perms + /poll + 新 ActionType 测试
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn e2e_engine_perms_command_valid() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/perms send_messages off", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::SetPermissions)));
        assert_eq!(action.reason, "command_perms");
        assert_eq!(action.params["permissions"]["can_send_messages"], false);
    }

    #[test]
    fn e2e_engine_perms_command_on() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/perms send_media on", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::SetPermissions)));
        assert_eq!(action.params["permissions"]["can_send_media_messages"], true);
    }

    #[test]
    fn e2e_engine_perms_unknown_permission() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/perms fly_mode on", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::Send)));
        assert_eq!(action.reason, "command_perms_unknown");
    }

    #[test]
    fn e2e_engine_perms_no_args_shows_usage() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/perms", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::Send)));
        assert_eq!(action.reason, "command_perms_usage");
    }

    #[test]
    fn e2e_engine_perms_blocked_non_admin() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/perms send_messages off", 42);
        let action = engine.evaluate(&msg, Some(&cache));
        assert_eq!(action.reason, "admin_permission_denied");
    }

    #[test]
    fn e2e_engine_poll_command_valid() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/poll Favorite color? | Red | Blue | Green", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::SendPoll)));
        assert_eq!(action.reason, "command_poll");
        assert_eq!(action.params["question"], "Favorite color?");
        let options = action.params["options"].as_array().unwrap();
        assert_eq!(options.len(), 3);
        assert_eq!(options[0]["text"], "Red");
        assert_eq!(options[1]["text"], "Blue");
        assert_eq!(options[2]["text"], "Green");
    }

    #[test]
    fn e2e_engine_poll_too_few_options() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        // Only 1 option → usage message
        let msg = make_e2e_msg("/poll Question | OnlyOne", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::Send)));
        assert_eq!(action.reason, "command_poll_usage");
    }

    #[test]
    fn e2e_engine_poll_blocked_non_admin() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/poll Q | A | B", 42);
        let action = engine.evaluate(&msg, Some(&cache));
        assert_eq!(action.reason, "admin_permission_denied");
    }

    #[test]
    fn e2e_engine_unban_command() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/unban", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::Unban)));
        assert_eq!(action.reason, "command_unban");
        assert_eq!(action.params["user_id"], 42); // reply_to_user_id from make_e2e_msg
    }

    #[test]
    fn test_new_action_types_roundtrip() {
        // Verify the new ActionType variants serialize/deserialize correctly
        let cases = vec![
            ActionType::Message(MessageAction::EditText),
            ActionType::Message(MessageAction::EditReplyMarkup),
            ActionType::Message(MessageAction::SendPoll),
            ActionType::Message(MessageAction::StopPoll),
            ActionType::Message(MessageAction::SendPhoto),
            ActionType::Message(MessageAction::SendDocument),
            ActionType::Message(MessageAction::SendMediaGroup),
            ActionType::Message(MessageAction::SendChatAction),
            ActionType::Message(MessageAction::UnpinAll),
            ActionType::Admin(AdminAction::SetMyCommands),
            ActionType::Admin(AdminAction::SetCustomTitle),
            ActionType::Admin(AdminAction::SetChatTitle),
            ActionType::Admin(AdminAction::SetChatDescription),
            ActionType::Admin(AdminAction::CreateInviteLink),
            ActionType::Admin(AdminAction::ExportInviteLink),
            ActionType::Admin(AdminAction::RevokeInviteLink),
            ActionType::Query(QueryAction::AnswerCallback),
        ];

        for action in &cases {
            let json = serde_json::to_string(action).unwrap();
            let back: ActionType = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{:?}", action), format!("{:?}", back));
        }
    }

    #[test]
    fn test_new_action_types_consensus_requirement() {
        // EditText, EditReplyMarkup, SendPoll, StopPoll require consensus
        assert!(ActionType::Message(MessageAction::EditText).requires_consensus());
        assert!(ActionType::Message(MessageAction::EditReplyMarkup).requires_consensus());
        assert!(ActionType::Message(MessageAction::SendPoll).requires_consensus());
        assert!(ActionType::Message(MessageAction::StopPoll).requires_consensus());
        // AnswerCallback is a Query → no consensus
        assert!(!ActionType::Query(QueryAction::AnswerCallback).requires_consensus());

        // Phase 6 new types
        assert!(ActionType::Message(MessageAction::SendPhoto).requires_consensus());
        assert!(ActionType::Message(MessageAction::SendDocument).requires_consensus());
        assert!(ActionType::Message(MessageAction::SendMediaGroup).requires_consensus());
        assert!(ActionType::Message(MessageAction::SendChatAction).requires_consensus());
        assert!(ActionType::Message(MessageAction::UnpinAll).requires_consensus());
        assert!(ActionType::Admin(AdminAction::SetMyCommands).requires_consensus());
        assert!(ActionType::Admin(AdminAction::SetCustomTitle).requires_consensus());
        assert!(ActionType::Admin(AdminAction::SetChatTitle).requires_consensus());
        assert!(ActionType::Admin(AdminAction::SetChatDescription).requires_consensus());
        assert!(ActionType::Admin(AdminAction::CreateInviteLink).requires_consensus());
        assert!(ActionType::Admin(AdminAction::ExportInviteLink).requires_consensus());
        assert!(ActionType::Admin(AdminAction::RevokeInviteLink).requires_consensus());
    }

    // ═══════════════════════════════════════════════════════════
    // Phase 6 Sprint 12: Bot 初始化 + 媒体
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn e2e_engine_setcommands_command() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/setcommands", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::SetMyCommands)));
        assert_eq!(action.reason, "command_setcommands");
        let cmds = action.params["commands"].as_array().unwrap();
        assert!(cmds.len() > 10, "Should register many commands");
    }

    #[test]
    fn e2e_engine_setcommands_blocked_non_admin() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/setcommands", 42);
        let action = engine.evaluate(&msg, Some(&cache));
        assert_eq!(action.reason, "admin_permission_denied");
    }

    #[test]
    fn e2e_engine_photo_command_valid() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/photo https://example.com/img.jpg My caption", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::SendPhoto)));
        assert_eq!(action.reason, "command_photo");
        assert_eq!(action.params["photo"], "https://example.com/img.jpg");
        assert_eq!(action.params["caption"], "My caption");
    }

    #[test]
    fn e2e_engine_photo_no_caption() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/photo AgACAgIAAx0CQ", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::SendPhoto)));
        assert_eq!(action.params["photo"], "AgACAgIAAx0CQ");
        assert!(action.params.get("caption").is_none() || action.params["caption"].is_null());
    }

    #[test]
    fn e2e_engine_photo_no_args_usage() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/photo", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::Send)));
        assert_eq!(action.reason, "command_photo_usage");
    }

    #[test]
    fn e2e_engine_unpinall_command() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/unpinall", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::UnpinAll)));
        assert_eq!(action.reason, "command_unpinall");
    }

    #[test]
    fn e2e_engine_unpinall_blocked_non_admin() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/unpinall", 42);
        let action = engine.evaluate(&msg, Some(&cache));
        assert_eq!(action.reason, "admin_permission_denied");
    }

    // ═══════════════════════════════════════════════════════════
    // Phase 6 Sprint 13: 邀请链接 + 管理员
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn e2e_engine_invite_command() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/invite", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::CreateInviteLink)));
        assert_eq!(action.reason, "command_invite");
    }

    #[test]
    fn e2e_engine_invite_with_name() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/invite VIP Link", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::CreateInviteLink)));
        assert_eq!(action.params["name"], "VIP Link");
    }

    #[test]
    fn e2e_engine_invite_blocked_non_admin() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/invite", 42);
        let action = engine.evaluate(&msg, Some(&cache));
        assert_eq!(action.reason, "admin_permission_denied");
    }

    #[test]
    fn e2e_engine_revoke_command_valid() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/revoke https://t.me/+abc123", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::RevokeInviteLink)));
        assert_eq!(action.reason, "command_revoke");
        assert_eq!(action.params["invite_link"], "https://t.me/+abc123");
    }

    #[test]
    fn e2e_engine_revoke_no_args_usage() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/revoke", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::Send)));
        assert_eq!(action.reason, "command_revoke_usage");
    }

    #[test]
    fn e2e_engine_exportlink_command() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/exportlink", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::ExportInviteLink)));
        assert_eq!(action.reason, "command_exportlink");
    }

    #[test]
    fn e2e_engine_title_command_valid() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/title New Group Title", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::SetChatTitle)));
        assert_eq!(action.reason, "command_title");
        assert_eq!(action.params["title"], "New Group Title");
    }

    #[test]
    fn e2e_engine_title_no_args_usage() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/title", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::Send)));
        assert_eq!(action.reason, "command_title_usage");
    }

    #[test]
    fn e2e_engine_desc_command_valid() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/desc This is the new group description", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::SetChatDescription)));
        assert_eq!(action.reason, "command_desc");
        assert_eq!(action.params["description"], "This is the new group description");
    }

    #[test]
    fn e2e_engine_desc_no_args_usage() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/desc", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::Send)));
        assert_eq!(action.reason, "command_desc_usage");
    }

    #[test]
    fn e2e_engine_admintitle_command_valid() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/admintitle Supreme Leader", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Admin(AdminAction::SetCustomTitle)));
        assert_eq!(action.reason, "command_admintitle");
        assert_eq!(action.params["custom_title"], "Supreme Leader");
        assert_eq!(action.params["user_id"], 42); // reply_to_user_id
    }

    #[test]
    fn e2e_engine_admintitle_no_args_usage() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        let msg = make_e2e_msg("/admintitle", 100);
        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::Send)));
        assert_eq!(action.reason, "command_admintitle_usage");
    }

    #[test]
    fn e2e_engine_sprint13_commands_blocked_non_admin() {
        let engine = RuleEngine::default_engine();
        let (cache, _tmp) = make_e2e_cache();

        for cmd in &[
            "/invite", "/revoke https://t.me/+x", "/exportlink",
            "/title Hi", "/desc Hi", "/admintitle Admin",
            "/setcommands", "/photo url", "/unpinall",
        ] {
            let msg = make_e2e_msg(cmd, 42);
            let action = engine.evaluate(&msg, Some(&cache));
            assert_eq!(action.reason, "admin_permission_denied",
                "Expected admin_permission_denied for cmd: {}", cmd);
        }
    }

    // ═══════════════════════════════════════════════════════════
    // WelcomeRule 媒体增强测试
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn e2e_engine_welcome_photo_media() {
        use ed25519_dalek::{SigningKey, Signer};
        use crate::chain_cache::ChainCache;
        use crate::types::SignedGroupConfig;

        let engine = RuleEngine::default_engine();

        let tmp = tempfile::TempDir::new().unwrap();
        let cache = ChainCache::with_data_dir(tmp.path().to_path_buf());

        let key = SigningKey::from_bytes(&[42u8; 32]);
        let pk_hex = hex::encode(key.verifying_key().as_bytes());

        cache.register_bot(crate::types::BotInfoCache {
            bot_id_hash: "wtest".into(),
            owner_public_key: key.verifying_key().to_bytes(),
            platform: "telegram".into(),
            is_active: true,
        });

        // Config with photo welcome message
        let mut config = make_bot3_config();
        config.bot_id_hash = "wtest".into();
        config.welcome_message = "{photo:AgACAgIAA123} Welcome {first} to {chatname}!".into();

        let json = serde_json::to_string(&config).unwrap();
        let sig = key.sign(json.as_bytes());
        let signed = SignedGroupConfig {
            config,
            signature: hex::encode(sig.to_bytes()),
            signer_public_key: pk_hex,
        };
        cache.apply_signed_config("wtest", signed).unwrap();

        // Simulate new_chat_members event
        let msg = SignedMessage {
            owner_public_key: String::new(),
            bot_id_hash: "wtest".into(),
            sequence: 1,
            timestamp: 1000,
            message_hash: String::new(),
            telegram_update: serde_json::json!({
                "message": {
                    "message_id": 99,
                    "chat": { "id": -100999, "title": "TestGroup", "type": "supergroup" },
                    "from": { "id": 55, "first_name": "Bot" },
                    "new_chat_members": [
                        { "id": 77, "first_name": "Alice", "last_name": "W", "username": "alice" }
                    ]
                }
            }),
            owner_signature: String::new(),
            platform: "telegram".into(),
        };

        let action = engine.evaluate(&msg, Some(&cache));
        assert!(matches!(action.action_type, ActionType::Message(MessageAction::SendPhoto)),
            "Expected SendPhoto, got {:?}", action.action_type);
        assert_eq!(action.reason, "welcome_photo");
        assert_eq!(action.params["photo"], "AgACAgIAA123");
        let caption = action.params["caption"].as_str().unwrap();
        assert!(caption.contains("Alice"), "Caption should contain user name");
        assert!(caption.contains("TestGroup"), "Caption should contain chat name");
    }
}
