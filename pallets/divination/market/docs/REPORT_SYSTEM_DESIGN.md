# 占卜大师举报机制设计文档

## 一、功能概述

### 1.1 背景

基于「信用机制 + 押金扣除」替代「审批机制」的大师管理方案，需要配套的社区监督机制。本模块实现去中心化举报系统，允许任何用户举报大师违规行为，委员会审核通过后扣除大师押金并奖励举报者。

### 1.2 核心原则

| 原则 | 说明 |
|------|------|
| **开放举报** | 所有用户均可举报，无门槛 |
| **押金约束** | 举报者需缴纳押金，防止恶意举报 |
| **经济激励** | 举报成立后，大师罚金部分奖励给举报者 |
| **委员会治理** | 由 `GovernanceOrigin` 或 `Collective` 审核裁决 |
| **链上透明** | 举报记录、审核结果全程上链 |

### 1.3 业务流程

```
┌─────────────┐    缴纳押金     ┌─────────────┐    委员会审核    ┌─────────────┐
│   举报者    │ ────────────→ │  举报待审核  │ ────────────→  │  审核结果   │
└─────────────┘                └─────────────┘                 └─────────────┘
                                                                     │
                    ┌────────────────────────────────────────────────┼────────────────────┐
                    │                                                │                    │
                    ▼                                                ▼                    ▼
            ┌─────────────┐                                  ┌─────────────┐      ┌─────────────┐
            │  举报成立   │                                  │  举报驳回   │      │  恶意举报   │
            └─────────────┘                                  └─────────────┘      └─────────────┘
                    │                                                │                    │
                    ▼                                                ▼                    ▼
            ┌─────────────┐                                  ┌─────────────┐      ┌─────────────┐
            │ 扣大师押金  │                                  │ 退举报押金  │      │ 没收押金   │
            │ 奖励举报者  │                                  │  无惩罚    │      │ 举报者扣分  │
            │ 退举报押金  │                                  └─────────────┘      └─────────────┘
            │ 大师扣信用  │
            └─────────────┘
```

---

## 二、数据结构设计

### 2.1 举报类型 `ReportType`

```rust
/// 举报类型
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
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

impl ReportType {
    /// 获取举报所需押金倍数（基于 MinReportDeposit）
    pub fn deposit_multiplier(&self) -> u16 {
        match self {
            ReportType::Pornography => 100,      // 1x
            ReportType::Gambling => 100,         // 1x
            ReportType::Drugs => 100,            // 1x
            ReportType::Fraud => 150,            // 1.5x（需要更多举证）
            ReportType::FalseAdvertising => 120, // 1.2x
            ReportType::Abuse => 80,             // 0.8x（易判断）
            ReportType::PrivacyBreach => 150,    // 1.5x
            ReportType::PoliticalContent => 100, // 1x
            ReportType::Superstition => 80,      // 0.8x
            ReportType::Other => 200,            // 2x（避免滥用）
        }
    }

    /// 获取大师押金扣除比例（基点，10000 = 100%）
    pub fn provider_penalty_rate(&self) -> u16 {
        match self {
            ReportType::Pornography => 5000,      // 50%
            ReportType::Gambling => 5000,         // 50%
            ReportType::Drugs => 10000,           // 100%（永久封禁）
            ReportType::Fraud => 8000,            // 80%
            ReportType::FalseAdvertising => 3000, // 30%
            ReportType::Abuse => 2000,            // 20%
            ReportType::PrivacyBreach => 4000,    // 40%
            ReportType::PoliticalContent => 5000, // 50%
            ReportType::Superstition => 1500,     // 15%
            ReportType::Other => 2000,            // 20%
        }
    }

    /// 获取举报者奖励比例（占大师罚金的百分比，基点）
    pub fn reporter_reward_rate(&self) -> u16 {
        match self {
            ReportType::Pornography => 4000,      // 40%
            ReportType::Gambling => 4000,         // 40%
            ReportType::Drugs => 5000,            // 50%
            ReportType::Fraud => 5000,            // 50%
            ReportType::FalseAdvertising => 3000, // 30%
            ReportType::Abuse => 3000,            // 30%
            ReportType::PrivacyBreach => 4000,    // 40%
            ReportType::PoliticalContent => 3000, // 30%
            ReportType::Superstition => 2000,     // 20%
            ReportType::Other => 2500,            // 25%
        }
    }

    /// 获取信用扣分值
    pub fn credit_deduction(&self) -> u16 {
        match self {
            ReportType::Pornography => 150,
            ReportType::Gambling => 150,
            ReportType::Drugs => 500,        // 直接封禁级别
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

### 2.2 举报状态 `ReportStatus`

```rust
/// 举报状态
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
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

