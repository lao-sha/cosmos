# Pallet Matchmaking Matching（婚恋模块 - 匹配算法）

## 📋 模块概述

`pallet-matchmaking-matching` 是婚恋系统的匹配算法模块，提供八字合婚和性格匹配功能。

### 核心特性

- ✅ **八字合婚**：日柱天干地支分析、五行互补分析
- ✅ **性格匹配**：互补性格、冲突性格、共同优点
- ✅ **合婚请求管理**：创建、授权、拒绝、取消
- ✅ **合婚报告生成**：综合评分、推荐等级

---

## 🔑 核心功能

### 1. 合婚算法权重

| 维度 | 权重 | 说明 |
|------|------|------|
| 日柱合婚 | 30% | 日柱天干地支相合分析 |
| 五行互补 | 25% | 用神、忌神、五行平衡 |
| 性格匹配 | 20% | 性格互补与冲突分析 |
| 神煞分析 | 15% | 吉凶神煞配合 |
| 大运配合 | 10% | 大运走势配合度 |

### 2. 合婚请求流程

```
甲方创建请求 → 乙方授权 → 生成报告
     ↓              ↓
   可取消        可拒绝
```

### 3. 日柱合婚算法

```rust
pub fn calculate_day_pillar_compatibility(
    day_ganzhi_a: GanZhi,
    day_ganzhi_b: GanZhi,
) -> DayPillarResult {
    // 天干相合检查
    let gan_he = check_tiangan_he(day_ganzhi_a.gan, day_ganzhi_b.gan);
    
    // 地支六合检查
    let zhi_liu_he = check_dizhi_liu_he(day_ganzhi_a.zhi, day_ganzhi_b.zhi);
    
    // 地支三合检查
    let zhi_san_he = check_dizhi_san_he(day_ganzhi_a.zhi, day_ganzhi_b.zhi);
    
    // 地支相冲检查
    let zhi_chong = check_dizhi_chong(day_ganzhi_a.zhi, day_ganzhi_b.zhi);
    
    // 综合评分
    calculate_overall_score(gan_he, zhi_liu_he, zhi_san_he, zhi_chong)
}
```

### 4. 五行互补算法

```rust
pub fn calculate_wuxing_compatibility(
    interp_a: &CoreInterpretation,
    interp_b: &CoreInterpretation,
) -> WuxingCompatibilityResult {
    // 用神互补分析
    let yongshen_score = analyze_yongshen_complement(
        interp_a.yong_shen,
        interp_b.yong_shen,
    );
    
    // 忌神规避分析
    let jishen_score = analyze_jishen_avoidance(
        interp_a.ji_shen,
        interp_b.ji_shen,
    );
    
    // 五行平衡分析
    let balance_score = analyze_wuxing_balance(
        &interp_a.wuxing_distribution,
        &interp_b.wuxing_distribution,
    );
    
    WuxingCompatibilityResult {
        yongshen_score,
        jishen_score,
        balance_score,
        overall: (yongshen_score + jishen_score + balance_score) / 3,
    }
}
```

### 5. 性格匹配算法

```rust
pub fn calculate_personality_compatibility(
    xingge_a: &CompactXingGe,
    xingge_b: &CompactXingGe,
) -> PersonalityResult {
    // 互补性格分析
    let complement_score = analyze_complement_traits(xingge_a, xingge_b);
    
    // 冲突性格分析
    let conflict_score = analyze_conflict_traits(xingge_a, xingge_b);
    
    // 共同优点分析
    let common_score = analyze_common_strengths(xingge_a, xingge_b);
    
    PersonalityResult {
        complement_score,
        conflict_score,
        common_score,
        overall: calculate_weighted_score(complement_score, conflict_score, common_score),
    }
}
```

---

## 📊 数据结构

### 合婚请求

```rust
pub struct CompatibilityRequest<T: Config> {
    pub id: u64,
    pub party_a: T::AccountId,
    pub party_b: T::AccountId,
    pub party_a_bazi_id: u64,
    pub party_b_bazi_id: u64,
    pub status: MatchStatus,
    pub created_at: BlockNumberFor<T>,
    pub authorized_at: Option<BlockNumberFor<T>>,
}
```

### 合婚报告

```rust
pub struct CompatibilityReport<T: Config> {
    pub id: u64,
    pub request_id: u64,
    pub overall_score: u8,
    pub score_detail: CompatibilityScoreDetail,
    pub recommendation: MatchRecommendation,
    pub report_cid: Option<BoundedVec<u8, ConstU32<64>>>,
    pub generated_at: BlockNumberFor<T>,
    pub algorithm_version: u8,
}
```

---

## 💾 存储项

```rust
/// 合婚请求
pub type Requests<T> = StorageMap<_, Blake2_128Concat, u64, CompatibilityRequest<T>>;

/// 合婚报告
pub type Reports<T> = StorageMap<_, Blake2_128Concat, u64, CompatibilityReport<T>>;

/// 用户请求索引（甲方）
pub type UserRequestsAsPartyA<T> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<u64, T::MaxRequestsPerUser>>;

/// 用户请求索引（乙方）
pub type UserRequestsAsPartyB<T> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<u64, T::MaxRequestsPerUser>>;

/// 请求 ID 计数器
pub type NextRequestId<T> = StorageValue<_, u64, ValueQuery>;
```

---

## 🎯 外部调用（Extrinsics）

| 调用 | 描述 | 权限 |
|------|------|------|
| `create_request` | 创建合婚请求 | 甲方 |
| `authorize_request` | 授权合婚请求 | 乙方 |
| `reject_request` | 拒绝合婚请求 | 乙方 |
| `cancel_request` | 取消合婚请求 | 甲方 |
| `generate_report` | 生成合婚报告 | 甲方/乙方 |

---

## 📡 事件定义

```rust
RequestCreated { request_id, party_a, party_b }
RequestAuthorized { request_id, party_b }
RequestRejected { request_id, party_b }
RequestCancelled { request_id, cancelled_by }
ReportGenerated { report_id, request_id, overall_score, recommendation }
```

---

## ⚙️ 配置参数

```rust
impl pallet_matchmaking_matching::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type BaziProvider = BaziPallet;
    type MaxRequestsPerUser = ConstU32<100>;
    type RequestExpiration = ConstU32<100800>;  // 约7天
    type WeightInfo = ();
}
```

---

## 📚 相关文档

- [婚恋模块主文档](../README.md)
- [八字模块](../../divination/bazi/README.md)
