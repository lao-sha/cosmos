/// Stardust智能群聊系统 - 类型定义
///
/// 定义了所有核心数据结构和枚举类型

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    pallet_prelude::*,
    BoundedVec,
};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

// ========== 量子抗性密钥类型 ==========

/// Kyber私钥 (2400字节)
pub type KyberSecretKey = BoundedVec<u8, ConstU32<2400>>;

/// Kyber公钥 (1184字节)
pub type KyberPublicKey = BoundedVec<u8, ConstU32<1184>>;

/// Kyber密文 (1088字节)
pub type KyberCiphertext = BoundedVec<u8, ConstU32<1088>>;

/// Kyber共享密钥 (32字节)
pub type KyberSharedSecret = BoundedVec<u8, ConstU32<32>>;

/// Dilithium私钥 (4000字节)
pub type DilithiumSecretKey = BoundedVec<u8, ConstU32<4000>>;

/// Dilithium公钥 (1952字节)
pub type DilithiumPublicKey = BoundedVec<u8, ConstU32<1952>>;

/// Dilithium签名 (3293字节)
pub type DilithiumSignature = BoundedVec<u8, ConstU32<3293>>;

/// 群组ID类型
pub type GroupId = u64;

/// 消息ID类型
pub type MessageId = u64;

/// 临时消息ID类型（用于乐观UI更新）
pub type TempMessageId = [u8; 16];

/// 群组信息结构
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct GroupInfo<T: frame_system::Config> {
    /// 群组创建者
    pub creator: T::AccountId,
    /// 群组名称
    pub name: BoundedVec<u8, ConstU32<64>>,
    /// 群组描述
    pub description: Option<BoundedVec<u8, ConstU32<512>>>,
    /// 加密模式
    pub encryption_mode: EncryptionMode,
    /// 最大成员数
    pub max_members: u32,
    /// 当前成员数
    pub current_member_count: u32,
    /// 创建时间戳
    pub created_at: u64,
    /// 是否公开可见
    pub is_public: bool,
    /// 群组是否活跃
    pub is_active: bool,
    /// 紧急状态（如果有）
    pub emergency_state: Option<EmergencyState<T>>,
    /// AI设置
    pub ai_settings: AISettings,
}

/// 群组成员信息
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct GroupMemberInfo<T: frame_system::Config> {
    /// 成员账户ID
    pub account_id: T::AccountId,
    /// 成员角色
    pub role: GroupRole,
    /// 加入时间戳
    pub joined_at: u64,
    /// 成员权限
    pub permissions: GroupPermissions,
    /// 加密密钥份额
    pub encryption_key_share: BoundedVec<u8, ConstU32<128>>,
    /// 最后活动时间
    pub last_activity: u64,
}

/// 群组角色枚举
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum GroupRole {
    /// 管理员（完全控制权限）
    Admin,
    /// 版主（管理权限）
    Moderator,
    /// 普通成员
    Member,
    /// 受限成员（只读权限）
    Restricted,
}

/// 群组权限位掩码
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct GroupPermissions {
    pub flags: u32,
}

impl GroupPermissions {
    /// 发送消息权限
    pub const SEND_MESSAGES: u32 = 1 << 0;
    /// 删除自己的消息权限
    pub const DELETE_OWN_MESSAGES: u32 = 1 << 1;
    /// 删除他人消息权限
    pub const DELETE_OTHERS_MESSAGES: u32 = 1 << 2;
    /// 邀请成员权限
    pub const INVITE_MEMBERS: u32 = 1 << 3;
    /// 踢出成员权限
    pub const KICK_MEMBERS: u32 = 1 << 4;
    /// 更改群组设置权限
    pub const CHANGE_GROUP_SETTINGS: u32 = 1 << 5;
    /// 更改加密模式权限
    pub const CHANGE_ENCRYPTION_MODE: u32 = 1 << 6;
    /// 管理角色权限
    pub const MANAGE_ROLES: u32 = 1 << 7;
    /// 查看成员列表权限
    pub const VIEW_MEMBER_LIST: u32 = 1 << 8;
    /// 发送文件权限
    pub const SEND_FILES: u32 = 1 << 9;

