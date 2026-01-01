# 六爻模块改进开发计划

## 概述

本文档详细规划四项改进任务的开发步骤：
1. 清理废弃的AI相关代码
2. 补充卦辞爻辞数据
3. 扩展神煞系统并集成到卦象
4. 添加地支关系判断（月破、进退神等）

---

## 第一阶段：清理废弃的AI相关代码

### 1.1 需要移除的代码

**lib.rs 中的废弃项：**

| 行号 | 内容 | 操作 |
|------|------|------|
| 105-107 | `AiInterpretationFee` 常量定义 | 移除 |
| 109-110 | `TreasuryAccount` 类型定义 | 移除 |
| 112-113 | `AiOracleOrigin` 类型定义 | 移除 |
| 161-164 | `AiInterpretationRequests` 存储项 | 移除 |
| 186-195 | `AiInterpretationRequested` 和 `AiInterpretationSubmitted` 事件 | 移除 |
| 225-230 | AI相关错误类型 | 移除 |
| 473-572 | `request_ai_interpretation` 和 `submit_ai_interpretation` 函数 | 移除 |

**types.rs 中的废弃项：**

| 行号 | 内容 | 操作 |
|------|------|------|
| 696-697 | `LiuYaoGua` 中的 `ai_interpretation_cid` 字段 | 移除 |
| 709-710 | `UserStats` 中的 `ai_interpretations` 字段 | 移除 |

### 1.2 开发步骤

```
步骤 1.1: 移除 Config trait 中的 AI 相关配置
  - 移除 AiInterpretationFee
  - 移除 TreasuryAccount
  - 移除 AiOracleOrigin

步骤 1.2: 移除存储项
  - 移除 AiInterpretationRequests

步骤 1.3: 移除事件
  - 移除 AiInterpretationRequested
  - 移除 AiInterpretationSubmitted

步骤 1.4: 移除错误类型
  - 移除 AiInterpretationAlreadyRequested
  - 移除 AiInterpretationNotRequested
  - 移除 InsufficientBalance (如果只用于AI功能)

步骤 1.5: 移除 Extrinsics
  - 移除 request_ai_interpretation
  - 移除 submit_ai_interpretation

步骤 1.6: 更新类型定义
  - 从 LiuYaoGua 移除 ai_interpretation_cid
  - 从 UserStats 移除 ai_interpretations

步骤 1.7: 更新 do_divine 函数
  - 移除创建卦象时的 ai_interpretation_cid: None

步骤 1.8: 运行测试确保编译通过
  - cargo test -p pallet-liuyao
```

---

## 第二阶段：补充卦辞爻辞数据

### 2.1 新建常量文件

创建 `src/constants.rs`，包含：
- 64卦卦辞
- 384爻爻辞（每卦6爻）
- 卦象图示符号

### 2.2 数据结构

```rust
// constants.rs

/// 六十四卦卦辞
/// 格式: (卦名, 卦辞, 彖辞摘要, 象辞摘要)
pub const GUA_CI: [GuaCi; 64] = [...];

pub struct GuaCi {
    pub name: &'static str,      // 卦名
    pub ci: &'static str,        // 卦辞
    pub tuan: &'static str,      // 彖辞（简）
    pub xiang: &'static str,     // 象辞（简）
}

/// 爻辞
/// 索引: [卦索引][爻位0-5]
pub const YAO_CI: [[&'static str; 6]; 64] = [...];

/// 用九用六（乾坤特殊）
pub const YONG_JIU: &str = "用九：见群龙无首，吉。";
pub const YONG_LIU: &str = "用六：利永贞。";
```

### 2.3 开发步骤

```
步骤 2.1: 创建 constants.rs 文件
  - 定义 GuaCi 结构体
  - 定义卦辞数组框架

步骤 2.2: 填充乾坤两卦数据（作为模板）
  - 乾为天完整卦辞爻辞
  - 坤为地完整卦辞爻辞

步骤 2.3: 填充剩余62卦数据
  - 按卦序依次填充
  - 参考周易原文

步骤 2.4: 在 lib.rs 中引入模块
  - pub mod constants;

步骤 2.5: 添加查询接口
  - get_gua_ci(index: u8) -> GuaCi
  - get_yao_ci(gua_index: u8, yao_pos: u8) -> &str

步骤 2.6: 测试验证
  - 测试所有64卦索引
  - 验证卦辞爻辞正确性
```

