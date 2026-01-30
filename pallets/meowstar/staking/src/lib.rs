//! # Meowstar Staking Pallet
//!
//! 喵星宇宙质押系统 Pallet
//!
//! ## 功能
//! - COS 代币质押
//! - 多档位锁定期
//! - 质押收益分发
//! - 投票权重计算

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency, LockableCurrency, LockIdentifier},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Saturating;

    const STAKING_ID: LockIdentifier = *b"meowstak";

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// 锁定期类型
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, codec::DecodeWithMemTracking)]
    pub enum LockPeriod {
        /// 灵活质押 (随时可取)
        #[default]
        Flexible,
        /// 30天锁定
        Days30,
        /// 90天锁定
        Days90,
        /// 180天锁定
        Days180,
        /// 365天锁定
        Days365,
    }

    impl LockPeriod {
        /// 获取锁定区块数 (假设 6 秒一个区块)
        pub fn lock_blocks(&self) -> u32 {
            match self {
                LockPeriod::Flexible => 0,
                LockPeriod::Days30 => 30 * 24 * 60 * 10,      // 432,000 blocks
                LockPeriod::Days90 => 90 * 24 * 60 * 10,      // 1,296,000 blocks
                LockPeriod::Days180 => 180 * 24 * 60 * 10,    // 2,592,000 blocks
                LockPeriod::Days365 => 365 * 24 * 60 * 10,    // 5,256,000 blocks
            }
        }

        /// 获取年化收益率 (百分比 * 100)
        pub fn apy(&self) -> u32 {
            match self {
                LockPeriod::Flexible => 500,   // 5%
                LockPeriod::Days30 => 1200,    // 12%
                LockPeriod::Days90 => 2000,    // 20%
                LockPeriod::Days180 => 3000,   // 30%
                LockPeriod::Days365 => 5000,   // 50%
            }
        }

        /// 获取投票权重倍数 (百分比 * 100)
        pub fn vote_weight(&self) -> u32 {
            match self {
                LockPeriod::Flexible => 100,   // 1.0x
                LockPeriod::Days30 => 120,     // 1.2x
                LockPeriod::Days90 => 150,     // 1.5x
                LockPeriod::Days180 => 200,    // 2.0x
                LockPeriod::Days365 => 300,    // 3.0x
            }
        }
    }

    /// 质押信息
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct StakeInfo<T: Config> {
        /// 质押金额
        pub amount: BalanceOf<T>,
        /// 锁定期类型
        pub lock_period: LockPeriod,
        /// 质押开始区块
        pub start_block: BlockNumberFor<T>,
        /// 解锁区块
        pub unlock_block: BlockNumberFor<T>,
        /// 已领取收益
        pub claimed_rewards: BalanceOf<T>,
        /// 上次领取区块
        pub last_claim_block: BlockNumberFor<T>,
    }

    /// 质押池信息
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct PoolInfo<Balance> {
        /// 总质押量
        pub total_staked: Balance,
        /// 总投票权重
        pub total_vote_weight: u128,
        /// 累计分发收益
        pub total_rewards_distributed: Balance,
        /// 质押者数量
        pub staker_count: u32,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 货币类型
        type Currency: Currency<Self::AccountId>
            + ReservableCurrency<Self::AccountId>
            + LockableCurrency<Self::AccountId>;

        /// 最小质押金额
        #[pallet::constant]
        type MinStakeAmount: Get<BalanceOf<Self>>;

        /// 每个账户最大质押数量
        #[pallet::constant]
        type MaxStakesPerAccount: Get<u32>;

        /// 奖励发放间隔 (区块数)
        #[pallet::constant]
        type RewardInterval: Get<BlockNumberFor<Self>>;

        /// 权重信息
        type WeightInfo: WeightInfo;
    }

    /// 用户质押信息
    #[pallet::storage]
    #[pallet::getter(fn stakes)]
    pub type Stakes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, T::AccountId,
        Blake2_128Concat, u32,  // stake_id
        StakeInfo<T>,
        OptionQuery,
    >;

    /// 用户质押数量
    #[pallet::storage]
    #[pallet::getter(fn stake_count)]
    pub type StakeCount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    /// 质押池信息
    #[pallet::storage]
    #[pallet::getter(fn pool_info)]
    pub type Pool<T: Config> = StorageValue<_, PoolInfo<BalanceOf<T>>, ValueQuery>;

    /// 奖励池余额
    #[pallet::storage]
    #[pallet::getter(fn reward_pool)]
    pub type RewardPool<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 质押成功
        Staked {
            who: T::AccountId,
            stake_id: u32,
            amount: BalanceOf<T>,
            lock_period: LockPeriod,
            unlock_block: BlockNumberFor<T>,
        },
        /// 解除质押
        Unstaked {
            who: T::AccountId,
            stake_id: u32,
            amount: BalanceOf<T>,
        },
        /// 领取收益
        RewardsClaimed {
            who: T::AccountId,
            stake_id: u32,
            amount: BalanceOf<T>,
        },
        /// 奖励池充值
        RewardPoolFunded {
            who: T::AccountId,
            amount: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 质押金额过低
        AmountTooLow,
        /// 达到最大质押数量
        MaxStakesReached,
        /// 质押不存在
        StakeNotFound,
        /// 尚未解锁
        NotUnlocked,
        /// 余额不足
        InsufficientBalance,
        /// 无可领取收益
        NoRewardsToClaim,
        /// 奖励池余额不足
        InsufficientRewardPool,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 质押 COS
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::stake())]
        pub fn stake(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
            lock_period: LockPeriod,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查最小质押金额
            ensure!(amount >= T::MinStakeAmount::get(), Error::<T>::AmountTooLow);

            // 检查质押数量限制
            let stake_count = StakeCount::<T>::get(&who);
            ensure!(
                stake_count < T::MaxStakesPerAccount::get(),
                Error::<T>::MaxStakesReached
            );

            // 锁定代币
            T::Currency::set_lock(
                STAKING_ID,
                &who,
                amount,
                frame_support::traits::WithdrawReasons::all(),
            );

            // 计算解锁区块
            let current_block = frame_system::Pallet::<T>::block_number();
            let lock_blocks: BlockNumberFor<T> = lock_period.lock_blocks().into();
            let unlock_block = current_block.saturating_add(lock_blocks);

            // 创建质押信息
            let stake_id = stake_count;
            let stake_info = StakeInfo {
                amount,
                lock_period: lock_period.clone(),
                start_block: current_block,
                unlock_block,
                claimed_rewards: 0u32.into(),
                last_claim_block: current_block,
            };

            // 存储质押信息
            Stakes::<T>::insert(&who, stake_id, stake_info);
            StakeCount::<T>::insert(&who, stake_count.saturating_add(1));

            // 更新质押池
            Pool::<T>::mutate(|pool| {
                pool.total_staked = pool.total_staked.saturating_add(amount);
                pool.total_vote_weight = pool.total_vote_weight.saturating_add(
                    Self::calculate_vote_weight(amount, &lock_period)
                );
                pool.staker_count = pool.staker_count.saturating_add(1);
            });

            Self::deposit_event(Event::Staked {
                who,
                stake_id,
                amount,
                lock_period,
                unlock_block,
            });

            Ok(())
        }

        /// 解除质押
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::unstake())]
        pub fn unstake(origin: OriginFor<T>, stake_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let stake_info = Stakes::<T>::get(&who, stake_id)
                .ok_or(Error::<T>::StakeNotFound)?;

            // 检查是否已解锁
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block >= stake_info.unlock_block,
                Error::<T>::NotUnlocked
            );

            // 先领取剩余收益
            let pending_rewards = Self::calculate_pending_rewards(&stake_info, current_block);
            if pending_rewards > 0u32.into() {
                Self::do_claim_rewards(&who, stake_id, pending_rewards)?;
            }

            // 解锁代币
            T::Currency::remove_lock(STAKING_ID, &who);

            // 更新质押池
            Pool::<T>::mutate(|pool| {
                pool.total_staked = pool.total_staked.saturating_sub(stake_info.amount);
                pool.total_vote_weight = pool.total_vote_weight.saturating_sub(
                    Self::calculate_vote_weight(stake_info.amount, &stake_info.lock_period)
                );
                pool.staker_count = pool.staker_count.saturating_sub(1);
            });

            // 删除质押记录
            Stakes::<T>::remove(&who, stake_id);

            Self::deposit_event(Event::Unstaked {
                who,
                stake_id,
                amount: stake_info.amount,
            });

            Ok(())
        }

        /// 领取收益
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::claim_rewards())]
        pub fn claim_rewards(origin: OriginFor<T>, stake_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let stake_info = Stakes::<T>::get(&who, stake_id)
                .ok_or(Error::<T>::StakeNotFound)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            let pending_rewards = Self::calculate_pending_rewards(&stake_info, current_block);

            ensure!(pending_rewards > 0u32.into(), Error::<T>::NoRewardsToClaim);

            Self::do_claim_rewards(&who, stake_id, pending_rewards)?;

            Ok(())
        }

        /// 充值奖励池
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::fund_reward_pool())]
        pub fn fund_reward_pool(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            T::Currency::reserve(&who, amount)?;
            RewardPool::<T>::mutate(|pool| *pool = pool.saturating_add(amount));

            Self::deposit_event(Event::RewardPoolFunded { who, amount });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 计算投票权重
        fn calculate_vote_weight(amount: BalanceOf<T>, lock_period: &LockPeriod) -> u128 {
            let amount_u128: u128 = amount.try_into().unwrap_or(0);
            let weight_multiplier = lock_period.vote_weight() as u128;
            amount_u128.saturating_mul(weight_multiplier) / 100
        }

        /// 计算待领取收益
        fn calculate_pending_rewards(
            stake_info: &StakeInfo<T>,
            current_block: BlockNumberFor<T>,
        ) -> BalanceOf<T> {
            let blocks_since_claim = current_block.saturating_sub(stake_info.last_claim_block);
            let blocks_u32: u32 = blocks_since_claim.try_into().unwrap_or(0);
            
            // 年化收益率
            let apy = stake_info.lock_period.apy();
            
            // 计算收益: amount * apy * blocks / (blocks_per_year * 10000)
            // blocks_per_year ≈ 5,256,000
            let amount_u128: u128 = stake_info.amount.try_into().unwrap_or(0);
            let rewards = amount_u128
                .saturating_mul(apy as u128)
                .saturating_mul(blocks_u32 as u128)
                / (5_256_000u128 * 10_000);
            
            rewards.try_into().unwrap_or(0u32.into())
        }

        /// 执行领取收益
        fn do_claim_rewards(
            who: &T::AccountId,
            stake_id: u32,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            // 检查奖励池余额
            let reward_pool = RewardPool::<T>::get();
            ensure!(reward_pool >= amount, Error::<T>::InsufficientRewardPool);

            // 从奖励池转账
            RewardPool::<T>::mutate(|pool| *pool = pool.saturating_sub(amount));
            T::Currency::deposit_into_existing(who, amount)?;

            // 更新质押信息
            Stakes::<T>::mutate(who, stake_id, |maybe_stake| {
                if let Some(stake) = maybe_stake {
                    stake.claimed_rewards = stake.claimed_rewards.saturating_add(amount);
                    stake.last_claim_block = frame_system::Pallet::<T>::block_number();
                }
            });

            // 更新质押池统计
            Pool::<T>::mutate(|pool| {
                pool.total_rewards_distributed = pool.total_rewards_distributed.saturating_add(amount);
            });

            Self::deposit_event(Event::RewardsClaimed {
                who: who.clone(),
                stake_id,
                amount,
            });

            Ok(())
        }

        /// 获取用户总投票权重
        pub fn get_vote_power(who: &T::AccountId) -> u128 {
            let stake_count = StakeCount::<T>::get(who);
            let mut total_power = 0u128;

            for stake_id in 0..stake_count {
                if let Some(stake_info) = Stakes::<T>::get(who, stake_id) {
                    total_power = total_power.saturating_add(
                        Self::calculate_vote_weight(stake_info.amount, &stake_info.lock_period)
                    );
                }
            }

            total_power
        }

        /// 获取用户总质押量
        pub fn get_total_staked(who: &T::AccountId) -> BalanceOf<T> {
            let stake_count = StakeCount::<T>::get(who);
            let mut total: BalanceOf<T> = 0u32.into();

            for stake_id in 0..stake_count {
                if let Some(stake_info) = Stakes::<T>::get(who, stake_id) {
                    total = total.saturating_add(stake_info.amount);
                }
            }

            total
        }
    }
}
