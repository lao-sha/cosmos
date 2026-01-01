# 奇门遁甲解卦数据结构设计

## 1. 设计概述

本文档定义了奇门遁甲占卜系统的解卦数据结构，参考了八字和梅花易数的设计经验，结合奇门遁甲的特点，实现轻量化、可扩展的链上存储方案。

### 1.1 设计原则

1. **链上存储最小化**：核心指标存储在链上，详细解读通过 Runtime API 实时计算
2. **隐私保护**：敏感信息（问题、姓名）仅存储哈希值
3. **可扩展性**：预留扩展字段，支持未来功能升级
4. **AI友好**：结构化数据便于AI解读和分析
5. **前端友好**：提供完整的查询API，减少前端计算负担

### 1.2 参考模块对比

| 模块 | 核心存储 | 扩展数据 | 特点 |
|------|---------|---------|------|
| 八字 | 13 bytes | 性格+职业 | 分层存储，Runtime API计算 |
| 梅花 | 体用分析+应期 | 辅助卦象 | 完整解卦数据 |
| 奇门 | 格局+用神+吉凶 | 宫位详解 | 综合两者优点 |

## 2. 核心数据结构

### 2.1 Layer 1: 核心解卦指标（链上存储）

```rust
/// 奇门遁甲核心解卦结果
///
/// 存储空间优化，总大小约 20-25 bytes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct QimenCoreInterpretation {
    /// 格局类型 (1 byte)
    pub ge_ju: GeJuType,

    /// 用神宫位 (1 byte, 1-9)
    pub yong_shen_gong: u8,

    /// 值符星 (1 byte)
    pub zhi_fu_xing: JiuXing,

    /// 值使门 (1 byte)
    pub zhi_shi_men: BaMen,

    /// 日干落宫 (1 byte, 1-9)
    pub ri_gan_gong: u8,

    /// 时干落宫 (1 byte, 1-9)
    pub shi_gan_gong: u8,

    /// 综合吉凶 (1 byte)
    pub fortune: Fortune,

    /// 吉凶等级 0-100 (1 byte)
    pub fortune_score: u8,

    /// 旺衰状态 (1 byte)
    pub wang_shuai: WangShuai,

    /// 特殊格局标记 (1 byte, 位标志)
    /// bit 0: 伏吟
    /// bit 1: 反吟
    /// bit 2: 天遁
    /// bit 3: 地遁
    /// bit 4: 人遁
    /// bit 5: 鬼遁
    /// bit 6: 神遁
    /// bit 7: 龙遁
    pub special_patterns: u8,

    /// 可信度 0-100 (1 byte)
    pub confidence: u8,

    /// 解盘时间戳 - 区块号 (4 bytes)
    pub timestamp: u32,

    /// 算法版本 (1 byte)
    pub algorithm_version: u8,
}
```

**存储大小**: 约 16 bytes

### 2.2 格局类型枚举

```rust
/// 奇门遁甲格局类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum GeJuType {
    /// 正格 - 常规格局
    ZhengGe,

    /// 伏吟格 - 天盘地盘相同
    FuYinGe,

    /// 反吟格 - 天盘地盘对冲
    FanYinGe,

    /// 天遁格 - 丙奇+天心星+开门
    TianDunGe,

    /// 地遁格 - 乙奇+六合+开门
    DiDunGe,

    /// 人遁格 - 丁奇+太阴+开门
    RenDunGe,

    /// 鬼遁格 - 丁奇+天心星+开门
    GuiDunGe,

    /// 神遁格 - 九天+值符+开门
    ShenDunGe,

    /// 龙遁格 - 九地+值符+开门
    LongDunGe,

    /// 青龙返首 - 特殊吉格
    QingLongFanShou,

    /// 飞鸟跌穴 - 特殊凶格
    FeiNiaoDieXue,
}
```

### 2.3 旺衰状态

```rust
/// 旺衰状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum WangShuai {
    /// 旺相 - 得令得时
    WangXiang,

    /// 相 - 次旺
    Xiang,

    /// 休 - 休息
    Xiu,

    /// 囚 - 受制
    Qiu,

    /// 死 - 最弱
    Si,
}
```

### 2.4 吉凶等级

```rust
/// 吉凶等级
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum Fortune {
    /// 大吉
    DaJi,

    /// 中吉
    ZhongJi,

    /// 小吉
    XiaoJi,

    /// 平
    Ping,

    /// 小凶
    XiaoXiong,

    /// 中凶
    ZhongXiong,

    /// 大凶
    DaXiong,
}
```

## 3. Layer 2: 扩展解卦数据（Runtime API 计算）

