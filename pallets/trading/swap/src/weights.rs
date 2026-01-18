//! # 权重定义模块
//!
//! 本模块定义了 Bridge Pallet 的所有extrinsics权重

use frame_support::weights::Weight;

/// 函数级详细中文注释：权重信息 trait
pub trait WeightInfo {
    fn maker_swap() -> Weight;
    fn mark_swap_complete() -> Weight;
    fn report_swap() -> Weight;
}

/// 函数级详细中文注释：默认权重实现（临时占位）
impl WeightInfo for () {
    fn maker_swap() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    
    fn mark_swap_complete() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    
    fn report_swap() -> Weight {
        Weight::from_parts(10_000, 0)
    }
}

