//! # Entity 返佣管理模块 (pallet-entity-commission)
//!
//! ## 概述
//!
//! 本模块实现店铺返佣管理，支持多选返佣模式：
//! - 直推奖励 (DirectReward)
//! - 三级分销 (MultiLevel)
//! - 团队业绩 (TeamPerformance)
//! - 等级差价 (LevelDiff)
//! - 固定金额 (FixedAmount)
//! - 首单奖励 (FirstOrder)
//! - 复购奖励 (RepeatPurchase)
//!
//! 店铺可同时启用多种返佣模式，返佣按顺序叠加计算。
//!
//! ## 版本历史
//!
//! - v0.1.0: 初始版本，实现多选返佣模式

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, Get},
    };
    use frame_system::pallet_prelude::*;
    use pallet_entity_common::{MemberLevel, ShopProvider};
    use sp_runtime::traits::{Saturating, Zero};

    /// 货币余额类型别名
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // ============================================================================
    // 返佣模式位标志
    // ============================================================================

    /// 返佣模式位标志（可多选）
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct CommissionModes(pub u16);

    impl CommissionModes {
        /// 无返佣
        pub const NONE: u16 = 0b0000_0000;
        /// 直推奖励
        pub const DIRECT_REWARD: u16 = 0b0000_0001;
        /// 三级分销
        pub const MULTI_LEVEL: u16 = 0b0000_0010;
        /// 团队业绩
        pub const TEAM_PERFORMANCE: u16 = 0b0000_0100;
        /// 等级差价
        pub const LEVEL_DIFF: u16 = 0b0000_1000;
        /// 固定金额
        pub const FIXED_AMOUNT: u16 = 0b0001_0000;
        /// 首单奖励
        pub const FIRST_ORDER: u16 = 0b0010_0000;
        /// 复购奖励
        pub const REPEAT_PURCHASE: u16 = 0b0100_0000;
        /// 单线上线收益
        pub const SINGLE_LINE_UPLINE: u16 = 0b1000_0000;
        /// 单线下线收益
        pub const SINGLE_LINE_DOWNLINE: u16 = 0b1_0000_0000;

        pub fn contains(&self, flag: u16) -> bool {
            self.0 & flag != 0
        }

        pub fn insert(&mut self, flag: u16) {
            self.0 |= flag;
        }

        pub fn remove(&mut self, flag: u16) {
            self.0 &= !flag;
        }
    }

    // ============================================================================
    // 返佣来源
    // ============================================================================

    /// 返佣来源
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub enum CommissionSource {
        #[default]
        PlatformFee,    // 从平台费中扣除
        ShopFund,       // 店铺运营资金承担
        Mixed,          // 混合模式
    }

    // ============================================================================
    // 各模式配置结构
    // ============================================================================

    /// 直推奖励配置
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct DirectRewardConfig {
        /// 直推返佣率（基点，500 = 5%）
        pub rate: u16,
    }

    /// 多级分销层级配置
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct MultiLevelTier {
        /// 返佣率（基点，500 = 5%）
        pub rate: u16,
        /// 激活所需直推人数（0 表示无条件）
        pub required_directs: u32,
        /// 激活所需团队人数（0 表示无条件）
        pub required_team_size: u32,
        /// 激活所需消费金额（USDT，6位精度，0 表示无条件）
        pub required_spent: u128,
    }

    /// 多级分销配置（支持 N 层 + 激活条件）
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    #[scale_info(skip_type_params(MaxLevels))]
    pub struct MultiLevelConfig<MaxLevels: Get<u32>> {
        /// 各层配置
        pub levels: BoundedVec<MultiLevelTier, MaxLevels>,
        /// 最大返佣比例上限（基点，防止超过利润）
        pub max_total_rate: u16,
    }

    impl<MaxLevels: Get<u32>> Default for MultiLevelConfig<MaxLevels> {
        fn default() -> Self {
            Self {
                levels: BoundedVec::default(),
                max_total_rate: 1500, // 默认 15% 上限
            }
        }
    }

    /// 多级分销配置类型别名
    pub type MultiLevelConfigOf<T> = MultiLevelConfig<<T as Config>::MaxMultiLevels>;

    /// 团队业绩配置
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct TeamPerformanceConfig {
        /// 团队业绩返佣率（基点）
        pub rate: u16,
        /// 结算周期（区块数）
        pub settlement_period: u32,
        /// 最低业绩门槛
        pub min_performance: u128,
    }

    /// 等级差价配置
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct LevelDiffConfig {
        /// Normal 等级返佣率（基点）
        pub normal_rate: u16,
        /// Silver 等级返佣率（基点）
        pub silver_rate: u16,
        /// Gold 等级返佣率（基点）
        pub gold_rate: u16,
        /// Platinum 等级返佣率（基点）
        pub platinum_rate: u16,
        /// Diamond 等级返佣率（基点）
        pub diamond_rate: u16,
    }

    impl LevelDiffConfig {
        pub fn rate_for_level(&self, level: MemberLevel) -> u16 {
            match level {
                MemberLevel::Normal => self.normal_rate,
                MemberLevel::Silver => self.silver_rate,
                MemberLevel::Gold => self.gold_rate,
                MemberLevel::Platinum => self.platinum_rate,
                MemberLevel::Diamond => self.diamond_rate,
            }
        }
    }

    /// 自定义等级极差分润配置（方案 B）
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    #[scale_info(skip_type_params(MaxLevels))]
    pub struct CustomLevelDiffConfig<MaxLevels: Get<u32>> {
        /// 各自定义等级的返佣率（按 level_id 顺序）
        /// levels[0] = 等级0的返佣率（基点）
        /// levels[1] = 等级1的返佣率（基点）
        pub level_rates: BoundedVec<u16, MaxLevels>,
        /// 最大遍历层级
        pub max_depth: u8,
    }

    impl<MaxLevels: Get<u32>> Default for CustomLevelDiffConfig<MaxLevels> {
        fn default() -> Self {
            Self {
                level_rates: BoundedVec::default(),
                max_depth: 10,
            }
        }
    }

    /// 自定义等级极差配置类型别名
    pub type CustomLevelDiffConfigOf<T> = CustomLevelDiffConfig<<T as Config>::MaxCustomLevels>;

    /// 固定金额配置
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct FixedAmountConfig<Balance> {
        /// 固定返佣金额
        pub amount: Balance,
    }

    impl<Balance: Default> Default for FixedAmountConfig<Balance> {
        fn default() -> Self {
            Self { amount: Balance::default() }
        }
    }

    /// 首单奖励配置
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct FirstOrderConfig<Balance> {
        /// 首单奖励金额（use_amount=true 时使用）
        pub amount: Balance,
        /// 首单奖励比例（基点，use_amount=false 时使用）
        pub rate: u16,
        /// 使用金额还是比例
        pub use_amount: bool,
    }

    impl<Balance: Default> Default for FirstOrderConfig<Balance> {
        fn default() -> Self {
            Self {
                amount: Balance::default(),
                rate: 0,
                use_amount: true,
            }
        }
    }

    /// 复购奖励配置
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct RepeatPurchaseConfig {
        /// 复购返佣率（基点）
        pub rate: u16,
        /// 最低复购次数（达到此次数后才有奖励）
        pub min_orders: u32,
    }

    /// 分级提现配置（方案 B）
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct WithdrawalTierConfig {
        /// 提现比例（基点，6000 = 60%）
        pub withdrawal_rate: u16,
        /// 复购比例（基点，4000 = 40%）
        pub repurchase_rate: u16,
    }

    impl Default for WithdrawalTierConfig {
        fn default() -> Self {
            Self {
                withdrawal_rate: 10000,  // 默认 100% 提现
                repurchase_rate: 0,      // 默认 0% 复购
            }
        }
    }

    /// 店铺提现配置
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    #[scale_info(skip_type_params(MaxLevels))]
    pub struct ShopWithdrawalConfig<MaxLevels: Get<u32>> {
        /// 各等级的提现配置（按 level_id 顺序）
        pub tier_configs: BoundedVec<WithdrawalTierConfig, MaxLevels>,
        /// 是否启用分级提现
        pub enabled: bool,
        /// 购物余额消费是否产生返佣
        pub shopping_balance_generates_commission: bool,
    }

    impl<MaxLevels: Get<u32>> Default for ShopWithdrawalConfig<MaxLevels> {
        fn default() -> Self {
            Self {
                tier_configs: BoundedVec::default(),
                enabled: false,
                shopping_balance_generates_commission: false,
            }
        }
    }

    /// 店铺提现配置类型别名
    pub type ShopWithdrawalConfigOf<T> = ShopWithdrawalConfig<<T as Config>::MaxCustomLevels>;

    /// 单线收益配置
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct SingleLineConfig<Balance> {
        /// 上线收益率（基点，10 = 0.1%）
        pub upline_rate: u16,
        /// 下线收益率（基点，10 = 0.1%）
        pub downline_rate: u16,
        /// 基础上线层数
        pub base_upline_levels: u8,
        /// 基础下线层数
        pub base_downline_levels: u8,
        /// 每增加此消费额，增加 1 层（USDT，6位精度）
        pub level_increment_threshold: Balance,
        /// 最大上线层数
        pub max_upline_levels: u8,
        /// 最大下线层数
        pub max_downline_levels: u8,
    }

    impl<Balance: Default> Default for SingleLineConfig<Balance> {
        fn default() -> Self {
            Self {
                upline_rate: 10,           // 0.1%
                downline_rate: 10,         // 0.1%
                base_upline_levels: 10,
                base_downline_levels: 15,
                level_increment_threshold: Balance::default(),
                max_upline_levels: 20,
                max_downline_levels: 30,
            }
        }
    }

    // ============================================================================
    // 店铺返佣配置
    // ============================================================================

    /// 店铺返佣配置
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    #[scale_info(skip_type_params(MaxLevels))]
    pub struct ShopCommissionConfig<Balance, MaxLevels: Get<u32>> {
        /// 启用的返佣模式（位标志，可多选）
        pub enabled_modes: CommissionModes,
        /// 返佣来源
        pub source: CommissionSource,
        /// 返佣上限比例（基点，相对于可用池，10000 = 100%）
        pub max_commission_rate: u16,
        /// 是否全局启用
        pub enabled: bool,
        /// 直推奖励配置
        pub direct_reward: DirectRewardConfig,
        /// 多级分销配置
        pub multi_level: MultiLevelConfig<MaxLevels>,
        /// 团队业绩配置
        pub team_performance: TeamPerformanceConfig,
        /// 等级差价配置
        pub level_diff: LevelDiffConfig,
        /// 固定金额配置
        pub fixed_amount: FixedAmountConfig<Balance>,
        /// 首单奖励配置
        pub first_order: FirstOrderConfig<Balance>,
        /// 复购奖励配置
        pub repeat_purchase: RepeatPurchaseConfig,
        /// 单线收益配置
        pub single_line: SingleLineConfig<Balance>,
    }

    impl<Balance: Default, MaxLevels: Get<u32>> Default for ShopCommissionConfig<Balance, MaxLevels> {
        fn default() -> Self {
            Self {
                enabled_modes: CommissionModes::default(),
                source: CommissionSource::default(),
                max_commission_rate: 10000, // 100%
                enabled: false,
                direct_reward: DirectRewardConfig::default(),
                multi_level: MultiLevelConfig::default(),
                team_performance: TeamPerformanceConfig::default(),
                level_diff: LevelDiffConfig::default(),
                fixed_amount: FixedAmountConfig::default(),
                first_order: FirstOrderConfig::default(),
                repeat_purchase: RepeatPurchaseConfig::default(),
                single_line: SingleLineConfig::default(),
            }
        }
    }

    /// 店铺返佣配置类型别名
    pub type ShopCommissionConfigOf<T> = ShopCommissionConfig<BalanceOf<T>, <T as Config>::MaxMultiLevels>;

    // ============================================================================
    // 返佣记录
    // ============================================================================

    /// 返佣类型
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub enum CommissionType {
        DirectReward,
        MultiLevel,
        TeamPerformance,
        LevelDiff,
        FixedAmount,
        FirstOrder,
        RepeatPurchase,
        SingleLineUpline,
        SingleLineDownline,
    }

    /// 返佣状态
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub enum CommissionStatus {
        #[default]
        Pending,      // 待发放
        Distributed,  // 已发放
        Withdrawn,    // 已提取
        Cancelled,    // 已取消
    }

    /// 返佣记录
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct CommissionRecord<AccountId, Balance, BlockNumber> {
        /// 店铺 ID
        pub shop_id: u64,
        /// 订单 ID
        pub order_id: u64,
        /// 买家
        pub buyer: AccountId,
        /// 受益人
        pub beneficiary: AccountId,
        /// 返佣金额
        pub amount: Balance,
        /// 返佣类型
        pub commission_type: CommissionType,
        /// 层级（三级分销用）
        pub level: u8,
        /// 状态
        pub status: CommissionStatus,
        /// 创建时间
        pub created_at: BlockNumber,
    }

    /// 返佣记录类型别名
    pub type CommissionRecordOf<T> = CommissionRecord<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
    >;

    /// 会员返佣统计
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct MemberCommissionStatsData<Balance: Default> {
        /// 累计获得返佣
        pub total_earned: Balance,
        /// 待提取返佣
        pub pending: Balance,
        /// 已提取返佣
        pub withdrawn: Balance,
        /// 订单数（用于复购判断）
        pub order_count: u32,
    }

    /// 会员返佣统计类型别名
    pub type MemberCommissionStatsOf<T> = MemberCommissionStatsData<BalanceOf<T>>;

    // ============================================================================
    // Pallet 配置
    // ============================================================================

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 货币类型
        type Currency: Currency<Self::AccountId>;

        /// 店铺查询接口
        type ShopProvider: ShopProvider<Self::AccountId>;

        /// 会员查询接口（获取推荐人、等级）
        type MemberProvider: MemberProvider<Self::AccountId>;

        /// 最大返佣记录数（每订单）
        #[pallet::constant]
        type MaxCommissionRecordsPerOrder: Get<u32>;

        /// 最大单线长度（每店铺）
        #[pallet::constant]
        type MaxSingleLineLength: Get<u32>;

        /// 最大多级分销层数
        #[pallet::constant]
        type MaxMultiLevels: Get<u32>;

        /// 最大自定义等级数
        #[pallet::constant]
        type MaxCustomLevels: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ============================================================================
    // 存储项
    // ============================================================================

    /// 店铺返佣配置 shop_id -> ShopCommissionConfig
    #[pallet::storage]
    #[pallet::getter(fn shop_commission_config)]
    pub type ShopCommissionConfigs<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        ShopCommissionConfigOf<T>,
    >;

    /// 会员返佣统计 (shop_id, account) -> MemberCommissionStatsData
    #[pallet::storage]
    #[pallet::getter(fn member_commission_stats)]
    pub type MemberCommissionStats<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u64,
        Blake2_128Concat, T::AccountId,
        MemberCommissionStatsOf<T>,
        ValueQuery,
    >;

    /// 订单返佣记录 order_id -> Vec<CommissionRecord>
    #[pallet::storage]
    #[pallet::getter(fn order_commission_records)]
    pub type OrderCommissionRecords<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        BoundedVec<CommissionRecordOf<T>, T::MaxCommissionRecordsPerOrder>,
        ValueQuery,
    >;

    /// 店铺返佣统计 shop_id -> (total_distributed, total_orders)
    #[pallet::storage]
    #[pallet::getter(fn shop_commission_stats)]
    pub type ShopCommissionTotals<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        (BalanceOf<T>, u64),
        ValueQuery,
    >;

    /// 店铺消费单链 shop_id -> Vec<AccountId>（按首次消费顺序）
    #[pallet::storage]
    #[pallet::getter(fn shop_single_line)]
    pub type ShopSingleLine<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        BoundedVec<T::AccountId, T::MaxSingleLineLength>,
        ValueQuery,
    >;

    /// 用户在单链中的位置 (shop_id, account) -> index
    #[pallet::storage]
    #[pallet::getter(fn single_line_index)]
    pub type SingleLineIndex<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u64,
        Blake2_128Concat, T::AccountId,
        u32,
    >;

    /// 店铺自定义等级极差配置 shop_id -> CustomLevelDiffConfig
    #[pallet::storage]
    #[pallet::getter(fn shop_custom_level_diff_config)]
    pub type ShopCustomLevelDiffConfigs<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        CustomLevelDiffConfigOf<T>,
    >;

    /// 店铺提现配置 shop_id -> ShopWithdrawalConfig
    #[pallet::storage]
    #[pallet::getter(fn shop_withdrawal_config)]
    pub type ShopWithdrawalConfigs<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        ShopWithdrawalConfigOf<T>,
    >;

    /// 会员购物余额 (shop_id, account) -> Balance
    #[pallet::storage]
    #[pallet::getter(fn member_shopping_balance)]
    pub type MemberShoppingBalance<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u64,
        Blake2_128Concat, T::AccountId,
        BalanceOf<T>,
        ValueQuery,
    >;

    // ============================================================================
    // 事件
    // ============================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 返佣配置更新
        CommissionConfigUpdated {
            shop_id: u64,
        },
        /// 返佣模式更新
        CommissionModesUpdated {
            shop_id: u64,
            modes: CommissionModes,
        },
        /// 返佣发放
        CommissionDistributed {
            shop_id: u64,
            order_id: u64,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            commission_type: CommissionType,
            level: u8,
        },
        /// 返佣提取
        CommissionWithdrawn {
            shop_id: u64,
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 返佣取消
        CommissionCancelled {
            order_id: u64,
        },
        /// 分级提现（部分转入购物余额）
        TieredWithdrawal {
            shop_id: u64,
            account: T::AccountId,
            withdrawn_amount: BalanceOf<T>,
            repurchase_amount: BalanceOf<T>,
        },
        /// 提现配置更新
        WithdrawalConfigUpdated {
            shop_id: u64,
        },
        /// 购物余额消费
        ShoppingBalanceUsed {
            shop_id: u64,
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
    }

    // ============================================================================
    // 错误
    // ============================================================================

    #[pallet::error]
    pub enum Error<T> {
        /// 店铺不存在
        ShopNotFound,
        /// 不是店主
        NotShopOwner,
        /// 返佣未配置
        CommissionNotConfigured,
        /// 返佣余额不足
        InsufficientCommission,
        /// 无效的返佣率
        InvalidCommissionRate,
        /// 记录数已满
        RecordsFull,
        /// 数值溢出
        Overflow,
        /// 单线已满
        SingleLineFull,
        /// 提现配置未启用
        WithdrawalConfigNotEnabled,
        /// 无效的提现配置
        InvalidWithdrawalConfig,
    }

    // ============================================================================
    // Extrinsics
    // ============================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 设置启用的返佣模式（多选）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `modes`: 返佣模式位标志
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn set_commission_modes(
            origin: OriginFor<T>,
            shop_id: u64,
            modes: CommissionModes,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
                let config = maybe_config.get_or_insert_with(ShopCommissionConfig::default);
                config.enabled_modes = modes;
            });

            Self::deposit_event(Event::CommissionModesUpdated { shop_id, modes });
            Ok(())
        }

        /// 设置直推奖励配置
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn set_direct_reward_config(
            origin: OriginFor<T>,
            shop_id: u64,
            rate: u16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;
            ensure!(rate <= 10000, Error::<T>::InvalidCommissionRate);

            ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
                let config = maybe_config.get_or_insert_with(ShopCommissionConfig::default);
                config.direct_reward.rate = rate;
            });

            Self::deposit_event(Event::CommissionConfigUpdated { shop_id });
            Ok(())
        }

        /// 设置多级分销配置
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `levels`: 各层配置列表
        /// - `max_total_rate`: 最大返佣比例上限（基点）
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn set_multi_level_config(
            origin: OriginFor<T>,
            shop_id: u64,
            levels: BoundedVec<MultiLevelTier, T::MaxMultiLevels>,
            max_total_rate: u16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            // 验证各层返佣率
            for tier in levels.iter() {
                ensure!(tier.rate <= 10000, Error::<T>::InvalidCommissionRate);
            }
            ensure!(max_total_rate <= 10000, Error::<T>::InvalidCommissionRate);

            ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
                let config = maybe_config.get_or_insert_with(ShopCommissionConfig::default);
                config.multi_level = MultiLevelConfig {
                    levels,
                    max_total_rate,
                };
            });

            Self::deposit_event(Event::CommissionConfigUpdated { shop_id });
            Ok(())
        }

        /// 设置等级差价配置（全局等级体系）
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn set_level_diff_config(
            origin: OriginFor<T>,
            shop_id: u64,
            normal_rate: u16,
            silver_rate: u16,
            gold_rate: u16,
            platinum_rate: u16,
            diamond_rate: u16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
                let config = maybe_config.get_or_insert_with(ShopCommissionConfig::default);
                config.level_diff = LevelDiffConfig {
                    normal_rate,
                    silver_rate,
                    gold_rate,
                    platinum_rate,
                    diamond_rate,
                };
            });

            Self::deposit_event(Event::CommissionConfigUpdated { shop_id });
            Ok(())
        }

        /// 设置自定义等级极差配置（方案 B）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `level_rates`: 各自定义等级的返佣率（按 level_id 顺序，基点）
        /// - `max_depth`: 最大遍历层级
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn set_custom_level_diff_config(
            origin: OriginFor<T>,
            shop_id: u64,
            level_rates: BoundedVec<u16, T::MaxCustomLevels>,
            max_depth: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            // 验证各等级返佣率
            for rate in level_rates.iter() {
                ensure!(*rate <= 10000, Error::<T>::InvalidCommissionRate);
            }
            ensure!(max_depth > 0 && max_depth <= 20, Error::<T>::InvalidCommissionRate);

            ShopCustomLevelDiffConfigs::<T>::insert(shop_id, CustomLevelDiffConfig {
                level_rates,
                max_depth,
            });

            Self::deposit_event(Event::CommissionConfigUpdated { shop_id });
            Ok(())
        }

        /// 设置固定金额配置
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn set_fixed_amount_config(
            origin: OriginFor<T>,
            shop_id: u64,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
                let config = maybe_config.get_or_insert_with(ShopCommissionConfig::default);
                config.fixed_amount = FixedAmountConfig { amount };
            });

            Self::deposit_event(Event::CommissionConfigUpdated { shop_id });
            Ok(())
        }

        /// 设置首单奖励配置
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn set_first_order_config(
            origin: OriginFor<T>,
            shop_id: u64,
            amount: BalanceOf<T>,
            rate: u16,
            use_amount: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
                let config = maybe_config.get_or_insert_with(ShopCommissionConfig::default);
                config.first_order = FirstOrderConfig { amount, rate, use_amount };
            });

            Self::deposit_event(Event::CommissionConfigUpdated { shop_id });
            Ok(())
        }

        /// 设置复购奖励配置
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn set_repeat_purchase_config(
            origin: OriginFor<T>,
            shop_id: u64,
            rate: u16,
            min_orders: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
                let config = maybe_config.get_or_insert_with(ShopCommissionConfig::default);
                config.repeat_purchase = RepeatPurchaseConfig { rate, min_orders };
            });

            Self::deposit_event(Event::CommissionConfigUpdated { shop_id });
            Ok(())
        }

        /// 设置单线收益配置
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn set_single_line_config(
            origin: OriginFor<T>,
            shop_id: u64,
            upline_rate: u16,
            downline_rate: u16,
            base_upline_levels: u8,
            base_downline_levels: u8,
            level_increment_threshold: BalanceOf<T>,
            max_upline_levels: u8,
            max_downline_levels: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;
            ensure!(upline_rate <= 1000 && downline_rate <= 1000, Error::<T>::InvalidCommissionRate);

            ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
                let config = maybe_config.get_or_insert_with(ShopCommissionConfig::default);
                config.single_line = SingleLineConfig {
                    upline_rate,
                    downline_rate,
                    base_upline_levels,
                    base_downline_levels,
                    level_increment_threshold,
                    max_upline_levels,
                    max_downline_levels,
                };
            });

            Self::deposit_event(Event::CommissionConfigUpdated { shop_id });
            Ok(())
        }

        /// 设置返佣来源和上限
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn set_commission_source(
            origin: OriginFor<T>,
            shop_id: u64,
            source: CommissionSource,
            max_rate: u16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;
            ensure!(max_rate <= 10000, Error::<T>::InvalidCommissionRate);

            ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
                let config = maybe_config.get_or_insert_with(ShopCommissionConfig::default);
                config.source = source;
                config.max_commission_rate = max_rate;
            });

            Self::deposit_event(Event::CommissionConfigUpdated { shop_id });
            Ok(())
        }

        /// 启用/禁用返佣
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn enable_commission(
            origin: OriginFor<T>,
            shop_id: u64,
            enabled: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
                let config = maybe_config.get_or_insert_with(ShopCommissionConfig::default);
                config.enabled = enabled;
            });

            Self::deposit_event(Event::CommissionConfigUpdated { shop_id });
            Ok(())
        }

        /// 提取返佣（支持分级提现）
        ///
        /// 如果店铺启用了分级提现，则根据用户等级确定提现比例和复购比例
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(35_000, 0))]
        pub fn withdraw_commission(
            origin: OriginFor<T>,
            shop_id: u64,
            amount: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            MemberCommissionStats::<T>::try_mutate(shop_id, &who, |stats| -> DispatchResult {
                let total_amount = amount.unwrap_or(stats.pending);
                ensure!(stats.pending >= total_amount, Error::<T>::InsufficientCommission);

                // 检查是否启用分级提现
                let withdrawal_config = ShopWithdrawalConfigs::<T>::get(shop_id);
                
                let (withdrawal_amount, repurchase_amount) = if let Some(ref config) = withdrawal_config {
                    if config.enabled && !config.tier_configs.is_empty() {
                        // 获取用户等级
                        let level_id = T::MemberProvider::custom_level_id(shop_id, &who);
                        
                        // 获取该等级的提现配置
                        let tier_config = config.tier_configs
                            .get(level_id as usize)
                            .cloned()
                            .unwrap_or_default();
                        
                        // 计算提现和复购金额
                        let withdrawal = total_amount
                            .saturating_mul(tier_config.withdrawal_rate.into())
                            / 10000u32.into();
                        let repurchase = total_amount.saturating_sub(withdrawal);
                        
                        (withdrawal, repurchase)
                    } else {
                        (total_amount, BalanceOf::<T>::zero())
                    }
                } else {
                    (total_amount, BalanceOf::<T>::zero())
                };

                // 从店铺派生账户转账提现部分到用户钱包
                if !withdrawal_amount.is_zero() {
                    let shop_account = T::ShopProvider::shop_account(shop_id);
                    // P0 安全修复: 预检查店铺账户余额
                    let shop_balance = T::Currency::free_balance(&shop_account);
                    ensure!(shop_balance >= withdrawal_amount, Error::<T>::InsufficientCommission);
                    
                    T::Currency::transfer(
                        &shop_account,
                        &who,
                        withdrawal_amount,
                        ExistenceRequirement::KeepAlive,
                    )?;
                }

                // 复购部分转入购物余额
                if !repurchase_amount.is_zero() {
                    MemberShoppingBalance::<T>::mutate(shop_id, &who, |balance| {
                        *balance = balance.saturating_add(repurchase_amount);
                    });
                }

                stats.pending = stats.pending.saturating_sub(total_amount);
                stats.withdrawn = stats.withdrawn.saturating_add(withdrawal_amount);

                // 发出事件
                if !repurchase_amount.is_zero() {
                    Self::deposit_event(Event::TieredWithdrawal {
                        shop_id,
                        account: who.clone(),
                        withdrawn_amount: withdrawal_amount,
                        repurchase_amount,
                    });
                } else {
                    Self::deposit_event(Event::CommissionWithdrawn {
                        shop_id,
                        account: who.clone(),
                        amount: withdrawal_amount,
                    });
                }

                Ok(())
            })
        }

        /// 设置分级提现配置
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `tier_configs`: 各等级的提现配置
        /// - `enabled`: 是否启用分级提现
        /// - `shopping_balance_generates_commission`: 购物余额消费是否产生返佣
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn set_withdrawal_config(
            origin: OriginFor<T>,
            shop_id: u64,
            tier_configs: BoundedVec<WithdrawalTierConfig, T::MaxCustomLevels>,
            enabled: bool,
            shopping_balance_generates_commission: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            // 验证配置
            for config in tier_configs.iter() {
                ensure!(
                    config.withdrawal_rate.saturating_add(config.repurchase_rate) == 10000,
                    Error::<T>::InvalidWithdrawalConfig
                );
            }

            ShopWithdrawalConfigs::<T>::insert(shop_id, ShopWithdrawalConfig {
                tier_configs,
                enabled,
                shopping_balance_generates_commission,
            });

            Self::deposit_event(Event::WithdrawalConfigUpdated { shop_id });
            Ok(())
        }

        /// 使用购物余额支付
        ///
        /// 此函数由订单模块调用，用于扣除购物余额
        #[pallet::call_index(13)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn use_shopping_balance(
            origin: OriginFor<T>,
            shop_id: u64,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            MemberShoppingBalance::<T>::try_mutate(shop_id, &who, |balance| -> DispatchResult {
                ensure!(*balance >= amount, Error::<T>::InsufficientCommission);
                *balance = balance.saturating_sub(amount);
                
                Self::deposit_event(Event::ShoppingBalanceUsed {
                    shop_id,
                    account: who.clone(),
                    amount,
                });
                
                Ok(())
            })
        }
    }

    // ============================================================================
    // 内部函数
    // ============================================================================

    impl<T: Config> Pallet<T> {
        /// 验证店主权限
        fn ensure_shop_owner(shop_id: u64, who: &T::AccountId) -> DispatchResult {
            let owner = T::ShopProvider::shop_owner(shop_id)
                .ok_or(Error::<T>::ShopNotFound)?;
            ensure!(*who == owner, Error::<T>::NotShopOwner);
            Ok(())
        }

        /// 处理订单返佣（多模式叠加）
        pub fn process_commission(
            shop_id: u64,
            order_id: u64,
            buyer: &T::AccountId,
            order_amount: BalanceOf<T>,
            available_pool: BalanceOf<T>,
        ) -> DispatchResult {
            let config = match ShopCommissionConfigs::<T>::get(shop_id) {
                Some(c) if c.enabled => c,
                _ => return Ok(()), // 未配置或未启用
            };

            // 计算最大可用返佣
            let max_commission = available_pool
                .saturating_mul(config.max_commission_rate.into())
                / 10000u32.into();
            let mut remaining = max_commission;
            let now = <frame_system::Pallet<T>>::block_number();

            // 获取买家订单数（用于首单/复购判断）
            let buyer_stats = MemberCommissionStats::<T>::get(shop_id, buyer);
            let is_first_order = buyer_stats.order_count == 0;

            // 1. 直推奖励
            if config.enabled_modes.contains(CommissionModes::DIRECT_REWARD) {
                Self::process_direct_reward(
                    shop_id, order_id, buyer, order_amount, &mut remaining, &config.direct_reward, now
                )?;
            }

            // 2. 三级分销
            if config.enabled_modes.contains(CommissionModes::MULTI_LEVEL) {
                Self::process_multi_level(
                    shop_id, order_id, buyer, order_amount, &mut remaining, &config.multi_level, now
                )?;
            }

            // 3. 等级差价
            if config.enabled_modes.contains(CommissionModes::LEVEL_DIFF) {
                Self::process_level_diff(
                    shop_id, order_id, buyer, order_amount, &mut remaining, &config.level_diff, now
                )?;
            }

            // 4. 固定金额
            if config.enabled_modes.contains(CommissionModes::FIXED_AMOUNT) {
                Self::process_fixed_amount(
                    shop_id, order_id, buyer, &mut remaining, &config.fixed_amount, now
                )?;
            }

            // 5. 首单奖励
            if config.enabled_modes.contains(CommissionModes::FIRST_ORDER) && is_first_order {
                Self::process_first_order(
                    shop_id, order_id, buyer, order_amount, &mut remaining, &config.first_order, now
                )?;
            }

            // 6. 复购奖励
            if config.enabled_modes.contains(CommissionModes::REPEAT_PURCHASE) {
                Self::process_repeat_purchase(
                    shop_id, order_id, buyer, order_amount, &mut remaining, &config.repeat_purchase, &buyer_stats, now
                )?;
            }

            // 7. 单线上线收益
            if config.enabled_modes.contains(CommissionModes::SINGLE_LINE_UPLINE) {
                Self::process_single_line_upline(
                    shop_id, order_id, buyer, &mut remaining, &config.single_line, &buyer_stats, now
                )?;
            }

            // 8. 单线下线收益
            if config.enabled_modes.contains(CommissionModes::SINGLE_LINE_DOWNLINE) {
                Self::process_single_line_downline(
                    shop_id, order_id, buyer, &mut remaining, &config.single_line, &buyer_stats, now
                )?;
            }

            // 将买家加入单链（如果是首次消费）
            if is_first_order {
                Self::add_to_single_line(shop_id, buyer)?;
            }

            // 更新买家订单数
            MemberCommissionStats::<T>::mutate(shop_id, buyer, |stats| {
                stats.order_count = stats.order_count.saturating_add(1);
            });

            // 更新店铺统计
            let distributed = max_commission.saturating_sub(remaining);
            ShopCommissionTotals::<T>::mutate(shop_id, |(total, orders)| {
                *total = total.saturating_add(distributed);
                *orders = orders.saturating_add(1);
            });

            Ok(())
        }

        /// 处理直推奖励
        fn process_direct_reward(
            shop_id: u64,
            order_id: u64,
            buyer: &T::AccountId,
            order_amount: BalanceOf<T>,
            remaining: &mut BalanceOf<T>,
            config: &DirectRewardConfig,
            now: BlockNumberFor<T>,
        ) -> DispatchResult {
            if config.rate == 0 {
                return Ok(());
            }

            if let Some(referrer) = T::MemberProvider::get_referrer(shop_id, buyer) {
                let commission = order_amount
                    .saturating_mul(config.rate.into())
                    / 10000u32.into();
                let actual = commission.min(*remaining);

                if !actual.is_zero() {
                    *remaining = remaining.saturating_sub(actual);
                    Self::credit_commission(
                        shop_id, order_id, buyer, &referrer, actual,
                        CommissionType::DirectReward, 1, now
                    )?;
                }
            }

            Ok(())
        }

        /// 处理多级分销（支持 N 层 + 激活条件）
        fn process_multi_level(
            shop_id: u64,
            order_id: u64,
            buyer: &T::AccountId,
            order_amount: BalanceOf<T>,
            remaining: &mut BalanceOf<T>,
            config: &MultiLevelConfigOf<T>,
            now: BlockNumberFor<T>,
        ) -> DispatchResult {
            if config.levels.is_empty() {
                return Ok(());
            }

            let mut current_referrer = T::MemberProvider::get_referrer(shop_id, buyer);
            let mut total_commission = BalanceOf::<T>::zero();
            let max_commission = order_amount
                .saturating_mul(config.max_total_rate.into())
                / 10000u32.into();

            for (level_idx, tier) in config.levels.iter().enumerate() {
                if tier.rate == 0 {
                    // 跳过但继续向上
                    current_referrer = current_referrer.and_then(|r| T::MemberProvider::get_referrer(shop_id, &r));
                    continue;
                }

                let Some(ref referrer) = current_referrer else { break };

                // 检查激活条件
                if !Self::check_tier_activation(shop_id, referrer, tier) {
                    // 未激活，跳过但继续向上
                    current_referrer = T::MemberProvider::get_referrer(shop_id, referrer);
                    continue;
                }

                // 计算返佣
                let commission = order_amount
                    .saturating_mul(tier.rate.into())
                    / 10000u32.into();
                let actual = commission.min(*remaining);

                if actual.is_zero() {
                    break;
                }

                // 检查总返佣上限
                let new_total = total_commission.saturating_add(actual);
                if new_total > max_commission {
                    // 超过上限，计算剩余可发放金额
                    let can_distribute = max_commission.saturating_sub(total_commission);
                    if !can_distribute.is_zero() {
                        *remaining = remaining.saturating_sub(can_distribute);
                        total_commission = total_commission.saturating_add(can_distribute);
                        Self::credit_commission(
                            shop_id, order_id, buyer, referrer, can_distribute,
                            CommissionType::MultiLevel, (level_idx + 1) as u8, now
                        )?;
                    }
                    break;
                }

                *remaining = remaining.saturating_sub(actual);
                total_commission = total_commission.saturating_add(actual);
                Self::credit_commission(
                    shop_id, order_id, buyer, referrer, actual,
                    CommissionType::MultiLevel, (level_idx + 1) as u8, now
                )?;

                current_referrer = T::MemberProvider::get_referrer(shop_id, referrer);
            }

            Ok(())
        }

        /// 检查层级激活条件
        fn check_tier_activation(
            shop_id: u64,
            account: &T::AccountId,
            tier: &MultiLevelTier,
        ) -> bool {
            // 无条件激活
            if tier.required_directs == 0 && tier.required_team_size == 0 && tier.required_spent == 0 {
                return true;
            }

            // 获取会员信息
            let (direct_referrals, team_size, total_spent) = T::MemberProvider::get_member_stats(shop_id, account);

            // 检查直推人数
            if tier.required_directs > 0 && direct_referrals < tier.required_directs {
                return false;
            }

            // 检查团队人数
            if tier.required_team_size > 0 && team_size < tier.required_team_size {
                return false;
            }

            // 检查消费金额
            if tier.required_spent > 0 && total_spent < tier.required_spent {
                return false;
            }

            true
        }

        /// 处理等级差价（支持全局等级和自定义等级）
        fn process_level_diff(
            shop_id: u64,
            order_id: u64,
            buyer: &T::AccountId,
            order_amount: BalanceOf<T>,
            remaining: &mut BalanceOf<T>,
            config: &LevelDiffConfig,
            now: BlockNumberFor<T>,
        ) -> DispatchResult {
            // 检查是否使用自定义等级
            let uses_custom = T::MemberProvider::uses_custom_levels(shop_id);
            let custom_config = ShopCustomLevelDiffConfigs::<T>::get(shop_id);

            // 确定最大遍历层级
            let max_depth = if uses_custom {
                custom_config.as_ref().map(|c| c.max_depth).unwrap_or(10)
            } else {
                10
            };

            let mut current_referrer = T::MemberProvider::get_referrer(shop_id, buyer);
            let mut prev_rate: u16 = 0;
            let mut level: u8 = 0;

            while let Some(ref referrer) = current_referrer {
                level += 1;
                if level > max_depth {
                    break;
                }

                // 根据等级体系获取返佣率
                let referrer_rate = if uses_custom {
                    let level_id = T::MemberProvider::custom_level_id(shop_id, referrer);
                    custom_config.as_ref()
                        .and_then(|c| c.level_rates.get(level_id as usize).copied())
                        .unwrap_or(0)
                } else {
                    let referrer_level = T::MemberProvider::member_level(shop_id, referrer)
                        .unwrap_or(MemberLevel::Normal);
                    config.rate_for_level(referrer_level)
                };

                // 等级差价 = 当前等级返佣率 - 下级等级返佣率
                if referrer_rate > prev_rate {
                    let diff_rate = referrer_rate - prev_rate;
                    let commission = order_amount
                        .saturating_mul(diff_rate.into())
                        / 10000u32.into();
                    let actual = commission.min(*remaining);

                    if !actual.is_zero() {
                        *remaining = remaining.saturating_sub(actual);
                        Self::credit_commission(
                            shop_id, order_id, buyer, referrer, actual,
                            CommissionType::LevelDiff, level, now
                        )?;
                    }

                    prev_rate = referrer_rate;
                }

                current_referrer = T::MemberProvider::get_referrer(shop_id, referrer);
            }

            Ok(())
        }

        /// 处理固定金额
        fn process_fixed_amount(
            shop_id: u64,
            order_id: u64,
            buyer: &T::AccountId,
            remaining: &mut BalanceOf<T>,
            config: &FixedAmountConfig<BalanceOf<T>>,
            now: BlockNumberFor<T>,
        ) -> DispatchResult {
            if config.amount.is_zero() {
                return Ok(());
            }

            if let Some(referrer) = T::MemberProvider::get_referrer(shop_id, buyer) {
                let actual = config.amount.min(*remaining);

                if !actual.is_zero() {
                    *remaining = remaining.saturating_sub(actual);
                    Self::credit_commission(
                        shop_id, order_id, buyer, &referrer, actual,
                        CommissionType::FixedAmount, 1, now
                    )?;
                }
            }

            Ok(())
        }

        /// 处理首单奖励
        fn process_first_order(
            shop_id: u64,
            order_id: u64,
            buyer: &T::AccountId,
            order_amount: BalanceOf<T>,
            remaining: &mut BalanceOf<T>,
            config: &FirstOrderConfig<BalanceOf<T>>,
            now: BlockNumberFor<T>,
        ) -> DispatchResult {
            if let Some(referrer) = T::MemberProvider::get_referrer(shop_id, buyer) {
                let commission = if config.use_amount {
                    config.amount
                } else {
                    order_amount
                        .saturating_mul(config.rate.into())
                        / 10000u32.into()
                };

                let actual = commission.min(*remaining);

                if !actual.is_zero() {
                    *remaining = remaining.saturating_sub(actual);
                    Self::credit_commission(
                        shop_id, order_id, buyer, &referrer, actual,
                        CommissionType::FirstOrder, 1, now
                    )?;
                }
            }

            Ok(())
        }

        /// 处理复购奖励
        fn process_repeat_purchase(
            shop_id: u64,
            order_id: u64,
            buyer: &T::AccountId,
            order_amount: BalanceOf<T>,
            remaining: &mut BalanceOf<T>,
            config: &RepeatPurchaseConfig,
            buyer_stats: &MemberCommissionStatsData<BalanceOf<T>>,
            now: BlockNumberFor<T>,
        ) -> DispatchResult {
            if config.rate == 0 || buyer_stats.order_count < config.min_orders {
                return Ok(());
            }

            if let Some(referrer) = T::MemberProvider::get_referrer(shop_id, buyer) {
                let commission = order_amount
                    .saturating_mul(config.rate.into())
                    / 10000u32.into();
                let actual = commission.min(*remaining);

                if !actual.is_zero() {
                    *remaining = remaining.saturating_sub(actual);
                    Self::credit_commission(
                        shop_id, order_id, buyer, &referrer, actual,
                        CommissionType::RepeatPurchase, 1, now
                    )?;
                }
            }

            Ok(())
        }

        /// 将用户加入单链
        fn add_to_single_line(shop_id: u64, account: &T::AccountId) -> DispatchResult {
            // 检查是否已在单链中
            if SingleLineIndex::<T>::contains_key(shop_id, account) {
                return Ok(());
            }

            ShopSingleLine::<T>::try_mutate(shop_id, |single_line| {
                let index = single_line.len() as u32;
                single_line.try_push(account.clone()).map_err(|_| Error::<T>::SingleLineFull)?;
                SingleLineIndex::<T>::insert(shop_id, account, index);
                Ok(())
            })
        }

        /// 处理单线上线收益
        fn process_single_line_upline(
            shop_id: u64,
            order_id: u64,
            buyer: &T::AccountId,
            remaining: &mut BalanceOf<T>,
            config: &SingleLineConfig<BalanceOf<T>>,
            buyer_stats: &MemberCommissionStatsData<BalanceOf<T>>,
            now: BlockNumberFor<T>,
        ) -> DispatchResult {
            if config.upline_rate == 0 {
                return Ok(());
            }

            // 获取买家在单链中的位置
            let buyer_index = match SingleLineIndex::<T>::get(shop_id, buyer) {
                Some(idx) => idx,
                None => return Ok(()), // 买家不在单链中（首次消费，稍后会加入）
            };

            if buyer_index == 0 {
                return Ok(()); // 没有上线
            }

            let single_line = ShopSingleLine::<T>::get(shop_id);

            // 计算可获取层数
            let extra_levels = if !config.level_increment_threshold.is_zero() {
                let threshold_u128: u128 = sp_runtime::SaturatedConversion::saturated_into(config.level_increment_threshold);
                let spent_u128: u128 = sp_runtime::SaturatedConversion::saturated_into(buyer_stats.total_earned);
                if threshold_u128 > 0 {
                    (spent_u128 / threshold_u128) as u8
                } else {
                    0
                }
            } else {
                0
            };
            let max_levels = config.base_upline_levels
                .saturating_add(extra_levels)
                .min(config.max_upline_levels) as u32;

            // 遍历上线
            for i in 1..=max_levels {
                if buyer_index < i {
                    break;
                }
                let upline_index = (buyer_index - i) as usize;
                if upline_index >= single_line.len() {
                    break;
                }
                let upline = &single_line[upline_index];

                // 获取上线累计消费额
                let upline_stats = MemberCommissionStats::<T>::get(shop_id, upline);

                // 计算收益：上线消费额 × upline_rate / 10000
                let commission = upline_stats.total_earned
                    .saturating_mul(config.upline_rate.into())
                    / 10000u32.into();
                let actual = commission.min(*remaining);

                if !actual.is_zero() {
                    *remaining = remaining.saturating_sub(actual);
                    Self::credit_commission(
                        shop_id, order_id, buyer, upline, actual,
                        CommissionType::SingleLineUpline, i as u8, now
                    )?;
                }
            }

            Ok(())
        }

        /// 处理单线下线收益
        fn process_single_line_downline(
            shop_id: u64,
            order_id: u64,
            buyer: &T::AccountId,
            remaining: &mut BalanceOf<T>,
            config: &SingleLineConfig<BalanceOf<T>>,
            buyer_stats: &MemberCommissionStatsData<BalanceOf<T>>,
            now: BlockNumberFor<T>,
        ) -> DispatchResult {
            if config.downline_rate == 0 {
                return Ok(());
            }

            // 获取买家在单链中的位置
            let buyer_index = match SingleLineIndex::<T>::get(shop_id, buyer) {
                Some(idx) => idx,
                None => return Ok(()), // 买家不在单链中
            };

            let single_line = ShopSingleLine::<T>::get(shop_id);
            let single_line_len = single_line.len() as u32;

            if buyer_index >= single_line_len - 1 {
                return Ok(()); // 没有下线
            }

            // 计算可获取层数
            let extra_levels = if !config.level_increment_threshold.is_zero() {
                let threshold_u128: u128 = sp_runtime::SaturatedConversion::saturated_into(config.level_increment_threshold);
                let spent_u128: u128 = sp_runtime::SaturatedConversion::saturated_into(buyer_stats.total_earned);
                if threshold_u128 > 0 {
                    (spent_u128 / threshold_u128) as u8
                } else {
                    0
                }
            } else {
                0
            };
            let max_levels = config.base_downline_levels
                .saturating_add(extra_levels)
                .min(config.max_downline_levels) as u32;

            // 遍历下线
            for i in 1..=max_levels {
                let downline_index = (buyer_index + i) as usize;
                if downline_index >= single_line.len() {
                    break;
                }
                let downline = &single_line[downline_index];

                // 获取下线累计消费额
                let downline_stats = MemberCommissionStats::<T>::get(shop_id, downline);

                // 计算收益：下线消费额 × downline_rate / 10000
                let commission = downline_stats.total_earned
                    .saturating_mul(config.downline_rate.into())
                    / 10000u32.into();
                let actual = commission.min(*remaining);

                if !actual.is_zero() {
                    *remaining = remaining.saturating_sub(actual);
                    Self::credit_commission(
                        shop_id, order_id, buyer, downline, actual,
                        CommissionType::SingleLineDownline, i as u8, now
                    )?;
                }
            }

            Ok(())
        }

        /// 记录并发放返佣
        fn credit_commission(
            shop_id: u64,
            order_id: u64,
            buyer: &T::AccountId,
            beneficiary: &T::AccountId,
            amount: BalanceOf<T>,
            commission_type: CommissionType,
            level: u8,
            now: BlockNumberFor<T>,
        ) -> DispatchResult {
            // 记录返佣
            let record = CommissionRecord {
                shop_id,
                order_id,
                buyer: buyer.clone(),
                beneficiary: beneficiary.clone(),
                amount,
                commission_type,
                level,
                status: CommissionStatus::Pending,
                created_at: now,
            };

            OrderCommissionRecords::<T>::try_mutate(order_id, |records| {
                records.try_push(record).map_err(|_| Error::<T>::RecordsFull)
            })?;

            // 更新受益人统计
            MemberCommissionStats::<T>::mutate(shop_id, beneficiary, |stats| {
                stats.total_earned = stats.total_earned.saturating_add(amount);
                stats.pending = stats.pending.saturating_add(amount);
            });

            Self::deposit_event(Event::CommissionDistributed {
                shop_id,
                order_id,
                beneficiary: beneficiary.clone(),
                amount,
                commission_type,
                level,
            });

            Ok(())
        }

        /// 取消订单返佣
        pub fn cancel_commission(order_id: u64) -> DispatchResult {
            OrderCommissionRecords::<T>::mutate(order_id, |records| {
                for record in records.iter_mut() {
                    if record.status == CommissionStatus::Pending {
                        // 扣减受益人待提取返佣
                        MemberCommissionStats::<T>::mutate(record.shop_id, &record.beneficiary, |stats| {
                            stats.pending = stats.pending.saturating_sub(record.amount);
                            stats.total_earned = stats.total_earned.saturating_sub(record.amount);
                        });
                        record.status = CommissionStatus::Cancelled;
                    }
                }
            });

            Self::deposit_event(Event::CommissionCancelled { order_id });
            Ok(())
        }
    }
}

