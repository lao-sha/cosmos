# 店铺代币交易市场模块 (pallet-entity-market)

## 概述

本模块实现店铺代币的 P2P 交易市场，支持任意主网地址之间的店铺代币买卖交易。

## 核心功能

### 1. COS 通道 ✅

使用原生 COS 代币进行店铺代币买卖，链上即时结算。

```
┌─────────────────────────────────────────────────────────────────┐
│                    COS 通道交易流程                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   卖家 (Alice)                          买家 (Bob)              │
│       │                                     │                   │
│       │ place_sell_order()                  │                   │
│       │ (锁定 Token)                        │                   │
│       ▼                                     │                   │
│   ┌─────────┐                               │                   │
│   │ 卖单    │                               │                   │
│   │ 挂单中  │                               │                   │
│   └─────────┘                               │                   │
│       │                                     │                   │
│       │ ◄─────────────────────── take_order()                  │
│       │                          (支付 COS)                     │
│       ▼                                     │                   │
│   ┌─────────────────────────────────────────┐                  │
│   │           原子交换                       │                  │
│   │  Token: Alice → Bob                     │                  │
│   │  COS: Bob → Alice                       │                  │
│   │  Fee: → 店铺金库                         │                  │
│   └─────────────────────────────────────────┘                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 2. USDT 通道 ✅

使用 TRC20 USDT 进行店铺代币买卖，需要 OCW 验证链下支付。

```
┌─────────────────────────────────────────────────────────────────┐
│                    USDT 通道交易流程                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   流程 A: 吃 USDT 卖单                                          │
│   ┌─────────────────────────────────────────────────────────┐  │
│   │  1. Alice 挂 USDT 卖单 (锁定 Token, 提供 TRON 地址)      │  │
│   │  2. Bob 链下转 USDT → Alice 的 TRON 地址                 │  │
│   │  3. Bob 调用 take_usdt_sell_order (提交 tx_hash)        │  │
│   │  4. OCW 验证 TRON 交易                                   │  │
│   │  5. 验证通过 → Token 转给 Bob                            │  │
│   │  6. 验证失败 → Token 退还 Alice                          │  │
│   └─────────────────────────────────────────────────────────┘  │
│                                                                 │
│   流程 B: 接受 USDT 买单                                        │
│   ┌─────────────────────────────────────────────────────────┐  │
│   │  1. Bob 挂 USDT 买单                                     │  │
│   │  2. Alice 调用 accept_usdt_buy_order (锁定 Token)       │  │
│   │  3. Bob 链下转 USDT → Alice 的 TRON 地址                 │  │
│   │  4. Bob 调用 confirm_usdt_payment (提交 tx_hash)        │  │
│   │  5. OCW 验证 TRON 交易                                   │  │
│   │  6. 验证通过 → Token 转给 Bob                            │  │
│   │  7. 超时未支付 → Token 退还 Alice                        │  │
│   └─────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 数据结构

### 订单 (TradeOrder)

```rust
pub struct TradeOrder<T: Config> {
    pub order_id: u64,           // 订单 ID
    pub shop_id: u64,            // 店铺 ID
    pub maker: T::AccountId,     // 挂单者
    pub side: OrderSide,         // Buy / Sell
    pub channel: PaymentChannel, // COS / USDT
    pub token_amount: Balance,   // 代币数量
    pub filled_amount: Balance,  // 已成交数量
    pub price: Balance,          // 单价（COS/Token）
    pub status: OrderStatus,     // 订单状态
    pub created_at: BlockNumber, // 创建区块
    pub expires_at: BlockNumber, // 过期区块
}
```

### 订单状态

| 状态 | 说明 |
|------|------|
| `Open` | 挂单中，等待成交 |
| `PartiallyFilled` | 部分成交 |
| `Filled` | 完全成交 |
| `Cancelled` | 已取消 |
| `Expired` | 已过期 |

### 市场配置 (MarketConfig)

