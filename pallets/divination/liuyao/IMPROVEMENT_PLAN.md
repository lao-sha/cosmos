# 六爻排盘模块改进计划

## 概述

根据对比分析 najia、divicast、hexagram 等参考项目，本计划旨在完善 pallet-liuyao 模块。

## 改进任务

### 任务1：修正六十四卦名索引 [优先级: 高]

**问题**：当前 `GUA_NAMES` 数组顺序与索引计算方式不匹配。

**修改文件**：`src/types.rs`

**修改内容**：
```rust
pub const GUA_NAMES: [&str; 64] = [
    "坤为地",    // 0  内坤外坤
    "雷地豫",    // 1  内震外坤
    "水地比",    // 2  内坎外坤
    "泽地萃",    // 3  内兑外坤
    "山地剥",    // 4  内艮外坤
    "火地晋",    // 5  内离外坤
    "风地观",    // 6  内巽外坤
    "天地否",    // 7  内乾外坤
    "地雷复",    // 8  内坤外震
    "震为雷",    // 9  内震外震
    "水雷屯",    // 10 内坎外震
    "泽雷随",    // 11 内兑外震
    "山雷颐",    // 12 内艮外震
    "火雷噬嗑",  // 13 内离外震
    "风雷益",    // 14 内巽外震
    "天雷无妄",  // 15 内乾外震
    "地水师",    // 16 内坤外坎
    "雷水解",    // 17 内震外坎
    "坎为水",    // 18 内坎外坎
    "泽水困",    // 19 内兑外坎
    "山水蒙",    // 20 内艮外坎
    "火水未济",  // 21 内离外坎
    "风水涣",    // 22 内巽外坎
    "天水讼",    // 23 内乾外坎
    "地泽临",    // 24 内坤外兑
    "雷泽归妹",  // 25 内震外兑
    "水泽节",    // 26 内坎外兑
    "兑为泽",    // 27 内兑外兑
    "山泽损",    // 28 内艮外兑
    "火泽睽",    // 29 内离外兑
    "风泽中孚",  // 30 内巽外兑
    "天泽履",    // 31 内乾外兑
    "地山谦",    // 32 内坤外艮
    "雷山小过",  // 33 内震外艮
    "水山蹇",    // 34 内坎外艮
    "泽山咸",    // 35 内兑外艮
    "艮为山",    // 36 内艮外艮
    "火山旅",    // 37 内离外艮
    "风山渐",    // 38 内巽外艮
    "天山遁",    // 39 内乾外艮
    "地火明夷",  // 40 内坤外离
    "雷火丰",    // 41 内震外离
    "水火既济",  // 42 内坎外离
    "泽火革",    // 43 内兑外离
    "山火贲",    // 44 内艮外离
    "离为火",    // 45 内离外离
    "风火家人",  // 46 内巽外离
    "天火同人",  // 47 内乾外离
    "地风升",    // 48 内坤外巽
    "雷风恒",    // 49 内震外巽
    "水风井",    // 50 内坎外巽
    "泽风大过",  // 51 内兑外巽
    "山风蛊",    // 52 内艮外巽
    "火风鼎",    // 53 内离外巽
    "巽为风",    // 54 内巽外巽
    "天风姤",    // 55 内乾外巽
    "地天泰",    // 56 内坤外乾
    "雷天大壮",  // 57 内震外乾
    "水天需",    // 58 内坎外乾
    "泽天夬",    // 59 内兑外乾
    "山天大畜",  // 60 内艮外乾
    "火天大有",  // 61 内离外乾
    "风天小畜",  // 62 内巽外乾
    "乾为天",    // 63 内乾外乾
];
```

**索引规则**：`index = (outer_trigram.binary() << 3) | inner_trigram.binary()`

其中八卦二进制（从下到上）：
- 坤=000(0), 震=001(1), 坎=010(2), 兑=011(3)
- 艮=100(4), 离=101(5), 巽=110(6), 乾=111(7)

---

### 任务2：完善六冲六合判断 [优先级: 高]

**修改文件**：`src/algorithm.rs`

**当前问题**：
- 六冲卦判断不完整
- 六合卦索引硬编码，可读性差

**修改内容**：

