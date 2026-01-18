# Design Document: Bridge-OTC Module Merge

## Overview

本设计文档描述将 `pallet-bridge` 和 `pallet-otc-order` 合并为统一的 `pallet-exchange` 模块的技术方案。合并后的模块将处理所有 DUST ↔ USDT 交易，包括官方桥接、做市商桥接、OTC 订单和首购订单。

### 设计目标

1. **代码复用**: 消除两个模块间的重复代码（约 40% 相似度）
2. **统一接口**: 提供一致的 API 和事件结构
3. **简化维护**: 减少模块数量，降低维护成本
4. **保持功能**: 100% 保留现有功能
5. **平滑迁移**: 提供存储迁移方案

### 当前架构分析

```
现有架构:
┌─────────────────┐     ┌─────────────────┐
│  pallet-bridge  │     │ pallet-otc-order│
├─────────────────┤     ├─────────────────┤
│ SwapRequest     │     │ Order           │
│ MakerSwapRecord │     │ OrderState      │
│ SwapStatus      │     │ KycConfig       │
├─────────────────┤     ├─────────────────┤
│ swap()          │     │ create_order()  │
│ maker_swap()    │     │ mark_paid()     │
│ mark_complete() │     │ release_dust()  │
│ report_swap()   │     │ dispute_order() │
└─────────────────┘     └─────────────────┘
        │                       │
        └───────────┬───────────┘
                    ▼
            共同依赖:
            - pallet-escrow
            - pallet-maker
            - pallet-pricing
            - pallet-credit
```

## Architecture

### 合并后架构

```
合并后架构:
┌─────────────────────────────────────────┐
│           pallet-exchange               │
├─────────────────────────────────────────┤
│  ExchangeRecord (统一记录结构)           │
│  ExchangeType (类型枚举)                 │
│  ExchangeStatus (统一状态枚举)           │
│  KycConfig (KYC配置)                    │
├─────────────────────────────────────────┤
│  official_swap()      - 官方桥接        │
│  complete_official()  - 完成官方桥接     │
│  maker_swap()         - 做市商桥接       │
│  mark_swap_complete() - 标记桥接完成     │
│  create_order()       - 创建OTC订单      │
│  create_first_purchase() - 首购订单     │
│  mark_paid()          - 标记已付款       │
│  release_dust()       - 释放DUST        │
│  cancel_exchange()    - 取消交易         │
│  dispute_exchange()   - 发起争议         │
│  report_exchange()    - 举报交易         │
├─────────────────────────────────────────┤
│  KYC管理函数 (enable/disable/exempt)    │
└─────────────────────────────────────────┘
            │
            ▼
    依赖模块 (不变):
    - pallet-escrow
    - pallet-maker  
    - pallet-pricing
    - pallet-trading-credit
    - pallet-chat-permission
    - pallet-arbitration
```

### 模块目录结构

```
pallets/trading/exchange/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs           # 主模块定义
    ├── types.rs         # 数据类型定义
    ├── kyc.rs           # KYC验证逻辑
    ├── official.rs      # 官方桥接实现
    ├── maker_swap.rs    # 做市商桥接实现
    ├── otc.rs           # OTC订单实现
    ├── first_purchase.rs # 首购订单实现
    ├── arbitration.rs   # 仲裁接口实现
    ├── migration.rs     # 存储迁移逻辑
    ├── weights.rs       # 权重定义
    ├── mock.rs          # 测试mock
    └── tests.rs         # 单元测试
```

## Components and Interfaces

### 核心 Trait 定义

