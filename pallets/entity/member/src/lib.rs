//! # Entity 会员管理模块 (pallet-entity-member)
//!
//! ## 概述
//!
//! 本模块实现店铺会员推荐关系管理，支持：
//! - 每个店铺独立的会员体系
//! - 三级分销推荐返佣
//! - 会员等级管理
//! - 推荐统计查询
//!
//! ## 版本历史
//!
//! - v0.1.0: 初始版本，实现基础会员推荐关系

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use alloc::vec::Vec;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, Get},
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use pallet_entity_common::{EntityProvider, MemberMode};
    use sp_runtime::traits::{Saturating, Zero};

    // ============================================================================
    // Entity-Shop 分离架构：会员范围
    // ============================================================================

    /// 会员范围（Entity 级别或 Shop 级别）
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub enum MemberScope {
        /// Entity 级别（所有 Shop 共享会员）
        #[default]
        Entity,
        /// Shop 级别（各 Shop 独立会员）
        Shop,
    }

    /// 货币余额类型别名
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // ============================================================================
    // 数据结构
    // ============================================================================

    /// 会员等级
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub enum MemberLevel {
        #[default]
        Normal,     // 普通会员
        Silver,     // 银卡会员
        Gold,       // 金卡会员
        Platinum,   // 白金会员
        Diamond,    // 钻石会员
    }

    /// 返佣来源
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub enum CommissionSource {
        #[default]
        PlatformFee,    // 从平台费中扣除
        ShopFund,       // 店铺运营资金承担
        Mixed,          // 混合模式
    }

    /// 等级升级方式
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub enum LevelUpgradeMode {
        #[default]
        AutoUpgrade,      // 自动升级（消费达标即升）
        ManualUpgrade,    // 手动升级（需店主审批）
        PeriodReset,      // 周期重置（每月/每年重新计算）
    }

    /// 自定义会员等级
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct CustomLevel<Balance> {
        /// 等级 ID（0, 1, 2, ...）
        pub id: u8,
        /// 等级名称（如 "VIP", "黑卡"）
        pub name: BoundedVec<u8, ConstU32<32>>,
        /// 升级阈值（累计消费）
        pub threshold: Balance,
        /// 折扣率（基点，500 = 5% 折扣）
        pub discount_rate: u16,
        /// 返佣加成（基点，100 = 1% 额外返佣）
        pub commission_bonus: u16,
    }

    /// 自定义等级类型别名
    pub type CustomLevelOf<T> = CustomLevel<BalanceOf<T>>;

    /// 实体会员等级系统配置
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    #[scale_info(skip_type_params(MaxLevels))]
    pub struct EntityLevelSystem<Balance, MaxLevels: Get<u32>> {
        /// 自定义等级列表（按阈值升序排列）
        pub levels: BoundedVec<CustomLevel<Balance>, MaxLevels>,
        /// 是否启用自定义等级（false 则使用全局默认）
        pub use_custom: bool,
        /// 等级升级方式
        pub upgrade_mode: LevelUpgradeMode,
    }

    impl<Balance: Default, MaxLevels: Get<u32>> Default for EntityLevelSystem<Balance, MaxLevels> {
        fn default() -> Self {
            Self {
                levels: BoundedVec::default(),
                use_custom: false,
                upgrade_mode: LevelUpgradeMode::AutoUpgrade,
            }
        }
    }

    /// 实体等级系统类型别名
    pub type EntityLevelSystemOf<T> = EntityLevelSystem<BalanceOf<T>, <T as Config>::MaxCustomLevels>;

    // ============================================================================
    // 升级规则相关数据结构
    // ============================================================================

    /// 规则冲突处理策略
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub enum ConflictStrategy {
        #[default]
        HighestLevel,     // 取最高等级
        HighestPriority,  // 取最高优先级规则
        LongestDuration,  // 取最长有效期
        FirstMatch,       // 第一个匹配的规则
    }

    /// 升级触发条件
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub enum UpgradeTrigger<Balance> {
        /// 购买特定产品
        PurchaseProduct {
            product_id: u64,
        },
        /// 累计消费达标
        TotalSpent {
            threshold: Balance,
        },
        /// 单笔消费达标
        SingleOrder {
            threshold: Balance,
        },
        /// 推荐人数达标
        ReferralCount {
            count: u32,
        },
        /// 团队人数达标
        TeamSize {
            size: u32,
        },
        /// 订单数量达标
        OrderCount {
            count: u32,
        },
    }

    /// 升级触发条件类型别名
    pub type UpgradeTriggerOf<T> = UpgradeTrigger<BalanceOf<T>>;

    /// 升级规则
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct UpgradeRule<Balance, BlockNumber> {
        /// 规则 ID
        pub id: u32,
        /// 规则名称
        pub name: BoundedVec<u8, ConstU32<64>>,
        /// 触发条件
        pub trigger: UpgradeTrigger<Balance>,
        /// 目标等级 ID
        pub target_level_id: u8,
        /// 有效期（区块数，None 表示永久）
        pub duration: Option<BlockNumber>,
        /// 是否启用
        pub enabled: bool,
        /// 优先级（数值越大优先级越高）
        pub priority: u8,
        /// 是否可叠加（多次触发是否延长有效期）
        pub stackable: bool,
        /// 最大触发次数（None 表示无限制）
        pub max_triggers: Option<u32>,
        /// 已触发次数
        pub trigger_count: u32,
    }

    /// 升级规则类型别名
    pub type UpgradeRuleOf<T> = UpgradeRule<BalanceOf<T>, BlockNumberFor<T>>;

    /// 实体升级规则系统
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    #[scale_info(skip_type_params(MaxRules))]
    pub struct EntityUpgradeRuleSystem<Balance, BlockNumber, MaxRules: Get<u32>> {
        /// 升级规则列表
        pub rules: BoundedVec<UpgradeRule<Balance, BlockNumber>, MaxRules>,
        /// 下一个规则 ID
        pub next_rule_id: u32,
        /// 是否启用规则系统
        pub enabled: bool,
        /// 规则冲突处理策略
        pub conflict_strategy: ConflictStrategy,
    }

    impl<Balance, BlockNumber, MaxRules: Get<u32>> Default for EntityUpgradeRuleSystem<Balance, BlockNumber, MaxRules> {
        fn default() -> Self {
            Self {
                rules: BoundedVec::default(),
                next_rule_id: 0,
                enabled: false,
                conflict_strategy: ConflictStrategy::HighestLevel,
            }
        }
    }

    /// 实体升级规则系统类型别名
    pub type EntityUpgradeRuleSystemOf<T> = EntityUpgradeRuleSystem<BalanceOf<T>, BlockNumberFor<T>, <T as Config>::MaxUpgradeRules>;

    /// 升级记录
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct UpgradeRecord<BlockNumber> {
        /// 触发的规则 ID
        pub rule_id: u32,
        /// 升级前等级
        pub from_level_id: u8,
        /// 升级后等级
        pub to_level_id: u8,
        /// 升级时间
        pub upgraded_at: BlockNumber,
        /// 过期时间
        pub expires_at: Option<BlockNumber>,
    }

    /// 升级记录类型别名
    pub type UpgradeRecordOf<T> = UpgradeRecord<BlockNumberFor<T>>;

    /// 实体会员信息
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct EntityMember<AccountId, Balance, BlockNumber> {
        /// 推荐人（上级）
        pub referrer: Option<AccountId>,
        /// 直接推荐人数
        pub direct_referrals: u32,
        /// 团队总人数
        pub team_size: u32,
        /// 累计消费金额
        pub total_spent: Balance,
        /// 累计获得返佣
        pub total_commission: Balance,
        /// 累计贡献返佣（下级消费产生的返佣）
        pub total_contributed: Balance,
        /// 待提取返佣
        pub pending_commission: Balance,
        /// 会员等级（全局默认体系）
        pub level: MemberLevel,
        /// 自定义等级 ID（店铺自定义体系，0 表示最低级）
        pub custom_level_id: u8,
        /// 加入时间
        pub joined_at: BlockNumber,
        /// 最后活跃时间
        pub last_active_at: BlockNumber,
        /// 周期消费（用于 PeriodReset 模式）
        pub period_spent: Balance,
        /// 周期开始时间
        pub period_start: BlockNumber,
    }

    /// 实体会员类型别名
    pub type EntityMemberOf<T> = EntityMember<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
    >;

    /// 返佣配置
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct CommissionConfig {
        /// 一级返佣率（基点，500 = 5%）
        pub level1_rate: u16,
        /// 二级返佣率（基点，200 = 2%）
        pub level2_rate: u16,
        /// 三级返佣率（基点，100 = 1%）
        pub level3_rate: u16,
        /// 返佣来源
        pub source: CommissionSource,
        /// 是否启用
        pub enabled: bool,
    }

    impl Default for CommissionConfig {
        fn default() -> Self {
            Self {
                level1_rate: 500,   // 5%
                level2_rate: 200,   // 2%
                level3_rate: 100,   // 1%
                source: CommissionSource::PlatformFee,
                enabled: true,
            }
        }
    }

    // ============================================================================
    // Pallet 配置
    // ============================================================================

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 货币类型
        type Currency: Currency<Self::AccountId>;

        /// 实体查询接口
        type EntityProvider: EntityProvider<Self::AccountId>;

        /// Shop 查询接口（Entity-Shop 分离架构）
        type ShopProvider: pallet_entity_common::ShopProvider<Self::AccountId>;

        /// 最大直接推荐人数
        #[pallet::constant]
        type MaxDirectReferrals: Get<u32>;

        /// 最大自定义等级数量
        #[pallet::constant]
        type MaxCustomLevels: Get<u32>;

        /// 银卡会员消费阈值（USDT，6位精度）
        #[pallet::constant]
        type SilverThreshold: Get<u64>;

        /// 金卡会员消费阈值（USDT，6位精度）
        #[pallet::constant]
        type GoldThreshold: Get<u64>;

        /// 白金会员消费阈值（USDT，6位精度）
        #[pallet::constant]
        type PlatinumThreshold: Get<u64>;

        /// 钻石会员消费阈值（USDT，6位精度）
        #[pallet::constant]
        type DiamondThreshold: Get<u64>;

        /// 最大升级规则数量
        #[pallet::constant]
        type MaxUpgradeRules: Get<u32>;

        /// 最大升级历史记录数量
        #[pallet::constant]
        type MaxUpgradeHistory: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ============================================================================
    // 存储项
    // ============================================================================

    /// 实体会员存储 (entity_id, account) -> EntityMember
    #[pallet::storage]
    #[pallet::getter(fn entity_members)]
    pub type EntityMembers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u64,
        Blake2_128Concat, T::AccountId,
        EntityMemberOf<T>,
    >;

    /// 店铺会员数量 shop_id -> count
    #[pallet::storage]
    #[pallet::getter(fn member_count)]
    pub type MemberCount<T: Config> = StorageMap<_, Blake2_128Concat, u64, u32, ValueQuery>;

    /// 推荐关系索引 (shop_id, referrer) -> Vec<AccountId>
    #[pallet::storage]
    #[pallet::getter(fn direct_referrals)]
    pub type DirectReferrals<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u64,
        Blake2_128Concat, T::AccountId,
        BoundedVec<T::AccountId, T::MaxDirectReferrals>,
        ValueQuery,
    >;

    /// 店铺返佣配置 shop_id -> CommissionConfig
    #[pallet::storage]
    #[pallet::getter(fn shop_commission_config)]
    pub type ShopCommissionConfig<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        CommissionConfig,
    >;

    /// 店铺返佣统计 shop_id -> (total_commission, total_orders)
    #[pallet::storage]
    #[pallet::getter(fn shop_commission_stats)]
    pub type ShopCommissionStats<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        (BalanceOf<T>, u64),
        ValueQuery,
    >;

    /// 实体等级系统配置 entity_id -> EntityLevelSystem
    #[pallet::storage]
    #[pallet::getter(fn entity_level_system)]
    pub type EntityLevelSystems<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        EntityLevelSystemOf<T>,
    >;

    /// 实体升级规则系统 entity_id -> EntityUpgradeRuleSystem
    #[pallet::storage]
    #[pallet::getter(fn entity_upgrade_rules)]
    pub type EntityUpgradeRules<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        EntityUpgradeRuleSystemOf<T>,
    >;

    /// 会员等级过期时间 (shop_id, account) -> expires_at
    #[pallet::storage]
    #[pallet::getter(fn member_level_expiry)]
    pub type MemberLevelExpiry<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u64,
        Blake2_128Concat, T::AccountId,
        BlockNumberFor<T>,
    >;

    /// 会员升级历史 (shop_id, account) -> Vec<UpgradeRecord>
    #[pallet::storage]
    #[pallet::getter(fn member_upgrade_history)]
    pub type MemberUpgradeHistory<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u64,
        Blake2_128Concat, T::AccountId,
        BoundedVec<UpgradeRecordOf<T>, T::MaxUpgradeHistory>,
        ValueQuery,
    >;

    /// 会员订单数量 (shop_id, account) -> order_count
    #[pallet::storage]
    #[pallet::getter(fn member_order_count)]
    pub type MemberOrderCount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u64,
        Blake2_128Concat, T::AccountId,
        u32,
        ValueQuery,
    >;

    // ============================================================================
    // Entity-Shop 分离架构：Shop 级别会员存储
    // ============================================================================

    /// Shop 级别会员存储 (shop_id, account) -> EntityMember
    /// 当 MemberMode 为 Independent 或 Hybrid 时使用
    #[pallet::storage]
    #[pallet::getter(fn shop_members)]
    pub type ShopMembers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u64,
        Blake2_128Concat, T::AccountId,
        EntityMemberOf<T>,
    >;

    /// Shop 会员数量 shop_id -> count
    #[pallet::storage]
    #[pallet::getter(fn shop_member_count)]
    pub type ShopMemberCount<T: Config> = StorageMap<_, Blake2_128Concat, u64, u32, ValueQuery>;

    // ============================================================================
    // 事件
    // ============================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 会员注册
        MemberRegistered {
            shop_id: u64,
            account: T::AccountId,
            referrer: Option<T::AccountId>,
        },
        /// 绑定推荐人
        ReferrerBound {
            shop_id: u64,
            account: T::AccountId,
            referrer: T::AccountId,
        },
        /// 返佣发放
        CommissionDistributed {
            shop_id: u64,
            referrer: T::AccountId,
            amount: BalanceOf<T>,
            level: u8,
        },
        /// 返佣提取
        CommissionWithdrawn {
            shop_id: u64,
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 会员升级
        MemberLevelUpgraded {
            shop_id: u64,
            account: T::AccountId,
            old_level: MemberLevel,
            new_level: MemberLevel,
        },
        /// 自定义等级升级
        CustomLevelUpgraded {
            shop_id: u64,
            account: T::AccountId,
            old_level_id: u8,
            new_level_id: u8,
        },
        /// 返佣配置更新
        CommissionConfigUpdated {
            shop_id: u64,
            config: CommissionConfig,
        },
        /// 等级系统初始化
        LevelSystemInitialized {
            shop_id: u64,
            use_custom: bool,
            upgrade_mode: LevelUpgradeMode,
        },
        /// 自定义等级添加
        CustomLevelAdded {
            shop_id: u64,
            level_id: u8,
            name: BoundedVec<u8, ConstU32<32>>,
            threshold: BalanceOf<T>,
        },
        /// 自定义等级更新
        CustomLevelUpdated {
            shop_id: u64,
            level_id: u8,
        },
        /// 自定义等级删除
        CustomLevelRemoved {
            shop_id: u64,
            level_id: u8,
        },
        /// 手动升级会员
        MemberManuallyUpgraded {
            shop_id: u64,
            account: T::AccountId,
            level_id: u8,
        },
        /// 升级规则系统初始化
        UpgradeRuleSystemInitialized {
            shop_id: u64,
            conflict_strategy: ConflictStrategy,
        },
        /// 升级规则添加
        UpgradeRuleAdded {
            shop_id: u64,
            rule_id: u32,
            name: BoundedVec<u8, ConstU32<64>>,
            target_level_id: u8,
        },
        /// 升级规则更新
        UpgradeRuleUpdated {
            shop_id: u64,
            rule_id: u32,
        },
        /// 升级规则删除
        UpgradeRuleRemoved {
            shop_id: u64,
            rule_id: u32,
        },
        /// 会员通过规则升级
        MemberUpgradedByRule {
            shop_id: u64,
            account: T::AccountId,
            rule_id: u32,
            from_level_id: u8,
            to_level_id: u8,
            expires_at: Option<BlockNumberFor<T>>,
        },
        /// 会员等级过期
        MemberLevelExpired {
            shop_id: u64,
            account: T::AccountId,
            expired_level_id: u8,
            new_level_id: u8,
        },
    }

    // ============================================================================
    // 错误
    // ============================================================================

    #[pallet::error]
    pub enum Error<T> {
        /// 已是会员
        AlreadyMember,
        /// 不是会员
        NotMember,
        /// 已绑定推荐人
        ReferrerAlreadyBound,
        /// 无效推荐人
        InvalidReferrer,
        /// 不能推荐自己
        SelfReferral,
        /// 循环推荐
        CircularReferral,
        /// 返佣未配置
        CommissionNotConfigured,
        /// 返佣余额不足
        InsufficientCommission,
        /// 不是店主
        NotShopOwner,
        /// 店铺不存在
        ShopNotFound,
        /// 推荐人数已满
        ReferralsFull,
        /// 数值溢出
        Overflow,
        /// 等级系统未初始化
        LevelSystemNotInitialized,
        /// 等级已存在
        LevelAlreadyExists,
        /// 等级不存在
        LevelNotFound,
        /// 等级数量已满
        LevelsFull,
        /// 无效等级 ID
        InvalidLevelId,
        /// 等级阈值无效（必须大于前一等级）
        InvalidThreshold,
        /// 等级名称为空
        EmptyLevelName,
        /// 不支持手动升级
        ManualUpgradeNotSupported,
        /// 等级有会员，无法删除
        LevelHasMembers,
        /// 升级规则系统未初始化
        UpgradeRuleSystemNotInitialized,
        /// 升级规则不存在
        UpgradeRuleNotFound,
        /// 升级规则数量已满
        UpgradeRulesFull,
        /// 规则名称为空
        EmptyRuleName,
        /// 无效目标等级
        InvalidTargetLevel,
    }

    // ============================================================================
    // Extrinsics
    // ============================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 注册成为店铺会员
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `referrer`: 推荐人（可选）
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn register_member(
            origin: OriginFor<T>,
            shop_id: u64,
            referrer: Option<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证店铺存在
            ensure!(T::EntityProvider::entity_exists(shop_id), Error::<T>::ShopNotFound);

            // 验证未注册
            ensure!(
                !EntityMembers::<T>::contains_key(shop_id, &who),
                Error::<T>::AlreadyMember
            );

            // 验证推荐人
            if let Some(ref ref_account) = referrer {
                ensure!(ref_account != &who, Error::<T>::SelfReferral);
                ensure!(
                    EntityMembers::<T>::contains_key(shop_id, ref_account),
                    Error::<T>::InvalidReferrer
                );
            }

            Self::do_register_member(shop_id, &who, referrer.clone())?;

            Self::deposit_event(Event::MemberRegistered {
                shop_id,
                account: who,
                referrer,
            });

            Ok(())
        }

        /// 绑定推荐人（未绑定过的会员）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `referrer`: 推荐人账户
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn bind_referrer(
            origin: OriginFor<T>,
            shop_id: u64,
            referrer: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证是会员
            let mut member = EntityMembers::<T>::get(shop_id, &who)
                .ok_or(Error::<T>::NotMember)?;

            // 验证未绑定推荐人
            ensure!(member.referrer.is_none(), Error::<T>::ReferrerAlreadyBound);

            // 验证推荐人
            ensure!(referrer != who, Error::<T>::SelfReferral);
            ensure!(
                EntityMembers::<T>::contains_key(shop_id, &referrer),
                Error::<T>::InvalidReferrer
            );

            // 检查循环推荐
            ensure!(
                !Self::is_circular_referral(shop_id, &who, &referrer),
                Error::<T>::CircularReferral
            );

            // 绑定推荐人
            member.referrer = Some(referrer.clone());
            EntityMembers::<T>::insert(shop_id, &who, member);

            // 更新推荐人的直接推荐人数
            EntityMembers::<T>::mutate(shop_id, &referrer, |maybe_member| {
                if let Some(ref mut m) = maybe_member {
                    m.direct_referrals = m.direct_referrals.saturating_add(1);
                }
            });

            // 更新推荐索引
            DirectReferrals::<T>::try_mutate(shop_id, &referrer, |referrals| {
                referrals.try_push(who.clone()).map_err(|_| Error::<T>::ReferralsFull)
            })?;

            // 更新团队人数
            Self::update_team_size(shop_id, &referrer);

            Self::deposit_event(Event::ReferrerBound {
                shop_id,
                account: who,
                referrer,
            });

            Ok(())
        }

        /// 设置店铺返佣配置（店主）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `level1_rate`: 一级返佣率（基点）
        /// - `level2_rate`: 二级返佣率（基点）
        /// - `level3_rate`: 三级返佣率（基点）
        /// - `source`: 返佣来源
        /// - `enabled`: 是否启用
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn set_commission_config(
            origin: OriginFor<T>,
            shop_id: u64,
            level1_rate: u16,
            level2_rate: u16,
            level3_rate: u16,
            source: CommissionSource,
            enabled: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证是店主
            let owner = T::EntityProvider::entity_owner(shop_id)
                .ok_or(Error::<T>::ShopNotFound)?;
            ensure!(who == owner, Error::<T>::NotShopOwner);

            let config = CommissionConfig {
                level1_rate,
                level2_rate,
                level3_rate,
                source,
                enabled,
            };

            ShopCommissionConfig::<T>::insert(shop_id, config.clone());

            Self::deposit_event(Event::CommissionConfigUpdated { shop_id, config });

            Ok(())
        }

        /// 提取返佣
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `amount`: 提取金额（None 表示全部提取）
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(35_000, 0))]
        pub fn withdraw_commission(
            origin: OriginFor<T>,
            shop_id: u64,
            amount: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            EntityMembers::<T>::try_mutate(shop_id, &who, |maybe_member| -> DispatchResult {
                let member = maybe_member.as_mut().ok_or(Error::<T>::NotMember)?;

                let withdraw_amount = amount.unwrap_or(member.pending_commission);
                ensure!(
                    member.pending_commission >= withdraw_amount,
                    Error::<T>::InsufficientCommission
                );

                // 从店铺派生账户转账给会员
                let shop_account = T::EntityProvider::entity_account(shop_id);

                T::Currency::transfer(
                    &shop_account,
                    &who,
                    withdraw_amount,
                    ExistenceRequirement::KeepAlive,
                )?;

                member.pending_commission = member.pending_commission.saturating_sub(withdraw_amount);

                Self::deposit_event(Event::CommissionWithdrawn {
                    shop_id,
                    account: who.clone(),
                    amount: withdraw_amount,
                });

                Ok(())
            })
        }

        /// 初始化店铺等级系统
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `use_custom`: 是否使用自定义等级
        /// - `upgrade_mode`: 等级升级方式
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn init_level_system(
            origin: OriginFor<T>,
            shop_id: u64,
            use_custom: bool,
            upgrade_mode: LevelUpgradeMode,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            let system = EntityLevelSystem {
                levels: BoundedVec::default(),
                use_custom,
                upgrade_mode,
            };

            EntityLevelSystems::<T>::insert(shop_id, system);

            Self::deposit_event(Event::LevelSystemInitialized {
                shop_id,
                use_custom,
                upgrade_mode,
            });

            Ok(())
        }

        /// 添加自定义等级
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `name`: 等级名称
        /// - `threshold`: 升级阈值
        /// - `discount_rate`: 折扣率（基点）
        /// - `commission_bonus`: 返佣加成（基点）
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn add_custom_level(
            origin: OriginFor<T>,
            shop_id: u64,
            name: BoundedVec<u8, ConstU32<32>>,
            threshold: BalanceOf<T>,
            discount_rate: u16,
            commission_bonus: u16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            ensure!(!name.is_empty(), Error::<T>::EmptyLevelName);

            EntityLevelSystems::<T>::try_mutate(shop_id, |maybe_system| -> DispatchResult {
                let system = maybe_system.as_mut().ok_or(Error::<T>::LevelSystemNotInitialized)?;

                // 验证阈值必须大于最后一个等级
                if let Some(last) = system.levels.last() {
                    ensure!(threshold > last.threshold, Error::<T>::InvalidThreshold);
                }

                let level_id = system.levels.len() as u8;

                let level = CustomLevel {
                    id: level_id,
                    name: name.clone(),
                    threshold,
                    discount_rate,
                    commission_bonus,
                };

                system.levels.try_push(level).map_err(|_| Error::<T>::LevelsFull)?;

                Self::deposit_event(Event::CustomLevelAdded {
                    shop_id,
                    level_id,
                    name,
                    threshold,
                });

                Ok(())
            })
        }

        /// 更新自定义等级
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `level_id`: 等级 ID
        /// - `name`: 新名称（可选）
        /// - `threshold`: 新阈值（可选）
        /// - `discount_rate`: 新折扣率（可选）
        /// - `commission_bonus`: 新返佣加成（可选）
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn update_custom_level(
            origin: OriginFor<T>,
            shop_id: u64,
            level_id: u8,
            name: Option<BoundedVec<u8, ConstU32<32>>>,
            threshold: Option<BalanceOf<T>>,
            discount_rate: Option<u16>,
            commission_bonus: Option<u16>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            EntityLevelSystems::<T>::try_mutate(shop_id, |maybe_system| -> DispatchResult {
                let system = maybe_system.as_mut().ok_or(Error::<T>::LevelSystemNotInitialized)?;

                ensure!((level_id as usize) < system.levels.len(), Error::<T>::LevelNotFound);

                // 验证阈值（先检查，再修改）
                if let Some(new_threshold) = threshold {
                    // 必须大于前一等级
                    if level_id > 0 {
                        if let Some(prev) = system.levels.get((level_id - 1) as usize) {
                            ensure!(new_threshold > prev.threshold, Error::<T>::InvalidThreshold);
                        }
                    }
                    // 必须小于后一等级
                    if let Some(next) = system.levels.get((level_id + 1) as usize) {
                        ensure!(new_threshold < next.threshold, Error::<T>::InvalidThreshold);
                    }
                }

                // 现在安全地获取可变引用并修改
                let level = system.levels.get_mut(level_id as usize)
                    .ok_or(Error::<T>::LevelNotFound)?;

                if let Some(new_threshold) = threshold {
                    level.threshold = new_threshold;
                }

                if let Some(new_name) = name {
                    ensure!(!new_name.is_empty(), Error::<T>::EmptyLevelName);
                    level.name = new_name;
                }

                if let Some(rate) = discount_rate {
                    level.discount_rate = rate;
                }

                if let Some(bonus) = commission_bonus {
                    level.commission_bonus = bonus;
                }

                Self::deposit_event(Event::CustomLevelUpdated { shop_id, level_id });

                Ok(())
            })
        }

        /// 删除自定义等级（只能删除最后一个等级）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `level_id`: 等级 ID
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn remove_custom_level(
            origin: OriginFor<T>,
            shop_id: u64,
            level_id: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            EntityLevelSystems::<T>::try_mutate(shop_id, |maybe_system| -> DispatchResult {
                let system = maybe_system.as_mut().ok_or(Error::<T>::LevelSystemNotInitialized)?;

                // 只能删除最后一个等级
                ensure!(
                    level_id as usize == system.levels.len().saturating_sub(1),
                    Error::<T>::InvalidLevelId
                );

                system.levels.pop();

                Self::deposit_event(Event::CustomLevelRemoved { shop_id, level_id });

                Ok(())
            })
        }

        /// 手动升级会员（仅 ManualUpgrade 模式）
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `member`: 会员账户
        /// - `target_level_id`: 目标等级 ID
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn manual_upgrade_member(
            origin: OriginFor<T>,
            shop_id: u64,
            member: T::AccountId,
            target_level_id: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            let system = EntityLevelSystems::<T>::get(shop_id)
                .ok_or(Error::<T>::LevelSystemNotInitialized)?;

            ensure!(
                system.upgrade_mode == LevelUpgradeMode::ManualUpgrade,
                Error::<T>::ManualUpgradeNotSupported
            );

            ensure!(
                (target_level_id as usize) < system.levels.len(),
                Error::<T>::InvalidLevelId
            );

            EntityMembers::<T>::try_mutate(shop_id, &member, |maybe_member| -> DispatchResult {
                let m = maybe_member.as_mut().ok_or(Error::<T>::NotMember)?;
                m.custom_level_id = target_level_id;

                Self::deposit_event(Event::MemberManuallyUpgraded {
                    shop_id,
                    account: member.clone(),
                    level_id: target_level_id,
                });

                Ok(())
            })
        }

        /// 切换等级系统模式
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `use_custom`: 是否使用自定义等级
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn set_use_custom_levels(
            origin: OriginFor<T>,
            shop_id: u64,
            use_custom: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            EntityLevelSystems::<T>::try_mutate(shop_id, |maybe_system| -> DispatchResult {
                let system = maybe_system.as_mut().ok_or(Error::<T>::LevelSystemNotInitialized)?;
                system.use_custom = use_custom;
                Ok(())
            })
        }

        /// 设置等级升级模式
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `upgrade_mode`: 升级模式
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn set_upgrade_mode(
            origin: OriginFor<T>,
            shop_id: u64,
            upgrade_mode: LevelUpgradeMode,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            EntityLevelSystems::<T>::try_mutate(shop_id, |maybe_system| -> DispatchResult {
                let system = maybe_system.as_mut().ok_or(Error::<T>::LevelSystemNotInitialized)?;
                system.upgrade_mode = upgrade_mode;
                Ok(())
            })
        }

        // ========================================================================
        // 升级规则相关 Extrinsics
        // ========================================================================

        /// 初始化升级规则系统
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `conflict_strategy`: 规则冲突处理策略
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn init_upgrade_rule_system(
            origin: OriginFor<T>,
            shop_id: u64,
            conflict_strategy: ConflictStrategy,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            let system = EntityUpgradeRuleSystem {
                rules: BoundedVec::default(),
                next_rule_id: 0,
                enabled: true,
                conflict_strategy,
            };

            EntityUpgradeRules::<T>::insert(shop_id, system);

            Self::deposit_event(Event::UpgradeRuleSystemInitialized {
                shop_id,
                conflict_strategy,
            });

            Ok(())
        }

        /// 添加升级规则
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `name`: 规则名称
        /// - `trigger`: 触发条件
        /// - `target_level_id`: 目标等级 ID
        /// - `duration`: 有效期（区块数，None 表示永久）
        /// - `priority`: 优先级
        /// - `stackable`: 是否可叠加
        /// - `max_triggers`: 最大触发次数
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn add_upgrade_rule(
            origin: OriginFor<T>,
            shop_id: u64,
            name: BoundedVec<u8, ConstU32<64>>,
            trigger: UpgradeTriggerOf<T>,
            target_level_id: u8,
            duration: Option<BlockNumberFor<T>>,
            priority: u8,
            stackable: bool,
            max_triggers: Option<u32>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            ensure!(!name.is_empty(), Error::<T>::EmptyRuleName);

            EntityUpgradeRules::<T>::try_mutate(shop_id, |maybe_system| -> DispatchResult {
                let system = maybe_system.as_mut().ok_or(Error::<T>::UpgradeRuleSystemNotInitialized)?;

                let rule_id = system.next_rule_id;
                system.next_rule_id = system.next_rule_id.saturating_add(1);

                let rule = UpgradeRule {
                    id: rule_id,
                    name: name.clone(),
                    trigger,
                    target_level_id,
                    duration,
                    enabled: true,
                    priority,
                    stackable,
                    max_triggers,
                    trigger_count: 0,
                };

                system.rules.try_push(rule).map_err(|_| Error::<T>::UpgradeRulesFull)?;

                Self::deposit_event(Event::UpgradeRuleAdded {
                    shop_id,
                    rule_id,
                    name,
                    target_level_id,
                });

                Ok(())
            })
        }

        /// 更新升级规则
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `rule_id`: 规则 ID
        /// - `enabled`: 是否启用
        /// - `priority`: 优先级
        #[pallet::call_index(13)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn update_upgrade_rule(
            origin: OriginFor<T>,
            shop_id: u64,
            rule_id: u32,
            enabled: Option<bool>,
            priority: Option<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            EntityUpgradeRules::<T>::try_mutate(shop_id, |maybe_system| -> DispatchResult {
                let system = maybe_system.as_mut().ok_or(Error::<T>::UpgradeRuleSystemNotInitialized)?;

                let rule = system.rules.iter_mut()
                    .find(|r| r.id == rule_id)
                    .ok_or(Error::<T>::UpgradeRuleNotFound)?;

                if let Some(e) = enabled {
                    rule.enabled = e;
                }

                if let Some(p) = priority {
                    rule.priority = p;
                }

                Self::deposit_event(Event::UpgradeRuleUpdated { shop_id, rule_id });

                Ok(())
            })
        }

        /// 删除升级规则
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `rule_id`: 规则 ID
        #[pallet::call_index(14)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn remove_upgrade_rule(
            origin: OriginFor<T>,
            shop_id: u64,
            rule_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            EntityUpgradeRules::<T>::try_mutate(shop_id, |maybe_system| -> DispatchResult {
                let system = maybe_system.as_mut().ok_or(Error::<T>::UpgradeRuleSystemNotInitialized)?;

                let pos = system.rules.iter()
                    .position(|r| r.id == rule_id)
                    .ok_or(Error::<T>::UpgradeRuleNotFound)?;

                system.rules.remove(pos);

                Self::deposit_event(Event::UpgradeRuleRemoved { shop_id, rule_id });

                Ok(())
            })
        }

        /// 设置升级规则系统启用状态
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `enabled`: 是否启用
        #[pallet::call_index(15)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn set_upgrade_rule_system_enabled(
            origin: OriginFor<T>,
            shop_id: u64,
            enabled: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            EntityUpgradeRules::<T>::try_mutate(shop_id, |maybe_system| -> DispatchResult {
                let system = maybe_system.as_mut().ok_or(Error::<T>::UpgradeRuleSystemNotInitialized)?;
                system.enabled = enabled;
                Ok(())
            })
        }

        /// 设置规则冲突策略
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `conflict_strategy`: 冲突策略
        #[pallet::call_index(16)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn set_conflict_strategy(
            origin: OriginFor<T>,
            shop_id: u64,
            conflict_strategy: ConflictStrategy,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            EntityUpgradeRules::<T>::try_mutate(shop_id, |maybe_system| -> DispatchResult {
                let system = maybe_system.as_mut().ok_or(Error::<T>::UpgradeRuleSystemNotInitialized)?;
                system.conflict_strategy = conflict_strategy;
                Ok(())
            })
        }
    }

    // ============================================================================
    // 内部函数
    // ============================================================================

    impl<T: Config> Pallet<T> {
        /// 注册会员内部实现
        fn do_register_member(
            shop_id: u64,
            account: &T::AccountId,
            referrer: Option<T::AccountId>,
        ) -> DispatchResult {
            let now = <frame_system::Pallet<T>>::block_number();

            let member = EntityMember {
                referrer: referrer.clone(),
                direct_referrals: 0,
                team_size: 0,
                total_spent: Zero::zero(),
                total_commission: Zero::zero(),
                total_contributed: Zero::zero(),
                pending_commission: Zero::zero(),
                level: MemberLevel::Normal,
                custom_level_id: 0,
                joined_at: now,
                last_active_at: now,
                period_spent: Zero::zero(),
                period_start: now,
            };

            EntityMembers::<T>::insert(shop_id, account, member);
            MemberCount::<T>::mutate(shop_id, |count| *count = count.saturating_add(1));

            // 更新推荐人统计
            if let Some(ref ref_account) = referrer {
                EntityMembers::<T>::mutate(shop_id, ref_account, |maybe_member| {
                    if let Some(ref mut m) = maybe_member {
                        m.direct_referrals = m.direct_referrals.saturating_add(1);
                    }
                });

                // 更新推荐索引
                let _ = DirectReferrals::<T>::try_mutate(shop_id, ref_account, |referrals| {
                    referrals.try_push(account.clone())
                });

                // 更新团队人数
                Self::update_team_size(shop_id, ref_account);
            }

            Ok(())
        }

        /// 检查是否存在循环推荐
        /// P2 安全修复: 增加已访问集合检测，防止无限循环
        fn is_circular_referral(
            shop_id: u64,
            account: &T::AccountId,
            referrer: &T::AccountId,
        ) -> bool {
            use alloc::collections::BTreeSet;
            
            let mut current = Some(referrer.clone());
            let mut depth = 0u32;
            let mut visited = BTreeSet::new();
            const MAX_DEPTH: u32 = 100;

            while let Some(ref curr_account) = current {
                // 检查是否回到了要绑定的账户
                if curr_account == account {
                    return true;
                }
                
                // 检查是否已访问过（检测链中的其他循环）
                if visited.contains(curr_account) {
                    // 链中存在循环，但不涉及 account，安全
                    break;
                }
                visited.insert(curr_account.clone());
                
                if depth >= MAX_DEPTH {
                    break;
                }
                current = EntityMembers::<T>::get(shop_id, curr_account)
                    .and_then(|m| m.referrer);
                depth += 1;
            }

            false
        }

        /// 更新团队人数（递归向上更新）
        fn update_team_size(shop_id: u64, account: &T::AccountId) {
            let mut current = Some(account.clone());
            let mut depth = 0u32;
            const MAX_DEPTH: u32 = 100;

            while let Some(ref curr_account) = current {
                if depth >= MAX_DEPTH {
                    break;
                }

                EntityMembers::<T>::mutate(shop_id, curr_account, |maybe_member| {
                    if let Some(ref mut m) = maybe_member {
                        m.team_size = m.team_size.saturating_add(1);
                    }
                });

                current = EntityMembers::<T>::get(shop_id, curr_account)
                    .and_then(|m| m.referrer);
                depth += 1;
            }
        }

        /// 验证店主权限
        fn ensure_shop_owner(shop_id: u64, who: &T::AccountId) -> DispatchResult {
            let owner = T::EntityProvider::entity_owner(shop_id)
                .ok_or(Error::<T>::ShopNotFound)?;
            ensure!(*who == owner, Error::<T>::NotShopOwner);
            Ok(())
        }

        /// 计算自定义等级（根据消费金额）
        pub fn calculate_custom_level(shop_id: u64, total_spent: BalanceOf<T>) -> u8 {
            let system = match EntityLevelSystems::<T>::get(shop_id) {
                Some(s) if s.use_custom && !s.levels.is_empty() => s,
                _ => return 0,
            };

            let mut current_level = 0u8;
            for level in system.levels.iter() {
                if total_spent >= level.threshold {
                    current_level = level.id;
                } else {
                    break;
                }
            }
            current_level
        }

        /// 获取等级信息
        pub fn get_custom_level_info(shop_id: u64, level_id: u8) -> Option<CustomLevelOf<T>> {
            EntityLevelSystems::<T>::get(shop_id)
                .and_then(|s| s.levels.iter().find(|l| l.id == level_id).cloned())
        }

        /// 获取等级折扣率
        pub fn get_level_discount(shop_id: u64, level_id: u8) -> u16 {
            Self::get_custom_level_info(shop_id, level_id)
                .map(|l| l.discount_rate)
                .unwrap_or(0)
        }

        /// 获取等级返佣加成
        pub fn get_level_commission_bonus(shop_id: u64, level_id: u8) -> u16 {
            Self::get_custom_level_info(shop_id, level_id)
                .map(|l| l.commission_bonus)
                .unwrap_or(0)
        }

        // ========================================================================
        // 升级规则相关内部函数
        // ========================================================================

        /// 检查订单完成时的升级规则
        pub fn check_order_upgrade_rules(
            shop_id: u64,
            buyer: &T::AccountId,
            product_id: u64,
            order_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let system = match EntityUpgradeRules::<T>::get(shop_id) {
                Some(s) if s.enabled => s,
                _ => return Ok(()),
            };

            let member = match EntityMembers::<T>::get(shop_id, buyer) {
                Some(m) => m,
                None => return Ok(()),
            };

            // 更新订单数量
            MemberOrderCount::<T>::mutate(shop_id, buyer, |count| {
                *count = count.saturating_add(1);
            });

            let order_count = MemberOrderCount::<T>::get(shop_id, buyer);

            // 收集匹配的规则
            let mut matched_rules: alloc::vec::Vec<(u32, u8, Option<BlockNumberFor<T>>, u8, bool)> = alloc::vec::Vec::new();

            for rule in system.rules.iter() {
                if !rule.enabled {
                    continue;
                }

                let matches = match &rule.trigger {
                    UpgradeTrigger::PurchaseProduct { product_id: pid } => {
                        *pid == product_id
                    },
                    UpgradeTrigger::SingleOrder { threshold } => {
                        order_amount >= *threshold
                    },
                    UpgradeTrigger::TotalSpent { threshold } => {
                        member.total_spent >= *threshold
                    },
                    UpgradeTrigger::ReferralCount { count } => {
                        member.direct_referrals >= *count
                    },
                    UpgradeTrigger::TeamSize { size } => {
                        member.team_size >= *size
                    },
                    UpgradeTrigger::OrderCount { count } => {
                        order_count >= *count
                    },
                };

                if matches {
                    matched_rules.push((
                        rule.id,
                        rule.target_level_id,
                        rule.duration,
                        rule.priority,
                        rule.stackable,
                    ));
                }
            }

            if matched_rules.is_empty() {
                return Ok(());
            }

            // 根据冲突策略选择规则
            let selected = Self::resolve_conflict(&matched_rules, &system.conflict_strategy);

            if let Some((rule_id, target_level_id, duration, _, stackable)) = selected {
                Self::apply_upgrade(shop_id, buyer, rule_id, target_level_id, duration, stackable)?;
            }

            Ok(())
        }

        /// 解决规则冲突
        fn resolve_conflict(
            rules: &[(u32, u8, Option<BlockNumberFor<T>>, u8, bool)],
            strategy: &ConflictStrategy,
        ) -> Option<(u32, u8, Option<BlockNumberFor<T>>, u8, bool)> {
            if rules.is_empty() {
                return None;
            }

            match strategy {
                ConflictStrategy::HighestLevel => {
                    rules.iter().max_by_key(|r| r.1).cloned()
                },
                ConflictStrategy::HighestPriority => {
                    rules.iter().max_by_key(|r| r.3).cloned()
                },
                ConflictStrategy::LongestDuration => {
                    rules.iter().max_by_key(|r| r.2).cloned()
                },
                ConflictStrategy::FirstMatch => {
                    rules.first().cloned()
                },
            }
        }

        /// 应用升级
        fn apply_upgrade(
            shop_id: u64,
            account: &T::AccountId,
            rule_id: u32,
            target_level_id: u8,
            duration: Option<BlockNumberFor<T>>,
            stackable: bool,
        ) -> DispatchResult {
            let now = <frame_system::Pallet<T>>::block_number();

            EntityMembers::<T>::try_mutate(shop_id, account, |maybe_member| -> DispatchResult {
                let member = maybe_member.as_mut().ok_or(Error::<T>::NotMember)?;

                let old_level_id = member.custom_level_id;

                // 检查是否需要升级
                if target_level_id <= old_level_id && !stackable {
                    return Ok(());
                }

                // 计算过期时间
                let expires_at = if stackable {
                    let current_expiry = MemberLevelExpiry::<T>::get(shop_id, account)
                        .unwrap_or(now);
                    duration.map(|d| current_expiry.saturating_add(d))
                } else {
                    duration.map(|d| now.saturating_add(d))
                };

                // 升级等级
                member.custom_level_id = target_level_id;

                // 设置过期时间
                if let Some(exp) = expires_at {
                    MemberLevelExpiry::<T>::insert(shop_id, account, exp);
                } else {
                    MemberLevelExpiry::<T>::remove(shop_id, account);
                }

                // 记录升级历史
                let _ = MemberUpgradeHistory::<T>::try_mutate(shop_id, account, |history| {
                    let record = UpgradeRecord {
                        rule_id,
                        from_level_id: old_level_id,
                        to_level_id: target_level_id,
                        upgraded_at: now,
                        expires_at,
                    };
                    history.try_push(record).ok();
                    Ok::<_, Error<T>>(())
                });

                // 更新规则触发计数
                EntityUpgradeRules::<T>::mutate(shop_id, |maybe_system| {
                    if let Some(system) = maybe_system {
                        if let Some(r) = system.rules.iter_mut().find(|r| r.id == rule_id) {
                            r.trigger_count = r.trigger_count.saturating_add(1);
                        }
                    }
                });

                Self::deposit_event(Event::MemberUpgradedByRule {
                    shop_id,
                    account: account.clone(),
                    rule_id,
                    from_level_id: old_level_id,
                    to_level_id: target_level_id,
                    expires_at,
                });

                Ok(())
            })
        }

        /// 获取有效等级（考虑过期）
        pub fn get_effective_level(shop_id: u64, account: &T::AccountId) -> u8 {
            let member = match EntityMembers::<T>::get(shop_id, account) {
                Some(m) => m,
                None => return 0,
            };

            if let Some(expires_at) = MemberLevelExpiry::<T>::get(shop_id, account) {
                let now = <frame_system::Pallet<T>>::block_number();
                if now > expires_at {
                    // 已过期，返回基于消费的等级
                    return Self::calculate_custom_level(shop_id, member.total_spent);
                }
            }

            member.custom_level_id
        }

        /// 检查店铺是否使用自定义等级
        pub fn uses_custom_levels(shop_id: u64) -> bool {
            EntityLevelSystems::<T>::get(shop_id)
                .map(|s| s.use_custom)
                .unwrap_or(false)
        }

        /// 计算会员等级
        fn calculate_level(total_spent_usdt: u64) -> MemberLevel {
            if total_spent_usdt >= T::DiamondThreshold::get() {
                MemberLevel::Diamond
            } else if total_spent_usdt >= T::PlatinumThreshold::get() {
                MemberLevel::Platinum
            } else if total_spent_usdt >= T::GoldThreshold::get() {
                MemberLevel::Gold
            } else if total_spent_usdt >= T::SilverThreshold::get() {
                MemberLevel::Silver
            } else {
                MemberLevel::Normal
            }
        }

        /// 发放推荐返佣
        pub fn distribute_commission(
            shop_id: u64,
            buyer: &T::AccountId,
            order_amount: BalanceOf<T>,
            available_commission: BalanceOf<T>,
        ) -> DispatchResult {
            let config = match ShopCommissionConfig::<T>::get(shop_id) {
                Some(c) if c.enabled => c,
                _ => return Ok(()), // 未配置或未启用，跳过
            };

            let member = match EntityMembers::<T>::get(shop_id, buyer) {
                Some(m) => m,
                None => return Ok(()), // 非会员，跳过
            };

            let mut remaining = available_commission;
            let mut referrer = member.referrer;
            let rates = [config.level1_rate, config.level2_rate, config.level3_rate];

            for (level, rate) in rates.iter().enumerate() {
                if let Some(ref ref_account) = referrer {
                    if *rate == 0 {
                        referrer = EntityMembers::<T>::get(shop_id, ref_account)
                            .and_then(|m| m.referrer);
                        continue;
                    }

                    let commission = order_amount
                        .saturating_mul((*rate).into())
                        / 10000u32.into();

                    let actual_commission = commission.min(remaining);
                    if actual_commission.is_zero() {
                        break;
                    }

                    remaining = remaining.saturating_sub(actual_commission);

                    // 记录返佣
                    EntityMembers::<T>::mutate(shop_id, ref_account, |maybe_member| {
                        if let Some(ref mut m) = maybe_member {
                            m.total_commission = m.total_commission.saturating_add(actual_commission);
                            m.pending_commission = m.pending_commission.saturating_add(actual_commission);
                        }
                    });

                    // 更新买家的贡献
                    EntityMembers::<T>::mutate(shop_id, buyer, |maybe_member| {
                        if let Some(ref mut m) = maybe_member {
                            m.total_contributed = m.total_contributed.saturating_add(actual_commission);
                        }
                    });

                    // 更新店铺返佣统计
                    ShopCommissionStats::<T>::mutate(shop_id, |(total, _)| {
                        *total = total.saturating_add(actual_commission);
                    });

                    Self::deposit_event(Event::CommissionDistributed {
                        shop_id,
                        referrer: ref_account.clone(),
                        amount: actual_commission,
                        level: (level + 1) as u8,
                    });

                    // 获取上级的推荐人
                    referrer = EntityMembers::<T>::get(shop_id, ref_account)
                        .and_then(|m| m.referrer);
                } else {
                    break;
                }
            }

            Ok(())
        }

        /// 更新会员消费金额
        pub fn update_spent(
            shop_id: u64,
            account: &T::AccountId,
            amount: BalanceOf<T>,
            amount_usdt: u64,
        ) -> DispatchResult {
            EntityMembers::<T>::try_mutate(shop_id, account, |maybe_member| -> DispatchResult {
                let member = maybe_member.as_mut().ok_or(Error::<T>::NotMember)?;

                member.total_spent = member.total_spent.saturating_add(amount);
                member.last_active_at = <frame_system::Pallet<T>>::block_number();

                // 计算新等级（全局默认体系）
                let current_spent_usdt = sp_runtime::SaturatedConversion::saturated_into::<u64>(member.total_spent);
                let new_level = Self::calculate_level(current_spent_usdt.saturating_add(amount_usdt));

                if new_level != member.level {
                    let old_level = member.level;
                    member.level = new_level;

                    Self::deposit_event(Event::MemberLevelUpgraded {
                        shop_id,
                        account: account.clone(),
                        old_level,
                        new_level,
                    });
                }

                // 计算自定义等级（如果启用且为自动升级模式）
                if let Some(system) = EntityLevelSystems::<T>::get(shop_id) {
                    if system.use_custom && system.upgrade_mode == LevelUpgradeMode::AutoUpgrade {
                        let new_custom_level = Self::calculate_custom_level(shop_id, member.total_spent);
                        if new_custom_level != member.custom_level_id {
                            let old_level_id = member.custom_level_id;
                            member.custom_level_id = new_custom_level;

                            Self::deposit_event(Event::CustomLevelUpgraded {
                                shop_id,
                                account: account.clone(),
                                old_level_id,
                                new_level_id: new_custom_level,
                            });
                        }
                    }
                }

                Ok(())
            })
        }

        /// 自动注册会员
        pub fn auto_register(
            shop_id: u64,
            account: &T::AccountId,
            referrer: Option<T::AccountId>,
        ) -> DispatchResult {
            if EntityMembers::<T>::contains_key(shop_id, account) {
                return Ok(()); // 已是会员
            }

            // 验证推荐人
            let valid_referrer = if let Some(ref ref_account) = referrer {
                if ref_account != account && EntityMembers::<T>::contains_key(shop_id, ref_account) {
                    referrer
                } else {
                    None
                }
            } else {
                None
            };

            Self::do_register_member(shop_id, account, valid_referrer.clone())?;

            Self::deposit_event(Event::MemberRegistered {
                shop_id,
                account: account.clone(),
                referrer: valid_referrer,
            });

            Ok(())
        }
    }
}

