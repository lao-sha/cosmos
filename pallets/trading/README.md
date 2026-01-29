# Pallet Trading（统一交易接口层）

## 📋 模块概述

`pallet-trading` 是 Cosmos 区块链的 **统一交易接口层**，采用模块化设计理念，聚合以下四个独立子模块：

1. **pallet-maker** - 做市商管理（申请、审核、押金、提现）
2. **pallet-otc-order** - OTC 订单管理（创建、支付、释放、取消、争议）
3. **pallet-swap** - COS → USDT 做市商兑换服务（OCW 验证、超时退款）
4. **pallet-trading-common** - 公共工具库（数据掩码、验证）

### 核心价值

- ✅ **模块化设计**：子模块独立开发、测试、部署
- ✅ **低耦合架构**：修改子模块不影响其他模块
- ✅ **统一接口**：重新导出子模块类型，简化前端调用
- ✅ **聚合查询 API**：提供跨模块的聚合查询接口
- ✅ **灵活集成**：Runtime 可选择性集成子模块或全部
- ✅ **零迁移策略**：主网未上线，允许破坏式重构

### 设计理念

本模块是 **接口层**，而非 **存储层**。它不直接实现业务逻辑，而是：

1. **重新导出**子模块的类型定义
2. **聚合查询**跨模块的统计数据
3. **简化集成** Runtime 和前端的调用复杂度

---

## 🏗️ 架构设计

### 模块化架构图

```text
新架构（Phase 5 - 模块化重构）
====================================
pallet-trading (统一接口层，本 Pallet)
  ├── 重新导出子模块类型
  ├── 提供聚合查询接口
  └── 简化 Runtime 集成

pallet-maker (独立模块 - 做市商管理)
  ├── 做市商申请/审核
  ├── 押金管理（锁定/解锁）
  ├── 提现流程（冷却期）
  ├── 溢价配置（Buy/Sell Premium）
  ├── 服务暂停/恢复
  └── 押金自动补充机制

pallet-otc-order (独立模块 - OTC 订单)
  ├── 订单创建/支付
  ├── COS 释放
  ├── 首购逻辑（固定 10 USD）
  ├── 自动过期清理
  ├── 争议处理
  └── 信用分记录

pallet-swap (独立模块 - 做市商兑换服务)
  ├── 做市商兑换（市场化服务）
  ├── OCW 自动验证
  ├── 超时退款机制
  └── TRC20 交易哈希防重放

pallet-trading-common (工具库)
  ├── 数据掩码（姓名、身份证、生日）
  └── 数据验证（TRON 地址）
```

### 模块依赖关系

```text
pallet-trading (统一接口层)
    │
    ├─► pallet-maker
    │   ├── 提供做市商信息查询接口
    │   └── 被 pallet-otc-order 和 pallet-swap 调用
    │
    ├─► pallet-otc-order
    │   ├── 调用 pallet-maker::MakerInterface 查询做市商信息
    │   ├── 调用 pallet-escrow::Escrow 锁定/释放资金
    │   ├── 调用 pallet-credit::BuyerCreditInterface 记录买家信用
    │   ├── 调用 pallet-credit::MakerCreditInterface 记录做市商信用
    │   └── 调用 pallet-pricing::PricingProvider 添加价格数据
    │
    ├─► pallet-swap
    │   ├── 调用 pallet-maker::MakerInterface 查询做市商信息
    │   ├── 调用 pallet-escrow::Escrow 锁定/释放资金
    │   ├── 调用 pallet-credit::CreditInterface 记录信用
    │   └── 调用 pallet-pricing::PricingProvider 添加价格数据
    │
    └─► pallet-trading-common
        ├── 被 pallet-maker 调用（数据脱敏、验证）
        ├── 被 pallet-otc-order 调用（数据脱敏、验证）
        └── 被 pallet-swap 调用（数据脱敏、验证）
```

### 重构优势

| 优势 | 说明 | 具体体现 |
|------|------|----------|
| **低耦合** | 子模块独立开发、测试、部署 | 修改 Maker 逻辑不影响 OTC 和 Swap |
| **高内聚** | 每个模块职责单一清晰 | Maker 只管做市商，OTC 只管订单 |
| **易维护** | 修改子模块不影响其他模块 | 升级 Swap 不需要重新测试 Maker |
| **易测试** | 独立模块独立测试 | 每个模块有独立的 mock 和 tests |
| **灵活集成** | Runtime 可选择性集成 | 可以只集成 OTC，不集成 Swap |

---

## 🔑 核心功能

### 1. 类型导出

#### 1.1 Maker 相关类型

```rust
pub mod maker_types {
    pub use pallet_maker::{
        MakerApplication,      // 做市商申请记录
        ApplicationStatus,     // 申请状态枚举
        Direction,             // 业务方向（Buy/Sell/BuyAndSell）
        WithdrawalRequest,     // 提现请求
        WithdrawalStatus,      // 提现状态
    };
}
```

**主要类型说明：**

- **MakerApplication**: 做市商申请记录，包含账户、押金、TRON地址、脱敏资料等
- **ApplicationStatus**: `DepositLocked | PendingReview | Active | Rejected | Cancelled | Expired`
- **Direction**: 业务方向
  - `Buy = 0` - 仅买入（仅Bridge）
  - `Sell = 1` - 仅卖出（仅OTC）
  - `BuyAndSell = 2` - 双向（OTC + Bridge）

#### 1.2 OTC 相关类型

```rust
pub mod otc_types {
    pub use pallet_otc_order::{
        Order,                 // OTC 订单
        OrderState,            // 订单状态枚举
        PricingProvider,       // 定价服务接口
    };
}
```

**主要类型说明：**

- **Order**: OTC 订单记录，包含做市商、买家、价格、数量、状态等
- **OrderState**: `Created | PaidOrCommitted | Released | Refunded | Canceled | Disputed | Closed | Expired`

#### 1.3 Swap 相关类型

```rust
pub mod swap_types {
    pub use pallet_swap::{
        SwapStatus,            // 兑换状态枚举
        MakerSwapRecord,       // 做市商兑换记录
    };
}
```

**主要类型说明：**

- **SwapStatus**: `Pending | AwaitingVerification | Completed | VerificationFailed | UserReported | Arbitrating | ArbitrationApproved | ArbitrationRejected | Refunded`
- **MakerSwapRecord**: 做市商兑换记录，包含 TRC20 交易哈希

#### 1.4 公共工具

