use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info, warn};

use crate::platform::PlatformEvent;

/// Discord Gateway 连接管理器
///
/// 负责:
/// 1. 连接到 wss://gateway.discord.gg/?v=10&encoding=json
/// 2. 处理 HELLO → 发送 IDENTIFY → 接收 READY
/// 3. 维护心跳循环 (heartbeat_interval)
/// 4. 断线后自动 RESUME（使用 session_id + last_sequence）
/// 5. 将接收到的事件通过 event_tx 发送给上层处理
pub struct DiscordGateway {
    bot_token: String,
    intents: u64,
    event_tx: mpsc::Sender<PlatformEvent>,
}

/// Discord Gateway Opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum GatewayOpcode {
    Dispatch = 0,
    Heartbeat = 1,
    Identify = 2,
    PresenceUpdate = 3,
    VoiceStateUpdate = 4,
    Resume = 6,
    Reconnect = 7,
    RequestGuildMembers = 8,
    InvalidSession = 9,
    Hello = 10,
    HeartbeatAck = 11,
}

impl GatewayOpcode {
    fn from_u64(v: u64) -> Option<Self> {
        match v {
            0 => Some(Self::Dispatch),
            1 => Some(Self::Heartbeat),
            2 => Some(Self::Identify),
            3 => Some(Self::PresenceUpdate),
            4 => Some(Self::VoiceStateUpdate),
            6 => Some(Self::Resume),
            7 => Some(Self::Reconnect),
            8 => Some(Self::RequestGuildMembers),
            9 => Some(Self::InvalidSession),
            10 => Some(Self::Hello),
            11 => Some(Self::HeartbeatAck),
            _ => None,
        }
    }
}

/// Gateway 消息帧
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayPayload {
    op: u64,
    #[serde(default)]
    d: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    s: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    t: Option<String>,
}

/// 会话状态（用于 Resume）
struct SessionState {
    session_id: String,
    resume_gateway_url: String,
    last_sequence: Option<u64>,
}

/// Discord Gateway 默认 URL
const GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=10&encoding=json";

/// 最大重连次数
const MAX_RECONNECT_ATTEMPTS: u32 = 10;

/// 重连基础延迟（毫秒）
const RECONNECT_BASE_DELAY_MS: u64 = 1000;

impl DiscordGateway {
    pub fn new(
        bot_token: String,
        intents: u64,
        event_tx: mpsc::Sender<PlatformEvent>,
    ) -> Self {
        Self { bot_token, intents, event_tx }
    }

    /// 启动 Gateway 连接（自动重连循环）
    ///
    /// 此函数会一直运行，断线后自动重连。
    /// 仅在 event_tx 关闭（接收端 drop）时返回。
    pub async fn run(&self) {
        let mut session: Option<SessionState> = None;
        let mut reconnect_count: u32 = 0;

        loop {
            let gateway_url = session.as_ref()
                .map(|s| s.resume_gateway_url.clone())
                .unwrap_or_else(|| GATEWAY_URL.to_string());

            info!(url = %gateway_url, attempt = reconnect_count, "Discord Gateway 连接中...");

            match self.connect_and_run(&gateway_url, &mut session).await {
                Ok(()) => {
                    info!("Discord Gateway 正常关闭");
                    return;
                }
                Err(GatewayError::InvalidSession) => {
                    warn!("Discord Gateway 会话无效，重新 IDENTIFY");
                    session = None;
                    reconnect_count = 0;
                }
                Err(GatewayError::Reconnect) => {
                    info!("Discord Gateway 请求重连");
                    reconnect_count += 1;
                }
                Err(GatewayError::ConnectionFailed(e)) => {
                    warn!(error = %e, "Discord Gateway 连接失败");
                    reconnect_count += 1;
                }
                Err(GatewayError::ChannelClosed) => {
                    info!("Event channel 已关闭，Gateway 退出");
                    return;
                }
            }

            if reconnect_count > MAX_RECONNECT_ATTEMPTS {
                error!("Discord Gateway 重连次数超限 ({}), 退出", MAX_RECONNECT_ATTEMPTS);
                return;
            }

            // 指数退避
            let delay = RECONNECT_BASE_DELAY_MS * (1 << reconnect_count.min(5));
            info!(delay_ms = delay, "等待重连...");
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }
    }