// ============================================================================
// MemberProvider Trait 定义
// ============================================================================

/// 会员服务接口（供其他模块调用）
pub trait MemberProvider<AccountId, Balance> {
    /// 检查是否为店铺会员
    fn is_member(shop_id: u64, account: &AccountId) -> bool;

    /// 获取会员等级
    fn member_level(shop_id: u64, account: &AccountId) -> Option<pallet::MemberLevel>;

    /// 获取自定义等级 ID
    fn custom_level_id(shop_id: u64, account: &AccountId) -> u8;

    /// 获取等级折扣率
    fn get_level_discount(shop_id: u64, level_id: u8) -> u16;

    /// 获取等级返佣加成
    fn get_level_commission_bonus(shop_id: u64, level_id: u8) -> u16;

    /// 检查店铺是否使用自定义等级
    fn uses_custom_levels(shop_id: u64) -> bool;

    /// 获取推荐人
    fn get_referrer(shop_id: u64, account: &AccountId) -> Option<AccountId>;

    /// 自动注册会员（首次下单时）
    fn auto_register(shop_id: u64, account: &AccountId, referrer: Option<AccountId>) -> sp_runtime::DispatchResult;

    /// 更新消费金额
    fn update_spent(shop_id: u64, account: &AccountId, amount: Balance, amount_usdt: u64) -> sp_runtime::DispatchResult;

