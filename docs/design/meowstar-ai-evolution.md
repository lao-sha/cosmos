# 喵星宇宙 (Meowstar Universe) - AI 智能体进化系统深度设计

## 1. 系统概述

AI 智能体是喵星宇宙中的核心创新，每个玩家可以培养专属的 AI 伙伴，它能够：
- 自动战斗决策
- 学习对手策略
- 进化提升能力
- 与其他 AI 融合

## 2. AI 智能体架构

```
┌─────────────────────────────────────────────────────────┐
│                   AI 智能体架构                          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │  感知模块   │  │  决策模块   │  │  学习模块   │    │
│  │ Perception │  │  Decision   │  │  Learning   │    │
│  └─────────────┘  └─────────────┘  └─────────────┘    │
│        │                │                │             │
│        v                v                v             │
│  ┌─────────────────────────────────────────────────┐  │
│  │              核心神经网络 (链下)                  │  │
│  │  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  │  │
│  │  │输入层│->│隐藏层│->│隐藏层│->│隐藏层│->│输出层│  │  │
│  │  └─────┘  └─────┘  └─────┘  └─────┘  └─────┘  │  │
│  └─────────────────────────────────────────────────┘  │
│                         │                              │
│                         v                              │
│  ┌─────────────────────────────────────────────────┐  │
│  │              链上状态存储                         │  │
│  │  能力值 | 进化阶段 | 训练记录 | 基因哈希         │  │
│  └─────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## 3. 核心数据结构

```rust
/// AI 智能体
pub struct AIAgent<T: Config> {
    /// 唯一ID
    pub id: u64,
    /// 所有者
    pub owner: T::AccountId,
    /// 名称
    pub name: BoundedVec<u8, ConstU32<32>>,
    /// 世代 (融合次数)
    pub generation: u32,
    /// 进化阶段
    pub evolution_stage: EvolutionStage,
    /// 能力值
    pub abilities: AIAbilities,
    /// 基因编码 (决定特性)
    pub genome: [u8; 32],
    /// 经验值
    pub experience: u64,
    /// 训练状态
    pub training_status: TrainingStatus,
    /// 绑定宠物
    pub bound_pet: Option<u64>,
    /// 创建区块
    pub created_at: BlockNumberFor<T>,
    /// 最后活跃区块
    pub last_active: BlockNumberFor<T>,
}

/// AI 能力值
pub struct AIAbilities {
    /// 战斗智能 - 影响战斗决策质量
    pub combat_iq: u32,
    /// 策略能力 - 影响长期规划
    pub strategy: u32,
    /// 学习速度 - 影响经验获取效率
    pub learning_rate: u32,
    /// 适应能力 - 影响对新对手的适应
    pub adaptability: u32,
    /// 创造力 - 影响非常规策略使用
    pub creativity: u32,
    /// 记忆容量 - 影响可记忆的对手数量
    pub memory_capacity: u32,
}

/// 进化阶段
pub enum EvolutionStage {
    /// 初生 - 基础能力
    Nascent = 1,
    /// 觉醒 - 解锁自动战斗
    Awakened = 2,
    /// 成熟 - 解锁策略建议
    Mature = 3,
    /// 精英 - 解锁对手分析
    Elite = 4,
    /// 大师 - 解锁完全自主
    Master = 5,
    /// 超越 - 解锁创造新策略
    Transcendent = 6,
}

/// 训练状态
pub enum TrainingStatus {
    /// 空闲
    Idle,
    /// 训练中
    Training {
        training_type: TrainingType,
        started_at: BlockNumber,
        ends_at: BlockNumber,
    },
    /// 冷却中
    Cooldown { ends_at: BlockNumber },
}
```

## 4. 进化系统

### 4.1 进化阶段详解

| 阶段 | 能力上限 | 解锁功能 | 进化条件 |
|------|----------|----------|----------|
| 初生 | 100 | 基础辅助 | - |
| 觉醒 | 300 | 自动战斗 | 经验1000 + 材料 |
| 成熟 | 600 | 策略建议 | 经验5000 + 材料 |
| 精英 | 1000 | 对手分析 | 经验20000 + 材料 |
| 大师 | 2000 | 完全自主 | 经验100000 + 材料 |
| 超越 | ∞ | 创造策略 | 大师满级 + 融合 |

### 4.2 进化公式

```
进化成功率 = 基础成功率 * (1 + 能力加成) * 材料品质加成

