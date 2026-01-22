# OTC 订单模块 (pallet-otc-order)

## 模块概述

OTC（场外交易）订单模块负责管理 OTC 订单的完整生命周期，是 Stardust 交易系统的核心组件之一。本模块提供了从订单创建到完成的全流程管理，包括首购订单特殊逻辑、订单争议与仲裁、自动过期处理等功能。

### 主要特性

- **订单生命周期管理**：支持订单创建、付款标记、DUST 释放、取消等完整流程
- **首购订单机制**：为新用户提供固定 USD 价值的首购订单，动态计算 DUST 数量
- **KYC 身份认证**：集成 pallet-identity，支持可配置的 KYC 认证要求
- **买家押金机制**：根据买家信用分动态计算押金比例，保护做市商权益
- **订单争议与仲裁**：支持买卖双方发起争议，由仲裁员进行裁决
- **自动过期处理**：链上自动检测并处理超时订单
- **聊天权限集成**：订单创建时自动授予买卖双方聊天权限
- **存储膨胀防护**：多级订单归档机制，防止链上存储无限增长

### 版本历史

| 版本 | 日期 | 更新内容 |
|------|------|----------|
| v0.1.0 | 2025-11-03 | 从 pallet-trading 拆分而来 |
| v0.2.0 | 2025-11-13 | 集成 KYC 认证功能 |
| v0.3.0 | 2025-11-28 | 集成聊天权限系统 |
| v0.4.0 | 2026-01-18 | 买家押金机制、自动过期处理、存储膨胀防护 |

---

## 核心功能

### 1. 订单创建与管理

#### 普通订单 (`create_order`)

买家指定做市商和 DUST 数量创建订单：

1. 验证买家 KYC 认证状态
2. 验证订单金额在允许范围内（20-200 USD）
3. 验证做市商状态和押金充足
4. 获取当前 DUST/USD 价格计算总金额
5. 锁定做市商 DUST 到托管账户
6. 计算并锁定买家押金（根据信用分）
7. 创建订单记录并授予聊天权限

#### 首购订单 (`create_first_purchase`)

为新用户提供的特殊订单类型：

- **固定 USD 价值**：10 USD（可配置）
- **动态 DUST 数量**：根据当前价格实时计算
- **免押金**：首购用户无需缴纳押金
- **配额限制**：每个做市商同时最多接收 5 个首购订单

### 2. 订单状态流转

```
┌─────────┐    mark_paid    ┌─────────────────┐   release_dust   ┌──────────┐
│ Created │ ──────────────► │ PaidOrCommitted │ ───────────────► │ Released │
└─────────┘                 └─────────────────┘                  └──────────┘
     │                              │                                  │
     │ cancel_order                 │ dispute_order                    │
     ▼                              ▼                                  ▼
┌──────────┐                 ┌───────────┐                      ┌───────────┐
│ Canceled │                 │ Disputed  │ ──────────────────► │  Closed   │
└──────────┘                 └───────────┘    resolve_dispute   └───────────┘
     ▲                              │
     │ 超时自动处理                  │ 仲裁判定
     │                              ▼
┌─────────┐                  ┌──────────┐
│ Expired │                  │ Refunded │
└─────────┘                  └──────────┘
```

### 3. 首购订单特殊逻辑

首购订单为新用户提供低门槛的入门体验：

| 特性 | 普通订单 | 首购订单 |
|------|----------|----------|
| 最小金额 | 20 USD | 10 USD（固定） |
| 最大金额 | 200 USD | 10 USD（固定） |
| DUST 数量 | 买家指定 | 系统计算 |
| 买家押金 | 根据信用分 | 免押金 |
| 每用户限制 | 无 | 仅限一次 |

### 4. KYC 身份认证

本模块集成 pallet-identity 进行身份认证：

- **认证等级**：Unknown(0) < FeePaid(1) < Reasonable(2) < KnownGood(3)
- **可配置要求**：委员会可设置最低认证等级
- **豁免机制**：支持将特定账户添加到豁免列表
- **紧急开关**：委员会可随时启用/禁用 KYC 要求

