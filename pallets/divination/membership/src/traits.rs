//! Trait definitions for the membership pallet.
//!
//! These traits allow other pallets to interact with the membership system
//! without tight coupling.

use crate::types::*;
use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;

/// Membership provider trait for other pallets to query membership data.
pub trait MembershipProvider<AccountId, Balance, BlockNumber> {
    /// Get the membership tier for an account.
    fn get_tier(who: &AccountId) -> MemberTier;

    /// Check if an account has at least a certain tier (and is active).
    fn is_active_member(who: &AccountId, min_tier: MemberTier) -> bool;

    /// Get storage deposit discount rate (basis points, e.g., 3000 = 30%).
    fn get_storage_discount(who: &AccountId) -> u32;

    /// Get AI interpretation discount rate (basis points).
    fn get_ai_discount(who: &AccountId) -> u32;

    /// Get daily free divination quota.
    fn get_daily_free_quota(who: &AccountId) -> u32;

    /// Get monthly free AI interpretation quota.
    fn get_monthly_free_ai_quota(who: &AccountId) -> u32;

    /// Get remaining free AI credits for current month.
    fn get_remaining_free_ai(who: &AccountId) -> u32;

    /// Use one free AI credit (returns error if none available).
    fn use_free_ai(who: &AccountId) -> DispatchResult;

    /// Get DUST reward multiplier (basis points, 10000 = 1.0x).
    fn get_reward_multiplier(who: &AccountId) -> u32;

    /// Grant DUST reward to an account.
    /// Automatically applies tier multiplier and pool adjustment.
    /// Returns the actual amount granted.
    fn grant_reward(
        who: &AccountId,
        base_amount: Balance,
        tx_type: RewardTxType,
        memo: &[u8],
    ) -> Result<Balance, DispatchError>;

    /// Check if an account can receive rewards.
    fn can_receive_reward(who: &AccountId) -> bool;

    /// Get member profile summary (plaintext data for divination).
    fn get_profile(who: &AccountId) -> Option<MemberProfileSummary<BlockNumber>>;

    /// Check if an account is a verified provider.
    fn is_verified_provider(who: &AccountId) -> bool;
}

/// Reward pool management trait.
pub trait RewardPoolManager<Balance> {
    /// Get current reward pool balance.
    fn reward_pool_balance() -> Balance;

    /// Get the dynamic adjustment factor (basis points).
    fn get_adjustment_factor() -> u32;
}
