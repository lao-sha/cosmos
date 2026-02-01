# pallet-sharemall

COSMOS ShareMall 多店铺商城模块

## 概述

本模块提供去中心化的多店铺商城功能，支持用户开设店铺、上架商品、进行交易。

## 核心功能

### 店铺管理
- `create_shop` - 创建店铺（需缴纳保证金）
- `update_shop` - 更新店铺信息
- `suspend_shop` - 暂停营业
- `resume_shop` - 恢复营业
- `close_shop` - 关闭店铺
- `approve_shop` - 审核通过（治理）
- `ban_shop` - 封禁店铺（治理）

### 商品管理
- `create_product` - 创建商品
- `update_product` - 更新商品信息
- `publish_product` - 上架商品
- `unpublish_product` - 下架商品

### 订单流程
- `place_order` - 下单并支付
- `cancel_order` - 取消订单
- `ship_order` - 发货
- `confirm_receipt` - 确认收货
- `request_refund` - 申请退款
- `approve_refund` - 同意退款

### 评价系统
- `submit_review` - 提交评价

## 订单流程

```
下单 → 支付(托管) → 发货 → 确认收货 → 完成
  │        │          │         │
  │        │          │    超时自动确认(7天)
  │        │          │
  │        │     申请退款 → 同意 → 退款
  │        │
  │   超时退款(72h)
  │
取消订单
```

## 配置参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| MinShopDeposit | 最低店铺保证金 | 100 COS |
| PlatformFeeRate | 平台费率 | 2% (200基点) |
| PaymentTimeout | 支付超时 | 24小时 |
| ShipTimeout | 发货超时 | 72小时 |
| ConfirmTimeout | 确认收货超时 | 7天 |
| MaxProductsPerShop | 每店铺最大商品数 | 100 |

## 依赖模块

- `pallet-escrow` - 资金托管
- `pallet-balances` - 余额管理

## 测试

```bash
cargo test -p pallet-sharemall
```

## License

MIT