// ============================================================================
// MemberProvider Trait（由 member 模块实现）
// ============================================================================

/// 会员服务接口
pub trait MemberProvider<AccountId> {
    /// 获取推荐人
    fn get_referrer(shop_id: u64, account: &AccountId) -> Option<AccountId>;

    /// 获取会员等级
    fn member_level(shop_id: u64, account: &AccountId) -> Option<pallet_entity_common::MemberLevel>;

    /// 获取会员统计信息 (直推人数, 团队人数, 累计消费USDT)
    fn get_member_stats(shop_id: u64, account: &AccountId) -> (u32, u32, u128);

    /// 检查店铺是否使用自定义等级
    fn uses_custom_levels(shop_id: u64) -> bool;

    /// 获取自定义等级 ID
    fn custom_level_id(shop_id: u64, account: &AccountId) -> u8;

    // ==================== 治理调用接口（新增）====================

    /// 启用/禁用自定义等级（治理调用）
    fn set_custom_levels_enabled(shop_id: u64, enabled: bool) -> sp_runtime::DispatchResult;

    /// 设置升级模式（治理调用）
    /// mode: 0=AutoUpgrade, 1=ManualUpgrade, 2=PeriodReset
    fn set_upgrade_mode(shop_id: u64, mode: u8) -> sp_runtime::DispatchResult;

