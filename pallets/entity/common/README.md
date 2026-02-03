# pallet-entity-common

实体模块公共组件库。

## 概述

本模块提供 Entity 系列模块的公共类型、Trait 和工具函数，是其他 Entity 子模块的基础依赖。

## 核心类型

### 实体类型 (EntityType)

```rust
pub enum EntityType {
    Shop,           // 店铺（零售/服务）
    Restaurant,     // 餐饮
    Investment,     // 投资基金
    DAO,            // 去中心化自治组织
    Cooperative,    // 合作社
    Project,        // 项目/众筹
    Club,           // 俱乐部/会员制
    Custom(u8),     // 自定义类型
}
```

### 通证类型 (TokenType)

```rust
pub enum TokenType {
    Utility,        // 实用型（消费积分）
    Equity,         // 权益型（分红权）
    Governance,     // 治理型（投票权）
    Membership,     // 会员型（身份凭证）
    Hybrid,         // 混合型
}
```

### 治理模式 (GovernanceMode)

```rust
pub enum GovernanceMode {
    None,           // 无治理（店主独断）
    Advisory,       // 咨询式（投票仅供参考）
    DualTrack,      // 双轨制（部分事项需投票）
    Committee,      // 委员会制
    FullDAO,        // 完全 DAO
    Tiered,         // 分层治理
}
```

## Trait 定义

### ShopProvider

```rust
pub trait ShopProvider<AccountId> {
    fn shop_exists(shop_id: u64) -> bool;
    fn shop_owner(shop_id: u64) -> Option<AccountId>;
    fn is_shop_active(shop_id: u64) -> bool;
}
```

### MemberLevel

```rust
pub enum MemberLevel {
    Normal,
    Silver,
    Gold,
    Platinum,
    Diamond,
}
```

## 使用方式

在其他 Entity 模块中引用：

```toml
[dependencies]
pallet-entity-common = { workspace = true }
```

```rust
use pallet_entity_common::{EntityType, TokenType, GovernanceMode, ShopProvider};
```

## 版本

- v0.1.0: 初始版本
- v0.2.0: Phase 2-4 扩展类型定义
