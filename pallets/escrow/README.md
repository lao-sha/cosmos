# Pallet Escrow（通用托管服务模块）

## 📋 模块概述

`pallet-escrow` 是 Stardust 区块链的 **通用托管服务模块**，提供资金锁定、分账释放、退款、争议处理、自动到期等完整的托管生命周期管理。本模块被 OTC 交易、Bridge 兑换、仲裁系统等多个 pallet 依赖，是整个 Stardust 生态系统中资金安全管理的核心基础设施。

### 核心特性

- ✅ **完整托管生命周期**：锁定 → 释放/退款 → 关闭
- ✅ **多次分账支持**：单笔托管可分多次转出，直至全部释放（支持多账户分账释放）
- ✅ **争议处理机制**：支持争议状态、仲裁决议执行（全额/部分按比例）
- ✅ **自动到期处理**：on_initialize 自动处理到期订单，可配置策略
- ✅ **幂等锁定**：支持 nonce 防重放攻击
- ✅ **全局暂停开关**：应急止血机制，保护资金安全
- ✅ **状态机管理**：Locked/Disputed/Resolved/Closed 四状态流转
- ✅ **H-1性能优化**：ExpiringAt 索引提升 on_initialize 性能 O(N) → O(1)

### 设计理念

1. **安全第一**：所有资金操作必须经过授权，外部 extrinsic 仅限 AuthorizedOrigin | Root 调用
2. **低耦合**：提供 `Escrow<AccountId, Balance>` trait 供其他 pallet 内部调用
3. **可扩展**：支持自定义到期策略（ExpiryPolicy trait）
4. **防御性设计**：全局暂停、幂等锁定、状态机管理等多层保护机制

---

## 🔑 核心功能

### 1. 资金锁定（Lock）

#### 1.1 lock（标准锁定）

**调用方**：AuthorizedOrigin | Root

**功能**：从付款人账户转账到托管账户，并记录到 `Locked` 映射。

**流程**：
1. 验证权限（AuthorizedOrigin | Root）
2. 检查全局暂停状态（暂停时拒绝）
3. 从 payer 转账到托管账户
4. 累加到 `Locked[id]`
5. 设置状态为 Locked (0)
6. 发出 `Locked` 事件

**安全要求**：
- 必须确保付款人余额充足（不足返回 `Error::Insufficient`）
- 仅授权 pallet 可调用（避免冒用 payer 盗划资金）
- 支持同一 id 多次锁定（累加）

**Rust 示例**：

```rust
// 从其他 pallet 内部调用（通过 Escrow trait）
use pallet_escrow::{Escrow as EscrowTrait};

// 锁定 1000 DUST 到订单 #100
let payer = origin.clone();
let order_id = 100u64;
let amount = 1000 * 10u128.pow(12); // 1000 DUST (12位小数)

T::Escrow::lock_from(&payer, order_id, amount)?;
```

**外部 extrinsic 调用**：

```rust
// 仅限 AuthorizedOrigin | Root
#[pallet::weight(10_000)]
pub fn lock(
    origin: OriginFor<T>,
    id: u64,
    payer: T::AccountId,
    amount: BalanceOf<T>,
) -> DispatchResult
```

#### 1.2 lock_with_nonce（幂等锁定）

**调用方**：AuthorizedOrigin | Root

**功能**：带 nonce 的幂等锁定，相同 id 下 nonce 必须严格递增，否则忽略（防止重放攻击）。

**幂等机制**：
- 记录每个 id 的最新 nonce（`LockNonces<T>`）
- 若新 nonce <= 已记录的 nonce，直接返回 Ok（忽略）
- 否则，更新 nonce 并执行正常锁定流程

**用途**：
- 防止重放攻击
- 确保同一笔订单不会被重复锁定
- 适用于网络不稳定或多节点环境

**Rust 示例**：

```rust
// 幂等锁定（带 nonce）
let payer = origin.clone();
let order_id = 100u64;
let amount = 1000 * 10u128.pow(12);
let nonce = current_nonce + 1; // 必须递增

// 调用外部 extrinsic
let call = Call::lock_with_nonce {
    id: order_id,
    payer: payer.clone(),
    amount,
    nonce,
};
call.dispatch(RawOrigin::Root.into())?;

// 重复调用相同 nonce 会被忽略（幂等）
```

---

### 2. 资金释放（Release）

#### 2.1 release（全额释放）

**调用方**：AuthorizedOrigin | Root

**功能**：将托管全部余额转给收款人（正常履约）。

**流程**：
1. 验证权限
2. 检查全局暂停状态
3. 检查非争议状态（Disputed 状态下不允许）
4. 调用内部 `release_all` 方法
5. 发出 `Released` 事件

**Rust 示例**：

```rust
// OTC 订单完成，释放给做市商
use pallet_escrow::{Escrow as EscrowTrait};

let order_id = 100u64;
let maker = maker_account.clone();

T::Escrow::release_all(order_id, &maker)?;
```

**外部 extrinsic 调用**：

