# pallet-sharemall-token

> 🏪 ShareMall 店铺代币模块 - pallet-assets 桥接层

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Substrate](https://img.shields.io/badge/Substrate-polkadot--sdk-blue)](https://github.com/paritytech/polkadot-sdk)

## 📖 概述

`pallet-sharemall-token` 是 ShareMall 商城系统的店铺代币模块，作为 `pallet-assets` 的桥接层，为每个店铺提供独立的代币（积分）系统。

### 核心功能

- 🎯 **店铺代币创建** - 每个店铺可发行专属代币
- 🎁 **购物返积分** - 消费自动获得店铺积分奖励
- 💰 **积分抵扣** - 使用积分抵扣订单金额
- 🔄 **积分转让** - 用户间自由转让积分
- ⚙️ **灵活配置** - 店主可自定义奖励率、兑换率等参数

## 🏗️ 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    pallet-sharemall-token                       │
│                        (桥接层)                                  │
├─────────────────────────────────────────────────────────────────┤
│  • 店铺代币配置管理                                              │
│  • 购物奖励逻辑                                                  │
│  • 积分兑换逻辑                                                  │
│  • 店铺 ID ↔ 资产 ID 映射                                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ fungibles::* traits
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       pallet-assets                             │
│                      (底层资产模块)                              │
├─────────────────────────────────────────────────────────────────┤
│  • 资产创建/销毁                                                 │
│  • 铸造/燃烧                                                     │
│  • 转账/授权                                                     │
│  • 冻结/解冻                                                     │
└─────────────────────────────────────────────────────────────────┘
```

## 📦 安装

### Cargo.toml

```toml
[dependencies]
pallet-sharemall-token = { path = "pallets/sharemall/token", default-features = false }

[features]
std = [
    "pallet-sharemall-token/std",
]
```

## ⚙️ Runtime 配置

```rust
parameter_types! {
    /// 店铺代币 ID 偏移量（避免与其他资产冲突）
    pub const ShareMallShopTokenOffset: u64 = 1_000_000;
}

impl pallet_sharemall_token::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AssetId = u64;
    type AssetBalance = Balance;
    type Assets = Assets;  // pallet-assets 实例
    type ShopProvider = ShareMallShop;
    type ShopTokenOffset = ShareMallShopTokenOffset;
    type MaxTokenNameLength = ConstU32<64>;
    type MaxTokenSymbolLength = ConstU32<8>;
}
```

## 📊 数据结构

### ShopTokenConfig

店铺代币配置结构：

```rust
pub struct ShopTokenConfig<Balance, BlockNumber> {
    /// 是否已启用代币
    pub enabled: bool,
    /// 购物返积分比例（基点，500 = 5%）
    pub reward_rate: u16,
    /// 积分兑换比例（基点，1000 = 10%，即 10 积分 = 1 元折扣）
    pub exchange_rate: u16,
    /// 最低兑换门槛
    pub min_redeem: Balance,
    /// 单笔最大兑换（0 = 无限制）
    pub max_redeem_per_order: Balance,
    /// 是否允许用户间转让
    pub transferable: bool,
    /// 创建时间
    pub created_at: BlockNumber,
}
```

### 配置参数说明

| 参数 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `reward_rate` | u16 | 购物返积分比例（基点） | 500 = 5% |
| `exchange_rate` | u16 | 积分兑换折扣比例（基点） | 1000 = 10% |
| `min_redeem` | Balance | 最低兑换门槛 | 100 积分 |
| `max_redeem_per_order` | Balance | 单笔最大兑换（0=无限制） | 1000 积分 |
| `transferable` | bool | 是否允许转让 | true |

## 🔧 Extrinsics

### 1. create_shop_token

为店铺创建代币。

```rust
fn create_shop_token(
    origin: OriginFor<T>,
    shop_id: u64,
    name: Vec<u8>,
    symbol: Vec<u8>,
    decimals: u8,
    reward_rate: u16,
    exchange_rate: u16,
) -> DispatchResult
```

**参数：**
- `shop_id` - 店铺 ID
- `name` - 代币名称（如 "星巴克积分"）
- `symbol` - 代币符号（如 "SBUX"）
- `decimals` - 小数位数（通常为 0 或 18）
- `reward_rate` - 购物返积分比例（基点）
- `exchange_rate` - 积分兑换比例（基点）

**权限：** 仅店主

**示例：**
```javascript
// Polkadot.js
api.tx.shareMallToken.createShopToken(
    1,                    // shop_id
    "Coffee Points",      // name
    "COFFEE",            // symbol
    0,                   // decimals
    500,                 // reward_rate: 5%
    1000                 // exchange_rate: 10%
)
```

### 2. update_token_config

更新代币配置。

```rust
fn update_token_config(
    origin: OriginFor<T>,
    shop_id: u64,
    reward_rate: Option<u16>,
    exchange_rate: Option<u16>,
    min_redeem: Option<T::AssetBalance>,
    max_redeem_per_order: Option<T::AssetBalance>,
    transferable: Option<bool>,
    enabled: Option<bool>,
) -> DispatchResult
```

**权限：** 仅店主

### 3. mint_tokens

店主铸造代币（用于活动奖励等）。

```rust
fn mint_tokens(
    origin: OriginFor<T>,
    shop_id: u64,
    to: T::AccountId,
    amount: T::AssetBalance,
) -> DispatchResult
```

**权限：** 仅店主

### 4. transfer_tokens

用户转让积分。

```rust
fn transfer_tokens(
    origin: OriginFor<T>,
    shop_id: u64,
    to: T::AccountId,
    amount: T::AssetBalance,
) -> DispatchResult
```

**权限：** 任何持有积分的用户

**前提条件：** `transferable = true`

## 📡 Events

| 事件 | 说明 |
|------|------|
| `ShopTokenCreated` | 店铺代币已创建 |
| `TokenConfigUpdated` | 代币配置已更新 |
| `RewardIssued` | 购物奖励已发放 |
| `TokensRedeemed` | 积分已兑换 |
| `TokensTransferred` | 积分已转让 |
| `TokensMinted` | 代币已铸造 |
| `TokensBurned` | 代币已销毁 |

## ❌ Errors

| 错误 | 说明 |
|------|------|
| `ShopNotFound` | 店铺不存在 |
| `NotShopOwner` | 不是店主 |
| `TokenNotEnabled` | 店铺代币未启用 |
| `TokenAlreadyExists` | 代币已存在 |
| `InsufficientBalance` | 余额不足 |
| `BelowMinRedeem` | 低于最低兑换门槛 |
| `ExceedsMaxRedeem` | 超过单笔最大兑换 |
| `TransferNotAllowed` | 不允许转让 |
| `InvalidRewardRate` | 无效的奖励率 |
| `InvalidExchangeRate` | 无效的兑换率 |

## 🔌 公共接口

### ShopTokenProvider Trait

本模块实现了 `ShopTokenProvider` trait，供其他模块调用：

```rust
pub trait ShopTokenProvider<AccountId, Balance> {
    /// 检查店铺代币是否启用
    fn is_token_enabled(shop_id: u64) -> bool;
    