```rust
pub struct MarketConfig {
    pub cos_enabled: bool,      // 是否启用 COS 交易
    pub usdt_enabled: bool,     // 是否启用 USDT 交易
    pub fee_rate: u16,          // 手续费率（基点）
    pub min_order_amount: u128, // 最小订单数量
    pub order_ttl: u32,         // 订单有效期（区块数）
}
```

## Extrinsics

### COS 通道

#### 1. `place_sell_order`

挂卖单（卖 Token 得 COS）

```rust
fn place_sell_order(
    origin: OriginFor<T>,
    shop_id: u64,
    token_amount: TokenBalance,
    price: Balance,  // 每个 Token 的 COS 价格
) -> DispatchResult;
```

#### 2. `place_buy_order`

挂买单（用 COS 买 Token）

```rust
fn place_buy_order(
    origin: OriginFor<T>,
    shop_id: u64,
    token_amount: TokenBalance,
    price: Balance,
) -> DispatchResult;
```

#### 3. `take_order`

吃单（成交对手盘订单）

```rust
fn take_order(
    origin: OriginFor<T>,
    order_id: u64,
    amount: Option<TokenBalance>,  // None = 全部吃掉
) -> DispatchResult;
```

#### 4. `cancel_order`

取消订单

```rust
fn cancel_order(
    origin: OriginFor<T>,
    order_id: u64,
) -> DispatchResult;
```

### USDT 通道

#### 5. `place_usdt_sell_order`

挂 USDT 卖单（卖 Token 收 USDT）

```rust
fn place_usdt_sell_order(
    origin: OriginFor<T>,
    shop_id: u64,
    token_amount: TokenBalance,
    usdt_price: u64,           // 每个 Token 的 USDT 价格（精度 10^6）
    tron_address: Vec<u8>,     // 卖家 TRON 收款地址
) -> DispatchResult;
```

#### 6. `place_usdt_buy_order`

挂 USDT 买单（用 USDT 买 Token）

```rust
fn place_usdt_buy_order(
    origin: OriginFor<T>,
    shop_id: u64,
    token_amount: TokenBalance,
    usdt_price: u64,
) -> DispatchResult;
```

#### 7. `take_usdt_sell_order`

吃 USDT 卖单（买家发起，提交已支付的 TRON 交易哈希）

```rust
fn take_usdt_sell_order(
    origin: OriginFor<T>,
    order_id: u64,
    amount: Option<TokenBalance>,
    tron_tx_hash: Vec<u8>,     // 64 字符 hex
) -> DispatchResult;
```

#### 8. `accept_usdt_buy_order`

接受 USDT 买单（卖家发起）

```rust
fn accept_usdt_buy_order(
    origin: OriginFor<T>,
    order_id: u64,
    amount: Option<TokenBalance>,
    tron_address: Vec<u8>,     // 卖家 TRON 收款地址
) -> DispatchResult;
```

#### 9. `confirm_usdt_payment`

买家确认 USDT 支付（提交交易哈希）

```rust
fn confirm_usdt_payment(
    origin: OriginFor<T>,
    trade_id: u64,
    tron_tx_hash: Vec<u8>,
) -> DispatchResult;
```

#### 10. `verify_usdt_payment`

OCW 验证 USDT 支付结果

```rust
fn verify_usdt_payment(
    origin: OriginFor<T>,
    trade_id: u64,
    verified: bool,
    actual_amount: u64,
) -> DispatchResult;
```

#### 11. `process_usdt_timeout`

处理超时的 USDT 交易（任何人可调用）

```rust
fn process_usdt_timeout(
    origin: OriginFor<T>,
    trade_id: u64,
) -> DispatchResult;
```

### 市价单

#### 12. `market_buy`

市价买单（立即以最优卖价成交）

```rust
fn market_buy(
    origin: OriginFor<T>,
    shop_id: u64,
    token_amount: TokenBalance,
    max_cost: Balance,             // 滑点保护
) -> DispatchResult;
```

