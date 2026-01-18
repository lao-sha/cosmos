# 塔罗牌排盘模块 (pallet-tarot)

区块链上的塔罗牌占卜系统，提供 78 张塔罗牌的链上占卜功能，支持多种起卦方式和牌阵。

## 概述

本模块实现了完整的塔罗牌占卜系统：

- **多种起卦方式**：随机抽牌、时间起卦、数字起卦、手动指定、带切牌抽牌
- **多种牌阵**：单张、三牌、凯尔特十字等
- **占卜记录**：链上存储与查询
- **AI 解读**：链下工作机触发

## 塔罗牌体系

### 大阿卡纳 (Major Arcana)

22 张主牌，代表人生重大主题：

| ID | 牌名 | ID | 牌名 |
|----|------|----|----- |
| 0 | 愚者 | 11 | 正义 |
| 1 | 魔术师 | 12 | 倒吊人 |
| 2 | 女祭司 | 13 | 死神 |
| 3 | 女皇 | 14 | 节制 |
| 4 | 皇帝 | 15 | 恶魔 |
| 5 | 教皇 | 16 | 塔 |
| 6 | 恋人 | 17 | 星星 |
| 7 | 战车 | 18 | 月亮 |
| 8 | 力量 | 19 | 太阳 |
| 9 | 隐士 | 20 | 审判 |
| 10 | 命运之轮 | 21 | 世界 |


### 小阿卡纳 (Minor Arcana)

56 张副牌，分四种花色：

| 花色 | ID 范围 | 元素 | 象征 |
|------|--------|------|------|
| 权杖 (Wands) | 22-35 | 火 | 行动、创造、热情 |
| 圣杯 (Cups) | 36-49 | 水 | 情感、直觉、关系 |
| 宝剑 (Swords) | 50-63 | 风 | 思想、冲突、真相 |
| 星币 (Pentacles) | 64-77 | 土 | 物质、财富、实际 |

每种花色包含：Ace、2-10、侍从、骑士、王后、国王

## 牌阵类型

```rust
pub enum SpreadType {
    Single,           // 单张牌 - 简单问题
    ThreeCard,        // 三牌阵 - 过去/现在/未来
    CelticCross,      // 凯尔特十字 - 10张牌，全面分析
    Horseshoe,        // 马蹄形 - 7张牌
    Relationship,     // 关系牌阵 - 5张牌
    YesNo,            // 是否牌阵 - 1张牌
    Custom(u8),       // 自定义牌数
}
```

## 核心功能

### 1. 随机抽牌

```rust
divine_random(
    spread_type: SpreadType,
    question_hash: [u8; 32],
    privacy_mode: PrivacyMode,
)
```

使用链上随机数生成塔罗牌占卜结果。

### 2. 时间起卦

```rust
divine_by_time(
    spread_type: SpreadType,
    question_hash: [u8; 32],
    privacy_mode: PrivacyMode,
)
```

使用当前区块时间戳生成占卜结果。

### 3. 数字起卦

```rust
divine_by_numbers(
    numbers: BoundedVec<u16, ConstU32<16>>,
    spread_type: SpreadType,
    question_hash: [u8; 32],
    privacy_mode: PrivacyMode,
)
```

使用用户提供的数字生成占卜结果。

### 4. 手动指定

```rust
divine_manual(
    cards: BoundedVec<(u8, bool), ConstU32<12>>,  // (牌ID, 是否逆位)
    spread_type: SpreadType,
    question_hash: [u8; 32],
    privacy_mode: PrivacyMode,
)
```

直接指定牌面和正逆位，用于记录已知的占卜结果。

### 5. 带切牌的随机抽牌

```rust
divine_random_with_cut(
    spread_type: SpreadType,
    cut_position: Option<u8>,  // 切牌位置 1-77，None 为随机
    question_hash: [u8; 32],
    privacy_mode: PrivacyMode,
)
```

模拟真实塔罗仪式，包含洗牌-切牌-抽牌的完整流程。

### 6. 隐私模式管理

```rust
set_reading_privacy_mode(
    reading_id: u64,
    privacy_mode: PrivacyMode,
)
```

### 7. 删除占卜记录

```rust
delete_reading(reading_id: u64)
```

删除记录并返还存储押金。

## 数据结构

### TarotReading - 占卜记录

```rust
pub struct TarotReading<AccountId, BlockNumber, MaxCards> {
    pub id: u64,
    pub diviner: AccountId,
    pub spread_type: SpreadType,
    pub method: DivinationMethod,
    pub cards: BoundedVec<DrawnCard, MaxCards>,
    pub question_hash: [u8; 32],
    pub block_number: BlockNumber,
    pub timestamp: u64,
    pub interpretation_cid: Option<BoundedVec<u8, ConstU32<64>>>,
    pub privacy_mode: PrivacyMode,
}
```

### DrawnCard - 抽取的牌

```rust
pub struct DrawnCard {
    pub card: TarotCard,
    pub position: CardPosition,
}

pub struct CardPosition {
    pub index: u8,        // 在牌阵中的位置
    pub reversed: bool,   // 是否逆位
}
```

### DivinationMethod - 起卦方式

```rust
pub enum DivinationMethod {
    Random,           // 随机抽牌
    ByTime,           // 时间起卦
    ByNumbers,        // 数字起卦
    Manual,           // 手动指定
    RandomWithCut,    // 带切牌随机
}
```

### PrivacyMode - 隐私模式

```rust
pub enum PrivacyMode {
    Public,      // 公开
    Private,     // 私密
    Authorized,  // 授权可见
}
```

## 用户统计

模块自动维护用户统计数据：

```rust
pub struct DivinationStats {
    pub total_readings: u32,      // 总占卜次数
    pub major_arcana_count: u32,  // 大阿卡纳出现次数
    pub reversed_count: u32,      // 逆位出现次数
    pub most_frequent_card: u8,   // 最常出现的牌
    pub most_frequent_count: u32, // 最常出现牌的次数
}
```

## 配置参数

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    /// 每次占卜最大牌数
    type MaxCardsPerReading: Get<u32>;
    
    /// 每用户最多存储的占卜记录数
    type MaxUserReadings: Get<u32>;
    
    /// 每日免费占卜次数
    type DailyFreeDivinations: Get<u32>;
    
    /// 每日最大占卜次数（防刷）
    type MaxDailyDivinations: Get<u32>;
    
    /// AI 解读费用
    type AiInterpretationFee: Get<Balance>;
    
    /// 存储押金配置
    type StorageDepositPerKb: Get<u128>;
    type MinStorageDeposit: Get<u128>;
    type MaxStorageDeposit: Get<u128>;
}
```

## 事件

```rust
ReadingCreated { reading_id, diviner, spread_type, method }
AiInterpretationRequested { reading_id, requester }
AiInterpretationSubmitted { reading_id, cid }
ReadingPrivacyModeChanged { reading_id, privacy_mode }
ReadingDeleted { reading_id, owner }
StorageDepositLocked { reading_id, owner, deposit, privacy_mode }
StorageDepositRefunded { reading_id, owner, refund, treasury }
```

## 防滥用机制

- 每日免费占卜次数限制
- 每日最大占卜次数限制
- 存储押金机制

## 依赖

```toml
[dependencies]
pallet-tarot = { path = "../tarot", default-features = false }
pallet-divination-common = { path = "../common", default-features = false }
pallet-divination-privacy = { path = "../privacy", default-features = false }
```

## License

Apache-2.0
