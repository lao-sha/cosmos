//! # 通用玄学占卜服务市场 Pallet
//!
//! 本模块实现了去中心化的占卜服务交易市场，支持多种玄学系统：
//! - 梅花易数
//! - 八字命理
//! - 六爻占卜
//! - 奇门遁甲
//! - 紫微斗数
//!
//! ## 核心功能
//!
//! 1. **服务提供者**: 注册、认证、等级晋升
//! 2. **服务套餐**: 文字/语音/视频/实时多种形式
//! 3. **订单系统**: 下单、支付、解读、评价完整流程
//! 4. **信誉机制**: 多维度评分、等级制度
//! 5. **收益管理**: 平台抽成、提现申请
//!
//! ## 架构说明
//!
//! 本模块通过 `DivinationProvider` trait 与各玄学核心模块解耦：
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                pallet-divination-market                 │
//! │    (通用服务市场、订单管理、评价系统)                      │
//! └──────────────────────────┬──────────────────────────────┘
//!                            │ DivinationProvider trait
//!                            ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │              Runtime: CombinedDivinationProvider        │
//! └───────┬─────────────────────────────────────┬───────────┘
//!         │                                     │
//!         ▼                                     ▼
//! ┌───────────────┐                     ┌───────────────┐
//! │ pallet-meihua │                     │ pallet-bazi   │
//! └───────────────┘                     └───────────────┘
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod types;