    /// 添加自定义等级（治理调用）
    fn add_custom_level(
        shop_id: u64,
        level_id: u8,
        name: &[u8],
        threshold: u128,
        discount_rate: u16,
        commission_bonus: u16,
    ) -> sp_runtime::DispatchResult;

    /// 更新自定义等级（治理调用）
    fn update_custom_level(
        shop_id: u64,
        level_id: u8,
        name: Option<&[u8]>,
        threshold: Option<u128>,
        discount_rate: Option<u16>,
        commission_bonus: Option<u16>,
    ) -> sp_runtime::DispatchResult;

    /// 删除自定义等级（治理调用）
    fn remove_custom_level(shop_id: u64, level_id: u8) -> sp_runtime::DispatchResult;

    /// 获取自定义等级数量
    fn custom_level_count(shop_id: u64) -> u8;
}

/// 空实现
pub struct NullMemberProvider;

impl<AccountId> MemberProvider<AccountId> for NullMemberProvider {
    fn get_referrer(_shop_id: u64, _account: &AccountId) -> Option<AccountId> { None }
    fn member_level(_shop_id: u64, _account: &AccountId) -> Option<pallet_entity_common::MemberLevel> { None }
    fn get_member_stats(_shop_id: u64, _account: &AccountId) -> (u32, u32, u128) { (0, 0, 0) }
    fn uses_custom_levels(_shop_id: u64) -> bool { false }
    fn custom_level_id(_shop_id: u64, _account: &AccountId) -> u8 { 0 }