### 5. 买家押金机制

根据买家信用评估动态计算押金：

| 信用等级 | 信用分范围 | 押金比例 |
|----------|------------|----------|
| 信用用户 | ≥70 分且 ≥5 单 | 免押金 |
| 普通用户 | 50-69 分 | 3% |
| 低信用用户 | 30-49 分 | 5% |
| 高风险用户 | <30 分 | 10% |

**押金处理规则**：
- 订单完成：100% 退还买家
- 买家主动取消：30% 没收给做市商，70% 退还
- 订单超时：100% 没收给做市商
- 争议买家胜诉：100% 退还买家
- 争议做市商胜诉：100% 没收给做市商

### 6. 订单争议与仲裁

争议流程：

1. **发起争议**：买家在证据窗口期内发起争议，提交证据 CID
2. **做市商响应**：做市商在响应期限内提交反驳证据
3. **仲裁判定**：仲裁员根据证据做出裁决
4. **执行裁决**：系统自动执行资金分配

> **注意**：争议功能已迁移到统一仲裁模块 (pallet-arbitration)，建议使用 `arbitration.file_complaint` 替代 `initiate_dispute`。

### 7. 自动过期处理

系统在 `on_initialize` 钩子中自动处理过期订单：

- **检查频率**：每 100 个区块（约 10 分钟）
- **处理数量**：每次最多处理 10 个过期订单
- **过期条件**：Created 状态且超过 OrderTimeout（默认 1 小时）
- **过期处理**：退还托管资金、释放买家额度、没收押金

---

## 数据结构

### OrderState（订单状态）

```rust
pub enum OrderState {
    Created,          // 已创建，等待买家付款
    PaidOrCommitted,  // 买家已标记付款
    Released,         // DUST 已释放
    Refunded,         // 已退款
    Canceled,         // 已取消
    Disputed,         // 争议中
    Closed,           // 已关闭
    Expired,          // 已过期
}
```

### DepositStatus（押金状态）

```rust
pub enum DepositStatus {
    None,               // 无押金（首购/信用免押）
    Locked,             // 押金已锁定
    Released,           // 押金已释放（订单完成）
    Forfeited,          // 押金已没收（超时/争议败诉）
    PartiallyForfeited, // 押金部分没收（买家主动取消）
}
```

### DisputeStatus（争议状态）

```rust
pub enum DisputeStatus {
    WaitingMakerResponse,  // 等待做市商响应
    WaitingArbitration,    // 等待仲裁
    BuyerWon,              // 买家胜诉
    MakerWon,              // 做市商胜诉
    Cancelled,             // 已取消
}
```

### Order（订单结构）

```rust
pub struct Order<T: Config> {
    pub maker_id: u64,                        // 做市商 ID
    pub maker: T::AccountId,                  // 做市商账户
    pub taker: T::AccountId,                  // 买家账户
    pub price: BalanceOf<T>,                  // 单价（USDT/DUST，精度 10^6）
    pub qty: BalanceOf<T>,                    // 数量（DUST 数量）
    pub amount: BalanceOf<T>,                 // 总金额（USDT 金额）
    pub created_at: MomentOf,                 // 创建时间（Unix 秒）
    pub expire_at: MomentOf,                  // 超时时间
    pub evidence_until: MomentOf,             // 证据窗口截止时间
    pub maker_tron_address: TronAddress,      // 做市商 TRON 收款地址
    pub payment_commit: H256,                 // 支付承诺哈希
    pub contact_commit: H256,                 // 联系方式承诺哈希
    pub state: OrderState,                    // 订单状态
    pub completed_at: Option<MomentOf>,       // 完成时间
    pub is_first_purchase: bool,              // 是否为首购订单
    pub buyer_deposit: BalanceOf<T>,          // 买家押金金额
    pub deposit_status: DepositStatus,        // 押金状态
}
```

