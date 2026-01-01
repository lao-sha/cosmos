# Pallet Arbitration（仲裁争议处理系统）

## 📋 模块概述

`pallet-arbitration` 是 Stardust 区块链的**仲裁争议处理系统**，提供去中心化的争议登记、证据管理、仲裁裁决、资金分账、双向押金管理等完整的纠纷解决功能。本模块通过域路由架构（`ArbitrationRouter`）实现与业务 pallet 的低耦合集成，支持 OTC 交易、Bridge 兑换、供奉订单等多种业务场景的争议处理。

### 核心特性

- ✅ **域路由架构**：通过 8 字节域常量标识业务场景，支持多业务统一仲裁
- ✅ **双向押金机制**：发起方与应诉方各自从托管账户锁定 15% 订单金额作为押金
- ✅ **灵活裁决系统**：支持全额释放、全额退款、按比例分配三种裁决方式
- ✅ **证据引用管理**：与 `pallet-evidence` 集成，通过 evidence_id 引用证据
- ✅ **托管集成**：与 `pallet-escrow` 深度集成，自动执行资金分账
- ✅ **治理授权**：仅允许 Root 或治理委员会执行裁决，确保公正性
- ✅ **应诉期限机制**：设置应诉截止期，超时未应诉视为弃权
- ✅ **押金罚没规则**：败诉方押金罚没 30%，部分胜诉各罚没 50%

### 设计理念

1. **低耦合架构**：通过 `ArbitrationRouter` trait 实现业务逻辑与仲裁逻辑分离
2. **域驱动设计**：每个业务域（OTC、Bridge、Offering）独立管理仲裁规则
3. **安全优先**：仅授权账户可发起争议，仅治理可执行裁决
4. **资金安全**：所有押金操作在托管账户上进行，无需用户额外转账
5. **防御性设计**：双向押金防止恶意发起争议，罚没机制惩罚违约方

### 版本历史

- **v0.1.0 (2025-10-22)**：初始版本，支持基础争议登记和裁决
- **v0.2.0 (2025-11-11)**：新增双向押金机制、应诉期限、罚没规则
- **v0.3.0 (TODO)**：计划集成 `pallet-credit` 信用分记录

---

## 🔑 核心功能

### 1. 争议登记（Dispute）

#### 1.1 `dispute`（基础争议登记）

**调用方**：授权账户（通过 `ArbitrationRouter::can_dispute` 验证）

**功能**：登记争议并提交证据 CID（旧版接口，兼容性保留）。

**处理流程**：

1. 验证权限（Router.can_dispute）
2. 检查未被登记（防止重复）
3. 登记争议标记（Disputed）
4. 存储证据 CID 列表（可选）
5. 触发 `Disputed` 事件

**函数签名**：

```rust
pub fn dispute(
    origin: OriginFor<T>,
    domain: [u8; 8],                                    // 域标识（如 b"otc_ord_"）
    id: u64,                                            // 订单/交易 ID
    _evidence: Vec<BoundedVec<u8, T::MaxCidLen>>,      // 证据 CID 列表（旧版，建议用 evidence_id）
) -> DispatchResult
```

**权重计算**：

```rust
#[pallet::weight(T::WeightInfo::dispute(_evidence.len() as u32))]
```

**使用示例**：

```rust
// OTC 订单争议
let domain = *b"otc_ord_";
let order_id = 12345u64;
let evidence_cids = vec![
    BoundedVec::try_from(b"QmEvidence1".to_vec()).unwrap(),
    BoundedVec::try_from(b"QmEvidence2".to_vec()).unwrap(),
];

Arbitration::dispute(
    RuntimeOrigin::signed(buyer),
    domain,
    order_id,
    evidence_cids,
)?;
```

---

#### 1.2 `dispute_with_evidence_id`（按证据 ID 登记争议）

**调用方**：授权账户

**功能**：登记争议并引用 `pallet-evidence` 中已提交的证据。

**处理流程**：

1. 验证权限（Router.can_dispute）
2. 检查未被登记
3. 登记争议标记（Disputed）
4. 将 evidence_id 追加到 EvidenceIds 列表
5. 触发 `Disputed` 事件