```rust
pub mod utils {
    pub use pallet_trading_common::{
        mask_name,             // 姓名脱敏
        mask_id_card,          // 身份证号脱敏
        mask_birthday,         // 生日脱敏
        is_valid_tron_address, // TRON 地址验证
    };
}
```

---

### 2. 聚合查询 API

#### TradingApi::get_platform_stats

获取平台统计信息，聚合所有子模块的数据。

**返回值：**

```rust
pub struct PlatformStats {
    pub total_makers: u64,   // 总做市商数（来自 pallet-maker）
    pub total_orders: u64,   // 总订单数（来自 pallet-otc-order）
    pub total_swaps: u64,    // 总兑换数（来自 pallet-swap）
}
```

**使用示例：**

```rust
// Runtime 端调用
let stats = TradingApi::get_platform_stats::<Runtime>();
println!("Total makers: {}", stats.total_makers);
println!("Total orders: {}", stats.total_orders);
println!("Total swaps: {}", stats.total_swaps);
```

---

## 📦 主要调用方法

### 1. pallet-maker（做市商管理）

#### 1.1 lock_deposit - 锁定押金

```rust
#[pallet::call_index(0)]
#[pallet::weight(T::WeightInfo::lock_deposit())]
pub fn lock_deposit(origin: OriginFor<T>) -> DispatchResult
```

**功能：** 做市商锁定押金，创建申请记录

**参数：** 无

**权限：** 已签名用户

**流程：**
1. 检查账户是否已申请
2. 锁定 `MakerDepositAmount` 押金
3. 创建申请记录（状态 = `DepositLocked`）
4. 设置资料提交截止时间（7天）

#### 1.2 submit_info - 提交做市商资料

```rust
#[pallet::call_index(1)]
#[pallet::weight(T::WeightInfo::submit_info())]
pub fn submit_info(
    origin: OriginFor<T>,
    real_name: Vec<u8>,           // 真实姓名
    id_card_number: Vec<u8>,      // 身份证号
    birthday: Vec<u8>,            // 生日（YYYY-MM-DD）
    tron_address: Vec<u8>,        // TRON 收款地址
    public_cid: Vec<u8>,          // 公开资料 IPFS CID
    private_cid: Vec<u8>,         // 私密资料 IPFS CID（加密）
    direction: u8,                // 业务方向（0=Buy, 1=Sell, 2=BuyAndSell）
    buy_premium_bps: i16,         // Buy 溢价（基点，-500~500）
    sell_premium_bps: i16,        // Sell 溢价（基点，-500~500）
    min_amount: BalanceOf<T>,     // 最小交易金额
    wechat_id: Vec<u8>,           // 微信号
    payment_methods_json: Vec<u8>, // 收款方式（JSON格式）
) -> DispatchResult
```

**功能：** 提交做市商资料，等待治理审核

**脱敏规则：**
- 姓名：`"张三" -> "×三"`
- 身份证：`"110101199001011234" -> "1101**********1234"`
- 生日：`"1990-01-01" -> "1990-xx-xx"`

#### 1.3 approve_maker / reject_maker - 审核做市商

```rust
#[pallet::call_index(2)]
#[pallet::weight(T::WeightInfo::approve_maker())]
pub fn approve_maker(origin: OriginFor<T>, maker_id: u64) -> DispatchResult

#[pallet::call_index(3)]
#[pallet::weight(T::WeightInfo::reject_maker())]
pub fn reject_maker(origin: OriginFor<T>, maker_id: u64) -> DispatchResult
```

**功能：** 治理委员会审批/驳回做市商申请

**权限：** `GovernanceOrigin`（治理委员会）

#### 1.4 request_withdrawal / execute_withdrawal - 提现流程

```rust
#[pallet::call_index(6)]
#[pallet::weight(T::WeightInfo::request_withdrawal())]
pub fn request_withdrawal(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult

#[pallet::call_index(7)]
#[pallet::weight(T::WeightInfo::execute_withdrawal())]
pub fn execute_withdrawal(origin: OriginFor<T>) -> DispatchResult
```

**功能：** 做市商申请提现 → 等待冷却期 → 执行提现

**冷却期：** `WithdrawalCooldown`（默认 7 天）

---

### 2. pallet-otc-order（OTC 订单管理）

#### 2.1 create_order - 创建普通订单

```rust
#[pallet::call_index(0)]
#[pallet::weight(T::WeightInfo::create_order())]
pub fn create_order(
    origin: OriginFor<T>,
    maker_id: u64,           // 做市商 ID
    cos_amount: BalanceOf<T>, // COS 数量
    payment_commit: H256,    // 支付承诺哈希
    contact_commit: H256,    // 联系方式承诺哈希
) -> DispatchResult
```

**功能：** 买家创建 OTC 订单，锁定 COS 到托管账户

**限制：**
- 最小金额：20 USD（`MinOrderUsdAmount`）
- 最大金额：200 USD（`MaxOrderUsdAmount`）
- 做市商必须 Active 状态

#### 2.2 create_first_purchase - 创建首购订单

```rust
#[pallet::call_index(1)]
#[pallet::weight(T::WeightInfo::create_first_purchase())]
pub fn create_first_purchase(
    origin: OriginFor<T>,
    maker_id: u64,           // 做市商 ID
    payment_commit: H256,    // 支付承诺哈希
    contact_commit: H256,    // 联系方式承诺哈希
) -> DispatchResult
```

**功能：** 买家创建首购订单（固定 10 USD 价值）

**首购规则：**
- 固定 USD 价值：10 USD（`FirstPurchaseUsdAmount`）
- 动态 COS 数量：根据实时汇率计算
- 每个买家只能首购一次
- 每个做市商最多同时接收 5 个首购订单

#### 2.3 mark_paid - 标记已付款

```rust
#[pallet::call_index(2)]
#[pallet::weight(T::WeightInfo::mark_paid())]
pub fn mark_paid(
    origin: OriginFor<T>,
    order_id: u64,
    tron_tx_hash: Option<Vec<u8>>, // TRON 交易哈希（可选）
) -> DispatchResult
```

**功能：** 买家标记已付款，通知做市商释放 COS

#### 2.4 release_cos - 释放 COS

```rust
#[pallet::call_index(3)]
#[pallet::weight(T::WeightInfo::release_cos())]
pub fn release_cos(origin: OriginFor<T>, order_id: u64) -> DispatchResult
```

