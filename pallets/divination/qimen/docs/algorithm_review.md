# 奇门遁甲排盘算法对比分析报告

## 概述

本文档对比分析了 `pallets/divination/qimen` 模块与参考项目 `xuanxue/qimen` 的实现差异，识别潜在错误并提出优化建议。

**参考项目**：
- `xuanxue/qimen/xuan-utils-pro/src/main/java/xuan/core/qimen/` - Java 实现
- `xuanxue/qimen/qimen/lib/` - JavaScript 实现
- `xuanxue/qimen/qiqi/qimen_algorithm_design.md` - 算法设计文档

**分析目标**：
- `pallets/divination/qimen/src/types.rs` - 数据类型定义
- `pallets/divination/qimen/src/algorithm.rs` - 核心排盘算法

---

## 一、局数表验证

### 1.1 阳遁局数表对比

| 节气 | Pallet `YANG_DUN_JU` | Java `JU_SHU` | 状态 |
|------|----------------------|---------------|------|
| 冬至 | [1, 7, 4] | [1, 7, 4] | ✅ 正确 |
| 小寒 | [2, 8, 5] | [2, 8, 5] | ✅ 正确 |
| 大寒 | [3, 9, 6] | [3, 9, 6] | ✅ 正确 |
| 立春 | [8, 5, 2] | [8, 5, 2] | ✅ 正确 |
| 雨水 | [9, 6, 3] | [9, 6, 3] | ✅ 正确 |
| 惊蛰 | [1, 7, 4] | [1, 7, 4] | ✅ 正确 |
| 春分 | [3, 9, 6] | [3, 9, 6] | ✅ 正确 |
| 清明 | [4, 1, 7] | [4, 1, 7] | ✅ 正确 |
| 谷雨 | [5, 2, 8] | [5, 2, 8] | ✅ 正确 |
| 立夏 | [4, 1, 7] | [4, 1, 7] | ✅ 正确 |
| 小满 | [5, 2, 8] | [5, 2, 8] | ✅ 正确 |
| 芒种 | [6, 3, 9] | [6, 3, 9] | ✅ 正确 |

### 1.2 阴遁局数表对比

| 节气 | Pallet `YIN_DUN_JU` | Java `JU_SHU` | 状态 |
|------|---------------------|---------------|------|
| 夏至 | [9, 3, 6] | [9, 3, 6] | ✅ 正确 |
| 小暑 | [8, 2, 5] | [8, 2, 5] | ✅ 正确 |
| 大暑 | [7, 1, 4] | [7, 1, 4] | ✅ 正确 |
| 立秋 | [2, 5, 8] | [2, 5, 8] | ✅ 正确 |
| 处暑 | [1, 4, 7] | [1, 4, 7] | ✅ 正确 |
| 白露 | [9, 3, 6] | [9, 3, 6] | ✅ 正确 |
| 秋分 | [7, 1, 4] | [7, 1, 4] | ✅ 正确 |
| 寒露 | [6, 9, 3] | [6, 9, 3] | ✅ 正确 |
| 霜降 | [5, 8, 2] | [5, 8, 2] | ✅ 正确 |
| 立冬 | [6, 9, 3] | [6, 9, 3] | ✅ 正确 |
| 小雪 | [5, 8, 2] | [5, 8, 2] | ✅ 正确 |
| 大雪 | [4, 7, 1] | [4, 7, 1] | ✅ 正确 |

**结论**：局数表完全正确，与传统口诀一致。

> 阳遁口诀：冬至惊蛰一七四，小寒二八五，大寒春分三九六...
> 阴遁口诀：夏至白露九三六，小暑八二五，大暑秋分七一四...

---

## 二、发现的问题

### 2.1 地盘三奇六仪排布算法 ⚠️ 潜在错误

**位置**：`algorithm.rs:163-206` `get_di_pan()` 函数

**问题描述**：

当前实现对阴遁和阳遁使用相同的三奇六仪顺序：

```rust
// 当前实现 - 阴阳遁使用相同顺序
let san_qi_liu_yi = [
    TianGan::Wu, TianGan::Ji, TianGan::Geng, TianGan::Xin,
    TianGan::Ren, TianGan::Gui, TianGan::Ding, TianGan::Bing, TianGan::Yi,
];
```

