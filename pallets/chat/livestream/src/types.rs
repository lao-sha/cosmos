//! 直播间模块类型定义

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

/// 直播间状态
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
    Default,
)]
pub enum LiveRoomStatus {
    /// 准备中（未开播）
    #[default]
    Preparing,
    /// 直播中
    Live,
    /// 暂停中
    Paused,
    /// 已结束
    Ended,
    /// 被封禁
    Banned,
}

/// 直播间类型
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
    Default,
)]
pub enum LiveRoomType {
    /// 普通直播
    #[default]
    Normal,
    /// 付费直播（需购票）
    Paid,
    /// 私密直播（仅邀请）
    Private,
    /// 连麦直播
    MultiHost,
}

/// 连麦类型
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
    Default,
)]
pub enum CoHostType {
    /// 语音连麦
    #[default]
    AudioOnly,
    /// 视频连麦
    VideoAndAudio,
}

/// 直播间信息
#[derive(Encode, Decode, DecodeWithMemTracking, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxTitleLen, MaxDescriptionLen, MaxCidLen))]
pub struct LiveRoom<AccountId, Balance, MaxTitleLen, MaxDescriptionLen, MaxCidLen>
where
    MaxTitleLen: Get<u32>,
    MaxDescriptionLen: Get<u32>,
    MaxCidLen: Get<u32>,
{
    /// 直播间ID（自增）
    pub id: u64,
    /// 主播账户
    pub host: AccountId,
    /// 直播间标题
    pub title: BoundedVec<u8, MaxTitleLen>,
    /// 直播间描述
    pub description: Option<BoundedVec<u8, MaxDescriptionLen>>,
    /// 直播间类型
    pub room_type: LiveRoomType,
    /// 直播间状态
    pub status: LiveRoomStatus,
    /// 封面图CID (IPFS)
    pub cover_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 累计观众数（直播结束时从 LiveKit 同步）
    pub total_viewers: u64,
    /// 峰值观众数
    pub peak_viewers: u32,
    /// 累计礼物收入
    pub total_gifts: Balance,
    /// 付费直播票价（仅Paid类型）
    pub ticket_price: Option<Balance>,
    /// 创建时间（区块号）
    pub created_at: u64,
    /// 开播时间（区块号）
    pub started_at: Option<u64>,
    /// 结束时间（区块号）
    pub ended_at: Option<u64>,
}

// 手动实现 Clone，避免 Get<u32> 类型参数的 Clone 约束
impl<AccountId, Balance, MaxTitleLen, MaxDescriptionLen, MaxCidLen> Clone
    for LiveRoom<AccountId, Balance, MaxTitleLen, MaxDescriptionLen, MaxCidLen>
where
    AccountId: Clone,
    Balance: Clone,
    MaxTitleLen: Get<u32>,
    MaxDescriptionLen: Get<u32>,
    MaxCidLen: Get<u32>,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            host: self.host.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            room_type: self.room_type,
            status: self.status,
            cover_cid: self.cover_cid.clone(),
            total_viewers: self.total_viewers,
            peak_viewers: self.peak_viewers,
            total_gifts: self.total_gifts.clone(),
            ticket_price: self.ticket_price.clone(),
            created_at: self.created_at,
            started_at: self.started_at,
            ended_at: self.ended_at,
        }
    }
}

/// 礼物定义
#[derive(Encode, Decode, DecodeWithMemTracking, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxGiftNameLen, MaxCidLen))]
pub struct Gift<Balance, MaxGiftNameLen, MaxCidLen>
where
    MaxGiftNameLen: Get<u32>,
    MaxCidLen: Get<u32>,
{
    /// 礼物ID
    pub id: u32,
    /// 礼物名称
    pub name: BoundedVec<u8, MaxGiftNameLen>,
    /// 礼物价格
    pub price: Balance,
    /// 礼物图标CID
    pub icon_cid: BoundedVec<u8, MaxCidLen>,
    /// 是否启用
    pub enabled: bool,
}

// 手动实现 Clone
impl<Balance, MaxGiftNameLen, MaxCidLen> Clone for Gift<Balance, MaxGiftNameLen, MaxCidLen>
where
    Balance: Clone,
    MaxGiftNameLen: Get<u32>,
    MaxCidLen: Get<u32>,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            price: self.price.clone(),
            icon_cid: self.icon_cid.clone(),
            enabled: self.enabled,
        }
    }
}