```rust
/// Exchange 模块配置 trait
#[pallet::config]
pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
    /// 货币类型
    type Currency: Currency<Self::AccountId>;
    
    /// 时间戳
    type Timestamp: UnixTime;
    
    /// 托管服务
    type Escrow: pallet_escrow::Escrow<Self::AccountId, BalanceOf<Self>>;
    
    /// 买家信用接口
    type BuyerCredit: pallet_trading_credit::BuyerCreditInterface<Self::AccountId>
        + pallet_trading_credit::quota::BuyerQuotaInterface<Self::AccountId>;
    
    /// 做市商信用接口
    type MakerCredit: MakerCreditInterface;
    
    /// 定价服务
    type Pricing: PricingProvider<BalanceOf<Self>>;
    
    /// 做市商模块
    type MakerPallet: MakerInterface<Self::AccountId, BalanceOf<Self>>;
    
    /// 治理权限
    type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    
    /// 委员会权限 (KYC管理)
    type CommitteeOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    
    /// 身份验证提供者
    type IdentityProvider: IdentityVerificationProvider<Self::AccountId>;
    
    /// 聊天权限管理
    type ChatPermission: SceneAuthorizationManager<Self::AccountId, BlockNumberFor<Self>>;
    
    // === 超时配置 ===
    
    /// 官方桥接超时 (区块数)
    #[pallet::constant]
    type OfficialSwapTimeout: Get<BlockNumberFor<Self>>;
    
    /// 做市商桥接超时 (区块数)
    #[pallet::constant]
    type MakerSwapTimeout: Get<BlockNumberFor<Self>>;
    
    /// OTC订单超时 (毫秒)
    #[pallet::constant]
    type OrderTimeout: Get<u64>;
    
    /// 证据窗口 (毫秒)
    #[pallet::constant]
    type EvidenceWindow: Get<u64>;
    
    // === 金额限制 ===
    
    /// 最小桥接金额
    #[pallet::constant]
    type MinSwapAmount: Get<BalanceOf<Self>>;
    
    /// OTC最小金额 (USD, 精度10^6)
    #[pallet::constant]
    type MinOrderUsdAmount: Get<u64>;
    
    /// OTC最大金额 (USD, 精度10^6)
    #[pallet::constant]
    type MaxOrderUsdAmount: Get<u64>;
    
    // === 首购配置 ===
    
    /// 首购固定USD金额 (精度10^6)
    #[pallet::constant]
    type FirstPurchaseUsdAmount: Get<u64>;
    
    /// 首购最小DUST数量
    #[pallet::constant]
    type MinFirstPurchaseDustAmount: Get<BalanceOf<Self>>;
    
    /// 首购最大DUST数量
    #[pallet::constant]
    type MaxFirstPurchaseDustAmount: Get<BalanceOf<Self>>;
    
    /// 每个做市商最大首购订单数
    #[pallet::constant]
    type MaxFirstPurchaseOrdersPerMaker: Get<u32>;
    
    /// 金额验证容差 (基点)
    #[pallet::constant]
    type AmountValidationTolerance: Get<u16>;
    
    /// 权重信息
    type WeightInfo: WeightInfo;
}
```

### 外部接口

```rust
/// 做市商信用接口 (复用现有定义)
pub trait MakerCreditInterface {
    fn record_maker_order_completed(maker_id: u64, order_id: u64, response_time_seconds: u32) -> DispatchResult;
    fn record_maker_order_timeout(maker_id: u64, order_id: u64) -> DispatchResult;
    fn record_maker_dispute_result(maker_id: u64, order_id: u64, maker_win: bool) -> DispatchResult;
}

/// 定价服务接口 (复用现有定义)
pub trait PricingProvider<Balance> {
    fn get_dust_to_usd_rate() -> Option<Balance>;
}

/// 做市商接口 (复用现有定义)
pub trait MakerInterface<AccountId, Balance> {
    fn get_maker_application(maker_id: u64) -> Option<MakerApplicationInfo<AccountId, Balance>>;
    fn is_maker_active(maker_id: u64) -> bool;
    fn get_maker_id(who: &AccountId) -> Option<u64>;
}

/// 身份验证接口 (复用现有定义)
pub trait IdentityVerificationProvider<AccountId> {
    fn get_highest_judgement_priority(who: &AccountId) -> Option<u8>;
    fn has_problematic_judgement(who: &AccountId) -> bool;
}
```

