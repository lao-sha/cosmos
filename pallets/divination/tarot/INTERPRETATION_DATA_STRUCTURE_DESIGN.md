# 塔罗牌解卦数据结构设计文档

## 一、设计背景

### 1.1 参考资料分析

#### 参考实现（xuanxue/tarot）

通过分析 `/home/xiaodong/文档/stardust/xuanxue/tarot` 项目，发现其塔罗牌系统包含以下核心数据：

**Python实现（tarot-reader）**：
- 完整的78张牌数据结构（TarotCard）
- 8种标准牌阵（SpreadType）
- LLM智能解读系统
- 正逆位随机生成
- 牌阵推荐系统

**Go实现（MiraiGo-module-tarot）**：
- QQ机器人集成
- 图片180度翻转显示逆位
- 基于哈希的占卜算法
- 78张牌的完整数据库（JSON）

#### 现有 Pallet 实现（pallets/divination/tarot）

- **types.rs**: 完整的排盘数据结构（TarotReading）
- **algorithm.rs**: 5种起卦方式、切牌机制、能量分析
- **constants.rs**: 78张牌的完整牌义数据库

#### 其他占卜模块参考

- **八字解盘**：分层设计（Core 13 bytes + Extended）
- **六爻解卦**：核心指标 20 bytes，完整解盘 ~162 bytes
- **梅花解卦**：核心指标 ~20 bytes，完整解盘 ~200 bytes

### 1.2 设计原则

1. **分层存储**：核心指标链上存储，详细解释链下生成
2. **存储优化**：使用枚举索引和位图而非字符串
3. **实时计算**：通过 Runtime API 免费获取解卦
4. **算法可升级**：无需数据迁移
5. **AI友好**：结构化数据便于AI深度解读

---

## 二、核心数据结构设计

### 2.1 枚举类型定义

#### 吉凶倾向等级（FortuneTendency）

```rust
pub enum FortuneTendency {
    Excellent = 0,  // 大吉 - 诸事顺遂，心想事成
    Good = 1,       // 吉 - 事可成，宜进取
    Neutral = 2,    // 中平 - 平稳发展，守成为上
    MinorBad = 3,   // 小凶 - 小有阻碍，谨慎行事
    Bad = 4,        // 凶 - 困难重重，需要调整
}
```

#### 主导元素类型（DominantElement）

```rust
pub enum DominantElement {
    None = 0,    // 无明显主导元素
    Fire = 1,    // 火元素主导（权杖）- 行动力、激情、创造力
    Water = 2,   // 水元素主导（圣杯）- 情感、直觉、人际关系
    Air = 3,     // 风元素主导（宝剑）- 思维、沟通、智力活动
    Earth = 4,   // 土元素主导（星币）- 物质、工作、实际事务
    Spirit = 5,  // 灵性主导（大阿卡纳）- 重大转折、命运指引
}
```

#### 能量流动方向（EnergyFlow）

```rust
pub enum EnergyFlow {
    Rising = 0,     // 上升 - 能量逐渐增强
    Declining = 1,  // 下降 - 能量逐渐减弱
    Stable = 2,     // 平稳 - 能量保持稳定
    Volatile = 3,   // 波动 - 能量起伏不定
}
```

#### 牌间关系类型（RelationshipType）

```rust
pub enum RelationshipType {
    None = 0,                  // 无明显关系
    Generating = 1,            // 相生 - 能量互相增强
    Controlling = 2,           // 相克 - 能量互相制约
    SameElementReinforce = 3,  // 同元素强化 - 同类能量叠加
    Opposing = 4,              // 对立冲突 - 能量相互对抗
    Complementary = 5,         // 互补 - 能量相互补充
}
```

### 2.2 核心解卦结构（TarotCoreInterpretation）

**总大小：约 30 bytes**