**参考实现**（Java `QiMenZhuanPanJiChuMap.java:815-850`）：

```java
// 阳遁：六仪在前，三奇在后（顺排）
// 戊 → 己 → 庚 → 辛 → 壬 → 癸 → 丁 → 丙 → 乙
DI_YANG_QI_YI.put(1, Arrays.asList("戊", "己", "庚", "辛", "壬", "癸", "丁", "丙", "乙"));

// 阴遁：六仪逆行，三奇顺行（不同的排布逻辑！）
// 戊 → 乙 → 丙 → 丁 → 癸 → 壬 → 辛 → 庚 → 己
DI_YIN_QI_YI.put(1, Arrays.asList("戊", "乙", "丙", "丁", "癸", "壬", "辛", "庚", "己"));
```

**关键差异**：

| 局数 | 宫位 | 阳遁一局 | 阴遁一局 |
|------|------|----------|----------|
| 1 | 坎一宫 | 戊 | 戊 |
| 2 | 坤二宫 | 己 | 乙 ⚠️ |
| 3 | 震三宫 | 庚 | 丙 ⚠️ |
| 4 | 巽四宫 | 辛 | 丁 ⚠️ |
| 5 | 中五宫 | 壬 | 癸 ⚠️ |
| 6 | 乾六宫 | 癸 | 壬 ⚠️ |
| 7 | 兑七宫 | 丁 | 辛 ⚠️ |
| 8 | 艮八宫 | 丙 | 庚 ⚠️ |
| 9 | 离九宫 | 乙 | 己 ⚠️ |

**建议修复**：

```rust
/// 阳遁地盘三奇六仪（1-9局，1-9宫）
pub const DI_YANG_QI_YI: [[TianGan; 9]; 9] = [
    // 阳遁一局（1-9宫）
    [TianGan::Wu, TianGan::Ji, TianGan::Geng, TianGan::Xin,
     TianGan::Ren, TianGan::Gui, TianGan::Ding, TianGan::Bing, TianGan::Yi],
    // 阳遁二局
    [TianGan::Yi, TianGan::Wu, TianGan::Ji, TianGan::Geng,
     TianGan::Xin, TianGan::Ren, TianGan::Gui, TianGan::Ding, TianGan::Bing],
    // ... 继续到九局
];

/// 阴遁地盘三奇六仪（1-9局，1-9宫）
pub const DI_YIN_QI_YI: [[TianGan; 9]; 9] = [
    // 阴遁一局（1-9宫）
    [TianGan::Wu, TianGan::Yi, TianGan::Bing, TianGan::Ding,
     TianGan::Gui, TianGan::Ren, TianGan::Xin, TianGan::Geng, TianGan::Ji],
    // 阴遁二局
    [TianGan::Ji, TianGan::Wu, TianGan::Yi, TianGan::Bing,
     TianGan::Ding, TianGan::Gui, TianGan::Ren, TianGan::Xin, TianGan::Geng],
    // ... 继续到九局
];

/// 优化后的地盘获取函数
pub fn get_di_pan(ju_number: u8, dun_type: DunType) -> [TianGan; 9] {
    let idx = (ju_number - 1) as usize;
    match dun_type {
        DunType::Yang => DI_YANG_QI_YI[idx],
        DunType::Yin => DI_YIN_QI_YI[idx],
    }
}
```

---

### 2.2 旬空计算算法 ⚠️ 需要验证

**位置**：`algorithm.rs:232-244` `get_xun_kong()` 函数

**当前实现**：
```rust
pub fn get_xun_kong(shi_gan: TianGan, shi_zhi: DiZhi) -> (DiZhi, DiZhi) {
    let gan_idx = shi_gan.index();
    let zhi_idx = shi_zhi.index();
    let xun_shou_zhi = (zhi_idx + 12 - gan_idx) % 12;
    let kong1 = DiZhi::from_index((xun_shou_zhi + 10) % 12).unwrap_or(DiZhi::Zi);
    let kong2 = DiZhi::from_index((xun_shou_zhi + 11) % 12).unwrap_or(DiZhi::Chou);
    (kong1, kong2)
}
```

**参考实现**（查找表方式）：