```rust
// 仅限 AuthorizedOrigin | Root
#[pallet::weight(10_000)]
pub fn release(
    origin: OriginFor<T>,
    id: u64,
    to: T::AccountId,
) -> DispatchResult
```

#### 2.2 release_split（分账释放）

**调用方**：AuthorizedOrigin | Root

**功能**：分多次转账，将托管余额按比例分配给多个账户（支持多账户分账）。

**流程**：
1. 验证权限
2. 检查全局暂停状态
3. 检查非争议状态
4. 验证合计金额不超过托管余额（`sum <= cur`）
5. 逐笔转账（原子事务）
6. 若余额为 0，设置状态为 Closed (3)
7. 发出多个 `Transfered` 事件

**用例场景**：
- **OTC 订单**：70% 给做市商，30% 给联盟计酬
- **Bridge 兑换**：扣除手续费后转给用户
- **供奉订单**：分配给多个受益人

**Rust 示例**：

```rust
// 分账释放：70% 给做市商，30% 给联盟计酬
use pallet_escrow::{Escrow as EscrowTrait};

let order_id = 100u64;
let total = T::Escrow::amount_of(order_id);
let maker_amount = total * 70 / 100;
let affiliate_amount = total - maker_amount;

// 构造分账条目
let entries = vec![
    (maker_account.clone(), maker_amount),
    (affiliate_account.clone(), affiliate_amount),
];

// 调用外部 extrinsic
let call = Call::release_split {
    id: order_id,
    entries,
};
call.dispatch(RawOrigin::Root.into())?;
```

**外部 extrinsic 调用**：

```rust
// 仅限 AuthorizedOrigin | Root
#[pallet::weight(10_000)]
pub fn release_split(
    origin: OriginFor<T>,
    id: u64,
    entries: Vec<(T::AccountId, BalanceOf<T>)>,
) -> DispatchResult
```

---

### 3. 资金退款（Refund）

#### 3.1 refund（全额退款）

**调用方**：AuthorizedOrigin | Root

**功能**：将托管全部余额退回给付款人（撤单/到期退款）。

**流程**：
1. 验证权限
2. 检查全局暂停状态
3. 检查非争议状态
4. 调用内部 `refund_all` 方法
5. 发出 `Refunded` 事件

**用例场景**：
- 订单取消
- 订单到期未完成
- 做市商拒绝接单

**Rust 示例**：

```rust
// 订单取消，退款给买家
use pallet_escrow::{Escrow as EscrowTrait};

let order_id = 100u64;
let buyer = buyer_account.clone();

T::Escrow::refund_all(order_id, &buyer)?;
```

**外部 extrinsic 调用**：

```rust
// 仅限 AuthorizedOrigin | Root
#[pallet::weight(10_000)]
pub fn refund(
    origin: OriginFor<T>,
    id: u64,
    to: T::AccountId,
) -> DispatchResult
```

---

### 4. 争议处理（Dispute & Arbitration）

#### 4.1 dispute（进入争议）

**调用方**：AuthorizedOrigin | Root

**功能**：将托管标记为争议状态，禁止普通释放/退款操作。

**流程**：
1. 验证权限
2. 检查托管余额是否存在（`Locked[id] > 0`）
3. 设置状态为 Disputed (1)
4. 发出 `Disputed` 事件（包含 reason 编码）

**状态转换**：
```
Locked (0) → Disputed (1)
```

**Rust 示例**：

```rust
// 订单进入争议
let order_id = 100u64;
let reason = 1u16; // 争议原因编码（1=质量问题，2=未收货等）

let call = Call::dispute {
    id: order_id,
    reason,
};
call.dispatch(RawOrigin::Root.into())?;
```

#### 4.2 apply_decision_release_all（仲裁决议-全额释放）

**调用方**：AuthorizedOrigin | Root（通常是 pallet-arbitration）

**功能**：仲裁裁决后，将托管全额释放给指定账户。

**流程**：
1. 验证权限
2. 调用内部 `release_all` 方法
3. 设置状态为 Resolved (2)
4. 发出 `DecisionApplied` 事件（decision=0）

**状态转换**：
```
Disputed (1) → Resolved (2)
```

**Rust 示例**：

```rust
// 仲裁裁决：全额释放给做市商
let order_id = 100u64;
let maker = maker_account.clone();

let call = Call::apply_decision_release_all {
    id: order_id,
    to: maker,
};
call.dispatch(RawOrigin::Root.into())?;
```

#### 4.3 apply_decision_refund_all（仲裁决议-全额退款）

**调用方**：AuthorizedOrigin | Root

**功能**：仲裁裁决后，将托管全额退款给指定账户。

**流程**：
1. 验证权限
2. 调用内部 `refund_all` 方法
3. 设置状态为 Resolved (2)
4. 发出 `DecisionApplied` 事件（decision=1）

**Rust 示例**：

```rust
// 仲裁裁决：全额退款给买家
let order_id = 100u64;
let buyer = buyer_account.clone();

let call = Call::apply_decision_refund_all {
    id: order_id,
    to: buyer,
};
call.dispatch(RawOrigin::Root.into())?;
```

