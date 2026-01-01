# 举报机制开发步骤

基于 `REPORT_SYSTEM_DESIGN.md` 设计文档，本文档详细描述具体开发步骤。

---

## 开发概览

| 阶段 | 内容 | 文件 |
|------|------|------|
| 1 | 数据类型定义 | `src/types.rs` |
| 2 | 存储项定义 | `src/lib.rs` |
| 3 | 配置参数 | `src/lib.rs` |
| 4 | 错误与事件 | `src/lib.rs` |
| 5 | 核心函数实现 | `src/helpers/report.rs` |
| 6 | 可调用函数 | `src/lib.rs` |
| 7 | 单元测试 | `src/tests/report_tests.rs` |
| 8 | Runtime 集成 | `runtime/src/lib.rs` |

---

## 阶段 1：数据类型定义

**文件**: `src/types.rs`

### 步骤 1.1：添加举报类型枚举

```rust
// 在 types.rs 末尾添加

// ============================================================================
// 举报系统类型定义
// ============================================================================

/// 举报类型
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
)]
pub enum ReportType {
    /// 黄色/色情内容
    Pornography = 0,
    /// 赌博相关
    Gambling = 1,
    /// 毒品/违禁品
    Drugs = 2,
    /// 诈骗行为
    Fraud = 3,
    /// 虚假宣传/夸大资质
    FalseAdvertising = 4,
    /// 辱骂/人身攻击
    Abuse = 5,
    /// 泄露用户隐私
    PrivacyBreach = 6,
    /// 政治敏感内容
    PoliticalContent = 7,
    /// 封建迷信（过度恐吓）
    Superstition = 8,
    /// 其他违规
    Other = 9,
}
```

### 步骤 1.2：添加 ReportType 实现

```rust
impl ReportType {
    /// 获取举报所需押金倍数（基于 MinReportDeposit，百分比）
    pub fn deposit_multiplier(&self) -> u16 {
        match self {
            ReportType::Pornography => 100,
            ReportType::Gambling => 100,
            ReportType::Drugs => 100,
            ReportType::Fraud => 150,
            ReportType::FalseAdvertising => 120,
            ReportType::Abuse => 80,
            ReportType::PrivacyBreach => 150,
            ReportType::PoliticalContent => 100,
            ReportType::Superstition => 80,
            ReportType::Other => 200,
        }
    }

    /// 获取大师押金扣除比例（基点，10000 = 100%）
    pub fn provider_penalty_rate(&self) -> u16 {
        match self {
            ReportType::Pornography => 5000,
            ReportType::Gambling => 5000,
            ReportType::Drugs => 10000,
            ReportType::Fraud => 8000,
            ReportType::FalseAdvertising => 3000,
            ReportType::Abuse => 2000,
            ReportType::PrivacyBreach => 4000,
            ReportType::PoliticalContent => 5000,
            ReportType::Superstition => 1500,
            ReportType::Other => 2000,
        }
    }

    /// 获取举报者奖励比例（占大师罚金的百分比，基点）
    pub fn reporter_reward_rate(&self) -> u16 {
        match self {
            ReportType::Pornography => 4000,
            ReportType::Gambling => 4000,
            ReportType::Drugs => 5000,
            ReportType::Fraud => 5000,
            ReportType::FalseAdvertising => 3000,
            ReportType::Abuse => 3000,
            ReportType::PrivacyBreach => 4000,
            ReportType::PoliticalContent => 3000,
            ReportType::Superstition => 2000,
            ReportType::Other => 2500,
        }
    }

    /// 获取信用扣分值
    pub fn credit_deduction(&self) -> u16 {
        match self {
            ReportType::Pornography => 150,
            ReportType::Gambling => 150,
            ReportType::Drugs => 500,
            ReportType::Fraud => 200,
            ReportType::FalseAdvertising => 80,
            ReportType::Abuse => 100,
            ReportType::PrivacyBreach => 150,
            ReportType::PoliticalContent => 120,
            ReportType::Superstition => 50,
            ReportType::Other => 50,
        }
    }

    /// 是否触发永久封禁
    pub fn triggers_permanent_ban(&self) -> bool {
        matches!(self, ReportType::Drugs | ReportType::Fraud)
    }
}
```

### 步骤 1.3：添加举报状态枚举

```rust
/// 举报状态
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub enum ReportStatus {
    /// 待审核
    #[default]
    Pending = 0,
    /// 审核中（委员会已介入）
    UnderReview = 1,
    /// 举报成立
    Upheld = 2,
    /// 举报驳回（证据不足）
    Rejected = 3,
    /// 恶意举报（反向惩罚举报者）
    Malicious = 4,
    /// 已撤销（举报者主动撤回）
    Withdrawn = 5,
    /// 已过期（超时未处理）
    Expired = 6,
}
```

### 步骤 1.4：添加举报记录结构