**功能：** 做市商确认收款，释放 COS 给买家

**副作用：**
- 记录做市商信用分（`MakerCredit::record_maker_order_completed`）
- 记录买家信用分（`BuyerCredit::record_buyer_order_completed`）
- 提升买家额度（`BuyerQuota::increase_buyer_quota`）

#### 2.5 cancel_order / dispute_order - 取消/争议

```rust
#[pallet::call_index(4)]
#[pallet::weight(T::WeightInfo::cancel_order())]
pub fn cancel_order(origin: OriginFor<T>, order_id: u64) -> DispatchResult

#[pallet::call_index(5)]
#[pallet::weight(T::WeightInfo::dispute_order())]
pub fn dispute_order(origin: OriginFor<T>, order_id: u64) -> DispatchResult
```

**功能：** 买家或做市商取消订单 / 发起争议

---

### 3. pallet-swap（做市商兑换服务）

> ℹ️ **版本说明**: v0.2.0 移除了官方桥接功能，仅保留做市商兑换服务

#### 3.1 maker_swap - 创建做市商兑换

```rust
#[pallet::call_index(0)]
#[pallet::weight(T::WeightInfo::maker_swap())]
pub fn maker_swap(
    origin: OriginFor<T>,
    maker_id: u64,             // 做市商 ID
    cos_amount: BalanceOf<T>, // COS 数量
    usdt_address: Vec<u8>,     // USDT 接收地址
) -> DispatchResult
```

**功能：** 用户发起做市商兑换，锁定 COS，等待做市商发送 USDT

**超时机制：** `SwapTimeoutBlocks` 区块后自动退款

#### 3.2 submit_trc20_tx_hash - 提交 TRC20 交易哈希

```rust
#[pallet::call_index(1)]
#[pallet::weight(T::WeightInfo::submit_trc20_tx_hash())]
pub fn submit_trc20_tx_hash(
    origin: OriginFor<T>,
    swap_id: u64,
    trc20_tx_hash: Vec<u8>, // TRC20 交易哈希（USDT 转账证明）
) -> DispatchResult
```

**功能：** 做市商提交 TRC20 交易哈希，等待 OCW 验证

**防重放机制：** 记录已使用的交易哈希，防止同一笔交易被重复使用

#### 3.3 report_swap - 举报做市商

```rust
#[pallet::call_index(4)]
#[pallet::weight(T::WeightInfo::report_swap())]
pub fn report_swap(origin: OriginFor<T>, swap_id: u64) -> DispatchResult
```

**功能：** 用户举报做市商未发送 USDT，提交仲裁

---

## 📡 事件定义

### 1. pallet-maker 事件

```rust
pub enum Event<T: Config> {
    /// 押金已锁定 [maker_id, account, deposit]
    DepositLocked(u64, T::AccountId, BalanceOf<T>),

    /// 资料已提交 [maker_id]
    InfoSubmitted(u64),

    /// 做市商已激活 [maker_id, approved_by]
    MakerApproved(u64, T::AccountId),

    /// 做市商已驳回 [maker_id, rejected_by]
    MakerRejected(u64, T::AccountId),

    /// 申请已取消 [maker_id]
    MakerCancelled(u64),

    /// 提现请求已创建 [maker_id, amount]
    WithdrawalRequested(u64, BalanceOf<T>),

    /// 提现已执行 [maker_id, amount]
    WithdrawalExecuted(u64, BalanceOf<T>),

    /// 提现已取消 [maker_id]
    WithdrawalCancelled(u64),
}
```

### 2. pallet-otc-order 事件

```rust
pub enum Event<T: Config> {
    /// 订单已创建 [order_id, buyer, maker_id, cos_amount]
    OrderCreated(u64, T::AccountId, u64, BalanceOf<T>),

    /// 首购订单已创建 [order_id, buyer, maker_id, cos_amount]
    FirstPurchaseCreated(u64, T::AccountId, u64, BalanceOf<T>),

    /// 买家已标记付款 [order_id, tron_tx_hash]
    BuyerMarkedPaid(u64, Option<Vec<u8>>),

    /// COS 已释放 [order_id, buyer]
    CosReleased(u64, T::AccountId),

    /// 订单已取消 [order_id]
    OrderCancelled(u64),

    /// 订单已争议 [order_id, initiator]
    OrderDisputed(u64, T::AccountId),

    /// 订单已过期 [order_id]
    OrderExpired(u64),
}
```

### 3. pallet-swap 事件

```rust
pub enum Event<T: Config> {
    /// 做市商兑换已创建 [swap_id, user, maker_id, cos_amount, usdt_address]
    SwapCreated(u64, T::AccountId, u64, BalanceOf<T>, Vec<u8>),

    /// TRC20 交易哈希已提交 [swap_id, trc20_tx_hash]
    TxHashSubmitted(u64, Vec<u8>),

    /// 兑换已完成 [swap_id]
    SwapCompleted(u64),

    /// 用户已举报 [swap_id, user]
    SwapReported(u64, T::AccountId),

    /// 兑换已超时退款 [swap_id, user, cos_amount]
    SwapTimeout(u64, T::AccountId, BalanceOf<T>),
}
```

---

## ⚙️ 配置参数

### 1. pallet-maker 配置

```rust
impl pallet_maker::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MakerCredit = Credit;
    type GovernanceOrigin = EnsureTreasury;
    type Timestamp = Timestamp;
    type Pricing = Pricing;

    // 常量参数
    type MakerDepositAmount = ConstU128<1_000_000_000_000_000>; // 1000 COS
    type TargetDepositUsd = ConstU64<1_000_000_000>;            // 1000 USD
    type DepositReplenishThreshold = ConstU64<950_000_000>;     // 950 USD
    type DepositReplenishTarget = ConstU64<1_050_000_000>;      // 1050 USD
    type PriceCheckInterval = ConstU32<600>;                     // 每小时检查一次
    type AppealDeadline = ConstU32<100_800>;                     // 7天申诉期
    type MakerApplicationTimeout = ConstU32<100_800>;            // 7天申请超时
    type WithdrawalCooldown = ConstU32<100_800>;                 // 7天提现冷却
    type WeightInfo = ();
}
```

### 2. pallet-otc-order 配置

