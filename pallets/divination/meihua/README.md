# 梅花易数排盘模块 (pallet-meihua)

区块链上的梅花易数排盘系统，实现传统梅花易数的链上占卜功能，支持多种起卦方式和完整的卦象解读。

## 概述

梅花易数是宋代邵雍所创的占卜方法，本模块实现了：

- **多种起卦方式**：农历时间、公历时间、双数、单数、随机、手动、链摇
- **完整卦象**：本卦、变卦、互卦、体用关系
- **五行生克**：自动计算体用关系和吉凶
- **隐私保护**：支持加密存储敏感信息

## 八卦基础

| 卦数 | 卦名 | 五行 | 象征 |
|------|------|------|------|
| 1 | 乾 ☰ | 金 | 天、父、刚健 |
| 2 | 兑 ☱ | 金 | 泽、少女、喜悦 |
| 3 | 离 ☲ | 火 | 火、中女、光明 |
| 4 | 震 ☳ | 木 | 雷、长男、动 |
| 5 | 巽 ☴ | 木 | 风、长女、入 |
| 6 | 坎 ☵ | 水 | 水、中男、险 |
| 7 | 艮 ☶ | 土 | 山、少男、止 |
| 8 | 坤 ☷ | 土 | 地、母、柔顺 |


## 核心功能

### 1. 农历时间起卦

```rust
divine_by_time(
    question_hash: [u8; 32],
    is_public: bool,
    gender: u8,      // 0: 未指定, 1: 男, 2: 女
    category: u8,    // 0-6: 未指定/事业/财运/感情/健康/学业/其他
)
```

使用区块时间戳转换为农历，按传统公式计算：
- 上卦 = (年支数 + 月数 + 日数) % 8
- 下卦 = (年支数 + 月数 + 日数 + 时辰数) % 8
- 动爻 = (年支数 + 月数 + 日数 + 时辰数) % 6

### 2. 公历时间起卦

```rust
divine_by_gregorian_time(
    question_hash: [u8; 32],
    is_public: bool,
    gender: u8,
    category: u8,
)
```

现代简化方式，无需农历转换：
- 上卦 = (年份后两位 + 月 + 日) % 8
- 下卦 = (年份后两位 + 月 + 日 + 小时) % 8
- 动爻 = (年份后两位 + 月 + 日 + 小时) % 6

### 3. 双数起卦

```rust
divine_by_numbers(
    num1: u16,       // 第一个数字（上卦）
    num2: u16,       // 第二个数字（下卦）
    question_hash: [u8; 32],
    is_public: bool,
    gender: u8,
    category: u8,
)
```

使用两个数字配合当前时辰计算动爻。

### 4. 单数起卦

```rust
divine_by_single_number(
    number: u32,     // 多位数字
    question_hash: [u8; 32],
    is_public: bool,
    gender: u8,
    category: u8,
)
```

将数字拆分为前后两半分别计算上下卦：
- 上卦 = 前半段各位数字之和 % 8
- 下卦 = 后半段各位数字之和 % 8

### 5. 随机起卦

```rust
divine_random(
    question_hash: [u8; 32],
    is_public: bool,
    gender: u8,
    category: u8,
)
```

使用链上随机数生成卦象。

### 6. 手动指定起卦

```rust
divine_manual(
    shang_gua_num: u8,  // 上卦数 1-8
    xia_gua_num: u8,    // 下卦数 1-8
    dong_yao: u8,       // 动爻 1-6
    question_hash: [u8; 32],
    is_public: bool,
    gender: u8,
    category: u8,
)
```

直接指定卦数，用于记录已知卦象。

### 7. 链摇起卦

```rust
divine_by_shake(
    yaos: [u8; 6],              // 6个爻值 (0=阴, 1=阳)
    shake_timestamps: [u64; 6], // 6次摇卦时间戳
    question_hash: [u8; 32],
    is_public: bool,
    gender: u8,
    category: u8,
)
```

用户交互式摇卦，增强参与感：
```text
yaos[5] - 上爻（第6爻）  ─┐
yaos[4] - 五爻（第5爻）   │ 上卦
yaos[3] - 四爻（第4爻）  ─┘
yaos[2] - 三爻（第3爻）  ─┐
yaos[1] - 二爻（第2爻）   │ 下卦
yaos[0] - 初爻（第1爻）  ─┘
```