#### 4.4 apply_decision_partial_bps（仲裁决议-按比例分配）

**调用方**：AuthorizedOrigin | Root

**功能**：仲裁裁决后，按 bps（基点）分配托管资金。

**参数**：
- `release_to`: 释放账户
- `refund_to`: 退款账户
- `bps`: 释放比例（0-10000，10000 = 100%）

**计算公式**：
```
释放金额 = floor(托管余额 × bps / 10000)
退款金额 = 托管余额 - 释放金额
```

**流程**：
1. 验证权限
2. 验证 bps <= 10000
3. 计算释放金额 = floor(cur × bps / 10000)
4. 转账释放金额给 release_to
5. 剩余金额退款给 refund_to
6. 设置状态为 Resolved (2)
7. 发出 `DecisionApplied` 事件（decision=2）

**用例场景**：
- 买家支付 30%，做市商得 70%
- 双方各 50%
- 根据证据比例分配

**Rust 示例**：

```rust
// 仲裁裁决：买家 30%，做市商 70%
let order_id = 100u64;
let buyer = buyer_account.clone();
let maker = maker_account.clone();
let bps = 7000u16; // 70% 给做市商

let call = Call::apply_decision_partial_bps {
    id: order_id,
    release_to: maker,
    refund_to: buyer,
    bps,
};
call.dispatch(RawOrigin::Root.into())?;
```

---

### 5. 自动到期处理（Expiry）

#### 5.1 schedule_expiry（安排到期处理）

**调用方**：AuthorizedOrigin | Root

**功能**：为托管设置到期时间，到期后自动执行策略。

**流程**：
1. 验证权限
2. 检查非争议状态（Disputed 状态下不生效）
3. 若已有到期时间，先从旧索引中移除
4. 更新 `ExpiryOf[id] = at`
5. 添加到 `ExpiringAt[at]` 索引（H-1优化）
6. 发出 `ExpiryScheduled` 事件

**H-1性能优化**：
- 使用 `ExpiringAt` 索引避免 on_initialize 遍历所有 `ExpiryOf`
- 性能提升：O(N) → O(1)，N = 总存储项数

**Rust 示例**：

```rust
// 创建订单时设置到期时间（30天后）
let order_id = 100u64;
let current_block = <frame_system::Pallet<T>>::block_number();
let expiry_at = current_block + 30 * 24 * 3600 / 6; // 30天后（假设6秒出块）

let call = Call::schedule_expiry {
    id: order_id,
    at: expiry_at,
};
call.dispatch(RawOrigin::Root.into())?;
```

#### 5.2 cancel_expiry（取消到期处理）

**调用方**：AuthorizedOrigin | Root

**功能**：取消托管的到期处理。

**流程**：
1. 验证权限
2. 从 `ExpiringAt` 索引中移除
3. 删除 `ExpiryOf[id]`

**用例场景**：
- 订单已提前完成
- 订单已取消
- 订单进入争议

**Rust 示例**：

```rust
// 订单完成，取消到期处理
let order_id = 100u64;

let call = Call::cancel_expiry {
    id: order_id,
};
call.dispatch(RawOrigin::Root.into())?;
```

#### 5.3 on_initialize（自动到期处理）

**调用方**：系统（每个块自动调用）

**功能**：处理当前块到期的托管订单。

**流程**：
1. 直接获取 `ExpiringAt[n]`（当前块到期的订单列表）
2. 跳过争议状态的订单
3. 调用 `ExpiryPolicy::on_expire(id)` 获取到期动作
4. 根据动作执行：
   - `ReleaseAll(to)`: 全额释放
   - `RefundAll(to)`: 全额退款
   - `Noop`: 无操作
5. 设置状态为 Resolved (2)
6. 清理 `ExpiryOf[id]`
7. 发出 `Expired` 事件（action=0/1/2）

**权重计算**：每个到期项约 20,000 单位

**限流保护**：每块最多处理 `MaxExpiringPerBlock` 个到期项（防止区块过重）

**ExpiryPolicy 实现示例**：

```rust
// Runtime 实现自定义到期策略
pub struct OtcExpiryPolicy;
impl pallet_escrow::ExpiryPolicy<AccountId, BlockNumber> for OtcExpiryPolicy {
    fn on_expire(id: u64) -> Result<pallet_escrow::ExpiryAction<AccountId>, DispatchError> {
        // 从 pallet-otc-order 查询订单信息
        if let Some(order) = pallet_otc_order::Orders::<Runtime>::get(id) {
            // OTC 订单到期：退款给买家
            Ok(pallet_escrow::ExpiryAction::RefundAll(order.buyer))
        } else {
            // 找不到订单：无操作
            Ok(pallet_escrow::ExpiryAction::Noop)
        }
    }

    fn now() -> BlockNumber {
        <frame_system::Pallet<Runtime>>::block_number()
    }
}
```

---

### 6. 全局暂停（Emergency Pause）

#### 6.1 set_pause（设置全局暂停）

**调用方**：AdminOrigin

**功能**：应急止血机制，暂停所有变更性操作（除 AdminOrigin 外）。