    /// 发放推荐返佣
    fn distribute_commission(
        shop_id: u64,
        buyer: &AccountId,
        order_amount: Balance,
        available_commission: Balance,
    ) -> sp_runtime::DispatchResult;

    /// 检查订单完成时的升级规则
    fn check_order_upgrade_rules(
        shop_id: u64,
        buyer: &AccountId,
        product_id: u64,
        order_amount: Balance,
    ) -> sp_runtime::DispatchResult;

    /// 获取有效等级（考虑过期）
    fn get_effective_level(shop_id: u64, account: &AccountId) -> u8;

    /// 获取会员统计信息 (直推人数, 团队人数, 累计消费USDT)
    fn get_member_stats(shop_id: u64, account: &AccountId) -> (u32, u32, u128);
}

/// MemberProvider 实现
impl<T: pallet::Config> MemberProvider<T::AccountId, pallet::BalanceOf<T>> for pallet::Pallet<T> {
    fn is_member(shop_id: u64, account: &T::AccountId) -> bool {
        pallet::EntityMembers::<T>::contains_key(shop_id, account)
    }

    fn member_level(shop_id: u64, account: &T::AccountId) -> Option<pallet::MemberLevel> {
        pallet::EntityMembers::<T>::get(shop_id, account).map(|m| m.level)
    }

