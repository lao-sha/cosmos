use super::{PlatformAdapter, PlatformEvent};

/// Discord 平台适配器 (Agent 侧)
///
/// 将 Discord Gateway 事件 JSON 解析为统一的 PlatformEvent。
/// 实际的事件解析逻辑已在 gateway/discord.rs 中实现，
/// 此适配器用于从原始 JSON（如节点转发的事件）重新解析。
pub struct DiscordAdapter;

impl DiscordAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformAdapter for DiscordAdapter {
    fn platform_name(&self) -> &str {
        "discord"
    }

    fn parse_event(&self, raw: &serde_json::Value) -> Option<PlatformEvent> {
        parse_discord_raw(raw)
    }
}

/// 从原始 Discord 事件 JSON 解析为 PlatformEvent
///
/// 支持的事件类型（通过 `_discord_event_type` 字段或结构推断）:
/// - MESSAGE_CREATE
/// - GUILD_MEMBER_ADD / GUILD_MEMBER_REMOVE / GUILD_MEMBER_UPDATE
/// - INTERACTION_CREATE
pub fn parse_discord_raw(raw: &serde_json::Value) -> Option<PlatformEvent> {
    // 尝试从包装格式中获取事件类型
    let event_type = raw.get("_discord_event_type")
        .and_then(|v| v.as_str())
        .or_else(|| raw.get("t").and_then(|v| v.as_str()));

    // 获取事件数据（可能在 "d" 字段中，也可能是顶层）
    let data = raw.get("d").unwrap_or(raw);

    match event_type {
        Some("MESSAGE_CREATE") => parse_message_create(data, raw),
        Some("GUILD_MEMBER_ADD") => parse_member_add(data, raw),
        Some("GUILD_MEMBER_REMOVE") => parse_member_remove(data, raw),
        Some("GUILD_MEMBER_UPDATE") => parse_member_update(data, raw),
        Some("INTERACTION_CREATE") => parse_interaction_create(data, raw),
        _ => {
            // 尝试通过结构推断
            if data.get("content").is_some() && data.get("author").is_some() {
                parse_message_create(data, raw)
            } else if data.get("guild_id").is_some() && data.pointer("/data/name").is_some() {
                parse_interaction_create(data, raw)
            } else {
                None
            }
        }
    }
}

