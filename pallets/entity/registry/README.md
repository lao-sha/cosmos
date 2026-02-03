# pallet-entity-shop

> 🏪 Entity 店铺管理模块 - 店铺生命周期管理与运营资金管理

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Substrate](https://img.shields.io/badge/Substrate-polkadot--sdk-blue)](https://github.com/paritytech/polkadot-sdk)

## 📖 概述

`pallet-entity-shop` 是 Entity 商城系统的店铺管理模块，负责店铺的完整生命周期管理，包括创建、运营资金管理、状态管理和治理审核。

### 核心功能

- 🏪 **店铺创建** - 转入 USDT 等值 COS 运营资金到派生账户
- 💰 **运营资金管理** - 充值、消费、健康监控
- ✏️ **店铺更新** - 修改店铺信息
- 🔒 **申请关闭** - 需治理审批，关闭后退还全部余额
- ✅ **治理审核** - 批准、暂停、恢复、封禁、审批关闭

## 🏗️ 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    pallet-entity-shop                        │
│                      (店铺管理模块)                              │
├─────────────────────────────────────────────────────────────────┤
│  • 店铺 CRUD 操作                                                │
│  • 派生账户运营资金管理                                          │
│  • 资金健康状态监控                                              │
│  • 店铺状态管理                                                  │
│  • 治理审核                                                      │
└─────────────────────────────────────────────────────────────────┘
         │                              │
         │ PricingProvider              │ ShopProvider
         ▼                              ▼
┌─────────────────────┐    ┌─────────────────────────────────────┐
│  pallet-trading     │    │        pallet-entity-product     │
│     -pricing        │    │           (商品模块)                 │
│    (定价模块)        │    │  • 验证店铺存在性                    │
│  • COS/USDT 价格    │    │  • 验证店主身份                      │
└─────────────────────┘    └─────────────────────────────────────┘
```

## 💰 运营资金机制

### 核心设计

创建店铺时，系统会根据实时 COS/USDT 价格计算 **50 USDT 等值的 COS** 转入**店铺派生账户**。

```
┌─────────────────────────────────────────────────────────────────┐
│                    店铺派生账户余额结构                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              全部余额（不可提取）                         │   │
│  │                                                         │   │
│  │  ✅ 可消费：IPFS Pin、存储租金、手续费                   │   │
│  │  ✅ 可充值：店主随时充值                                 │   │
│  │  ❌ 不可提取：运营期间锁定                               │   │
│  │  ✅ 关闭退还：治理审批关闭后全额退还                     │   │
│  │                                                         │   │
│  │  ⚠️ 低于最低余额时店铺自动暂停                          │   │
│  │                                                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  地址: PalletId(*b"et/shop/").into_sub_account(shop_id)        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 资金健康状态

| 状态 | 条件 | 说明 |
|------|------|------|
| `Healthy` | 余额 > 预警阈值 | 正常运营 |
| `Warning` | 最低余额 < 余额 ≤ 预警阈值 | 发出预警 |
| `Critical` | 余额 ≤ 最低余额 | 店铺暂停 |
| `Depleted` | 余额 = 0 | 资金耗尽 |

### 计算公式

```
COS 运营资金 = USDT 金额 × 10^12 / COS价格
```

### 示例

| COS/USDT 价格 | 50 USDT 等值 COS |
|---------------|------------------|
| 0.000001 | 50,000,000 COS |
| 0.0001 | 500,000 COS |
| 0.001 | 50,000 COS |
| 0.01 | 5,000 COS |
| 0.1 | 500 COS |

### 安全限制

```rust
// 初始资金限制在合理范围内
let final_fund = cos_amount
    .max(T::MinInitialFundCos::get())  // 最小 10 COS
    .min(T::MaxInitialFundCos::get()); // 最大 100,000 COS
```

## 📦 安装

### Cargo.toml

```toml
[dependencies]
pallet-entity-shop = { path = "pallets/entity/shop", default-features = false }

[features]
std = [
    "pallet-entity-shop/std",
]
```

## ⚙️ Runtime 配置

```rust
parameter_types! {
    /// 初始运营资金：50 USDT（精度 10^6）
    pub const InitialFundUsdt: u64 = 50_000_000;
    /// 最小初始资金：10 COS
    pub const MinInitialFundCos: Balance = 10 * UNIT;
    /// 最大初始资金：100,000 COS
    pub const MaxInitialFundCos: Balance = 100_000 * UNIT;
    /// 最低运营余额：100 COS（低于此值店铺暂停）
    pub const MinOperatingBalance: Balance = 100 * UNIT;
    /// 资金预警阈值：200 COS
    pub const FundWarningThreshold: Balance = 200 * UNIT;
}

impl pallet_entity_shop::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxShopNameLength = ConstU32<64>;
    type MaxCidLength = ConstU32<64>;
    type GovernanceOrigin = EnsureRoot<AccountId>;
    type PricingProvider = TradingPricing;
    type InitialFundUsdt = InitialFundUsdt;
    type MinInitialFundCos = MinInitialFundCos;
    type MaxInitialFundCos = MaxInitialFundCos;
    type MinOperatingBalance = MinOperatingBalance;
    type FundWarningThreshold = FundWarningThreshold;
}
```

### 配置参数说明

| 参数 | 类型 | 说明 | 示例值 |
|------|------|------|--------|
| `Currency` | Currency + ReservableCurrency | 货币类型 | `Balances` |
| `MaxShopNameLength` | u32 | 店铺名称最大长度 | 64 |
| `MaxCidLength` | u32 | CID 最大长度 | 64 |
| `GovernanceOrigin` | EnsureOrigin | 治理 Origin | `EnsureRoot` |
| `PricingProvider` | PricingProvider | 定价提供者 | `TradingPricing` |
| `InitialFundUsdt` | u64 | **初始资金 USDT（精度 10^6）** | 50_000_000 |
| `MinInitialFundCos` | Balance | **最小初始资金 COS** | 10 UNIT |
| `MaxInitialFundCos` | Balance | **最大初始资金 COS** | 100,000 UNIT |
| `MinOperatingBalance` | Balance | **最低运营余额** | 100 UNIT |
| `FundWarningThreshold` | Balance | **资金预警阈值** | 200 UNIT |

## 📊 数据结构

### Shop - 店铺信息

```rust
pub struct Shop<AccountId, Balance, BlockNumber, MaxNameLen, MaxCidLen> {
    pub id: u64,                              // 店铺 ID
    pub owner: AccountId,                     // 店主账户
    pub customer_service: Option<AccountId>,  // 客服聊天账户（Pallet Chat）
    pub name: BoundedVec<u8, MaxNameLen>,     // 店铺名称
    pub logo_cid: Option<BoundedVec<u8, MaxCidLen>>,        // Logo CID
    pub description_cid: Option<BoundedVec<u8, MaxCidLen>>, // 描述 CID
    pub initial_fund: Balance,                // 初始运营资金
    pub status: ShopStatus,                   // 店铺状态
    pub product_count: u32,                   // 商品数量
    pub total_sales: Balance,                 // 累计销售额
    pub total_orders: u32,                    // 累计订单数
    pub rating: u16,                          // 店铺评分 (0-500)
    pub rating_count: u32,                    // 评价数量
    pub created_at: BlockNumber,              // 创建时间
}
```

### ShopStatus - 店铺状态

```rust
pub enum ShopStatus {
    Pending,    // 待审核 / 待关闭审批
    Active,     // 正常营业
    Suspended,  // 暂停营业（治理暂停或资金不足）
    Banned,     // 被封禁（治理处罚）
    Closed,     // 已关闭
}
```

### FundHealth - 资金健康状态

```rust
pub enum FundHealth {
    Healthy,   // 健康（余额 > 预警阈值）
    Warning,   // 预警（最低余额 < 余额 ≤ 预警阈值）
    Critical,  // 危险（余额 ≤ 最低余额）
    Depleted,  // 耗尽（余额 = 0）
}
```

### FeeType - 运营费用类型

```rust
pub enum FeeType {
    IpfsPin,        // IPFS Pin 费用
    StorageRent,    // 链上存储租金
    TransactionFee, // 交易手续费
    Promotion,      // 推广费用
    Other,          // 其他费用
}
```

## 🔧 Extrinsics

### 1. create_shop

创建店铺（转入运营资金到派生账户）。

```rust
fn create_shop(
    origin: OriginFor<T>,
    name: Vec<u8>,
    logo_cid: Option<Vec<u8>>,
    description_cid: Option<Vec<u8>>,
) -> DispatchResult
```

**运营资金计算：**
- 获取实时 COS/USDT 价格
- 计算 50 USDT 等值的 COS
- 限制在 [MinInitialFundCos, MaxInitialFundCos] 范围内
- 转入店铺派生账户（不可提取）

### 2. update_shop

更新店铺信息。

```rust
fn update_shop(
    origin: OriginFor<T>,
    shop_id: u64,
    name: Option<Vec<u8>>,
    logo_cid: Option<Vec<u8>>,
    description_cid: Option<Vec<u8>>,
    customer_service: Option<AccountId>,
) -> DispatchResult
```

**权限：** 仅店主

**参数：**
- `customer_service` - 客服聊天账户（用于 Pallet Chat，None 表示使用店主账户）

### 3. request_close_shop

申请关闭店铺（需治理审批）。

```rust
fn request_close_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult
```

**权限：** 仅店主
**说明：** 申请后需等待治理审批，审批通过后全部余额退还

### 4. top_up_fund

充值运营资金。

```rust
fn top_up_fund(
    origin: OriginFor<T>,
    shop_id: u64,
    amount: BalanceOf<T>,
) -> DispatchResult
```

**权限：** 仅店主
**说明：** 如果店铺因资金不足暂停，充值后自动恢复

### 5. approve_shop（治理）

审核通过店铺。

```rust
fn approve_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult
```

**权限：** GovernanceOrigin

### 6. approve_close_shop（治理）

审批关闭店铺（退还全部余额）。

```rust
fn approve_close_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult
```

**权限：** GovernanceOrigin
**说明：** 退还派生账户全部余额给店主

### 7. suspend_shop（治理）

暂停店铺营业。

```rust
fn suspend_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult
```

**权限：** GovernanceOrigin

### 8. resume_shop（治理）

恢复店铺营业（需资金充足）。

```rust
fn resume_shop(origin: OriginFor<T>, shop_id: u64) -> DispatchResult
```

**权限：** GovernanceOrigin
**前提：** 派生账户余额 ≥ 最低运营余额

### 9. ban_shop（治理）

封禁店铺（可选没收资金）。

```rust
fn ban_shop(
    origin: OriginFor<T>,
    shop_id: u64,
    confiscate_fund: bool,
) -> DispatchResult
```

**权限：** GovernanceOrigin
**参数：** `confiscate_fund` - 是否没收资金

## 📡 Events

| 事件 | 说明 | 字段 |
|------|------|------|
| `ShopCreated` | 店铺已创建 | `shop_id`, `owner`, `shop_account`, `initial_fund` |
| `ShopUpdated` | 店铺已更新 | `shop_id` |
| `ShopStatusChanged` | 店铺状态已变更 | `shop_id`, `status` |
| `FundToppedUp` | **运营资金已充值** | `shop_id`, `amount`, `new_balance` |
| `OperatingFeeDeducted` | **运营费用已扣除** | `shop_id`, `fee`, `fee_type`, `remaining_balance` |
| `FundWarning` | **资金预警** | `shop_id`, `current_balance`, `warning_threshold` |
| `ShopSuspendedLowFund` | **店铺因资金不足暂停** | `shop_id`, `current_balance`, `minimum_balance` |
| `ShopResumedAfterFunding` | **充值后店铺恢复** | `shop_id` |
| `ShopCloseRequested` | **店主申请关闭店铺** | `shop_id` |
| `ShopClosed` | 店铺已关闭 | `shop_id`, `fund_refunded` |
| `ShopBanned` | **店铺被封禁** | `shop_id`, `fund_confiscated` |
| `FundConfiscated` | **资金被没收** | `shop_id`, `amount` |

## ❌ Errors

| 错误 | 说明 |
|------|------|
| `ShopNotFound` | 店铺不存在 |
| `ShopAlreadyExists` | 用户已有店铺 |
| `NotShopOwner` | 不是店主 |
| `ShopNotActive` | 店铺未激活 |
| `ShopHasPendingOrders` | 店铺有进行中的订单 |
| `InsufficientOperatingFund` | **运营资金不足** |
| `InvalidShopStatus` | 无效的店铺状态 |
| `NameTooLong` | 名称过长 |
| `CidTooLong` | CID 过长 |
| `PriceUnavailable` | 价格不可用 |
| `ArithmeticOverflow` | 算术溢出 |
| `InsufficientBalanceForInitialFund` | **余额不足以支付初始资金** |

## 🔌 ShopProvider Trait

本模块实现了 `ShopProvider` trait，供其他模块调用：

```rust
pub trait ShopProvider<AccountId> {
    fn shop_exists(shop_id: u64) -> bool;
    fn is_shop_active(shop_id: u64) -> bool;
    fn shop_owner(shop_id: u64) -> Option<AccountId>;
    fn update_shop_stats(shop_id: u64, sales_amount: u128, order_count: u32) -> Result<(), DispatchError>;
    fn update_shop_rating(shop_id: u64, rating: u8) -> Result<(), DispatchError>;
}
```

## 🔌 辅助函数

```rust
impl<T: Config> Pallet<T> {
    /// 获取店铺派生账户
    pub fn shop_account(shop_id: u64) -> T::AccountId;
    
    /// 计算初始运营资金
    pub fn calculate_initial_fund() -> Result<BalanceOf<T>, DispatchError>;
    
    /// 获取资金健康状态
    pub fn get_fund_health(balance: BalanceOf<T>) -> FundHealth;
    
    /// 获取店铺运营资金余额
    pub fn get_shop_fund_balance(shop_id: u64) -> BalanceOf<T>;
    
    /// 扣除运营费用（供其他模块调用）
    pub fn deduct_operating_fee(shop_id: u64, fee: BalanceOf<T>, fee_type: FeeType) -> DispatchResult;
}
```

## 💡 店铺生命周期

```
┌─────────────────────────────────────────────────────────────────┐
│                        店铺生命周期                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. 创建店铺                                                     │
│     ├── 计算 50 USDT 等值 COS                                   │
│     ├── 转入派生账户（不可提取）                                 │
│     └── 状态: Pending                                           │
│                                                                 │
│  2. 治理审核                                                     │
│     └── approve_shop() → 状态: Active                           │
│                                                                 │
│  3. 正常运营                                                     │
│     ├── 消费运营费用（IPFS Pin 等）                              │
│     ├── 可随时充值                                               │
│     └── 资金不足 → 状态: Suspended                              │
│                                                                 │
│  4. 充值恢复                                                     │
│     └── top_up_fund() → 状态: Active                            │
│                                                                 │
│  5. 申请关闭                                                     │
│     └── request_close_shop() → 状态: Pending                    │
│                                                                 │
│  6. 治理审批关闭                                                 │
│     └── approve_close_shop() → 状态: Closed                     │
│         └── 全部余额退还给店主                                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 🔐 安全机制

### 1. 派生账户隔离

```rust
const SHOP_PALLET_ID: PalletId = PalletId(*b"et/shop/");

pub fn shop_account(shop_id: u64) -> T::AccountId {
    SHOP_PALLET_ID.into_sub_account_truncating(shop_id)
}
```

每个店铺有独立的派生账户，资金隔离。

### 2. 资金不可提取

运营期间资金完全锁定在派生账户，仅可用于：
- 支付运营费用
- 治理关闭后退还

### 3. 资金健康监控

| 状态 | 触发条件 | 行为 |
|------|----------|------|
| Warning | 余额 ≤ 预警阈值 | 发出预警事件 |
| Critical | 余额 ≤ 最低余额 | 自动暂停店铺 |

### 4. 一用户一店铺

```rust
ensure!(!UserShop::<T>::contains_key(&who), Error::<T>::ShopAlreadyExists);
```

**目的**：防止刷店铺

## 🧪 测试

```bash
# 运行单元测试
cargo test -p pallet-entity-shop

# 运行特定测试
cargo test -p pallet-entity-shop test_create_shop
```

## 📝 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1.0 | 2026-01-31 | 从 pallet-mall 拆分 |
| v0.2.0 | 2026-02-01 | 实现 USDT 等值 COS 押金机制 |

## 📄 许可证

MIT License

## 🔗 相关链接

- [pallet-entity-product](../product/README.md)
- [pallet-entity-common](../common/README.md)
- [pallet-trading-pricing](../../trading/pricing/README.md)