### 2.4 示例数据

```rust
pub const GUA_CI: [GuaCi; 64] = [
    // 0: 坤为地
    GuaCi {
        name: "坤",
        ci: "元亨，利牝马之贞。君子有攸往，先迷后得主，利西南得朋，东北丧朋。安贞吉。",
        tuan: "至哉坤元，万物资生，乃顺承天。",
        xiang: "地势坤，君子以厚德载物。",
    },
    // ... 其他卦
    // 63: 乾为天
    GuaCi {
        name: "乾",
        ci: "元亨利贞。",
        tuan: "大哉乾元，万物资始，乃统天。",
        xiang: "天行健，君子以自强不息。",
    },
];

pub const YAO_CI: [[&str; 6]; 64] = [
    // 0: 坤为地
    [
        "初六：履霜，坚冰至。",
        "六二：直方大，不习无不利。",
        "六三：含章可贞，或从王事，无成有终。",
        "六四：括囊，无咎无誉。",
        "六五：黄裳，元吉。",
        "上六：龙战于野，其血玄黄。",
    ],
    // ... 其他卦
];
```

---

## 第三阶段：扩展神煞系统并集成到卦象

### 3.1 扩展神煞类型

**新增5种神煞：**

| 神煞 | 说明 | 计算基准 |
|------|------|---------|
| 天喜 | 喜庆之神 | 年支 |
| 天医 | 医药之神 | 日支 |
| 阳刃 | 凶煞，主血光 | 日干 |
| 灾煞 | 灾祸之神 | 年支/日支 |
| 月德 | 吉神，主贵人 | 月支 |

### 3.2 更新 ShenSha 枚举

```rust
pub enum ShenSha {
    TianYiGuiRen = 0,
    YiMa = 1,
    TaoHua = 2,
    LuShen = 3,
    WenChang = 4,
    JieSha = 5,
    HuaGai = 6,
    JiangXing = 7,
    WangShen = 8,
    // 新增
    TianXi = 9,      // 天喜
    TianYi = 10,     // 天医
    YangRen = 11,    // 阳刃
    ZaiSha = 12,     // 灾煞
    YueDe = 13,      // 月德
}
```

### 3.3 更新 ShenShaInfo 结构

```rust
pub struct ShenShaInfo {
    pub tian_yi_gui_ren: [DiZhi; 2],
    pub yi_ma: DiZhi,
    pub tao_hua: DiZhi,
    pub lu_shen: DiZhi,
    pub wen_chang: DiZhi,
    pub jie_sha: DiZhi,
    pub hua_gai: DiZhi,
    pub jiang_xing: DiZhi,
    pub wang_shen: DiZhi,
    // 新增
    pub tian_xi: DiZhi,
    pub tian_yi: DiZhi,
    pub yang_ren: DiZhi,
    pub zai_sha: DiZhi,
    pub yue_de: DiZhi,
}
```

### 3.4 集成到卦象结构体

**在 LiuYaoGua 中新增字段：**

```rust
pub struct LiuYaoGua<...> {
    // ... 现有字段 ...

    /// 神煞信息
    pub shen_sha: ShenShaInfo,

    /// 每爻携带的神煞（位图）
    /// 每个元素是该爻携带的神煞位图
    pub yao_shen_sha: [u16; 6],
}
```

### 3.5 开发步骤