```rust
/// 举报记录
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxEvidenceLen, MaxReasonLen))]
pub struct Report<AccountId, Balance, BlockNumber, MaxEvidenceLen: Get<u32>, MaxReasonLen: Get<u32>> {
    /// 举报 ID
    pub id: u64,
    /// 举报者账户
    pub reporter: AccountId,
    /// 被举报的大师账户
    pub provider: AccountId,
    /// 举报类型
    pub report_type: ReportType,
    /// 证据 IPFS CID
    pub evidence_cid: BoundedVec<u8, MaxEvidenceLen>,
    /// 举报描述
    pub description: BoundedVec<u8, MaxReasonLen>,
    /// 关联的订单 ID
    pub related_order_id: Option<u64>,
    /// 关联的悬赏 ID
    pub related_bounty_id: Option<u64>,
    /// 关联的回答 ID
    pub related_answer_id: Option<u64>,
    /// 举报者缴纳的押金
    pub reporter_deposit: Balance,
    /// 当前状态
    pub status: ReportStatus,
    /// 创建时间
    pub created_at: BlockNumber,
    /// 处理时间
    pub resolved_at: Option<BlockNumber>,
    /// 处理结果说明 CID
    pub resolution_cid: Option<BoundedVec<u8, MaxEvidenceLen>>,
    /// 处理人
    pub resolved_by: Option<AccountId>,
    /// 大师被扣除的押金金额
    pub provider_penalty: Balance,
    /// 举报者获得的奖励金额
    pub reporter_reward: Balance,
    /// 是否为匿名举报
    pub is_anonymous: bool,
}
```

### 步骤 1.5：添加统计结构

```rust
/// 举报统计
#[derive(
    Clone,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub struct ReportStats<Balance: Default> {
    /// 总举报数
    pub total_reports: u64,
    /// 待处理举报数
    pub pending_reports: u64,
    /// 举报成立数
    pub upheld_reports: u64,
    /// 驳回举报数
    pub rejected_reports: u64,
    /// 恶意举报数
    pub malicious_reports: u64,
    /// 总罚没金额
    pub total_penalties: Balance,
    /// 总奖励发放金额
    pub total_rewards: Balance,
    /// 总没收的举报押金
    pub total_confiscated_deposits: Balance,
}

/// 大师举报档案
#[derive(
    Clone,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    Debug,
    Default,
)]
pub struct ProviderReportProfile<BlockNumber: Default> {
    /// 被举报总次数
    pub total_reported: u32,
    /// 举报成立次数
    pub upheld_count: u32,
    /// 累计被扣押金（u128 避免溢出）
    pub total_penalty_amount: u128,
    /// 最近一次被举报时间
    pub last_reported_at: BlockNumber,
    /// 是否处于观察期
    pub under_watch: bool,
    /// 观察期结束时间
    pub watch_period_end: Option<BlockNumber>,
}
```

### 步骤 1.6：添加类型别名

```rust
// 在 lib.rs 的类型别名区域添加

/// 举报记录类型别名
pub type ReportOf<T> = Report<
    <T as frame_system::Config>::AccountId,
    BalanceOf<T>,
    BlockNumberFor<T>,
    <T as Config>::MaxEvidenceLen,
    <T as Config>::MaxReasonLen,
>;

/// 大师举报档案类型别名
pub type ProviderReportProfileOf<T> = ProviderReportProfile<BlockNumberFor<T>>;
```

---

## 阶段 2：存储项定义

**文件**: `src/lib.rs`

### 步骤 2.1：添加存储项

```rust
// 在 #[pallet::storage] 区域添加

// ==================== 举报系统存储项 ====================

/// 下一个举报 ID
#[pallet::storage]
#[pallet::getter(fn next_report_id)]
pub type NextReportId<T> = StorageValue<_, u64, ValueQuery>;

/// 举报记录存储
#[pallet::storage]
#[pallet::getter(fn reports)]
pub type Reports<T: Config> = StorageMap<_, Blake2_128Concat, u64, ReportOf<T>>;

/// 大师收到的举报索引
#[pallet::storage]
#[pallet::getter(fn provider_reports)]
pub type ProviderReports<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<u64, ConstU32<500>>,
    ValueQuery,
>;

/// 用户提交的举报索引
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

/// 待处理举报队列
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

/// 举报冷却期
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

/// 信用黑名单（永久封禁）
#[pallet::storage]
#[pallet::getter(fn credit_blacklist)]
pub type CreditBlacklist<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BlockNumberFor<T>,
>;
```

---

## 阶段 3：配置参数

**文件**: `src/lib.rs`

