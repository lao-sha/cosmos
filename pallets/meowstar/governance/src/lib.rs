//! # Meowstar Governance Pallet
//!
//! 喵星宇宙 DAO 治理系统 Pallet
//!
//! ## 功能
//! - 提案创建和投票
//! - 基于质押权重的投票
//! - 国库管理
//! - 紧急暂停机制

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Saturating;
    use sp_std::vec::Vec;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// 提案状态
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, codec::DecodeWithMemTracking)]
    pub enum ProposalStatus {
        /// 投票中
        #[default]
        Active,
        /// 已通过
        Passed,
        /// 已拒绝
        Rejected,
        /// 已执行
        Executed,
        /// 已取消
        Cancelled,
    }

    /// 提案类型
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, codec::DecodeWithMemTracking)]
    pub enum ProposalType {
        /// 普通提案
        #[default]
        General,
        /// 参数修改
        ParameterChange,
        /// 国库支出
        TreasurySpend,
        /// 紧急提案
        Emergency,
    }

    /// 提案信息
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Proposal<T: Config> {
        /// 提案ID
        pub id: u32,
        /// 提案者
        pub proposer: T::AccountId,
        /// 提案类型
        pub proposal_type: ProposalType,
        /// 标题 (哈希)
        pub title_hash: [u8; 32],
        /// 描述 (哈希)
        pub description_hash: [u8; 32],
        /// 赞成票权重
        pub yes_votes: u128,
        /// 反对票权重
        pub no_votes: u128,
        /// 状态
        pub status: ProposalStatus,
        /// 创建区块
        pub created_at: BlockNumberFor<T>,
        /// 投票截止区块
        pub voting_end: BlockNumberFor<T>,
        /// 执行延迟区块
        pub execution_delay: BlockNumberFor<T>,
    }

    /// 投票记录
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, codec::DecodeWithMemTracking)]
    pub struct VoteRecord {
        /// 是否赞成
        pub approve: bool,
        /// 投票权重
        pub weight: u128,
    }

    /// 国库信息
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct TreasuryInfo<Balance> {
        /// 总余额
        pub balance: Balance,
        /// 已支出
        pub total_spent: Balance,
        /// 待支出
        pub pending_spend: Balance,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_meowstar_staking::Config {
        /// 事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 货币类型
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 创建提案所需最小质押权重
        #[pallet::constant]
        type MinProposalWeight: Get<u128>;

        /// 提案押金
        #[pallet::constant]
        type ProposalDeposit: Get<BalanceOf<Self>>;

        /// 投票期 (区块数)
        #[pallet::constant]
        type VotingPeriod: Get<BlockNumberFor<Self>>;

        /// 执行延迟 (区块数)
        #[pallet::constant]
        type ExecutionDelay: Get<BlockNumberFor<Self>>;

        /// 通过阈值 (百分比 * 100)
        #[pallet::constant]
        type PassThreshold: Get<u32>;

        /// 最低参与率 (百分比 * 100)
        #[pallet::constant]
        type QuorumThreshold: Get<u32>;

        /// 权重信息
        type GovWeightInfo: WeightInfo;
    }

    /// 提案存储
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> = StorageMap<_, Blake2_128Concat, u32, Proposal<T>, OptionQuery>;

    /// 下一个提案ID
    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// 投票记录
    #[pallet::storage]
    #[pallet::getter(fn votes)]
    pub type Votes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u32,  // proposal_id
        Blake2_128Concat, T::AccountId,
        VoteRecord,
        OptionQuery,
    >;

    /// 国库信息
    #[pallet::storage]
    #[pallet::getter(fn treasury)]
    pub type Treasury<T: Config> = StorageValue<_, TreasuryInfo<BalanceOf<T>>, ValueQuery>;

    /// 紧急暂停状态
    #[pallet::storage]
    #[pallet::getter(fn is_paused)]
    pub type IsPaused<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 提案已创建
        ProposalCreated {
            proposal_id: u32,
            proposer: T::AccountId,
            proposal_type: ProposalType,
        },
        /// 已投票
        Voted {
            proposal_id: u32,
            voter: T::AccountId,
            approve: bool,
            weight: u128,
        },
        /// 提案已通过
        ProposalPassed {
            proposal_id: u32,
        },
        /// 提案已拒绝
        ProposalRejected {
            proposal_id: u32,
        },
        /// 提案已执行
        ProposalExecuted {
            proposal_id: u32,
        },
        /// 国库支出
        TreasurySpent {
            proposal_id: u32,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 紧急暂停
        EmergencyPaused {
            by: T::AccountId,
        },
        /// 解除暂停
        EmergencyUnpaused {
            by: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 投票权重不足
        InsufficientVoteWeight,
        /// 提案不存在
        ProposalNotFound,
        /// 提案不在投票期
        ProposalNotActive,
        /// 已经投过票
        AlreadyVoted,
        /// 投票期未结束
        VotingNotEnded,
        /// 提案未通过
        ProposalNotPassed,
        /// 执行延迟未到
        ExecutionDelayNotMet,
        /// 系统已暂停
        SystemPaused,
        /// 余额不足
        InsufficientBalance,
        /// 国库余额不足
        InsufficientTreasury,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建提案
        #[pallet::call_index(0)]
        #[pallet::weight(T::GovWeightInfo::create_proposal())]
        pub fn create_proposal(
            origin: OriginFor<T>,
            proposal_type: ProposalType,
            title_hash: [u8; 32],
            description_hash: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!IsPaused::<T>::get(), Error::<T>::SystemPaused);

            // 检查投票权重
            let vote_power = pallet_meowstar_staking::Pallet::<T>::get_vote_power(&who);
            ensure!(
                vote_power >= T::MinProposalWeight::get(),
                Error::<T>::InsufficientVoteWeight
            );

            // 收取押金
            let deposit = T::ProposalDeposit::get();
            <T as pallet::Config>::Currency::reserve(&who, deposit)?;

            // 创建提案
            let proposal_id = NextProposalId::<T>::get();
            let current_block = frame_system::Pallet::<T>::block_number();
            let voting_end = current_block.saturating_add(T::VotingPeriod::get());

            let proposal = Proposal {
                id: proposal_id,
                proposer: who.clone(),
                proposal_type: proposal_type.clone(),
                title_hash,
                description_hash,
                yes_votes: 0,
                no_votes: 0,
                status: ProposalStatus::Active,
                created_at: current_block,
                voting_end,
                execution_delay: T::ExecutionDelay::get(),
            };

            Proposals::<T>::insert(proposal_id, proposal);
            NextProposalId::<T>::put(proposal_id.saturating_add(1));

            Self::deposit_event(Event::ProposalCreated {
                proposal_id,
                proposer: who,
                proposal_type,
            });

            Ok(())
        }

        /// 投票
        #[pallet::call_index(1)]
        #[pallet::weight(T::GovWeightInfo::vote())]
        pub fn vote(
            origin: OriginFor<T>,
            proposal_id: u32,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!IsPaused::<T>::get(), Error::<T>::SystemPaused);

            // 检查提案状态
            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                proposal.status == ProposalStatus::Active,
                Error::<T>::ProposalNotActive
            );

            // 检查投票期
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block <= proposal.voting_end,
                Error::<T>::ProposalNotActive
            );

            // 检查是否已投票
            ensure!(
                !Votes::<T>::contains_key(proposal_id, &who),
                Error::<T>::AlreadyVoted
            );

            // 获取投票权重
            let vote_power = pallet_meowstar_staking::Pallet::<T>::get_vote_power(&who);
            ensure!(vote_power > 0, Error::<T>::InsufficientVoteWeight);

            // 记录投票
            if approve {
                proposal.yes_votes = proposal.yes_votes.saturating_add(vote_power);
            } else {
                proposal.no_votes = proposal.no_votes.saturating_add(vote_power);
            }

            Proposals::<T>::insert(proposal_id, proposal);
            Votes::<T>::insert(proposal_id, &who, VoteRecord {
                approve,
                weight: vote_power,
            });

            Self::deposit_event(Event::Voted {
                proposal_id,
                voter: who,
                approve,
                weight: vote_power,
            });

            Ok(())
        }

        /// 结算提案
        #[pallet::call_index(2)]
        #[pallet::weight(T::GovWeightInfo::finalize_proposal())]
        pub fn finalize_proposal(origin: OriginFor<T>, proposal_id: u32) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                proposal.status == ProposalStatus::Active,
                Error::<T>::ProposalNotActive
            );

            // 检查投票期是否结束
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block > proposal.voting_end,
                Error::<T>::VotingNotEnded
            );

            // 计算结果
            let total_votes = proposal.yes_votes.saturating_add(proposal.no_votes);
            let pool_info = pallet_meowstar_staking::Pallet::<T>::pool_info();
            let total_weight = pool_info.total_vote_weight;

            // 检查最低参与率
            let quorum = T::QuorumThreshold::get() as u128;
            let participation = if total_weight > 0 {
                total_votes.saturating_mul(10000) / total_weight
            } else {
                0
            };

            // 检查通过阈值
            let pass_threshold = T::PassThreshold::get() as u128;
            let approval_rate = if total_votes > 0 {
                proposal.yes_votes.saturating_mul(10000) / total_votes
            } else {
                0
            };

            if participation >= quorum && approval_rate >= pass_threshold {
                proposal.status = ProposalStatus::Passed;
                Self::deposit_event(Event::ProposalPassed { proposal_id });
            } else {
                proposal.status = ProposalStatus::Rejected;
                Self::deposit_event(Event::ProposalRejected { proposal_id });
            }

            // 退还押金
            let deposit = T::ProposalDeposit::get();
            <T as pallet::Config>::Currency::unreserve(&proposal.proposer, deposit);

            Proposals::<T>::insert(proposal_id, proposal);

            Ok(())
        }

        /// 执行提案
        #[pallet::call_index(3)]
        #[pallet::weight(T::GovWeightInfo::execute_proposal())]
        pub fn execute_proposal(origin: OriginFor<T>, proposal_id: u32) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            ensure!(!IsPaused::<T>::get(), Error::<T>::SystemPaused);

            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                proposal.status == ProposalStatus::Passed,
                Error::<T>::ProposalNotPassed
            );

            // 检查执行延迟
            let current_block = frame_system::Pallet::<T>::block_number();
            let execution_block = proposal.voting_end.saturating_add(proposal.execution_delay);
            ensure!(
                current_block >= execution_block,
                Error::<T>::ExecutionDelayNotMet
            );

            // 执行提案 (这里简化处理，实际需要根据提案类型执行不同逻辑)
            proposal.status = ProposalStatus::Executed;
            Proposals::<T>::insert(proposal_id, proposal);

            Self::deposit_event(Event::ProposalExecuted { proposal_id });

            Ok(())
        }

        /// 紧急暂停 (需要 sudo 权限)
        #[pallet::call_index(4)]
        #[pallet::weight(T::GovWeightInfo::emergency_pause())]
        pub fn emergency_pause(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // TODO: 添加权限检查 (多签或 sudo)
            
            IsPaused::<T>::put(true);
            Self::deposit_event(Event::EmergencyPaused { by: who });

            Ok(())
        }

        /// 解除暂停
        #[pallet::call_index(5)]
        #[pallet::weight(T::GovWeightInfo::emergency_unpause())]
        pub fn emergency_unpause(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // TODO: 添加权限检查

            IsPaused::<T>::put(false);
            Self::deposit_event(Event::EmergencyUnpaused { by: who });

            Ok(())
        }

        /// 向国库充值
        #[pallet::call_index(6)]
        #[pallet::weight(T::GovWeightInfo::fund_treasury())]
        pub fn fund_treasury(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            <T as pallet::Config>::Currency::reserve(&who, amount)?;
            Treasury::<T>::mutate(|t| {
                t.balance = t.balance.saturating_add(amount);
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 获取提案列表
        pub fn get_active_proposals() -> Vec<u32> {
            let mut active = Vec::new();
            let next_id = NextProposalId::<T>::get();
            
            for id in 0..next_id {
                if let Some(proposal) = Proposals::<T>::get(id) {
                    if proposal.status == ProposalStatus::Active {
                        active.push(id);
                    }
                }
            }
            
            active
        }
    }
}