```
步骤 3.1: 扩展 ShenSha 枚举
  - 添加 TianXi, TianYi, YangRen, ZaiSha, YueDe

步骤 3.2: 添加新神煞计算函数
  - calculate_tian_xi(year_zhi) -> DiZhi
  - calculate_tian_yi(day_zhi) -> DiZhi
  - calculate_yang_ren(day_gan) -> DiZhi
  - calculate_zai_sha(day_zhi) -> DiZhi
  - calculate_yue_de(month_zhi) -> DiZhi

步骤 3.3: 更新 ShenShaInfo 结构
  - 添加新字段
  - 更新 Default 实现

步骤 3.4: 更新 calculate_all_shen_sha 函数
  - 增加月支参数
  - 计算所有新神煞

步骤 3.5: 修改 LiuYaoGua 结构体
  - 添加 shen_sha: ShenShaInfo 字段
  - 添加 yao_shen_sha: [u16; 6] 字段

步骤 3.6: 修改 do_divine 函数
  - 计算神煞信息
  - 计算每爻携带的神煞
  - 存储到卦象中

步骤 3.7: 添加辅助函数
  - calculate_yao_shen_sha() 计算单爻神煞
  - shen_sha_to_bitmap() 转换为位图

步骤 3.8: 测试验证
  - 测试新神煞计算
  - 测试集成后的卦象
```

### 3.6 神煞计算公式

```rust
/// 天喜（年支起）
/// 口诀：子见酉，丑见申，寅见未...
const TIAN_XI: [DiZhi; 12] = [
    DiZhi::You,  // 子 -> 酉
    DiZhi::Shen, // 丑 -> 申
    DiZhi::Wei,  // 寅 -> 未
    DiZhi::Wu,   // 卯 -> 午
    DiZhi::Si,   // 辰 -> 巳
    DiZhi::Chen, // 巳 -> 辰
    DiZhi::Mao,  // 午 -> 卯
    DiZhi::Yin,  // 未 -> 寅
    DiZhi::Chou, // 申 -> 丑
    DiZhi::Zi,   // 酉 -> 子
    DiZhi::Hai,  // 戌 -> 亥
    DiZhi::Xu,   // 亥 -> 戌
];

/// 阳刃（日干起）
/// 口诀：甲刃在卯，乙刃在辰，丙戊刃在午...
const YANG_REN: [DiZhi; 10] = [
    DiZhi::Mao,  // 甲 -> 卯
    DiZhi::Chen, // 乙 -> 辰
    DiZhi::Wu,   // 丙 -> 午
    DiZhi::Wei,  // 丁 -> 未
    DiZhi::Wu,   // 戊 -> 午
    DiZhi::Wei,  // 己 -> 未
    DiZhi::You,  // 庚 -> 酉
    DiZhi::Xu,   // 辛 -> 戌
    DiZhi::Zi,   // 壬 -> 子
    DiZhi::Chou, // 癸 -> 丑
];
```

---

## 第四阶段：添加地支关系判断

### 4.1 需要添加的功能

| 功能 | 说明 | 用途 |
|------|------|------|
| 六冲 | 子午冲、丑未冲等 | 判断爻与爻、爻与日月的冲克 |
| 六合 | 子丑合、寅亥合等 | 判断爻与爻、爻与日月的相合 |
| 三合 | 申子辰合水等 | 判断三合局成局 |
| 三刑 | 寅巳申刑等 | 判断刑害关系 |
| 月破 | 爻被月建所冲 | 判断爻的衰旺 |
| 日破 | 爻被日建所冲 | 判断爻的衰旺 |
| 进神 | 动爻变化后五行增强 | 判断事物发展 |
| 退神 | 动爻变化后五行减弱 | 判断事物发展 |
| 旺相休囚 | 爻在月令的状态 | 判断爻的力量 |

### 4.2 新建关系模块

创建 `src/relations.rs`：