## Data Models

### 统一交易类型枚举

```rust
/// 交易类型枚举
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ExchangeType {
    /// 官方桥接 (DUST → USDT, 治理管理)
    OfficialBridge,
    /// 做市商桥接 (DUST → USDT, 做市商服务)
    MakerBridge,
    /// OTC订单 (USDT → DUST, 普通购买)
    OtcOrder,
    /// 首购订单 (USDT → DUST, 固定10USD)
    FirstPurchase,
}
```

### 统一状态枚举

```rust
/// 统一交易状态枚举
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ExchangeStatus {
    // === 通用状态 ===
    /// 已创建，等待处理
    Created,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 已退款
    Refunded,
    /// 已过期
    Expired,
    
    // === OTC/首购特有状态 ===
    /// 买家已标记付款
    PaidOrCommitted,
    /// DUST已释放
    Released,
    
    // === 争议相关状态 ===
    /// 用户举报 (桥接)
    UserReported,
    /// 争议中 (OTC)
    Disputed,
    /// 仲裁中
    Arbitrating,
    /// 仲裁通过 (放款给做市商)
    ArbitrationApproved,
    /// 仲裁拒绝 (退款给用户)
    ArbitrationRejected,
}
```

### 统一交易记录结构

```rust
/// 统一交易记录结构
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct ExchangeRecord<T: Config> {
    /// 交易ID
    pub id: u64,
    /// 交易类型
    pub exchange_type: ExchangeType,
    /// 交易状态
    pub status: ExchangeStatus,
    
    // === 参与方信息 ===
    /// 用户账户 (发起方)
    pub user: T::AccountId,
    /// 做市商ID (MakerBridge/OtcOrder/FirstPurchase)
    pub maker_id: Option<u64>,
    /// 做市商账户
    pub maker: Option<T::AccountId>,
    
    // === 金额信息 ===
    /// DUST数量
    pub dust_amount: BalanceOf<T>,
    /// USDT金额 (精度10^6)
    pub usdt_amount: u64,
    /// 成交价格 (DUST/USD, 精度10^6)
    pub price_usdt: u64,
    
    // === 地址信息 ===
    /// TRON地址 (用户接收USDT / 做市商接收USDT)
    pub tron_address: TronAddress,
    
    // === 时间信息 ===
    /// 创建时间 (区块号或时间戳)
    pub created_at: u64,
    /// 超时时间
    pub expire_at: u64,
    /// 证据窗口截止时间 (OTC)
    pub evidence_until: Option<u64>,
    /// 完成时间
    pub completed_at: Option<u64>,
    
    // === OTC特有字段 ===
    /// 支付承诺哈希
    pub payment_commit: Option<H256>,
    /// 联系方式承诺哈希
    pub contact_commit: Option<H256>,
    /// EPAY交易号
    pub epay_trade_no: Option<BoundedVec<u8, ConstU32<64>>>,
    
    // === 桥接特有字段 ===
    /// TRC20交易哈希 (做市商提交)
    pub trc20_tx_hash: Option<BoundedVec<u8, ConstU32<128>>>,
    /// 证据CID
    pub evidence_cid: Option<BoundedVec<u8, ConstU32<256>>>,
}
```

### KYC 配置结构 (复用)

```rust
/// KYC配置结构 (从 pallet-otc-order 复用)
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug, Default)]
pub struct KycConfig<BlockNumber> {
    pub enabled: bool,
    pub min_judgment_priority: u8,
    pub effective_block: BlockNumber,
    pub updated_at: BlockNumber,
}
```

### 存储定义

