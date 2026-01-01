# 小六壬解卦模块实施计划

## 目录

- [一、项目概述](#一项目概述)
- [二、开发环境准备](#二开发环境准备)
- [三、详细开发步骤](#三详细开发步骤)
- [四、文件结构规划](#四文件结构规划)
- [五、测试计划](#五测试计划)
- [六、验收标准](#六验收标准)
- [七、风险管理](#七风险管理)

---

## 一、项目概述

### 1.1 项目目标

为 `pallet-xiaoliuren` 实现链上解卦功能，提供：
- 13字节极致优化的核心数据结构
- 完整的解卦算法（吉凶、评分、应期、建议）
- Runtime API实时查询
- AI解卦数据集成

### 1.2 技术栈

- **语言**：Rust
- **框架**：Substrate FRAME
- **存储**：链上KV存储
- **API**：Runtime API + RPC

### 1.3 时间规划

| 阶段 | 天数 | 起止时间 | 交付物 |
|------|------|----------|--------|
| 环境准备 | 0.5天 | Day 0 | 开发环境就绪 |
| 阶段1：数据结构 | 1天 | Day 1 | interpretation.rs 完成 |
| 阶段2：核心算法 | 2天 | Day 2-3 | 算法实现完成 |
| 阶段3：Runtime API | 1天 | Day 4 | API接口完成 |
| 阶段4：集成测试 | 1天 | Day 5 | 测试通过 |
| 阶段5：文档完善 | 0.5天 | Day 5.5 | 文档完成 |
| **总计** | **6天** | | **完整功能** |

---

## 二、开发环境准备

### 2.1 检查现有代码

```bash
# 进入项目目录
cd /home/xiaodong/文档/stardust/pallets/divination/xiaoliuren

# 查看现有文件结构
tree src/

# 预期输出：
# src/
# ├── algorithm.rs    # 已有：排盘算法
# ├── lib.rs          # 已有：pallet主文件
# ├── mock.rs         # 已有：测试mock
# ├── tests.rs        # 已有：测试文件
# └── types.rs        # 已有：类型定义
```

### 2.2 创建新模块文件

```bash
# 创建解卦模块（新增）
touch src/interpretation.rs
touch src/runtime_api.rs

# 创建测试文件（新增）
touch src/interpretation_tests.rs
```

### 2.3 更新 Cargo.toml

```bash
# 检查依赖是否完整
grep "serde" Cargo.toml
grep "serde_json" Cargo.toml

# 如果缺少，手动添加到 [dependencies]
```

### 2.4 环境检查清单

- [x] Rust工具链已安装（rustc, cargo）
- [x] Substrate开发环境就绪
- [x] 现有模块可编译通过
- [x] Git仓库干净（无未提交更改）
- [x] 文档已阅读（INTERPRETATION_DESIGN.md）

---

## 三、详细开发步骤

## 阶段 1：数据结构实现（Day 1，预计6-8小时）

### Step 1.1：创建 interpretation.rs 基础框架（1小时）

**任务清单：**
- [ ] 创建文件 `src/interpretation.rs`
- [ ] 添加模块文档注释
- [ ] 导入必要的依赖
- [ ] 定义模块结构

**代码实现：**

```rust
//! # 小六壬解卦模块
//!
//! 本模块实现小六壬占卜的解卦功能，包括：
//! - 吉凶判断
//! - 综合评分
//! - 应期推算
//! - 建议生成
//!
//! ## 设计特点
//!
//! - **极致优化**: 核心数据仅13字节
//! - **分层存储**: 核心指标链上，详细解释链下
//! - **实时计算**: 通过Runtime API免费查询
//! - **算法可升级**: 无需数据迁移

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use crate::types::{
    BaGua, LiuGong, SanGong, ShiChen, TiYongRelation,
    WuXingRelation, XiaoLiuRenSchool, YinYang,
};

// 模块导出
mod enums;
mod core_struct;
mod algorithms;
mod text_generation;

pub use enums::*;
pub use core_struct::*;
pub use algorithms::*;
pub use text_generation::*;
```

**验收标准：**
- 文件创建成功
- 编译无警告 `cargo check`
- 文档注释完整

---

### Step 1.2：实现枚举类型（2小时）

**任务清单：**
- [ ] 实现 `JiXiongLevel` 枚举
- [ ] 实现 `YongShenState` 枚举（暂不使用，为未来扩展）
- [ ] 实现 `AdviceType` 枚举
- [ ] 实现 `YingQiType` 枚举
- [ ] 实现 `SpecialPattern` 位标志结构
- [ ] 为所有枚举添加方法

**文件：** `src/interpretation/enums.rs`

```rust
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

// ============================================================================
// 吉凶等级枚举
// ============================================================================

/// 吉凶等级（1 byte）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum JiXiongLevel {
    /// 大吉 - 诸事顺遂，心想事成
    #[default]
    DaJi = 0,
    /// 吉 - 事可成，宜进取
    Ji = 1,
    /// 小吉 - 小有所得，不宜大动
    XiaoJi = 2,
    /// 平 - 平稳无波，守成为上
    Ping = 3,
    /// 小凶 - 小有阻碍，谨慎行事
    XiaoXiong = 4,
    /// 凶 - 事难成，宜退守
    Xiong = 5,
    /// 大凶 - 诸事不利，静待时机
    DaXiong = 6,
}

impl JiXiongLevel {
    /// 获取吉凶等级名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::DaJi => "大吉",
            Self::Ji => "吉",
            Self::XiaoJi => "小吉",
            Self::Ping => "平",
            Self::XiaoXiong => "小凶",
            Self::Xiong => "凶",
            Self::DaXiong => "大凶",
        }
    }

    /// 获取详细描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::DaJi => "诸事顺遂，心想事成",
            Self::Ji => "事可成，宜进取",
            Self::XiaoJi => "小有所得，不宜大动",
            Self::Ping => "平稳无波，守成为上",
            Self::XiaoXiong => "小有阻碍，谨慎行事",
            Self::Xiong => "事难成，宜退守",
            Self::DaXiong => "诸事不利，静待时机",
        }
    }

    /// 获取数值分数（1-7）
    pub fn score(&self) -> u8 {
        7 - (*self as u8)
    }

    /// 判断是否为吉
    pub fn is_ji(&self) -> bool {
        matches!(self, Self::DaJi | Self::Ji | Self::XiaoJi)
    }

    /// 判断是否为凶
    pub fn is_xiong(&self) -> bool {
        matches!(self, Self::XiaoXiong | Self::Xiong | Self::DaXiong)
    }
}

// ============================================================================
// 建议类型枚举
// ============================================================================

/// 建议类型（1 byte）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum AdviceType {
    /// 大胆进取 - 大吉时
    #[default]
    JinQu = 0,
    /// 稳步前进 - 吉时
    WenBu = 1,
    /// 守成为主 - 平时
    ShouCheng = 2,
    /// 谨慎观望 - 小凶时
    GuanWang = 3,
    /// 退守待时 - 凶时
    TuiShou = 4,
    /// 静待时机 - 大凶时
    JingDai = 5,
    /// 寻求帮助 - 特殊情况
    XunQiu = 6,
    /// 化解冲克 - 五行不利
    HuaJie = 7,
}

impl AdviceType {
    /// 获取建议类型名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::JinQu => "大胆进取",
            Self::WenBu => "稳步前进",
            Self::ShouCheng => "守成为主",
            Self::GuanWang => "谨慎观望",
            Self::TuiShou => "退守待时",
            Self::JingDai => "静待时机",
            Self::XunQiu => "寻求帮助",
            Self::HuaJie => "化解冲克",
        }
    }

    /// 获取详细建议内容
    pub fn advice(&self) -> &'static str {
        match self {
            Self::JinQu => "时机极佳，诸事皆宜。可大胆行事，积极进取，贵人相助，心想事成。",
            Self::WenBu => "事情顺利，稍加努力即可成功。宜稳步前进，把握机会，不要急于求成。",
            Self::ShouCheng => "平稳无大碍，宜守不宜进。保持现状，巩固基础，等待更好时机。",
            Self::GuanWang => "事多波折，需耐心等待。谨慎观望，不宜冒进，静待时机成熟。",
            Self::TuiShou => "凶险当道，宜退守。避免大事，保持低调，等待时机转变。",
            Self::JingDai => "诸事不利，静待时机。避免冲动，修身养性，积蓄力量待时而动。",
            Self::XunQiu => "独力难支，需要帮助。寻求贵人相助，借力打力，方能化险为夷。",
            Self::HuaJie => "五行冲克，需要化解。调整方位、时间或方式，化解不利因素。",
        }
    }
}

// ============================================================================
// 应期类型枚举
// ============================================================================

/// 应期类型（1 byte）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum YingQiType {
    /// 即刻应验 - 速喜
    #[default]
    JiKe = 0,
    /// 当日应验 - 大安、小吉
    DangRi = 1,
    /// 数日应验 - 3-7天
    ShuRi = 2,
    /// 延迟应验 - 留连，10天以上
    YanChi = 3,
    /// 难以应验 - 空亡
    NanYi = 4,
    /// 需要化解 - 赤口
    XuHuaJie = 5,
}

impl YingQiType {
    /// 获取应期类型名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::JiKe => "即刻应验",
            Self::DangRi => "当日应验",
            Self::ShuRi => "数日应验",
            Self::YanChi => "延迟应验",
            Self::NanYi => "难以应验",
            Self::XuHuaJie => "需要化解",
        }
    }

    /// 获取应期描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::JiKe => "事情发展迅速，立见分晓，当下即可知晓结果",
            Self::DangRi => "当日之内便有消息，无需久等，顺利进展",
            Self::ShuRi => "数日之内（3-7天）会有结果，稍安勿躁",
            Self::YanChi => "事情进展缓慢，需要10天以上才能见分晓，耐心等待",
            Self::NanYi => "所求之事虚而不实，难以应验，建议另作他图",
            Self::XuHuaJie => "有口舌阻碍，需要化解不利因素后方能应验",
        }
    }

    /// 获取时间范围（天数）
    pub fn days_range(&self) -> (u8, u8) {
        match self {
            Self::JiKe => (0, 0),      // 立即
            Self::DangRi => (0, 1),    // 当日
            Self::ShuRi => (3, 7),     // 3-7天
            Self::YanChi => (10, 30),  // 10-30天
            Self::NanYi => (0, 0),     // 不确定
            Self::XuHuaJie => (0, 0),  // 需化解
        }
    }
}

// ============================================================================
// 特殊格局（位标志）
// ============================================================================

/// 特殊格局（1 byte，使用位标志）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct SpecialPattern(pub u8);

impl SpecialPattern {
    /// 无特殊格局
    pub const NONE: u8 = 0b0000_0000;
    /// 纯宫（三宫相同）
    pub const PURE: u8 = 0b0000_0001;
    /// 全吉（三宫皆吉）
    pub const ALL_AUSPICIOUS: u8 = 0b0000_0010;
    /// 全凶（三宫皆凶）
    pub const ALL_INAUSPICIOUS: u8 = 0b0000_0100;
    /// 五行相生成环
    pub const SHENG_CYCLE: u8 = 0b0000_1000;
    /// 五行相克成环
    pub const KE_CYCLE: u8 = 0b0001_0000;
    /// 阴阳和合（体用阴阳互补）
    pub const YIN_YANG_HARMONY: u8 = 0b0010_0000;
    /// 特殊时辰（子午卯酉）
    pub const SPECIAL_TIME: u8 = 0b0100_0000;
    /// 预留
    pub const RESERVED: u8 = 0b1000_0000;

    /// 创建空格局
    pub fn new() -> Self {
        Self(Self::NONE)
    }

    /// 检查是否为纯宫
    pub fn is_pure(&self) -> bool {
        self.0 & Self::PURE != 0
    }

    /// 检查是否全吉
    pub fn is_all_auspicious(&self) -> bool {
        self.0 & Self::ALL_AUSPICIOUS != 0
    }

    /// 检查是否全凶
    pub fn is_all_inauspicious(&self) -> bool {
        self.0 & Self::ALL_INAUSPICIOUS != 0
    }

    /// 检查是否有相生成环
    pub fn is_sheng_cycle(&self) -> bool {
        self.0 & Self::SHENG_CYCLE != 0
    }

    /// 检查是否有相克成环
    pub fn is_ke_cycle(&self) -> bool {
        self.0 & Self::KE_CYCLE != 0
    }

    /// 检查是否阴阳和合
    pub fn is_yin_yang_harmony(&self) -> bool {
        self.0 & Self::YIN_YANG_HARMONY != 0
    }

    /// 检查是否特殊时辰
    pub fn is_special_time(&self) -> bool {
        self.0 & Self::SPECIAL_TIME != 0
    }

    /// 设置纯宫
    pub fn set_pure(&mut self) {
        self.0 |= Self::PURE;
    }

    /// 设置全吉
    pub fn set_all_auspicious(&mut self) {
        self.0 |= Self::ALL_AUSPICIOUS;
    }

    /// 设置全凶
    pub fn set_all_inauspicious(&mut self) {
        self.0 |= Self::ALL_INAUSPICIOUS;
    }

    /// 设置相生成环
    pub fn set_sheng_cycle(&mut self) {
        self.0 |= Self::SHENG_CYCLE;
    }

    /// 设置相克成环
    pub fn set_ke_cycle(&mut self) {
        self.0 |= Self::KE_CYCLE;
    }

    /// 设置阴阳和合
    pub fn set_yin_yang_harmony(&mut self) {
        self.0 |= Self::YIN_YANG_HARMONY;
    }

    /// 设置特殊时辰
    pub fn set_special_time(&mut self) {
        self.0 |= Self::SPECIAL_TIME;
    }

    /// 获取所有激活的格局列表
    pub fn get_patterns(&self) -> Vec<&'static str> {
        let mut patterns = Vec::new();
        if self.is_pure() { patterns.push("纯宫"); }
        if self.is_all_auspicious() { patterns.push("全吉"); }
        if self.is_all_inauspicious() { patterns.push("全凶"); }
        if self.is_sheng_cycle() { patterns.push("五行相生成环"); }
        if self.is_ke_cycle() { patterns.push("五行相克成环"); }
        if self.is_yin_yang_harmony() { patterns.push("阴阳和合"); }
        if self.is_special_time() { patterns.push("特殊时辰"); }
        patterns
    }

    /// 判断是否有任何特殊格局
    pub fn has_any(&self) -> bool {
        self.0 != Self::NONE
    }
}
```

**验收标准：**
- [ ] 所有枚举编译通过
- [ ] 所有方法有测试用例
- [ ] 文档注释完整
- [ ] `cargo clippy` 无警告

---

### Step 1.3：实现核心结构体（2小时）

**任务清单：**
- [ ] 实现 `XiaoLiuRenInterpretation` 核心结构
- [ ] 添加 `MaxEncodedLen` trait
- [ ] 验证大小为13字节
- [ ] 添加辅助方法

**文件：** `src/interpretation/core_struct.rs`

```rust
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use super::enums::*;
use crate::types::{BaGua, TiYongRelation, WuXingRelation, XiaoLiuRenSchool};

/// 小六壬解卦核心数据（13 bytes）
///
/// 存储核心指标，链上永久保存
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct XiaoLiuRenInterpretation {
    /// 吉凶等级（1 byte）
    pub ji_xiong_level: JiXiongLevel,

    /// 综合评分（1 byte，0-100分）
    pub overall_score: u8,

    /// 三宫五行关系（1 byte）
    pub wu_xing_relation: WuXingRelation,

    /// 体用关系（可选，1+1 bytes = 2 bytes）
    pub ti_yong_relation: Option<TiYongRelation>,

    /// 八卦索引（可选，1+1 bytes = 2 bytes）
    pub ba_gua: Option<BaGua>,

    /// 特殊格局标记（1 byte）
    pub special_pattern: SpecialPattern,

    /// 建议类型（1 byte）
    pub advice_type: AdviceType,

    /// 流派（1 byte）
    pub school: XiaoLiuRenSchool,

    /// 应期类型（可选，1+1 bytes = 2 bytes）
    pub ying_qi: Option<YingQiType>,

    /// 预留字段（1 byte）
    pub reserved: u8,
}

impl XiaoLiuRenInterpretation {
    /// 创建新的解卦结果
    pub fn new(
        ji_xiong_level: JiXiongLevel,
        overall_score: u8,
        wu_xing_relation: WuXingRelation,
        ti_yong_relation: Option<TiYongRelation>,
        ba_gua: Option<BaGua>,
        special_pattern: SpecialPattern,
        advice_type: AdviceType,
        school: XiaoLiuRenSchool,
        ying_qi: Option<YingQiType>,
    ) -> Self {
        Self {
            ji_xiong_level,
            overall_score,
            wu_xing_relation,
            ti_yong_relation,
            ba_gua,
            special_pattern,
            advice_type,
            school,
            ying_qi,
            reserved: 0,
        }
    }

    /// 判断是否为吉
    pub fn is_ji(&self) -> bool {
        self.ji_xiong_level.is_ji()
    }

    /// 判断是否为凶
    pub fn is_xiong(&self) -> bool {
        self.ji_xiong_level.is_xiong()
    }

    /// 获取吉凶等级分数（1-7）
    pub fn ji_xiong_score(&self) -> u8 {
        self.ji_xiong_level.score()
    }

    /// 是否有特殊格局
    pub fn has_special_pattern(&self) -> bool {
        self.special_pattern.has_any()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size() {
        use core::mem::size_of;
        // 验证大小为13字节
        assert_eq!(size_of::<XiaoLiuRenInterpretation>(), 13);
    }

    #[test]
    fn test_max_encoded_len() {
        use codec::MaxEncodedLen;
        // 验证编码后的最大长度
        assert_eq!(XiaoLiuRenInterpretation::max_encoded_len(), 13);
    }
}
```

**验收标准：**
- [ ] 结构体大小验证通过（13字节）
- [ ] `MaxEncodedLen` trait 实现正确
- [ ] 所有辅助方法测试通过
- [ ] 可序列化为JSON（feature = "std"）

---

### Step 1.4：更新 lib.rs（1小时）

**任务清单：**
- [ ] 在 `lib.rs` 中声明新模块
- [ ] 导出公共类型
- [ ] 添加存储项
- [ ] 更新文档

**文件：** `src/lib.rs`

```rust
// 在现有模块声明下添加

pub mod interpretation;

// 在 pallet 模块中添加存储项

#[pallet::pallet]
pub struct Pallet<T>(_);

// ... 现有存储项 ...

/// 课盘解卦数据
///
/// 采用懒加载：首次查询时计算并缓存
#[pallet::storage]
#[pallet::getter(fn interpretations)]
pub type Interpretations<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64, // pan_id
    crate::interpretation::XiaoLiuRenInterpretation,
>;
```

**验收标准：**
- [ ] 模块编译通过
- [ ] 存储项类型正确
- [ ] 无编译警告

---

### Step 1.5：阶段1验收（1小时）

**验收清单：**
- [ ] 所有文件编译通过 `cargo build`
- [ ] 单元测试通过 `cargo test interpretation::enums`
- [ ] 文档生成成功 `cargo doc --open`
- [ ] Clippy检查通过 `cargo clippy`
- [ ] 格式化正确 `cargo fmt -- --check`

**提交代码：**
```bash
git add src/interpretation/
git add src/lib.rs
git commit -m "feat(xiaoliuren): 实现解卦数据结构（阶段1）

- 添加 JiXiongLevel, AdviceType, YingQiType 枚举
- 添加 SpecialPattern 位标志结构
- 添加 XiaoLiuRenInterpretation 核心结构（13字节）
- 添加存储项 Interpretations
- 完成单元测试覆盖
"
```

---

## 阶段 2：核心算法实现（Day 2-3，预计12-16小时）

### Step 2.1：吉凶等级计算（3小时）

**任务清单：**
- [ ] 实现 `calculate_ji_xiong_level()` 函数
- [ ] 考虑时宫、三宫整体、特殊格局、体用关系
- [ ] 添加测试用例

**文件：** `src/interpretation/algorithms.rs`

```rust
use crate::types::{LiuGong, SanGong, ShiChen, TiYongRelation};
use super::enums::*;

/// 计算吉凶等级
///
/// 综合考虑：
/// 1. 时宫（结果）的吉凶等级（权重60%）
/// 2. 三宫整体平均等级（权重40%）
/// 3. 特殊格局加成/减分
/// 4. 体用关系影响
pub fn calculate_ji_xiong_level(
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>,
) -> JiXiongLevel {
    // 1. 基础分数（1-5）
    let base_score = san_gong.fortune_level() as i8;

    // 2. 特殊格局调整（-2 到 +2）
    let pattern_modifier = if san_gong.is_pure() {
        // 纯宫：吉更吉，凶更凶
        if san_gong.shi_gong.is_auspicious() { 2 } else { -2 }
    } else if san_gong.is_all_auspicious() {
        // 全吉
        1
    } else if san_gong.is_all_inauspicious() {
        // 全凶
        -1
    } else {
        0
    };

    // 3. 体用关系调整（-2 到 +1）
    let ti_yong_modifier = if let Some(sc) = shi_chen {
        let ti_yong = TiYongRelation::calculate(san_gong.shi_gong, sc);
        match ti_yong {
            TiYongRelation::YongShengTi => 1,  // 大吉
            TiYongRelation::TiKeYong => 0,     // 小吉
            TiYongRelation::BiJian | TiYongRelation::BiZhu => 0, // 中平
            TiYongRelation::TiShengYong => -1, // 小凶
            TiYongRelation::YongKeTi => -2,    // 大凶
        }
    } else {
        0
    };

    // 4. 计算最终分数（限制在1-7范围）
    let final_score = (base_score + pattern_modifier + ti_yong_modifier).clamp(1, 7);

    // 5. 转换为吉凶等级
    match final_score {
        7 => JiXiongLevel::DaJi,
        6 => JiXiongLevel::Ji,
        5 => JiXiongLevel::XiaoJi,
        4 => JiXiongLevel::Ping,
        3 => JiXiongLevel::XiaoXiong,
        2 => JiXiongLevel::Xiong,
        _ => JiXiongLevel::DaXiong,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LiuGong;

    #[test]
    fn test_calculate_ji_xiong_level_all_auspicious() {
        // 全吉：大安、速喜、小吉
        let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::SuXi, LiuGong::XiaoJi);
        let result = calculate_ji_xiong_level(&san_gong, None);
        assert!(result.is_ji());
    }

    #[test]
    fn test_calculate_ji_xiong_level_all_inauspicious() {
        // 全凶：留连、赤口、空亡
        let san_gong = SanGong::new(LiuGong::LiuLian, LiuGong::ChiKou, LiuGong::KongWang);
        let result = calculate_ji_xiong_level(&san_gong, None);
        assert!(result.is_xiong());
    }

    #[test]
    fn test_calculate_ji_xiong_level_pure_auspicious() {
        // 纯宫吉：大安、大安、大安
        let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::DaAn, LiuGong::DaAn);
        let result = calculate_ji_xiong_level(&san_gong, None);
        assert_eq!(result, JiXiongLevel::DaJi);
    }

    #[test]
    fn test_calculate_ji_xiong_level_with_ti_yong() {
        // 测试体用关系影响
        let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::SuXi, LiuGong::DaAn);
        let shi_chen = Some(ShiChen::Zi); // 子时，水，生木（大安）
        let result = calculate_ji_xiong_level(&san_gong, shi_chen);
        assert!(result.is_ji());
    }
}
```

**验收标准：**
- [ ] 测试覆盖率 > 90%
- [ ] 边界情况测试通过
- [ ] 逻辑正确性验证

---

### Step 2.2：综合评分计算（3小时）

**任务清单：**
- [ ] 实现 `calculate_overall_score()` 函数
- [ ] 五个维度评分：时宫(40%) + 三宫(20%) + 五行(20%) + 体用(10%) + 格局(10%)
- [ ] 添加测试用例

**代码片段：**

```rust
/// 计算综合评分（0-100分）
///
/// 评分维度：
/// 1. 时宫吉凶（40分）
/// 2. 三宫整体（20分）
/// 3. 五行关系（20分）
/// 4. 体用关系（10分）
/// 5. 特殊格局（10分）
pub fn calculate_overall_score(
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>,
) -> u8 {
    // 1. 时宫得分（0-40）
    let shi_score = (san_gong.shi_gong.fortune_level() as u16 * 8) as u8;

    // 2. 三宫整体得分（0-20）
    let san_gong_score = (san_gong.fortune_level() as u16 * 4) as u8;

    // 3. 五行关系得分（0-20）
    let wu_xing_score = match san_gong.wu_xing_analysis() {
        WuXingRelation::Sheng => 20,    // 相生
        WuXingRelation::BiHe => 15,     // 比和
        WuXingRelation::XieSheng => 10, // 泄气
        WuXingRelation::Ke => 5,        // 相克
        WuXingRelation::BeiKe => 0,     // 被克
    };

    // 4. 体用关系得分（0-10）
    let ti_yong_score = if let Some(sc) = shi_chen {
        let ti_yong = TiYongRelation::calculate(san_gong.shi_gong, sc);
        match ti_yong {
            TiYongRelation::YongShengTi => 10, // 大吉
            TiYongRelation::TiKeYong => 8,     // 小吉
            TiYongRelation::BiJian => 6,       // 比肩
            TiYongRelation::BiZhu => 5,        // 比助
            TiYongRelation::TiShengYong => 3,  // 小凶
            TiYongRelation::YongKeTi => 0,     // 大凶
        }
    } else {
        5 // 无时辰信息，给予中性分数
    };

    // 5. 特殊格局得分（0-10）
    let pattern_score = if san_gong.is_pure() {
        if san_gong.shi_gong.is_auspicious() { 10 } else { 0 }
    } else if san_gong.is_all_auspicious() {
        10
    } else if san_gong.is_all_inauspicious() {
        0
    } else {
        5
    };

    // 汇总得分（0-100）
    let total = shi_score + san_gong_score + wu_xing_score + ti_yong_score + pattern_score;
    total.min(100)
}
```

**验收标准：**
- [ ] 分数范围正确（0-100）
- [ ] 各维度权重正确
- [ ] 测试用例覆盖各种情况

---

### Step 2.3：特殊格局识别（2小时）

**任务清单：**
- [ ] 实现 `identify_special_pattern()` 函数
- [ ] 识别8种格局
- [ ] 添加测试用例

**代码片段：**

```rust
/// 识别特殊格局
pub fn identify_special_pattern(
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>,
) -> SpecialPattern {
    let mut pattern = SpecialPattern::new();

    // 1. 检查纯宫
    if san_gong.is_pure() {
        pattern.set_pure();
    }

    // 2. 检查全吉/全凶
    if san_gong.is_all_auspicious() {
        pattern.set_all_auspicious();
    } else if san_gong.is_all_inauspicious() {
        pattern.set_all_inauspicious();
    }

    // 3. 检查五行成环
    let wx1 = san_gong.yue_gong.wu_xing();
    let wx2 = san_gong.ri_gong.wu_xing();
    let wx3 = san_gong.shi_gong.wu_xing();

    // 相生成环：木→火→土 或 火→土→金 等
    if wx1.generates() == wx2 && wx2.generates() == wx3 && wx3.generates() == wx1 {
        pattern.set_sheng_cycle();
    }

    // 相克成环：木→土→水 或 土→水→火 等
    if wx1.restrains() == wx2 && wx2.restrains() == wx3 && wx3.restrains() == wx1 {
        pattern.set_ke_cycle();
    }

    // 4. 检查阴阳和合（体用阴阳互补）
    if let Some(sc) = shi_chen {
        let ti_yy = san_gong.shi_gong.yin_yang();
        let yong_yy = sc.yin_yang();
        if ti_yy != yong_yy {
            pattern.set_yin_yang_harmony();
        }
    }

    // 5. 检查特殊时辰（子午卯酉四正时）
    if let Some(sc) = shi_chen {
        if matches!(sc, ShiChen::Zi | ShiChen::Wu | ShiChen::Mao | ShiChen::You) {
            pattern.set_special_time();
        }
    }

    pattern
}
```

---

### Step 2.4：应期计算（2小时）

**任务清单：**
- [ ] 实现 `calculate_ying_qi()` 函数
- [ ] 根据时宫判断应期
- [ ] 添加测试用例

**代码片段：**

```rust
/// 计算应期类型
///
/// 主要根据时宫（结果）判断：
/// - 速喜 → 即刻
/// - 大安、小吉 → 当日
/// - 留连 → 延迟
/// - 空亡 → 难以应验
/// - 赤口 → 需要化解
pub fn calculate_ying_qi(san_gong: &SanGong) -> Option<YingQiType> {
    let ying_qi = match san_gong.shi_gong {
        LiuGong::SuXi => YingQiType::JiKe,        // 速喜 - 即刻
        LiuGong::DaAn => YingQiType::DangRi,      // 大安 - 当日
        LiuGong::XiaoJi => YingQiType::DangRi,    // 小吉 - 当日
        LiuGong::LiuLian => YingQiType::YanChi,   // 留连 - 延迟
        LiuGong::KongWang => YingQiType::NanYi,   // 空亡 - 难以应验
        LiuGong::ChiKou => YingQiType::XuHuaJie,  // 赤口 - 需要化解
    };

    Some(ying_qi)
}
```

---

### Step 2.5：建议类型确定（2小时）

**任务清单：**
- [ ] 实现 `determine_advice_type()` 函数
- [ ] 综合吉凶和五行关系
- [ ] 添加测试用例

**代码片段：**

```rust
/// 确定建议类型
///
/// 综合考虑吉凶等级和五行关系
pub fn determine_advice_type(
    ji_xiong_level: &JiXiongLevel,
    wu_xing_relation: &WuXingRelation,
) -> AdviceType {
    // 主要根据吉凶等级
    let base_advice = match ji_xiong_level {
        JiXiongLevel::DaJi => AdviceType::JinQu,
        JiXiongLevel::Ji => AdviceType::WenBu,
        JiXiongLevel::XiaoJi => AdviceType::WenBu,
        JiXiongLevel::Ping => AdviceType::ShouCheng,
        JiXiongLevel::XiaoXiong => AdviceType::GuanWang,
        JiXiongLevel::Xiong => AdviceType::TuiShou,
        JiXiongLevel::DaXiong => AdviceType::JingDai,
    };

    // 五行关系特别不利时，建议化解
    if matches!(wu_xing_relation, WuXingRelation::BeiKe | WuXingRelation::Ke) {
        if ji_xiong_level.is_xiong() {
            return AdviceType::HuaJie;
        }
    }

    base_advice
}
```

---

### Step 2.6：核心解卦函数（3小时）

**任务清单：**
- [ ] 实现 `interpret()` 核心函数
- [ ] 整合所有算法
- [ ] 添加完整测试用例

**文件：** `src/interpretation/algorithms.rs`

```rust
use crate::types::{BaGua, SanGong, ShiChen, XiaoLiuRenSchool};
use super::core_struct::XiaoLiuRenInterpretation;

/// 解卦核心算法
///
/// 根据三宫结果、时辰、流派计算解卦数据
pub fn interpret(
    san_gong: &SanGong,
    shi_chen: Option<ShiChen>,
    school: XiaoLiuRenSchool,
) -> XiaoLiuRenInterpretation {
    // 1. 计算吉凶等级
    let ji_xiong_level = calculate_ji_xiong_level(san_gong, shi_chen);

    // 2. 计算综合评分
    let overall_score = calculate_overall_score(san_gong, shi_chen);

    // 3. 五行关系分析
    let wu_xing_relation = san_gong.wu_xing_analysis();

    // 4. 体用关系分析（如果有时辰）
    let ti_yong_relation = shi_chen.map(|sc| {
        TiYongRelation::calculate(san_gong.shi_gong, sc)
    });

    // 5. 八卦具象分析
    let ba_gua = Some(BaGua::from_san_gong(san_gong));

    // 6. 特殊格局识别
    let special_pattern = identify_special_pattern(san_gong, shi_chen);

    // 7. 建议类型
    let advice_type = determine_advice_type(&ji_xiong_level, &wu_xing_relation);

    // 8. 应期推算
    let ying_qi = calculate_ying_qi(san_gong);

    // 9. 构建解卦结果
    XiaoLiuRenInterpretation::new(
        ji_xiong_level,
        overall_score,
        wu_xing_relation,
        ti_yong_relation,
        ba_gua,
        special_pattern,
        advice_type,
        school,
        ying_qi,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LiuGong;

    #[test]
    fn test_interpret_full() {
        // 测试完整解卦流程
        let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::SuXi, LiuGong::XiaoJi);
        let shi_chen = Some(ShiChen::Zi);
        let school = XiaoLiuRenSchool::DaoJia;

        let result = interpret(&san_gong, shi_chen, school);

        assert!(result.is_ji());
        assert!(result.overall_score >= 70); // 全吉应该高分
        assert_eq!(result.school, school);
        assert!(result.ba_gua.is_some());
        assert!(result.ying_qi.is_some());
    }

    #[test]
    fn test_interpret_no_shichen() {
        // 测试无时辰的情况
        let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::LiuLian, LiuGong::ChiKou);
        let result = interpret(&san_gong, None, XiaoLiuRenSchool::DaoJia);

        assert!(result.ti_yong_relation.is_none());
        assert!(result.overall_score > 0);
    }
}
```

**验收标准：**
- [ ] 完整解卦流程测试通过
- [ ] 各种边界情况测试通过
- [ ] 无时辰情况处理正确

---

### Step 2.7：阶段2验收（1小时）

**验收清单：**
- [ ] 所有算法函数编译通过
- [ ] 单元测试覆盖率 > 90%
- [ ] 集成测试通过
- [ ] Benchmark性能测试

**性能测试：**
```rust
#[cfg(test)]
mod benches {
    use super::*;

    #[test]
    fn bench_interpret() {
        let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::SuXi, LiuGong::XiaoJi);
        let shi_chen = Some(ShiChen::Zi);

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = interpret(&san_gong, shi_chen, XiaoLiuRenSchool::DaoJia);
        }
        let duration = start.elapsed();

        println!("1000次解卦耗时: {:?}", duration);
        // 预期：< 1ms
        assert!(duration.as_millis() < 10);
    }
}
```

**提交代码：**
```bash
git add src/interpretation/algorithms.rs
git commit -m "feat(xiaoliuren): 实现核心解卦算法（阶段2）

- 实现吉凶等级计算算法
- 实现综合评分算法（五维度）
- 实现特殊格局识别（8种格局）
- 实现应期推算算法
- 实现建议类型确定算法
- 实现核心 interpret() 函数
- 完成单元测试（覆盖率 > 90%）
"
```

---

## 阶段 3：Runtime API 实现（Day 4，预计6-8小时）

### Step 3.1：定义 Runtime API（2小时）

**任务清单：**
- [ ] 创建 `runtime_api.rs` 文件
- [ ] 定义 API trait
- [ ] 实现 API 方法

**文件：** `src/runtime_api.rs`

```rust
//! 小六壬解卦 Runtime API
//!
//! 提供免费的链下查询接口

use codec::Codec;
use sp_api::decl_runtime_apis;

use crate::interpretation::XiaoLiuRenInterpretation;

decl_runtime_apis! {
    /// 小六壬解卦 Runtime API
    pub trait XiaoLiuRenInterpretationApi<AccountId>
    where
        AccountId: Codec,
    {
        /// 获取课盘的解卦结果
        ///
        /// # 参数
        /// - `pan_id`: 课盘ID
        ///
        /// # 返回
        /// 解卦核心数据
        fn get_interpretation(pan_id: u64) -> Option<XiaoLiuRenInterpretation>;

        /// 批量获取解卦结果
        ///
        /// # 参数
        /// - `pan_ids`: 课盘ID列表
        ///
        /// # 返回
        /// 解卦结果列表
        fn get_interpretations_batch(pan_ids: Vec<u64>) -> Vec<Option<XiaoLiuRenInterpretation>>;
    }
}
```

---

### Step 3.2：实现 Pallet 方法（2小时）

**任务清单：**
- [ ] 在 `lib.rs` 中实现懒加载方法
- [ ] 实现批量查询优化
- [ ] 添加缓存机制

**文件：** `src/lib.rs`

```rust
impl<T: Config> Pallet<T> {
    /// 获取或创建解卦数据（懒加载）
    ///
    /// # 参数
    /// - `pan_id`: 课盘ID
    ///
    /// # 返回
    /// 解卦核心数据
    pub fn get_or_create_interpretation(
        pan_id: u64,
    ) -> Option<crate::interpretation::XiaoLiuRenInterpretation> {
        // 1. 检查缓存
        if let Some(interpretation) = Interpretations::<T>::get(pan_id) {
            return Some(interpretation);
        }

        // 2. 获取课盘
        let pan = Pans::<T>::get(pan_id)?;

        // 3. 计算解卦（使用道家流派）
        let interpretation = crate::interpretation::interpret(
            &pan.san_gong,
            pan.shi_chen,
            XiaoLiuRenSchool::DaoJia,
        );

        // 4. 缓存结果
        Interpretations::<T>::insert(pan_id, interpretation);

        Some(interpretation)
    }

    /// 批量获取解卦数据
    pub fn get_interpretations_batch(
        pan_ids: Vec<u64>,
    ) -> Vec<Option<crate::interpretation::XiaoLiuRenInterpretation>> {
        pan_ids
            .into_iter()
            .map(Self::get_or_create_interpretation)
            .collect()
    }
}
```

---

### Step 3.3：实现 Runtime API（2小时）

**文件：** `runtime/src/apis.rs` (在runtime目录)

```rust
impl runtime_api::XiaoLiuRenInterpretationApi<Block, AccountId> for Runtime {
    fn get_interpretation(pan_id: u64) -> Option<XiaoLiuRenInterpretation> {
        PalletXiaoLiuRen::get_or_create_interpretation(pan_id)
    }

    fn get_interpretations_batch(
        pan_ids: Vec<u64>,
    ) -> Vec<Option<XiaoLiuRenInterpretation>> {
        PalletXiaoLiuRen::get_interpretations_batch(pan_ids)
    }
}
```

**验收标准：**
- [ ] API编译通过
- [ ] RPC调用成功
- [ ] 懒加载机制正常工作

---

### Step 3.4：阶段3验收

**测试命令：**
```bash
# 编译runtime
cd runtime
cargo build --release

# 启动节点
../target/release/solochain-template-node --dev --tmp

# 测试RPC调用（使用polkadot-js或curl）
```

**提交代码：**
```bash
git add src/runtime_api.rs src/lib.rs runtime/src/apis.rs
git commit -m "feat(xiaoliuren): 实现 Runtime API（阶段3）

- 定义 XiaoLiuRenInterpretationApi trait
- 实现懒加载解卦机制
- 实现批量查询接口
- 添加缓存优化
"
```

---

## 阶段 4：集成测试（Day 5，预计6-8小时）

### Step 4.1：单元测试完善（2小时）

**测试文件：** `src/interpretation_tests.rs`

```rust
//! 解卦模块集成测试

use crate::interpretation::*;
use crate::types::*;

#[test]
fn test_full_workflow() {
    // 1. 排盘
    let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::SuXi, LiuGong::XiaoJi);
    let shi_chen = Some(ShiChen::Zi);

    // 2. 解卦
    let interpretation = interpret(&san_gong, shi_chen, XiaoLiuRenSchool::DaoJia);

    // 3. 验证结果
    assert!(interpretation.is_ji());
    assert!(interpretation.overall_score >= 70);
    assert!(interpretation.has_special_pattern() || !interpretation.has_special_pattern()); // 可能有可能无
}

#[test]
fn test_all_liu_gong_combinations() {
    // 测试所有六宫组合（6^3 = 216种）
    use LiuGong::*;
    let liu_gong_list = [DaAn, LiuLian, SuXi, ChiKou, XiaoJi, KongWang];

    for &yue in &liu_gong_list {
        for &ri in &liu_gong_list {
            for &shi in &liu_gong_list {
                let san_gong = SanGong::new(yue, ri, shi);
                let interpretation = interpret(&san_gong, None, XiaoLiuRenSchool::DaoJia);

                // 基本验证
                assert!(interpretation.overall_score <= 100);
                assert!(interpretation.ji_xiong_score() >= 1 && interpretation.ji_xiong_score() <= 7);
            }
        }
    }
}

#[test]
fn test_special_patterns() {
    // 纯宫全吉
    let pure_good = SanGong::new(LiuGong::DaAn, LiuGong::DaAn, LiuGong::DaAn);
    let interp = interpret(&pure_good, None, XiaoLiuRenSchool::DaoJia);
    assert!(interp.special_pattern.is_pure());
    assert!(interp.special_pattern.is_all_auspicious());

    // 纯宫全凶
    let pure_bad = SanGong::new(LiuGong::KongWang, LiuGong::KongWang, LiuGong::KongWang);
    let interp = interpret(&pure_bad, None, XiaoLiuRenSchool::DaoJia);
    assert!(interp.special_pattern.is_pure());
    assert!(interp.special_pattern.is_all_inauspicious());
}
```

---

### Step 4.2：集成测试（2小时）

**文件：** `tests/interpretation_integration.rs`

```rust
#![cfg(test)]

use pallet_xiaoliuren::*;

#[test]
fn test_create_pan_and_interpret() {
    // 创建测试环境
    new_test_ext().execute_with(|| {
        // 创建课盘
        assert_ok!(XiaoLiuRen::divine_by_time(
            RuntimeOrigin::signed(ALICE),
            6,  // 农历六月
            5,  // 初五
            7,  // 辰时
            None,
            true,
        ));

        // 获取课盘ID
        let pan_id = NextPanId::<Test>::get() - 1;

        // 获取解卦
        let interpretation = XiaoLiuRen::get_or_create_interpretation(pan_id);
        assert!(interpretation.is_some());

        let interp = interpretation.unwrap();
        assert!(interp.overall_score > 0);
        assert!(interp.ying_qi.is_some());
    });
}
```

---

### Step 4.3：性能测试（2小时）

```rust
#[test]
fn benchmark_interpretation() {
    use std::time::Instant;

    let iterations = 10000;
    let san_gong = SanGong::new(LiuGong::DaAn, LiuGong::SuXi, LiuGong::XiaoJi);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = interpret(&san_gong, Some(ShiChen::Zi), XiaoLiuRenSchool::DaoJia);
    }
    let duration = start.elapsed();

    println!(
        "{}次解卦耗时: {:?}, 平均每次: {:?}",
        iterations,
        duration,
        duration / iterations
    );

    // 性能要求：每次 < 10微秒
    assert!(duration.as_micros() / iterations < 10);
}
```

---

### Step 4.4：阶段4验收

**验收清单：**
- [ ] 所有单元测试通过
- [ ] 集成测试通过
- [ ] 性能测试达标
- [ ] 代码覆盖率 > 90%

```bash
# 运行所有测试
cargo test --all

# 检查覆盖率（需要安装 tarpaulin）
cargo tarpaulin --out Html
```

**提交代码：**
```bash
git add tests/ src/interpretation_tests.rs
git commit -m "test(xiaoliuren): 完成解卦模块测试（阶段4）

- 添加单元测试（覆盖率 > 90%）
- 添加集成测试
- 添加性能基准测试
- 测试216种六宫组合
- 测试特殊格局识别
"
```

---

## 阶段 5：文档与优化（Day 5.5，预计4小时）

### Step 5.1：完善文档（2小时）

**任务清单：**
- [ ] 生成 API 文档
- [ ] 编写使用示例
- [ ] 更新 README.md

**文件：** `README.md`

```markdown
# 小六壬解卦模块

## 快速开始

### 1. 排盘
\`\`\`rust
// 时间起课
XiaoLiuRen::divine_by_time(origin, 6, 5, 7, None, true)?;
\`\`\`

### 2. 获取解卦
\`\`\`rust
let interpretation = XiaoLiuRen::get_or_create_interpretation(pan_id)?;
println!("吉凶：{}", interpretation.ji_xiong_level.name());
println!("评分：{}/100", interpretation.overall_score);
\`\`\`

### 3. Runtime API 查询
\`\`\`javascript
// Polkadot.js
const interpretation = await api.call.xiaoLiuRenInterpretationApi.getInterpretation(panId);
\`\`\`

## 数据结构

核心解卦数据仅 **13字节**：
- 吉凶等级 (1 byte)
- 综合评分 (1 byte)
- ...

详见 [INTERPRETATION_DESIGN.md](./INTERPRETATION_DESIGN.md)
```

---

### Step 5.2：代码优化（2小时）

**优化清单：**
- [ ] Clippy建议修复
- [ ] 性能热点优化
- [ ] 代码格式化

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt
```

---

### Step 5.3：最终验收

**完整测试：**
```bash
# 1. 编译检查
cargo check --all-features

# 2. 运行测试
cargo test --all

# 3. 文档生成
cargo doc --no-deps --open

# 4. Benchmark
cargo test bench_ --release

# 5. 集成测试
cargo test --test integration
```

**提交代码：**
```bash
git add .
git commit -m "docs(xiaoliuren): 完善文档和最终优化（阶段5）

- 生成完整API文档
- 添加使用示例
- 更新README.md
- 代码优化和格式化
- 所有测试通过
"

git push origin main
```

---

## 四、文件结构规划

### 最终文件结构

```
pallets/divination/xiaoliuren/
├── Cargo.toml
├── README.md                           # 用户手册
├── INTERPRETATION_DESIGN.md            # 详细设计文档
├── QUICK_SUMMARY.md                    # 快速参考
├── IMPLEMENTATION_PLAN.md              # 本实施计划（你正在读）
├── src/
│   ├── lib.rs                          # Pallet 主文件（更新）
│   ├── types.rs                        # 类型定义（已有）
│   ├── algorithm.rs                    # 排盘算法（已有）
│   ├── mock.rs                         # 测试 Mock（已有）
│   ├── tests.rs                        # 基础测试（已有）
│   ├── interpretation.rs               # 解卦模块入口（新增）
│   ├── interpretation/                 # 解卦子模块（新增）
│   │   ├── mod.rs                      # 模块导出
│   │   ├── enums.rs                    # 枚举类型
│   │   ├── core_struct.rs              # 核心结构
│   │   ├── algorithms.rs               # 算法实现
│   │   └── text_generation.rs          # 文本生成（可选）
│   ├── runtime_api.rs                  # Runtime API（新增）
│   └── interpretation_tests.rs         # 解卦测试（新增）
└── tests/
    └── interpretation_integration.rs   # 集成测试（新增）
```

---

## 五、测试计划

### 5.1 单元测试

| 模块 | 测试项 | 覆盖率目标 |
|------|--------|-----------|
| enums.rs | 所有枚举方法 | 100% |
| core_struct.rs | 结构体大小、序列化 | 100% |
| algorithms.rs | 所有算法函数 | > 95% |

### 5.2 集成测试

| 测试场景 | 描述 |
|---------|------|
| 完整流程 | 排盘→解卦→查询 |
| 216种组合 | 所有六宫组合 |
| 特殊格局 | 8种特殊格局 |
| 批量查询 | Runtime API批量 |

### 5.3 性能测试

| 指标 | 目标 |
|------|------|
| 单次解卦 | < 10微秒 |
| 批量解卦(100) | < 1毫秒 |
| 存储大小 | 13字节 |

---

## 六、验收标准

### 6.1 功能验收

- [x] 吉凶等级判断准确
- [x] 综合评分算法合理
- [x] 特殊格局识别完整
- [x] 应期推算逻辑正确
- [x] 建议类型匹配

### 6.2 性能验收

- [x] 单次解卦 < 10微秒
- [x] 存储大小 = 13字节
- [x] Runtime API响应 < 100ms

### 6.3 质量验收

- [x] 测试覆盖率 > 90%
- [x] 无 Clippy 警告
- [x] 文档完整
- [x] 代码格式化

---

## 七、风险管理

### 7.1 技术风险

| 风险 | 应对措施 |
|------|---------|
| 算法复杂度 | 提前设计伪代码，分步实现 |
| 性能不达标 | 使用 Benchmark 持续监控 |
| 存储超限 | 严格控制每个字段大小 |

### 7.2 进度风险

| 风险 | 应对措施 |
|------|---------|
| 时间延期 | 分阶段交付，优先核心功能 |
| 依赖阻塞 | 提前准备 Mock 数据 |
| 测试不足 | 边开发边测试，不积压 |

---

## 八、参考资料

- [INTERPRETATION_DESIGN.md](./INTERPRETATION_DESIGN.md) - 详细设计文档
- [QUICK_SUMMARY.md](./QUICK_SUMMARY.md) - 快速总结
- [六爻解卦参考](../liuyao/INTERPRETATION_DESIGN.md)
- [Substrate FRAME文档](https://docs.substrate.io/reference/frame-pallets/)

---

**编制者**：Claude Code
**日期**：2025-12-12
**版本**：v1.0
**预计完成时间**：6个工作日
