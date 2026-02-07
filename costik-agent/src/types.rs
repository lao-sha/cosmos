use serde::{Deserialize, Serialize};

/// Telegram Update 对象（简化版，覆盖主要类型）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUpdate {
    /// Update ID
    pub update_id: i64,

    /// 消息（普通消息、频道消息）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<TelegramMessage>,

    /// 编辑的消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_message: Option<TelegramMessage>,

    /// 回调查询（按钮点击）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_query: Option<CallbackQuery>,

    /// 群成员变动
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_member: Option<serde_json::Value>,

    /// 入群申请
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_join_request: Option<serde_json::Value>,
}

/// Telegram Message 对象（简化版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramMessage {
    pub message_id: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<TelegramUser>,

    pub chat: TelegramChat,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// 其他字段作为通用 JSON 保留
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Telegram User
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub is_bot: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// Telegram Chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Callback Query (按钮点击)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackQuery {
    pub id: String,
    pub from: TelegramUser,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<TelegramMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

/// 签名消息（Agent → Node）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedMessage {
    /// Agent 公钥 (hex)
    pub owner_public_key: String,

    /// bot_id_hash (hex)
    pub bot_id_hash: String,

    /// 消息序列号（单调递增）
    pub sequence: u64,

    /// 时间戳（Unix 秒）
    pub timestamp: u64,

    /// 消息哈希 SHA256(telegram_update_json) (hex)
    pub message_hash: String,

    /// 原始 Telegram Update JSON
    pub telegram_update: serde_json::Value,

    /// Ed25519 签名 (hex)
    pub owner_signature: String,

    /// 平台类型
    pub platform: String,
}

/// 节点信息（从链上获取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// 节点 ID (hex)
    pub node_id: String,

    /// 节点 API 端点 URL
    pub endpoint: String,

    /// 节点公钥 (hex)
    pub node_public_key: String,

    /// 节点状态
    pub status: String,
}

/// 多播结果
#[derive(Debug, Clone)]
pub struct MulticastResult {
    /// 成功发送的节点数
    pub success_count: usize,
    /// 失败的节点数
    pub failure_count: usize,
    /// 超时的节点数
    pub timeout_count: usize,
    /// 各节点响应详情
    pub details: Vec<NodeSendResult>,
}

/// 单节点发送结果
#[derive(Debug, Clone)]
pub struct NodeSendResult {
    pub node_id: String,
    pub success: bool,
    pub latency_ms: u64,
    pub error: Option<String>,
}

/// Telegram API setWebhook 响应
#[derive(Debug, Deserialize)]
pub struct TelegramApiResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Health check 响应
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub bot_id_hash: String,
    pub public_key: String,
    pub sequence: u64,
    pub uptime_seconds: u64,
    pub nodes_count: usize,
}
