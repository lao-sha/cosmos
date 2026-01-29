//! # 权重定义模块
//!
//! 本模块定义了 OTC Order Pallet 的所有extrinsics权重

use frame_support::weights::Weight;

/// 函数级详细中文注释：权重信息 trait
pub trait WeightInfo {
    fn create_order() -> Weight;
    fn mark_paid() -> Weight;
    fn release_cos() -> Weight;
    fn cancel_order() -> Weight;
    fn dispute_order() -> Weight;

    // === KYC相关函数权重 ===
    fn enable_kyc_requirement() -> Weight;
    fn disable_kyc_requirement() -> Weight;
    fn update_min_judgment_level() -> Weight;
    fn exempt_account_from_kyc() -> Weight;
    fn remove_kyc_exemption() -> Weight;
}

/// 函数级详细中文注释：默认权重实现（临时占位）
impl WeightInfo for () {
    fn create_order() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn mark_paid() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn release_cos() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn cancel_order() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn dispute_order() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    // === KYC相关函数权重实现 ===
    fn enable_kyc_requirement() -> Weight {
        Weight::from_parts(20_000, 0)
    }

    fn disable_kyc_requirement() -> Weight {
        Weight::from_parts(15_000, 0)
    }

    fn update_min_judgment_level() -> Weight {
        Weight::from_parts(15_000, 0)
    }

    fn exempt_account_from_kyc() -> Weight {
        Weight::from_parts(25_000, 0)
    }

    fn remove_kyc_exemption() -> Weight {
        Weight::from_parts(20_000, 0)
    }
}