```rust
// === 主存储 ===

/// 下一个交易ID
#[pallet::storage]
pub type NextExchangeId<T> = StorageValue<_, u64, ValueQuery>;

/// 交易记录
#[pallet::storage]
pub type Exchanges<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,  // exchange_id
    ExchangeRecord<T>,
>;

// === 索引存储 ===

/// 用户交易列表
#[pallet::storage]
pub type UserExchanges<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<u64, ConstU32<200>>,  // 合并后每用户最多200个
    ValueQuery,
>;

/// 做市商交易列表
#[pallet::storage]
pub type MakerExchanges<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,  // maker_id
    BoundedVec<u64, ConstU32<2000>>,  // 合并后每做市商最多2000个
    ValueQuery,
>;

// === 官方桥接特有存储 ===

/// 桥接账户
#[pallet::storage]
pub type BridgeAccount<T: Config> = StorageValue<_, T::AccountId>;

// === 防重放存储 ===

/// 已使用的TRON交易哈希
#[pallet::storage]
pub type UsedTronTxHashes<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    BoundedVec<u8, ConstU32<128>>,
    (),
    OptionQuery,
>;

/// TRON交易哈希队列 (用于清理)
#[pallet::storage]
pub type TronTxQueue<T: Config> = StorageValue<
    _,
    BoundedVec<(H256, BlockNumberFor<T>), ConstU32<10000>>,
    ValueQuery,
>;

// === 首购相关存储 ===

/// 用户是否已首购
#[pallet::storage]
pub type HasFirstPurchased<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    bool,
    ValueQuery,
>;

/// 做市商首购订单计数
#[pallet::storage]
pub type MakerFirstPurchaseCount<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,
    u32,
    ValueQuery,
>;

/// 做市商首购订单列表
#[pallet::storage]
pub type MakerFirstPurchaseOrders<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,
    BoundedVec<u64, ConstU32<10>>,
    ValueQuery,
>;

// === KYC存储 ===

/// KYC配置
#[pallet::storage]
pub type KycConfig<T: Config> = StorageValue<
    _,
    crate::types::KycConfig<BlockNumberFor<T>>,
    ValueQuery,
>;

/// KYC豁免账户
#[pallet::storage]
pub type KycExemptAccounts<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    (),
    OptionQuery,
>;
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*



### Property 1: Data Structure Round-Trip Consistency

*For any* valid ExchangeRecord with any ExchangeType (OfficialBridge, MakerBridge, OtcOrder, FirstPurchase), encoding then decoding the record SHALL produce an equivalent record with all fields preserved.

**Validates: Requirements 1.1, 1.2, 1.3**

### Property 2: Exchange ID Uniqueness

*For any* sequence of exchange creations (regardless of type), all assigned exchange_ids SHALL be unique and form a strictly increasing sequence.

**Validates: Requirements 1.4**

### Property 3: Official Bridge Lifecycle Invariants

*For any* official bridge swap:
- After creation, user's free balance SHALL decrease by dust_amount
- After completion, bridge account SHALL receive the dust_amount
- After timeout refund, user's balance SHALL be restored to pre-swap amount

**Validates: Requirements 3.1, 3.2, 3.3, 3.4**

### Property 4: Maker Bridge Lifecycle Invariants

*For any* maker bridge swap:
- Creation with inactive maker SHALL fail
- Creation with active maker SHALL lock user's DUST
- mark_swap_complete with duplicate tx_hash SHALL fail
- mark_swap_complete with unique tx_hash SHALL release DUST to maker
- Completed swaps SHALL trigger maker credit recording

**Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5**

### Property 5: TRC20 Transaction Hash Uniqueness

*For any* TRC20 transaction hash, once used in mark_swap_complete, subsequent attempts to use the same hash SHALL fail with TronTxHashAlreadyUsed error.

**Validates: Requirements 4.2**

### Property 6: OTC Order Lifecycle Invariants

*For any* OTC order:
- Creation SHALL occupy buyer quota and lock maker's DUST
- mark_paid SHALL transition status to PaidOrCommitted
- release_dust SHALL transfer DUST to buyer and update credit
- cancel_order SHALL refund DUST to maker and release quota
- dispute_order SHALL transition status to Disputed

**Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5**

### Property 7: OTC Order Amount Limits

*For any* regular OTC order (non-first-purchase), the USD equivalent amount SHALL be within [MinOrderUsdAmount, MaxOrderUsdAmount] range (20-200 USD). Orders outside this range SHALL be rejected.

**Validates: Requirements 5.6**

### Property 8: First Purchase Uniqueness

*For any* user, after completing a first purchase, subsequent calls to create_first_purchase SHALL fail with AlreadyFirstPurchased error.

**Validates: Requirements 6.1, 6.4**

### Property 9: First Purchase DUST Calculation

*For any* first purchase at price P (DUST/USD), the calculated DUST amount SHALL equal FirstPurchaseUsdAmount / P, within the bounds [MinFirstPurchaseDustAmount, MaxFirstPurchaseDustAmount].

**Validates: Requirements 6.2**

### Property 10: Maker First Purchase Quota

*For any* maker with MaxFirstPurchaseOrdersPerMaker pending first purchase orders, additional first purchase order creation SHALL fail with FirstPurchaseQuotaExhausted error.

**Validates: Requirements 6.3**

### Property 11: KYC Enforcement

*For any* OTC order creation when KYC is enabled:
- Users without identity SHALL be rejected
- Users with judgment priority below min_judgment_priority SHALL be rejected
- Users in KycExemptAccounts SHALL bypass KYC check
- KYC failures SHALL emit KycVerificationFailed event

**Validates: Requirements 7.1, 7.2, 7.3, 7.4**

### Property 12: Arbitration Decision Application

*For any* disputed exchange:
- Decision::Release SHALL transfer funds to maker/seller
- Decision::Refund SHALL transfer funds to user/buyer
- Decision::Partial(bps) SHALL split funds proportionally
- Applied decisions SHALL update exchange status and credit records

**Validates: Requirements 8.1, 8.2, 8.3, 8.4**

### Property 13: Buyer Quota Conservation

*For any* buyer, the sum of (occupied_quota + available_quota) SHALL remain constant across order creation, completion, and cancellation operations.

**Validates: Requirements 11.1, 11.2, 11.3**

### Property 14: Migration Data Preservation

*For any* existing Bridge SwapRequest, MakerSwapRecord, or OTC Order, after migration to ExchangeRecord:
- All original field values SHALL be preserved
- Exchange type SHALL correctly reflect the source record type
- Status SHALL map to equivalent ExchangeStatus value

**Validates: Requirements 14.1, 14.4**

## Error Handling

### 错误类型定义

```rust
#[pallet::error]
pub enum Error<T> {
    // === 通用错误 ===
    /// 交易不存在
    ExchangeNotFound,
    /// 交易状态无效
    InvalidExchangeStatus,
    /// 未授权操作
    NotAuthorized,
    /// 编码错误
    EncodingError,
    /// 存储限制已达到
    StorageLimitReached,
    /// 计算溢出
    CalculationOverflow,
    