```java
// QiMenZhuanPanJiChuMap.java:913-930
LIU_JIA_XUN_KONG.put("甲子", Arrays.asList("戌", "亥")); // 甲子旬空戌亥
LIU_JIA_XUN_KONG.put("甲戌", Arrays.asList("申", "酉")); // 甲戌旬空申酉
LIU_JIA_XUN_KONG.put("甲申", Arrays.asList("午", "未")); // 甲申旬空午未
LIU_JIA_XUN_KONG.put("甲午", Arrays.asList("辰", "巳")); // 甲午旬空辰巳
LIU_JIA_XUN_KONG.put("甲辰", Arrays.asList("寅", "卯")); // 甲辰旬空寅卯
LIU_JIA_XUN_KONG.put("甲寅", Arrays.asList("子", "丑")); // 甲寅旬空子丑
```

**建议**：使用查找表更可靠，避免边界计算错误。

```rust
/// 六甲旬空表
pub const XUN_KONG_TABLE: [(DiZhi, DiZhi); 6] = [
    (DiZhi::Xu, DiZhi::Hai),   // 甲子旬
    (DiZhi::Shen, DiZhi::You), // 甲戌旬
    (DiZhi::Wu, DiZhi::Wei),   // 甲申旬
    (DiZhi::Chen, DiZhi::Si),  // 甲午旬
    (DiZhi::Yin, DiZhi::Mao),  // 甲辰旬
    (DiZhi::Zi, DiZhi::Chou),  // 甲寅旬
];

pub fn get_xun_kong(shi_gan: TianGan, shi_zhi: DiZhi) -> (DiZhi, DiZhi) {
    let sexagenary = GanZhi::new(shi_gan, shi_zhi).sexagenary_index();
    let xun_index = (sexagenary / 10) as usize;
    XUN_KONG_TABLE[xun_index]
}
```

---

### 2.3 九宫飞布顺序 ✅ 正确

**位置**：`algorithm.rs:90-98`

```rust
pub const GONG_ORDER_YANG: [u8; 8] = [1, 8, 3, 4, 9, 2, 7, 6];
pub const GONG_ORDER_YIN: [u8; 8] = [1, 6, 7, 2, 9, 4, 3, 8];
```

与 `types.rs` 中 `JiuGong::next_yang()` / `next_yin()` 方法一致，正确实现了洛书九宫顺逆飞布。

---

## 三、缺失的功能

对比参考实现 `QiMenZhuanPanJiChuMap.java`，当前 pallet 缺少以下重要功能：

### 3.1 六仪击刑

**参考定义** (`QiMenZhuanPanJiChuMap.java:2046-2057`)：

| 天干 | 宫位 | 说明 |
|------|------|------|
| 戊 | 震三宫 | 戊击刑 |
| 己 | 坤二宫 | 己击刑 |
| 庚 | 艮八宫 | 庚击刑 |
| 辛 | 离九宫 | 辛击刑 |
| 壬 | 巽四宫 | 壬击刑 |
| 癸 | 巽四宫 | 癸击刑 |

**建议实现**：

```rust
/// 检测六仪击刑
pub fn check_liu_yi_ji_xing(tian_pan_gan: TianGan, gong: JiuGong) -> Option<&'static str> {
    match (tian_pan_gan, gong) {
        (TianGan::Wu, JiuGong::Zhen) => Some("戊击刑（震三宫）"),
        (TianGan::Ji, JiuGong::Kun) => Some("己击刑（坤二宫）"),
        (TianGan::Geng, JiuGong::Gen) => Some("庚击刑（艮八宫）"),
        (TianGan::Xin, JiuGong::Li) => Some("辛击刑（离九宫）"),
        (TianGan::Ren, JiuGong::Xun) => Some("壬击刑（巽四宫）"),
        (TianGan::Gui, JiuGong::Xun) => Some("癸击刑（巽四宫）"),
        _ => None,
    }
}
```

---

### 3.2 奇仪入墓

**参考定义** (`QiMenZhuanPanJiChuMap.java:2062-2076`)：

| 天干 | 入墓宫位 |
|------|----------|
| 甲 | 坤二宫（未土） |
| 乙 | 乾六宫（戌土） |
| 丙 | 乾六宫（戌土） |
| 丁 | 艮八宫（丑土） |
| 戊 | 乾六宫（戌土） |
| 己 | 艮八宫（丑土） |
| 庚 | 艮八宫（丑土） |
| 辛 | 巽四宫（辰土） |
| 壬 | 巽四宫（辰土） |
| 癸 | 坤二宫（未土） |