    /// 所有权限
    pub fn all_permissions() -> Self {
        Self { flags: u32::MAX }
    }

    /// 默认成员权限
    pub fn default_member_permissions() -> Self {
        Self {
            flags: Self::SEND_MESSAGES |
                   Self::DELETE_OWN_MESSAGES |
                   Self::VIEW_MEMBER_LIST |
                   Self::SEND_FILES,
        }
    }

    /// 版主权限
    pub fn moderator_permissions() -> Self {
        Self {
            flags: Self::SEND_MESSAGES |
                   Self::DELETE_OWN_MESSAGES |
                   Self::DELETE_OTHERS_MESSAGES |
                   Self::INVITE_MEMBERS |
                   Self::KICK_MEMBERS |
                   Self::VIEW_MEMBER_LIST |
                   Self::SEND_FILES,
        }
    }

    /// 检查是否有特定权限
    pub fn has_permission(&self, permission: u32) -> bool {
        (self.flags & permission) != 0
    }
}

/// 加密模式枚举
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum EncryptionMode {
    /// 军用级加密：量子抗性 + 多层加密 + 完美前向安全
    Military,
    /// 商用级加密：标准端到端加密 + 群密钥管理
    Business,
    /// 选择性加密：用户主导选择 + 智能建议
    Selective,
    /// 透明公开：完全公开 + 链上直接存储
    Transparent,
}

impl Default for EncryptionMode {
    fn default() -> Self {
        EncryptionMode::Business
    }
}

/// 消息类型枚举
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum MessageType {
    /// 普通文本消息
    Text,
    /// 图片消息
    Image,
    /// 视频消息
    Video,
    /// 音频消息
    Audio,
    /// 文件消息
    File,
    /// 系统消息
    System,
    /// 临时消息（阅后即焚）
    Ephemeral,
    /// 定时消息
    Temporary,
}

/// 群组消息元数据
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct GroupMessageMeta<T: frame_system::Config> {
    /// 消息ID
    pub id: MessageId,
    /// 群组ID
    pub group_id: GroupId,
    /// 发送者
    pub sender: T::AccountId,
    /// 消息内容或IPFS CID
    pub content: BoundedVec<u8, ConstU32<2048>>,
    /// 消息类型
    pub message_type: MessageType,
    /// 加密模式
    pub encryption_mode: EncryptionMode,
    /// 存储层级
    pub storage_tier: StorageTier,
    /// 发送时间戳
    pub sent_at: u64,
    /// 临时消息ID（用于乐观更新）
    pub temp_id: Option<TempMessageId>,
    /// 确认状态
    pub confirmation_status: ConfirmationStatus,
    /// AI分析结果
    pub ai_analysis: Option<AIAnalysisResult>,
    /// 访问次数
    pub access_count: u32,
    /// 最后访问时间
    pub last_accessed: u64,
}

/// 存储层级枚举
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum StorageTier {
    /// 链上存储：高可靠性，高成本，快速访问
    OnChain,
    /// IPFS存储：去中心化，中成本，中等速度
    IPFS,
    /// 混合存储：元数据链上，内容IPFS
    Hybrid,
    /// 临时存储：高性能，低成本，自动清理
    Temporary,
}

/// 确认状态枚举
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ConfirmationStatus {
    /// 乐观确认（前端显示，等待链上确认）
    Optimistic,
    /// 已确认（链上确认完成）
    Confirmed,
    /// 失败（确认失败）
    Failed,
}

