//! # 提供者辅助函数
//!
//! 本模块包含提供者等级计算等辅助函数

use crate::pallet::*;
use crate::types::*;

impl<T: Config> Pallet<T> {
    /// 尝试提升提供者等级
    ///
    /// 根据完成订单数和平均评分判断是否可以升级
    pub(crate) fn try_upgrade_tier(provider: &mut ProviderOf<T>) {
        let current_tier = provider.tier;
        let avg_rating = provider.average_rating();
        let completed = provider.completed_orders;

        let new_tier = if completed >= ProviderTier::Master.min_orders()
            && avg_rating >= ProviderTier::Master.min_rating()
        {
            ProviderTier::Master
        } else if completed >= ProviderTier::Expert.min_orders()
            && avg_rating >= ProviderTier::Expert.min_rating()
        {
            ProviderTier::Expert
        } else if completed >= ProviderTier::Senior.min_orders()
            && avg_rating >= ProviderTier::Senior.min_rating()
        {
            ProviderTier::Senior
        } else if completed >= ProviderTier::Certified.min_orders()
            && avg_rating >= ProviderTier::Certified.min_rating()
        {
            ProviderTier::Certified
        } else {
            ProviderTier::Novice
        };

        if new_tier as u8 > current_tier as u8 {
            provider.tier = new_tier;
        }
    }
}