基础成功率:
- 初生→觉醒: 100%
- 觉醒→成熟: 80%
- 成熟→精英: 60%
- 精英→大师: 40%
- 大师→超越: 20%

失败惩罚:
- 经验损失 10%
- 冷却期 24小时
```

### 4.3 进化路径

```
                    ┌─────────────┐
                    │   超越者    │
                    │Transcendent │
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
       ┌──────┴──────┐ ┌───┴───┐ ┌──────┴──────┐
       │ 战术大师   │ │ 策略  │ │ 创意大师   │
       │ Tactician  │ │ Master│ │ Innovator  │
       └──────┬──────┘ └───┬───┘ └──────┬──────┘
              │            │            │
              └────────────┼────────────┘
                           │
                    ┌──────┴──────┐
                    │    精英     │
                    │   Elite     │
                    └──────┬──────┘
                           │
                    ┌──────┴──────┐
                    │    成熟     │
                    │   Mature    │
                    └──────┬──────┘
                           │
                    ┌──────┴──────┐
                    │    觉醒     │
                    │  Awakened   │
                    └──────┬──────┘
                           │
                    ┌──────┴──────┐
                    │    初生     │
                    │   Nascent   │
                    └─────────────┘
```

## 5. 训练系统

### 5.1 训练类型

| 类型 | 描述 | 主要提升 | 时长 | 消耗 |
|------|------|----------|------|------|
| 战斗训练 | 实战对抗 | 战斗智能 | 1-4小时 | 能量 |
| 策略训练 | 模拟推演 | 策略能力 | 4-12小时 | 代币 |
| 数据训练 | 分析历史 | 学习速度 | 2-8小时 | 数据点 |
| 适应训练 | 多样对手 | 适应能力 | 6-24小时 | 能量+代币 |
| 创意训练 | 随机探索 | 创造力 | 12-48小时 | 高消耗 |

### 5.2 训练效率公式

```
经验获取 = 基础经验 * 学习速度加成 * 训练时长加成 * 稀有度加成

能力提升 = 训练点数 * (1 + 当前能力/1000) * 训练类型匹配度

训练类型匹配度:
- 战斗训练 → 战斗智能: 1.5x
- 策略训练 → 策略能力: 1.5x
- 其他组合: 1.0x
```

### 5.3 训练数据结构

```rust
pub struct TrainingSession<T: Config> {
    /// 训练ID
    pub id: u64,
    /// AI 智能体ID
    pub agent_id: u64,
    /// 训练类型
    pub training_type: TrainingType,
    /// 开始区块
    pub started_at: BlockNumberFor<T>,
    /// 结束区块
    pub ends_at: BlockNumberFor<T>,
    /// 预期收益
    pub expected_gains: AIAbilities,
    /// 消耗资源
    pub cost: TrainingCost,
}

pub enum TrainingType {
    Combat,
    Strategy,
    DataAnalysis,
    Adaptation,
    Creativity,
}

pub struct TrainingCost {
    pub energy: u32,
    pub tokens: Balance,
    pub data_points: u32,
}
```

## 6. AI 融合系统

### 6.1 融合规则

```
融合条件:
├── 两个 AI 都达到成熟阶段以上
├── 同一所有者或双方授权
├── 支付融合费用
└── 冷却期结束