### 2.3 举报记录 `Report`

```rust
/// 举报记录
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
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

    /// 证据 IPFS CID（截图、录音、聊天记录等）
    pub evidence_cid: BoundedVec<u8, MaxEvidenceLen>,

    /// 举报描述
    pub description: BoundedVec<u8, MaxReasonLen>,

    /// 关联的订单 ID（如有）
    pub related_order_id: Option<u64>,

    /// 关联的悬赏 ID（如有）
    pub related_bounty_id: Option<u64>,

    /// 关联的回答 ID（如有）
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

    /// 处理人（委员会成员）
    pub resolved_by: Option<AccountId>,

    /// 大师被扣除的押金金额
    pub provider_penalty: Balance,

    /// 举报者获得的奖励金额
    pub reporter_reward: Balance,

    /// 是否为匿名举报
    pub is_anonymous: bool,
}
```

### 2.4 举报统计 `ReportStats`

```rust
/// 举报统计
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
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
```

### 2.5 大师举报档案 `ProviderReportProfile`

```rust
/// 大师举报档案
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct ProviderReportProfile {
    /// 被举报总次数
    pub total_reported: u32,
    /// 举报成立次数
    pub upheld_count: u32,
    /// 累计被扣押金
    pub total_penalty_amount: u128,
    /// 最近一次被举报时间（区块号）
    pub last_reported_at: u32,
    /// 是否处于观察期
    pub under_watch: bool,
    /// 观察期结束时间
    pub watch_period_end: Option<u32>,
}
```

---

## 三、存储设计

```rust
// ==================== 举报存储项 ====================

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
    ProviderReportProfile,
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
```

---

## 四、配置参数

```rust
#[pallet::config]
pub trait Config: frame_system::Config + pallet_timestamp::Config {
    // ... 其他配置 ...

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
    type ReportReviewOrigin: EnsureOrigin<Self::RuntimeOrigin>;

    /// 国库账户（罚金剩余部分归国库）
    #[pallet::constant]
    type TreasuryAccount: Get<Self::AccountId>;
}
```

### 推荐参数值

| 参数 | 推荐值 | 说明 |
|------|--------|------|
| `MinReportDeposit` | 10 DUST | 基础举报押金 |
| `ReportTimeout` | 100800 (7天) | 超时未处理自动退押金 |
| `ReportCooldownPeriod` | 14400 (1天) | 防止频繁举报骚扰 |
| `ReportWithdrawWindow` | 7200 (12小时) | 撤回窗口期 |
| `MaliciousReportPenalty` | 30分 | 恶意举报扣信用分 |

---

## 五、可调用函数

### 5.1 提交举报

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

    // 不能举报自己
    ensure!(reporter != provider, Error::<T>::CannotReportSelf);

    // 验证大师存在
    ensure!(
        Providers::<T>::contains_key(&provider),
        Error::<T>::ProviderNotFound
    );

    // 验证冷却期
    let current_block = <frame_system::Pallet<T>>::block_number();
    if let Some(last_report) = ReportCooldown::<T>::get(&reporter, &provider) {
        ensure!(
            current_block > last_report + T::ReportCooldownPeriod::get(),
            Error::<T>::ReportCooldownActive
        );
    }

    // 计算举报押金
    let base_deposit = T::MinReportDeposit::get();
    let multiplier = report_type.deposit_multiplier();
    let required_deposit = base_deposit.saturating_mul(multiplier.into()) / 100u32.into();

    // 转账押金到平台账户
    T::Currency::transfer(
        &reporter,
        &T::PlatformAccount::get(),
        required_deposit,
        ExistenceRequirement::KeepAlive,
    )?;

    // 创建举报记录
    let report_id = NextReportId::<T>::get();
    NextReportId::<T>::put(report_id.saturating_add(1));

    let evidence_bounded = BoundedVec::try_from(evidence_cid)
        .map_err(|_| Error::<T>::EvidenceTooLong)?;
    let description_bounded = BoundedVec::try_from(description)
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

    Reports::<T>::insert(report_id, report);

    // 更新索引
    ProviderReports::<T>::try_mutate(&provider, |list| {
        list.try_push(report_id).map_err(|_| Error::<T>::TooManyReports)
    })?;
    UserReports::<T>::try_mutate(&reporter, |list| {
        list.try_push(report_id).map_err(|_| Error::<T>::TooManyReports)
    })?;
    PendingReports::<T>::try_mutate(|list| {
        list.try_push(report_id).map_err(|_| Error::<T>::TooManyPendingReports)
    })?;

    // 更新冷却期
    ReportCooldown::<T>::insert(&reporter, &provider, current_block);

    // 更新统计
    ReportStatistics::<T>::mutate(|stats| {
        stats.total_reports += 1;
        stats.pending_reports += 1;
    });

    // 更新大师档案
    ProviderReportProfiles::<T>::mutate(&provider, |profile| {
        profile.total_reported += 1;
        profile.last_reported_at = current_block.saturated_into();
    });

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