    // === 做市商相关 ===
    /// 做市商不存在
    MakerNotFound,
    /// 做市商未激活
    MakerNotActive,
    /// 做市商余额不足
    MakerInsufficientBalance,
    
    // === 金额相关 ===
    /// 金额低于最小值
    AmountBelowMinimum,
    /// 金额超过最大值
    AmountExceedsMaximum,
    /// 定价服务不可用
    PricingUnavailable,
    /// 价格无效
    InvalidPrice,
    
    // === 地址相关 ===
    /// 无效的TRON地址
    InvalidTronAddress,
    /// 桥接账户未设置
    BridgeAccountNotSet,
    
    // === 防重放 ===
    /// TRON交易哈希已使用
    TronTxHashAlreadyUsed,
    /// 无效的交易哈希
    InvalidTxHash,
    
    // === 首购相关 ===
    /// 已经首购过
    AlreadyFirstPurchased,
    /// 首购配额已用完
    FirstPurchaseQuotaExhausted,
    
    // === KYC相关 ===
    /// 未设置身份信息
    IdentityNotSet,
    /// 没有有效的身份判断
    NoValidJudgement,
    /// KYC认证等级不足
    InsufficientKycLevel,
    /// 身份认证质量问题
    IdentityQualityIssue,
    /// 账户已在豁免列表
    AccountAlreadyExempted,
    /// 账户不在豁免列表
    AccountNotExempted,
    
