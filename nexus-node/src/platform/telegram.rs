use crate::types::{SignedMessage, ActionType, AdminAction, MessageAction};
use crate::chain_cache::ChainCache;
use crate::rule_engine::RuleContext;
use super::NodePlatformAdapter;

/// Telegram Node 侧适配器
///
/// 从 SignedMessage 中提取 Telegram Update JSON 的字段，
/// 构建 RuleContext 和 determine_action。
pub struct TelegramNodeAdapter;

impl NodePlatformAdapter for TelegramNodeAdapter {
    fn platform_name(&self) -> &str {
        "telegram"
    }

    fn build_context(
        &self,
        message: &SignedMessage,
        _chain_cache: Option<&ChainCache>,
    ) -> Option<RuleContext> {
        let update = &message.platform_event;

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

        Some(RuleContext {
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
            group_config: None, // 由调用方通过 ChainCache 填充
        })
    }

    fn determine_action(
        &self,
        message: &SignedMessage,
    ) -> (ActionType, i64, serde_json::Value) {
        let update = &message.platform_event;

        let chat_id = update
            .pointer("/message/chat/id")
            .or_else(|| update.pointer("/callback_query/message/chat/id"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let text = update
            .pointer("/message/text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 入群申请
        if update.get("chat_join_request").is_some() {
            let user_id = update
                .pointer("/chat_join_request/from/id")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            return (
                ActionType::Admin(AdminAction::ApproveJoinRequest),
                chat_id,
                serde_json::json!({ "user_id": user_id }),
            );
        }

        // 群成员变动
        if update.get("chat_member").is_some() {
            return (ActionType::NoAction, chat_id, serde_json::json!({}));
        }

        // 回调查询（按钮）
        if update.get("callback_query").is_some() {
            let callback_data = update
                .pointer("/callback_query/data")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            return (
                ActionType::NoAction,
                chat_id,
                serde_json::json!({ "callback_data": callback_data }),
            );
        }

        // 命令消息
        if text.starts_with('/') {
            let parts: Vec<&str> = text.splitn(2, ' ').collect();
            let command = parts[0];

            return match command {
                "/ban" | "/kick" => {
                    let reply_user_id = update
                        .pointer("/message/reply_to_message/from/id")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    (
                        ActionType::Admin(AdminAction::Ban),
                        chat_id,
                        serde_json::json!({ "user_id": reply_user_id }),
                    )
                }
                "/unban" => {
                    let reply_user_id = update
                        .pointer("/message/reply_to_message/from/id")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    (
                        ActionType::Admin(AdminAction::Unban),
                        chat_id,
                        serde_json::json!({ "user_id": reply_user_id }),
                    )
                }
                "/mute" => {
                    let reply_user_id = update
                        .pointer("/message/reply_to_message/from/id")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    (
                        ActionType::Admin(AdminAction::Mute),
                        chat_id,
                        serde_json::json!({
                            "user_id": reply_user_id,
                            "duration_seconds": 3600,
                        }),
                    )
                }
                "/unmute" => {
                    let reply_user_id = update
                        .pointer("/message/reply_to_message/from/id")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    (
                        ActionType::Admin(AdminAction::Unmute),
                        chat_id,
                        serde_json::json!({ "user_id": reply_user_id }),
                    )
                }
                "/pin" => {
                    let reply_msg_id = update
                        .pointer("/message/reply_to_message/message_id")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    (
                        ActionType::Message(MessageAction::Pin),
                        chat_id,
                        serde_json::json!({ "message_id": reply_msg_id }),
                    )
                }
                "/del" | "/delete" => {
                    let reply_msg_id = update
                        .pointer("/message/reply_to_message/message_id")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    (
                        ActionType::Message(MessageAction::Delete),
                        chat_id,
                        serde_json::json!({ "message_id": reply_msg_id }),
                    )
                }
                _ => (ActionType::NoAction, chat_id, serde_json::json!({})),
            };
        }

        // 普通消息 — 无需动作
        (ActionType::NoAction, chat_id, serde_json::json!({}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SignedMessage;

    fn make_msg(update: serde_json::Value) -> SignedMessage {
        SignedMessage {
            owner_public_key: String::new(),
            bot_id_hash: String::new(),
            sequence: 1,
            timestamp: 0,
            message_hash: String::new(),
            platform_event: update,
            owner_signature: String::new(),
            platform: "telegram".into(),
        }
    }

    #[test]
    fn test_build_context_message() {
        let adapter = TelegramNodeAdapter;
        let msg = make_msg(serde_json::json!({
            "message": {
                "message_id": 100,
                "from": {"id": 42, "is_bot": false},
                "chat": {"id": -100123, "type": "supergroup"},
                "text": "/ban some_arg"
            }
        }));

        let ctx = adapter.build_context(&msg, None).unwrap();
        assert_eq!(ctx.chat_id, -100123);
        assert_eq!(ctx.sender_id, 42);
        assert!(ctx.is_command);
        assert_eq!(ctx.command.as_deref(), Some("ban"));
        assert_eq!(ctx.command_args.as_deref(), Some("some_arg"));
    }

    #[test]
    fn test_build_context_join_request() {
        let adapter = TelegramNodeAdapter;
        let msg = make_msg(serde_json::json!({
            "chat_join_request": {
                "chat": {"id": -100123},
                "from": {"id": 456, "is_bot": false}
            }
        }));

        let ctx = adapter.build_context(&msg, None).unwrap();
        assert!(ctx.is_join_request);
        assert_eq!(ctx.join_request_user_id, Some(456));
    }

    #[test]
    fn test_determine_action_ban() {
        let adapter = TelegramNodeAdapter;
        let msg = make_msg(serde_json::json!({
            "message": {
                "message_id": 100,
                "chat": {"id": -100123, "type": "supergroup"},
                "text": "/ban",
                "reply_to_message": {
                    "message_id": 99,
                    "from": {"id": 789}
                }
            }
        }));

        let (action, chat_id, params) = adapter.determine_action(&msg);
        assert!(matches!(action, ActionType::Admin(AdminAction::Ban)));
        assert_eq!(chat_id, -100123);
        assert_eq!(params["user_id"], 789);
    }

    #[test]
    fn test_determine_action_join_request() {
        let adapter = TelegramNodeAdapter;
        let msg = make_msg(serde_json::json!({
            "chat_join_request": {
                "chat": {"id": -100123},
                "from": {"id": 456}
            }
        }));

        let (action, _, params) = adapter.determine_action(&msg);
        assert!(matches!(action, ActionType::Admin(AdminAction::ApproveJoinRequest)));
        assert_eq!(params["user_id"], 456);
    }

    #[test]
    fn test_determine_action_plain_message() {
        let adapter = TelegramNodeAdapter;
        let msg = make_msg(serde_json::json!({
            "message": {
                "message_id": 100,
                "chat": {"id": -100123, "type": "supergroup"},
                "text": "hello world"
            }
        }));

        let (action, _, _) = adapter.determine_action(&msg);
        assert!(action.is_no_action());
    }
}