### 步骤 3.1：在 Config trait 中添加参数

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    // ... 现有配置 ...

    // ==================== 举报系统配置 ====================

    /// 最小举报押金
    #[pallet::constant]
    type MinReportDeposit: Get<BalanceOf<Self>>;

    /// 举报处理超时时间（区块数）
    #[pallet::constant]
    type ReportTimeout: Get<BlockNumberFor<Self>>;

    /// 举报冷却期（同一用户对同一大师）
    #[pallet::constant]
    type ReportCooldownPeriod: Get<BlockNumberFor<Self>>;

    /// 撤回举报的时间窗口
    #[pallet::constant]
    type ReportWithdrawWindow: Get<BlockNumberFor<Self>>;

    /// 恶意举报的信用扣分
    #[pallet::constant]
    type MaliciousReportPenalty: Get<u16>;

    /// 举报审核委员会权限来源
    type ReportReviewOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

    /// 国库账户
    #[pallet::constant]
    type TreasuryAccount: Get<Self::AccountId>;

    /// 证据 CID 最大长度
    #[pallet::constant]
    type MaxEvidenceLen: Get<u32>;

    /// 举报描述最大长度
    #[pallet::constant]
    type MaxReasonLen: Get<u32>;
}
```

---

## 阶段 4：错误与事件

**文件**: `src/lib.rs`

### 步骤 4.1：添加错误类型

```rust
#[pallet::error]
pub enum Error<T> {
    // ... 现有错误 ...

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
    /// 证据内容过长
    EvidenceTooLong,
    /// 描述内容过长
    DescriptionTooLong,
    /// 举报过多
    TooManyReports,
    /// 待处理举报过多
    TooManyPendingReports,
    /// 大师已被封禁
    ProviderAlreadyBanned,
}
```

### 步骤 4.2：添加事件

```rust
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    // ... 现有事件 ...

    // ==================== 举报系统事件 ====================

    /// 举报已提交
    ReportSubmitted {
        report_id: u64,
        reporter: Option<T::AccountId>,
        provider: T::AccountId,
        report_type: ReportType,
        deposit: BalanceOf<T>,
    },

    /// 举报已撤回
    ReportWithdrawn {
        report_id: u64,
    },

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
    ReportExpired {
        report_id: u64,
    },

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
```

---

## 阶段 5：核心函数实现

**文件**: `src/helpers/report.rs`（新建）

### 步骤 5.1：创建文件结构

```bash
# 在 src/helpers/ 目录下创建 report.rs
touch src/helpers/report.rs

# 在 src/helpers/mod.rs 中添加
pub mod report;
```

### 步骤 5.2：实现内部处理函数

```rust
//! # 举报系统辅助函数
//!
//! 本模块包含举报处理相关的内部函数

use crate::pallet::*;
use crate::types::*;
use frame_support::traits::{Currency, ExistenceRequirement};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{Saturating, Zero};

impl<T: Config> Pallet<T> {
    // ==================== 举报成立处理 ====================

    /// 处理举报成立
    ///
    /// 执行流程：
    /// 1. 计算大师罚金
    /// 2. 计算举报者奖励
    /// 3. 扣除大师押金
    /// 4. 发放奖励给举报者
    /// 5. 剩余转入国库
    /// 6. 扣除大师信用分
    /// 7. 判断是否永久封禁
    /// 8. 更新举报档案
    pub(crate) fn handle_upheld_report(
        report_id: u64,
        report: &ReportOf<T>,
        custom_penalty_rate: Option<u16>,
    ) -> DispatchResult {
        let provider = &report.provider;
        let reporter = &report.reporter;
        let report_type = report.report_type;

        // 获取大师信息
        let provider_info = Providers::<T>::get(provider)
            .ok_or(Error::<T>::ProviderNotFound)?;
        let provider_deposit = provider_info.deposit;

        // 1. 计算惩罚金额
        let penalty_rate = custom_penalty_rate.unwrap_or(report_type.provider_penalty_rate());
        let penalty_amount = Self::calculate_penalty(provider_deposit, penalty_rate);

        // 2. 计算举报者奖励
        let reward_rate = report_type.reporter_reward_rate();
        let reporter_reward = Self::calculate_penalty(penalty_amount, reward_rate);

        // 3. 计算国库收入
        let treasury_income = penalty_amount.saturating_sub(reporter_reward);

        // 4. 扣除大师押金（从 reserved 中释放）
        let actual_unreserved = T::Currency::unreserve(provider, penalty_amount);

        // 5. 奖励举报者（包括退还举报押金）
        let total_to_reporter = reporter_reward.saturating_add(report.reporter_deposit);
        T::Currency::transfer(
            &Self::platform_account(),
            reporter,
            total_to_reporter,
            ExistenceRequirement::KeepAlive,
        )?;

        // 6. 剩余部分转入国库
        if !treasury_income.is_zero() {
            T::Currency::transfer(
                &Self::platform_account(),
                &T::TreasuryAccount::get(),
                treasury_income,
                ExistenceRequirement::KeepAlive,
            )?;
        }

        // 7. 扣除大师信用分
        let credit_deduction = report_type.credit_deduction();
        Self::deduct_credit_for_report(provider, credit_deduction);

        // 8. 判断是否永久封禁
        let is_banned = report_type.triggers_permanent_ban();
        if is_banned {
            Self::ban_provider(provider, report_type)?;
        }

        // 9. 更新举报记录
        Reports::<T>::mutate(report_id, |maybe_report| {
            if let Some(r) = maybe_report {
                r.provider_penalty = penalty_amount;
                r.reporter_reward = reporter_reward;
            }
        });

        // 10. 更新大师举报档案
        Self::update_provider_report_profile(provider, penalty_amount);

        // 11. 更新全局统计
        ReportStatistics::<T>::mutate(|stats| {
            stats.total_penalties = stats.total_penalties.saturating_add(penalty_amount);
            stats.total_rewards = stats.total_rewards.saturating_add(reporter_reward);
        });

        Self::deposit_event(Event::ReportUpheld {
            report_id,
            provider: provider.clone(),
            penalty_amount,
            reporter_reward,
            is_banned,
        });

        Ok(())
    }

