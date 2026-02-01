//! # 商城店铺管理模块 (pallet-sharemall-shop)
//!
//! ## 概述
//!
//! 本模块负责店铺的生命周期管理，包括：
//! - 店铺创建（转入运营资金到派生账户）
//! - 店铺信息更新
//! - 运营资金管理（充值、消费、健康监控）
//! - 店铺状态管理（暂停、恢复、申请关闭）
//! - 治理审核（批准、封禁、审批关闭）
//!
//! ## 运营资金机制
//!
//! - 创建店铺时转入 50 USDT 等值 COS 到派生账户
//! - 资金可用于支付 IPFS Pin、存储租金等运营费用
//! - 资金不可提取，仅治理关闭后退还
//! - 低于最低余额时店铺自动暂停
//!
//! ## 版本历史
//!
//! - v0.1.0 (2026-01-31): 从 pallet-mall 拆分
//! - v0.2.0 (2026-02-01): 实现运营资金派生账户机制

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;
pub use pallet_sharemall_common::ShopStatus;

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
        traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
        BoundedVec, PalletId,
    };
    use frame_system::pallet_prelude::*;
    use pallet_sharemall_common::{PricingProvider, ShopProvider, ShopStatus};
    use sp_runtime::{
        traits::{AccountIdConversion, Saturating, Zero},
        SaturatedConversion,
    };

    /// 店铺派生账户 PalletId
    const SHOP_PALLET_ID: PalletId = PalletId(*b"sm/shop/");

    /// 资金健康状态
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub enum FundHealth {
        /// 健康（余额 > 预警阈值）
        Healthy,
        /// 预警（最低余额 < 余额 ≤ 预警阈值）
        Warning,
        /// 危险（余额 ≤ 最低余额，店铺暂停）
        Critical,
        /// 耗尽（余额 = 0）
        Depleted,
    }

    /// 运营费用类型
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub enum FeeType {
        /// IPFS Pin 费用
        IpfsPin,
        /// 链上存储租金
        StorageRent,
        /// 交易手续费
        TransactionFee,
        /// 推广费用
        Promotion,
        /// 其他费用
        Other,
    }

    /// 货币余额类型别名
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// 店铺信息
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    #[scale_info(skip_type_params(MaxNameLen, MaxCidLen))]
    pub struct Shop<AccountId, Balance, BlockNumber, MaxNameLen: Get<u32>, MaxCidLen: Get<u32>> {
        /// 店铺 ID
        pub id: u64,
        /// 店主账户
        pub owner: AccountId,
        /// 客服聊天账户（用于 Pallet Chat，默认使用店主账户）
        pub customer_service: Option<AccountId>,
        /// 店铺名称
        pub name: BoundedVec<u8, MaxNameLen>,
        /// 店铺 Logo IPFS CID
        pub logo_cid: Option<BoundedVec<u8, MaxCidLen>>,
        /// 店铺描述 IPFS CID
        pub description_cid: Option<BoundedVec<u8, MaxCidLen>>,
        /// 初始运营资金
        pub initial_fund: Balance,
        /// 店铺状态
        pub status: ShopStatus,
        /// 商品数量
        pub product_count: u32,
        /// 累计销售额
        pub total_sales: Balance,
        /// 累计订单数
        pub total_orders: u32,
        /// 店铺评分 (0-500，代表 0.0-5.0)
        pub rating: u16,
        /// 评价数量
        pub rating_count: u32,
        /// 创建时间
        pub created_at: BlockNumber,
    }

    /// 店铺类型别名
    pub type ShopOf<T> = Shop<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxShopNameLength,
        <T as Config>::MaxCidLength,
    >;

    /// 店铺统计
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct ShopStatistics {
        /// 总店铺数
        pub total_shops: u64,
        /// 活跃店铺数
        pub active_shops: u64,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 货币类型
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 店铺名称最大长度
        #[pallet::constant]
        type MaxShopNameLength: Get<u32>;

        /// CID 最大长度
        #[pallet::constant]
        type MaxCidLength: Get<u32>;

        /// 治理 Origin
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 定价提供者（用于计算 USDT 等值 COS 押金）
        type PricingProvider: PricingProvider;

        /// 初始运营资金 USDT 金额（精度 10^6，即 50_000_000 = 50 USDT）
        #[pallet::constant]
        type InitialFundUsdt: Get<u64>;

        /// 最小初始资金 COS（防止价格过高时资金过低）
        #[pallet::constant]
        type MinInitialFundCos: Get<BalanceOf<Self>>;

        /// 最大初始资金 COS（防止价格过低时资金过高）
        #[pallet::constant]
        type MaxInitialFundCos: Get<BalanceOf<Self>>;

        /// 最低运营余额（低于此值店铺暂停）
        #[pallet::constant]
        type MinOperatingBalance: Get<BalanceOf<Self>>;

        /// 资金预警阈值（低于此值发出预警）
        #[pallet::constant]
        type FundWarningThreshold: Get<BalanceOf<Self>>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ==================== 存储项 ====================

    /// 下一个店铺 ID
    #[pallet::storage]
    #[pallet::getter(fn next_shop_id)]
    pub type NextShopId<T> = StorageValue<_, u64, ValueQuery>;

    /// 店铺存储
    #[pallet::storage]
    #[pallet::getter(fn shops)]
    pub type Shops<T: Config> = StorageMap<_, Blake2_128Concat, u64, ShopOf<T>>;

    /// 用户店铺索引（一个用户只能有一个店铺）
    #[pallet::storage]
    #[pallet::getter(fn user_shop)]
    pub type UserShop<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64>;

    /// 店铺统计
    #[pallet::storage]
    #[pallet::getter(fn shop_stats)]
    pub type ShopStats<T: Config> = StorageValue<_, ShopStatistics, ValueQuery>;

    /// 店铺关闭申请时间
    #[pallet::storage]
    #[pallet::getter(fn shop_close_requests)]
    pub type ShopCloseRequests<T: Config> = StorageMap<_, Blake2_128Concat, u64, BlockNumberFor<T>>;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 店铺已创建
        ShopCreated {
            shop_id: u64,
            owner: T::AccountId,
            shop_account: T::AccountId,
            initial_fund: BalanceOf<T>,
        },
        /// 店铺已更新
        ShopUpdated { shop_id: u64 },
        /// 店铺状态已变更
        ShopStatusChanged { shop_id: u64, status: ShopStatus },
        /// 运营资金已充值
        FundToppedUp {
            shop_id: u64,
            amount: BalanceOf<T>,
            new_balance: BalanceOf<T>,
        },
        /// 运营费用已扣除
        OperatingFeeDeducted {
            shop_id: u64,
            fee: BalanceOf<T>,
            fee_type: FeeType,
            remaining_balance: BalanceOf<T>,
        },
        /// 资金预警
        FundWarning {
            shop_id: u64,
            current_balance: BalanceOf<T>,
            warning_threshold: BalanceOf<T>,
        },
        /// 店铺因资金不足暂停
        ShopSuspendedLowFund {
            shop_id: u64,
            current_balance: BalanceOf<T>,
            minimum_balance: BalanceOf<T>,
        },
        /// 充值后店铺恢复
        ShopResumedAfterFunding { shop_id: u64 },
        /// 店主申请关闭店铺
        ShopCloseRequested { shop_id: u64 },
        /// 店铺已关闭（资金已退还）
        ShopClosed {
            shop_id: u64,
            fund_refunded: BalanceOf<T>,
        },
        /// 店铺被封禁
        ShopBanned {
            shop_id: u64,
            fund_confiscated: bool,
        },
        /// 资金被没收
        FundConfiscated {
            shop_id: u64,
            amount: BalanceOf<T>,
        },
    }

    // ==================== 错误 ====================

    #[pallet::error]
    pub enum Error<T> {
        /// 店铺不存在
        ShopNotFound,
        /// 用户已有店铺
        ShopAlreadyExists,
        /// 不是店主
        NotShopOwner,
        /// 店铺未激活
        ShopNotActive,
        /// 店铺有进行中的订单
        ShopHasPendingOrders,
        /// 运营资金不足
        InsufficientOperatingFund,
        /// 无效的店铺状态
        InvalidShopStatus,
        /// 名称过长
        NameTooLong,
        /// CID 过长
        CidTooLong,
        /// 价格不可用
        PriceUnavailable,
        /// 算术溢出
        ArithmeticOverflow,
        /// 余额不足以支付初始资金
        InsufficientBalanceForInitialFund,
    }

    // ==================== Extrinsics ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建店铺（转入运营资金到派生账户）
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(50_000, 0))]
        pub fn create_shop(
            origin: OriginFor<T>,
            name: Vec<u8>,
            logo_cid: Option<Vec<u8>>,
            description_cid: Option<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!UserShop::<T>::contains_key(&who), Error::<T>::ShopAlreadyExists);

            let name: BoundedVec<u8, T::MaxShopNameLength> =
                name.try_into().map_err(|_| Error::<T>::NameTooLong)?;

            let logo_cid: Option<BoundedVec<u8, T::MaxCidLength>> = logo_cid
                .map(|c| c.try_into().map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;
            let description_cid: Option<BoundedVec<u8, T::MaxCidLength>> = description_cid
                .map(|c| c.try_into().map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            // 计算初始运营资金（50 USDT 等值 COS）
            let initial_fund = Self::calculate_initial_fund()?;
            
            // 生成店铺派生账户
            let shop_id = NextShopId::<T>::get();
            let shop_account = Self::shop_account(shop_id);
            
            // 转入派生账户（不可提取）
            T::Currency::transfer(
                &who,
                &shop_account,
                initial_fund,
                ExistenceRequirement::KeepAlive,
            ).map_err(|_| Error::<T>::InsufficientBalanceForInitialFund)?;

            let now = <frame_system::Pallet<T>>::block_number();

            let shop = Shop {
                id: shop_id,
                owner: who.clone(),
                customer_service: None,  // 默认使用店主账户
                name,
                logo_cid,
                description_cid,
                initial_fund,
                status: ShopStatus::Pending,
                product_count: 0,
                total_sales: Zero::zero(),
                total_orders: 0,
                rating: 0,
                rating_count: 0,
                created_at: now,
            };

            Shops::<T>::insert(shop_id, shop);
            UserShop::<T>::insert(&who, shop_id);
            NextShopId::<T>::put(shop_id.saturating_add(1));

            ShopStats::<T>::mutate(|stats| {
                stats.total_shops = stats.total_shops.saturating_add(1);
            });

            Self::deposit_event(Event::ShopCreated {
                shop_id,
                owner: who,
                shop_account,
                initial_fund,
            });

            Ok(())
        }

        /// 更新店铺信息
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn update_shop(
            origin: OriginFor<T>,
            shop_id: u64,
            name: Option<Vec<u8>>,
            logo_cid: Option<Vec<u8>>,
            description_cid: Option<Vec<u8>>,
            customer_service: Option<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Shops::<T>::try_mutate(shop_id, |maybe_shop| -> DispatchResult {
                let shop = maybe_shop.as_mut().ok_or(Error::<T>::ShopNotFound)?;
                ensure!(shop.owner == who, Error::<T>::NotShopOwner);

                if let Some(n) = name {
                    shop.name = n.try_into().map_err(|_| Error::<T>::NameTooLong)?;
                }
                if let Some(c) = logo_cid {
                    shop.logo_cid = Some(c.try_into().map_err(|_| Error::<T>::CidTooLong)?);
                }
                if let Some(c) = description_cid {
                    shop.description_cid = Some(c.try_into().map_err(|_| Error::<T>::CidTooLong)?);
                }
                if customer_service.is_some() {
                    shop.customer_service = customer_service;
                }

                Ok(())
            })?;

            Self::deposit_event(Event::ShopUpdated { shop_id });
            Ok(())
        }

        /// 申请关闭店铺（需治理审批）
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn request_close_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Shops::<T>::try_mutate(shop_id, |maybe_shop| -> DispatchResult {
                let shop = maybe_shop.as_mut().ok_or(Error::<T>::ShopNotFound)?;
                ensure!(shop.owner == who, Error::<T>::NotShopOwner);
                ensure!(
                    shop.status == ShopStatus::Active || shop.status == ShopStatus::Suspended,
                    Error::<T>::InvalidShopStatus
                );

                shop.status = ShopStatus::Pending;  // 复用 Pending 状态表示待关闭审批
                Ok(())
            })?;

            // 记录申请时间
            let now = <frame_system::Pallet<T>>::block_number();
            ShopCloseRequests::<T>::insert(shop_id, now);

            Self::deposit_event(Event::ShopCloseRequested { shop_id });
            Ok(())
        }

        /// 充值运营资金
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn top_up_fund(
            origin: OriginFor<T>,
            shop_id: u64,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let shop = Shops::<T>::get(shop_id).ok_or(Error::<T>::ShopNotFound)?;
            ensure!(shop.owner == who, Error::<T>::NotShopOwner);
            ensure!(
                shop.status != ShopStatus::Closed && shop.status != ShopStatus::Banned,
                Error::<T>::InvalidShopStatus
            );

            let shop_account = Self::shop_account(shop_id);

            T::Currency::transfer(
                &who,
                &shop_account,
                amount,
                ExistenceRequirement::KeepAlive,
            )?;

            let new_balance = T::Currency::free_balance(&shop_account);
            let min_balance = T::MinOperatingBalance::get();

            // 如果店铺因资金不足暂停，检查是否可以恢复
            if shop.status == ShopStatus::Suspended && new_balance >= min_balance {
                Shops::<T>::mutate(shop_id, |s| {
                    if let Some(shop) = s {
                        shop.status = ShopStatus::Active;
                    }
                });
                ShopStats::<T>::mutate(|stats| {
                    stats.active_shops = stats.active_shops.saturating_add(1);
                });
                Self::deposit_event(Event::ShopResumedAfterFunding { shop_id });
            }

            Self::deposit_event(Event::FundToppedUp {
                shop_id,
                amount,
                new_balance,
            });

            Ok(())
        }

        /// 审核通过店铺（治理）
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn approve_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            Shops::<T>::try_mutate(shop_id, |maybe_shop| -> DispatchResult {
                let shop = maybe_shop.as_mut().ok_or(Error::<T>::ShopNotFound)?;
                // 只有新创建的店铺（无关闭申请记录）才能审批通过
                ensure!(shop.status == ShopStatus::Pending, Error::<T>::InvalidShopStatus);
                ensure!(!ShopCloseRequests::<T>::contains_key(shop_id), Error::<T>::InvalidShopStatus);

                shop.status = ShopStatus::Active;
                Ok(())
            })?;

            ShopStats::<T>::mutate(|stats| {
                stats.active_shops = stats.active_shops.saturating_add(1);
            });

            Self::deposit_event(Event::ShopStatusChanged {
                shop_id,
                status: ShopStatus::Active,
            });
            Ok(())
        }

        /// 审批关闭店铺（治理，退还全部余额）
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn approve_close_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            let shop = Shops::<T>::get(shop_id).ok_or(Error::<T>::ShopNotFound)?;
            // 必须有关闭申请记录
            ensure!(ShopCloseRequests::<T>::contains_key(shop_id), Error::<T>::InvalidShopStatus);

            let shop_account = Self::shop_account(shop_id);
            let balance = T::Currency::free_balance(&shop_account);

            // 退还全部余额给店主
            if !balance.is_zero() {
                T::Currency::transfer(
                    &shop_account,
                    &shop.owner,
                    balance,
                    ExistenceRequirement::AllowDeath,
                )?;
            }

            // 更新状态
            Shops::<T>::mutate(shop_id, |s| {
                if let Some(shop) = s {
                    shop.status = ShopStatus::Closed;
                }
            });

            // 清理关闭申请记录
            ShopCloseRequests::<T>::remove(shop_id);

            // 更新统计
            if shop.status == ShopStatus::Active {
                ShopStats::<T>::mutate(|stats| {
                    stats.active_shops = stats.active_shops.saturating_sub(1);
                });
            }

            Self::deposit_event(Event::ShopClosed {
                shop_id,
                fund_refunded: balance,
            });
            Ok(())
        }

        /// 暂停店铺（治理）
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn suspend_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            Shops::<T>::try_mutate(shop_id, |maybe_shop| -> DispatchResult {
                let shop = maybe_shop.as_mut().ok_or(Error::<T>::ShopNotFound)?;
                ensure!(shop.status == ShopStatus::Active, Error::<T>::InvalidShopStatus);

                shop.status = ShopStatus::Suspended;
                Ok(())
            })?;

            ShopStats::<T>::mutate(|stats| {
                stats.active_shops = stats.active_shops.saturating_sub(1);
            });

            Self::deposit_event(Event::ShopStatusChanged {
                shop_id,
                status: ShopStatus::Suspended,
            });
            Ok(())
        }

        /// 恢复店铺（治理，需资金充足）
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(20_000, 0))]
        pub fn resume_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            // 检查资金是否充足
            let shop_account = Self::shop_account(shop_id);
            let balance = T::Currency::free_balance(&shop_account);
            let min_balance = T::MinOperatingBalance::get();
            ensure!(balance >= min_balance, Error::<T>::InsufficientOperatingFund);

            Shops::<T>::try_mutate(shop_id, |maybe_shop| -> DispatchResult {
                let shop = maybe_shop.as_mut().ok_or(Error::<T>::ShopNotFound)?;
                ensure!(shop.status == ShopStatus::Suspended, Error::<T>::InvalidShopStatus);

                shop.status = ShopStatus::Active;
                Ok(())
            })?;

            ShopStats::<T>::mutate(|stats| {
                stats.active_shops = stats.active_shops.saturating_add(1);
            });

            Self::deposit_event(Event::ShopStatusChanged {
                shop_id,
                status: ShopStatus::Active,
            });
            Ok(())
        }

        /// 封禁店铺（治理，可选没收资金）
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn ban_shop(
            origin: OriginFor<T>,
            shop_id: u64,
            confiscate_fund: bool,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            let shop = Shops::<T>::get(shop_id).ok_or(Error::<T>::ShopNotFound)?;

            let shop_account = Self::shop_account(shop_id);
            let balance = T::Currency::free_balance(&shop_account);

            if !balance.is_zero() {
                if confiscate_fund {
                    // 没收资金（销毁或转入国库，这里简化为销毁）
                    // 实际项目中应转入国库
                    let _ = T::Currency::slash(&shop_account, balance);
                    Self::deposit_event(Event::FundConfiscated { shop_id, amount: balance });
                } else {
                    // 退还给店主
                    T::Currency::transfer(
                        &shop_account,
                        &shop.owner,
                        balance,
                        ExistenceRequirement::AllowDeath,
                    )?;
                }
            }

            Shops::<T>::mutate(shop_id, |maybe_shop| {
                if let Some(s) = maybe_shop {
                    s.status = ShopStatus::Banned;
                }
            });

            if shop.status == ShopStatus::Active {
                ShopStats::<T>::mutate(|stats| {
                    stats.active_shops = stats.active_shops.saturating_sub(1);
                });
            }

            Self::deposit_event(Event::ShopBanned {
                shop_id,
                fund_confiscated: confiscate_fund,
            });
            Ok(())
        }
    }

    // ==================== 辅助函数 ====================

    impl<T: Config> Pallet<T> {
        /// 获取店铺派生账户
        pub fn shop_account(shop_id: u64) -> T::AccountId {
            SHOP_PALLET_ID.into_sub_account_truncating(shop_id)
        }

        /// 计算初始运营资金（USDT 等值 COS）
        /// 
        /// # 算法
        /// 1. 获取 COS/USDT 价格（精度 10^6）
        /// 2. 计算所需 COS 数量 = USDT 金额 * 10^12 / 价格
        /// 3. 限制在 [MinInitialFundCos, MaxInitialFundCos] 范围内
        pub fn calculate_initial_fund() -> Result<BalanceOf<T>, sp_runtime::DispatchError> {
            let price = T::PricingProvider::get_cos_usdt_price();
            ensure!(price > 0, Error::<T>::PriceUnavailable);

            let usdt_amount = T::InitialFundUsdt::get();

            // cos_amount = usdt_amount * 10^12 / price
            let cos_amount_u128 = (usdt_amount as u128)
                .checked_mul(1_000_000_000_000u128)
                .ok_or(Error::<T>::ArithmeticOverflow)?
                .checked_div(price as u128)
                .ok_or(Error::<T>::ArithmeticOverflow)?;

            let cos_amount: BalanceOf<T> = cos_amount_u128.saturated_into();

            let min_fund = T::MinInitialFundCos::get();
            let max_fund = T::MaxInitialFundCos::get();
            let final_fund = cos_amount.max(min_fund).min(max_fund);

            Ok(final_fund)
        }

        /// 获取资金健康状态
        pub fn get_fund_health(balance: BalanceOf<T>) -> FundHealth {
            let min_balance = T::MinOperatingBalance::get();
            let warning_threshold = T::FundWarningThreshold::get();

            if balance.is_zero() {
                FundHealth::Depleted
            } else if balance <= min_balance {
                FundHealth::Critical
            } else if balance <= warning_threshold {
                FundHealth::Warning
            } else {
                FundHealth::Healthy
            }
        }

        /// 获取店铺运营资金余额
        pub fn get_shop_fund_balance(shop_id: u64) -> BalanceOf<T> {
            let shop_account = Self::shop_account(shop_id);
            T::Currency::free_balance(&shop_account)
        }

        /// 扣除运营费用（供其他模块调用）
        pub fn deduct_operating_fee(
            shop_id: u64,
            fee: BalanceOf<T>,
            fee_type: FeeType,
        ) -> sp_runtime::DispatchResult {
            let shop_account = Self::shop_account(shop_id);
            let balance = T::Currency::free_balance(&shop_account);

            ensure!(balance >= fee, Error::<T>::InsufficientOperatingFund);

            // 销毁费用（实际项目中应转入国库或服务提供者）
            let _ = T::Currency::slash(&shop_account, fee);

            let new_balance = T::Currency::free_balance(&shop_account);
            let min_balance = T::MinOperatingBalance::get();
            let warning_threshold = T::FundWarningThreshold::get();

            // 检查资金健康状态
            if new_balance <= min_balance {
                // 低于最低余额，暂停店铺
                Shops::<T>::mutate(shop_id, |s| {
                    if let Some(shop) = s {
                        if shop.status == ShopStatus::Active {
                            shop.status = ShopStatus::Suspended;
                            ShopStats::<T>::mutate(|stats| {
                                stats.active_shops = stats.active_shops.saturating_sub(1);
                            });
                        }
                    }
                });
                Self::deposit_event(Event::ShopSuspendedLowFund {
                    shop_id,
                    current_balance: new_balance,
                    minimum_balance: min_balance,
                });
            } else if new_balance <= warning_threshold {
                // 发出预警
                Self::deposit_event(Event::FundWarning {
                    shop_id,
                    current_balance: new_balance,
                    warning_threshold,
                });
            }

            Self::deposit_event(Event::OperatingFeeDeducted {
                shop_id,
                fee,
                fee_type,
                remaining_balance: new_balance,
            });

            Ok(())
        }

        /// 获取当前初始资金金额（供前端查询）
        pub fn get_current_initial_fund() -> Result<BalanceOf<T>, sp_runtime::DispatchError> {
            Self::calculate_initial_fund()
        }

        /// 获取初始资金计算详情（供前端查询）
        pub fn get_initial_fund_details() -> (u64, u64, u128) {
            let price = T::PricingProvider::get_cos_usdt_price();
            let usdt_amount = T::InitialFundUsdt::get();

            let cos_amount = if price > 0 {
                (usdt_amount as u128)
                    .saturating_mul(1_000_000_000_000u128)
                    .checked_div(price as u128)
                    .unwrap_or(0)
            } else {
                0
            };

            (usdt_amount, price, cos_amount)
        }
    }

    // ==================== ShopProvider 实现 ====================

    impl<T: Config> ShopProvider<T::AccountId> for Pallet<T> {
        fn shop_exists(shop_id: u64) -> bool {
            Shops::<T>::contains_key(shop_id)
        }

        fn is_shop_active(shop_id: u64) -> bool {
            Shops::<T>::get(shop_id)
                .map(|s| s.status == ShopStatus::Active)
                .unwrap_or(false)
        }

        fn shop_owner(shop_id: u64) -> Option<T::AccountId> {
            Shops::<T>::get(shop_id).map(|s| s.owner)
        }

        fn shop_account(shop_id: u64) -> T::AccountId {
            Self::shop_account(shop_id)
        }

        fn update_shop_stats(shop_id: u64, sales_amount: u128, order_count: u32) -> Result<(), sp_runtime::DispatchError> {
            Shops::<T>::try_mutate(shop_id, |maybe_shop| -> Result<(), sp_runtime::DispatchError> {
                let shop = maybe_shop.as_mut().ok_or(Error::<T>::ShopNotFound)?;
                shop.total_sales = shop.total_sales.saturating_add(sales_amount.saturated_into());
                shop.total_orders = shop.total_orders.saturating_add(order_count);
                Ok(())
            })
        }

        fn update_shop_rating(shop_id: u64, rating: u8) -> Result<(), sp_runtime::DispatchError> {
            Shops::<T>::try_mutate(shop_id, |maybe_shop| -> Result<(), sp_runtime::DispatchError> {
                let shop = maybe_shop.as_mut().ok_or(Error::<T>::ShopNotFound)?;
                let total_rating = (shop.rating as u32)
                    .saturating_mul(shop.rating_count)
                    .saturating_add((rating as u32) * 100);
                shop.rating_count = shop.rating_count.saturating_add(1);
                shop.rating = (total_rating / shop.rating_count) as u16;
                Ok(())
            })
        }
    }
}
