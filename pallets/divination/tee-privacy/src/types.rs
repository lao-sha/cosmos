//! # TEE 隐私计算模块 - 类型定义
//!
//! 本模块定义了 TEE 隐私计算系统所需的所有核心数据结构。
//!
//! ## 核心类型
//!
//! - `TeeType`: TEE 平台类型（Intel SGX / ARM TrustZone / AMD SEV）
//! - `TeeNodeStatus`: TEE 节点状态
//! - `TeeNode`: TEE 节点信息
//! - `TeeAttestation`: 远程认证报告
//! - `TeeComputeRequest`: 计算请求
//! - `TeeComputeResult`: 计算结果
//! - `ComputeType`: 计算类型（八字/梅花/奇门等）
//! - `ComputationProof`: 计算证明
//! - `EncryptedData`: 加密数据

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_std::vec::Vec;

// ============================================================================
// TEE 平台类型
// ============================================================================

/// TEE 平台类型
///
/// 支持多种可信执行环境
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum TeeType {
    /// Intel SGX (EPID 认证)
    #[default]
    IntelSgx = 0,
    /// Intel SGX (DCAP 认证)
    IntelSgxDcap = 1,
    /// ARM TrustZone
    ArmTrustZone = 2,
    /// AMD SEV
    AmdSev = 3,
    /// RISC-V Keystone
    RiscVKeystone = 4,
}

// ============================================================================
// TEE 节点状态
// ============================================================================

/// TEE 节点状态
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum TeeNodeStatus {
    /// 待验证
    #[default]
    Pending = 0,
    /// 活跃
    Active = 1,
    /// 暂停
    Suspended = 2,
    /// 已注销
    Deregistered = 3,
}

// ============================================================================
// 请求状态
// ============================================================================

/// 计算请求状态
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum RequestStatus {
    /// 待处理
    #[default]
    Pending = 0,
    /// 处理中
    Processing = 1,
    /// 已完成
    Completed = 2,
    /// 已失败
    Failed = 3,
    /// 已超时
    Timeout = 4,
}

// ============================================================================
// 失败原因
// ============================================================================

/// 计算失败原因
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum FailureReason {
    /// 未知错误
    #[default]
    Unknown = 0,
    /// 输入数据无效
    InvalidInput = 1,
    /// 解密失败
    DecryptionFailed = 2,
    /// 计算错误
    ComputationError = 3,
    /// 加密失败
    EncryptionFailed = 4,
    /// 证明生成失败
    ProofGenerationFailed = 5,
    /// 节点故障
    NodeFailure = 6,
    /// 超时
    Timeout = 7,
}

// ============================================================================
// 计算类型
// ============================================================================

/// 计算类型
///
/// 定义支持的占卜计算类型
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub enum ComputeType {
    /// 八字命理
    BaZi(BaZiParams),
    /// 梅花易数
    MeiHua(MeiHuaParams),
    /// 奇门遁甲
    QiMen(QiMenParams),
    /// 六爻占卜
    LiuYao(LiuYaoParams),
    /// 紫微斗数
    ZiWei(ZiWeiParams),
    /// 塔罗占卜
    Tarot(TarotParams),
    /// 大六壬
    DaLiuRen(DaLiuRenParams),
    /// 小六壬
    XiaoLiuRen(XiaoLiuRenParams),
}

impl Default for ComputeType {
    fn default() -> Self {
        ComputeType::BaZi(BaZiParams::default())
    }
}

// ============================================================================
// 性别
// ============================================================================

/// 性别枚举
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum Gender {
    /// 男性
    #[default]
    Male = 0,
    /// 女性
    Female = 1,
}

// ============================================================================
// 占卜参数类型
// ============================================================================

/// 八字参数
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug, Default)]
pub struct BaZiParams {
    /// 出生年
    pub year: u16,
    /// 出生月
    pub month: u8,
    /// 出生日
    pub day: u8,
    /// 出生时辰 (0-23)
    pub hour: u8,
    /// 性别
    pub gender: Gender,
    /// 经度 (用于真太阳时，单位：0.01度，可选)
    pub longitude: Option<i32>,
}