```rust
pub struct TarotCoreInterpretation {
    // ===== 基础判断 (4 bytes) =====

    /// 总体能量等级 (1 byte, 0-100)
    /// 计算公式：(正位牌数 × 10 + 大阿卡纳数 × 15) / 牌数
    pub overall_energy: u8,

    /// 主导元素 (1 byte)
    pub dominant_element: DominantElement,

    /// 吉凶倾向 (1 byte)
    pub fortune_tendency: FortuneTendency,

    /// 逆位比例 (1 byte, 0-100)
    pub reversed_ratio: u8,

    // ===== 牌组特征 (8 bytes) =====

    /// 大阿卡纳数量 (1 byte, 0-12)
    pub major_arcana_count: u8,

    /// 宫廷牌数量 (1 byte, 0-12)
    pub court_cards_count: u8,

    /// 数字牌数量 (1 byte, 0-12)
    pub number_cards_count: u8,

    /// 元素分布位图 (1 byte)
    /// bit 0-1: 火元素数量(0-3)
    /// bit 2-3: 水元素数量(0-3)
    /// bit 4-5: 风元素数量(0-3)
    /// bit 6-7: 土元素数量(0-3)
    pub element_bitmap: u8,

    /// 特殊组合标志 (1 byte, 位图)
    /// bit 0: 愚者+世界组合
    /// bit 1: 三张以上大阿卡纳
    /// bit 2: 同花色三连号
    /// bit 3: 全逆位
    /// bit 4: 全正位
    /// bit 5-7: 保留
    pub special_combination: u8,

    /// 关键牌ID (1 byte, 0-77)
    pub key_card_id: u8,

    /// 关键牌正逆位 (1 byte)
    pub key_card_reversed: u8,

    /// 牌阵类型 (1 byte)
    pub spread_type: u8,

    // ===== 能量分析 (8 bytes) =====

    /// 行动力指数 (1 byte, 0-100)
    pub action_index: u8,

    /// 情感指数 (1 byte, 0-100)
    pub emotion_index: u8,

    /// 思维指数 (1 byte, 0-100)
    pub intellect_index: u8,

    /// 物质指数 (1 byte, 0-100)
    pub material_index: u8,

    /// 灵性指数 (1 byte, 0-100)
    pub spiritual_index: u8,

    /// 稳定性指数 (1 byte, 0-100)
    pub stability_index: u8,

    /// 变化性指数 (1 byte, 0-100)
    pub change_index: u8,

    /// 综合评分 (1 byte, 0-100)
    pub overall_score: u8,

    // ===== 元数据 (10 bytes) =====

    /// 解卦时间戳 - 区块号 (4 bytes)
    pub block_number: u32,

    /// 解卦算法版本 (1 byte)
    pub algorithm_version: u8,

    /// 可信度 (1 byte, 0-100)
    pub confidence: u8,

    /// 保留字段 (4 bytes)
    pub reserved: [u8; 4],
}
```

### 2.3 牌阵能量分析（SpreadEnergyAnalysis）

**总大小：约 8 bytes**

```rust
pub struct SpreadEnergyAnalysis {
    /// 过去能量 (0-100)
    pub past_energy: u8,

    /// 现在能量 (0-100)
    pub present_energy: u8,

    /// 未来能量 (0-100)
    pub future_energy: u8,

    /// 内在能量 (0-100)
    pub inner_energy: u8,

    /// 外在能量 (0-100)
    pub outer_energy: u8,

    /// 能量流动方向
    pub energy_flow: EnergyFlow,

    /// 能量平衡度 (0-100, 100最平衡)
    pub energy_balance: u8,
}
```

### 2.4 单张牌分析（CardInterpretation）

**总大小：约 7 bytes**

```rust
pub struct CardInterpretation {
    /// 牌ID (0-77)
    pub card_id: u8,

    /// 是否逆位
    pub is_reversed: bool,

    /// 在牌阵中的位置索引 (0-based)
    pub spread_position: u8,

    /// 位置权重 (1-10, 10最重要)
    pub position_weight: u8,

    /// 牌的能量强度 (0-100)
    pub energy_strength: u8,

    /// 与前一张牌的关系类型
    pub relation_to_prev: RelationshipType,

    /// 与后一张牌的关系类型
    pub relation_to_next: RelationshipType,
}
```

### 2.5 完整解卦结构（TarotFullInterpretation）

```rust
pub struct TarotFullInterpretation<MaxCards: Get<u32>> {
    /// 核心指标（必有）
    pub core: TarotCoreInterpretation,

    /// 牌阵能量分析（必有）
    pub spread_energy: SpreadEnergyAnalysis,

    /// 各牌分析（可选，最多12张）
    pub card_analyses: Option<BoundedVec<CardInterpretation, MaxCards>>,

    /// 牌间关系分析（可选）
    pub card_relationships: Option<BoundedVec<CardRelationship, MaxCards>>,

    /// 时间线分析（可选，仅适用于时间相关牌阵）
    pub timeline_analysis: Option<TimelineAnalysis>,
}
```