```rust
/// 六冲卦列表（按卦名）
/// 八纯卦 + 天雷无妄 + 雷天大壮
pub const LIU_CHONG_INDICES: [u8; 10] = [
    0,   // 坤为地
    9,   // 震为雷
    18,  // 坎为水
    27,  // 兑为泽
    36,  // 艮为山
    45,  // 离为火
    54,  // 巽为风
    63,  // 乾为天
    15,  // 天雷无妄
    57,  // 雷天大壮
];

/// 六合卦列表
/// 否、泰、困、节、贲、复、旅、豫
pub const LIU_HE_INDICES: [u8; 8] = [
    7,   // 天地否
    56,  // 地天泰
    19,  // 泽水困
    26,  // 水泽节
    44,  // 山火贲
    8,   // 地雷复
    37,  // 火山旅
    1,   // 雷地豫
];

/// 判断是否为六冲卦
pub fn is_liu_chong(gua_index: u8) -> bool {
    LIU_CHONG_INDICES.contains(&gua_index)
}

/// 判断是否为六合卦
pub fn is_liu_he(gua_index: u8) -> bool {
    LIU_HE_INDICES.contains(&gua_index)
}
```

---

### 任务3：改进数字起卦参数命名 [优先级: 中]

**修改文件**：`src/algorithm.rs`, `src/lib.rs`

**当前问题**：`num1`、`num2` 命名不清晰

**修改内容**：

```rust
/// 从两个数字起卦（报数法）
///
/// # 参数
/// - `upper_num`: 上卦数（对应外卦）
/// - `lower_num`: 下卦数（对应内卦）
/// - `dong`: 动爻位置（1-6）
pub fn numbers_to_yaos(upper_num: u16, lower_num: u16, dong: u8) -> [Yao; 6] {
    let inner_idx = ((lower_num - 1) % 8) as u8;
    let outer_idx = ((upper_num - 1) % 8) as u8;
    // ...
}
```

同步修改 `lib.rs` 中的 extrinsic 参数名和注释。

---

### 任务4：添加互卦计算 [优先级: 中]

**修改文件**：`src/algorithm.rs`, `src/types.rs`

**新增内容**：

```rust
// algorithm.rs
/// 计算互卦
///
/// 互卦取本卦的2、3、4爻为下卦，3、4、5爻为上卦
/// 爻位从1开始（初爻=1）
pub fn calculate_hu_gua(original_yaos: &[Yao; 6]) -> (Trigram, Trigram) {
    // 内卦取2,3,4爻（索引1,2,3）
    let inner_bin = (original_yaos[1].original_value()) |
                    (original_yaos[2].original_value() << 1) |
                    (original_yaos[3].original_value() << 2);
    // 外卦取3,4,5爻（索引2,3,4）
    let outer_bin = (original_yaos[2].original_value()) |
                    (original_yaos[3].original_value() << 1) |
                    (original_yaos[4].original_value() << 2);

    (Trigram::from_binary(inner_bin), Trigram::from_binary(outer_bin))
}

/// 计算互卦索引
pub fn calculate_hu_gua_index(original_yaos: &[Yao; 6]) -> u8 {
    let (inner, outer) = calculate_hu_gua(original_yaos);
    calculate_gua_index(inner, outer)
}
```

**修改 LiuYaoGua 结构体**：
```rust
// types.rs - 在 LiuYaoGua 中添加
/// 互卦内卦
pub hu_inner: Trigram,
/// 互卦外卦
pub hu_outer: Trigram,
/// 互卦卦名索引
pub hu_name_idx: u8,
```

---

### 任务5：添加卦身计算 [优先级: 中]

**修改文件**：`src/algorithm.rs`, `src/types.rs`

**算法说明**：
- 世爻为阳爻：从子起，数到世爻所在位置
- 世爻为阴爻：从午起，数到世爻所在位置

**新增内容**：

```rust
// algorithm.rs
/// 计算卦身
///
/// 世爻为阳爻从子起数，世爻为阴爻从午起数
/// 数到世爻所在位置的地支即为卦身
pub fn calculate_gua_shen(shi_pos: u8, shi_is_yang: bool) -> DiZhi {
    let start = if shi_is_yang { 0 } else { 6 }; // 子=0, 午=6
    DiZhi::from_index((start + shi_pos - 1) % 12)
}
```