### 5.2 撤回举报

```rust
/// 撤回举报
///
/// 仅在窗口期内且状态为 Pending 时可撤回
/// 撤回后退还 80% 押金（20% 作为滥用费用）
#[pallet::call_index(41)]
#[pallet::weight(Weight::from_parts(30_000_000, 0))]
pub fn withdraw_report(origin: OriginFor<T>, report_id: u64) -> DispatchResult {
    let who = ensure_signed(origin)?;

    Reports::<T>::try_mutate(report_id, |maybe_report| {
        let report = maybe_report.as_mut().ok_or(Error::<T>::ReportNotFound)?;

        ensure!(report.reporter == who, Error::<T>::NotReporter);
        ensure!(report.status == ReportStatus::Pending, Error::<T>::ReportNotPending);

        let current_block = <frame_system::Pallet<T>>::block_number();
        ensure!(
            current_block <= report.created_at + T::ReportWithdrawWindow::get(),
            Error::<T>::WithdrawWindowExpired
        );

        // 退还 80% 押金
        let refund = report.reporter_deposit.saturating_mul(80u32.into()) / 100u32.into();
        T::Currency::transfer(
            &T::PlatformAccount::get(),
            &who,
            refund,
            ExistenceRequirement::KeepAlive,
        )?;

        report.status = ReportStatus::Withdrawn;
        report.resolved_at = Some(current_block);

        Ok::<_, DispatchError>(())
    })?;

    // 从待处理队列移除
    PendingReports::<T>::mutate(|list| {
        list.retain(|&id| id != report_id);
    });

    // 更新统计
    ReportStatistics::<T>::mutate(|stats| {
        stats.pending_reports = stats.pending_reports.saturating_sub(1);
    });

    Self::deposit_event(Event::ReportWithdrawn { report_id });

    Ok(())
}
```

### 5.3 审核举报（委员会）

```rust
/// 审核举报
///
/// 仅委员会/治理权限可调用
///
/// # 参数
/// - `report_id`: 举报 ID
/// - `result`: 审核结果（Upheld/Rejected/Malicious）
/// - `resolution_cid`: 处理说明 IPFS CID
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
    let resolver = T::ReportReviewOrigin::ensure_origin(origin)?;

    ensure!(
        matches!(result, ReportStatus::Upheld | ReportStatus::Rejected | ReportStatus::Malicious),
        Error::<T>::InvalidReportResult
    );

    let report = Reports::<T>::get(report_id).ok_or(Error::<T>::ReportNotFound)?;
    ensure!(
        report.status == ReportStatus::Pending || report.status == ReportStatus::UnderReview,
        Error::<T>::ReportAlreadyResolved
    );

    let current_block = <frame_system::Pallet<T>>::block_number();
    let resolution_bounded = resolution_cid
        .map(|cid| BoundedVec::try_from(cid).map_err(|_| Error::<T>::EvidenceTooLong))
        .transpose()?;

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
    Reports::<T>::mutate(report_id, |maybe_report| {
        if let Some(r) = maybe_report {
            r.status = result;
            r.resolved_at = Some(current_block);
            r.resolution_cid = resolution_bounded;
            r.resolved_by = Some(resolver.clone());
        }
    });

    // 从待处理队列移除
    PendingReports::<T>::mutate(|list| {
        list.retain(|&id| id != report_id);
    });

    // 更新统计
    ReportStatistics::<T>::mutate(|stats| {
        stats.pending_reports = stats.pending_reports.saturating_sub(1);
        match result {
            ReportStatus::Upheld => stats.upheld_reports += 1,
            ReportStatus::Rejected => stats.rejected_reports += 1,
            ReportStatus::Malicious => stats.malicious_reports += 1,
            _ => {}
        }
    });

    Self::deposit_event(Event::ReportResolved {
        report_id,
        result,
        resolver,
    });

    Ok(())
}
```