#### 13. `market_sell`

市价卖单（立即以最优买价成交）

```rust
fn market_sell(
    origin: OriginFor<T>,
    shop_id: u64,
    token_amount: TokenBalance,
    min_receive: Balance,          // 滑点保护
) -> DispatchResult;
```

### 管理

#### 14. `configure_market`

配置店铺市场（店主调用）

```rust
fn configure_market(
    origin: OriginFor<T>,
    shop_id: u64,
    cos_enabled: bool,
    usdt_enabled: bool,
    fee_rate: u16,
    min_order_amount: u128,
    order_ttl: u32,
    usdt_timeout: u32,
) -> DispatchResult;
```

## 事件

| 事件 | 说明 |
|------|------|
| `OrderCreated` | COS 订单已创建 |
| `OrderFilled` | COS 订单已成交（部分或全部） |
| `OrderCancelled` | 订单已取消 |
| `MarketConfigured` | 市场配置已更新 |
| `UsdtSellOrderCreated` | USDT 卖单已创建 |
| `UsdtBuyOrderCreated` | USDT 买单已创建 |
| `UsdtTradeCreated` | USDT 交易已创建（等待支付） |
| `UsdtPaymentSubmitted` | USDT 支付已提交（等待验证） |
| `UsdtTradeCompleted` | USDT 交易已完成 |
| `UsdtTradeVerificationFailed` | USDT 交易验证失败 |
| `UsdtTradeRefunded` | USDT 交易已超时退款 |
| `MarketOrderExecuted` | 市价单已执行 |
| `BestPricesUpdated` | 最优价格已更新 |
| `TwapUpdated` | TWAP 价格已更新 |
| `CircuitBreakerTriggered` | 熔断已触发 |
| `CircuitBreakerLifted` | 熔断已解除 |
| `PriceProtectionConfigured` | 价格保护配置已更新 |
| `InitialPriceSet` | 初始价格已设置 |

## 错误

| 错误 | 说明 |
|------|------|
| `ShopNotFound` | 店铺不存在 |
| `TokenNotEnabled` | 店铺代币未启用 |
| `MarketNotEnabled` | COS 市场未启用 |
| `UsdtMarketNotEnabled` | USDT 市场未启用 |
| `OrderNotFound` | 订单不存在 |
| `NotOrderOwner` | 不是订单所有者 |
| `OrderClosed` | 订单已关闭 |
| `InsufficientBalance` | COS 余额不足 |
| `InsufficientTokenBalance` | Token 余额不足 |
| `CannotTakeOwnOrder` | 不能吃自己的单 |
| `InvalidTronAddress` | 无效的 TRON 地址 |
| `UsdtTradeNotFound` | USDT 交易不存在 |
| `NotTradeParticipant` | 不是交易参与者 |
| `InvalidTradeStatus` | 交易状态无效 |
| `TradeTimeout` | 交易已超时 |
| `InvalidTxHash` | 无效的交易哈希 |
| `ChannelMismatch` | 通道不匹配 |
| `NoOrdersAvailable` | 没有可用订单 |
| `SlippageExceeded` | 滑点超限 |
| `PriceDeviationTooHigh` | 价格偏离 TWAP 过大 |
| `MarketCircuitBreakerActive` | 市场处于熔断状态 |
| `InsufficientTwapData` | TWAP 数据不足 |

## 存储项

| 存储项 | 说明 |
|--------|------|
| `NextOrderId` | 下一个订单 ID |
| `Orders` | 订单存储 |
| `ShopSellOrders` | 店铺卖单列表 |
| `ShopBuyOrders` | 店铺买单列表 |
| `UserOrders` | 用户订单列表 |
| `MarketConfigs` | 店铺市场配置 |
| `MarketStatsStorage` | 市场统计数据 |
| `NextUsdtTradeId` | 下一个 USDT 交易 ID |
| `UsdtTrades` | USDT 交易记录存储 |
| `PendingUsdtTrades` | 待验证的 USDT 交易列表 |
| `BestAsk` | 店铺最优卖价 |
| `BestBid` | 店铺最优买价 |
| `LastTradePrice` | 店铺最新成交价 |
| `MarketSummaryStorage` | 店铺市场摘要 |
| `TwapAccumulators` | TWAP 累积器（每店铺） |
| `PriceProtection` | 价格保护配置（每店铺） |