**流程**：
1. 验证 AdminOrigin 权限
2. 设置 `Paused = true/false`

**影响范围**：
- ✅ lock, lock_with_nonce
- ✅ release, release_split
- ✅ refund

**不影响**：
- ❌ 查询操作（amount_of）
- ❌ 仲裁决议执行（apply_decision_*）
- ❌ on_initialize 到期处理

**用例场景**：
- 发现安全漏洞，紧急暂停所有操作
- 系统升级维护
- 应对突发攻击

**Rust 示例**：

```rust
// 紧急暂停所有托管操作
let call = Call::set_pause {
    paused: true,
};
call.dispatch(RawOrigin::Root.into())?;

// 恢复正常
let call = Call::set_pause {
    paused: false,
};
call.dispatch(RawOrigin::Root.into())?;
```

---

## 📊 数据结构

### 托管状态（LockStateOf）

| 状态 | 代码 | 说明 | 允许操作 |
|------|-----|------|---------|
| Locked | 0 | 已锁定，可正常释放/退款 | lock, release, refund |
| Disputed | 1 | 争议中，仅允许仲裁决议接口处理 | dispute, apply_decision_* |
| Resolved | 2 | 已解决（仲裁裁决后） | 无（已结清） |
| Closed | 3 | 已关闭（全部结清，不再接受出金） | 无（余额为0） |

### 状态转换图

```
        lock
┌─────────────────┐
│  Locked (0)     │◄─── 初始状态
└────┬────────────┘
     │ dispute
     ▼
┌─────────────────┐
│  Disputed (1)   │─── 争议中，只能通过仲裁决议处理
└────┬────────────┘
     │ apply_decision_*
     ▼
┌─────────────────┐
│  Resolved (2)   │─── 已解决（仲裁裁决后）
└─────────────────┘

        release_split (余额=0)
┌─────────────────┐
│  Closed (3)     │─── 已关闭（全部结清）
└─────────────────┘
```

### 到期动作（ExpiryAction）

```rust
pub enum ExpiryAction<AccountId> {
    /// 全额释放给指定账户
    ReleaseAll(AccountId),
    /// 全额退款给指定账户
    RefundAll(AccountId),
    /// 无操作
    Noop,
}
```

### ExpiryPolicy Trait

```rust
pub trait ExpiryPolicy<AccountId, BlockNumber> {
    /// 返回到期应执行的动作
    fn on_expire(id: u64) -> Result<ExpiryAction<AccountId>, DispatchError>;
    /// 返回当前块
    fn now() -> BlockNumber;
}
```

**实现方**：Runtime（由业务 pallet 决定到期策略）

**用例**：
- **OTC 订单**：到期退款给买家
- **Bridge 兑换**：到期退款给用户
- **供奉订单**：到期无操作（已扣费）

---

## 🗄️ 存储项

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `Locked<T>` | `StorageMap<u64, BalanceOf<T>>` | 订单 → 锁定余额 |
| `LockStateOf<T>` | `StorageMap<u64, u8>` | 订单 → 托管状态（0-3） |
| `LockNonces<T>` | `StorageMap<u64, u64>` | 订单 → 最新 nonce（幂等） |
| `ExpiryOf<T>` | `StorageMap<u64, BlockNumber>` | 订单 → 到期块号 |
| `ExpiringAt<T>` | `StorageMap<BlockNumber, BoundedVec<u64>>` | 块号 → 到期订单列表（H-1优化） |
| `Paused<T>` | `StorageValue<bool>` | 全局暂停开关 |

### 存储查询示例

```rust
// 查询托管余额
let amount = pallet_escrow::Locked::<Runtime>::get(order_id);

// 查询托管状态
let state = pallet_escrow::LockStateOf::<Runtime>::get(order_id);

// 查询到期时间
let expiry = pallet_escrow::ExpiryOf::<Runtime>::get(order_id);

// 查询全局暂停状态
let paused = pallet_escrow::Paused::<Runtime>::get();
```

---

## 📝 事件定义

| 事件 | 参数 | 说明 |
|------|------|------|
| `Locked` | `id, amount` | 锁定到托管账户 |
| `Transfered` | `id, to, amount, remaining` | 从托管部分划转 |
| `Released` | `id, to, amount` | 全额释放 |
| `Refunded` | `id, to, amount` | 全额退款 |
| `Disputed` | `id, reason` | 进入争议 |
| `DecisionApplied` | `id, decision` | 已应用仲裁决议（0=ReleaseAll,1=RefundAll,2=PartialBps） |
| `ExpiryScheduled` | `id, at` | 已安排到期处理 |
| `Expired` | `id, action` | 到期已处理（0=Release,1=Refund,2=Noop） |

### 事件监听示例（TypeScript）