融合结果:
├── 新 AI 世代 = max(父代1, 父代2) + 1
├── 能力继承 = (父代1能力 + 父代2能力) / 2 * 继承系数
├── 基因融合 = 随机选择 + 变异
└── 有概率产生突变能力
```

### 6.2 融合公式

```rust
/// 计算融合结果
pub fn calculate_fusion_result(
    parent1: &AIAgent,
    parent2: &AIAgent,
    random_seed: [u8; 32],
) -> FusionResult {
    // 新世代
    let new_generation = parent1.generation.max(parent2.generation) + 1;
    
    // 继承系数 (世代越高，继承越多)
    let inheritance_factor = 50 + new_generation.min(50);
    
    // 能力继承
    let new_abilities = AIAbilities {
        combat_iq: blend_ability(
            parent1.abilities.combat_iq,
            parent2.abilities.combat_iq,
            inheritance_factor,
            random_seed[0],
        ),
        strategy: blend_ability(
            parent1.abilities.strategy,
            parent2.abilities.strategy,
            inheritance_factor,
            random_seed[1],
        ),
        // ... 其他能力
    };
    
    // 变异检查 (5% + 世代*0.5%)
    let mutation_chance = 500 + new_generation * 50;
    let has_mutation = (random_seed[10] as u32 * 100) < mutation_chance;
    
    FusionResult {
        generation: new_generation,
        abilities: new_abilities,
        has_mutation,
        mutation_type: if has_mutation {
            Some(determine_mutation(random_seed))
        } else {
            None
        },
    }
}

fn blend_ability(a: u32, b: u32, factor: u32, random: u8) -> u32 {
    let base = (a + b) / 2;
    let variance = (random as u32 % 20) as i32 - 10; // -10% to +10%
    let result = base as i32 * (100 + variance) / 100;
    (result as u32 * factor / 100).max(1)
}
```

### 6.3 变异类型

| 变异 | 概率 | 效果 |
|------|------|------|
| 能力突破 | 40% | 随机能力 +50% |
| 隐藏天赋 | 25% | 获得特殊被动 |
| 基因强化 | 20% | 所有能力 +10% |
| 进化加速 | 10% | 下次进化成功率 +20% |
| 传说变异 | 5% | 获得传说技能 |

## 7. AI 决策引擎

### 7.1 决策流程

```
┌─────────────────────────────────────────────────────────┐
│                   AI 决策流程                            │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  输入层                                                  │
│  ┌─────────────────────────────────────────────────┐   │
│  │ 己方状态 | 敌方状态 | 历史记录 | 回合数         │   │
│  └─────────────────────────────────────────────────┘   │
│                         │                               │
│                         v                               │
│  分析层                                                  │
│  ┌─────────────────────────────────────────────────┐   │
│  │ 威胁评估 | 机会识别 | 资源计算 | 预测对手       │   │
│  └─────────────────────────────────────────────────┘   │
│                         │                               │
│                         v                               │
│  策略层                                                  │
│  ┌─────────────────────────────────────────────────┐   │
│  │ 激进策略 | 防守策略 | 平衡策略 | 特殊策略       │   │
│  └─────────────────────────────────────────────────┘   │
│                         │                               │
│                         v                               │
│  输出层                                                  │
│  ┌─────────────────────────────────────────────────┐   │
│  │ 最优行动 | 备选行动 | 行动理由                  │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### 7.2 决策权重

```rust
pub struct DecisionWeights {
    /// 生存优先权重
    pub survival_weight: u32,
    /// 伤害优先权重
    pub damage_weight: u32,
    /// 控制优先权重
    pub control_weight: u32,
    /// 资源效率权重
    pub efficiency_weight: u32,
    /// 风险承受权重
    pub risk_tolerance: u32,
}

impl DecisionWeights {
    /// 根据 AI 能力计算权重
    pub fn from_abilities(abilities: &AIAbilities) -> Self {
        Self {
            survival_weight: 100 - abilities.creativity / 20,
            damage_weight: abilities.combat_iq / 10,
            control_weight: abilities.strategy / 10,
            efficiency_weight: abilities.adaptability / 10,
            risk_tolerance: abilities.creativity / 10,
        }
    }
}
```

### 7.3 对手建模

```rust
/// 对手模型
pub struct OpponentModel {
    /// 对手ID
    pub opponent_id: AccountId,
    /// 遭遇次数
    pub encounters: u32,
    /// 胜率
    pub win_rate: u32,
    /// 常用策略
    pub common_strategies: Vec<StrategyPattern>,
    /// 弱点分析
    pub weaknesses: Vec<Weakness>,
    /// 最后更新
    pub last_updated: BlockNumber,
}

/// 策略模式
pub struct StrategyPattern {
    /// 模式名称
    pub name: BoundedVec<u8, ConstU32<32>>,
    /// 触发条件
    pub trigger_conditions: Vec<Condition>,
    /// 使用频率
    pub frequency: u32,
    /// 应对策略
    pub counter_strategy: Option<CounterStrategy>,
}
```