## 配置参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `DefaultOrderTTL` | 默认订单有效期 | 14400 区块（约24小时） |
| `MaxActiveOrdersPerUser` | 每用户最大活跃订单数 | 100 |
| `DefaultFeeRate` | 默认手续费率 | 100 基点（1%） |
| `DefaultUsdtTimeout` | USDT 交易默认超时 | 7200 区块（约12小时） |
| `BlocksPerHour` | 1小时对应区块数 | 600 区块 |
| `BlocksPerDay` | 24小时对应区块数 | 14400 区块 |
| `BlocksPerWeek` | 7天对应区块数 | 100800 区块 |
| `CircuitBreakerDuration` | 熔断持续时间 | 600 区块（约1小时） |

## 使用示例

### 1. 店主配置市场

```rust
// 启用 COS 和 USDT 交易
EntityMarket::configure_market(
    Origin::signed(shop_owner),
    shop_id,
    true,   // cos_enabled
    true,   // usdt_enabled
    50,     // fee_rate (0.5%)
    100,    // min_order_amount
    14400,  // order_ttl (24h)
    7200,   // usdt_timeout (12h)
)?;
```

### 2. COS 通道 - 挂卖单

```rust
// 以 0.1 COS/Token 的价格出售 1000 Token
EntityMarket::place_sell_order(
    Origin::signed(alice),
    shop_id,
    1000,           // token_amount
    100_000_000,    // price (0.1 COS, 精度 10^9)
)?;
```

### 3. COS 通道 - 吃单

```rust
// 吃掉订单 #1 的全部
EntityMarket::take_order(
    Origin::signed(bob),
    1,      // order_id
    None,   // amount (全部)
)?;
```

### 4. USDT 通道 - 挂卖单

```rust
// 以 0.01 USDT/Token 的价格出售 1000 Token
EntityMarket::place_usdt_sell_order(
    Origin::signed(alice),
    shop_id,
    1000,                           // token_amount
    10_000,                         // usdt_price (0.01 USDT, 精度 10^6)
    b"TXyz...".to_vec(),           // tron_address
)?;
```

### 5. USDT 通道 - 吃卖单

```rust
// Bob 先链下转 USDT 到 Alice 的 TRON 地址，然后提交交易哈希
EntityMarket::take_usdt_sell_order(
    Origin::signed(bob),
    order_id,
    None,                                           // amount (全部)
    b"abc123...".to_vec(),                         // tron_tx_hash (64 字符)
)?;
// 等待 OCW 验证...
```

### 6. USDT 通道 - 接受买单

```rust
// Alice 接受 Bob 的 USDT 买单
EntityMarket::accept_usdt_buy_order(
    Origin::signed(alice),
    order_id,
    None,                           // amount (全部)
    b"TXyz...".to_vec(),           // tron_address
)?;

// Bob 链下转 USDT 后确认支付
EntityMarket::confirm_usdt_payment(
    Origin::signed(bob),
    trade_id,
    b"abc123...".to_vec(),         // tron_tx_hash
)?;
// 等待 OCW 验证...
```

### 7. 市价买单

```rust
// 以最优价格立即购买 1000 Token，最多支付 200 COS
EntityMarket::market_buy(
    Origin::signed(bob),
    shop_id,
    1000,                           // token_amount
    200_000_000_000,               // max_cost (200 COS, 滑点保护)
)?;
```

### 8. 市价卖单

