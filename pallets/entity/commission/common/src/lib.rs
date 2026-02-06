//! # Commission Common Types
//!
//! Shared types and traits for the commission plugin system.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

// ============================================================================
// 返佣模式位标志
// ============================================================================

/// 返佣模式位标志（可多选）
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct CommissionModes(pub u16);

impl CommissionModes {
    pub const NONE: u16 = 0b0000_0000;
    pub const DIRECT_REWARD: u16 = 0b0000_0001;
    pub const MULTI_LEVEL: u16 = 0b0000_0010;
    pub const TEAM_PERFORMANCE: u16 = 0b0000_0100;
    pub const LEVEL_DIFF: u16 = 0b0000_1000;
    pub const FIXED_AMOUNT: u16 = 0b0001_0000;
    pub const FIRST_ORDER: u16 = 0b0010_0000;
    pub const REPEAT_PURCHASE: u16 = 0b0100_0000;
    pub const SINGLE_LINE_UPLINE: u16 = 0b1000_0000;
    pub const SINGLE_LINE_DOWNLINE: u16 = 0b1_0000_0000;

    pub fn contains(&self, flag: u16) -> bool {
        self.0 & flag != 0
    }

    pub fn insert(&mut self, flag: u16) {
        self.0 |= flag;
    }

    pub fn remove(&mut self, flag: u16) {
        self.0 &= !flag;
    }
}

// ============================================================================
// 返佣来源
// ============================================================================

/// 返佣来源（预留，当前版本返佣统一从 Shop 运营账户出）
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum CommissionSource {
    #[default]
    PlatformFee,
    ShopFund,
    Mixed,
}

// ============================================================================
// 返佣类型 / 状态
// ============================================================================

/// 返佣类型
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum CommissionType {
    DirectReward,
    MultiLevel,
    TeamPerformance,
    LevelDiff,
    FixedAmount,
    FirstOrder,
    RepeatPurchase,
    SingleLineUpline,
    SingleLineDownline,
}

/// 返佣状态
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum CommissionStatus {
    #[default]
    Pending,
    Distributed,
    Withdrawn,
    Cancelled,
}

// ============================================================================
// 返佣记录
// ============================================================================

/// 返佣记录
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct CommissionRecord<AccountId, Balance, BlockNumber> {
    pub shop_id: u64,
    pub order_id: u64,
    pub buyer: AccountId,
    pub beneficiary: AccountId,
    pub amount: Balance,
    pub commission_type: CommissionType,
    pub level: u8,
    pub status: CommissionStatus,
    pub created_at: BlockNumber,
}

/// 会员返佣统计
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct MemberCommissionStatsData<Balance: Default> {
    pub total_earned: Balance,
    pub pending: Balance,
    pub withdrawn: Balance,
    pub repurchased: Balance,
    pub order_count: u32,
}

// ============================================================================
// 提现配置
// ============================================================================

/// 分级提现配置
#[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct WithdrawalTierConfig {
    pub withdrawal_rate: u16,
    pub repurchase_rate: u16,
}

impl Default for WithdrawalTierConfig {
    fn default() -> Self {
        Self {
            withdrawal_rate: 10000,
            repurchase_rate: 0,
        }
    }
}

// ============================================================================
// 插件输出
// ============================================================================

/// 单条返佣输出（插件计算结果）
#[derive(Clone, RuntimeDebug)]
pub struct CommissionOutput<AccountId, Balance> {
    pub beneficiary: AccountId,
    pub amount: Balance,
    pub commission_type: CommissionType,
    pub level: u8,
}

// ============================================================================
// CommissionPlugin Trait
// ============================================================================

/// 返佣插件接口
///
/// 每个返佣模式实现此 trait，由 core 调度引擎调用。
/// `calculate` 接收订单上下文和剩余可分配额度，返回返佣输出列表和更新后的剩余额度。
pub trait CommissionPlugin<AccountId, Balance> {
    /// 计算返佣
    ///
    /// # 参数
    /// - `shop_id`: 店铺 ID
    /// - `buyer`: 买家账户
    /// - `order_amount`: 订单金额
    /// - `remaining`: 剩余可分配额度
    /// - `enabled_modes`: 启用的返佣模式位标志
    /// - `is_first_order`: 是否首单
    /// - `buyer_order_count`: 买家订单数
    ///
    /// # 返回
    /// `(outputs, new_remaining)` — 返佣输出列表和剩余额度
    fn calculate(
        shop_id: u64,
        buyer: &AccountId,
        order_amount: Balance,
        remaining: Balance,
        enabled_modes: CommissionModes,
        is_first_order: bool,
        buyer_order_count: u32,
    ) -> (Vec<CommissionOutput<AccountId, Balance>>, Balance);
}

/// 空插件实现（占位）
impl<AccountId, Balance> CommissionPlugin<AccountId, Balance> for () {
    fn calculate(
        _shop_id: u64,
        _buyer: &AccountId,
        _order_amount: Balance,
        remaining: Balance,
        _enabled_modes: CommissionModes,
        _is_first_order: bool,
        _buyer_order_count: u32,
    ) -> (Vec<CommissionOutput<AccountId, Balance>>, Balance) {
        (Vec::new(), remaining)
    }
}

// ============================================================================
// CommissionProvider Trait（供外部模块调用）
// ============================================================================

/// 返佣服务接口
pub trait CommissionProvider<AccountId, Balance> {
    fn process_commission(
        shop_id: u64,
        order_id: u64,
        buyer: &AccountId,
        order_amount: Balance,
        available_pool: Balance,
    ) -> Result<(), DispatchError>;