```typescript
// 监听托管锁定事件
api.query.system.events((events) => {
  events.forEach((record) => {
    const { event } = record;
    if (event.section === 'escrow' && event.method === 'Locked') {
      const [id, amount] = event.data;
      console.log(`订单 ${id} 锁定 ${amount} DUST`);
    }
  });
});

// 监听分账释放事件
api.query.system.events((events) => {
  events.forEach((record) => {
    const { event } = record;
    if (event.section === 'escrow' && event.method === 'Transfered') {
      const [id, to, amount, remaining] = event.data;
      console.log(`订单 ${id} 转账 ${amount} 给 ${to}，剩余 ${remaining}`);
    }
  });
});
```

---

## ⚠️ 错误定义

| 错误 | 说明 |
|------|------|
| `Insufficient` | 余额不足（付款人余额不足 或 托管余额不足） |
| `NoLock` | 托管不存在（id 无对应记录） |

---

## 🔌 Escrow Trait（供其他 Pallet 调用）

### 接口定义

```rust
pub trait Escrow<AccountId, Balance> {
    /// 从付款人转入托管并记录
    /// 安全要求：
    /// - 必须确保付款人余额充足（不足则返回 Error::Insufficient）
    /// - 仅供其他 Pallet 内部调用，不对外暴露权限判断
    fn lock_from(payer: &AccountId, id: u64, amount: Balance) -> DispatchResult;

    /// 从托管转出部分金额到指定账户（可多次分账）
    /// 安全要求：
    /// - 必须确保本 id 当前托管余额充足（amount ≤ cur），否则拒绝
    /// - 一次成功划转为原子事务，状态与实际转账保持一致
    fn transfer_from_escrow(id: u64, to: &AccountId, amount: Balance) -> DispatchResult;

    /// 将托管全部释放给收款人
    /// 用于正常履约或仲裁裁决
    fn release_all(id: u64, to: &AccountId) -> DispatchResult;

    /// 将托管全部退款给收款人
    /// 用于撤单/到期退款等场景
    fn refund_all(id: u64, to: &AccountId) -> DispatchResult;

    /// 查询当前托管余额
    fn amount_of(id: u64) -> Balance;
}
```

### 调用方

- **pallet-otc-order**：订单创建时锁定 DUST，完成时释放，取消时退款
- **pallet-dust-bridge**：兑换时锁定 DUST，OCW 确认后释放
- **pallet-arbitration**：争议处理时调用仲裁决议接口
- **pallet-memo-affiliate**：联盟计酬时分账释放

### 集成示例（pallet-otc-order）

```rust
// 在 pallet-otc-order 的 Cargo.toml 中添加依赖
[dependencies]
pallet-escrow = { path = "../escrow", default-features = false }

// 在 Config trait 中添加 Escrow 关联类型
pub trait Config: frame_system::Config {
    type Escrow: pallet_escrow::Escrow<Self::AccountId, BalanceOf<Self>>;
}

// 创建订单时锁定资金
#[pallet::weight(10_000)]
pub fn create_order(
    origin: OriginFor<T>,
    amount: BalanceOf<T>,
) -> DispatchResult {
    let buyer = ensure_signed(origin)?;
    let order_id = Self::next_order_id();

    // 锁定资金到托管
    T::Escrow::lock_from(&buyer, order_id, amount)?;

    // 创建订单记录
    Orders::<T>::insert(order_id, Order {
        buyer: buyer.clone(),
        amount,
        status: OrderStatus::Pending,
    });

    Ok(())
}

// 订单完成时释放资金
#[pallet::weight(10_000)]
pub fn complete_order(
    origin: OriginFor<T>,
    order_id: u64,
) -> DispatchResult {
    let maker = ensure_signed(origin)?;
    let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;

    // 释放资金给做市商
    T::Escrow::release_all(order_id, &maker)?;

    // 更新订单状态
    Orders::<T>::mutate(order_id, |o| {
        if let Some(order) = o {
            order.status = OrderStatus::Completed;
        }
    });

    Ok(())
}

// 订单取消时退款
#[pallet::weight(10_000)]
pub fn cancel_order(
    origin: OriginFor<T>,
    order_id: u64,
) -> DispatchResult {
    let buyer = ensure_signed(origin)?;
    let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
    ensure!(order.buyer == buyer, Error::<T>::NotOrderOwner);

    // 退款给买家
    T::Escrow::refund_all(order_id, &buyer)?;

    // 删除订单记录
    Orders::<T>::remove(order_id);

    Ok(())
}
```

---

## ⚙️ 配置参数

### Runtime 配置

```rust
parameter_types! {
    pub const EscrowPalletId: PalletId = PalletId(*b"py/escro");
    pub const MaxExpiringPerBlock: u32 = 100;
}

impl pallet_escrow::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type EscrowPalletId = EscrowPalletId;
    type AuthorizedOrigin = EnsureRoot<AccountId>; // 或自定义白名单
    type AdminOrigin = EnsureRoot<AccountId>; // 或治理委员会
    type MaxExpiringPerBlock = MaxExpiringPerBlock;
    type ExpiryPolicy = OtcExpiryPolicy; // 自定义到期策略
}
```