fn parse_message_create(data: &serde_json::Value, raw: &serde_json::Value) -> Option<PlatformEvent> {
    let guild_id = data.get("guild_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let channel_id = data.get("channel_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let sender_id = data.pointer("/author/id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let sender_is_bot = data.pointer("/author/bot").and_then(|v| v.as_bool()).unwrap_or(false);
    let content = data.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let message_id = data.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let is_command = content.starts_with('/') || content.starts_with('!');
    let (command, command_args) = if is_command {
        let text = &content[1..];
        let parts: Vec<&str> = text.splitn(2, ' ').collect();
        (Some(parts[0].to_string()), parts.get(1).map(|s| s.to_string()))
    } else {
        (None, None)
    };

    let reply_to_user_id = data.pointer("/referenced_message/author/id")
        .and_then(|v| v.as_str()).map(|s| s.to_string());
    let reply_to_message_id = data.pointer("/message_reference/message_id")
        .and_then(|v| v.as_str()).map(|s| s.to_string());

    Some(PlatformEvent {
        platform: "discord".to_string(),
        group_id: guild_id,
        channel_id,
        sender_id,
        sender_is_bot,
        text: content,
        message_id,
        is_command,
        command,
        command_args,
        reply_to_user_id,
        reply_to_message_id,
        is_join_event: false,
        join_user_id: None,
        is_leave_event: false,
        is_interaction: false,
        interaction_id: None,
        interaction_token: None,
        interaction_data: None,
        is_member_update: false,
        raw_event: raw.clone(),
    })
}

fn parse_member_add(data: &serde_json::Value, raw: &serde_json::Value) -> Option<PlatformEvent> {
    let guild_id = data.get("guild_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let user_id = data.pointer("/user/id").and_then(|v| v.as_str()).map(|s| s.to_string());

    Some(PlatformEvent {
        platform: "discord".to_string(),
        group_id: guild_id,
        channel_id: String::new(),
        sender_id: user_id.clone().unwrap_or_default(),
        sender_is_bot: false,
        text: String::new(),
        message_id: String::new(),
        is_command: false,
        command: None,
        command_args: None,
        reply_to_user_id: None,
        reply_to_message_id: None,
        is_join_event: true,
        join_user_id: user_id,
        is_leave_event: false,
        is_interaction: false,
        interaction_id: None,
        interaction_token: None,
        interaction_data: None,
        is_member_update: false,
        raw_event: raw.clone(),
    })
}

fn parse_member_remove(data: &serde_json::Value, raw: &serde_json::Value) -> Option<PlatformEvent> {
    let guild_id = data.get("guild_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let user_id = data.pointer("/user/id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    Some(PlatformEvent {
        platform: "discord".to_string(),
        group_id: guild_id,
        channel_id: String::new(),
        sender_id: user_id,
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
        is_leave_event: true,
        is_interaction: false,
        interaction_id: None,
        interaction_token: None,
        interaction_data: None,
        is_member_update: false,
        raw_event: raw.clone(),
    })
}

fn parse_member_update(data: &serde_json::Value, raw: &serde_json::Value) -> Option<PlatformEvent> {
    let guild_id = data.get("guild_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let user_id = data.pointer("/user/id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    Some(PlatformEvent {
        platform: "discord".to_string(),
        group_id: guild_id,
        channel_id: String::new(),
        sender_id: user_id,
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
        is_member_update: true,
        raw_event: raw.clone(),
    })
}

fn parse_interaction_create(data: &serde_json::Value, raw: &serde_json::Value) -> Option<PlatformEvent> {
    let guild_id = data.get("guild_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let channel_id = data.get("channel_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let interaction_id = data.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let interaction_token = data.get("token").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let sender_id = data.pointer("/member/user/id")
        .or_else(|| data.pointer("/user/id"))
        .and_then(|v| v.as_str())
        .unwrap_or("").to_string();

    let command_name = data.pointer("/data/name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let command_args = data.pointer("/data/options")
        .and_then(|v| v.as_array())
        .map(|opts| {
            opts.iter()
                .filter_map(|opt| {
                    let name = opt.get("name")?.as_str()?;
                    let value = opt.get("value")?;
                    let val_str = match value {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    Some(format!("{}:{}", name, val_str))
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|s| !s.is_empty());

    Some(PlatformEvent {
        platform: "discord".to_string(),
        group_id: guild_id,
        channel_id,
        sender_id,
        sender_is_bot: false,
        text: String::new(),
        message_id: String::new(),
        is_command: command_name.is_some(),
        command: command_name,
        command_args,
        reply_to_user_id: None,
        reply_to_message_id: None,
        is_join_event: false,
        join_user_id: None,
        is_leave_event: false,
        is_interaction: true,
        interaction_id: Some(interaction_id),
        interaction_token: Some(interaction_token),
        interaction_data: data.get("data").map(|d| d.to_string()),
        is_member_update: false,
        raw_event: raw.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_message_create() {
        let raw = serde_json::json!({
            "_discord_event_type": "MESSAGE_CREATE",
            "d": {
                "id": "msg_123",
                "channel_id": "chan_456",
                "guild_id": "guild_789",
                "author": {"id": "user_001", "bot": false},
                "content": "hello world"
            }
        });
        let event = parse_discord_raw(&raw).unwrap();
        assert_eq!(event.platform, "discord");
        assert_eq!(event.group_id, "guild_789");
        assert_eq!(event.text, "hello world");
        assert!(!event.is_command);
    }

    #[test]
    fn test_parse_command_with_bang() {
        let raw = serde_json::json!({
            "_discord_event_type": "MESSAGE_CREATE",
            "d": {
                "id": "msg_124",
                "channel_id": "chan_456",
                "guild_id": "guild_789",
                "author": {"id": "user_001"},
                "content": "!ban bad_user"
            }
        });
        let event = parse_discord_raw(&raw).unwrap();
        assert!(event.is_command);
        assert_eq!(event.command.as_deref(), Some("ban"));
        assert_eq!(event.command_args.as_deref(), Some("bad_user"));
    }

    #[test]
    fn test_parse_interaction() {
        let raw = serde_json::json!({
            "_discord_event_type": "INTERACTION_CREATE",
            "d": {
                "id": "inter_001",
                "token": "tok_abc",
                "guild_id": "guild_789",
                "channel_id": "chan_456",
                "member": {"user": {"id": "user_001"}},
                "data": {
                    "name": "warn",
                    "options": [{"name": "user", "value": "target", "type": 6}]
                }
            }
        });
        let event = parse_discord_raw(&raw).unwrap();
        assert!(event.is_interaction);
        assert_eq!(event.command.as_deref(), Some("warn"));
        assert_eq!(event.interaction_id.as_deref(), Some("inter_001"));
    }

    #[test]
    fn test_parse_member_add() {
        let raw = serde_json::json!({
            "_discord_event_type": "GUILD_MEMBER_ADD",
            "d": {
                "guild_id": "guild_789",
                "user": {"id": "new_user"}
            }
        });
        let event = parse_discord_raw(&raw).unwrap();
        assert!(event.is_join_event);
        assert_eq!(event.join_user_id.as_deref(), Some("new_user"));
    }

    #[test]
    fn test_parse_member_remove() {
        let raw = serde_json::json!({
            "_discord_event_type": "GUILD_MEMBER_REMOVE",
            "d": {
                "guild_id": "guild_789",
                "user": {"id": "leaving_user"}
            }
        });
        let event = parse_discord_raw(&raw).unwrap();
        assert!(event.is_leave_event);
        assert_eq!(event.sender_id, "leaving_user");
    }

    #[test]
    fn test_parse_infer_message_from_structure() {
        let raw = serde_json::json!({
            "id": "msg_200",
            "channel_id": "chan_1",
            "guild_id": "guild_1",
            "author": {"id": "u1", "bot": true},
            "content": "bot says hi"
        });
        let event = parse_discord_raw(&raw).unwrap();
        assert_eq!(event.platform, "discord");
        assert!(event.sender_is_bot);
        assert_eq!(event.text, "bot says hi");
    }

    #[test]
    fn test_adapter_trait() {
        let adapter = DiscordAdapter::new();
        assert_eq!(adapter.platform_name(), "discord");

        let raw = serde_json::json!({
            "_discord_event_type": "MESSAGE_CREATE",
            "d": {
                "id": "m1", "channel_id": "c1", "guild_id": "g1",
                "author": {"id": "u1"}, "content": "test"
            }
        });
        let event = adapter.parse_event(&raw).unwrap();
        assert_eq!(event.platform, "discord");
    }
}