/// 乐观消息状态
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct OptimisticMessageState<T: frame_system::Config> {
    /// 临时消息ID
    pub temp_id: TempMessageId,
    /// 真实消息ID（确认后）
    pub real_message_id: Option<MessageId>,
    /// 当前状态
    pub status: OptimisticStatus,
    /// 进度百分比（0-100）
    pub progress: u8,
    /// 当前处理阶段
    pub stage: ProcessingStage,
    /// 创建时间
    pub created_at: u64,
    /// 预估确认时间
    pub estimated_confirm_time: u64,
    /// 错误信息（如果有）
    pub error_info: Option<BoundedVec<u8, ConstU32<256>>>,
    /// 重试次数
    pub retry_count: u8,
}

/// 乐观消息状态枚举
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum OptimisticStatus {
    /// 处理中
    Processing,
    /// 已确认
    Confirmed,
    /// 失败
    Failed,
    /// 重试中
    Retrying,
    /// 已过期
    Expired,
}

/// 处理阶段枚举
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ProcessingStage {
    /// 开始处理
    Starting,
    /// 加密中
    Encrypting,
    /// 上传IPFS中
    UploadingIPFS,
    /// 提交交易中
    SubmittingTransaction,
    /// 等待确认中
    WaitingConfirmation,
    /// 最终处理中
    Finalizing,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
}

/// 加密配置
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct EncryptionConfig<T: frame_system::Config> {
    /// 加密模式
    pub mode: EncryptionMode,
    /// 主密钥
    pub master_key: BoundedVec<u8, ConstU32<128>>,
    /// 管理员密钥份额
    pub admin_key_share: BoundedVec<u8, ConstU32<128>>,
    /// 密钥轮换间隔（秒）
    pub key_rotation_interval: u64,
    /// 是否量子抗性
    pub quantum_resistant: bool,
    /// 是否完美前向安全
    pub perfect_forward_secrecy: bool,
    /// 附加加密层数
    pub additional_layers: u8,
    /// 创建时间
    pub created_at: u64,
}

/// AI决策类型
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum AIDecisionType {
    /// 加密模式选择
    EncryptionMode,
    /// 存储策略选择
    StorageStrategy,
    /// 内容敏感性分析
    ContentSensitivity,
    /// 用户行为预测
    UserBehaviorPrediction,
}

/// AI决策结果
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct AIDecisionResult<T: frame_system::Config> {
    /// 推荐的加密模式
    pub recommended_mode: EncryptionMode,
    /// 置信度（0.0-1.0）
    pub confidence: f32,
    /// 决策理由
    pub reasoning: BoundedVec<u8, ConstU32<512>>,
    /// 可选选项
    pub alternative_options: BoundedVec<EncryptionMode, ConstU32<4>>,
}

/// AI分析结果
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AIAnalysisResult {
    /// 内容敏感性评分（0.0-1.0）
    pub sensitivity_score: u8, // 存储为0-100的整数
    /// 检测到的关键词标签
    pub detected_keywords: BoundedVec<BoundedVec<u8, ConstU32<32>>, ConstU32<10>>,
    /// 推荐的存储策略
    pub recommended_storage: StorageTier,
    /// 分析时间戳
    pub analyzed_at: u64,
}

/// AI设置
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AISettings {
    /// 启用AI自动加密选择
    pub auto_encryption_enabled: bool,
    /// 启用AI内容分析
    pub content_analysis_enabled: bool,
    /// 启用AI存储优化
    pub storage_optimization_enabled: bool,
    /// AI学习模式
    pub learning_mode: AILearningMode,
    /// 敏感性阈值
    pub sensitivity_threshold: u8, // 0-100
}

impl Default for AISettings {
    fn default() -> Self {
        Self {
            auto_encryption_enabled: true,
            content_analysis_enabled: true,
            storage_optimization_enabled: true,
            learning_mode: AILearningMode::Adaptive,
            sensitivity_threshold: 50,
        }
    }
}

