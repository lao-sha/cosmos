use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// 动作类型（嵌套枚举，Sprint 7 重构）
// ═══════════════════════════════════════════════════════════════

/// 消息操作
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageAction {
    /// 发送消息
    Send,
    /// 删除消息
    Delete,
    /// 批量删除消息
    DeleteBatch,
    /// 置顶消息
    Pin,
    /// 取消置顶
    Unpin,
}

/// 管理操作
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdminAction {
    /// 封禁用户
    Ban,
    /// 解封用户
    Unban,
    /// 禁言用户
    Mute,
    /// 解除禁言
    Unmute,
    /// 审批入群
    ApproveJoinRequest,
    /// 拒绝入群
    DeclineJoinRequest,
    /// 设置群权限
    SetPermissions,
    /// 踢出用户（ban + 立即 unban）
    Kick,
    /// 提升为管理员
    Promote,
    /// 降级管理员
    Demote,
}

/// 查询操作（只读，不需要共识）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryAction {
    /// 获取群成员信息
    GetChatMember,
    /// 获取管理员列表
    GetAdmins,
    /// 获取群信息
    GetChat,
    /// 获取 Bot 自身信息
    GetMe,
}

/// 动作类型（嵌套枚举）
///
/// 分为四类:
/// - Message: 消息操作（发送/删除/置顶）
/// - Admin: 管理操作（封禁/禁言/审批）
/// - Query: 查询操作（不需要共识，任意节点可直接请求 Agent）
/// - NoAction: 无需动作
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    /// 消息操作
    Message(MessageAction),
    /// 管理操作
    Admin(AdminAction),
    /// 查询操作（不需要共识）
    Query(QueryAction),
    /// 无需动作
    NoAction,
}

impl ActionType {
    /// 是否需要共识流程
    pub fn requires_consensus(&self) -> bool {
        !matches!(self, ActionType::Query(_) | ActionType::NoAction)
    }

    /// 是否为无动作
    pub fn is_no_action(&self) -> bool {
        matches!(self, ActionType::NoAction)
    }
}

/// Agent 发来的签名消息（与 nexus-agent 的 SignedMessage 一致）
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
    /// 配置同步（Agent → Node 签名配置广播）
    ConfigSync,
    /// 配置拉取请求（Node → Node）
    ConfigPull,
    /// 配置拉取响应（携带 SignedGroupConfig）
    ConfigPullResponse,
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
    ConfigSync(ConfigSyncPayload),
    ConfigPull(ConfigPullPayload),
    ConfigPullResponse(ConfigPullResponsePayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeenPayload {
    /// 消息唯一 ID (bot_id_hash + sequence)
    pub msg_id: String,
    /// 消息哈希
    pub msg_hash: String,
    /// 发送节点 ID
    pub node_id: String,
    /// 发送节点当前的配置版本号
    pub config_version: u64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSyncPayload {
    /// Bot ID 哈希
    pub bot_id_hash: String,
    /// 签名的群配置
    pub signed_config: SignedGroupConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPullPayload {
    /// 请求的 Bot ID 哈希
    pub bot_id_hash: String,
    /// 请求者当前的配置版本
    pub current_version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPullResponsePayload {
    /// Bot ID 哈希
    pub bot_id_hash: String,
    /// 签名的群配置（如果版本更新）
    pub signed_config: Option<SignedGroupConfig>,
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

// ═══════════════════════════════════════════════════════════════
// 群配置（全节点同步，不上链）
// ═══════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════
// Bot 功能枚举类型
// ═══════════════════════════════════════════════════════════════

/// 防刷屏动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FloodAction {
    /// 禁言（默认）
    Mute,
    /// 踢出
    Kick,
    /// 封禁
    Ban,
    /// 仅删除消息
    DeleteOnly,
}

impl Default for FloodAction {
    fn default() -> Self { Self::Mute }
}

/// 警告超限动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WarnAction {
    /// 封禁（默认）
    Ban,
    /// 踢出
    Kick,
    /// 禁言
    Mute,
}

impl Default for WarnAction {
    fn default() -> Self { Self::Ban }
}

/// 黑名单匹配模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlacklistMode {
    /// 精确匹配
    Exact,
    /// 包含匹配（默认）
    Contains,
    /// 正则匹配
    Regex,
}

impl Default for BlacklistMode {
    fn default() -> Self { Self::Contains }
}

/// 黑名单触发动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlacklistAction {
    /// 仅删除（默认）
    Delete,
    /// 删除 + 警告
    DeleteAndWarn,
    /// 删除 + 禁言
    DeleteAndMute,
    /// 删除 + 封禁
    DeleteAndBan,
}

impl Default for BlacklistAction {
    fn default() -> Self { Self::Delete }
}

/// 可锁定的消息类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum LockType {
    Audio,
    Video,
    Photo,
    Document,
    Sticker,
    Gif,
    Url,
    Forward,
    Voice,
    Contact,
    Location,
    Poll,
    Game,
    Inline,
}