    fn set_custom_levels_enabled(_shop_id: u64, _enabled: bool) -> sp_runtime::DispatchResult { Ok(()) }
    fn set_upgrade_mode(_shop_id: u64, _mode: u8) -> sp_runtime::DispatchResult { Ok(()) }
    fn add_custom_level(
        _shop_id: u64,
        _level_id: u8,
        _name: &[u8],
        _threshold: u128,
        _discount_rate: u16,
        _commission_bonus: u16,
    ) -> sp_runtime::DispatchResult { Ok(()) }
    fn update_custom_level(
        _shop_id: u64,
        _level_id: u8,
        _name: Option<&[u8]>,
        _threshold: Option<u128>,
        _discount_rate: Option<u16>,
        _commission_bonus: Option<u16>,
    ) -> sp_runtime::DispatchResult { Ok(()) }
    fn remove_custom_level(_shop_id: u64, _level_id: u8) -> sp_runtime::DispatchResult { Ok(()) }
    fn custom_level_count(_shop_id: u64) -> u8 { 0 }
}

// ============================================================================
// CommissionProvider Trait（供其他模块调用）
// ============================================================================

/// 返佣服务接口
pub trait CommissionProvider<AccountId, Balance> {
    /// 处理订单返佣
    fn process_commission(
        shop_id: u64,
        order_id: u64,
        buyer: &AccountId,
        order_amount: Balance,
        available_pool: Balance,
    ) -> sp_runtime::DispatchResult;