### 3.1 宫位详细分析

```rust
/// 单宫详细解读
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct PalaceInterpretation {
    /// 宫位
    pub gong: JiuGong,

    /// 天盘干
    pub tian_pan_gan: TianGan,

    /// 地盘干
    pub di_pan_gan: TianGan,

    /// 九星
    pub xing: JiuXing,

    /// 八门
    pub men: Option<BaMen>,

    /// 八神
    pub shen: Option<BaShen>,

    /// 宫位五行
    pub gong_wuxing: WuXing,

    /// 天盘五行
    pub tian_wuxing: WuXing,

    /// 地盘五行
    pub di_wuxing: WuXing,

    /// 星门关系
    pub xing_men_relation: XingMenRelation,

    /// 宫位旺衰
    pub wang_shuai: WangShuai,

    /// 是否伏吟
    pub is_fu_yin: bool,

    /// 是否反吟
    pub is_fan_yin: bool,

    /// 是否旬空
    pub is_xun_kong: bool,

    /// 是否马星
    pub is_ma_xing: bool,

    /// 宫位吉凶
    pub fortune: Fortune,

    /// 吉凶评分 0-100
    pub fortune_score: u8,
}
```

### 3.2 星门关系

```rust
/// 星门关系
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum XingMenRelation {
    /// 星生门 - 吉
    XingShengMen,

    /// 门生星 - 平
    MenShengXing,

    /// 星克门 - 凶
    XingKeMen,

    /// 门克星 - 平
    MenKeXing,

    /// 比和 - 吉
    BiHe,
}
```

### 3.3 用神分析

```rust
/// 用神分析结果
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct YongShenAnalysis {
    /// 问事类型
    pub question_type: QuestionType,

    /// 主用神宫位
    pub primary_gong: JiuGong,

    /// 主用神类型
    pub primary_type: YongShenType,

    /// 次用神宫位（可选）
    pub secondary_gong: Option<JiuGong>,

    /// 次用神类型（可选）
    pub secondary_type: Option<YongShenType>,

    /// 用神旺衰
    pub wang_shuai: WangShuai,

    /// 用神得力情况
    pub de_li: DeLiStatus,

    /// 用神吉凶
    pub fortune: Fortune,

    /// 用神评分 0-100
    pub score: u8,
}
```

### 3.4 用神类型

```rust
/// 用神类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum YongShenType {
    /// 日干 - 代表自己
    RiGan,

    /// 时干 - 代表事情
    ShiGan,

    /// 值符 - 代表贵人
    ZhiFu,

    /// 值使 - 代表行动
    ZhiShi,

    /// 年命 - 代表本命
    NianMing,

    /// 特定星 - 根据问事类型
    SpecificXing(JiuXing),

    /// 特定门 - 根据问事类型
    SpecificMen(BaMen),

    /// 特定宫 - 根据问事类型
    SpecificGong(JiuGong),
}
```

### 3.5 得力状态

```rust
/// 得力状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum DeLiStatus {
    /// 大得力 - 旺相+吉星吉门
    DaDeLi,

    /// 得力 - 旺相或吉星吉门
    DeLi,

    /// 平 - 休囚但无克
    Ping,

    /// 失力 - 休囚+凶星凶门
    ShiLi,

    /// 大失力 - 死绝+凶星凶门
    DaShiLi,
}
```

## 4. Layer 3: 应期推算

### 4.1 应期分析

```rust
/// 应期推算结果
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct YingQiAnalysis {
    /// 主应期数（基于用神宫位）
    pub primary_num: u8,

    /// 次应期数（基于值符值使）
    pub secondary_nums: [u8; 2],

    /// 应期单位
    pub unit: YingQiUnit,

    /// 应期范围描述
    pub range_desc: BoundedVec<u8, ConstU32<128>>,

    /// 吉利时间
    pub auspicious_times: BoundedVec<u8, ConstU32<64>>,

    /// 不利时间
    pub inauspicious_times: BoundedVec<u8, ConstU32<64>>,
}
```

### 4.2 应期单位

```rust
/// 应期单位
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum YingQiUnit {
    /// 时辰
    Hour,

    /// 日
    Day,

    /// 旬（10天）
    Xun,

    /// 月
    Month,

    /// 季
    Season,

    /// 年
    Year,
}
```

## 5. 完整解卦数据结构

### 5.1 完整解读结果

