# pallet-sharemall-product

> 📦 ShareMall 商品管理模块 - 商品生命周期管理与押金机制

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Substrate](https://img.shields.io/badge/Substrate-polkadot--sdk-blue)](https://github.com/paritytech/polkadot-sdk)

## 📖 概述

`pallet-sharemall-product` 是 ShareMall 商城系统的商品管理模块，负责商品的完整生命周期管理，包括创建、更新、上架、下架和库存管理。

### 核心功能

- 📝 **商品创建** - 从店铺派生账户扣取 1 USDT 等值 COS 押金
- ✏️ **商品更新** - 修改商品信息
- 🚀 **商品上架** - 发布商品到店铺
- 📥 **商品下架** - 从店铺移除商品
- �️ **商品删除** - 退还押金到店铺派生账户
- � **库存管理** - 库存扣减与恢复
- 📈 **销量统计** - 自动记录销售数量

## 💰 押金机制

### 核心设计

创建商品时从**店铺派生账户**扣取 **1 USDT 等值 COS** 作为押金，转入 **Pallet 账户**托管。

```
┌─────────────────────────────────────────────────────────────────┐
│                    商品押金流程                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  创建商品:                                                       │
│  店铺派生账户 ──→ Product Pallet 账户                            │
│                   PalletId(*b"sm/prod/")                        │
│                                                                 │
│  删除商品:                                                       │
│  Product Pallet 账户 ──→ 店铺派生账户                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 押金计算

```
COS 押金 = USDT 金额 × 10^12 / COS价格
```

| COS/USDT 价格 | 1 USDT 等值 COS |
|---------------|-----------------|
| 0.001 | 1,000 COS |
| 0.01 | 100 COS |
| 0.1 | 10 COS |

## 🏗️ 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                   pallet-sharemall-product                      │
│                      (商品管理模块)                              │
├─────────────────────────────────────────────────────────────────┤
│  • 商品 CRUD 操作                                                │
│  • 商品押金管理（从店铺派生账户扣取）                             │
│  • 商品状态管理                                                  │
│  • 库存管理                                                      │
│  • 销量统计                                                      │
└─────────────────────────────────────────────────────────────────┘
         │                              │
         │ ShopProvider                 │ ProductProvider
         ▼                              ▼
┌─────────────────────┐    ┌─────────────────────────────────────┐
│  pallet-sharemall   │    │        pallet-sharemall-order       │
│      -shop          │    │           (订单模块)                 │
│    (店铺模块)        │    │  • 下单时扣减库存                    │
│  • 店铺存在性验证    │    │  • 取消时恢复库存                    │
│  • 店主身份验证      │    │  • 完成时增加销量                    │
│  • 派生账户提供      │    │                                     │
└─────────────────────┘    └─────────────────────────────────────┘
```

## 📦 安装

### Cargo.toml

```toml
[dependencies]
pallet-sharemall-product = { path = "pallets/sharemall/product", default-features = false }

[features]
std = [
    "pallet-sharemall-product/std",
]
```

## ⚙️ Runtime 配置

```rust
parameter_types! {
    /// 商品押金：1 USDT（精度 10^6）
    pub const ProductDepositUsdt: u64 = 1_000_000;
    /// 最小押金：1 COS
    pub const MinProductDepositCos: Balance = 1 * UNIT;
    /// 最大押金：100 COS
    pub const MaxProductDepositCos: Balance = 100 * UNIT;
}

impl pallet_sharemall_product::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ShopProvider = ShareMallShop;
    type PricingProvider = TradingPricing;
    type MaxProductsPerShop = ConstU32<1000>;
    type MaxCidLength = ConstU32<64>;
    type ProductDepositUsdt = ProductDepositUsdt;
    type MinProductDepositCos = MinProductDepositCos;
    type MaxProductDepositCos = MaxProductDepositCos;
}
```

### 配置参数说明

| 参数 | 类型 | 说明 | 示例值 |
|------|------|------|--------|
| `Currency` | Currency | 货币类型 | `Balances` |
| `ShopProvider` | ShopProvider | 店铺查询接口 | `ShareMallShop` |
| `PricingProvider` | PricingProvider | **定价提供者** | `TradingPricing` |
| `MaxProductsPerShop` | u32 | 每店铺最大商品数 | 1000 |
| `MaxCidLength` | u32 | CID 最大长度 | 64 |
| `ProductDepositUsdt` | u64 | **押金 USDT（精度 10^6）** | 1_000_000 |
| `MinProductDepositCos` | Balance | **最小押金 COS** | 1 UNIT |
| `MaxProductDepositCos` | Balance | **最大押金 COS** | 100 UNIT |

## 📊 数据结构

### Product - 商品信息

```rust
pub struct Product<Balance, BlockNumber, MaxCidLen> {
    pub id: u64,                              // 商品 ID
    pub shop_id: u64,                         // 所属店铺 ID
    pub name_cid: BoundedVec<u8, MaxCidLen>,  // 商品名称 IPFS CID
    pub images_cid: BoundedVec<u8, MaxCidLen>,// 商品图片 IPFS CID
    pub detail_cid: BoundedVec<u8, MaxCidLen>,// 商品详情 IPFS CID
    pub price: Balance,                       // 单价
    pub stock: u32,                           // 库存数量（0 = 无限）
    pub sold_count: u32,                      // 已售数量
    pub status: ProductStatus,                // 商品状态
    pub category: ProductCategory,            // 商品类别
    pub created_at: BlockNumber,              // 创建时间
    pub updated_at: BlockNumber,              // 更新时间
}
```

### ProductStatus - 商品状态

```rust
pub enum ProductStatus {
    Draft,      // 草稿（未上架）
    OnSale,     // 在售
    OffShelf,   // 已下架
    SoldOut,    // 售罄
}
```

### ProductCategory - 商品类别

```rust
pub enum ProductCategory {
    Digital,    // 数字商品（虚拟物品）
    Physical,   // 实物商品
    Service,    // 服务类
    Other,      // 其他
}
```

### ProductStatistics - 商品统计

```rust
pub struct ProductStatistics {
    pub total_products: u64,     // 总商品数
    pub on_sale_products: u64,   // 在售商品数
}
```

## 🔧 Extrinsics

### 1. create_product

创建商品（从店铺派生账户扣取押金）。

```rust
fn create_product(
    origin: OriginFor<T>,
    shop_id: u64,
    name_cid: Vec<u8>,
    images_cid: Vec<u8>,
    detail_cid: Vec<u8>,
    price: BalanceOf<T>,
    stock: u32,
    category: ProductCategory,
) -> DispatchResult
```

**参数：**
- `shop_id` - 店铺 ID
- `name_cid` - 商品名称 IPFS CID
- `images_cid` - 商品图片 IPFS CID
- `detail_cid` - 商品详情 IPFS CID
- `price` - 商品单价
- `stock` - 库存数量（0 = 无限库存）
- `category` - 商品类别

**权限：** 仅店主

**押金：** 从店铺派生账户扣取 1 USDT 等值 COS

**示例：**
```javascript
api.tx.shareMallProduct.createProduct(
    1,                    // shop_id
    "QmName...",         // name_cid
    "QmImages...",       // images_cid
    "QmDetail...",       // detail_cid
    1000000000000,       // price: 1 UNIT
    100,                 // stock
    { Physical: null }   // category
)
```

### 2. update_product

更新商品信息。

```rust
fn update_product(
    origin: OriginFor<T>,
    product_id: u64,
    name_cid: Option<Vec<u8>>,
    images_cid: Option<Vec<u8>>,
    detail_cid: Option<Vec<u8>>,
    price: Option<BalanceOf<T>>,
    stock: Option<u32>,
    category: Option<ProductCategory>,
) -> DispatchResult
```

**权限：** 仅店主

**说明：** 所有参数均为可选，仅更新提供的字段

### 3. publish_product

上架商品（草稿 → 在售）。

```rust
fn publish_product(
    origin: OriginFor<T>,
    product_id: u64,
) -> DispatchResult
```

**权限：** 仅店主

**前提条件：** 店铺必须处于激活状态

### 4. unpublish_product

下架商品（在售 → 已下架）。

```rust
fn unpublish_product(
    origin: OriginFor<T>,
    product_id: u64,
) -> DispatchResult
```

**权限：** 仅店主

### 5. delete_product

删除商品（退还押金）。

```rust
fn delete_product(
    origin: OriginFor<T>,
    product_id: u64,
) -> DispatchResult
```

**权限：** 仅店主

**前提条件：** 商品状态必须为 `Draft` 或 `OffShelf`

**说明：** 删除商品后，创建时支付的押金将自动退还给店主

## 📡 Events

| 事件 | 说明 | 字段 |
|------|------|------|
| `ProductCreated` | 商品已创建 | `product_id`, `shop_id` |
| `ProductUpdated` | 商品已更新 | `product_id` |
| `ProductStatusChanged` | 商品状态已变更 | `product_id`, `status` |
| `ProductDeleted` | 商品已删除 | `product_id` |
| `StockUpdated` | 库存已更新 | `product_id`, `new_stock` |
| `DepositReserved` | **押金已收取** | `product_id`, `depositor`, `amount` |
| `DepositUnreserved` | **押金已退还** | `product_id`, `depositor`, `amount` |

## ❌ Errors

| 错误 | 说明 |
|------|------|
| `ProductNotFound` | 商品不存在 |
| `ShopNotFound` | 店铺不存在 |
| `NotShopOwner` | 不是店主 |
| `ShopNotActive` | 店铺未激活 |
| `InsufficientStock` | 库存不足 |
| `MaxProductsReached` | 达到最大商品数 |
| `InvalidProductStatus` | 无效的商品状态 |
| `CidTooLong` | CID 过长 |
| `InsufficientBalanceForDeposit` | **余额不足以支付押金** |

## 🔌 ProductProvider Trait

本模块实现了 `ProductProvider` trait，供其他模块（如订单模块）调用：

```rust
pub trait ProductProvider<AccountId, Balance> {
    /// 商品是否存在
    fn product_exists(product_id: u64) -> bool;
    
    /// 商品是否在售
    fn is_product_on_sale(product_id: u64) -> bool;
    
    /// 获取商品所属店铺 ID
    fn product_shop_id(product_id: u64) -> Option<u64>;
    
    /// 获取商品价格
    fn product_price(product_id: u64) -> Option<Balance>;
    
    /// 获取商品库存
    fn product_stock(product_id: u64) -> Option<u32>;
    
    /// 扣减库存（下单时调用）
    fn deduct_stock(product_id: u64, quantity: u32) -> Result<(), DispatchError>;
    
    /// 恢复库存（取消订单时调用）
    fn restore_stock(product_id: u64, quantity: u32) -> Result<(), DispatchError>;
    
    /// 增加销量（订单完成时调用）
    fn add_sold_count(product_id: u64, quantity: u32) -> Result<(), DispatchError>;
}
```

## 💡 商品生命周期

```
┌─────────────────────────────────────────────────────────────────┐
│                        商品生命周期                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. 创建商品                                                     │
│     └── create_product() → 状态: Draft                          │
│                                                                 │
│  2. 上架商品                                                     │
│     └── publish_product() → 状态: OnSale                        │
│                                                                 │
│  3. 销售中                                                       │
│     ├── 用户下单 → deduct_stock()                               │
│     ├── 订单取消 → restore_stock()                              │
│     └── 订单完成 → add_sold_count()                             │
│                                                                 │
│  4. 库存售罄                                                     │
│     └── stock = 0 → 状态: SoldOut                               │
│                                                                 │
│  5. 下架商品                                                     │
│     └── unpublish_product() → 状态: OffShelf                    │
│                                                                 │
│  6. 重新上架                                                     │
│     └── publish_product() → 状态: OnSale                        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 📈 库存管理

### 库存扣减逻辑

```rust
fn deduct_stock(product_id: u64, quantity: u32) -> Result<(), DispatchError> {
    // 1. 检查库存是否足够
    // 2. 扣减库存
    // 3. 如果库存归零，状态变为 SoldOut
}
```

### 库存恢复逻辑

```rust
fn restore_stock(product_id: u64, quantity: u32) -> Result<(), DispatchError> {
    // 1. 增加库存
    // 2. 如果之前是 SoldOut，状态恢复为 OnSale
}
```

### 无限库存

当 `stock = 0` 时，表示无限库存：
- 不会扣减库存
- 不会变为 SoldOut 状态

## 🔐 安全考虑

### 权限控制

| 操作 | 权限 |
|------|------|
| 创建商品 | 店主 |
| 更新商品 | 店主 |
| 上架商品 | 店主（店铺需激活） |
| 下架商品 | 店主 |
| 扣减库存 | 系统（订单模块） |
| 恢复库存 | 系统（订单模块） |

### 商品数量限制

```rust
ensure!(
    product_ids.len() < T::MaxProductsPerShop::get() as usize,
    Error::<T>::MaxProductsReached
);
```

**目的**：防止单店铺创建过多商品

### 💰 存储押金机制

为防止存储膨胀，创建商品时需要支付押金：

```rust
// 创建商品时收取押金
let deposit = T::ProductDeposit::get();
T::Currency::reserve(&who, deposit)?;

// 删除商品时退还押金
T::Currency::unreserve(&depositor, deposit);
```

**机制说明：**

| 操作 | 押金行为 |
|------|----------|
| `create_product` | 收取押金（锁定） |
| `delete_product` | 退还押金（解锁） |

**优点：**
- 经济激励用户清理无用商品
- 防止恶意创建大量商品占用存储
- 押金可配置，灵活调整

**存储成本估算：**
- 单个商品约 242 bytes
- 1000 个商品约 242 KB
- 押金应覆盖存储成本

## 🧪 测试

```bash
# 运行单元测试
cargo test -p pallet-sharemall-product

# 运行特定测试
cargo test -p pallet-sharemall-product test_create_product
```

## 📝 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1.0 | 2026-01-31 | 从 pallet-mall 拆分 |

## 📄 许可证

MIT License

## 🔗 相关链接

- [pallet-sharemall-shop](../shop/README.md)
- [pallet-sharemall-order](../order/README.md)
- [pallet-sharemall-common](../common/README.md)