---

## 三、核心算法设计

### 3.1 总体能量计算

```
overall_energy = (正位牌数 × 10 + 大阿卡纳数 × 15) / 牌数
```

### 3.2 主导元素判断

```
if 大阿卡纳比例 > 50% => 灵性主导
else if 某元素数量 >= 2 且为最多 => 该元素主导
else => 无主导
```

### 3.3 吉凶判断规则

```
// 特殊组合优先
if 愚者+世界组合 => 吉

// 大阿卡纳为主
if 大阿卡纳比例 > 50% {
    if 逆位比例 < 40% => 吉
    else => 中平
}

// 逆位较多
if 逆位比例 >= 70% => 凶
if 逆位比例 >= 50% => 小凶

// 按主导元素判断
火元素主导 + 逆位 < 30% => 大吉
火元素主导 + 逆位 >= 30% => 吉
水元素主导 + 逆位 < 40% => 吉
水元素主导 + 逆位 >= 40% => 中平
风元素主导 => 中平
土元素主导 + 逆位 < 30% => 吉
土元素主导 + 逆位 >= 30% => 中平
灵性主导 => 吉
无主导 => 中平
```

### 3.4 能量指数计算

| 指数 | 计算公式 |
|------|---------|
| 行动力 | 权杖牌数 × 25 + 正位比例 × 0.5 |
| 情感 | 圣杯牌数 × 25 |
| 思维 | 宝剑牌数 × 25 |
| 物质 | 星币牌数 × 25 |
| 灵性 | 大阿卡纳比例 × 100 |
| 稳定性 | 正位比例 × 60% + 数字牌比例 × 40% |
| 变化性 | 逆位比例 × 60% + 宫廷牌比例 × 40% |

### 3.5 综合评分计算

```
综合评分 = (吉凶评分 × 40% + 总体能量 × 30% + 稳定性 × 20% + 灵性 × 10%)

吉凶评分映射：
- 大吉 => 90
- 吉 => 75
- 中平 => 50
- 小凶 => 35
- 凶 => 20
```

### 3.6 可信度计算

```
牌数评分 = min(牌数 × 10, 100)

大阿卡纳比例评分：
- if 20% <= 比例 <= 60% => 100
- if 比例 < 20% => 50 + 比例 × 2
- if 比例 > 60% => 160 - 比例

可信度 = (牌数评分 + 大阿卡纳评分) / 2
```

---

## 四、存储优化分析

### 4.1 存储大小估算

| 结构 | 大小 | 说明 |
|------|------|------|
| TarotCoreInterpretation | ~30 bytes | 核心指标 |
| SpreadEnergyAnalysis | ~8 bytes | 能量分析 |
| CardInterpretation × 12 | ~84 bytes | 各牌分析 |
| CardRelationship × 12 | ~48 bytes | 牌间关系 |
| TimelineAnalysis | ~5 bytes | 时间线分析 |
| **总计** | **~175 bytes** | 完整解卦 |

### 4.2 与其他占卜模块对比

| 项目 | 八字 | 六爻 | 梅花 | 塔罗 |
|------|------|------|------|------|
| 核心指标 | 13 bytes | 20 bytes | ~20 bytes | 30 bytes |
| 完整解盘 | ~50 bytes | ~162 bytes | ~200 bytes | ~175 bytes |
| 实时计算 | ✅ | ✅ | ✅ | ✅ |
| AI解读 | ✅ | ✅ | ✅ | ✅ |

### 4.3 位图优化

**元素分布位图**（1 byte）：
```
bit 0-1: 火元素数量(0-3)
bit 2-3: 水元素数量(0-3)
bit 4-5: 风元素数量(0-3)
bit 6-7: 土元素数量(0-3)
```

**特殊组合标志**（1 byte）：
```
bit 0: 愚者+世界组合
bit 1: 三张以上大阿卡纳
bit 2: 同花色三连号
bit 3: 全逆位
bit 4: 全正位
bit 5-7: 保留
```

---

## 五、Runtime API 设计

### 5.1 解卦 API

