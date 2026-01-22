# Pallet Arbitration（仲裁争议处理系统）

## 📋 模块概述

`pallet-arbitration` 是 Stardust 区块链的**仲裁争议处理系统**，提供去中心化的争议登记、证据管理、仲裁裁决、资金分账、双向押金管理以及**统一投诉系统**等完整的纠纷解决功能。本模块通过域路由架构（`ArbitrationRouter`）实现与业务 pallet 的低耦合集成，支持 OTC 交易、直播、占卜服务、聊天、NFT 交易等 12 个业务域的争议处理。

### 核心特性

- ✅ **域路由架构**：通过 8 字节域常量标识业务场景，支持多业务统一仲裁
- ✅ **双向押金机制**：发起方与应诉方各自从托管账户锁定 15% 订单金额作为押金
- ✅ **灵活裁决系统**：支持全额释放、全额退款、按比例分配三种裁决方式
- ✅ **证据引用管理**：与 `pallet-evidence` 集成，通过 evidence_id 引用证据
- ✅ **CID 锁定机制**：仲裁期间自动锁定证据 CID，防止证据被删除
- ✅ **托管集成**：与 `pallet-escrow` 深度集成，自动执行资金分账
- ✅ **治理授权**：仅允许 Root 或治理委员会执行裁决，确保公正性
- ✅ **应诉期限机制**：设置应诉截止期，超时未应诉视为弃权
- ✅ **押金罚没规则**：败诉方押金罚没 30%，部分胜诉各罚没 50%
- ✅ **信用分集成**：仲裁结果自动反馈到信用系统，做市商败诉扣分
- ✅ **统一投诉系统**：支持 56 种投诉类型，覆盖 12 个业务域
- ✅ **存储膨胀防护**：自动归档已解决的仲裁和投诉记录

---

## 🏗️ 系统架构

### 两大子系统

本模块包含两个相对独立的子系统：

| 子系统 | 功能定位 | 适用场景 |
|-------|---------|---------|
| **仲裁系统** | 处理托管资金争议 | OTC 订单、Bridge 兑换等涉及资金托管的场景 |
| **投诉系统** | 处理行为/内容投诉 | 直播违规、聊天骚扰、服务质量等非资金争议 |


---

## 🔑 核心数据结构

### Decision（裁决类型）

```rust
pub enum Decision {
    /// 全额释放给收款人（卖家胜诉）
    Release,
    /// 全额退款给付款人（买家胜诉）
    Refund,
    /// 按比例分配（部分胜诉），bps 为释放比例（0-10000）
    Partial(u16),
}
```

### ComplaintType（投诉类型枚举）

支持 56 种投诉类型，覆盖 12 个业务域：

| 业务域 | 域标识 | 投诉类型示例 |
|-------|-------|-------------|
| OTC 交易 | `otc_ord_` | OtcSellerNotDeliver, OtcBuyerFalseClaim, OtcTradeFraud, OtcPriceDispute |
| 直播 | `livstrm_` | LiveIllegalContent, LiveFalseAdvertising, LiveHarassment, LiveFraud, LiveGiftRefund |
| 占卜服务 | `divine__` | DivinePornography, DivineGambling, DivineDrugs, DivineFraud, DivineAbuse 等 |
| 聊天 | `chat____` | ChatHarassment, ChatFraud, ChatIllegalContent, ChatPrivateHarassment |
| 群组 | `chatgrp_` | GroupIllegalContent, GroupHarassment, GroupFraud, GroupSpam, GroupAdminAbuse |
| 做市商 | `maker___` | MakerCreditDefault, MakerMaliciousOperation, MakerFalseQuote |
| NFT 交易 | `nft_trd_` | NftSellerNotDeliver, NftCounterfeit, NftTradeFraud, NftAuctionDispute |
| Swap 交换 | `swap____` | SwapMakerNotComplete, SwapVerificationTimeout, SwapFraud |
| 联系人 | `contact_` | ContactRequestHarassment, ContactBlockAppeal |
| 会员 | `member__` | MemberBenefitNotProvided, MemberServiceQuality |
| 推荐分成 | `affiliat` | AffiliateCommissionDispute, AffiliateRelationDispute |
| 信用系统 | `credit__` | CreditScoreDispute, CreditPenaltyAppeal |

