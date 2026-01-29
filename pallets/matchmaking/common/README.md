# Pallet Matchmaking Common（婚恋模块 - 共享类型）

## 📋 模块概述

`pallet-matchmaking-common` 是婚恋系统的共享类型库，提供所有婚恋子模块使用的类型定义和 Trait 接口。

### 核心特性

- ✅ **类型定义**：用户资料、匹配评分、互动记录等共享类型
- ✅ **Trait 接口**：八字数据提供者、匹配算法、推荐系统等接口
- ✅ **枚举定义**：性别、学历、婚姻状况、隐私模式等枚举类型

---

## 🔑 核心类型

### 基础枚举类型

```rust
/// 性别
pub enum Gender {
    Male,    // 男
    Female,  // 女
}

/// 学历等级
pub enum EducationLevel {
    HighSchool,      // 高中
    Associate,       // 大专
    Bachelor,        // 本科
    Master,          // 硕士
    Doctorate,       // 博士
}

/// 婚姻状况
pub enum MaritalStatus {
    Single,          // 未婚
    Divorced,        // 离异
    Widowed,         // 丧偶
}

/// 房产状况
pub enum PropertyStatus {
    None,            // 无房
    Renting,         // 租房
    OwnedWithLoan,   // 有房有贷
    OwnedOutright,   // 有房无贷
}

/// 车辆状况
pub enum VehicleStatus {
    None,            // 无车
    Owned,           // 有车
}

/// 生活方式
pub enum Lifestyle {
    EarlyBird,       // 早起型
    NightOwl,        // 夜猫子
    Flexible,        // 灵活
}
```

### 隐私相关类型

```rust
/// 资料隐私模式
pub enum ProfilePrivacyMode {
    Public,          // 公开
    MatchesOnly,     // 仅匹配可见
    Private,         // 私密
}

/// 资料状态
pub enum ProfileStatus {
    Active,          // 活跃
    Suspended,       // 暂停
    Banned,          // 封禁
}
```

### 性格相关类型

```rust
/// 用户自选性格特征
pub enum PersonalityTrait {
    Outgoing,        // 外向
    Introverted,     // 内向
    Optimistic,      // 乐观
    Calm,            // 沉稳
    Humorous,        // 幽默
    Romantic,        // 浪漫
    Practical,       // 务实
    Creative,        // 有创意
}

/// 八字解盘性格特征
pub enum BaziPersonalityTrait {
    Leadership,      // 领导力
    Creativity,      // 创造力
    Analytical,      // 分析能力
    Empathy,         // 同理心
    Determination,   // 决心
    Flexibility,     // 灵活性
    Patience,        // 耐心
    Ambition,        // 野心
}

/// 性格来源
pub enum PersonalitySource {
    UserFilled,      // 用户自填
    BaziAnalysis,    // 八字解盘
    Combined,        // 综合
}
```

### 互动相关类型

```rust
/// 互动类型
pub enum InteractionType {
    Like,            // 点赞
    SuperLike,       // 超级喜欢
    Pass,            // 跳过
    Block,           // 屏蔽
}

/// 匹配状态
pub enum MatchStatus {
    PendingAuthorization,  // 待授权
    Authorized,            // 已授权
    Rejected,              // 已拒绝
    Cancelled,             // 已取消
    Completed,             // 已完成
}

/// 匹配推荐等级
pub enum MatchRecommendation {
    HighlyRecommended,     // 强烈推荐（>= 85分）
    Recommended,           // 推荐（70-84分）
    Neutral,               // 中性（55-69分）
    NotRecommended,        // 不推荐（< 55分）
}
```

### 合婚评分详情

```rust
/// 合婚评分详情
pub struct CompatibilityScoreDetail {
    /// 日柱合婚分（30%权重）
    pub day_pillar_score: u8,
    /// 五行互补分（25%权重）
    pub wuxing_score: u8,
    /// 性格匹配分（20%权重）
    pub personality_score: u8,
    /// 神煞分析分（15%权重）
    pub shensha_score: u8,
    /// 大运配合分（10%权重）
    pub dayun_score: u8,
}

impl CompatibilityScoreDetail {
    /// 计算综合评分
    pub fn calculate_overall(&self) -> u8 {
        let weighted = (self.day_pillar_score as u32 * 30
            + self.wuxing_score as u32 * 25
            + self.personality_score as u32 * 20
            + self.shensha_score as u32 * 15
            + self.dayun_score as u32 * 10) / 100;
        weighted as u8
    }
}
```

### 推荐结果

```rust
/// 推荐结果
pub struct RecommendationResult<AccountId> {
    /// 推荐用户
    pub account: AccountId,
    /// 匹配评分
    pub score: u8,
    /// 推荐原因代码
    pub reason_codes: Vec<u8>,
}
```

---

## 🔗 Trait 接口

### 字段隐私设置

```rust
/// 字段级隐私设置
#[derive(Default)]
pub struct FieldPrivacySettings {
    /// 年龄是否公开
    pub age_public: bool,
    /// 收入是否公开
    pub income_public: bool,
    /// 位置是否公开
    pub location_public: bool,
    /// 照片是否公开
    pub photos_public: bool,
}
```

### 合婚偏好

```rust
/// 合婚偏好设置
pub struct CompatibilityPreferences {
    /// 是否启用八字合婚
    pub enable_bazi_matching: bool,
    /// 最低合婚评分要求
    pub min_compatibility_score: Option<u8>,
    /// 是否显示合婚详情
    pub show_compatibility_details: bool,
}
```

---

## 📦 模块结构

```text
pallet-matchmaking-common
├── src/
│   ├── lib.rs      # 模块入口，导出所有类型
│   ├── types.rs    # 共享类型定义
│   └── traits.rs   # Trait 接口定义
└── Cargo.toml
```

---

## 🔗 依赖关系

本模块被以下婚恋子模块依赖：

- `pallet-matchmaking-profile` - 用户资料管理
- `pallet-matchmaking-matching` - 匹配算法
- `pallet-matchmaking-interaction` - 互动功能
- `pallet-matchmaking-recommendation` - 推荐系统

---

## 📚 相关文档

- [婚恋模块主文档](../README.md)
- [用户资料模块](../profile/README.md)
- [匹配算法模块](../matching/README.md)
- [互动功能模块](../interaction/README.md)
- [推荐系统模块](../recommendation/README.md)

---

## 📝 版本历史

### v0.1.0（当前版本）

- ✅ 基础类型定义
- ✅ 枚举类型定义
- ✅ Trait 接口定义