/// 梅花易数起卦方式
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum MeiHuaMethod {
    /// 时间起卦
    #[default]
    DateTime = 0,
    /// 数字起卦
    Number = 1,
    /// 文字起卦
    Text = 2,
    /// 声音起卦
    Sound = 3,
    /// 方位起卦
    Direction = 4,
}

/// 梅花易数参数
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug, Default)]
pub struct MeiHuaParams {
    /// 起卦方式
    pub method: MeiHuaMethod,
    /// 起卦数据 (最大64字节)
    pub data: BoundedVec<u8, ConstU32<64>>,
    /// 时间戳 (Unix timestamp)
    pub timestamp: u64,
}

/// 奇门遁甲盘类型
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum QiMenPanType {
    /// 转盘
    #[default]
    Zhuan = 0,
    /// 飞盘
    Fei = 1,
}

/// 奇门遁甲排盘方法
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum QiMenMethod {
    /// 拆补法
    #[default]
    ChaiBu = 0,
    /// 置闰法
    ZhiRun = 1,
    /// 茅山法
    MaoShan = 2,
}

/// 问事类型
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum QuestionType {
    /// 事业
    #[default]
    Career = 0,
    /// 财运
    Wealth = 1,
    /// 婚姻
    Marriage = 2,
    /// 健康
    Health = 3,
    /// 学业
    Study = 4,
    /// 出行
    Travel = 5,
    /// 诉讼
    Lawsuit = 6,
    /// 失物
    LostItem = 7,
    /// 其他
    Other = 255,
}

/// 奇门遁甲参数
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug, Default)]
pub struct QiMenParams {
    /// 排盘时间 (Unix timestamp)
    pub datetime: u64,
    /// 盘类型
    pub pan_type: QiMenPanType,
    /// 排盘方法
    pub method: QiMenMethod,
    /// 问事类型
    pub question_type: Option<QuestionType>,
}

/// 六爻起卦方式
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum LiuYaoMethod {
    /// 铜钱摇卦
    #[default]
    Coin = 0,
    /// 时间起卦
    DateTime = 1,
    /// 随机起卦
    Random = 2,
    /// 手动输入
    Manual = 3,
}

/// 六爻参数
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug, Default)]
pub struct LiuYaoParams {
    /// 起卦方式
    pub method: LiuYaoMethod,
    /// 六爻数据 (6个爻，每爻 0-3)
    pub yao_data: [u8; 6],
    /// 时间戳
    pub timestamp: u64,
    /// 问事类型
    pub question_type: Option<QuestionType>,
}

/// 紫微斗数参数
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug, Default)]
pub struct ZiWeiParams {
    /// 出生年 (农历)
    pub lunar_year: u16,
    /// 出生月 (农历)
    pub lunar_month: u8,
    /// 出生日 (农历)
    pub lunar_day: u8,
    /// 出生时辰 (0-11 对应子丑寅...)
    pub shi_chen: u8,
    /// 性别
    pub gender: Gender,
    /// 是否闰月
    pub is_leap_month: bool,
}

/// 塔罗牌阵类型
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum TarotSpread {
    /// 单牌
    #[default]
    Single = 0,
    /// 三牌阵
    ThreeCard = 1,
    /// 凯尔特十字
    CelticCross = 2,
    /// 马蹄形
    Horseshoe = 3,
    /// 关系阵
    Relationship = 4,
}

/// 塔罗参数
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug, Default)]
pub struct TarotParams {
    /// 牌阵类型
    pub spread: TarotSpread,
    /// 随机种子
    pub seed: [u8; 32],
    /// 时间戳
    pub timestamp: u64,
    /// 问事类型
    pub question_type: Option<QuestionType>,
}