    /// 取消订单返佣
    fn cancel_commission(order_id: u64) -> sp_runtime::DispatchResult;

    /// 获取待提取返佣
    fn pending_commission(shop_id: u64, account: &AccountId) -> Balance;

    // ==================== 治理调用接口（新增）====================

    /// 设置返佣模式（治理调用）
    fn set_commission_modes(shop_id: u64, modes: u16) -> sp_runtime::DispatchResult;

    /// 设置直推奖励率（治理调用）
    fn set_direct_reward_rate(shop_id: u64, rate: u16) -> sp_runtime::DispatchResult;

    /// 设置等级差价配置（治理调用）
    fn set_level_diff_config(
        shop_id: u64,
        normal_rate: u16,
        silver_rate: u16,
        gold_rate: u16,
        platinum_rate: u16,
        diamond_rate: u16,
    ) -> sp_runtime::DispatchResult;

    /// 设置固定金额配置（治理调用）
    fn set_fixed_amount(shop_id: u64, amount: Balance) -> sp_runtime::DispatchResult;

    /// 设置首单奖励配置（治理调用）
    fn set_first_order_config(
        shop_id: u64,
        amount: Balance,
        rate: u16,
        use_amount: bool,
    ) -> sp_runtime::DispatchResult;

    /// 设置复购奖励配置（治理调用）
    fn set_repeat_purchase_config(shop_id: u64, rate: u16, min_orders: u32) -> sp_runtime::DispatchResult;