**函数签名**：

```rust
pub fn dispute_with_evidence_id(
    origin: OriginFor<T>,
    domain: [u8; 8],          // 域标识
    id: u64,                  // 订单/交易 ID
    evidence_id: u64,         // 证据 ID（来自 pallet-evidence）
) -> DispatchResult
```

**使用示例**：

```rust
// 步骤 1：先在 pallet-evidence 中提交证据
let evidence_id = Evidence::commit_hash(
    RuntimeOrigin::signed(buyer),
    *b"otc_ord_",
    order_id,
    commit_hash,
    None,
)?;

// 步骤 2：引用证据 ID 发起争议
Arbitration::dispute_with_evidence_id(
    RuntimeOrigin::signed(buyer),
    *b"otc_ord_",
    order_id,
    evidence_id,
)?;
```

---

#### 1.3 `append_evidence_id`（追加证据）

**调用方**：授权账户

**功能**：为已登记的争议追加新证据。

**使用场景**：
- 补充证据
- 反驳对方证据
- 多轮举证

**函数签名**：

```rust
pub fn append_evidence_id(
    origin: OriginFor<T>,
    domain: [u8; 8],          // 域标识
    id: u64,                  // 订单/交易 ID
    evidence_id: u64,         // 新证据 ID
) -> DispatchResult
```

**使用示例**：

```rust
// 追加反驳证据
let counter_evidence_id = Evidence::commit_hash(
    RuntimeOrigin::signed(seller),
    *b"otc_ord_",
    order_id,
    counter_commit,
    None,
)?;

Arbitration::append_evidence_id(
    RuntimeOrigin::signed(seller),
    *b"otc_ord_",
    order_id,
    counter_evidence_id,
)?;
```

---

#### 1.4 `dispute_with_two_way_deposit`（双向押金争议）

**调用方**：授权账户（通常是买家）

**功能**：发起争议并从托管账户锁定发起方押金（订单金额的 15%），同时通知应诉方应诉。

**处理流程**：

1. 验证权限（Router.can_dispute）
2. 检查未被登记
3. 获取订单金额（Router.get_order_amount）
4. 计算押金金额（15% = 1500 基点）
5. 从托管账户锁定发起方押金（使用 HoldReason::DisputeInitiator）
6. 获取应诉方账户（Router.get_counterparty）
7. 计算应诉截止期（当前块 + ResponseDeadline）
8. 登记争议和双向押金记录
9. 添加证据引用
10. 触发 `DisputeWithDepositInitiated` 事件

**函数签名**：

```rust
pub fn dispute_with_two_way_deposit(
    origin: OriginFor<T>,
    domain: [u8; 8],          // 域标识
    id: u64,                  // 订单/交易 ID
    evidence_id: u64,         // 证据 ID
) -> DispatchResult
```

**押金计算**：

```rust
// 订单金额的 15%
let deposit_ratio_bps = T::DepositRatioBps::get();  // 1500 基点
let deposit_amount = Perbill::from_parts((deposit_ratio_bps as u32) * 100).mul_floor(order_amount);

// 示例：订单金额 1000 DUST
// 押金 = 1000 * 15% = 150 DUST
```

**应诉截止期**：

```rust
// 7 天后（默认）
let current_block = frame_system::Pallet::<T>::block_number();
let deadline = current_block + T::ResponseDeadline::get();  // 7 * 24 * 3600 / 6 = 100800 块
```

**使用示例**：

```rust
// 买家发起双向押金争议
let domain = *b"otc_ord_";
let order_id = 12345u64;

// 先提交证据
let evidence_id = Evidence::commit_hash(
    RuntimeOrigin::signed(buyer),
    domain,
    order_id,
    buyer_evidence_commit,
    None,
)?;

// 发起争议（从托管扣押金）
Arbitration::dispute_with_two_way_deposit(
    RuntimeOrigin::signed(buyer),
    domain,
    order_id,
    evidence_id,
)?;

// 事件：DisputeWithDepositInitiated
// - initiator: buyer
// - respondent: seller
// - deposit: 150 DUST (假设订单 1000 DUST)
// - deadline: block_number + 100800
```

---