每种投诉类型包含：
- `domain()` - 获取所属业务域
- `penalty_rate()` - 获取惩罚比例（基点）
- `triggers_permanent_ban()` - 是否触发永久封禁

### ComplaintStatus（投诉状态）

```rust
pub enum ComplaintStatus {
    Submitted,              // 已提交，等待响应
    Responded,              // 已响应/申诉
    Mediating,              // 调解中
    Arbitrating,            // 仲裁中
    ResolvedComplainantWin, // 已解决 - 投诉方胜诉
    ResolvedRespondentWin,  // 已解决 - 被投诉方胜诉
    ResolvedSettlement,     // 已解决 - 和解
    Withdrawn,              // 已撤销
    Expired,                // 已过期
}
```

### Complaint（投诉记录）

```rust
pub struct Complaint<T: Config> {
    pub id: u64,                                    // 投诉唯一ID
    pub domain: [u8; 8],                            // 业务域标识
    pub object_id: u64,                             // 业务对象ID
    pub complaint_type: ComplaintType,              // 投诉类型
    pub complainant: T::AccountId,                  // 投诉发起人
    pub respondent: T::AccountId,                   // 被投诉人
    pub details_cid: BoundedVec<u8, T::MaxCidLen>,  // 详情CID（指向IPFS）
    pub amount: Option<BalanceOf<T>>,               // 涉及金额
    pub status: ComplaintStatus,                    // 当前状态
    pub created_at: BlockNumberFor<T>,              // 创建时间
    pub response_deadline: BlockNumberFor<T>,       // 响应截止时间
    pub updated_at: BlockNumberFor<T>,              // 最后更新时间
}
```

### TwoWayDepositRecord（双向押金记录）

```rust
pub struct TwoWayDepositRecord<AccountId, Balance, BlockNumber> {
    pub initiator: AccountId,                // 发起方账户
    pub initiator_deposit: Balance,          // 发起方押金金额
    pub respondent: AccountId,               // 应诉方账户
    pub respondent_deposit: Option<Balance>, // 应诉方押金金额（未应诉时为 None）
    pub response_deadline: BlockNumber,      // 应诉截止区块
    pub has_responded: bool,                 // 是否已应诉
}
```

### HoldReason（押金锁定原因）

```rust
pub enum HoldReason {
    DisputeInitiator,   // 纠纷发起方押金
    DisputeRespondent,  // 应诉方押金
    ComplaintDeposit,   // 投诉押金（防止恶意投诉）
}
```


---

## 📞 Extrinsics（可调用函数）

### 仲裁系统 Extrinsics

#### `dispute` (call_index: 0)

**功能**：发起仲裁，记录争议并提交证据 CID（旧版接口，兼容性保留）

**调用方**：授权账户（通过 `ArbitrationRouter::can_dispute` 验证）

```rust
pub fn dispute(
    origin: OriginFor<T>,
    domain: [u8; 8],                                    // 域标识
    id: u64,                                            // 订单/交易 ID
    _evidence: Vec<BoundedVec<u8, T::MaxCidLen>>,      // 证据 CID 列表
) -> DispatchResult
```

#### `arbitrate` (call_index: 1)

**功能**：仲裁者执行裁决（仅治理起源）

**调用方**：Root 或治理委员会

```rust
pub fn arbitrate(
    origin: OriginFor<T>,
    domain: [u8; 8],          // 域标识
    id: u64,                  // 订单/交易 ID
    decision_code: u8,        // 裁决类型（0=Release, 1=Refund, 2=Partial）
    bps: Option<u16>,         // 部分裁决比例（仅 decision_code=2 时需要）
) -> DispatchResult
```

**裁决后处理**：
1. 调用 `Router.apply_decision` 执行业务逻辑
2. 处理双向押金（罚没/释放）
3. 解锁仲裁期间锁定的证据 CID
4. 更新做市商信用分（如适用）
5. 归档仲裁记录并清理存储