```rust
impl pallet_otc_order::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Timestamp = Timestamp;
    type Escrow = Escrow;
    type Credit = Credit;
    type MakerCredit = Credit;
    type Pricing = Pricing;
    type MakerPallet = Maker;

    // 常量参数
    type OrderTimeout = ConstU64<3_600_000>;                     // 1小时订单超时（毫秒）
    type EvidenceWindow = ConstU64<86_400_000>;                  // 24小时证据窗口（毫秒）
    type FirstPurchaseUsdValue = ConstU128<10_000_000>;          // 10 USD（已废弃）
    type FirstPurchaseUsdAmount = ConstU64<10_000_000>;          // 10 USD
    type MinFirstPurchaseCosAmount = ConstU128<1_000_000_000_000>; // 最小 1000 COS
    type MaxFirstPurchaseCosAmount = ConstU128<1_000_000_000_000_000>; // 最大 1M COS
    type MaxOrderUsdAmount = ConstU64<200_000_000>;              // 最大 200 USD
    type MinOrderUsdAmount = ConstU64<20_000_000>;               // 最小 20 USD
    type AmountValidationTolerance = ConstU16<100>;              // 1% 容差
    type MaxFirstPurchaseOrdersPerMaker = ConstU32<5>;           // 每个做市商最多 5 个首购订单
    type WeightInfo = ();
}
```

### 3. pallet-swap 配置

```rust
impl pallet_swap::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Escrow = Escrow;
    type Pricing = Pricing;
    type MakerPallet = Maker;
    type Credit = Credit;

    // 常量参数
    type SwapTimeoutBlocks = ConstU32<14_400>;                   // 1天超时（区块数）
    type TxHashTtlBlocks = ConstU32<{ 30 * DAYS }>;              // 30天 TTL 过期清理
    type MinSwapAmount = ConstU128<100_000_000_000>;             // 最小 100 COS
    type WeightInfo = ();
}
```

---

## 📱 前端调用示例

### 1. 做市商管理

#### 1.1 完整的做市商申请流程

```typescript
import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';

// 第一步：锁定押金
async function lockDeposit(api: ApiPromise, account: KeyringPair) {
  const tx = api.tx.maker.lockDeposit();
  await tx.signAndSend(account, ({ status, events }) => {
    if (status.isInBlock) {
      console.log('押金已锁定，区块哈希:', status.asInBlock.toString());

      // 解析事件获取 maker_id
      events.forEach(({ event }) => {
        if (api.events.maker.DepositLocked.is(event)) {
          const [makerId, account, deposit] = event.data;
          console.log('Maker ID:', makerId.toString());
          console.log('押金金额:', deposit.toString());
        }
      });
    }
  });
}

// 第二步：提交资料
async function submitInfo(api: ApiPromise, account: KeyringPair) {
  const tx = api.tx.maker.submitInfo(
    '张三',                                // real_name
    '110101199001011234',                 // id_card_number
    '1990-01-01',                         // birthday
    'TYASr5UV6HEcXatwdFQfmLVUqQQQMUxHLS', // tron_address
    'QmXXXpublicCID',                     // public_cid
    'QmXXXprivateCID',                    // private_cid
    2,                                    // direction (BuyAndSell)
    10,                                   // buy_premium_bps (0.1%)
    20,                                   // sell_premium_bps (0.2%)
    100_000_000_000,                      // min_amount (100 COS)
    'wechat_12345',                       // wechat_id
    JSON.stringify({ alipay: '13812345678' }), // payment_methods_json
  );

  await tx.signAndSend(account);
}

// 第三步：查询申请状态
async function queryMakerInfo(api: ApiPromise, makerId: number) {
  const makerInfo = await api.query.maker.makerApplications(makerId);

  if (makerInfo.isSome) {
    const data = makerInfo.unwrap();
    console.log('做市商信息:', {
      account: data.owner.toString(),
      status: data.status.toString(),
      direction: data.direction.toNumber(),
      tronAddress: data.tronAddress.toHuman(),
      buyPremium: data.buyPremiumBps.toNumber() / 100 + '%',
      sellPremium: data.sellPremiumBps.toNumber() / 100 + '%',
      maskedName: data.maskedFullName.toHuman(),
      maskedIdCard: data.maskedIdCard.toHuman(),
      wechatId: data.wechatId.toHuman(),
      isActive: data.status.isActive,
      servicePaused: data.servicePaused.isTrue,
    });
  }
}
```

#### 1.2 提现流程

```typescript
// 申请提现
async function requestWithdrawal(api: ApiPromise, account: KeyringPair, amount: string) {
  const tx = api.tx.maker.requestWithdrawal(amount);
  await tx.signAndSend(account);
}

// 等待 7 天冷却期后执行提现
async function executeWithdrawal(api: ApiPromise, account: KeyringPair) {
  const tx = api.tx.maker.executeWithdrawal();
  await tx.signAndSend(account);
}

// 查询提现请求
async function queryWithdrawalRequest(api: ApiPromise, makerId: number) {
  const request = await api.query.maker.withdrawalRequests(makerId);

  if (request.isSome) {
    const data = request.unwrap();
    console.log('提现请求:', {
      amount: data.amount.toString(),
      requestedAt: data.requestedAt.toNumber(),
      executableAt: data.executableAt.toNumber(),
      status: data.status.toString(),
    });
  }
}
```

---

### 2. OTC 订单管理

#### 2.1 创建首购订单

```typescript
import CryptoJS from 'crypto-js';

// 生成支付承诺哈希（买家本地加密）
function generatePaymentCommit(realName: string, idCard: string, phone: string): string {
  const data = `${realName}|${idCard}|${phone}`;
  return CryptoJS.SHA256(data).toString();
}

// 生成联系方式承诺哈希
function generateContactCommit(wechat: string, phone: string): string {
  const data = `${wechat}|${phone}`;
  return CryptoJS.SHA256(data).toString();
}

// 创建首购订单
async function createFirstPurchase(
  api: ApiPromise,
  account: KeyringPair,
  makerId: number,
) {
  // 买家本地生成承诺哈希
  const paymentCommit = generatePaymentCommit('李四', '110101199001011234', '13812345678');
  const contactCommit = generateContactCommit('wechat_12345', '13812345678');

  const tx = api.tx.otcOrder.createFirstPurchase(
    makerId,
    paymentCommit,
    contactCommit,
  );

  await tx.signAndSend(account, ({ status, events }) => {
    if (status.isInBlock) {
      events.forEach(({ event }) => {
        if (api.events.otcOrder.FirstPurchaseCreated.is(event)) {
          const [orderId, buyer, makerId, cosAmount] = event.data;
          console.log('首购订单创建成功:', {
            orderId: orderId.toString(),
            buyer: buyer.toString(),
            makerId: makerId.toString(),
            cosAmount: cosAmount.toString(),
          });
        }
      });
    }
  });
}
```