#### 1.5 `respond_to_dispute`（应诉并锁定押金）

**调用方**：应诉方（通常是卖家）

**功能**：应诉方从托管账户锁定押金（与发起方相同金额）并提交反驳证据。

**处理流程**：

1. 验证是应诉方（deposit_record.respondent == 签名者）
2. 确保未应诉（has_responded == false）
3. 检查未超时（current_block <= response_deadline）
4. 从托管账户锁定应诉方押金（使用 HoldReason::DisputeRespondent）
5. 更新押金记录（respondent_deposit, has_responded）
6. 添加反驳证据
7. 触发 `RespondentDepositLocked` 事件

**函数签名**：

```rust
pub fn respond_to_dispute(
    origin: OriginFor<T>,
    domain: [u8; 8],          // 域标识
    id: u64,                  // 订单/交易 ID
    counter_evidence_id: u64, // 反驳证据 ID
) -> DispatchResult
```

**超时处理**：

```rust
// 如果超时未应诉，仲裁时视为弃权
// 仲裁时只罚没发起方押金，应诉方押金为 None
```

**使用示例**：

```rust
// 卖家应诉
let domain = *b"otc_ord_";
let order_id = 12345u64;

// 先提交反驳证据
let counter_evidence_id = Evidence::commit_hash(
    RuntimeOrigin::signed(seller),
    domain,
    order_id,
    seller_evidence_commit,
    None,
)?;

// 应诉（从托管扣押金）
Arbitration::respond_to_dispute(
    RuntimeOrigin::signed(seller),
    domain,
    order_id,
    counter_evidence_id,
)?;

// 事件：RespondentDepositLocked
// - respondent: seller
// - deposit: 150 DUST（与发起方相同）
```

---

### 2. 仲裁裁决（Arbitrate）

#### 2.1 `arbitrate`（执行裁决）

**调用方**：治理起源（Root 或治理委员会）

**功能**：仲裁委员会/Root 执行裁决，调用业务 pallet 的 apply_decision 钩子，并处理双向押金。

**处理流程**：

1. 验证治理权限（DecisionOrigin::ensure_origin）
2. 检查争议已登记（Disputed 存在）
3. 解码裁决参数（decision_code, bps）
4. 调用 Router.apply_decision 执行业务逻辑
5. 处理双向押金（handle_deposits_on_arbitration）
6. 触发 `Arbitrated` 事件

**函数签名**：

```rust
pub fn arbitrate(
    origin: OriginFor<T>,
    domain: [u8; 8],          // 域标识
    id: u64,                  // 订单/交易 ID
    decision_code: u8,        // 裁决类型（0=Release, 1=Refund, 2=Partial）
    bps: Option<u16>,         // 部分裁决比例（仅 decision_code=2 时需要）
) -> DispatchResult
```

**裁决类型**：

| decision_code | 裁决类型 | 说明 | bps 参数 |
|--------------|---------|------|---------|
| 0 | Release | 全额释放给收款人（卖家胜诉） | 不需要 |
| 1 | Refund | 全额退款给付款人（买家胜诉） | 不需要 |
| 2 | Partial | 按比例分配 | 需要（0-10000） |

**部分裁决比例（bps）**：

```rust
// bps = 7000 表示 70% 给卖家，30% 给买家
// bps = 5000 表示 50% 给卖家，50% 给买家
// bps = 3000 表示 30% 给卖家，70% 给买家
```

**使用示例**：

```rust
// 场景 1：卖家胜诉（全额释放）
Arbitration::arbitrate(
    RuntimeOrigin::root(),
    *b"otc_ord_",
    order_id,
    0,      // decision_code: Release
    None,   // bps 不需要
)?;

// 场景 2：买家胜诉（全额退款）
Arbitration::arbitrate(
    RuntimeOrigin::root(),
    *b"otc_ord_",
    order_id,
    1,      // decision_code: Refund
    None,   // bps 不需要
)?;

// 场景 3：部分胜诉（70% 给卖家，30% 给买家）
Arbitration::arbitrate(
    RuntimeOrigin::root(),
    *b"otc_ord_",
    order_id,
    2,          // decision_code: Partial
    Some(7000), // bps: 70%
)?;
```