// ═══════════════════════════════════════════════════════════════
// 群配置（全节点同步，不上链）
// ═══════════════════════════════════════════════════════════════

/// 入群审批策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JoinApprovalPolicy {
    /// 自动通过所有申请
    AutoApprove,
    /// 需要人工审批
    ManualApproval,
    /// 需要验证码
    CaptchaRequired,
    /// 需要链上身份（Token-Gating）
    TokenGating {
        /// 最低持有量
        min_balance: u64,
        /// Token 合约或资产 ID
        asset_id: String,
    },
}

/// 群配置（Agent 为唯一数据源，通过 Gossip 同步到所有 Node）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    /// 配置版本号（单调递增）
    pub version: u64,
    /// Bot ID 哈希
    pub bot_id_hash: String,
    /// 入群审批策略
    pub join_policy: JoinApprovalPolicy,
    /// 是否过滤链接
    pub filter_links: bool,
    /// 是否限制 @everyone
    pub restrict_mentions: bool,
    /// 限流: 每分钟最大消息数（0 = 不限）
    pub rate_limit_per_minute: u16,
    /// 自动禁言时长（秒，触发限流后）
    pub auto_mute_duration: u64,
    /// 新成员限制时长（秒，0 = 不限制）
    pub new_member_restrict_duration: u64,
    /// 欢迎消息（空 = 不发送）
    pub welcome_message: String,
    /// 白名单用户 ID 哈希列表
    pub whitelist: Vec<String>,
    /// 管理员列表（平台 user_id 哈希）
    pub admins: Vec<String>,
    /// 静默时段开始（UTC 小时，None = 不启用）
    pub quiet_hours_start: Option<u8>,
    /// 静默时段结束（UTC 小时）
    pub quiet_hours_end: Option<u8>,
    /// 最后更新时间戳
    pub updated_at: u64,

    // ═══ Bot 功能扩展字段 ═══

    /// 防刷屏阈值（N 条/时间窗口，0 = 关闭）
    #[serde(default)]
    pub antiflood_limit: u16,
    /// 防刷屏时间窗口（秒，默认 10）
    #[serde(default = "default_antiflood_window")]
    pub antiflood_window: u16,
    /// 防刷屏动作
    #[serde(default)]
    pub antiflood_action: FloodAction,

    /// 警告上限（达到后执行 warn_action，默认 3）
    #[serde(default = "default_warn_limit")]
    pub warn_limit: u8,
    /// 超限动作
    #[serde(default)]
    pub warn_action: WarnAction,

    /// 黑名单关键词列表
    #[serde(default)]
    pub blacklist_words: Vec<String>,
    /// 黑名单匹配模式
    #[serde(default)]
    pub blacklist_mode: BlacklistMode,
    /// 黑名单触发动作
    #[serde(default)]
    pub blacklist_action: BlacklistAction,

    /// 锁定的消息类型列表
    #[serde(default)]
    pub lock_types: Vec<LockType>,

    /// 反垃圾检测开关
    #[serde(default)]
    pub spam_detection_enabled: bool,
    /// 反垃圾：最大 emoji 数（0 = 不限）
    #[serde(default)]
    pub spam_max_emoji: u8,
    /// 反垃圾：只检查新成员的前 N 条消息（0 = 检查所有）
    #[serde(default)]
    pub spam_first_messages_only: u8,
}

fn default_antiflood_window() -> u16 { 10 }
fn default_warn_limit() -> u8 { 3 }

/// 签名的群配置（含 Agent Ed25519 签名，保证完整性）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedGroupConfig {
    /// 群配置
    pub config: GroupConfig,
    /// Agent 的 Ed25519 签名 (hex)
    pub signature: String,
    /// 签名者的公钥 (hex)
    pub signer_public_key: String,
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
