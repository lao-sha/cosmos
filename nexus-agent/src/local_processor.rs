use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug};

use crate::group_config::{
    GroupConfig, FloodAction, BlacklistMode, BlacklistAction, LockType,
};
use crate::local_store::LocalStore;
use crate::group_config::ConfigStore;

/// 本地快速路径动作
///
/// Agent 直接执行，不走 Node 共识。
/// 用于高频检测类操作（防刷屏/黑名单/锁定/欢迎/重复检测）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAction {
    /// 动作类型
    pub action: LocalActionType,
    /// 目标 chat_id
    pub chat_id: i64,
    /// 动作参数
    pub params: serde_json::Value,
    /// 触发原因
    pub reason: String,
}

/// 本地动作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocalActionType {
    /// 删除消息
    DeleteMessage,
    /// 禁言用户
    MuteUser,
    /// 封禁用户
    BanUser,
    /// 踢出用户
    KickUser,
    /// 发送消息
    SendMessage,
}

/// 本地快速路径处理器
///
/// 在 Webhook 收到消息后、转发到 Node 之前执行。
/// 处理不需要共识的高频操作。
///
/// 参考:
///   - FallenRobot: 各模块独立处理 (antiflood → bans → locks → blacklist → welcome)
///   - tg-spam: SpamFilter.OnMessage() 检测链
pub struct LocalProcessor;

