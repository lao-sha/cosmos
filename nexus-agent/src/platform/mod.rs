pub mod discord;
pub mod telegram;

use serde::{Deserialize, Serialize};

/// 平台事件（统一格式）
///
/// 所有平台的事件都被解析为此结构，后续流程（签名、多播、共识、规则引擎）
/// 全部基于 PlatformEvent 操作，不再直接访问平台特定 JSON。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformEvent {
    /// 平台类型: "telegram" | "discord"
    pub platform: String,
    /// 群组/服务器 ID（TG: chat_id, Discord: guild_id）
    pub group_id: String,
    /// 频道 ID（Discord 专用，TG 为空）
    #[serde(default)]
    pub channel_id: String,
    /// 发送者 ID
    #[serde(default)]
    pub sender_id: String,
    /// 发送者是否为 Bot
    #[serde(default)]
    pub sender_is_bot: bool,
    /// 消息文本
    #[serde(default)]
    pub text: String,
    /// 消息 ID
    #[serde(default)]
    pub message_id: String,
    /// 是否为命令
    #[serde(default)]
    pub is_command: bool,
    /// 命令名（不含前缀 /）
    #[serde(default)]
    pub command: Option<String>,
    /// 命令参数
    #[serde(default)]
    pub command_args: Option<String>,
    /// 回复目标用户 ID
    #[serde(default)]
    pub reply_to_user_id: Option<String>,
    /// 回复目标消息 ID
    #[serde(default)]
    pub reply_to_message_id: Option<String>,
    /// 是否为入群事件
    #[serde(default)]
    pub is_join_event: bool,
    /// 入群用户 ID
    #[serde(default)]
    pub join_user_id: Option<String>,
    /// 是否为离群事件
    #[serde(default)]
    pub is_leave_event: bool,
    /// 是否为交互事件（Discord Slash Command / TG Callback）
    #[serde(default)]
    pub is_interaction: bool,
    /// 交互 ID（Discord interaction_id / TG callback_query_id）
    #[serde(default)]
    pub interaction_id: Option<String>,
    /// 交互 token（Discord interaction_token）
    #[serde(default)]
    pub interaction_token: Option<String>,
    /// 交互回调数据（TG callback_data）
    #[serde(default)]
    pub interaction_data: Option<String>,
    /// 是否为群成员变动事件
    #[serde(default)]
    pub is_member_update: bool,
    /// 原始平台事件 JSON（保留完整数据供平台特定逻辑使用）
    #[serde(default)]
    pub raw_event: serde_json::Value,
}

impl PlatformEvent {
    /// 获取 chat_id 的 i64 表示（兼容现有代码）
    pub fn chat_id_i64(&self) -> i64 {
        self.group_id.parse::<i64>().unwrap_or(0)
    }

    /// 获取 sender_id 的 i64 表示
    pub fn sender_id_i64(&self) -> i64 {
        self.sender_id.parse::<i64>().unwrap_or(0)
    }

    /// 获取 message_id 的 i64 表示
    pub fn message_id_i64(&self) -> i64 {
        self.message_id.parse::<i64>().unwrap_or(0)
    }

    /// 获取 reply_to_user_id 的 i64 表示
    pub fn reply_to_user_id_i64(&self) -> i64 {
        self.reply_to_user_id.as_deref()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0)
    }

    /// 获取 reply_to_message_id 的 i64 表示
    pub fn reply_to_message_id_i64(&self) -> i64 {
        self.reply_to_message_id.as_deref()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0)
    }
}

/// 平台适配器 Trait (Agent 侧)
///
/// 每个平台实现此 trait，将原始事件 JSON 解析为统一的 PlatformEvent。
pub trait PlatformAdapter: Send + Sync {
    /// 平台名称
    fn platform_name(&self) -> &str;

    /// 解析原始事件 JSON 为统一 PlatformEvent
    fn parse_event(&self, raw: &serde_json::Value) -> Option<PlatformEvent>;
}
