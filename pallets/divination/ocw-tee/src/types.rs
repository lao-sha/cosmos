//! # OCW + TEE 通用类型定义
//!
//! 本模块定义所有占卜模块共享的 OCW + TEE 相关类型。

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_std::prelude::*;

// ==================== 隐私模式 ====================

/// 隐私模式（OCW + TEE 架构专用）
///
/// 与 `pallet-divination-common::deposit::PrivacyMode` 保持兼容，
/// 但专门用于 OCW + TEE 架构的数据处理流程。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
pub enum PrivacyMode {
    /// 公开模式：链上存储索引，IPFS 明文
    /// - 前端明文提交
    /// - OCW 直接计算
    /// - IPFS 存储明文 JSON
    #[default]
    Public = 0,

    /// 加密模式：链上存储索引，IPFS 加密
    /// - 前端加密提交
    /// - TEE 解密计算
    /// - IPFS 存储加密 JSON
    Encrypted = 1,

    /// 私密模式：链上不存储索引，IPFS 加密
    /// - 前端加密提交
    /// - TEE 解密计算
    /// - 链上不存储任何敏感索引
    /// - IPFS 存储加密 JSON
    Private = 2,
}

impl PrivacyMode {
    /// 获取隐私模式名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Encrypted => "encrypted",
            Self::Private => "private",
        }
    }

    /// 是否需要 TEE 处理
    pub fn requires_tee(&self) -> bool {
        !matches!(self, Self::Public)
    }

    /// 是否在链上存储索引
    pub fn stores_index_on_chain(&self) -> bool {
        !matches!(self, Self::Private)
    }

    /// 从 u8 转换
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Public),
            1 => Some(Self::Encrypted),
            2 => Some(Self::Private),
            _ => None,
        }
    }
}

// ==================== 请求状态 ====================

/// 请求状态（所有占卜模块通用）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
pub enum RequestStatus {
    /// 待处理 - 请求已提交，等待 OCW 处理
    #[default]
    Pending = 0,

    /// 处理中 - OCW 已接收，正在处理
    Processing = 1,

    /// 已完成 - 计算完成，结果已存储
    Completed = 2,

    /// 失败 - 处理失败（可重试）
    Failed = 3,

    /// 超时 - 请求超时未处理
    Timeout = 4,
}

impl RequestStatus {
    /// 获取状态名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Timeout => "timeout",
        }
    }

    /// 是否为终态
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Completed | Self::Timeout)
    }

    /// 是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Failed)
    }
}

// ==================== 加密数据 ====================

/// 加密数据结构（通用）
///
/// 使用 X25519 + XSalsa20-Poly1305 或 AES-256-GCM 加密
#[derive(Clone, Debug, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxLen))]
pub struct EncryptedData<MaxLen: Get<u32>> {
    /// 密文
    pub ciphertext: BoundedVec<u8, MaxLen>,
    /// 随机数（24 字节，兼容 XSalsa20）
    pub nonce: [u8; 24],
    /// 发送方公钥（用于 ECDH 密钥交换）
    pub sender_pubkey: [u8; 32],
}

impl<MaxLen: Get<u32>> EncryptedData<MaxLen> {
    /// 从 AES-GCM 格式转换（12 字节 nonce）
    pub fn from_aes_gcm(
        ciphertext: Vec<u8>,
        nonce_12: [u8; 12],
        sender_pubkey: [u8; 32],
    ) -> Result<Self, &'static str> {
        let mut nonce_24 = [0u8; 24];
        nonce_24[..12].copy_from_slice(&nonce_12);

        let bounded_ciphertext: BoundedVec<u8, MaxLen> = ciphertext
            .try_into()
            .map_err(|_| "Ciphertext too long")?;

        Ok(Self {
            ciphertext: bounded_ciphertext,
            nonce: nonce_24,
            sender_pubkey,
        })
    }

    /// 获取 12 字节 nonce（用于 AES-GCM）
    pub fn nonce_12(&self) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&self.nonce[..12]);
        nonce
    }
}

/// 默认加密数据长度（256 字节）
pub type DefaultEncryptedData = EncryptedData<ConstU32<256>>;

