//! # 实体代币发售模块 (pallet-entity-tokensale)
//!
//! ## 概述
//!
//! 本模块实现实体代币公开发售功能：
//! - 多种发售模式（固定价格、荷兰拍卖、白名单分配）
//! - 多轮发售支持
//! - 代币锁仓和线性解锁
//! - 多资产支付支持
//! - KYC 集成
//!
//! ## 发售模式
//!
//! - **FixedPrice**: 固定价格发售
//! - **DutchAuction**: 荷兰拍卖（价格递减）
//! - **WhitelistAllocation**: 白名单定向分配
//! - **FCFS**: 先到先得
//! - **Lottery**: 抽签发售
//!
//! ## 版本历史
//!
//! - v0.1.0 (2026-02-03): Phase 8 初始版本

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

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
        traits::Get,
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Saturating, Zero};

    // ==================== 类型定义 ====================

    /// 发售模式
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub enum SaleMode {
        /// 固定价格
        #[default]
        FixedPrice,
        /// 荷兰拍卖（价格递减）
        DutchAuction,
        /// 白名单分配
        WhitelistAllocation,
        /// 先到先得
        FCFS,
        /// 抽签发售
        Lottery,
    }

    /// 发售轮次状态
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub enum RoundStatus {
        /// 未开始
        #[default]
        NotStarted,
        /// 白名单注册中
        WhitelistOpen,
        /// 发售进行中
        Active,
        /// 已售罄
        SoldOut,
        /// 已结束
        Ended,
        /// 已取消
        Cancelled,
        /// 结算中
        Settling,
        /// 已完成
        Completed,
    }

    /// 锁仓类型
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub enum VestingType {
        /// 无锁仓
        #[default]
        None,
        /// 线性解锁
        Linear,
        /// 阶梯解锁
        Cliff,
        /// 自定义解锁
        Custom,
    }

    /// 锁仓配置
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub struct VestingConfig<BlockNumber> {
        /// 锁仓类型
        pub vesting_type: VestingType,
        /// 初始解锁比例（基点，如 1000 = 10%）
        pub initial_unlock_bps: u16,
        /// 悬崖期（区块数）
        pub cliff_duration: BlockNumber,
        /// 总解锁期（区块数）
        pub total_duration: BlockNumber,
        /// 解锁间隔（区块数，用于阶梯解锁）
        pub unlock_interval: BlockNumber,
    }

    /// 支付配置
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct PaymentConfig<AssetId, Balance> {
        /// 支付资产 ID（None = 原生代币）
        pub asset_id: Option<AssetId>,
        /// 单价
        pub price: Balance,
        /// 最小购买量
        pub min_purchase: Balance,
        /// 最大购买量（每人）
        pub max_purchase_per_account: Balance,
        /// 是否启用
        pub enabled: bool,
    }

    /// 发售轮次
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    #[scale_info(skip_type_params(MaxPaymentOptions, MaxWhitelistSize))]
    pub struct SaleRound<AccountId, Balance, BlockNumber, AssetId, MaxPaymentOptions: Get<u32>, MaxWhitelistSize: Get<u32>> {
        /// 轮次 ID
        pub id: u64,
        /// 实体 ID
        pub entity_id: u64,
        /// 发售模式
        pub mode: SaleMode,
        /// 状态
        pub status: RoundStatus,
        /// 代币总量
        pub total_supply: Balance,
        /// 已售数量
        pub sold_amount: Balance,
        /// 剩余数量
        pub remaining_amount: Balance,
        /// 参与人数
        pub participants_count: u32,
        /// 支付选项
        pub payment_options: BoundedVec<PaymentConfig<AssetId, Balance>, MaxPaymentOptions>,
        /// 锁仓配置
        pub vesting_config: VestingConfig<BlockNumber>,
        /// 白名单（如果适用）
        pub whitelist: BoundedVec<AccountId, MaxWhitelistSize>,
        /// 是否需要 KYC
        pub kyc_required: bool,
        /// 最低 KYC 级别（0-4）
        pub min_kyc_level: u8,
        /// 开始时间
        pub start_block: BlockNumber,
        /// 结束时间
        pub end_block: BlockNumber,
        /// 荷兰拍卖起始价格
        pub dutch_start_price: Option<Balance>,
        /// 荷兰拍卖结束价格
        pub dutch_end_price: Option<Balance>,
        /// 创建者
        pub creator: AccountId,
        /// 创建时间
        pub created_at: BlockNumber,
    }

    /// 发售轮次类型别名
    pub type SaleRoundOf<T> = SaleRound<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        AssetIdOf<T>,
        <T as Config>::MaxPaymentOptions,
        <T as Config>::MaxWhitelistSize,
    >;

    /// 认购记录
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct Subscription<AccountId, Balance, BlockNumber, AssetId> {
        /// 认购者
        pub subscriber: AccountId,
        /// 轮次 ID
        pub round_id: u64,
        /// 认购数量
        pub amount: Balance,
        /// 支付资产
        pub payment_asset: Option<AssetId>,
        /// 支付金额
        pub payment_amount: Balance,
        /// 认购时间
        pub subscribed_at: BlockNumber,
        /// 是否已领取
        pub claimed: bool,
        /// 已解锁数量
        pub unlocked_amount: Balance,
        /// 上次解锁时间
        pub last_unlock_at: BlockNumber,
    }

    /// 认购记录类型别名
    pub type SubscriptionOf<T> = Subscription<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        AssetIdOf<T>,
    >;

    /// 余额类型别名
    pub type BalanceOf<T> = <T as Config>::Balance;
    /// 资产 ID 类型别名
    pub type AssetIdOf<T> = <T as Config>::AssetId;

    // ==================== 配置 ====================

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 余额类型
        type Balance: Member
            + Parameter
            + sp_runtime::traits::AtLeast32BitUnsigned
            + Default
            + Copy
            + MaxEncodedLen
            + From<u128>
            + Into<u128>;

        /// 资产 ID 类型
        type AssetId: Member
            + Parameter
            + Default
            + Copy
            + MaxEncodedLen
            + From<u64>
            + Into<u64>;

        /// 最大支付选项数
        #[pallet::constant]
        type MaxPaymentOptions: Get<u32>;

        /// 最大白名单大小
        #[pallet::constant]
        type MaxWhitelistSize: Get<u32>;

        /// 最大历史轮次数
        #[pallet::constant]
        type MaxRoundsHistory: Get<u32>;

        /// 最大认购记录数（每人每轮）
        #[pallet::constant]
        type MaxSubscriptionsPerRound: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ==================== 存储项 ====================

    /// 下一个轮次 ID
    #[pallet::storage]
    #[pallet::getter(fn next_round_id)]
    pub type NextRoundId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 发售轮次存储
    #[pallet::storage]
    #[pallet::getter(fn sale_rounds)]
    pub type SaleRounds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // round_id
        SaleRoundOf<T>,
    >;

    /// 实体发售轮次索引
    #[pallet::storage]
    #[pallet::getter(fn entity_rounds)]
    pub type EntityRounds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // entity_id
        BoundedVec<u64, T::MaxRoundsHistory>,
        ValueQuery,
    >;

    /// 认购记录 (round_id, subscriber) -> Subscription
    #[pallet::storage]
    #[pallet::getter(fn subscriptions)]
    pub type Subscriptions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,  // round_id
        Blake2_128Concat,
        T::AccountId,
        SubscriptionOf<T>,
    >;

    /// 轮次参与者列表
    #[pallet::storage]
    #[pallet::getter(fn round_participants)]
    pub type RoundParticipants<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // round_id
        BoundedVec<T::AccountId, T::MaxSubscriptionsPerRound>,
        ValueQuery,
    >;

    /// 已募集资金 (round_id, asset_id) -> amount
    #[pallet::storage]
    #[pallet::getter(fn raised_funds)]
    pub type RaisedFunds<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,  // round_id
        Blake2_128Concat,
        Option<AssetIdOf<T>>,  // None = native
        BalanceOf<T>,
        ValueQuery,
    >;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 发售轮次已创建
        SaleRoundCreated {
            round_id: u64,
            entity_id: u64,
            mode: SaleMode,
            total_supply: BalanceOf<T>,
        },
        /// 发售轮次已开始
        SaleRoundStarted {
            round_id: u64,
        },
        /// 发售轮次已结束
        SaleRoundEnded {
            round_id: u64,
            sold_amount: BalanceOf<T>,
            participants_count: u32,
        },
        /// 发售轮次已取消
        SaleRoundCancelled {
            round_id: u64,
        },
        /// 用户已认购
        Subscribed {
            round_id: u64,
            subscriber: T::AccountId,
            amount: BalanceOf<T>,
            payment_amount: BalanceOf<T>,
        },
        /// 代币已领取
        TokensClaimed {
            round_id: u64,
            subscriber: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 代币已解锁
        TokensUnlocked {
            round_id: u64,
            subscriber: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 白名单已更新
        WhitelistUpdated {
            round_id: u64,
            count: u32,
        },
        /// 募集资金已提取
        FundsWithdrawn {
            round_id: u64,
            amount: BalanceOf<T>,
            asset_id: Option<AssetIdOf<T>>,
        },
        /// 退款已处理
        RefundProcessed {
            round_id: u64,
            subscriber: T::AccountId,
            amount: BalanceOf<T>,
        },
    }

    // ==================== 错误 ====================

    #[pallet::error]
    pub enum Error<T> {
        /// 轮次不存在
        RoundNotFound,
        /// 轮次未开始
        RoundNotStarted,
        /// 轮次已结束
        RoundEnded,
        /// 轮次已取消
        RoundCancelled,
        /// 轮次已售罄
        SoldOut,
        /// 无效的轮次状态
        InvalidRoundStatus,
        /// 余额不足
        InsufficientBalance,
        /// 超过购买限额
        ExceedsPurchaseLimit,
        /// 低于最小购买量
        BelowMinPurchase,
        /// 不在白名单中
        NotInWhitelist,
        /// KYC 级别不足
        InsufficientKycLevel,
        /// 无效的支付资产
        InvalidPaymentAsset,
        /// 已认购
        AlreadySubscribed,
        /// 未认购
        NotSubscribed,
        /// 代币已领取
        AlreadyClaimed,
        /// 无可解锁代币
        NoTokensToUnlock,
        /// 悬崖期未到
        CliffNotReached,
        /// 无权限
        Unauthorized,
        /// 白名单已满
        WhitelistFull,
        /// 轮次历史已满
        RoundsHistoryFull,
        /// 参与者已满
        ParticipantsFull,
        /// 支付选项已满
        PaymentOptionsFull,
        /// 荷兰拍卖配置无效
        InvalidDutchAuctionConfig,
        /// 锁仓配置无效
        InvalidVestingConfig,
    }

    // ==================== Extrinsics ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建发售轮次
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(80_000, 0))]
        pub fn create_sale_round(
            origin: OriginFor<T>,
            entity_id: u64,
            mode: SaleMode,
            total_supply: BalanceOf<T>,
            start_block: BlockNumberFor<T>,
            end_block: BlockNumberFor<T>,
            kyc_required: bool,
            min_kyc_level: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let now = <frame_system::Pallet<T>>::block_number();
            let round_id = NextRoundId::<T>::get();

            let round = SaleRound {
                id: round_id,
                entity_id,
                mode,
                status: RoundStatus::NotStarted,
                total_supply,
                sold_amount: Zero::zero(),
                remaining_amount: total_supply,
                participants_count: 0,
                payment_options: BoundedVec::default(),
                vesting_config: VestingConfig::default(),
                whitelist: BoundedVec::default(),
                kyc_required,
                min_kyc_level,
                start_block,
                end_block,
                dutch_start_price: None,
                dutch_end_price: None,
                creator: who.clone(),
                created_at: now,
            };

            SaleRounds::<T>::insert(round_id, round);
            NextRoundId::<T>::put(round_id.saturating_add(1));

            EntityRounds::<T>::try_mutate(entity_id, |rounds| -> DispatchResult {
                rounds.try_push(round_id).map_err(|_| Error::<T>::RoundsHistoryFull)?;
                Ok(())
            })?;

            Self::deposit_event(Event::SaleRoundCreated {
                round_id,
                entity_id,
                mode,
                total_supply,
            });
            Ok(())
        }

        /// 添加支付选项
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn add_payment_option(
            origin: OriginFor<T>,
            round_id: u64,
            asset_id: Option<AssetIdOf<T>>,
            price: BalanceOf<T>,
            min_purchase: BalanceOf<T>,
            max_purchase_per_account: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            SaleRounds::<T>::try_mutate(round_id, |maybe_round| -> DispatchResult {
                let round = maybe_round.as_mut().ok_or(Error::<T>::RoundNotFound)?;
                ensure!(round.creator == who, Error::<T>::Unauthorized);
                ensure!(round.status == RoundStatus::NotStarted, Error::<T>::InvalidRoundStatus);

                let option = PaymentConfig {
                    asset_id,
                    price,
                    min_purchase,
                    max_purchase_per_account,
                    enabled: true,
                };

                round.payment_options.try_push(option).map_err(|_| Error::<T>::PaymentOptionsFull)?;
                Ok(())
            })
        }

        /// 设置锁仓配置
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn set_vesting_config(
            origin: OriginFor<T>,
            round_id: u64,
            vesting_type: VestingType,
            initial_unlock_bps: u16,
            cliff_duration: BlockNumberFor<T>,
            total_duration: BlockNumberFor<T>,
            unlock_interval: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            SaleRounds::<T>::try_mutate(round_id, |maybe_round| -> DispatchResult {
                let round = maybe_round.as_mut().ok_or(Error::<T>::RoundNotFound)?;
                ensure!(round.creator == who, Error::<T>::Unauthorized);
                ensure!(round.status == RoundStatus::NotStarted, Error::<T>::InvalidRoundStatus);

                // 验证配置
                ensure!(initial_unlock_bps <= 10000, Error::<T>::InvalidVestingConfig);

                round.vesting_config = VestingConfig {
                    vesting_type,
                    initial_unlock_bps,
                    cliff_duration,
                    total_duration,
                    unlock_interval,
                };
                Ok(())
            })
        }

        /// 配置荷兰拍卖
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(25_000, 0))]
        pub fn configure_dutch_auction(
            origin: OriginFor<T>,
            round_id: u64,
            start_price: BalanceOf<T>,
            end_price: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            SaleRounds::<T>::try_mutate(round_id, |maybe_round| -> DispatchResult {
                let round = maybe_round.as_mut().ok_or(Error::<T>::RoundNotFound)?;
                ensure!(round.creator == who, Error::<T>::Unauthorized);
                ensure!(round.mode == SaleMode::DutchAuction, Error::<T>::InvalidRoundStatus);
                ensure!(start_price > end_price, Error::<T>::InvalidDutchAuctionConfig);

                round.dutch_start_price = Some(start_price);
                round.dutch_end_price = Some(end_price);
                Ok(())
            })
        }

        /// 添加白名单
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(50_000, 0))]
        pub fn add_to_whitelist(
            origin: OriginFor<T>,
            round_id: u64,
            accounts: Vec<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            SaleRounds::<T>::try_mutate(round_id, |maybe_round| -> DispatchResult {
                let round = maybe_round.as_mut().ok_or(Error::<T>::RoundNotFound)?;
                ensure!(round.creator == who, Error::<T>::Unauthorized);

                for account in accounts {
                    if !round.whitelist.contains(&account) {
                        round.whitelist.try_push(account).map_err(|_| Error::<T>::WhitelistFull)?;
                    }
                }

                let count = round.whitelist.len() as u32;
                Self::deposit_event(Event::WhitelistUpdated { round_id, count });
                Ok(())
            })
        }

        /// 开始发售
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn start_sale(
            origin: OriginFor<T>,
            round_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            SaleRounds::<T>::try_mutate(round_id, |maybe_round| -> DispatchResult {
                let round = maybe_round.as_mut().ok_or(Error::<T>::RoundNotFound)?;
                ensure!(round.creator == who, Error::<T>::Unauthorized);
                ensure!(round.status == RoundStatus::NotStarted, Error::<T>::InvalidRoundStatus);

                round.status = RoundStatus::Active;
                Self::deposit_event(Event::SaleRoundStarted { round_id });
                Ok(())
            })
        }

        /// 认购
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(60_000, 0))]
        pub fn subscribe(
            origin: OriginFor<T>,
            round_id: u64,
            amount: BalanceOf<T>,
            payment_asset: Option<AssetIdOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let round = SaleRounds::<T>::get(round_id).ok_or(Error::<T>::RoundNotFound)?;

            // 验证状态
            ensure!(round.status == RoundStatus::Active, Error::<T>::InvalidRoundStatus);
            ensure!(round.remaining_amount >= amount, Error::<T>::SoldOut);

            // 验证未重复认购
            ensure!(!Subscriptions::<T>::contains_key(round_id, &who), Error::<T>::AlreadySubscribed);

            // 验证白名单（如果是白名单模式）
            if round.mode == SaleMode::WhitelistAllocation {
                ensure!(round.whitelist.contains(&who), Error::<T>::NotInWhitelist);
            }

            // 查找支付选项
            let payment_option = round.payment_options.iter()
                .find(|o| o.asset_id == payment_asset && o.enabled)
                .ok_or(Error::<T>::InvalidPaymentAsset)?;

            // 验证购买限额
            ensure!(amount >= payment_option.min_purchase, Error::<T>::BelowMinPurchase);
            ensure!(amount <= payment_option.max_purchase_per_account, Error::<T>::ExceedsPurchaseLimit);

            // 计算支付金额
            let payment_amount = Self::calculate_payment_amount(&round, amount, payment_option)?;

            let now = <frame_system::Pallet<T>>::block_number();

            // 创建认购记录
            let subscription = Subscription {
                subscriber: who.clone(),
                round_id,
                amount,
                payment_asset,
                payment_amount,
                subscribed_at: now,
                claimed: false,
                unlocked_amount: Zero::zero(),
                last_unlock_at: now,
            };

            Subscriptions::<T>::insert(round_id, &who, subscription);

            // 更新轮次数据
            SaleRounds::<T>::mutate(round_id, |maybe_round| {
                if let Some(round) = maybe_round {
                    round.sold_amount = round.sold_amount.saturating_add(amount);
                    round.remaining_amount = round.remaining_amount.saturating_sub(amount);
                    round.participants_count = round.participants_count.saturating_add(1);
                }
            });

            // 添加到参与者列表
            RoundParticipants::<T>::try_mutate(round_id, |participants| -> DispatchResult {
                participants.try_push(who.clone()).map_err(|_| Error::<T>::ParticipantsFull)?;
                Ok(())
            })?;

            // 更新募集资金
            RaisedFunds::<T>::mutate(round_id, payment_asset, |funds| {
                *funds = funds.saturating_add(payment_amount);
            });

            Self::deposit_event(Event::Subscribed {
                round_id,
                subscriber: who,
                amount,
                payment_amount,
            });
            Ok(())
        }

        /// 结束发售
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn end_sale(
            origin: OriginFor<T>,
            round_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            SaleRounds::<T>::try_mutate(round_id, |maybe_round| -> DispatchResult {
                let round = maybe_round.as_mut().ok_or(Error::<T>::RoundNotFound)?;
                ensure!(round.creator == who, Error::<T>::Unauthorized);
                ensure!(round.status == RoundStatus::Active, Error::<T>::InvalidRoundStatus);

                round.status = RoundStatus::Ended;

                Self::deposit_event(Event::SaleRoundEnded {
                    round_id,
                    sold_amount: round.sold_amount,
                    participants_count: round.participants_count,
                });
                Ok(())
            })
        }

        /// 领取代币（初始解锁部分）
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn claim_tokens(
            origin: OriginFor<T>,
            round_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let round = SaleRounds::<T>::get(round_id).ok_or(Error::<T>::RoundNotFound)?;
            ensure!(
                round.status == RoundStatus::Ended || round.status == RoundStatus::Completed,
                Error::<T>::InvalidRoundStatus
            );

            Subscriptions::<T>::try_mutate(round_id, &who, |maybe_sub| -> DispatchResult {
                let sub = maybe_sub.as_mut().ok_or(Error::<T>::NotSubscribed)?;
                ensure!(!sub.claimed, Error::<T>::AlreadyClaimed);

                // 计算初始解锁量
                let initial_unlock = Self::calculate_initial_unlock(&round.vesting_config, sub.amount);

                sub.claimed = true;
                sub.unlocked_amount = initial_unlock;

                Self::deposit_event(Event::TokensClaimed {
                    round_id,
                    subscriber: who.clone(),
                    amount: initial_unlock,
                });
                Ok(())
            })
        }

        /// 解锁代币（锁仓期后）
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn unlock_tokens(
            origin: OriginFor<T>,
            round_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let round = SaleRounds::<T>::get(round_id).ok_or(Error::<T>::RoundNotFound)?;

            Subscriptions::<T>::try_mutate(round_id, &who, |maybe_sub| -> DispatchResult {
                let sub = maybe_sub.as_mut().ok_or(Error::<T>::NotSubscribed)?;
                ensure!(sub.claimed, Error::<T>::NotSubscribed);

                let now = <frame_system::Pallet<T>>::block_number();

                // 计算可解锁量
                let unlockable = Self::calculate_unlockable(
                    &round.vesting_config,
                    sub.amount,
                    sub.unlocked_amount,
                    sub.subscribed_at,
                    now,
                )?;

                ensure!(!unlockable.is_zero(), Error::<T>::NoTokensToUnlock);

                sub.unlocked_amount = sub.unlocked_amount.saturating_add(unlockable);
                sub.last_unlock_at = now;

                Self::deposit_event(Event::TokensUnlocked {
                    round_id,
                    subscriber: who.clone(),
                    amount: unlockable,
                });
                Ok(())
            })
        }

        /// 取消发售
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn cancel_sale(
            origin: OriginFor<T>,
            round_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            SaleRounds::<T>::try_mutate(round_id, |maybe_round| -> DispatchResult {
                let round = maybe_round.as_mut().ok_or(Error::<T>::RoundNotFound)?;
                ensure!(round.creator == who, Error::<T>::Unauthorized);
                ensure!(
                    round.status == RoundStatus::NotStarted || round.status == RoundStatus::Active,
                    Error::<T>::InvalidRoundStatus
                );

                round.status = RoundStatus::Cancelled;
                Self::deposit_event(Event::SaleRoundCancelled { round_id });
                Ok(())
            })
        }
    }

    // ==================== 辅助函数 ====================

    impl<T: Config> Pallet<T> {
        /// 计算支付金额
        fn calculate_payment_amount(
            round: &SaleRoundOf<T>,
            amount: BalanceOf<T>,
            payment_option: &PaymentConfig<AssetIdOf<T>, BalanceOf<T>>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let price = if round.mode == SaleMode::DutchAuction {
                Self::calculate_dutch_price(round)?
            } else {
                payment_option.price
            };

            Ok(amount.saturating_mul(price))
        }

        /// 计算荷兰拍卖当前价格
        fn calculate_dutch_price(round: &SaleRoundOf<T>) -> Result<BalanceOf<T>, DispatchError> {
            let start_price = round.dutch_start_price.ok_or(Error::<T>::InvalidDutchAuctionConfig)?;
            let end_price = round.dutch_end_price.ok_or(Error::<T>::InvalidDutchAuctionConfig)?;

            let now = <frame_system::Pallet<T>>::block_number();
            let start = round.start_block;
            let end = round.end_block;

            if now <= start {
                return Ok(start_price);
            }
            if now >= end {
                return Ok(end_price);
            }

            // 线性递减
            let total_duration: u128 = Self::block_to_u128(end.saturating_sub(start));
            let elapsed: u128 = Self::block_to_u128(now.saturating_sub(start));
            let price_range: u128 = (start_price - end_price).into();

            let price_drop = price_range.saturating_mul(elapsed) / total_duration;
            let current_price: u128 = start_price.into() - price_drop;

            Ok(current_price.into())
        }

        /// 计算初始解锁量
        fn calculate_initial_unlock(
            vesting: &VestingConfig<BlockNumberFor<T>>,
            total: BalanceOf<T>,
        ) -> BalanceOf<T> {
            if vesting.vesting_type == VestingType::None {
                return total;
            }

            let initial_bps: u128 = vesting.initial_unlock_bps.into();
            let total_u128: u128 = total.into();
            let initial: u128 = total_u128.saturating_mul(initial_bps) / 10000;

            initial.into()
        }

        /// 计算可解锁量
        fn calculate_unlockable(
            vesting: &VestingConfig<BlockNumberFor<T>>,
            total: BalanceOf<T>,
            already_unlocked: BalanceOf<T>,
            start: BlockNumberFor<T>,
            now: BlockNumberFor<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            if vesting.vesting_type == VestingType::None {
                return Ok(total.saturating_sub(already_unlocked));
            }

            // 检查悬崖期
            let cliff_end = start.saturating_add(vesting.cliff_duration);
            if now < cliff_end {
                return Err(Error::<T>::CliffNotReached.into());
            }

            let total_end = start.saturating_add(vesting.total_duration);
            
            if now >= total_end {
                // 全部解锁
                return Ok(total.saturating_sub(already_unlocked));
            }

            // 计算线性解锁
            let vesting_duration: u128 = Self::block_to_u128(vesting.total_duration.saturating_sub(vesting.cliff_duration));
            let elapsed: u128 = Self::block_to_u128(now.saturating_sub(cliff_end));
            
            let initial_bps: u128 = vesting.initial_unlock_bps.into();
            let vesting_bps: u128 = 10000u128.saturating_sub(initial_bps);
            
            let total_u128: u128 = total.into();
            let vesting_amount = total_u128.saturating_mul(vesting_bps) / 10000;
            
            let unlocked_vesting = if vesting_duration > 0 {
                vesting_amount.saturating_mul(elapsed) / vesting_duration
            } else {
                vesting_amount
            };

            let initial_amount = total_u128.saturating_mul(initial_bps) / 10000;
            let total_unlockable: u128 = initial_amount.saturating_add(unlocked_vesting);

            let unlockable = BalanceOf::<T>::from(total_unlockable).saturating_sub(already_unlocked);
            Ok(unlockable)
        }

        /// 获取轮次当前价格
        pub fn get_current_price(round_id: u64, asset_id: Option<AssetIdOf<T>>) -> Option<BalanceOf<T>> {
            let round = SaleRounds::<T>::get(round_id)?;
            
            if round.mode == SaleMode::DutchAuction {
                Self::calculate_dutch_price(&round).ok()
            } else {
                round.payment_options.iter()
                    .find(|o| o.asset_id == asset_id && o.enabled)
                    .map(|o| o.price)
            }
        }

        /// 获取用户认购信息
        pub fn get_subscription(round_id: u64, account: &T::AccountId) -> Option<SubscriptionOf<T>> {
            Subscriptions::<T>::get(round_id, account)
        }

        /// BlockNumber 转 u128
        fn block_to_u128(block: BlockNumberFor<T>) -> u128 {
            use sp_runtime::traits::UniqueSaturatedInto;
            block.unique_saturated_into()
        }

        /// 获取用户可解锁代币数量
        pub fn get_unlockable_amount(round_id: u64, account: &T::AccountId) -> BalanceOf<T> {
            let round = match SaleRounds::<T>::get(round_id) {
                Some(r) => r,
                None => return Zero::zero(),
            };

            let sub = match Subscriptions::<T>::get(round_id, account) {
                Some(s) => s,
                None => return Zero::zero(),
            };

            if !sub.claimed {
                return Zero::zero();
            }

            let now = <frame_system::Pallet<T>>::block_number();
            Self::calculate_unlockable(
                &round.vesting_config,
                sub.amount,
                sub.unlocked_amount,
                sub.subscribed_at,
                now,
            ).unwrap_or(Zero::zero())
        }
    }
}