#### 2.2 创建普通订单

```typescript
async function createOrder(
  api: ApiPromise,
  account: KeyringPair,
  makerId: number,
  cosAmount: string,
) {
  const paymentCommit = generatePaymentCommit('李四', '110101199001011234', '13812345678');
  const contactCommit = generateContactCommit('wechat_12345', '13812345678');

  const tx = api.tx.otcOrder.createOrder(
    makerId,
    cosAmount,
    paymentCommit,
    contactCommit,
  );

  await tx.signAndSend(account);
}
```

#### 2.3 完整的订单支付流程

```typescript
// 买家标记已付款
async function markPaid(
  api: ApiPromise,
  account: KeyringPair,
  orderId: number,
  tronTxHash?: string,
) {
  const tx = api.tx.otcOrder.markPaid(orderId, tronTxHash || null);
  await tx.signAndSend(account);
}

// 做市商释放 COS
async function releaseCos(
  api: ApiPromise,
  makerAccount: KeyringPair,
  orderId: number,
) {
  const tx = api.tx.otcOrder.releaseCos(orderId);
  await tx.signAndSend(makerAccount);
}

// 查询订单详情
async function queryOrder(api: ApiPromise, orderId: number) {
  const order = await api.query.otcOrder.orders(orderId);
  if (order.isSome) {
    const data = order.unwrap();
    console.log('订单详情:', {
      orderId,
      makerId: data.makerId.toString(),
      maker: data.maker.toString(),
      taker: data.taker.toString(),
      price: data.price.toString(),
      qty: data.qty.toString(),
      amount: data.amount.toString(),
      state: data.state.toString(),
      createdAt: new Date(data.createdAt.toNumber()).toLocaleString(),
      expireAt: new Date(data.expireAt.toNumber()).toLocaleString(),
      makerTronAddress: data.makerTronAddress.toHuman(),
      isFirstPurchase: data.isFirstPurchase.isTrue,
    });
  }
}

// 查询买家订单列表
async function queryBuyerOrders(api: ApiPromise, buyer: string) {
  const orderIds = await api.query.otcOrder.buyerOrders(buyer);
  console.log('买家订单列表:', orderIds.map(id => id.toNumber()));

  // 查询每个订单的详情
  for (const orderId of orderIds) {
    await queryOrder(api, orderId.toNumber());
  }
}

// 查询做市商订单列表
async function queryMakerOrders(api: ApiPromise, makerId: number) {
  const orderIds = await api.query.otcOrder.makerOrders(makerId);
  console.log('做市商订单列表:', orderIds.map(id => id.toNumber()));
}
```

---

### 3. Swap 兑换管理

```typescript
// 用户发起做市商兑换
async function createSwap(
  api: ApiPromise,
  account: KeyringPair,
  makerId: number,
  cosAmount: string,
  usdtAddress: string,
) {
  const tx = api.tx.swap.createSwap(makerId, cosAmount, usdtAddress);

  await tx.signAndSend(account, ({ status, events }) => {
    if (status.isInBlock) {
      events.forEach(({ event }) => {
        if (api.events.swap.SwapCreated.is(event)) {
          const [swapId, user, makerId, cosAmount, usdtAddress] = event.data;
          console.log('做市商兑换创建成功:', {
            swapId: swapId.toString(),
            user: user.toString(),
            makerId: makerId.toString(),
            cosAmount: cosAmount.toString(),
            usdtAddress: usdtAddress.toHuman(),
          });
        }
      });
    }
  });
}

// 做市商提交 TRC20 交易哈希
async function submitTrc20TxHash(
  api: ApiPromise,
  makerAccount: KeyringPair,
  swapId: number,
  trc20TxHash: string,
) {
  const tx = api.tx.swap.submitTrc20TxHash(swapId, trc20TxHash);
  await tx.signAndSend(makerAccount);
}

// 用户举报做市商
async function reportSwap(
  api: ApiPromise,
  account: KeyringPair,
  swapId: number,
) {
  const tx = api.tx.swap.reportSwap(swapId);
  await tx.signAndSend(account);
}

// 查询兑换详情
async function querySwap(api: ApiPromise, swapId: number) {
  const swap = await api.query.swap.makerSwaps(swapId);
  if (swap.isSome) {
    const data = swap.unwrap();
    console.log('做市商兑换详情:', {
      swapId: data.swapId.toString(),
      makerId: data.makerId.toString(),
      user: data.user.toString(),
      cosAmount: data.cosAmount.toString(),
      usdtAmount: data.usdtAmount.toString(),
      usdtAddress: data.usdtAddress.toHuman(),
      status: data.status.toString(),
      trc20TxHash: data.trc20TxHash.isSome ? data.trc20TxHash.unwrap().toHuman() : null,
      createdAt: data.createdAt.toString(),
      timeoutAt: data.timeoutAt.toString(),
    });
  }
}
```

---

### 4. 聚合查询

```typescript
// 查询平台统计（前端自行聚合）
async function getPlatformStats(api: ApiPromise) {
  const totalMakers = await api.query.maker.nextMakerId();
  const totalOrders = await api.query.otcOrder.nextOrderId();
  const totalSwaps = await api.query.swap.nextSwapId();

  console.log('平台统计:', {
    totalMakers: totalMakers.toNumber(),
    totalOrders: totalOrders.toNumber(),
    totalSwaps: totalSwaps.toNumber(),
  });
}

// 查询活跃做市商列表
async function getActiveMakers(api: ApiPromise) {
  const nextMakerId = await api.query.maker.nextMakerId();
  const activeMakers = [];

  for (let i = 1; i < nextMakerId.toNumber(); i++) {
    const makerInfo = await api.query.maker.makerApplications(i);
    if (makerInfo.isSome) {
      const data = makerInfo.unwrap();
      if (data.status.isActive && !data.servicePaused.isTrue) {
        activeMakers.push({
          makerId: i,
          account: data.owner.toString(),
          direction: data.direction.toNumber(),
          buyPremium: data.buyPremiumBps.toNumber() / 100,
          sellPremium: data.sellPremiumBps.toNumber() / 100,
          minAmount: data.minAmount.toString(),
          usersServed: data.usersServed.toNumber(),
        });
      }
    }
  }

  console.log('活跃做市商列表:', activeMakers);
  return activeMakers;
}
```