/// 大型加密数据长度（512 字节，适用于奇门等复杂输入）
pub type LargeEncryptedData = EncryptedData<ConstU32<512>>;

/// 超大型加密数据长度（1024 字节）
pub type XLargeEncryptedData = EncryptedData<ConstU32<1024>>;

// ==================== 生成信息 ====================

/// 生成信息（记录数据生成方式）
#[derive(Clone, Debug, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
pub enum GenerationInfo<AccountId: Clone + Encode + Decode + TypeInfo + MaxEncodedLen> {
    /// OCW 生成（公开模式）
    Ocw,

    /// TEE 生成（加密/私密模式）
    Tee {
        /// TEE 节点账户
        node: AccountId,
        /// 计算证明
        proof: ComputationProof,
    },
}

/// 计算证明
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
pub struct ComputationProof {
    /// MRENCLAVE（Enclave 代码哈希，用于身份验证）
    pub mrenclave: [u8; 32],
    /// 输入哈希（用于数据完整性验证）
    pub input_hash: [u8; 32],
    /// 输出哈希（用于数据完整性验证）
    pub output_hash: [u8; 32],
    /// 计算时间戳
    pub timestamp: u64,
    /// Enclave 签名
    pub signature: [u8; 64],
}

impl ComputationProof {
    /// 验证证明签名（简化版，实际需要验证 Enclave 签名）
    pub fn verify(&self) -> bool {
        // TODO: 实现完整的签名验证
        // 当前仅检查非空
        self.mrenclave != [0u8; 32] && self.signature != [0u8; 64]
    }
}

// ==================== 占卜类型扩展 ====================

/// 占卜类型（OCW + TEE 架构专用）
///
/// 与 `pallet-divination-common::DivinationType` 保持一致，
/// 但增加了 TEE 端点等扩展方法。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Hash)]
pub enum DivinationType {
    /// 梅花易数
    Meihua = 0,
    /// 八字排盘
    BaZi = 1,
    /// 六爻占卜
    LiuYao = 2,
    /// 奇门遁甲
    QiMen = 3,
    /// 紫微斗数
    ZiWei = 4,
    /// 太乙神数（预留）
    TaiYi = 5,
    /// 大六壬
    DaLiuRen = 6,
    /// 小六壬
    XiaoLiuRen = 7,
    /// 塔罗牌
    Tarot = 8,
}

