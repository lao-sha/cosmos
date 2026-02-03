//! # 店铺代币交易市场模块 (pallet-entity-market)
//!
//! ## 概述
//!
//! 本模块实现店铺代币的 P2P 交易市场，支持：
//! - COS 通道：使用原生 COS 代币买卖店铺代币（链上即时结算）
//! - USDT 通道：使用 TRC20 USDT 买卖店铺代币（需 OCW 验证）
//!
//! ## 交易模式
//!
//! - 限价单：挂单等待撮合
//! - 吃单：直接成交对手盘订单
//!
//! ## 版本历史
//!
//! - v0.1.0 (2026-02-01): 初始版本，实现 COS 通道限价单
//! - v0.2.0 (2026-02-01): Phase 2，实现 USDT 通道 + OCW 验证

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use alloc::vec::Vec;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, ReservableCurrency},
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use pallet_entity_common::{ShopProvider, ShopTokenProvider};
    use sp_runtime::traits::{CheckedAdd, CheckedMul, CheckedSub, Saturating, SaturatedConversion, Zero};

    /// Balance 类型别名
    pub type BalanceOf<T> = <T as Config>::Balance;

    // ==================== 数据结构 ====================

    /// 订单方向
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub enum OrderSide {
        /// 买单（用 COS 买 Token）
        Buy,
        /// 卖单（卖 Token 得 COS）
        Sell,
    }

    /// 支付通道
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub enum PaymentChannel {
        /// 原生 COS
        COS,
        /// TRC20 USDT（Phase 2）
        USDT,
    }

    /// 订单类型
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub enum OrderType {
        /// 限价单（挂单等待撮合）
        #[default]
        Limit,
        /// 市价单（立即以最优价成交）
        Market,
    }

    /// 订单状态
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub enum OrderStatus {
        /// 挂单中
        Open,
        /// 部分成交
        PartiallyFilled,
        /// 完全成交
        Filled,
        /// 已取消
        Cancelled,
        /// 已过期
        Expired,
    }

    /// USDT 交易状态
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub enum UsdtTradeStatus {
        /// 等待买家支付 USDT
        AwaitingPayment,
        /// 等待 OCW 验证
        AwaitingVerification,
        /// 已完成
        Completed,
        /// 争议中
        Disputed,
        /// 已取消
        Cancelled,
        /// 已退款（超时）
        Refunded,
    }

    /// TRON 地址类型（34 字节 Base58）
    pub type TronAddress = BoundedVec<u8, ConstU32<34>>;

    /// TRON 交易哈希类型（64 字节 hex）
    pub type TronTxHash = BoundedVec<u8, ConstU32<64>>;

    /// 交易订单
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct TradeOrder<T: Config> {
        /// 订单 ID
        pub order_id: u64,
        /// 店铺 ID
        pub shop_id: u64,
        /// 挂单者
        pub maker: T::AccountId,
        /// 订单方向
        pub side: OrderSide,
        /// 订单类型
        pub order_type: OrderType,
        /// 支付通道
        pub channel: PaymentChannel,
        /// 代币数量（总量）
        pub token_amount: T::TokenBalance,
        /// 已成交数量
        pub filled_amount: T::TokenBalance,
        /// 价格（COS 通道：每个 Token 的 COS 价格；USDT 通道：每个 Token 的 USDT 价格，精度 10^6）
        /// 市价单时为 0
        pub price: BalanceOf<T>,
        /// USDT 价格（仅 USDT 通道使用，精度 10^6）
        pub usdt_price: u64,
        /// TRON 收款地址（仅 USDT 卖单使用）
        pub tron_address: Option<TronAddress>,
        /// 订单状态
        pub status: OrderStatus,
        /// 创建区块
        pub created_at: BlockNumberFor<T>,
        /// 过期区块
        pub expires_at: BlockNumberFor<T>,
    }

    /// USDT 交易记录（等待验证）
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct UsdtTrade<T: Config> {
        /// 交易 ID
        pub trade_id: u64,
        /// 关联订单 ID
        pub order_id: u64,
        /// 店铺 ID
        pub shop_id: u64,
        /// 卖家
        pub seller: T::AccountId,
        /// 买家
        pub buyer: T::AccountId,
        /// 代币数量
        pub token_amount: T::TokenBalance,
        /// USDT 金额（精度 10^6）
        pub usdt_amount: u64,
        /// 卖家 TRON 收款地址
        pub seller_tron_address: TronAddress,
        /// 买家提交的 TRON 交易哈希
        pub tron_tx_hash: Option<TronTxHash>,
        /// 交易状态
        pub status: UsdtTradeStatus,
        /// 创建区块
        pub created_at: BlockNumberFor<T>,
        /// 超时区块
        pub timeout_at: BlockNumberFor<T>,
    }

    /// 店铺市场配置
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct MarketConfig<Balance> {
        /// 是否启用 COS 交易
        pub cos_enabled: bool,
        /// 是否启用 USDT 交易
        pub usdt_enabled: bool,
        /// 交易手续费率（基点，100 = 1%）
        pub fee_rate: u16,
        /// 最小订单 Token 数量
        pub min_order_amount: u128,
        /// 订单有效期（区块数）
        pub order_ttl: u32,
        /// USDT 交易超时（区块数）
        pub usdt_timeout: u32,
        /// 手续费接收账户（None = 店铺账户）
        pub fee_recipient: Option<Balance>,
    }

    /// 市场统计数据
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct MarketStats {
        /// 总订单数
        pub total_orders: u64,
        /// 总成交数
        pub total_trades: u64,
        /// COS 总交易量
        pub total_volume_cos: u128,
        /// USDT 总交易量（精度 10^6）
        pub total_volume_usdt: u64,
        /// 总手续费（COS）
        pub total_fees_cos: u128,
        /// 总手续费（USDT，精度 10^6）
        pub total_fees_usdt: u64,
    }

    // ==================== Phase 4: 订单簿深度数据结构 ====================

    /// 价格档位（聚合同一价格的订单）
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct PriceLevel<Balance, TokenBalance> {
        /// 价格
        pub price: Balance,
        /// 该价格的总数量
        pub total_amount: TokenBalance,
        /// 订单数量
        pub order_count: u32,
    }

    /// 订单簿深度（买卖盘）
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, RuntimeDebug)]
    pub struct OrderBookDepth<Balance, TokenBalance> {
        /// 店铺 ID
        pub shop_id: u64,
        /// 卖盘（按价格升序，最优卖价在前）
        pub asks: Vec<PriceLevel<Balance, TokenBalance>>,
        /// 买盘（按价格降序，最优买价在前）
        pub bids: Vec<PriceLevel<Balance, TokenBalance>>,
        /// 最优卖价
        pub best_ask: Option<Balance>,
        /// 最优买价
        pub best_bid: Option<Balance>,
        /// 买卖价差
        pub spread: Option<Balance>,
        /// 快照区块
        pub block_number: u32,
    }

    /// 市场摘要
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct MarketSummary<Balance, TokenBalance> {
        /// 最优卖价
        pub best_ask: Option<Balance>,
        /// 最优买价
        pub best_bid: Option<Balance>,
        /// 24h 最高价
        pub high_24h: Balance,
        /// 24h 最低价
        pub low_24h: Balance,
        /// 24h 成交量
        pub volume_24h: TokenBalance,
        /// 最新成交价
        pub last_price: Option<Balance>,
        /// 卖单总量
        pub total_ask_amount: TokenBalance,
        /// 买单总量
        pub total_bid_amount: TokenBalance,
    }

    // ==================== Phase 5: TWAP 价格预言机数据结构 ====================

    /// 价格快照（用于 TWAP 计算）
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct PriceSnapshot {
        /// 累积价格 (price × blocks)
        pub cumulative_price: u128,
        /// 快照区块号
        pub block_number: u32,
    }

    /// TWAP 累积器（三周期：1小时、24小时、7天）
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct TwapAccumulator<Balance> {
        /// 当前累积价格
        pub current_cumulative: u128,
        /// 当前区块号
        pub current_block: u32,
        /// 最新成交价
        pub last_price: Balance,
        /// 总成交次数（用于判断市场活跃度）
        pub trade_count: u64,

        /// 1小时前快照（用于 1小时 TWAP）
        pub hour_snapshot: PriceSnapshot,
        /// 24小时前快照（用于 24小时 TWAP）
        pub day_snapshot: PriceSnapshot,
        /// 7天前快照（用于 7天 TWAP）
        pub week_snapshot: PriceSnapshot,

        /// 上次更新 1小时快照的区块
        pub last_hour_update: u32,
        /// 上次更新 24小时快照的区块
        pub last_day_update: u32,
        /// 上次更新 7天快照的区块
        pub last_week_update: u32,
    }

    /// TWAP 周期类型
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub enum TwapPeriod {
        /// 1小时（600 区块，假设 6秒/区块）
        OneHour,
        /// 24小时（14400 区块）
        OneDay,
        /// 7天（100800 区块）
        OneWeek,
    }

    /// 价格保护配置
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct PriceProtectionConfig<Balance> {
        /// 是否启用价格保护
        pub enabled: bool,
        /// 限价单最大价格偏离（基点，1000 = 10%）
        pub max_price_deviation: u16,
        /// 市价单最大滑点（基点，500 = 5%）
        pub max_slippage: u16,
        /// 熔断触发阈值（基点，5000 = 50%）
        pub circuit_breaker_threshold: u16,
        /// 启用 TWAP 保护的最小成交数
        pub min_trades_for_twap: u64,
        /// 市场是否处于熔断状态
        pub circuit_breaker_active: bool,
        /// 熔断结束区块
        pub circuit_breaker_until: u32,
        /// 店主设定的初始参考价格（用于 TWAP 冷启动）
        pub initial_price: Option<Balance>,
    }

    impl<Balance: Default> Default for PriceProtectionConfig<Balance> {
        fn default() -> Self {
            Self {
                enabled: true,
                max_price_deviation: 2000,        // 20%
                max_slippage: 500,                // 5%
                circuit_breaker_threshold: 5000,  // 50%
                min_trades_for_twap: 100,
                circuit_breaker_active: false,
                circuit_breaker_until: 0,
                initial_price: None,
            }
        }
    }

    // ==================== Config ====================

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 原生货币（COS）
        type Currency: Currency<Self::AccountId, Balance = Self::Balance> + ReservableCurrency<Self::AccountId>;

        /// Balance 类型（需要支持 u128 转换）
        type Balance: Member
            + Parameter
            + Copy
            + Default
            + MaxEncodedLen
            + From<u128>
            + Into<u128>
            + From<u32>
            + From<u16>
            + Saturating
            + Zero
            + Ord
            + sp_runtime::traits::CheckedDiv;

        /// 店铺代币余额类型
        type TokenBalance: Member
            + Parameter
            + Copy
            + Default
            + MaxEncodedLen
            + From<u128>
            + Into<u128>
            + CheckedAdd
            + CheckedSub
            + CheckedMul
            + Saturating
            + Zero
            + Ord;

        /// 店铺查询接口
        type ShopProvider: ShopProvider<Self::AccountId>;

        /// 店铺代币接口
        type TokenProvider: ShopTokenProvider<Self::AccountId, Self::TokenBalance>;

        /// 默认订单有效期（区块数）
        #[pallet::constant]
        type DefaultOrderTTL: Get<u32>;

        /// 最大活跃订单数（每用户每店铺）
        #[pallet::constant]
        type MaxActiveOrdersPerUser: Get<u32>;

        /// 默认手续费率（基点）
        #[pallet::constant]
        type DefaultFeeRate: Get<u16>;

        /// USDT 交易默认超时（区块数）
        #[pallet::constant]
        type DefaultUsdtTimeout: Get<u32>;

        /// 1小时对应的区块数（默认 600，假设 6秒/区块）
        #[pallet::constant]
        type BlocksPerHour: Get<u32>;

        /// 24小时对应的区块数（默认 14400）
        #[pallet::constant]
        type BlocksPerDay: Get<u32>;

        /// 7天对应的区块数（默认 100800）
        #[pallet::constant]
        type BlocksPerWeek: Get<u32>;

        /// 熔断持续时间（区块数，默认 600 = 1小时）
        #[pallet::constant]
        type CircuitBreakerDuration: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ==================== 存储项 ====================

    /// 下一个订单 ID
    #[pallet::storage]
    #[pallet::getter(fn next_order_id)]
    pub type NextOrderId<T> = StorageValue<_, u64, ValueQuery>;

    /// 订单存储
    #[pallet::storage]
    #[pallet::getter(fn orders)]
    pub type Orders<T: Config> = StorageMap<_, Blake2_128Concat, u64, TradeOrder<T>>;

    /// 店铺订单簿 - 卖单（按店铺索引）
    #[pallet::storage]
    #[pallet::getter(fn shop_sell_orders)]
    pub type ShopSellOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // shop_id
        BoundedVec<u64, ConstU32<1000>>,
        ValueQuery,
    >;

    /// 店铺订单簿 - 买单（按店铺索引）
    #[pallet::storage]
    #[pallet::getter(fn shop_buy_orders)]
    pub type ShopBuyOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // shop_id
        BoundedVec<u64, ConstU32<1000>>,
        ValueQuery,
    >;

    /// 用户订单（按用户索引）
    #[pallet::storage]
    #[pallet::getter(fn user_orders)]
    pub type UserOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<100>>,
        ValueQuery,
    >;

    /// 店铺市场配置
    #[pallet::storage]
    #[pallet::getter(fn market_configs)]
    pub type MarketConfigs<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, MarketConfig<BalanceOf<T>>>;

    /// 店铺市场统计
    #[pallet::storage]
    #[pallet::getter(fn market_stats)]
    pub type MarketStatsStorage<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, MarketStats, ValueQuery>;

    /// 下一个 USDT 交易 ID
    #[pallet::storage]
    #[pallet::getter(fn next_usdt_trade_id)]
    pub type NextUsdtTradeId<T> = StorageValue<_, u64, ValueQuery>;

    /// USDT 交易记录存储
    #[pallet::storage]
    #[pallet::getter(fn usdt_trades)]
    pub type UsdtTrades<T: Config> = StorageMap<_, Blake2_128Concat, u64, UsdtTrade<T>>;

    /// 待验证的 USDT 交易列表（供 OCW 使用）
    #[pallet::storage]
    #[pallet::getter(fn pending_usdt_trades)]
    pub type PendingUsdtTrades<T: Config> = StorageValue<_, BoundedVec<u64, ConstU32<100>>, ValueQuery>;

    // ==================== Phase 4: 订单簿深度存储 ====================

    /// 店铺最优卖价
    #[pallet::storage]
    #[pallet::getter(fn best_ask)]
    pub type BestAsk<T: Config> = StorageMap<_, Blake2_128Concat, u64, BalanceOf<T>>;

    /// 店铺最优买价
    #[pallet::storage]
    #[pallet::getter(fn best_bid)]
    pub type BestBid<T: Config> = StorageMap<_, Blake2_128Concat, u64, BalanceOf<T>>;

    /// 店铺最新成交价
    #[pallet::storage]
    #[pallet::getter(fn last_trade_price)]
    pub type LastTradePrice<T: Config> = StorageMap<_, Blake2_128Concat, u64, BalanceOf<T>>;

    /// 店铺市场摘要
    #[pallet::storage]
    #[pallet::getter(fn market_summary)]
    pub type MarketSummaryStorage<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        MarketSummary<BalanceOf<T>, T::TokenBalance>,
    >;

    // ==================== Phase 5: TWAP 价格预言机存储 ====================

    /// TWAP 累积器（每个店铺一个）
    #[pallet::storage]
    #[pallet::getter(fn twap_accumulator)]
    pub type TwapAccumulators<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // shop_id
        TwapAccumulator<BalanceOf<T>>,
    >;

    /// 价格保护配置（每个店铺一个）
    #[pallet::storage]
    #[pallet::getter(fn price_protection)]
    pub type PriceProtection<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // shop_id
        PriceProtectionConfig<BalanceOf<T>>,
    >;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 订单已创建
        OrderCreated {
            order_id: u64,
            shop_id: u64,
            maker: T::AccountId,
            side: OrderSide,
            token_amount: T::TokenBalance,
            price: BalanceOf<T>,
        },
        /// 订单已成交（部分或全部）
        OrderFilled {
            order_id: u64,
            taker: T::AccountId,
            filled_amount: T::TokenBalance,
            total_cost: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 订单已取消
        OrderCancelled { order_id: u64 },
        /// 市场配置已更新
        MarketConfigured { shop_id: u64 },
        /// USDT 卖单已创建
        UsdtSellOrderCreated {
            order_id: u64,
            shop_id: u64,
            maker: T::AccountId,
            token_amount: T::TokenBalance,
            usdt_price: u64,
            tron_address: TronAddress,
        },
        /// USDT 买单已创建
        UsdtBuyOrderCreated {
            order_id: u64,
            shop_id: u64,
            maker: T::AccountId,
            token_amount: T::TokenBalance,
            usdt_price: u64,
        },
        /// USDT 交易已创建（等待支付）
        UsdtTradeCreated {
            trade_id: u64,
            order_id: u64,
            seller: T::AccountId,
            buyer: T::AccountId,
            token_amount: T::TokenBalance,
            usdt_amount: u64,
        },
        /// USDT 支付已提交（等待验证）
        UsdtPaymentSubmitted {
            trade_id: u64,
            tron_tx_hash: TronTxHash,
        },
        /// USDT 交易已完成（OCW 验证通过）
        UsdtTradeCompleted {
            trade_id: u64,
            order_id: u64,
        },
        /// USDT 交易验证失败
        UsdtTradeVerificationFailed {
            trade_id: u64,
            reason: Vec<u8>,
        },
        /// USDT 交易已超时退款
        UsdtTradeRefunded {
            trade_id: u64,
        },
        /// 市价单已执行
        MarketOrderExecuted {
            shop_id: u64,
            trader: T::AccountId,
            side: OrderSide,
            filled_amount: T::TokenBalance,
            total_cost: BalanceOf<T>,
            total_fee: BalanceOf<T>,
        },
        /// TWAP 价格已更新
        TwapUpdated {
            shop_id: u64,
            new_price: BalanceOf<T>,
            twap_1h: Option<BalanceOf<T>>,
            twap_24h: Option<BalanceOf<T>>,
            twap_7d: Option<BalanceOf<T>>,
        },
        /// 熔断已触发
        CircuitBreakerTriggered {
            shop_id: u64,
            current_price: BalanceOf<T>,
            twap_7d: BalanceOf<T>,
            deviation_bps: u16,
            until_block: u32,
        },
        /// 熔断已解除
        CircuitBreakerLifted {
            shop_id: u64,
        },
        /// 价格保护配置已更新
        PriceProtectionConfigured {
            shop_id: u64,
            enabled: bool,
            max_deviation: u16,
            max_slippage: u16,
        },
        /// 初始价格已设置
        InitialPriceSet {
            shop_id: u64,
            initial_price: BalanceOf<T>,
        },
    }

    // ==================== 错误 ====================

    #[pallet::error]
    pub enum Error<T> {
        /// 店铺不存在
        ShopNotFound,
        /// 不是店主
        NotShopOwner,
        /// 店铺代币未启用
        TokenNotEnabled,
        /// 市场未启用
        MarketNotEnabled,
        /// 订单不存在
        OrderNotFound,
        /// 不是订单所有者
        NotOrderOwner,
        /// 订单已关闭
        OrderClosed,
        /// 余额不足
        InsufficientBalance,
        /// Token 余额不足
        InsufficientTokenBalance,
        /// 数量过小
        AmountTooSmall,
        /// 数量超过可用
        AmountExceedsAvailable,
        /// 价格为零
        ZeroPrice,
        /// 订单簿已满
        OrderBookFull,
        /// 用户订单数已满
        UserOrdersFull,
        /// 不能吃自己的单
        CannotTakeOwnOrder,
        /// 算术溢出
        ArithmeticOverflow,
        /// 订单方向不匹配
        OrderSideMismatch,
        /// USDT 市场未启用
        UsdtMarketNotEnabled,
        /// 无效的 TRON 地址
        InvalidTronAddress,
        /// USDT 交易不存在
        UsdtTradeNotFound,
        /// 不是交易参与者
        NotTradeParticipant,
        /// 交易状态无效
        InvalidTradeStatus,
        /// 交易已超时
        TradeTimeout,
        /// 待验证队列已满
        PendingQueueFull,
        /// 无效的交易哈希
        InvalidTxHash,
        /// 通道不匹配
        ChannelMismatch,
        /// 没有可用订单
        NoOrdersAvailable,
        /// 滑点超限
        SlippageExceeded,
        /// 价格偏离 TWAP 过大
        PriceDeviationTooHigh,
        /// 市场处于熔断状态
        MarketCircuitBreakerActive,
        /// TWAP 数据不足
        InsufficientTwapData,
    }

    // ==================== Extrinsics ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 挂卖单（卖 Token 得 COS）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `token_amount`: 出售的 Token 数量
        /// - `price`: 每个 Token 的 COS 价格
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(80_000, 0))]
        pub fn place_sell_order(
            origin: OriginFor<T>,
            shop_id: u64,
            token_amount: T::TokenBalance,
            price: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证店铺和市场
            Self::ensure_market_enabled(shop_id)?;

            // 验证参数
            ensure!(!price.is_zero(), Error::<T>::ZeroPrice);
            ensure!(!token_amount.is_zero(), Error::<T>::AmountTooSmall);

            // Phase 5: 价格偏离检查
            Self::check_price_deviation(shop_id, price)?;

            // 检查用户 Token 余额
            let balance = T::TokenProvider::token_balance(shop_id, &who);
            ensure!(balance >= token_amount, Error::<T>::InsufficientTokenBalance);

            // 锁定 Token（通过转移到模块账户）
            // 注意：这里需要 TokenProvider 支持转账，暂时跳过实际锁定
            // 在实际实现中，应该调用 pallet-assets 的 transfer 或 reserve

            // 创建订单
            let order_id = Self::do_create_order(
                shop_id,
                who.clone(),
                OrderSide::Sell,
                OrderType::Limit,
                PaymentChannel::COS,
                token_amount,
                price,
                0,    // usdt_price (COS 通道不使用)
                None, // tron_address (COS 通道不使用)
            )?;

            // 更新最优价格
            Self::update_best_prices(shop_id);

            Self::deposit_event(Event::OrderCreated {
                order_id,
                shop_id,
                maker: who,
                side: OrderSide::Sell,
                token_amount,
                price,
            });

            Ok(())
        }

        /// 挂买单（用 COS 买 Token）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `token_amount`: 想购买的 Token 数量
        /// - `price`: 每个 Token 愿意支付的 COS 价格
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(80_000, 0))]
        pub fn place_buy_order(
            origin: OriginFor<T>,
            shop_id: u64,
            token_amount: T::TokenBalance,
            price: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证店铺和市场
            Self::ensure_market_enabled(shop_id)?;

            // 验证参数
            ensure!(!price.is_zero(), Error::<T>::ZeroPrice);
            ensure!(!token_amount.is_zero(), Error::<T>::AmountTooSmall);

            // Phase 5: 价格偏离检查
            Self::check_price_deviation(shop_id, price)?;

            // 计算需要锁定的 COS 总量
            let token_u128: u128 = token_amount.into();
            let total_cost = Self::calculate_total_cost(token_u128, price)?;

            // 锁定 COS
            T::Currency::reserve(&who, total_cost).map_err(|_| Error::<T>::InsufficientBalance)?;

            // 创建订单
            let order_id = Self::do_create_order(
                shop_id,
                who.clone(),
                OrderSide::Buy,
                OrderType::Limit,
                PaymentChannel::COS,
                token_amount,
                price,
                0,    // usdt_price (COS 通道不使用)
                None, // tron_address (COS 通道不使用)
            )?;

            // 更新最优价格
            Self::update_best_prices(shop_id);

            Self::deposit_event(Event::OrderCreated {
                order_id,
                shop_id,
                maker: who,
                side: OrderSide::Buy,
                token_amount,
                price,
            });

            Ok(())
        }

        /// 吃单（成交对手盘订单）
        ///
        /// # 参数
        /// - `order_id`: 要吃的订单 ID
        /// - `amount`: 成交数量（None = 全部）
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(100_000, 0))]
        pub fn take_order(
            origin: OriginFor<T>,
            order_id: u64,
            amount: Option<T::TokenBalance>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取订单
            let mut order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;

            // 验证订单状态
            ensure!(
                order.status == OrderStatus::Open || order.status == OrderStatus::PartiallyFilled,
                Error::<T>::OrderClosed
            );

            // 不能吃自己的单
            ensure!(order.maker != who, Error::<T>::CannotTakeOwnOrder);

            // 计算可成交数量
            let available = order
                .token_amount
                .checked_sub(&order.filled_amount)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
            let fill_amount = amount.unwrap_or(available).min(available);
            ensure!(!fill_amount.is_zero(), Error::<T>::AmountTooSmall);

            // 计算成交金额
            let fill_u128: u128 = fill_amount.into();
            let total_cost = Self::calculate_total_cost(fill_u128, order.price)?;

            // 计算手续费
            let config = MarketConfigs::<T>::get(order.shop_id).unwrap_or_default();
            let fee_rate = if config.fee_rate > 0 {
                config.fee_rate
            } else {
                T::DefaultFeeRate::get()
            };
            let fee = total_cost
                .saturating_mul(fee_rate.into())
                .checked_div(&10000u32.into())
                .unwrap_or_else(Zero::zero);
            let net_amount = total_cost.saturating_sub(fee);

            // 执行交易
            match order.side {
                OrderSide::Sell => {
                    // 卖单：taker 支付 COS，获得 Token
                    // taker (who) 支付 COS → maker
                    T::Currency::transfer(
                        &who,
                        &order.maker,
                        net_amount,
                        ExistenceRequirement::KeepAlive,
                    )?;

                    // 手续费转给店铺
                    if !fee.is_zero() {
                        if let Some(shop_owner) = T::ShopProvider::shop_owner(order.shop_id) {
                            T::Currency::transfer(
                                &who,
                                &shop_owner,
                                fee,
                                ExistenceRequirement::KeepAlive,
                            )?;
                        }
                    }

                    // Token: maker → taker
                    // 注意：实际实现需要从托管账户转出
                    // 这里暂时直接铸造给 taker（简化实现）
                }
                OrderSide::Buy => {
                    // 买单：taker 提供 Token，获得 COS
                    // 检查 taker 的 Token 余额
                    let taker_balance = T::TokenProvider::token_balance(order.shop_id, &who);
                    ensure!(
                        taker_balance >= fill_amount,
                        Error::<T>::InsufficientTokenBalance
                    );

                    // COS: 从 maker 的锁定中释放 → taker
                    T::Currency::unreserve(&order.maker, total_cost);
                    T::Currency::transfer(
                        &order.maker,
                        &who,
                        net_amount,
                        ExistenceRequirement::KeepAlive,
                    )?;

                    // 手续费
                    if !fee.is_zero() {
                        if let Some(shop_owner) = T::ShopProvider::shop_owner(order.shop_id) {
                            T::Currency::transfer(
                                &order.maker,
                                &shop_owner,
                                fee,
                                ExistenceRequirement::KeepAlive,
                            )?;
                        }
                    }

                    // Token: taker → maker
                    // 注意：实际实现需要调用 TokenProvider 的转账方法
                }
            }

            // 更新订单状态
            order.filled_amount = order
                .filled_amount
                .checked_add(&fill_amount)
                .ok_or(Error::<T>::ArithmeticOverflow)?;

            if order.filled_amount >= order.token_amount {
                order.status = OrderStatus::Filled;
                // 从订单簿移除
                Self::remove_from_order_book(order.shop_id, order_id, order.side);
            } else {
                order.status = OrderStatus::PartiallyFilled;
            }

            Orders::<T>::insert(order_id, &order);

            // 更新统计
            MarketStatsStorage::<T>::mutate(order.shop_id, |stats| {
                stats.total_trades = stats.total_trades.saturating_add(1);
                stats.total_volume_cos = stats.total_volume_cos.saturating_add(total_cost.into());
                stats.total_fees_cos = stats.total_fees_cos.saturating_add(fee.into());
            });

            // 更新最优价格和 TWAP
            Self::update_best_prices(order.shop_id);
            Self::on_trade_completed(order.shop_id, order.price);

            Self::deposit_event(Event::OrderFilled {
                order_id,
                taker: who,
                filled_amount: fill_amount,
                total_cost,
                fee,
            });

            Ok(())
        }

        /// 取消订单
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(50_000, 0))]
        pub fn cancel_order(origin: OriginFor<T>, order_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;

            // 验证所有权
            ensure!(order.maker == who, Error::<T>::NotOrderOwner);

            // 验证状态
            ensure!(
                order.status == OrderStatus::Open || order.status == OrderStatus::PartiallyFilled,
                Error::<T>::OrderClosed
            );

            // 计算未成交数量
            let unfilled = order
                .token_amount
                .checked_sub(&order.filled_amount)
                .ok_or(Error::<T>::ArithmeticOverflow)?;

            // 退还锁定资产
            match order.side {
                OrderSide::Sell => {
                    // 退还锁定的 Token
                    // 注意：实际实现需要从托管账户转回
                }
                OrderSide::Buy => {
                    // 退还锁定的 COS
                    let unfilled_u128: u128 = unfilled.into();
                    let refund = Self::calculate_total_cost(unfilled_u128, order.price)?;
                    T::Currency::unreserve(&who, refund);
                }
            }

            // 更新订单状态
            order.status = OrderStatus::Cancelled;
            Orders::<T>::insert(order_id, &order);

            // 从订单簿移除
            Self::remove_from_order_book(order.shop_id, order_id, order.side);

            // 更新最优价格
            Self::update_best_prices(order.shop_id);

            Self::deposit_event(Event::OrderCancelled { order_id });

            Ok(())
        }

        /// 配置店铺市场
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn configure_market(
            origin: OriginFor<T>,
            shop_id: u64,
            cos_enabled: bool,
            usdt_enabled: bool,
            fee_rate: u16,
            min_order_amount: u128,
            order_ttl: u32,
            usdt_timeout: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证店主
            ensure!(T::ShopProvider::shop_exists(shop_id), Error::<T>::ShopNotFound);
            let owner = T::ShopProvider::shop_owner(shop_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(owner == who, Error::<T>::NotShopOwner);

            let config = MarketConfig {
                cos_enabled,
                usdt_enabled,
                fee_rate,
                min_order_amount,
                order_ttl,
                usdt_timeout,
                fee_recipient: None,
            };

            MarketConfigs::<T>::insert(shop_id, config);

            Self::deposit_event(Event::MarketConfigured { shop_id });

            Ok(())
        }

        /// 配置价格保护（店主调用）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `enabled`: 是否启用价格保护
        /// - `max_price_deviation`: 最大价格偏离（基点，2000 = 20%）
        /// - `max_slippage`: 最大滑点（基点，500 = 5%）
        /// - `circuit_breaker_threshold`: 熔断阈值（基点，5000 = 50%）
        /// - `min_trades_for_twap`: 启用 TWAP 的最小成交数
        #[pallet::call_index(15)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn configure_price_protection(
            origin: OriginFor<T>,
            shop_id: u64,
            enabled: bool,
            max_price_deviation: u16,
            max_slippage: u16,
            circuit_breaker_threshold: u16,
            min_trades_for_twap: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证店主
            let owner = T::ShopProvider::shop_owner(shop_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(owner == who, Error::<T>::NotShopOwner);

            // 获取现有配置或创建新配置
            let mut config = PriceProtection::<T>::get(shop_id).unwrap_or_default();

            config.enabled = enabled;
            config.max_price_deviation = max_price_deviation;
            config.max_slippage = max_slippage;
            config.circuit_breaker_threshold = circuit_breaker_threshold;
            config.min_trades_for_twap = min_trades_for_twap;

            PriceProtection::<T>::insert(shop_id, config);

            Self::deposit_event(Event::PriceProtectionConfigured {
                shop_id,
                enabled,
                max_deviation: max_price_deviation,
                max_slippage,
            });

            Ok(())
        }

        /// 手动解除熔断（店主调用，仅在熔断时间到期后）
        #[pallet::call_index(16)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn lift_circuit_breaker(
            origin: OriginFor<T>,
            shop_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证店主
            let owner = T::ShopProvider::shop_owner(shop_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(owner == who, Error::<T>::NotShopOwner);

            let current_block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();

            PriceProtection::<T>::mutate(shop_id, |maybe_config| {
                if let Some(config) = maybe_config {
                    // 只有在熔断时间到期后才能解除
                    if config.circuit_breaker_active && current_block >= config.circuit_breaker_until {
                        config.circuit_breaker_active = false;
                        config.circuit_breaker_until = 0;
                    }
                }
            });

            Self::deposit_event(Event::CircuitBreakerLifted { shop_id });

            Ok(())
        }

        /// 设置店铺代币初始价格（店主调用，用于 TWAP 冷启动）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `initial_price`: 初始参考价格（每个 Token 的 COS 价格）
        ///
        /// # 说明
        /// 初始价格用于 TWAP 冷启动期间的价格偏离检查。
        /// 当市场成交量不足时，将使用此价格作为参考。
        /// 一旦成交量达到 `min_trades_for_twap`，将自动切换到 TWAP 价格。
        #[pallet::call_index(17)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn set_initial_price(
            origin: OriginFor<T>,
            shop_id: u64,
            initial_price: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证店主
            let owner = T::ShopProvider::shop_owner(shop_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(owner == who, Error::<T>::NotShopOwner);

            // 验证价格
            ensure!(!initial_price.is_zero(), Error::<T>::ZeroPrice);

            // 更新价格保护配置中的初始价格
            PriceProtection::<T>::mutate(shop_id, |maybe_config| {
                let config = maybe_config.get_or_insert_with(Default::default);
                config.initial_price = Some(initial_price);
            });

            // 初始化 TWAP 累积器（如果不存在）
            let current_block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();
            TwapAccumulators::<T>::mutate(shop_id, |maybe_acc| {
                if maybe_acc.is_none() {
                    *maybe_acc = Some(TwapAccumulator {
                        current_cumulative: 0,
                        current_block,
                        last_price: initial_price,
                        trade_count: 0,
                        hour_snapshot: PriceSnapshot { cumulative_price: 0, block_number: current_block },
                        day_snapshot: PriceSnapshot { cumulative_price: 0, block_number: current_block },
                        week_snapshot: PriceSnapshot { cumulative_price: 0, block_number: current_block },
                        last_hour_update: current_block,
                        last_day_update: current_block,
                        last_week_update: current_block,
                    });
                }
            });

            // 设置最新成交价为初始价格
            LastTradePrice::<T>::insert(shop_id, initial_price);

            Self::deposit_event(Event::InitialPriceSet { shop_id, initial_price });

            Ok(())
        }

        // ==================== USDT 通道 Extrinsics ====================

        /// 挂 USDT 卖单（卖 Token 收 USDT）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `token_amount`: 出售的 Token 数量
        /// - `usdt_price`: 每个 Token 的 USDT 价格（精度 10^6）
        /// - `tron_address`: 卖家的 TRON 收款地址
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(80_000, 0))]
        pub fn place_usdt_sell_order(
            origin: OriginFor<T>,
            shop_id: u64,
            token_amount: T::TokenBalance,
            usdt_price: u64,
            tron_address: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证 USDT 市场
            Self::ensure_usdt_market_enabled(shop_id)?;

            // 验证参数
            ensure!(usdt_price > 0, Error::<T>::ZeroPrice);
            ensure!(!token_amount.is_zero(), Error::<T>::AmountTooSmall);

            // 验证 TRON 地址格式（Base58，以 T 开头，34 字符）
            ensure!(tron_address.len() == 34, Error::<T>::InvalidTronAddress);
            ensure!(tron_address.first() == Some(&b'T'), Error::<T>::InvalidTronAddress);
            let tron_addr: TronAddress = tron_address.try_into().map_err(|_| Error::<T>::InvalidTronAddress)?;

            // 检查用户 Token 余额
            let balance = T::TokenProvider::token_balance(shop_id, &who);
            ensure!(balance >= token_amount, Error::<T>::InsufficientTokenBalance);

            // 锁定 Token
            T::TokenProvider::reserve(shop_id, &who, token_amount)?;

            // 创建订单
            let order_id = Self::do_create_order(
                shop_id,
                who.clone(),
                OrderSide::Sell,
                OrderType::Limit,
                PaymentChannel::USDT,
                token_amount,
                Zero::zero(), // COS price (USDT 通道不使用)
                usdt_price,
                Some(tron_addr.clone()),
            )?;

            Self::deposit_event(Event::UsdtSellOrderCreated {
                order_id,
                shop_id,
                maker: who,
                token_amount,
                usdt_price,
                tron_address: tron_addr,
            });

            Ok(())
        }

        /// 挂 USDT 买单（用 USDT 买 Token）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `token_amount`: 想购买的 Token 数量
        /// - `usdt_price`: 每个 Token 愿意支付的 USDT 价格（精度 10^6）
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(80_000, 0))]
        pub fn place_usdt_buy_order(
            origin: OriginFor<T>,
            shop_id: u64,
            token_amount: T::TokenBalance,
            usdt_price: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证 USDT 市场
            Self::ensure_usdt_market_enabled(shop_id)?;

            // 验证参数
            ensure!(usdt_price > 0, Error::<T>::ZeroPrice);
            ensure!(!token_amount.is_zero(), Error::<T>::AmountTooSmall);

            // USDT 买单不需要锁定链上资产（USDT 在链下）
            // 创建订单
            let order_id = Self::do_create_order(
                shop_id,
                who.clone(),
                OrderSide::Buy,
                OrderType::Limit,
                PaymentChannel::USDT,
                token_amount,
                Zero::zero(), // COS price (USDT 通道不使用)
                usdt_price,
                None, // 买单不需要 TRON 地址
            )?;

            Self::deposit_event(Event::UsdtBuyOrderCreated {
                order_id,
                shop_id,
                maker: who,
                token_amount,
                usdt_price,
            });

            Ok(())
        }

        /// 吃 USDT 卖单（买家发起，提交已支付的 TRON 交易哈希）
        ///
        /// # 流程
        /// 1. 买家链下转 USDT 到卖家 TRON 地址
        /// 2. 买家调用此函数提交交易哈希
        /// 3. 创建 USDT 交易记录，等待 OCW 验证
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(100_000, 0))]
        pub fn take_usdt_sell_order(
            origin: OriginFor<T>,
            order_id: u64,
            amount: Option<T::TokenBalance>,
            tron_tx_hash: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取订单
            let mut order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;

            // 验证订单
            ensure!(order.channel == PaymentChannel::USDT, Error::<T>::ChannelMismatch);
            ensure!(order.side == OrderSide::Sell, Error::<T>::OrderSideMismatch);
            ensure!(
                order.status == OrderStatus::Open || order.status == OrderStatus::PartiallyFilled,
                Error::<T>::OrderClosed
            );
            ensure!(order.maker != who, Error::<T>::CannotTakeOwnOrder);

            // 验证交易哈希格式（64 字符 hex）
            ensure!(tron_tx_hash.len() == 64, Error::<T>::InvalidTxHash);
            let tx_hash: TronTxHash = tron_tx_hash.try_into().map_err(|_| Error::<T>::InvalidTxHash)?;

            // 计算成交数量
            let available = order.token_amount.checked_sub(&order.filled_amount)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
            let fill_amount = amount.unwrap_or(available).min(available);
            ensure!(!fill_amount.is_zero(), Error::<T>::AmountTooSmall);

            // 计算 USDT 金额
            let fill_u128: u128 = fill_amount.into();
            let usdt_amount = fill_u128
                .checked_mul(order.usdt_price as u128)
                .ok_or(Error::<T>::ArithmeticOverflow)? as u64;

            // 获取卖家 TRON 地址
            let seller_tron_address = order.tron_address.clone()
                .ok_or(Error::<T>::InvalidTronAddress)?;

            // 创建 USDT 交易记录
            let trade_id = Self::do_create_usdt_trade(
                order_id,
                order.shop_id,
                order.maker.clone(),
                who.clone(),
                fill_amount,
                usdt_amount,
                seller_tron_address,
            )?;

            // 更新交易记录，添加交易哈希，状态改为等待验证
            UsdtTrades::<T>::mutate(trade_id, |maybe_trade| {
                if let Some(trade) = maybe_trade {
                    trade.tron_tx_hash = Some(tx_hash.clone());
                    trade.status = UsdtTradeStatus::AwaitingVerification;
                }
            });

            // 添加到待验证队列
            PendingUsdtTrades::<T>::try_mutate(|pending| {
                pending.try_push(trade_id).map_err(|_| Error::<T>::PendingQueueFull)
            })?;

            // 更新订单已成交数量（Token 仍锁定，等待验证通过后释放）
            order.filled_amount = order.filled_amount.checked_add(&fill_amount)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
            if order.filled_amount >= order.token_amount {
                order.status = OrderStatus::Filled;
            } else {
                order.status = OrderStatus::PartiallyFilled;
            }
            Orders::<T>::insert(order_id, &order);

            Self::deposit_event(Event::UsdtTradeCreated {
                trade_id,
                order_id,
                seller: order.maker,
                buyer: who,
                token_amount: fill_amount,
                usdt_amount,
            });

            Self::deposit_event(Event::UsdtPaymentSubmitted {
                trade_id,
                tron_tx_hash: tx_hash,
            });

            Ok(())
        }

        /// 接受 USDT 买单（卖家发起）
        ///
        /// # 流程
        /// 1. 卖家看到买单，调用此函数接受
        /// 2. 锁定卖家的 Token
        /// 3. 创建 USDT 交易记录，等待买家支付
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(100_000, 0))]
        pub fn accept_usdt_buy_order(
            origin: OriginFor<T>,
            order_id: u64,
            amount: Option<T::TokenBalance>,
            tron_address: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取订单
            let mut order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;

            // 验证订单
            ensure!(order.channel == PaymentChannel::USDT, Error::<T>::ChannelMismatch);
            ensure!(order.side == OrderSide::Buy, Error::<T>::OrderSideMismatch);
            ensure!(
                order.status == OrderStatus::Open || order.status == OrderStatus::PartiallyFilled,
                Error::<T>::OrderClosed
            );
            ensure!(order.maker != who, Error::<T>::CannotTakeOwnOrder);

            // 验证 TRON 地址
            ensure!(tron_address.len() == 34, Error::<T>::InvalidTronAddress);
            ensure!(tron_address.first() == Some(&b'T'), Error::<T>::InvalidTronAddress);
            let tron_addr: TronAddress = tron_address.try_into().map_err(|_| Error::<T>::InvalidTronAddress)?;

            // 计算成交数量
            let available = order.token_amount.checked_sub(&order.filled_amount)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
            let fill_amount = amount.unwrap_or(available).min(available);
            ensure!(!fill_amount.is_zero(), Error::<T>::AmountTooSmall);

            // 检查卖家 Token 余额并锁定
            let seller_balance = T::TokenProvider::token_balance(order.shop_id, &who);
            ensure!(seller_balance >= fill_amount, Error::<T>::InsufficientTokenBalance);
            T::TokenProvider::reserve(order.shop_id, &who, fill_amount)?;

            // 计算 USDT 金额
            let fill_u128: u128 = fill_amount.into();
            let usdt_amount = fill_u128
                .checked_mul(order.usdt_price as u128)
                .ok_or(Error::<T>::ArithmeticOverflow)? as u64;

            // 创建 USDT 交易记录（等待买家支付）
            let trade_id = Self::do_create_usdt_trade(
                order_id,
                order.shop_id,
                who.clone(),        // 卖家
                order.maker.clone(), // 买家
                fill_amount,
                usdt_amount,
                tron_addr,
            )?;

            // 更新订单已成交数量
            order.filled_amount = order.filled_amount.checked_add(&fill_amount)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
            if order.filled_amount >= order.token_amount {
                order.status = OrderStatus::Filled;
            } else {
                order.status = OrderStatus::PartiallyFilled;
            }
            Orders::<T>::insert(order_id, &order);

            Self::deposit_event(Event::UsdtTradeCreated {
                trade_id,
                order_id,
                seller: who,
                buyer: order.maker,
                token_amount: fill_amount,
                usdt_amount,
            });

            Ok(())
        }

        /// 买家确认 USDT 支付（提交交易哈希）
        ///
        /// # 说明
        /// 用于 accept_usdt_buy_order 流程，买家支付后提交交易哈希
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(50_000, 0))]
        pub fn confirm_usdt_payment(
            origin: OriginFor<T>,
            trade_id: u64,
            tron_tx_hash: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut trade = UsdtTrades::<T>::get(trade_id).ok_or(Error::<T>::UsdtTradeNotFound)?;

            // 验证是买家
            ensure!(trade.buyer == who, Error::<T>::NotTradeParticipant);

            // 验证状态
            ensure!(trade.status == UsdtTradeStatus::AwaitingPayment, Error::<T>::InvalidTradeStatus);

            // 检查是否超时
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(now <= trade.timeout_at, Error::<T>::TradeTimeout);

            // 验证交易哈希格式
            ensure!(tron_tx_hash.len() == 64, Error::<T>::InvalidTxHash);
            let tx_hash: TronTxHash = tron_tx_hash.try_into().map_err(|_| Error::<T>::InvalidTxHash)?;

            // 更新交易记录
            trade.tron_tx_hash = Some(tx_hash.clone());
            trade.status = UsdtTradeStatus::AwaitingVerification;
            UsdtTrades::<T>::insert(trade_id, &trade);

            // 添加到待验证队列
            PendingUsdtTrades::<T>::try_mutate(|pending| {
                pending.try_push(trade_id).map_err(|_| Error::<T>::PendingQueueFull)
            })?;

            Self::deposit_event(Event::UsdtPaymentSubmitted {
                trade_id,
                tron_tx_hash: tx_hash,
            });

            Ok(())
        }

        /// OCW 验证 USDT 支付结果（无签名交易）
        ///
        /// # 说明
        /// 由 OCW 调用，验证 TRON 交易是否有效
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(80_000, 0))]
        pub fn verify_usdt_payment(
            origin: OriginFor<T>,
            trade_id: u64,
            verified: bool,
            _actual_amount: u64,
        ) -> DispatchResult {
            // TODO: 实际应该使用 ensure_none 或 ValidateUnsigned
            let _who = ensure_signed(origin)?;

            let mut trade = UsdtTrades::<T>::get(trade_id).ok_or(Error::<T>::UsdtTradeNotFound)?;

            // 验证状态
            ensure!(trade.status == UsdtTradeStatus::AwaitingVerification, Error::<T>::InvalidTradeStatus);

            if verified {
                // 验证通过，完成交易
                // 将锁定的 Token 转给买家
                T::TokenProvider::repatriate_reserved(
                    trade.shop_id,
                    &trade.seller,
                    &trade.buyer,
                    trade.token_amount,
                )?;

                trade.status = UsdtTradeStatus::Completed;
                UsdtTrades::<T>::insert(trade_id, &trade);

                // 更新统计
                MarketStatsStorage::<T>::mutate(trade.shop_id, |stats| {
                    stats.total_trades = stats.total_trades.saturating_add(1);
                    stats.total_volume_usdt = stats.total_volume_usdt.saturating_add(trade.usdt_amount);
                });

                Self::deposit_event(Event::UsdtTradeCompleted {
                    trade_id,
                    order_id: trade.order_id,
                });
            } else {
                // 验证失败，退还 Token 给卖家
                T::TokenProvider::unreserve(trade.shop_id, &trade.seller, trade.token_amount);

                trade.status = UsdtTradeStatus::Cancelled;
                UsdtTrades::<T>::insert(trade_id, &trade);

                Self::deposit_event(Event::UsdtTradeVerificationFailed {
                    trade_id,
                    reason: b"OCW verification failed".to_vec(),
                });
            }

            // 从待验证队列移除
            PendingUsdtTrades::<T>::mutate(|pending| {
                pending.retain(|&id| id != trade_id);
            });

            Ok(())
        }

        /// 处理超时的 USDT 交易（任何人可调用）
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(50_000, 0))]
        pub fn process_usdt_timeout(
            origin: OriginFor<T>,
            trade_id: u64,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let mut trade = UsdtTrades::<T>::get(trade_id).ok_or(Error::<T>::UsdtTradeNotFound)?;

            // 只能处理等待支付或等待验证状态的交易
            ensure!(
                trade.status == UsdtTradeStatus::AwaitingPayment ||
                trade.status == UsdtTradeStatus::AwaitingVerification,
                Error::<T>::InvalidTradeStatus
            );

            // 检查是否已超时
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(now > trade.timeout_at, Error::<T>::InvalidTradeStatus);

            // 退还锁定的 Token 给卖家
            T::TokenProvider::unreserve(trade.shop_id, &trade.seller, trade.token_amount);

            trade.status = UsdtTradeStatus::Refunded;
            UsdtTrades::<T>::insert(trade_id, &trade);

            // 从待验证队列移除（如果存在）
            PendingUsdtTrades::<T>::mutate(|pending| {
                pending.retain(|&id| id != trade_id);
            });

            Self::deposit_event(Event::UsdtTradeRefunded { trade_id });

            Ok(())
        }

        // ==================== Phase 3: 市价单 ====================

        /// 市价买单（立即以最优卖价成交）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `token_amount`: 想购买的 Token 数量
        /// - `max_cost`: 最大愿意支付的 COS 总额（滑点保护）
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(150_000, 0))]
        pub fn market_buy(
            origin: OriginFor<T>,
            shop_id: u64,
            token_amount: T::TokenBalance,
            max_cost: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证市场
            Self::ensure_market_enabled(shop_id)?;

            // 验证参数
            ensure!(!token_amount.is_zero(), Error::<T>::AmountTooSmall);
            ensure!(!max_cost.is_zero(), Error::<T>::ZeroPrice);

            // 获取卖单列表（按价格升序排列）
            let mut sell_orders = Self::get_sorted_sell_orders(shop_id);
            ensure!(!sell_orders.is_empty(), Error::<T>::NoOrdersAvailable);

            // 执行市价买入
            let (filled, total_cost, fees) = Self::do_market_buy(
                &who,
                shop_id,
                token_amount,
                max_cost,
                &mut sell_orders,
            )?;

            ensure!(!filled.is_zero(), Error::<T>::AmountTooSmall);

            Self::deposit_event(Event::MarketOrderExecuted {
                shop_id,
                trader: who,
                side: OrderSide::Buy,
                filled_amount: filled,
                total_cost,
                total_fee: fees,
            });

            Ok(())
        }

        /// 市价卖单（立即以最优买价成交）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `token_amount`: 想出售的 Token 数量
        /// - `min_receive`: 最低愿意收到的 COS 总额（滑点保护）
        #[pallet::call_index(13)]
        #[pallet::weight(Weight::from_parts(150_000, 0))]
        pub fn market_sell(
            origin: OriginFor<T>,
            shop_id: u64,
            token_amount: T::TokenBalance,
            min_receive: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证市场
            Self::ensure_market_enabled(shop_id)?;

            // 验证参数
            ensure!(!token_amount.is_zero(), Error::<T>::AmountTooSmall);

            // 检查用户 Token 余额
            let balance = T::TokenProvider::token_balance(shop_id, &who);
            ensure!(balance >= token_amount, Error::<T>::InsufficientTokenBalance);

            // 获取买单列表（按价格降序排列）
            let mut buy_orders = Self::get_sorted_buy_orders(shop_id);
            ensure!(!buy_orders.is_empty(), Error::<T>::NoOrdersAvailable);

            // 执行市价卖出
            let (filled, total_receive, fees) = Self::do_market_sell(
                &who,
                shop_id,
                token_amount,
                min_receive,
                &mut buy_orders,
            )?;

            ensure!(!filled.is_zero(), Error::<T>::AmountTooSmall);

            Self::deposit_event(Event::MarketOrderExecuted {
                shop_id,
                trader: who,
                side: OrderSide::Sell,
                filled_amount: filled,
                total_cost: total_receive,
                total_fee: fees,
            });

            Ok(())
        }
    }

    // ==================== 内部函数 ====================

    impl<T: Config> Pallet<T> {
        /// 验证市场是否启用
        fn ensure_market_enabled(shop_id: u64) -> DispatchResult {
            ensure!(T::ShopProvider::shop_exists(shop_id), Error::<T>::ShopNotFound);
            ensure!(
                T::TokenProvider::is_token_enabled(shop_id),
                Error::<T>::TokenNotEnabled
            );

            // 检查市场配置（如果没有配置，默认启用）
            if let Some(config) = MarketConfigs::<T>::get(shop_id) {
                ensure!(config.cos_enabled, Error::<T>::MarketNotEnabled);
            }

            Ok(())
        }

        /// 计算总成本
        fn calculate_total_cost(token_amount: u128, price: BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError> {
            let price_u128: u128 = price.into();
            let total = token_amount
                .checked_mul(price_u128)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
            Ok(total.into())
        }

        /// 创建订单（通用）
        fn do_create_order(
            shop_id: u64,
            maker: T::AccountId,
            side: OrderSide,
            order_type: OrderType,
            channel: PaymentChannel,
            token_amount: T::TokenBalance,
            price: BalanceOf<T>,
            usdt_price: u64,
            tron_address: Option<TronAddress>,
        ) -> Result<u64, DispatchError> {
            let order_id = NextOrderId::<T>::get();
            NextOrderId::<T>::put(order_id.saturating_add(1));

            let now = <frame_system::Pallet<T>>::block_number();
            let config = MarketConfigs::<T>::get(shop_id).unwrap_or_default();
            let ttl = if config.order_ttl > 0 {
                config.order_ttl
            } else {
                T::DefaultOrderTTL::get()
            };
            let expires_at = now.saturating_add(ttl.into());

            let order = TradeOrder {
                order_id,
                shop_id,
                maker: maker.clone(),
                side,
                order_type,
                channel,
                token_amount,
                filled_amount: Zero::zero(),
                price,
                usdt_price,
                tron_address,
                status: OrderStatus::Open,
                created_at: now,
                expires_at,
            };

            Orders::<T>::insert(order_id, order);

            // 添加到订单簿
            match side {
                OrderSide::Sell => {
                    ShopSellOrders::<T>::try_mutate(shop_id, |orders| {
                        orders.try_push(order_id).map_err(|_| Error::<T>::OrderBookFull)
                    })?;
                }
                OrderSide::Buy => {
                    ShopBuyOrders::<T>::try_mutate(shop_id, |orders| {
                        orders.try_push(order_id).map_err(|_| Error::<T>::OrderBookFull)
                    })?;
                }
            }

            // 添加到用户订单
            UserOrders::<T>::try_mutate(&maker, |orders| {
                orders.try_push(order_id).map_err(|_| Error::<T>::UserOrdersFull)
            })?;

            // 更新统计
            MarketStatsStorage::<T>::mutate(shop_id, |stats| {
                stats.total_orders = stats.total_orders.saturating_add(1);
            });

            Ok(order_id)
        }

        /// 验证 USDT 市场是否启用
        fn ensure_usdt_market_enabled(shop_id: u64) -> DispatchResult {
            ensure!(T::ShopProvider::shop_exists(shop_id), Error::<T>::ShopNotFound);
            ensure!(
                T::TokenProvider::is_token_enabled(shop_id),
                Error::<T>::TokenNotEnabled
            );

            // 检查 USDT 市场配置
            if let Some(config) = MarketConfigs::<T>::get(shop_id) {
                ensure!(config.usdt_enabled, Error::<T>::UsdtMarketNotEnabled);
            } else {
                // 默认不启用 USDT 市场
                return Err(Error::<T>::UsdtMarketNotEnabled.into());
            }

            Ok(())
        }

        /// 获取 USDT 交易超时区块数
        fn get_usdt_timeout(shop_id: u64) -> u32 {
            MarketConfigs::<T>::get(shop_id)
                .map(|c| if c.usdt_timeout > 0 { c.usdt_timeout } else { T::DefaultUsdtTimeout::get() })
                .unwrap_or_else(|| T::DefaultUsdtTimeout::get())
        }

        /// 创建 USDT 交易记录
        fn do_create_usdt_trade(
            order_id: u64,
            shop_id: u64,
            seller: T::AccountId,
            buyer: T::AccountId,
            token_amount: T::TokenBalance,
            usdt_amount: u64,
            seller_tron_address: TronAddress,
        ) -> Result<u64, DispatchError> {
            let trade_id = NextUsdtTradeId::<T>::get();
            NextUsdtTradeId::<T>::put(trade_id.saturating_add(1));

            let now = <frame_system::Pallet<T>>::block_number();
            let timeout = Self::get_usdt_timeout(shop_id);
            let timeout_at = now.saturating_add(timeout.into());

            let trade = UsdtTrade {
                trade_id,
                order_id,
                shop_id,
                seller,
                buyer,
                token_amount,
                usdt_amount,
                seller_tron_address,
                tron_tx_hash: None,
                status: UsdtTradeStatus::AwaitingPayment,
                created_at: now,
                timeout_at,
            };

            UsdtTrades::<T>::insert(trade_id, trade);

            Ok(trade_id)
        }

        /// 从订单簿移除订单
        fn remove_from_order_book(shop_id: u64, order_id: u64, side: OrderSide) {
            match side {
                OrderSide::Sell => {
                    ShopSellOrders::<T>::mutate(shop_id, |orders| {
                        orders.retain(|&id| id != order_id);
                    });
                }
                OrderSide::Buy => {
                    ShopBuyOrders::<T>::mutate(shop_id, |orders| {
                        orders.retain(|&id| id != order_id);
                    });
                }
            }
        }

        /// 获取排序后的卖单列表（按价格升序）
        pub fn get_sorted_sell_orders(shop_id: u64) -> Vec<TradeOrder<T>> {
            let mut orders: Vec<TradeOrder<T>> = ShopSellOrders::<T>::get(shop_id)
                .iter()
                .filter_map(|&id| Orders::<T>::get(id))
                .filter(|o| {
                    o.channel == PaymentChannel::COS &&
                    (o.status == OrderStatus::Open || o.status == OrderStatus::PartiallyFilled)
                })
                .collect();
            orders.sort_by(|a, b| a.price.cmp(&b.price));
            orders
        }

        /// 获取排序后的买单列表（按价格降序）
        pub fn get_sorted_buy_orders(shop_id: u64) -> Vec<TradeOrder<T>> {
            let mut orders: Vec<TradeOrder<T>> = ShopBuyOrders::<T>::get(shop_id)
                .iter()
                .filter_map(|&id| Orders::<T>::get(id))
                .filter(|o| {
                    o.channel == PaymentChannel::COS &&
                    (o.status == OrderStatus::Open || o.status == OrderStatus::PartiallyFilled)
                })
                .collect();
            orders.sort_by(|a, b| b.price.cmp(&a.price));
            orders
        }

        /// 执行市价买入
        fn do_market_buy(
            buyer: &T::AccountId,
            shop_id: u64,
            mut remaining: T::TokenBalance,
            max_cost: BalanceOf<T>,
            sell_orders: &mut Vec<TradeOrder<T>>,
        ) -> Result<(T::TokenBalance, BalanceOf<T>, BalanceOf<T>), DispatchError> {
            let mut total_filled: T::TokenBalance = Zero::zero();
            let mut total_cost: BalanceOf<T> = Zero::zero();
            let mut total_fees: BalanceOf<T> = Zero::zero();

            let config = MarketConfigs::<T>::get(shop_id).unwrap_or_default();
            let fee_rate = if config.fee_rate > 0 { config.fee_rate } else { T::DefaultFeeRate::get() };

            for order in sell_orders.iter_mut() {
                if remaining.is_zero() {
                    break;
                }

                // 计算可成交数量
                let available = order.token_amount.saturating_sub(order.filled_amount);
                let fill_amount = remaining.min(available);

                // 计算成本
                let fill_u128: u128 = fill_amount.into();
                let cost = Self::calculate_total_cost(fill_u128, order.price)?;

                // 检查滑点
                if total_cost.saturating_add(cost) > max_cost {
                    // 计算在预算内能买多少
                    let budget_left = max_cost.saturating_sub(total_cost);
                    if budget_left.is_zero() {
                        break;
                    }
                    // 简化：跳过这个订单
                    continue;
                }

                // 计算手续费
                let fee = cost
                    .saturating_mul(fee_rate.into())
                    .checked_div(&10000u32.into())
                    .unwrap_or_else(Zero::zero);

                // 执行转账
                // buyer 支付 COS → maker
                T::Currency::transfer(
                    buyer,
                    &order.maker,
                    cost.saturating_sub(fee),
                    ExistenceRequirement::KeepAlive,
                )?;

                // 手续费转给店铺
                if !fee.is_zero() {
                    if let Some(shop_owner) = T::ShopProvider::shop_owner(shop_id) {
                        T::Currency::transfer(
                            buyer,
                            &shop_owner,
                            fee,
                            ExistenceRequirement::KeepAlive,
                        )?;
                    }
                }

                // Token: maker → buyer (简化实现)
                // 实际应调用 TokenProvider::transfer

                // 更新订单
                let mut updated_order = order.clone();
                updated_order.filled_amount = updated_order.filled_amount.saturating_add(fill_amount);
                if updated_order.filled_amount >= updated_order.token_amount {
                    updated_order.status = OrderStatus::Filled;
                    Self::remove_from_order_book(shop_id, order.order_id, OrderSide::Sell);
                } else {
                    updated_order.status = OrderStatus::PartiallyFilled;
                }
                Orders::<T>::insert(order.order_id, &updated_order);

                // 累计
                total_filled = total_filled.saturating_add(fill_amount);
                total_cost = total_cost.saturating_add(cost);
                total_fees = total_fees.saturating_add(fee);
                remaining = remaining.saturating_sub(fill_amount);
            }

            // 更新统计和最优价格
            if !total_filled.is_zero() {
                MarketStatsStorage::<T>::mutate(shop_id, |stats| {
                    stats.total_trades = stats.total_trades.saturating_add(1);
                    stats.total_volume_cos = stats.total_volume_cos.saturating_add(total_cost.into());
                    stats.total_fees_cos = stats.total_fees_cos.saturating_add(total_fees.into());
                });

                // 更新最优价格和 TWAP（使用加权平均价格）
                Self::update_best_prices(shop_id);
                if !total_filled.is_zero() {
                    let avg_price = total_cost.checked_div(&total_filled.into().into()).unwrap_or_else(Zero::zero);
                    Self::on_trade_completed(shop_id, avg_price);
                }
            }

            Ok((total_filled, total_cost, total_fees))
        }

        /// 执行市价卖出
        fn do_market_sell(
            seller: &T::AccountId,
            shop_id: u64,
            mut remaining: T::TokenBalance,
            min_receive: BalanceOf<T>,
            buy_orders: &mut Vec<TradeOrder<T>>,
        ) -> Result<(T::TokenBalance, BalanceOf<T>, BalanceOf<T>), DispatchError> {
            let mut total_filled: T::TokenBalance = Zero::zero();
            let mut total_receive: BalanceOf<T> = Zero::zero();
            let mut total_fees: BalanceOf<T> = Zero::zero();

            let config = MarketConfigs::<T>::get(shop_id).unwrap_or_default();
            let fee_rate = if config.fee_rate > 0 { config.fee_rate } else { T::DefaultFeeRate::get() };

            for order in buy_orders.iter_mut() {
                if remaining.is_zero() {
                    break;
                }

                // 计算可成交数量
                let available = order.token_amount.saturating_sub(order.filled_amount);
                let fill_amount = remaining.min(available);

                // 计算收入
                let fill_u128: u128 = fill_amount.into();
                let gross = Self::calculate_total_cost(fill_u128, order.price)?;

                // 计算手续费
                let fee = gross
                    .saturating_mul(fee_rate.into())
                    .checked_div(&10000u32.into())
                    .unwrap_or_else(Zero::zero);
                let net = gross.saturating_sub(fee);

                // 从 maker 的锁定中释放 COS → seller
                T::Currency::unreserve(&order.maker, gross);
                T::Currency::transfer(
                    &order.maker,
                    seller,
                    net,
                    ExistenceRequirement::KeepAlive,
                )?;

                // 手续费
                if !fee.is_zero() {
                    if let Some(shop_owner) = T::ShopProvider::shop_owner(shop_id) {
                        T::Currency::transfer(
                            &order.maker,
                            &shop_owner,
                            fee,
                            ExistenceRequirement::KeepAlive,
                        )?;
                    }
                }

                // Token: seller → maker (简化实现)
                // 实际应调用 TokenProvider::transfer

                // 更新订单
                let mut updated_order = order.clone();
                updated_order.filled_amount = updated_order.filled_amount.saturating_add(fill_amount);
                if updated_order.filled_amount >= updated_order.token_amount {
                    updated_order.status = OrderStatus::Filled;
                    Self::remove_from_order_book(shop_id, order.order_id, OrderSide::Buy);
                } else {
                    updated_order.status = OrderStatus::PartiallyFilled;
                }
                Orders::<T>::insert(order.order_id, &updated_order);

                // 累计
                total_filled = total_filled.saturating_add(fill_amount);
                total_receive = total_receive.saturating_add(net);
                total_fees = total_fees.saturating_add(fee);
                remaining = remaining.saturating_sub(fill_amount);
            }

            // 滑点检查
            if total_receive < min_receive && !total_filled.is_zero() {
                return Err(Error::<T>::SlippageExceeded.into());
            }

            // 更新统计和最优价格
            if !total_filled.is_zero() {
                MarketStatsStorage::<T>::mutate(shop_id, |stats| {
                    stats.total_trades = stats.total_trades.saturating_add(1);
                    stats.total_volume_cos = stats.total_volume_cos.saturating_add(total_receive.saturating_add(total_fees).into());
                    stats.total_fees_cos = stats.total_fees_cos.saturating_add(total_fees.into());
                });

                // 更新最优价格和 TWAP
                Self::update_best_prices(shop_id);
                let total_gross = total_receive.saturating_add(total_fees);
                if !total_gross.is_zero() {
                    let avg_price = total_gross.checked_div(&total_filled.into().into()).unwrap_or_else(Zero::zero);
                    Self::on_trade_completed(shop_id, avg_price);
                }
            }

            Ok((total_filled, total_receive, total_fees))
        }

        /// 更新最优买卖价格
        fn update_best_prices(shop_id: u64) {
            // 更新最优卖价
            if let Some(best_ask) = Self::calculate_best_ask(shop_id) {
                BestAsk::<T>::insert(shop_id, best_ask);
            } else {
                BestAsk::<T>::remove(shop_id);
            }

            // 更新最优买价
            if let Some(best_bid) = Self::calculate_best_bid(shop_id) {
                BestBid::<T>::insert(shop_id, best_bid);
            } else {
                BestBid::<T>::remove(shop_id);
            }
        }

        /// 更新最新成交价
        fn update_last_trade_price(shop_id: u64, price: BalanceOf<T>) {
            LastTradePrice::<T>::insert(shop_id, price);
        }

        // ==================== Phase 5: TWAP 价格预言机内部函数 ====================

        /// 更新 TWAP 累积器（每次成交时调用）
        /// P1 安全修复: 添加异常价格过滤，防止价格操纵
        fn update_twap_accumulator(shop_id: u64, trade_price: BalanceOf<T>) {
            let current_block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();

            TwapAccumulators::<T>::mutate(shop_id, |maybe_acc| {
                let acc = maybe_acc.get_or_insert_with(|| TwapAccumulator {
                    current_cumulative: 0,
                    current_block,
                    last_price: trade_price,
                    trade_count: 0,
                    hour_snapshot: PriceSnapshot { cumulative_price: 0, block_number: current_block },
                    day_snapshot: PriceSnapshot { cumulative_price: 0, block_number: current_block },
                    week_snapshot: PriceSnapshot { cumulative_price: 0, block_number: current_block },
                    last_hour_update: current_block,
                    last_day_update: current_block,
                    last_week_update: current_block,
                });

                // P1: 异常价格过滤 - 如果价格偏离上次价格超过 100%，使用加权平均
                let filtered_price = if acc.trade_count > 0 && !acc.last_price.is_zero() {
                    let last_price_u128: u128 = acc.last_price.into();
                    let trade_price_u128: u128 = trade_price.into();
                    let max_deviation = last_price_u128; // 100% 偏离
                    
                    let deviation = if trade_price_u128 > last_price_u128 {
                        trade_price_u128.saturating_sub(last_price_u128)
                    } else {
                        last_price_u128.saturating_sub(trade_price_u128)
                    };
                    
                    if deviation > max_deviation {
                        // 异常价格: 限制价格变动幅度为上次价格的 50%
                        // 如果新价格过高，使用 last_price * 1.5
                        // 如果新价格过低，使用 last_price * 0.5
                        if trade_price_u128 > last_price_u128 {
                            // 价格上涨过快，限制为 +50%
                            acc.last_price.saturating_mul(3u32.into()) / 2u32.into()
                        } else {
                            // 价格下跌过快，限制为 -50%
                            acc.last_price / 2u32.into()
                        }
                    } else {
                        trade_price
                    }
                } else {
                    trade_price
                };

                // 计算自上次更新以来经过的区块数
                let blocks_elapsed = current_block.saturating_sub(acc.current_block);

                // 更新累积价格: cumulative += last_price × blocks_elapsed
                if blocks_elapsed > 0 {
                    let price_u128: u128 = acc.last_price.into();
                    acc.current_cumulative = acc.current_cumulative
                        .saturating_add(price_u128.saturating_mul(blocks_elapsed as u128));
                }

                // 更新当前状态（使用过滤后的价格）
                acc.current_block = current_block;
                acc.last_price = filtered_price;
                acc.trade_count = acc.trade_count.saturating_add(1);

                // 滚动更新快照
                let blocks_per_hour = T::BlocksPerHour::get();
                let blocks_per_day = T::BlocksPerDay::get();
                let blocks_per_week = T::BlocksPerWeek::get();

                // 更新 1小时快照（每 10 分钟更新一次，即 blocks_per_hour / 6）
                let hour_update_interval = blocks_per_hour / 6;
                if current_block.saturating_sub(acc.last_hour_update) >= hour_update_interval {
                    acc.hour_snapshot = PriceSnapshot {
                        cumulative_price: acc.current_cumulative,
                        block_number: current_block,
                    };
                    acc.last_hour_update = current_block;
                }

                // 更新 24小时快照（每小时更新一次）
                if current_block.saturating_sub(acc.last_day_update) >= blocks_per_hour {
                    acc.day_snapshot = PriceSnapshot {
                        cumulative_price: acc.current_cumulative,
                        block_number: current_block,
                    };
                    acc.last_day_update = current_block;
                }

                // 更新 7天快照（每天更新一次）
                if current_block.saturating_sub(acc.last_week_update) >= blocks_per_day {
                    acc.week_snapshot = PriceSnapshot {
                        cumulative_price: acc.current_cumulative,
                        block_number: current_block,
                    };
                    acc.last_week_update = current_block;
                }
            });
        }

        /// 计算指定周期的 TWAP
        pub fn calculate_twap(shop_id: u64, period: TwapPeriod) -> Option<BalanceOf<T>> {
            let acc = TwapAccumulators::<T>::get(shop_id)?;
            let current_block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();

            // 获取对应周期的快照
            let snapshot = match period {
                TwapPeriod::OneHour => &acc.hour_snapshot,
                TwapPeriod::OneDay => &acc.day_snapshot,
                TwapPeriod::OneWeek => &acc.week_snapshot,
            };

            // 计算当前累积价格（包含自上次更新以来的部分）
            let blocks_since_update = current_block.saturating_sub(acc.current_block);
            let price_u128: u128 = acc.last_price.into();
            let current_cumulative = acc.current_cumulative
                .saturating_add(price_u128.saturating_mul(blocks_since_update as u128));

            // 计算区块差
            let block_diff = current_block.saturating_sub(snapshot.block_number);
            if block_diff == 0 {
                return Some(acc.last_price);
            }

            // 计算累积价格差
            let cumulative_diff = current_cumulative.saturating_sub(snapshot.cumulative_price);

            // TWAP = 累积价格差 / 区块差
            let twap_u128 = cumulative_diff / (block_diff as u128);

            Some(twap_u128.into())
        }

        /// 检查价格是否偏离参考价格过大
        ///
        /// 参考价格优先级：
        /// 1. 如果三周期 TWAP 数据都充足，使用 1小时 TWAP
        /// 2. 如果 TWAP 数据不足但有初始价格，使用店主设定的初始价格
        /// 3. 如果都没有，跳过检查
        ///
        /// 三周期 TWAP 充足条件：
        /// - 成交量 >= min_trades_for_twap
        /// - 1小时快照已更新（距离当前 >= 1小时）
        /// - 24小时快照已更新（距离当前 >= 24小时）
        /// - 7天快照已更新（距离当前 >= 7天）
        pub fn check_price_deviation(
            shop_id: u64,
            price: BalanceOf<T>,
        ) -> Result<(), Error<T>> {
            // 获取价格保护配置
            let config = PriceProtection::<T>::get(shop_id).unwrap_or_default();

            // 如果未启用价格保护，直接通过
            if !config.enabled {
                return Ok(());
            }

            // 检查熔断状态
            let current_block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();
            if config.circuit_breaker_active && current_block < config.circuit_breaker_until {
                return Err(Error::<T>::MarketCircuitBreakerActive);
            }

            // 获取参考价格
            let reference_price: Option<BalanceOf<T>> = {
                // 获取 TWAP 累积器
                let acc = TwapAccumulators::<T>::get(shop_id);

                match acc {
                    Some(ref a) if Self::is_twap_data_sufficient(a, current_block, &config) => {
                        // 三周期 TWAP 数据充足，使用 1小时 TWAP
                        Self::calculate_twap(shop_id, TwapPeriod::OneHour)
                    }
                    _ => {
                        // TWAP 数据不足，使用店主设定的初始价格
                        config.initial_price
                    }
                }
            };

            // 如果没有参考价格，跳过检查
            let ref_price = match reference_price {
                Some(p) => p,
                None => return Ok(()),
            };

            // 计算偏离度 (基点)
            let price_u128: u128 = price.into();
            let ref_price_u128: u128 = ref_price.into();

            if ref_price_u128 == 0 {
                return Ok(());
            }

            let deviation_bps = if price_u128 > ref_price_u128 {
                ((price_u128 - ref_price_u128) * 10000 / ref_price_u128) as u16
            } else {
                ((ref_price_u128 - price_u128) * 10000 / ref_price_u128) as u16
            };

            // 检查是否超过最大偏离
            if deviation_bps > config.max_price_deviation {
                return Err(Error::<T>::PriceDeviationTooHigh);
            }

            Ok(())
        }

        /// 检查三周期 TWAP 数据是否充足
        ///
        /// 条件：
        /// 1. 成交量 >= min_trades_for_twap
        /// 2. 1小时快照已有足够历史（当前区块 - 快照区块 >= BlocksPerHour）
        /// 3. 24小时快照已有足够历史（当前区块 - 快照区块 >= BlocksPerDay）
        /// 4. 7天快照已有足够历史（当前区块 - 快照区块 >= BlocksPerWeek）
        fn is_twap_data_sufficient(
            acc: &TwapAccumulator<BalanceOf<T>>,
            current_block: u32,
            config: &PriceProtectionConfig<BalanceOf<T>>,
        ) -> bool {
            // 检查成交量
            if acc.trade_count < config.min_trades_for_twap {
                return false;
            }

            let blocks_per_hour = T::BlocksPerHour::get();
            let blocks_per_day = T::BlocksPerDay::get();
            let blocks_per_week = T::BlocksPerWeek::get();

            // 检查 1小时快照是否有足够历史
            let hour_history = current_block.saturating_sub(acc.hour_snapshot.block_number);
            if hour_history < blocks_per_hour {
                return false;
            }

            // 检查 24小时快照是否有足够历史
            let day_history = current_block.saturating_sub(acc.day_snapshot.block_number);
            if day_history < blocks_per_day {
                return false;
            }

            // 检查 7天快照是否有足够历史
            let week_history = current_block.saturating_sub(acc.week_snapshot.block_number);
            if week_history < blocks_per_week {
                return false;
            }

            true
        }

        /// 检查并触发熔断机制
        fn check_circuit_breaker(shop_id: u64, current_price: BalanceOf<T>) {
            let config = match PriceProtection::<T>::get(shop_id) {
                Some(c) => c,
                None => return,
            };

            if !config.enabled {
                return;
            }

            // 使用 7天 TWAP 判断熔断
            let twap_7d = match Self::calculate_twap(shop_id, TwapPeriod::OneWeek) {
                Some(t) => t,
                None => return,
            };

            let price_u128: u128 = current_price.into();
            let twap_u128: u128 = twap_7d.into();

            if twap_u128 == 0 {
                return;
            }

            let deviation_bps = if price_u128 > twap_u128 {
                ((price_u128 - twap_u128) * 10000 / twap_u128) as u16
            } else {
                ((twap_u128 - price_u128) * 10000 / twap_u128) as u16
            };

            // 如果偏离超过熔断阈值，触发熔断
            if deviation_bps > config.circuit_breaker_threshold {
                let current_block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();
                let until_block = current_block.saturating_add(T::CircuitBreakerDuration::get());

                PriceProtection::<T>::mutate(shop_id, |maybe_config| {
                    if let Some(c) = maybe_config {
                        c.circuit_breaker_active = true;
                        c.circuit_breaker_until = until_block;
                    }
                });

                Self::deposit_event(Event::CircuitBreakerTriggered {
                    shop_id,
                    current_price,
                    twap_7d,
                    deviation_bps,
                    until_block,
                });
            }
        }

        /// 在成交后更新 TWAP 并检查熔断
        fn on_trade_completed(shop_id: u64, trade_price: BalanceOf<T>) {
            // 更新 TWAP 累积器
            Self::update_twap_accumulator(shop_id, trade_price);

            // 更新最新成交价
            Self::update_last_trade_price(shop_id, trade_price);

            // 检查熔断
            Self::check_circuit_breaker(shop_id, trade_price);
        }
    }
}