### Dispute（争议记录）

```rust
pub struct Dispute<T: Config> {
    pub order_id: u64,                        // 订单 ID
    pub initiator: T::AccountId,              // 发起方（买家）
    pub respondent: T::AccountId,             // 被告方（做市商）
    pub created_at: MomentOf,                 // 发起时间
    pub response_deadline: MomentOf,          // 做市商响应截止时间
    pub arbitration_deadline: MomentOf,       // 仲裁截止时间
    pub status: DisputeStatus,                // 争议状态
    pub buyer_evidence: Option<Cid>,          // 买家证据 CID
    pub maker_evidence: Option<Cid>,          // 做市商证据 CID
}
```

### ArchivedOrder（归档订单 L1）

```rust
pub struct ArchivedOrder<T: Config> {
    pub maker_id: u64,           // 做市商 ID
    pub taker: T::AccountId,     // 买家账户
    pub qty: u64,                // DUST 数量（压缩）
    pub amount: u64,             // USDT 金额（压缩）
    pub state: OrderState,       // 订单状态
    pub completed_at: u64,       // 完成时间
}
```

### ArchivedOrderL2（归档订单 L2）

```rust
pub struct ArchivedOrderL2 {
    pub id: u64,           // 订单 ID
    pub status: u8,        // 订单状态 (0-7)
    pub year_month: u16,   // 年月 (YYMM 格式)
    pub amount_tier: u8,   // 金额档位 (0-5)
    pub flags: u32,        // 保留标志位
}
```

### KycConfig（KYC 配置）

```rust
pub struct KycConfig<BlockNumber> {
    pub enabled: bool,                  // 是否启用 KYC 要求
    pub min_judgment_priority: u8,      // 最低认证等级
    pub effective_block: BlockNumber,   // 配置生效区块
    pub updated_at: BlockNumber,        // 最后更新时间
}
```

---

## 存储项

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `NextOrderId` | `u64` | 下一个订单 ID |
| `Orders` | `Map<u64, Order>` | 订单记录 |
| `BuyerOrders` | `Map<AccountId, Vec<u64>>` | 买家订单列表（最多 100 个） |
| `MakerOrders` | `Map<u64, Vec<u64>>` | 做市商订单列表（最多 200 个） |
| `HasFirstPurchased` | `Map<AccountId, bool>` | 买家是否已首购 |
| `MakerFirstPurchaseCount` | `Map<u64, u32>` | 做市商首购订单计数 |
| `MakerFirstPurchaseOrders` | `Map<u64, Vec<u64>>` | 做市商首购订单列表 |
| `TronTxUsed` | `Map<H256, BlockNumber>` | TRON 交易哈希使用记录 |
| `TronTxQueue` | `Vec<(H256, BlockNumber)>` | TRON 交易哈希清理队列 |
| `Disputes` | `Map<u64, Dispute>` | 争议记录 |
| `BuyerCompletedOrderCount` | `Map<AccountId, u32>` | 买家已完成订单计数 |
| `TotalDepositPoolBalance` | `Balance` | 押金池总余额 |
| `ArchivedOrders` | `Map<u64, ArchivedOrder>` | L1 归档订单 |
| `ArchivedOrdersL2` | `Map<u64, ArchivedOrderL2>` | L2 归档订单 |
| `ArchiveCursor` | `u64` | 归档游标 |
| `L1ArchiveCursor` | `u64` | L1 归档游标 |
| `OtcStats` | `OtcPermanentStats` | OTC 永久统计 |
| `KycConfig` | `KycConfig` | KYC 配置 |
| `KycExemptAccounts` | `Map<AccountId, ()>` | KYC 豁免账户 |

---

## Extrinsics（可调用函数）

### 订单管理

