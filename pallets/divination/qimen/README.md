# pallet-qimen

## 奇门遁甲排盘系统 - 区块链玄学占卜模块

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Substrate](https://img.shields.io/badge/Substrate-stable2506-blue)](https://github.com/paritytech/polkadot-sdk)

## 概述

`pallet-qimen` 是 Cosmos 区块链上的奇门遁甲排盘系统，实现了完整的奇门遁甲四盘排布算法和多层次解卦系统。本模块支持：

- **双排盘方法**：转盘奇门（主流）和飞盘奇门（古法）
- **四种排盘类型**：时家、日家、月家、年家奇门
- **四种起局方式**：时间起局、数字起局、随机起局、手动指定
- **三层解卦架构**：核心指标（链上存储）→ 扩展解读（实时计算）→ AI 解读（IPFS）
- **完整格局检测**：九遁格局、六仪击刑、门迫、十干克应等
- **用神分析系统**：12种问事类型的专属用神配置

### 什么是奇门遁甲？

奇门遁甲是中国古代最高层次的预测学，与太乙神数、六壬神课并称"三式"。其核心是通过天时、地利、人和、神助四个维度的综合分析，预测事物发展趋势。

## 核心特性

### 🎯 双排盘方法

| 方法 | 说明 | 特点 |
|------|------|------|
| **转盘奇门** | 九星、八门、八神作为整体旋转 | 当前主流方法，便于理解和计算 |
| **飞盘奇门** | 按洛书九宫数序分别飞入各宫 | 古法排盘，灵活多变 |

### 🔄 四种排盘类型

| 类型 | 三元依据 | 起局依据 | 应用场景 |
|------|----------|----------|----------|
| **时家奇门** | 时支 | 时干支 | 日常占断，最常用 |
| **日家奇门** | 节气天数 | 日干支 | 日课择吉 |
| **月家奇门** | 月支 | 月干支 | 月度规划 |
| **年家奇门** | 年支 | 年干支 | 年度运势 |

### 🎲 四种起局方式

| 方式 | 说明 | 适用场景 |
|------|------|----------|
| **时间起局** | 根据四柱和节气自动计算阴阳遁和局数 | 最传统、最常用的起局方式 |
| **公历起局** | 输入公历日期，自动转换为四柱和节气 | 便捷用户，无需了解干支 |
| **数字起局** | 用户输入数字，配合区块哈希生成卦象 | 心念起局、即时决策 |
| **随机起局** | 使用链上随机数完全随机生成 | 娱乐性占卜、测试用途 |
| **手动指定** | 直接指定阴阳遁类型和局数（1-9） | 专业用户、教学演示 |

### 🏛️ 完整的四盘系统

```
┌─────────────────────────────────────────────────────────────────┐
│                      奇门遁甲四盘结构                             │
├─────────────────────────────────────────────────────────────────┤
│  天盘（九星）     地盘（三奇六仪）    人盘（八门）    神盘（八神）    │
│  ┌───┬───┬───┐   ┌───┬───┬───┐   ┌───┬───┬───┐   ┌───┬───┬───┐  │
│  │巽4│离9│坤2│   │乙 │丙 │丁 │   │杜 │景 │死 │   │腾蛇│太阴│六合│  │
│  ├───┼───┼───┤   ├───┼───┼───┤   ├───┼───┼───┤   ├───┼───┼───┤  │
│  │震3│中5│兑7│   │戊 │己 │庚 │   │伤 │  │惊 │   │白虎│  │玄武│  │
│  ├───┼───┼───┤   ├───┼───┼───┤   ├───┼───┼───┤   ├───┼───┼───┤  │
│  │艮8│坎1│乾6│   │辛 │壬 │癸 │   │生 │休 │开 │   │九地│九天│值符│  │
│  └───┴───┴───┘   └───┴───┴───┘   └───┴───┴───┘   └───┴───┴───┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 📊 三层解卦架构

```
┌─────────────────────────────────────────────────────────────────┐
│                      三层解卦架构                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Layer 1: 核心解卦指标 (~16 bytes)                         │    │
│  │ • 格局类型、用神宫位、值符值使                              │    │
│  │ • 日干/时干落宫、吉凶评分、旺衰状态                         │    │
│  │ • 特殊格局标记（位标志）、可信度                            │    │
│  │ → 链上存储，永久保存                                       │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                    │
│                              ▼                                    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Layer 2: 扩展解读数据                                      │    │
│  │ • 九宫详细解读（星门神关系、五行生克）                       │    │
│  │ • 用神分析（主/次用神、得力状态）                           │    │
│  │ • 应期推算（时间单位、吉凶时段）                            │    │
│  │ → Runtime API 实时计算，不存储                             │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                    │
│                              ▼                                    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Layer 3: AI 智能解读                                        │    │
│  │ • 自然语言综合解读                                          │    │
│  │ • 针对问事类型的专业建议                                     │    │
│  │ → IPFS 存储，链上保存 CID                                   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### 🔮 格局检测系统

#### 九遁格局（大吉格）

| 格局 | 条件 | 说明 |
|------|------|------|
| **天遁** | 丙奇+生门+吉神 | 天时相助，求财大吉 |
| **地遁** | 乙奇+开门+九地 | 藏形匿迹，避祸大吉 |
| **人遁** | 丁奇+休门+太阴 | 密谋私事，贵人相助 |
| **风遁** | 乙奇+休门+六合 | 出行远征，商旅通畅 |
| **云遁** | 乙奇+生门+九天 | 升迁高就，名声远扬 |
| **龙遁** | 乙奇+休门+六合在坎宫 | 龙入大海，大富大贵 |
| **虎遁** | 乙奇+开门+太阴在艮宫 | 虎踞山林，威镇四方 |
| **神遁** | 丙奇+休门+九天 | 神明护佑，祈福灵验 |
| **鬼遁** | 丁奇+生门+九地 | 鬼神相助，化险为夷 |

#### 凶格检测

| 格局 | 条件 | 说明 |
|------|------|------|
| **六仪击刑** | 六仪临特定宫位 | 戊临震宫、己临坤宫等 |
| **奇仪入墓** | 天干临其墓库宫 | 甲戊入乾宫、丁己入艮宫等 |
| **门迫** | 八门五行克落宫五行 | 休门临离宫、生门临坎宫等 |
| **伏吟/反吟** | 天地盘干相同/对冲 | 事情停滞或反复变化 |
| **白虎猖狂** | 辛+乙同宫 | 血光刑伤，意外伤害 |
| **螣蛇夭矫** | 壬+壬同宫 | 虚惊怪异，心神不宁 |
| **朱雀投江** | 丙+壬在坎宫 | 文书失误，口舌招灾 |

### 📋 用神分析系统

支持 12 种问事类型的专属用神配置：

| 问事类型 | 主用神 | 次用神 | 吉利条件 |
|----------|--------|--------|----------|
| 综合运势 | 日干/值符 | 时干 | 值符临吉门吉星 |
| 事业工作 | 开门 | 天心星 | 开门旺相无迫 |
| 财运求财 | 生门 | 戊(正财) | 生门得奇不空 |
| 婚姻感情 | 六合神 | 乙(日奇) | 六合吉门相合 |
| 健康疾病 | 天芮星 | 死门 | 天芮死门受克 |
| 学业考试 | 天辅星 | 景门 | 天辅临吉门旺相 |
| 出行远行 | 驿马宫 | 开门 | 开门无凶格 |
| 官司诉讼 | 开门(官) | 庚(对方) | 我克彼有利 |
| 寻人寻物 | 六合神 | 相关宫位 | 六合无空亡 |
| 投资理财 | 生门 | 天任星 | 生门天任同宫 |
| 合作交易 | 六合神 | 生门 | 六合生门相生 |
| 祈福求神 | 九天神 | 景门 | 九天临吉门 |

## 技术架构

### 模块依赖

```
pallet-qimen
    ├── pallet-divination-common  (通用占卜类型)
    ├── pallet-almanac            (农历转换、四柱计算)
    ├── pallet-timestamp          (时间戳获取)
    ├── frame-support             (Substrate 框架)
    └── frame-system              (系统模块)
```

### 存储设计

| 存储项 | 类型 | 大小 | 说明 |
|--------|------|------|------|
| `NextChartId` | `u64` | 8 bytes | 下一个排盘 ID |
| `Charts` | `Map<u64, QimenChart>` | ~550 bytes/盘 | 完整排盘数据 |
| `UserCharts` | `Map<AccountId, Vec<u64>>` | 可变 | 用户排盘索引 |
| `PublicCharts` | `Vec<u64>` | 可变 | 公开排盘列表 |
| `DailyChartCount` | `DoubleMap<AccountId, u32, u32>` | 8 bytes | 每日计数 |
| `UserStatsStorage` | `Map<AccountId, UserStats>` | 40 bytes | 用户统计 |

### 核心解卦存储优化

`QimenCoreInterpretation` 仅 16 bytes：

| 字段 | 大小 | 说明 |
|------|------|------|
| `ge_ju` | 1 byte | 格局类型 |
| `yong_shen_gong` | 1 byte | 用神宫位 (1-9) |
| `zhi_fu_xing` | 1 byte | 值符星 |
| `zhi_shi_men` | 1 byte | 值使门 |
| `ri_gan_gong` | 1 byte | 日干落宫 |
| `shi_gan_gong` | 1 byte | 时干落宫 |
| `fortune` | 1 byte | 综合吉凶 |
| `fortune_score` | 1 byte | 吉凶评分 (0-100) |
| `wang_shuai` | 1 byte | 旺衰状态 |
| `special_patterns` | 1 byte | 特殊格局（位标志） |
| `confidence` | 1 byte | 可信度 (0-100) |
| `timestamp` | 4 bytes | 区块号 |
| `algorithm_version` | 1 byte | 算法版本 |

## 数据结构

### QimenChart（奇门盘）

```rust
pub struct QimenChart<AccountId, BlockNumber, MaxCidLen> {
    // 基础信息
    pub id: u64,
    pub diviner: AccountId,
    pub method: DivinationMethod,

    // 命主信息
    pub name: Option<BoundedVec<u8, MaxNameLen>>,
    pub gender: Option<Gender>,
    pub birth_year: Option<u16>,
    pub question: Option<BoundedVec<u8, MaxQuestionLen>>,
    pub question_type: Option<QuestionType>,
    pub pan_method: PanMethod,

    // 四柱干支
    pub year_ganzhi: GanZhi,
    pub month_ganzhi: GanZhi,
    pub day_ganzhi: GanZhi,
    pub hour_ganzhi: GanZhi,
    pub jie_qi: JieQi,

    // 局数信息
    pub dun_type: DunType,
    pub san_yuan: SanYuan,
    pub ju_number: u8,

    // 盘面数据
    pub zhi_fu_xing: JiuXing,
    pub zhi_shi_men: BaMen,
    pub palaces: [Palace; 9],

    // 元数据
    pub timestamp: u64,
    pub block_number: BlockNumber,
    pub interpretation_cid: Option<BoundedVec<u8, MaxCidLen>>,
    pub is_public: bool,
    pub question_hash: [u8; 32],
}
```

### Palace（宫位）

```rust
pub struct Palace {
    pub gong: JiuGong,          // 宫位 (1-9)
    pub tian_pan_gan: TianGan,  // 天盘干
    pub di_pan_gan: TianGan,    // 地盘干
    pub xing: JiuXing,          // 九星
    pub men: Option<BaMen>,     // 八门（中宫无门）
    pub shen: Option<BaShen>,   // 八神（中宫无神）
    pub is_xun_kong: bool,      // 是否旬空
    pub is_ma_xing: bool,       // 是否马星
}
```

### 核心枚举类型

```rust
/// 阴阳遁
pub enum DunType { Yang, Yin }

/// 排盘方法
pub enum PanMethod { ZhuanPan, FeiPan }

/// 排盘类型
pub enum QimenType { ShiJia, RiJia, YueJia, NianJia }

/// 九星
pub enum JiuXing {
    TianPeng,   // 天蓬星（坎一宫，水）
    TianRui,    // 天芮星（坤二宫，土）
    TianChong,  // 天冲星（震三宫，木）
    TianFu,     // 天辅星（巽四宫，木）
    TianQin,    // 天禽星（中五宫，土）
    TianXin,    // 天心星（乾六宫，金）
    TianZhu,    // 天柱星（兑七宫，金）
    TianRen,    // 天任星（艮八宫，土）
    TianYing,   // 天英星（离九宫，火）
}

/// 八门
pub enum BaMen {
    Xiu,    // 休门（大吉，水）
    Si,     // 死门（大凶，土）
    Shang,  // 伤门（凶，木）
    Du,     // 杜门（凶，木）
    Jing,   // 景门（中平，火）
    Kai,    // 开门（大吉，金）
    Jing2,  // 惊门（凶，金）
    Sheng,  // 生门（大吉，土）
}

/// 八神
pub enum BaShen {
    ZhiFu,    // 值符（最吉）
    TengShe,  // 腾蛇（惊恐）
    TaiYin,   // 太阴（阴私）
    LiuHe,    // 六合（和合）
    BaiHu,    // 白虎（凶险）
    XuanWu,   // 玄武（盗贼）
    JiuDi,    // 九地（柔顺）
    JiuTian,  // 九天（刚健）
}

/// 问事类型
pub enum QuestionType {
    General,     // 综合运势
    Career,      // 事业工作
    Wealth,      // 财运求财
    Marriage,    // 婚姻感情
    Health,      // 健康疾病
    Study,       // 学业考试
    Travel,      // 出行远行
    Lawsuit,     // 官司诉讼
    Finding,     // 寻人寻物
    Investment,  // 投资理财
    Business,    // 合作交易
    Prayer,      // 祈福求神
}
```

## Extrinsics（可调用函数）

### 1. divine_by_time - 四柱时间起局

```rust
#[pallet::call_index(0)]
pub fn divine_by_time(
    origin: OriginFor<T>,
    year_ganzhi: (u8, u8),      // 年柱（干0-9，支0-11）
    month_ganzhi: (u8, u8),     // 月柱
    day_ganzhi: (u8, u8),       // 日柱
    hour_ganzhi: (u8, u8),      // 时柱
    jie_qi: u8,                 // 节气（0-23）
    day_in_jieqi: u8,           // 节气内天数（1-15）
    question_hash: [u8; 32],    // 问题哈希
    is_public: bool,            // 是否公开
    name: Option<BoundedVec<u8, MaxNameLen>>,
    gender: Option<u8>,
    birth_year: Option<u16>,
    question: Option<BoundedVec<u8, MaxQuestionLen>>,
    question_type: Option<u8>,
    pan_method: u8,             // 0=转盘，1=飞盘
) -> DispatchResult;
```

### 2. divine_by_solar_time - 公历时间起局

```rust
#[pallet::call_index(7)]
pub fn divine_by_solar_time(
    origin: OriginFor<T>,
    solar_year: u16,            // 公历年份（1901-2100）
    solar_month: u8,            // 公历月份（1-12）
    solar_day: u8,              // 公历日期（1-31）
    hour: u8,                   // 小时（0-23）
    question_hash: [u8; 32],
    is_public: bool,
    // ... 命主信息参数
) -> DispatchResult;
```

### 3. divine_by_numbers - 数字起局

```rust
#[pallet::call_index(1)]
pub fn divine_by_numbers(
    origin: OriginFor<T>,
    numbers: BoundedVec<u16, ConstU32<16>>,  // 数字列表
    yang_dun: bool,                          // 是否阳遁
    question_hash: [u8; 32],
    is_public: bool,
    // ... 命主信息参数
) -> DispatchResult;
```

### 4. divine_random - 随机起局

```rust
#[pallet::call_index(2)]
pub fn divine_random(
    origin: OriginFor<T>,
    question_hash: [u8; 32],
    is_public: bool,
    // ... 命主信息参数
) -> DispatchResult;
```

### 5. divine_manual - 手动指定

```rust
#[pallet::call_index(3)]
pub fn divine_manual(
    origin: OriginFor<T>,
    yang_dun: bool,             // 是否阳遁
    ju_number: u8,              // 局数（1-9）
    hour_ganzhi: (u8, u8),      // 时柱
    question_hash: [u8; 32],
    is_public: bool,
    // ... 命主信息参数
) -> DispatchResult;
```

### 6. set_chart_visibility - 设置公开状态

```rust
#[pallet::call_index(6)]
pub fn set_chart_visibility(
    origin: OriginFor<T>,
    chart_id: u64,
    is_public: bool,
) -> DispatchResult;
```

## Runtime API

提供高效的链上查询接口：

```rust
/// 获取核心解卦（Layer 1）
fn api_get_core_interpretation(chart_id: u64)
    -> Option<QimenCoreInterpretation>;

/// 获取完整解卦（Layer 1 + 2 + 3）
fn api_get_full_interpretation(chart_id: u64, question_type: QuestionType)
    -> Option<QimenFullInterpretation>;

/// 获取单宫详细解读
fn api_get_palace_interpretation(chart_id: u64, palace_num: u8)
    -> Option<PalaceInterpretation>;

/// 获取用神分析
fn api_get_yong_shen_analysis(chart_id: u64, question_type: QuestionType)
    -> Option<YongShenAnalysis>;

/// 获取应期推算
fn api_get_ying_qi_analysis(chart_id: u64)
    -> Option<YingQiAnalysis>;
```

## 算法说明

### 阴阳遁判定

```
冬至 → 夏至：阳遁（顺行）
夏至 → 冬至：阴遁（逆行）

阳遁：甲子 → 甲戌 → 甲申 → 甲午 → 甲辰 → 甲寅 → 甲子...
阴遁：甲子 → 甲寅 → 甲辰 → 甲午 → 甲申 → 甲戌 → 甲子...
```

### 三元计算

```rust
// 按时支判断（时家奇门）
子午卯酉 → 上元
寅申巳亥 → 中元
辰戌丑未 → 下元

// 按节气天数判断（日家奇门）
第 1-5 天  → 上元
第 6-10 天 → 中元
第 11-15 天 → 下元
```

### 阴阳遁地盘差异

**重要**：阴遁和阳遁的地盘三奇六仪排布顺序不同！

```rust
// 阳遁顺序（六仪顺+三奇顺）
戊 → 己 → 庚 → 辛 → 壬 → 癸 → 丁 → 丙 → 乙

// 阴遁顺序（三奇顺+六仪逆）
戊 → 乙 → 丙 → 丁 → 癸 → 壬 → 辛 → 庚 → 己
```

### 旬空计算

六甲旬空口诀：
- 甲子旬空戌亥
- 甲戌旬空申酉
- 甲申旬空午未
- 甲午旬空辰巳
- 甲辰旬空寅卯
- 甲寅旬空子丑

### 驿马计算

```rust
申子辰三合水局 → 驿马在寅（艮八宫）
寅午戌三合火局 → 驿马在申（坤二宫）
巳酉丑三合金局 → 驿马在亥（乾六宫）
亥卯未三合木局 → 驿马在巳（巽四宫）
```

### 转盘与飞盘差异

| 项目 | 转盘 | 飞盘 |
|------|------|------|
| 九星 | 整体旋转 | 按洛书顺序飞布 |
| 八门 | 整体旋转 | 按洛书顺序飞布 |
| 八神 | 整体旋转 | 按洛书顺序飞布 |
| 天盘干 | 随九星移动 | 随九星飞布 |

## 配置参数

```rust
#[pallet::config]
pub trait Config: frame_system::Config + pallet_timestamp::Config {
    /// 货币类型
    type Currency: Currency<Self::AccountId>;

    /// 随机数生成器
    type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

    /// 每用户最大排盘数量（默认: 100）
    type MaxUserCharts: Get<u32>;

    /// 公开列表最大长度（默认: 1000）
    type MaxPublicCharts: Get<u32>;

    /// 每日免费排盘次数（默认: 3）
    type DailyFreeCharts: Get<u32>;

    /// 每日最大排盘次数（默认: 20）
    type MaxDailyCharts: Get<u32>;

    /// AI 解读费用（默认: 15 COS）
    type AiInterpretationFee: Get<BalanceOf<Self>>;

    /// 国库账户
    type TreasuryAccount: Get<Self::AccountId>;

    /// AI 预言机权限
    type AiOracleOrigin: EnsureOrigin<Self::RuntimeOrigin>;

    /// IPFS CID 最大长度（默认: 64）
    type MaxCidLen: Get<u32>;
}
```

## 使用示例

### Polkadot.js API

```javascript
// 公历时间起局（推荐）
await api.tx.qimen.divineBySolarTime(
    2025,     // 年
    12,       // 月
    25,       // 日
    10,       // 时
    questionHash,
    true,     // 公开
    null,     // 姓名
    0,        // 性别（男）
    1990,     // 出生年份
    null,     // 问题
    0,        // 问事类型（综合运势）
    0         // 排盘方法（转盘）
).signAndSend(alice);

// 手动指定起局
await api.tx.qimen.divineManual(
    true,       // 阳遁
    1,          // 一局
    [0, 0],     // 甲子时
    questionHash,
    false,
    // ... 命主信息
).signAndSend(alice);

// 查询排盘结果
const chart = await api.query.qimen.charts(chartId);
console.log('阴阳遁:', chart.dunType.toString());
console.log('局数:', chart.juNumber.toNumber());
console.log('值符星:', chart.zhiFuXing.toString());
console.log('九宫:', chart.palaces);

// 调用 Runtime API 获取解卦
const interpretation = await api.call.qimenApi.getCoreInterpretation(chartId);
console.log('格局:', interpretation.geJu);
console.log('吉凶:', interpretation.fortune);
console.log('评分:', interpretation.fortuneScore);
```

### 前端集成示例

```typescript
import { ApiPromise } from '@polkadot/api';

// 创建奇门盘并获取解卦
async function createAndInterpret(
    api: ApiPromise,
    account: string,
    year: number,
    month: number,
    day: number,
    hour: number
) {
    // 1. 创建排盘
    const chartId = await new Promise((resolve, reject) => {
        api.tx.qimen.divineBySolarTime(
            year, month, day, hour,
            new Uint8Array(32),
            true, null, null, null, null, 0, 0
        ).signAndSend(account, ({ events, status }) => {
            if (status.isInBlock) {
                events.forEach(({ event }) => {
                    if (api.events.qimen.ChartCreated.is(event)) {
                        resolve(event.data[0].toString());
                    }
                });
            }
        });
    });

    // 2. 获取核心解卦
    const core = await api.call.qimenApi.getCoreInterpretation(chartId);

    // 3. 获取完整解卦（指定问事类型）
    const full = await api.call.qimenApi.getFullInterpretation(
        chartId,
        { Career: null }  // 事业工作
    );

    return { chartId, core, full };
}
```

## 测试

```bash
# 运行所有测试
cargo test -p pallet-qimen

# 运行特定测试
cargo test -p pallet-qimen test_divine_by_time
cargo test -p pallet-qimen test_di_pan_yin_yang_difference
cargo test -p pallet-qimen test_xun_kong_all_sixty

# 显示测试输出
cargo test -p pallet-qimen -- --nocapture
```

### 测试覆盖

- ✅ 时间起局功能
- ✅ 公历时间起局
- ✅ 数字起局功能
- ✅ 随机起局功能
- ✅ 手动指定功能
- ✅ 每日限制检查
- ✅ 阴阳遁地盘差异验证
- ✅ 旬空计算（六十甲子全覆盖）
- ✅ 六仪击刑检测
- ✅ 奇仪入墓检测
- ✅ 门迫检测
- ✅ 十干克应检测
- ✅ 驿马计算
- ✅ 旺衰计算
- ✅ 转盘与飞盘对比
- ✅ 九遁格局检测
- ✅ 核心解卦编码大小验证

## 事件

```rust
/// 排盘创建成功
ChartCreated { chart_id, diviner, dun_type, ju_number }

/// AI 解读请求已提交
AiInterpretationRequested { chart_id, requester }

/// AI 解读结果已提交
AiInterpretationSubmitted { chart_id, cid }

/// 公开状态已更改
ChartVisibilityChanged { chart_id, is_public }
```

## 错误类型

```rust
ChartNotFound,              // 排盘记录不存在
NotOwner,                   // 非记录所有者
DailyLimitExceeded,         // 每日次数超限
UserChartsFull,             // 用户列表已满
PublicChartsFull,           // 公开列表已满
InvalidJuNumber,            // 无效局数
InvalidJieQi,               // 无效节气
AiRequestAlreadyExists,     // AI请求已存在
AiRequestNotFound,          // AI请求不存在
MissingNumberParams,        // 数字参数缺失
MissingManualParams,        // 手动参数缺失
InvalidGanZhi,              // 无效干支组合
InvalidDayInJieQi,          // 节气天数超范围
```

## 许可证

MIT License

## 更新日志

### v0.2.0 (2025-12-25)

- 🎉 新增飞盘奇门排盘方法
- 🎉 新增时家/日家/月家/年家排盘类型
- 🎉 新增三层解卦架构（Layer 1-3）
- 🎉 新增完整格局检测系统（九遁、六仪击刑、门迫等）
- 🎉 新增用神分析系统（12种问事类型）
- 🎉 新增 Runtime API 实时解卦接口
- 🎉 新增公历时间起局功能（集成 pallet-almanac）
- ✅ 修复阴遁地盘排布（使用独立的三奇六仪顺序）
- ✅ 修复旬空计算算法
- ✅ 核心解卦存储优化至 16 bytes
- ✅ 完善单元测试覆盖

### v0.1.0 (2025-12-01)

- 🎉 初始版本发布
- ✅ 实现四种起局方式
- ✅ 完整的四盘排布算法
- ✅ 与 pallet-divination-common 集成

## 参考资料

- 《奇门遁甲》古籍
- 《奇门预测学》张志春
- 《神奇之门》张志春
- [Polkadot SDK 文档](https://docs.substrate.io/)
