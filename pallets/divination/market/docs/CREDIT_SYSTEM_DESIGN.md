# 服务提供者信用体系设计方案

## 1. 概述

### 1.1 当前实现分析

现有系统已有基础信用机制：
- **等级制度** (`ProviderTier`): Novice → Certified → Senior → Expert → Master
- **晋升条件**: 订单数 + 平均评分
- **评分维度**: 总体评分、准确度、服务态度、响应速度
- **等级权益**: 平台抽成递减（20% → 8%）

**不足之处**：
1. 只能升级不能降级，缺乏动态调整
2. 无惩罚机制，违规成本低
3. 评分维度单一，缺乏行为信用
4. 无信用分数，难以精细化运营
5. 缺乏信用修复机制

### 1.2 设计目标

构建一个**多维度、动态调整、奖惩分明**的信用体系：

1. **综合信用分** - 量化信用状态
2. **多维度评估** - 服务质量 + 行为规范 + 履约能力
3. **动态调整** - 支持升降级
4. **奖惩机制** - 激励优质服务，惩罚违规行为
5. **信用修复** - 提供改正机会
6. **透明公开** - 信用信息可查询

---

## 2. 信用分数模型

### 2.1 信用分构成

```
总信用分 = 基础分 + 服务质量分 + 行为规范分 + 履约能力分 + 加分项 - 扣分项
```

**分数范围**: 0 - 1000 分

| 分数区间 | 信用等级 | 状态描述 |
|---------|---------|---------|
| 900-1000 | 卓越 (Excellent) | 顶级信用，享受最高权益 |
| 750-899 | 优秀 (Good) | 信用良好，正常服务 |
| 600-749 | 一般 (Fair) | 信用一般，需要改进 |
| 400-599 | 警示 (Warning) | 信用警示，限制部分功能 |
| 200-399 | 不良 (Poor) | 信用不良，严重限制 |
| 0-199 | 失信 (Bad) | 失信状态，暂停服务 |

### 2.2 分数计算公式

```rust
/// 信用分计算参数
pub struct CreditScoreParams {
    /// 基础分（新注册提供者起始分）
    pub base_score: u16,                    // 默认 500

    /// 服务质量权重（基点）
    pub service_quality_weight: u16,        // 默认 3500 (35%)

    /// 行为规范权重（基点）
    pub behavior_weight: u16,               // 默认 2500 (25%)

    /// 履约能力权重（基点）
    pub fulfillment_weight: u16,            // 默认 3000 (30%)

    /// 加分项权重（基点）
    pub bonus_weight: u16,                  // 默认 1000 (10%)
}
```

---

## 3. 数据结构设计

### 3.1 核心类型定义