**修改 LiuYaoGua 结构体**：
```rust
// types.rs - 在 LiuYaoGua 中添加
/// 卦身地支
pub gua_shen: DiZhi,
```

---

### 任务6：添加神煞系统 [优先级: 低]

**修改文件**：新建 `src/shensha.rs`, 修改 `src/types.rs`

**神煞列表**（按优先级）：
1. 天乙贵人
2. 驿马
3. 桃花
4. 禄神
5. 文昌
6. 劫煞
7. 华盖
8. 将星

**新建文件内容概要**：

```rust
// src/shensha.rs
//! 神煞计算模块

use crate::types::{DiZhi, TianGan};

/// 神煞类型
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum ShenSha {
    TianYiGuiRen,  // 天乙贵人
    YiMa,          // 驿马
    TaoHua,        // 桃花
    LuShen,        // 禄神
    WenChang,      // 文昌
    JieSha,        // 劫煞
    HuaGai,        // 华盖
    JiangXing,     // 将星
}

/// 计算天乙贵人
pub fn calculate_tian_yi_gui_ren(day_gan: TianGan) -> [DiZhi; 2] {
    match day_gan {
        TianGan::Jia | TianGan::Wu => [DiZhi::Chou, DiZhi::Wei],
        TianGan::Yi | TianGan::Ji => [DiZhi::Zi, DiZhi::Shen],
        TianGan::Bing | TianGan::Ding => [DiZhi::Hai, DiZhi::You],
        TianGan::Geng | TianGan::Xin => [DiZhi::Wu, DiZhi::Yin],
        TianGan::Ren | TianGan::Gui => [DiZhi::Mao, DiZhi::Si],
    }
}

/// 计算驿马（按日支）
pub fn calculate_yi_ma(day_zhi: DiZhi) -> DiZhi {
    match day_zhi {
        DiZhi::Shen | DiZhi::Zi | DiZhi::Chen => DiZhi::Yin,
        DiZhi::Si | DiZhi::You | DiZhi::Chou => DiZhi::Hai,
        DiZhi::Hai | DiZhi::Mao | DiZhi::Wei => DiZhi::Si,
        DiZhi::Yin | DiZhi::Wu | DiZhi::Xu => DiZhi::Shen,
    }
}

// ... 其他神煞计算函数
```

**说明**：神煞系统较复杂，建议作为可选功能，后续版本再完善。

---

### 任务7：增加单元测试 [优先级: 高]

**修改文件**：`src/tests.rs`

**测试用例**：