---

#### 2.2 押金处理逻辑（`handle_deposits_on_arbitration`）

**触发时机**：arbitrate 调用后自动执行

**处理规则**：

| 裁决结果 | 发起方押金 | 应诉方押金 | 罚没去向 |
|---------|----------|----------|---------|
| **Release（卖家胜诉）** | 罚没 30%，70% 返还托管 | 全额返还托管 | 国库 |
| **Refund（买家胜诉）** | 全额返还托管 | 罚没 30%，70% 返还托管 | 国库 |
| **Partial（部分胜诉）** | 罚没 50%，50% 返还托管 | 罚没 50%，50% 返还托管 | 国库 |

**罚没比例配置**：

```rust
parameter_types! {
    pub const RejectedSlashBps: u16 = 3000;  // 30%（败诉方）
    pub const PartialSlashBps: u16 = 5000;   // 50%（部分胜诉）
}
```

**押金处理示例**：

```rust
// 假设订单金额 1000 DUST，押金各 150 DUST

// 场景 1：卖家胜诉
// - 买家押金：罚没 45 DUST（30%），返还 105 DUST
// - 卖家押金：返还 150 DUST
// - 国库收入：45 DUST

// 场景 2：买家胜诉
// - 买家押金：返还 150 DUST
// - 卖家押金：罚没 45 DUST（30%），返还 105 DUST
// - 国库收入：45 DUST

// 场景 3：部分胜诉
// - 买家押金：罚没 75 DUST（50%），返还 75 DUST
// - 卖家押金：罚没 75 DUST（50%），返还 75 DUST
// - 国库收入：150 DUST
```

---

### 3. 域路由机制（ArbitrationRouter）

#### 3.1 Router Trait 定义

**设计目的**：
- 以 8 字节域常量标识业务域（与 PalletId 字节对齐）
- 实现业务逻辑与仲裁逻辑解耦
- 支持多业务统一仲裁

**Trait 定义**：

```rust
pub trait ArbitrationRouter<AccountId, Balance> {
    /// 校验是否允许发起争议
    /// - 例如：OTC 订单的买家或卖家可以发起
    fn can_dispute(domain: [u8; 8], who: &AccountId, id: u64) -> bool;

    /// 应用裁决（放款/退款/部分放款）
    /// - 由各业务 pallet 实现具体的资金分账逻辑
    fn apply_decision(domain: [u8; 8], id: u64, decision: Decision) -> DispatchResult;

    /// 获取纠纷对方账户（发起方是买家，返回卖家；反之亦然）
    /// - 用于双向押金机制
    fn get_counterparty(domain: [u8; 8], initiator: &AccountId, id: u64) -> Result<AccountId, DispatchError>;

    /// 获取订单/交易金额（用于计算押金）
    /// - 押金 = 订单金额 × 15%
    fn get_order_amount(domain: [u8; 8], id: u64) -> Result<Balance, DispatchError>;
}
```

---

#### 3.2 Runtime 实现示例

**实现方案**：在 runtime 中实现 Router，根据 domain 分发到对应的业务 pallet。