#### `dispute_with_evidence_id` (call_index: 2)

**功能**：按证据 ID 登记争议（推荐方式）

```rust
pub fn dispute_with_evidence_id(
    origin: OriginFor<T>,
    domain: [u8; 8],
    id: u64,
    evidence_id: u64,         // 证据 ID（来自 pallet-evidence）
) -> DispatchResult
```

#### `append_evidence_id` (call_index: 3)

**功能**：为已登记的争议追加新证据

```rust
pub fn append_evidence_id(
    origin: OriginFor<T>,
    domain: [u8; 8],
    id: u64,
    evidence_id: u64,
) -> DispatchResult
```

#### `dispute_with_two_way_deposit` (call_index: 4)

**功能**：以双向押金方式发起纠纷（推荐方式）

```rust
pub fn dispute_with_two_way_deposit(
    origin: OriginFor<T>,
    domain: [u8; 8],
    id: u64,
    evidence_id: u64,
) -> DispatchResult
```

**处理流程**：
1. 权限校验（Router.can_dispute）
2. 获取订单金额（Router.get_order_amount）
3. 计算押金金额（订单金额 × 15%）
4. 从托管账户锁定发起方押金
5. 获取应诉方账户（Router.get_counterparty）
6. 设置应诉截止期限（当前块 + ResponseDeadline）
7. 登记争议和双向押金记录
8. 添加证据引用

#### `respond_to_dispute` (call_index: 5)

**功能**：应诉方从托管锁定押金并提交反驳证据

```rust
pub fn respond_to_dispute(
    origin: OriginFor<T>,
    domain: [u8; 8],
    id: u64,
    counter_evidence_id: u64,
) -> DispatchResult
```

**处理流程**：
1. 验证是应诉方
2. 确保未应诉且未超时
3. 从托管账户锁定应诉方押金（与发起方相同金额）
4. 更新押金记录
5. 添加反驳证据


### 投诉系统 Extrinsics

#### `file_complaint` (call_index: 10)

**功能**：发起投诉（需缴纳押金防止恶意投诉）

```rust
pub fn file_complaint(
    origin: OriginFor<T>,
    domain: [u8; 8],                              // 业务域
    object_id: u64,                               // 业务对象ID
    complaint_type: ComplaintType,                // 投诉类型
    details_cid: BoundedVec<u8, T::MaxCidLen>,   // 详情CID
    amount: Option<BalanceOf<T>>,                 // 涉及金额（可选）
) -> DispatchResult
```

**押金计算**：
- 使用 Pricing 接口换算 1 USDT 价值的 DUST
- 如 Pricing 不可用，使用 `ComplaintDeposit` 兜底值

#### `respond_to_complaint` (call_index: 11)

**功能**：响应/申诉投诉

```rust
pub fn respond_to_complaint(
    origin: OriginFor<T>,
    complaint_id: u64,
    response_cid: BoundedVec<u8, T::MaxCidLen>,  // 申诉内容CID
) -> DispatchResult
```

**限制**：
- 仅被投诉人可调用
- 状态必须为 `Submitted`
- 必须在响应截止时间前

#### `withdraw_complaint` (call_index: 12)

**功能**：撤销投诉

```rust
pub fn withdraw_complaint(
    origin: OriginFor<T>,
    complaint_id: u64,
) -> DispatchResult
```

**限制**：
- 仅投诉人可调用
- 状态必须为 `Submitted` 或 `Responded`

#### `settle_complaint` (call_index: 13)

**功能**：达成和解

```rust
pub fn settle_complaint(
    origin: OriginFor<T>,
    complaint_id: u64,
    settlement_cid: BoundedVec<u8, T::MaxCidLen>,  // 和解协议CID
) -> DispatchResult
```

**限制**：
- 投诉人或被投诉人均可调用
- 状态必须为 `Responded` 或 `Mediating`

#### `escalate_to_arbitration` (call_index: 14)

**功能**：提交仲裁（升级到仲裁委员会）

```rust
pub fn escalate_to_arbitration(
    origin: OriginFor<T>,
    complaint_id: u64,
) -> DispatchResult
```