```rust
/// 奇门遁甲完整解读结果
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct QimenFullInterpretation {
    /// 核心指标（必有）
    pub core: QimenCoreInterpretation,

    /// 九宫详细解读（可选）
    pub palaces: Option<[PalaceInterpretation; 9]>,

    /// 用神分析（可选）
    pub yong_shen: Option<YongShenAnalysis>,

    /// 应期推算（可选）
    pub ying_qi: Option<YingQiAnalysis>,

    /// 格局详解（可选）
    pub ge_ju_detail: Option<GeJuDetail>,
}
```

### 5.2 格局详解

```rust
/// 格局详解
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct GeJuDetail {
    /// 格局类型
    pub ge_ju: GeJuType,

    /// 格局名称
    pub name: BoundedVec<u8, ConstU32<32>>,

    /// 格局描述
    pub description: BoundedVec<u8, ConstU32<256>>,

    /// 格局吉凶
    pub fortune: Fortune,

    /// 适用场景
    pub applicable_scenarios: BoundedVec<QuestionType, ConstU32<8>>,

    /// 注意事项
    pub notes: BoundedVec<u8, ConstU32<256>>,
}
```

## 6. 解卦算法设计

### 6.1 核心解卦流程

```rust
/// 计算核心解卦（Layer 1）
///
/// 免费实时计算，不存储
pub fn calculate_core_interpretation<T: Config>(
    chart: &QimenChart<T::AccountId, BlockNumberFor<T>, T::MaxCidLen>,
    current_block: u32,
) -> QimenCoreInterpretation {
    // 1. 分析格局
    let ge_ju = analyze_ge_ju(chart);

    // 2. 确定用神宫位
    let yong_shen_gong = determine_yong_shen_gong(chart);

    // 3. 计算日干时干落宫
    let ri_gan_gong = find_gan_palace(&chart.palaces, chart.day_ganzhi.gan);
    let shi_gan_gong = find_gan_palace(&chart.palaces, chart.hour_ganzhi.gan);

    // 4. 分析旺衰
    let wang_shuai = analyze_wang_shuai(chart, yong_shen_gong);

    // 5. 检测特殊格局
    let special_patterns = detect_special_patterns(chart);

    // 6. 计算综合吉凶
    let (fortune, fortune_score) = calculate_fortune(
        ge_ju,
        wang_shuai,
        chart.zhi_fu_xing,
        chart.zhi_shi_men,
        special_patterns,
    );

    // 7. 计算可信度
    let confidence = calculate_confidence(chart, ge_ju);

    QimenCoreInterpretation {
        ge_ju,
        yong_shen_gong,
        zhi_fu_xing: chart.zhi_fu_xing,
        zhi_shi_men: chart.zhi_shi_men,
        ri_gan_gong,
        shi_gan_gong,
        fortune,
        fortune_score,
        wang_shuai,
        special_patterns,
        confidence,
        timestamp: current_block,
        algorithm_version: 1,
    }
}
```

### 6.2 格局分析算法

```rust
/// 分析格局
fn analyze_ge_ju<T: Config>(
    chart: &QimenChart<T::AccountId, BlockNumberFor<T>, T::MaxCidLen>,
) -> GeJuType {
    // 1. 检查伏吟（天盘地盘相同）
    if is_fu_yin(&chart.palaces) {
        return GeJuType::FuYinGe;
    }

    // 2. 检查反吟（天盘地盘对冲）
    if is_fan_yin(&chart.palaces) {
        return GeJuType::FanYinGe;
    }

    // 3. 检查三遁（天遁、地遁、人遁）
    if let Some(dun_ge) = check_san_dun(&chart.palaces) {
        return dun_ge;
    }

    // 4. 检查鬼遁、神遁、龙遁
    if let Some(special_ge) = check_special_dun(&chart.palaces) {
        return special_ge;
    }

    // 5. 检查特殊吉凶格局
    if let Some(special_ge) = check_special_patterns(&chart.palaces) {
        return special_ge;
    }

    // 6. 默认为正格
    GeJuType::ZhengGe
}
```

### 6.3 旺衰分析算法

```rust
/// 分析旺衰
fn analyze_wang_shuai<T: Config>(
    chart: &QimenChart<T::AccountId, BlockNumberFor<T>, T::MaxCidLen>,
    yong_shen_gong: u8,
) -> WangShuai {
    let palace = &chart.palaces[(yong_shen_gong - 1) as usize];
    let yong_shen_wuxing = palace.tian_pan_gan.wu_xing();

    // 根据节气判断旺衰
    let jie_qi_wuxing = get_jie_qi_wuxing(chart.jie_qi);

    if yong_shen_wuxing == jie_qi_wuxing {
        WangShuai::WangXiang // 当令为旺
    } else if jie_qi_wuxing.generates(&yong_shen_wuxing) {
        WangShuai::Xiang // 生我为相
    } else if yong_shen_wuxing.generates(&jie_qi_wuxing) {
        WangShuai::Xiu // 我生为休
    } else if jie_qi_wuxing.conquers(&yong_shen_wuxing) {
        WangShuai::Qiu // 克我为囚
    } else {
        WangShuai::Si // 我克为死
    }
}
```