```rust
/// 信用等级
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum CreditLevel {
    /// 失信 (0-199)
    Bad = 0,
    /// 不良 (200-399)
    Poor = 1,
    /// 警示 (400-599)
    Warning = 2,
    /// 一般 (600-749)
    #[default]
    Fair = 3,
    /// 优秀 (750-899)
    Good = 4,
    /// 卓越 (900-1000)
    Excellent = 5,
}

impl CreditLevel {
    /// 根据分数获取等级
    pub fn from_score(score: u16) -> Self {
        match score {
            0..=199 => CreditLevel::Bad,
            200..=399 => CreditLevel::Poor,
            400..=599 => CreditLevel::Warning,
            600..=749 => CreditLevel::Fair,
            750..=899 => CreditLevel::Good,
            900..=1000 => CreditLevel::Excellent,
            _ => CreditLevel::Excellent,
        }
    }

    /// 获取等级对应的功能限制
    pub fn get_restrictions(&self) -> CreditRestrictions {
        match self {
            CreditLevel::Bad => CreditRestrictions {
                can_accept_orders: false,
                can_create_packages: false,
                can_answer_bounties: false,
                max_active_orders: 0,
                withdrawal_delay_blocks: 0, // 禁止提现
                platform_fee_modifier: 0,   // 不适用
                visibility_penalty: 10000,  // 100% 隐藏
            },
            CreditLevel::Poor => CreditRestrictions {
                can_accept_orders: true,
                can_create_packages: false,
                can_answer_bounties: false,
                max_active_orders: 1,
                withdrawal_delay_blocks: 14400 * 7, // 7天延迟
                platform_fee_modifier: 3000,        // +30%
                visibility_penalty: 5000,           // 50% 降权
            },
            CreditLevel::Warning => CreditRestrictions {
                can_accept_orders: true,
                can_create_packages: true,
                can_answer_bounties: false,
                max_active_orders: 3,
                withdrawal_delay_blocks: 14400 * 3, // 3天延迟
                platform_fee_modifier: 1500,        // +15%
                visibility_penalty: 2000,           // 20% 降权
            },
            CreditLevel::Fair => CreditRestrictions {
                can_accept_orders: true,
                can_create_packages: true,
                can_answer_bounties: true,
                max_active_orders: 5,
                withdrawal_delay_blocks: 14400,     // 1天延迟
                platform_fee_modifier: 0,           // 无额外费用
                visibility_penalty: 0,              // 无降权
            },
            CreditLevel::Good => CreditRestrictions {
                can_accept_orders: true,
                can_create_packages: true,
                can_answer_bounties: true,
                max_active_orders: 10,
                withdrawal_delay_blocks: 0,         // 即时提现
                platform_fee_modifier: -500,        // -5% 优惠
                visibility_penalty: 0,
            },
            CreditLevel::Excellent => CreditRestrictions {
                can_accept_orders: true,
                can_create_packages: true,
                can_answer_bounties: true,
                max_active_orders: 20,
                withdrawal_delay_blocks: 0,
                platform_fee_modifier: -1000,       // -10% 优惠
                visibility_penalty: 0,
            },
        }
    }
}

/// 信用限制配置
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct CreditRestrictions {
    /// 是否可以接单
    pub can_accept_orders: bool,
    /// 是否可以创建套餐
    pub can_create_packages: bool,
    /// 是否可以回答悬赏
    pub can_answer_bounties: bool,
    /// 最大同时进行订单数
    pub max_active_orders: u8,
    /// 提现延迟（区块数）
    pub withdrawal_delay_blocks: u32,
    /// 平台费用调整（基点，正数增加，负数减少）
    pub platform_fee_modifier: i16,
    /// 搜索展示降权（基点，10000=完全隐藏）
    pub visibility_penalty: u16,
}

/// 服务提供者信用档案
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct CreditProfile<BlockNumber> {
    /// 当前信用分
    pub score: u16,

    /// 当前信用等级
    pub level: CreditLevel,

    /// 历史最高分
    pub highest_score: u16,

    /// 历史最低分
    pub lowest_score: u16,

    // ========== 服务质量维度 ==========

    /// 服务质量分（0-350）
    pub service_quality_score: u16,

    /// 平均综合评分（*100，如 450 = 4.5星）
    pub avg_overall_rating: u16,

    /// 平均准确度评分
    pub avg_accuracy_rating: u16,

    /// 平均服务态度评分
    pub avg_attitude_rating: u16,

    /// 平均响应速度评分
    pub avg_response_rating: u16,

    /// 5星好评数
    pub five_star_count: u32,

    /// 1星差评数
    pub one_star_count: u32,

    // ========== 行为规范维度 ==========

    /// 行为规范分（0-250）
    pub behavior_score: u16,

    /// 累计违规次数
    pub violation_count: u32,

    /// 累计警告次数
    pub warning_count: u32,

    /// 累计投诉次数
    pub complaint_count: u32,

    /// 投诉成立次数
    pub complaint_upheld_count: u32,

    /// 当前活跃违规数（未过期）
    pub active_violations: u8,

    // ========== 履约能力维度 ==========

    /// 履约能力分（0-300）
    pub fulfillment_score: u16,

    /// 订单完成率（基点）
    pub completion_rate: u16,

    /// 按时完成率（基点）
    pub on_time_rate: u16,

    /// 取消率（基点）
    pub cancellation_rate: u16,

    /// 超时次数
    pub timeout_count: u32,

    /// 主动取消次数
    pub active_cancel_count: u32,

    /// 平均响应时间（区块数）
    pub avg_response_blocks: u32,

    // ========== 加分项 ==========

    /// 加分项总分（0-100）
    pub bonus_score: u16,

    /// 悬赏被采纳次数
    pub bounty_adoption_count: u32,

    /// 获得认证数
    pub certification_count: u8,

    /// 连续好评天数
    pub consecutive_positive_days: u16,

    /// 是否通过实名认证
    pub is_verified: bool,

    /// 是否缴纳保证金
    pub has_deposit: bool,

    // ========== 扣分记录 ==========

    /// 累计扣分
    pub total_deductions: u16,

    /// 最近一次扣分原因
    pub last_deduction_reason: Option<DeductionReason>,

    /// 最近一次扣分时间
    pub last_deduction_at: Option<BlockNumber>,

    // ========== 时间戳 ==========

    /// 信用档案创建时间
    pub created_at: BlockNumber,

    /// 最近更新时间
    pub updated_at: BlockNumber,

    /// 最近评估时间
    pub last_evaluated_at: BlockNumber,
}

/// 扣分原因
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum DeductionReason {
    /// 差评扣分
    NegativeReview = 0,
    /// 订单取消
    OrderCancellation = 1,
    /// 订单超时
    OrderTimeout = 2,
    /// 客户投诉成立
    ComplaintUpheld = 3,
    /// 违规行为
    Violation = 4,
    /// 虚假宣传
    FalseAdvertising = 5,
    /// 服务欺诈
    Fraud = 6,
    /// 辱骂客户
    Abuse = 7,
    /// 泄露隐私
    PrivacyBreach = 8,
    /// 其他
    Other = 9,
}

impl DeductionReason {
    /// 获取默认扣分值
    pub fn default_deduction(&self) -> u16 {
        match self {
            DeductionReason::NegativeReview => 5,
            DeductionReason::OrderCancellation => 10,
            DeductionReason::OrderTimeout => 15,
            DeductionReason::ComplaintUpheld => 30,
            DeductionReason::Violation => 50,
            DeductionReason::FalseAdvertising => 80,
            DeductionReason::Fraud => 200,
            DeductionReason::Abuse => 100,
            DeductionReason::PrivacyBreach => 150,
            DeductionReason::Other => 20,
        }
    }
}

/// 违规记录
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxReasonLen))]
pub struct ViolationRecord<AccountId, BlockNumber, MaxReasonLen: Get<u32>> {
    /// 记录 ID
    pub id: u64,

    /// 提供者账户
    pub provider: AccountId,

    /// 违规类型
    pub violation_type: ViolationType,

    /// 违规原因描述
    pub reason: BoundedVec<u8, MaxReasonLen>,

    /// 关联订单 ID（如有）
    pub related_order_id: Option<u64>,

    /// 扣分数值
    pub deduction_points: u16,

    /// 处罚措施
    pub penalty: PenaltyType,

    /// 处罚期限（区块数，0表示永久）
    pub penalty_duration: u32,

    /// 是否已申诉
    pub is_appealed: bool,

    /// 申诉结果
    pub appeal_result: Option<AppealResult>,

    /// 记录时间
    pub recorded_at: BlockNumber,

    /// 过期时间（信用恢复点）
    pub expires_at: Option<BlockNumber>,

    /// 是否活跃（未过期）
    pub is_active: bool,
}

/// 违规类型
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum ViolationType {
    /// 轻微违规
    Minor = 0,
    /// 一般违规
    Moderate = 1,
    /// 严重违规
    Severe = 2,
    /// 特别严重违规
    Critical = 3,
}

impl ViolationType {
    /// 获取违规等级对应的惩罚系数
    pub fn penalty_multiplier(&self) -> u16 {
        match self {
            ViolationType::Minor => 100,      // 1x
            ViolationType::Moderate => 200,   // 2x
            ViolationType::Severe => 500,     // 5x
            ViolationType::Critical => 1000,  // 10x
        }
    }

    /// 获取违规记录有效期（区块数）
    pub fn record_duration(&self) -> u32 {
        match self {
            ViolationType::Minor => 14400 * 30,      // 30天
            ViolationType::Moderate => 14400 * 90,   // 90天
            ViolationType::Severe => 14400 * 180,    // 180天
            ViolationType::Critical => 14400 * 365,  // 1年
        }
    }
}

/// 处罚类型
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum PenaltyType {
    /// 仅扣分
    #[default]
    DeductionOnly = 0,
    /// 警告
    Warning = 1,
    /// 限制接单
    OrderRestriction = 2,
    /// 暂停服务
    ServiceSuspension = 3,
    /// 永久封禁
    PermanentBan = 4,
}

/// 申诉结果
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum AppealResult {
    /// 申诉成功，撤销处罚
    Upheld = 0,
    /// 申诉部分成功，减轻处罚
    PartiallyUpheld = 1,
    /// 申诉失败
    Rejected = 2,
}

/// 信用变更记录
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxReasonLen))]
pub struct CreditChangeRecord<BlockNumber, MaxReasonLen: Get<u32>> {
    /// 变更前分数
    pub previous_score: u16,

    /// 变更后分数
    pub new_score: u16,

    /// 变更值（正数加分，负数扣分）
    pub change_amount: i16,

    /// 变更原因
    pub reason: CreditChangeReason,

    /// 详细说明
    pub description: Option<BoundedVec<u8, MaxReasonLen>>,

    /// 关联 ID（订单/违规记录等）
    pub related_id: Option<u64>,

    /// 变更时间
    pub changed_at: BlockNumber,
}

/// 信用变更原因
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum CreditChangeReason {
    /// 好评加分
    PositiveReview = 0,
    /// 差评扣分
    NegativeReview = 1,
    /// 完成订单
    OrderCompleted = 2,
    /// 取消订单
    OrderCancelled = 3,
    /// 超时未响应
    ResponseTimeout = 4,
    /// 悬赏被采纳
    BountyAdopted = 5,
    /// 获得认证
    CertificationGained = 6,
    /// 违规处罚
    ViolationPenalty = 7,
    /// 申诉成功恢复
    AppealRestored = 8,
    /// 信用修复
    CreditRepair = 9,
    /// 定期评估调整
    PeriodicAdjustment = 10,
    /// 系统奖励
    SystemBonus = 11,
    /// 连续好评奖励
    ConsecutiveBonus = 12,
}

/// 信用修复任务
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct CreditRepairTask<BlockNumber> {
    /// 任务 ID
    pub id: u32,

    /// 任务类型
    pub task_type: RepairTaskType,

    /// 完成后恢复的分数
    pub reward_points: u16,

    /// 任务目标值
    pub target_value: u32,

    /// 当前进度
    pub current_progress: u32,

    /// 是否已完成
    pub is_completed: bool,

    /// 任务开始时间
    pub started_at: BlockNumber,

    /// 任务截止时间
    pub deadline: BlockNumber,

    /// 完成时间
    pub completed_at: Option<BlockNumber>,
}

/// 信用修复任务类型
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum RepairTaskType {
    /// 完成 N 个订单
    CompleteOrders = 0,
    /// 获得 N 个好评
    GetPositiveReviews = 1,
    /// 连续 N 天无投诉
    NoComplaintDays = 2,
    /// 缴纳额外保证金
    ExtraDeposit = 3,
    /// 完成培训课程
    CompleteTraining = 4,
    /// 通过认证考试
    PassCertification = 5,
}

impl RepairTaskType {
    /// 获取任务的默认奖励分数
    pub fn default_reward(&self) -> u16 {
        match self {
            RepairTaskType::CompleteOrders => 20,
            RepairTaskType::GetPositiveReviews => 30,
            RepairTaskType::NoComplaintDays => 25,
            RepairTaskType::ExtraDeposit => 50,
            RepairTaskType::CompleteTraining => 40,
            RepairTaskType::PassCertification => 60,
        }
    }
}
```