```rust
sp_api::decl_runtime_apis! {
    pub trait TarotInterpretationApi {
        /// 获取核心解卦（免费实时计算）
        fn get_core_interpretation(
            reading_id: u64,
        ) -> Option<TarotCoreInterpretation>;

        /// 获取完整解卦（免费实时计算）
        fn get_full_interpretation(
            reading_id: u64,
        ) -> Option<TarotFullInterpretation<ConstU32<12>>>;

        /// 获取解读文本索引列表
        fn get_interpretation_texts(
            reading_id: u64,
        ) -> Option<BoundedVec<InterpretationTextType, ConstU32<20>>>;

        /// 分析单张牌在特定牌阵位置的含义
        fn analyze_card_in_spread(
            card_id: u8,
            is_reversed: bool,
            spread_type: SpreadType,
            position: u8,
        ) -> Option<CardInterpretation>;

        /// 分析两张牌之间的关系
        fn analyze_card_relationship(
            card1_id: u8,
            card2_id: u8,
        ) -> Option<CardRelationship>;

        /// 生成AI解读提示词上下文
        fn generate_ai_prompt_context(
            reading_id: u64,
        ) -> Option<BoundedVec<u8, ConstU32<2048>>>;
    }
}
```

---

## 六、实现计划

### 第一阶段：核心结构 ✅

1. ✅ 定义所有枚举类型
2. ✅ 实现 TarotCoreInterpretation
3. ✅ 实现 SpreadEnergyAnalysis
4. ✅ 实现 CardInterpretation
5. ✅ 实现 TarotFullInterpretation
6. ✅ 实现核心解卦算法

### 第二阶段：Runtime API

1. 定义 Runtime API trait
2. 实现解卦查询接口
3. 实现AI提示词生成
4. 添加批量查询支持

### 第三阶段：前端集成

1. 前端调用 Runtime API
2. 解读结果展示
3. AI解读集成
4. 历史记录查询

---

## 七、使用示例

### 7.1 生成核心解卦

```rust
use crate::algorithm::generate_core_interpretation;
use crate::types::SpreadType;

// 抽取的牌：愚者(正位)、魔术师(正位)、世界(逆位)
let cards = vec![(0, false), (1, false), (21, true)];
let block_number = 1000;

let core = generate_core_interpretation(
    &cards,
    SpreadType::ThreeCardTime,
    block_number,
);

// 核心指标
assert_eq!(core.major_arcana_count, 3);
assert_eq!(core.reversed_ratio, 33);
assert_eq!(core.dominant_element, DominantElement::Spirit);
assert_eq!(core.fortune_tendency, FortuneTendency::Good);

// 特殊组合
assert!(core.has_fool_world_combo());
assert!(core.has_many_major_arcana());

// 能量指数
println!("灵性指数: {}", core.spiritual_index);
println!("综合评分: {}", core.overall_score);
```

### 7.2 生成牌阵能量分析

```rust
use crate::algorithm::generate_spread_energy_analysis;

let cards = vec![(0, false), (1, false), (21, false)];
let energy = generate_spread_energy_analysis(&cards);

println!("过去能量: {}", energy.past_energy);
println!("现在能量: {}", energy.present_energy);
println!("未来能量: {}", energy.future_energy);
println!("能量流动: {:?}", energy.energy_flow);
println!("能量平衡度: {}", energy.energy_balance);
```

---

## 八、总结

本设计方案：

1. **完整性**：涵盖塔罗占卜的所有核心要素（牌义、牌阵、元素、能量）
2. **高效性**：核心指标仅30 bytes，完整解卦约175 bytes
3. **实时计算**：通过 Runtime API 免费获取解卦，无需链上存储冗余数据
4. **AI友好**：结构化数据便于AI深度解读
5. **可扩展性**：预留字段支持未来算法升级

相比 xuanxue/tarot 的实现，本方案：
- ✅ 保留了所有核心解读功能
- ✅ 新增了能量分析、牌间关系等高级功能
- ✅ 集成了AI解读能力
- ✅ 优化了存储结构，适合区块链环境
- ✅ 提供了完整的 Runtime API，便于前端使用

---

**设计日期**：2025-12-13
**实施状态**：第一阶段完成 ✅
**代码位置**：`src/interpretation.rs` + `src/algorithm.rs`