    // ==================== 举报驳回处理 ====================

    /// 处理举报驳回
    ///
    /// 全额退还举报者押金
    pub(crate) fn handle_rejected_report(
        report_id: u64,
        report: &ReportOf<T>,
    ) -> DispatchResult {
        // 全额退还举报押金
        T::Currency::transfer(
            &Self::platform_account(),
            &report.reporter,
            report.reporter_deposit,
            ExistenceRequirement::KeepAlive,
        )?;

        Self::deposit_event(Event::ReportRejected {
            report_id,
            reporter: report.reporter.clone(),
            deposit_refunded: report.reporter_deposit,
        });

        Ok(())
    }

    // ==================== 恶意举报处理 ====================

    /// 处理恶意举报
    ///
    /// 执行流程：
    /// 1. 没收举报押金转入国库
    /// 2. 扣除举报者信用分
    pub(crate) fn handle_malicious_report(
        report_id: u64,
        report: &ReportOf<T>,
    ) -> DispatchResult {
        // 1. 没收举报押金，转入国库
        T::Currency::transfer(
            &Self::platform_account(),
            &T::TreasuryAccount::get(),
            report.reporter_deposit,
            ExistenceRequirement::KeepAlive,
        )?;

        // 2. 扣除举报者信用分
        let penalty = T::MaliciousReportPenalty::get();
        CreditProfiles::<T>::mutate(&report.reporter, |maybe_profile| {
            if let Some(profile) = maybe_profile {
                profile.total_deductions = profile.total_deductions.saturating_add(penalty);
                Self::recalculate_credit_score(profile);
            }
        });

        // 3. 更新统计
        ReportStatistics::<T>::mutate(|stats| {
            stats.total_confiscated_deposits = stats
                .total_confiscated_deposits
                .saturating_add(report.reporter_deposit);
        });

        Self::deposit_event(Event::MaliciousReportPenalized {
            report_id,
            reporter: report.reporter.clone(),
            deposit_confiscated: report.reporter_deposit,
        });

        Ok(())
    }

    // ==================== 辅助函数 ====================

    /// 计算惩罚/奖励金额
    fn calculate_penalty(amount: BalanceOf<T>, rate_bps: u16) -> BalanceOf<T> {
        amount.saturating_mul(rate_bps.into()) / 10000u32.into()
    }

    /// 扣除信用分（因举报）
    fn deduct_credit_for_report(provider: &T::AccountId, deduction: u16) {
        CreditProfiles::<T>::mutate(provider, |maybe_profile| {
            if let Some(profile) = maybe_profile {
                profile.total_deductions = profile.total_deductions.saturating_add(deduction);
                profile.complaint_count = profile.complaint_count.saturating_add(1);
                profile.complaint_upheld_count = profile.complaint_upheld_count.saturating_add(1);
                profile.last_deduction_reason = Some(DeductionReason::ComplaintUpheld);
                profile.last_deduction_at = Some(<frame_system::Pallet<T>>::block_number());
                Self::recalculate_credit_score(profile);
            }
        });
    }

    /// 封禁大师
    fn ban_provider(provider: &T::AccountId, reason: ReportType) -> DispatchResult {
        Providers::<T>::mutate(provider, |maybe_p| {
            if let Some(p) = maybe_p {
                p.status = ProviderStatus::Banned;
            }
        });

        let current_block = <frame_system::Pallet<T>>::block_number();
        CreditBlacklist::<T>::insert(provider, current_block);

        Self::deposit_event(Event::ProviderBanned {
            provider: provider.clone(),
            reason,
        });

        Ok(())
    }

    /// 更新大师举报档案
    fn update_provider_report_profile(provider: &T::AccountId, penalty_amount: BalanceOf<T>) {
        let current_block = <frame_system::Pallet<T>>::block_number();

        ProviderReportProfiles::<T>::mutate(provider, |profile| {
            profile.upheld_count = profile.upheld_count.saturating_add(1);
            profile.total_penalty_amount = profile
                .total_penalty_amount
                .saturating_add(penalty_amount.saturated_into());

            // 多次举报成立，进入观察期
            if profile.upheld_count >= 3 && !profile.under_watch {
                profile.under_watch = true;
                let watch_duration: BlockNumberFor<T> = 432000u32.into(); // 30天
                profile.watch_period_end = Some(current_block.saturating_add(watch_duration));

                Self::deposit_event(Event::ProviderUnderWatch {
                    provider: provider.clone(),
                    watch_end: current_block.saturating_add(watch_duration),
                });
            }
        });
    }

    /// 计算举报押金
    pub(crate) fn calculate_report_deposit(report_type: ReportType) -> BalanceOf<T> {
        let base_deposit = T::MinReportDeposit::get();
        let multiplier = report_type.deposit_multiplier();
        base_deposit.saturating_mul(multiplier.into()) / 100u32.into()
    }

