use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent 发来的签名消息（与 costik-agent 的 SignedMessage 一致）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedMessage {
    pub owner_public_key: String,
    pub bot_id_hash: String,
    pub sequence: u64,
    pub timestamp: u64,
    pub message_hash: String,
    pub telegram_update: serde_json::Value,
    pub owner_signature: String,
    pub platform: String,
}

/// Gossip 信封（所有 gossip 消息的外层包装）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipEnvelope {
    /// 协议版本
    pub version: u8,
    /// 消息类型
    pub msg_type: GossipType,
    /// 发送节点 ID
    pub sender_node_id: String,
    /// 发送时间戳 (Unix ms)
    pub timestamp: u64,
    /// 载荷
    pub payload: GossipPayload,
    /// 发送者签名 (hex)
    pub sender_signature: String,
}

/// Gossip 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GossipType {
    /// "我看到了这条消息"
    MessageSeen,
    /// 请求原始消息（Pull 补偿）
    MessagePull,
    /// Pull 响应（携带原始消息）
    MessagePullResponse,
    /// 投票决定执行何种动作
    DecisionVote,
    /// Equivocation 警报
    EquivocationAlert,
    /// Leader 执行结果
    ExecutionResult,
    /// Backup 接管 Leader
    LeaderTakeover,
    /// 心跳
    Heartbeat,
}

/// Gossip 载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GossipPayload {
    Seen(SeenPayload),
    Pull(PullPayload),
    PullResponse(PullResponsePayload),
    Vote(VotePayload),
    Alert(AlertPayload),
    Result(ResultPayload),
    Takeover(TakeoverPayload),
    Heartbeat(HeartbeatPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeenPayload {
    /// 消息唯一 ID (bot_id_hash + sequence)
    pub msg_id: String,
    /// 消息哈希
    pub msg_hash: String,
    /// 发送节点 ID
    pub node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullPayload {
    /// 需要拉取的消息 ID
    pub msg_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResponsePayload {
    /// 消息 ID
    pub msg_id: String,
    /// 完整的 SignedMessage
    pub signed_message: SignedMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotePayload {
    /// 消息 ID
    pub msg_id: String,
    /// 动作类型
    pub action_type: String,
    /// 动作参数
    pub action_params: serde_json::Value,
    /// 投票者签名
    pub voter_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertPayload {
    /// 作弊 owner 公钥
    pub owner_public_key: String,
    /// 序列号
    pub sequence: u64,
    /// 哈希 A
    pub msg_hash_a: String,
    /// 签名 A
    pub signature_a: String,
    /// 哈希 B
    pub msg_hash_b: String,
    /// 签名 B
    pub signature_b: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultPayload {
    /// 消息 ID
    pub msg_id: String,
    /// 动作 ID
    pub action_id: String,
    /// 是否成功
    pub success: bool,
    /// Agent 回执签名
    pub agent_receipt: String,
    /// 执行者节点 ID
    pub executor_node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeoverPayload {
    /// 消息 ID
    pub msg_id: String,
    /// 原 Leader 节点 ID
    pub original_leader: String,
    /// 接管者排名
    pub backup_rank: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    /// 节点 ID
    pub node_id: String,
    /// 已处理消息数
    pub messages_processed: u64,
    /// 活跃连接数
    pub active_connections: u32,
}

/// 消息状态机
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    /// 通过 gossip Seen 得知（尚无原始消息）
    HeardViaSeen,
    /// 已收到原始消息
    Received,
    /// M/K 共识达成
    Confirmed,
    /// 正在执行（Leader 执行中）
    Executing,
    /// 执行完成
    Completed,
    /// 超时
    Timeout,
    /// 失败
    Failed,
}

/// 单条消息的完整状态
#[derive(Debug, Clone)]
pub struct MessageState {
    /// 消息 ID
    pub msg_id: String,
    /// 当前状态
    pub status: MessageStatus,
    /// 原始签名消息（可能为 None，如果是通过 Seen 得知）
    pub original_message: Option<SignedMessage>,
    /// 已收到 Seen 的节点及其 msg_hash
    pub seen_nodes: HashMap<String, SeenRecord>,
    /// 目标节点列表（确定性选择结果）
    pub target_nodes: Vec<String>,
    /// 决策投票
    pub decision_votes: Vec<VotePayload>,
    /// Leader 节点 ID
    pub leader: Option<String>,
    /// Backup 节点列表
    pub backups: Vec<String>,
    /// 创建时间
    pub created_at: u64,
    /// Pull 定时器是否已启动
    pub pull_timer_started: bool,
}

/// Seen 记录
#[derive(Debug, Clone)]
pub struct SeenRecord {
    pub node_id: String,
    pub msg_hash: String,
    pub seen_at: u64,
}

/// Bot 注册信息缓存（从链上读取）
#[derive(Debug, Clone)]
pub struct BotInfoCache {
    pub bot_id_hash: String,
    pub owner_public_key: [u8; 32],
    pub platform: String,
    pub is_active: bool,
}

/// 节点信息缓存（从链上读取）
#[derive(Debug, Clone)]
pub struct NodeInfoCache {
    pub node_id: String,
    pub endpoint: String,
    pub node_public_key: [u8; 32],
    pub status: String,
    pub reputation: u16,
}
