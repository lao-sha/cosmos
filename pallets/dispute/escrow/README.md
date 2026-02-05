# pallet-escrow

> 路径：`pallets/dispute/escrow/`

资金托管系统，提供安全的资金锁定、释放、退款和分账功能，作为交易、仲裁等业务的底层基础设施。

## 设计理念

- **安全优先**：外部调用需授权，内部 trait 供其他 pallet 调用
- **原子操作**：所有资金操作为原子事务，失败自动回滚
- **状态一致**：托管状态与实际余额保持一致
- **可扩展**：通过 `ExpiryPolicy` 支持自定义到期策略

## 核心功能

| 功能 | 说明 |
|------|------|
| 资金锁定 | 从付款人转入托管账户 |
| 全额释放 | 转给收款人（卖家） |
| 全额退款 | 退给付款人（买家） |
| 分账释放 | 按金额或比例多方分配 |
| 争议冻结 | 争议期间冻结资金操作 |
| 仲裁执行 | Release / Refund / Partial |
| 到期处理 | 自动执行到期策略 |
| 全局暂停 | 应急暂停所有操作 |

## 托管状态

| 状态 | 值 | 说明 |
|------|---|------|
| Locked | 0 | 资金已锁定，可正常操作 |
| Disputed | 1 | 争议中，仅允许仲裁操作 |
| Resolved | 2 | 仲裁完成 |
| Closed | 3 | 资金已全部转出 |

## Extrinsics

### 基础操作
| 方法 | call_index | 说明 |
|------|-----------|------|
| `lock` | 0 | 锁定资金到托管 |
| `release` | 1 | 全额释放给收款人 |
| `refund` | 2 | 全额退款给付款人 |
| `lock_with_nonce` | 3 | 幂等锁定（防重放） |
| `release_split` | 4 | 分账释放（多方） |

### 争议处理
| 方法 | call_index | 说明 |
|------|-----------|------|
| `dispute` | 5 | 进入争议状态 |
| `apply_decision_release_all` | 6 | 裁决：全额释放 |
| `apply_decision_refund_all` | 7 | 裁决：全额退款 |
| `apply_decision_partial_bps` | 8 | 裁决：按比例分账 |

### 管理操作
| 方法 | call_index | 说明 |
|------|-----------|------|
| `set_pause` | 9 | 设置全局暂停（Admin） |
| `schedule_expiry` | 10 | 安排到期处理 |
| `cancel_expiry` | 11 | 取消到期处理 |

## Trait 接口

### Escrow（供其他 pallet 调用）
```rust
pub trait Escrow<AccountId, Balance> {
    /// 锁定资金
    fn lock_from(payer: &AccountId, id: u64, amount: Balance) -> DispatchResult;
    /// 转出部分金额
    fn transfer_from_escrow(id: u64, to: &AccountId, amount: Balance) -> DispatchResult;
    /// 全额释放
    fn release_all(id: u64, to: &AccountId) -> DispatchResult;
    /// 全额退款
    fn refund_all(id: u64, to: &AccountId) -> DispatchResult;
    /// 查询余额
    fn amount_of(id: u64) -> Balance;
    /// 托管账户地址
    fn escrow_account() -> AccountId;
    /// 按比例分账
    fn split_partial(id: u64, release_to: &AccountId, refund_to: &AccountId, bps: u16) -> DispatchResult;
}
```

### ExpiryPolicy（到期策略）
```rust
pub trait ExpiryPolicy<AccountId, BlockNumber> {
    fn on_expire(id: u64) -> Result<ExpiryAction<AccountId>, DispatchError>;
    fn now() -> BlockNumber;
}

pub enum ExpiryAction<AccountId> {
    ReleaseAll(AccountId),
    RefundAll(AccountId),
    NoAction,
}
```

## 配置参数

| 参数 | 说明 |
|------|------|
| `EscrowPalletId` | 托管模块 PalletId |
| `AdminOrigin` | 管理员 Origin |
| `AuthorizedOrigin` | 授权操作 Origin |

## 托管账户

托管资金存放在由 `PalletId` 派生的模块账户：

```rust
fn account() -> T::AccountId {
    T::EscrowPalletId::get().into_account_truncating()
}
```

## 集成示例

```rust
// OTC 模块使用 Escrow trait
impl<T: Config> Pallet<T> {
    fn create_order(buyer: &T::AccountId, amount: Balance) -> DispatchResult {
        let order_id = Self::next_order_id();
        // 锁定买家资金到托管
        T::Escrow::lock_from(buyer, order_id, amount)?;
        Ok(())
    }
    
    fn complete_order(order_id: u64, seller: &T::AccountId) -> DispatchResult {
        // 释放资金给卖家
        T::Escrow::release_all(order_id, seller)
    }
}
```

## 相关模块

- `@/home/xiaodong/桌面/cosmos/pallets/dispute/arbitration/` - 仲裁系统（调用裁决接口）
- `@/home/xiaodong/桌面/cosmos/pallets/trading/otc/` - OTC 交易（使用托管）
