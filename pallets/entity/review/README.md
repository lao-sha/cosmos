# pallet-entity-review

实体评价管理模块。

## 概述

本模块实现实体服务/商品评价功能，支持评分、评论和评价激励。

## 功能特性

- **多维度评分**: 服务、质量、物流等多维度评分
- **评价验证**: 只有完成交易的用户才能评价
- **评价激励**: 评价可获得代币奖励
- **追评机制**: 支持追加评价
- **商家回复**: 商家可回复评价

## 数据结构

### 评价记录

```rust
pub struct Review {
    pub id: u64,
    pub entity_id: u64,
    pub order_id: u64,
    pub reviewer: AccountId,
    pub rating: u8,              // 1-5 星
    pub content_cid: BoundedVec<u8>,
    pub images_cid: Option<BoundedVec<u8>>,
    pub created_at: BlockNumber,
    pub reply_cid: Option<BoundedVec<u8>>,
    pub helpful_count: u32,
    pub reported: bool,
}
```

## Extrinsics

| 函数 | 说明 |
|------|------|
| `create_review` | 创建评价 |
| `append_review` | 追加评价 |
| `reply_review` | 商家回复 |
| `mark_helpful` | 标记有用 |
| `report_review` | 举报评价 |

## 存储项

| 名称 | 说明 |
|------|------|
| `Reviews` | 评价记录 |
| `EntityReviews` | 实体评价索引 |
| `OrderReview` | 订单评价映射 |
| `EntityStats` | 实体评价统计 |

## 评价统计

```rust
pub struct ReviewStats {
    pub total_reviews: u32,
    pub average_rating: u16,     // 基点，如 450 = 4.5 星
    pub rating_distribution: [u32; 5],
}
```

## 版本

- v0.1.0: 初始版本
