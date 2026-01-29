# Pallet Escrow（资金托管模块）

## 📋 模块概述

`pallet-escrow` 是 Cosmos 区块链的**资金托管系统**，提供安全的资金锁定、释放、退款和分账功能。本模块作为交易、仲裁等业务模块的底层基础设施，确保资金在交易过程中的安全托管。

### 核心特性

- ✅ **资金锁定**：从付款人账户转入托管账户并记录
- ✅ **全额释放/退款**：一次性将托管资金转给指定账户
- ✅ **分账释放**：支持多方分账，按金额或比例分配
- ✅ **争议状态管理**：争议期间冻结资金操作
- ✅ **仲裁决议执行**：支持全额释放、全额退款、按比例分配
- ✅ **到期自动处理**：支持设置到期时间，自动执行释放/退款
- ✅ **全局暂停**：应急情况下可暂停所有操作
- ✅ **幂等锁定**：支持 nonce 机制防止重放攻击

### 设计理念

1. **安全优先**：所有外部调用需授权，内部 trait 接口供其他 pallet 调用
2. **原子操作**：所有资金操作为原子事务，失败自动回滚
3. **状态一致**：托管状态与实际余额保持一致
4. **可扩展**：通过 `ExpiryPolicy` trait 支持自定义到期策略

---

## 🔑 核心概念

### 托管状态 (LockState)

| 状态值 | 名称 | 说明 |
|--------|------|------|
| 0 | Locked | 资金已锁定，可正常操作 |
| 1 | Disputed | 争议中，仅允许仲裁决议操作 |
| 2 | Resolved | 已解决（仲裁完成） |
| 3 | Closed | 已关闭（资金已全部转出） |

### 托管账户

托管资金存放在由 `PalletId` 派生的模块账户中：

```rust
fn account() -> T::AccountId {
    T::EscrowPalletId::get().into_account_truncating()
}
```

---

## 📡 Trait 接口

### `Escrow<AccountId, Balance>`

供其他 Pallet 内部调用的托管接口：

```rust
pub trait Escrow<AccountId, Balance> {
    /// 从付款人转入托管并记录
    fn lock_from(payer: &AccountId, id: u64, amount: Balance) -> DispatchResult;
    
    /// 从托管转出部分金额到指定账户
    fn transfer_from_escrow(id: u64, to: &AccountId, amount: Balance) -> DispatchResult;
    
    /// 将托管全部释放给收款人
    fn release_all(id: u64, to: &AccountId) -> DispatchResult;
    
    /// 将托管全部退款给付款人
    fn refund_all(id: u64, to: &AccountId) -> DispatchResult;
    
    /// 查询当前托管余额
    fn amount_of(id: u64) -> Balance;
    
    /// 获取托管账户地址
    fn escrow_account() -> AccountId;
    
    /// 按比例分账
    fn split_partial(
        id: u64,
        release_to: &AccountId,
        refund_to: &AccountId,
        bps: u16,
    ) -> DispatchResult;
}
```

### `ExpiryPolicy<AccountId, BlockNumber>`

到期处理策略接口（由 runtime 实现）：

```rust
pub trait ExpiryPolicy<AccountId, BlockNumber> {
    /// 返回到期应执行的动作
    fn on_expire(id: u64) -> Result<ExpiryAction<AccountId>, DispatchError>;
    
    /// 返回当前块号
    fn now() -> BlockNumber;
}

pub enum ExpiryAction<AccountId> {
    ReleaseAll(AccountId),  // 释放给指定账户
    RefundAll(AccountId),   // 退款给指定账户
    Noop,                   // 不执行任何操作
}
```

---

## 📝 Extrinsics

### 1. `lock` - 锁定资金

**调用方**：AuthorizedOrigin | Root

**功能**：从付款人账户转入托管账户并记录。

```rust
pub fn lock(
    origin: OriginFor<T>,
    id: u64,              // 托管 ID（订单ID/交易ID）
    payer: T::AccountId,  // 付款人账户
    amount: BalanceOf<T>, // 锁定金额
) -> DispatchResult
```

**示例**：
```rust
// 锁定 100 COS 到托管 ID 12345
Escrow::lock(RuntimeOrigin::root(), 12345, alice, 100 * UNIT)?;
```

---

### 2. `lock_with_nonce` - 幂等锁定

**调用方**：AuthorizedOrigin | Root

**功能**：带 nonce 的幂等锁定，防止重放攻击。

```rust
pub fn lock_with_nonce(
    origin: OriginFor<T>,
    id: u64,
    payer: T::AccountId,
    amount: BalanceOf<T>,
    nonce: u64,           // 必须严格递增
) -> DispatchResult
```

**幂等性**：相同 id 下 nonce 必须严格递增，否则忽略（不报错）。

---

### 3. `release` - 释放资金

**调用方**：AuthorizedOrigin | Root

**功能**：将托管资金全额释放给指定账户。

```rust
pub fn release(
    origin: OriginFor<T>,
    id: u64,
    to: T::AccountId,     // 收款人
) -> DispatchResult
```

**限制**：
- 暂停时拒绝
- 争议状态下拒绝