```rust
// runtime/src/lib.rs
pub struct ArbitrationRouterImpl;

impl pallet_arbitration::ArbitrationRouter<AccountId, Balance> for ArbitrationRouterImpl {
    fn can_dispute(domain: [u8; 8], who: &AccountId, id: u64) -> bool {
        match &domain {
            b"otc_ord_" => {
                // OTC 订单：买家或卖家可以发起
                if let Some(order) = OtcOrder::orders(id) {
                    &order.buyer == who || &order.maker == who
                } else {
                    false
                }
            }
            b"bridge__" => {
                // Bridge 兑换：用户可以发起
                if let Some(swap) = DustBridge::swaps(id) {
                    &swap.user == who
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn apply_decision(domain: [u8; 8], id: u64, decision: Decision) -> DispatchResult {
        match &domain {
            b"otc_ord_" => {
                // OTC 订单裁决
                match decision {
                    Decision::Release => {
                        // 全额释放给做市商
                        if let Some(order) = OtcOrder::orders(id) {
                            Escrow::release_all(id, &order.maker)?;
                        }
                    }
                    Decision::Refund => {
                        // 全额退款给买家
                        if let Some(order) = OtcOrder::orders(id) {
                            Escrow::refund_all(id, &order.buyer)?;
                        }
                    }
                    Decision::Partial(bps) => {
                        // 按比例分配
                        // ...
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn get_counterparty(domain: [u8; 8], initiator: &AccountId, id: u64) -> Result<AccountId, DispatchError> {
        match &domain {
            b"otc_ord_" => {
                if let Some(order) = OtcOrder::orders(id) {
                    if &order.buyer == initiator {
                        Ok(order.maker)
                    } else {
                        Ok(order.buyer)
                    }
                } else {
                    Err(Error::<Runtime>::OrderNotFound.into())
                }
            }
            _ => Err(Error::<Runtime>::UnknownDomain.into()),
        }
    }

    fn get_order_amount(domain: [u8; 8], id: u64) -> Result<Balance, DispatchError> {
        match &domain {
            b"otc_ord_" => {
                if let Some(order) = OtcOrder::orders(id) {
                    Ok(order.amount)
                } else {
                    Err(Error::<Runtime>::OrderNotFound.into())
                }
            }
            _ => Err(Error::<Runtime>::UnknownDomain.into()),
        }
    }
}
```

**Runtime Config**：

```rust
impl pallet_arbitration::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxEvidence = ConstU32<100>;
    type MaxCidLen = ConstU32<64>;
    type Escrow = Escrow;
    type WeightInfo = pallet_arbitration::weights::SubstrateWeight<Runtime>;
    type Router = ArbitrationRouterImpl;  // 注入自定义 Router
    type DecisionOrigin = EnsureRoot<AccountId>;  // 或治理委员会
    type Fungible = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type DepositRatioBps = ConstU16<1500>;  // 15%
    type ResponseDeadline = ConstU32<100800>;  // 7 天
    type RejectedSlashBps = ConstU16<3000>;  // 30%
    type PartialSlashBps = ConstU16<5000>;  // 50%
    type TreasuryAccount = TreasuryAccountId;
}
```

---

#### 3.3 域标识规范

**推荐格式**：8 字节 ASCII，末尾用下划线填充

| 域标识 | 业务场景 | 说明 |
|-------|---------|------|
| `b"otc_ord_"` | OTC 订单 | OTC 交易争议 |
| `b"bridge__"` | Bridge 兑换 | 跨链兑换争议 |
| `b"offering"` | 供奉订单 | 纪念馆供奉争议 |
| `b"grave___"` | 墓地订单 | 墓地购买争议 |

---

## 📊 数据结构

### Decision（裁决类型）

```rust
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Decision {
    /// 全额释放给收款人（卖家胜诉）
    Release,

    /// 全额退款给付款人（买家胜诉）
    Refund,

    /// 按比例分配（部分胜诉）
    /// - bps: 释放比例（0-10000，10000 = 100%）
    Partial(u16),  // bps
}
```

---

### TwoWayDepositRecord（双向押金记录）

```rust
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct TwoWayDepositRecord<AccountId, Balance, BlockNumber> {
    /// 发起方账户（通常是买家）
    pub initiator: AccountId,

    /// 发起方押金金额
    pub initiator_deposit: Balance,

    /// 应诉方账户（通常是卖家）
    pub respondent: AccountId,

    /// 应诉方押金金额（可选，未应诉时为 None）
    pub respondent_deposit: Option<Balance>,

    /// 应诉截止区块
    pub response_deadline: BlockNumber,

    /// 是否已应诉
    pub has_responded: bool,
}
```

---

### HoldReason（押金锁定原因）

```rust
#[pallet::composite_enum]
pub enum HoldReason {
    /// 纠纷发起方押金（通常是买家）
    DisputeInitiator,

    /// 应诉方押金（通常是卖家）
    DisputeRespondent,
}
```

---

## 🗄️ 存储项