    /// 验证举报冷却期
    pub(crate) fn check_report_cooldown(
        reporter: &T::AccountId,
        provider: &T::AccountId,
    ) -> DispatchResult {
        let current_block = <frame_system::Pallet<T>>::block_number();

        if let Some(last_report) = ReportCooldown::<T>::get(reporter, provider) {
            ensure!(
                current_block > last_report.saturating_add(T::ReportCooldownPeriod::get()),
                Error::<T>::ReportCooldownActive
            );
        }

        Ok(())
    }

    /// 从待处理队列移除
    pub(crate) fn remove_from_pending(report_id: u64) {
        PendingReports::<T>::mutate(|list| {
            list.retain(|&id| id != report_id);
        });
    }

    /// 更新举报统计
    pub(crate) fn update_report_stats_on_resolve(result: ReportStatus) {
        ReportStatistics::<T>::mutate(|stats| {
            stats.pending_reports = stats.pending_reports.saturating_sub(1);
            match result {
                ReportStatus::Upheld => stats.upheld_reports += 1,
                ReportStatus::Rejected => stats.rejected_reports += 1,
                ReportStatus::Malicious => stats.malicious_reports += 1,
                _ => {}
            }
        });
    }
}
```

---

## 阶段 6：可调用函数

**文件**: `src/lib.rs`

### 步骤 6.1：添加提交举报函数

```rust
/// 提交举报
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
#[pallet::call_index(50)]
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

    // 1. 基础验证
    ensure!(reporter != provider, Error::<T>::CannotReportSelf);
    ensure!(
        Providers::<T>::contains_key(&provider),
        Error::<T>::ProviderNotFound
    );
    ensure!(
        !CreditBlacklist::<T>::contains_key(&provider),
        Error::<T>::ProviderAlreadyBanned
    );

    // 2. 验证冷却期
    Self::check_report_cooldown(&reporter, &provider)?;

    // 3. 计算并收取举报押金
    let required_deposit = Self::calculate_report_deposit(report_type);
    T::Currency::transfer(
        &reporter,
        &Self::platform_account(),
        required_deposit,
        ExistenceRequirement::KeepAlive,
    )?;

    // 4. 构建举报记录
    let current_block = <frame_system::Pallet<T>>::block_number();
    let report_id = NextReportId::<T>::get();
    NextReportId::<T>::put(report_id.saturating_add(1));

    let evidence_bounded: BoundedVec<u8, T::MaxEvidenceLen> = evidence_cid
        .try_into()
        .map_err(|_| Error::<T>::EvidenceTooLong)?;
    let description_bounded: BoundedVec<u8, T::MaxReasonLen> = description
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

    // 5. 存储举报
    Reports::<T>::insert(report_id, report);

    // 6. 更新索引
    ProviderReports::<T>::try_mutate(&provider, |list| {
        list.try_push(report_id).map_err(|_| Error::<T>::TooManyReports)
    })?;
    UserReports::<T>::try_mutate(&reporter, |list| {
        list.try_push(report_id).map_err(|_| Error::<T>::TooManyReports)
    })?;
    PendingReports::<T>::try_mutate(|list| {
        list.try_push(report_id).map_err(|_| Error::<T>::TooManyPendingReports)
    })?;

    // 7. 更新冷却期
    ReportCooldown::<T>::insert(&reporter, &provider, current_block);

    // 8. 更新统计
    ReportStatistics::<T>::mutate(|stats| {
        stats.total_reports += 1;
        stats.pending_reports += 1;
    });

    // 9. 更新大师档案
    ProviderReportProfiles::<T>::mutate(&provider, |profile| {
        profile.total_reported += 1;
        profile.last_reported_at = current_block;
    });

    // 10. 发送事件
    Self::deposit_event(Event::ReportSubmitted {
        report_id,
        reporter: if is_anonymous { None } else { Some(reporter) },
        provider,
        report_type,
        deposit: required_deposit,
    });

    Ok(())
}
```

### 步骤 6.2：添加撤回举报函数

```rust
/// 撤回举报
///
/// 仅在窗口期内且状态为 Pending 时可撤回
/// 撤回后退还 80% 押金
#[pallet::call_index(51)]
#[pallet::weight(Weight::from_parts(30_000_000, 0))]
pub fn withdraw_report(origin: OriginFor<T>, report_id: u64) -> DispatchResult {
    let who = ensure_signed(origin)?;

    Reports::<T>::try_mutate(report_id, |maybe_report| {
        let report = maybe_report.as_mut().ok_or(Error::<T>::ReportNotFound)?;

        // 验证
        ensure!(report.reporter == who, Error::<T>::NotReporter);
        ensure!(report.status == ReportStatus::Pending, Error::<T>::ReportNotPending);

        let current_block = <frame_system::Pallet<T>>::block_number();
        ensure!(
            current_block <= report.created_at.saturating_add(T::ReportWithdrawWindow::get()),
            Error::<T>::WithdrawWindowExpired
        );

        // 退还 80% 押金
        let refund = report.reporter_deposit.saturating_mul(80u32.into()) / 100u32.into();
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
```

### 步骤 6.3：添加审核举报函数

```rust
/// 审核举报（委员会专用）
///
/// # 参数
/// - `report_id`: 举报 ID
/// - `result`: 审核结果（Upheld/Rejected/Malicious）
/// - `resolution_cid`: 处理说明 IPFS CID
/// - `custom_penalty_rate`: 自定义惩罚比例（可选）
#[pallet::call_index(52)]
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
        matches!(result, ReportStatus::Upheld | ReportStatus::Rejected | ReportStatus::Malicious),
        Error::<T>::InvalidReportResult
    );

    // 获取举报记录
    let report = Reports::<T>::get(report_id).ok_or(Error::<T>::ReportNotFound)?;
    ensure!(
        report.status == ReportStatus::Pending || report.status == ReportStatus::UnderReview,
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
    let resolution_bounded: Option<BoundedVec<u8, T::MaxEvidenceLen>> = resolution_cid
        .map(|cid| cid.try_into().map_err(|_| Error::<T>::EvidenceTooLong))
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
```

### 步骤 6.4：添加过期举报处理函数

```rust
/// 处理超时举报
///
/// 任何人可调用，超时后举报者可取回全额押金
#[pallet::call_index(53)]
#[pallet::weight(Weight::from_parts(40_000_000, 0))]
pub fn expire_report(origin: OriginFor<T>, report_id: u64) -> DispatchResult {
    ensure_signed(origin)?;

    let report = Reports::<T>::get(report_id).ok_or(Error::<T>::ReportNotFound)?;
    ensure!(report.status == ReportStatus::Pending, Error::<T>::ReportNotPending);

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
```

---

## 阶段 7：单元测试

**文件**: `src/tests/report_tests.rs`（新建）

### 步骤 7.1：创建测试文件

```bash
touch src/tests/report_tests.rs
```

### 步骤 7.2：编写测试用例

```rust
//! 举报系统测试

use crate::{mock::*, Error, Event, ReportStatus, ReportType};
use frame_support::{assert_noop, assert_ok};

// ==================== 提交举报测试 ====================

#[test]
fn submit_report_works() {
    new_test_ext().execute_with(|| {
        // 注册大师
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(PROVIDER),
            b"Master".to_vec(),
            b"Bio".to_vec(),
            None,
            0,
            0,
        ));

        // 提交举报
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(USER),
            PROVIDER,
            ReportType::FalseAdvertising,
            b"QmEvidence".to_vec(),
            b"Description".to_vec(),
            None,
            None,
            None,
            false,
        ));

        // 验证举报存在
        assert!(DivinationMarket::reports(0).is_some());

        // 验证统计更新
        let stats = DivinationMarket::report_stats();
        assert_eq!(stats.total_reports, 1);
        assert_eq!(stats.pending_reports, 1);
    });
}