### 3.2 存储项设计

```rust
/// 提供者信用档案
#[pallet::storage]
#[pallet::getter(fn credit_profiles)]
pub type CreditProfiles<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    CreditProfile<BlockNumberFor<T>>,
>;

/// 违规记录存储
#[pallet::storage]
#[pallet::getter(fn violation_records)]
pub type ViolationRecords<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,  // violation_id
    ViolationRecord<T::AccountId, BlockNumberFor<T>, T::MaxDescriptionLength>,
>;

/// 提供者违规记录索引
#[pallet::storage]
#[pallet::getter(fn provider_violations)]
pub type ProviderViolations<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<u64, ConstU32<100>>,  // violation_ids
    ValueQuery,
>;

/// 下一个违规记录 ID
#[pallet::storage]
#[pallet::getter(fn next_violation_id)]
pub type NextViolationId<T> = StorageValue<_, u64, ValueQuery>;

/// 信用变更历史（最近 N 条）
#[pallet::storage]
#[pallet::getter(fn credit_history)]
pub type CreditHistory<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<CreditChangeRecord<BlockNumberFor<T>, ConstU32<256>>, ConstU32<50>>,
    ValueQuery,
>;

/// 信用修复任务
#[pallet::storage]
#[pallet::getter(fn repair_tasks)]
pub type RepairTasks<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<CreditRepairTask<BlockNumberFor<T>>, ConstU32<5>>,
    ValueQuery,
>;

/// 信用黑名单（永久封禁）
#[pallet::storage]
#[pallet::getter(fn credit_blacklist)]
pub type CreditBlacklist<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BlockNumberFor<T>,  // 封禁时间
>;

/// 全局信用统计
#[pallet::storage]
#[pallet::getter(fn credit_stats)]
pub type CreditStatistics<T: Config> = StorageValue<_, GlobalCreditStats, ValueQuery>;

/// 全局信用统计
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct GlobalCreditStats {
    /// 总提供者数
    pub total_providers: u32,
    /// 各等级分布
    pub excellent_count: u32,
    pub good_count: u32,
    pub fair_count: u32,
    pub warning_count: u32,
    pub poor_count: u32,
    pub bad_count: u32,
    /// 黑名单数量
    pub blacklisted_count: u32,
    /// 平均信用分
    pub average_score: u16,
    /// 本周新增违规数
    pub weekly_violations: u32,
}
```

