# 玄学公共模块 (pallet-divination-common)

玄学公共模块是 Cosmos 链上玄学生态的基础设施层，为所有玄学系统（梅花易数、八字排盘、六爻、塔罗等）提供统一的类型定义、核心 Trait 和工具函数。

## 概述

本模块作为玄学系统的公共依赖，提供：

- **统一类型定义**：`DivinationType`、`Rarity`、`RarityInput` 等
- **核心 Trait**：`DivinationProvider`、`InterpretationContextGenerator`、`StorageDepositManager`
- **状态枚举**：订单状态、解读状态、争议状态等
- **存储押金管理**：`PrivacyMode`、`DepositConfig`、押金计算与返还逻辑

## 模块架构

```text
┌─────────────────────────────────────────────────────────────────┐
│                        应用层 (Application Layer)                │
├─────────────┬─────────────┬─────────────┬─────────────┬─────────┤
│ pallet-     │ pallet-     │ pallet-     │ pallet-     │ ...     │
│ meihua      │ bazi-chart  │ liuyao      │ tarot       │         │
├─────────────┴─────────────┴─────────────┴─────────────┴─────────┤
│                        公共服务层 (Common Service Layer)          │
├─────────────────────┬─────────────────────┬─────────────────────┤
│ pallet-divination-  │ pallet-divination-  │ pallet-divination-  │
│ ai                  │ market              │ nft                 │
├─────────────────────┴─────────────────────┴─────────────────────┤
│              pallet-divination-common (本模块)                    │
└─────────────────────────────────────────────────────────────────┘
```

## 核心类型

### DivinationType - 占卜类型枚举

```rust
pub enum DivinationType {
    Meihua,      // 梅花易数
    Bazi,        // 八字排盘
    Liuyao,      // 六爻占卜
    Qimen,       // 奇门遁甲
    Ziwei,       // 紫微斗数
    Tarot,       // 塔罗牌
    Daliuren,    // 大六壬
    Xiaoliuren,  // 小六壬
    // ... 预留扩展
}
```

### Rarity - 稀有度等级

```rust
pub enum Rarity {
    Common,     // 普通 (0-100分)
    Rare,       // 稀有 (101-200分)
    Epic,       // 史诗 (201-400分)
    Legendary,  // 传说 (401+分)
}
```

### RarityInput - 稀有度计算输入

```rust
pub struct RarityInput {
    pub primary_score: u8,           // 主要分数 (0-100)
    pub secondary_score: u8,         // 次要分数 (0-100)
    pub is_special_date: bool,       // 是否特殊日期
    pub is_special_combination: bool, // 是否特殊组合
    pub custom_factors: [u8; 4],     // 自定义因子
}
```

## 核心 Trait

### DivinationProvider

占卜结果查询接口，各玄学系统需实现此 trait：

```rust
pub trait DivinationProvider<AccountId> {
    /// 检查占卜结果是否存在
    fn result_exists(divination_type: DivinationType, result_id: u64) -> bool;
    
    /// 获取占卜结果创建者
    fn result_creator(divination_type: DivinationType, result_id: u64) -> Option<AccountId>;
    
    /// 获取稀有度计算数据
    fn rarity_data(divination_type: DivinationType, result_id: u64) -> Option<RarityInput>;
    
    /// 检查是否可铸造 NFT
    fn is_nftable(divination_type: DivinationType, result_id: u64) -> bool;
    
    /// 标记为已铸造 NFT
    fn mark_as_nfted(divination_type: DivinationType, result_id: u64);
}
```

### StorageDepositManager

存储押金管理接口：

```rust
pub trait StorageDepositManager<AccountId, Balance, BlockNumber> {
    /// 锁定存储押金
    fn reserve_storage_deposit(
        who: &AccountId,
        data_size: u32,
        privacy_mode: PrivacyMode,
    ) -> Result<Balance, DispatchError>;
    
    /// 返还存储押金
    fn unreserve_storage_deposit(
        who: &AccountId,
        record_id: u64,
    ) -> Result<(Balance, Balance), DispatchError>;
}
```

## 存储押金机制

### 押金计算

```rust
/// 计算存储押金
/// deposit = max(min_deposit, min(data_size * rate_per_kb / 1024, max_deposit))
pub fn calculate_storage_deposit(
    data_size: u32,
    config: &DepositConfig,
) -> u128;
```

### 返还规则

- **30天内删除**：100% 返还
- **30天后删除**：90% 返还，10% 进入国库

### 隐私模式加成

| 隐私模式 | 押金倍率 |
|---------|---------|
| Public  | 1.0x    |
| Private | 1.5x    |
| Authorized | 2.0x |

## 使用示例

### 在 Runtime 中实现 DivinationProvider

```rust
use pallet_divination_common::{DivinationProvider, DivinationType, RarityInput};

pub struct MeihuaDivinationProvider;

impl DivinationProvider<AccountId> for MeihuaDivinationProvider {
    fn result_exists(divination_type: DivinationType, result_id: u64) -> bool {
        match divination_type {
            DivinationType::Meihua => Meihua::hexagrams(result_id).is_some(),
            _ => false,
        }
    }

    fn rarity_data(divination_type: DivinationType, result_id: u64) -> Option<RarityInput> {
        match divination_type {
            DivinationType::Meihua => {
                Meihua::hexagrams(result_id).map(|h| {
                    let is_pure = h.ben_gua.shang_gua == h.ben_gua.xia_gua;
                    RarityInput {
                        primary_score: if is_pure { 80 } else { 30 },
                        secondary_score: 10,
                        is_special_date: false,
                        is_special_combination: is_pure,
                        custom_factors: [0, 0, 0, 0],
                    }
                })
            },
            _ => None,
        }
    }
    // ... 其他方法
}
```

### 使用稀有度计算

```rust
use pallet_divination_common::{RarityInput, Rarity};

let input = RarityInput {
    primary_score: 80,
    secondary_score: 20,
    is_special_date: true,
    is_special_combination: true,
    custom_factors: [0, 0, 0, 0],
};

let rarity = input.calculate_rarity();
assert_eq!(rarity, Rarity::Legendary);
```

## 扩展新玄学系统

添加新的玄学系统时：

1. 在 `DivinationType` 枚举中添加新类型（已预留）
2. 创建新的核心 pallet（如 `pallet-liuyao`）
3. 在 Runtime 中实现 `DivinationProvider` trait
4. 公共服务模块（AI、Market、NFT）自动支持新系统

## 依赖

```toml
[dependencies]
pallet-divination-common = { path = "../common", default-features = false }
```

## License

MIT