/// AI学习模式
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum AILearningMode {
    /// 保守模式：优先安全
    Conservative,
    /// 平衡模式：安全与性能平衡
    Balanced,
    /// 激进模式：优先性能
    Aggressive,
    /// 自适应模式：根据使用模式自动调整
    Adaptive,
}

/// 存储策略
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct StorageStrategy<T: frame_system::Config> {
    /// 主要存储层
    pub primary_tier: StorageTier,
    /// 备份存储层
    pub backup_tier: Option<StorageTier>,
    /// 复制因子
    pub replication_factor: u32,
    /// 生存时间（秒，None表示永久）
    pub ttl_seconds: Option<u64>,
    /// 是否启用压缩
    pub compression_enabled: bool,
    /// 是否启用自动迁移
    pub auto_migration_enabled: bool,
}

impl<T: frame_system::Config> StorageStrategy<T> {
    /// 获取初始存储层级
    pub fn initial_tier(&self) -> StorageTier {
        self.primary_tier.clone()
    }
}

/// 存储统计信息
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct StorageStats<T: frame_system::Config> {
    /// 总消息数
    pub total_messages: u32,
    /// 链上存储消息数
    pub on_chain_storage: u32,
    /// IPFS存储消息数
    pub ipfs_storage: u32,
    /// 缓存存储消息数
    pub cached_storage: u32,
    /// 临时存储消息数
    pub temporary_storage: u32,
    /// 最后清理时间
    pub last_cleanup: u64,
}

impl<T: frame_system::Config> Default for StorageStats<T> {
    fn default() -> Self {
        Self {
            total_messages: 0,
            on_chain_storage: 0,
            ipfs_storage: 0,
            cached_storage: 0,
            temporary_storage: 0,
            last_cleanup: 0,
        }
    }
}

/// 用户行为数据
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct UserBehaviorData<T: frame_system::Config> {
    /// 安全偏好评分（0.0-1.0）
    pub security_preference: f32,
    /// 偏好的加密模式
    pub preferred_encryption_mode: EncryptionMode,
    /// 响应时间偏好
    pub response_time_preference: ResponseTimePreference,
    /// 平均消息长度
    pub average_message_length: u32,
    /// 活跃时间段
    pub active_hours: BoundedVec<u8, ConstU32<24>>, // 0-23小时
    /// 最常用的消息类型
    pub most_used_message_type: MessageType,
    /// 总发送消息数
    pub total_messages_sent: u32,
    /// 数据更新时间
    pub updated_at: u64,
}

/// 响应时间偏好
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ResponseTimePreference {
    /// 快速响应优先
    Fast,
    /// 平衡
    Balanced,
    /// 安全优先
    Secure,
}

/// 紧急状态
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct EmergencyState<T: frame_system::Config> {
    /// 紧急状态原因
    pub reason: EmergencyReason,
    /// 激活时间
    pub activated_at: u64,
    /// 激活者
    pub activated_by: T::AccountId,
}

/// 紧急状态原因
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum EmergencyReason {
    /// 安全威胁检测
    SecurityThreat,
    /// 数据泄露风险
    DataLeakRisk,
    /// 恶意行为检测
    MaliciousActivity,
    /// 系统漏洞发现
    SystemVulnerability,
    /// 管理员手动激活
    ManualActivation,
}

/// 快照状态（用于数据恢复）
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct SnapshotState<T: frame_system::Config> {
    /// 快照ID
    pub snapshot_id: [u8; 32],
    /// 群组状态哈希
    pub group_state_hash: [u8; 32],
    /// 消息状态哈希
    pub message_state_hash: [u8; 32],
    /// 快照创建时间
    pub created_at: u64,
    /// 包含的消息数量
    pub message_count: u32,
    /// 快照大小（字节）
    pub snapshot_size: u64,
}

/// 性能监控指标
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct PerformanceMetrics {
    /// 平均消息处理时间（毫秒）
    pub average_processing_time: u32,
    /// 成功率（0-100）
    pub success_rate: u8,
    /// 错误计数
    pub error_count: u32,
    /// 重试计数
    pub retry_count: u32,
    /// 最后更新时间
    pub last_updated: u64,
}