```rust
// 以最优价格立即出售 1000 Token，最少收到 80 COS
EntityMarket::market_sell(
    Origin::signed(alice),
    shop_id,
    1000,                           // token_amount
    80_000_000_000,                // min_receive (80 COS, 滑点保护)
)?;
```

### 9. 查询订单簿深度

```rust
// 获取订单簿深度（每边 10 档）
let depth = EntityMarket::get_order_book_depth(shop_id, 10);
// depth.asks: 卖盘（按价格升序）
// depth.bids: 买盘（按价格降序）
// depth.best_ask: 最优卖价
// depth.best_bid: 最优买价
// depth.spread: 买卖价差
```

### 10. 查询市场摘要

```rust
// 获取市场摘要
let summary = EntityMarket::get_market_summary(shop_id);
// summary.best_ask: 最优卖价
// summary.best_bid: 最优买价
// summary.last_price: 最新成交价
// summary.total_ask_amount: 卖单总量
// summary.total_bid_amount: 买单总量
```

### 11. 查询最优价格

```rust
// 获取最优买卖价
let (best_ask, best_bid) = EntityMarket::get_best_prices(shop_id);

// 获取买卖价差
let spread = EntityMarket::get_spread(shop_id);
```

### 12. 配置价格保护

```rust
// 店主配置价格保护参数
EntityMarket::configure_price_protection(
    Origin::signed(shop_owner),
    shop_id,
    true,   // enabled: 启用价格保护
    2000,   // max_price_deviation: 20% (基点)
    500,    // max_slippage: 5% (基点)
    5000,   // circuit_breaker_threshold: 50% (基点)
    100,    // min_trades_for_twap: 最少100笔交易后启用TWAP
)?;
```

### 13. 查询 TWAP 价格

```rust
// 获取 1小时 TWAP
let twap_1h = EntityMarket::calculate_twap(shop_id, TwapPeriod::OneHour);

// 获取 24小时 TWAP
let twap_24h = EntityMarket::calculate_twap(shop_id, TwapPeriod::OneDay);

// 获取 7天 TWAP
let twap_7d = EntityMarket::calculate_twap(shop_id, TwapPeriod::OneWeek);
```

### 14. 解除熔断

```rust
// 店主在熔断时间到期后手动解除
EntityMarket::lift_circuit_breaker(
    Origin::signed(shop_owner),
    shop_id,
)?;
```

### 15. 设置店铺代币初始价格

```rust
// 店主设置初始参考价格（用于 TWAP 冷启动）
// 在市场成交量不足时，将使用此价格作为价格偏离检查的参考
EntityMarket::set_initial_price(
    Origin::signed(shop_owner),
    shop_id,
    100_000_000_000,  // 初始价格: 100 COS / Token
)?;
```

**初始价格使用逻辑：**
1. 如果成交量 >= `min_trades_for_twap`，使用 1小时 TWAP
2. 如果成交量不足但有初始价格，使用店主设定的初始价格
3. 如果都没有，跳过价格偏离检查

## 依赖模块

- `pallet-entity-common`: 公共类型和 Trait
- `pallet-entity-shop`: 店铺信息查询
- `pallet-entity-token`: 店铺代币操作

## 版本历史

- **v0.1.0** (2026-02-01): Phase 1，实现 COS 通道限价单
- **v0.2.0** (2026-02-01): Phase 2，实现 USDT 通道 + OCW 验证
- **v0.3.0** (2026-02-01): Phase 3，实现市价单支持
- **v0.4.0** (2026-02-01): Phase 4，实现订单簿深度优化
- **v0.5.0** (2026-02-01): Phase 5，实现三周期 TWAP 价格预言机

## 后续计划

- [x] Phase 1: COS 通道限价单 ✅
- [x] Phase 2: USDT 通道 + OCW 验证 ✅
- [x] Phase 3: 市价单支持 ✅
- [x] Phase 4: 订单簿深度优化 ✅
- [x] Phase 5: 价格预言机集成 ✅
