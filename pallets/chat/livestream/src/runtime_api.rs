//! 直播间模块 Runtime API 定义
//!
//! 提供链下查询接口，用于前端获取直播间信息

use codec::Codec;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    /// 直播间模块 Runtime API
    pub trait LivestreamApi<AccountId, Balance>
    where
        AccountId: Codec,
        Balance: Codec,
    {
        /// 获取直播间信息
        fn get_room(room_id: u64) -> Option<LiveRoomInfo<AccountId, Balance>>;

        /// 获取主播的活跃直播间 ID
        fn get_host_room(host: AccountId) -> Option<u64>;

        /// 获取所有活跃直播间 (直播中)
        fn get_live_rooms() -> Vec<u64>;

        /// 获取礼物信息
        fn get_gift(gift_id: u32) -> Option<GiftInfo<Balance>>;

        /// 获取所有启用的礼物
        fn get_enabled_gifts() -> Vec<GiftInfo<Balance>>;

        /// 检查用户是否有门票
        fn has_ticket(room_id: u64, user: AccountId) -> bool;

        /// 检查用户是否在黑名单
        fn is_blacklisted(room_id: u64, user: AccountId) -> bool;

        /// 获取主播累计收益
        fn get_host_earnings(host: AccountId) -> Balance;

        /// 获取用户在直播间的累计打赏
        fn get_user_room_gifts(room_id: u64, user: AccountId) -> Balance;

        /// 获取直播间当前连麦者
        fn get_co_hosts(room_id: u64) -> Vec<AccountId>;
    }
}

/// 直播间信息 (用于 API 返回)
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug, scale_info::TypeInfo)]
pub struct LiveRoomInfo<AccountId, Balance> {
    pub id: u64,
    pub host: AccountId,
    pub title: Vec<u8>,
    pub description: Option<Vec<u8>>,
    pub room_type: u8,      // 0: Normal, 1: Paid, 2: Private, 3: MultiHost
    pub status: u8,         // 0: Preparing, 1: Live, 2: Paused, 3: Ended, 4: Banned
    pub cover_cid: Option<Vec<u8>>,
    pub total_viewers: u64,
    pub peak_viewers: u32,
    pub total_gifts: Balance,
    pub ticket_price: Option<Balance>,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub ended_at: Option<u64>,
}

/// 礼物信息 (用于 API 返回)
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug, scale_info::TypeInfo)]
pub struct GiftInfo<Balance> {
    pub id: u32,
    pub name: Vec<u8>,
    pub price: Balance,
    pub icon_cid: Vec<u8>,
    pub enabled: bool,
}