### 6.4 吉凶计算算法

```rust
/// 计算综合吉凶
fn calculate_fortune(
    ge_ju: GeJuType,
    wang_shuai: WangShuai,
    zhi_fu_xing: JiuXing,
    zhi_shi_men: BaMen,
    special_patterns: u8,
) -> (Fortune, u8) {
    let mut score = 50u8;

    // 1. 格局分 (0-20)
    score += match ge_ju {
        GeJuType::TianDunGe | GeJuType::DiDunGe | GeJuType::RenDunGe => 20,
        GeJuType::ShenDunGe | GeJuType::LongDunGe => 15,
        GeJuType::ZhengGe => 10,
        GeJuType::FuYinGe | GeJuType::FanYinGe => 0,
        _ => 5,
    };

    // 2. 旺衰分 (0-15)
    score += match wang_shuai {
        WangShuai::WangXiang => 15,
        WangShuai::Xiang => 12,
        WangShuai::Xiu => 8,
        WangShuai::Qiu => 4,
        WangShuai::Si => 0,
    };

    // 3. 值符分 (0-10)
    if zhi_fu_xing.is_auspicious() {
        score += 10;
    }

    // 4. 值使分 (0-10)
    if zhi_shi_men.is_auspicious() {
        score += 10;
    }

    // 5. 特殊格局加分 (0-15)
    let special_count = special_patterns.count_ones();
    score += (special_count as u8 * 3).min(15);

    score = score.min(100);

    // 根据分数确定吉凶等级
    let fortune = match score {
        90..=100 => Fortune::DaJi,
        75..=89 => Fortune::ZhongJi,
        60..=74 => Fortune::XiaoJi,
        45..=59 => Fortune::Ping,
        30..=44 => Fortune::XiaoXiong,
        15..=29 => Fortune::ZhongXiong,
        _ => Fortune::DaXiong,
    };

    (fortune, score)
}
```

## 7. 存储优化策略

### 7.1 存储层次

| 层次 | 数据 | 存储位置 | 大小 |
|------|------|---------|------|
| Layer 1 | 核心指标 | 链上 | ~16 bytes |
| Layer 2 | 宫位详解 | Runtime API | 实时计算 |
| Layer 3 | 应期推算 | Runtime API | 实时计算 |
| Layer 4 | AI解读 | IPFS | CID存储 |

### 7.2 优化效果

- **八字模块**: 13 bytes（81% 优化）
- **梅花模块**: 完整存储（约 200+ bytes）
- **奇门模块**: 16 bytes（预计 85% 优化）

## 8. 使用示例

### 8.1 获取核心解卦

```rust
// 通过 Runtime API 获取核心解卦
let core = runtime_api.get_qimen_core_interpretation(chart_id)?;

println!("格局: {:?}", core.ge_ju);
println!("吉凶: {:?} ({}分)", core.fortune, core.fortune_score);
println!("旺衰: {:?}", core.wang_shuai);
println!("可信度: {}%", core.confidence);
```

### 8.2 获取完整解卦

```rust
// 通过 Runtime API 获取完整解卦
let full = runtime_api.get_qimen_full_interpretation(chart_id)?;

// 核心指标
println!("核心指标: {:?}", full.core);

// 九宫详解
if let Some(palaces) = full.palaces {
    for palace in palaces.iter() {
        println!("{}宫: {:?}", palace.gong.name(), palace.fortune);
    }
}

// 用神分析
if let Some(yong_shen) = full.yong_shen {
    println!("用神: {:?} ({}分)", yong_shen.fortune, yong_shen.score);
}

// 应期推算
if let Some(ying_qi) = full.ying_qi {
    println!("应期: {} {:?}", ying_qi.primary_num, ying_qi.unit);
}
```

## 9. 总结

本设计方案综合了八字和梅花易数的优点：

1. **轻量化存储**：核心指标仅 16 bytes，实现 85% 存储优化
2. **实时计算**：通过 Runtime API 免费获取详细解读
3. **算法升级**：无需数据迁移，立即生效
4. **隐私保护**：敏感信息仅存储哈希值
5. **AI友好**：结构化数据便于AI解读

该设计既保证了链上存储的经济性，又提供了完整的解卦功能，适合区块链环境下的奇门遁甲占卜系统。
