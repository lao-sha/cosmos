//! Implementation of the MembershipProvider trait.

use crate::{pallet::*, types::*, traits::*};
use frame_support::dispatch::DispatchResult;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::DispatchError;

impl<T: Config> MembershipProvider<T::AccountId, BalanceOf<T>, BlockNumberFor<T>> for Pallet<T> {
    fn get_tier(who: &T::AccountId) -> MemberTier {
        Self::get_tier(who)
    }

    fn is_active_member(who: &T::AccountId, min_tier: MemberTier) -> bool {
        let tier = Self::get_tier(who);
        tier.is_at_least(min_tier)
    }

    fn get_storage_discount(who: &T::AccountId) -> u32 {
        let tier = Self::get_tier(who);
        Self::get_storage_discount(tier)
    }

    fn get_ai_discount(who: &T::AccountId) -> u32 {
        let tier = Self::get_tier(who);
        Self::get_ai_discount(tier)
    }

    fn get_daily_free_quota(who: &T::AccountId) -> u32 {
        let tier = Self::get_tier(who);
        Self::get_daily_free_divination_quota(tier)
    }

    fn get_monthly_free_ai_quota(who: &T::AccountId) -> u32 {
        let tier = Self::get_tier(who);
        Self::get_monthly_free_ai_quota(tier)
    }

    fn get_remaining_free_ai(who: &T::AccountId) -> u32 {
        Self::get_remaining_free_ai(who)
    }

    fn use_free_ai(who: &T::AccountId) -> DispatchResult {
        let tier = Self::get_tier(who);
        let quota = Self::get_monthly_free_ai_quota(tier);

        if quota == 0 {
            return Err(Error::<T>::FreeAiQuotaExceeded.into());
        }

        let now = frame_system::Pallet::<T>::block_number();
        let current_month = Self::block_to_month(now);

        let used = MonthlyFreeAiUsage::<T>::get(who, current_month);
        if used >= quota {
            return Err(Error::<T>::FreeAiQuotaExceeded.into());
        }

        MonthlyFreeAiUsage::<T>::insert(who, current_month, used + 1);
        Ok(())
    }

    fn get_reward_multiplier(who: &T::AccountId) -> u32 {
        let tier = Self::get_tier(who);
        Self::get_reward_multiplier_base(tier)
    }

    fn grant_reward(
        who: &T::AccountId,
        base_amount: BalanceOf<T>,
        tx_type: RewardTxType,
        memo: &[u8],
    ) -> Result<BalanceOf<T>, DispatchError> {
        Self::do_grant_reward(who, base_amount, tx_type, memo)
    }

    fn can_receive_reward(who: &T::AccountId) -> bool {
        Self::can_receive_reward(who)
    }

    fn get_profile(who: &T::AccountId) -> Option<MemberProfileSummary<BlockNumberFor<T>>> {
        MemberProfiles::<T>::get(who).map(|p| MemberProfileSummary {
            display_name: p.display_name.into_inner(),
            gender: p.gender,
            birth_date: p.birth_date,
            birth_hour: p.birth_hour,
            longitude: p.longitude,
            latitude: p.latitude,
            is_provider: p.is_provider,
            provider_verified: p.provider_verified,
            updated_at: p.updated_at,
        })
    }

    fn is_verified_provider(who: &T::AccountId) -> bool {
        MemberProfiles::<T>::get(who)
            .map(|p| p.is_provider && p.provider_verified)
            .unwrap_or(false)
    }
}

impl<T: Config> RewardPoolManager<BalanceOf<T>> for Pallet<T> {
    fn reward_pool_balance() -> BalanceOf<T> {
        Self::reward_pool_balance()
    }

    fn get_adjustment_factor() -> u32 {
        Self::get_pool_adjustment_factor()
    }
}
