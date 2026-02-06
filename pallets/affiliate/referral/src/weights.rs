//! Pallet referral 权重接口与占位实现（上线前用 benchmark 生成）

use core::marker::PhantomData;
use frame_support::weights::Weight;

/// 权重信息 trait
pub trait WeightInfo {
    /// bind_sponsor：绑定推荐人
    fn bind_sponsor() -> Weight;
    /// claim_code：认领推荐码
    fn claim_code() -> Weight;
}

/// 默认实现（用于测试）
impl WeightInfo for () {
    fn bind_sponsor() -> Weight {
        // 读取：Sponsors(1) + AccountByCode(1) + 循环检测(最多20次)
        // 写入：Sponsors(1) + ReferralStats(1)
        Weight::from_parts(25_000, 0)
            .saturating_add(Weight::from_parts(0, 3000))
    }

    fn claim_code() -> Weight {
        // 读取：MembershipProvider(1) + AccountByCode(1) + CodeByAccount(1)
        // 写入：AccountByCode(1) + CodeByAccount(1) + ReferralStats(1)
        Weight::from_parts(20_000, 0)
            .saturating_add(Weight::from_parts(0, 2500))
    }
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn bind_sponsor() -> Weight {
        // 读取：Sponsors(1) + AccountByCode(1) + 循环检测(最多20次)
        // 写入：Sponsors(1) + ReferralStats(1)
        Weight::from_parts(25_000, 0)
            .saturating_add(Weight::from_parts(0, 3000))
    }

    fn claim_code() -> Weight {
        // 读取：MembershipProvider(1) + AccountByCode(1) + CodeByAccount(1)
        // 写入：AccountByCode(1) + CodeByAccount(1) + ReferralStats(1)
        Weight::from_parts(20_000, 0)
            .saturating_add(Weight::from_parts(0, 2500))
    }
}