    fn custom_level_id(shop_id: u64, account: &T::AccountId) -> u8 {
        pallet::EntityMembers::<T>::get(shop_id, account)
            .map(|m| m.custom_level_id)
            .unwrap_or(0)
    }

    fn get_level_discount(shop_id: u64, level_id: u8) -> u16 {
        pallet::Pallet::<T>::get_level_discount(shop_id, level_id)
    }

    fn get_level_commission_bonus(shop_id: u64, level_id: u8) -> u16 {
        pallet::Pallet::<T>::get_level_commission_bonus(shop_id, level_id)
    }

    fn uses_custom_levels(shop_id: u64) -> bool {
        pallet::Pallet::<T>::uses_custom_levels(shop_id)
    }

    fn get_referrer(shop_id: u64, account: &T::AccountId) -> Option<T::AccountId> {
        pallet::EntityMembers::<T>::get(shop_id, account).and_then(|m| m.referrer)
    }

    fn auto_register(shop_id: u64, account: &T::AccountId, referrer: Option<T::AccountId>) -> sp_runtime::DispatchResult {
        pallet::Pallet::<T>::auto_register(shop_id, account, referrer)
    }

    fn update_spent(shop_id: u64, account: &T::AccountId, amount: pallet::BalanceOf<T>, amount_usdt: u64) -> sp_runtime::DispatchResult {
        pallet::Pallet::<T>::update_spent(shop_id, account, amount, amount_usdt)
    }