// ========== 量子抗性安全结构 ==========

/// 量子密钥对
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct QuantumKeyPair {
    /// Kyber密钥封装机制密钥对
    pub kyber_keypair: KyberKeyPair,
    /// Dilithium数字签名密钥对
    pub dilithium_keypair: DilithiumKeyPair,
    /// 密钥生成时间
    pub created_at: u64,
    /// 密钥版本
    pub version: u32,
}

/// Kyber密钥对
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct KyberKeyPair {
    pub secret_key: KyberSecretKey,
    pub public_key: KyberPublicKey,
}

/// Dilithium密钥对
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct DilithiumKeyPair {
    pub secret_key: DilithiumSecretKey,
    pub public_key: DilithiumPublicKey,
}

/// 量子加密配置
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct QuantumEncryptionConfig {
    /// 是否启用量子抗性加密
    pub enabled: bool,
    /// 密钥轮换间隔（块数）
    pub key_rotation_interval: u32,
    /// 侧信道攻击防护等级
    pub side_channel_protection: SideChannelProtectionLevel,
    /// 是否启用完美前向安全
    pub perfect_forward_secrecy: bool,
    /// 量子随机数生成器配置
    pub quantum_rng_config: QuantumRngConfig,
}

impl Default for QuantumEncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            key_rotation_interval: 1000, // 每1000个块轮换一次密钥
            side_channel_protection: SideChannelProtectionLevel::High,
            perfect_forward_secrecy: true,
            quantum_rng_config: QuantumRngConfig::default(),
        }
    }
}

/// 侧信道攻击防护等级
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum SideChannelProtectionLevel {
    /// 最低防护（仅基本保护）
    Low,
    /// 中等防护（标准保护措施）
    Medium,
    /// 高等防护（完整保护套件）
    High,
    /// 最高防护（军用级保护）
    Military,
}

/// 量子随机数生成器配置
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct QuantumRngConfig {
    /// 是否启用量子随机数生成器
    pub enabled: bool,
    /// 熵源数量
    pub entropy_sources: u8,
    /// 随机数质量检查
    pub quality_check_enabled: bool,
    /// 回退到伪随机数的条件
    pub fallback_threshold: u8,
}

impl Default for QuantumRngConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            entropy_sources: 3, // 使用多个熵源
            quality_check_enabled: true,
            fallback_threshold: 80, // 质量低于80%时回退
        }
    }
}

/// 混合密文结构
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct HybridCiphertext {
    /// Kyber封装的密文
    pub kyber_ciphertext: KyberCiphertext,
    /// AES-GCM加密的数据
    pub aes_ciphertext: BoundedVec<u8, ConstU32<8192>>,
    /// AES-GCM随机数
    pub aes_nonce: BoundedVec<u8, ConstU32<12>>,
    /// AES-GCM认证标签
    pub aes_tag: BoundedVec<u8, ConstU32<16>>,
}

/// 量子数字信封
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct QuantumEnvelope {
    /// 混合加密的密文
    pub ciphertext: HybridCiphertext,
    /// Dilithium数字签名
    pub signature: DilithiumSignature,
    /// 发送方Dilithium公钥
    pub sender_public_key: DilithiumPublicKey,
    /// 时间戳（防重放攻击）
    pub timestamp: u64,
    /// 多重校验和
    pub checksums: MultipleChecksums,
}

/// 多重校验和结构
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct MultipleChecksums {
    /// SHA3-256哈希
    pub sha3_256: BoundedVec<u8, ConstU32<32>>,
    /// BLAKE2b-256哈希
    pub blake2b_256: BoundedVec<u8, ConstU32<32>>,
    /// Keccak-256哈希
    pub keccak_256: BoundedVec<u8, ConstU32<32>>,
    /// CRC32校验和
    pub crc32: u32,
}

