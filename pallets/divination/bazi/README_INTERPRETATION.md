# 八字解盘模块 (Ba Zi Interpretation Module)

## 概述

八字解盘模块是八字排盘 Pallet 的核心功能之一，提供完整的中国传统命理解盘分析。本模块基于已计算的四柱八字数据，进行深度的命理分析和人生指导。

## 功能特性

### 🎯 核心分析功能

1. **格局分析 (GeJu Analysis)**
   - 正格：身旺财官，五行平衡
   - 从强格：身旺无制，顺势发展
   - 从弱格：身弱无助，借助外力
   - 从财格：财星当令
   - 从官格：官星当令
   - 从儿格：食伤当令
   - 化气格：干支化合
   - 特殊格局：其他特殊情况

2. **命局强弱分析 (Strength Analysis)**
   - 身旺：日主强势，自主性强
   - 身弱：日主较弱，需要扶助
   - 中和：五行平衡，发展顺遂
   - 太旺：过于强势，需要制约
   - 太弱：极度虚弱，需要大力扶助

3. **用神分析 (Beneficial Element Analysis)**
   - 扶抑用神：扶弱抑强，平衡命局
   - 调候用神：调节寒暖，改善气候
   - 通关用神：化解冲突，协调关系
   - 专旺用神：顺势而为，发挥优势

4. **性格分析 (Personality Analysis)**
   - 基于日主天干的性格特征
   - 优点和缺点分析
   - 适合的职业方向
   - 人际关系特点

5. **综合评分 (Comprehensive Score)**
   - 格局评分：不同格局的基础分数
   - 平衡评分：五行平衡程度
   - 综合评分：0-100分的整体评价

## 技术实现

### 数据结构

```rust
/// 解盘结果
pub struct JiePanResult {
    /// 格局类型
    pub ge_ju: GeJuType,
    /// 命局强弱
    pub qiang_ruo: MingJuQiangRuo,
    /// 用神
    pub yong_shen: WuXing,
    /// 用神类型
    pub yong_shen_type: YongShenType,
    /// 忌神 (最多3个)
    pub ji_shen: BoundedVec<WuXing, ConstU32<3>>,
    /// 性格分析
    pub xing_ge: XingGeTeZheng,
    /// 综合评分 (0-100)
    pub zong_he_ping_fen: u8,
    /// 解盘文本 (最多10条)
    pub jie_pan_text: BoundedVec<&'static str, ConstU32<10>>,
}
```

### 核心算法

#### 1. 格局判断算法

```rust
pub fn analyze_ge_ju(sizhu: &SiZhu<T>, wuxing_strength: &WuXingStrength) -> GeJuType {
    let rizhu_wuxing = sizhu.rizhu.to_wuxing();
    let rizhu_strength = get_wuxing_strength(wuxing_strength, rizhu_wuxing);
    let total_strength = /* 计算总强度 */;
    let strength_ratio = (rizhu_strength * 100) / total_strength;
    
    // 根据强度比例判断格局
    if strength_ratio >= 60 {
        // 检查是否有制约 -> 正格 or 从强格
    } else if strength_ratio <= 20 {
        // 检查是否有生扶 -> 正格 or 从弱格
    } else {
        GeJuType::ZhengGe
    }
}
```

#### 2. 用神分析算法

```rust
pub fn analyze_yong_shen(
    ge_ju: GeJuType,
    qiang_ruo: MingJuQiangRuo,
    sizhu: &SiZhu<T>,
    wuxing_strength: &WuXingStrength,
) -> (WuXing, YongShenType) {
    match (ge_ju, qiang_ruo) {
        (GeJuType::ZhengGe, MingJuQiangRuo::ShenWang) => {
            // 身旺用克泄耗
            (get_ke_wo(rizhu_wuxing), YongShenType::FuYi)
        },
        (GeJuType::ZhengGe, MingJuQiangRuo::ShenRuo) => {
            // 身弱用生扶
            (get_sheng_wo(rizhu_wuxing), YongShenType::FuYi)
        },
        // ... 其他情况
    }
}
```

#### 3. 五行生克关系

```rust
// 生我者为印
fn get_sheng_wo(wuxing: WuXing) -> WuXing {
    match wuxing {
        WuXing::Jin => WuXing::Tu,   // 土生金
        WuXing::Mu => WuXing::Shui,  // 水生木
        WuXing::Shui => WuXing::Jin, // 金生水
        WuXing::Huo => WuXing::Mu,   // 木生火
        WuXing::Tu => WuXing::Huo,   // 火生土
    }
}

// 克我者为官杀
fn get_ke_wo(wuxing: WuXing) -> WuXing {
    match wuxing {
        WuXing::Jin => WuXing::Huo,   // 火克金
        WuXing::Mu => WuXing::Jin,    // 金克木
        WuXing::Shui => WuXing::Tu,   // 土克水
        WuXing::Huo => WuXing::Shui,  // 水克火
        WuXing::Tu => WuXing::Mu,     // 木克土
    }
}
```

## 使用方法

### 1. 创建八字并解盘