/// 大六壬参数
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug, Default)]
pub struct DaLiuRenParams {
    /// 起课时间 (Unix timestamp)
    pub datetime: u64,
    /// 问事类型
    pub question_type: Option<QuestionType>,
}

/// 小六壬参数
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug, Default)]
pub struct XiaoLiuRenParams {
    /// 起课月
    pub month: u8,
    /// 起课日
    pub day: u8,
    /// 起课时
    pub hour: u8,
    /// 时间戳
    pub timestamp: u64,
}

// ============================================================================
// 远程认证报告
// ============================================================================

/// 远程认证报告
///
/// 存储 TEE 远程认证的关键信息
#[derive(Clone, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct TeeAttestation {
    /// TEE 类型
    pub tee_type: TeeType,
    /// MRENCLAVE (Enclave 度量值，32字节)
    pub mr_enclave: [u8; 32],
    /// MRSIGNER (签名者度量值，32字节)
    pub mr_signer: [u8; 32],
    /// ISV Product ID
    pub isv_prod_id: u16,
    /// ISV SVN (安全版本号)
    pub isv_svn: u16,
    /// 报告数据 (64字节，包含 Enclave 公钥哈希等)
    pub report_data: [u8; 64],
    /// IAS 签名 (Intel Attestation Service)
    pub ias_signature: BoundedVec<u8, ConstU32<512>>,
    /// 认证时间 (Unix timestamp)
    pub timestamp: u64,
}

impl Default for TeeAttestation {
    fn default() -> Self {
        Self {
            tee_type: TeeType::default(),
            mr_enclave: [0u8; 32],
            mr_signer: [0u8; 32],
            isv_prod_id: 0,
            isv_svn: 0,
            report_data: [0u8; 64],
            ias_signature: BoundedVec::new(),
            timestamp: 0,
        }
    }
}

// ============================================================================
// TEE 节点信息
// ============================================================================

/// TEE 节点信息
#[derive(Clone, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct TeeNode<AccountId> {
    /// 节点账户
    pub account: AccountId,
    /// Enclave 公钥 (Ed25519, 32字节)
    pub enclave_pubkey: [u8; 32],
    /// 远程认证报告
    pub attestation: TeeAttestation,
    /// 注册时间 (Unix timestamp)
    pub registered_at: u64,
    /// 状态
    pub status: TeeNodeStatus,
}

// ============================================================================
// 加密数据
// ============================================================================

/// 加密数据结构
///
/// 使用 ECDH + AES-256-GCM 加密
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct EncryptedData {
    /// 密文 (最大 64KB)
    pub ciphertext: BoundedVec<u8, ConstU32<65536>>,
    /// 临时公钥 (X25519, 32字节)
    pub ephemeral_pubkey: [u8; 32],
    /// Nonce (12字节)
    pub nonce: [u8; 12],
    /// 认证标签 (16字节)
    pub auth_tag: [u8; 16],
}

impl Default for EncryptedData {
    fn default() -> Self {
        Self {
            ciphertext: BoundedVec::new(),
            ephemeral_pubkey: [0u8; 32],
            nonce: [0u8; 12],
            auth_tag: [0u8; 16],
        }
    }
}

impl EncryptedData {
    /// 检查是否为 TEE 加密数据
    ///
    /// 通过检查临时公钥是否非零来判断
    pub fn is_tee_encrypted(&self) -> bool {
        self.ephemeral_pubkey != [0u8; 32]
    }

    /// 获取数据大小
    pub fn data_size(&self) -> usize {
        self.ciphertext.len()
    }
}

// ============================================================================
// 计算证明
// ============================================================================

/// 计算证明
///
/// 证明计算在真实 TEE 内执行
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct ComputationProof {
    /// 输入数据哈希 (SHA-256)
    pub input_hash: [u8; 32],
    /// 输出数据哈希 (SHA-256)
    pub output_hash: [u8; 32],
    /// Enclave 签名 (Ed25519, 64字节)
    pub enclave_signature: [u8; 64],
    /// 时间戳
    pub timestamp: u64,
}