| 函数 | 调用者 | 说明 |
|------|--------|------|
| `create_order` | 买家 | 创建 OTC 订单 |
| `create_first_purchase` | 买家 | 创建首购订单 |
| `mark_paid` | 买家 | 标记已付款 |
| `release_dust` | 做市商 | 释放 DUST 给买家 |
| `cancel_order` | 买家/做市商 | 取消订单 |
| `dispute_order` | 买家/做市商 | 发起订单争议 |

### 争议管理（已废弃，请使用 pallet-arbitration）

| 函数 | 调用者 | 说明 |
|------|--------|------|
| `initiate_dispute` | 买家 | 发起争议（已废弃） |
| `respond_dispute` | 做市商 | 响应争议（已废弃） |
| `resolve_dispute` | 仲裁员 | 判定争议（已废弃） |

### KYC 管理

| 函数 | 调用者 | 说明 |
|------|--------|------|
| `enable_kyc_requirement` | 委员会 | 启用 KYC 要求 |
| `disable_kyc_requirement` | 委员会 | 禁用 KYC 要求 |
| `update_min_judgment_level` | 委员会 | 更新最低认证等级 |
| `exempt_account_from_kyc` | 委员会 | 添加 KYC 豁免账户 |
| `remove_kyc_exemption` | 委员会 | 移除 KYC 豁免 |

---

## 事件（Events）

### 订单事件

| 事件 | 说明 |
|------|------|
| `OrderCreated` | 订单已创建 |
| `OrderStateChanged` | 订单状态已变更 |
| `FirstPurchaseOrderCreated` | 首购订单已创建 |
| `OrderAutoExpired` | 订单已自动过期 |
| `ExpiredOrdersProcessed` | 过期订单批量处理完成 |

### 押金事件

| 事件 | 说明 |
|------|------|
| `BuyerDepositLocked` | 买家押金已锁定 |
| `BuyerDepositReleased` | 买家押金已释放 |
| `BuyerDepositForfeited` | 买家押金已没收 |
| `BuyerDepositPartiallyForfeited` | 买家押金部分没收 |

### 争议事件

| 事件 | 说明 |
|------|------|
| `DisputeInitiated` | 争议已发起 |
| `DisputeResponded` | 做市商已响应争议 |
| `DisputeResolved` | 争议已判定 |

### KYC 事件

| 事件 | 说明 |
|------|------|
| `KycEnabled` | KYC 要求已启用 |
| `KycDisabled` | KYC 要求已禁用 |
| `KycLevelUpdated` | KYC 最低等级已更新 |
| `AccountExemptedFromKyc` | 账户已添加到豁免列表 |
| `AccountRemovedFromKycExemption` | 账户已从豁免列表移除 |
| `KycVerificationFailed` | KYC 验证失败 |

### 其他事件

| 事件 | 说明 |
|------|------|
| `TronTxHashRecorded` | TRON 交易哈希已记录 |
| `TronTxHashCleaned` | TRON 交易哈希已清理 |

---

## 错误（Errors）

### 订单错误

| 错误 | 说明 |
|------|------|
| `OrderNotFound` | 订单不存在 |
| `InvalidOrderStatus` | 订单状态不正确 |
| `NotAuthorized` | 未授权 |
| `TooManyOrders` | 订单太多 |
| `OrderAmountExceedsLimit` | 订单金额超过限制 |
| `OrderAmountTooSmall` | 订单金额太小 |

### 做市商错误

| 错误 | 说明 |
|------|------|
| `MakerNotFound` | 做市商不存在 |
| `MakerNotActive` | 做市商未激活 |
| `MakerInsufficientBalance` | 做市商余额不足 |
| `MakerDepositInsufficient` | 做市商押金不足 |

### 首购错误

| 错误 | 说明 |
|------|------|
| `AlreadyFirstPurchased` | 已经首购过 |
| `FirstPurchaseQuotaExhausted` | 首购配额已用完 |

### 定价错误

| 错误 | 说明 |
|------|------|
| `PricingUnavailable` | 定价不可用 |
| `PricingServiceUnavailable` | 定价服务不可用 |
| `InvalidPrice` | 价格无效 |
| `CalculationOverflow` | 计算溢出 |
| `AmountCalculationOverflow` | 金额计算溢出 |