| 存储项 | 类型 | 说明 |
|-------|------|-----|
| `Disputed` | `StorageDoubleMap<Blake2_128Concat, [u8; 8], Blake2_128Concat, u64, ()>` | 争议登记：(domain, id) → () |
| `EvidenceIds` | `StorageDoubleMap<Blake2_128Concat, [u8; 8], Blake2_128Concat, u64, BoundedVec<u64>>` | 证据引用列表：(domain, id) → [evidence_id] |
| `TwoWayDeposits` | `StorageDoubleMap<Blake2_128Concat, [u8; 8], Blake2_128Concat, u64, TwoWayDepositRecord>` | 双向押金记录：(domain, id) → deposit_record |

---

## 📡 事件定义

### 争议事件

```rust
/// 发起争议事件（含域）
Disputed {
    domain: [u8; 8],
    id: u64,
}

/// 完成裁决事件（含域）
Arbitrated {
    domain: [u8; 8],
    id: u64,
    decision: u8,        // 0=Release, 1=Refund, 2=Partial
    bps: Option<u16>,    // 部分裁决比例（仅 decision=2 时有值）
}
```

### 双向押金事件

```rust
/// 发起纠纷并锁定押金
DisputeWithDepositInitiated {
    domain: [u8; 8],
    id: u64,
    initiator: T::AccountId,
    respondent: T::AccountId,
    deposit: BalanceOf<T>,
    deadline: BlockNumberFor<T>,
}

/// 应诉方锁定押金
RespondentDepositLocked {
    domain: [u8; 8],
    id: u64,
    respondent: T::AccountId,
    deposit: BalanceOf<T>,
}

/// 押金已处理（罚没或释放）
DepositProcessed {
    domain: [u8; 8],
    id: u64,
    account: T::AccountId,
    released: BalanceOf<T>,
    slashed: BalanceOf<T>,
}
```

---

## ❌ 错误定义

```rust
pub enum Error<T> {
    /// 争议已存在（防止重复登记）
    AlreadyDisputed,

    /// 争议不存在（仲裁时需先登记）
    NotDisputed,

    /// 押金不足（托管余额不足以锁定押金）
    InsufficientDeposit,

    /// 已经应诉（不能重复应诉）
    AlreadyResponded,

    /// 应诉期已过（超过截止时间）
    ResponseDeadlinePassed,

    /// 无法获取对方账户（Router 返回错误）
    CounterpartyNotFound,
}
```

---

## ⚙️ 配置参数

### Runtime 配置示例

```rust
parameter_types! {
    pub const ArbitrationMaxEvidence: u32 = 100;
    pub const ArbitrationMaxCidLen: u32 = 64;
    pub const ArbitrationDepositRatioBps: u16 = 1500;  // 15%
    pub const ArbitrationResponseDeadline: BlockNumber = 100800;  // 7 天
    pub const ArbitrationRejectedSlashBps: u16 = 3000;  // 30%
    pub const ArbitrationPartialSlashBps: u16 = 5000;  // 50%
    pub TreasuryAccountId: AccountId = AccountId::from([0u8; 32]);
}

impl pallet_arbitration::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxEvidence = ArbitrationMaxEvidence;
    type MaxCidLen = ArbitrationMaxCidLen;
    type Escrow = Escrow;
    type WeightInfo = pallet_arbitration::weights::SubstrateWeight<Runtime>;
    type Router = ArbitrationRouterImpl;
    type DecisionOrigin = EnsureRoot<AccountId>;
    type Fungible = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type DepositRatioBps = ArbitrationDepositRatioBps;
    type ResponseDeadline = ArbitrationResponseDeadline;
    type RejectedSlashBps = ArbitrationRejectedSlashBps;
    type PartialSlashBps = ArbitrationPartialSlashBps;
    type TreasuryAccount = TreasuryAccountId;
}
```

---

## 💻 TypeScript 前端示例

### 示例 1：提交争议