---

## 4. 信用分计算逻辑

### 4.1 服务质量分计算（满分 350 分）

```rust
impl<T: Config> Pallet<T> {
    /// 计算服务质量分
    fn calculate_service_quality_score(profile: &CreditProfile<BlockNumberFor<T>>) -> u16 {
        let mut score: u32 = 0;

        // 1. 综合评分贡献（最高 150 分）
        // avg_overall_rating 范围: 0-500 (对应 0-5 星)
        // 4.5星及以上得满分
        let rating_score = if profile.avg_overall_rating >= 450 {
            150
        } else if profile.avg_overall_rating >= 400 {
            130
        } else if profile.avg_overall_rating >= 350 {
            100
        } else if profile.avg_overall_rating >= 300 {
            70
        } else {
            (profile.avg_overall_rating as u32 * 70) / 300
        };
        score += rating_score;

        // 2. 评分一致性（最高 50 分）
        // 各维度评分差异小说明稳定
        let rating_variance = Self::calculate_rating_variance(profile);
        let consistency_score = if rating_variance < 20 {
            50
        } else if rating_variance < 50 {
            35
        } else if rating_variance < 100 {
            20
        } else {
            10
        };
        score += consistency_score;

        // 3. 好评率（最高 100 分）
        // five_star_count / total_ratings
        let total = profile.five_star_count + profile.one_star_count;
        if total > 0 {
            let positive_rate = (profile.five_star_count as u32 * 10000) / total as u32;
            let rate_score = if positive_rate >= 9500 {
                100
            } else if positive_rate >= 9000 {
                85
            } else if positive_rate >= 8000 {
                70
            } else if positive_rate >= 7000 {
                50
            } else {
                (positive_rate * 50) / 7000
            };
            score += rate_score;
        }

        // 4. 差评惩罚（最多扣 50 分）
        let penalty = (profile.one_star_count as u32 * 5).min(50);
        score = score.saturating_sub(penalty);

        score.min(350) as u16
    }

    /// 计算评分方差
    fn calculate_rating_variance(profile: &CreditProfile<BlockNumberFor<T>>) -> u16 {
        let ratings = [
            profile.avg_overall_rating,
            profile.avg_accuracy_rating,
            profile.avg_attitude_rating,
            profile.avg_response_rating,
        ];

        let avg: u32 = ratings.iter().map(|&r| r as u32).sum::<u32>() / 4;
        let variance: u32 = ratings.iter()
            .map(|&r| {
                let diff = if r as u32 > avg { r as u32 - avg } else { avg - r as u32 };
                diff * diff
            })
            .sum::<u32>() / 4;

        (variance as f64).sqrt() as u16
    }
}
```

### 4.2 行为规范分计算（满分 250 分）