### KYC 错误

| 错误 | 说明 |
|------|------|
| `IdentityNotSet` | 未设置身份信息 |
| `NoValidJudgement` | 没有有效的身份判断 |
| `InsufficientKycLevel` | KYC 认证等级不足 |
| `IdentityQualityIssue` | 身份认证质量问题 |
| `AccountAlreadyExempted` | 账户已在豁免列表中 |
| `AccountNotExempted` | 账户不在豁免列表中 |

### 押金错误

| 错误 | 说明 |
|------|------|
| `InsufficientDepositBalance` | 买家押金余额不足 |

### 争议错误

| 错误 | 说明 |
|------|------|
| `DisputeNotFound` | 争议不存在 |
| `InvalidDisputeStatus` | 争议状态不正确 |
| `NotDisputeInitiator` | 非争议发起方 |
| `NotDisputeRespondent` | 非争议响应方 |
| `DisputeResponseTimeout` | 争议响应已超时 |
| `NotOrderBuyer` | 不是订单买家 |

### 其他错误

| 错误 | 说明 |
|------|------|
| `EncodingError` | 编码错误 |
| `StorageLimitReached` | 存储限制已达到 |
| `TronTxHashAlreadyUsed` | TRON 交易哈希已使用 |

---

## 配置参数

### 时间配置

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `OrderTimeout` | 3600 秒（1 小时） | 订单超时时间 |
| `EvidenceWindow` | 86400 秒（24 小时） | 证据窗口时间 |
| `DisputeResponseTimeout` | 86400 秒（24 小时） | 争议响应超时时间 |
| `DisputeArbitrationTimeout` | 172800 秒（48 小时） | 争议仲裁超时时间 |

### 金额配置

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `FirstPurchaseUsdValue` | 10,000,000（10 USD） | 首购订单固定 USD 价值 |
| `FirstPurchaseUsdAmount` | 10,000,000（10 USD） | 首购订单固定 USD 金额 |
| `MaxOrderUsdAmount` | 200,000,000（200 USD） | 最大订单金额 |
| `MinOrderUsdAmount` | 20,000,000（20 USD） | 最小订单金额 |
| `MinMakerDepositUsd` | 800,000,000（800 USD） | 做市商最低押金 USD 价值 |

### 首购配置

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `MinFirstPurchaseDustAmount` | - | 首购订单最小 DUST 数量 |
| `MaxFirstPurchaseDustAmount` | - | 首购订单最大 DUST 数量 |
| `MaxFirstPurchaseOrdersPerMaker` | 5 | 每个做市商最多同时接收的首购订单数 |
| `AmountValidationTolerance` | 100（1%） | 金额验证容差 |

### 押金配置

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `MinDeposit` | - | 最小押金金额 |
| `DepositRateLow` | 300（3%） | 低风险押金比例（信用分 50-69） |
| `DepositRateMedium` | 500（5%） | 中风险押金比例（信用分 30-49） |
| `DepositRateHigh` | 1000（10%） | 高风险押金比例（信用分 <30） |
| `CreditScoreExempt` | 70 | 免押金信用分阈值 |
| `MinOrdersForExempt` | 5 | 免押金最少完成订单数 |
| `CancelPenaltyRate` | 3000（30%） | 取消订单押金扣除比例 |

---

## 使用示例

### 1. 创建普通订单

```rust
// 买家创建 OTC 订单
// maker_id: 做市商 ID
// dust_amount: 购买的 DUST 数量
// payment_commit: 支付承诺哈希（用于验证付款）
// contact_commit: 联系方式承诺哈希
OtcOrder::create_order(
    origin,
    maker_id,
    dust_amount,
    payment_commit,
    contact_commit,
)?;
```

### 2. 创建首购订单