impl DivinationType {
    /// 获取 TEE HTTP 端点路径
    pub fn tee_endpoint(&self) -> &'static str {
        match self {
            Self::Meihua => "/compute/meihua",
            Self::BaZi => "/compute/bazi",
            Self::LiuYao => "/compute/liuyao",
            Self::QiMen => "/compute/qimen",
            Self::ZiWei => "/compute/ziwei",
            Self::TaiYi => "/compute/taiyi",
            Self::DaLiuRen => "/compute/daliuren",
            Self::XiaoLiuRen => "/compute/xiaoliuren",
            Self::Tarot => "/compute/tarot",
        }
    }

    /// 获取推荐超时时间（区块数）
    pub fn recommended_timeout(&self) -> u32 {
        match self {
            Self::Meihua => 80,       // ~8 分钟（简单）
            Self::BaZi => 100,        // ~10 分钟
            Self::LiuYao => 100,      // ~10 分钟
            Self::QiMen => 150,       // ~15 分钟（计算复杂）
            Self::ZiWei => 150,       // ~15 分钟（计算复杂）
            Self::TaiYi => 150,       // ~15 分钟
            Self::DaLiuRen => 150,    // ~15 分钟（计算复杂）
            Self::XiaoLiuRen => 60,   // ~6 分钟（简单）
            Self::Tarot => 60,        // ~6 分钟（简单）
        }
    }

    /// 获取推荐最大输入大小（字节）
    pub fn max_input_size(&self) -> u32 {
        match self {
            Self::Meihua => 128,
            Self::BaZi => 256,
            Self::LiuYao => 256,
            Self::QiMen => 512,       // 包含占问事宜
            Self::ZiWei => 256,
            Self::TaiYi => 256,
            Self::DaLiuRen => 256,
            Self::XiaoLiuRen => 128,
            Self::Tarot => 256,
        }
    }

    /// 获取中文名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Meihua => "梅花易数",
            Self::BaZi => "八字排盘",
            Self::LiuYao => "六爻占卜",
            Self::QiMen => "奇门遁甲",
            Self::ZiWei => "紫微斗数",
            Self::TaiYi => "太乙神数",
            Self::DaLiuRen => "大六壬",
            Self::XiaoLiuRen => "小六壬",
            Self::Tarot => "塔罗牌",
        }
    }

    /// 从 pallet-divination-common 的 DivinationType 转换
    pub fn from_common(common: pallet_divination_common::DivinationType) -> Self {
        match common {
            pallet_divination_common::DivinationType::Meihua => Self::Meihua,
            pallet_divination_common::DivinationType::Bazi => Self::BaZi,
            pallet_divination_common::DivinationType::Liuyao => Self::LiuYao,
            pallet_divination_common::DivinationType::Qimen => Self::QiMen,
            pallet_divination_common::DivinationType::Ziwei => Self::ZiWei,
            pallet_divination_common::DivinationType::Taiyi => Self::TaiYi,
            pallet_divination_common::DivinationType::Daliuren => Self::DaLiuRen,
            pallet_divination_common::DivinationType::XiaoLiuRen => Self::XiaoLiuRen,
            pallet_divination_common::DivinationType::Tarot => Self::Tarot,
        }
    }

    /// 转换为 pallet-divination-common 的 DivinationType
    pub fn to_common(&self) -> pallet_divination_common::DivinationType {
        match self {
            Self::Meihua => pallet_divination_common::DivinationType::Meihua,
            Self::BaZi => pallet_divination_common::DivinationType::Bazi,
            Self::LiuYao => pallet_divination_common::DivinationType::Liuyao,
            Self::QiMen => pallet_divination_common::DivinationType::Qimen,
            Self::ZiWei => pallet_divination_common::DivinationType::Ziwei,
            Self::TaiYi => pallet_divination_common::DivinationType::Taiyi,
            Self::DaLiuRen => pallet_divination_common::DivinationType::Daliuren,
            Self::XiaoLiuRen => pallet_divination_common::DivinationType::XiaoLiuRen,
            Self::Tarot => pallet_divination_common::DivinationType::Tarot,
        }
    }
}

impl Default for DivinationType {
    fn default() -> Self {
        Self::Meihua
    }
}

impl DivinationType {
    /// 从 u8 转换
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Meihua,
            1 => Self::BaZi,
            2 => Self::LiuYao,
            3 => Self::QiMen,
            4 => Self::ZiWei,
            5 => Self::TaiYi,
            6 => Self::DaLiuRen,
            7 => Self::XiaoLiuRen,
            8 => Self::Tarot,
            _ => Self::Meihua, // 默认
        }
    }
}

// ==================== 通用链上存储 ====================

/// 通用链上存储结构
///
/// 所有占卜模块的链上存储都使用此结构。
/// 支持版本控制，允许用户修正错误输入。
#[derive(Clone, Debug, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(Index))]
pub struct DivinationOnChain<AccountId, BlockNumber, Index>
where
    AccountId: Clone + Encode + Decode + TypeInfo + MaxEncodedLen,
    BlockNumber: Clone + Encode + Decode + TypeInfo + MaxEncodedLen,
    Index: Clone + Encode + Decode + TypeInfo + MaxEncodedLen,
{
    /// 所有者
    pub owner: AccountId,

    /// 占卜类型
    pub divination_type: DivinationType,

    /// 隐私模式
    pub privacy_mode: PrivacyMode,

    /// 占卜类型特定索引（Private 模式下为 None）
    pub type_index: Option<Index>,

    /// JSON 清单 CID（IPFS）
    pub manifest_cid: BoundedVec<u8, ConstU32<64>>,

    /// 清单哈希（用于验证完整性）
    pub manifest_hash: [u8; 32],

    /// 生成方式
    pub generation: GenerationInfo<AccountId>,

    // ========== 版本控制字段 ==========
    
    /// 版本号（从 1 开始，每次更新 +1）
    pub version: u32,

    /// 首版请求 ID（用于版本链索引，首版时等于自身 request_id）
    pub first_version_id: u64,

    /// 前一版本的 request_id（首版为 None）
    pub previous_version: Option<u64>,

    /// 是否为最新版本
    pub is_latest: bool,

    /// 创建区块
    pub created_at: BlockNumber,

    /// 更新区块
    pub updated_at: BlockNumber,
}