#[test]
fn cannot_report_self() {
    new_test_ext().execute_with(|| {
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(PROVIDER),
            b"Master".to_vec(),
            b"Bio".to_vec(),
            None,
            0,
            0,
        ));

        assert_noop!(
            DivinationMarket::submit_report(
                RuntimeOrigin::signed(PROVIDER),
                PROVIDER,
                ReportType::Abuse,
                b"QmEvidence".to_vec(),
                b"Description".to_vec(),
                None,
                None,
                None,
                false,
            ),
            Error::<Test>::CannotReportSelf
        );
    });
}

#[test]
fn report_cooldown_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(PROVIDER),
            b"Master".to_vec(),
            b"Bio".to_vec(),
            None,
            0,
            0,
        ));

        // 第一次举报成功
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(USER),
            PROVIDER,
            ReportType::Abuse,
            b"QmEvidence".to_vec(),
            b"Description".to_vec(),
            None,
            None,
            None,
            false,
        ));

        // 冷却期内再次举报失败
        assert_noop!(
            DivinationMarket::submit_report(
                RuntimeOrigin::signed(USER),
                PROVIDER,
                ReportType::Abuse,
                b"QmEvidence2".to_vec(),
                b"Description2".to_vec(),
                None,
                None,
                None,
                false,
            ),
            Error::<Test>::ReportCooldownActive
        );
    });
}

// ==================== 撤回举报测试 ====================

#[test]
fn withdraw_report_in_window_works() {
    new_test_ext().execute_with(|| {
        // 设置并提交举报
        setup_report();

        // 撤回举报
        assert_ok!(DivinationMarket::withdraw_report(
            RuntimeOrigin::signed(USER),
            0
        ));

        // 验证状态
        let report = DivinationMarket::reports(0).unwrap();
        assert_eq!(report.status, ReportStatus::Withdrawn);
    });
}

#[test]
fn withdraw_report_after_window_fails() {
    new_test_ext().execute_with(|| {
        setup_report();

        // 超过撤回窗口
        run_to_block(REPORT_WITHDRAW_WINDOW + 1);

        assert_noop!(
            DivinationMarket::withdraw_report(RuntimeOrigin::signed(USER), 0),
            Error::<Test>::WithdrawWindowExpired
        );
    });
}