## 8. 链上实现

### 8.1 存储设计

```rust
/// AI 智能体存储
#[pallet::storage]
pub type AIAgents<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat, T::AccountId,
    Blake2_128Concat, u64,
    AIAgent<T>,
>;

/// 训练会话
#[pallet::storage]
pub type TrainingSessions<T: Config> = StorageMap<
    _,
    Blake2_128Concat, u64,
    TrainingSession<T>,
>;

/// 对手模型 (链下存储，链上哈希验证)
#[pallet::storage]
pub type OpponentModelHashes<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat, u64,  // AI ID
    Blake2_128Concat, T::AccountId,  // 对手
    [u8; 32],  // 模型哈希
>;

/// 全局 AI 计数
#[pallet::storage]
pub type NextAIAgentId<T: Config> = StorageValue<_, u64, ValueQuery>;
```

### 8.2 核心调用

```rust
#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 创建 AI 智能体
    #[pallet::weight(T::WeightInfo::create_ai_agent())]
    pub fn create_ai_agent(
        origin: OriginFor<T>,
        name: BoundedVec<u8, ConstU32<32>>,
        bound_pet: Option<u64>,
    ) -> DispatchResult;
    
    /// 开始训练
    #[pallet::weight(T::WeightInfo::start_training())]
    pub fn start_training(
        origin: OriginFor<T>,
        agent_id: u64,
        training_type: TrainingType,
        duration: u32,
    ) -> DispatchResult;
    
    /// 完成训练
    #[pallet::weight(T::WeightInfo::complete_training())]
    pub fn complete_training(
        origin: OriginFor<T>,
        agent_id: u64,
    ) -> DispatchResult;
    
    /// 进化 AI
    #[pallet::weight(T::WeightInfo::evolve_ai())]
    pub fn evolve_ai(
        origin: OriginFor<T>,
        agent_id: u64,
        evolution_path: Option<EvolutionPath>,
    ) -> DispatchResult;
    
    /// 融合 AI
    #[pallet::weight(T::WeightInfo::fuse_ai())]
    pub fn fuse_ai(
        origin: OriginFor<T>,
        agent1_id: u64,
        agent2_id: u64,
        new_name: BoundedVec<u8, ConstU32<32>>,
    ) -> DispatchResult;
}
```

### 8.3 事件定义

```rust
#[pallet::event]
pub enum Event<T: Config> {
    /// AI 创建
    AIAgentCreated {
        owner: T::AccountId,
        agent_id: u64,
        name: BoundedVec<u8, ConstU32<32>>,
    },
    /// 训练开始
    TrainingStarted {
        agent_id: u64,
        training_type: TrainingType,
        ends_at: BlockNumberFor<T>,
    },
    /// 训练完成
    TrainingCompleted {
        agent_id: u64,
        ability_gains: AIAbilities,
    },
    /// AI 进化
    AIEvolved {
        agent_id: u64,
        from_stage: EvolutionStage,
        to_stage: EvolutionStage,
    },
    /// AI 融合
    AIFused {
        parent1_id: u64,
        parent2_id: u64,
        child_id: u64,
        has_mutation: bool,
    },
}
```

## 9. 经济模型

### 9.1 消耗与收益

| 操作 | 消耗 | 收益 |
|------|------|------|
| 创建 AI | 100 MEOW | 初始 AI |
| 战斗训练 | 10 能量/小时 | 经验 + 战斗智能 |
| 策略训练 | 50 MEOW/次 | 经验 + 策略能力 |
| 进化 | 材料 + MEOW | 能力上限提升 |
| 融合 | 500 MEOW + 2 AI | 新世代 AI |

### 9.2 AI 市场

- AI 可以作为 NFT 交易
- 高世代/高能力 AI 更有价值
- 变异 AI 具有收藏价值
- 租借系统：临时使用他人 AI

---

*文档版本: v1.0 | 最后更新: 2026-01-30*