#### `resolve_complaint` (call_index: 15)

**功能**：仲裁裁决投诉（仅仲裁委员会/Root）

```rust
pub fn resolve_complaint(
    origin: OriginFor<T>,
    complaint_id: u64,
    decision: u8,                                  // 0=投诉方胜, 1=被投诉方胜, 2=和解
    reason_cid: BoundedVec<u8, T::MaxCidLen>,     // 裁决理由CID
) -> DispatchResult
```

**押金处理**：
- 投诉方胜诉：全额退还押金
- 被投诉方胜诉：罚没部分押金（ComplaintSlashBps）给被投诉方
- 和解：全额退还押金


---

## 🗄️ 存储项

### 仲裁系统存储

| 存储项 | 类型 | 说明 |
|-------|------|-----|
| `Disputed` | `StorageDoubleMap<[u8;8], u64, ()>` | 争议登记：(domain, id) → () |
| `EvidenceIds` | `StorageDoubleMap<[u8;8], u64, BoundedVec<u64>>` | 证据引用列表：(domain, id) → [evidence_id] |
| `TwoWayDeposits` | `StorageDoubleMap<[u8;8], u64, TwoWayDepositRecord>` | 双向押金记录 |
| `LockedCidHashes` | `StorageDoubleMap<[u8;8], u64, BoundedVec<Hash>>` | 锁定的 CID 哈希列表 |
| `NextArchivedId` | `StorageValue<u64>` | 下一个归档ID |
| `ArchivedDisputes` | `StorageMap<u64, ArchivedDispute>` | 归档仲裁记录 |
| `ArbitrationStats` | `StorageValue<ArbitrationPermanentStats>` | 仲裁永久统计 |

### 投诉系统存储

| 存储项 | 类型 | 说明 |
|-------|------|-----|
| `NextComplaintId` | `StorageValue<u64>` | 投诉ID计数器 |
| `Complaints` | `StorageMap<u64, Complaint>` | 活跃投诉主存储 |
| `ArchivedComplaints` | `StorageMap<u64, ArchivedComplaint>` | 归档投诉存储 |
| `UserActiveComplaints` | `StorageMap<AccountId, BoundedVec<u64, 50>>` | 用户活跃投诉索引 |
| `ComplaintDeposits` | `StorageMap<u64, Balance>` | 投诉押金记录 |
| `DomainStats` | `StorageMap<[u8;8], DomainStatistics>` | 域统计信息 |
| `ComplaintArchiveCursor` | `StorageValue<u64>` | 投诉归档游标 |

---

## 📡 事件定义

### 仲裁系统事件

```rust
/// 发起争议事件
Disputed { domain: [u8; 8], id: u64 }

/// 完成裁决事件
Arbitrated { domain: [u8; 8], id: u64, decision: u8, bps: Option<u16> }

/// 发起纠纷并锁定押金
DisputeWithDepositInitiated {
    domain: [u8; 8],
    id: u64,
    initiator: AccountId,
    respondent: AccountId,
    deposit: Balance,
    deadline: BlockNumber,
}

/// 应诉方锁定押金
RespondentDepositLocked { domain: [u8; 8], id: u64, respondent: AccountId, deposit: Balance }

/// 押金已处理（罚没或释放）
DepositProcessed { domain: [u8; 8], id: u64, account: AccountId, released: Balance, slashed: Balance }
```

### 投诉系统事件

```rust
/// 投诉已提交
ComplaintFiled {
    complaint_id: u64,
    domain: [u8; 8],
    object_id: u64,
    complainant: AccountId,
    respondent: AccountId,
    complaint_type: ComplaintType,
}

/// 投诉已响应/申诉
ComplaintResponded { complaint_id: u64, respondent: AccountId }

/// 投诉已撤销
ComplaintWithdrawn { complaint_id: u64 }

/// 投诉已和解
ComplaintSettled { complaint_id: u64 }

/// 投诉已升级到仲裁
ComplaintEscalated { complaint_id: u64 }

/// 投诉已裁决
ComplaintResolved { complaint_id: u64, decision: u8 }

/// 投诉已过期
ComplaintExpired { complaint_id: u64 }

/// 投诉已归档
ComplaintArchived { complaint_id: u64 }
```