// ==================== 审核举报测试 ====================

#[test]
fn resolve_report_upheld_works() {
    new_test_ext().execute_with(|| {
        setup_report();

        let provider_balance_before = Balances::free_balance(PROVIDER);
        let reporter_balance_before = Balances::free_balance(USER);

        // 委员会审核通过
        assert_ok!(DivinationMarket::resolve_report(
            RuntimeOrigin::signed(COMMITTEE),
            0,
            ReportStatus::Upheld,
            Some(b"QmResolution".to_vec()),
            None,
        ));

        // 验证大师被扣款
        let provider_balance_after = Balances::free_balance(PROVIDER);
        assert!(provider_balance_after < provider_balance_before);

        // 验证举报者获得奖励
        let reporter_balance_after = Balances::free_balance(USER);
        assert!(reporter_balance_after > reporter_balance_before);

        // 验证状态
        let report = DivinationMarket::reports(0).unwrap();
        assert_eq!(report.status, ReportStatus::Upheld);
    });
}

#[test]
fn resolve_report_rejected_works() {
    new_test_ext().execute_with(|| {
        setup_report();

        let reporter_balance_before = Balances::free_balance(USER);

        // 委员会驳回
        assert_ok!(DivinationMarket::resolve_report(
            RuntimeOrigin::signed(COMMITTEE),
            0,
            ReportStatus::Rejected,
            None,
            None,
        ));

        // 验证押金退还
        let reporter_balance_after = Balances::free_balance(USER);
        assert!(reporter_balance_after > reporter_balance_before);

        // 验证状态
        let report = DivinationMarket::reports(0).unwrap();
        assert_eq!(report.status, ReportStatus::Rejected);
    });
}

#[test]
fn resolve_report_malicious_works() {
    new_test_ext().execute_with(|| {
        setup_report();

        let treasury_balance_before = Balances::free_balance(TREASURY);

        // 委员会判定恶意举报
        assert_ok!(DivinationMarket::resolve_report(
            RuntimeOrigin::signed(COMMITTEE),
            0,
            ReportStatus::Malicious,
            None,
            None,
        ));

        // 验证押金转入国库
        let treasury_balance_after = Balances::free_balance(TREASURY);
        assert!(treasury_balance_after > treasury_balance_before);

        // 验证举报者信用扣分
        let credit = DivinationMarket::credit_profiles(USER);
        if let Some(profile) = credit {
            assert!(profile.total_deductions > 0);
        }
    });
}

// ==================== 过期举报测试 ====================

#[test]
fn expire_report_works() {
    new_test_ext().execute_with(|| {
        setup_report();

        let reporter_balance_before = Balances::free_balance(USER);

        // 超过超时时间
        run_to_block(REPORT_TIMEOUT + 1);

        // 处理过期
        assert_ok!(DivinationMarket::expire_report(
            RuntimeOrigin::signed(ANYONE),
            0
        ));

        // 验证全额退还
        let reporter_balance_after = Balances::free_balance(USER);
        assert!(reporter_balance_after > reporter_balance_before);

        // 验证状态
        let report = DivinationMarket::reports(0).unwrap();
        assert_eq!(report.status, ReportStatus::Expired);
    });
}

// ==================== 永久封禁测试 ====================

#[test]
fn permanent_ban_on_drugs_report() {
    new_test_ext().execute_with(|| {
        // 注册大师
        assert_ok!(DivinationMarket::register_provider(
            RuntimeOrigin::signed(PROVIDER),
            b"Master".to_vec(),
            b"Bio".to_vec(),
            None,
            0,
            0,
        ));

        // 提交毒品相关举报
        assert_ok!(DivinationMarket::submit_report(
            RuntimeOrigin::signed(USER),
            PROVIDER,
            ReportType::Drugs,
            b"QmEvidence".to_vec(),
            b"Description".to_vec(),
            None,
            None,
            None,
            false,
        ));

        // 委员会审核通过
        assert_ok!(DivinationMarket::resolve_report(
            RuntimeOrigin::signed(COMMITTEE),
            0,
            ReportStatus::Upheld,
            None,
            None,
        ));

        // 验证大师被永久封禁
        let provider = DivinationMarket::providers(PROVIDER).unwrap();
        assert_eq!(provider.status, ProviderStatus::Banned);

        // 验证在黑名单中
        assert!(DivinationMarket::credit_blacklist(PROVIDER).is_some());
    });
}

// ==================== 奖励计算测试 ====================

#[test]
fn reward_calculation_correct() {
    new_test_ext().execute_with(|| {
        // 测试各类型举报的押金计算
        assert_eq!(
            ReportType::Pornography.deposit_multiplier(),
            100 // 1x
        );
        assert_eq!(
            ReportType::Other.deposit_multiplier(),
            200 // 2x
        );

        // 测试惩罚比例
        assert_eq!(
            ReportType::Drugs.provider_penalty_rate(),
            10000 // 100%
        );
        assert_eq!(
            ReportType::FalseAdvertising.provider_penalty_rate(),
            3000 // 30%
        );

        // 测试奖励比例
        assert_eq!(
            ReportType::Fraud.reporter_reward_rate(),
            5000 // 50%
        );
    });
}