### 5.4 处理超时举报

```rust
/// 处理超时举报
///
/// 任何人可调用，超时后举报者可取回押金
#[pallet::call_index(43)]
#[pallet::weight(Weight::from_parts(40_000_000, 0))]
pub fn expire_report(origin: OriginFor<T>, report_id: u64) -> DispatchResult {
    ensure_signed(origin)?;

    let report = Reports::<T>::get(report_id).ok_or(Error::<T>::ReportNotFound)?;
    ensure!(report.status == ReportStatus::Pending, Error::<T>::ReportNotPending);

    let current_block = <frame_system::Pallet<T>>::block_number();
    ensure!(
        current_block > report.created_at + T::ReportTimeout::get(),
        Error::<T>::ReportNotExpired
    );

    // 全额退还举报押金
    T::Currency::transfer(
        &T::PlatformAccount::get(),
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
    PendingReports::<T>::mutate(|list| {
        list.retain(|&id| id != report_id);
    });

    ReportStatistics::<T>::mutate(|stats| {
        stats.pending_reports = stats.pending_reports.saturating_sub(1);
    });

    Self::deposit_event(Event::ReportExpired { report_id });

    Ok(())
}
```

---

## 六、内部处理函数

### 6.1 举报成立处理

```rust
/// 处理举报成立
fn handle_upheld_report(
    report_id: u64,
    report: &ReportOf<T>,
    custom_penalty_rate: Option<u16>,
) -> DispatchResult {
    let provider = &report.provider;
    let reporter = &report.reporter;
    let report_type = report.report_type;

    // 获取大师押金
    let provider_info = Providers::<T>::get(provider)
        .ok_or(Error::<T>::ProviderNotFound)?;
    let provider_deposit = provider_info.deposit;

    // 计算惩罚金额
    let penalty_rate = custom_penalty_rate.unwrap_or(report_type.provider_penalty_rate());
    let penalty_amount = provider_deposit.saturating_mul(penalty_rate.into()) / 10000u32.into();

    // 计算举报者奖励（惩罚金额的一部分）
    let reward_rate = report_type.reporter_reward_rate();
    let reporter_reward = penalty_amount.saturating_mul(reward_rate.into()) / 10000u32.into();

    // 计算国库收入（惩罚金额剩余部分）
    let treasury_income = penalty_amount.saturating_sub(reporter_reward);

    // 1. 扣除大师押金
    T::Currency::unreserve(provider, penalty_amount);

    // 2. 奖励举报者（包括退还举报押金）
    let total_to_reporter = reporter_reward.saturating_add(report.reporter_deposit);
    T::Currency::transfer(
        &T::PlatformAccount::get(),
        reporter,
        total_to_reporter,
        ExistenceRequirement::KeepAlive,
    )?;

    // 3. 剩余部分转入国库
    if !treasury_income.is_zero() {
        T::Currency::transfer(
            &T::PlatformAccount::get(),
            &T::TreasuryAccount::get(),
            treasury_income,
            ExistenceRequirement::KeepAlive,
        )?;
    }

    // 4. 扣除大师信用分
    let credit_deduction = report_type.credit_deduction();
    CreditProfiles::<T>::mutate(provider, |maybe_profile| {
        if let Some(profile) = maybe_profile {
            profile.total_deductions = profile.total_deductions.saturating_add(credit_deduction);
            profile.complaint_count = profile.complaint_count.saturating_add(1);
            profile.complaint_upheld_count = profile.complaint_upheld_count.saturating_add(1);
            Self::recalculate_credit_score(profile);
        }
    });

    // 5. 判断是否永久封禁
    if report_type.triggers_permanent_ban() {
        Providers::<T>::mutate(provider, |maybe_p| {
            if let Some(p) = maybe_p {
                p.status = ProviderStatus::Banned;
            }
        });
        CreditBlacklist::<T>::insert(provider, <frame_system::Pallet<T>>::block_number());
        Self::deposit_event(Event::ProviderBanned { provider: provider.clone() });
    }

    // 6. 更新举报记录
    Reports::<T>::mutate(report_id, |maybe_report| {
        if let Some(r) = maybe_report {
            r.provider_penalty = penalty_amount;
            r.reporter_reward = reporter_reward;
        }
    });

    // 7. 更新大师举报档案
    ProviderReportProfiles::<T>::mutate(provider, |profile| {
        profile.upheld_count += 1;
        profile.total_penalty_amount = profile
            .total_penalty_amount
            .saturating_add(penalty_amount.saturated_into());

        // 如果短期内多次举报成立，进入观察期
        if profile.upheld_count >= 3 {
            profile.under_watch = true;
            let current_block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();
            profile.watch_period_end = Some(current_block + 432000); // 30天观察期
        }
    });

    // 8. 更新统计
    ReportStatistics::<T>::mutate(|stats| {
        stats.total_penalties = stats.total_penalties.saturating_add(penalty_amount);
        stats.total_rewards = stats.total_rewards.saturating_add(reporter_reward);
    });

    Self::deposit_event(Event::ReportUpheld {
        report_id,
        provider: provider.clone(),
        penalty_amount,
        reporter_reward,
        is_banned: report_type.triggers_permanent_ban(),
    });

    Ok(())
}
```

