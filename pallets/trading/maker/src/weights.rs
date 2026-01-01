// 函数级详细中文注释：Maker Pallet 权重定义
//
// 本文件定义了 Maker Pallet 所有 extrinsic 的权重计算

use frame_support::weights::Weight;

/// 函数级详细中文注释：权重信息 trait
pub trait WeightInfo {
    fn lock_deposit() -> Weight;
    fn submit_info() -> Weight;
    fn approve_maker() -> Weight;
    fn reject_maker() -> Weight;
}

/// 函数级详细中文注释：默认权重实现（用于测试）
impl WeightInfo for () {
    fn lock_deposit() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    
    fn submit_info() -> Weight {
        Weight::from_parts(20_000, 0)
    }
    
    fn approve_maker() -> Weight {
        Weight::from_parts(15_000, 0)
    }
    
    fn reject_maker() -> Weight {
        Weight::from_parts(15_000, 0)
    }
}