impl Default for ComputationProof {
    fn default() -> Self {
        Self {
            input_hash: [0u8; 32],
            output_hash: [0u8; 32],
            enclave_signature: [0u8; 64],
            timestamp: 0,
        }
    }
}

// ============================================================================
// 计算请求
// ============================================================================

/// TEE 计算请求
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct TeeComputeRequest<AccountId, BlockNumber> {
    /// 请求 ID
    pub id: u64,
    /// 请求者
    pub requester: AccountId,
    /// 计算类型
    pub compute_type: ComputeType,
    /// 加密输入数据
    pub encrypted_input: EncryptedData,
    /// 指定的 TEE 节点 (可选，None 表示自动分配)
    pub assigned_node: Option<AccountId>,
    /// 创建区块
    pub created_at: BlockNumber,
    /// 超时区块
    pub timeout_at: BlockNumber,
    /// 状态
    pub status: RequestStatus,
}

// ============================================================================
// 计算结果
// ============================================================================

/// TEE 计算结果
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct TeeComputeResult<AccountId> {
    /// 请求 ID
    pub request_id: u64,
    /// 执行节点
    pub executor: AccountId,
    /// 加密结果
    pub encrypted_output: EncryptedData,
    /// 计算证明
    pub computation_proof: ComputationProof,
    /// 完成时间 (Unix timestamp)
    pub completed_at: u64,
}

// ============================================================================
// Runtime API 返回类型
// ============================================================================

/// TEE 节点信息 (用于 RPC 返回)
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct TeeNodeInfo {
    /// 节点地址 (字符串格式)
    pub account: Vec<u8>,
    /// Enclave 公钥
    pub enclave_pubkey: [u8; 32],
    /// TEE 类型
    pub tee_type: TeeType,
    /// 状态
    pub status: TeeNodeStatus,
    /// 注册时间
    pub registered_at: u64,
    /// MRENCLAVE
    pub mr_enclave: [u8; 32],
    /// 认证时间
    pub attestation_timestamp: u64,
}

/// 请求状态信息 (用于 RPC 返回)
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct RequestStatusInfo {
    /// 请求 ID
    pub request_id: u64,
    /// 请求者 (字符串格式)
    pub requester: Vec<u8>,
    /// 计算类型 ID
    pub compute_type_id: u8,
    /// 输入数据哈希
    pub input_hash: [u8; 32],
    /// 分配节点 (字符串格式，可选)
    pub assigned_node: Option<Vec<u8>>,
    /// 创建区块
    pub created_at: u32,
    /// 超时区块
    pub timeout_at: u32,
    /// 状态
    pub status: RequestStatus,
    /// 故障转移次数
    pub failover_count: u8,
    /// 失败原因
    pub failure_reason: Option<FailureReason>,
}

/// 认证验证结果
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug, Default)]
pub struct AttestationVerifyResult {
    /// 是否有效
    pub is_valid: bool,
    /// MRENCLAVE 是否匹配
    pub mr_enclave_match: bool,
    /// 是否过期
    pub is_expired: bool,
    /// 错误信息
    pub error_message: Option<Vec<u8>>,
}

// ============================================================================
// Phase 2: 计算请求存储类型
// ============================================================================

/// 计算请求信息（链上存储）
#[derive(Clone, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct ComputeRequestInfo<AccountId, BlockNumber> {
    /// 请求 ID
    pub id: u64,
    /// 请求者
    pub requester: AccountId,
    /// 计算类型标识 (0=BaZi, 1=MeiHua, 2=QiMen, 等)
    pub compute_type_id: u8,
    /// 加密输入数据哈希（原始数据存储在链下）
    pub input_hash: [u8; 32],
    /// 分配的 TEE 节点
    pub assigned_node: Option<AccountId>,
    /// 创建区块
    pub created_at: BlockNumber,
    /// 超时区块
    pub timeout_at: BlockNumber,
    /// 状态
    pub status: RequestStatus,
    /// 故障转移次数
    pub failover_count: u8,
    /// 失败原因（如果失败）
    pub failure_reason: Option<FailureReason>,
}