---

### 5. 使用公共工具（前端实现）

```typescript
// 前端实现数据脱敏（参考 pallet-trading-common）
function maskName(fullName: string): string {
  const len = fullName.length;
  if (len === 0) return '';
  if (len === 1) return '×';
  if (len === 2) return '×' + fullName[1];
  if (len === 3) return fullName[0] + '×' + fullName[2];
  return fullName[0] + '×' + fullName[len - 1];
}

function maskIdCard(idCard: string): string {
  if (idCard.length < 8) return '*'.repeat(idCard.length);
  const front = idCard.slice(0, 4);
  const back = idCard.slice(-4);
  const middle = '*'.repeat(idCard.length - 8);
  return front + middle + back;
}

function maskBirthday(birthday: string): string {
  if (birthday.length >= 4) {
    return birthday.slice(0, 4) + '-xx-xx';
  }
  return '****-xx-xx';
}

// TRON 地址验证
function isValidTronAddress(address: string): boolean {
  if (address.length !== 34) return false;
  if (address[0] !== 'T') return false;

  const base58Chars = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
  for (const char of address) {
    if (!base58Chars.includes(char)) return false;
  }

  return true;
}

// 使用示例
console.log(maskName('张三'));        // '×三'
console.log(maskIdCard('110101199001011234')); // '1101**********1234'
console.log(maskBirthday('1990-01-01')); // '1990-xx-xx'
console.log(isValidTronAddress('TYASr5UV6HEcXatwdFQfmLVUqQQQMUxHLS')); // true
```

---

## 🔧 Runtime 集成

### 推荐方式：直接集成子模块

```rust
// runtime/src/lib.rs

// 1. 配置 pallet-maker
parameter_types! {
    pub const MakerDeposit: Balance = 1_000_000_000_000_000; // 1000 COS
    pub const MakerTimeout: BlockNumber = 100_800;           // 7天
    pub const WithdrawalCooldown: BlockNumber = 100_800;     // 7天
}

impl pallet_maker::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MakerCredit = Credit;
    type GovernanceOrigin = EnsureTreasury;
    type Timestamp = Timestamp;
    type Pricing = Pricing;
    type MakerDepositAmount = MakerDeposit;
    type TargetDepositUsd = ConstU64<1_000_000_000>;
    type DepositReplenishThreshold = ConstU64<950_000_000>;
    type DepositReplenishTarget = ConstU64<1_050_000_000>;
    type PriceCheckInterval = ConstU32<600>;
    type AppealDeadline = ConstU32<100_800>;
    type MakerApplicationTimeout = MakerTimeout;
    type WithdrawalCooldown = WithdrawalCooldown;
    type WeightInfo = ();
}

// 2. 配置 pallet-otc-order
impl pallet_otc_order::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Timestamp = Timestamp;
    type Escrow = Escrow;
    type Credit = Credit;
    type MakerCredit = Credit;
    type Pricing = Pricing;
    type MakerPallet = Maker;
    type OrderTimeout = ConstU64<3_600_000>;
    type EvidenceWindow = ConstU64<86_400_000>;
    type FirstPurchaseUsdValue = ConstU128<10_000_000>;
    type FirstPurchaseUsdAmount = ConstU64<10_000_000>;
    type MinFirstPurchaseCosAmount = ConstU128<1_000_000_000_000>;
    type MaxFirstPurchaseCosAmount = ConstU128<1_000_000_000_000_000>;
    type MaxOrderUsdAmount = ConstU64<200_000_000>;
    type MinOrderUsdAmount = ConstU64<20_000_000>;
    type AmountValidationTolerance = ConstU16<100>;
    type MaxFirstPurchaseOrdersPerMaker = ConstU32<5>;
    type WeightInfo = ();
}

// 3. 配置 pallet-swap
impl pallet_swap::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Escrow = Escrow;
    type Pricing = Pricing;
    type MakerPallet = Maker;
    type Credit = Credit;
    type SwapTimeoutBlocks = ConstU32<14_400>;
    type TxHashTtlBlocks = ConstU32<{ 30 * DAYS }>;
    type MinSwapAmount = ConstU128<100_000_000_000>;
    type WeightInfo = ();
}

// 4. 在 construct_runtime! 中添加
construct_runtime! {
    pub struct Runtime {
        // System
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,

        // Trading modules
        Maker: pallet_maker,
        OtcOrder: pallet_otc_order,
        Swap: pallet_swap,

        // Dependencies
        Escrow: pallet_escrow,
        Credit: pallet_credit,
        Pricing: pallet_pricing,
        // ... 其他模块
    }
}
```

---

## 📊 数据结构详解

### 1. MakerApplication（做市商申请记录）

```rust
pub struct MakerApplication<T: Config> {
    pub owner: T::AccountId,              // 所有者账户
    pub deposit: BalanceOf<T>,            // 押金金额
    pub status: ApplicationStatus,        // 申请状态
    pub direction: Direction,             // 业务方向（Buy/Sell/BuyAndSell）
    pub tron_address: TronAddress,        // TRON 地址（34字节）
    pub public_cid: Cid,                  // 公开资料 IPFS CID
    pub private_cid: Cid,                 // 私密资料 IPFS CID（加密）
    pub buy_premium_bps: i16,             // Buy 溢价（基点，-500~500）
    pub sell_premium_bps: i16,            // Sell 溢价（基点，-500~500）
    pub min_amount: BalanceOf<T>,         // 最小交易金额
    pub created_at: u32,                  // 创建时间（Unix秒）
    pub info_deadline: u32,               // 资料提交截止时间
    pub review_deadline: u32,             // 审核截止时间
    pub service_paused: bool,             // 服务暂停状态
    pub users_served: u32,                // 已服务用户数量
    pub masked_full_name: BoundedVec<u8, ConstU32<64>>,     // 脱敏姓名
    pub masked_id_card: BoundedVec<u8, ConstU32<32>>,       // 脱敏身份证
    pub masked_birthday: BoundedVec<u8, ConstU32<16>>,      // 脱敏生日
    pub masked_payment_info: BoundedVec<u8, ConstU32<512>>, // 脱敏收款方式
    pub wechat_id: BoundedVec<u8, ConstU32<64>>,            // 微信号
    pub target_deposit_usd: u64,          // 押金目标 USD 价值（1000 USD）
    pub last_price_check: BlockNumberFor<T>, // 上次价格检查时间
    pub deposit_warning: bool,            // 押金不足警告
}
```

