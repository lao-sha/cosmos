# pallet-entity-review

> ⭐ Entity 订单评价模块 — 订单完成后提交评分与评价

## 概述

`pallet-entity-review` 是 Entity 商城系统的评价管理模块，负责订单完成后的评分提交和店铺评分更新。

### 核心功能

- **订单评价** — 买家在订单完成后提交 1-5 星评分
- **评价内容** — 支持通过 IPFS CID 关联评价详情
- **店铺评分更新** — 评价提交后自动调用 `ShopProvider::update_shop_rating`
- **一单一评** — 每个订单仅允许一次评价

## 数据结构

### MallReview — 评价记录

```rust
pub struct MallReview<AccountId, BlockNumber, MaxCidLen> {
    pub order_id: u64,                              // 订单 ID
    pub reviewer: AccountId,                        // 评价者
    pub rating: u8,                                 // 评分（1-5）
    pub content_cid: Option<BoundedVec<u8, MaxCidLen>>, // 评价内容 IPFS CID
    pub created_at: BlockNumber,                    // 评价时间
}
```

## Runtime 配置

```rust
impl pallet_entity_review::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OrderProvider = EntityTransaction;   // 订单查询接口
    type ShopProvider = EntityShop;           // 店铺更新接口
    type MaxCidLength = ConstU32<64>;         // CID 最大长度
}
```

## Extrinsics

### submit_review (call_index 0)

提交订单评价。

```rust
fn submit_review(
    origin: OriginFor<T>,
    order_id: u64,
    rating: u8,                    // 1-5 星
    content_cid: Option<Vec<u8>>,  // IPFS CID（可选）
) -> DispatchResult
```

**权限：** 仅订单买家

**前提条件：**
1. 订单存在且已完成（`OrderProvider::is_order_completed`）
2. 调用者是订单买家（`OrderProvider::order_buyer`）
3. 该订单尚未评价

**流程：**
1. 验证评分范围 1-5
2. 验证订单存在、已完成、调用者是买家
3. 检查未重复评价
4. 存储评价记录
5. 递增全局评价计数
6. 通过 `ShopProvider::update_shop_rating` 更新店铺评分

## Storage

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `Reviews` | `StorageMap<u64, MallReview>` | 订单 ID → 评价记录 |
| `ReviewCount` | `StorageValue<u64>` | 全局评价总数 |

## Events

| 事件 | 说明 | 字段 |
|------|------|------|
| `ReviewSubmitted` | 评价已提交 | `order_id`, `reviewer`, `rating` |

## Errors

| 错误 | 说明 |
|------|------|
| `OrderNotFound` | 订单不存在 |
| `NotOrderBuyer` | 不是订单买家 |
| `OrderNotCompleted` | 订单未完成 |
| `AlreadyReviewed` | 已评价过 |
| `InvalidRating` | 无效评分（不在 1-5 范围） |
| `CidTooLong` | CID 超过最大长度 |

## 依赖接口

| Trait | 提供者 | 用途 |
|-------|--------|------|
| `OrderProvider` | `pallet-entity-transaction` | 验证订单状态和买家身份 |
| `ShopProvider` | `pallet-entity-shop` | 更新店铺评分 |

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1.0 | 2026-01-31 | 从 pallet-mall 拆分 |