/// 计算结果信息（链上存储）
#[derive(Clone, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct ComputeResultInfo<AccountId> {
    /// 请求 ID
    pub request_id: u64,
    /// 执行节点
    pub executor: AccountId,
    /// 加密结果数据哈希（原始数据存储在链下）
    pub output_hash: [u8; 32],
    /// Enclave 签名
    pub enclave_signature: [u8; 64],
    /// 完成时间戳 (Unix timestamp)
    pub completed_at: u64,
}

// ============================================================================
// Phase 2: 经济激励类型
// ============================================================================

/// 质押信息
#[derive(Clone, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct StakeInfo<Balance, BlockNumber> {
    /// 质押金额
    pub amount: Balance,
    /// 质押开始区块
    pub staked_at: BlockNumber,
    /// 解锁区块（申请解除质押后）
    pub unlock_at: Option<BlockNumber>,
    /// 是否正在解除质押
    pub is_unbonding: bool,
}

/// 节点统计信息
#[derive(Clone, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct NodeStatistics {
    /// 完成的请求总数
    pub completed_requests: u64,
    /// 失败的请求总数
    pub failed_requests: u64,
    /// 超时的请求总数
    pub timeout_requests: u64,
    /// 被惩罚次数
    pub slash_count: u32,
    /// 累计获得奖励
    pub total_rewards: u128,
    /// 累计被惩罚金额
    pub total_slashed: u128,
    /// 平均处理时间（区块数）
    pub avg_processing_blocks: u32,
    /// 最后活跃区块
    pub last_active_block: u64,
}

// ============================================================================
// Phase 5: 批处理优化类型
// ============================================================================

/// 最大批处理请求数量
pub const MAX_BATCH_SIZE: u32 = 10;

/// 批处理请求项
#[derive(Clone, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct BatchRequestItem {
    /// 计算类型 ID (0=BaZi, 1=MeiHua, 等)
    pub compute_type_id: u8,
    /// 加密输入数据的哈希
    pub input_hash: [u8; 32],
}

impl Default for BatchRequestItem {
    fn default() -> Self {
        Self {
            compute_type_id: 0,
            input_hash: [0u8; 32],
        }
    }
}

/// 批处理结果项
#[derive(Clone, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct BatchResultItem {
    /// 对应的请求 ID
    pub request_id: u64,
    /// 加密输出数据的哈希
    pub output_hash: [u8; 32],
    /// Enclave 签名
    pub enclave_signature: [u8; 64],
}

impl Default for BatchResultItem {
    fn default() -> Self {
        Self {
            request_id: 0,
            output_hash: [0u8; 32],
            enclave_signature: [0u8; 64],
        }
    }
}

/// 批处理状态
#[derive(Clone, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct BatchStatus {
    /// 批处理中的请求总数
    pub total_requests: u32,
    /// 已完成的请求数
    pub completed_requests: u32,
    /// 失败的请求数
    pub failed_requests: u32,
    /// 批处理开始区块
    pub started_at: u64,
}

impl Default for BatchStatus {
    fn default() -> Self {
        Self {
            total_requests: 0,
            completed_requests: 0,
            failed_requests: 0,
            started_at: 0,
        }
    }
}

// ============================================================================
// Phase 5: 审计日志类型
// ============================================================================

/// 最大审计日志条目数
pub const MAX_AUDIT_LOGS: u32 = 1000;