/// 密钥轮换记录
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct KeyRotationRecord<T: frame_system::Config> {
    /// 轮换时间
    pub rotation_time: u64,
    /// 旧密钥指纹
    pub old_key_fingerprint: [u8; 32],
    /// 新密钥指纹
    pub new_key_fingerprint: [u8; 32],
    /// 轮换原因
    pub reason: KeyRotationReason,
    /// 执行轮换的账户
    pub rotated_by: T::AccountId,
}

/// 密钥轮换原因
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum KeyRotationReason {
    /// 定期轮换
    Scheduled,
    /// 安全事件触发
    SecurityEvent,
    /// 密钥泄露
    KeyCompromise,
    /// 系统升级
    SystemUpgrade,
    /// 管理员手动
    Manual,
}

/// 量子安全事件
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct QuantumSecurityEvent<T: frame_system::Config> {
    /// 事件类型
    pub event_type: QuantumSecurityEventType,
    /// 严重等级
    pub severity: SecuritySeverity,
    /// 检测时间
    pub detected_at: u64,
    /// 相关群组（如果有）
    pub group_id: Option<GroupId>,
    /// 相关账户（如果有）
    pub account_id: Option<T::AccountId>,
    /// 事件描述
    pub description: BoundedVec<u8, ConstU32<512>>,
    /// 是否已处理
    pub handled: bool,
}

/// 量子安全事件类型
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum QuantumSecurityEventType {
    /// 量子攻击尝试检测
    QuantumAttackDetected,
    /// 侧信道攻击检测
    SideChannelAttackDetected,
    /// 密钥泄露检测
    KeyLeakageDetected,
    /// 时间攻击检测
    TimingAttackDetected,
    /// 数据完整性违反
    IntegrityViolation,
    /// 异常加密活动
    AbnormalCryptoActivity,
}

/// 安全严重等级
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum SecuritySeverity {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 严重
    Critical,
}

/// 量子抗性状态
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct QuantumResistanceStatus {
    /// 是否启用量子抗性
    pub enabled: bool,
    /// 当前密钥版本
    pub current_key_version: u32,
    /// 最后密钥轮换时间
    pub last_key_rotation: u64,
    /// 总轮换次数
    pub total_rotations: u32,
    /// 安全事件计数
    pub security_events_count: u32,
    /// 最后安全检查时间
    pub last_security_check: u64,
}

// ========== 插件生态系统额外类型 ==========

/// 插件执行记录
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct PluginExecutionRecord<T: frame_system::Config> {
    /// 执行时间戳
    pub timestamp: u64,
    /// 钩子类型
    pub hook_type: PluginHookType,
    /// 执行是否成功
    pub success: bool,
    /// 执行时长（毫秒）
    pub execution_time: u32,
    /// 错误信息（如果有）
    pub error_message: Option<BoundedVec<u8, ConstU32<256>>>,
}

/// 全局插件配置
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct GlobalPluginConfig {
    /// 是否启用插件系统
    pub plugins_enabled: bool,
    /// 最大同时运行的插件数量
    pub max_concurrent_plugins: u32,
    /// 单个插件最大内存限制（字节）
    pub max_plugin_memory: u64,
    /// 单个插件最大执行时间（毫秒）
    pub max_execution_time: u32,
    /// 是否允许未验证的插件
    pub allow_unverified_plugins: bool,
    /// 插件存储空间限制（字节）
    pub max_plugin_storage: u64,
}

impl Default for GlobalPluginConfig {
    fn default() -> Self {
        Self {
            plugins_enabled: true,
            max_concurrent_plugins: 20,
            max_plugin_memory: 50 * 1024 * 1024, // 50MB
            max_execution_time: 5000, // 5秒
            allow_unverified_plugins: false,
            max_plugin_storage: 1024 * 1024 * 1024, // 1GB
        }
    }
}