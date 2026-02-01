//! # 店铺代币治理模块 (pallet-sharemall-governance)
//!
//! ## 概述
//!
//! 本模块实现店铺代币治理功能：
//! - 提案创建与管理
//! - 代币加权投票
//! - 店主否决权
//! - 提案执行
//!
//! ## 治理模式
//!
//! - **咨询式**: 投票仅作建议，店主有最终决定权
//! - **双轨制**: 代币投票 + 店主否决权
//! - **完全 DAO**: 纯代币投票决定
//!
//! ## 版本历史
//!
//! - v0.1.0 (2026-01-31): 初始版本

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
    use pallet_sharemall_common::ShopProvider;
    use pallet_sharemall_commission::{CommissionProvider, MemberProvider};
    use sp_runtime::traits::{Saturating, Zero};

    // ==================== 类型定义 ====================

    /// 提案 ID 类型
    pub type ProposalId = u64;

    /// 提案状态
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub enum ProposalStatus {
        /// 已创建，等待投票
        Created,
        /// 投票中
        Voting,
        /// 投票通过
        Passed,
        /// 投票未通过
        Failed,
        /// 排队等待执行
        Queued,
        /// 已执行
        Executed,
        /// 已取消
        Cancelled,
        /// 已过期
        Expired,
    }

    impl Default for ProposalStatus {
        fn default() -> Self {
            Self::Created
        }
    }

    /// 投票类型
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
    pub enum VoteType {
        /// 赞成
        #[default]
        Yes,
        /// 反对
        No,
        /// 弃权
        Abstain,
    }


    /// 提案类型（纯代币投票）
    #[derive(Encode, Decode, codec::DecodeWithMemTracking, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub enum ProposalType<Balance> {
        // ==================== 商品管理类 ====================
        /// 商品价格调整
        PriceChange { product_id: u64, new_price: Balance },
        /// 新商品上架
        ProductListing { product_cid: BoundedVec<u8, ConstU32<64>> },
        /// 商品下架
        ProductDelisting { product_id: u64 },
        /// 库存调整
        InventoryAdjustment { product_id: u64, new_inventory: u64 },

        // ==================== 店铺运营类 ====================
        /// 促销活动
        Promotion { discount_rate: u16, duration_blocks: u32 },
        /// 修改店铺名称
        ShopNameChange { new_name: BoundedVec<u8, ConstU32<64>> },
        /// 修改店铺描述
        ShopDescriptionChange { description_cid: BoundedVec<u8, ConstU32<64>> },
        /// 暂停店铺营业
        ShopPause,
        /// 恢复店铺营业
        ShopResume,

        // ==================== 代币经济类 ====================
        /// 代币配置修改
        TokenConfigChange { reward_rate: Option<u16>, exchange_rate: Option<u16> },
        /// 增发代币
        TokenMint { amount: Balance, recipient_cid: BoundedVec<u8, ConstU32<64>> },
        /// 销毁代币（从金库）
        TokenBurn { amount: Balance },
        /// 空投分发
        AirdropDistribution { airdrop_cid: BoundedVec<u8, ConstU32<64>>, total_amount: Balance },
        /// 分红提案
        Dividend { rate: u16 },

        // ==================== 财务管理类 ====================
        /// 店铺金库支出
        TreasurySpend { amount: Balance, recipient_cid: BoundedVec<u8, ConstU32<64>>, reason_cid: BoundedVec<u8, ConstU32<64>> },
        /// 手续费调整
        FeeAdjustment { new_fee_rate: u16 },
        /// 收益分配比例调整
        RevenueShare { owner_share: u16, token_holder_share: u16 },
        /// 退款政策调整
        RefundPolicy { policy_cid: BoundedVec<u8, ConstU32<64>> },

        // ==================== 治理参数类 ====================
        /// 投票期调整
        VotingPeriodChange { new_period_blocks: u32 },
        /// 法定人数调整
        QuorumChange { new_quorum: u8 },
        /// 提案门槛调整
        ProposalThresholdChange { new_threshold: u16 },

        // ==================== 返佣配置类（新增）====================
        /// 启用/禁用返佣模式
        CommissionModesChange { modes: u16 },
        /// 直推奖励配置
        DirectRewardChange { rate: u16 },
        /// 多级分销配置
        MultiLevelChange { 
            /// 各层级配置 (rate, required_directs, required_team_size, required_spent)
            levels_cid: BoundedVec<u8, ConstU32<64>>,
            max_total_rate: u16,
        },
        /// 等级差价配置（全局等级）
        LevelDiffChange {
            normal_rate: u16,
            silver_rate: u16,
            gold_rate: u16,
            platinum_rate: u16,
            diamond_rate: u16,
        },
        /// 自定义等级极差配置
        CustomLevelDiffChange {
            /// 各等级返佣率 CID（JSON 格式）
            rates_cid: BoundedVec<u8, ConstU32<64>>,
            max_depth: u8,
        },
        /// 固定金额配置
        FixedAmountChange { amount: Balance },
        /// 首单奖励配置
        FirstOrderChange { amount: Balance, rate: u16, use_amount: bool },
        /// 复购奖励配置
        RepeatPurchaseChange { rate: u16, min_orders: u32 },
        /// 单线收益配置
        SingleLineChange {
            upline_rate: u16,
            downline_rate: u16,
            base_upline_levels: u8,
            base_downline_levels: u8,
            max_upline_levels: u8,
            max_downline_levels: u8,
        },

        // ==================== 分级提现配置类（新增）====================
        /// 分级提现配置
        WithdrawalConfigChange {
            /// 各等级提现配置 CID（JSON 格式）
            tier_configs_cid: BoundedVec<u8, ConstU32<64>>,
            enabled: bool,
            shopping_balance_generates_commission: bool,
        },

        // ==================== 会员等级体系类（新增）====================
        /// 添加自定义等级
        AddCustomLevel {
            level_id: u8,
            name: BoundedVec<u8, ConstU32<32>>,
            threshold: Balance,
            discount_rate: u16,
            commission_bonus: u16,
        },
        /// 更新自定义等级
        UpdateCustomLevel {
            level_id: u8,
            name: Option<BoundedVec<u8, ConstU32<32>>>,
            threshold: Option<Balance>,
            discount_rate: Option<u16>,
            commission_bonus: Option<u16>,
        },
        /// 删除自定义等级
        RemoveCustomLevel { level_id: u8 },
        /// 设置等级升级模式
        SetUpgradeMode { mode: u8 },  // 0=AutoUpgrade, 1=ManualUpgrade, 2=PeriodReset
        /// 启用/禁用自定义等级
        EnableCustomLevels { enabled: bool },
        /// 添加升级规则
        AddUpgradeRule {
            /// 规则配置 CID（JSON 格式）
            rule_cid: BoundedVec<u8, ConstU32<64>>,
        },
        /// 删除升级规则
        RemoveUpgradeRule { rule_id: u32 },

        // ==================== 社区类 ====================
        /// 社区活动
        CommunityEvent { event_cid: BoundedVec<u8, ConstU32<64>> },
        /// 规则建议
        RuleSuggestion { suggestion_cid: BoundedVec<u8, ConstU32<64>> },
        /// 通用提案（自定义内容）
        General { title_cid: BoundedVec<u8, ConstU32<64>>, content_cid: BoundedVec<u8, ConstU32<64>> },
    }

    /// 提案
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct Proposal<T: Config> {
        /// 提案 ID
        pub id: ProposalId,
        /// 店铺 ID
        pub shop_id: u64,
        /// 提案者
        pub proposer: T::AccountId,
        /// 提案类型
        pub proposal_type: ProposalType<BalanceOf<T>>,
        /// 提案标题
        pub title: BoundedVec<u8, T::MaxTitleLength>,
        /// 提案描述 CID
        pub description_cid: Option<BoundedVec<u8, T::MaxCidLength>>,
        /// 提案状态
        pub status: ProposalStatus,
        /// 创建时间
        pub created_at: BlockNumberFor<T>,
        /// 投票开始时间
        pub voting_start: BlockNumberFor<T>,
        /// 投票结束时间
        pub voting_end: BlockNumberFor<T>,
        /// 执行时间（通过后）
        pub execution_time: Option<BlockNumberFor<T>>,
        /// 赞成票
        pub yes_votes: BalanceOf<T>,
        /// 反对票
        pub no_votes: BalanceOf<T>,
        /// 弃权票
        pub abstain_votes: BalanceOf<T>,
    }

    /// 投票记录
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub struct VoteRecord<AccountId, Balance, BlockNumber> {
        /// 投票者
        pub voter: AccountId,
        /// 投票类型
        pub vote: VoteType,
        /// 投票权重
        pub weight: Balance,
        /// 投票时间
        pub voted_at: BlockNumber,
    }

    /// 余额类型别名
    pub type BalanceOf<T> = <T as Config>::Balance;

    /// 提案类型别名
    pub type ProposalOf<T> = Proposal<T>;

    /// 投票记录类型别名
    pub type VoteRecordOf<T> = VoteRecord<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
    >;

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

        /// 店铺查询接口
        type ShopProvider: ShopProvider<Self::AccountId>;

        /// 代币余额查询接口
        type TokenProvider: ShopTokenProvider<Self::AccountId, Self::Balance>;

        /// 返佣服务接口（治理调用）
        type CommissionProvider: pallet_sharemall_commission::CommissionProvider<Self::AccountId, Self::Balance>;

        /// 会员服务接口（治理调用）
        type MemberProvider: pallet_sharemall_commission::MemberProvider<Self::AccountId>;

        /// 投票期（区块数）
        #[pallet::constant]
        type VotingPeriod: Get<BlockNumberFor<Self>>;

        /// 执行延迟（区块数）
        #[pallet::constant]
        type ExecutionDelay: Get<BlockNumberFor<Self>>;

        /// 通过阈值（百分比，如 50 = 50%）
        #[pallet::constant]
        type PassThreshold: Get<u8>;

        /// 法定人数阈值（百分比）
        #[pallet::constant]
        type QuorumThreshold: Get<u8>;

        /// 创建提案所需最低代币持有比例（基点，如 100 = 1%）
        #[pallet::constant]
        type MinProposalThreshold: Get<u16>;

        /// 提案标题最大长度
        #[pallet::constant]
        type MaxTitleLength: Get<u32>;

        /// CID 最大长度
        #[pallet::constant]
        type MaxCidLength: Get<u32>;

        /// 每个店铺最大活跃提案数
        #[pallet::constant]
        type MaxActiveProposals: Get<u32>;
    }

    /// 代币余额查询 trait
    pub trait ShopTokenProvider<AccountId, Balance> {
        /// 获取用户在店铺的代币余额
        fn token_balance(shop_id: u64, holder: &AccountId) -> Balance;
        /// 获取店铺代币总供应量
        fn total_supply(shop_id: u64) -> Balance;
        /// 检查店铺代币是否启用
        fn is_enabled(shop_id: u64) -> bool;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ==================== 存储项 ====================

    /// 下一个提案 ID
    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T: Config> = StorageValue<_, ProposalId, ValueQuery>;

    /// 提案存储
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ProposalId,
        ProposalOf<T>,
    >;

    /// 店铺活跃提案列表
    #[pallet::storage]
    #[pallet::getter(fn shop_proposals)]
    pub type ShopProposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // shop_id
        BoundedVec<ProposalId, T::MaxActiveProposals>,
        ValueQuery,
    >;

    /// 投票记录
    #[pallet::storage]
    #[pallet::getter(fn vote_records)]
    pub type VoteRecords<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ProposalId,
        Blake2_128Concat,
        T::AccountId,
        VoteRecordOf<T>,
    >;

    /// 用户首次持有代币时间（用于时间加权）
    #[pallet::storage]
    #[pallet::getter(fn first_hold_time)]
    pub type FirstHoldTime<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,  // shop_id
        Blake2_128Concat,
        T::AccountId,
        BlockNumberFor<T>,
    >;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 提案已创建
        ProposalCreated {
            proposal_id: ProposalId,
            shop_id: u64,
            proposer: T::AccountId,
            title: Vec<u8>,
        },
        /// 已投票
        Voted {
            proposal_id: ProposalId,
            voter: T::AccountId,
            vote: VoteType,
            weight: BalanceOf<T>,
        },
        /// 提案已通过
        ProposalPassed {
            proposal_id: ProposalId,
        },
        /// 提案未通过
        ProposalFailed {
            proposal_id: ProposalId,
        },
        /// 提案已执行
        ProposalExecuted {
            proposal_id: ProposalId,
        },
        /// 提案已取消
        ProposalCancelled {
            proposal_id: ProposalId,
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
        /// 提案不存在
        ProposalNotFound,
        /// 代币余额不足以创建提案
        InsufficientTokensForProposal,
        /// 已达到最大活跃提案数
        TooManyActiveProposals,
        /// 提案状态不允许此操作
        InvalidProposalStatus,
        /// 已经投过票
        AlreadyVoted,
        /// 没有投票权
        NoVotingPower,
        /// 投票期已结束
        VotingEnded,
        /// 投票期未结束
        VotingNotEnded,
        /// 执行时间未到
        ExecutionTimeNotReached,
        /// 标题过长
        TitleTooLong,
        /// CID 过长
        CidTooLong,
        /// 无权取消
        CannotCancel,
    }

    // ==================== Extrinsics ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建提案
        ///
        /// # 参数
        /// - `shop_id`: 店铺 ID
        /// - `proposal_type`: 提案类型
        /// - `title`: 提案标题
        /// - `description_cid`: 提案描述 CID（可选）
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(50_000, 0))]
        pub fn create_proposal(
            origin: OriginFor<T>,
            shop_id: u64,
            proposal_type: ProposalType<BalanceOf<T>>,
            title: Vec<u8>,
            description_cid: Option<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证店铺存在
            ensure!(T::ShopProvider::shop_exists(shop_id), Error::<T>::ShopNotFound);

            // 验证代币已启用
            ensure!(T::TokenProvider::is_enabled(shop_id), Error::<T>::TokenNotEnabled);

            // 验证持有足够代币
            let balance = T::TokenProvider::token_balance(shop_id, &who);
            let total_supply = T::TokenProvider::total_supply(shop_id);
            let min_threshold = total_supply
                .saturating_mul(T::MinProposalThreshold::get().into())
                / 10000u128.into();
            ensure!(balance >= min_threshold, Error::<T>::InsufficientTokensForProposal);

            // 检查活跃提案数量
            let mut shop_proposals = ShopProposals::<T>::get(shop_id);
            ensure!(
                shop_proposals.len() < T::MaxActiveProposals::get() as usize,
                Error::<T>::TooManyActiveProposals
            );

            // 转换标题和描述
            let title_bounded: BoundedVec<u8, T::MaxTitleLength> =
                title.clone().try_into().map_err(|_| Error::<T>::TitleTooLong)?;
            let description_bounded = description_cid
                .map(|cid| cid.try_into().map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            // 创建提案
            let proposal_id = NextProposalId::<T>::get();
            let now = <frame_system::Pallet<T>>::block_number();
            let voting_end = now.saturating_add(T::VotingPeriod::get());

            let proposal = Proposal {
                id: proposal_id,
                shop_id,
                proposer: who.clone(),
                proposal_type,
                title: title_bounded,
                description_cid: description_bounded,
                status: ProposalStatus::Voting,
                created_at: now,
                voting_start: now,
                voting_end,
                execution_time: None,
                yes_votes: Zero::zero(),
                no_votes: Zero::zero(),
                abstain_votes: Zero::zero(),
            };

            // 保存
            Proposals::<T>::insert(proposal_id, proposal);
            shop_proposals.try_push(proposal_id).map_err(|_| Error::<T>::TooManyActiveProposals)?;
            ShopProposals::<T>::insert(shop_id, shop_proposals);
            NextProposalId::<T>::put(proposal_id.saturating_add(1));

            Self::deposit_event(Event::ProposalCreated {
                proposal_id,
                shop_id,
                proposer: who,
                title,
            });

            Ok(())
        }

        /// 投票
        ///
        /// # 参数
        /// - `proposal_id`: 提案 ID
        /// - `vote`: 投票类型
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn vote(
            origin: OriginFor<T>,
            proposal_id: ProposalId,
            vote: VoteType,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取提案
            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // 验证状态
            ensure!(proposal.status == ProposalStatus::Voting, Error::<T>::InvalidProposalStatus);

            // 验证投票期
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(now <= proposal.voting_end, Error::<T>::VotingEnded);

            // 验证未投过票
            ensure!(
                !VoteRecords::<T>::contains_key(proposal_id, &who),
                Error::<T>::AlreadyVoted
            );

            // 获取投票权重
            let weight = Self::calculate_voting_power(proposal.shop_id, &who);
            ensure!(!weight.is_zero(), Error::<T>::NoVotingPower);

            // 记录投票
            match vote {
                VoteType::Yes => proposal.yes_votes = proposal.yes_votes.saturating_add(weight),
                VoteType::No => proposal.no_votes = proposal.no_votes.saturating_add(weight),
                VoteType::Abstain => proposal.abstain_votes = proposal.abstain_votes.saturating_add(weight),
            }

            let record = VoteRecord {
                voter: who.clone(),
                vote: vote.clone(),
                weight,
                voted_at: now,
            };

            Proposals::<T>::insert(proposal_id, proposal);
            VoteRecords::<T>::insert(proposal_id, &who, record);

            Self::deposit_event(Event::Voted {
                proposal_id,
                voter: who,
                vote,
                weight,
            });

            Ok(())
        }

        /// 结束投票并计算结果
        ///
        /// 任何人都可以调用（投票期结束后）
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(40_000, 0))]
        pub fn finalize_voting(
            origin: OriginFor<T>,
            proposal_id: ProposalId,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // 验证状态
            ensure!(proposal.status == ProposalStatus::Voting, Error::<T>::InvalidProposalStatus);

            // 验证投票期已结束
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(now > proposal.voting_end, Error::<T>::VotingNotEnded);

            // 计算结果
            let total_votes = proposal.yes_votes
                .saturating_add(proposal.no_votes)
                .saturating_add(proposal.abstain_votes);
            let total_supply = T::TokenProvider::total_supply(proposal.shop_id);

            // 检查法定人数
            let quorum_threshold: BalanceOf<T> = total_supply
                .saturating_mul(T::QuorumThreshold::get().into())
                / 100u128.into();
            
            if total_votes < quorum_threshold {
                proposal.status = ProposalStatus::Failed;
                Self::remove_from_active(proposal_id, proposal.shop_id);
                Proposals::<T>::insert(proposal_id, proposal);
                Self::deposit_event(Event::ProposalFailed { proposal_id });
                return Ok(());
            }

            // 检查通过阈值
            let pass_threshold: BalanceOf<T> = total_votes
                .saturating_mul(T::PassThreshold::get().into())
                / 100u128.into();

            if proposal.yes_votes > pass_threshold {
                proposal.status = ProposalStatus::Passed;
                proposal.execution_time = Some(now.saturating_add(T::ExecutionDelay::get()));
                Proposals::<T>::insert(proposal_id, proposal);
                Self::deposit_event(Event::ProposalPassed { proposal_id });
            } else {
                proposal.status = ProposalStatus::Failed;
                Self::remove_from_active(proposal_id, proposal.shop_id);
                Proposals::<T>::insert(proposal_id, proposal);
                Self::deposit_event(Event::ProposalFailed { proposal_id });
            }

            Ok(())
        }

        /// 执行提案
        ///
        /// 任何人都可以调用（执行时间到达后）
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(80_000, 0))]
        pub fn execute_proposal(
            origin: OriginFor<T>,
            proposal_id: ProposalId,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // 验证状态
            ensure!(proposal.status == ProposalStatus::Passed, Error::<T>::InvalidProposalStatus);

            // 验证执行时间
            let now = <frame_system::Pallet<T>>::block_number();
            let exec_time = proposal.execution_time.ok_or(Error::<T>::ExecutionTimeNotReached)?;
            ensure!(now >= exec_time, Error::<T>::ExecutionTimeNotReached);

            // 执行提案（根据类型）
            Self::do_execute_proposal(&proposal)?;

            // 更新状态
            proposal.status = ProposalStatus::Executed;
            let shop_id = proposal.shop_id;
            Proposals::<T>::insert(proposal_id, proposal);
            Self::remove_from_active(proposal_id, shop_id);

            Self::deposit_event(Event::ProposalExecuted { proposal_id });

            Ok(())
        }

        /// 取消提案
        ///
        /// 提案者或店主可以取消
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(30_000, 0))]
        pub fn cancel_proposal(
            origin: OriginFor<T>,
            proposal_id: ProposalId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // 验证权限（提案者或店主）
            let owner = T::ShopProvider::shop_owner(proposal.shop_id);
            ensure!(
                proposal.proposer == who || owner == Some(who.clone()),
                Error::<T>::CannotCancel
            );

            // 验证状态（只能取消 Created 或 Voting 状态的提案）
            ensure!(
                proposal.status == ProposalStatus::Created || proposal.status == ProposalStatus::Voting,
                Error::<T>::InvalidProposalStatus
            );

            // 取消
            proposal.status = ProposalStatus::Cancelled;
            let shop_id = proposal.shop_id;
            Proposals::<T>::insert(proposal_id, proposal);
            Self::remove_from_active(proposal_id, shop_id);

            Self::deposit_event(Event::ProposalCancelled { proposal_id });

            Ok(())
        }
    }

    // ==================== 内部函数 ====================

    impl<T: Config> Pallet<T> {
        /// 计算投票权重（时间加权）
        pub fn calculate_voting_power(shop_id: u64, holder: &T::AccountId) -> BalanceOf<T> {
            let balance = T::TokenProvider::token_balance(shop_id, holder);
            
            if balance.is_zero() {
                return Zero::zero();
            }

            // 简单实现：直接返回余额
            // TODO: 实现时间加权
            balance
        }

        /// 从活跃提案列表移除
        fn remove_from_active(proposal_id: ProposalId, shop_id: u64) {
            ShopProposals::<T>::mutate(shop_id, |proposals| {
                proposals.retain(|&id| id != proposal_id);
            });
        }

        /// 执行提案
        fn do_execute_proposal(proposal: &ProposalOf<T>) -> DispatchResult {
            let _shop_id = proposal.shop_id;
            
            match &proposal.proposal_type {
                // ==================== 商品管理类 ====================
                ProposalType::PriceChange { product_id: _, new_price: _ } => {
                    // TODO: 调用 product 模块更新价格
                    // T::ProductProvider::update_price(shop_id, *product_id, *new_price)?;
                    Ok(())
                },
                ProposalType::ProductListing { product_cid: _ } => {
                    // TODO: 调用 product 模块上架商品
                    // 需要解析 CID 获取商品信息
                    Ok(())
                },
                ProposalType::ProductDelisting { product_id: _ } => {
                    // TODO: 调用 product 模块下架商品
                    // T::ProductProvider::delist(shop_id, *product_id)?;
                    Ok(())
                },
                ProposalType::InventoryAdjustment { product_id: _, new_inventory: _ } => {
                    // TODO: 调用 product 模块调整库存
                    // T::ProductProvider::set_inventory(shop_id, *product_id, *new_inventory)?;
                    Ok(())
                },

                // ==================== 店铺运营类 ====================
                ProposalType::Promotion { discount_rate: _, duration_blocks: _ } => {
                    // TODO: 创建促销活动
                    // T::ShopProvider::create_promotion(shop_id, *discount_rate, *duration_blocks)?;
                    Ok(())
                },
                ProposalType::ShopNameChange { new_name: _ } => {
                    // TODO: 调用 shop 模块更新名称
                    // T::ShopProvider::update_name(shop_id, new_name.clone())?;
                    Ok(())
                },
                ProposalType::ShopDescriptionChange { description_cid: _ } => {
                    // TODO: 调用 shop 模块更新描述
                    // T::ShopProvider::update_description(shop_id, description_cid.clone())?;
                    Ok(())
                },
                ProposalType::ShopPause => {
                    // TODO: 暂停店铺
                    // T::ShopProvider::pause(shop_id)?;
                    Ok(())
                },
                ProposalType::ShopResume => {
                    // TODO: 恢复店铺
                    // T::ShopProvider::resume(shop_id)?;
                    Ok(())
                },

                // ==================== 代币经济类 ====================
                ProposalType::TokenConfigChange { reward_rate: _, exchange_rate: _ } => {
                    // TODO: 调用 token 模块更新配置
                    // T::TokenProvider::update_config(shop_id, *reward_rate, *exchange_rate)?;
                    Ok(())
                },
                ProposalType::TokenMint { amount: _, recipient_cid: _ } => {
                    // TODO: 增发代币
                    // 需要解析 recipient_cid 获取接收者列表
                    Ok(())
                },
                ProposalType::TokenBurn { amount: _ } => {
                    // TODO: 从金库销毁代币
                    // T::TokenProvider::burn_from_treasury(shop_id, *amount)?;
                    Ok(())
                },
                ProposalType::AirdropDistribution { airdrop_cid: _, total_amount: _ } => {
                    // TODO: 执行空投
                    // 需要解析 airdrop_cid 获取空投列表
                    Ok(())
                },
                ProposalType::Dividend { rate: _ } => {
                    // TODO: 执行分红
                    // T::TokenProvider::distribute_dividend(shop_id, *rate)?;
                    Ok(())
                },

                // ==================== 财务管理类 ====================
                ProposalType::TreasurySpend { amount: _, recipient_cid: _, reason_cid: _ } => {
                    // TODO: 从金库支出
                    // 需要解析 recipient_cid 获取接收者
                    Ok(())
                },
                ProposalType::FeeAdjustment { new_fee_rate: _ } => {
                    // TODO: 调整手续费
                    // T::ShopProvider::set_fee_rate(shop_id, *new_fee_rate)?;
                    Ok(())
                },
                ProposalType::RevenueShare { owner_share: _, token_holder_share: _ } => {
                    // TODO: 调整收益分配比例
                    // T::ShopProvider::set_revenue_share(shop_id, *owner_share, *token_holder_share)?;
                    Ok(())
                },
                ProposalType::RefundPolicy { policy_cid: _ } => {
                    // TODO: 更新退款政策
                    // T::ShopProvider::set_refund_policy(shop_id, policy_cid.clone())?;
                    Ok(())
                },

                // ==================== 治理参数类 ====================
                ProposalType::VotingPeriodChange { new_period_blocks: _ } => {
                    // 注意：治理参数修改需要特殊处理
                    // 可能需要存储在店铺级别的治理配置中
                    Ok(())
                },
                ProposalType::QuorumChange { new_quorum: _ } => {
                    // 注意：治理参数修改需要特殊处理
                    Ok(())
                },
                ProposalType::ProposalThresholdChange { new_threshold: _ } => {
                    // 注意：治理参数修改需要特殊处理
                    Ok(())
                },

                // ==================== 返佣配置类 ====================
                ProposalType::CommissionModesChange { modes } => {
                    T::CommissionProvider::set_commission_modes(_shop_id, *modes)
                },
                ProposalType::DirectRewardChange { rate } => {
                    T::CommissionProvider::set_direct_reward_rate(_shop_id, *rate)
                },
                ProposalType::MultiLevelChange { levels_cid: _, max_total_rate: _ } => {
                    // 多级分销配置需要解析 CID，暂不支持链上直接执行
                    // 可通过链下工作者或预言机解析后调用
                    Ok(())
                },
                ProposalType::LevelDiffChange { normal_rate, silver_rate, gold_rate, platinum_rate, diamond_rate } => {
                    T::CommissionProvider::set_level_diff_config(
                        _shop_id,
                        *normal_rate,
                        *silver_rate,
                        *gold_rate,
                        *platinum_rate,
                        *diamond_rate,
                    )
                },
                ProposalType::CustomLevelDiffChange { rates_cid: _, max_depth: _ } => {
                    // 自定义等级极差配置需要解析 CID，暂不支持链上直接执行
                    Ok(())
                },
                ProposalType::FixedAmountChange { amount } => {
                    T::CommissionProvider::set_fixed_amount(_shop_id, *amount)
                },
                ProposalType::FirstOrderChange { amount, rate, use_amount } => {
                    T::CommissionProvider::set_first_order_config(_shop_id, *amount, *rate, *use_amount)
                },
                ProposalType::RepeatPurchaseChange { rate, min_orders } => {
                    T::CommissionProvider::set_repeat_purchase_config(_shop_id, *rate, *min_orders)
                },
                ProposalType::SingleLineChange { upline_rate: _, downline_rate: _, base_upline_levels: _, base_downline_levels: _, max_upline_levels: _, max_downline_levels: _ } => {
                    // 单线收益配置较复杂，需要扩展 CommissionProvider trait
                    // 暂时只记录提案，后续实现
                    Ok(())
                },

                // ==================== 分级提现配置类 ====================
                ProposalType::WithdrawalConfigChange { tier_configs_cid: _, enabled, shopping_balance_generates_commission } => {
                    // tier_configs_cid 需要链下解析，这里只设置基本配置
                    T::CommissionProvider::set_withdrawal_config_by_governance(
                        _shop_id,
                        *enabled,
                        *shopping_balance_generates_commission,
                    )
                },

                // ==================== 会员等级体系类 ====================
                ProposalType::AddCustomLevel { level_id, name, threshold, discount_rate, commission_bonus } => {
                    T::MemberProvider::add_custom_level(
                        _shop_id,
                        *level_id,
                        name.as_slice(),
                        (*threshold).into(),
                        *discount_rate,
                        *commission_bonus,
                    )
                },
                ProposalType::UpdateCustomLevel { level_id, name, threshold, discount_rate, commission_bonus } => {
                    T::MemberProvider::update_custom_level(
                        _shop_id,
                        *level_id,
                        name.as_ref().map(|n| n.as_slice()),
                        threshold.map(|t| t.into()),
                        *discount_rate,
                        *commission_bonus,
                    )
                },
                ProposalType::RemoveCustomLevel { level_id } => {
                    T::MemberProvider::remove_custom_level(_shop_id, *level_id)
                },
                ProposalType::SetUpgradeMode { mode } => {
                    T::MemberProvider::set_upgrade_mode(_shop_id, *mode)
                },
                ProposalType::EnableCustomLevels { enabled } => {
                    T::MemberProvider::set_custom_levels_enabled(_shop_id, *enabled)
                },
                ProposalType::AddUpgradeRule { rule_cid: _ } => {
                    // 升级规则配置需要解析 CID，暂不支持链上直接执行
                    Ok(())
                },
                ProposalType::RemoveUpgradeRule { rule_id: _ } => {
                    // 需要扩展 MemberProvider trait 添加删除规则方法
                    Ok(())
                },

                // ==================== 社区类 ====================
                ProposalType::CommunityEvent { event_cid: _ } => {
                    // 社区活动只是记录，无需执行
                    Ok(())
                },
                ProposalType::RuleSuggestion { suggestion_cid: _ } => {
                    // 规则建议只是记录，无需执行
                    Ok(())
                },
                ProposalType::General { title_cid: _, content_cid: _ } => {
                    // 通用提案只是记录，无需执行
                    Ok(())
                },
            }
        }
    }
}