/// 审计事件类型
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    codec::DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
)]
pub enum AuditEventType {
    /// TEE 节点注册
    NodeRegistered = 0,
    /// TEE 节点注销
    NodeDeregistered = 1,
    /// 认证更新
    AttestationUpdated = 2,
    /// 计算请求提交
    ComputeRequestSubmitted = 3,
    /// 计算结果提交
    ComputeResultSubmitted = 4,
    /// 请求超时
    RequestTimeout = 5,
    /// 节点惩罚
    NodeSlashed = 6,
    /// MRENCLAVE 添加
    MrEnclaveAdded = 7,
    /// MRENCLAVE 移除
    MrEnclaveRemoved = 8,
    /// MRSIGNER 添加
    MrSignerAdded = 9,
    /// MRSIGNER 移除
    MrSignerRemoved = 10,
    /// 质押
    Staked = 11,
    /// 解除质押
    Unstaked = 12,
    /// 批量请求提交
    BatchRequestsSubmitted = 13,
    /// 批量结果提交
    BatchResultsSubmitted = 14,
}

impl Default for AuditEventType {
    fn default() -> Self {
        AuditEventType::NodeRegistered
    }
}

/// 审计日志条目
#[derive(Clone, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct AuditLogEntry<AccountId, BlockNumber> {
    /// 日志 ID
    pub id: u64,
    /// 事件类型
    pub event_type: AuditEventType,
    /// 相关账户
    pub account: AccountId,
    /// 区块号
    pub block_number: BlockNumber,
    /// 时间戳 (Unix timestamp)
    pub timestamp: u64,
    /// 相关数据哈希 (用于链下查询详细信息)
    pub data_hash: [u8; 32],
    /// 操作是否成功
    pub success: bool,
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tee_type_values() {
        assert_eq!(TeeType::IntelSgx as u8, 0);
        assert_eq!(TeeType::IntelSgxDcap as u8, 1);
        assert_eq!(TeeType::ArmTrustZone as u8, 2);
        assert_eq!(TeeType::AmdSev as u8, 3);
        assert_eq!(TeeType::RiscVKeystone as u8, 4);
    }

    #[test]
    fn test_node_status_values() {
        assert_eq!(TeeNodeStatus::Pending as u8, 0);
        assert_eq!(TeeNodeStatus::Active as u8, 1);
        assert_eq!(TeeNodeStatus::Suspended as u8, 2);
        assert_eq!(TeeNodeStatus::Deregistered as u8, 3);
    }

    #[test]
    fn test_request_status_values() {
        assert_eq!(RequestStatus::Pending as u8, 0);
        assert_eq!(RequestStatus::Processing as u8, 1);
        assert_eq!(RequestStatus::Completed as u8, 2);
        assert_eq!(RequestStatus::Failed as u8, 3);
        assert_eq!(RequestStatus::Timeout as u8, 4);
    }

    #[test]
    fn test_encrypted_data_default() {
        let data = EncryptedData::default();
        assert!(!data.is_tee_encrypted());
        assert_eq!(data.data_size(), 0);
    }

    #[test]
    fn test_encrypted_data_is_tee_encrypted() {
        let mut data = EncryptedData::default();
        data.ephemeral_pubkey = [1u8; 32];
        assert!(data.is_tee_encrypted());
    }

    #[test]
    fn test_bazi_params_default() {
        let params = BaZiParams::default();
        assert_eq!(params.year, 0);
        assert_eq!(params.gender, Gender::Male);
        assert!(params.longitude.is_none());
    }

    #[test]
    fn test_compute_type_default() {
        let compute_type = ComputeType::default();
        match compute_type {
            ComputeType::BaZi(_) => {}
            _ => panic!("Default should be BaZi"),
        }
    }

    #[test]
    fn test_attestation_default() {
        let att = TeeAttestation::default();
        assert_eq!(att.tee_type, TeeType::IntelSgx);
        assert_eq!(att.mr_enclave, [0u8; 32]);
        assert_eq!(att.timestamp, 0);
    }

    #[test]
    fn test_computation_proof_default() {
        let proof = ComputationProof::default();
        assert_eq!(proof.input_hash, [0u8; 32]);
        assert_eq!(proof.output_hash, [0u8; 32]);
        assert_eq!(proof.enclave_signature, [0u8; 64]);
    }
}