---

## ❌ 错误定义

```rust
pub enum Error<T> {
    // 仲裁系统错误
    AlreadyDisputed,           // 争议已存在
    NotDisputed,               // 争议不存在
    InsufficientDeposit,       // 押金不足
    AlreadyResponded,          // 已经应诉
    ResponseDeadlinePassed,    // 应诉期已过
    CounterpartyNotFound,      // 无法获取对方账户

    // 投诉系统错误
    ComplaintNotFound,         // 投诉不存在
    NotAuthorized,             // 无权操作
    InvalidComplaintType,      // 无效的投诉类型（与域不匹配）
    InvalidState,              // 无效的状态转换
    TooManyComplaints,         // 该对象投诉数量过多
    TooManyActiveComplaints,   // 用户活跃投诉数量已达上限（50个）
}
```

---

## ⚙️ 配置参数

### Config Trait

```rust
pub trait Config: frame_system::Config + pallet_escrow::Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    
    /// 最大证据数量
    type MaxEvidence: Get<u32>;
    
    /// 最大 CID 长度
    type MaxCidLen: Get<u32>;
    
    /// 托管接口
    type Escrow: EscrowTrait<Self::AccountId, BalanceOf<Self>>;
    
    /// 权重信息
    type WeightInfo: weights::WeightInfo;
    
    /// 域路由器
    type Router: ArbitrationRouter<Self::AccountId, BalanceOf<Self>>;
    
    /// 仲裁决策起源（治理）
    type DecisionOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    
    /// Fungible 接口（用于押金锁定）
    type Fungible: FungibleInspect<Self::AccountId> + FungibleMutate<Self::AccountId> 
        + FungibleMutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;
    
    /// RuntimeHoldReason
    type RuntimeHoldReason: From<HoldReason>;
    
    /// 押金比例（基点，1500 = 15%）
    type DepositRatioBps: Get<u16>;
    
    /// 应诉期限（区块数，默认 7 天）
    type ResponseDeadline: Get<BlockNumberFor<Self>>;
    
    /// 驳回罚没比例（基点，3000 = 30%）
    type RejectedSlashBps: Get<u16>;
    
    /// 部分胜诉罚没比例（基点，5000 = 50%）
    type PartialSlashBps: Get<u16>;
    
    /// 投诉押金兜底金额
    #[pallet::constant]
    type ComplaintDeposit: Get<BalanceOf<Self>>;
    
    /// 投诉押金USD价值（精度10^6，1_000_000 = 1 USDT）
    #[pallet::constant]
    type ComplaintDepositUsd: Get<u64>;
    
    /// 定价接口
    type Pricing: PricingProvider<BalanceOf<Self>>;
    
    /// 投诉败诉罚没比例（基点，5000 = 50%）
    #[pallet::constant]
    type ComplaintSlashBps: Get<u16>;
    
    /// 国库账户
    type TreasuryAccount: Get<Self::AccountId>;
    
    /// CID 锁定管理器
    type CidLockManager: CidLockManager<Self::Hash, BlockNumberFor<Self>>;
    
    /// 信用分更新器
    type CreditUpdater: CreditUpdater;
}
```

### Runtime 配置示例

```rust
parameter_types! {
    pub const ArbitrationMaxEvidence: u32 = 100;
    pub const ArbitrationMaxCidLen: u32 = 64;
    pub const ArbitrationDepositRatioBps: u16 = 1500;      // 15%
    pub const ArbitrationResponseDeadline: BlockNumber = 100800;  // 7 天
    pub const ArbitrationRejectedSlashBps: u16 = 3000;     // 30%
    pub const ArbitrationPartialSlashBps: u16 = 5000;      // 50%
    pub const ComplaintDeposit: Balance = 10_000_000_000;  // 10 DUST
    pub const ComplaintDepositUsd: u64 = 1_000_000;        // 1 USDT
    pub const ComplaintSlashBps: u16 = 5000;               // 50%
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
    type ComplaintDeposit = ComplaintDeposit;
    type ComplaintDepositUsd = ComplaintDepositUsd;
    type Pricing = TradingPricing;
    type ComplaintSlashBps = ComplaintSlashBps;
    type TreasuryAccount = TreasuryAccountId;
    type CidLockManager = StardustIpfs;
    type CreditUpdater = TradingCreditUpdater;
}
```