mod helpers;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use crate::types::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, ReservableCurrency},
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use pallet_divination_common::{DivinationProvider, DivinationType};
    use sp_runtime::traits::{Saturating, Zero};
    use sp_std::prelude::*;

    /// Pallet 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// 货币类型
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 占卜结果查询接口
        type DivinationProvider: DivinationProvider<Self::AccountId>;

        /// 最小保证金
        #[pallet::constant]
        type MinDeposit: Get<BalanceOf<Self>>;

        /// 最小服务价格
        #[pallet::constant]
        type MinServicePrice: Get<BalanceOf<Self>>;

        /// 订单超时时间（区块数）
        #[pallet::constant]
        type OrderTimeout: Get<BlockNumberFor<Self>>;

        /// 接单超时时间（区块数）
        #[pallet::constant]
        type AcceptTimeout: Get<BlockNumberFor<Self>>;

        /// 评价期限（区块数）
        #[pallet::constant]
        type ReviewPeriod: Get<BlockNumberFor<Self>>;

        /// 提现冷却期（区块数）
        #[pallet::constant]
        type WithdrawalCooldown: Get<BlockNumberFor<Self>>;

        /// 最大名称长度
        #[pallet::constant]
        type MaxNameLength: Get<u32>;

        /// 最大简介长度
        #[pallet::constant]
        type MaxBioLength: Get<u32>;

        /// 最大描述长度
        #[pallet::constant]
        type MaxDescriptionLength: Get<u32>;

        /// 最大 CID 长度
        #[pallet::constant]
        type MaxCidLength: Get<u32>;

        /// 每个提供者最大套餐数
        #[pallet::constant]
        type MaxPackagesPerProvider: Get<u32>;

        /// 每个订单最大追问数
        #[pallet::constant]
        type MaxFollowUpsPerOrder: Get<u32>;

        /// 平台收款账户
        #[pallet::constant]
        type PlatformAccount: Get<Self::AccountId>;

        /// 治理权限来源
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        // ==================== 举报系统配置 ====================

        /// 最小举报押金
        #[pallet::constant]
        type MinReportDeposit: Get<BalanceOf<Self>>;

        /// 举报处理超时时间（区块数，超时后举报者可取回押金）
        #[pallet::constant]
        type ReportTimeout: Get<BlockNumberFor<Self>>;

        /// 举报冷却期（同一用户对同一大师的举报间隔）
        #[pallet::constant]
        type ReportCooldownPeriod: Get<BlockNumberFor<Self>>;

        /// 撤回举报的时间窗口（仅在此期间内可撤回）
        #[pallet::constant]
        type ReportWithdrawWindow: Get<BlockNumberFor<Self>>;

        /// 恶意举报的信用扣分
        #[pallet::constant]
        type MaliciousReportPenalty: Get<u16>;

        /// 举报审核委员会权限来源
        type ReportReviewOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

        /// 国库账户（罚金剩余部分归国库）
        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;
    }

    /// 货币余额类型别名
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// 提供者类型别名
    pub type ProviderOf<T> = Provider<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxNameLength,
        <T as Config>::MaxBioLength,
    >;

    /// 服务套餐类型别名
    pub type ServicePackageOf<T> = ServicePackage<BalanceOf<T>, <T as Config>::MaxDescriptionLength>;

    /// 订单类型别名
    pub type OrderOf<T> = Order<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
    >;

    /// 追问类型别名
    pub type FollowUpOf<T> = FollowUp<BlockNumberFor<T>, <T as Config>::MaxCidLength>;

    /// 评价类型别名
    pub type ReviewOf<T> = Review<
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
    >;

    /// 悬赏问题类型别名
    pub type BountyQuestionOf<T> = BountyQuestion<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
    >;

    /// 悬赏回答类型别名
    pub type BountyAnswerOf<T> = BountyAnswer<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
    >;

    /// 投票记录类型别名
    pub type BountyVoteOf<T> = BountyVote<
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
    >;

    // ==================== 个人主页类型别名 ====================

    /// 提供者详细资料类型别名
    pub type ProviderProfileOf<T> = ProviderProfile<
        BlockNumberFor<T>,
        <T as Config>::MaxDescriptionLength,
        <T as Config>::MaxCidLength,
    >;

    /// 资质证书类型别名
    pub type CertificateOf<T> = Certificate<
        BlockNumberFor<T>,
        <T as Config>::MaxNameLength,
        <T as Config>::MaxCidLength,
    >;

    /// 作品集类型别名
    pub type PortfolioItemOf<T> = PortfolioItem<
        BlockNumberFor<T>,
        <T as Config>::MaxNameLength,
        <T as Config>::MaxCidLength,
    >;

    /// 技能标签类型别名
    pub type SkillTagOf = SkillTag<ConstU32<32>>;

    // ==================== 信用体系类型别名 ====================

    /// 信用档案类型别名
    pub type CreditProfileOf<T> = CreditProfile<BlockNumberFor<T>>;

    /// 违规记录类型别名
    pub type ViolationRecordOf<T> = ViolationRecord<
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
        <T as Config>::MaxDescriptionLength,
    >;

    /// 信用变更记录类型别名
    pub type CreditChangeRecordOf<T> = CreditChangeRecord<
        BlockNumberFor<T>,
        ConstU32<256>,
    >;

    /// 信用修复任务类型别名
    pub type CreditRepairTaskOf<T> = CreditRepairTask<BlockNumberFor<T>>;

    // ==================== 举报系统类型别名 ====================

    /// 举报记录类型别名
    pub type ReportOf<T> = Report<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
        <T as Config>::MaxDescriptionLength,
    >;

    /// 大师举报档案类型别名
    pub type ProviderReportProfileOf<T> = ProviderReportProfile<BlockNumberFor<T>>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ==================== 存储项 ====================

    /// 下一个订单 ID
    #[pallet::storage]
    #[pallet::getter(fn next_order_id)]
    pub type NextOrderId<T> = StorageValue<_, u64, ValueQuery>;

    /// 下一个提现请求 ID
    #[pallet::storage]
    #[pallet::getter(fn next_withdrawal_id)]
    pub type NextWithdrawalId<T> = StorageValue<_, u64, ValueQuery>;

    /// 提供者下一个套餐 ID
    #[pallet::storage]
    #[pallet::getter(fn next_package_id)]
    pub type NextPackageId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    /// 服务提供者存储
    #[pallet::storage]
    #[pallet::getter(fn providers)]
    pub type Providers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ProviderOf<T>>;

    /// 服务套餐存储（提供者 -> 套餐ID -> 套餐）
    #[pallet::storage]
    #[pallet::getter(fn packages)]
    pub type Packages<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32,
        ServicePackageOf<T>,
    >;

    /// 订单存储
    #[pallet::storage]
    #[pallet::getter(fn orders)]
    pub type Orders<T: Config> = StorageMap<_, Blake2_128Concat, u64, OrderOf<T>>;

    /// 订单追问存储
    #[pallet::storage]
    #[pallet::getter(fn follow_ups)]
    pub type FollowUps<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<FollowUpOf<T>, T::MaxFollowUpsPerOrder>,
        ValueQuery,
    >;

    /// 评价存储
    #[pallet::storage]
    #[pallet::getter(fn reviews)]
    pub type Reviews<T: Config> = StorageMap<_, Blake2_128Concat, u64, ReviewOf<T>>;

    /// 提供者收入余额
    #[pallet::storage]
    #[pallet::getter(fn provider_balances)]
    pub type ProviderBalances<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    /// 提现请求存储
    #[pallet::storage]
    #[pallet::getter(fn withdrawals)]
    pub type Withdrawals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        WithdrawalRequest<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
    >;

    /// 客户订单索引
    #[pallet::storage]
    #[pallet::getter(fn customer_orders)]
    pub type CustomerOrders<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<u64, ConstU32<1000>>, ValueQuery>;

    /// 提供者订单索引
    #[pallet::storage]
    #[pallet::getter(fn provider_orders)]
    pub type ProviderOrders<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<u64, ConstU32<1000>>, ValueQuery>;

    /// 市场统计
    #[pallet::storage]
    #[pallet::getter(fn market_stats)]
    pub type MarketStatistics<T: Config> = StorageValue<_, MarketStats<BalanceOf<T>>, ValueQuery>;

    /// 按占卜类型的市场统计
    #[pallet::storage]
    #[pallet::getter(fn type_stats)]
    pub type TypeStatistics<T: Config> =
        StorageMap<_, Blake2_128Concat, DivinationType, TypeMarketStats<BalanceOf<T>>, ValueQuery>;

    // ==================== 悬赏问答存储项 ====================

    /// 下一个悬赏问题 ID
    #[pallet::storage]
    #[pallet::getter(fn next_bounty_id)]
    pub type NextBountyId<T> = StorageValue<_, u64, ValueQuery>;

    /// 下一个悬赏回答 ID
    #[pallet::storage]
    #[pallet::getter(fn next_bounty_answer_id)]
    pub type NextBountyAnswerId<T> = StorageValue<_, u64, ValueQuery>;

    /// 悬赏问题存储
    #[pallet::storage]
    #[pallet::getter(fn bounty_questions)]
    pub type BountyQuestions<T: Config> = StorageMap<_, Blake2_128Concat, u64, BountyQuestionOf<T>>;

    /// 悬赏回答存储
    #[pallet::storage]
    #[pallet::getter(fn bounty_answers)]
    pub type BountyAnswers<T: Config> = StorageMap<_, Blake2_128Concat, u64, BountyAnswerOf<T>>;

    /// 悬赏问题的回答列表索引（bounty_id -> answer_ids）
    #[pallet::storage]
    #[pallet::getter(fn bounty_answer_ids)]
    pub type BountyAnswerIds<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BoundedVec<u64, ConstU32<100>>, ValueQuery>;

    /// 用户创建的悬赏问题索引
    #[pallet::storage]
    #[pallet::getter(fn user_bounties)]
    pub type UserBounties<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<u64, ConstU32<500>>, ValueQuery>;

    /// 用户提交的悬赏回答索引
    #[pallet::storage]
    #[pallet::getter(fn user_bounty_answers)]
    pub type UserBountyAnswers<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<u64, ConstU32<1000>>, ValueQuery>;

    /// 悬赏投票记录（bounty_id -> voter -> vote）
    #[pallet::storage]
    #[pallet::getter(fn bounty_votes)]
    pub type BountyVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        BountyVoteOf<T>,
    >;

    /// 悬赏问答统计
    #[pallet::storage]
    #[pallet::getter(fn bounty_stats)]
    pub type BountyStatistics<T: Config> = StorageValue<_, BountyStats<BalanceOf<T>>, ValueQuery>;

    // ==================== 个人主页存储项 ====================

    /// 提供者详细资料
    #[pallet::storage]
    #[pallet::getter(fn provider_profiles)]
    pub type ProviderProfiles<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ProviderProfileOf<T>>;

    /// 提供者资质证书（提供者 -> 证书ID -> 证书）
    #[pallet::storage]
    #[pallet::getter(fn certificates)]
    pub type Certificates<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32,
        CertificateOf<T>,
    >;

    /// 提供者下一个证书 ID
    #[pallet::storage]
    #[pallet::getter(fn next_certificate_id)]
    pub type NextCertificateId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    /// 提供者作品集（提供者 -> 作品ID -> 作品）
    #[pallet::storage]
    #[pallet::getter(fn portfolios)]
    pub type Portfolios<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32,
        PortfolioItemOf<T>,
    >;

    /// 提供者下一个作品 ID
    #[pallet::storage]
    #[pallet::getter(fn next_portfolio_id)]
    pub type NextPortfolioId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    /// 提供者技能标签
    #[pallet::storage]
    #[pallet::getter(fn skill_tags)]
    pub type SkillTags<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<SkillTagOf, ConstU32<20>>,
        ValueQuery,
    >;

    /// 提供者评价标签统计
    #[pallet::storage]
    #[pallet::getter(fn review_tag_stats)]
    pub type ReviewTagStatistics<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ReviewTagStats, ValueQuery>;

    /// 作品点赞记录（(提供者, 作品ID) -> 用户 -> 是否点赞）
    #[pallet::storage]
    #[pallet::getter(fn portfolio_likes)]
    pub type PortfolioLikes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::AccountId, u32),
        Blake2_128Concat,
        T::AccountId,
        bool,
        ValueQuery,
    >;

    // ==================== 信用体系存储项 ====================

    /// 提供者信用档案
    #[pallet::storage]
    #[pallet::getter(fn credit_profiles)]
    pub type CreditProfiles<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, CreditProfileOf<T>>;

    /// 违规记录存储
    #[pallet::storage]
    #[pallet::getter(fn violation_records)]
    pub type ViolationRecords<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ViolationRecordOf<T>>;

    /// 提供者违规记录索引
    #[pallet::storage]
    #[pallet::getter(fn provider_violations)]
    pub type ProviderViolations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<100>>,
        ValueQuery,
    >;

    /// 下一个违规记录 ID
    #[pallet::storage]
    #[pallet::getter(fn next_violation_id)]
    pub type NextViolationId<T> = StorageValue<_, u64, ValueQuery>;

    /// 信用变更历史（最近 50 条）
    #[pallet::storage]
    #[pallet::getter(fn credit_history)]
    pub type CreditHistory<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<CreditChangeRecordOf<T>, ConstU32<50>>,
        ValueQuery,
    >;

    /// 信用修复任务
    #[pallet::storage]
    #[pallet::getter(fn repair_tasks)]
    pub type RepairTasks<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<CreditRepairTaskOf<T>, ConstU32<5>>,
        ValueQuery,
    >;

    /// 信用黑名单（永久封禁）
    #[pallet::storage]
    #[pallet::getter(fn credit_blacklist)]
    pub type CreditBlacklist<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberFor<T>>;

    /// 全局信用统计
    #[pallet::storage]
    #[pallet::getter(fn credit_stats)]
    pub type CreditStatistics<T: Config> = StorageValue<_, GlobalCreditStats, ValueQuery>;

    // ==================== 举报系统存储项 ====================

    /// 下一个举报 ID
    #[pallet::storage]
    #[pallet::getter(fn next_report_id)]
    pub type NextReportId<T> = StorageValue<_, u64, ValueQuery>;

    /// 举报记录存储
    #[pallet::storage]
    #[pallet::getter(fn reports)]
    pub type Reports<T: Config> = StorageMap<_, Blake2_128Concat, u64, ReportOf<T>>;

    /// 大师收到的举报索引（provider -> report_ids）
    #[pallet::storage]
    #[pallet::getter(fn provider_reports)]
    pub type ProviderReports<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<500>>,
        ValueQuery,
    >;

    /// 用户提交的举报索引（reporter -> report_ids）
    #[pallet::storage]
    #[pallet::getter(fn user_reports)]
    pub type UserReports<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<100>>,
        ValueQuery,
    >;

    /// 大师举报档案
    #[pallet::storage]
    #[pallet::getter(fn provider_report_profiles)]
    pub type ProviderReportProfiles<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ProviderReportProfileOf<T>,
        ValueQuery,
    >;

    /// 待处理举报队列（按时间排序）
    #[pallet::storage]
    #[pallet::getter(fn pending_reports)]
    pub type PendingReports<T: Config> = StorageValue<
        _,
        BoundedVec<u64, ConstU32<1000>>,
        ValueQuery,
    >;

    /// 举报统计
    #[pallet::storage]
    #[pallet::getter(fn report_stats)]
    pub type ReportStatistics<T: Config> = StorageValue<_, ReportStats<BalanceOf<T>>, ValueQuery>;

    /// 举报冷却期（防止同一用户短时间内重复举报同一大师）
    /// (reporter, provider) -> last_report_block
    #[pallet::storage]
    #[pallet::getter(fn report_cooldown)]
    pub type ReportCooldown<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AccountId,
        BlockNumberFor<T>,
    >;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 提供者已注册
        ProviderRegistered {
            provider: T::AccountId,
            deposit: BalanceOf<T>,
            supported_types: u8,
        },

        /// 提供者信息已更新
        ProviderUpdated { provider: T::AccountId },

        /// 提供者已暂停
        ProviderPaused { provider: T::AccountId },

        /// 提供者已恢复
        ProviderResumed { provider: T::AccountId },

        /// 提供者已注销
        ProviderDeactivated { provider: T::AccountId },

        /// 提供者等级已提升
        ProviderTierUpgraded {
            provider: T::AccountId,
            new_tier: ProviderTier,
        },

        /// 服务套餐已创建
        PackageCreated {
            provider: T::AccountId,
            package_id: u32,
            divination_type: DivinationType,
            price: BalanceOf<T>,
        },

        /// 服务套餐已更新
        PackageUpdated {
            provider: T::AccountId,
            package_id: u32,
        },

        /// 服务套餐已删除
        PackageRemoved {
            provider: T::AccountId,
            package_id: u32,
        },

        /// 订单已创建
        OrderCreated {
            order_id: u64,
            customer: T::AccountId,
            provider: T::AccountId,
            divination_type: DivinationType,
            result_id: u64,
            amount: BalanceOf<T>,
        },

        /// 订单已支付
        OrderPaid { order_id: u64 },

        /// 订单已接受
        OrderAccepted {
            order_id: u64,
            provider: T::AccountId,
        },

        /// 订单已拒绝
        OrderRejected {
            order_id: u64,
            provider: T::AccountId,
        },

        /// 解读结果已提交（服务提供者完成解读）
        InterpretationSubmitted {
            order_id: u64,
            interpretation_cid: BoundedVec<u8, T::MaxCidLength>,
        },

        /// 订单已完成
        OrderCompleted {
            order_id: u64,
            provider_earnings: BalanceOf<T>,
            platform_fee: BalanceOf<T>,
        },

        /// 订单已取消
        OrderCancelled { order_id: u64 },

        /// 订单已退款
        OrderRefunded {
            order_id: u64,
            amount: BalanceOf<T>,
        },

        /// 追问已提交
        FollowUpSubmitted { order_id: u64, index: u32 },

        /// 追问已回复（服务提供者回复追问）
        FollowUpReplied { order_id: u64, index: u32 },

        /// 评价已提交
        ReviewSubmitted {
            order_id: u64,
            divination_type: DivinationType,
            rating: u8,
        },

        /// 提供者已回复评价
        ReviewReplied { order_id: u64 },

        /// 提现已申请
        WithdrawalRequested {
            withdrawal_id: u64,
            provider: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// 提现已完成
        WithdrawalCompleted { withdrawal_id: u64 },

        /// 提现已取消
        WithdrawalCancelled { withdrawal_id: u64 },

        // ==================== 悬赏问答事件 ====================

        /// 悬赏问题已创建
        BountyCreated {
            bounty_id: u64,
            creator: T::AccountId,
            divination_type: DivinationType,
            bounty_amount: BalanceOf<T>,
            deadline: BlockNumberFor<T>,
        },

        /// 悬赏回答已提交
        BountyAnswerSubmitted {
            answer_id: u64,
            bounty_id: u64,
            answerer: T::AccountId,
        },

        /// 悬赏问题已关闭（停止接受回答）
        BountyClosed { bounty_id: u64 },

        /// 悬赏答案已被投票
        BountyAnswerVoted {
            bounty_id: u64,
            answer_id: u64,
            voter: T::AccountId,
        },

        /// 悬赏答案已采纳（选择前三名）
        BountyAnswersAdopted {
            bounty_id: u64,
            first_place: u64,
            second_place: Option<u64>,
            third_place: Option<u64>,
        },

        /// 悬赏已结算（奖励已分配）
        BountySettled {
            bounty_id: u64,
            total_distributed: BalanceOf<T>,
            platform_fee: BalanceOf<T>,
            participant_count: u32,
        },

        /// 悬赏已取消
        BountyCancelled {
            bounty_id: u64,
            refund_amount: BalanceOf<T>,
        },

        /// 悬赏已过期
        BountyExpired {
            bounty_id: u64,
            refund_amount: BalanceOf<T>,
        },

        /// 悬赏奖励已发放
        BountyRewardPaid {
            bounty_id: u64,
            recipient: T::AccountId,
            amount: BalanceOf<T>,
            rank: u8, // 1=第一名, 2=第二名, 3=第三名, 0=参与奖
        },

        // ==================== 个人主页事件 ====================

        /// 个人资料已更新
        ProfileUpdated { provider: T::AccountId },

        /// 资质证书已添加
        CertificateAdded {
            provider: T::AccountId,
            certificate_id: u32,
        },

        /// 资质证书已删除
        CertificateRemoved {
            provider: T::AccountId,
            certificate_id: u32,
        },

        /// 资质证书验证状态已更新
        CertificateVerified {
            provider: T::AccountId,
            certificate_id: u32,
            is_verified: bool,
        },

        /// 作品已发布
        PortfolioPublished {
            provider: T::AccountId,
            portfolio_id: u32,
            divination_type: DivinationType,
        },

        /// 作品已更新
        PortfolioUpdated {
            provider: T::AccountId,
            portfolio_id: u32,
        },

        /// 作品已删除
        PortfolioRemoved {
            provider: T::AccountId,
            portfolio_id: u32,
        },

        /// 作品被点赞
        PortfolioLiked {
            provider: T::AccountId,
            portfolio_id: u32,
            liker: T::AccountId,
        },

        /// 技能标签已更新
        SkillTagsUpdated { provider: T::AccountId },

        // ==================== 信用体系事件 ====================

        /// 信用档案已创建
        CreditProfileCreated { provider: T::AccountId },

        /// 信用评估完成
        CreditEvaluated {
            provider: T::AccountId,
            new_score: u16,
            new_level: CreditLevel,
        },

        /// 信用等级变更
        CreditLevelChanged {
            provider: T::AccountId,
            old_level: CreditLevel,
            new_level: CreditLevel,
        },

        /// 违规记录创建
        ViolationRecorded {
            provider: T::AccountId,
            violation_id: u64,
            violation_type: ViolationType,
            penalty: PenaltyType,
            deduction_points: u16,
        },

        /// 违规申诉提交
        ViolationAppealed {
            provider: T::AccountId,
            violation_id: u64,
        },

        /// 申诉结果处理完成
        AppealResolved {
            provider: T::AccountId,
            violation_id: u64,
            result: AppealResult,
            restored_points: u16,
        },

        /// 信用修复任务申请
        CreditRepairRequested {
            provider: T::AccountId,
            task_type: RepairTaskType,
            target_value: u32,
        },

        /// 信用修复任务完成
        CreditRepairCompleted {
            provider: T::AccountId,
            task_type: RepairTaskType,
            restored_points: u16,
        },

        /// 加入信用黑名单
        AddedToBlacklist { provider: T::AccountId },

        // ==================== 举报系统事件 ====================

        /// 举报已提交
        ReportSubmitted {
            report_id: u64,
            reporter: Option<T::AccountId>, // 匿名时为 None
            provider: T::AccountId,
            report_type: ReportType,
            deposit: BalanceOf<T>,
        },

        /// 举报已撤回
        ReportWithdrawn { report_id: u64 },

        /// 举报审核完成
        ReportResolved {
            report_id: u64,
            result: ReportStatus,
            resolver: T::AccountId,
        },

        /// 举报成立
        ReportUpheld {
            report_id: u64,
            provider: T::AccountId,
            penalty_amount: BalanceOf<T>,
            reporter_reward: BalanceOf<T>,
            is_banned: bool,
        },

        /// 举报驳回
        ReportRejected {
            report_id: u64,
            reporter: T::AccountId,
            deposit_refunded: BalanceOf<T>,
        },

        /// 恶意举报被处罚
        MaliciousReportPenalized {
            report_id: u64,
            reporter: T::AccountId,
            deposit_confiscated: BalanceOf<T>,
        },

        /// 举报已过期
        ReportExpired { report_id: u64 },

        /// 大师被封禁
        ProviderBanned {
            provider: T::AccountId,
            reason: ReportType,
        },

        /// 大师进入观察期
        ProviderUnderWatch {
            provider: T::AccountId,
            watch_end: BlockNumberFor<T>,
        },
    }

    // ==================== 错误 ====================

    #[pallet::error]
    pub enum Error<T> {
        /// 提供者已存在
        ProviderAlreadyExists,
        /// 提供者不存在
        ProviderNotFound,
        /// 提供者未激活
        ProviderNotActive,
        /// 保证金不足
        InsufficientDeposit,
        /// 套餐不存在
        PackageNotFound,
        /// 套餐已达上限
        TooManyPackages,
        /// 价格低于最低限制
        PriceTooLow,
        /// 订单不存在
        OrderNotFound,
        /// 订单状态无效
        InvalidOrderStatus,
        /// 非订单所有者
        NotOrderOwner,
        /// 非服务提供者
        NotProvider,
        /// 余额不足
        InsufficientBalance,
        /// 无追问次数
        NoFollowUpsRemaining,
        /// 追问不存在
        FollowUpNotFound,
        /// 已评价
        AlreadyReviewed,
        /// 评分无效
        InvalidRating,
        /// 评价期已过
        ReviewPeriodExpired,
        /// 提现金额无效
        InvalidWithdrawalAmount,
        /// 提现请求不存在
        WithdrawalNotFound,
        /// 名称过长
        NameTooLong,
        /// 简介过长
        BioTooLong,
        /// 描述过长
        DescriptionTooLong,
        /// CID 过长
        CidTooLong,
        /// 订单列表已满
        OrderListFull,
        /// 追问列表已满
        FollowUpListFull,
        /// 不能给自己下单
        CannotOrderSelf,
        /// 提供者已被封禁
        ProviderBanned,
        /// 占卜结果不存在
        DivinationResultNotFound,
        /// 不是占卜结果的创建者
        NotResultCreator,
        /// 提供者不支持该占卜类型
        DivinationTypeNotSupported,
        /// 提供者状态无效（非预期的状态转换）
        InvalidProviderStatus,
        /// 加急服务不可用
        UrgentNotAvailable,
        /// 投票功能未启用
        VotingNotAllowed,
        /// 悬赏未被采纳
        BountyNotAdopted,

        // ==================== 悬赏问答错误 ====================

        /// 悬赏问题不存在
        BountyNotFound,
        /// 悬赏问题不是开放状态
        BountyNotOpen,
        /// 悬赏问题已关闭
        BountyAlreadyClosed,
        /// 悬赏回答不存在
        BountyAnswerNotFound,
        /// 不能回答自己的悬赏
        CannotAnswerOwnBounty,
        /// 已经回答过该悬赏
        AlreadyAnswered,
        /// 悬赏回答数已达上限
        BountyAnswerLimitReached,
        /// 不是悬赏创建者
        NotBountyCreator,
        /// 悬赏金额过低
        BountyAmountTooLow,
        /// 悬赏已过截止时间
        BountyDeadlinePassed,
        /// 悬赏截止时间无效
        InvalidBountyDeadline,
        /// 回答数不足以采纳
        NotEnoughAnswers,
        /// 已投票
        AlreadyVoted,
        /// 悬赏已被采纳
        BountyAlreadyAdopted,
        /// 悬赏已结算
        BountyAlreadySettled,
        /// 悬赏不能取消（已有回答）
        BountyCannotCancel,
        /// 悬赏未过期
        BountyNotExpired,
        /// 仅限认证提供者
        CertifiedProviderOnly,
        /// 悬赏列表已满
        BountyListFull,
        /// 奖励分配比例无效
        InvalidRewardDistribution,

        // ==================== 个人主页错误 ====================

        /// 资质证书不存在
        CertificateNotFound,
        /// 证书数量已达上限
        TooManyCertificates,
        /// 作品不存在
        PortfolioNotFound,
        /// 作品数量已达上限
        TooManyPortfolios,
        /// 已点赞
        AlreadyLiked,
        /// 标签数量过多
        TooManyTags,

        // ==================== 信用体系错误 ====================

        /// 信用档案不存在
        CreditProfileNotFound,
        /// 违规记录不存在
        ViolationNotFound,
        /// 不是违规记录所有者
        NotViolationOwner,
        /// 已申诉
        AlreadyAppealed,
        /// 违规已过期
        ViolationExpired,
        /// 未申诉
        NotAppealed,
        /// 信用分过高，无需修复
        CreditTooHighForRepair,
        /// 重复的修复任务
        DuplicateRepairTask,
        /// 活跃任务过多
        TooManyActiveTasks,
        /// 任务数量过多
        TooManyTasks,
        /// 违规记录过多
        TooManyViolations,
        /// 已被列入黑名单
        InBlacklist,
        /// 信用等级不足
        InsufficientCreditLevel,

        // ==================== 举报系统错误 ====================

        /// 不能举报自己
        CannotReportSelf,
        /// 举报冷却期中
        ReportCooldownActive,
        /// 举报不存在
        ReportNotFound,
        /// 不是举报者
        NotReporter,
        /// 举报非待处理状态
        ReportNotPending,
        /// 撤回窗口已过期
        WithdrawWindowExpired,
        /// 举报已处理
        ReportAlreadyResolved,
        /// 无效的审核结果
        InvalidReportResult,
        /// 举报未过期
        ReportNotExpired,
        /// 举报过多
        TooManyReports,
        /// 待处理举报过多
        TooManyPendingReports,
        /// 大师已被封禁（举报相关）
        ProviderAlreadyBanned,
    }

    // ==================== 可调用函数 ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 注册成为服务提供者
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `name`: 显示名称
        /// - `bio`: 个人简介
        /// - `specialties`: 擅长领域位图
        /// - `supported_divination_types`: 支持的占卜类型位图
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn register_provider(
            origin: OriginFor<T>,
            name: Vec<u8>,
            bio: Vec<u8>,
            specialties: u16,
            supported_divination_types: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 确保未注册
            ensure!(
                !Providers::<T>::contains_key(&who),
                Error::<T>::ProviderAlreadyExists
            );

            let name_bounded: BoundedVec<u8, T::MaxNameLength> =
                BoundedVec::try_from(name).map_err(|_| Error::<T>::NameTooLong)?;
            let bio_bounded: BoundedVec<u8, T::MaxBioLength> =
                BoundedVec::try_from(bio).map_err(|_| Error::<T>::BioTooLong)?;

            // 锁定保证金
            let deposit = T::MinDeposit::get();
            T::Currency::reserve(&who, deposit)?;

            let block_number = <frame_system::Pallet<T>>::block_number();

            let provider = Provider {
                account: who.clone(),
                name: name_bounded,
                bio: bio_bounded,
                avatar_cid: None,
                tier: ProviderTier::Novice,
                status: ProviderStatus::Active,
                deposit,
                registered_at: block_number,
                total_orders: 0,
                completed_orders: 0,
                cancelled_orders: 0,
                total_ratings: 0,
                rating_sum: 0,
                total_earnings: Zero::zero(),
                specialties,
                supported_divination_types,
                accepts_urgent: false,
                last_active_at: block_number,
            };

            Providers::<T>::insert(&who, provider);

            // 更新统计
            MarketStatistics::<T>::mutate(|s| s.active_providers += 1);

            Self::deposit_event(Event::ProviderRegistered {
                provider: who,
                deposit,
                supported_types: supported_divination_types,
            });

            Ok(())
        }

        /// 更新提供者信息
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn update_provider(
            origin: OriginFor<T>,
            name: Option<Vec<u8>>,
            bio: Option<Vec<u8>>,
            avatar_cid: Option<Vec<u8>>,
            specialties: Option<u16>,
            supported_divination_types: Option<u8>,
            accepts_urgent: Option<bool>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Providers::<T>::try_mutate(&who, |maybe_provider| {
                let provider = maybe_provider.as_mut().ok_or(Error::<T>::ProviderNotFound)?;

                if let Some(n) = name {
                    provider.name =
                        BoundedVec::try_from(n).map_err(|_| Error::<T>::NameTooLong)?;
                }
                if let Some(b) = bio {
                    provider.bio = BoundedVec::try_from(b).map_err(|_| Error::<T>::BioTooLong)?;
                }
                if let Some(cid) = avatar_cid {
                    provider.avatar_cid =
                        Some(BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?);
                }
                if let Some(s) = specialties {
                    provider.specialties = s;
                }
                if let Some(types) = supported_divination_types {
                    provider.supported_divination_types = types;
                }
                if let Some(u) = accepts_urgent {
                    provider.accepts_urgent = u;
                }

                provider.last_active_at = <frame_system::Pallet<T>>::block_number();

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::ProviderUpdated { provider: who });

            Ok(())
        }

        /// 暂停接单
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn pause_provider(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Providers::<T>::try_mutate(&who, |maybe_provider| {
                let provider = maybe_provider.as_mut().ok_or(Error::<T>::ProviderNotFound)?;
                ensure!(
                    provider.status == ProviderStatus::Active,
                    Error::<T>::ProviderNotActive
                );
                provider.status = ProviderStatus::Paused;
                Ok::<_, DispatchError>(())
            })?;

            MarketStatistics::<T>::mutate(|s| {
                s.active_providers = s.active_providers.saturating_sub(1)
            });

            Self::deposit_event(Event::ProviderPaused { provider: who });

            Ok(())
        }

        /// 恢复接单
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn resume_provider(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Providers::<T>::try_mutate(&who, |maybe_provider| {
                let provider = maybe_provider.as_mut().ok_or(Error::<T>::ProviderNotFound)?;
                ensure!(
                    provider.status == ProviderStatus::Paused,
                    Error::<T>::InvalidProviderStatus
                );
                provider.status = ProviderStatus::Active;
                provider.last_active_at = <frame_system::Pallet<T>>::block_number();
                Ok::<_, DispatchError>(())
            })?;

            MarketStatistics::<T>::mutate(|s| s.active_providers += 1);

            Self::deposit_event(Event::ProviderResumed { provider: who });

            Ok(())
        }

        /// 注销提供者（需要无进行中订单）
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn deactivate_provider(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let provider = Providers::<T>::get(&who).ok_or(Error::<T>::ProviderNotFound)?;

            // 退还保证金
            T::Currency::unreserve(&who, provider.deposit);

            // 退还余额
            let balance = ProviderBalances::<T>::take(&who);
            if !balance.is_zero() {
                T::Currency::transfer(
                    &T::PlatformAccount::get(),
                    &who,
                    balance,
                    ExistenceRequirement::KeepAlive,
                )?;
            }

            Providers::<T>::remove(&who);

            MarketStatistics::<T>::mutate(|s| {
                if provider.status == ProviderStatus::Active {
                    s.active_providers = s.active_providers.saturating_sub(1);
                }
            });

            Self::deposit_event(Event::ProviderDeactivated { provider: who });

            Ok(())
        }

        /// 创建服务套餐
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn create_package(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            service_type: ServiceType,
            name: Vec<u8>,
            description: Vec<u8>,
            price: BalanceOf<T>,
            duration: u32,
            follow_up_count: u8,
            urgent_available: bool,
            urgent_surcharge: u16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证提供者
            let provider = Providers::<T>::get(&who).ok_or(Error::<T>::ProviderNotFound)?;
            ensure!(
                provider.supports_divination_type(divination_type),
                Error::<T>::DivinationTypeNotSupported
            );
            ensure!(price >= T::MinServicePrice::get(), Error::<T>::PriceTooLow);

            let name_bounded: BoundedVec<u8, ConstU32<64>> =
                BoundedVec::try_from(name).map_err(|_| Error::<T>::NameTooLong)?;
            let desc_bounded: BoundedVec<u8, T::MaxDescriptionLength> =
                BoundedVec::try_from(description).map_err(|_| Error::<T>::DescriptionTooLong)?;

            let package_id = NextPackageId::<T>::get(&who);
            ensure!(
                package_id < T::MaxPackagesPerProvider::get(),
                Error::<T>::TooManyPackages
            );

            let package = ServicePackage {
                id: package_id,
                divination_type,
                service_type,
                name: name_bounded,
                description: desc_bounded,
                price,
                duration,
                follow_up_count,
                urgent_available,
                urgent_surcharge,
                is_active: true,
                sales_count: 0,
            };

            Packages::<T>::insert(&who, package_id, package);
            NextPackageId::<T>::insert(&who, package_id.saturating_add(1));

            Self::deposit_event(Event::PackageCreated {
                provider: who,
                package_id,
                divination_type,
                price,
            });

            Ok(())
        }

        /// 更新服务套餐
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn update_package(
            origin: OriginFor<T>,
            package_id: u32,
            price: Option<BalanceOf<T>>,
            description: Option<Vec<u8>>,
            is_active: Option<bool>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Packages::<T>::try_mutate(&who, package_id, |maybe_package| {
                let package = maybe_package.as_mut().ok_or(Error::<T>::PackageNotFound)?;

                if let Some(p) = price {
                    ensure!(p >= T::MinServicePrice::get(), Error::<T>::PriceTooLow);
                    package.price = p;
                }
                if let Some(d) = description {
                    package.description =
                        BoundedVec::try_from(d).map_err(|_| Error::<T>::DescriptionTooLong)?;
                }
                if let Some(a) = is_active {
                    package.is_active = a;
                }

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::PackageUpdated {
                provider: who,
                package_id,
            });

            Ok(())
        }

        /// 删除服务套餐
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn remove_package(origin: OriginFor<T>, package_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Packages::<T>::contains_key(&who, package_id),
                Error::<T>::PackageNotFound
            );
            Packages::<T>::remove(&who, package_id);

            Self::deposit_event(Event::PackageRemoved {
                provider: who,
                package_id,
            });

            Ok(())
        }

        /// 创建订单
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn create_order(
            origin: OriginFor<T>,
            provider_account: T::AccountId,
            divination_type: DivinationType,
            result_id: u64,
            package_id: u32,
            question_cid: Vec<u8>,
            is_urgent: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 不能给自己下单
            ensure!(who != provider_account, Error::<T>::CannotOrderSelf);

            // 验证占卜结果存在
            ensure!(
                T::DivinationProvider::result_exists(divination_type, result_id),
                Error::<T>::DivinationResultNotFound
            );

            // 验证提供者
            let provider =
                Providers::<T>::get(&provider_account).ok_or(Error::<T>::ProviderNotFound)?;
            ensure!(
                provider.status == ProviderStatus::Active,
                Error::<T>::ProviderNotActive
            );
            ensure!(
                provider.status != ProviderStatus::Banned,
                Error::<T>::ProviderBanned
            );
            ensure!(
                provider.supports_divination_type(divination_type),
                Error::<T>::DivinationTypeNotSupported
            );

            // 验证套餐
            let package = Packages::<T>::get(&provider_account, package_id)
                .ok_or(Error::<T>::PackageNotFound)?;
            ensure!(package.is_active, Error::<T>::PackageNotFound);
            ensure!(
                package.divination_type == divination_type,
                Error::<T>::DivinationTypeNotSupported
            );

            // 验证加急
            if is_urgent {
                ensure!(
                    package.urgent_available && provider.accepts_urgent,
                    Error::<T>::UrgentNotAvailable
                );
            }

            let question_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(question_cid).map_err(|_| Error::<T>::CidTooLong)?;

            // 计算价格
            let mut amount = package.price;
            if is_urgent {
                let surcharge =
                    amount.saturating_mul(package.urgent_surcharge.into()) / 10000u32.into();
                amount = amount.saturating_add(surcharge);
            }

            // 计算平台手续费
            let platform_fee_rate = provider.tier.platform_fee_rate();
            let platform_fee =
                amount.saturating_mul(platform_fee_rate.into()) / 10000u32.into();

            // 扣款到平台账户（托管）
            T::Currency::transfer(
                &who,
                &T::PlatformAccount::get(),
                amount,
                ExistenceRequirement::KeepAlive,
            )?;

            let order_id = NextOrderId::<T>::get();
            NextOrderId::<T>::put(order_id.saturating_add(1));

            let block_number = <frame_system::Pallet<T>>::block_number();

            let order = Order {
                id: order_id,
                customer: who.clone(),
                provider: provider_account.clone(),
                divination_type,
                result_id,
                package_id,
                amount,
                platform_fee,
                is_urgent,
                status: OrderStatus::Paid,
                question_cid: question_cid_bounded,
                interpretation_cid: None,
                created_at: block_number,
                paid_at: Some(block_number),
                accepted_at: None,
                completed_at: None,
                follow_ups_remaining: package.follow_up_count,
                rating: None,
                review_cid: None,
            };

            Orders::<T>::insert(order_id, order);

            // 更新索引
            CustomerOrders::<T>::try_mutate(&who, |list| {
                list.try_push(order_id)
                    .map_err(|_| Error::<T>::OrderListFull)
            })?;
            ProviderOrders::<T>::try_mutate(&provider_account, |list| {
                list.try_push(order_id)
                    .map_err(|_| Error::<T>::OrderListFull)
            })?;

            // 更新套餐销量
            Packages::<T>::mutate(&provider_account, package_id, |maybe_package| {
                if let Some(p) = maybe_package {
                    p.sales_count += 1;
                }
            });

            // 更新统计
            MarketStatistics::<T>::mutate(|s| {
                s.total_orders += 1;
                s.total_volume = s.total_volume.saturating_add(amount);
            });
            TypeStatistics::<T>::mutate(divination_type, |s| {
                s.order_count += 1;
                s.volume = s.volume.saturating_add(amount);
            });

            Self::deposit_event(Event::OrderCreated {
                order_id,
                customer: who,
                provider: provider_account,
                divination_type,
                result_id,
                amount,
            });

            Self::deposit_event(Event::OrderPaid { order_id });

            Ok(())
        }

        /// 接受订单
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn accept_order(origin: OriginFor<T>, order_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Orders::<T>::try_mutate(order_id, |maybe_order| {
                let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                ensure!(order.provider == who, Error::<T>::NotProvider);
                ensure!(
                    order.status == OrderStatus::Paid,
                    Error::<T>::InvalidOrderStatus
                );

                order.status = OrderStatus::Accepted;
                order.accepted_at = Some(<frame_system::Pallet<T>>::block_number());

                Ok::<_, DispatchError>(())
            })?;

            // 更新提供者活跃时间
            Providers::<T>::mutate(&who, |maybe_provider| {
                if let Some(p) = maybe_provider {
                    p.last_active_at = <frame_system::Pallet<T>>::block_number();
                }
            });

            Self::deposit_event(Event::OrderAccepted {
                order_id,
                provider: who,
            });

            Ok(())
        }

        /// 拒绝订单（退款给客户）
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn reject_order(origin: OriginFor<T>, order_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.provider == who, Error::<T>::NotProvider);
            ensure!(
                order.status == OrderStatus::Paid,
                Error::<T>::InvalidOrderStatus
            );

            // 退款给客户
            T::Currency::transfer(
                &T::PlatformAccount::get(),
                &order.customer,
                order.amount,
                ExistenceRequirement::KeepAlive,
            )?;

            Orders::<T>::mutate(order_id, |maybe_order| {
                if let Some(o) = maybe_order {
                    o.status = OrderStatus::Cancelled;
                }
            });

            Self::deposit_event(Event::OrderRejected {
                order_id,
                provider: who,
            });

            Ok(())
        }

        /// 提交解读结果
        ///
        /// 服务提供者完成对客户问题的专业解读并提交结果
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn submit_interpretation(
            origin: OriginFor<T>,
            order_id: u64,
            interpretation_cid: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let interpretation_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(interpretation_cid.clone()).map_err(|_| Error::<T>::CidTooLong)?;

            let divination_type = Orders::<T>::try_mutate(order_id, |maybe_order| {
                let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                ensure!(order.provider == who, Error::<T>::NotProvider);
                ensure!(
                    order.status == OrderStatus::Accepted,
                    Error::<T>::InvalidOrderStatus
                );

                order.interpretation_cid = Some(interpretation_cid_bounded.clone());
                order.status = OrderStatus::Completed;
                order.completed_at = Some(<frame_system::Pallet<T>>::block_number());

                Ok::<_, DispatchError>(order.divination_type)
            })?;

            // 结算费用
            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            let provider_earnings = order.amount.saturating_sub(order.platform_fee);

            // 转给提供者余额
            ProviderBalances::<T>::mutate(&who, |balance| {
                *balance = balance.saturating_add(provider_earnings);
            });

            // 更新提供者统计
            Providers::<T>::mutate(&who, |maybe_provider| {
                if let Some(p) = maybe_provider {
                    p.total_orders += 1;
                    p.completed_orders += 1;
                    p.total_earnings = p.total_earnings.saturating_add(provider_earnings);
                    p.last_active_at = <frame_system::Pallet<T>>::block_number();
                }
            });

            // 更新市场统计
            MarketStatistics::<T>::mutate(|s| {
                s.completed_orders += 1;
                s.platform_earnings = s.platform_earnings.saturating_add(order.platform_fee);
            });
            TypeStatistics::<T>::mutate(divination_type, |s| {
                s.completed_count += 1;
            });

            Self::deposit_event(Event::InterpretationSubmitted {
                order_id,
                interpretation_cid: interpretation_cid_bounded,
            });

            Self::deposit_event(Event::OrderCompleted {
                order_id,
                provider_earnings,
                platform_fee: order.platform_fee,
            });

            Ok(())
        }

        /// 提交追问
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn submit_follow_up(
            origin: OriginFor<T>,
            order_id: u64,
            question_cid: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let question_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(question_cid).map_err(|_| Error::<T>::CidTooLong)?;

            // 验证订单
            Orders::<T>::try_mutate(order_id, |maybe_order| {
                let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                ensure!(order.customer == who, Error::<T>::NotOrderOwner);
                ensure!(
                    order.status == OrderStatus::Completed,
                    Error::<T>::InvalidOrderStatus
                );
                ensure!(
                    order.follow_ups_remaining > 0,
                    Error::<T>::NoFollowUpsRemaining
                );

                order.follow_ups_remaining -= 1;

                Ok::<_, DispatchError>(())
            })?;

            let follow_up = FollowUp {
                question_cid: question_cid_bounded,
                reply_cid: None,
                asked_at: <frame_system::Pallet<T>>::block_number(),
                replied_at: None,
            };

            let index = FollowUps::<T>::try_mutate(order_id, |list| {
                let idx = list.len() as u32;
                list.try_push(follow_up)
                    .map_err(|_| Error::<T>::FollowUpListFull)?;
                Ok::<u32, DispatchError>(idx)
            })?;

            Self::deposit_event(Event::FollowUpSubmitted { order_id, index });

            Ok(())
        }

        /// 回复追问
        ///
        /// 服务提供者对客户追问进行回复
        #[pallet::call_index(13)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn reply_follow_up(
            origin: OriginFor<T>,
            order_id: u64,
            follow_up_index: u32,
            reply_cid: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let reply_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(reply_cid).map_err(|_| Error::<T>::CidTooLong)?;

            // 验证订单
            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.provider == who, Error::<T>::NotProvider);

            FollowUps::<T>::try_mutate(order_id, |list| {
                let follow_up = list
                    .get_mut(follow_up_index as usize)
                    .ok_or(Error::<T>::FollowUpNotFound)?;
                follow_up.reply_cid = Some(reply_cid_bounded);
                follow_up.replied_at = Some(<frame_system::Pallet<T>>::block_number());
                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::FollowUpReplied {
                order_id,
                index: follow_up_index,
            });

            Ok(())
        }

        /// 提交评价
        #[pallet::call_index(14)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn submit_review(
            origin: OriginFor<T>,
            order_id: u64,
            overall_rating: u8,
            accuracy_rating: u8,
            attitude_rating: u8,
            response_rating: u8,
            content_cid: Option<Vec<u8>>,
            is_anonymous: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证评分
            ensure!(
                overall_rating >= 1
                    && overall_rating <= 5
                    && accuracy_rating >= 1
                    && accuracy_rating <= 5
                    && attitude_rating >= 1
                    && attitude_rating <= 5
                    && response_rating >= 1
                    && response_rating <= 5,
                Error::<T>::InvalidRating
            );

            // 验证订单
            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.customer == who, Error::<T>::NotOrderOwner);
            ensure!(
                order.status == OrderStatus::Completed,
                Error::<T>::InvalidOrderStatus
            );

            // 检查是否已评价
            ensure!(
                !Reviews::<T>::contains_key(order_id),
                Error::<T>::AlreadyReviewed
            );

            // 检查评价期限
            let current_block = <frame_system::Pallet<T>>::block_number();
            if let Some(completed_at) = order.completed_at {
                ensure!(
                    current_block <= completed_at + T::ReviewPeriod::get(),
                    Error::<T>::ReviewPeriodExpired
                );
            }

            let content_cid_bounded = content_cid
                .map(|cid| BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            let review = Review {
                order_id,
                reviewer: who.clone(),
                reviewee: order.provider.clone(),
                divination_type: order.divination_type,
                overall_rating,
                accuracy_rating,
                attitude_rating,
                response_rating,
                content_cid: content_cid_bounded,
                created_at: current_block,
                is_anonymous,
                provider_reply_cid: None,
            };

            Reviews::<T>::insert(order_id, review);

            // 更新订单状态
            Orders::<T>::mutate(order_id, |maybe_order| {
                if let Some(o) = maybe_order {
                    o.status = OrderStatus::Reviewed;
                    o.rating = Some(overall_rating);
                }
            });

            // 更新提供者评分
            Providers::<T>::mutate(&order.provider, |maybe_provider| {
                if let Some(p) = maybe_provider {
                    p.total_ratings += 1;
                    p.rating_sum += overall_rating as u64;

                    // 检查是否可以升级
                    Self::try_upgrade_tier(p);
                }
            });

            // 更新市场统计
            MarketStatistics::<T>::mutate(|s| {
                s.total_reviews += 1;
                // 简单计算平均评分
                let total =
                    s.average_rating as u64 * (s.total_reviews - 1) + overall_rating as u64 * 100;
                s.average_rating = (total / s.total_reviews) as u16;
            });

            Self::deposit_event(Event::ReviewSubmitted {
                order_id,
                divination_type: order.divination_type,
                rating: overall_rating,
            });

            Ok(())
        }

        /// 提供者回复评价
        #[pallet::call_index(15)]
        #[pallet::weight(Weight::from_parts(25_000_000, 0))]
        pub fn reply_review(
            origin: OriginFor<T>,
            order_id: u64,
            reply_cid: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let reply_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(reply_cid).map_err(|_| Error::<T>::CidTooLong)?;

            Reviews::<T>::try_mutate(order_id, |maybe_review| {
                let review = maybe_review.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                ensure!(review.reviewee == who, Error::<T>::NotProvider);

                review.provider_reply_cid = Some(reply_cid_bounded);

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::ReviewReplied { order_id });

            Ok(())
        }

        /// 申请提现
        #[pallet::call_index(16)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn request_withdrawal(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Providers::<T>::contains_key(&who),
                Error::<T>::ProviderNotFound
            );

            let balance = ProviderBalances::<T>::get(&who);
            ensure!(balance >= amount, Error::<T>::InsufficientBalance);
            ensure!(!amount.is_zero(), Error::<T>::InvalidWithdrawalAmount);

            // 扣除余额
            ProviderBalances::<T>::mutate(&who, |b| {
                *b = b.saturating_sub(amount);
            });

            // 转账给提供者
            T::Currency::transfer(
                &T::PlatformAccount::get(),
                &who,
                amount,
                ExistenceRequirement::KeepAlive,
            )?;

            let withdrawal_id = NextWithdrawalId::<T>::get();
            NextWithdrawalId::<T>::put(withdrawal_id.saturating_add(1));

            let withdrawal = WithdrawalRequest {
                id: withdrawal_id,
                provider: who.clone(),
                amount,
                status: WithdrawalStatus::Completed,
                requested_at: <frame_system::Pallet<T>>::block_number(),
                processed_at: Some(<frame_system::Pallet<T>>::block_number()),
            };

            Withdrawals::<T>::insert(withdrawal_id, withdrawal);

            Self::deposit_event(Event::WithdrawalRequested {
                withdrawal_id,
                provider: who,
                amount,
            });

            Self::deposit_event(Event::WithdrawalCompleted { withdrawal_id });

            Ok(())
        }

        /// 取消订单（仅限未接单状态）
        #[pallet::call_index(17)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn cancel_order(origin: OriginFor<T>, order_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.customer == who, Error::<T>::NotOrderOwner);
            ensure!(
                order.status == OrderStatus::Paid,
                Error::<T>::InvalidOrderStatus
            );

            // 退款
            T::Currency::transfer(
                &T::PlatformAccount::get(),
                &who,
                order.amount,
                ExistenceRequirement::KeepAlive,
            )?;

            Orders::<T>::mutate(order_id, |maybe_order| {
                if let Some(o) = maybe_order {
                    o.status = OrderStatus::Cancelled;
                }
            });

            Self::deposit_event(Event::OrderCancelled { order_id });

            Ok(())
        }

        // ==================== 悬赏问答可调用函数 ====================

        /// 创建悬赏问题
        ///
        /// # 参数
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 关联的占卜结果 ID（可选）
        /// - `question_cid`: 问题描述 IPFS CID
        /// - `bounty_amount`: 悬赏金额
        /// - `deadline`: 截止区块
        /// - `min_answers`: 最小回答数
        /// - `max_answers`: 最大回答数
        /// - `specialty`: 擅长领域（可选）
        /// - `certified_only`: 是否仅限认证提供者回答
        /// - `allow_voting`: 是否允许社区投票
        #[pallet::call_index(18)]
        #[pallet::weight(Weight::from_parts(60_000_000, 0))]
        pub fn create_bounty(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            result_id: u64,
            question_cid: Vec<u8>,
            bounty_amount: BalanceOf<T>,
            deadline: BlockNumberFor<T>,
            min_answers: u8,
            max_answers: u8,
            specialty: Option<Specialty>,
            certified_only: bool,
            allow_voting: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证悬赏金额
            ensure!(
                bounty_amount >= T::MinServicePrice::get(),
                Error::<T>::BountyAmountTooLow
            );

            // 验证截止时间
            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(deadline > current_block, Error::<T>::InvalidBountyDeadline);

            // 验证占卜结果存在（悬赏必须基于已存在的占卜结果）
            ensure!(
                T::DivinationProvider::result_exists(divination_type, result_id),
                Error::<T>::DivinationResultNotFound
            );

            // 验证调用者是占卜结果的创建者
            let result_creator = T::DivinationProvider::result_creator(divination_type, result_id)
                .ok_or(Error::<T>::DivinationResultNotFound)?;
            ensure!(result_creator == who, Error::<T>::NotResultCreator);

            let question_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(question_cid).map_err(|_| Error::<T>::CidTooLong)?;

            // 转账悬赏金到平台账户托管
            T::Currency::transfer(
                &who,
                &T::PlatformAccount::get(),
                bounty_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            let bounty_id = NextBountyId::<T>::get();
            NextBountyId::<T>::put(bounty_id.saturating_add(1));

            let bounty = BountyQuestion {
                id: bounty_id,
                creator: who.clone(),
                divination_type,
                result_id,
                question_cid: question_cid_bounded,
                bounty_amount,
                deadline,
                min_answers,
                max_answers,
                status: BountyStatus::Open,
                adopted_answer_id: None,
                second_place_id: None,
                third_place_id: None,
                answer_count: 0,
                reward_distribution: RewardDistribution::default(),
                created_at: current_block,
                closed_at: None,
                settled_at: None,
                specialty,
                certified_only,
                allow_voting,
                total_votes: 0,
            };

            BountyQuestions::<T>::insert(bounty_id, bounty);

            // 更新用户悬赏索引
            UserBounties::<T>::try_mutate(&who, |list| {
                list.try_push(bounty_id)
                    .map_err(|_| Error::<T>::BountyListFull)
            })?;

            // 更新统计
            BountyStatistics::<T>::mutate(|s| {
                s.total_bounties += 1;
                s.active_bounties += 1;
                s.total_bounty_amount = s.total_bounty_amount.saturating_add(bounty_amount);
            });

            Self::deposit_event(Event::BountyCreated {
                bounty_id,
                creator: who,
                divination_type,
                bounty_amount,
                deadline,
            });

            Ok(())
        }

        /// 提交悬赏回答
        ///
        /// # 参数
        /// - `bounty_id`: 悬赏问题 ID
        /// - `answer_cid`: 回答内容 IPFS CID
        #[pallet::call_index(19)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn submit_bounty_answer(
            origin: OriginFor<T>,
            bounty_id: u64,
            answer_cid: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let bounty = BountyQuestions::<T>::get(bounty_id).ok_or(Error::<T>::BountyNotFound)?;

            // 验证状态
            ensure!(bounty.status == BountyStatus::Open, Error::<T>::BountyNotOpen);

            // 验证截止时间
            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(
                current_block <= bounty.deadline,
                Error::<T>::BountyDeadlinePassed
            );

            // 不能回答自己的悬赏
            ensure!(who != bounty.creator, Error::<T>::CannotAnswerOwnBounty);

            // 验证回答数量限制
            ensure!(
                bounty.answer_count < bounty.max_answers as u32,
                Error::<T>::BountyAnswerLimitReached
            );

            // 检查是否已回答
            let answer_ids = BountyAnswerIds::<T>::get(bounty_id);
            for answer_id in answer_ids.iter() {
                if let Some(ans) = BountyAnswers::<T>::get(answer_id) {
                    ensure!(ans.answerer != who, Error::<T>::AlreadyAnswered);
                }
            }

            // 检查认证要求
            let (is_certified, provider_tier) = if bounty.certified_only {
                let provider =
                    Providers::<T>::get(&who).ok_or(Error::<T>::CertifiedProviderOnly)?;
                ensure!(
                    provider.tier as u8 >= ProviderTier::Certified as u8,
                    Error::<T>::CertifiedProviderOnly
                );
                (true, Some(provider.tier))
            } else {
                // 非强制认证时，检查是否为提供者
                if let Some(provider) = Providers::<T>::get(&who) {
                    (provider.tier as u8 >= ProviderTier::Certified as u8, Some(provider.tier))
                } else {
                    (false, None)
                }
            };

            let answer_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(answer_cid).map_err(|_| Error::<T>::CidTooLong)?;

            let answer_id = NextBountyAnswerId::<T>::get();
            NextBountyAnswerId::<T>::put(answer_id.saturating_add(1));

            let answer = BountyAnswer {
                id: answer_id,
                bounty_id,
                answerer: who.clone(),
                answer_cid: answer_cid_bounded,
                status: BountyAnswerStatus::Pending,
                votes: 0,
                reward_amount: Zero::zero(),
                submitted_at: current_block,
                is_certified,
                provider_tier,
            };

            BountyAnswers::<T>::insert(answer_id, answer);

            // 更新悬赏回答索引
            BountyAnswerIds::<T>::try_mutate(bounty_id, |list| {
                list.try_push(answer_id)
                    .map_err(|_| Error::<T>::BountyAnswerLimitReached)
            })?;

            // 更新用户回答索引
            UserBountyAnswers::<T>::try_mutate(&who, |list| {
                list.try_push(answer_id)
                    .map_err(|_| Error::<T>::BountyListFull)
            })?;

            // 更新悬赏回答数
            BountyQuestions::<T>::mutate(bounty_id, |maybe_bounty| {
                if let Some(b) = maybe_bounty {
                    b.answer_count += 1;
                }
            });

            // 更新统计
            BountyStatistics::<T>::mutate(|s| {
                s.total_answers += 1;
            });

            Self::deposit_event(Event::BountyAnswerSubmitted {
                answer_id,
                bounty_id,
                answerer: who,
            });

            Ok(())
        }

        /// 关闭悬赏（停止接受回答）
        ///
        /// 仅悬赏创建者可调用，需要达到最小回答数
        #[pallet::call_index(20)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn close_bounty(origin: OriginFor<T>, bounty_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            BountyQuestions::<T>::try_mutate(bounty_id, |maybe_bounty| {
                let bounty = maybe_bounty.as_mut().ok_or(Error::<T>::BountyNotFound)?;

                ensure!(bounty.creator == who, Error::<T>::NotBountyCreator);
                ensure!(bounty.status == BountyStatus::Open, Error::<T>::BountyAlreadyClosed);
                ensure!(
                    bounty.answer_count >= bounty.min_answers as u32,
                    Error::<T>::NotEnoughAnswers
                );

                bounty.status = BountyStatus::Closed;
                bounty.closed_at = Some(<frame_system::Pallet<T>>::block_number());

                Ok::<_, DispatchError>(())
            })?;

            // 更新统计
            BountyStatistics::<T>::mutate(|s| {
                s.active_bounties = s.active_bounties.saturating_sub(1);
            });

            Self::deposit_event(Event::BountyClosed { bounty_id });

            Ok(())
        }

        /// 投票支持回答
        ///
        /// 任何人都可以投票（如果悬赏允许投票）
        #[pallet::call_index(21)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn vote_bounty_answer(
            origin: OriginFor<T>,
            bounty_id: u64,
            answer_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let bounty = BountyQuestions::<T>::get(bounty_id).ok_or(Error::<T>::BountyNotFound)?;

            // 验证投票功能已开启
            ensure!(bounty.allow_voting, Error::<T>::VotingNotAllowed);

            // 验证状态：Open 或 Closed 时可投票
            ensure!(
                bounty.status == BountyStatus::Open || bounty.status == BountyStatus::Closed,
                Error::<T>::BountyAlreadyAdopted
            );

            // 验证答案存在且属于该悬赏
            let answer = BountyAnswers::<T>::get(answer_id).ok_or(Error::<T>::BountyAnswerNotFound)?;
            ensure!(answer.bounty_id == bounty_id, Error::<T>::BountyAnswerNotFound);

            // 检查是否已投票
            ensure!(
                !BountyVotes::<T>::contains_key(bounty_id, &who),
                Error::<T>::AlreadyVoted
            );

            let current_block = <frame_system::Pallet<T>>::block_number();

            // 记录投票
            let vote = BountyVote {
                voter: who.clone(),
                answer_id,
                voted_at: current_block,
            };
            BountyVotes::<T>::insert(bounty_id, &who, vote);

            // 更新答案票数
            BountyAnswers::<T>::mutate(answer_id, |maybe_answer| {
                if let Some(a) = maybe_answer {
                    a.votes += 1;
                }
            });

            // 更新悬赏总票数
            BountyQuestions::<T>::mutate(bounty_id, |maybe_bounty| {
                if let Some(b) = maybe_bounty {
                    b.total_votes += 1;
                }
            });

            Self::deposit_event(Event::BountyAnswerVoted {
                bounty_id,
                answer_id,
                voter: who,
            });

            Ok(())
        }

        /// 采纳答案（选择前三名）
        ///
        /// 仅悬赏创建者可调用
        #[pallet::call_index(22)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn adopt_bounty_answers(
            origin: OriginFor<T>,
            bounty_id: u64,
            first_place: u64,
            second_place: Option<u64>,
            third_place: Option<u64>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            BountyQuestions::<T>::try_mutate(bounty_id, |maybe_bounty| {
                let bounty = maybe_bounty.as_mut().ok_or(Error::<T>::BountyNotFound)?;

                ensure!(bounty.creator == who, Error::<T>::NotBountyCreator);
                ensure!(
                    bounty.status == BountyStatus::Open || bounty.status == BountyStatus::Closed,
                    Error::<T>::BountyAlreadyAdopted
                );
                ensure!(bounty.answer_count >= 1, Error::<T>::NotEnoughAnswers);

                // 验证第一名答案
                let first_ans = BountyAnswers::<T>::get(first_place)
                    .ok_or(Error::<T>::BountyAnswerNotFound)?;
                ensure!(first_ans.bounty_id == bounty_id, Error::<T>::BountyAnswerNotFound);

                // 验证第二名答案（如果提供）
                if let Some(second_id) = second_place {
                    let second_ans = BountyAnswers::<T>::get(second_id)
                        .ok_or(Error::<T>::BountyAnswerNotFound)?;
                    ensure!(second_ans.bounty_id == bounty_id, Error::<T>::BountyAnswerNotFound);
                }

                // 验证第三名答案（如果提供）
                if let Some(third_id) = third_place {
                    let third_ans = BountyAnswers::<T>::get(third_id)
                        .ok_or(Error::<T>::BountyAnswerNotFound)?;
                    ensure!(third_ans.bounty_id == bounty_id, Error::<T>::BountyAnswerNotFound);
                }

                bounty.status = BountyStatus::Adopted;
                bounty.adopted_answer_id = Some(first_place);
                bounty.second_place_id = second_place;
                bounty.third_place_id = third_place;

                Ok::<_, DispatchError>(())
            })?;

            // 更新答案状态
            BountyAnswers::<T>::mutate(first_place, |maybe_answer| {
                if let Some(a) = maybe_answer {
                    a.status = BountyAnswerStatus::Adopted;
                }
            });

            if let Some(second_id) = second_place {
                BountyAnswers::<T>::mutate(second_id, |maybe_answer| {
                    if let Some(a) = maybe_answer {
                        a.status = BountyAnswerStatus::Selected;
                    }
                });
            }

            if let Some(third_id) = third_place {
                BountyAnswers::<T>::mutate(third_id, |maybe_answer| {
                    if let Some(a) = maybe_answer {
                        a.status = BountyAnswerStatus::Selected;
                    }
                });
            }

            Self::deposit_event(Event::BountyAnswersAdopted {
                bounty_id,
                first_place,
                second_place,
                third_place,
            });

            Ok(())
        }

        /// 结算悬赏奖励（方案B - 多人奖励）
        ///
        /// 采纳后由任何人调用执行奖励分配
        #[pallet::call_index(23)]
        #[pallet::weight(Weight::from_parts(100_000_000, 0))]
        pub fn settle_bounty(origin: OriginFor<T>, bounty_id: u64) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let bounty = BountyQuestions::<T>::get(bounty_id).ok_or(Error::<T>::BountyNotFound)?;

            ensure!(
                bounty.status == BountyStatus::Adopted,
                Error::<T>::BountyNotAdopted
            );

            let first_place_id = bounty.adopted_answer_id.ok_or(Error::<T>::NotEnoughAnswers)?;

            // 计算奖励金额
            let dist = bounty.reward_distribution;
            let total = bounty.bounty_amount;
            let answer_count = bounty.answer_count;

            // 计算各名次奖励
            let first_reward = total.saturating_mul(dist.first_place.into()) / 10000u32.into();
            let second_reward = total.saturating_mul(dist.second_place.into()) / 10000u32.into();
            let third_reward = total.saturating_mul(dist.third_place.into()) / 10000u32.into();
            let platform_fee = total.saturating_mul(dist.platform_fee.into()) / 10000u32.into();
            let participation_pool =
                total.saturating_mul(dist.participation_pool.into()) / 10000u32.into();

            // 发放第一名奖励
            let first_ans =
                BountyAnswers::<T>::get(first_place_id).ok_or(Error::<T>::BountyAnswerNotFound)?;
            T::Currency::transfer(
                &T::PlatformAccount::get(),
                &first_ans.answerer,
                first_reward,
                ExistenceRequirement::KeepAlive,
            )?;
            BountyAnswers::<T>::mutate(first_place_id, |maybe_answer| {
                if let Some(a) = maybe_answer {
                    a.reward_amount = first_reward;
                }
            });
            Self::deposit_event(Event::BountyRewardPaid {
                bounty_id,
                recipient: first_ans.answerer.clone(),
                amount: first_reward,
                rank: 1,
            });

            let mut distributed = first_reward;

            // 发放第二名奖励
            if let Some(second_id) = bounty.second_place_id {
                if let Some(second_ans) = BountyAnswers::<T>::get(second_id) {
                    T::Currency::transfer(
                        &T::PlatformAccount::get(),
                        &second_ans.answerer,
                        second_reward,
                        ExistenceRequirement::KeepAlive,
                    )?;
                    BountyAnswers::<T>::mutate(second_id, |maybe_answer| {
                        if let Some(a) = maybe_answer {
                            a.reward_amount = second_reward;
                        }
                    });
                    Self::deposit_event(Event::BountyRewardPaid {
                        bounty_id,
                        recipient: second_ans.answerer,
                        amount: second_reward,
                        rank: 2,
                    });
                    distributed = distributed.saturating_add(second_reward);
                }
            }

            // 发放第三名奖励
            if let Some(third_id) = bounty.third_place_id {
                if let Some(third_ans) = BountyAnswers::<T>::get(third_id) {
                    T::Currency::transfer(
                        &T::PlatformAccount::get(),
                        &third_ans.answerer,
                        third_reward,
                        ExistenceRequirement::KeepAlive,
                    )?;
                    BountyAnswers::<T>::mutate(third_id, |maybe_answer| {
                        if let Some(a) = maybe_answer {
                            a.reward_amount = third_reward;
                        }
                    });
                    Self::deposit_event(Event::BountyRewardPaid {
                        bounty_id,
                        recipient: third_ans.answerer,
                        amount: third_reward,
                        rank: 3,
                    });
                    distributed = distributed.saturating_add(third_reward);
                }
            }

            // 计算并发放参与奖
            let top_three = [
                bounty.adopted_answer_id,
                bounty.second_place_id,
                bounty.third_place_id,
            ];
            let answer_ids = BountyAnswerIds::<T>::get(bounty_id);
            let other_participants: Vec<_> = answer_ids
                .iter()
                .filter(|id| !top_three.contains(&Some(**id)))
                .collect();

            let other_count = other_participants.len() as u32;
            if other_count > 0 {
                let per_participant = participation_pool / other_count.into();
                for answer_id in other_participants {
                    if let Some(ans) = BountyAnswers::<T>::get(answer_id) {
                        T::Currency::transfer(
                            &T::PlatformAccount::get(),
                            &ans.answerer,
                            per_participant,
                            ExistenceRequirement::KeepAlive,
                        )?;
                        BountyAnswers::<T>::mutate(answer_id, |maybe_answer| {
                            if let Some(a) = maybe_answer {
                                a.status = BountyAnswerStatus::Participated;
                                a.reward_amount = per_participant;
                            }
                        });
                        Self::deposit_event(Event::BountyRewardPaid {
                            bounty_id,
                            recipient: ans.answerer,
                            amount: per_participant,
                            rank: 0,
                        });
                        distributed = distributed.saturating_add(per_participant);
                    }
                }
            }

            // 平台手续费保留在平台账户（无需转账）
            distributed = distributed.saturating_add(platform_fee);

            // 更新悬赏状态
            BountyQuestions::<T>::mutate(bounty_id, |maybe_bounty| {
                if let Some(b) = maybe_bounty {
                    b.status = BountyStatus::Settled;
                    b.settled_at = Some(<frame_system::Pallet<T>>::block_number());
                }
            });

            // 更新统计
            BountyStatistics::<T>::mutate(|s| {
                s.settled_bounties += 1;
                s.total_rewards_paid = s.total_rewards_paid.saturating_add(distributed);
                // 更新平均回答数
                if s.settled_bounties > 0 {
                    s.avg_answers_per_bounty =
                        ((s.total_answers as u64 * 100) / s.settled_bounties) as u16;
                }
            });

            Self::deposit_event(Event::BountySettled {
                bounty_id,
                total_distributed: distributed,
                platform_fee,
                participant_count: answer_count,
            });

            Ok(())
        }

        /// 取消悬赏（仅限无回答时）
        #[pallet::call_index(24)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn cancel_bounty(origin: OriginFor<T>, bounty_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let bounty = BountyQuestions::<T>::get(bounty_id).ok_or(Error::<T>::BountyNotFound)?;

            ensure!(bounty.creator == who, Error::<T>::NotBountyCreator);
            ensure!(bounty.status == BountyStatus::Open, Error::<T>::BountyAlreadyClosed);
            ensure!(bounty.answer_count == 0, Error::<T>::BountyCannotCancel);

            // 退款
            T::Currency::transfer(
                &T::PlatformAccount::get(),
                &who,
                bounty.bounty_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            // 更新状态
            BountyQuestions::<T>::mutate(bounty_id, |maybe_bounty| {
                if let Some(b) = maybe_bounty {
                    b.status = BountyStatus::Cancelled;
                }
            });

            // 更新统计
            BountyStatistics::<T>::mutate(|s| {
                s.active_bounties = s.active_bounties.saturating_sub(1);
            });

            Self::deposit_event(Event::BountyCancelled {
                bounty_id,
                refund_amount: bounty.bounty_amount,
            });

            Ok(())
        }

        /// 处理过期悬赏（任何人可调用）
        ///
        /// 超过截止时间且无人回答的悬赏可退款
        #[pallet::call_index(25)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn expire_bounty(origin: OriginFor<T>, bounty_id: u64) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let bounty = BountyQuestions::<T>::get(bounty_id).ok_or(Error::<T>::BountyNotFound)?;

            ensure!(bounty.status == BountyStatus::Open, Error::<T>::BountyAlreadyClosed);

            // 验证已过期
            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(current_block > bounty.deadline, Error::<T>::BountyNotExpired);

            // 如果有回答，不能简单过期处理，需要创建者采纳
            if bounty.answer_count > 0 {
                // 仅关闭，等待创建者采纳
                BountyQuestions::<T>::mutate(bounty_id, |maybe_bounty| {
                    if let Some(b) = maybe_bounty {
                        b.status = BountyStatus::Closed;
                        b.closed_at = Some(current_block);
                    }
                });

                BountyStatistics::<T>::mutate(|s| {
                    s.active_bounties = s.active_bounties.saturating_sub(1);
                });

                Self::deposit_event(Event::BountyClosed { bounty_id });
            } else {
                // 无回答，退款并标记过期
                T::Currency::transfer(
                    &T::PlatformAccount::get(),
                    &bounty.creator,
                    bounty.bounty_amount,
                    ExistenceRequirement::KeepAlive,
                )?;

                BountyQuestions::<T>::mutate(bounty_id, |maybe_bounty| {
                    if let Some(b) = maybe_bounty {
                        b.status = BountyStatus::Expired;
                    }
                });

                BountyStatistics::<T>::mutate(|s| {
                    s.active_bounties = s.active_bounties.saturating_sub(1);
                });

                Self::deposit_event(Event::BountyExpired {
                    bounty_id,
                    refund_amount: bounty.bounty_amount,
                });
            }

            Ok(())
        }

        // ==================== 个人主页管理函数 ====================

        /// 更新提供者详细资料
        ///
        /// # 参数
        /// - `introduction_cid`: 详细自我介绍 IPFS CID
        /// - `experience_years`: 从业年限
        /// - `background`: 师承/学习背景
        /// - `motto`: 服务理念/座右铭
        /// - `expertise_description`: 擅长问题类型描述
        /// - `working_hours`: 工作时间说明
        /// - `avg_response_time`: 平均响应时间（分钟）
        /// - `accepts_appointment`: 是否接受预约
        /// - `banner_cid`: 主页背景图 CID
        #[pallet::call_index(26)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn update_profile(
            origin: OriginFor<T>,
            introduction_cid: Option<Vec<u8>>,
            experience_years: Option<u8>,
            background: Option<Vec<u8>>,
            motto: Option<Vec<u8>>,
            expertise_description: Option<Vec<u8>>,
            working_hours: Option<Vec<u8>>,
            avg_response_time: Option<u32>,
            accepts_appointment: Option<bool>,
            banner_cid: Option<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证是注册的提供者
            ensure!(
                Providers::<T>::contains_key(&who),
                Error::<T>::ProviderNotFound
            );

            let current_block = <frame_system::Pallet<T>>::block_number();

            ProviderProfiles::<T>::try_mutate(&who, |maybe_profile| {
                let profile = match maybe_profile {
                    Some(p) => p,
                    None => {
                        *maybe_profile = Some(ProviderProfile {
                            introduction_cid: None,
                            experience_years: 0,
                            background: None,
                            motto: None,
                            expertise_description: None,
                            working_hours: None,
                            avg_response_time: None,
                            accepts_appointment: false,
                            banner_cid: None,
                            updated_at: current_block,
                        });
                        maybe_profile.as_mut().unwrap()
                    }
                };

                if let Some(cid) = introduction_cid {
                    profile.introduction_cid = Some(
                        BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?
                    );
                }
                if let Some(years) = experience_years {
                    profile.experience_years = years;
                }
                if let Some(bg) = background {
                    profile.background = Some(
                        BoundedVec::try_from(bg).map_err(|_| Error::<T>::DescriptionTooLong)?
                    );
                }
                if let Some(m) = motto {
                    profile.motto = Some(
                        BoundedVec::try_from(m).map_err(|_| Error::<T>::DescriptionTooLong)?
                    );
                }
                if let Some(exp) = expertise_description {
                    profile.expertise_description = Some(
                        BoundedVec::try_from(exp).map_err(|_| Error::<T>::DescriptionTooLong)?
                    );
                }
                if let Some(wh) = working_hours {
                    profile.working_hours = Some(
                        BoundedVec::try_from(wh).map_err(|_| Error::<T>::DescriptionTooLong)?
                    );
                }
                if let Some(time) = avg_response_time {
                    profile.avg_response_time = Some(time);
                }
                if let Some(accepts) = accepts_appointment {
                    profile.accepts_appointment = accepts;
                }
                if let Some(cid) = banner_cid {
                    profile.banner_cid = Some(
                        BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?
                    );
                }

                profile.updated_at = current_block;

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::ProfileUpdated { provider: who });

            Ok(())
        }

        /// 添加资质证书
        #[pallet::call_index(27)]
        #[pallet::weight(Weight::from_parts(35_000_000, 0))]
        pub fn add_certificate(
            origin: OriginFor<T>,
            name: Vec<u8>,
            cert_type: CertificateType,
            issuer: Option<Vec<u8>>,
            image_cid: Vec<u8>,
            issued_at: Option<BlockNumberFor<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Providers::<T>::contains_key(&who),
                Error::<T>::ProviderNotFound
            );

            let cert_id = NextCertificateId::<T>::get(&who);
            // 限制每个提供者最多 20 个证书
            ensure!(cert_id < 20, Error::<T>::TooManyCertificates);

            let name_bounded = BoundedVec::try_from(name).map_err(|_| Error::<T>::NameTooLong)?;
            let image_cid_bounded = BoundedVec::try_from(image_cid).map_err(|_| Error::<T>::CidTooLong)?;
            let issuer_bounded = issuer
                .map(|i| BoundedVec::try_from(i).map_err(|_| Error::<T>::NameTooLong))
                .transpose()?;

            let certificate = Certificate {
                id: cert_id,
                name: name_bounded,
                cert_type,
                issuer: issuer_bounded,
                image_cid: image_cid_bounded,
                issued_at,
                is_verified: false,
                uploaded_at: <frame_system::Pallet<T>>::block_number(),
            };

            Certificates::<T>::insert(&who, cert_id, certificate);
            NextCertificateId::<T>::insert(&who, cert_id.saturating_add(1));

            Self::deposit_event(Event::CertificateAdded {
                provider: who,
                certificate_id: cert_id,
            });

            Ok(())
        }

        /// 删除资质证书
        #[pallet::call_index(28)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn remove_certificate(
            origin: OriginFor<T>,
            certificate_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Certificates::<T>::contains_key(&who, certificate_id),
                Error::<T>::CertificateNotFound
            );

            Certificates::<T>::remove(&who, certificate_id);

            Self::deposit_event(Event::CertificateRemoved {
                provider: who,
                certificate_id,
            });

            Ok(())
        }

        /// 验证资质证书（治理权限）
        #[pallet::call_index(29)]
        #[pallet::weight(Weight::from_parts(25_000_000, 0))]
        pub fn verify_certificate(
            origin: OriginFor<T>,
            provider: T::AccountId,
            certificate_id: u32,
            is_verified: bool,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            Certificates::<T>::try_mutate(&provider, certificate_id, |maybe_cert| {
                let cert = maybe_cert.as_mut().ok_or(Error::<T>::CertificateNotFound)?;
                cert.is_verified = is_verified;
                Ok::<_, DispatchError>(())
            })?;

            // 更新信用档案中的认证数
            if is_verified {
                CreditProfiles::<T>::mutate(&provider, |maybe_profile| {
                    if let Some(profile) = maybe_profile {
                        profile.certification_count = profile.certification_count.saturating_add(1);
                    }
                });
            }

            Self::deposit_event(Event::CertificateVerified {
                provider,
                certificate_id,
                is_verified,
            });

            Ok(())
        }

        /// 发布作品/案例
        #[pallet::call_index(30)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn publish_portfolio(
            origin: OriginFor<T>,
            title: Vec<u8>,
            divination_type: DivinationType,
            case_type: PortfolioCaseType,
            content_cid: Vec<u8>,
            cover_cid: Option<Vec<u8>>,
            is_featured: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Providers::<T>::contains_key(&who),
                Error::<T>::ProviderNotFound
            );

            let portfolio_id = NextPortfolioId::<T>::get(&who);
            // 限制每个提供者最多 100 个作品
            ensure!(portfolio_id < 100, Error::<T>::TooManyPortfolios);

            let title_bounded = BoundedVec::try_from(title).map_err(|_| Error::<T>::NameTooLong)?;
            let content_cid_bounded = BoundedVec::try_from(content_cid).map_err(|_| Error::<T>::CidTooLong)?;
            let cover_cid_bounded = cover_cid
                .map(|c| BoundedVec::try_from(c).map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            let portfolio = PortfolioItem {
                id: portfolio_id,
                title: title_bounded,
                divination_type,
                case_type,
                content_cid: content_cid_bounded,
                cover_cid: cover_cid_bounded,
                is_featured,
                view_count: 0,
                like_count: 0,
                published_at: <frame_system::Pallet<T>>::block_number(),
            };

            Portfolios::<T>::insert(&who, portfolio_id, portfolio);
            NextPortfolioId::<T>::insert(&who, portfolio_id.saturating_add(1));

            Self::deposit_event(Event::PortfolioPublished {
                provider: who,
                portfolio_id,
                divination_type,
            });

            Ok(())
        }

        /// 更新作品
        #[pallet::call_index(31)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn update_portfolio(
            origin: OriginFor<T>,
            portfolio_id: u32,
            title: Option<Vec<u8>>,
            content_cid: Option<Vec<u8>>,
            cover_cid: Option<Vec<u8>>,
            is_featured: Option<bool>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Portfolios::<T>::try_mutate(&who, portfolio_id, |maybe_portfolio| {
                let portfolio = maybe_portfolio.as_mut().ok_or(Error::<T>::PortfolioNotFound)?;

                if let Some(t) = title {
                    portfolio.title = BoundedVec::try_from(t).map_err(|_| Error::<T>::NameTooLong)?;
                }
                if let Some(cid) = content_cid {
                    portfolio.content_cid = BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?;
                }
                if let Some(cid) = cover_cid {
                    portfolio.cover_cid = Some(
                        BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?
                    );
                }
                if let Some(f) = is_featured {
                    portfolio.is_featured = f;
                }

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::PortfolioUpdated {
                provider: who,
                portfolio_id,
            });

            Ok(())
        }

        /// 删除作品
        #[pallet::call_index(32)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn remove_portfolio(
            origin: OriginFor<T>,
            portfolio_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Portfolios::<T>::contains_key(&who, portfolio_id),
                Error::<T>::PortfolioNotFound
            );

            Portfolios::<T>::remove(&who, portfolio_id);

            Self::deposit_event(Event::PortfolioRemoved {
                provider: who,
                portfolio_id,
            });

            Ok(())
        }

        /// 点赞作品
        #[pallet::call_index(33)]
        #[pallet::weight(Weight::from_parts(25_000_000, 0))]
        pub fn like_portfolio(
            origin: OriginFor<T>,
            provider: T::AccountId,
            portfolio_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证作品存在
            ensure!(
                Portfolios::<T>::contains_key(&provider, portfolio_id),
                Error::<T>::PortfolioNotFound
            );

            // 检查是否已点赞
            let key = (provider.clone(), portfolio_id);
            ensure!(
                !PortfolioLikes::<T>::get(&key, &who),
                Error::<T>::AlreadyLiked
            );

            // 记录点赞
            PortfolioLikes::<T>::insert(&key, &who, true);

            // 更新点赞数
            Portfolios::<T>::mutate(&provider, portfolio_id, |maybe_portfolio| {
                if let Some(p) = maybe_portfolio {
                    p.like_count = p.like_count.saturating_add(1);
                }
            });

            Self::deposit_event(Event::PortfolioLiked {
                provider,
                portfolio_id,
                liker: who,
            });

            Ok(())
        }

        /// 设置技能标签
        #[pallet::call_index(34)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn set_skill_tags(
            origin: OriginFor<T>,
            tags: Vec<(Vec<u8>, SkillTagType, u8)>, // (label, type, proficiency)
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Providers::<T>::contains_key(&who),
                Error::<T>::ProviderNotFound
            );

            let mut skill_tags: BoundedVec<SkillTagOf, ConstU32<20>> = BoundedVec::new();

            for (label, tag_type, proficiency) in tags {
                ensure!(proficiency >= 1 && proficiency <= 5, Error::<T>::InvalidRating);

                let label_bounded = BoundedVec::try_from(label)
                    .map_err(|_| Error::<T>::NameTooLong)?;

                skill_tags.try_push(SkillTag {
                    label: label_bounded,
                    tag_type,
                    proficiency,
                }).map_err(|_| Error::<T>::TooManyTags)?;
            }

            SkillTags::<T>::insert(&who, skill_tags);

            Self::deposit_event(Event::SkillTagsUpdated { provider: who });

            Ok(())
        }

        // ==================== 信用体系管理函数 ====================

        /// 初始化提供者信用档案
        ///
        /// 在提供者注册时自动调用，也可手动为老用户创建
        #[pallet::call_index(35)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn init_credit_profile(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Providers::<T>::contains_key(&who),
                Error::<T>::ProviderNotFound
            );

            // 检查是否已有信用档案
            ensure!(
                !CreditProfiles::<T>::contains_key(&who),
                Error::<T>::ProviderAlreadyExists
            );

            let current_block = <frame_system::Pallet<T>>::block_number();
            let provider = Providers::<T>::get(&who).ok_or(Error::<T>::ProviderNotFound)?;

            // 创建初始信用档案，基础分 650
            let initial_score: u16 = 650;
            let profile = CreditProfile {
                score: initial_score,
                level: CreditLevel::from_score(initial_score),
                highest_score: initial_score,
                lowest_score: initial_score,
                service_quality_score: 0,
                avg_overall_rating: 0,
                avg_accuracy_rating: 0,
                avg_attitude_rating: 0,
                avg_response_rating: 0,
                five_star_count: 0,
                one_star_count: 0,
                behavior_score: 250, // 满分
                violation_count: 0,
                warning_count: 0,
                complaint_count: 0,
                complaint_upheld_count: 0,
                active_violations: 0,
                fulfillment_score: 0,
                completion_rate: 10000, // 100%
                on_time_rate: 10000,
                cancellation_rate: 0,
                timeout_count: 0,
                active_cancel_count: 0,
                avg_response_blocks: 0,
                bonus_score: 0,
                bounty_adoption_count: 0,
                certification_count: 0,
                consecutive_positive_days: 0,
                is_verified: false,
                has_deposit: !provider.deposit.is_zero(),
                total_deductions: 0,
                last_deduction_reason: None,
                last_deduction_at: None,
                total_orders: provider.total_orders,
                completed_orders: provider.completed_orders,
                total_reviews: provider.total_ratings,
                created_at: current_block,
                updated_at: current_block,
                last_evaluated_at: current_block,
            };

            CreditProfiles::<T>::insert(&who, profile);

            // 更新全局统计
            CreditStatistics::<T>::mutate(|stats| {
                stats.total_providers = stats.total_providers.saturating_add(1);
                stats.fair_count = stats.fair_count.saturating_add(1);
            });

            Self::deposit_event(Event::CreditProfileCreated { provider: who });

            Ok(())
        }

        /// 记录违规（治理权限）
        #[pallet::call_index(36)]
        #[pallet::weight(Weight::from_parts(60_000_000, 0))]
        pub fn record_violation(
            origin: OriginFor<T>,
            provider: T::AccountId,
            violation_type: ViolationType,
            reason: Vec<u8>,
            related_order_id: Option<u64>,
            penalty: PenaltyType,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            ensure!(
                Providers::<T>::contains_key(&provider),
                Error::<T>::ProviderNotFound
            );

            // 检查是否在黑名单中
            ensure!(
                !CreditBlacklist::<T>::contains_key(&provider),
                Error::<T>::InBlacklist
            );

            let reason_bounded = BoundedVec::try_from(reason)
                .map_err(|_| Error::<T>::DescriptionTooLong)?;

            let violation_id = NextViolationId::<T>::get();
            NextViolationId::<T>::put(violation_id.saturating_add(1));

            let current_block = <frame_system::Pallet<T>>::block_number();
            let duration = violation_type.record_duration();
            let expires_at = if duration > 0 {
                Some(current_block + duration.into())
            } else {
                None
            };

            // 计算扣分
            let base_deduction: u16 = match &penalty {
                PenaltyType::DeductionOnly => 20,
                PenaltyType::Warning => 30,
                PenaltyType::OrderRestriction => 50,
                PenaltyType::ServiceSuspension => 100,
                PenaltyType::PermanentBan => 500,
            };
            let deduction_points = (base_deduction as u32 * violation_type.penalty_multiplier() as u32 / 100) as u16;

            let record = ViolationRecord {
                id: violation_id,
                provider: provider.clone(),
                violation_type,
                reason: reason_bounded,
                related_order_id,
                deduction_points,
                penalty,
                penalty_duration: duration,
                is_appealed: false,
                appeal_result: None,
                recorded_at: current_block,
                expires_at,
                is_active: true,
            };

            ViolationRecords::<T>::insert(violation_id, record);

            // 更新提供者违规索引
            ProviderViolations::<T>::try_mutate(&provider, |list| {
                list.try_push(violation_id)
                    .map_err(|_| Error::<T>::TooManyViolations)
            })?;

            // 更新信用档案
            CreditProfiles::<T>::mutate(&provider, |maybe_profile| {
                if let Some(profile) = maybe_profile {
                    profile.violation_count = profile.violation_count.saturating_add(1);
                    profile.active_violations = profile.active_violations.saturating_add(1);
                    profile.total_deductions = profile.total_deductions.saturating_add(deduction_points);
                    profile.last_deduction_reason = Some(DeductionReason::Violation);
                    profile.last_deduction_at = Some(current_block);

                    // 重新计算分数
                    let new_score = profile.score.saturating_sub(deduction_points);
                    let old_level = profile.level;
                    let new_level = CreditLevel::from_score(new_score);

                    profile.score = new_score;
                    profile.level = new_level;
                    if new_score < profile.lowest_score {
                        profile.lowest_score = new_score;
                    }
                    profile.updated_at = current_block;

                    // 如果等级变更，发送事件
                    if old_level != new_level {
                        Self::deposit_event(Event::CreditLevelChanged {
                            provider: provider.clone(),
                            old_level,
                            new_level,
                        });
                    }
                }
            });

            // 处理永久封禁
            if penalty == PenaltyType::PermanentBan {
                CreditBlacklist::<T>::insert(&provider, current_block);

                // 更新提供者状态
                Providers::<T>::mutate(&provider, |maybe_p| {
                    if let Some(p) = maybe_p {
                        p.status = ProviderStatus::Banned;
                    }
                });

                CreditStatistics::<T>::mutate(|stats| {
                    stats.blacklisted_count = stats.blacklisted_count.saturating_add(1);
                });

                Self::deposit_event(Event::AddedToBlacklist { provider: provider.clone() });
            }

            Self::deposit_event(Event::ViolationRecorded {
                provider,
                violation_id,
                violation_type,
                penalty,
                deduction_points,
            });

            Ok(())
        }

        /// 申诉违规（提供者调用）
        #[pallet::call_index(37)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn appeal_violation(
            origin: OriginFor<T>,
            violation_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ViolationRecords::<T>::try_mutate(violation_id, |maybe_record| {
                let record = maybe_record.as_mut()
                    .ok_or(Error::<T>::ViolationNotFound)?;

                ensure!(record.provider == who, Error::<T>::NotViolationOwner);
                ensure!(!record.is_appealed, Error::<T>::AlreadyAppealed);
                ensure!(record.is_active, Error::<T>::ViolationExpired);

                record.is_appealed = true;

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::ViolationAppealed {
                provider: who,
                violation_id,
            });

            Ok(())
        }

        /// 处理申诉（治理权限）
        #[pallet::call_index(38)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn resolve_appeal(
            origin: OriginFor<T>,
            violation_id: u64,
            result: AppealResult,
            restore_points: Option<u16>,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            let record = ViolationRecords::<T>::get(violation_id)
                .ok_or(Error::<T>::ViolationNotFound)?;

            ensure!(record.is_appealed, Error::<T>::NotAppealed);

            let provider = record.provider.clone();
            let original_deduction = record.deduction_points;

            // 更新违规记录
            ViolationRecords::<T>::mutate(violation_id, |maybe_record| {
                if let Some(r) = maybe_record {
                    r.appeal_result = Some(result);
                    if result == AppealResult::Upheld {
                        r.is_active = false;
                    }
                }
            });

            // 根据申诉结果恢复信用分
            let points_to_restore = match result {
                AppealResult::Upheld => original_deduction,
                AppealResult::PartiallyUpheld => restore_points.unwrap_or(original_deduction / 2),
                AppealResult::Rejected => 0,
            };

            if points_to_restore > 0 {
                CreditProfiles::<T>::mutate(&provider, |maybe_profile| {
                    if let Some(profile) = maybe_profile {
                        profile.total_deductions = profile.total_deductions.saturating_sub(points_to_restore);

                        if result == AppealResult::Upheld {
                            profile.violation_count = profile.violation_count.saturating_sub(1);
                            profile.active_violations = profile.active_violations.saturating_sub(1);
                        }

                        let new_score = profile.score.saturating_add(points_to_restore).min(1000);
                        let old_level = profile.level;
                        let new_level = CreditLevel::from_score(new_score);

                        profile.score = new_score;
                        profile.level = new_level;
                        if new_score > profile.highest_score {
                            profile.highest_score = new_score;
                        }
                        profile.updated_at = <frame_system::Pallet<T>>::block_number();

                        if old_level != new_level {
                            Self::deposit_event(Event::CreditLevelChanged {
                                provider: provider.clone(),
                                old_level,
                                new_level,
                            });
                        }
                    }
                });
            }

            Self::deposit_event(Event::AppealResolved {
                provider,
                violation_id,
                result,
                restored_points: points_to_restore,
            });

            Ok(())
        }

        /// 申请信用修复任务
        #[pallet::call_index(39)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn request_credit_repair(
            origin: OriginFor<T>,
            task_type: RepairTaskType,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let profile = CreditProfiles::<T>::get(&who)
                .ok_or(Error::<T>::CreditProfileNotFound)?;

            // 只有信用分低于 750 的用户才能申请修复
            ensure!(profile.score < 750, Error::<T>::CreditTooHighForRepair);

            // 检查是否已有相同类型的进行中任务
            let tasks = RepairTasks::<T>::get(&who);
            ensure!(
                !tasks.iter().any(|t| t.task_type == task_type && !t.is_completed),
                Error::<T>::DuplicateRepairTask
            );

            // 检查活跃任务数量上限
            ensure!(
                tasks.iter().filter(|t| !t.is_completed).count() < 3,
                Error::<T>::TooManyActiveTasks
            );

            let current_block = <frame_system::Pallet<T>>::block_number();
            let target_value = task_type.default_target();
            let duration = task_type.default_duration();

            let task_id = tasks.len() as u32;

            let task = CreditRepairTask {
                id: task_id,
                task_type,
                reward_points: task_type.default_reward(),
                target_value,
                current_progress: 0,
                is_completed: false,
                started_at: current_block,
                deadline: current_block + duration.into(),
                completed_at: None,
            };

            RepairTasks::<T>::try_mutate(&who, |tasks| {
                tasks.try_push(task)
                    .map_err(|_| Error::<T>::TooManyTasks)
            })?;

            Self::deposit_event(Event::CreditRepairRequested {
                provider: who,
                task_type,
                target_value,
            });

            Ok(())
        }

        // ==================== 举报系统可调用函数 ====================

        /// 提交举报
        ///
        /// 任何用户都可以举报大师的违规行为。举报者需要缴纳押金以防止恶意举报。
        ///
        /// # 参数
        /// - `provider`: 被举报的大师账户
        /// - `report_type`: 举报类型
        /// - `evidence_cid`: 证据 IPFS CID
        /// - `description`: 举报描述
        /// - `related_order_id`: 关联订单 ID（可选）
        /// - `related_bounty_id`: 关联悬赏 ID（可选）
        /// - `related_answer_id`: 关联回答 ID（可选）
        /// - `is_anonymous`: 是否匿名举报
        ///
        /// # 逻辑
        /// 1. 验证被举报者是已注册的大师
        /// 2. 验证举报冷却期
        /// 3. 计算并收取举报押金
        /// 4. 创建举报记录
        /// 5. 加入待处理队列
        #[pallet::call_index(40)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn submit_report(
            origin: OriginFor<T>,
            provider: T::AccountId,
            report_type: ReportType,
            evidence_cid: Vec<u8>,
            description: Vec<u8>,
            related_order_id: Option<u64>,
            related_bounty_id: Option<u64>,
            related_answer_id: Option<u64>,
            is_anonymous: bool,
        ) -> DispatchResult {
            let reporter = ensure_signed(origin)?;

            // 1. 基础验证：不能举报自己
            ensure!(reporter != provider, Error::<T>::CannotReportSelf);

            // 2. 验证大师存在且未被封禁
            ensure!(
                Providers::<T>::contains_key(&provider),
                Error::<T>::ProviderNotFound
            );
            ensure!(
                !CreditBlacklist::<T>::contains_key(&provider),
                Error::<T>::ProviderAlreadyBanned
            );

            // 3. 验证冷却期
            Self::check_report_cooldown(&reporter, &provider)?;

            // 4. 计算并收取举报押金
            let required_deposit = Self::calculate_report_deposit(report_type);
            T::Currency::transfer(
                &reporter,
                &Self::platform_account(),
                required_deposit,
                ExistenceRequirement::KeepAlive,
            )?;

            // 5. 构建举报记录
            let current_block = <frame_system::Pallet<T>>::block_number();
            let report_id = NextReportId::<T>::get();
            NextReportId::<T>::put(report_id.saturating_add(1));

            let evidence_bounded: BoundedVec<u8, T::MaxCidLength> = evidence_cid
                .try_into()
                .map_err(|_| Error::<T>::CidTooLong)?;
            let description_bounded: BoundedVec<u8, T::MaxDescriptionLength> = description
                .try_into()
                .map_err(|_| Error::<T>::DescriptionTooLong)?;

            let report = Report {
                id: report_id,
                reporter: reporter.clone(),
                provider: provider.clone(),
                report_type,
                evidence_cid: evidence_bounded,
                description: description_bounded,
                related_order_id,
                related_bounty_id,
                related_answer_id,
                reporter_deposit: required_deposit,
                status: ReportStatus::Pending,
                created_at: current_block,
                resolved_at: None,
                resolution_cid: None,
                resolved_by: None,
                provider_penalty: Zero::zero(),
                reporter_reward: Zero::zero(),
                is_anonymous,
            };

            // 6. 存储举报
            Reports::<T>::insert(report_id, report);

            // 7. 更新索引
            ProviderReports::<T>::try_mutate(&provider, |list| {
                list.try_push(report_id)
                    .map_err(|_| Error::<T>::TooManyReports)
            })?;
            UserReports::<T>::try_mutate(&reporter, |list| {
                list.try_push(report_id)
                    .map_err(|_| Error::<T>::TooManyReports)
            })?;
            PendingReports::<T>::try_mutate(|list| {
                list.try_push(report_id)
                    .map_err(|_| Error::<T>::TooManyPendingReports)
            })?;

            // 8. 更新冷却期
            ReportCooldown::<T>::insert(&reporter, &provider, current_block);

            // 9. 更新统计
            ReportStatistics::<T>::mutate(|stats| {
                stats.total_reports += 1;
                stats.pending_reports += 1;
            });

            // 10. 更新大师举报档案
            ProviderReportProfiles::<T>::mutate(&provider, |profile| {
                profile.total_reported += 1;
                profile.last_reported_at = current_block;
            });

            // 11. 发送事件
            Self::deposit_event(Event::ReportSubmitted {
                report_id,
                reporter: if is_anonymous { None } else { Some(reporter) },
                provider,
                report_type,
                deposit: required_deposit,
            });

            Ok(())
        }

        /// 撤回举报
        ///
        /// 仅在窗口期内且状态为 Pending 时可撤回。
        /// 撤回后退还 80% 押金（20% 作为滥用费用）。
        ///
        /// # 参数
        /// - `report_id`: 举报 ID
        #[pallet::call_index(41)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn withdraw_report(origin: OriginFor<T>, report_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Reports::<T>::try_mutate(report_id, |maybe_report| {
                let report = maybe_report.as_mut().ok_or(Error::<T>::ReportNotFound)?;

                // 验证是举报者
                ensure!(report.reporter == who, Error::<T>::NotReporter);

                // 验证状态为待处理
                ensure!(
                    report.status == ReportStatus::Pending,
                    Error::<T>::ReportNotPending
                );

                // 验证在撤回窗口期内
                let current_block = <frame_system::Pallet<T>>::block_number();
                ensure!(
                    current_block
                        <= report.created_at.saturating_add(T::ReportWithdrawWindow::get()),
                    Error::<T>::WithdrawWindowExpired
                );

                // 退还 80% 押金
                let refund =
                    report.reporter_deposit.saturating_mul(80u32.into()) / 100u32.into();
                T::Currency::transfer(
                    &Self::platform_account(),
                    &who,
                    refund,
                    ExistenceRequirement::KeepAlive,
                )?;

                // 更新状态
                report.status = ReportStatus::Withdrawn;
                report.resolved_at = Some(current_block);

                Ok::<_, DispatchError>(())
            })?;

            // 从待处理队列移除
            Self::remove_from_pending(report_id);

            // 更新统计
            ReportStatistics::<T>::mutate(|stats| {
                stats.pending_reports = stats.pending_reports.saturating_sub(1);
            });

            Self::deposit_event(Event::ReportWithdrawn { report_id });

            Ok(())
        }

        /// 审核举报（委员会专用）
        ///
        /// 仅委员会/治理权限可调用。
        ///
        /// # 参数
        /// - `report_id`: 举报 ID
        /// - `result`: 审核结果（Upheld/Rejected/Malicious）
        /// - `resolution_cid`: 处理说明 IPFS CID（可选）
        /// - `custom_penalty_rate`: 自定义惩罚比例（可选，覆盖默认值）
        #[pallet::call_index(42)]
        #[pallet::weight(Weight::from_parts(80_000_000, 0))]
        pub fn resolve_report(
            origin: OriginFor<T>,
            report_id: u64,
            result: ReportStatus,
            resolution_cid: Option<Vec<u8>>,
            custom_penalty_rate: Option<u16>,
        ) -> DispatchResult {
            // 验证委员会权限
            let resolver = T::ReportReviewOrigin::ensure_origin(origin)?;

            // 验证结果有效性
            ensure!(
                matches!(
                    result,
                    ReportStatus::Upheld | ReportStatus::Rejected | ReportStatus::Malicious
                ),
                Error::<T>::InvalidReportResult
            );

            // 获取举报记录
            let report = Reports::<T>::get(report_id).ok_or(Error::<T>::ReportNotFound)?;
            ensure!(
                report.status == ReportStatus::Pending
                    || report.status == ReportStatus::UnderReview,
                Error::<T>::ReportAlreadyResolved
            );

            // 处理不同结果
            match result {
                ReportStatus::Upheld => {
                    Self::handle_upheld_report(report_id, &report, custom_penalty_rate)?;
                }
                ReportStatus::Rejected => {
                    Self::handle_rejected_report(report_id, &report)?;
                }
                ReportStatus::Malicious => {
                    Self::handle_malicious_report(report_id, &report)?;
                }
                _ => return Err(Error::<T>::InvalidReportResult.into()),
            }

            // 更新举报记录
            let current_block = <frame_system::Pallet<T>>::block_number();
            let resolution_bounded: Option<BoundedVec<u8, T::MaxCidLength>> = resolution_cid
                .map(|cid| cid.try_into().map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            Reports::<T>::mutate(report_id, |maybe_report| {
                if let Some(r) = maybe_report {
                    r.status = result;
                    r.resolved_at = Some(current_block);
                    r.resolution_cid = resolution_bounded;
                    r.resolved_by = Some(resolver.clone());
                }
            });

            // 从待处理队列移除
            Self::remove_from_pending(report_id);

            // 更新统计
            Self::update_report_stats_on_resolve(result);

            Self::deposit_event(Event::ReportResolved {
                report_id,
                result,
                resolver,
            });

            Ok(())
        }

        /// 处理超时举报
        ///
        /// 任何人可调用，超时后举报者可取回全额押金。
        ///
        /// # 参数
        /// - `report_id`: 举报 ID
        #[pallet::call_index(43)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn expire_report(origin: OriginFor<T>, report_id: u64) -> DispatchResult {
            ensure_signed(origin)?;

            let report = Reports::<T>::get(report_id).ok_or(Error::<T>::ReportNotFound)?;
            ensure!(
                report.status == ReportStatus::Pending,
                Error::<T>::ReportNotPending
            );

            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(
                current_block > report.created_at.saturating_add(T::ReportTimeout::get()),
                Error::<T>::ReportNotExpired
            );

            // 全额退还举报押金
            T::Currency::transfer(
                &Self::platform_account(),
                &report.reporter,
                report.reporter_deposit,
                ExistenceRequirement::KeepAlive,
            )?;

            // 更新状态
            Reports::<T>::mutate(report_id, |maybe_report| {
                if let Some(r) = maybe_report {
                    r.status = ReportStatus::Expired;
                    r.resolved_at = Some(current_block);
                }
            });

            // 从待处理队列移除
            Self::remove_from_pending(report_id);

            ReportStatistics::<T>::mutate(|stats| {
                stats.pending_reports = stats.pending_reports.saturating_sub(1);
            });

            Self::deposit_event(Event::ReportExpired { report_id });

            Ok(())
        }
    }
}