**建议实现**：

```rust
/// 检测奇仪入墓
pub fn check_qi_yi_ru_mu(tian_pan_gan: TianGan, gong: JiuGong) -> Option<&'static str> {
    match (tian_pan_gan, gong) {
        (TianGan::Jia, JiuGong::Kun) => Some("甲入墓（坤二宫）"),
        (TianGan::Yi, JiuGong::Qian) => Some("乙入墓（乾六宫）"),
        (TianGan::Bing, JiuGong::Qian) => Some("丙入墓（乾六宫）"),
        (TianGan::Ding, JiuGong::Gen) => Some("丁入墓（艮八宫）"),
        (TianGan::Wu, JiuGong::Qian) => Some("戊入墓（乾六宫）"),
        (TianGan::Ji, JiuGong::Gen) => Some("己入墓（艮八宫）"),
        (TianGan::Geng, JiuGong::Gen) => Some("庚入墓（艮八宫）"),
        (TianGan::Xin, JiuGong::Xun) => Some("辛入墓（巽四宫）"),
        (TianGan::Ren, JiuGong::Xun) => Some("壬入墓（巽四宫）"),
        (TianGan::Gui, JiuGong::Kun) => Some("癸入墓（坤二宫）"),
        _ => None,
    }
}
```

---

### 3.3 门迫

**参考定义** (`QiMenZhuanPanJiChuMap.java:2082-2099`)：

八门五行克落宫五行时为门迫：

| 八门 | 门迫宫位 |
|------|----------|
| 休门（水） | 离九宫（火） |
| 生门（土） | 坎一宫（水） |
| 伤门（木） | 艮八宫（土）、坤二宫（土） |
| 杜门（木） | 艮八宫（土）、坤二宫（土） |
| 景门（火） | 兑七宫（金）、乾六宫（金） |
| 死门（土） | 坎一宫（水） |
| 惊门（金） | 震三宫（木）、巽四宫（木） |
| 开门（金） | 震三宫（木）、巽四宫（木） |

---

### 3.4 十干克应

**参考定义** (`QiMenZhuanPanJiChuMap.java:1109-1200`)：

天盘干与地盘干相遇的吉凶格局，例如：
- 乙+乙：日奇伏吟
- 丙+戊：飞鸟跌穴（大吉）
- 庚+庚：太白同宫（凶）

---

### 3.5 驿马

**参考定义** (`QiMenZhuanPanJiChuMap.java:951-982`)：

根据时支确定驿马位置：

| 时支 | 驿马地支 | 落宫 |
|------|----------|------|
| 申、子、辰 | 寅 | 艮八宫 |
| 寅、午、戌 | 申 | 坤二宫 |
| 巳、酉、丑 | 亥 | 乾六宫 |
| 亥、卯、未 | 巳 | 巽四宫 |

---

### 3.6 九星/八门旺衰

根据季节或月支判断旺衰状态（旺、相、休、囚、死/废）。

---

## 四、优化建议

### 4.1 架构优化：使用查找表

参考 Java 实现，将动态计算改为预计算的查找表：

**优点**：
1. 执行效率高（O(1) 查表 vs O(n) 计算）
2. 减少运行时错误风险
3. 便于验证和测试
4. 符合传统奇门排盘的"定式"概念

**示例**：

```rust
// 九星顺飞查找表（天蓬落各宫时的九星分布）
pub const JIU_XING_SHUN: [[JiuXing; 9]; 8] = [
    // 天蓬落坎一宫
    [TianPeng, TianRui, TianChong, TianFu, TianQin, TianXin, TianZhu, TianRen, TianYing],
    // 天蓬落坤二宫
    [TianFu, TianPeng, TianRui, TianZhu, TianQin, TianChong, TianRen, TianYing, TianXin],
    // ... 继续
];
```

---

### 4.2 增加格局判断结构