### 配置参数说明

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `EscrowPalletId` | "py/escro" | 托管账户 PalletId（8字节） |
| `MaxExpiringPerBlock` | 100 | 每块最多处理的到期项（防御性限制） |
| `AuthorizedOrigin` | `EnsureRoot` | 授权外部入口的 Origin（白名单 Origin） |
| `AdminOrigin` | `EnsureRoot` | 管理员 Origin（治理/应急） |
| `ExpiryPolicy` | 自定义 | 到期处理策略（由 runtime 实现） |

---

## 📱 前端集成示例

### TypeScript 查询示例

```typescript
import { ApiPromise, WsProvider } from '@polkadot/api';

// 连接到 Stardust 节点
const provider = new WsProvider('ws://localhost:9944');
const api = await ApiPromise.create({ provider });

// 查询托管余额
const orderId = 100;
const amount = await api.query.escrow.locked(orderId);
console.log('托管余额:', amount.toString());

// 查询托管状态
const state = await api.query.escrow.lockStateOf(orderId);
console.log('托管状态:', state.toNumber()); // 0=Locked, 1=Disputed, 2=Resolved, 3=Closed

// 查询到期时间
const expiryAt = await api.query.escrow.expiryOf(orderId);
if (expiryAt.isSome) {
  console.log('到期块:', expiryAt.unwrap().toNumber());
}

// 查询全局暂停状态
const paused = await api.query.escrow.paused();
console.log('全局暂停:', paused.toHuman());
```

### 管理员操作示例

```typescript
// 管理员暂停托管（应急）
const pauseTx = api.tx.escrow.setPause(true);
await pauseTx.signAndSend(adminAccount, ({ status }) => {
  if (status.isInBlock) {
    console.log('已暂停托管系统');
  }
});

// 恢复正常
const resumeTx = api.tx.escrow.setPause(false);
await resumeTx.signAndSend(adminAccount);
```

### 监听托管事件

```typescript
// 监听托管相关事件
api.query.system.events((events) => {
  events.forEach((record) => {
    const { event, phase } = record;

    if (event.section === 'escrow') {
      switch (event.method) {
        case 'Locked':
          const [id, amount] = event.data;
          console.log(`[Locked] 订单 ${id} 锁定 ${amount} DUST`);
          break;

        case 'Transfered':
          const [id2, to, amt, remaining] = event.data;
          console.log(`[Transfered] 订单 ${id2} 转账 ${amt} 给 ${to}，剩余 ${remaining}`);
          break;

        case 'Released':
          const [id3, to2, amt2] = event.data;
          console.log(`[Released] 订单 ${id3} 释放 ${amt2} 给 ${to2}`);
          break;

        case 'Refunded':
          const [id4, to3, amt3] = event.data;
          console.log(`[Refunded] 订单 ${id4} 退款 ${amt3} 给 ${to3}`);
          break;

        case 'Disputed':
          const [id5, reason] = event.data;
          console.log(`[Disputed] 订单 ${id5} 进入争议，原因: ${reason}`);
          break;

        case 'DecisionApplied':
          const [id6, decision] = event.data;
          const decisionText = ['ReleaseAll', 'RefundAll', 'PartialBps'][decision];
          console.log(`[DecisionApplied] 订单 ${id6} 应用仲裁决议: ${decisionText}`);
          break;

        case 'Expired':
          const [id7, action] = event.data;
          const actionText = ['Release', 'Refund', 'Noop'][action];
          console.log(`[Expired] 订单 ${id7} 到期处理: ${actionText}`);
          break;
      }
    }
  });
});
```

---

## 🔗 依赖关系

### 上游依赖

- **frame_support**：Pallet 框架
- **frame_system**：系统模块（获取当前块号）
- **pallet-balances**（或其他 Currency 实现）：货币转账
- **sp_runtime**：运行时类型和工具

### 下游调用方

- **pallet-otc-order**：OTC 订单托管管理
- **pallet-dust-bridge**：Bridge 兑换托管管理
- **pallet-arbitration**：争议处理和仲裁决议
- **pallet-memo-affiliate**：联盟计酬分账释放

### 依赖图

```
┌─────────────────┐
│ pallet-balances │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ pallet-escrow   │◄─── 核心基础设施
└────────┬────────┘
         │
         ├──────────┬──────────┬──────────┐
         ▼          ▼          ▼          ▼
  ┌──────────┐ ┌─────────┐ ┌──────────┐ ┌──────────┐
  │ otc-order│ │  bridge │ │arbitration│ │ affiliate│
  └──────────┘ └─────────┘ └──────────┘ └──────────┘
```

---

## 🎯 使用场景详解

### 场景1：OTC 订单托管

```rust
// 1. 买家创建订单，锁定 1000 DUST
let buyer = ensure_signed(origin)?;
let order_id = 100u64;
let amount = 1000 * 10u128.pow(12);

T::Escrow::lock_from(&buyer, order_id, amount)?;
T::Escrow::schedule_expiry(order_id, current_block + 43200)?; // 3天到期

// 2. 做市商完成订单，分账释放
let maker_amount = amount * 70 / 100;
let affiliate_amount = amount * 30 / 100;

let entries = vec![
    (maker.clone(), maker_amount),
    (affiliate.clone(), affiliate_amount),
];

// 调用 release_split
// T::Escrow::release_split(order_id, entries)?; // 需要通过 extrinsic

// 3. 或者到期自动退款（ExpiryPolicy 返回 RefundAll）
// on_initialize 会自动处理
```

