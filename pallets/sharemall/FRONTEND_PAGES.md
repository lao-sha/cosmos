# COSMOS ShareMall 前端页面设计文档

## 系统架构概览

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         COSMOS ShareMall 多店铺商城系统                          │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐           │
│   │   Shop      │  │   Product   │  │   Order     │  │   Token     │           │
│   │   店铺管理   │  │   商品管理   │  │   订单管理   │  │   代币管理   │           │
│   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘           │
│          │                │                │                │                   │
│   ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐           │
│   │   Member    │  │   Review    │  │ Commission  │  │   Market    │           │
│   │   会员管理   │  │   评价管理   │  │   返佣管理   │  │   P2P交易   │           │
│   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘           │
│          │                │                │                │                   │
│          └────────────────┴────────────────┴────────────────┘                   │
│                                    │                                            │
│                           ┌────────┴────────┐                                   │
│                           │   Governance    │                                   │
│                           │   店铺代币治理   │                                   │
│                           └─────────────────┘                                   │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 模块说明

| 模块 | 功能 | Pallet |
|------|------|--------|
| Shop | 店铺创建、运营资金、状态管理 | `pallet-sharemall-shop` |
| Product | 商品发布、上下架、库存管理 | `pallet-sharemall-product` |
| Order | 下单、支付、发货、收货、退款 | `pallet-sharemall-order` |
| Token | 店铺积分、返积分、积分抵扣 | `pallet-sharemall-token` |
| Member | 会员等级、推荐关系 | `pallet-sharemall-member` |
| Commission | 多模式返佣 | `pallet-sharemall-commission` |
| Review | 订单评价 | `pallet-sharemall-review` |
| Market | P2P 代币交易、TWAP 预言机 | `pallet-sharemall-market` |
| Governance | 代币治理投票 | `pallet-sharemall-governance` |

---

## 一、公共页面

### 1.1 首页 `/`
- 热门店铺推荐
- 热门商品推荐
- 搜索入口

### 1.2 店铺列表 `/shops`
- 浏览所有店铺
- 按评分/销量/时间排序

### 1.3 店铺详情 `/shop/:shopId`
- 店铺信息、商品列表
- 店铺代币、会员入口

### 1.4 商品详情 `/product/:productId`
- 商品信息、评价
- 加入购物车、立即购买

### 1.5 购物车 `/cart`
- 管理购物车商品

### 1.6 结算 `/checkout`
- 确认订单、选择支付方式

---

## 二、买家中心

### 2.1 我的订单 `/buyer/orders`
- 订单列表、状态筛选
- 取消、确认收货、申请退款、评价

### 2.2 订单详情 `/buyer/order/:orderId`
- 订单详情、物流信息

### 2.3 我的积分 `/buyer/points`
- 各店铺积分余额、转让

### 2.4 我的会员 `/buyer/membership`
- 会员等级、推荐关系、返佣记录

---

## 三、卖家中心

### 3.1 店铺首页 `/seller`
- 数据概览、运营资金、待处理事项

### 3.2 创建店铺 `/seller/create`
- 填写信息、支付运营资金

### 3.3 店铺设置 `/seller/settings`
- 修改信息、资金管理、状态管理

### 3.4 商品管理 `/seller/products`
- 商品列表、上下架

### 3.5 发布商品 `/seller/product/create`
- 填写商品信息、发布

### 3.6 订单管理 `/seller/orders`
- 店铺订单、发货、处理退款

### 3.7 代币管理 `/seller/token`
- 启用代币、配置返积分

### 3.8 会员管理 `/seller/members`
- 会员列表、等级配置

### 3.9 返佣配置 `/seller/commission`
- 选择返佣模式、配置比例

---

## 四、代币交易市场

### 4.1 市场首页 `/market`
- 代币列表、市场概览

### 4.2 代币交易 `/market/:shopId`
- 订单簿、挂单/吃单、K线图
- 限价单、市价单

### 4.3 USDT 交易 `/market/:shopId/usdt`
- USDT 通道交易

---

## 五、治理页面

### 5.1 店铺治理 `/governance/:shopId`
- 提案列表、创建提案、投票

---

## 链上查询接口

| 接口 | 说明 |
|------|------|
| `shop.shops(id)` | 店铺详情 |
| `shop.shopFundBalance(id)` | 运营资金余额 |
| `product.products(id)` | 商品详情 |
| `product.shopProducts(shopId)` | 店铺商品列表 |
| `order.orders(id)` | 订单详情 |
| `order.buyerOrders(account)` | 买家订单 |
| `order.shopOrders(shopId)` | 店铺订单 |
| `token.tokenBalance(shopId, account)` | 积分余额 |
| `member.memberInfo(shopId, account)` | 会员信息 |
| `market.getOrderBookDepth(shopId, depth)` | 订单簿深度 |
| `market.getBestPrices(shopId)` | 最优买卖价 |
| `market.calculateTwap(shopId, period)` | TWAP 价格 |

---

## 链上交易接口

| 接口 | 说明 |
|------|------|
| `shop.createShop()` | 创建店铺 |
| `shop.updateShop()` | 更新店铺 |
| `shop.depositFund()` | 充值运营资金 |
| `product.createProduct()` | 创建商品 |
| `product.publishProduct()` | 上架商品 |
| `order.placeOrder()` | 下单 |
| `order.cancelOrder()` | 取消订单 |
| `order.shipOrder()` | 发货 |
| `order.confirmReceipt()` | 确认收货 |
| `order.requestRefund()` | 申请退款 |
| `review.submitReview()` | 提交评价 |
| `token.transfer()` | 转让积分 |
| `market.placeSellOrder()` | 挂卖单 |
| `market.placeBuyOrder()` | 挂买单 |
| `market.takeOrder()` | 吃单 |
| `market.marketBuy()` | 市价买入 |
| `market.marketSell()` | 市价卖出 |
| `market.setInitialPrice()` | 设置初始价格 |
| `governance.createProposal()` | 创建提案 |
| `governance.vote()` | 投票 |