    /// 设置分级提现配置（治理调用）
    fn set_withdrawal_config_by_governance(
        shop_id: u64,
        enabled: bool,
        shopping_balance_generates_commission: bool,
    ) -> sp_runtime::DispatchResult;

    /// 获取购物余额
    fn shopping_balance(shop_id: u64, account: &AccountId) -> Balance;
}

/// CommissionProvider 实现
impl<T: pallet::Config> CommissionProvider<T::AccountId, pallet::BalanceOf<T>> for pallet::Pallet<T> {
    fn process_commission(
        shop_id: u64,
        order_id: u64,
        buyer: &T::AccountId,
        order_amount: pallet::BalanceOf<T>,
        available_pool: pallet::BalanceOf<T>,
    ) -> sp_runtime::DispatchResult {
        pallet::Pallet::<T>::process_commission(shop_id, order_id, buyer, order_amount, available_pool)
    }

    fn cancel_commission(order_id: u64) -> sp_runtime::DispatchResult {
        pallet::Pallet::<T>::cancel_commission(order_id)
    }

    fn pending_commission(shop_id: u64, account: &T::AccountId) -> pallet::BalanceOf<T> {
        pallet::MemberCommissionStats::<T>::get(shop_id, account).pending
    }

    fn set_commission_modes(shop_id: u64, modes: u16) -> sp_runtime::DispatchResult {
        pallet::ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
            let config = maybe_config.get_or_insert_with(pallet::ShopCommissionConfig::default);
            config.enabled_modes = pallet::CommissionModes(modes);
        });
        Ok(())
    }

    fn set_direct_reward_rate(shop_id: u64, rate: u16) -> sp_runtime::DispatchResult {
        pallet::ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
            let config = maybe_config.get_or_insert_with(pallet::ShopCommissionConfig::default);
            config.direct_reward.rate = rate;
        });
        Ok(())
    }

    fn set_level_diff_config(
        shop_id: u64,
        normal_rate: u16,
        silver_rate: u16,
        gold_rate: u16,
        platinum_rate: u16,
        diamond_rate: u16,
    ) -> sp_runtime::DispatchResult {
        pallet::ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
            let config = maybe_config.get_or_insert_with(pallet::ShopCommissionConfig::default);
            config.level_diff.normal_rate = normal_rate;
            config.level_diff.silver_rate = silver_rate;
            config.level_diff.gold_rate = gold_rate;
            config.level_diff.platinum_rate = platinum_rate;
            config.level_diff.diamond_rate = diamond_rate;
        });
        Ok(())
    }

    fn set_fixed_amount(shop_id: u64, amount: pallet::BalanceOf<T>) -> sp_runtime::DispatchResult {
        pallet::ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
            let config = maybe_config.get_or_insert_with(pallet::ShopCommissionConfig::default);
            config.fixed_amount.amount = amount;
        });
        Ok(())
    }

    fn set_first_order_config(
        shop_id: u64,
        amount: pallet::BalanceOf<T>,
        rate: u16,
        use_amount: bool,
    ) -> sp_runtime::DispatchResult {
        pallet::ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
            let config = maybe_config.get_or_insert_with(pallet::ShopCommissionConfig::default);
            config.first_order.amount = amount;
            config.first_order.rate = rate;
            config.first_order.use_amount = use_amount;
        });
        Ok(())
    }

    fn set_repeat_purchase_config(shop_id: u64, rate: u16, min_orders: u32) -> sp_runtime::DispatchResult {
        pallet::ShopCommissionConfigs::<T>::mutate(shop_id, |maybe_config| {
            let config = maybe_config.get_or_insert_with(pallet::ShopCommissionConfig::default);
            config.repeat_purchase.rate = rate;
            config.repeat_purchase.min_orders = min_orders;
        });
        Ok(())
    }

    fn set_withdrawal_config_by_governance(
        shop_id: u64,
        enabled: bool,
        shopping_balance_generates_commission: bool,
    ) -> sp_runtime::DispatchResult {
        pallet::ShopWithdrawalConfigs::<T>::mutate(shop_id, |maybe_config| {
            let config = maybe_config.get_or_insert_with(pallet::ShopWithdrawalConfig::default);
            config.enabled = enabled;
            config.shopping_balance_generates_commission = shopping_balance_generates_commission;
        });
        Ok(())
    }

    fn shopping_balance(shop_id: u64, account: &T::AccountId) -> pallet::BalanceOf<T> {
        pallet::MemberShoppingBalance::<T>::get(shop_id, account)
    }
}