---

### 4. `refund` - 退款

**调用方**：AuthorizedOrigin | Root

**功能**：将托管资金全额退回给指定账户。

```rust
pub fn refund(
    origin: OriginFor<T>,
    id: u64,
    to: T::AccountId,     // 退款接收人
) -> DispatchResult
```

---

### 5. `release_split` - 分账释放

**调用方**：AuthorizedOrigin | Root

**功能**：将托管资金按指定金额分配给多个账户。

```rust
pub fn release_split(
    origin: OriginFor<T>,
    id: u64,
    entries: Vec<(T::AccountId, BalanceOf<T>)>,  // (收款人, 金额) 列表
) -> DispatchResult
```

**示例**：
```rust
// 将托管资金分配：Alice 70 COS, Bob 30 COS
let entries = vec![
    (alice, 70 * UNIT),
    (bob, 30 * UNIT),
];
Escrow::release_split(RuntimeOrigin::root(), 12345, entries)?;
```

---

### 6. `dispute` - 进入争议

**调用方**：AuthorizedOrigin | Root

**功能**：将托管标记为争议状态，冻结普通释放/退款操作。

```rust
pub fn dispute(
    origin: OriginFor<T>,
    id: u64,
    reason: u16,          // 争议原因代码
) -> DispatchResult
```

---

### 7. `apply_decision_release_all` - 仲裁决议：全额释放

**调用方**：AuthorizedOrigin | Root

**功能**：仲裁决议执行，将托管资金全额释放给指定账户。

```rust
pub fn apply_decision_release_all(
    origin: OriginFor<T>,
    id: u64,
    to: T::AccountId,
) -> DispatchResult
```

---

### 8. `apply_decision_refund_all` - 仲裁决议：全额退款

**调用方**：AuthorizedOrigin | Root

**功能**：仲裁决议执行，将托管资金全额退回给指定账户。

```rust
pub fn apply_decision_refund_all(
    origin: OriginFor<T>,
    id: u64,
    to: T::AccountId,
) -> DispatchResult
```

---

### 9. `apply_decision_partial_bps` - 仲裁决议：按比例分配

**调用方**：AuthorizedOrigin | Root

**功能**：仲裁决议执行，按 bps 比例分配资金。

```rust
pub fn apply_decision_partial_bps(
    origin: OriginFor<T>,
    id: u64,
    release_to: T::AccountId,  // 释放接收人
    refund_to: T::AccountId,   // 退款接收人
    bps: u16,                  // 释放比例（0-10000）
) -> DispatchResult
```

**比例说明**：
- `bps = 7000`：release_to 获得 70%，refund_to 获得 30%
- `bps = 10000`：release_to 获得 100%
- `bps = 0`：refund_to 获得 100%

---

### 10. `set_pause` - 设置全局暂停

**调用方**：AdminOrigin

**功能**：应急情况下暂停所有变更性操作。

```rust
pub fn set_pause(
    origin: OriginFor<T>,
    paused: bool,
) -> DispatchResult
```

---

### 11. `schedule_expiry` - 安排到期处理

**调用方**：AuthorizedOrigin | Root

**功能**：设置托管的到期时间，到期后自动执行策略。

```rust
pub fn schedule_expiry(
    origin: OriginFor<T>,
    id: u64,
    at: BlockNumberFor<T>,  // 到期区块号
) -> DispatchResult
```

---

### 12. `cancel_expiry` - 取消到期处理

**调用方**：AuthorizedOrigin | Root

**功能**：取消已设置的到期处理。

```rust
pub fn cancel_expiry(
    origin: OriginFor<T>,
    id: u64,
) -> DispatchResult
```

---

## 🗄️ 存储项

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `Locked` | `StorageMap<u64, Balance>` | 托管余额：id → 锁定金额 |
| `LockStateOf` | `StorageMap<u64, u8>` | 托管状态：id → 状态码 |
| `LockNonces` | `StorageMap<u64, u64>` | 幂等 nonce：id → 最新 nonce |
| `Paused` | `StorageValue<bool>` | 全局暂停开关 |
| `ExpiryOf` | `StorageMap<u64, BlockNumber>` | 到期时间：id → 到期区块 |
| `ExpiringAt` | `StorageMap<BlockNumber, Vec<u64>>` | 到期索引：区块 → id 列表 |

---

## 📡 事件

```rust
pub enum Event<T: Config> {
    /// 资金已锁定
    Locked { id: u64, amount: BalanceOf<T> },
    
    /// 部分转出
    Transfered { id: u64, to: T::AccountId, amount: BalanceOf<T>, remaining: BalanceOf<T> },
    
    /// 全额释放
    Released { id: u64, to: T::AccountId, amount: BalanceOf<T> },
    
    /// 全额退款
    Refunded { id: u64, to: T::AccountId, amount: BalanceOf<T> },
    
    /// 进入争议
    Disputed { id: u64, reason: u16 },
    
    /// 仲裁决议已执行
    DecisionApplied { id: u64, decision: u8 },
    
    /// 到期已安排
    ExpiryScheduled { id: u64, at: BlockNumberFor<T> },
    
    /// 到期已处理
    Expired { id: u64, action: u8 },
}
```

