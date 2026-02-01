# pallet-sharemall-member

> 👥 ShareMall 店铺会员管理模块 - 会员推荐关系与三级分销返佣

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Substrate](https://img.shields.io/badge/Substrate-polkadot--sdk-blue)](https://github.com/paritytech/polkadot-sdk)

## 📖 概述

`pallet-sharemall-member` 是 ShareMall 商城系统的店铺会员管理模块，实现**每个店铺独立的会员推荐关系**，支持三级分销返佣机制。

### 核心功能

- 👥 **会员注册** - 用户注册成为店铺会员，可填写推荐人
- 🔗 **推荐关系** - 绑定推荐人，建立上下级关系
- 💰 **三级返佣** - 订单完成时，推荐人获得返佣（最多三级）
- ⭐ **会员等级** - 根据消费金额自动升级会员等级
- 🎨 **自定义等级** - 店铺可完全自定义等级体系（名称、阈值、权益）
- 📊 **推荐统计** - 查询推荐人数、返佣金额、团队规模
- ⚙️ **店铺配置** - 店主可自定义返佣比例和来源

## 🏗️ 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    pallet-sharemall-member                       │
│                      (店铺会员管理模块)                           │
├─────────────────────────────────────────────────────────────────┤
│  • 会员注册与管理                                                 │
│  • 推荐关系绑定                                                   │
│  • 推荐返佣计算与发放                                             │
│  • 会员等级管理                                                   │
│  • 推荐统计查询                                                   │
└─────────────────────────────────────────────────────────────────┘
         │              │              │
         │ ShopProvider │ Currency     │ MemberProvider (对外)
         ▼              ▼              ▼
┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│    shop     │ │  Balances   │ │    order    │
│   (店铺)    │ │   (余额)    │ │   (订单)    │
│ • 店铺验证  │ │ • 返佣转账  │ │ • 订单回调  │
│ • 派生账户  │ │             │ │ • 消费更新  │
└─────────────┘ └─────────────┘ └─────────────┘
```

## 💰 三级分销返佣模型

```
┌─────────────────────────────────────────────────────────────────┐
│                      三级分销返佣模型                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  买家 David 消费 1000 COS                                        │
│                                                                 │
│  推荐链：Alice → Bob → Carol → David                            │
│                                                                 │
│  返佣分配（默认比例）：                                          │
│  ├── Carol (一级推荐人): 1000 × 5% = 50 COS                     │
│  ├── Bob   (二级推荐人): 1000 × 2% = 20 COS                     │
│  └── Alice (三级推荐人): 1000 × 1% = 10 COS                     │
│                                                                 │
│  总返佣：80 COS（从平台费中扣除）                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 📦 安装

### Cargo.toml

```toml
[dependencies]
pallet-sharemall-member = { path = "pallets/sharemall/member", default-features = false }

[features]
std = [
    "pallet-sharemall-member/std",
]
```

## ⚙️ Runtime 配置

```rust
parameter_types! {
    /// 最大直接推荐人数
    pub const MaxDirectReferrals: u32 = 1000;
    /// 最大自定义等级数量
    pub const MaxCustomLevels: u32 = 20;
    /// 银卡会员消费阈值（100 USDT）
    pub const SilverThreshold: u64 = 100_000_000;
    /// 金卡会员消费阈值（500 USDT）
    pub const GoldThreshold: u64 = 500_000_000;
    /// 白金会员消费阈值（2000 USDT）
    pub const PlatinumThreshold: u64 = 2_000_000_000;
    /// 钻石会员消费阈值（10000 USDT）
    pub const DiamondThreshold: u64 = 10_000_000_000;
}

impl pallet_sharemall_member::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ShopProvider = ShareMallShop;
    type MaxDirectReferrals = MaxDirectReferrals;
    type MaxCustomLevels = MaxCustomLevels;
    type SilverThreshold = SilverThreshold;
    type GoldThreshold = GoldThreshold;
    type PlatinumThreshold = PlatinumThreshold;
    type DiamondThreshold = DiamondThreshold;
}
```

## 📊 数据结构

### ShopMember - 店铺会员

```rust
pub struct ShopMember<AccountId, Balance, BlockNumber> {
    pub referrer: Option<AccountId>,      // 推荐人（上级）
    pub direct_referrals: u32,            // 直接推荐人数
    pub team_size: u32,                   // 团队总人数
    pub total_spent: Balance,             // 累计消费金额
    pub total_commission: Balance,        // 累计获得返佣
    pub total_contributed: Balance,       // 累计贡献返佣
    pub pending_commission: Balance,      // 待提取返佣
    pub level: MemberLevel,               // 会员等级
    pub joined_at: BlockNumber,           // 加入时间
    pub last_active_at: BlockNumber,      // 最后活跃时间
}
```

### MemberLevel - 会员等级

```rust
pub enum MemberLevel {
    Normal,     // 普通会员 - 无门槛
    Silver,     // 银卡会员 - 消费满 100 USDT
    Gold,       // 金卡会员 - 消费满 500 USDT
    Platinum,   // 白金会员 - 消费满 2000 USDT
    Diamond,    // 钻石会员 - 消费满 10000 USDT
}
```

### CommissionConfig - 返佣配置

```rust
pub struct CommissionConfig {
    pub level1_rate: u16,       // 一级返佣率（基点，500 = 5%）
    pub level2_rate: u16,       // 二级返佣率（基点，200 = 2%）
    pub level3_rate: u16,       // 三级返佣率（基点，100 = 1%）
    pub source: CommissionSource, // 返佣来源
    pub enabled: bool,          // 是否启用
}

pub enum CommissionSource {
    PlatformFee,    // 从平台费中扣除
    ShopFund,       // 店铺运营资金承担
    Mixed,          // 混合模式
}
```

### CustomLevel - 自定义等级

```rust
pub struct CustomLevel<Balance> {
    pub id: u8,                              // 等级 ID（0, 1, 2, ...）
    pub name: BoundedVec<u8, ConstU32<32>>,  // 等级名称（如 "VIP", "黑卡"）
    pub threshold: Balance,                  // 升级阈值（累计消费）
    pub discount_rate: u16,                  // 折扣率（基点，500 = 5% 折扣）
    pub commission_bonus: u16,               // 返佣加成（基点，100 = 1% 额外返佣）
}
```

### ShopLevelSystem - 店铺等级系统

```rust
pub struct ShopLevelSystem<Balance, MaxLevels> {
    pub levels: BoundedVec<CustomLevel<Balance>, MaxLevels>, // 自定义等级列表
    pub use_custom: bool,                                     // 是否启用自定义等级
    pub upgrade_mode: LevelUpgradeMode,                       // 等级升级方式
}

pub enum LevelUpgradeMode {
    AutoUpgrade,      // 自动升级（消费达标即升）
    ManualUpgrade,    // 手动升级（需店主审批）
    PeriodReset,      // 周期重置（每月/每年重新计算）
}
```

## 🎨 自定义等级体系

店铺可以完全自定义会员等级，不受全局 5 级体系限制：

```
┌─────────────────────────────────────────────────────────────────┐
│                      自定义等级示例                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  店铺 A（奢侈品店）：                                            │
│  ├── Level 0: 普通客户 (0 USDT)                                 │
│  ├── Level 1: 尊享会员 (5000 USDT) - 5% 折扣                    │
│  ├── Level 2: 至尊会员 (20000 USDT) - 10% 折扣                  │
│  └── Level 3: 黑卡会员 (100000 USDT) - 15% 折扣                 │
│                                                                 │
│  店铺 B（日用品店）：                                            │
│  ├── Level 0: 新用户 (0 USDT)                                   │
│  ├── Level 1: 铜牌 (20 USDT) - 2% 折扣                          │
│  ├── Level 2: 银牌 (50 USDT) - 3% 折扣                          │
│  ├── Level 3: 金牌 (100 USDT) - 5% 折扣                         │
│  └── Level 4: 钻石 (500 USDT) - 8% 折扣                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 等级权益

| 权益 | 说明 |
|------|------|
| **折扣率** | 购物时自动享受折扣（基点，500 = 5%） |
| **返佣加成** | 推荐他人时获得额外返佣加成（基点） |

## 🔧 Extrinsics

### 1. register_member

注册成为店铺会员。

```rust
fn register_member(
    origin: OriginFor<T>,
    shop_id: u64,
    referrer: Option<T::AccountId>,
) -> DispatchResult
```

**参数：**
- `shop_id` - 店铺 ID
- `referrer` - 推荐人账户（可选）

**权限：** 任意用户

### 2. bind_referrer

绑定推荐人（未绑定过的会员）。

```rust
fn bind_referrer(
    origin: OriginFor<T>,
    shop_id: u64,
    referrer: T::AccountId,
) -> DispatchResult
```

**权限：** 会员（未绑定过推荐人）

### 3. set_commission_config

设置店铺返佣配置。

```rust
fn set_commission_config(
    origin: OriginFor<T>,
    shop_id: u64,
    level1_rate: u16,
    level2_rate: u16,
    level3_rate: u16,
    source: CommissionSource,
    enabled: bool,
) -> DispatchResult
```

**权限：** 仅店主

### 4. withdraw_commission

提取返佣。

```rust
fn withdraw_commission(
    origin: OriginFor<T>,
    shop_id: u64,
    amount: Option<BalanceOf<T>>,
) -> DispatchResult
```

**权限：** 会员

### 5. init_level_system

初始化店铺等级系统。

```rust
fn init_level_system(
    origin: OriginFor<T>,
    shop_id: u64,
    use_custom: bool,
    upgrade_mode: LevelUpgradeMode,
) -> DispatchResult
```

**权限：** 仅店主

### 6. add_custom_level

添加自定义等级。

```rust
fn add_custom_level(
    origin: OriginFor<T>,
    shop_id: u64,
    name: BoundedVec<u8, ConstU32<32>>,
    threshold: BalanceOf<T>,
    discount_rate: u16,
    commission_bonus: u16,
) -> DispatchResult
```

**权限：** 仅店主

### 7. update_custom_level

更新自定义等级。

```rust
fn update_custom_level(
    origin: OriginFor<T>,
    shop_id: u64,
    level_id: u8,
    name: Option<BoundedVec<u8, ConstU32<32>>>,
    threshold: Option<BalanceOf<T>>,
    discount_rate: Option<u16>,
    commission_bonus: Option<u16>,
) -> DispatchResult
```

**权限：** 仅店主

### 8. remove_custom_level

删除自定义等级（只能删除最后一个）。

```rust
fn remove_custom_level(
    origin: OriginFor<T>,
    shop_id: u64,
    level_id: u8,
) -> DispatchResult
```

**权限：** 仅店主

### 9. manual_upgrade_member

手动升级会员（仅 ManualUpgrade 模式）。

```rust
fn manual_upgrade_member(
    origin: OriginFor<T>,
    shop_id: u64,
    member: T::AccountId,
    target_level_id: u8,
) -> DispatchResult
```

**权限：** 仅店主

### 10. set_use_custom_levels

切换是否使用自定义等级。

```rust
fn set_use_custom_levels(
    origin: OriginFor<T>,
    shop_id: u64,
    use_custom: bool,
) -> DispatchResult
```

**权限：** 仅店主

## 📡 Events

| 事件 | 说明 | 字段 |
|------|------|------|
| `MemberRegistered` | 会员注册 | `shop_id`, `account`, `referrer` |
| `ReferrerBound` | 绑定推荐人 | `shop_id`, `account`, `referrer` |
| `CommissionDistributed` | 返佣发放 | `shop_id`, `referrer`, `amount`, `level` |
| `CommissionWithdrawn` | 返佣提取 | `shop_id`, `account`, `amount` |
| `MemberLevelUpgraded` | 会员升级 | `shop_id`, `account`, `old_level`, `new_level` |
| `CustomLevelUpgraded` | 自定义等级升级 | `shop_id`, `account`, `old_level_id`, `new_level_id` |
| `CommissionConfigUpdated` | 返佣配置更新 | `shop_id`, `config` |
| `LevelSystemInitialized` | 等级系统初始化 | `shop_id`, `use_custom`, `upgrade_mode` |
| `CustomLevelAdded` | 自定义等级添加 | `shop_id`, `level_id`, `name`, `threshold` |
| `CustomLevelUpdated` | 自定义等级更新 | `shop_id`, `level_id` |
| `CustomLevelRemoved` | 自定义等级删除 | `shop_id`, `level_id` |
| `MemberManuallyUpgraded` | 手动升级会员 | `shop_id`, `account`, `level_id` |

## ❌ Errors

| 错误 | 说明 |
|------|------|
| `AlreadyMember` | 已是会员 |
| `NotMember` | 不是会员 |
| `ReferrerAlreadyBound` | 已绑定推荐人 |
| `InvalidReferrer` | 无效推荐人 |
| `SelfReferral` | 不能推荐自己 |
| `CircularReferral` | 循环推荐 |
| `CommissionNotConfigured` | 返佣未配置 |
| `InsufficientCommission` | 返佣余额不足 |
| `NotShopOwner` | 不是店主 |
| `ShopNotFound` | 店铺不存在 |
| `ReferralsFull` | 推荐人数已满 |
| `LevelSystemNotInitialized` | 等级系统未初始化 |
| `LevelAlreadyExists` | 等级已存在 |
| `LevelNotFound` | 等级不存在 |
| `LevelsFull` | 等级数量已满 |
| `InvalidLevelId` | 无效等级 ID |
| `InvalidThreshold` | 等级阈值无效 |
| `EmptyLevelName` | 等级名称为空 |
| `ManualUpgradeNotSupported` | 不支持手动升级 |
| `UpgradeRuleSystemNotInitialized` | 升级规则系统未初始化 |
| `UpgradeRuleNotFound` | 升级规则不存在 |
| `UpgradeRulesFull` | 升级规则数量已满 |
| `EmptyRuleName` | 规则名称为空 |
| `InvalidTargetLevel` | 无效目标等级 |

## 🎯 升级规则系统

店铺可配置多种升级规则，支持不同触发条件：

### 触发条件类型

| 类型 | 说明 |
|------|------|
| `PurchaseProduct` | 购买特定产品 |
| `TotalSpent` | 累计消费达标 |
| `SingleOrder` | 单笔消费达标 |
| `ReferralCount` | 推荐人数达标 |
| `TeamSize` | 团队人数达标 |
| `OrderCount` | 订单数量达标 |

### 规则冲突策略

| 策略 | 说明 |
|------|------|
| `HighestLevel` | 取最高等级（默认） |
| `HighestPriority` | 取最高优先级规则 |
| `LongestDuration` | 取最长有效期 |
| `FirstMatch` | 第一个匹配的规则 |

### 升级规则 Extrinsics

| call_index | 函数 | 说明 |
|------------|------|------|
| 11 | `init_upgrade_rule_system` | 初始化升级规则系统 |
| 12 | `add_upgrade_rule` | 添加升级规则 |
| 13 | `update_upgrade_rule` | 更新升级规则 |
| 14 | `remove_upgrade_rule` | 删除升级规则 |
| 15 | `set_upgrade_rule_system_enabled` | 设置规则系统启用状态 |
| 16 | `set_conflict_strategy` | 设置冲突策略 |

### 使用示例

```rust
// 1. 初始化升级规则系统
ShareMallMember::init_upgrade_rule_system(
    origin, shop_id, ConflictStrategy::HighestLevel
)?;

// 2. 添加规则：购买产品 #101 升级为 VIP（永久）
ShareMallMember::add_upgrade_rule(
    origin, shop_id,
    b"购买VIP会员卡".to_vec().try_into().unwrap(),
    UpgradeTrigger::PurchaseProduct { product_id: 101 },
    2,              // 目标等级 ID
    None,           // 永久
    10,             // 优先级
    true,           // 可叠加
    None,           // 无触发次数限制
)?;

// 3. 添加规则：累计消费 1000 USDT 升级为金卡
ShareMallMember::add_upgrade_rule(
    origin, shop_id,
    b"累计消费升金卡".to_vec().try_into().unwrap(),
    UpgradeTrigger::TotalSpent { threshold: 1000_000_000 },
    2,
    None,
    5,
    false,
    Some(1),        // 只触发一次
)?;
```

## 🔌 MemberProvider Trait

本模块实现了 `MemberProvider` trait，供其他模块调用：

```rust
pub trait MemberProvider<AccountId, Balance> {
    /// 检查是否为店铺会员
    fn is_member(shop_id: u64, account: &AccountId) -> bool;
    
    /// 获取会员等级
    fn member_level(shop_id: u64, account: &AccountId) -> Option<MemberLevel>;
    
    /// 获取自定义等级 ID
    fn custom_level_id(shop_id: u64, account: &AccountId) -> u8;
    
    /// 获取等级折扣率
    fn get_level_discount(shop_id: u64, level_id: u8) -> u16;
    
    /// 获取等级返佣加成
    fn get_level_commission_bonus(shop_id: u64, level_id: u8) -> u16;
    
    /// 检查店铺是否使用自定义等级
    fn uses_custom_levels(shop_id: u64) -> bool;
    
    /// 获取推荐人
    fn get_referrer(shop_id: u64, account: &AccountId) -> Option<AccountId>;
    
    /// 自动注册会员（首次下单时）
    fn auto_register(shop_id: u64, account: &AccountId, referrer: Option<AccountId>) -> DispatchResult;
    
    /// 更新消费金额
    fn update_spent(shop_id: u64, account: &AccountId, amount: Balance, amount_usdt: u64) -> DispatchResult;
    
    /// 发放推荐返佣
    fn distribute_commission(
        shop_id: u64,
        buyer: &AccountId,
        order_amount: Balance,
        available_commission: Balance,
    ) -> DispatchResult;
}
```

## 🔗 与订单模块集成

在 `pallet-sharemall-order` 的 `do_complete_order` 中调用：

```rust
// 自动注册会员（如果还不是）
T::MemberProvider::auto_register(order.shop_id, &order.buyer, referrer)?;

// 发放推荐返佣（从平台费中扣除）
T::MemberProvider::distribute_commission(
    order.shop_id,
    &order.buyer,
    order.total_amount,
    order.platform_fee,
)?;

// 更新会员消费金额
T::MemberProvider::update_spent(
    order.shop_id,
    &order.buyer,
    order.total_amount,
    amount_usdt,
)?;
```

## 💡 会员等级升级

会员等级根据累计消费金额自动升级：

| 等级 | 消费阈值（USDT） | 特权 |
|------|------------------|------|
| Normal | 0 | 基础会员 |
| Silver | 100 | 银卡会员 |
| Gold | 500 | 金卡会员 |
| Platinum | 2,000 | 白金会员 |
| Diamond | 10,000 | 钻石会员 |

## 🔒 安全机制

1. **循环推荐检测** - 防止 A → B → A 的循环推荐
2. **自我推荐检测** - 不能推荐自己
3. **推荐人验证** - 推荐人必须是店铺会员
4. **返佣上限** - 返佣从平台费中扣除，不会超过平台费总额
5. **店主权限** - 只有店主可以修改返佣配置

## 📈 推荐关系示例

```
店铺 A 的会员推荐关系：

Alice (推荐人: None)
├── Bob (推荐人: Alice)
│   ├── David (推荐人: Bob)
│   │   └── Frank (推荐人: David)
│   └── Eve (推荐人: Bob)
└── Carol (推荐人: Alice)
    └── Grace (推荐人: Carol)

当 Frank 消费 1000 COS 时：
- David (一级): 50 COS (5%)
- Bob (二级): 20 COS (2%)
- Alice (三级): 10 COS (1%)
```

## 📝 注意事项

1. **每个店铺独立** - 同一用户在不同店铺可以有不同的推荐人
2. **推荐人不可更改** - 绑定后不能更换推荐人
3. **返佣需提取** - 返佣记录在 `pending_commission`，需调用 `withdraw_commission` 提取
4. **店铺配置** - 店主需先调用 `set_commission_config` 启用返佣功能
