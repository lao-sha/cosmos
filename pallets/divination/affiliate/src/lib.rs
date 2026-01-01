#![cfg_attr(not(feature = "std"), no_std)]
#![allow(deprecated)]

//! # 统一联盟计酬系统 (pallet-affiliate)
//!
//! ## 功能概述
//!
//! 本模块整合了原有的5个联盟计酬相关pallet，提供统一的联盟计酬解决方案：
//! - **推荐关系管理**：推荐人绑定、推荐码管理、推荐链查询
//! - **资金托管**：独立托管账户、资金存取
//! - **即时分成**：实时转账、立即到账
//! - **周结算**：记账分配、周期结算
//! - **配置管理**：模式切换、分成比例配置
//!
//! ## 架构设计
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │                   pallet-affiliate                       │
//! │                  （统一联盟计酬系统）                      │
//! ├──────────────────────────────────────────────────────────┤
//! │  📦 推荐关系管理  →  referral.rs                          │
//! │  ⚙️ 配置管理      →  types.rs (SettlementMode等)         │
//! │  💰 资金托管      →  escrow.rs                            │
//! │  ⚡ 即时分成      →  instant.rs                           │
//! │  📅 周结算        →  weekly.rs                            │
//! │  📊 统一分配入口  →  distribute.rs                        │
//! └──────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 整合自
//!
//! - `pallet-affiliate`: 资金托管
//! - `pallet-affiliate-config`: 配置管理
//! - `pallet-affiliate-instant`: 即时分成
//! - `pallet-affiliate-weekly`: 周结算
//! - `pallet-memo-referrals`: 推荐关系
//!
//! **版本**: 1.0.0  
//! **整合日期**: 2025-10-28

pub use pallet::*;

pub mod types;
// 🆕 2025-12-30：推荐关系抽离为独立 pallet
// mod referral;  // 已移动到 pallet-affiliate-referral
// 通过 Config: pallet_affiliate_referral::Config 继承推荐关系功能
pub use pallet_affiliate_referral::{MembershipProvider, ReferralProvider};
mod escrow;
mod instant;
mod weekly;
mod distribute;
pub mod governance;  // 新增：治理模块，使用 pub mod 避免重复导出

