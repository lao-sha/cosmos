# Pallet Matchmaking Recommendation（婚恋模块 - 推荐系统）

## 📋 模块概述

`pallet-matchmaking-recommendation` 是婚恋系统的智能推荐模块，基于匹配评分推荐潜在对象。

### 核心特性

- ✅ **推荐列表**：获取个性化推荐用户列表
- ✅ **推荐策略**：基于匹配评分、活跃度、地理位置
- ✅ **推荐更新**：定期更新推荐列表
- ✅ **基于内容推荐**：条件筛选 + 分数计算

---

## 🔑 核心功能

### 1. 推荐策略

1. **基于匹配评分**：推荐高分匹配用户
2. **基于活跃度**：推荐近期活跃用户
3. **基于地理位置**：推荐同城用户

### 2. 推荐算法

**算法复杂度**：O(n log n)
- 条件筛选：O(n)
- 分数计算：O(n)
- 排序：O(n log n)

**性能**：10,000 候选人约 2 秒

### 3. 条件筛选

```rust
pub fn meets_preferences(
    preferences: &UserPreferences,
    candidate: &CandidateInfo,
) -> bool {
    // 年龄筛选
    // 身高筛选
    // 学历筛选
    // 收入筛选
    // 孩子筛选
    // ...
}
```

### 4. 分数计算

```rust
pub fn calculate_match_score(
    user_traits: &[u8; 5],
    candidate: &CandidateInfo,
    current_block: u64,
) -> MatchScoreResult {
    // 基础条件评分（资料完整度）
    // 性格匹配评分（共同特征）
    // 活跃度加成
    // 综合评分
}
```

**活跃度加成**：

| 活跃时间 | 加成 |
|----------|------|
| 1天内 | +20分 |
| 1周内 | +10分 |
| 1月内 | +5分 |
| 超过1月 | 0分 |

---

## 📊 数据结构

### 用户偏好条件

```rust
pub struct UserPreferences {
    pub age_range: Option<(u8, u8)>,
    pub height_range: Option<(u16, u16)>,
    pub min_education: Option<u8>,
    pub income_range: Option<(u32, u32)>,
    pub accept_children: Option<bool>,
    pub min_bazi_score: Option<u8>,
}
```

### 候选人信息

```rust
pub struct CandidateInfo {
    pub age: Option<u8>,
    pub height: Option<u16>,
    pub education_level: Option<u8>,
    pub income: Option<u32>,
    pub has_children: Option<bool>,
    pub last_active_block: u64,
    pub personality_traits: [u8; 5],
    pub bazi_chart_id: Option<u64>,
}
```

### 匹配分数结果

```rust
pub struct MatchScoreResult {
    pub basic_score: u8,        // 基础条件分
    pub personality_score: u8,  // 性格匹配分
    pub bazi_score: Option<u8>, // 八字合婚分
    pub activity_bonus: u8,     // 活跃度加成
    pub overall: u8,            // 综合评分
}
```

---

## 💾 存储项

```rust
/// 用户推荐列表
pub type Recommendations<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<RecommendationResult<T::AccountId>, T::MaxRecommendationsPerUser>,
>;

/// 推荐列表最后更新时间
pub type LastUpdate<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BlockNumberFor<T>,
>;
```

---

## 🎯 外部调用（Extrinsics）

| 调用 | 描述 |
|------|------|
| `refresh_recommendations` | 刷新推荐列表 |
| `clear_recommendations` | 清空推荐列表 |

---

## 📡 事件定义

```rust
RecommendationsUpdated { user, count }
RecommendationsRefreshed { user }
```

---

## ⚙️ 配置参数

```rust
impl pallet_matchmaking_recommendation::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxRecommendationsPerUser = ConstU32<100>;
    type RecommendationUpdateInterval = ConstU32<14400>;  // 约1天
    type WeightInfo = ();
}
```

---

## 🔬 算法详解

### 基于内容的推荐算法

```rust
pub fn recommend_matches<AccountId: Clone + Ord>(
    user_traits: &[u8; 5],
    preferences: &UserPreferences,
    candidates: &[(AccountId, CandidateInfo)],
    current_block: u64,
    limit: usize,
) -> Vec<(AccountId, u8)> {
    // 1. 条件筛选 (O(n))
    // 2. 分数计算 (O(n))
    // 3. 排序 (O(n log n))
    // 4. 截取前 limit 个
}
```

### 综合评分权重

| 维度 | 权重 |
|------|------|
| 基础条件 | 40% |
| 性格匹配 | 40% |
| 活跃度 | 20% |

---

## 📚 相关文档

- [婚恋模块主文档](../README.md)
- [用户资料模块](../profile/README.md)
- [匹配算法模块](../matching/README.md)