### 2. Order（OTC 订单）

```rust
pub struct Order<T: Config> {
    pub maker_id: u64,                    // 做市商 ID
    pub maker: T::AccountId,              // 做市商账户
    pub taker: T::AccountId,              // 买家账户
    pub price: BalanceOf<T>,              // 单价（USDT/COS，精度 10^6）
    pub qty: BalanceOf<T>,                // 数量（COS 数量）
    pub amount: BalanceOf<T>,             // 总金额（USDT 金额）
    pub created_at: MomentOf,             // 创建时间（毫秒）
    pub expire_at: MomentOf,              // 超时时间（毫秒）
    pub evidence_until: MomentOf,         // 证据窗口截止时间（毫秒）
    pub maker_tron_address: TronAddress,  // 做市商 TRON 收款地址
    pub payment_commit: H256,             // 支付承诺哈希
    pub contact_commit: H256,             // 联系方式承诺哈希
    pub state: OrderState,                // 订单状态
    pub completed_at: Option<MomentOf>,   // 订单完成时间
    pub is_first_purchase: bool,          // 是否为首购订单
}
```

### 3. MakerSwapRecord（做市商兑换）

```rust
pub struct MakerSwapRecord<T: Config> {
    pub swap_id: u64,                     // 兑换 ID
    pub maker_id: u64,                    // 做市商 ID
    pub maker: T::AccountId,              // 做市商账户
    pub user: T::AccountId,               // 用户账户
    pub cos_amount: BalanceOf<T>,         // COS 数量
    pub usdt_amount: u64,                 // USDT 金额（精度 10^6）
    pub usdt_address: TronAddress,        // USDT 接收地址
    pub created_at: BlockNumberFor<T>,    // 创建时间
    pub timeout_at: BlockNumberFor<T>,    // 超时时间
    pub trc20_tx_hash: Option<BoundedVec<u8, ConstU32<128>>>, // TRC20 交易哈希
    pub completed_at: Option<BlockNumberFor<T>>,              // 完成时间
    pub evidence_cid: Option<BoundedVec<u8, ConstU32<256>>>,  // 证据 CID
    pub status: SwapStatus,               // 兑换状态
    pub price_usdt: u64,                  // 兑换价格（精度 10^6）
}
```

---

## 🔗 集成说明

### 如何聚合其他模块

本 Pallet 作为统一接口层，通过以下方式聚合子模块：

#### 1. 依赖声明（Cargo.toml）

```toml
[dependencies]
# 子模块依赖
pallet-maker = { path = "../maker", default-features = false }
pallet-otc-order = { path = "../otc-order", default-features = false }
pallet-swap = { path = "../swap", default-features = false }
pallet-trading-common = { path = "../trading-common", default-features = false }
```

#### 2. 重新导出（lib.rs）

```rust
// 直接导出子模块
pub use pallet_maker;
pub use pallet_otc_order;
pub use pallet_swap;
pub use pallet_trading_common;

// 聚合类型导出
pub mod maker_types {
    pub use pallet_maker::{MakerApplication, ApplicationStatus, Direction};
}
```

#### 3. 聚合查询 API

```rust
pub struct TradingApi;

impl TradingApi {
    pub fn get_platform_stats<T>() -> PlatformStats
    where
        T: pallet_maker::Config + pallet_otc_order::Config + pallet_swap::Config,
    {
        PlatformStats {
            total_makers: pallet_maker::NextMakerId::<T>::get(),
            total_orders: pallet_otc_order::NextOrderId::<T>::get(),
            total_swaps: pallet_swap::NextSwapId::<T>::get(),
        }
    }
}
```

---

## 🛡️ 安全考虑

### 1. 模块隔离

- ✅ **独立存储**：每个子模块有独立的存储空间，不会相互污染
- ✅ **独立权限**：每个子模块有独立的权限控制（GovernanceOrigin）
- ✅ **错误隔离**：一个模块的错误不影响其他模块

### 2. 接口设计

- ✅ **类型安全**：使用 Rust 的类型系统确保接口正确性
- ✅ **Trait 约束**：通过 Trait 定义清晰的模块接口
  - `MakerInterface`: Maker Pallet 提供的接口
  - `PricingProvider`: 定价服务接口
  - `Escrow`: 托管服务接口
  - `MakerCreditInterface`: 信用记录接口
- ✅ **版本兼容**：支持独立升级子模块

### 3. 测试策略

- ✅ **独立测试**：每个子模块有独立的单元测试（`tests.rs`）
- ✅ **集成测试**：统一接口层提供集成测试
- ✅ **Mock 接口**：便于测试子模块之间的交互（`mock.rs`）

### 4. 数据安全

#### 4.1 数据脱敏

```rust
// pallet-trading-common 提供脱敏函数
pub fn mask_name(full_name: &str) -> Vec<u8>;
pub fn mask_id_card(id_card: &str) -> Vec<u8>;
pub fn mask_birthday(birthday: &str) -> Vec<u8>;
```

**脱敏规则：**
- 姓名：`"张三" -> "×三"`，`"李四五" -> "李×五"`
- 身份证：`"110101199001011234" -> "1101**********1234"`
- 生日：`"1990-01-01" -> "1990-xx-xx"`

#### 4.2 数据验证

```rust
// TRON 地址验证
pub fn is_valid_tron_address(address: &[u8]) -> bool;
// 规则：长度 34，开头 'T'，Base58 编码
```

#### 4.3 防重放攻击

```rust
// pallet-swap 记录已使用的 TRC20 交易哈希
pub type UsedTronTxHashes<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    BoundedVec<u8, ConstU32<128>>,  // TRC20 tx hash
    (),
>;
```

---

## 💡 最佳实践

### 1. Runtime 集成

**推荐**：直接集成子模块，而不是集成 `pallet-trading`

```rust
// ✅ 推荐
construct_runtime! {
    pub struct Runtime {
        Maker: pallet_maker,
        OtcOrder: pallet_otc_order,
        Swap: pallet_swap,
    }
}

// ❌ 不推荐（pallet-trading 只是接口层，无存储）
construct_runtime! {
    pub struct Runtime {
        Trading: pallet_trading,
    }
}
```

### 2. 前端调用

**推荐**：直接调用子模块 API