```typescript
import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { blake2AsHex } from '@polkadot/util-crypto';

// 连接到节点
const provider = new WsProvider('ws://localhost:9944');
const api = await ApiPromise.create({ provider });

// 准备账户
const keyring = new Keyring({ type: 'sr25519' });
const buyer = keyring.addFromUri('//Alice');

// 步骤 1：计算证据承诺哈希
const domain = new Uint8Array([111, 116, 99, 95, 111, 114, 100, 95]); // "otc_ord_"
const orderId = 12345;
const evidenceCid = 'enc-QmBuyerEvidence';
const salt = 'random_salt_12345678';
const version = 1;

const preimage = new Uint8Array([
  ...domain,
  ...new Uint8Array(new BigUint64Array([BigInt(orderId)]).buffer),
  ...new TextEncoder().encode(evidenceCid),
  ...new TextEncoder().encode(salt),
  ...new Uint8Array(new Uint32Array([version]).buffer),
]);

const commit = blake2AsHex(preimage, 256);

// 步骤 2：提交证据
const commitEvidenceTx = api.tx.evidence.commitHash(
  Array.from(domain),
  orderId,
  commit,
  null
);

let evidenceId;
await commitEvidenceTx.signAndSend(buyer, ({ status, events }) => {
  if (status.isInBlock) {
    events.forEach(({ event }) => {
      if (api.events.evidence.EvidenceCommittedV2.is(event)) {
        evidenceId = event.data[0].toNumber();
        console.log(`证据已提交：ID=${evidenceId}`);
      }
    });
  }
});

// 步骤 3：发起双向押金争议
const disputeTx = api.tx.arbitration.disputeWithTwoWayDeposit(
  Array.from(domain),
  orderId,
  evidenceId
);

await disputeTx.signAndSend(buyer, ({ status, events }) => {
  if (status.isInBlock) {
    console.log(`争议已登记，交易在区块 ${status.asInBlock}`);

    events.forEach(({ event }) => {
      if (api.events.arbitration.DisputeWithDepositInitiated.is(event)) {
        const [dom, id, initiator, respondent, deposit, deadline] = event.data;
        console.log(`发起方：${initiator}`);
        console.log(`应诉方：${respondent}`);
        console.log(`押金：${deposit.toString()} DUST`);
        console.log(`应诉截止：块 ${deadline.toNumber()}`);
      }
    });
  }
});
```

---

### 示例 2：查询争议状态

```typescript
// 查询是否已登记争议
const isDisputed = await api.query.arbitration.disputed(
  Array.from(domain),
  orderId
);

if (isDisputed.isSome) {
  console.log('争议已登记');
} else {
  console.log('争议未登记');
}

// 查询双向押金记录
const depositRecord = await api.query.arbitration.twoWayDeposits(
  Array.from(domain),
  orderId
);

if (depositRecord.isSome) {
  const record = depositRecord.unwrap();
  console.log('发起方：', record.initiator.toString());
  console.log('发起方押金：', record.initiatorDeposit.toString());
  console.log('应诉方：', record.respondent.toString());
  console.log('应诉方押金：', record.respondentDeposit.toHuman());
  console.log('应诉截止：', record.responseDeadline.toNumber());
  console.log('是否已应诉：', record.hasResponded.toHuman());
}
```

---

## 🔗 集成说明

### 与 pallet-escrow 集成

**集成点**：
1. 争议时标记托管为 Disputed 状态
2. 裁决时调用 apply_decision_* 接口
3. 双向押金从托管账户锁定和释放

---

### 与 pallet-evidence 集成

**集成点**：
1. 通过 evidence_id 引用证据
2. 支持多轮举证（append_evidence_id）
3. 证据本体存储在 pallet-evidence 中

---

## 📌 最佳实践

### 1. 双向押金机制

**优点**：
- ✅ 防止恶意发起争议
- ✅ 双方都有损失风险，促进和解
- ✅ 押金从托管扣除，无需额外转账

**使用建议**：
- 押金比例：15%（DepositRatioBps=1500）
- 应诉期限：7 天（ResponseDeadline=100800 块）
- 罚没比例：败诉方 30%，部分胜诉各 50%

---

## 📚 相关文档

- [pallet-escrow README](../escrow/README.md) - 托管系统文档
- [pallet-evidence README](../evidence/README.md) - 证据管理文档
- [Polkadot SDK 文档](https://docs.substrate.io/)
- [Stardust 项目总览](../../README.md)

---

## 📄 许可证

MIT-0

---

**最后更新**：2025-11-11
**版本**：v0.2.0
**维护者**：Stardust Team
