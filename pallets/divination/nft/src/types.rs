//! # 玄学 NFT 类型定义
//!
//! 支持多种占卜类型（梅花、八字、六爻等）的通用 NFT 数据结构。

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, BoundedVec};
use pallet_divination_common::{DivinationType, Rarity};
use scale_info::TypeInfo;

/// NFT 状态
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum NftStatus {
    /// 正常持有
    #[default]
    Normal,
    /// 挂单出售中
    Listed,
    /// 锁定中（质押等）
    Locked,
    /// 已销毁
    Burned,
}

/// NFT 元数据
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxCidLen, MaxNameLen))]
pub struct NftMetadata<MaxCidLen: Get<u32>, MaxNameLen: Get<u32>> {
    /// NFT 名称
    pub name: BoundedVec<u8, MaxNameLen>,
    /// 描述 CID
    pub description_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 图片 CID
    pub image_cid: BoundedVec<u8, MaxCidLen>,
    /// 动画 CID（可选，用于动态展示）
    pub animation_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 外部链接 CID
    pub external_url_cid: Option<BoundedVec<u8, MaxCidLen>>,
}

/// 玄学占卜 NFT 数据
///
/// 泛化的 NFT 结构，支持多种占卜类型。
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxCidLen, MaxNameLen))]
pub struct DivinationNft<AccountId, Balance, BlockNumber, MaxCidLen: Get<u32>, MaxNameLen: Get<u32>>
{
    /// NFT ID
    pub id: u64,
    /// 占卜类型（梅花、八字、六爻等）
    pub divination_type: DivinationType,
    /// 关联的占卜结果 ID（卦象 ID、命盘 ID 等）
    pub result_id: u64,
    /// 所有者
    pub owner: AccountId,
    /// 创建者（铸造者）
    pub creator: AccountId,
    /// 稀有度
    pub rarity: Rarity,
    /// 状态
    pub status: NftStatus,
    /// 元数据
    pub metadata: NftMetadata<MaxCidLen, MaxNameLen>,
    /// 铸造时间
    pub minted_at: BlockNumber,
    /// 铸造时支付的费用
    pub mint_fee: Balance,
    /// 版税比例（万分比，创作者在每次转售时获得的比例）
    pub royalty_rate: u16,
    /// 转移次数
    pub transfer_count: u32,
}

/// NFT 挂单信息
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct Listing<AccountId, Balance, BlockNumber> {
    /// NFT ID
    pub nft_id: u64,
    /// 卖家
    pub seller: AccountId,
    /// 价格
    pub price: Balance,
    /// 挂单时间
    pub listed_at: BlockNumber,
    /// 过期时间（区块数）
    pub expires_at: Option<BlockNumber>,
}

/// 出价信息
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct Offer<AccountId, Balance, BlockNumber> {
    /// 出价 ID
    pub id: u64,
    /// NFT ID
    pub nft_id: u64,
    /// 出价者
    pub bidder: AccountId,
    /// 出价金额
    pub amount: Balance,
    /// 出价时间
    pub offered_at: BlockNumber,
    /// 过期时间
    pub expires_at: BlockNumber,
    /// 是否有效
    pub is_active: bool,
}

/// 收藏集
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxNameLen, MaxCidLen))]
pub struct Collection<AccountId, BlockNumber, MaxNameLen: Get<u32>, MaxCidLen: Get<u32>> {
    /// 收藏集 ID
    pub id: u32,
    /// 创建者
    pub creator: AccountId,
    /// 名称
    pub name: BoundedVec<u8, MaxNameLen>,
    /// 描述 CID
    pub description_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 封面图 CID
    pub cover_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// NFT 数量
    pub nft_count: u32,
    /// 创建时间
    pub created_at: BlockNumber,
    /// 是否公开
    pub is_public: bool,
}

/// NFT 统计数据
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct NftStats<Balance: Default> {
    /// 总铸造数
    pub total_minted: u64,
    /// 总销毁数
    pub total_burned: u64,
    /// 总交易次数
    pub total_trades: u64,
    /// 总交易额
    pub total_volume: Balance,
    /// 当前挂单数
    pub active_listings: u32,
    /// 各稀有度铸造数
    pub common_count: u64,
    pub rare_count: u64,
    pub epic_count: u64,
    pub legendary_count: u64,
}

/// 按占卜类型的 NFT 统计
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct TypeStats {
    /// 铸造数量
    pub minted_count: u64,
    /// 销毁数量
    pub burned_count: u64,
    /// 交易数量
    pub trade_count: u64,
}