```rust
//! # 地支关系判断模块
//!
//! 实现六爻断卦中的地支关系判断

/// 六冲关系
pub const LIU_CHONG: [(DiZhi, DiZhi); 6] = [
    (DiZhi::Zi, DiZhi::Wu),   // 子午冲
    (DiZhi::Chou, DiZhi::Wei), // 丑未冲
    (DiZhi::Yin, DiZhi::Shen), // 寅申冲
    (DiZhi::Mao, DiZhi::You),  // 卯酉冲
    (DiZhi::Chen, DiZhi::Xu),  // 辰戌冲
    (DiZhi::Si, DiZhi::Hai),   // 巳亥冲
];

/// 六合关系
pub const LIU_HE: [(DiZhi, DiZhi, WuXing); 6] = [
    (DiZhi::Zi, DiZhi::Chou, WuXing::Earth),  // 子丑合土
    (DiZhi::Yin, DiZhi::Hai, WuXing::Wood),   // 寅亥合木
    (DiZhi::Mao, DiZhi::Xu, WuXing::Fire),    // 卯戌合火
    (DiZhi::Chen, DiZhi::You, WuXing::Metal), // 辰酉合金
    (DiZhi::Si, DiZhi::Shen, WuXing::Water),  // 巳申合水
    (DiZhi::Wu, DiZhi::Wei, WuXing::Fire),    // 午未合火
];

/// 三合局
pub const SAN_HE: [(DiZhi, DiZhi, DiZhi, WuXing); 4] = [
    (DiZhi::Shen, DiZhi::Zi, DiZhi::Chen, WuXing::Water), // 申子辰合水
    (DiZhi::Hai, DiZhi::Mao, DiZhi::Wei, WuXing::Wood),   // 亥卯未合木
    (DiZhi::Yin, DiZhi::Wu, DiZhi::Xu, WuXing::Fire),     // 寅午戌合火
    (DiZhi::Si, DiZhi::You, DiZhi::Chou, WuXing::Metal),  // 巳酉丑合金
];

/// 三刑关系
pub const SAN_XING: [(DiZhi, DiZhi, DiZhi); 4] = [
    (DiZhi::Yin, DiZhi::Si, DiZhi::Shen),   // 寅巳申无恩之刑
    (DiZhi::Chou, DiZhi::Xu, DiZhi::Wei),   // 丑戌未恃势之刑
    (DiZhi::Zi, DiZhi::Mao, DiZhi::Zi),     // 子卯无礼之刑（自刑）
    (DiZhi::Chen, DiZhi::Wu, DiZhi::You),   // 辰午酉亥自刑
];
```

### 4.3 爻状态结构

```rust
/// 爻的状态信息
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct YaoState {
    /// 是否月破
    pub is_yue_po: bool,
    /// 是否日破
    pub is_ri_po: bool,
    /// 是否旬空
    pub is_xun_kong: bool,
    /// 是否入墓
    pub is_ru_mu: bool,
    /// 旺相休囚死状态
    pub wang_xiang: WangXiang,
    /// 与月建关系
    pub month_relation: Option<ZhiRelation>,
    /// 与日建关系
    pub day_relation: Option<ZhiRelation>,
}

/// 旺相休囚死
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub enum WangXiang {
    #[default]
    Wang = 0,   // 旺 - 当令
    Xiang = 1,  // 相 - 得令生
    Xiu = 2,    // 休 - 生令
    Qiu = 3,    // 囚 - 克令
    Si = 4,     // 死 - 被令克
}

/// 地支关系
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum ZhiRelation {
    Chong,      // 冲
    He,         // 合
    Sheng,      // 生
    Ke,         // 克
    BiHe,       // 比和
}
```

### 4.4 进退神判断