### 6.2 举报驳回处理

```rust
/// 处理举报驳回
fn handle_rejected_report(report_id: u64, report: &ReportOf<T>) -> DispatchResult {
    // 全额退还举报押金
    T::Currency::transfer(
        &T::PlatformAccount::get(),
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
```

### 6.3 恶意举报处理

```rust
/// 处理恶意举报
fn handle_malicious_report(report_id: u64, report: &ReportOf<T>) -> DispatchResult {
    // 没收举报押金，转入国库
    T::Currency::transfer(
        &T::PlatformAccount::get(),
        &T::TreasuryAccount::get(),
        report.reporter_deposit,
        ExistenceRequirement::KeepAlive,
    )?;

    // 扣除举报者信用分（如果有信用档案）
    CreditProfiles::<T>::mutate(&report.reporter, |maybe_profile| {
        if let Some(profile) = maybe_profile {
            let penalty = T::MaliciousReportPenalty::get();
            profile.total_deductions = profile.total_deductions.saturating_add(penalty);
            Self::recalculate_credit_score(profile);
        }
    });

    // 更新统计
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
```

---

## 七、事件定义

```rust
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    /// 举报已提交
    ReportSubmitted {
        report_id: u64,
        reporter: Option<T::AccountId>, // 匿名时为 None
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
    },
}
```

---

## 八、错误定义

```rust
#[pallet::error]
pub enum Error<T> {
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
    /// 举报过多
    TooManyReports,
    /// 待处理举报过多
    TooManyPendingReports,
}
```

---

## 九、奖励分配比例总结

### 9.1 举报成立时的资金流向

```
大师押金扣除 (penalty_amount)
    │
    ├─→ 举报者奖励 (reporter_reward_rate%)
    │       └─→ 40%-50% of penalty
    │
    └─→ 国库收入 (剩余部分)
            └─→ 50%-60% of penalty

举报者押金
    └─→ 全额退还给举报者
```

### 9.2 各类型举报的经济参数一览

| 举报类型 | 押金倍数 | 大师罚金比例 | 举报者奖励比例 | 信用扣分 | 是否封禁 |
|----------|----------|--------------|----------------|----------|----------|
| 黄色内容 | 1x | 50% | 40% | 150分 | 否 |
| 赌博 | 1x | 50% | 40% | 150分 | 否 |
| 毒品 | 1x | 100% | 50% | 500分 | 永久 |
| 诈骗 | 1.5x | 80% | 50% | 200分 | 永久 |
| 虚假宣传 | 1.2x | 30% | 30% | 80分 | 否 |
| 辱骂 | 0.8x | 20% | 30% | 100分 | 否 |
| 泄露隐私 | 1.5x | 40% | 40% | 150分 | 否 |
| 政治敏感 | 1x | 50% | 30% | 120分 | 否 |
| 封建迷信 | 0.8x | 15% | 20% | 50分 | 否 |
| 其他 | 2x | 20% | 25% | 50分 | 否 |

### 9.3 示例计算

**场景**：用户举报大师发布色情内容

