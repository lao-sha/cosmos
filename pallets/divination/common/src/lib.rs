//! # 玄学公共模块 (pallet-divination-common)
//!
//! 本模块提供所有玄学系统（梅花易数、八字排盘、六爻等）共用的类型和 trait 定义。
//!
//! ## 概述
//!
//! 玄学公共模块是 Cosmos 链上玄学生态的基础设施层，提供：
//!
//! - **统一类型定义**：`DivinationType`、`Rarity`、`RarityInput` 等
//! - **核心 Trait**：`DivinationProvider`、`InterpretationContextGenerator`
//! - **状态枚举**：订单状态、解读状态、争议状态等
//! - **存储押金**：`PrivacyMode`、`DepositConfig`、押金计算与返还逻辑
//!
//! ## 模块架构
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        应用层 (Application Layer)                │
//! ├─────────────┬─────────────┬─────────────┬─────────────┬─────────┤
//! │ pallet-     │ pallet-     │ pallet-     │ pallet-     │ ...     │
//! │ meihua      │ bazi-chart  │ liuyao      │ qimen       │         │
//! ├─────────────┴─────────────┴─────────────┴─────────────┴─────────┤
//! │                        公共服务层 (Common Service Layer)          │
//! ├─────────────────────┬─────────────────────┬─────────────────────┤
//! │ pallet-divination-  │ pallet-divination-  │ pallet-divination-  │
//! │ ai                  │ market              │ nft                 │
//! ├─────────────────────┴─────────────────────┴─────────────────────┤
//! │              pallet-divination-common (本模块)                    │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ### 在 Runtime 中实现 DivinationProvider
//!
//! ```ignore
//! use pallet_divination_common::{DivinationProvider, DivinationType, RarityInput};
//!
//! pub struct MeihuaDivinationProvider;
//!
//! impl DivinationProvider<AccountId> for MeihuaDivinationProvider {
//!     fn result_exists(divination_type: DivinationType, result_id: u64) -> bool {
//!         match divination_type {
//!             DivinationType::Meihua => Meihua::hexagrams(result_id).is_some(),
//!             _ => false,
//!         }
//!     }
//!
//!     fn result_creator(divination_type: DivinationType, result_id: u64) -> Option<AccountId> {
//!         match divination_type {
//!             DivinationType::Meihua => {
//!                 Meihua::hexagrams(result_id).map(|h| h.ben_gua.diviner)
//!             },
//!             _ => None,
//!         }
//!     }
//!
//!     fn rarity_data(divination_type: DivinationType, result_id: u64) -> Option<RarityInput> {
//!         match divination_type {
//!             DivinationType::Meihua => {
//!                 Meihua::hexagrams(result_id).map(|h| {
//!                     let is_pure = h.ben_gua.shang_gua == h.ben_gua.xia_gua;
//!                     RarityInput {
//!                         primary_score: if is_pure { 80 } else { 30 },
//!                         secondary_score: 10,
//!                         is_special_date: false,
//!                         is_special_combination: is_pure,
//!                         custom_factors: [0, 0, 0, 0],
//!                     }
//!                 })
//!             },
//!             _ => None,
//!         }
//!     }
//!
//!     // ... 其他方法实现
//! }
//! ```
//!
//! ### 使用稀有度计算
//!
//! ```ignore
//! use pallet_divination_common::{RarityInput, Rarity};
//!
//! let input = RarityInput {
//!     primary_score: 80,      // 纯卦高分
//!     secondary_score: 20,    // 初爻变
//!     is_special_date: true,  // 重阳节
//!     is_special_combination: true, // 纯卦
//!     custom_factors: [0, 0, 0, 0],
//! };
//!
//! let rarity = input.calculate_rarity();
//! assert_eq!(rarity, Rarity::Legendary);
//! ```
//!
//! ## 扩展新玄学系统
//!
//! 添加新的玄学系统（如六爻占卜）时：
//!
//! 1. 在 `DivinationType` 枚举中添加新类型（已预留）
//! 2. 创建新的核心 pallet（如 `pallet-liuyao`）
//! 3. 在 Runtime 中实现 `DivinationProvider` trait
//! 4. 公共服务模块（AI、Market、NFT）自动支持新系统

#![cfg_attr(not(feature = "std"), no_std)]

pub mod deposit;
pub mod traits;
pub mod types;

// 重新导出所有公共类型
pub use traits::{
    Deletable,
    DivinationProvider,
    InterpretationContextGenerator,
    NftMetadataGenerator,
    NullContextGenerator,
    NullDeletable,
    NullDivinationProvider,
    NullMetadataGenerator,
    NullStorageDepositManager,
    StorageDepositManager,
};

pub use types::{
    BountyAnswerStatus,
    BountyStatus,
    DisputeResolution,
    DisputeStatus,
    DivinationType,
    InterpretationStatus,
    InterpretationType,
    OrderStatus,
    ProviderTier,
    Rarity,
    RarityInput,
    ServiceType,
    Specialty,
};

pub use deposit::{
    calculate_refund_amount,
    calculate_storage_deposit,
    estimate_data_size,
    DepositConfig,
    DepositRecord,
    PrivacyMode,
    BLOCKS_PER_30_DAYS,
};

/// 模块版本
pub const VERSION: &str = "0.1.0";

/// 支持的最大占卜类型数量
pub const MAX_DIVINATION_TYPES: u8 = 16;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.1.0");
    }

    #[test]
    fn test_exports() {
        // 验证所有类型都正确导出
        let _ = DivinationType::Meihua;
        let _ = Rarity::Common;
        let _ = RarityInput::common();
        let _ = InterpretationType::Basic;
        let _ = InterpretationStatus::Pending;
        let _ = ServiceType::TextReading;
        let _ = OrderStatus::PendingPayment;
        let _ = ProviderTier::Novice;
        let _ = DisputeStatus::Pending;
    }

    #[test]
    fn test_rarity_calculation_integration() {
        // 测试完整的稀有度计算流程
        // common() 返回 primary_score=30, secondary_score=10
        // 30*3 + 10*2 = 90 + 20 = 110 → Rare (101-200)
        let common_input = RarityInput::common();
        assert_eq!(common_input.calculate_rarity(), Rarity::Rare);

        let legendary_input = RarityInput {
            primary_score: 100,
            secondary_score: 100,
            is_special_date: true,
            is_special_combination: true,
            custom_factors: [50, 50, 0, 0],
        };
        assert_eq!(legendary_input.calculate_rarity(), Rarity::Legendary);
    }
}