// ==================== 通用待处理请求 ====================

/// 输入数据类型
#[derive(Clone, Debug, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxLen))]
pub enum InputData<MaxLen: Get<u32>> {
    /// 明文输入（Public 模式）
    Plaintext(BoundedVec<u8, MaxLen>),

    /// 加密输入（Encrypted/Private 模式）
    Encrypted(EncryptedData<MaxLen>),
}

/// 通用待处理请求结构
#[derive(Clone, Debug, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxInputLen))]
pub struct PendingRequest<AccountId, BlockNumber, MaxInputLen>
where
    AccountId: Clone + Encode + Decode + TypeInfo + MaxEncodedLen,
    BlockNumber: Clone + Encode + Decode + TypeInfo + MaxEncodedLen,
    MaxInputLen: Get<u32>,
{
    /// 请求者
    pub requester: AccountId,

    /// 占卜类型
    pub divination_type: DivinationType,

    /// 输入数据（明文或密文）
    pub input_data: InputData<MaxInputLen>,

    /// 用户公钥（用于加密返回结果）
    pub user_pubkey: Option<[u8; 32]>,

    /// 隐私模式
    pub privacy_mode: PrivacyMode,

    /// 分配的 TEE 节点
    pub assigned_node: Option<AccountId>,

    /// 请求状态
    pub status: RequestStatus,

    /// 重试次数
    pub retry_count: u8,

    /// 创建区块
    pub created_at: BlockNumber,
}

/// 默认待处理请求类型（256 字节输入）
pub type DefaultPendingRequest<AccountId, BlockNumber> =
    PendingRequest<AccountId, BlockNumber, ConstU32<256>>;

/// 大型待处理请求类型（512 字节输入）
pub type LargePendingRequest<AccountId, BlockNumber> =
    PendingRequest<AccountId, BlockNumber, ConstU32<512>>;

// ==================== TEE 响应 ====================

/// TEE 计算响应
#[derive(Clone, Debug, Encode, Decode)]
pub struct TeeComputeResponse {
    /// 加密的 JSON 清单（或明文，取决于隐私模式）
    pub manifest: Vec<u8>,

    /// 清单哈希
    pub manifest_hash: [u8; 32],

    /// 类型特定索引（编码后）
    pub type_index: Option<Vec<u8>>,

    /// 计算证明
    pub computation_proof: ComputationProof,

    /// IPFS CID（如果 TEE 已上传）
    pub ipfs_cid: Option<Vec<u8>>,
}

// ==================== 处理结果 ====================

/// OCW 处理结果
#[derive(Clone, Debug)]
pub struct ProcessResult {
    /// IPFS CID
    pub manifest_cid: Vec<u8>,

    /// 清单哈希
    pub manifest_hash: [u8; 32],

    /// 类型特定索引（编码后）
    pub type_index: Option<Vec<u8>>,

    /// 计算证明（TEE 模式）
    pub proof: Option<ComputationProof>,

    /// 清单数据（用于 IPFS 上传）
    pub manifest_data: Option<Vec<u8>>,
}

// ==================== 模块错误 ====================