// ==================== 公共查询接口 ====================

impl<T: Config> Pallet<T> {
    /// 获取店铺卖单列表
    pub fn get_sell_orders(shop_id: u64) -> Vec<TradeOrder<T>> {
        ShopSellOrders::<T>::get(shop_id)
            .iter()
            .filter_map(|&id| Orders::<T>::get(id))
            .filter(|o| o.status == OrderStatus::Open || o.status == OrderStatus::PartiallyFilled)
            .collect()
    }

    /// 获取店铺买单列表
    pub fn get_buy_orders(shop_id: u64) -> Vec<TradeOrder<T>> {
        ShopBuyOrders::<T>::get(shop_id)
            .iter()
            .filter_map(|&id| Orders::<T>::get(id))
            .filter(|o| o.status == OrderStatus::Open || o.status == OrderStatus::PartiallyFilled)
            .collect()
    }

    /// 获取用户订单列表
    pub fn get_user_orders(user: &T::AccountId) -> Vec<TradeOrder<T>> {
        UserOrders::<T>::get(user)
            .iter()
            .filter_map(|&id| Orders::<T>::get(id))
            .collect()
    }

    // ==================== Phase 4: 订单簿深度查询接口 ====================

    /// 获取订单簿深度
    ///
    /// # 参数
    /// - `shop_id`: 店铺 ID
    /// - `depth`: 返回的档位数量（每边）
    pub fn get_order_book_depth(shop_id: u64, depth: u32) -> OrderBookDepth<BalanceOf<T>, T::TokenBalance> {
        use sp_runtime::traits::{Saturating, SaturatedConversion};

        let asks = Self::aggregate_price_levels(shop_id, OrderSide::Sell, depth);
        let bids = Self::aggregate_price_levels(shop_id, OrderSide::Buy, depth);

        let best_ask = asks.first().map(|l| l.price);
        let best_bid = bids.first().map(|l| l.price);

        let spread = match (best_ask, best_bid) {
            (Some(ask), Some(bid)) if ask > bid => Some(ask.saturating_sub(bid)),
            _ => None,
        };

        let block_number = <frame_system::Pallet<T>>::block_number();

        OrderBookDepth {
            shop_id,
            asks,
            bids,
            best_ask,
            best_bid,
            spread,
            block_number: block_number.saturated_into(),
        }
    }