### 场景2：Bridge 兑换托管

```rust
// 1. 用户发起兑换，锁定 100 DUST
let user = ensure_signed(origin)?;
let swap_id = 200u64;
let amount = 100 * 10u128.pow(12);

T::Escrow::lock_from(&user, swap_id, amount)?;
T::Escrow::schedule_expiry(swap_id, current_block + 7200)?; // 12小时到期

// 2. OCW 确认链上转账成功，扣除手续费后释放
let fee = amount * 1 / 100; // 1% 手续费
let net_amount = amount - fee;

T::Escrow::transfer_from_escrow(swap_id, &treasury, fee)?; // 手续费转国库
T::Escrow::release_all(swap_id, &user)?; // 剩余转给用户
```

### 场景3：争议处理流程

```rust
// 1. 买家发起争议
let order_id = 100u64;
let reason = 1u16; // 1=质量问题

// 调用 dispute extrinsic
// api.tx.escrow.dispute(order_id, reason)

// 2. 仲裁委员会投票裁决
// 投票结果：买家 30%，做市商 70%

// 3. 应用仲裁决议
let bps = 7000u16; // 70% 给做市商

// 调用 apply_decision_partial_bps extrinsic
// api.tx.escrow.applyDecisionPartialBps(order_id, maker, buyer, bps)
```

---

## 📌 最佳实践

### 1. 安全实践

#### 1.1 权限控制

```rust
// ✅ 正确：通过 Escrow trait 内部调用
impl<T: Config> Pallet<T> {
    pub fn internal_function() -> DispatchResult {
        let payer = Self::get_payer();
        let order_id = 100u64;
        let amount = 1000;

        T::Escrow::lock_from(&payer, order_id, amount)?;
        Ok(())
    }
}

// ❌ 错误：直接暴露外部 extrinsic
#[pallet::weight(10_000)]
pub fn lock(
    origin: OriginFor<T>,
    id: u64,
    payer: T::AccountId,
    amount: BalanceOf<T>,
) -> DispatchResult {
    let _ = ensure_signed(origin)?; // 任何人都能调用，不安全！
    T::Escrow::lock_from(&payer, id, amount)
}
```

#### 1.2 余额检查

```rust
// ✅ 正确：先检查余额再锁定
let balance = T::Currency::free_balance(&payer);
ensure!(balance >= amount, Error::<T>::InsufficientBalance);

T::Escrow::lock_from(&payer, order_id, amount)?;

// ❌ 错误：不检查余额直接锁定（会导致交易失败）
T::Escrow::lock_from(&payer, order_id, amount)?; // 可能失败
```

#### 1.3 状态检查

```rust
// ✅ 正确：检查托管状态
let state = pallet_escrow::LockStateOf::<T>::get(order_id);
ensure!(state != 1u8, Error::<T>::OrderInDispute); // 争议中不允许操作

T::Escrow::release_all(order_id, &maker)?;

// ❌ 错误：不检查状态直接操作（可能违反业务逻辑）
T::Escrow::release_all(order_id, &maker)?;
```

### 2. 性能优化

#### 2.1 批量操作

```rust
// ✅ 正确：使用 release_split 批量分账
let entries = vec![
    (account1, amount1),
    (account2, amount2),
    (account3, amount3),
];
// 调用 release_split extrinsic（一次性完成）

// ❌ 错误：多次调用 transfer_from_escrow（效率低）
T::Escrow::transfer_from_escrow(id, &account1, amount1)?;
T::Escrow::transfer_from_escrow(id, &account2, amount2)?;
T::Escrow::transfer_from_escrow(id, &account3, amount3)?;
```

#### 2.2 到期索引优化

```rust
// ✅ 正确：使用 ExpiringAt 索引（H-1优化）
// on_initialize 直接获取当前块到期的订单
let expiring_ids = ExpiringAt::<T>::take(current_block); // O(1)

// ❌ 错误：遍历所有 ExpiryOf（性能差）
for (id, at) in ExpiryOf::<T>::iter() { // O(N)
    if at == current_block {
        // 处理到期
    }
}
```

### 3. 幂等性保证

```rust
// ✅ 正确：使用 lock_with_nonce 防止重放
let nonce = current_nonce + 1;
// 调用 lock_with_nonce extrinsic

// 重复调用相同 nonce 会被忽略（幂等）

// ❌ 错误：使用 lock 多次调用会累加
// 调用 lock extrinsic（多次调用会累加余额）
```

### 4. 错误处理

```rust
// ✅ 正确：详细错误处理
match T::Escrow::lock_from(&payer, order_id, amount) {
    Ok(_) => {
        // 继续后续逻辑
        Self::create_order(order_id)?;
    }
    Err(e) => {
        // 记录日志
        log::error!("Failed to lock escrow: {:?}", e);
        return Err(Error::<T>::EscrowLockFailed.into());
    }
}

// ❌ 错误：忽略错误
T::Escrow::lock_from(&payer, order_id, amount).ok(); // 忽略错误
Self::create_order(order_id)?; // 可能导致状态不一致
```