/// 空实现
pub struct NullCommissionProvider;

impl<AccountId, Balance: Default> CommissionProvider<AccountId, Balance> for NullCommissionProvider {
    fn process_commission(
        _shop_id: u64,
        _order_id: u64,
        _buyer: &AccountId,
        _order_amount: Balance,
        _available_pool: Balance,
    ) -> sp_runtime::DispatchResult { Ok(()) }

    fn cancel_commission(_order_id: u64) -> sp_runtime::DispatchResult { Ok(()) }

    fn pending_commission(_shop_id: u64, _account: &AccountId) -> Balance { Balance::default() }

    fn set_commission_modes(_shop_id: u64, _modes: u16) -> sp_runtime::DispatchResult { Ok(()) }

    fn set_direct_reward_rate(_shop_id: u64, _rate: u16) -> sp_runtime::DispatchResult { Ok(()) }

    fn set_level_diff_config(
        _shop_id: u64,
        _normal_rate: u16,
        _silver_rate: u16,
        _gold_rate: u16,
        _platinum_rate: u16,
        _diamond_rate: u16,
    ) -> sp_runtime::DispatchResult { Ok(()) }

    fn set_fixed_amount(_shop_id: u64, _amount: Balance) -> sp_runtime::DispatchResult { Ok(()) }

    fn set_first_order_config(
        _shop_id: u64,
        _amount: Balance,
        _rate: u16,
        _use_amount: bool,
    ) -> sp_runtime::DispatchResult { Ok(()) }

    fn set_repeat_purchase_config(_shop_id: u64, _rate: u16, _min_orders: u32) -> sp_runtime::DispatchResult { Ok(()) }

    fn set_withdrawal_config_by_governance(
        _shop_id: u64,
        _enabled: bool,
        _shopping_balance_generates_commission: bool,
    ) -> sp_runtime::DispatchResult { Ok(()) }

    fn shopping_balance(_shop_id: u64, _account: &AccountId) -> Balance { Balance::default() }
}