---

## 🔌 ArbitrationRouter Trait

域路由接口，由 runtime 实现，根据域将仲裁请求路由到对应业务 pallet。

```rust
pub trait ArbitrationRouter<AccountId, Balance> {
    /// 校验是否允许发起争议
    fn can_dispute(domain: [u8; 8], who: &AccountId, id: u64) -> bool;
    
    /// 应用裁决（放款/退款/部分放款）
    fn apply_decision(domain: [u8; 8], id: u64, decision: Decision) -> DispatchResult;
    
    /// 获取纠纷对方账户
    fn get_counterparty(domain: [u8; 8], initiator: &AccountId, id: u64) -> Result<AccountId, DispatchError>;
    
    /// 获取订单/交易金额（用于计算押金）
    fn get_order_amount(domain: [u8; 8], id: u64) -> Result<Balance, DispatchError>;
    
    /// 获取做市商ID（用于信用分更新，仅OTC域有效）
    fn get_maker_id(domain: [u8; 8], id: u64) -> Option<u64> { None }
}
```

---

## 🔄 CreditUpdater Trait

信用分更新接口，用于仲裁结果反馈到信用系统。

```rust
pub trait CreditUpdater {
    /// 记录做市商争议结果
    /// - maker_id: 做市商ID
    /// - order_id: 订单ID
    /// - maker_win: 做市商是否胜诉
    fn record_maker_dispute_result(maker_id: u64, order_id: u64, maker_win: bool) -> DispatchResult;
}
```

---

## ⏰ Hooks 实现

模块在 `on_idle` 中执行后台任务：

```rust
fn on_idle(_now: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
    // 阶段1：处理过期投诉（每次最多5个）
    Self::expire_old_complaints(5);
    
    // 阶段2：归档已解决投诉（每次最多10个）
    Self::archive_old_complaints(10);
}
```

**归档规则**：
- 归档延迟：30 天（432000 区块）
- 状态为已解决（ResolvedComplainantWin/ResolvedRespondentWin/ResolvedSettlement/Withdrawn/Expired）
- 归档后从 `Complaints` 移动到 `ArchivedComplaints`

**过期规则**：
- 状态为 `Submitted` 且已过响应截止时间
- 自动标记为 `Expired` 状态


---

## 💻 使用示例

### 示例 1：双向押金争议流程

```rust
// 1. 买家提交证据
let evidence_id = Evidence::commit_hash(
    RuntimeOrigin::signed(buyer),
    *b"otc_ord_",
    order_id,
    buyer_evidence_commit,
    None,
)?;

// 2. 买家发起双向押金争议
Arbitration::dispute_with_two_way_deposit(
    RuntimeOrigin::signed(buyer),
    *b"otc_ord_",
    order_id,
    evidence_id,
)?;

// 3. 卖家提交反驳证据并应诉
let counter_evidence_id = Evidence::commit_hash(
    RuntimeOrigin::signed(seller),
    *b"otc_ord_",
    order_id,
    seller_evidence_commit,
    None,
)?;

Arbitration::respond_to_dispute(
    RuntimeOrigin::signed(seller),
    *b"otc_ord_",
    order_id,
    counter_evidence_id,
)?;

// 4. 仲裁委员会裁决（卖家胜诉）
Arbitration::arbitrate(
    RuntimeOrigin::root(),
    *b"otc_ord_",
    order_id,
    0,      // Release
    None,
)?;
```

### 示例 2：投诉流程