    fn cancel_commission(order_id: u64) -> Result<(), DispatchError>;

    fn pending_commission(shop_id: u64, account: &AccountId) -> Balance;

    fn set_commission_modes(shop_id: u64, modes: u16) -> Result<(), DispatchError>;

    fn set_direct_reward_rate(shop_id: u64, rate: u16) -> Result<(), DispatchError>;

    fn set_level_diff_config(
        shop_id: u64,
        normal_rate: u16,
        silver_rate: u16,
        gold_rate: u16,
        platinum_rate: u16,
        diamond_rate: u16,
    ) -> Result<(), DispatchError>;

    fn set_fixed_amount(shop_id: u64, amount: Balance) -> Result<(), DispatchError>;

    fn set_first_order_config(
        shop_id: u64,
        amount: Balance,
        rate: u16,
        use_amount: bool,
    ) -> Result<(), DispatchError>;

    fn set_repeat_purchase_config(shop_id: u64, rate: u16, min_orders: u32) -> Result<(), DispatchError>;

    fn set_withdrawal_config_by_governance(
        shop_id: u64,
        enabled: bool,
        shopping_balance_generates_commission: bool,
    ) -> Result<(), DispatchError>;

    fn shopping_balance(shop_id: u64, account: &AccountId) -> Balance;
}

/// 空 CommissionProvider 实现
pub struct NullCommissionProvider;

impl<AccountId, Balance: Default> CommissionProvider<AccountId, Balance> for NullCommissionProvider {
    fn process_commission(_: u64, _: u64, _: &AccountId, _: Balance, _: Balance) -> Result<(), DispatchError> { Ok(()) }
    fn cancel_commission(_: u64) -> Result<(), DispatchError> { Ok(()) }
    fn pending_commission(_: u64, _: &AccountId) -> Balance { Balance::default() }
    fn set_commission_modes(_: u64, _: u16) -> Result<(), DispatchError> { Ok(()) }
    fn set_direct_reward_rate(_: u64, _: u16) -> Result<(), DispatchError> { Ok(()) }
    fn set_level_diff_config(_: u64, _: u16, _: u16, _: u16, _: u16, _: u16) -> Result<(), DispatchError> { Ok(()) }
    fn set_fixed_amount(_: u64, _: Balance) -> Result<(), DispatchError> { Ok(()) }
    fn set_first_order_config(_: u64, _: Balance, _: u16, _: bool) -> Result<(), DispatchError> { Ok(()) }
    fn set_repeat_purchase_config(_: u64, _: u16, _: u32) -> Result<(), DispatchError> { Ok(()) }
    fn set_withdrawal_config_by_governance(_: u64, _: bool, _: bool) -> Result<(), DispatchError> { Ok(()) }
    fn shopping_balance(_: u64, _: &AccountId) -> Balance { Balance::default() }
}

// ============================================================================
// MemberProvider Trait（由 member 模块实现）
// ============================================================================

/// 会员服务接口（供返佣插件查询推荐人、等级等）
pub trait MemberProvider<AccountId> {
    fn get_referrer(shop_id: u64, account: &AccountId) -> Option<AccountId>;
    fn member_level(shop_id: u64, account: &AccountId) -> Option<pallet_entity_common::MemberLevel>;
    fn get_member_stats(shop_id: u64, account: &AccountId) -> (u32, u32, u128);
    fn uses_custom_levels(shop_id: u64) -> bool;
    fn custom_level_id(shop_id: u64, account: &AccountId) -> u8;

    fn set_custom_levels_enabled(shop_id: u64, enabled: bool) -> Result<(), DispatchError>;
    fn set_upgrade_mode(shop_id: u64, mode: u8) -> Result<(), DispatchError>;
    fn add_custom_level(shop_id: u64, level_id: u8, name: &[u8], threshold: u128, discount_rate: u16, commission_bonus: u16) -> Result<(), DispatchError>;
    fn update_custom_level(shop_id: u64, level_id: u8, name: Option<&[u8]>, threshold: Option<u128>, discount_rate: Option<u16>, commission_bonus: Option<u16>) -> Result<(), DispatchError>;
    fn remove_custom_level(shop_id: u64, level_id: u8) -> Result<(), DispatchError>;
    fn custom_level_count(shop_id: u64) -> u8;
}

/// 空 MemberProvider 实现
pub struct NullMemberProvider;

impl<AccountId> MemberProvider<AccountId> for NullMemberProvider {
    fn get_referrer(_: u64, _: &AccountId) -> Option<AccountId> { None }
    fn member_level(_: u64, _: &AccountId) -> Option<pallet_entity_common::MemberLevel> { None }
    fn get_member_stats(_: u64, _: &AccountId) -> (u32, u32, u128) { (0, 0, 0) }
    fn uses_custom_levels(_: u64) -> bool { false }
    fn custom_level_id(_: u64, _: &AccountId) -> u8 { 0 }
    fn set_custom_levels_enabled(_: u64, _: bool) -> Result<(), DispatchError> { Ok(()) }
    fn set_upgrade_mode(_: u64, _: u8) -> Result<(), DispatchError> { Ok(()) }
    fn add_custom_level(_: u64, _: u8, _: &[u8], _: u128, _: u16, _: u16) -> Result<(), DispatchError> { Ok(()) }
    fn update_custom_level(_: u64, _: u8, _: Option<&[u8]>, _: Option<u128>, _: Option<u16>, _: Option<u16>) -> Result<(), DispatchError> { Ok(()) }
    fn remove_custom_level(_: u64, _: u8) -> Result<(), DispatchError> { Ok(()) }
    fn custom_level_count(_: u64) -> u8 { 0 }
}