```rust
// 新用户创建首购订单（固定 10 USD）
OtcOrder::create_first_purchase(
    origin,
    maker_id,
    payment_commit,
    contact_commit,
)?;
```

### 3. 买家标记付款

```rust
// 买家完成链下付款后，标记订单为已付款
// tron_tx_hash: 可选的 TRON 交易哈希（用于防重放）
OtcOrder::mark_paid(
    origin,
    order_id,
    Some(tron_tx_hash),
)?;
```

### 4. 做市商释放 DUST

```rust
// 做市商确认收到付款后，释放 DUST 给买家
OtcOrder::release_dust(
    origin,
    order_id,
)?;
```

### 5. 取消订单

```rust
// 买家或做市商取消订单（仅限 Created 状态）
OtcOrder::cancel_order(
    origin,
    order_id,
)?;
```

### 6. 发起争议

```rust
// 买家发起订单争议
OtcOrder::dispute_order(
    origin,
    order_id,
)?;

// 推荐使用统一仲裁模块
Arbitration::file_complaint(
    origin,
    b"otc_ord_",  // OTC 域常量
    order_id,
    complaint_type,
    evidence_cid,
)?;
```

### 7. KYC 管理

```rust
// 委员会启用 KYC 要求（最低等级：Reasonable = 2）
OtcOrder::enable_kyc_requirement(
    committee_origin,
    2,  // min_judgment_priority
)?;

// 委员会禁用 KYC 要求
OtcOrder::disable_kyc_requirement(committee_origin)?;

// 添加 KYC 豁免账户
OtcOrder::exempt_account_from_kyc(
    committee_origin,
    account,
)?;
```

### 8. 查询订单信息

```rust
// 获取订单详情（含可读时间）
let order_info = OtcOrder::get_order_with_time(order_id);

// 获取买家所有订单
let buyer_orders = OtcOrder::get_buyer_orders_with_time(&buyer);

// 检查买家是否已首购
let has_purchased = OtcOrder::has_user_first_purchased(&buyer);

// 获取做市商首购订单数量
let count = OtcOrder::get_maker_first_purchase_count(maker_id);
```

---

## 依赖模块

| 模块 | 用途 |
|------|------|
| `pallet-escrow` | 托管服务，锁定/释放 DUST |
| `pallet-trading-credit` | 买家信用记录和额度管理 |
| `pallet-trading-common` | 公共类型和 Trait 定义 |
| `pallet-chat-permission` | 聊天权限管理 |
| `pallet-identity` | KYC 身份认证 |
| `pallet-arbitration` | 统一仲裁服务 |
| `pallet-stardust-ipfs` | CID 锁定管理（争议证据） |
| `pallet-storage-lifecycle` | 存储生命周期管理 |

---

## 存储膨胀防护

本模块实现了多级订单归档机制，防止链上存储无限增长：

### 归档流程

```
活跃订单 (Orders)
    │
    │ 30天后（on_idle）
    ▼
L1 归档 (ArchivedOrders)
    │  ~48 字节/订单
    │
    │ 90天后（on_idle）
    ▼
L2 归档 (ArchivedOrdersL2)
       ~16 字节/订单
```

### 归档条件

- **L1 归档**：订单完成 30 天后，状态为 Closed/Released/Refunded/Canceled/Expired
- **L2 归档**：L1 归档 90 天后

### 永久统计

`OtcStats` 存储永久统计数据：
- 总订单数
- 已完成订单数
- 已取消订单数
- 总交易额

---

## 安全考虑

1. **KYC 认证**：可配置的身份认证要求，防止匿名恶意交易
2. **买家押金**：根据信用分动态计算，保护做市商权益
3. **TRON 交易哈希防重放**：记录已使用的交易哈希，防止重复提交
4. **做市商押金验证**：创建订单时验证做市商押金 USD 价值
5. **证据 CID 锁定**：争议期间自动锁定证据 CID，防止删除
6. **自动过期处理**：链上自动处理超时订单，避免资金长期锁定

---

## 许可证

Apache-2.0