/// 模块错误类型
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub enum ModuleError {
    /// 输入无效
    InvalidInput(BoundedVec<u8, ConstU32<128>>),

    /// 计算失败
    ComputationFailed(BoundedVec<u8, ConstU32<128>>),

    /// 序列化失败
    SerializationFailed,

    /// 模块未注册
    ModuleNotRegistered,

    /// 配置错误
    ConfigurationError,

    /// TEE 节点不可用
    TeeNodeUnavailable,

    /// TEE 请求失败
    TeeRequestFailed(BoundedVec<u8, ConstU32<128>>),

    /// IPFS 上传失败
    IpfsUploadFailed,

    /// 请求超时
    RequestTimeout,

    /// 最大重试次数已达
    MaxRetriesExceeded,

    /// 其他错误
    Other(BoundedVec<u8, ConstU32<128>>),
}

impl ModuleError {
    /// 创建输入无效错误
    pub fn invalid_input(msg: &[u8]) -> Self {
        Self::InvalidInput(
            msg.to_vec()
                .try_into()
                .unwrap_or_else(|_| BoundedVec::default()),
        )
    }

    /// 创建计算失败错误
    pub fn computation_failed(msg: &[u8]) -> Self {
        Self::ComputationFailed(
            msg.to_vec()
                .try_into()
                .unwrap_or_else(|_| BoundedVec::default()),
        )
    }

    /// 创建 TEE 请求失败错误
    pub fn tee_request_failed(msg: &[u8]) -> Self {
        Self::TeeRequestFailed(
            msg.to_vec()
                .try_into()
                .unwrap_or_else(|_| BoundedVec::default()),
        )
    }

    /// 创建其他错误
    pub fn other(msg: &[u8]) -> Self {
        Self::Other(
            msg.to_vec()
                .try_into()
                .unwrap_or_else(|_| BoundedVec::default()),
        )
    }
}

// ==================== 版本历史 ====================

/// 版本历史条目
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VersionHistoryEntry<BlockNumber>
where
    BlockNumber: Clone + Encode + Decode + TypeInfo + MaxEncodedLen,
{
    /// 请求 ID
    pub request_id: u64,
    /// 版本号
    pub version: u32,
    /// 创建时间
    pub created_at: BlockNumber,
    /// 是否为最新版本
    pub is_latest: bool,
}

/// 版本信息（临时存储，用于创建过程）
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct VersionInfo {
    /// 首版请求 ID
    pub first_version_id: u64,
    /// 版本号
    pub version: u32,
    /// 前一版本 ID
    pub previous_version: Option<u64>,
}

// ==================== 从 pallet-tee-privacy 重新导出 TEE 类型 ====================

pub use pallet_tee_privacy::types::{
    TeeType,
    TeeNodeStatus,
    TeeAttestation,
    TeeNode,
    EncryptedData as TeeEncryptedData,
    FailureReason,
    ComputeRequestInfo,
    ComputeResultInfo,
    StakeInfo,
    NodeStatistics,
    TeeNodeInfo,
    RequestStatusInfo,
    AttestationVerifyResult,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_mode() {
        assert_eq!(PrivacyMode::Public.name(), "public");
        assert!(!PrivacyMode::Public.requires_tee());
        assert!(PrivacyMode::Public.stores_index_on_chain());

        assert_eq!(PrivacyMode::Encrypted.name(), "encrypted");
        assert!(PrivacyMode::Encrypted.requires_tee());
        assert!(PrivacyMode::Encrypted.stores_index_on_chain());

        assert_eq!(PrivacyMode::Private.name(), "private");
        assert!(PrivacyMode::Private.requires_tee());
        assert!(!PrivacyMode::Private.stores_index_on_chain());
    }

    #[test]
    fn test_request_status() {
        assert!(!RequestStatus::Pending.is_final());
        assert!(!RequestStatus::Processing.is_final());
        assert!(RequestStatus::Completed.is_final());
        assert!(!RequestStatus::Failed.is_final());
        assert!(RequestStatus::Timeout.is_final());

        assert!(RequestStatus::Failed.is_retryable());
        assert!(!RequestStatus::Completed.is_retryable());
    }

    #[test]
    fn test_divination_type() {
        assert_eq!(DivinationType::BaZi.tee_endpoint(), "/compute/bazi");
        assert_eq!(DivinationType::QiMen.recommended_timeout(), 150);
        assert_eq!(DivinationType::XiaoLiuRen.recommended_timeout(), 60);
    }
}