```rust
/// 计算行为规范分
fn calculate_behavior_score(profile: &CreditProfile<BlockNumberFor<T>>) -> u16 {
    let mut score: u32 = 250; // 起始满分，扣分制

    // 1. 违规扣分（每次违规扣 20-100 分）
    let violation_deduction = match profile.violation_count {
        0 => 0,
        1 => 20,
        2..=3 => 50,
        4..=5 => 100,
        _ => 150,
    };
    score = score.saturating_sub(violation_deduction);

    // 2. 投诉扣分（每次成立投诉扣 15 分）
    let complaint_deduction = (profile.complaint_upheld_count as u32 * 15).min(80);
    score = score.saturating_sub(complaint_deduction);

    // 3. 活跃违规额外扣分
    let active_deduction = (profile.active_violations as u32 * 30).min(60);
    score = score.saturating_sub(active_deduction);

    // 4. 警告扣分
    let warning_deduction = (profile.warning_count as u32 * 5).min(30);
    score = score.saturating_sub(warning_deduction);

    score.min(250) as u16
}
```

### 4.3 履约能力分计算（满分 300 分）

```rust
/// 计算履约能力分
fn calculate_fulfillment_score(profile: &CreditProfile<BlockNumberFor<T>>) -> u16 {
    let mut score: u32 = 0;

    // 1. 完成率（最高 120 分）
    // completion_rate 范围: 0-10000
    let completion_score = if profile.completion_rate >= 9800 {
        120
    } else if profile.completion_rate >= 9500 {
        100
    } else if profile.completion_rate >= 9000 {
        80
    } else if profile.completion_rate >= 8000 {
        60
    } else {
        (profile.completion_rate as u32 * 60) / 8000
    };
    score += completion_score;

    // 2. 按时完成率（最高 80 分）
    let ontime_score = if profile.on_time_rate >= 9500 {
        80
    } else if profile.on_time_rate >= 9000 {
        65
    } else if profile.on_time_rate >= 8000 {
        50
    } else {
        (profile.on_time_rate as u32 * 50) / 8000
    };
    score += ontime_score;

    // 3. 响应速度（最高 60 分）
    // avg_response_blocks: 越少越好
    let response_score = if profile.avg_response_blocks == 0 {
        30 // 新用户默认
    } else if profile.avg_response_blocks <= 600 { // ~1小时
        60
    } else if profile.avg_response_blocks <= 1800 { // ~3小时
        50
    } else if profile.avg_response_blocks <= 7200 { // ~12小时
        35
    } else if profile.avg_response_blocks <= 14400 { // ~24小时
        20
    } else {
        10
    };
    score += response_score;

    // 4. 取消率惩罚（最多扣 40 分）
    let cancel_penalty = if profile.cancellation_rate <= 200 { // 2%以下
        0
    } else if profile.cancellation_rate <= 500 {
        15
    } else if profile.cancellation_rate <= 1000 {
        30
    } else {
        40
    };
    score = score.saturating_sub(cancel_penalty);

    score.min(300) as u16
}
```

### 4.4 加分项计算（满分 100 分）

```rust
/// 计算加分项
fn calculate_bonus_score(profile: &CreditProfile<BlockNumberFor<T>>) -> u16 {
    let mut score: u32 = 0;

    // 1. 实名认证（+15 分）
    if profile.is_verified {
        score += 15;
    }

    // 2. 保证金（+10 分）
    if profile.has_deposit {
        score += 10;
    }

    // 3. 资质认证（每个 +8 分，最多 40 分）
    let cert_bonus = (profile.certification_count as u32 * 8).min(40);
    score += cert_bonus;

    // 4. 悬赏被采纳（每次 +2 分，最多 20 分）
    let bounty_bonus = (profile.bounty_adoption_count as u32 * 2).min(20);
    score += bounty_bonus;

    // 5. 连续好评天数（每 7 天 +3 分，最多 15 分）
    let consecutive_bonus = ((profile.consecutive_positive_days as u32 / 7) * 3).min(15);
    score += consecutive_bonus;

    score.min(100) as u16
}
```

### 4.5 综合分数计算

```rust
/// 计算综合信用分
pub fn calculate_total_credit_score(
    profile: &CreditProfile<BlockNumberFor<T>>
) -> u16 {
    // 基础分 500
    let base_score: u32 = 500;

    // 各维度得分
    let service_quality = Self::calculate_service_quality_score(profile) as u32;
    let behavior = Self::calculate_behavior_score(profile) as u32;
    let fulfillment = Self::calculate_fulfillment_score(profile) as u32;
    let bonus = Self::calculate_bonus_score(profile) as u32;

    // 加权计算
    // 总分 = (服务质量 * 0.35) + (行为规范 * 0.25) + (履约能力 * 0.30) + (加分项 * 0.10)
    // 为避免浮点，使用基点: 3500 + 2500 + 3000 + 1000 = 10000

    let weighted_service = (service_quality * 3500) / 350;  // 转换为 0-350 范围
    let weighted_behavior = (behavior * 2500) / 250;
    let weighted_fulfillment = (fulfillment * 3000) / 300;
    let weighted_bonus = (bonus * 1000) / 100;

    let weighted_total = weighted_service + weighted_behavior + weighted_fulfillment + weighted_bonus;

    // 映射到 0-500 范围（加上基础分最高 1000）
    let additional = (weighted_total * 500) / 10000;

    let total = base_score + additional;

    // 扣除累计扣分
    let final_score = total.saturating_sub(profile.total_deductions as u32);

    final_score.min(1000) as u16
}
```

---

## 5. 外部调用函数

