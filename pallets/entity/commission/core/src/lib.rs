//! # Commission Core (pallet-commission-core)
//!
//! 返佣系统核心调度引擎。负责：
//! - 全局返佣配置（启用模式、来源、上限）
//! - 返佣记账（credit_commission）与取消（cancel_commission）
//! - 提现系统（分级提现 + 购物余额）
//! - 偿付安全（ShopPendingTotal + ShopShoppingTotal）
//! - 调度各插件（ReferralPlugin / LevelDiffPlugin / SingleLinePlugin / TeamPlugin）

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;
pub use pallet_commission_common::{
    CommissionModes, CommissionOutput, CommissionPlugin, CommissionProvider,
    CommissionRecord, CommissionSource, CommissionStatus, CommissionType,
    MemberCommissionStatsData, MemberProvider, WithdrawalTierConfig,
};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, Get},
    };
    use frame_system::pallet_prelude::*;
    use pallet_entity_common::ShopProvider;
    use sp_runtime::traits::{Saturating, Zero};

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type CommissionRecordOf<T> = CommissionRecord<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
    >;

    pub type MemberCommissionStatsOf<T> = MemberCommissionStatsData<BalanceOf<T>>;

    /// 全局返佣开关配置（per-shop）
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct CoreCommissionConfig {
        /// 启用的返佣模式（位标志）
        pub enabled_modes: CommissionModes,
        /// 返佣来源（预留）
        pub source: CommissionSource,
        /// 返佣上限比例（基点，10000 = 100%）
        pub max_commission_rate: u16,
        /// 是否全局启用
        pub enabled: bool,
    }

    impl Default for CoreCommissionConfig {
        fn default() -> Self {
            Self {
                enabled_modes: CommissionModes::default(),
                source: CommissionSource::default(),
                max_commission_rate: 10000,
                enabled: false,
            }
        }
    }

    /// 实体提现配置
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    #[scale_info(skip_type_params(MaxLevels))]
    pub struct EntityWithdrawalConfig<MaxLevels: Get<u32>> {
        pub tier_configs: BoundedVec<WithdrawalTierConfig, MaxLevels>,
        pub enabled: bool,
        pub shopping_balance_generates_commission: bool,
    }

    impl<MaxLevels: Get<u32>> Default for EntityWithdrawalConfig<MaxLevels> {
        fn default() -> Self {
            Self {
                tier_configs: BoundedVec::default(),
                enabled: false,
                shopping_balance_generates_commission: false,
            }
        }
    }

    pub type EntityWithdrawalConfigOf<T> = EntityWithdrawalConfig<<T as Config>::MaxCustomLevels>;

    // ========================================================================
    // Config
    // ========================================================================

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: Currency<Self::AccountId>;

        /// Shop 查询接口
        type ShopProvider: ShopProvider<Self::AccountId>;

        /// 会员查询接口
        type MemberProvider: MemberProvider<Self::AccountId>;

        /// 推荐链返佣插件
        type ReferralPlugin: CommissionPlugin<Self::AccountId, BalanceOf<Self>>;

        /// 等级极差返佣插件
        type LevelDiffPlugin: CommissionPlugin<Self::AccountId, BalanceOf<Self>>;

        /// 单线收益插件
        type SingleLinePlugin: CommissionPlugin<Self::AccountId, BalanceOf<Self>>;

        /// 团队业绩插件（预留）
        type TeamPlugin: CommissionPlugin<Self::AccountId, BalanceOf<Self>>;

        /// 最大返佣记录数（每订单）
        #[pallet::constant]
        type MaxCommissionRecordsPerOrder: Get<u32>;

        /// 最大自定义等级数
        #[pallet::constant]
        type MaxCustomLevels: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ========================================================================
    // Storage
    // ========================================================================

    /// Shop 返佣核心配置 shop_id -> CoreCommissionConfig
    #[pallet::storage]
    #[pallet::getter(fn commission_config)]
    pub type CommissionConfigs<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        CoreCommissionConfig,
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

    /// Shop 返佣统计 shop_id -> (total_distributed, total_orders)
    #[pallet::storage]
    #[pallet::getter(fn shop_commission_totals)]
    pub type ShopCommissionTotals<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        (BalanceOf<T>, u64),
        ValueQuery,
    >;

    /// Shop 待提取佣金总额 shop_id -> Balance
    #[pallet::storage]
    #[pallet::getter(fn shop_pending_total)]
    pub type ShopPendingTotal<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        BalanceOf<T>,
        ValueQuery,
    >;

    /// Shop 购物余额总额 shop_id -> Balance（资金锁定）
    #[pallet::storage]
    #[pallet::getter(fn shop_shopping_total)]
    pub type ShopShoppingTotal<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        BalanceOf<T>,
        ValueQuery,
    >;

    /// 提现配置 shop_id -> EntityWithdrawalConfig
    #[pallet::storage]
    #[pallet::getter(fn withdrawal_config)]
    pub type WithdrawalConfigs<T: Config> = StorageMap<
        _,
        Blake2_128Concat, u64,
        EntityWithdrawalConfigOf<T>,
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

    // ========================================================================
    // Events
    // ========================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CommissionConfigUpdated { shop_id: u64 },
        CommissionModesUpdated { shop_id: u64, modes: CommissionModes },
        CommissionDistributed {
            shop_id: u64,
            order_id: u64,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            commission_type: CommissionType,
            level: u8,
        },
        CommissionWithdrawn {
            shop_id: u64,
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
        CommissionCancelled { order_id: u64 },
        TieredWithdrawal {
            shop_id: u64,
            account: T::AccountId,
            withdrawn_amount: BalanceOf<T>,
            repurchase_amount: BalanceOf<T>,
        },
        WithdrawalConfigUpdated { shop_id: u64 },
        ShoppingBalanceUsed {
            shop_id: u64,
            account: T::AccountId,
            amount: BalanceOf<T>,
        },
    }

    // ========================================================================
    // Errors
    // ========================================================================

    #[pallet::error]
    pub enum Error<T> {
        ShopNotFound,
        NotShopOwner,
        CommissionNotConfigured,
        InsufficientCommission,
        InvalidCommissionRate,
        RecordsFull,
        Overflow,
        WithdrawalConfigNotEnabled,
        InvalidWithdrawalConfig,
        InsufficientShoppingBalance,
    }

    // ========================================================================
    // Extrinsics
    // ========================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 设置启用的返佣模式
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn set_commission_modes(
            origin: OriginFor<T>,
            shop_id: u64,
            modes: CommissionModes,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            CommissionConfigs::<T>::mutate(shop_id, |maybe| {
                let config = maybe.get_or_insert_with(CoreCommissionConfig::default);
                config.enabled_modes = modes;
            });

            Self::deposit_event(Event::CommissionModesUpdated { shop_id, modes });
            Ok(())
        }

        /// 设置返佣来源和上限
        #[pallet::call_index(1)]
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

            CommissionConfigs::<T>::mutate(shop_id, |maybe| {
                let config = maybe.get_or_insert_with(CoreCommissionConfig::default);
                config.source = source;
                config.max_commission_rate = max_rate;
            });

            Self::deposit_event(Event::CommissionConfigUpdated { shop_id });
            Ok(())
        }

        /// 启用/禁用返佣
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn enable_commission(
            origin: OriginFor<T>,
            shop_id: u64,
            enabled: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_shop_owner(shop_id, &who)?;

            CommissionConfigs::<T>::mutate(shop_id, |maybe| {
                let config = maybe.get_or_insert_with(CoreCommissionConfig::default);
                config.enabled = enabled;
            });

            Self::deposit_event(Event::CommissionConfigUpdated { shop_id });
            Ok(())
        }

        /// 提取返佣（支持分级提现）
        #[pallet::call_index(3)]
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

                // 计算提现/复购分配
                let (withdrawal_amount, repurchase_amount) = Self::calc_withdrawal_split(shop_id, &who, total_amount);

                // 从 Shop 运营账户转账提现部分到用户钱包
                if !withdrawal_amount.is_zero() {
                    let shop_account = T::ShopProvider::shop_account(shop_id);
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
                    // 锁定购物余额对应资金
                    ShopShoppingTotal::<T>::mutate(shop_id, |total| {
                        *total = total.saturating_add(repurchase_amount);
                    });
                }

                stats.pending = stats.pending.saturating_sub(total_amount);
                stats.withdrawn = stats.withdrawn.saturating_add(withdrawal_amount);
                stats.repurchased = stats.repurchased.saturating_add(repurchase_amount);

                // 释放 pending 锁定
                ShopPendingTotal::<T>::mutate(shop_id, |total| {
                    *total = total.saturating_sub(total_amount);
                });

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
        #[pallet::call_index(4)]
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

            for config in tier_configs.iter() {
                ensure!(
                    config.withdrawal_rate.saturating_add(config.repurchase_rate) == 10000,
                    Error::<T>::InvalidWithdrawalConfig
                );
            }

            WithdrawalConfigs::<T>::insert(shop_id, EntityWithdrawalConfig {
                tier_configs,
                enabled,
                shopping_balance_generates_commission,
            });

            Self::deposit_event(Event::WithdrawalConfigUpdated { shop_id });
            Ok(())
        }

        /// 使用购物余额支付（由订单模块调用）
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn use_shopping_balance(
            origin: OriginFor<T>,
            shop_id: u64,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            MemberShoppingBalance::<T>::try_mutate(shop_id, &who, |balance| -> DispatchResult {
                ensure!(*balance >= amount, Error::<T>::InsufficientShoppingBalance);
                *balance = balance.saturating_sub(amount);

                // 释放购物余额对应的资金锁定
                ShopShoppingTotal::<T>::mutate(shop_id, |total| {
                    *total = total.saturating_sub(amount);
                });

                Self::deposit_event(Event::ShoppingBalanceUsed {
                    shop_id,
                    account: who.clone(),
                    amount,
                });

                Ok(())
            })
        }
    }

    // ========================================================================
    // Internal functions
    // ========================================================================

    impl<T: Config> Pallet<T> {
        /// 验证店主权限
        fn ensure_shop_owner(shop_id: u64, who: &T::AccountId) -> DispatchResult {
            let owner = T::ShopProvider::shop_owner(shop_id)
                .ok_or(Error::<T>::ShopNotFound)?;
            ensure!(*who == owner, Error::<T>::NotShopOwner);
            Ok(())
        }

        /// 计算提现/复购分配
        fn calc_withdrawal_split(
            shop_id: u64,
            who: &T::AccountId,
            total_amount: BalanceOf<T>,
        ) -> (BalanceOf<T>, BalanceOf<T>) {
            let config = WithdrawalConfigs::<T>::get(shop_id);
            if let Some(ref config) = config {
                if config.enabled && !config.tier_configs.is_empty() {
                    let level_id = T::MemberProvider::custom_level_id(shop_id, who);
                    let tier_config = config.tier_configs
                        .get(level_id as usize)
                        .cloned()
                        .unwrap_or_default();

                    let withdrawal = total_amount
                        .saturating_mul(tier_config.withdrawal_rate.into())
                        / 10000u32.into();
                    let repurchase = total_amount.saturating_sub(withdrawal);
                    return (withdrawal, repurchase);
                }
            }
            (total_amount, BalanceOf::<T>::zero())
        }

        /// 调度引擎：处理订单返佣
        pub fn process_commission(
            shop_id: u64,
            order_id: u64,
            buyer: &T::AccountId,
            order_amount: BalanceOf<T>,
            available_pool: BalanceOf<T>,
        ) -> DispatchResult {
            let config = match CommissionConfigs::<T>::get(shop_id) {
                Some(c) if c.enabled => c,
                _ => return Ok(()),
            };

            // 计算最大可用返佣
            let max_commission = available_pool
                .saturating_mul(config.max_commission_rate.into())
                / 10000u32.into();

            // 偿付安全: 可用 = 余额 - pending - shopping
            let shop_account = T::ShopProvider::shop_account(shop_id);
            let shop_balance = T::Currency::free_balance(&shop_account);
            let pending_total = ShopPendingTotal::<T>::get(shop_id);
            let shopping_total = ShopShoppingTotal::<T>::get(shop_id);
            let available_funds = shop_balance
                .saturating_sub(pending_total)
                .saturating_sub(shopping_total);
            let mut remaining = max_commission.min(available_funds);

            if remaining.is_zero() {
                return Ok(());
            }

            let now = <frame_system::Pallet<T>>::block_number();
            let buyer_stats = MemberCommissionStats::<T>::get(shop_id, buyer);
            let is_first_order = buyer_stats.order_count == 0;
            let enabled_modes = config.enabled_modes;

            // 1. Referral Plugin（Direct + MultiLevel + Fixed + FirstOrder + RepeatPurchase）
            let (outputs, new_remaining) = T::ReferralPlugin::calculate(
                shop_id, buyer, order_amount, remaining, enabled_modes, is_first_order, buyer_stats.order_count,
            );
            remaining = new_remaining;
            for output in outputs {
                Self::credit_commission(
                    shop_id, order_id, buyer, &output.beneficiary, output.amount,
                    output.commission_type, output.level, now,
                )?;
            }

            // 2. LevelDiff Plugin
            let (outputs, new_remaining) = T::LevelDiffPlugin::calculate(
                shop_id, buyer, order_amount, remaining, enabled_modes, is_first_order, buyer_stats.order_count,
            );
            remaining = new_remaining;
            for output in outputs {
                Self::credit_commission(
                    shop_id, order_id, buyer, &output.beneficiary, output.amount,
                    output.commission_type, output.level, now,
                )?;
            }

            // 3. SingleLine Plugin
            let (outputs, new_remaining) = T::SingleLinePlugin::calculate(
                shop_id, buyer, order_amount, remaining, enabled_modes, is_first_order, buyer_stats.order_count,
            );
            remaining = new_remaining;
            for output in outputs {
                Self::credit_commission(
                    shop_id, order_id, buyer, &output.beneficiary, output.amount,
                    output.commission_type, output.level, now,
                )?;
            }

            // 4. Team Plugin (预留)
            let (outputs, _new_remaining) = T::TeamPlugin::calculate(
                shop_id, buyer, order_amount, remaining, enabled_modes, is_first_order, buyer_stats.order_count,
            );
            for output in outputs {
                Self::credit_commission(
                    shop_id, order_id, buyer, &output.beneficiary, output.amount,
                    output.commission_type, output.level, now,
                )?;
            }

            // 更新买家订单数
            MemberCommissionStats::<T>::mutate(shop_id, buyer, |stats| {
                stats.order_count = stats.order_count.saturating_add(1);
            });

            // 更新 Shop 统计
            let distributed = max_commission.min(available_funds).saturating_sub(remaining);
            ShopCommissionTotals::<T>::mutate(shop_id, |(total, orders)| {
                *total = total.saturating_add(distributed);
                *orders = orders.saturating_add(1);
            });

            Ok(())
        }

        /// 记录并发放返佣
        pub fn credit_commission(
            shop_id: u64,
            order_id: u64,
            buyer: &T::AccountId,
            beneficiary: &T::AccountId,
            amount: BalanceOf<T>,
            commission_type: CommissionType,
            level: u8,
            now: BlockNumberFor<T>,
        ) -> DispatchResult {
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

            MemberCommissionStats::<T>::mutate(shop_id, beneficiary, |stats| {
                stats.total_earned = stats.total_earned.saturating_add(amount);
                stats.pending = stats.pending.saturating_add(amount);
            });

            ShopPendingTotal::<T>::mutate(shop_id, |total| {
                *total = total.saturating_add(amount);
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
                        MemberCommissionStats::<T>::mutate(record.shop_id, &record.beneficiary, |stats| {
                            stats.pending = stats.pending.saturating_sub(record.amount);
                            stats.total_earned = stats.total_earned.saturating_sub(record.amount);
                        });
                        ShopPendingTotal::<T>::mutate(record.shop_id, |total| {
                            *total = total.saturating_sub(record.amount);
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
// CommissionProvider impl
// ============================================================================

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
        pallet::CommissionConfigs::<T>::mutate(shop_id, |maybe| {
            let config = maybe.get_or_insert_with(pallet::CoreCommissionConfig::default);
            config.enabled_modes = CommissionModes(modes);
        });
        Ok(())
    }

    fn set_direct_reward_rate(_shop_id: u64, _rate: u16) -> sp_runtime::DispatchResult {
        // 委托给 referral 插件（通过 governance 模块 trait 调用）
        Ok(())
    }

    fn set_level_diff_config(_: u64, _: u16, _: u16, _: u16, _: u16, _: u16) -> sp_runtime::DispatchResult {
        Ok(())
    }

    fn set_fixed_amount(_: u64, _: pallet::BalanceOf<T>) -> sp_runtime::DispatchResult {
        Ok(())
    }

    fn set_first_order_config(_: u64, _: pallet::BalanceOf<T>, _: u16, _: bool) -> sp_runtime::DispatchResult {
        Ok(())
    }

    fn set_repeat_purchase_config(_: u64, _: u16, _: u32) -> sp_runtime::DispatchResult {
        Ok(())
    }

    fn set_withdrawal_config_by_governance(
        shop_id: u64,
        enabled: bool,
        shopping_balance_generates_commission: bool,
    ) -> sp_runtime::DispatchResult {
        pallet::WithdrawalConfigs::<T>::mutate(shop_id, |maybe| {
            let config = maybe.get_or_insert_with(pallet::EntityWithdrawalConfig::default);
            config.enabled = enabled;
            config.shopping_balance_generates_commission = shopping_balance_generates_commission;
        });
        Ok(())
    }

    fn shopping_balance(shop_id: u64, account: &T::AccountId) -> pallet::BalanceOf<T> {
        pallet::MemberShoppingBalance::<T>::get(shop_id, account)
    }
}