    fn distribute_commission(
        shop_id: u64,
        buyer: &T::AccountId,
        order_amount: pallet::BalanceOf<T>,
        available_commission: pallet::BalanceOf<T>,
    ) -> sp_runtime::DispatchResult {
        pallet::Pallet::<T>::distribute_commission(shop_id, buyer, order_amount, available_commission)
    }

    fn check_order_upgrade_rules(
        shop_id: u64,
        buyer: &T::AccountId,
        product_id: u64,
        order_amount: pallet::BalanceOf<T>,
    ) -> sp_runtime::DispatchResult {
        pallet::Pallet::<T>::check_order_upgrade_rules(shop_id, buyer, product_id, order_amount)
    }

    fn get_effective_level(shop_id: u64, account: &T::AccountId) -> u8 {
        pallet::Pallet::<T>::get_effective_level(shop_id, account)
    }

    fn get_member_stats(shop_id: u64, account: &T::AccountId) -> (u32, u32, u128) {
        pallet::EntityMembers::<T>::get(shop_id, account)
            .map(|m| {
                let spent_usdt: u128 = sp_runtime::SaturatedConversion::saturated_into(m.total_spent);
                (m.direct_referrals, m.team_size, spent_usdt)
            })
            .unwrap_or((0, 0, 0))
    }
}