### 5.1 信用评估与更新

```rust
/// 触发信用评估（定期调用或事件触发）
#[pallet::call_index(40)]
#[pallet::weight(Weight::from_parts(50_000_000, 0))]
pub fn evaluate_credit(
    origin: OriginFor<T>,
    provider: T::AccountId,
) -> DispatchResult {
    ensure_signed(origin)?;

    ensure!(
        Providers::<T>::contains_key(&provider),
        Error::<T>::ProviderNotFound
    );

    let current_block = <frame_system::Pallet<T>>::block_number();

    CreditProfiles::<T>::try_mutate(&provider, |maybe_profile| {
        let profile = maybe_profile.as_mut()
            .ok_or(Error::<T>::CreditProfileNotFound)?;

        let previous_score = profile.score;

        // 重新计算各维度分数
        profile.service_quality_score = Self::calculate_service_quality_score(profile);
        profile.behavior_score = Self::calculate_behavior_score(profile);
        profile.fulfillment_score = Self::calculate_fulfillment_score(profile);
        profile.bonus_score = Self::calculate_bonus_score(profile);

        // 计算总分
        let new_score = Self::calculate_total_credit_score(profile);
        let new_level = CreditLevel::from_score(new_score);

        // 更新最高/最低分记录
        if new_score > profile.highest_score {
            profile.highest_score = new_score;
        }
        if new_score < profile.lowest_score {
            profile.lowest_score = new_score;
        }

        profile.score = new_score;
        profile.level = new_level;
        profile.last_evaluated_at = current_block;
        profile.updated_at = current_block;

        // 记录变更历史
        if previous_score != new_score {
            Self::record_credit_change(
                &provider,
                previous_score,
                new_score,
                CreditChangeReason::PeriodicAdjustment,
                None,
                None,
            )?;
        }

        Ok::<_, DispatchError>(())
    })?;

    Self::deposit_event(Event::CreditEvaluated {
        provider: provider.clone(),
    });

    Ok(())
}
```

### 5.2 违规记录管理

```rust
/// 记录违规（治理权限）
#[pallet::call_index(41)]
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
    let base_deduction = match &penalty {
        PenaltyType::DeductionOnly => 20,
        PenaltyType::Warning => 30,
        PenaltyType::OrderRestriction => 50,
        PenaltyType::ServiceSuspension => 100,
        PenaltyType::PermanentBan => 500,
    };
    let deduction_points = (base_deduction * violation_type.penalty_multiplier() / 100) as u16;

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
    CreditProfiles::<T>::try_mutate(&provider, |maybe_profile| {
        let profile = maybe_profile.as_mut()
            .ok_or(Error::<T>::CreditProfileNotFound)?;

        let previous_score = profile.score;

        profile.violation_count += 1;
        profile.active_violations += 1;
        profile.total_deductions += deduction_points;
        profile.last_deduction_reason = Some(DeductionReason::Violation);
        profile.last_deduction_at = Some(current_block);

        // 重新计算分数
        profile.score = Self::calculate_total_credit_score(profile);
        profile.level = CreditLevel::from_score(profile.score);
        profile.updated_at = current_block;

        // 记录变更
        Self::record_credit_change(
            &provider,
            previous_score,
            profile.score,
            CreditChangeReason::ViolationPenalty,
            None,
            Some(violation_id),
        )?;

        Ok::<_, DispatchError>(())
    })?;

    // 处理永久封禁
    if penalty == PenaltyType::PermanentBan {
        CreditBlacklist::<T>::insert(&provider, current_block);

        // 更新提供者状态
        Providers::<T>::mutate(&provider, |maybe_p| {
            if let Some(p) = maybe_p {
                p.status = ProviderStatus::Banned;
            }
        });
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
#[pallet::call_index(42)]
#[pallet::weight(Weight::from_parts(30_000_000, 0))]
pub fn appeal_violation(
    origin: OriginFor<T>,
    violation_id: u64,
    appeal_reason: Vec<u8>,
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
#[pallet::call_index(43)]
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
        CreditProfiles::<T>::try_mutate(&provider, |maybe_profile| {
            let profile = maybe_profile.as_mut()
                .ok_or(Error::<T>::CreditProfileNotFound)?;

            let previous_score = profile.score;

            profile.total_deductions = profile.total_deductions.saturating_sub(points_to_restore);

            if result == AppealResult::Upheld {
                profile.violation_count = profile.violation_count.saturating_sub(1);
                profile.active_violations = profile.active_violations.saturating_sub(1);
            }

            profile.score = Self::calculate_total_credit_score(profile);
            profile.level = CreditLevel::from_score(profile.score);
            profile.updated_at = <frame_system::Pallet<T>>::block_number();

            Self::record_credit_change(
                &provider,
                previous_score,
                profile.score,
                CreditChangeReason::AppealRestored,
                None,
                Some(violation_id),
            )?;

            Ok::<_, DispatchError>(())
        })?;
    }

    Self::deposit_event(Event::AppealResolved {
        provider,
        violation_id,
        result,
        restored_points: points_to_restore,
    });

    Ok(())
}
```

### 5.3 信用修复