```rust
/// 进神退神判断
///
/// 进神：动爻变化后，地支五行相同但进入更旺状态
/// 退神：动爻变化后，地支五行相同但退入更衰状态
///
/// 十二长生顺序：长生、沐浴、冠带、临官、帝旺、衰、病、死、墓、绝、胎、养
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub enum JinTuiShen {
    /// 进神 - 寅进卯、巳进午、申进酉、亥进子
    JinShen,
    /// 退神 - 卯退寅、午退巳、酉退申、子退亥
    TuiShen,
    /// 非进退
    None,
}

/// 进神地支对
const JIN_SHEN: [(DiZhi, DiZhi); 4] = [
    (DiZhi::Yin, DiZhi::Mao),   // 寅进卯（木）
    (DiZhi::Si, DiZhi::Wu),     // 巳进午（火）
    (DiZhi::Shen, DiZhi::You),  // 申进酉（金）
    (DiZhi::Hai, DiZhi::Zi),    // 亥进子（水）
];

/// 判断进退神
pub fn calculate_jin_tui_shen(original_zhi: DiZhi, changed_zhi: DiZhi) -> JinTuiShen {
    for (jin_from, jin_to) in JIN_SHEN.iter() {
        if original_zhi == *jin_from && changed_zhi == *jin_to {
            return JinTuiShen::JinShen;
        }
        if original_zhi == *jin_to && changed_zhi == *jin_from {
            return JinTuiShen::TuiShen;
        }
    }
    JinTuiShen::None
}
```

### 4.5 开发步骤

```
步骤 4.1: 创建 relations.rs 模块
  - 定义六冲、六合、三合、三刑常量
  - 实现基础判断函数

步骤 4.2: 添加地支方法扩展
  - DiZhi::chong() -> DiZhi  获取冲位
  - DiZhi::he() -> Option<(DiZhi, WuXing)>  获取合位
  - DiZhi::is_chong(other) -> bool
  - DiZhi::is_he(other) -> bool

步骤 4.3: 实现月破日破判断
  - is_yue_po(yao_zhi, month_zhi) -> bool
  - is_ri_po(yao_zhi, day_zhi) -> bool

步骤 4.4: 实现旺相休囚判断
  - calculate_wang_xiang(yao_wx, month_zhi) -> WangXiang

步骤 4.5: 实现进退神判断
  - calculate_jin_tui_shen(original, changed) -> JinTuiShen

步骤 4.6: 添加 YaoState 结构
  - 定义爻状态结构体
  - 实现状态计算函数

步骤 4.7: 集成到 YaoInfo
  - 在 YaoInfo 中添加 state: YaoState 字段
  - 或创建扩展查询函数

步骤 4.8: 在 lib.rs 中引入模块
  - pub mod relations;
  - pub use relations::*;

步骤 4.9: 更新 do_divine 计算爻状态
  - 计算每爻的月破、日破
  - 计算旺相休囚

步骤 4.10: 添加查询接口
  - get_yao_state(gua_id, yao_pos) -> YaoState
  - get_yao_relations(gua_id) -> 爻间关系

步骤 4.11: 测试验证
  - 测试六冲六合判断
  - 测试月破日破判断
  - 测试进退神判断
```

---

## 开发时间估算

| 阶段 | 任务 | 预计工作量 |
|------|------|-----------|
| 第一阶段 | 清理AI代码 | 简单 |
| 第二阶段 | 补充卦辞爻辞 | 中等（数据录入为主） |
| 第三阶段 | 扩展神煞系统 | 中等 |
| 第四阶段 | 地支关系判断 | 较复杂 |

---

## 依赖关系

```
第一阶段（独立）
    ↓
第二阶段（独立）
    ↓
第三阶段（依赖：types.rs 结构变更）
    ↓
第四阶段（依赖：第三阶段完成后进行）
```

建议按顺序执行，每阶段完成后运行测试确保稳定。

---

## 文件变更清单

| 文件 | 变更类型 | 说明 |
|------|---------|------|
| `src/lib.rs` | 修改 | 清理AI代码、集成新模块 |
| `src/types.rs` | 修改 | 更新结构体、添加新类型 |
| `src/shensha.rs` | 修改 | 扩展神煞 |
| `src/constants.rs` | 新建 | 卦辞爻辞数据 |
| `src/relations.rs` | 新建 | 地支关系判断 |
| `src/tests.rs` | 修改 | 添加新测试 |

---

## 测试计划

每个阶段完成后执行：

```bash
# 编译检查
cargo check -p pallet-liuyao

# 运行测试
cargo test -p pallet-liuyao

# 代码格式化
cargo fmt -p pallet-liuyao

# Clippy 检查
cargo clippy -p pallet-liuyao
```