---

## ❌ 错误定义

```rust
pub enum Error<T> {
    /// 余额不足
    Insufficient,
    
    /// 托管记录不存在
    NoLock,
    
    /// 全局暂停中
    Paused,
    
    /// 订单处于争议状态
    InDispute,
    
    /// 到期索引已满
    MaxExpiryReached,
    
    /// 无效的 bps 参数（必须 0-10000）
    InvalidBps,
}
```

---

## ⚙️ 配置参数

### Runtime 配置示例

```rust
parameter_types! {
    pub const EscrowPalletId: PalletId = PalletId(*b"py/escro");
}

impl pallet_escrow::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type EscrowPalletId = EscrowPalletId;
    type AuthorizedOrigin = EnsureSigned<AccountId>;
    type AdminOrigin = EnsureRoot<AccountId>;
    type MaxExpiringPerBlock = ConstU32<100>;
    type ExpiryPolicy = DefaultExpiryPolicy;
}
```

### 到期策略实现示例

```rust
pub struct DefaultExpiryPolicy;

impl ExpiryPolicy<AccountId, BlockNumber> for DefaultExpiryPolicy {
    fn on_expire(id: u64) -> Result<ExpiryAction<AccountId>, DispatchError> {
        // 根据业务逻辑决定到期动作
        // 例如：查询订单状态，决定释放还是退款
        Ok(ExpiryAction::Noop)
    }

    fn now() -> BlockNumber {
        System::block_number()
    }
}
```

---

## 💻 使用示例

### 示例 1：OTC 交易托管流程

```rust
// 1. 买家下单，锁定做市商 COS 到托管
let order_id = 12345u64;
let amount = 1000 * UNIT;
<Escrow as EscrowTrait>::lock_from(&maker, order_id, amount)?;

// 2. 买家付款后，释放给买家
<Escrow as EscrowTrait>::release_all(order_id, &buyer)?;

// 或者：订单超时，退回给做市商
<Escrow as EscrowTrait>::refund_all(order_id, &maker)?;
```

### 示例 2：争议处理流程

```rust
// 1. 发起争议
Escrow::dispute(RuntimeOrigin::root(), order_id, 1)?;

// 2. 仲裁决议：买家胜诉，全额退款
Escrow::apply_decision_refund_all(RuntimeOrigin::root(), order_id, buyer)?;

// 或者：部分胜诉，70% 给卖家，30% 给买家
Escrow::apply_decision_partial_bps(
    RuntimeOrigin::root(),
    order_id,
    seller,  // release_to
    buyer,   // refund_to
    7000,    // 70%
)?;
```

### 示例 3：设置到期自动处理

```rust
// 设置订单 1 小时后到期
let expiry_block = System::block_number() + 600; // 假设 6 秒/块
Escrow::schedule_expiry(RuntimeOrigin::root(), order_id, expiry_block)?;

// 到期后，on_initialize 会自动调用 ExpiryPolicy::on_expire
// 根据策略执行 ReleaseAll / RefundAll / Noop
```

---

## 🔗 集成说明

### 与 pallet-trading-otc 集成

OTC 模块通过 trait 接口调用托管功能：

```rust
impl pallet_trading_otc::Config for Runtime {
    type Escrow = pallet_escrow::Pallet<Runtime>;
    // ...
}
```

### 与 pallet-arbitration 集成

仲裁模块通过 trait 接口执行裁决：

```rust
// 仲裁裁决后调用
match decision {
    Decision::Release => Escrow::release_all(id, &seller)?,
    Decision::Refund => Escrow::refund_all(id, &buyer)?,
    Decision::Partial(bps) => Escrow::split_partial(id, &seller, &buyer, bps)?,
}
```

---

## 🔒 安全考虑

### 1. 权限控制

- 外部 extrinsic 仅允许 `AuthorizedOrigin` 或 `Root` 调用
- 内部 trait 接口供其他 pallet 调用，不对外暴露

### 2. 争议保护

- 争议状态下，普通 `release` / `refund` 被拒绝
- 仅 `apply_decision_*` 系列函数可操作争议中的托管

### 3. 全局暂停

- 应急情况下可通过 `set_pause` 暂停所有变更操作
- 仅 `AdminOrigin` 可设置暂停状态

### 4. 幂等性

- `lock_with_nonce` 支持幂等锁定，防止重放攻击
- 相同 nonce 的重复调用会被忽略

### 5. 原子性

- 所有资金操作为原子事务
- 任意步骤失败会回滚整个操作

---

## 📚 相关文档

- [pallet-arbitration README](../arbitration/README.md) - 仲裁系统文档
- [pallet-trading-otc README](../trading/otc/README.md) - OTC 交易文档
- [DEPOSIT_ANALYSIS.md](../../docs/DEPOSIT_ANALYSIS.md) - 押金机制分析

---

## 📄 许可证

MIT-0

---

**最后更新**：2026-01-22  
**版本**：v1.0  
**维护者**：Cosmos Team
