use serde::{Deserialize, Serialize};
use tracing::{info, debug};

use crate::types::{ActionType, MessageAction, AdminAction, QueryAction, SignedMessage, GroupConfig, JoinApprovalPolicy};
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
            Box::new(CommandRule),
            Box::new(AntifloodRule),
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
                        "text": "📋 Available commands:\n/ban - Ban user\n/unban - Unban user\n/mute <sec> - Mute user\n/unmute - Unmute user\n/kick - Kick user\n/warn - Warn user\n/unwarn - Remove warning\n/warns - View warnings\n/pin - Pin message\n/del - Delete message\n/help - This message",
                    }),
                    reason: "command_help".into(),
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

        Some(RuleAction {
            action_type: ActionType::Message(MessageAction::Send),
            chat_id: ctx.chat_id,
            params: serde_json::json!({ "text": text }),
            reason: "welcome_message".into(),
        })
    }
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

    fn extract_context(&self, raw_json: &serde_json::Value) -> Option<RuleContext> {
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
                let message_id = data.get("id").and_then(|v| v.as_str()).unwrap_or("0");

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

                let channel = event.get("channel")?.as_str()?;
                let user = event.get("user").and_then(|v| v.as_str()).unwrap_or("unknown");
                let text = event.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let ts = event.get("ts").and_then(|v| v.as_str()).unwrap_or("0");

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
                let thread_ts = event.get("thread_ts").and_then(|v| v.as_str());

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
}