```rust
/// 申请信用修复任务
#[pallet::call_index(44)]
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

    // 检查任务数量上限
    ensure!(
        tasks.iter().filter(|t| !t.is_completed).count() < 3,
        Error::<T>::TooManyActiveTasks
    );

    let current_block = <frame_system::Pallet<T>>::block_number();

    // 根据任务类型设置目标和期限
    let (target_value, duration_blocks) = match task_type {
        RepairTaskType::CompleteOrders => (5, 14400 * 30),      // 30天完成5单
        RepairTaskType::GetPositiveReviews => (3, 14400 * 30),  // 30天获得3个好评
        RepairTaskType::NoComplaintDays => (14, 14400 * 14),    // 14天无投诉
        RepairTaskType::ExtraDeposit => (1, 14400 * 7),         // 7天内缴纳
        RepairTaskType::CompleteTraining => (1, 14400 * 14),    // 14天完成培训
        RepairTaskType::PassCertification => (1, 14400 * 30),   // 30天通过认证
    };

    let task_id = tasks.len() as u32;

    let task = CreditRepairTask {
        id: task_id,
        task_type,
        reward_points: task_type.default_reward(),
        target_value,
        current_progress: 0,
        is_completed: false,
        started_at: current_block,
        deadline: current_block + duration_blocks.into(),
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

/// 更新修复任务进度（内部调用）
fn update_repair_progress(
    provider: &T::AccountId,
    task_type: RepairTaskType,
    increment: u32,
) -> DispatchResult {
    RepairTasks::<T>::try_mutate(provider, |tasks| {
        for task in tasks.iter_mut() {
            if task.task_type == task_type && !task.is_completed {
                task.current_progress = task.current_progress.saturating_add(increment);

                if task.current_progress >= task.target_value {
                    task.is_completed = true;
                    task.completed_at = Some(<frame_system::Pallet<T>>::block_number());

                    // 恢复信用分
                    CreditProfiles::<T>::mutate(provider, |maybe_profile| {
                        if let Some(profile) = maybe_profile {
                            let previous_score = profile.score;
                            profile.total_deductions = profile.total_deductions
                                .saturating_sub(task.reward_points);
                            profile.score = Self::calculate_total_credit_score(profile);
                            profile.level = CreditLevel::from_score(profile.score);

                            Self::record_credit_change(
                                provider,
                                previous_score,
                                profile.score,
                                CreditChangeReason::CreditRepair,
                                None,
                                None,
                            ).ok();
                        }
                    });

                    Self::deposit_event(Event::CreditRepairCompleted {
                        provider: provider.clone(),
                        task_type,
                        restored_points: task.reward_points,
                    });
                }

                break;
            }
        }

        Ok::<_, DispatchError>(())
    })?;

    Ok(())
}
```

---

## 6. 信用联动机制

### 6.1 订单完成时更新信用

```rust
/// 在 submit_interpretation 函数中添加信用更新逻辑
fn on_order_completed(provider: &T::AccountId, order: &OrderOf<T>) -> DispatchResult {
    CreditProfiles::<T>::try_mutate(provider, |maybe_profile| {
        if let Some(profile) = maybe_profile {
            let current_block = <frame_system::Pallet<T>>::block_number();

            // 更新完成率
            let total_orders = profile.completion_rate; // 需要额外存储总订单数
            // ... 更新统计

            // 检查是否按时完成
            if let Some(accepted_at) = order.accepted_at {
                let expected_duration = 14400; // 默认24小时
                if current_block <= accepted_at + expected_duration.into() {
                    // 按时完成
                } else {
                    // 超时
                    profile.timeout_count += 1;
                }
            }

            profile.updated_at = current_block;
        }

        Ok::<_, DispatchError>(())
    })?;

    // 更新修复任务进度
    Self::update_repair_progress(provider, RepairTaskType::CompleteOrders, 1)?;

    Ok(())
}
```

### 6.2 评价提交时更新信用

```rust
/// 在 submit_review 函数中添加信用更新逻辑
fn on_review_submitted(
    provider: &T::AccountId,
    overall_rating: u8,
    accuracy_rating: u8,
    attitude_rating: u8,
    response_rating: u8,
) -> DispatchResult {
    CreditProfiles::<T>::try_mutate(provider, |maybe_profile| {
        if let Some(profile) = maybe_profile {
            let current_block = <frame_system::Pallet<T>>::block_number();
            let previous_score = profile.score;

            // 更新评分统计（滑动平均）
            let total_ratings = profile.five_star_count + profile.one_star_count + 1;

            profile.avg_overall_rating = Self::update_average(
                profile.avg_overall_rating,
                overall_rating as u16 * 100,
                total_ratings,
            );
            profile.avg_accuracy_rating = Self::update_average(
                profile.avg_accuracy_rating,
                accuracy_rating as u16 * 100,
                total_ratings,
            );
            profile.avg_attitude_rating = Self::update_average(
                profile.avg_attitude_rating,
                attitude_rating as u16 * 100,
                total_ratings,
            );
            profile.avg_response_rating = Self::update_average(
                profile.avg_response_rating,
                response_rating as u16 * 100,
                total_ratings,
            );

            // 更新好评/差评计数
            if overall_rating == 5 {
                profile.five_star_count += 1;
                profile.consecutive_positive_days += 1;

                // 更新修复任务
                Self::update_repair_progress(provider, RepairTaskType::GetPositiveReviews, 1).ok();
            } else if overall_rating == 1 {
                profile.one_star_count += 1;
                profile.consecutive_positive_days = 0;

                // 差评扣分
                profile.total_deductions += DeductionReason::NegativeReview.default_deduction();
                profile.last_deduction_reason = Some(DeductionReason::NegativeReview);
                profile.last_deduction_at = Some(current_block);
            }

            // 重新计算分数
            profile.score = Self::calculate_total_credit_score(profile);
            profile.level = CreditLevel::from_score(profile.score);
            profile.updated_at = current_block;

            // 记录变更
            let change_reason = if overall_rating >= 4 {
                CreditChangeReason::PositiveReview
            } else {
                CreditChangeReason::NegativeReview
            };

            Self::record_credit_change(
                provider,
                previous_score,
                profile.score,
                change_reason,
                None,
                None,
            ).ok();
        }

        Ok::<_, DispatchError>(())
    })?;

    Ok(())
}

/// 滑动平均计算
fn update_average(old_avg: u16, new_value: u16, total_count: u32) -> u16 {
    if total_count <= 1 {
        return new_value;
    }

    let old_sum = old_avg as u32 * (total_count - 1);
    let new_sum = old_sum + new_value as u32;
    (new_sum / total_count) as u16
}
```

