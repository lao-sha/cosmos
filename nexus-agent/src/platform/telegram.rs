use super::{PlatformAdapter, PlatformEvent};

/// Telegram 平台适配器
///
/// 将 Telegram Update JSON 解析为统一的 PlatformEvent。
pub struct TelegramAdapter;

impl TelegramAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformAdapter for TelegramAdapter {
    fn platform_name(&self) -> &str {
        "telegram"
    }

    fn parse_event(&self, raw: &serde_json::Value) -> Option<PlatformEvent> {
        parse_telegram_update(raw)
    }
}

/// 将 Telegram Update JSON 解析为 PlatformEvent
///
/// 支持的 Update 类型:
/// - message (普通消息 + 命令)
/// - edited_message
/// - callback_query (按钮回调)
/// - chat_join_request (入群申请)
/// - chat_member (成员变动)
pub fn parse_telegram_update(raw: &serde_json::Value) -> Option<PlatformEvent> {
    let mut event = PlatformEvent {
        platform: "telegram".to_string(),
        group_id: String::new(),
        channel_id: String::new(),
        sender_id: String::new(),
        sender_is_bot: false,
        text: String::new(),
        message_id: String::new(),
        is_command: false,
        command: None,
        command_args: None,
        reply_to_user_id: None,
        reply_to_message_id: None,
        is_join_event: false,
        join_user_id: None,
        is_leave_event: false,
        is_interaction: false,
        interaction_id: None,
        interaction_token: None,
        interaction_data: None,
        is_member_update: false,
        raw_event: raw.clone(),
    };

    // ── callback_query ──
    if let Some(cq) = raw.get("callback_query") {
        event.is_interaction = true;
        event.interaction_id = cq.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        event.interaction_data = cq.get("data")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        event.sender_id = cq.pointer("/from/id")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string())
            .unwrap_or_default();
        event.sender_is_bot = cq.pointer("/from/is_bot")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        event.group_id = cq.pointer("/message/chat/id")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string())
            .unwrap_or_default();
        event.message_id = cq.pointer("/message/message_id")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string())
            .unwrap_or_default();
        return Some(event);
    }

    // ── chat_join_request ──
    if let Some(jreq) = raw.get("chat_join_request") {
        event.is_join_event = true;
        event.group_id = jreq.pointer("/chat/id")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string())
            .unwrap_or_default();
        event.join_user_id = jreq.pointer("/from/id")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string());
        event.sender_id = event.join_user_id.clone().unwrap_or_default();
        return Some(event);
    }

    // ── chat_member (成员变动) ──
    if raw.get("chat_member").is_some() {
        event.is_member_update = true;
        event.group_id = raw.pointer("/chat_member/chat/id")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string())
            .unwrap_or_default();
        return Some(event);
    }

    // ── message / edited_message ──
    let msg = raw.get("message")
        .or_else(|| raw.get("edited_message"));

    if let Some(msg) = msg {
        event.group_id = msg.pointer("/chat/id")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string())
            .unwrap_or_default();
        event.sender_id = msg.pointer("/from/id")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string())
            .unwrap_or_default();
        event.sender_is_bot = msg.pointer("/from/is_bot")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        event.message_id = msg.get("message_id")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string())
            .unwrap_or_default();
        event.text = msg.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 解析命令
        if event.text.starts_with('/') {
            event.is_command = true;
            let parts: Vec<&str> = event.text.splitn(2, ' ').collect();
            // 去掉 @botname 后缀
            let cmd = parts[0].split('@').next().unwrap_or(parts[0]);
            event.command = Some(cmd[1..].to_string()); // 去掉 '/'
            event.command_args = parts.get(1).map(|s| s.to_string());
        }

        // 解析 reply_to_message
        if let Some(reply) = msg.get("reply_to_message") {
            event.reply_to_user_id = reply.pointer("/from/id")
                .and_then(|v| v.as_i64())
                .map(|id| id.to_string());
            event.reply_to_message_id = reply.get("message_id")
                .and_then(|v| v.as_i64())
                .map(|id| id.to_string());
        }

        // 检查 new_chat_members (群内加人事件)
        if msg.get("new_chat_members").is_some() {
            event.is_join_event = true;
            // new_chat_members 是数组，取第一个
            event.join_user_id = msg.pointer("/new_chat_members/0/id")
                .and_then(|v| v.as_i64())
                .map(|id| id.to_string());
        }

        // 检查 left_chat_member (离群事件)
        if msg.get("left_chat_member").is_some() {
            event.is_leave_event = true;
        }

        return Some(event);
    }

    // 无法识别的 Update 类型
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_message() {
        let raw = serde_json::json!({
            "update_id": 1,
            "message": {
                "message_id": 100,
                "from": {"id": 42, "is_bot": false, "first_name": "Test"},
                "chat": {"id": -100123, "type": "supergroup"},
                "text": "hello world",
                "date": 1234567890
            }
        });

        let event = parse_telegram_update(&raw).unwrap();
        assert_eq!(event.platform, "telegram");
        assert_eq!(event.group_id, "-100123");
        assert_eq!(event.sender_id, "42");
        assert!(!event.sender_is_bot);
        assert_eq!(event.text, "hello world");
        assert_eq!(event.message_id, "100");
        assert!(!event.is_command);
        assert!(event.command.is_none());
    }

    #[test]
    fn test_parse_command() {
        let raw = serde_json::json!({
            "update_id": 2,
            "message": {
                "message_id": 101,
                "from": {"id": 42, "is_bot": false},
                "chat": {"id": -100123, "type": "supergroup"},
                "text": "/ban @user reason here"
            }
        });

        let event = parse_telegram_update(&raw).unwrap();
        assert!(event.is_command);
        assert_eq!(event.command.as_deref(), Some("ban"));
        assert_eq!(event.command_args.as_deref(), Some("@user reason here"));
    }

    #[test]
    fn test_parse_command_with_bot_mention() {
        let raw = serde_json::json!({
            "update_id": 3,
            "message": {
                "message_id": 102,
                "from": {"id": 42, "is_bot": false},
                "chat": {"id": -100123, "type": "supergroup"},
                "text": "/help@my_bot"
            }
        });

        let event = parse_telegram_update(&raw).unwrap();
        assert!(event.is_command);
        assert_eq!(event.command.as_deref(), Some("help"));
    }

    #[test]
    fn test_parse_reply() {
        let raw = serde_json::json!({
            "update_id": 4,
            "message": {
                "message_id": 103,
                "from": {"id": 42, "is_bot": false},
                "chat": {"id": -100123, "type": "supergroup"},
                "text": "/ban",
                "reply_to_message": {
                    "message_id": 99,
                    "from": {"id": 789, "is_bot": false}
                }
            }
        });

        let event = parse_telegram_update(&raw).unwrap();
        assert_eq!(event.reply_to_user_id.as_deref(), Some("789"));
        assert_eq!(event.reply_to_message_id.as_deref(), Some("99"));
    }

    #[test]
    fn test_parse_callback_query() {
        let raw = serde_json::json!({
            "update_id": 5,
            "callback_query": {
                "id": "cb_123",
                "from": {"id": 42, "is_bot": false},
                "message": {
                    "message_id": 100,
                    "chat": {"id": -100123, "type": "supergroup"}
                },
                "data": "approve_join"
            }
        });

        let event = parse_telegram_update(&raw).unwrap();
        assert!(event.is_interaction);
        assert_eq!(event.interaction_id.as_deref(), Some("cb_123"));
        assert_eq!(event.interaction_data.as_deref(), Some("approve_join"));
        assert_eq!(event.group_id, "-100123");
        assert_eq!(event.sender_id, "42");
    }

    #[test]
    fn test_parse_join_request() {
        let raw = serde_json::json!({
            "update_id": 6,
            "chat_join_request": {
                "chat": {"id": -100123, "type": "supergroup"},
                "from": {"id": 456, "is_bot": false}
            }
        });

        let event = parse_telegram_update(&raw).unwrap();
        assert!(event.is_join_event);
        assert_eq!(event.join_user_id.as_deref(), Some("456"));
        assert_eq!(event.group_id, "-100123");
    }

    #[test]
    fn test_parse_chat_member() {
        let raw = serde_json::json!({
            "update_id": 7,
            "chat_member": {
                "chat": {"id": -100123, "type": "supergroup"},
                "from": {"id": 42, "is_bot": false},
                "new_chat_member": {"status": "left"}
            }
        });

        let event = parse_telegram_update(&raw).unwrap();
        assert!(event.is_member_update);
    }

    #[test]
    fn test_parse_new_chat_members() {
        let raw = serde_json::json!({
            "update_id": 8,
            "message": {
                "message_id": 200,
                "from": {"id": 42, "is_bot": false},
                "chat": {"id": -100123, "type": "supergroup"},
                "new_chat_members": [{"id": 999, "is_bot": false, "first_name": "New"}]
            }
        });

        let event = parse_telegram_update(&raw).unwrap();
        assert!(event.is_join_event);
        assert_eq!(event.join_user_id.as_deref(), Some("999"));
    }

    #[test]
    fn test_unknown_update_returns_none() {
        let raw = serde_json::json!({
            "update_id": 99,
            "unknown_field": {}
        });

        assert!(parse_telegram_update(&raw).is_none());
    }

    #[test]
    fn test_chat_id_i64() {
        let raw = serde_json::json!({
            "update_id": 1,
            "message": {
                "message_id": 1,
                "from": {"id": 1, "is_bot": false},
                "chat": {"id": -100123456789_i64, "type": "supergroup"},
                "text": "test"
            }
        });

        let event = parse_telegram_update(&raw).unwrap();
        assert_eq!(event.chat_id_i64(), -100123456789);
        assert_eq!(event.sender_id_i64(), 1);
    }
}