impl LocalProcessor {
    /// 处理一条 Update，返回需要立即执行的动作列表
    pub fn process(
        update: &serde_json::Value,
        config: Option<&GroupConfig>,
        local_store: &LocalStore,
    ) -> Vec<LocalAction> {
        let mut actions = vec![];

        let config = match config {
            Some(c) => c,
            None => return actions,
        };

        let msg = match update.get("message") {
            Some(m) => m,
            None => return actions,
        };

        let chat_id = msg.pointer("/chat/id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let sender_id = msg.pointer("/from/id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let text = msg.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let message_id = msg.get("message_id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        if chat_id == 0 || sender_id == 0 {
            return actions;
        }

        // 跳过管理员和白名单
        let sender_str = sender_id.to_string();
        let is_exempt = config.whitelist.contains(&sender_str)
            || config.admins.contains(&sender_str);

        // 跳过命令消息（由 Node 侧 CommandRule 处理）
        let is_command = text.starts_with('/');

        // 检测是否为新成员加入
        let is_new_member = msg.get("new_chat_members").is_some();

        // ═══ 1. 欢迎消息（最先检查，不受 exempt 限制）═══
        if is_new_member {
            if let Some(action) = Self::check_welcome(update, config, chat_id) {
                actions.push(action);
            }
        }

        // 以下检查仅对非豁免、非命令的普通消息
        if !is_exempt && !is_command && !is_new_member {
            // ═══ 2. 防刷屏 ═══
            if let Some(action) = Self::check_antiflood(
                config, local_store, chat_id, sender_id, message_id,
            ) {
                actions.push(action);
                // 防刷屏触发后跳过其他检查
                return actions;
            }

            // ═══ 3. 黑名单词过滤 ═══
            if let Some(action) = Self::check_blacklist(
                config, chat_id, sender_id, message_id, text,
            ) {
                actions.push(action);
                return actions;
            }

            // ═══ 4. 消息类型锁定 ═══
            if let Some(action) = Self::check_lock(config, msg, chat_id, message_id) {
                actions.push(action);
                return actions;
            }

            // ═══ 5. 重复消息检测 ═══
            if config.spam_detection_enabled && !text.is_empty() {
                if let Some(action) = Self::check_duplicate(
                    config, local_store, chat_id, sender_id, message_id, text,
                ) {
                    actions.push(action);
                    return actions;
                }
            }

            // ═══ 6. Emoji 过多检测 ═══
            if config.spam_detection_enabled && config.spam_max_emoji > 0 && !text.is_empty() {
                if let Some(action) = Self::check_emoji(
                    config, chat_id, sender_id, message_id, text,
                ) {
                    actions.push(action);
                    return actions;
                }
            }
        }

        actions
    }

    /// 防刷屏检查
    ///
    /// 参考: FallenRobot/modules/antiflood.py
    ///   - CHAT_FLOOD[chat_id][user_id] += 1 每条消息
    ///   - 超限 → ban/kick/mute/tban/tmute
    fn check_antiflood(
        config: &GroupConfig,
        local_store: &LocalStore,
        chat_id: i64,
        sender_id: i64,
        message_id: i64,
    ) -> Option<LocalAction> {
        if config.antiflood_limit == 0 {
            return None;
        }

        let triggered = local_store.check_flood(
            chat_id, sender_id,
            config.antiflood_limit,
            config.antiflood_window,
        );

        if !triggered {
            return None;
        }

        debug!(chat_id, sender_id, "防刷屏触发");

        match &config.antiflood_action {
            FloodAction::Mute => Some(LocalAction {
                action: LocalActionType::MuteUser,
                chat_id,
                params: serde_json::json!({
                    "user_id": sender_id,
                    "message_id": message_id,
                    "duration_seconds": config.auto_mute_duration,
                }),
                reason: "antiflood_mute".into(),
            }),
            FloodAction::Ban => Some(LocalAction {
                action: LocalActionType::BanUser,
                chat_id,
                params: serde_json::json!({
                    "user_id": sender_id,
                    "message_id": message_id,
                }),
                reason: "antiflood_ban".into(),
            }),
            FloodAction::Kick => Some(LocalAction {
                action: LocalActionType::KickUser,
                chat_id,
                params: serde_json::json!({
                    "user_id": sender_id,
                    "message_id": message_id,
                }),
                reason: "antiflood_kick".into(),
            }),
            FloodAction::DeleteOnly => Some(LocalAction {
                action: LocalActionType::DeleteMessage,
                chat_id,
                params: serde_json::json!({ "message_id": message_id }),
                reason: "antiflood_delete".into(),
            }),
        }
    }

    /// 黑名单词过滤
    ///
    /// 参考: FallenRobot/modules/blacklist.py — 正则/包含/精确匹配
    fn check_blacklist(
        config: &GroupConfig,
        chat_id: i64,
        sender_id: i64,
        message_id: i64,
        text: &str,
    ) -> Option<LocalAction> {
        if config.blacklist_words.is_empty() || text.is_empty() {
            return None;
        }

        let text_lower = text.to_lowercase();
        let matched = match &config.blacklist_mode {
            BlacklistMode::Exact => {
                config.blacklist_words.iter().any(|w| text_lower == w.to_lowercase())
            }
            BlacklistMode::Contains => {
                config.blacklist_words.iter().any(|w| text_lower.contains(&w.to_lowercase()))
            }
            BlacklistMode::Regex => {
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

        debug!(chat_id, sender_id, "黑名单词命中");

        // 始终删除消息
        let mut action = LocalAction {
            action: LocalActionType::DeleteMessage,
            chat_id,
            params: serde_json::json!({ "message_id": message_id }),
            reason: "blacklist_hit".into(),
        };

        // 根据 blacklist_action 升级动作
        match &config.blacklist_action {
            BlacklistAction::Delete => {} // 仅删除
            BlacklistAction::DeleteAndWarn => {
                action.params = serde_json::json!({
                    "message_id": message_id,
                    "user_id": sender_id,
                    "also_warn": true,
                });
            }
            BlacklistAction::DeleteAndMute => {
                action.action = LocalActionType::MuteUser;
                action.params = serde_json::json!({
                    "message_id": message_id,
                    "user_id": sender_id,
                    "duration_seconds": config.auto_mute_duration,
                    "also_delete": true,
                });
                action.reason = "blacklist_mute".into();
            }
            BlacklistAction::DeleteAndBan => {
                action.action = LocalActionType::BanUser;
                action.params = serde_json::json!({
                    "message_id": message_id,
                    "user_id": sender_id,
                    "also_delete": true,
                });
                action.reason = "blacklist_ban".into();
            }
        }

        Some(action)
    }

    /// 消息类型锁定检查
    ///
    /// 参考: FallenRobot/modules/locks.py — 25 种 LOCK_TYPES
    fn check_lock(
        config: &GroupConfig,
        msg: &serde_json::Value,
        chat_id: i64,
        message_id: i64,
    ) -> Option<LocalAction> {
        if config.lock_types.is_empty() {
            return None;
        }

        let detected = if msg.get("photo").is_some() {
            Some(LockType::Photo)
        } else if msg.get("video").is_some() {
            Some(LockType::Video)
        } else if msg.get("audio").is_some() {
            Some(LockType::Audio)
        } else if msg.get("document").is_some() {
            Some(LockType::Document)
        } else if msg.get("sticker").is_some() {
            Some(LockType::Sticker)
        } else if msg.get("animation").is_some() {
            Some(LockType::Gif)
        } else if msg.get("voice").is_some() {
            Some(LockType::Voice)
        } else if msg.get("contact").is_some() {
            Some(LockType::Contact)
        } else if msg.get("location").is_some() {
            Some(LockType::Location)
        } else if msg.get("poll").is_some() {
            Some(LockType::Poll)
        } else if msg.get("game").is_some() {
            Some(LockType::Game)
        } else if msg.get("forward_from").is_some() || msg.get("forward_from_chat").is_some() {
            Some(LockType::Forward)
        } else if msg.get("via_bot").is_some() {
            Some(LockType::Inline)
        } else {
            // URL 检查（在文本中）
            let text = msg.get("text").and_then(|v| v.as_str()).unwrap_or("");
            if (text.contains("http://") || text.contains("https://") || text.contains("t.me/"))
                && config.lock_types.contains(&LockType::Url)
            {
                Some(LockType::Url)
            } else {
                None
            }
        };

        let lock_type = detected?;
        if !config.lock_types.contains(&lock_type) {
            return None;
        }

        debug!(chat_id, ?lock_type, "锁定类型命中");

        Some(LocalAction {
            action: LocalActionType::DeleteMessage,
            chat_id,
            params: serde_json::json!({ "message_id": message_id }),
            reason: format!("lock_{:?}", lock_type),
        })
    }

    /// 欢迎消息
    ///
    /// 参考: FallenRobot/modules/welcome.py — 变量替换 {first}/{chatname}/{id}
    fn check_welcome(
        update: &serde_json::Value,
        config: &GroupConfig,
        chat_id: i64,
    ) -> Option<LocalAction> {
        if config.welcome_message.is_empty() {
            return None;
        }

        let new_members = update.pointer("/message/new_chat_members")?;
        let members = new_members.as_array()?;
        if members.is_empty() {
            return None;
        }

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

        Some(LocalAction {
            action: LocalActionType::SendMessage,
            chat_id,
            params: serde_json::json!({ "text": text }),
            reason: "welcome_message".into(),
        })
    }

    /// 重复消息检测
    ///
    /// 参考: tg-spam/lib/tgspam/duplicate.go — Window + Threshold
    fn check_duplicate(
        config: &GroupConfig,
        local_store: &LocalStore,
        chat_id: i64,
        sender_id: i64,
        message_id: i64,
        text: &str,
    ) -> Option<LocalAction> {
        // 使用 60 秒窗口，3 次重复触发
        let dup_count = local_store.record_message(chat_id, sender_id, text, 60);

        if dup_count < 3 {
            return None;
        }

        debug!(chat_id, sender_id, dup_count, "重复消息检测触发");

        Some(LocalAction {
            action: LocalActionType::DeleteMessage,
            chat_id,
            params: serde_json::json!({
                "message_id": message_id,
                "user_id": sender_id,
            }),
            reason: format!("duplicate_message_{}", dup_count),
        })
    }

    /// Emoji 过多检测
    ///
    /// 参考: tg-spam/lib/tgspam/detector.go — MaxAllowedEmoji
    fn check_emoji(
        config: &GroupConfig,
        chat_id: i64,
        sender_id: i64,
        message_id: i64,
        text: &str,
    ) -> Option<LocalAction> {
        let emoji_count = text.chars()
            .filter(|c| is_emoji(*c))
            .count() as u8;

        if emoji_count <= config.spam_max_emoji {
            return None;
        }

        debug!(chat_id, sender_id, emoji_count, "emoji 过多");

        Some(LocalAction {
            action: LocalActionType::DeleteMessage,
            chat_id,
            params: serde_json::json!({
                "message_id": message_id,
                "user_id": sender_id,
            }),
            reason: format!("spam_emoji_{}", emoji_count),
        })
    }
}

/// 简单 emoji 检测
/// 覆盖常见 Unicode emoji 范围
fn is_emoji(c: char) -> bool {
    let cp = c as u32;
    // Emoticons
    (0x1F600..=0x1F64F).contains(&cp) ||
    // Misc Symbols and Pictographs
    (0x1F300..=0x1F5FF).contains(&cp) ||
    // Transport and Map
    (0x1F680..=0x1F6FF).contains(&cp) ||
    // Misc Symbols
    (0x2600..=0x26FF).contains(&cp) ||
    // Dingbats
    (0x2700..=0x27BF).contains(&cp) ||
    // Supplemental Symbols
    (0x1F900..=0x1F9FF).contains(&cp) ||
    // Flags
    (0x1F1E0..=0x1F1FF).contains(&cp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group_config::*;
    use crate::local_store::LocalStore;

    fn make_config() -> GroupConfig {
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
            admins: vec![],
            quiet_hours_start: None,
            quiet_hours_end: None,
            updated_at: 0,
            antiflood_limit: 0,
            antiflood_window: 10,
            antiflood_action: FloodAction::Mute,
            warn_limit: 3,
            warn_action: WarnAction::Ban,
            blacklist_words: vec![],
            blacklist_mode: BlacklistMode::Contains,
            blacklist_action: BlacklistAction::Delete,
            lock_types: vec![],
            spam_detection_enabled: false,
            spam_max_emoji: 0,
            spam_first_messages_only: 0,
        }
    }

    fn make_update(text: &str) -> serde_json::Value {
        serde_json::json!({
            "update_id": 1,
            "message": {
                "message_id": 100,
                "chat": { "id": -100, "type": "supergroup", "title": "TestGroup" },
                "from": { "id": 42, "is_bot": false, "first_name": "Alice" },
                "text": text
            }
        })
    }

    fn make_photo_update() -> serde_json::Value {
        serde_json::json!({
            "update_id": 1,
            "message": {
                "message_id": 100,
                "chat": { "id": -100, "type": "supergroup" },
                "from": { "id": 42, "is_bot": false },
                "photo": [{"file_id": "abc", "width": 100, "height": 100}]
            }
        })
    }

    fn make_new_member_update() -> serde_json::Value {
        serde_json::json!({
            "update_id": 1,
            "message": {
                "message_id": 100,
                "chat": { "id": -100, "type": "supergroup", "title": "TestGroup" },
                "from": { "id": 42, "is_bot": false },
                "new_chat_members": [{
                    "id": 999,
                    "first_name": "Bob",
                    "last_name": "Smith",
                    "username": "bobsmith"
                }]
            }
        })
    }

    #[test]
    fn test_no_config_no_actions() {
        let store = LocalStore::new();
        let update = make_update("hello");
        let actions = LocalProcessor::process(&update, None, &store);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_normal_message_no_actions() {
        let store = LocalStore::new();
        let config = make_config();
        let update = make_update("hello world");
        let actions = LocalProcessor::process(&update, Some(&config), &store);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_antiflood_triggers() {
        let store = LocalStore::new();
        let mut config = make_config();
        config.antiflood_limit = 3;
        config.antiflood_window = 10;
        config.antiflood_action = FloodAction::Mute;

        let update = make_update("msg");
        // 前 3 条不触发
        for _ in 0..3 {
            let actions = LocalProcessor::process(&update, Some(&config), &store);
            assert!(actions.is_empty());
        }
        // 第 4 条触发
        let actions = LocalProcessor::process(&update, Some(&config), &store);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].reason, "antiflood_mute");
    }

    #[test]
    fn test_antiflood_admin_exempt() {
        let store = LocalStore::new();
        let mut config = make_config();
        config.antiflood_limit = 1;
        config.admins = vec!["42".to_string()]; // sender_id = 42

        let update = make_update("msg");
        for _ in 0..5 {
            let actions = LocalProcessor::process(&update, Some(&config), &store);
            assert!(actions.is_empty());
        }
    }

    #[test]
    fn test_blacklist_contains_match() {
        let store = LocalStore::new();
        let mut config = make_config();
        config.blacklist_words = vec!["spam".to_string(), "scam".to_string()];
        config.blacklist_action = BlacklistAction::Delete;

        let update = make_update("this is a spam message");
        let actions = LocalProcessor::process(&update, Some(&config), &store);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].reason, "blacklist_hit");
    }

    #[test]
    fn test_blacklist_no_match() {
        let store = LocalStore::new();
        let mut config = make_config();
        config.blacklist_words = vec!["spam".to_string()];

        let update = make_update("hello world");
        let actions = LocalProcessor::process(&update, Some(&config), &store);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_lock_photo() {
        let store = LocalStore::new();
        let mut config = make_config();
        config.lock_types = vec![LockType::Photo];

        let update = make_photo_update();
        let actions = LocalProcessor::process(&update, Some(&config), &store);
        assert_eq!(actions.len(), 1);
        assert!(actions[0].reason.contains("Photo"));
    }

    #[test]
    fn test_lock_photo_not_locked() {
        let store = LocalStore::new();
        let mut config = make_config();
        config.lock_types = vec![LockType::Video]; // only video locked

        let update = make_photo_update();
        let actions = LocalProcessor::process(&update, Some(&config), &store);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_welcome_message() {
        let store = LocalStore::new();
        let mut config = make_config();
        config.welcome_message = "Welcome {first} to {chatname}!".to_string();

        let update = make_new_member_update();
        let actions = LocalProcessor::process(&update, Some(&config), &store);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].reason, "welcome_message");
        let text = actions[0].params.get("text").unwrap().as_str().unwrap();
        assert_eq!(text, "Welcome Bob to TestGroup!");
    }

    #[test]
    fn test_welcome_no_message_configured() {
        let store = LocalStore::new();
        let config = make_config(); // welcome_message is empty

        let update = make_new_member_update();
        let actions = LocalProcessor::process(&update, Some(&config), &store);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_emoji_spam() {
        let store = LocalStore::new();
        let mut config = make_config();
        config.spam_detection_enabled = true;
        config.spam_max_emoji = 3;

        let update = make_update("Check this 😀😀😀😀😀 out!");
        let actions = LocalProcessor::process(&update, Some(&config), &store);
        assert_eq!(actions.len(), 1);
        assert!(actions[0].reason.starts_with("spam_emoji_"));
    }

    #[test]
    fn test_command_skipped() {
        let store = LocalStore::new();
        let mut config = make_config();
        config.blacklist_words = vec!["ban".to_string()];

        let update = make_update("/ban @user");
        let actions = LocalProcessor::process(&update, Some(&config), &store);
        // Commands are skipped by LocalProcessor
        assert!(actions.is_empty());
    }
}