    /// 聚合价格档位
    fn aggregate_price_levels(
        shop_id: u64,
        side: OrderSide,
        max_levels: u32,
    ) -> Vec<PriceLevel<BalanceOf<T>, T::TokenBalance>> {
        use alloc::collections::BTreeMap;
        use sp_runtime::traits::{Saturating, Zero};

        let orders = match side {
            OrderSide::Sell => Self::get_sorted_sell_orders(shop_id),
            OrderSide::Buy => Self::get_sorted_buy_orders(shop_id),
        };

        // 按价格聚合
        let mut price_map: BTreeMap<u128, (T::TokenBalance, u32)> = BTreeMap::new();

        for order in orders.iter() {
            let available = order.token_amount.saturating_sub(order.filled_amount);
            if available.is_zero() {
                continue;
            }

            let price_key: u128 = order.price.into();
            let entry = price_map.entry(price_key).or_insert((Zero::zero(), 0));
            entry.0 = entry.0.saturating_add(available);
            entry.1 = entry.1.saturating_add(1);
        }

        // 转换为 Vec 并限制数量
        let mut levels: Vec<PriceLevel<BalanceOf<T>, T::TokenBalance>> = price_map
            .into_iter()
            .map(|(price, (amount, count))| PriceLevel {
                price: price.into(),
                total_amount: amount,
                order_count: count,
            })
            .collect();

        // 卖单按价格升序，买单按价格降序（已在 get_sorted_* 中排序）
        if side == OrderSide::Buy {
            levels.reverse();
        }

        levels.truncate(max_levels as usize);
        levels
    }