// ==================== 辅助函数 ====================

fn setup_report() {
    // 注册大师
    assert_ok!(DivinationMarket::register_provider(
        RuntimeOrigin::signed(PROVIDER),
        b"Master".to_vec(),
        b"Bio".to_vec(),
        None,
        0,
        0,
    ));

    // 提交举报
    assert_ok!(DivinationMarket::submit_report(
        RuntimeOrigin::signed(USER),
        PROVIDER,
        ReportType::FalseAdvertising,
        b"QmEvidence".to_vec(),
        b"Description".to_vec(),
        None,
        None,
        None,
        false,
    ));
}

fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
    }
}
```

### 步骤 7.3：更新测试模块

在 `src/tests/mod.rs` 中添加：

```rust
mod report_tests;
```

---

## 阶段 8：Runtime 集成

**文件**: `runtime/src/lib.rs`

### 步骤 8.1：配置参数

```rust
parameter_types! {
    // ... 现有参数 ...

    // 举报系统参数
    pub const MinReportDeposit: Balance = 10 * DUST;
    pub const ReportTimeout: BlockNumber = 100800; // 7天
    pub const ReportCooldownPeriod: BlockNumber = 14400; // 1天
    pub const ReportWithdrawWindow: BlockNumber = 7200; // 12小时
    pub const MaliciousReportPenalty: u16 = 30;
    pub const MaxEvidenceLen: u32 = 128;
    pub const MaxReasonLen: u32 = 512;
}
```

### 步骤 8.2：配置 Pallet

```rust
impl pallet_divination_market::Config for Runtime {
    // ... 现有配置 ...

    // 举报系统配置
    type MinReportDeposit = MinReportDeposit;
    type ReportTimeout = ReportTimeout;
    type ReportCooldownPeriod = ReportCooldownPeriod;
    type ReportWithdrawWindow = ReportWithdrawWindow;
    type MaliciousReportPenalty = MaliciousReportPenalty;
    type ReportReviewOrigin = EnsureSignedBy<CouncilCollective, AccountId>;
    type TreasuryAccount = TreasuryPalletId;
    type MaxEvidenceLen = MaxEvidenceLen;
    type MaxReasonLen = MaxReasonLen;
}
```

---

## 开发检查清单

### 阶段 1：数据类型
- [ ] `ReportType` 枚举及实现
- [ ] `ReportStatus` 枚举
- [ ] `Report` 结构体
- [ ] `ReportStats` 结构体
- [ ] `ProviderReportProfile` 结构体
- [ ] 类型别名定义

### 阶段 2：存储项
- [ ] `NextReportId`
- [ ] `Reports`
- [ ] `ProviderReports`
- [ ] `UserReports`
- [ ] `ProviderReportProfiles`
- [ ] `PendingReports`
- [ ] `ReportStatistics`
- [ ] `ReportCooldown`
- [ ] `CreditBlacklist`

### 阶段 3：配置参数
- [ ] `MinReportDeposit`
- [ ] `ReportTimeout`
- [ ] `ReportCooldownPeriod`
- [ ] `ReportWithdrawWindow`
- [ ] `MaliciousReportPenalty`
- [ ] `ReportReviewOrigin`
- [ ] `TreasuryAccount`
- [ ] `MaxEvidenceLen`
- [ ] `MaxReasonLen`

### 阶段 4：错误与事件
- [ ] 所有错误类型
- [ ] 所有事件类型

### 阶段 5：核心函数
- [ ] `handle_upheld_report`
- [ ] `handle_rejected_report`
- [ ] `handle_malicious_report`
- [ ] `calculate_report_deposit`
- [ ] `check_report_cooldown`
- [ ] `deduct_credit_for_report`
- [ ] `ban_provider`
- [ ] `update_provider_report_profile`

### 阶段 6：可调用函数
- [ ] `submit_report`
- [ ] `withdraw_report`
- [ ] `resolve_report`
- [ ] `expire_report`

### 阶段 7：测试
- [ ] 提交举报测试
- [ ] 不能举报自己测试
- [ ] 冷却期测试
- [ ] 撤回举报测试
- [ ] 审核成立测试
- [ ] 审核驳回测试
- [ ] 恶意举报测试
- [ ] 过期举报测试
- [ ] 永久封禁测试
- [ ] 奖励计算测试

### 阶段 8：集成
- [ ] Runtime 参数配置
- [ ] Pallet 配置
- [ ] 编译通过
- [ ] 测试通过

---

## 命令参考

```bash
# 检查编译
cargo check -p pallet-divination-market

# 运行测试
cargo test -p pallet-divination-market --lib -- --nocapture

# 运行特定测试
cargo test -p pallet-divination-market report_tests --lib -- --nocapture

# 构建 release
cargo build --release

# 生成文档
cargo doc -p pallet-divination-market --open
```