### 5. 测试覆盖

```rust
#[test]
fn test_escrow_lifecycle() {
    new_test_ext().execute_with(|| {
        // 1. 锁定
        assert_ok!(Escrow::lock_from(&1, 100, 1000));
        assert_eq!(Escrow::amount_of(100), 1000);

        // 2. 部分转出
        assert_ok!(Escrow::transfer_from_escrow(100, &2, 300));
        assert_eq!(Escrow::amount_of(100), 700);

        // 3. 全额释放
        assert_ok!(Escrow::release_all(100, &3));
        assert_eq!(Escrow::amount_of(100), 0);
    });
}

#[test]
fn test_insufficient_balance() {
    new_test_ext().execute_with(|| {
        // 测试余额不足场景
        assert_noop!(
            Escrow::lock_from(&1, 100, 999999999),
            Error::<Test>::Insufficient
        );
    });
}
```

---

## 🔧 运维指南

### 监控指标

#### 1. 托管总余额

```typescript
// 计算所有托管订单的总余额
const allLockedEntries = await api.query.escrow.locked.entries();
let totalLocked = 0n;
for (const [key, amount] of allLockedEntries) {
  totalLocked += amount.toBigInt();
}
console.log('托管总余额:', totalLocked);
```

#### 2. 状态分布

```typescript
// 统计各状态的订单数量
const stateDistribution = {
  locked: 0,
  disputed: 0,
  resolved: 0,
  closed: 0,
};

const allStates = await api.query.escrow.lockStateOf.entries();
for (const [key, state] of allStates) {
  const stateNum = state.toNumber();
  switch (stateNum) {
    case 0: stateDistribution.locked++; break;
    case 1: stateDistribution.disputed++; break;
    case 2: stateDistribution.resolved++; break;
    case 3: stateDistribution.closed++; break;
  }
}
console.log('状态分布:', stateDistribution);
```

#### 3. 到期订单监控

```typescript
// 查询未来24小时内到期的订单
const currentBlock = await api.query.system.number();
const blocksPerDay = 24 * 3600 / 6; // 假设6秒出块
const endBlock = currentBlock + blocksPerDay;

const expiringOrders = [];
for (let block = currentBlock; block < endBlock; block++) {
  const orders = await api.query.escrow.expiringAt(block);
  if (orders.length > 0) {
    expiringOrders.push({ block, orders: orders.toHuman() });
  }
}
console.log('未来24小时到期订单:', expiringOrders);
```

### 应急操作

#### 1. 全局暂停

```bash
# 发现安全问题，立即暂停
polkadot-js-api tx.escrow.setPause(true) --sudo --seed "//Alice"

# 修复问题后恢复
polkadot-js-api tx.escrow.setPause(false) --sudo --seed "//Alice"
```

#### 2. 手动处理到期

```bash
# 查询特定到期块的订单
polkadot-js-api query.escrow.expiringAt 1000000

# 手动触发释放（如果 ExpiryPolicy 出问题）
polkadot-js-api tx.escrow.release 100 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY --sudo
```

---

## 📚 参考资料

### 相关 Pallet 文档

- [`pallet-otc-order`](/home/xiaodong/文档/stardust/pallets/otc-order/README.md) - OTC 订单管理
- [`pallet-dust-bridge`](/home/xiaodong/文档/stardust/pallets/dust-bridge/README.md) - Bridge 兑换管理
- [`pallet-arbitration`](/home/xiaodong/文档/stardust/pallets/arbitration/README.md) - 争议仲裁系统
- [`pallet-memo-affiliate`](/home/xiaodong/文档/stardust/pallets/affiliate/README.md) - 联盟计酬系统

### 技术文档

- [Substrate Pallet 开发指南](https://docs.substrate.io/reference/how-to-guides/pallet-design/)
- [FRAME Currency Trait](https://docs.substrate.io/rustdocs/latest/frame_support/traits/tokens/currency/trait.Currency.html)
- [Polkadot SDK 文档](https://docs.substrate.io/)

---

## 📄 许可证

MIT-0

---

## 👥 维护者

Stardust Team

---

## 📝 更新日志

### v0.1.0 (当前版本)

- ✅ 基础托管功能（锁定、释放、退款）
- ✅ 多次分账支持（release_split）
- ✅ 争议处理机制（dispute + 仲裁决议）
- ✅ 自动到期处理（ExpiryPolicy + on_initialize）
- ✅ 幂等锁定（lock_with_nonce）
- ✅ 全局暂停开关（set_pause）
- ✅ H-1性能优化（ExpiringAt 索引）

### 待实现功能

- [ ] 基准权重（WeightInfo）替换常量权重
- [ ] 更细粒度的权限控制（白名单管理）
- [ ] 托管历史记录（链下索引）
- [ ] 多币种支持（MultiCurrency trait）

---

**完整功能，安全可靠，性能优化，生产就绪！**