    /// 获取用户在店铺的代币余额
    fn token_balance(shop_id: u64, holder: &AccountId) -> Balance;
    
    /// 购物奖励（由 order 模块调用）
    fn reward_on_purchase(
        shop_id: u64,
        buyer: &AccountId,
        purchase_amount: Balance,
    ) -> Result<Balance, DispatchError>;
    
    /// 积分兑换折扣（由 order 模块调用）
    fn redeem_for_discount(
        shop_id: u64,
        buyer: &AccountId,
        tokens: Balance,
    ) -> Result<Balance, DispatchError>;
}
```

### 查询函数

```rust
// 获取用户在店铺的代币余额
Pallet::<T>::get_balance(shop_id, &account) -> Balance

// 获取店铺代币总供应量
Pallet::<T>::get_total_supply(shop_id) -> Balance

// 检查店铺代币是否启用
Pallet::<T>::is_token_enabled(shop_id) -> bool

// 获取店铺代币配置
Pallet::<T>::shop_token_configs(shop_id) -> Option<ShopTokenConfig>
```

## 💡 使用示例

### 场景 1：店铺创建代币

```
1. 店主调用 create_shop_token
   - name: "咖啡积分"
   - symbol: "CAFE"
   - reward_rate: 500 (5%)
   - exchange_rate: 1000 (10%)

2. 系统通过 pallet-assets 创建资产
   - asset_id = ShopTokenOffset + shop_id
```

### 场景 2：购物返积分

```
1. 用户下单购买 100 元商品
2. 订单完成时，order 模块调用 reward_on_purchase
3. 计算奖励：100 * 5% = 5 积分
4. 铸造 5 积分给买家
```

### 场景 3：积分抵扣

```
1. 用户下单时选择使用 100 积分
2. order 模块调用 redeem_for_discount
3. 计算折扣：100 * 10% = 10 元
4. 销毁 100 积分
5. 订单金额减少 10 元
```

## 🔐 安全考虑

### 资产 ID 隔离

```rust
// 店铺代币 ID = 偏移量 + 店铺 ID
// 避免与其他资产（如原生代币）冲突
pub fn shop_to_asset_id(shop_id: u64) -> T::AssetId {
    (T::ShopTokenOffset::get() + shop_id).into()
}
```

### 权限控制

| 操作 | 权限 |
|------|------|
| 创建代币 | 店主 |
| 更新配置 | 店主 |
| 铸造代币 | 店主 |
| 转让积分 | 持有者（需 transferable=true）|
| 兑换积分 | 持有者 |

### 参数验证

- `reward_rate` 和 `exchange_rate` 不能超过 10000（100%）
- 兑换时检查最低门槛和最大限额
- 转让前检查 `transferable` 配置

## 📈 与其他模块的集成

```
┌─────────────────┐     ┌─────────────────┐
│  pallet-order   │────▶│  pallet-token   │
│  (订单模块)      │     │  (代币模块)      │
└─────────────────┘     └─────────────────┘
        │                       │
        │ 下单时                 │ fungibles traits
        │ • reward_on_purchase  │
        │ • redeem_for_discount │
        ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│  pallet-shop    │     │  pallet-assets  │
│  (店铺模块)      │     │  (资产模块)      │
└─────────────────┘     └─────────────────┘
```

## 🧪 测试

```bash
# 运行单元测试
cargo test -p pallet-sharemall-token

# 运行特定测试
cargo test -p pallet-sharemall-token test_create_shop_token
```

## 📝 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1.0 | 2026-01-31 | 初始版本 |

## 📄 许可证

MIT License

## 🔗 相关链接

- [ShareMall 设计文档](../../docs/design/sharemall-token-governance.md)
- [pallet-assets 文档](https://docs.substrate.io/reference/frame-pallets/#assets)
- [Substrate 文档](https://docs.substrate.io/)