    // === 额度相关 ===
    /// 买家额度不足
    InsufficientQuota,
    
    // === 状态转换 ===
    /// 无法取消
    CannotCancel,
    /// 无法举报
    CannotReport,
    /// 无法争议
    CannotDispute,
}
```

### 错误处理策略

| 错误场景 | 处理方式 | 用户提示 |
|---------|---------|---------|
| 做市商不存在/未激活 | 拒绝交易 | 请选择有效的做市商 |
| 金额超限 | 拒绝交易 | 金额需在20-200 USD之间 |
| KYC验证失败 | 拒绝交易 | 请先完成身份认证 |
| 额度不足 | 拒绝交易 | 可用额度不足，请等待订单完成 |
| 交易哈希重复 | 拒绝操作 | 该交易哈希已被使用 |
| 状态不允许操作 | 拒绝操作 | 当前状态不允许此操作 |

## Testing Strategy

### 测试框架选择

- **单元测试**: Rust 内置测试框架
- **属性测试**: `proptest` crate
- **集成测试**: Substrate 测试框架 (`sp-io`, `frame-support`)

### 测试分类

#### 1. 单元测试

```rust
// 测试数据结构
#[test]
fn test_exchange_type_encoding() { ... }

#[test]
fn test_exchange_status_transitions() { ... }

// 测试金额计算
#[test]
fn test_usd_to_dust_calculation() { ... }

#[test]
fn test_first_purchase_dust_amount() { ... }
```

#### 2. 属性测试

```rust
use proptest::prelude::*;

// Property 2: Exchange ID Uniqueness
proptest! {
    #[test]
    fn prop_exchange_id_uniqueness(
        count in 1..100usize
    ) {
        // 创建 count 个交易，验证所有 ID 唯一且递增
    }
}

// Property 5: TRC20 Hash Uniqueness
proptest! {
    #[test]
    fn prop_trc20_hash_uniqueness(
        hash in prop::array::uniform32(any::<u8>())
    ) {
        // 使用 hash 完成交易，再次使用应失败
    }
}

// Property 7: OTC Amount Limits
proptest! {
    #[test]
    fn prop_otc_amount_limits(
        usd_amount in 1u64..500_000_000u64
    ) {
        // 验证金额限制逻辑
    }
}

// Property 13: Quota Conservation
proptest! {
    #[test]
    fn prop_quota_conservation(
        initial_quota in 100_000_000u64..1_000_000_000u64,
        order_amount in 20_000_000u64..200_000_000u64
    ) {
        // 验证额度守恒
    }
}
```

#### 3. 集成测试

```rust
// 完整生命周期测试
#[test]
fn test_official_bridge_full_lifecycle() { ... }

#[test]
fn test_maker_bridge_full_lifecycle() { ... }

#[test]
fn test_otc_order_full_lifecycle() { ... }

#[test]
fn test_first_purchase_full_lifecycle() { ... }

// 迁移测试
#[test]
fn test_migration_from_bridge_pallet() { ... }

#[test]
fn test_migration_from_otc_pallet() { ... }
```

### 测试覆盖目标

| 模块 | 行覆盖率目标 | 分支覆盖率目标 |
|-----|------------|--------------|
| types.rs | 90% | 85% |
| official.rs | 85% | 80% |
| maker_swap.rs | 85% | 80% |
| otc.rs | 85% | 80% |
| first_purchase.rs | 85% | 80% |
| kyc.rs | 90% | 85% |
| arbitration.rs | 85% | 80% |
| migration.rs | 95% | 90% |

### 属性测试配置

```rust
// proptest.toml 或测试文件中配置
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,  // 每个属性测试运行100次
        max_shrink_iters: 1000,
        ..ProptestConfig::default()
    })]
}
```