// 导出特定的治理类型，避免冲突
pub use governance::{
    Vote, Conviction, ProposalStatus,
    PercentageAdjustmentProposal, VoteRecord, PercentageChangeRecord
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{pallet_prelude::*, PalletId, BoundedVec};
use frame_system::pallet_prelude::*;
use sp_runtime::{traits::{Zero, Saturating}, SaturatedConversion};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::{Currency, Get, ReservableCurrency};

    /// 余额类型
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 每个区块结束时执行，检查并执行已通过的提案
        fn on_finalize(block_number: BlockNumberFor<T>) {
            // 检查是否有需要执行的提案
            for (proposal_id, proposal) in ReadyForExecution::<T>::iter() {
                // 比较区块高度
                let effective_block: BlockNumberFor<T> = proposal.effective_block;
                if effective_block <= block_number {
                    // 执行提案
                    match Self::execute_percentage_change(&proposal_id, &proposal) {
                        Ok(_) => {
                            // 执行成功，清理状态
                            ReadyForExecution::<T>::remove(&proposal_id);
                            ActiveProposals::<T>::remove(&proposal_id);
                            Self::return_proposal_deposit(&proposal_id);

                            // 发射事件
                            Self::deposit_event(Event::PercentageAdjustmentExecuted {
                                proposal_id,
                                new_percentages: proposal.new_percentages,
                                effective_block: block_number,
                            });
                        },
                        Err(_) => {
                            // 执行失败，跳过此提案
                            // 注意：生产环境应记录错误日志
                        }
                    }
                }
            }
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_affiliate_referral::Config {
        /// 事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 货币系统（支持锁定和保留）
        type Currency: Currency<Self::AccountId> + frame_support::traits::ReservableCurrency<Self::AccountId>;

        /// 托管 PalletId（派生独立的托管账户）
        #[pallet::constant]
        type EscrowPalletId: Get<PalletId>;

        /// 提款权限控制（可选）
        type WithdrawOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 管理员权限（配置管理）
        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 销毁账户
        type BurnAccount: Get<Self::AccountId>;

        /// 国库账户
        type TreasuryAccount: Get<Self::AccountId>;

        /// 存储费用账户
        type StorageAccount: Get<Self::AccountId>;
    }

    // ========================================
    // 存储项
    // ========================================

    // === 推荐关系存储（已移动到 pallet-affiliate-referral）===
    // 🆕 2025-12-30：Sponsors、AccountByCode、CodeByAccount 已迁移到独立 pallet
    // 通过 T::ReferralProvider trait 访问推荐关系数据

    // === 配置存储（4个）===

    /// 结算模式：Weekly / Instant / Hybrid
    #[pallet::storage]
    #[pallet::getter(fn settlement_mode)]
    pub type SettlementMode<T: Config> = 
        StorageValue<_, types::SettlementMode, ValueQuery>;

    /// 即时分成比例（15层）
    #[pallet::storage]
    #[pallet::getter(fn instant_percents)]
    pub type InstantLevelPercents<T: Config> = 
        StorageValue<_, types::LevelPercents, ValueQuery, DefaultInstantPercents>;

    /// 周结算分成比例（15层）
    #[pallet::storage]
    #[pallet::getter(fn weekly_percents)]
    pub type WeeklyLevelPercents<T: Config> = 
        StorageValue<_, types::LevelPercents, ValueQuery, DefaultWeeklyPercents>;

    /// 每周区块数
    #[pallet::storage]
    #[pallet::getter(fn blocks_per_week)]
    pub type BlocksPerWeek<T: Config> = 
        StorageValue<_, BlockNumberFor<T>, ValueQuery, DefaultBlocksPerWeek<T>>;

    // === 托管存储（2个）===

    /// 累计存入金额
    #[pallet::storage]
    pub type TotalDeposited<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// 累计提取金额
    #[pallet::storage]
    pub type TotalWithdrawn<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    // === 即时分成存储（1个）===

    /// 累计即时分配金额
    #[pallet::storage]
    pub type TotalInstantDistributed<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    // === 周结算存储（6个）===

    /// 应得金额：(周编号, 账户) → 金额
    #[pallet::storage]
    pub type Entitlement<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        u32,  // cycle
        Blake2_128Concat,
        T::AccountId,
        BalanceOf<T>,
        ValueQuery,
    >;

    /// 活跃期：账户 → 活跃截止周
    #[pallet::storage]
    pub type ActiveUntilWeek<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,  // week_number
        ValueQuery,
    >;

    /// 直推活跃数：账户 → 活跃直推数量
    #[pallet::storage]
    pub type DirectActiveCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    /// 结算游标：周编号 → 当前结算账户索引
    #[pallet::storage]
    pub type SettleCursor<T: Config> = StorageMap<
        _,
        Twox64Concat,
        u32,  // cycle
        u32,  // account_index
        ValueQuery,
    >;

    /// 当前结算周期
    #[pallet::storage]
    pub type CurrentSettlingCycle<T: Config> = StorageValue<_, Option<u32>>;

    /// 累计周结算分配金额
    #[pallet::storage]
    pub type TotalWeeklyDistributed<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    // === 治理存储（12个）===

    /// 下一个提案ID
    #[pallet::storage]
    pub type NextProposalId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 活跃提案
    #[pallet::storage]
    #[pallet::unbounded]
    pub type ActiveProposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        governance::PercentageAdjustmentProposal<T>,
    >;

    /// 提案押金
    #[pallet::storage]
    pub type ProposalDeposits<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        (T::AccountId, BalanceOf<T>),
    >;

    /// 投票记录
    #[pallet::storage]
    #[pallet::unbounded]
    pub type ProposalVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        governance::VoteRecord<T>,
    >;

    /// 投票统计
    #[pallet::storage]
    pub type VoteTally<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        governance::VoteTally,
        ValueQuery,
    >;

    /// 投票历史（用于参与权重计算）
    #[pallet::storage]
    pub type VoteHistory<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<100>>,
        ValueQuery,
    >;

    /// 比例变更历史
    #[pallet::storage]
    #[pallet::unbounded]
    pub type PercentageHistory<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        governance::PercentageChangeRecord<T>,
    >;

    /// 治理暂停标记
    #[pallet::storage]
    pub type GovernancePaused<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// 暂停原因
    #[pallet::storage]
    pub type PauseReason<T: Config> = StorageValue<_, BoundedVec<u8, ConstU32<64>>>;

    /// 账户冷却期（提案失败后）
    #[pallet::storage]
    pub type ProposalCooldown<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BlockNumberFor<T>,
    >;

    /// 账户活跃提案数
    #[pallet::storage]
    pub type ActiveProposalsByAccount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<3>>,
        ValueQuery,
    >;

    /// 账户最后提案区块
    #[pallet::storage]
    pub type LastProposalBlock<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BlockNumberFor<T>,
    >;

    /// 待执行提案
    #[pallet::storage]
    #[pallet::unbounded]
    pub type ReadyForExecution<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        governance::PercentageAdjustmentProposal<T>,
    >;

    // ========================================
    // 年费价格治理存储项（🆕）
    // ========================================

    /// 活跃年费价格提案
    #[pallet::storage]
    #[pallet::unbounded]
    pub type ActiveMembershipPriceProposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        governance::MembershipPriceProposal<T>,
    >;

    /// 年费价格提案押金
    #[pallet::storage]
    pub type MembershipPriceProposalDeposits<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        (T::AccountId, BalanceOf<T>),
    >;

    /// 年费价格提案投票统计
    #[pallet::storage]
    #[pallet::getter(fn membership_price_vote_tally)]
    pub type MembershipPriceVoteTally<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        governance::VoteTally,
        ValueQuery,
    >;

    /// 年费价格提案投票记录
    #[pallet::storage]
    pub type MembershipPriceProposalVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        governance::VoteRecord<T>,
    >;

    /// 待执行年费价格提案
    #[pallet::storage]
    #[pallet::unbounded]
    pub type ReadyForMembershipPriceExecution<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        governance::MembershipPriceProposal<T>,
    >;

    /// 年费价格变更历史记录
    #[pallet::storage]
    #[pallet::unbounded]
    pub type MembershipPriceHistory<T: Config> = StorageValue<
        _,
        BoundedVec<governance::MembershipPriceChangeRecord<T>, ConstU32<100>>,
        ValueQuery,
    >;

    // ========================================
    // 默认值
    // ========================================

    /// 默认每周区块数（假设6秒出块，1周≈100800块）
    #[pallet::type_value]
    pub fn DefaultBlocksPerWeek<T: Config>() -> BlockNumberFor<T> {
        100800u32.into()
    }

    /// 默认即时分成比例
    #[pallet::type_value]
    pub fn DefaultInstantPercents() -> types::LevelPercents {
        types::default_instant_percents()
    }

    /// 默认周结算分成比例
    #[pallet::type_value]
    pub fn DefaultWeeklyPercents() -> types::LevelPercents {
        types::default_weekly_percents()
    }

    // ========================================
    // 事件
    // ========================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // === 推荐关系事件 ===
        /// 推荐人已绑定
        SponsorBound {
            who: T::AccountId,
            sponsor: T::AccountId,
        },
        /// 推荐码已认领
        CodeClaimed {
            who: T::AccountId,
            code: BoundedVec<u8, T::MaxCodeLen>,
        },

        // === 配置管理事件 ===
        /// 结算模式已更新
        SettlementModeSet,
        /// 即时分成比例已更新
        InstantPercentsSet,
        /// 周结算分成比例已更新
        WeeklyPercentsSet,
        /// 每周区块数已更新
        BlocksPerWeekSet {
            blocks: BlockNumberFor<T>,
        },

        // === 托管事件 ===
        /// 资金已存入托管
        Deposited {
            from: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 资金已从托管提取
        Withdrawn {
            to: T::AccountId,
            amount: BalanceOf<T>,
        },

        // === 即时分成事件 ===
        /// 即时奖励已分配
        InstantRewardDistributed {
            referrer: T::AccountId,
            buyer: T::AccountId,
            level: u8,
            amount: BalanceOf<T>,
        },

        // === 周结算事件 ===
        /// 周期已结算
        CycleSettled {
            cycle: u32,
            settled_count: u32,
            total_amount: BalanceOf<T>,
        },

        // === 治理事件 ===
        /// 提案已创建
        PercentageAdjustmentProposed {
            proposal_id: u64,
            proposer: T::AccountId,
            change_magnitude: u32,
            is_major: bool,
        },
        /// 投票已提交
        /// vote_type: 0=Aye, 1=Nay, 2=Abstain
        VoteCast {
            proposal_id: u64,
            voter: T::AccountId,
            vote_type: u8,
            weight: u128,
        },
        /// 提案已通过
        ProposalPassed {
            proposal_id: u64,
            approval_rate: sp_runtime::Perbill,
            participation_rate: sp_runtime::Perbill,
            effective_block: BlockNumberFor<T>,
        },
        /// 提案已拒绝
        ProposalRejected {
            proposal_id: u64,
            approval_rate: sp_runtime::Perbill,
            participation_rate: sp_runtime::Perbill,
        },
        /// 提案已取消
        ProposalCancelled {
            proposal_id: u64,
            proposer: T::AccountId,
        },
        /// 比例调整已执行
        PercentageAdjustmentExecuted {
            proposal_id: u64,
            new_percentages: types::LevelPercents,
            effective_block: BlockNumberFor<T>,
        },
        /// 治理紧急暂停
        GovernanceEmergencyPaused {
            reason_cid: BoundedVec<u8, ConstU32<64>>,
        },
        /// 治理已恢复
        GovernanceResumed {
            by: BoundedVec<u8, ConstU32<32>>,
        },

        // ========================================
        // 年费价格治理事件（🆕）
        // ========================================

        /// 年费价格调整提案已创建
        MembershipPriceProposed {
            proposal_id: u64,
            proposer: T::AccountId,
            new_prices_usdt: [u64; 4],
            is_major: bool,
            deposit: BalanceOf<T>,
        },

        /// 年费价格提案投票已提交
        MembershipPriceVoteCast {
            proposal_id: u64,
            voter: T::AccountId,
            vote: u8,
            conviction: u8,
            voting_power: u64,
        },

        /// 年费价格提案已通过
        MembershipPriceProposalPassed {
            proposal_id: u64,
            approval_rate: sp_runtime::Perbill,
            participation_rate: sp_runtime::Perbill,
            effective_block: BlockNumberFor<T>,
        },

        /// 年费价格提案已拒绝
        MembershipPriceProposalRejected {
            proposal_id: u64,
            approval_rate: sp_runtime::Perbill,
            participation_rate: sp_runtime::Perbill,
        },

        /// 年费价格提案已取消
        MembershipPriceProposalCancelled {
            proposal_id: u64,
            proposer: T::AccountId,
        },

        /// 年费价格调整已执行
        MembershipPriceAdjustmentExecuted {
            proposal_id: u64,
            new_prices_usdt: [u64; 4],
            effective_block: BlockNumberFor<T>,
        },
    }

    // ========================================
    // 错误
    // ========================================

    #[pallet::error]
    pub enum Error<T> {
        // === 推荐关系错误 ===
        /// 已绑定推荐人
        AlreadyBound,
        /// 推荐码不存在
        CodeNotFound,
        /// 不能绑定自己
        CannotBindSelf,
        /// 会形成循环
        WouldCreateCycle,
        /// 不是有效会员
        NotMember,
        /// 推荐码过长
        CodeTooLong,
        /// 推荐码过短
        CodeTooShort,
        /// 推荐码已被占用
        CodeAlreadyTaken,
        /// 已拥有推荐码
        AlreadyHasCode,

        // === 配置管理错误 ===
        /// 无效的分成比例
        InvalidPercents,
        /// 混合模式层数超限
        HybridLevelsTooMany,

        // === 托管错误 ===
        /// 提款失败
        WithdrawFailed,

        // === 配置错误 ===
        /// 无效的模式ID
        InvalidMode,

        // === 治理错误 ===
        /// 比例数组长度必须为15
        InvalidPercentageLength,
        /// 单层比例超过100%
        PercentageTooHigh,
        /// 🔥 2025-11-13 更新：前2层比例不能为0（第3层可以为0）
        CriticalLayerZero,
        /// 总比例低于50%
        TotalPercentageTooLow,
        /// 总比例超过99%
        TotalPercentageTooHigh,
        /// 前5层比例应递减
        NonDecreasingPercentage,
        /// L1比例超过50%
        FirstLayerTooHigh,
        /// 提案押金不足
        InsufficientBalance,
        /// 提案不存在
        ProposalNotFound,
        /// 投票期未开始或已结束
        VotingNotActive,
        /// 已经投过票
        AlreadyVoted,
        /// 不是提案发起人
        NotProposer,
        /// 投票开始后不能取消
        CannotCancelAfterVoting,
        /// 活跃提案过多
        TooManyActiveProposals,
        /// 提案间隔过短
        ProposalTooFrequent,
        /// 冷却期内不能提案
        InCooldownPeriod,
        /// 治理功能已暂停
        GovernancePausedError,
        /// 权限不足
        InsufficientAuthority,

        // === 年费价格治理错误（🆕）===
        /// 年费价格超出范围 (10-1000 USDT)
        PriceOutOfRange,
        /// 年费价格必须递增
        PriceMustBeAscending,
        /// 相邻等级价格差距过大
        PriceGapTooLarge,
        /// 年费价格提案不存在
        MembershipPriceProposalNotFound,
        /// 年费价格投票期未开始或已结束
        MembershipPriceVotingNotActive,
        /// 已经对此年费价格提案投过票
        MembershipPriceAlreadyVoted,
    }

    // ========================================
    // 可调用函数
    // ========================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // === 推荐关系接口（2个，委托到 referral pallet）===

        /// 函数级中文注释：绑定推荐人（委托到 referral pallet）
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn bind_sponsor(
            origin: OriginFor<T>,
            sponsor_code: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 🆕 2025-12-30：委托到 referral pallet
            pallet_affiliate_referral::Pallet::<T>::bind_sponsor(
                frame_system::RawOrigin::Signed(who).into(),
                sponsor_code,
            )
        }

        /// 函数级中文注释：认领推荐码（委托到 referral pallet）
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn claim_code(
            origin: OriginFor<T>,
            code: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 🆕 2025-12-30：委托到 referral pallet
            pallet_affiliate_referral::Pallet::<T>::claim_code(
                frame_system::RawOrigin::Signed(who).into(),
                code,
            )
        }

        // === 配置管理接口（4个）===

        /// 函数级中文注释：设置结算模式
        #[pallet::call_index(10)]
        #[pallet::weight(10_000)]
        pub fn set_settlement_mode(
            origin: OriginFor<T>,
            mode_id: u8,
            instant_levels: u8,
            weekly_levels: u8,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            // 构建模式
            let mode = match mode_id {
                0 => types::SettlementMode::Weekly,
                1 => types::SettlementMode::Instant,
                2 => {
                    ensure!(
                        instant_levels.saturating_add(weekly_levels) <= 15,
                        Error::<T>::HybridLevelsTooMany
                    );
                    types::SettlementMode::Hybrid {
                        instant_levels,
                        weekly_levels,
                    }
                }
                _ => return Err(Error::<T>::InvalidMode.into()),
            };

            SettlementMode::<T>::put(mode);

            Self::deposit_event(Event::SettlementModeSet);

            Ok(())
        }

        // ========================================
        // ⚠️ 已废弃：直接修改即时分成比例的接口
        // ========================================
        //
        // 为确保治理安全，InstantLevelPercents 现在只能通过全民投票治理流程修改。
        // 下列函数已被注释掉，保留代码仅供参考。
        //
        // 唯一合法的修改通道：
        // - Pallet::execute_percentage_change() - 由治理提案自动执行
        //
        // 如需修改比例，请使用：
        // - affiliate.propose_percentage_adjustment() - 发起提案
        // - affiliate.vote_on_percentage_proposal() - 社区投票
        // ========================================

        // /// 函数级中文注释：设置即时分成比例（已废弃）
        // #[pallet::call_index(11)]
        // #[pallet::weight(10_000)]
        // pub fn set_instant_percents(
        //     origin: OriginFor<T>,
        //     percents: sp_std::vec::Vec<u8>,
        // ) -> DispatchResult {
        //     T::AdminOrigin::ensure_origin(origin)?;
        //
        //     // 验证长度
        //     ensure!(percents.len() == 15, Error::<T>::InvalidPercents);
        //
        //     let bounded: types::LevelPercents = percents
        //         .try_into()
        //         .map_err(|_| Error::<T>::InvalidPercents)?;
        //
        //     InstantLevelPercents::<T>::put(bounded);
        //
        //     Self::deposit_event(Event::InstantPercentsSet);
        //
        //     Ok(())
        // }

        /// 函数级中文注释：设置周结算分成比例
        #[pallet::call_index(12)]
        #[pallet::weight(10_000)]
        pub fn set_weekly_percents(
            origin: OriginFor<T>,
            percents: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            // 验证长度
            ensure!(percents.len() == 15, Error::<T>::InvalidPercents);

            let bounded: types::LevelPercents = percents
                .try_into()
                .map_err(|_| Error::<T>::InvalidPercents)?;

            WeeklyLevelPercents::<T>::put(bounded);

            Self::deposit_event(Event::WeeklyPercentsSet);

            Ok(())
        }

        /// 函数级中文注释：设置每周区块数
        #[pallet::call_index(13)]
        #[pallet::weight(10_000)]
        pub fn set_blocks_per_week(
            origin: OriginFor<T>,
            blocks: BlockNumberFor<T>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            BlocksPerWeek::<T>::put(blocks);

            Self::deposit_event(Event::BlocksPerWeekSet { blocks });

            Ok(())
        }

        // === 周结算接口（1个）===

        /// 函数级中文注释：结算指定周期
        #[pallet::call_index(30)]
        #[pallet::weight(10_000)]
        pub fn settle_cycle(
            origin: OriginFor<T>,
            cycle: u32,
            max_accounts: u32,
        ) -> DispatchResult {
            ensure_signed(origin)?;  // 任何人都可以调用

            Self::do_settle_cycle(cycle, max_accounts)?;

            Ok(())
        }

        // === 治理接口（5个）===

        /// 发起分成比例调整提案
        ///
        /// 权限要求（满足其一）:
        /// - 持币量 ≥ 10,000 DUST（大户提案）
        /// - ≥ 1000 人联署（联署提案）
        /// - 技术委员会成员提议（委员会提案）
        ///
        /// 参数:
        /// - `new_percentages`: 新的15层分成比例
        /// - `title_cid`: 提案标题 IPFS CID
        /// - `description_cid`: 提案详情 IPFS CID
        /// - `rationale_cid`: 提案理由 IPFS CID
        #[pallet::call_index(50)]
        #[pallet::weight(Weight::from_parts(10_000_000, 0))]
        pub fn propose_percentage_adjustment(
            origin: OriginFor<T>,
            new_percentages: types::LevelPercents,
            title_cid: BoundedVec<u8, ConstU32<64>>,
            description_cid: BoundedVec<u8, ConstU32<64>>,
            rationale_cid: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查治理是否暂停
            ensure!(
                !GovernancePaused::<T>::get(),
                Error::<T>::GovernancePausedError
            );

            // 验证新比例有效性
            Self::validate_percentages(&new_percentages)?;

            // 检查反垃圾提案限制
            Self::check_proposal_spam(&who)?;

            // 验证提案权限（TODO: 实现完整的权限检查）
            // Self::ensure_proposal_authority(&who)?;

            // 计算变化幅度
            let current_percentages = InstantLevelPercents::<T>::get();
            let change_magnitude =
                Self::calculate_change_magnitude(&current_percentages, &new_percentages);

            // 🔥 2025-11-13：删除微调/重大提案区分，所有提案都使用统一押金
            // 统一押金：5000 DUST（介于原来的1000和10000之间）
            let deposit_amount = 5000u128.saturated_into();

            // 缴纳押金
            T::Currency::reserve(&who, deposit_amount)?;

            // 创建提案
            let proposal_id = NextProposalId::<T>::get();
            let current_block = <frame_system::Pallet<T>>::block_number();

            let proposal = governance::PercentageAdjustmentProposal {
                proposal_id,
                proposer: who.clone(),
                title_cid,
                description_cid,
                new_percentages: new_percentages.clone(),
                effective_block: current_block + 43200u32.into(), // 3天后生效
                rationale_cid,
                impact_analysis_cid: None,
                status: governance::ProposalStatus::Discussion,
                is_major: false, // 🔥 2025-11-13：统一设为false，因为现在都是全民投票
                created_at: current_block,
                voting_start: None,
                voting_end: None,
            };

            // 存储提案
            ActiveProposals::<T>::insert(proposal_id, &proposal);
            ProposalDeposits::<T>::insert(proposal_id, (&who, deposit_amount));

            // 更新账户提案统计
            ActiveProposalsByAccount::<T>::try_mutate(&who, |proposals| -> DispatchResult {
                proposals
                    .try_push(proposal_id)
                    .map_err(|_| Error::<T>::TooManyActiveProposals)?;
                Ok(())
            })?;

            LastProposalBlock::<T>::insert(&who, current_block);
            NextProposalId::<T>::set(proposal_id + 1);

            // 发射事件
            Self::deposit_event(Event::PercentageAdjustmentProposed {
                proposal_id,
                proposer: who,
                change_magnitude,
                is_major: false, // 🔥 2025-11-13：统一设为false，所有提案都是全民投票
            });

            Ok(())
        }

        /// 对分成比例提案投票
        ///
        /// 参数:
        /// - `proposal_id`: 提案ID
        /// - `vote_type`: 投票选项（0=Aye, 1=Nay, 2=Abstain）
        /// - `conviction_type`: 信念投票（0=None, 1=Locked1x, ..., 6=Locked6x）
        #[pallet::call_index(51)]
        #[pallet::weight(Weight::from_parts(5_000_000, 0))]
        pub fn vote_on_percentage_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
            vote_type: u8,
            conviction_type: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查治理是否暂停
            ensure!(
                !GovernancePaused::<T>::get(),
                Error::<T>::GovernancePausedError
            );

            // 验证提案存在且在投票期
            let proposal = ActiveProposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            ensure!(
                proposal.status == governance::ProposalStatus::Voting,
                Error::<T>::VotingNotActive
            );

            // 检查是否已投票
            ensure!(
                !ProposalVotes::<T>::contains_key(proposal_id, &who),
                Error::<T>::AlreadyVoted
            );

            // 转换 vote_type 为 Vote enum
            let vote = match vote_type {
                0 => governance::Vote::Aye,
                1 => governance::Vote::Nay,
                2 => governance::Vote::Abstain,
                _ => return Err(Error::<T>::InvalidPercentageLength.into()), // 复用错误码
            };

            // 转换 conviction_type 为 Conviction enum
            let conviction = match conviction_type {
                0 => governance::Conviction::None,
                1 => governance::Conviction::Locked1x,
                2 => governance::Conviction::Locked2x,
                3 => governance::Conviction::Locked3x,
                4 => governance::Conviction::Locked4x,
                5 => governance::Conviction::Locked5x,
                6 => governance::Conviction::Locked6x,
                _ => return Err(Error::<T>::InvalidPercentageLength.into()),
            };

            // 计算投票权重
            let base_weight = Self::calculate_total_voting_power(&who);
            let conviction_multiplier = conviction.multiplier();
            let final_weight = base_weight
                .saturating_mul(conviction_multiplier)
                .saturating_div(10); // 除以10因为multiplier是10倍

            // TODO: 锁定投票资金（如果有信念投票）
            // if conviction != governance::Conviction::None {
            //     let lock_amount = Self::calculate_lock_amount(&who, &conviction);
            //     T::Currency::set_lock(...);
            // }

            // 记录投票
            let vote_record = governance::VoteRecord {
                voter: who.clone(),
                vote: vote.clone(),
                conviction,
                weight: final_weight,
                timestamp: <frame_system::Pallet<T>>::block_number(),
            };

            ProposalVotes::<T>::insert(proposal_id, &who, &vote_record);

            // 更新统计
            VoteTally::<T>::mutate(proposal_id, |tally| {
                match vote {
                    governance::Vote::Aye => {
                        tally.aye_votes = tally.aye_votes.saturating_add(final_weight);
                    }
                    governance::Vote::Nay => {
                        tally.nay_votes = tally.nay_votes.saturating_add(final_weight);
                    }
                    governance::Vote::Abstain => {
                        tally.abstain_votes = tally.abstain_votes.saturating_add(final_weight);
                    }
                }
                tally.total_turnout = tally.total_turnout.saturating_add(final_weight);
            });

            // 更新投票历史
            VoteHistory::<T>::try_mutate(&who, |history| -> DispatchResult {
                history
                    .try_push(proposal_id)
                    .map_err(|_| Error::<T>::TooManyActiveProposals)?;
                Ok(())
            })?;

            // 发射事件
            Self::deposit_event(Event::VoteCast {
                proposal_id,
                voter: who,
                vote_type: vote.to_u8(),
                weight: final_weight,
            });

            Ok(())
        }

        /// 取消提案（仅提案发起人可调用）
        #[pallet::call_index(52)]
        #[pallet::weight(Weight::from_parts(3_000_000, 0))]
        pub fn cancel_proposal(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let proposal = ActiveProposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            // 检查权限
            ensure!(proposal.proposer == who, Error::<T>::NotProposer);

            // 只能在投票前取消
            ensure!(
                proposal.status == governance::ProposalStatus::Discussion,
                Error::<T>::CannotCancelAfterVoting
            );

            // 退还押金
            if let Some((_, deposit)) = ProposalDeposits::<T>::take(proposal_id) {
                T::Currency::unreserve(&who, deposit);
            }

            // 移除提案
            ActiveProposals::<T>::remove(proposal_id);

            // 更新账户统计
            ActiveProposalsByAccount::<T>::mutate(&who, |proposals| {
                proposals.retain(|&id| id != proposal_id);
            });

            Self::deposit_event(Event::ProposalCancelled {
                proposal_id,
                proposer: who,
            });

            Ok(())
        }

        /// 紧急暂停治理（仅技术委员会超级多数可调用）
        #[pallet::call_index(60)]
        #[pallet::weight(Weight::from_parts(2_000_000, 0))]
        pub fn emergency_pause_governance(
            origin: OriginFor<T>,
            reason_cid: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            // TODO: 验证技术委员会超级多数（5/7）
            T::AdminOrigin::ensure_origin(origin)?;

            GovernancePaused::<T>::put(true);
            PauseReason::<T>::put(reason_cid.clone());

            Self::deposit_event(Event::GovernanceEmergencyPaused { reason_cid });

            Ok(())
        }

        /// 恢复治理机制（仅 Root 或委员会全票可调用）
        #[pallet::call_index(61)]
        #[pallet::weight(Weight::from_parts(2_000_000, 0))]
        pub fn resume_governance(origin: OriginFor<T>) -> DispatchResult {
            // TODO: 验证 Root 或委员会全票（7/7）
            T::AdminOrigin::ensure_origin(origin)?;

            GovernancePaused::<T>::kill();
            PauseReason::<T>::kill();

            Self::deposit_event(Event::GovernanceResumed {
                by: b"Admin".to_vec().try_into().unwrap(),
            });

            Ok(())
        }

        // ========================================
        // 年费价格治理接口（🆕）
        // ========================================

        /// 发起年费价格调整提案
        #[pallet::call_index(70)]
        #[pallet::weight(Weight::from_parts(100_000_000, 0))]
        pub fn propose_membership_price_adjustment(
            origin: OriginFor<T>,
            new_prices_usdt: [u64; 4],
            title_cid: BoundedVec<u8, ConstU32<64>>,
            description_cid: BoundedVec<u8, ConstU32<64>>,
            rationale_cid: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查治理是否暂停
            ensure!(
                !GovernancePaused::<T>::get(),
                Error::<T>::GovernancePausedError
            );

            // 验证年费价格
            governance::MembershipPriceProposal::<T>::validate_prices(&new_prices_usdt)
                .map_err(|_| Error::<T>::PriceOutOfRange)?;

            // 检查反垃圾提案规则
            let active_proposals = ActiveProposalsByAccount::<T>::get(&who);
            ensure!(
                active_proposals.len() < 3,
                Error::<T>::TooManyActiveProposals
            );

            // 检查提案间隔（7天）
            if let Some(last_block) = LastProposalBlock::<T>::get(&who) {
                let current_block = frame_system::Pallet::<T>::block_number();
                ensure!(
                    current_block.saturating_sub(last_block) >= 100800u32.into(),
                    Error::<T>::ProposalTooFrequent
                );
            }

            // 获取下一个提案ID
            let proposal_id = NextProposalId::<T>::get();

            // 创建提案
            let current_block = frame_system::Pallet::<T>::block_number();
            let proposal = governance::MembershipPriceProposal::<T>::new(
                proposal_id,
                who.clone(),
                title_cid.clone(),
                description_cid.clone(),
                rationale_cid.clone(),
                new_prices_usdt,
                current_block,
            ).map_err(|_| Error::<T>::PriceOutOfRange)?;

            // 计算押金
            let deposit = proposal.calculate_deposit();

            // 扣除押金
            T::Currency::reserve(&who, deposit)?;

            // 存储提案
            ActiveMembershipPriceProposals::<T>::insert(proposal_id, &proposal);
            MembershipPriceProposalDeposits::<T>::insert(proposal_id, (&who, deposit));

            // 更新账户统计
            ActiveProposalsByAccount::<T>::try_mutate(&who, |vec| {
                vec.try_push(proposal_id).map_err(|_| Error::<T>::TooManyActiveProposals)
            })?;
            LastProposalBlock::<T>::insert(&who, current_block);

            // 更新提案ID
            NextProposalId::<T>::set(proposal_id + 1);

            // 发射事件
            Self::deposit_event(Event::MembershipPriceProposed {
                proposal_id,
                proposer: who,
                new_prices_usdt,
                is_major: proposal.is_major,
                deposit,
            });

            Ok(())
        }

        /// 对年费价格提案投票
        #[pallet::call_index(71)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn vote_on_membership_price_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
            vote_type: u8,
            conviction_type: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查治理是否暂停
            ensure!(
                !GovernancePaused::<T>::get(),
                Error::<T>::GovernancePausedError
            );

            // 验证提案是否存在
            let _proposal = ActiveMembershipPriceProposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::MembershipPriceProposalNotFound)?;

            // 检查是否已投票
            ensure!(
                !MembershipPriceProposalVotes::<T>::contains_key(proposal_id, &who),
                Error::<T>::MembershipPriceAlreadyVoted
            );

            // 验证投票类型 (0=Aye, 1=Nay, 2=Abstain)
            ensure!(vote_type <= 2, Error::<T>::InvalidPercents);

            // 验证信念投票类型 (0-6)
            ensure!(conviction_type <= 6, Error::<T>::InvalidPercents);

            // 转换投票类型
            let vote = match vote_type {
                0 => governance::Vote::Aye,
                1 => governance::Vote::Nay,
                _ => governance::Vote::Abstain,
            };

            let conviction = match conviction_type {
                0 => governance::Conviction::None,
                1 => governance::Conviction::Locked1x,
                2 => governance::Conviction::Locked2x,
                3 => governance::Conviction::Locked3x,
                4 => governance::Conviction::Locked4x,
                5 => governance::Conviction::Locked5x,
                _ => governance::Conviction::Locked6x,
            };

            // 计算投票权重
            let base_weight = Self::calculate_total_voting_power(&who);
            let conviction_multiplier = conviction.multiplier();
            let final_weight = base_weight.saturating_mul(conviction_multiplier).saturating_div(10);

            // 创建投票记录
            let vote_record = governance::VoteRecord {
                voter: who.clone(),
                vote: vote.clone(),
                conviction: conviction.clone(),
                weight: final_weight,
                timestamp: frame_system::Pallet::<T>::block_number(),
            };

            // 存储投票
            MembershipPriceProposalVotes::<T>::insert(proposal_id, &who, &vote_record);

            // 更新投票统计
            MembershipPriceVoteTally::<T>::mutate(proposal_id, |tally| {
                match vote {
                    governance::Vote::Aye => {
                        tally.aye_votes = tally.aye_votes.saturating_add(final_weight);
                    }
                    governance::Vote::Nay => {
                        tally.nay_votes = tally.nay_votes.saturating_add(final_weight);
                    }
                    governance::Vote::Abstain => {
                        tally.abstain_votes = tally.abstain_votes.saturating_add(final_weight);
                    }
                }
                tally.total_turnout = tally.total_turnout.saturating_add(final_weight);
            });

            // 更新账户投票历史
            VoteHistory::<T>::mutate(&who, |history| {
                let _ = history.try_push(proposal_id);
            });

            // 发射事件
            Self::deposit_event(Event::MembershipPriceVoteCast {
                proposal_id,
                voter: who,
                vote: vote.to_u8(),
                conviction: conviction_type,
                voting_power: final_weight.saturated_into(),
            });

            Ok(())
        }

        /// 取消年费价格提案（仅提案发起人可调用）
        #[pallet::call_index(72)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn cancel_membership_price_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let proposal = ActiveMembershipPriceProposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::MembershipPriceProposalNotFound)?;

            // 检查权限
            ensure!(proposal.proposer == who, Error::<T>::NotProposer);

            // 只能在讨论期取消
            ensure!(
                proposal.status == governance::ProposalStatus::Discussion,
                Error::<T>::CannotCancelAfterVoting
            );

            // 退还押金
            if let Some((_, deposit)) = MembershipPriceProposalDeposits::<T>::take(proposal_id) {
                T::Currency::unreserve(&who, deposit);
            }

            // 清理提案数据
            ActiveMembershipPriceProposals::<T>::remove(proposal_id);
            MembershipPriceVoteTally::<T>::remove(proposal_id);

            // 清理投票记录
            let _ = MembershipPriceProposalVotes::<T>::remove_prefix(proposal_id, None);

            // 更新账户活跃提案列表
            ActiveProposalsByAccount::<T>::mutate(&who, |vec| {
                vec.retain(|&id| id != proposal_id);
            });

            // 发射事件
            Self::deposit_event(Event::MembershipPriceProposalCancelled {
                proposal_id,
                proposer: who,
            });

            Ok(())
        }
    }

    // ========================================
    // 公开方法（供其他 pallet 调用）
    // ========================================

    impl<T: Config> Pallet<T> {
        /// 函数级中文注释：绑定推荐人（内部方法，供其他 pallet 调用）
        ///
        /// 此方法不验证，不发射事件，仅用于其他 pallet 内部绑定推荐关系。
        /// 🆕 2025-12-30：委托到 referral pallet
        pub fn bind_sponsor_internal(who: &T::AccountId, sponsor: &T::AccountId) {
            pallet_affiliate_referral::Sponsors::<T>::insert(who, sponsor);
        }

        /// 函数级中文注释：获取推荐链（代理方法）
        ///
        /// 🆕 2025-12-30：委托到 referral pallet
        ///
        /// 参数：
        /// - buyer: 购买者账户
        ///
        /// 返回：推荐链（最多15层）
        pub fn get_referral_chain(buyer: &T::AccountId) -> sp_std::vec::Vec<T::AccountId> {
            pallet_affiliate_referral::Pallet::<T>::get_referral_chain(buyer)
        }

        /// 函数级中文注释：通过推荐码查找账户（代理方法）
        ///
        /// 🆕 2025-12-30：委托到 referral pallet
        ///
        /// 参数：
        /// - code: 推荐码
        ///
        /// 返回：对应的账户（如果存在）
        pub fn find_account_by_code(
            code: &BoundedVec<u8, T::MaxCodeLen>,
        ) -> Option<T::AccountId> {
            pallet_affiliate_referral::Pallet::<T>::find_account_by_code(code)
        }

        /// 函数级详细中文注释：分配会员费奖励（供 membership pallet 调用）
        ///
        /// ## 功能
        /// - 将会员费100%分配到推荐链
        /// - 无系统扣费
        /// - 使用即时分成模式（快速到账）
        /// - 分配15层推荐链
        ///
        /// ## 参数
        /// - `buyer`: 购买会员的账户
        /// - `amount`: 会员费金额
        ///
        /// ## 返回
        /// - `Ok(distributed)`: 实际分配的金额
        /// - `Err(...)`: 分配失败原因
        ///
        /// ## 使用场景
        /// - pallet-membership::purchase() 购买会员时调用
        /// - pallet-membership::upgrade_to_year10() 升级会员时调用
        pub fn distribute_membership_rewards(
            buyer: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            Self::do_distribute_membership_rewards(buyer, amount)
        }

        // ========================================
        // 治理辅助函数
        // ========================================

        /// 执行比例调整（唯一修改通道）
        ///
        /// ⚠️ 这是修改 InstantLevelPercents 的唯一合法途径！
        /// 所有其他修改方法都应被删除。
        pub fn execute_percentage_change(
            proposal_id: &u64,
            proposal: &governance::PercentageAdjustmentProposal<T>,
        ) -> DispatchResult {
            // 验证新比例仍然有效
            Self::validate_percentages(&proposal.new_percentages)?;

            // 获取旧比例
            let old_percentages = InstantLevelPercents::<T>::get();

            // ⚠️ 唯一修改通道：通过治理提案修改
            InstantLevelPercents::<T>::put(&proposal.new_percentages);

            // 记录历史
            let change_record = governance::PercentageChangeRecord {
                proposal_id: *proposal_id,
                old_percentages,
                new_percentages: proposal.new_percentages.clone(),
                executed_at: <frame_system::Pallet<T>>::block_number(),
                executed_by: b"Governance".to_vec().try_into().unwrap(),
            };

            PercentageHistory::<T>::insert(proposal_id, &change_record);

            Ok(())
        }

        /// 退还提案押金
        pub fn return_proposal_deposit(proposal_id: &u64) {
            if let Some((account, deposit)) = ProposalDeposits::<T>::take(proposal_id) {
                T::Currency::unreserve(&account, deposit);
            }
        }

        /// 检查提案频率限制
        pub fn check_proposal_spam(proposer: &T::AccountId) -> DispatchResult {
            // 检查同时发起的提案数
            let active_proposals = ActiveProposalsByAccount::<T>::get(proposer);
            ensure!(
                active_proposals.len() <= 3,
                Error::<T>::TooManyActiveProposals
            );

            // 检查最近提案间隔（7天）
            if let Some(last_proposal_block) = LastProposalBlock::<T>::get(proposer) {
                let blocks_since =
                    <frame_system::Pallet<T>>::block_number() - last_proposal_block;
                let min_interval = 100800u32.into(); // 7天
                ensure!(
                    blocks_since >= min_interval,
                    Error::<T>::ProposalTooFrequent
                );
            }

            // 检查失败冷却期（30天）
            if let Some(cooldown_until) = ProposalCooldown::<T>::get(proposer) {
                ensure!(
                    <frame_system::Pallet<T>>::block_number() > cooldown_until,
                    Error::<T>::InCooldownPeriod
                );
            }

            Ok(())
        }

        // ========================================
        // 年费价格治理辅助函数（🆕）
        // ========================================

        /// 执行年费价格调整（唯一修改通道）
        ///
        /// ⚠️ 这是修改会员年费价格的唯一合法途径！
        /// 所有其他修改方法都应被删除。
        pub fn execute_membership_price_change(
            proposal_id: &u64,
            proposal: &governance::MembershipPriceProposal<T>,
        ) -> DispatchResult {
            // 验证新价格仍然有效
            governance::MembershipPriceProposal::<T>::validate_prices(&proposal.new_prices_usdt)
                .map_err(|_| Error::<T>::PriceOutOfRange)?;

            // 获取旧价格（从当前的价格函数获取）
            let old_prices_usdt = [50_000_000u64, 100_000_000, 200_000_000, 300_000_000];

            // ⚠️ 唯一修改通道：通过治理提案修改
            // 注意：这里需要与 pallet-membership 集成，暂时记录变更
            // TODO: 与 pallet-membership 集成，实际修改价格

            // 记录历史
            let change_record = governance::MembershipPriceChangeRecord {
                proposal_id: *proposal_id,
                old_prices_usdt,
                new_prices_usdt: proposal.new_prices_usdt,
                executed_at: <frame_system::Pallet<T>>::block_number(),
                executed_by: b"Governance".to_vec().try_into().unwrap(),
            };

            // 将记录添加到历史
            MembershipPriceHistory::<T>::mutate(|history| {
                if history.len() >= 100 {
                    let _ = history.remove(0); // 删除最老的记录
                }
                let _ = history.try_push(change_record);
            });

            Ok(())
        }

        /// 退还年费价格提案押金
        pub fn return_membership_price_proposal_deposit(proposal_id: &u64) {
            if let Some((account, deposit)) = MembershipPriceProposalDeposits::<T>::take(proposal_id) {
                T::Currency::unreserve(&account, deposit);
            }
        }

        /// 检查年费价格提案是否通过（技术委员会不可干预）
        pub fn check_membership_price_proposal_passed(
            proposal: &governance::MembershipPriceProposal<T>,
            tally: &governance::VoteTally,
        ) -> bool {
            // 直接调用内部检查逻辑
            Self::check_membership_price_proposal_passed_internal(proposal, tally)
        }

        /// 年费价格提案通过检查的内部实现
        fn check_membership_price_proposal_passed_internal(
            proposal: &governance::MembershipPriceProposal<T>,
            tally: &governance::VoteTally,
        ) -> bool {
            // TODO: 获取总投票权（这里需要实现真实的投票权计算）
            let total_power = 100000u128; // 临时值
            let participation = tally.participation_rate(total_power);

            // 最低参与率要求：15%
            if participation < sp_runtime::Perbill::from_percent(15) {
                return false;
            }

            // 年费价格治理的自适应阈值（基于参与率）
            // 注意：无论是微调还是重大提案都使用相同的全民投票逻辑
            let required_approval = if participation >= sp_runtime::Perbill::from_percent(50) {
                sp_runtime::Perbill::from_percent(50) // 50%参与 → 50%支持
            } else if participation >= sp_runtime::Perbill::from_percent(30) {
                sp_runtime::Perbill::from_percent(55) // 30%参与 → 55%支持
            } else {
                sp_runtime::Perbill::from_percent(60) // 15%参与 → 60%支持
            };

            // 对于微调年费价格提案，适当降低门槛以便于调整
            let final_approval = if !proposal.is_major {
                // 微调提案：在原基础上降低5%
                required_approval.saturating_sub(sp_runtime::Perbill::from_percent(5))
            } else {
                // 重大提案：使用标准阈值
                required_approval
            };

            tally.approval_rate() >= final_approval
        }
    }
}

// ===== 🆕 2025-10-29: Trading Pallet 集成 - AffiliateDistributor 实现 =====

/// 函数级详细中文注释：为Trading Pallet实现AffiliateDistributor
/// 
/// 这个实现提供了Trading Pallet所需的联盟奖励分配功能。
/// 根据当前的结算模式（即时/周结算/混合），自动选择分配方式。
impl<T: Config> types::AffiliateDistributor<T::AccountId, u128, BlockNumberFor<T>> 
    for Pallet<T> 
{
    fn distribute_rewards(
        _buyer: &T::AccountId,
        _amount: u128,
        _target: Option<(u8, u64)>,
    ) -> Result<u128, sp_runtime::DispatchError> {
        // TODO: 实现完整的分配逻辑
        // 1. 根据结算模式选择即时或周结算
        // 2. 调用对应的分配函数
        // 3. 返回实际分配的金额

        // 当前简化实现：直接返回Ok(0)
        // 后续需要实现完整的分配逻辑
        Ok(0)
    }
}

// 🆕 2025-12-30：MembershipProvider trait 已移动到 pallet-affiliate-referral
// 通过 pub use pallet_affiliate_referral::MembershipProvider 重新导出