```typescript
// ✅ 推荐
await api.tx.maker.lockDeposit().signAndSend(account);
await api.tx.otcOrder.createOrder(...).signAndSend(account);
await api.tx.swap.createSwap(...).signAndSend(account);

// ❌ 不推荐（无此 API）
await api.tx.trading.lockDeposit().signAndSend(account);
```

### 3. 类型导入

**推荐**：使用 `pallet-trading` 的类型导出

```typescript
// ✅ 推荐（统一导入）
import { maker_types, otc_types, swap_types } from 'pallet-trading';

// ✅ 也可以直接导入子模块
import { MakerApplication } from 'pallet-maker';
import { Order } from 'pallet-otc-order';
```

### 4. 错误处理

```typescript
// 推荐的错误处理方式
try {
  await api.tx.otcOrder.createOrder(...).signAndSend(account, ({ status, events }) => {
    if (status.isInBlock) {
      events.forEach(({ event }) => {
        if (api.events.system.ExtrinsicFailed.is(event)) {
          const [dispatchError] = event.data;
          let errorMessage = 'Unknown error';

          if (dispatchError.isModule) {
            const decoded = api.registry.findMetaError(dispatchError.asModule);
            errorMessage = `${decoded.section}.${decoded.name}: ${decoded.docs}`;
          }

          console.error('交易失败:', errorMessage);
        } else if (api.events.otcOrder.OrderCreated.is(event)) {
          console.log('订单创建成功');
        }
      });
    }
  });
} catch (error) {
  console.error('交易提交失败:', error);
}
```

---

## 📚 相关模块

### 核心业务模块

- **[pallet-maker](../maker/README.md)**: 做市商管理
- **[pallet-otc-order](../otc/README.md)**: OTC 订单管理
- **[pallet-swap](../swap/README.md)**: COS → USDT 做市商兑换
- **[pallet-trading-common](../common/README.md)**: 公共工具库

### 依赖模块

- **[pallet-escrow](../escrow/README.md)**: 托管服务
- **[pallet-credit](../credit/README.md)**: 信用管理
- **[pallet-pricing](../pricing/README.md)**: 动态定价与市场统计

### 治理模块

- **pallet-democracy**: 治理投票
- **pallet-treasury**: 国库管理

---

## 🚀 版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| v0.1.0 | 2025-11-03 | 重构为统一接口层，拆分为 4 个子模块（maker/otc/swap/common） |
| v0.2.0 | 2026-01-18 | pallet-swap: 移除官方桥接功能，仅保留做市商兑换 |
| v0.3.0 | 2026-01-18 | pallet-swap: 重命名 bridge → swap |

---

## 🔍 完整使用示例

### 做市商完整流程

```typescript
import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';

async function makerCompleteFlow() {
  // 1. 连接节点
  const provider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({ provider });

  // 2. 创建账户
  const keyring = new Keyring({ type: 'sr25519' });
  const makerAccount = keyring.addFromUri('//Alice');

  // 3. 锁定押金
  console.log('Step 1: 锁定押金...');
  await api.tx.maker.lockDeposit()
    .signAndSend(makerAccount, ({ status, events }) => {
      if (status.isInBlock) {
        events.forEach(({ event }) => {
          if (api.events.maker.DepositLocked.is(event)) {
            const [makerId] = event.data;
            console.log('押金锁定成功，Maker ID:', makerId.toString());
          }
        });
      }
    });

  // 4. 提交资料
  console.log('Step 2: 提交资料...');
  await api.tx.maker.submitInfo(
    '张三',
    '110101199001011234',
    '1990-01-01',
    'TYASr5UV6HEcXatwdFQfmLVUqQQQMUxHLS',
    'QmPublicCID',
    'QmPrivateCID',
    2, // BuyAndSell
    10, 20, // premium
    100_000_000_000, // min_amount
    'wechat_12345',
    JSON.stringify({ alipay: '13812345678' }),
    null, null,
  ).signAndSend(makerAccount);

  console.log('资料提交成功，等待治理审核...');

  // 5. 查询状态
  const makerId = 1; // 从事件中获取
  const makerInfo = await api.query.maker.makerApplications(makerId);
  console.log('做市商状态:', makerInfo.unwrap().status.toString());
}

makerCompleteFlow().catch(console.error);
```

### 买家完整流程（首购 → 普通订单）

```typescript
async function buyerCompleteFlow() {
  const provider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({ provider });

  const keyring = new Keyring({ type: 'sr25519' });
  const buyerAccount = keyring.addFromUri('//Bob');

  // 1. 创建首购订单（10 USD）
  console.log('Step 1: 创建首购订单...');
  const paymentCommit = '0x' + CryptoJS.SHA256('李四|110101199001011234|13812345678').toString();
  const contactCommit = '0x' + CryptoJS.SHA256('wechat_12345|13812345678').toString();

  let orderId: number;
  await api.tx.otcOrder.createFirstPurchase(1, paymentCommit, contactCommit)
    .signAndSend(buyerAccount, ({ status, events }) => {
      if (status.isInBlock) {
        events.forEach(({ event }) => {
          if (api.events.otcOrder.FirstPurchaseCreated.is(event)) {
            const [id] = event.data;
            orderId = id.toNumber();
            console.log('首购订单创建成功，Order ID:', orderId);
          }
        });
      }
    });

  // 2. 买家付款后标记已付款
  console.log('Step 2: 标记已付款...');
  await api.tx.otcOrder.markPaid(orderId, null)
    .signAndSend(buyerAccount);

  // 3. 做市商释放 COS
  console.log('Step 3: 等待做市商释放 COS...');
  // （做市商账户调用 releaseDust）

  // 4. 首购完成后，创建普通订单（20-200 USD）
  console.log('Step 4: 创建普通订单...');
  await api.tx.otcOrder.createOrder(
    1, // maker_id
    '50000000000000', // 50 COS
    paymentCommit,
    contactCommit,
  ).signAndSend(buyerAccount);

  console.log('普通订单创建成功');
}

buyerCompleteFlow().catch(console.error);
```

---

## 📞 技术支持

如有问题，请参考：

1. **子模块文档**：查看各子模块的 README.md
2. **代码示例**：参考本文档的使用示例
3. **源码**：阅读 `pallets/trading/src/lib.rs` 和子模块源码
4. **测试用例**：参考 `pallets/*/src/tests.rs` 中的测试用例

---

**License**: Unlicense

**Repository**: https://github.com/memoio/memopark