### 6.3 悬赏采纳时更新信用

```rust
/// 在 adopt_bounty_answers 函数中添加信用更新逻辑
fn on_bounty_adopted(provider: &T::AccountId, rank: u8) -> DispatchResult {
    CreditProfiles::<T>::mutate(provider, |maybe_profile| {
        if let Some(profile) = maybe_profile {
            profile.bounty_adoption_count += 1;
            profile.updated_at = <frame_system::Pallet<T>>::block_number();

            // 根据名次给予信用奖励
            let bonus = match rank {
                1 => 10, // 第一名 +10 分
                2 => 5,  // 第二名 +5 分
                3 => 3,  // 第三名 +3 分
                _ => 1,  // 参与奖 +1 分
            };

            // 增加分数（通过减少扣分实现）
            profile.total_deductions = profile.total_deductions.saturating_sub(bonus);
            profile.score = Self::calculate_total_credit_score(profile);
            profile.level = CreditLevel::from_score(profile.score);
        }
    });

    Self::deposit_event(Event::CreditBonusAwarded {
        provider: provider.clone(),
        reason: CreditChangeReason::BountyAdopted,
    });

    Ok(())
}
```

---

## 7. 新增事件

```rust
/// 信用评估完成
CreditEvaluated { provider: T::AccountId },

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

/// 信用奖励发放
CreditBonusAwarded {
    provider: T::AccountId,
    reason: CreditChangeReason,
},

/// 信用等级变更
CreditLevelChanged {
    provider: T::AccountId,
    old_level: CreditLevel,
    new_level: CreditLevel,
},

/// 加入信用黑名单
AddedToBlacklist {
    provider: T::AccountId,
},
```

---

## 8. 新增错误类型

```rust
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
```

---

## 9. 信用体系权益对照表

| 信用等级 | 分数区间 | 接单 | 创建套餐 | 悬赏回答 | 最大在线订单 | 提现延迟 | 平台费调整 | 搜索权重 |
|---------|---------|------|---------|---------|-------------|---------|-----------|---------|
| 卓越 | 900-1000 | ✅ | ✅ | ✅ | 20 | 即时 | -10% | 优先展示 |
| 优秀 | 750-899 | ✅ | ✅ | ✅ | 10 | 即时 | -5% | 正常 |
| 一般 | 600-749 | ✅ | ✅ | ✅ | 5 | 1天 | 0 | 正常 |
| 警示 | 400-599 | ✅ | ✅ | ❌ | 3 | 3天 | +15% | -20% |
| 不良 | 200-399 | ✅ | ❌ | ❌ | 1 | 7天 | +30% | -50% |
| 失信 | 0-199 | ❌ | ❌ | ❌ | 0 | 禁止 | - | 隐藏 |

---

## 10. 实施计划

### 阶段一：数据结构（1-2天）
1. 定义信用相关类型（`types.rs`）
2. 添加存储项（`lib.rs`）
3. 添加配置常量

### 阶段二：核心算法（2-3天）
1. 实现分数计算函数
2. 实现等级判定逻辑
3. 实现限制检查逻辑

### 阶段三：外部函数（2-3天）
1. 实现违规记录管理
2. 实现申诉流程
3. 实现信用修复机制

### 阶段四：联动机制（2-3天）
1. 集成到订单流程
2. 集成到评价流程
3. 集成到悬赏流程

### 阶段五：测试与优化（2-3天）
1. 单元测试
2. 集成测试
3. 性能优化

### 阶段六：前端开发（3-5天）
1. 信用档案展示页
2. 违规记录管理
3. 信用修复任务界面

---

## 11. 总结

本方案构建了一个**多维度、动态调整、奖惩分明**的信用体系：

### 核心特点

1. **量化评估** - 0-1000 分制，直观反映信用状态
2. **多维度** - 服务质量(35%) + 行为规范(25%) + 履约能力(30%) + 加分项(10%)
3. **动态调整** - 支持升降级，实时反映服务状态
4. **奖惩分明** - 好评加分、违规扣分，激励正向行为
5. **修复机制** - 提供信用修复任务，给予改正机会
6. **权益联动** - 信用等级与平台权益挂钩

### 与现有系统整合

- 保留原有 `ProviderTier` 等级制度
- `CreditLevel` 作为信用评估的补充维度
- 两套体系并行，互相印证
