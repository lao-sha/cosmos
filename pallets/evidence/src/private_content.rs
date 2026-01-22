use super::*;
use frame_support::{pallet_prelude::*, BoundedVec};
use codec::DecodeWithMemTracking;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_core::{ConstU32, H256};

pub type AuthorizedUsers<T> =
    BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxAuthorizedUsers>;

pub type EncryptedKeyBundles<T> = BoundedVec<
    (
        <T as frame_system::Config>::AccountId,
        BoundedVec<u8, <T as Config>::MaxKeyLen>,
    ),
    <T as Config>::MaxAuthorizedUsers,
>;

/// 私密内容存储结构
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct PrivateContent<T: Config> {
    /// 内容ID
    pub id: u64,
    /// 命名空间
    pub ns: [u8; 8],
    /// 业务主体ID
    pub subject_id: u64,
    /// IPFS CID (明文存储，便于索引和去重)
    pub cid: BoundedVec<u8, T::MaxCidLen>,
    /// 原始内容的哈希（用于验证完整性）
    pub content_hash: H256,
    /// 加密方法标识 (1=AES-256-GCM, 2=ChaCha20-Poly1305, etc.)
    pub encryption_method: u8,
    /// 创建者
    pub creator: <T as frame_system::Config>::AccountId,
    /// 访问控制策略
    pub access_policy: AccessPolicy<T>,
    /// 每个授权用户的加密密钥包
    pub encrypted_keys: EncryptedKeyBundles<T>,
    /// 创建时间
    pub created_at: BlockNumberFor<T>,
    /// 最后更新时间
    pub updated_at: BlockNumberFor<T>,
}

/// 访问控制策略
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub enum AccessPolicy<T: Config> {
    /// 仅创建者可访问
    OwnerOnly,
    /// 指定用户列表
    SharedWith(AuthorizedUsers<T>),
    /// 定时访问（到期后自动撤销）
    TimeboxedAccess {
        users: AuthorizedUsers<T>,
        expires_at: BlockNumberFor<T>,
    },
    /// 治理控制
    GovernanceControlled,
    /// 基于角色的访问（扩展用）
    RoleBased(BoundedVec<u8, ConstU32<32>>),
}

/// 用户公钥存储
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct UserPublicKey<T: Config> {
    /// RSA-2048 公钥 (DER格式)
    pub key_data: BoundedVec<u8, <T as Config>::MaxKeyLen>,
    /// 密钥类型 (1=RSA-2048, 2=Ed25519, 3=ECDSA-P256)
    pub key_type: u8,
    /// 注册时间（区块）
    pub registered_at: BlockNumberFor<T>,
}

/// 密钥轮换记录
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct KeyRotationRecord<T: Config> {
    /// 内容ID
    pub content_id: u64,
    /// 轮换批次
    pub rotation_round: u32,
    /// 轮换时间
    pub rotated_at: BlockNumberFor<T>,
    /// 轮换者
    pub rotated_by: <T as frame_system::Config>::AccountId,
}
