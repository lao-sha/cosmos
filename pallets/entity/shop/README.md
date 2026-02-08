# Shop 模块 (pallet-entity-shop)

> 🏪 NEXUS 业务层店铺管理模块

## 概述

Shop 模块是 Entity-Shop 分离架构的业务层，负责管理具体的经营场所或线上店铺。每个 Shop 归属于一个 Entity（组织），一个 Entity 可以拥有多个 Shop。

## 核心概念

### Entity 与 Shop 的关系

```
Entity (组织层)              Shop (业务层)
─────────────────            ─────────────────
• 所有权/治理               • 日常运营
• 代币发行/分红             • 商品管理
• KYC/合规                  • 订单处理
• 组织金库                  • 运营资金
• 管理员权限                • 门店管理员
```

### Shop 类型 (ShopType)

| 类型 | 说明 | 需要位置 |
|------|------|---------|
| `OnlineStore` | 线上商城（默认） | ❌ |
| `PhysicalStore` | 实体门店 | ✅ |
| `ServicePoint` | 服务网点 | ✅ |
| `Warehouse` | 仓储/自提点 | ✅ |
| `Franchise` | 加盟店 | ❌ |
| `Popup` | 快闪店/临时店 | ✅ |
| `Virtual` | 虚拟店铺（纯服务） | ❌ |

### 会员模式 (MemberMode)

| 模式 | 说明 |
|------|------|
| `Inherit` | 继承 Entity 会员体系，所有 Shop 共享会员 |
| `Independent` | 独立会员体系，各 Shop 独立管理会员 |
| `Hybrid` | 混合模式，Entity + Shop 双层会员 |

## 数据结构

### Shop

```rust
pub struct Shop<...> {
    pub id: u64,                    // Shop ID
    pub entity_id: u64,             // 所属 Entity
    pub name: BoundedVec<u8>,       // 名称
    pub shop_type: ShopType,        // 类型
    pub status: ShopOperatingStatus,// 状态
    pub is_primary: bool,           // 是否主 Shop
    pub managers: BoundedVec<AccountId>, // 管理员
    pub member_mode: MemberMode,    // 会员模式
    pub initial_fund: Balance,      // 初始运营资金
    pub location: Option<(i64, i64)>, // 地理位置
    // ... 统计字段
}
```

### PointsConfig (Shop 积分)

```rust
pub struct PointsConfig<...> {
    pub name: BoundedVec<u8>,       // 积分名称
    pub symbol: BoundedVec<u8>,     // 积分符号
    pub reward_rate: u16,           // 购物返积分比例（基点）
    pub exchange_rate: u16,         // 积分兑换比例（基点）
    pub transferable: bool,         // 是否可转让
    pub enabled: bool,              // 是否启用
}
```

## Extrinsics

| 调用 | 说明 | 权限 |
|------|------|------|
| `create_shop` | 创建 Shop | Entity Owner |
| `update_shop` | 更新 Shop 信息 | Shop Manager |
| `add_manager` | 添加管理员 | Entity Owner |
| `remove_manager` | 移除管理员 | Entity Owner |
| `fund_operating` | 充值运营资金 | Shop Manager |
| `pause_shop` | 暂停营业 | Shop Manager |
| `resume_shop` | 恢复营业 | Shop Manager |
| `set_location` | 设置位置信息 | Shop Manager |
| `enable_points` | 启用 Shop 积分 | Shop Manager |
| `close_shop` | 关闭 Shop | Entity Owner |

## Storage

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `Shops` | `Map<u64, Shop>` | Shop 数据 |
| `ShopEntity` | `Map<u64, u64>` | Shop -> Entity 索引 |
| `NextShopId` | `u64` | 下一个 Shop ID |
| `ShopPointsConfigs` | `Map<u64, PointsConfig>` | Shop 积分配置 |
| `ShopPointsBalances` | `DoubleMap<u64, AccountId, Balance>` | Shop 积分余额 |
| `ShopPointsTotalSupply` | `Map<u64, Balance>` | Shop 积分总供应量 |

## 配置参数

```rust
type MaxShopNameLength = ConstU32<64>;      // 名称最大长度
type MaxCidLength = ConstU32<64>;           // CID 最大长度
type MaxManagers = ConstU32<10>;            // 最大管理员数
type MaxPointsNameLength = ConstU32<32>;    // 积分名称最大长度
type MaxPointsSymbolLength = ConstU32<8>;   // 积分符号最大长度
type MinOperatingBalance = ...;             // 最低运营余额
type WarningThreshold = ...;                // 资金预警阈值
```

## ShopProvider Trait

```rust
pub trait ShopProvider<AccountId> {
    fn shop_exists(shop_id: u64) -> bool;
    fn is_shop_active(shop_id: u64) -> bool;
    fn shop_entity_id(shop_id: u64) -> Option<u64>;
    fn shop_account(shop_id: u64) -> AccountId;
    fn shop_type(shop_id: u64) -> Option<ShopType>;
    fn shop_member_mode(shop_id: u64) -> MemberMode;
    fn is_shop_manager(shop_id: u64, account: &AccountId) -> bool;
    fn is_primary_shop(shop_id: u64) -> bool;
    fn update_shop_stats(shop_id: u64, sales: u128, orders: u32) -> DispatchResult;
    fn update_shop_rating(shop_id: u64, rating: u8) -> DispatchResult;
    fn deduct_operating_fund(shop_id: u64, amount: u128) -> DispatchResult;
    fn operating_balance(shop_id: u64) -> u128;
    fn pause_shop(shop_id: u64) -> DispatchResult;
    fn resume_shop(shop_id: u64) -> DispatchResult;
}
```

## 使用示例

### 创建 Shop

```rust
// Entity owner 创建一个线下门店
EntityShop::create_shop(
    origin,
    entity_id,          // 所属 Entity
    b"北京旗舰店".to_vec().try_into().unwrap(),
    ShopType::PhysicalStore,
    MemberMode::Inherit, // 继承 Entity 会员
    100_000_000_000,     // 100 NXS 运营资金
)?;
```

### 设置位置信息

```rust
// 设置门店地理位置（精度 10^6）
EntityShop::set_location(
    origin,
    shop_id,
    Some((116_397_128, 39_907_689)), // 北京天安门
    Some(address_cid),
    Some(business_hours_cid),
)?;
```

### 启用 Shop 积分

```rust
// 启用独立的 Shop 积分系统
EntityShop::enable_points(
    origin,
    shop_id,
    b"门店积分".to_vec().try_into().unwrap(),
    b"SP".to_vec().try_into().unwrap(),
    500,   // 5% 购物返积分
    1000,  // 10% 兑换比例
    false, // 不可转让
)?;
```

## 版本历史

- v0.1.0 (2026-02-05): 初始版本