```rust
// 1. 用户发起投诉
Arbitration::file_complaint(
    RuntimeOrigin::signed(complainant),
    *b"livstrm_",                                    // 直播域
    livestream_id,
    ComplaintType::LiveIllegalContent,              // 直播违规内容
    BoundedVec::try_from(b"QmComplaintDetails".to_vec()).unwrap(),
    None,
)?;

// 2. 被投诉人响应
Arbitration::respond_to_complaint(
    RuntimeOrigin::signed(respondent),
    complaint_id,
    BoundedVec::try_from(b"QmResponseDetails".to_vec()).unwrap(),
)?;

// 3. 升级到仲裁
Arbitration::escalate_to_arbitration(
    RuntimeOrigin::signed(complainant),
    complaint_id,
)?;

// 4. 仲裁裁决
Arbitration::resolve_complaint(
    RuntimeOrigin::root(),
    complaint_id,
    0,  // 投诉方胜诉
    BoundedVec::try_from(b"QmDecisionReason".to_vec()).unwrap(),
)?;
```

### 示例 3：TypeScript 前端调用

```typescript
import { ApiPromise, WsProvider } from '@polkadot/api';

const api = await ApiPromise.create({ provider: new WsProvider('ws://localhost:9944') });

// 发起投诉
const domain = new Uint8Array([108, 105, 118, 115, 116, 114, 109, 95]); // "livstrm_"
const tx = api.tx.arbitration.fileComplaint(
    Array.from(domain),
    livestreamId,
    'LiveIllegalContent',
    '0x' + Buffer.from('QmComplaintDetails').toString('hex'),
    null
);

await tx.signAndSend(complainant, ({ status, events }) => {
    if (status.isInBlock) {
        events.forEach(({ event }) => {
            if (api.events.arbitration.ComplaintFiled.is(event)) {
                const [complaintId] = event.data;
                console.log(`投诉已提交：ID=${complaintId}`);
            }
        });
    }
});

// 查询投诉状态
const complaint = await api.query.arbitration.complaints(complaintId);
if (complaint.isSome) {
    const data = complaint.unwrap();
    console.log('状态:', data.status.toString());
    console.log('投诉人:', data.complainant.toString());
}

// 查询域统计
const stats = await api.query.arbitration.domainStats(Array.from(domain));
console.log('总投诉数:', stats.totalComplaints.toString());
console.log('已解决数:', stats.resolvedCount.toString());
```


---

## 📊 押金处理规则

### 仲裁押金处理

| 裁决结果 | 发起方押金 | 应诉方押金 | 罚没去向 |
|---------|----------|----------|---------|
| **Release（卖家胜诉）** | 罚没 30%，70% 返还托管 | 全额返还托管 | 国库 |
| **Refund（买家胜诉）** | 全额返还托管 | 罚没 30%，70% 返还托管 | 国库 |
| **Partial（部分胜诉）** | 罚没 50%，50% 返还托管 | 罚没 50%，50% 返还托管 | 国库 |

### 投诉押金处理

| 裁决结果 | 押金处理 |
|---------|---------|
| **投诉方胜诉** | 全额退还押金 |
| **被投诉方胜诉** | 罚没 50% 给被投诉方，50% 退还 |
| **和解** | 全额退还押金 |

---

## 🔗 集成说明

### 与 pallet-escrow 集成

- 双向押金从托管账户锁定和释放
- 裁决时调用 `apply_decision` 执行资金分账

### 与 pallet-evidence 集成

- 通过 `evidence_id` 引用证据
- 支持多轮举证（`append_evidence_id`）

### 与 pallet-stardust-ipfs 集成

- 仲裁期间自动锁定证据 CID
- 仲裁完成后自动解锁

### 与 pallet-trading-credit 集成

- 做市商败诉时扣除信用分
- 通过 `CreditUpdater` trait 实现

---

## 📚 相关文档

- [pallet-escrow README](../escrow/README.md) - 托管系统文档
- [pallet-evidence README](../evidence/README.md) - 证据管理文档
- [pallet-trading-credit README](../trading-credit/README.md) - 信用系统文档
- [Polkadot SDK 文档](https://docs.substrate.io/)

---

## 📄 许可证

MIT-0

---

**最后更新**：2025-01-20  
**版本**：v0.3.0  
**维护者**：Stardust Team