    /// 单次连接循环
    async fn connect_and_run(
        &self,
        url: &str,
        session: &mut Option<SessionState>,
    ) -> Result<(), GatewayError> {
        let (ws_stream, _) = connect_async(url).await
            .map_err(|e| GatewayError::ConnectionFailed(e.to_string()))?;

        info!("Discord Gateway WebSocket 已连接");

        let (mut write, mut read) = ws_stream.split();

        // ── 1. 等待 HELLO ──
        let heartbeat_interval = loop {
            match read.next().await {
                Some(Ok(WsMessage::Text(text))) => {
                    let payload: GatewayPayload = serde_json::from_str(&text)
                        .map_err(|e| GatewayError::ConnectionFailed(format!("JSON 解析失败: {}", e)))?;

                    if payload.op == GatewayOpcode::Hello as u64 {
                        let interval = payload.d.get("heartbeat_interval")
                            .and_then(|v| v.as_u64())
                            .ok_or_else(|| GatewayError::ConnectionFailed("HELLO 缺少 heartbeat_interval".into()))?;
                        info!(heartbeat_interval_ms = interval, "收到 HELLO");
                        break interval;
                    }
                }
                Some(Ok(WsMessage::Close(frame))) => {
                    warn!(?frame, "WebSocket 在 HELLO 前关闭");
                    return Err(GatewayError::Reconnect);
                }
                Some(Err(e)) => {
                    return Err(GatewayError::ConnectionFailed(format!("WebSocket 错误: {}", e)));
                }
                None => {
                    return Err(GatewayError::ConnectionFailed("WebSocket 流结束".into()));
                }
                _ => continue,
            }
        };

        // ── 2. IDENTIFY 或 RESUME ──
        if let Some(ref sess) = session {
            let resume = GatewayPayload {
                op: GatewayOpcode::Resume as u64,
                d: serde_json::json!({
                    "token": self.bot_token,
                    "session_id": sess.session_id,
                    "seq": sess.last_sequence,
                }),
                s: None,
                t: None,
            };
            let json = serde_json::to_string(&resume)
                .map_err(|e| GatewayError::ConnectionFailed(e.to_string()))?;
            write.send(WsMessage::Text(json.into())).await
                .map_err(|e| GatewayError::ConnectionFailed(e.to_string()))?;
            info!("已发送 RESUME");
        } else {
            let identify = GatewayPayload {
                op: GatewayOpcode::Identify as u64,
                d: serde_json::json!({
                    "token": self.bot_token,
                    "intents": self.intents,
                    "properties": {
                        "os": "linux",
                        "browser": "nexus-agent",
                        "device": "nexus-agent"
                    }
                }),
                s: None,
                t: None,
            };
            let json = serde_json::to_string(&identify)
                .map_err(|e| GatewayError::ConnectionFailed(e.to_string()))?;
            write.send(WsMessage::Text(json.into())).await
                .map_err(|e| GatewayError::ConnectionFailed(e.to_string()))?;
            info!("已发送 IDENTIFY");
        }

        // ── 3. 心跳 + 事件循环 ──
        let mut heartbeat_acked = true;
        let mut heartbeat_timer = tokio::time::interval(
            std::time::Duration::from_millis(heartbeat_interval)
        );
        // 首次心跳在 heartbeat_interval * jitter 后发送
        heartbeat_timer.tick().await; // 跳过立即 tick

        let mut last_sequence: Option<u64> = session.as_ref().and_then(|s| s.last_sequence);

        loop {
            tokio::select! {
                // 心跳定时器
                _ = heartbeat_timer.tick() => {
                    if !heartbeat_acked {
                        warn!("Discord Gateway 心跳超时（未收到 ACK），断线重连");
                        return Err(GatewayError::Reconnect);
                    }
                    heartbeat_acked = false;

                    let hb = GatewayPayload {
                        op: GatewayOpcode::Heartbeat as u64,
                        d: last_sequence.map(serde_json::Value::from).unwrap_or(serde_json::Value::Null),
                        s: None,
                        t: None,
                    };
                    let json = serde_json::to_string(&hb).unwrap_or_default();
                    if let Err(e) = write.send(WsMessage::Text(json.into())).await {
                        warn!(error = %e, "发送心跳失败");
                        return Err(GatewayError::Reconnect);
                    }
                    debug!("心跳已发送 (seq={:?})", last_sequence);
                }

                // WebSocket 消息
                msg = read.next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text))) => {
                            let payload: GatewayPayload = match serde_json::from_str(&text) {
                                Ok(p) => p,
                                Err(e) => {
                                    warn!(error = %e, "Gateway JSON 解析失败");
                                    continue;
                                }
                            };

                            // 更新 sequence
                            if let Some(seq) = payload.s {
                                last_sequence = Some(seq);
                            }

                            match GatewayOpcode::from_u64(payload.op) {
                                Some(GatewayOpcode::Dispatch) => {
                                    let event_name = payload.t.as_deref().unwrap_or("UNKNOWN");
                                    debug!(event = event_name, seq = ?payload.s, "收到事件");

                                    // READY → 保存 session
                                    if event_name == "READY" {
                                        let session_id = payload.d.get("session_id")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let resume_url = payload.d.get("resume_gateway_url")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or(GATEWAY_URL)
                                            .to_string();
                                        info!(session_id = %session_id, "Discord READY");
                                        *session = Some(SessionState {
                                            session_id,
                                            resume_gateway_url: resume_url,
                                            last_sequence,
                                        });
                                        continue;
                                    }

                                    // RESUMED → 恢复成功
                                    if event_name == "RESUMED" {
                                        info!("Discord RESUMED 成功");
                                        continue;
                                    }

                                    // 解析事件 → PlatformEvent
                                    if let Some(event) = self.parse_discord_event(event_name, &payload.d) {
                                        if self.event_tx.send(event).await.is_err() {
                                            return Err(GatewayError::ChannelClosed);
                                        }
                                    }
                                }
                                Some(GatewayOpcode::HeartbeatAck) => {
                                    heartbeat_acked = true;
                                    debug!("心跳 ACK 已收到");
                                }
                                Some(GatewayOpcode::Heartbeat) => {
                                    // 服务端请求立即心跳
                                    let hb = GatewayPayload {
                                        op: GatewayOpcode::Heartbeat as u64,
                                        d: last_sequence.map(serde_json::Value::from).unwrap_or(serde_json::Value::Null),
                                        s: None,
                                        t: None,
                                    };
                                    let json = serde_json::to_string(&hb).unwrap_or_default();
                                    let _ = write.send(WsMessage::Text(json.into())).await;
                                }
                                Some(GatewayOpcode::Reconnect) => {
                                    info!("服务端请求 Reconnect");
                                    if let Some(ref mut s) = session {
                                        s.last_sequence = last_sequence;
                                    }
                                    return Err(GatewayError::Reconnect);
                                }
                                Some(GatewayOpcode::InvalidSession) => {
                                    let resumable = payload.d.as_bool().unwrap_or(false);
                                    warn!(resumable, "收到 Invalid Session");
                                    if !resumable {
                                        return Err(GatewayError::InvalidSession);
                                    }
                                    // 等 1-5 秒后重新 IDENTIFY
                                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                    return Err(GatewayError::InvalidSession);
                                }
                                _ => {
                                    debug!(op = payload.op, "未处理的 Gateway opcode");
                                }
                            }
                        }
                        Some(Ok(WsMessage::Close(frame))) => {
                            let code = frame.as_ref().map(|f| f.code);
                            warn!(?code, "WebSocket 关闭");

                            if let Some(ref mut s) = session {
                                s.last_sequence = last_sequence;
                            }

                            // 4004 = Authentication failed (不可恢复)
                            // 4014 = Disallowed intents (不可恢复)
                            if let Some(ref f) = frame {
                                let code_u16: u16 = f.code.into();
                                if matches!(code_u16, 4004 | 4010 | 4011 | 4012 | 4013 | 4014) {
                                    error!(code = code_u16, reason = %f.reason, "不可恢复的 Gateway 关闭码");
                                    return Ok(());
                                }
                            }

                            return Err(GatewayError::Reconnect);
                        }
                        Some(Ok(WsMessage::Ping(data))) => {
                            let _ = write.send(WsMessage::Pong(data)).await;
                        }
                        Some(Err(e)) => {
                            warn!(error = %e, "WebSocket 错误");
                            if let Some(ref mut s) = session {
                                s.last_sequence = last_sequence;
                            }
                            return Err(GatewayError::Reconnect);
                        }
                        None => {
                            warn!("WebSocket 流结束");
                            if let Some(ref mut s) = session {
                                s.last_sequence = last_sequence;
                            }
                            return Err(GatewayError::Reconnect);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// 将 Discord Gateway 事件解析为 PlatformEvent
    fn parse_discord_event(&self, event_name: &str, data: &serde_json::Value) -> Option<PlatformEvent> {
        match event_name {
            "MESSAGE_CREATE" => {
                let guild_id = data.get("guild_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let channel_id = data.get("channel_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let sender_id = data.pointer("/author/id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let sender_is_bot = data.pointer("/author/bot")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let content = data.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let message_id = data.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Discord 命令: / 前缀 或 ! 前缀
                let is_command = content.starts_with('/') || content.starts_with('!');
                let (command, command_args) = if is_command {
                    let text = if content.starts_with('!') || content.starts_with('/') {
                        &content[1..]
                    } else {
                        &content
                    };
                    let parts: Vec<&str> = text.splitn(2, ' ').collect();
                    (
                        Some(parts[0].to_string()),
                        parts.get(1).map(|s| s.to_string()),
                    )
                } else {
                    (None, None)
                };

                // 检测回复
                let reply_to_user_id = data.pointer("/referenced_message/author/id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let reply_to_message_id = data.get("message_reference")
                    .and_then(|v| v.get("message_id"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // 检测 @mention
                let _mentions = data.get("mentions")
                    .and_then(|v| v.as_array());

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
                    raw_event: data.clone(),
                })
            }

            "GUILD_MEMBER_ADD" => {
                let guild_id = data.get("guild_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let user_id = data.pointer("/user/id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

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
                    raw_event: data.clone(),
                })
            }

            "GUILD_MEMBER_REMOVE" => {
                let guild_id = data.get("guild_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                Some(PlatformEvent {
                    platform: "discord".to_string(),
                    group_id: guild_id,
                    channel_id: String::new(),
                    sender_id: data.pointer("/user/id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
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
                    raw_event: data.clone(),
                })
            }

            "GUILD_MEMBER_UPDATE" => {
                let guild_id = data.get("guild_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                Some(PlatformEvent {
                    platform: "discord".to_string(),
                    group_id: guild_id,
                    channel_id: String::new(),
                    sender_id: data.pointer("/user/id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
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
                    raw_event: data.clone(),
                })
            }

            "INTERACTION_CREATE" => {
                let guild_id = data.get("guild_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let channel_id = data.get("channel_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let interaction_id = data.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let interaction_token = data.get("token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let sender_id = data.pointer("/member/user/id")
                    .or_else(|| data.pointer("/user/id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Slash command name
                let command_name = data.pointer("/data/name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // 提取 Slash command 选项为 command_args
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
                    raw_event: data.clone(),
                })
            }

            // 未处理的事件类型
            _ => {
                debug!(event = event_name, "未处理的 Discord 事件");
                None
            }
        }
    }
}

/// Gateway 错误类型
#[derive(Debug)]
enum GatewayError {
    ConnectionFailed(String),
    Reconnect,
    InvalidSession,
    ChannelClosed,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_gateway() -> (DiscordGateway, mpsc::Receiver<PlatformEvent>) {
        let (tx, rx) = mpsc::channel(100);
        let gw = DiscordGateway::new("test_token".into(), 33283, tx);
        (gw, rx)
    }

    #[test]
    fn test_parse_message_create() {
        let (gw, _rx) = make_gateway();
        let data = serde_json::json!({
            "id": "msg_123",
            "channel_id": "chan_456",
            "guild_id": "guild_789",
            "author": {"id": "user_001", "bot": false},
            "content": "hello world",
            "timestamp": "2024-01-01T00:00:00Z"
        });

        let event = gw.parse_discord_event("MESSAGE_CREATE", &data).unwrap();
        assert_eq!(event.platform, "discord");
        assert_eq!(event.group_id, "guild_789");
        assert_eq!(event.channel_id, "chan_456");
        assert_eq!(event.sender_id, "user_001");
        assert!(!event.sender_is_bot);
        assert_eq!(event.text, "hello world");
        assert_eq!(event.message_id, "msg_123");
        assert!(!event.is_command);
    }

    #[test]
    fn test_parse_message_command() {
        let (gw, _rx) = make_gateway();
        let data = serde_json::json!({
            "id": "msg_124",
            "channel_id": "chan_456",
            "guild_id": "guild_789",
            "author": {"id": "user_001", "bot": false},
            "content": "!ban some user"
        });

        let event = gw.parse_discord_event("MESSAGE_CREATE", &data).unwrap();
        assert!(event.is_command);
        assert_eq!(event.command.as_deref(), Some("ban"));
        assert_eq!(event.command_args.as_deref(), Some("some user"));
    }

    #[test]
    fn test_parse_message_with_reply() {
        let (gw, _rx) = make_gateway();
        let data = serde_json::json!({
            "id": "msg_125",
            "channel_id": "chan_456",
            "guild_id": "guild_789",
            "author": {"id": "user_001", "bot": false},
            "content": "!ban",
            "referenced_message": {
                "id": "msg_100",
                "author": {"id": "target_user"}
            },
            "message_reference": {
                "message_id": "msg_100"
            }
        });

        let event = gw.parse_discord_event("MESSAGE_CREATE", &data).unwrap();
        assert_eq!(event.reply_to_user_id.as_deref(), Some("target_user"));
        assert_eq!(event.reply_to_message_id.as_deref(), Some("msg_100"));
    }

    #[test]
    fn test_parse_member_add() {
        let (gw, _rx) = make_gateway();
        let data = serde_json::json!({
            "guild_id": "guild_789",
            "user": {"id": "new_user_123", "username": "Alice"}
        });

        let event = gw.parse_discord_event("GUILD_MEMBER_ADD", &data).unwrap();
        assert!(event.is_join_event);
        assert_eq!(event.join_user_id.as_deref(), Some("new_user_123"));
        assert_eq!(event.group_id, "guild_789");
    }

    #[test]
    fn test_parse_member_remove() {
        let (gw, _rx) = make_gateway();
        let data = serde_json::json!({
            "guild_id": "guild_789",
            "user": {"id": "leaving_user"}
        });

        let event = gw.parse_discord_event("GUILD_MEMBER_REMOVE", &data).unwrap();
        assert!(event.is_leave_event);
        assert_eq!(event.sender_id, "leaving_user");
    }

    #[test]
    fn test_parse_interaction_create() {
        let (gw, _rx) = make_gateway();
        let data = serde_json::json!({
            "id": "inter_001",
            "type": 2,
            "token": "interaction_token_abc",
            "guild_id": "guild_789",
            "channel_id": "chan_456",
            "member": {"user": {"id": "user_001"}},
            "data": {
                "name": "ban",
                "options": [
                    {"name": "user", "value": "target_123", "type": 6},
                    {"name": "reason", "value": "spam", "type": 3}
                ]
            }
        });

        let event = gw.parse_discord_event("INTERACTION_CREATE", &data).unwrap();
        assert!(event.is_interaction);
        assert_eq!(event.interaction_id.as_deref(), Some("inter_001"));
        assert_eq!(event.interaction_token.as_deref(), Some("interaction_token_abc"));
        assert!(event.is_command);
        assert_eq!(event.command.as_deref(), Some("ban"));
        assert_eq!(event.command_args.as_deref(), Some("user:target_123 reason:spam"));
        assert_eq!(event.group_id, "guild_789");
    }

    #[test]
    fn test_parse_member_update() {
        let (gw, _rx) = make_gateway();
        let data = serde_json::json!({
            "guild_id": "guild_789",
            "user": {"id": "user_001"},
            "roles": ["role_1", "role_2"]
        });

        let event = gw.parse_discord_event("GUILD_MEMBER_UPDATE", &data).unwrap();
        assert!(event.is_member_update);
    }

    #[test]
    fn test_parse_unknown_event_returns_none() {
        let (gw, _rx) = make_gateway();
        let data = serde_json::json!({});
        assert!(gw.parse_discord_event("TYPING_START", &data).is_none());
    }

    #[test]
    fn test_gateway_opcode_roundtrip() {
        assert_eq!(GatewayOpcode::from_u64(0), Some(GatewayOpcode::Dispatch));
        assert_eq!(GatewayOpcode::from_u64(1), Some(GatewayOpcode::Heartbeat));
        assert_eq!(GatewayOpcode::from_u64(10), Some(GatewayOpcode::Hello));
        assert_eq!(GatewayOpcode::from_u64(11), Some(GatewayOpcode::HeartbeatAck));
        assert_eq!(GatewayOpcode::from_u64(99), None);
    }

    #[test]
    fn test_bot_message_detected() {
        let (gw, _rx) = make_gateway();
        let data = serde_json::json!({
            "id": "msg_bot",
            "channel_id": "chan_456",
            "guild_id": "guild_789",
            "author": {"id": "bot_001", "bot": true},
            "content": "I am a bot"
        });

        let event = gw.parse_discord_event("MESSAGE_CREATE", &data).unwrap();
        assert!(event.sender_is_bot);
    }
}