### 8. 带隐私数据的起卦

```rust
divine_with_privacy(
    question_hash: [u8; 32],
    is_public: bool,
    gender: u8,
    birth_year: Option<u16>,
    category: u8,
    method: DivinationMethod,
    encrypted_privacy: Option<EncryptedPrivacyData>,
)
```

原子性操作，同时创建卦象和加密记录。

## 数据结构

### FullDivination - 完整卦象

```rust
pub struct FullDivination<AccountId, BlockNumber> {
    pub ben_gua: Hexagram<AccountId, BlockNumber>,  // 本卦
    pub bian_gua: SimpleHexagram,                   // 变卦
    pub hu_gua: SimpleHexagram,                     // 互卦
    pub ti_yong: TiYongRelation,                    // 体用关系
}
```

### Hexagram - 卦象详情

```rust
pub struct Hexagram<AccountId, BlockNumber> {
    pub diviner: AccountId,
    pub shang_gua: Gua,           // 上卦
    pub xia_gua: Gua,             // 下卦
    pub dong_yao: u8,             // 动爻 (1-6)
    pub method: DivinationMethod, // 起卦方式
    pub question_hash: [u8; 32],
    pub block_number: BlockNumber,
    pub timestamp: u64,
    pub is_public: bool,
    pub gender: u8,
    pub birth_year: Option<u16>,
    pub category: u8,
    pub interpretation_cid: Option<BoundedVec<u8, ConstU32<64>>>,
}
```

### TiYongRelation - 体用关系

```rust
pub struct TiYongRelation {
    pub ti_gua: Gua,              // 体卦
    pub yong_gua: Gua,            // 用卦
    pub ti_element: WuXing,       // 体卦五行
    pub yong_element: WuXing,     // 用卦五行
    pub relation: WuXingRelation, // 五行关系
    pub fortune: Fortune,         // 吉凶判断
}
```

### DivinationMethod - 起卦方式

```rust
pub enum DivinationMethod {
    LunarDateTime,      // 农历时间
    GregorianDateTime,  // 公历时间
    TwoNumbers,         // 双数起卦
    SingleNumber,       // 单数起卦
    Random,             // 随机起卦
    Manual,             // 手动指定
    ChainShake,         // 链摇起卦
}
```

## 五行生克

```rust
pub enum WuXingRelation {
    Sheng,    // 生（用生体，吉）
    Ke,       // 克（用克体，凶）
    BeSheng,  // 被生（体生用，泄）
    BeKe,     // 被克（体克用，耗）
    BiHe,     // 比和（同五行，平）
}
```

## 配置参数

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    /// 每用户最多存储的卦象数量
    type MaxUserHexagrams: Get<u32>;
    
    /// 每日免费起卦次数
    type DailyFreeDivinations: Get<u32>;
    
    /// 每日最大起卦次数（防刷）
    type MaxDailyDivinations: Get<u32>;
    
    /// AI 解卦费用
    type AiInterpretationFee: Get<Balance>;
    
    /// 存储押金配置
    type StorageDepositPerKb: Get<u128>;
    type MinStorageDeposit: Get<u128>;
    type MaxStorageDeposit: Get<u128>;
}
```

## 事件

```rust
HexagramCreated { hexagram_id, diviner, method }
AiInterpretationRequested { hexagram_id, requester }
AiInterpretationSubmitted { hexagram_id, cid }
HexagramVisibilityChanged { hexagram_id, is_public }
HexagramCreatedWithPrivacy { hexagram_id, diviner, has_encrypted_data }
HexagramDeleted { hexagram_id, owner }
StorageDepositLocked { hexagram_id, owner, deposit, privacy_mode }
StorageDepositRefunded { hexagram_id, owner, refund, treasury }
```

## 农历转换

本模块使用 `pallet-almanac` 进行统一的公历农历转换：

```rust
pub use pallet_almanac::{
    MeihuaLunarDate as LunarDate,
    timestamp_to_meihua_lunar as timestamp_to_lunar,
    hour_to_dizhi_num,
    year_to_dizhi_num,
};
```

## 依赖

```toml
[dependencies]
pallet-meihua = { path = "../meihua", default-features = false }
pallet-almanac = { path = "../almanac", default-features = false }
pallet-divination-privacy = { path = "../privacy", default-features = false }
pallet-divination-common = { path = "../common", default-features = false }
```

## License

Unlicense