建议在 `Palace` 结构体中增加格局状态：

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct PalaceStatus {
    /// 是否六仪击刑
    pub liu_yi_ji_xing: bool,
    /// 是否奇仪入墓
    pub qi_yi_ru_mu: bool,
    /// 是否门迫
    pub men_po: bool,
    /// 是否伏吟
    pub fu_yin: bool,
    /// 是否反吟
    pub fan_yin: bool,
}
```

---

### 4.3 三元判断方法改进

当前使用"节气内天数"判断三元，传统奇门更常用"日柱干支"。

**参考** (`QiMenZhuanPanJiChuMap.java:583-647`)：

```java
// 根据日柱判断三元
RI_ZHU_SAN_YUAN.put("甲子", "上元");
RI_ZHU_SAN_YUAN.put("甲午", "上元");
RI_ZHU_SAN_YUAN.put("甲申", "中元");
RI_ZHU_SAN_YUAN.put("甲寅", "中元");
RI_ZHU_SAN_YUAN.put("甲戌", "下元");
RI_ZHU_SAN_YUAN.put("甲辰", "下元");
// ...
```

---

## 五、测试验证建议

### 5.1 单元测试用例

```rust
#[test]
fn test_di_pan_yang_dun_1() {
    let di_pan = get_di_pan(1, DunType::Yang);
    // 阳遁一局：坎一宫戊、坤二宫己、震三宫庚...
    assert_eq!(di_pan[0], TianGan::Wu);  // 坎一宫
    assert_eq!(di_pan[1], TianGan::Ji);  // 坤二宫
    assert_eq!(di_pan[2], TianGan::Geng); // 震三宫
    assert_eq!(di_pan[8], TianGan::Yi);  // 离九宫
}

#[test]
fn test_di_pan_yin_dun_1() {
    let di_pan = get_di_pan(1, DunType::Yin);
    // 阴遁一局：坎一宫戊、坤二宫乙、震三宫丙...
    assert_eq!(di_pan[0], TianGan::Wu);   // 坎一宫
    assert_eq!(di_pan[1], TianGan::Yi);   // 坤二宫 ⚠️ 与阳遁不同
    assert_eq!(di_pan[2], TianGan::Bing); // 震三宫 ⚠️ 与阳遁不同
}

#[test]
fn test_xun_kong() {
    // 甲子时，旬空戌亥
    let (kong1, kong2) = get_xun_kong(TianGan::Jia, DiZhi::Zi);
    assert_eq!(kong1, DiZhi::Xu);
    assert_eq!(kong2, DiZhi::Hai);

    // 甲戌时，旬空申酉
    let (kong1, kong2) = get_xun_kong(TianGan::Jia, DiZhi::Xu);
    assert_eq!(kong1, DiZhi::Shen);
    assert_eq!(kong2, DiZhi::You);
}
```

---

## 六、总结

### 正确实现 ✅

| 模块 | 状态 |
|------|------|
| 阳遁局数表 | ✅ 正确 |
| 阴遁局数表 | ✅ 正确 |
| 九宫飞布顺序 | ✅ 正确 |
| 基础数据类型 | ✅ 完善 |
| 值符值使计算 | ✅ 基本正确 |

### 需要修正 ⚠️

| 模块 | 问题 | 优先级 |
|------|------|--------|
| 阴遁地盘排布 | 三奇六仪顺序可能有误 | 高 |
| 旬空算法 | 建议改用查找表 | 中 |

### 建议添加 📝

| 功能 | 重要性 |
|------|--------|
| 六仪击刑检测 | 高 |
| 奇仪入墓检测 | 高 |
| 门迫检测 | 高 |
| 十干克应 | 中 |
| 驿马计算 | 中 |
| 旺衰判断 | 中 |
| 星门克应 | 低 |

---

## 附录：参考文件索引

| 文件 | 说明 |
|------|------|
| `xuan-utils-pro/.../QiMenZhuanPanJiChuMap.java` | 完整的常量映射表 |
| `xuan-utils-pro/.../QiMenZhuanPan.java` | 排盘主逻辑 |
| `qimen/lib/qimen.js` | JavaScript 实现 |
| `qimen/lib/dipan.js` | 地盘配置 |
| `qimen/lib/jiuxing.js` | 九星配置 |
| `qiqi/qimen_algorithm_design.md` | 算法设计文档 |

---

*文档生成日期：2025-12-03*
*分析版本：pallets/divination/qimen v1.0*
