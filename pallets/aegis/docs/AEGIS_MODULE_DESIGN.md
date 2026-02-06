# Cosmos Aegis 模块设计方案

> 链上 AI 自我进化防御系统 — Blockchain + AI Self-Evolving Guardian

## 目录

1. [设计理念](#1-设计理念)
2. [核心架构](#2-核心架构)
3. [Pallet 子模块详细设计](#3-pallet-子模块详细设计)
4. [AI 自我进化机制](#4-ai-自我进化机制)
5. [与现有模块集成方案](#5-与现有模块集成方案)
6. [经济模型](#6-经济模型)
7. [实施路线图](#7-实施路线图)

---

## 1. 设计理念

### 1.1 模块定位

**Aegis**（宙斯神盾）是 Cosmos 平台的链上 AI 自我进化防御系统。核心理念：

> **链上行为感知 → AI 分析推理 → 自适应策略调整 → 链上自动执行 → 反馈学习进化**

传统安全方案是静态规则，攻击者一旦摸清规则即可绑架；Aegis 通过 AI 持续学习链上行为模式，**自主发现异常、自主进化防御策略**，实现"越攻越强"的活防御体系。

### 1.2 三大核心原则

| 原则 | 说明 |
|------|------|
| **自感知** | 链上行为数据实时采集，构建全链态势感知图谱 |
| **自进化** | AI 模型持续训练迭代，防御策略随威胁变化自动升级 |
| **自执行** | 检测到威胁后自动触发链上响应，无需人工干预 |

### 1.3 与传统安全模块的区别

| 维度 | 传统安全模块 | Aegis |
|------|------------|-------|
| **规则来源** | 人工编写静态规则 | AI 从链上数据自动学习 |
| **应对新攻击** | 需人工分析 + 升级 | 自动检测异常模式并适应 |
| **参数调优** | 管理员手动调整 | AI 根据反馈自动优化阈值 |
| **响应速度** | 人工介入，小时级 | 自动执行，区块级（6秒） |
| **进化能力** | 无 | 持续学习，防御策略越来越精准 |

---

## 2. 核心架构

### 2.1 架构概览

```
┌──────────────────────────────────────────────────────────────────────┐
│                       Cosmos Aegis 架构                               │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   链上层 (Substrate Pallets)                                          │
│   ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────────┐     │
│   │   aegis-   │ │   aegis-   │ │   aegis-   │ │    aegis-    │     │
│   │   sensor   │ │   brain    │ │   shield   │ │    oracle    │     │
│   │  行为感知   │ │  AI 决策   │ │  防御执行   │ │  模型预言机   │     │
│   └─────┬──────┘ └─────┬──────┘ └─────┬──────┘ └──────┬───────┘     │
│         │              │              │               │              │
│   ┌─────▼──────────────▼──────────────▼───────────────▼──────┐      │
│   │                     aegis-common                          │      │
│   │                  公共类型 & 评分引擎                         │      │
│   └───────────────────────┬──────────────────────────────────┘      │
│                           │                                          │
├───────────────────────────┼──────────────────────────────────────────┤
│   链下层 (Off-chain)       │                                          │
│                           ▼                                          │
│   ┌──────────────────────────────────────────────────────────────┐   │
│   │                  AI Evolution Engine                          │   │
│   │  ┌──────────┐ ┌──────────────┐ ┌──────────┐ ┌────────────┐  │   │
│   │  │ 异常检测  │ │ 行为聚类分析  │ │ 策略优化  │ │ 模型训练   │  │   │
│   │  │ Engine   │ │ Engine       │ │ Engine   │ │ Pipeline   │  │   │
│   │  └──────────┘ └──────────────┘ └──────────┘ └────────────┘  │   │
│   └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│   ┌──────────────────────────────────────────────────────────────┐   │
│   │               Privacy Compute (TEE/MPC)                      │   │
│   │          AI 模型推理在隐私环境中执行，保护用户数据              │   │
│   └──────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘
```

### 2.2 数据流

```
                    ┌─── 自我进化循环 ───┐
                    │                    │
                    ▼                    │
┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
│  sensor   │──▶│  oracle   │──▶│  brain    │──▶│  shield  │
│  行为采集  │   │ AI 推理   │   │ 决策引擎  │   │ 防御执行  │
└──────────┘   └──────────┘   └──────────┘   └──────────┘
     ▲                                             │
     │              反馈回路                         │
     └─────────────────────────────────────────────┘
                  效果评估 → 模型更新
```

**闭环流程：**
1. **sensor** 实时采集链上交易、调用、状态变更等行为数据
2. **oracle** 将链下 AI 模型的推理结果（风险评分、异常标记）提交到链上
3. **brain** 基于 AI 评分 + 链上规则，做出防御决策（拦截/限流/告警/放行）
4. **shield** 执行防御动作（冻结账户、限制调用频率、触发多签审批等）
5. **反馈回路**：shield 执行结果回传到 sensor，AI 模型根据"误报/漏报"持续优化

### 2.3 子模块划分

| 子模块 | Crate 名称 | 路径 | 职责 |
|--------|-----------|------|------|
| **common** | `pallet-aegis-common` | `pallets/aegis/common/` | 公共类型、Trait、风险评分框架、进化指标 |
| **sensor** | `pallet-aegis-sensor` | `pallets/aegis/sensor/` | 链上行为采集、特征提取、行为画像 |
| **oracle** | `pallet-aegis-oracle` | `pallets/aegis/oracle/` | AI 模型注册、推理结果上链、模型版本管理 |
| **brain** | `pallet-aegis-brain` | `pallets/aegis/brain/` | 决策引擎、规则融合、自适应阈值、进化策略 |
| **shield** | `pallet-aegis-shield` | `pallets/aegis/shield/` | 防御动作执行、账户冻结、限流、应急响应 |

---

## 3. Pallet 子模块详细设计

### 3.1 aegis-common

公共基础模块，定义 Aegis 全局共享类型和评分框架。

```rust
// pallets/aegis/common/src/lib.rs

/// 风险等级
pub enum RiskLevel {
    Safe,       // 安全 (0-20 分)
    Low,        // 低风险 (21-40 分)
    Medium,     // 中风险 (41-60 分)
    High,       // 高风险 (61-80 分)
    Critical,   // 严重 (81-100 分)
}

/// 行为类别
pub enum BehaviorCategory {
    Transaction,       // 交易行为
    Governance,        // 治理行为
    Social,            // 社交行为
    Trading,           // 交易撮合
    Membership,        // 会员操作
    SystemCall,        // 系统调用
}

/// 行为事件（sensor 采集的原始数据）
pub struct BehaviorEvent<AccountId, BlockNumber> {
    pub actor: AccountId,
    pub category: BehaviorCategory,
    pub action_hash: [u8; 32],       // 行为指纹
    pub magnitude: u128,             // 行为量级（如交易金额）
    pub block_number: BlockNumber,
    pub metadata_cid: Option<BoundedVec<u8, ConstU32<64>>>, // IPFS CID
}

/// AI 风险评分
pub struct RiskScore {
    pub score: u8,                    // 0-100 综合评分
    pub risk_level: RiskLevel,
    pub confidence: u8,               // AI 置信度 0-100
    pub factors: BoundedVec<RiskFactor, ConstU32<10>>, // 风险因子
}

/// 风险因子
pub struct RiskFactor {
    pub factor_type: FactorType,
    pub weight: u8,                   // 权重 0-100
    pub value: u32,                   // 因子值
}

/// 风险因子类型
pub enum FactorType {
    FrequencyAnomaly,     // 频率异常（短时间大量操作）
    AmountAnomaly,        // 金额异常（偏离历史均值）
    PatternAnomaly,       // 模式异常（行为序列异常）
    NetworkAnomaly,       // 网络异常（关联账户异常聚集）
    TimeAnomaly,          // 时间异常（异常时段活跃）
    NewAccountRisk,       // 新账户风险（无历史数据）
    ReputationDrop,       // 信誉骤降
    SybilIndicator,       // 女巫攻击指标
}

/// 进化指标（衡量 AI 系统自身的进化程度）
pub struct EvolutionMetrics {
    pub model_version: u32,           // 当前模型版本
    pub accuracy: u16,                // 准确率 (basis points, 9500 = 95.00%)
    pub false_positive_rate: u16,     // 误报率
    pub false_negative_rate: u16,     // 漏报率
    pub total_predictions: u64,       // 累计预测次数
    pub total_correct: u64,           // 累计正确次数
    pub last_evolved_at: u32,         // 上次进化区块
    pub evolution_count: u32,         // 累计进化次数
}

/// 防御动作
pub enum DefenseAction {
    Allow,                            // 放行
    Alert { level: RiskLevel },       // 告警
    RateLimit { max_per_hour: u32 },  // 限流
    RequireMultiSig,                  // 要求多签确认
    Freeze { duration_blocks: u32 },  // 临时冻结
    Quarantine,                       // 隔离（限制部分功能）
    EmergencyHalt,                    // 紧急停止（需治理恢复）
}

/// AI 模型元数据
pub struct ModelMetadata {
    pub version: u32,
    pub model_hash: [u8; 32],        // 模型文件哈希
    pub model_cid: BoundedVec<u8, ConstU32<64>>, // IPFS CID
    pub training_data_hash: [u8; 32],// 训练数据指纹
    pub accuracy_score: u16,         // 验证集准确率
    pub published_at: u32,
}

/// Trait: 行为感知提供者（供其他 pallet 上报行为）
pub trait BehaviorReporter<AccountId> {
    fn report_behavior(actor: &AccountId, category: BehaviorCategory, magnitude: u128);
}

/// Trait: 风险评估提供者（供其他 pallet 查询风险）
pub trait RiskAssessor<AccountId> {
    fn get_risk_score(who: &AccountId) -> RiskScore;
    fn is_safe(who: &AccountId) -> bool;
    fn get_risk_level(who: &AccountId) -> RiskLevel;
}

/// Trait: 防御执行者（供 brain 调用）
pub trait DefenseExecutor<AccountId> {
    fn execute_action(target: &AccountId, action: DefenseAction) -> DispatchResult;
}
```

### 3.2 aegis-sensor — 行为感知层

链上行为数据的实时采集与特征提取，是 AI 进化的"眼睛"。

**核心职责：**
- 接收各 pallet 上报的行为事件
- 维护账户行为画像（滑动窗口统计）
- 提取行为特征向量（供 AI 模型输入）
- 检测明显的统计异常（阈值触发，不依赖 AI）

```rust
// pallets/aegis/sensor/src/lib.rs

#[pallet::storage]
/// 账户行为画像（滑动窗口统计）
pub type BehaviorProfile<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId, AccountBehaviorProfile<T>
>;

#[pallet::storage]
/// 最近行为事件缓冲区（环形缓冲，供 AI 分析）
pub type RecentEvents<T> = StorageValue<
    _, BoundedVec<BehaviorEvent<T::AccountId, BlockNumberFor<T>>, ConstU32<1000>>
>;

#[pallet::storage]
/// 行为特征向量（提取后的结构化特征，供 oracle 读取）
pub type FeatureVectors<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId, FeatureVector
>;

#[pallet::storage]
/// 全局行为统计（基线数据，用于异常检测）
pub type GlobalBaseline<T> = StorageValue<_, GlobalBehaviorBaseline>;

/// 账户行为画像
pub struct AccountBehaviorProfile<T: Config> {
    pub first_seen: BlockNumberFor<T>,        // 首次活跃区块
    pub total_actions: u64,                   // 总行为次数
    pub last_active: BlockNumberFor<T>,       // 最后活跃区块
    // 滑动窗口统计（最近 N 个区块）
    pub recent_tx_count: u32,                 // 近期交易数
    pub recent_tx_volume: u128,               // 近期交易总额
    pub recent_call_count: u32,               // 近期调用数
    pub avg_tx_amount: u128,                  // 平均交易金额
    pub max_tx_amount: u128,                  // 最大单笔交易
    // 行为多样性
    pub category_distribution: [u16; 6],      // 各类别行为占比
    // 关联网络
    pub unique_counterparties: u32,           // 独立交互对手数
    pub interaction_entropy: u16,             // 交互熵（衡量行为分散程度）
}

/// 行为特征向量（16维，作为 AI 模型输入）
pub struct FeatureVector {
    pub features: [u32; 16],
    pub extracted_at: u32,
}

/// 全局行为基线（所有用户的统计均值，用于检测偏离）
pub struct GlobalBehaviorBaseline {
    pub avg_daily_tx_count: u32,
    pub avg_daily_tx_volume: u128,
    pub avg_unique_counterparties: u32,
    pub total_active_accounts: u64,
    pub updated_at: u32,
}

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 上报行为事件（由其他 pallet 内部调用或通过 Trait 调用）
    #[pallet::call_index(0)]
    pub fn report_event(
        origin: OriginFor<T>,
        category: BehaviorCategory,
        magnitude: u128,
        metadata_cid: Option<BoundedVec<u8, ConstU32<64>>>,
    ) -> DispatchResult;

    /// 更新全局基线（定期由 off-chain worker 或管理员触发）
    #[pallet::call_index(1)]
    pub fn update_baseline(
        origin: OriginFor<T>,
    ) -> DispatchResult;
}

#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    /// 每个区块末尾：
    /// 1. 清理过期事件
    /// 2. 更新滑动窗口统计
    /// 3. 提取特征向量
    fn on_finalize(n: BlockNumberFor<T>) {
        Self::cleanup_expired_events(n);
        Self::update_sliding_windows(n);
        Self::extract_features(n);
    }
}
```

### 3.3 aegis-oracle — AI 模型预言机

将链下 AI 推理结果可信地带入链上，是链上与链下 AI 的桥梁。

**核心职责：**
- AI 模型注册、版本管理、哈希验证
- 接收链下 AI 推理结果（风险评分）并上链
- 多 oracle 交叉验证（防止单点篡改）
- 模型进化：新版本通过治理投票上线

```rust
// pallets/aegis/oracle/src/lib.rs

#[pallet::storage]
/// 已注册 AI 模型
pub type Models<T> = StorageMap<_, Blake2_128Concat, u32, ModelMetadata>;

#[pallet::storage]
/// 当前活跃模型版本
pub type ActiveModelVersion<T> = StorageValue<_, u32>;

#[pallet::storage]
/// Oracle 节点注册表
pub type Oracles<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId, OracleInfo<T>
>;

#[pallet::storage]
/// 账户风险评分（由 oracle 提交）
pub type RiskScores<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId, AggregatedRiskScore
>;

#[pallet::storage]
/// 待聚合的评分提交（多 oracle 提交后聚合）
pub type PendingScores<T> = StorageDoubleMap<
    _,
    Blake2_128Concat, T::AccountId,    // 被评估账户
    Blake2_128Concat, T::AccountId,    // oracle 节点
    RiskScore,
>;

#[pallet::storage]
/// 进化指标
pub type Evolution<T> = StorageValue<_, EvolutionMetrics>;

/// Oracle 节点信息
pub struct OracleInfo<T: Config> {
    pub owner: T::AccountId,
    pub stake: BalanceOf<T>,                // 质押金额
    pub model_version: u32,                 // 运行的模型版本
    pub accuracy_record: u16,               // 历史准确率
    pub total_submissions: u64,
    pub slashed_count: u32,
    pub registered_at: BlockNumberFor<T>,
}

/// 聚合后的风险评分
pub struct AggregatedRiskScore {
    pub score: u8,                          // 聚合评分 (中位数)
    pub risk_level: RiskLevel,
    pub confidence: u8,                     // 聚合置信度
    pub oracle_count: u8,                   // 参与评分的 oracle 数
    pub consensus: bool,                    // 是否达成共识
    pub updated_at: u32,
}

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 注册 oracle 节点（需质押）
    #[pallet::call_index(0)]
    pub fn register_oracle(
        origin: OriginFor<T>,
        stake: BalanceOf<T>,
    ) -> DispatchResult;

    /// 提交 AI 风险评分
    #[pallet::call_index(1)]
    pub fn submit_risk_score(
        origin: OriginFor<T>,
        target: T::AccountId,
        score: RiskScore,
        model_version: u32,
    ) -> DispatchResult;

    /// 注册新 AI 模型版本（需治理审批）
    #[pallet::call_index(2)]
    pub fn register_model(
        origin: OriginFor<T>,
        model_hash: [u8; 32],
        model_cid: BoundedVec<u8, ConstU32<64>>,
        accuracy_score: u16,
    ) -> DispatchResult;

    /// 激活新模型版本（治理通过后）
    #[pallet::call_index(3)]
    pub fn activate_model(
        origin: OriginFor<T>,
        version: u32,
    ) -> DispatchResult;

    /// 上报模型进化反馈（实际结果 vs 预测）
    #[pallet::call_index(4)]
    pub fn report_feedback(
        origin: OriginFor<T>,
        target: T::AccountId,
        was_correct: bool,
    ) -> DispatchResult;

    /// 触发模型进化（链下训练完成后，提交新模型）
    #[pallet::call_index(5)]
    pub fn propose_evolution(
        origin: OriginFor<T>,
        new_model_hash: [u8; 32],
        new_model_cid: BoundedVec<u8, ConstU32<64>>,
        new_accuracy: u16,
        training_data_hash: [u8; 32],
    ) -> DispatchResult;
}
```

### 3.4 aegis-brain — 决策引擎

融合 AI 评分与链上规则，做出最终防御决策。是整个系统的"大脑"。

**核心职责：**
- 融合 AI 风险评分与静态规则
- 自适应阈值管理（阈值随进化自动调整）
- 决策生成（放行/告警/限流/冻结）
- 决策审计（所有决策可追溯、可申诉）

```rust
// pallets/aegis/brain/src/lib.rs

#[pallet::storage]
/// 自适应阈值配置
pub type Thresholds<T> = StorageValue<_, AdaptiveThresholds>;

#[pallet::storage]
/// 防御策略规则集
pub type PolicyRules<T> = StorageValue<
    _, BoundedVec<PolicyRule, ConstU32<50>>
>;

#[pallet::storage]
/// 决策记录（审计追溯）
pub type DecisionLog<T> = StorageMap<
    _, Blake2_128Concat, u64, Decision<T>
>;

#[pallet::storage]
/// 下一个决策 ID
pub type NextDecisionId<T> = StorageValue<_, u64>;

#[pallet::storage]
/// 账户防御状态
pub type AccountDefenseState<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId, DefenseState<T>
>;

#[pallet::storage]
/// 申诉记录
pub type Appeals<T> = StorageMap<
    _, Blake2_128Concat, u64, Appeal<T>
>;

/// 自适应阈值（会随 AI 进化自动调整）
pub struct AdaptiveThresholds {
    pub alert_threshold: u8,          // 告警阈值 (默认 40)
    pub rate_limit_threshold: u8,     // 限流阈值 (默认 60)
    pub freeze_threshold: u8,         // 冻结阈值 (默认 80)
    pub emergency_threshold: u8,      // 紧急阈值 (默认 95)
    pub min_confidence: u8,           // 最低置信度要求 (默认 70)
    pub auto_adjust: bool,            // 是否开启自适应调整
    pub last_adjusted: u32,
    pub adjustment_count: u32,
}

/// 策略规则
pub struct PolicyRule {
    pub id: u32,
    pub category: BehaviorCategory,   // 适用的行为类别
    pub condition: RuleCondition,     // 触发条件
    pub action: DefenseAction,        // 防御动作
    pub priority: u8,                 // 优先级
    pub enabled: bool,
    pub is_ai_generated: bool,        // 是否由 AI 生成
}

/// 规则条件
pub enum RuleCondition {
    ScoreAbove(u8),                   // 评分超过阈值
    FrequencyAbove(u32),              // 频率超过阈值（次/小时）
    AmountAbove(u128),                // 金额超过阈值
    PatternMatch(u32),                // 匹配特定行为模式 ID
    Composite(Vec<RuleCondition>),    // 组合条件（AND）
}

/// 决策记录
pub struct Decision<T: Config> {
    pub id: u64,
    pub target: T::AccountId,
    pub risk_score: RiskScore,
    pub action: DefenseAction,
    pub rule_id: Option<u32>,         // 触发的规则
    pub ai_driven: bool,              // 是否由 AI 驱动
    pub block_number: BlockNumberFor<T>,
    pub appealed: bool,
    pub overturned: bool,             // 是否被申诉推翻
}

/// 防御状态
pub struct DefenseState<T: Config> {
    pub current_action: DefenseAction,
    pub applied_at: BlockNumberFor<T>,
    pub expires_at: Option<BlockNumberFor<T>>,
    pub escalation_count: u32,        // 连续升级次数
}

/// 申诉
pub struct Appeal<T: Config> {
    pub decision_id: u64,
    pub appellant: T::AccountId,
    pub reason_cid: BoundedVec<u8, ConstU32<64>>,
    pub submitted_at: BlockNumberFor<T>,
    pub status: AppealStatus,
}

pub enum AppealStatus {
    Pending,
    Approved,      // 申诉成功，推翻决策
    Rejected,
}

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 请求风险评估并做出决策
    #[pallet::call_index(0)]
    pub fn evaluate_and_decide(
        origin: OriginFor<T>,
        target: T::AccountId,
    ) -> DispatchResult;

    /// 添加策略规则（AdminOrigin）
    #[pallet::call_index(1)]
    pub fn add_policy_rule(
        origin: OriginFor<T>,
        rule: PolicyRule,
    ) -> DispatchResult;

    /// AI 提议新规则（通过 oracle 提交，需治理审批）
    #[pallet::call_index(2)]
    pub fn propose_ai_rule(
        origin: OriginFor<T>,
        rule: PolicyRule,
        justification_cid: BoundedVec<u8, ConstU32<64>>,
    ) -> DispatchResult;

    /// 手动调整阈值（AdminOrigin）
    #[pallet::call_index(3)]
    pub fn set_thresholds(
        origin: OriginFor<T>,
        thresholds: AdaptiveThresholds,
    ) -> DispatchResult;

    /// AI 自适应调整阈值（由进化引擎触发）
    #[pallet::call_index(4)]
    pub fn auto_adjust_thresholds(
        origin: OriginFor<T>,
        new_thresholds: AdaptiveThresholds,
        evolution_proof: EvolutionProof,
    ) -> DispatchResult;

    /// 用户申诉
    #[pallet::call_index(10)]
    pub fn submit_appeal(
        origin: OriginFor<T>,
        decision_id: u64,
        reason_cid: BoundedVec<u8, ConstU32<64>>,
    ) -> DispatchResult;

    /// 审批申诉（AdminOrigin 或治理）
    #[pallet::call_index(11)]
    pub fn resolve_appeal(
        origin: OriginFor<T>,
        appeal_id: u64,
        approved: bool,
    ) -> DispatchResult;
}
```

### 3.5 aegis-shield — 防御执行层

将 brain 的决策转化为实际的链上防御动作。

**核心职责：**
- 执行账户冻结/解冻
- 调用频率限制
- 多签审批强制
- 应急响应（紧急停止特定 pallet 功能）
- 与其他 pallet 联动（如 trading 暂停、membership 冻结）

```rust
// pallets/aegis/shield/src/lib.rs

#[pallet::storage]
/// 冻结账户列表
pub type FrozenAccounts<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId, FreezeInfo<T>
>;

#[pallet::storage]
/// 频率限制表
pub type RateLimits<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId, RateLimitInfo<T>
>;

#[pallet::storage]
/// 隔离区账户
pub type QuarantinedAccounts<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId, QuarantineInfo<T>
>;

#[pallet::storage]
/// 紧急停止的模块列表
pub type HaltedModules<T> = StorageValue<
    _, BoundedVec<BoundedVec<u8, ConstU32<32>>, ConstU32<20>>
>;

#[pallet::storage]
/// 防御动作执行历史
pub type ActionHistory<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId,
    BoundedVec<ActionRecord<T>, ConstU32<100>>
>;

/// 冻结信息
pub struct FreezeInfo<T: Config> {
    pub frozen_at: BlockNumberFor<T>,
    pub expires_at: Option<BlockNumberFor<T>>,
    pub reason: DefenseAction,
    pub decision_id: u64,
}

/// 频率限制信息
pub struct RateLimitInfo<T: Config> {
    pub max_calls_per_hour: u32,
    pub current_count: u32,
    pub window_start: BlockNumberFor<T>,
    pub applied_at: BlockNumberFor<T>,
}

/// 隔离信息
pub struct QuarantineInfo<T: Config> {
    pub restricted_pallets: BoundedVec<BoundedVec<u8, ConstU32<32>>, ConstU32<10>>,
    pub applied_at: BlockNumberFor<T>,
    pub decision_id: u64,
}

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 冻结账户（由 brain 调用或 AdminOrigin）
    #[pallet::call_index(0)]
    pub fn freeze_account(
        origin: OriginFor<T>,
        target: T::AccountId,
        duration_blocks: Option<u32>,
        decision_id: u64,
    ) -> DispatchResult;

    /// 解冻账户
    #[pallet::call_index(1)]
    pub fn unfreeze_account(
        origin: OriginFor<T>,
        target: T::AccountId,
    ) -> DispatchResult;

    /// 设置频率限制
    #[pallet::call_index(2)]
    pub fn set_rate_limit(
        origin: OriginFor<T>,
        target: T::AccountId,
        max_per_hour: u32,
    ) -> DispatchResult;

    /// 隔离账户（限制部分功能）
    #[pallet::call_index(3)]
    pub fn quarantine_account(
        origin: OriginFor<T>,
        target: T::AccountId,
        restricted_pallets: Vec<Vec<u8>>,
    ) -> DispatchResult;

    /// 紧急停止模块
    #[pallet::call_index(10)]
    pub fn emergency_halt_module(
        origin: OriginFor<T>,
        module_name: BoundedVec<u8, ConstU32<32>>,
        reason_cid: BoundedVec<u8, ConstU32<64>>,
    ) -> DispatchResult;

    /// 恢复模块
    #[pallet::call_index(11)]
    pub fn resume_module(
        origin: OriginFor<T>,
        module_name: BoundedVec<u8, ConstU32<32>>,
    ) -> DispatchResult;
}

#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_initialize(n: BlockNumberFor<T>) -> Weight {
        // 自动解冻过期冻结
        Self::auto_unfreeze(n);
        // 重置频率限制窗口
        Self::reset_rate_limit_windows(n);
        Weight::zero()
    }
}
```

---

## 4. AI 自我进化机制

### 4.1 进化循环

Aegis 的核心创新在于 **AI 模型的持续自我进化**，不是一次训练永久使用，而是形成闭环反馈持续提升。

```
┌────────────────────────────────────────────────────────────────────┐
│                      AI 自我进化循环                                 │
│                                                                    │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    │
│   │ 1. 采集   │───▶│ 2. 推理   │───▶│ 3. 决策   │───▶│ 4. 执行   │    │
│   │ sensor   │    │ oracle   │    │ brain    │    │ shield   │    │
│   └──────────┘    └──────────┘    └──────────┘    └──────────┘    │
│        ▲                                               │           │
│        │          ┌──────────────────────────┐         │           │
│        │          │    5. 反馈评估             │         │           │
│        │          │  - 决策是否正确？          │◀────────┘           │
│        │          │  - 误报？漏报？            │                     │
│        │          │  - 用户申诉结果            │                     │
│        │          └────────────┬─────────────┘                     │
│        │                       │                                    │
│        │          ┌────────────▼─────────────┐                     │
│        │          │    6. 链下模型训练          │                     │
│        │          │  - 用反馈数据重新训练       │                     │
│        │          │  - 生成新版本模型          │                     │
│        │          │  - 验证集测试             │                     │
│        │          └────────────┬─────────────┘                     │
│        │                       │                                    │
│        │          ┌────────────▼─────────────┐                     │
│        │          │    7. 链上治理审批          │                     │
│        │          │  - 提交新模型哈希          │                     │
│        │          │  - 社区投票通过            │                     │
│        │          │  - 激活新版本             │                     │
│        │          └────────────┬─────────────┘                     │
│        │                       │                                    │
│        └───────────────────────┘                                    │
│              新模型上线，进入下一轮循环                                │
└────────────────────────────────────────────────────────────────────┘
```

### 4.2 进化触发条件

| 触发条件 | 阈值 | 说明 |
|----------|------|------|
| **误报率上升** | > 5% | 太多正常用户被误判，需优化模型 |
| **漏报率上升** | > 3% | 有攻击未被检测，需增强模型 |
| **新攻击模式** | 未知模式出现 3 次 | 发现模型不认识的新型行为 |
| **定期进化** | 每 30 天 | 无论指标如何，定期用新数据重训练 |
| **重大事件后** | 人工触发 | 发生安全事件后紧急进化 |

### 4.3 自适应阈值调整算法

```
输入：近 7 天的决策反馈数据
      当前阈值 T = {alert, rate_limit, freeze, emergency}
      误报率 FPR, 漏报率 FNR

算法：
  if FPR > 5%:
      # 误报太多，放宽阈值
      T.alert     += 2  (最高 60)
      T.rate_limit += 2  (最高 80)
      T.freeze     += 2  (最高 95)

  if FNR > 3%:
      # 漏报太多，收紧阈值
      T.alert     -= 3  (最低 20)
      T.rate_limit -= 3  (最低 40)
      T.freeze     -= 3  (最低 60)

  if FPR < 1% and FNR < 1%:
      # 模型很准，保持当前阈值
      pass

输出：更新后的阈值 T'
约束：alert < rate_limit < freeze < emergency
      每次调整幅度不超过 5 分
      需 evolution_proof 包含反馈数据的 Merkle root
```

### 4.4 模型版本治理

为防止恶意模型上线，AI 模型更新必须通过链上治理：

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ Oracle 提交   │────▶│  影子运行期   │────▶│  治理投票     │
│ 新模型版本    │     │ (7 天并行对比) │     │ (技术委员会)  │
└──────────────┘     └──────────────┘     └──────────────┘
                                                │
                            ┌───────────────────┼────────────────┐
                            ▼                                    ▼
                     ┌──────────────┐                     ┌──────────────┐
                     │  投票通过     │                     │  投票否决     │
                     │  激活新模型   │                     │  保留旧模型   │
                     └──────────────┘                     └──────────────┘
```

**影子运行期**：新模型并行运行 7 天，不执行任何防御动作，仅记录预测结果。对比新旧模型准确率后，社区决定是否切换。

### 4.5 进化等级

Aegis 系统自身有"进化等级"，反映系统成熟度：

| 等级 | 名称 | 条件 | 能力 |
|------|------|------|------|
| Lv.1 | **觉醒** | 初始部署 | 仅统计规则检测，无 AI |
| Lv.2 | **感知** | 累计 10,000 行为事件 | AI 基础异常检测上线 |
| Lv.3 | **洞察** | 准确率 > 85%，进化 3 次 | 自适应阈值启用 |
| Lv.4 | **预判** | 准确率 > 90%，进化 10 次 | AI 可自主提议新规则 |
| Lv.5 | **先知** | 准确率 > 95%，进化 30 次 | 全自动防御 + 预测性拦截 |

每个等级解锁更多自主权限，从纯辅助到全自动，**渐进式信任建设**。

---

## 5. 与现有模块集成方案

### 5.1 Trading 模块集成（交易风控）

```
用户发起交易        aegis-sensor              aegis-brain           交易执行
     │                  │                        │                    │
     ├── swap/otc ──────▶ report_event()         │                    │
     │                  ├── 更新行为画像           │                    │
     │                  ├── 提取特征 ─────────────▶│                    │
     │                  │                        ├── 查询 risk_score  │
     │                  │                        ├── 匹配规则          │
     │                  │                        ├── 生成决策          │
     │                  │                        │                    │
     │                  │                 [Safe]  ├──── Allow ────────▶│ 正常执行
     │                  │                 [High]  ├──── RateLimit ────▶│ 限流
     │                  │              [Critical] ├──── Freeze ───────▶│ 拒绝 + 冻结
```

**集成点：**
- `pallet-trading-swap` 和 `pallet-trading-otc` 在执行前调用 `RiskAssessor::is_safe()`
- 大额交易自动触发 `evaluate_and_decide()`
- 异常交易模式（如 wash trading）由 AI 自动识别

### 5.2 Affiliate 模块集成（反刷单）

**集成点：**
- sensor 监控推荐关系建立频率和模式
- AI 检测 Sybil 攻击（一人控制多账户刷推荐奖励）
- 异常推荐链模式自动识别（如环形推荐网络）
- 刷单账户自动冻结推荐奖励

### 5.3 Membership 模块集成（会员风控）

**集成点：**
- 新会员注册时建立初始行为画像
- 异常批量注册检测
- 会员等级升降与行为信誉关联

### 5.4 Chat 模块集成（内容安全）

**集成点：**
- 消息频率异常检测（垃圾消息轰炸）
- 群组行为异常检测（恶意拉群）
- 与 `pallet-privacy-compute` 协同：在 TEE 中分析消息内容，不泄露隐私

### 5.5 Governance 模块集成（治理攻击防护）

**集成点：**
- 检测治理攻击模式（闪电贷投票、提案垃圾攻击）
- 投票行为异常检测（短时间大量投票、复制投票模式）
- 保护 `pallet-affiliate::governance` 的投票公正性

### 5.6 集成方式汇总

```rust
// 其他 pallet 集成 Aegis 的方式

// 方式 1：Config Trait 注入
#[pallet::config]
pub trait Config: frame_system::Config {
    /// 行为上报接口
    type BehaviorReporter: aegis_common::BehaviorReporter<Self::AccountId>;
    /// 风险评估接口
    type RiskAssessor: aegis_common::RiskAssessor<Self::AccountId>;
}

// 方式 2：交易前置检查
impl<T: Config> Pallet<T> {
    fn pre_check(who: &T::AccountId) -> DispatchResult {
        ensure!(
            T::RiskAssessor::is_safe(who),
            Error::<T>::AccountRestricted
        );
        // 上报行为
        T::BehaviorReporter::report_behavior(who, BehaviorCategory::Trading, amount);
        Ok(())
    }
}
```

---

## 6. 经济模型

### 6.1 Oracle 激励

| 行为 | 奖惩 |
|------|------|
| 提交准确评分 | +10 COS / 次 |
| 评分被采纳（进入聚合） | +5 COS / 次 |
| 评分偏离共识（可能恶意） | -50 COS（Slash 质押） |
| 发现新攻击模式 | +500 COS 奖励 |
| 提交有效模型进化 | +1,000 COS 奖励 |

### 6.2 质押要求

| 角色 | 最低质押 | 说明 |
|------|---------|------|
| Oracle 节点 | 10,000 COS | 保证评分诚实 |
| 模型提交者 | 5,000 COS | 保证模型质量 |

### 6.3 误判补偿

被误判的用户（申诉成功），从 Oracle 的质押中补偿：
- 临时冻结误判：补偿 100 COS
- 限流误判：补偿 50 COS
- 误判 Oracle 被 Slash 等额

---

## 7. 实施路线图

### Phase 1：基础感知（4 周）

| 任务 | 优先级 | 依赖 |
|------|--------|------|
| 创建 `pallets/aegis/common/` 公共类型和 Trait 定义 | P0 | 无 |
| 实现 `pallets/aegis/sensor/` 行为采集和画像 | P0 | common |
| 实现 `pallets/aegis/shield/` 基础防御动作（冻结/限流） | P0 | common |
| Trading 模块集成行为上报 | P0 | sensor |
| 单元测试 | P0 | 上述全部 |

**里程碑：** 链上行为数据可采集，基于统计规则的基础风控可用（Lv.1 觉醒）。

### Phase 2：AI 上链（4-6 周）

| 任务 | 优先级 | 依赖 |
|------|--------|------|
| 实现 `pallets/aegis/oracle/` 模型注册和评分提交 | P1 | common |
| 实现 `pallets/aegis/brain/` 决策引擎和规则融合 | P1 | common, oracle |
| 开发链下 AI 异常检测模型（Isolation Forest / Autoencoder） | P1 | sensor 数据 |
| Oracle 质押和激励机制 | P1 | oracle |
| Affiliate + Membership 模块集成 | P1 | sensor, brain |

**里程碑：** AI 风险评分上链，自动风控决策可用（Lv.2 感知）。

### Phase 3：自我进化（4-6 周）

| 任务 | 优先级 | 依赖 |
|------|--------|------|
| 实现反馈回路（决策结果 → 模型训练数据） | P2 | brain, shield |
| 自适应阈值调整算法 | P2 | brain |
| 模型影子运行和治理投票机制 | P2 | oracle |
| 进化等级系统 | P2 | oracle |
| 用户申诉流程 | P2 | brain, shield |

**里程碑：** AI 可自我进化，阈值自适应调整，达到 Lv.3 洞察。

### Phase 4：高级能力（持续迭代）

| 任务 | 优先级 | 依赖 |
|------|--------|------|
| AI 自主提议防御规则 | P3 | brain 进化到 Lv.4 |
| 预测性拦截（在攻击发生前预警） | P3 | brain 进化到 Lv.5 |
| 与 `pallet-privacy-compute` 集成（TEE 中运行 AI） | P3 | privacy 模块就绪 |
| 跨模块联合防御（多 pallet 协同响应） | P3 | 全部子模块 |
| 图神经网络检测关联账户网络 | P3 | 足够的链上数据 |

---

## 附录 A：目录结构

```
pallets/aegis/
├── README.md
├── docs/
│   └── AEGIS_MODULE_DESIGN.md   # 本文档
├── common/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs               # 公共类型、Trait、风险评分框架
├── sensor/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs               # Pallet 定义
│       ├── profile.rs           # 行为画像引擎
│       ├── features.rs          # 特征提取
│       ├── weights.rs
│       └── tests.rs
├── oracle/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs               # Pallet 定义
│       ├── aggregator.rs        # 评分聚合
│       ├── evolution.rs         # 进化管理
│       ├── weights.rs
│       └── tests.rs
├── brain/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs               # Pallet 定义
│       ├── rules.rs             # 规则引擎
│       ├── threshold.rs         # 自适应阈值
│       ├── appeal.rs            # 申诉处理
│       ├── weights.rs
│       └── tests.rs
└── shield/
    ├── Cargo.toml
    └── src/
        ├── lib.rs               # Pallet 定义
        ├── freeze.rs            # 冻结/解冻
        ├── limiter.rs           # 频率限制
        ├── emergency.rs         # 应急响应
        ├── weights.rs
        └── tests.rs
```

## 附录 B：事件定义汇总

| 模块 | 事件 | 字段 | 说明 |
|------|------|------|------|
| sensor | `BehaviorRecorded` | actor, category, magnitude | 行为已记录 |
| sensor | `AnomalyDetected` | actor, anomaly_type, severity | 统计异常检测到 |
| sensor | `BaselineUpdated` | block_number | 全局基线已更新 |
| oracle | `RiskScoreSubmitted` | oracle, target, score | 风险评分已提交 |
| oracle | `ScoreAggregated` | target, final_score, oracle_count | 评分已聚合 |
| oracle | `ModelRegistered` | version, model_hash | 新模型已注册 |
| oracle | `ModelActivated` | version | 新模型已激活 |
| oracle | `EvolutionTriggered` | old_version, new_version, accuracy_delta | 进化已触发 |
| brain | `DecisionMade` | target, action, risk_score, ai_driven | 防御决策已生成 |
| brain | `ThresholdAdjusted` | old, new, reason | 阈值已自适应调整 |
| brain | `AiRuleProposed` | rule_id, justification_cid | AI 提议新规则 |
| brain | `AppealSubmitted` | decision_id, appellant | 申诉已提交 |
| brain | `AppealResolved` | appeal_id, approved | 申诉已处理 |
| shield | `AccountFrozen` | target, duration, decision_id | 账户已冻结 |
| shield | `AccountUnfrozen` | target | 账户已解冻 |
| shield | `RateLimitApplied` | target, max_per_hour | 限流已生效 |
| shield | `AccountQuarantined` | target, restricted_pallets | 账户已隔离 |
| shield | `ModuleHalted` | module_name, reason_cid | 模块已紧急停止 |
| shield | `ModuleResumed` | module_name | 模块已恢复 |

## 附录 C：错误定义汇总

| 模块 | 错误 | 说明 |
|------|------|------|
| sensor | `EventBufferFull` | 事件缓冲区已满 |
| sensor | `InvalidCategory` | 无效行为类别 |
| oracle | `NotRegisteredOracle` | 非注册 Oracle 节点 |
| oracle | `InsufficientStake` | 质押不足 |
| oracle | `ModelVersionMismatch` | 模型版本不匹配 |
| oracle | `ModelAlreadyActive` | 模型已是活跃版本 |
| oracle | `EvolutionCooldown` | 进化冷却期内 |
| brain | `AccountAlreadyFrozen` | 账户已被冻结 |
| brain | `NoActiveRiskScore` | 无有效风险评分 |
| brain | `RuleLimitExceeded` | 规则数超限 |
| brain | `DecisionNotFound` | 决策记录不存在 |
| brain | `AppealAlreadySubmitted` | 已提交申诉 |
| brain | `AppealWindowClosed` | 申诉窗口已关闭 |
| shield | `AccountNotFrozen` | 账户未被冻结 |
| shield | `ModuleNotHalted` | 模块未被停止 |
| shield | `UnauthorizedAction` | 无权执行此操作 |

## 附录 D：AI 模型技术选型

| 阶段 | 模型 | 用途 | 部署方式 |
|------|------|------|----------|
| Phase 1 | 统计规则 (Z-Score, IQR) | 基础异常检测 | 链上直接计算 |
| Phase 2 | Isolation Forest | 无监督异常检测 | 链下推理 → Oracle 上链 |
| Phase 2 | Autoencoder | 行为模式重建误差检测 | 链下推理 → Oracle 上链 |
| Phase 3 | LightGBM | 有监督分类（正常/异常） | 链下推理 → Oracle 上链 |
| Phase 4 | GNN (图神经网络) | 关联账户网络分析 | TEE 内推理 |
| Phase 4 | Transformer | 行为序列预测 | TEE 内推理 |