    /// 获取最优买卖价
    pub fn get_best_prices(shop_id: u64) -> (Option<BalanceOf<T>>, Option<BalanceOf<T>>) {
        let best_ask = Self::calculate_best_ask(shop_id);
        let best_bid = Self::calculate_best_bid(shop_id);
        (best_ask, best_bid)
    }

    /// 计算最优卖价
    fn calculate_best_ask(shop_id: u64) -> Option<BalanceOf<T>> {
        ShopSellOrders::<T>::get(shop_id)
            .iter()
            .filter_map(|&id| Orders::<T>::get(id))
            .filter(|o| {
                o.channel == PaymentChannel::COS &&
                (o.status == OrderStatus::Open || o.status == OrderStatus::PartiallyFilled)
            })
            .map(|o| o.price)
            .min()
    }

    /// 计算最优买价
    fn calculate_best_bid(shop_id: u64) -> Option<BalanceOf<T>> {
        ShopBuyOrders::<T>::get(shop_id)
            .iter()
            .filter_map(|&id| Orders::<T>::get(id))
            .filter(|o| {
                o.channel == PaymentChannel::COS &&
                (o.status == OrderStatus::Open || o.status == OrderStatus::PartiallyFilled)
            })
            .map(|o| o.price)
            .max()
    }

