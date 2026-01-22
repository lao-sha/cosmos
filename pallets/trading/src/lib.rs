#![cfg_attr(not(feature = "std"), no_std)]

//! # Trading Pallet - 统一接口层 (Unified Interface Layer)
//! 
//! ## 📦 重构后的架构 (Phase 5)
//! 
//! ### 函数级详细中文注释：模块化设计
//! 
//! 本 Pallet 是**统一接口层**，聚合以下独立模块：
//! 
//! 1. **pallet-maker** - 做市商管理（Application、审核、押金、提现）
//! 2. **pallet-otc-order** - OTC 订单管理（创建、支付、释放、取消、争议）
//! 3. **pallet-bridge** - DUST ↔ USDT 桥接（Swap、兑换、OCW）
//! 4. **pallet-trading-common** - 公共工具（数据掩码、验证）
//! 
//! ### 架构优势
//! 
//! ```text
//! 新架构（模块化）
//! ========================
//! pallet-trading (统一接口层，本文件)
//!   ├── 重新导出子模块类型
//!   ├── 提供聚合查询接口
//!   └── 简化 Runtime 集成
//! 
//! pallet-maker (独立模块)
//!   ├── 做市商申请/审核
//!   ├── 押金管理
//!   └── 提现流程
//! 
//! pallet-otc-order (独立模块)
//!   ├── 订单创建/支付
//!   ├── DUST释放
//!   ├── 首购逻辑
//!   └── 自动过期
//! 
//! pallet-bridge (独立模块)
//!   ├── DUST ↔ USDT兑换
//!   ├── OCW处理
//!   └── 做市商兑换
//! 
//! pallet-trading-common (工具库)
//!   ├── 数据掩码（姓名、身份证、生日）
//!   └── 数据验证（TRON地址）
//! ```
//! 
//! ### 重构优势
//! 
//! - ✅ **低耦合**: 子模块独立开发、测试、部署
//! - ✅ **高内聚**: 每个模块职责单一清晰
//! - ✅ **易维护**: 修改子模块不影响其他模块
//! - ✅ **易测试**: 独立模块独立测试
//! - ✅ **灵活集成**: Runtime 可选择性集成子模块或全部
//! 
//! ### 兼容性
//! 
//! - ✅ 保留所有现有功能
//! - ✅ 前端 API 可平滑迁移
//! - ✅ 零迁移策略（主网未上线）
//! 
//! ## 使用示例
//! 
//! ### Runtime 集成 - 推荐方式：直接集成子模块
//! 
//! ```rust,ignore
//! impl pallet_maker::Config for Runtime {
//!     type RuntimeEvent = RuntimeEvent;
//!     type Currency = Balances;
//!     type MakerCredit = Credit;
//!     type GovernanceOrigin = EnsureTreasury;
//!     type Timestamp = Timestamp;
//!     type MakerDepositAmount = MakerDeposit;
//!     type MakerApplicationTimeout = MakerTimeout;
//!     type WithdrawalCooldown = WithdrawalCooldown;
//!     type WeightInfo = ();
//! }
//! 
//! impl pallet_otc_order::Config for Runtime {
//!     type RuntimeEvent = RuntimeEvent;
//!     type Currency = Balances;
//!     type Timestamp = Timestamp;
//!     type Escrow = Escrow;
//!     type Credit = Credit;
//!     type Pricing = Pricing;
//!     type OrderTimeout = OrderTimeout;
//!     // ... 其他配置
//!     type WeightInfo = ();
//! }
//! 
//! impl pallet_bridge::Config for Runtime {
//!     type RuntimeEvent = RuntimeEvent;
//!     type Currency = Balances;
//!     type Escrow = Escrow;
//!     type MakerPallet = Maker;
//!     type Credit = Credit;
//!     type Pricing = Pricing;
//!     type OcwSwapTimeoutBlocks = OcwSwapTimeout;
//!     type MinSwapAmount = MinSwapAmount;
//!     type WeightInfo = ();
//! }
//! 
//! construct_runtime! {
//!     pub struct Runtime {
//!         // ... 其他模块
//!         Maker: pallet_maker,
//!         OtcOrder: pallet_otc_order,
//!         Bridge: pallet_bridge,
//!     }
//! }
//! ```
//! 
//! ### 前端调用
//! 
//! ```typescript
//! // 直接调用子模块（推荐）
//! await api.tx.maker.lockDeposit().signAndSend(account);
//! const makerInfo = await api.query.maker.makerApplications(makerId);
//! 
//! await api.tx.otcOrder.createOrder(makerId, amount, tronAddr).signAndSend(account);
//! const orderInfo = await api.query.otcOrder.orders(orderId);
//! 
//! await api.tx.bridge.swap(amount, tronAddr).signAndSend(account);
//! const swapInfo = await api.query.bridge.swapRequests(swapId);
//! ```