- 大师押金：1000 DUST
- 基础举报押金：10 DUST
- 实际举报押金：10 × 1.0 = **10 DUST**

**举报成立后**：
- 大师被扣除：1000 × 50% = **500 DUST**
- 举报者奖励：500 × 40% = **200 DUST**
- 国库收入：500 × 60% = **300 DUST**
- 举报押金退还：**10 DUST**
- 举报者总获得：200 + 10 = **210 DUST**
- 大师信用扣分：**-150 分**

---

## 十、安全考虑

### 10.1 防恶意举报

| 机制 | 说明 |
|------|------|
| 举报押金 | 需缴纳押金，恶意举报被没收 |
| 冷却期 | 同一用户对同一大师每天只能举报1次 |
| 恶意标记 | 委员会可标记恶意举报，没收押金并扣信用分 |
| 押金倍数 | "其他"类型押金2倍，防止滥用 |

### 10.2 防串通举报

| 风险 | 对策 |
|------|------|
| A让B举报自己套取资金 | 举报成立后大师押金扣除 > 举报者奖励 |
| 多账户联合举报 | 同一大师被多次举报，后续举报需更多证据 |

### 10.3 隐私保护

- 支持匿名举报（`is_anonymous`）
- 证据存储在 IPFS，仅存 CID 上链
- 举报者身份可选择性公开

---

## 十一、与现有模块的集成

### 11.1 与信用体系集成

```rust
// 在 handle_upheld_report 中已实现
CreditProfiles::<T>::mutate(provider, |maybe_profile| {
    if let Some(profile) = maybe_profile {
        profile.complaint_count += 1;
        profile.complaint_upheld_count += 1;
        profile.total_deductions += credit_deduction;
        Self::recalculate_credit_score(profile);
    }
});
```

### 11.2 与大师状态集成

```rust
// 永久封禁
if report_type.triggers_permanent_ban() {
    Providers::<T>::mutate(provider, |maybe_p| {
        if let Some(p) = maybe_p {
            p.status = ProviderStatus::Banned;
        }
    });
    CreditBlacklist::<T>::insert(provider, current_block);
}
```

### 11.3 与违规记录集成

举报成立后，自动创建 `ViolationRecord`：

```rust
// 可在 handle_upheld_report 中添加
let violation_type = match report.report_type {
    ReportType::Drugs | ReportType::Fraud => ViolationType::Critical,
    ReportType::Pornography | ReportType::Gambling => ViolationType::Severe,
    ReportType::PrivacyBreach | ReportType::Abuse => ViolationType::Moderate,
    _ => ViolationType::Minor,
};

// 调用现有的违规记录函数
Self::record_violation_internal(provider, violation_type, ...);
```

---

## 十二、测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_report_works() {
        // 测试正常举报流程
    }

    #[test]
    fn test_cannot_report_self() {
        // 测试不能举报自己
    }

    #[test]
    fn test_report_cooldown() {
        // 测试冷却期限制
    }

    #[test]
    fn test_withdraw_report_in_window() {
        // 测试窗口期内撤回
    }

    #[test]
    fn test_withdraw_report_after_window() {
        // 测试窗口期后不能撤回
    }

    #[test]
    fn test_resolve_report_upheld() {
        // 测试举报成立：押金扣除、奖励发放、信用扣分
    }

    #[test]
    fn test_resolve_report_rejected() {
        // 测试举报驳回：退还押金
    }

    #[test]
    fn test_resolve_report_malicious() {
        // 测试恶意举报：没收押金、扣信用分
    }

    #[test]
    fn test_report_expired() {
        // 测试超时后退还押金
    }

    #[test]
    fn test_permanent_ban_on_drugs_report() {
        // 测试毒品举报成立后永久封禁
    }

    #[test]
    fn test_reward_calculation() {
        // 测试各类型举报的奖励计算正确性
    }
}
```

---

## 十三、总结

本举报机制实现了：

1. **开放举报**：任何用户均可举报
2. **押金约束**：举报需缴纳押金，防止恶意举报
3. **经济激励**：举报成立后奖励举报者
4. **委员会治理**：由 `GovernanceOrigin` 审核裁决
5. **信用联动**：与现有信用体系深度集成
6. **分级惩罚**：不同违规类型对应不同惩罚力度
7. **永久封禁**：严重违规（毒品、诈骗）触发永久封禁