    /// 获取买卖价差
    pub fn get_spread(shop_id: u64) -> Option<BalanceOf<T>> {
        use sp_runtime::traits::Saturating;

        let (best_ask, best_bid) = Self::get_best_prices(shop_id);
        match (best_ask, best_bid) {
            (Some(ask), Some(bid)) if ask > bid => Some(ask.saturating_sub(bid)),
            _ => None,
        }
    }

    /// 获取市场摘要
    pub fn get_market_summary(shop_id: u64) -> MarketSummary<BalanceOf<T>, T::TokenBalance> {
        use sp_runtime::traits::{Saturating, Zero};

        let (best_ask, best_bid) = Self::get_best_prices(shop_id);
        let last_price = LastTradePrice::<T>::get(shop_id);

        // 计算卖单总量
        let total_ask_amount: T::TokenBalance = ShopSellOrders::<T>::get(shop_id)
            .iter()
            .filter_map(|&id| Orders::<T>::get(id))
            .filter(|o| o.channel == PaymentChannel::COS &&
                (o.status == OrderStatus::Open || o.status == OrderStatus::PartiallyFilled))
            .fold(Zero::zero(), |acc: T::TokenBalance, o| {
                acc.saturating_add(o.token_amount.saturating_sub(o.filled_amount))
            });

        // 计算买单总量
        let total_bid_amount: T::TokenBalance = ShopBuyOrders::<T>::get(shop_id)
            .iter()
            .filter_map(|&id| Orders::<T>::get(id))
            .filter(|o| o.channel == PaymentChannel::COS &&
                (o.status == OrderStatus::Open || o.status == OrderStatus::PartiallyFilled))
            .fold(Zero::zero(), |acc: T::TokenBalance, o| {
                acc.saturating_add(o.token_amount.saturating_sub(o.filled_amount))
            });

        MarketSummary {
            best_ask,
            best_bid,
            high_24h: Zero::zero(), // TODO: 需要历史数据
            low_24h: Zero::zero(),  // TODO: 需要历史数据
            volume_24h: Zero::zero(), // TODO: 需要历史数据
            last_price,
            total_ask_amount,
            total_bid_amount,
        }
    }

    /// 获取订单簿快照（简化版）
    pub fn get_order_book_snapshot(shop_id: u64) -> (Vec<(BalanceOf<T>, T::TokenBalance)>, Vec<(BalanceOf<T>, T::TokenBalance)>) {
        let depth = Self::get_order_book_depth(shop_id, 20);

        let asks: Vec<(BalanceOf<T>, T::TokenBalance)> = depth.asks
            .into_iter()
            .map(|l| (l.price, l.total_amount))
            .collect();

        let bids: Vec<(BalanceOf<T>, T::TokenBalance)> = depth.bids
            .into_iter()
            .map(|l| (l.price, l.total_amount))
            .collect();

        (asks, bids)
    }
}