```rust
// 1. 创建八字
let result = BaziChart::create_bazi_chart(
    origin,
    1990, 11, 15, 14, 30,  // 出生时间
    Gender::Male,
    ZiShiMode::Modern,
);

// 2. 获取八字ID
let charts = BaziChart::bazi_charts(&account_id);
let chart_id = /* 计算chart_id */;

// 3. 执行解盘
let result = BaziChart::interpret_bazi_chart(
    origin,
    chart_id,
);

// 4. 获取解盘结果
let interpretation = BaziChart::interpretation_by_id(&chart_id);
```

### 2. 直接调用解盘函数

```rust
use crate::interpretation::*;

// 使用已有的四柱和五行强度数据
let interpretation_result = full_interpretation(
    &sizhu,
    &wuxing_strength,
);

println!("格局: {:?}", interpretation_result.ge_ju);
println!("强弱: {:?}", interpretation_result.qiang_ruo);
println!("用神: {:?}", interpretation_result.yong_shen);
println!("评分: {}", interpretation_result.zong_he_ping_fen);
```

## 解盘示例

### 示例1：正格身弱

```
出生时间: 1990年11月15日14:30 (男)
四柱: 庚午年 丁亥月 戊午日 己未时
日主: 戊土

分析结果:
- 格局: 正格
- 强弱: 身弱
- 用神: 火 (印星生扶)
- 忌神: 水 (官杀克身)
- 评分: 75分

性格特征:
- 优点: 稳重、可靠、有责任心
- 缺点: 保守、变化慢
- 适合职业: 房地产、建筑、农业、服务

解盘文本:
1. 命局为正格，五行相对平衡，发展较为稳定。
2. 日主偏弱，需要贵人相助，宜团队合作。
3. 宜从事能源、娱乐、化工相关行业。
```

### 示例2：从强格

```
出生时间: 1985年6月8日10:15 (女)
四柱: 乙丑年 壬午月 丙午日 癸巳时
日主: 丙火

分析结果:
- 格局: 从强格
- 强弱: 身旺
- 用神: 火 (顺势而为)
- 忌神: 水、金
- 评分: 82分

性格特征:
- 优点: 热情、开朗、有领导力
- 缺点: 急躁、缺乏耐心
- 适合职业: 能源、娱乐、餐饮、化工

解盘文本:
1. 命局为从强格，日主旺盛，宜顺势发展。
2. 日主偏旺，自主性强，但需注意克制。
3. 宜从事能源、娱乐、化工相关行业。
```

## 评分标准

### 格局评分
- 正格: +20分
- 从强格/从弱格: +15分
- 其他格局: +10分

### 强弱评分
- 中和: +20分
- 身旺/身弱: +10分
- 太旺/太弱: +5分

### 五行平衡评分
- 五行相对平衡: +10分
- 五行失衡: +0分

### 基础分: 50分

## 扩展功能

### 1. 流年大运分析 (待实现)

```rust
/// 流年运势分析
pub fn analyze_liu_nian(
    chart: &BaziChart<T>,
    target_year: u16,
) -> LiuNianYunShi {
    // 分析指定年份的运势
}

/// 大运分析
pub fn analyze_da_yun(
    chart: &BaziChart<T>,
    current_age: u8,
) -> DaYunAnalysis {
    // 分析当前大运的影响
}
```

### 2. 合婚分析 (待实现)

```rust
/// 八字合婚分析
pub fn analyze_he_hun(
    male_chart: &BaziChart<T>,
    female_chart: &BaziChart<T>,
) -> HeHunResult {
    // 分析两个八字的匹配度
}
```

### 3. 择日分析 (待实现)

```rust
/// 择日分析
pub fn analyze_ze_ri(
    chart: &BaziChart<T>,
    event_type: EventType,
    date_range: (u16, u8, u8, u16, u8, u8),
) -> Vec<GoodDate> {
    // 分析适合的日期
}
```

## 注意事项

1. **准确性说明**: 本模块提供的解盘结果仅供参考，不应作为人生决策的唯一依据。

2. **算法局限**: 当前实现是简化版算法，真实的八字解盘需要考虑更多复杂因素：
   - 节气的精确影响
   - 地域时差
   - 真太阳时
   - 更复杂的格局判断

3. **文化背景**: 八字命理是中国传统文化的重要组成部分，使用时应尊重其文化内涵。

4. **技术限制**: 
   - 使用 `BoundedVec` 限制了数据长度
   - `no_std` 环境限制了某些功能
   - 静态字符串限制了文本的动态生成

## 测试

模块包含全面的测试用例：

```bash
# 运行所有测试
cargo test

# 运行解盘相关测试
cargo test interpretation

# 运行集成测试
cargo test test_interpretation_integration
```

## 贡献指南

欢迎贡献代码来改进八字解盘模块：

1. **算法优化**: 改进格局判断和用神分析算法
2. **功能扩展**: 添加流年大运、合婚分析等功能
3. **测试完善**: 增加更多测试用例
4. **文档改进**: 完善文档和示例

## 参考资料

- 《滴天髓》- 古代命理经典
- 《穷通宝鉴》- 调候用神理论
- 《子平真诠》- 格局理论基础
- 现代八字软件算法参考

## 版权声明

本模块遵循 MIT-0 许可证，可自由使用和修改。