```rust
#[test]
fn test_gua_index_calculation() {
    // 乾为天: 内乾(111=7) 外乾(111=7) => (7<<3)|7 = 63
    assert_eq!(calculate_gua_index(Trigram::Qian, Trigram::Qian), 63);
    assert_eq!(gua64::GUA_NAMES[63], "乾为天");

    // 坤为地: 内坤(000=0) 外坤(000=0) => 0
    assert_eq!(calculate_gua_index(Trigram::Kun, Trigram::Kun), 0);
    assert_eq!(gua64::GUA_NAMES[0], "坤为地");

    // 天地否: 内乾(7) 外坤(0) => (0<<3)|7 = 7
    assert_eq!(calculate_gua_index(Trigram::Qian, Trigram::Kun), 7);
    assert_eq!(gua64::GUA_NAMES[7], "天地否");

    // 地天泰: 内坤(0) 外乾(7) => (7<<3)|0 = 56
    assert_eq!(calculate_gua_index(Trigram::Kun, Trigram::Qian), 56);
    assert_eq!(gua64::GUA_NAMES[56], "地天泰");
}

#[test]
fn test_najia() {
    // 离为火: 内离 外离
    // 内卦纳甲: 己卯、己丑、己亥
    // 外卦纳甲: 己酉、己未、己巳
    let inner = Trigram::Li;
    let outer = Trigram::Li;

    assert_eq!(get_inner_najia(inner, 0), (TianGan::Ji, DiZhi::Mao));
    assert_eq!(get_inner_najia(inner, 1), (TianGan::Ji, DiZhi::Chou));
    assert_eq!(get_inner_najia(inner, 2), (TianGan::Ji, DiZhi::Hai));

    assert_eq!(get_outer_najia(outer, 0), (TianGan::Ji, DiZhi::You));
    assert_eq!(get_outer_najia(outer, 1), (TianGan::Ji, DiZhi::Wei));
    assert_eq!(get_outer_najia(outer, 2), (TianGan::Ji, DiZhi::Si));
}

#[test]
fn test_xun_kong() {
    // 甲子日空亡戌亥
    assert_eq!(
        calculate_xun_kong(TianGan::Jia, DiZhi::Zi),
        (DiZhi::Xu, DiZhi::Hai)
    );

    // 甲寅日空亡子丑
    assert_eq!(
        calculate_xun_kong(TianGan::Jia, DiZhi::Yin),
        (DiZhi::Zi, DiZhi::Chou)
    );
}

#[test]
fn test_shi_ying_gong() {
    // 乾为天: 本宫六世，世在六爻
    let (gua_xu, gong) = calculate_shi_ying_gong(Trigram::Qian, Trigram::Qian);
    assert_eq!(gua_xu, GuaXu::BenGong);
    assert_eq!(gong, Trigram::Qian);
    assert_eq!(gua_xu.shi_yao_pos(), 6);
    assert_eq!(gua_xu.ying_yao_pos(), 3);
}

#[test]
fn test_liu_shen() {
    // 甲日起青龙
    let shen = calculate_liu_shen(TianGan::Jia);
    assert_eq!(shen[0], LiuShen::QingLong);
    assert_eq!(shen[1], LiuShen::ZhuQue);
    assert_eq!(shen[2], LiuShen::GouChen);

    // 丙日起朱雀
    let shen = calculate_liu_shen(TianGan::Bing);
    assert_eq!(shen[0], LiuShen::ZhuQue);
}

#[test]
fn test_liu_chong() {
    // 八纯卦都是六冲
    assert!(is_liu_chong(0));   // 坤为地
    assert!(is_liu_chong(63));  // 乾为天
    assert!(is_liu_chong(15));  // 天雷无妄
    assert!(is_liu_chong(57));  // 雷天大壮

    // 非六冲卦
    assert!(!is_liu_chong(7));  // 天地否
}

#[test]
fn test_hu_gua() {
    // 乾为天的互卦是乾为天
    let yaos = [Yao::ShaoYang; 6];
    let (inner, outer) = calculate_hu_gua(&yaos);
    assert_eq!(inner, Trigram::Qian);
    assert_eq!(outer, Trigram::Qian);
}
```

---

## 实施顺序

1. **第一阶段**（立即执行）✅ 已完成
   - [x] 任务1：修正六十四卦名索引
   - [x] 任务2：完善六冲六合判断
   - [x] 任务7：增加核心单元测试

2. **第二阶段**（短期）✅ 已完成
   - [x] 任务3：改进参数命名
   - [x] 任务4：添加互卦计算
   - [x] 任务5：添加卦身计算

3. **第三阶段**（中期）✅ 已完成
   - [x] 任务6：添加神煞系统（天乙贵人、驿马、桃花、禄神、文昌、劫煞、华盖、将星、亡神）

---

## 完成状态

**所有改进任务已完成！** 共计 49 个单元测试通过。

### 新增文件
- `src/shensha.rs` - 神煞计算模块

### 修改文件
- `src/types.rs` - 修正 GUA_NAMES 数组
- `src/algorithm.rs` - 添加六冲六合、互卦、卦身计算
- `src/lib.rs` - 导出神煞模块，改进参数命名
- `src/tests.rs` - 添加全面的单元测试

---

## 验证方法

每个任务完成后运行：

```bash
# 编译检查
SKIP_WASM_BUILD=1 cargo check -p pallet-liuyao

# 运行测试
SKIP_WASM_BUILD=1 cargo test -p pallet-liuyao
```

---

## 参考资料

- najia 项目：Python 六爻排盘实现
- divicast 项目：Python 六爻排盘，含神煞系统
- hexagram 项目：Java 六十四卦数据
- LiuYaoDivining 项目：设计文档参考
