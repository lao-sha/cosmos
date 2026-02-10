use crate::types::{SignedMessage, ActionType, AdminAction, MessageAction};
use crate::chain_cache::ChainCache;
use crate::rule_engine::RuleContext;
use super::NodePlatformAdapter;

/// Discord Node 侧适配器
///
/// 从 SignedMessage 中提取 Discord 事件 JSON 的字段，
/// 构建 RuleContext 和 determine_action。
pub struct DiscordNodeAdapter;

impl NodePlatformAdapter for DiscordNodeAdapter {
    fn platform_name(&self) -> &str {
        "discord"
    }

    fn build_context(
        &self,
        message: &SignedMessage,
        _chain_cache: Option<&ChainCache>,
    ) -> Option<RuleContext> {
        let raw = &message.platform_event;

        // 尝试获取事件数据（可能在 "d" 字段中或顶层）
        let data = raw.get("d").unwrap_or(raw);
        let event_type = raw.get("_discord_event_type")
            .or_else(|| raw.get("t"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match event_type {
            "MESSAGE_CREATE" | "" if data.get("content").is_some() => {
                self.build_message_context(message, data)
            }
            "GUILD_MEMBER_ADD" => {
                self.build_member_add_context(message, data)
            }
            "GUILD_MEMBER_REMOVE" => {
                self.build_member_remove_context(message, data)
            }
            "INTERACTION_CREATE" => {
                self.build_interaction_context(message, data)
            }
            _ => None,
        }
    }

    fn determine_action(
        &self,
        message: &SignedMessage,
    ) -> (ActionType, i64, serde_json::Value) {
        let raw = &message.platform_event;
        let data = raw.get("d").unwrap_or(raw);
        let event_type = raw.get("_discord_event_type")
            .or_else(|| raw.get("t"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let guild_id = data.get("guild_id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        match event_type {
            "GUILD_MEMBER_ADD" => {
                let user_id = data.pointer("/user/id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0");
                (
                    ActionType::Admin(AdminAction::ApproveJoinRequest),
                    guild_id,
                    serde_json::json!({ "user_id": user_id }),
                )
            }
            "INTERACTION_CREATE" => {
                let command = data.pointer("/data/name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                self.determine_slash_command_action(command, data, guild_id)
            }
            "MESSAGE_CREATE" | "" if data.get("content").is_some() => {
                let content = data.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if content.starts_with('!') || content.starts_with('/') {
                    let text = &content[1..];
                    let parts: Vec<&str> = text.splitn(2, ' ').collect();
                    let cmd = parts[0];
                    self.determine_text_command_action(cmd, data, guild_id)
                } else {
                    (ActionType::NoAction, guild_id, serde_json::json!({}))
                }
            }
            _ => (ActionType::NoAction, guild_id, serde_json::json!({})),
        }
    }
}

impl DiscordNodeAdapter {
    fn build_message_context(&self, message: &SignedMessage, data: &serde_json::Value) -> Option<RuleContext> {
        let guild_id = data.get("guild_id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        let sender_id = data.pointer("/author/id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        let content = data.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let is_command = content.starts_with('!') || content.starts_with('/');
        let (command, command_args) = if is_command {
            let text = &content[1..];
            let parts: Vec<&str> = text.splitn(2, ' ').collect();
            (Some(parts[0].to_string()), parts.get(1).map(|s| s.to_string()))
        } else {
            (None, None)
        };

        let reply_to_user_id = data.pointer("/referenced_message/author/id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok());

        let reply_to_message_id = data.pointer("/message_reference/message_id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok());

        Some(RuleContext {
            message: message.clone(),
            text: content,
            chat_id: guild_id,
            sender_id,
            is_command,
            command,
            command_args,
            reply_to_user_id,
            reply_to_message_id,
            is_join_request: false,
            join_request_user_id: None,
            is_callback: false,
            callback_data: None,
            group_config: None,
        })
    }

    fn build_member_add_context(&self, message: &SignedMessage, data: &serde_json::Value) -> Option<RuleContext> {
        let guild_id = data.get("guild_id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        let user_id = data.pointer("/user/id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok());

        Some(RuleContext {
            message: message.clone(),
            text: String::new(),
            chat_id: guild_id,
            sender_id: user_id.unwrap_or(0),
            is_command: false,
            command: None,
            command_args: None,
            reply_to_user_id: None,
            reply_to_message_id: None,
            is_join_request: true,
            join_request_user_id: user_id,
            is_callback: false,
            callback_data: None,
            group_config: None,
        })
    }

    fn build_member_remove_context(&self, message: &SignedMessage, data: &serde_json::Value) -> Option<RuleContext> {
        let guild_id = data.get("guild_id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        Some(RuleContext {
            message: message.clone(),
            text: String::new(),
            chat_id: guild_id,
            sender_id: 0,
            is_command: false,
            command: None,
            command_args: None,
            reply_to_user_id: None,
            reply_to_message_id: None,
            is_join_request: false,
            join_request_user_id: None,
            is_callback: false,
            callback_data: None,
            group_config: None,
        })
    }

    fn build_interaction_context(&self, message: &SignedMessage, data: &serde_json::Value) -> Option<RuleContext> {
        let guild_id = data.get("guild_id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        let sender_id = data.pointer("/member/user/id")
            .or_else(|| data.pointer("/user/id"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let command_name = data.pointer("/data/name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

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

        Some(RuleContext {
            message: message.clone(),
            text: String::new(),
            chat_id: guild_id,
            sender_id,
            is_command: command_name.is_some(),
            command: command_name,
            command_args,
            reply_to_user_id: None,
            reply_to_message_id: None,
            is_join_request: false,
            join_request_user_id: None,
            is_callback: false,
            callback_data: None,
            group_config: None,
        })
    }

    fn determine_slash_command_action(
        &self,
        command: &str,
        data: &serde_json::Value,
        guild_id: i64,
    ) -> (ActionType, i64, serde_json::Value) {
        // 从 options 中提取 user 参数
        let target_user = data.pointer("/data/options")
            .and_then(|v| v.as_array())
            .and_then(|opts| {
                opts.iter().find(|o| o.get("name").and_then(|n| n.as_str()) == Some("user"))
            })
            .and_then(|o| o.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("0");

        match command {
            "ban" => (
                ActionType::Admin(AdminAction::Ban),
                guild_id,
                serde_json::json!({ "user_id": target_user }),
            ),
            "unban" => (
                ActionType::Admin(AdminAction::Unban),
                guild_id,
                serde_json::json!({ "user_id": target_user }),
            ),
            "kick" => (
                ActionType::Admin(AdminAction::Kick),
                guild_id,
                serde_json::json!({ "user_id": target_user }),
            ),
            "mute" => {
                let duration = data.pointer("/data/options")
                    .and_then(|v| v.as_array())
                    .and_then(|opts| {
                        opts.iter().find(|o| o.get("name").and_then(|n| n.as_str()) == Some("duration"))
                    })
                    .and_then(|o| o.get("value"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3600);
                (
                    ActionType::Admin(AdminAction::Mute),
                    guild_id,
                    serde_json::json!({ "user_id": target_user, "duration_seconds": duration }),
                )
            }
            "unmute" => (
                ActionType::Admin(AdminAction::Unmute),
                guild_id,
                serde_json::json!({ "user_id": target_user }),
            ),
            "warn" => (
                ActionType::Message(MessageAction::Send),
                guild_id,
                serde_json::json!({ "user_id": target_user, "warn": true }),
            ),
            _ => (ActionType::NoAction, guild_id, serde_json::json!({})),
        }
    }

    fn determine_text_command_action(
        &self,
        cmd: &str,
        data: &serde_json::Value,
        guild_id: i64,
    ) -> (ActionType, i64, serde_json::Value) {
        let reply_user = data.pointer("/referenced_message/author/id")
            .and_then(|v| v.as_str())
            .unwrap_or("0");

        match cmd {
            "ban" => (
                ActionType::Admin(AdminAction::Ban),
                guild_id,
                serde_json::json!({ "user_id": reply_user }),
            ),
            "unban" => (
                ActionType::Admin(AdminAction::Unban),
                guild_id,
                serde_json::json!({ "user_id": reply_user }),
            ),
            "kick" => (
                ActionType::Admin(AdminAction::Kick),
                guild_id,
                serde_json::json!({ "user_id": reply_user }),
            ),
            "mute" => (
                ActionType::Admin(AdminAction::Mute),
                guild_id,
                serde_json::json!({ "user_id": reply_user, "duration_seconds": 3600 }),
            ),
            "unmute" => (
                ActionType::Admin(AdminAction::Unmute),
                guild_id,
                serde_json::json!({ "user_id": reply_user }),
            ),
            "del" | "delete" => {
                let msg_id = data.pointer("/message_reference/message_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0");
                (
                    ActionType::Message(MessageAction::Delete),
                    guild_id,
                    serde_json::json!({ "message_id": msg_id }),
                )
            }
            "pin" => {
                let msg_id = data.pointer("/message_reference/message_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0");
                (
                    ActionType::Message(MessageAction::Pin),
                    guild_id,
                    serde_json::json!({ "message_id": msg_id }),
                )
            }
            _ => (ActionType::NoAction, guild_id, serde_json::json!({})),
        }
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
            platform_event: update,
            owner_signature: String::new(),
            platform: "discord".into(),
        }
    }

    #[test]
    fn test_build_context_message() {
        let adapter = DiscordNodeAdapter;
        let msg = make_msg(serde_json::json!({
            "_discord_event_type": "MESSAGE_CREATE",
            "d": {
                "guild_id": "100200",
                "channel_id": "300400",
                "author": {"id": "42"},
                "content": "!ban some_user"
            }
        }));

        let ctx = adapter.build_context(&msg, None).unwrap();
        assert_eq!(ctx.chat_id, 100200);
        assert_eq!(ctx.sender_id, 42);
        assert!(ctx.is_command);
        assert_eq!(ctx.command.as_deref(), Some("ban"));
        assert_eq!(ctx.command_args.as_deref(), Some("some_user"));
    }

    #[test]
    fn test_build_context_member_add() {
        let adapter = DiscordNodeAdapter;
        let msg = make_msg(serde_json::json!({
            "_discord_event_type": "GUILD_MEMBER_ADD",
            "d": {
                "guild_id": "100200",
                "user": {"id": "456"}
            }
        }));

        let ctx = adapter.build_context(&msg, None).unwrap();
        assert!(ctx.is_join_request);
        assert_eq!(ctx.join_request_user_id, Some(456));
    }

    #[test]
    fn test_build_context_interaction() {
        let adapter = DiscordNodeAdapter;
        let msg = make_msg(serde_json::json!({
            "_discord_event_type": "INTERACTION_CREATE",
            "d": {
                "guild_id": "100200",
                "channel_id": "300400",
                "member": {"user": {"id": "42"}},
                "data": {
                    "name": "ban",
                    "options": [
                        {"name": "user", "value": "target_789", "type": 6}
                    ]
                }
            }
        }));

        let ctx = adapter.build_context(&msg, None).unwrap();
        assert!(ctx.is_command);
        assert_eq!(ctx.command.as_deref(), Some("ban"));
        assert_eq!(ctx.command_args.as_deref(), Some("user:target_789"));
    }

    #[test]
    fn test_determine_action_slash_ban() {
        let adapter = DiscordNodeAdapter;
        let msg = make_msg(serde_json::json!({
            "_discord_event_type": "INTERACTION_CREATE",
            "d": {
                "guild_id": "100200",
                "data": {
                    "name": "ban",
                    "options": [
                        {"name": "user", "value": "target_123", "type": 6}
                    ]
                }
            }
        }));

        let (action, guild_id, params) = adapter.determine_action(&msg);
        assert!(matches!(action, ActionType::Admin(AdminAction::Ban)));
        assert_eq!(guild_id, 100200);
        assert_eq!(params["user_id"], "target_123");
    }

    #[test]
    fn test_determine_action_text_command_ban() {
        let adapter = DiscordNodeAdapter;
        let msg = make_msg(serde_json::json!({
            "_discord_event_type": "MESSAGE_CREATE",
            "d": {
                "guild_id": "100200",
                "author": {"id": "42"},
                "content": "!ban",
                "referenced_message": {"author": {"id": "789"}}
            }
        }));

        let (action, _, params) = adapter.determine_action(&msg);
        assert!(matches!(action, ActionType::Admin(AdminAction::Ban)));
        assert_eq!(params["user_id"], "789");
    }

    #[test]
    fn test_determine_action_slash_mute_with_duration() {
        let adapter = DiscordNodeAdapter;
        let msg = make_msg(serde_json::json!({
            "_discord_event_type": "INTERACTION_CREATE",
            "d": {
                "guild_id": "100200",
                "data": {
                    "name": "mute",
                    "options": [
                        {"name": "user", "value": "target_123", "type": 6},
                        {"name": "duration", "value": 7200, "type": 4}
                    ]
                }
            }
        }));

        let (action, _, params) = adapter.determine_action(&msg);
        assert!(matches!(action, ActionType::Admin(AdminAction::Mute)));
        assert_eq!(params["duration_seconds"], 7200);
    }

    #[test]
    fn test_determine_action_member_add() {
        let adapter = DiscordNodeAdapter;
        let msg = make_msg(serde_json::json!({
            "_discord_event_type": "GUILD_MEMBER_ADD",
            "d": {
                "guild_id": "100200",
                "user": {"id": "new_user_456"}
            }
        }));

        let (action, _, params) = adapter.determine_action(&msg);
        assert!(matches!(action, ActionType::Admin(AdminAction::ApproveJoinRequest)));
        assert_eq!(params["user_id"], "new_user_456");
    }

    #[test]
    fn test_determine_action_plain_message() {
        let adapter = DiscordNodeAdapter;
        let msg = make_msg(serde_json::json!({
            "_discord_event_type": "MESSAGE_CREATE",
            "d": {
                "guild_id": "100200",
                "author": {"id": "42"},
                "content": "just chatting"
            }
        }));

        let (action, _, _) = adapter.determine_action(&msg);
        assert!(action.is_no_action());
    }
}