/// 空实现（用于测试或不需要会员功能的场景）
pub struct NullMemberProvider;

impl<AccountId, Balance> MemberProvider<AccountId, Balance> for NullMemberProvider {
    fn is_member(_shop_id: u64, _account: &AccountId) -> bool { false }
    fn member_level(_shop_id: u64, _account: &AccountId) -> Option<pallet::MemberLevel> { None }
    fn custom_level_id(_shop_id: u64, _account: &AccountId) -> u8 { 0 }
    fn get_level_discount(_shop_id: u64, _level_id: u8) -> u16 { 0 }
    fn get_level_commission_bonus(_shop_id: u64, _level_id: u8) -> u16 { 0 }
    fn uses_custom_levels(_shop_id: u64) -> bool { false }
    fn get_referrer(_shop_id: u64, _account: &AccountId) -> Option<AccountId> { None }
    fn auto_register(_shop_id: u64, _account: &AccountId, _referrer: Option<AccountId>) -> sp_runtime::DispatchResult { Ok(()) }
    fn update_spent(_shop_id: u64, _account: &AccountId, _amount: Balance, _amount_usdt: u64) -> sp_runtime::DispatchResult { Ok(()) }
    fn distribute_commission(
        _shop_id: u64,
        _buyer: &AccountId,
        _order_amount: Balance,
        _available_commission: Balance,
    ) -> sp_runtime::DispatchResult { Ok(()) }
    fn check_order_upgrade_rules(
        _shop_id: u64,
        _buyer: &AccountId,
        _product_id: u64,
        _order_amount: Balance,
    ) -> sp_runtime::DispatchResult { Ok(()) }
    fn get_effective_level(_shop_id: u64, _account: &AccountId) -> u8 { 0 }
    fn get_member_stats(_shop_id: u64, _account: &AccountId) -> (u32, u32, u128) { (0, 0, 0) }
}