// ===== 重新导出子模块 =====

/// 做市商管理模块
/// 
/// 提供做市商申请、审核、押金管理、提现流程等功能。
pub use pallet_maker;

/// OTC 订单管理模块
/// 
/// 提供 OTC 订单创建、支付、释放、取消、争议、首购逻辑等功能。
pub use pallet_otc_order;

/// DUST ↔ USDT 桥接模块
/// 
/// 提供官方桥接、做市商兑换、OCW 处理等功能。
pub use pallet_bridge;

/// 公共工具模块
/// 
/// 提供数据掩码（姓名、身份证、生日）和数据验证（TRON 地址）功能。
pub use pallet_trading_common;

// ===== 聚合类型导出（便于前端使用）=====

/// Maker 相关类型
pub mod maker_types {
    pub use pallet_maker::{
        MakerApplication,
        ApplicationStatus,
        Direction,
        WithdrawalRequest,
        WithdrawalStatus,
    };
}

/// OTC 相关类型
pub mod otc_types {
    pub use pallet_otc_order::{
        Order,
        OrderState,
        PricingProvider,
    };
}

/// Bridge 相关类型
pub mod bridge_types {
    pub use pallet_bridge::{
        SwapRequest,
        SwapStatus,
        MakerSwapRecord,
    };
}

/// 公共工具
pub mod utils {
    pub use pallet_trading_common::{
        mask_name,
        mask_id_card,
        mask_birthday,
        is_valid_tron_address,
    };
}

// ===== 聚合查询 API（可选，便于前端使用）=====

/// 函数级详细中文注释：Trading 聚合查询 API
/// 
/// 提供跨模块的聚合查询接口，简化前端调用。
/// 
/// # 注意
/// 
/// 这些是纯查询接口，不需要在 Runtime 中集成 pallet-trading。
/// 前端可以直接调用这些静态方法。
pub struct TradingApi;

impl TradingApi {
    /// 函数级详细中文注释：获取平台统计信息
    /// 
    /// 聚合查询平台的总做市商数、总订单数、总兑换数。
    /// 
    /// # 返回
    /// - `PlatformStats`: 平台统计信息
    /// 
    /// # 示例
    /// 
    /// ```rust,ignore
    /// let stats = TradingApi::get_platform_stats::<Runtime>();
    /// println!("Total makers: {}", stats.total_makers);
    /// ```
    pub fn get_platform_stats<T>() -> PlatformStats
    where
        T: pallet_maker::Config + pallet_otc_order::Config + pallet_bridge::Config,
    {
        PlatformStats {
            total_makers: pallet_maker::NextMakerId::<T>::get(),
            total_orders: pallet_otc_order::NextOrderId::<T>::get(),
            total_swaps: pallet_bridge::NextSwapId::<T>::get(),
        }
    }
}

// ===== 聚合数据结构 =====

use codec::{Encode, Decode};
use scale_info::TypeInfo;

/// 函数级详细中文注释：平台统计信息
/// 
/// 聚合平台的总做市商数、总订单数、总兑换数。
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct PlatformStats {
    /// 总做市商数
    pub total_makers: u64,
    /// 总订单数
    pub total_orders: u64,
    /// 总兑换数
    pub total_swaps: u64,
}
