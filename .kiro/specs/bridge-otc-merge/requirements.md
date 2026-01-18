# Requirements Document

## Introduction

本文档定义了将 `pallet-bridge` 和 `pallet-otc-order` 两个模块合并为统一的 `pallet-exchange` 模块的需求规格。合并的目标是消除代码重复、统一业务流程、简化维护成本，同时保持现有功能的完整性。

## Glossary

- **Exchange_Module**: 合并后的统一兑换模块，处理所有 DUST ↔ USDT 交易
- **Official_Bridge**: 官方桥接服务，由治理委员会管理的 DUST → USDT 兑换
- **Maker_Bridge**: 做市商桥接服务，用户通过做市商进行 DUST → USDT 兑换
- **OTC_Order**: 场外交易订单，用户通过做市商购买 DUST（USDT → DUST）
- **First_Purchase**: 首购订单，新用户固定 10 USD 价值的首次购买
- **Swap_Record**: 统一的兑换/订单记录结构
- **Escrow_Service**: 托管服务，用于锁定和释放资金
- **KYC_Verification**: 身份认证验证服务
- **Credit_System**: 信用分系统，记录买家和做市商的信用

## Requirements

### Requirement 1: 统一数据结构

**User Story:** As a developer, I want unified data structures for all exchange types, so that I can reduce code duplication and simplify maintenance.

#### Acceptance Criteria

1. THE Exchange_Module SHALL define a unified `ExchangeRecord` structure that supports all exchange types (Official_Bridge, Maker_Bridge, OTC_Order, First_Purchase)
2. THE Exchange_Module SHALL use a single `ExchangeStatus` enum to represent all possible states across exchange types
3. THE Exchange_Module SHALL maintain backward compatibility with existing storage data through migration
4. WHEN a new exchange is created, THE Exchange_Module SHALL assign a unique exchange_id from a single sequence

### Requirement 2: 统一存储层

**User Story:** As a system architect, I want consolidated storage maps, so that I can reduce storage complexity and improve query efficiency.

#### Acceptance Criteria

1. THE Exchange_Module SHALL use a single `Exchanges` storage map for all exchange records
2. THE Exchange_Module SHALL maintain `UserExchanges` index for querying user's exchanges
3. THE Exchange_Module SHALL maintain `MakerExchanges` index for querying maker's exchanges
4. THE Exchange_Module SHALL preserve the `UsedTronTxHashes` storage for replay attack prevention
5. THE Exchange_Module SHALL preserve the `HasFirstPurchased` storage for first purchase tracking

### Requirement 3: 官方桥接功能保留

**User Story:** As a user, I want to use official bridge service, so that I can exchange DUST to USDT through governance-managed channel.

#### Acceptance Criteria

1. WHEN a user calls `official_swap`, THE Exchange_Module SHALL lock user's DUST to escrow
2. WHEN governance calls `complete_official_swap`, THE Exchange_Module SHALL release DUST to bridge account
3. IF an official swap times out, THEN THE Exchange_Module SHALL allow refund to user
4. THE Exchange_Module SHALL emit `OfficialSwapCreated` and `OfficialSwapCompleted` events

### Requirement 4: 做市商桥接功能保留

**User Story:** As a user, I want to use maker bridge service, so that I can exchange DUST to USDT through market makers.

#### Acceptance Criteria

1. WHEN a user calls `maker_swap`, THE Exchange_Module SHALL verify maker is active and lock user's DUST
2. WHEN a maker calls `mark_swap_complete` with TRC20 tx hash, THE Exchange_Module SHALL verify hash uniqueness and release DUST to maker
3. IF a maker swap is not completed within timeout, THEN THE Exchange_Module SHALL allow user to report
4. WHEN a user reports a swap, THE Exchange_Module SHALL change status to `UserReported` for arbitration
5. THE Exchange_Module SHALL record maker credit for completed swaps

### Requirement 5: OTC 订单功能保留

**User Story:** As a user, I want to buy DUST through OTC orders, so that I can acquire DUST by paying USDT to makers.

#### Acceptance Criteria

1. WHEN a user calls `create_order`, THE Exchange_Module SHALL verify KYC, check quota, and lock maker's DUST
2. WHEN a buyer calls `mark_paid`, THE Exchange_Module SHALL update order status to `PaidOrCommitted`
3. WHEN a maker calls `release_dust`, THE Exchange_Module SHALL release DUST to buyer and update credit
4. WHEN either party calls `cancel_order`, THE Exchange_Module SHALL refund DUST to maker and release quota
5. WHEN either party calls `dispute_order`, THE Exchange_Module SHALL change status to `Disputed`
6. THE Exchange_Module SHALL enforce order amount limits (20-200 USD for regular orders)

### Requirement 6: 首购订单功能保留

**User Story:** As a new user, I want to make a first purchase with fixed USD value, so that I can easily onboard to the platform.

#### Acceptance Criteria

1. WHEN a new user calls `create_first_purchase`, THE Exchange_Module SHALL verify user has not first purchased before
2. THE Exchange_Module SHALL calculate DUST amount based on current price for fixed 10 USD value
3. THE Exchange_Module SHALL limit each maker to maximum 5 concurrent first purchase orders
4. WHEN a first purchase completes, THE Exchange_Module SHALL mark user as `HasFirstPurchased`

### Requirement 7: KYC 认证集成

**User Story:** As a compliance officer, I want KYC verification for OTC orders, so that I can ensure regulatory compliance.

#### Acceptance Criteria

1. WHEN KYC is enabled, THE Exchange_Module SHALL verify buyer's identity before creating OTC orders
2. THE Exchange_Module SHALL support configurable minimum judgment level (Unknown/FeePaid/Reasonable/KnownGood)
3. THE Exchange_Module SHALL support KYC exempt accounts list
4. WHEN KYC verification fails, THE Exchange_Module SHALL emit `KycVerificationFailed` event with reason

### Requirement 8: 仲裁系统集成

**User Story:** As a dispute resolver, I want unified arbitration interface, so that I can handle disputes across all exchange types.

#### Acceptance Criteria

1. THE Exchange_Module SHALL provide `can_dispute_exchange` function for arbitration pallet
2. THE Exchange_Module SHALL provide `apply_arbitration_decision` function supporting Release/Refund/Partial decisions
3. WHEN arbitration decision is applied, THE Exchange_Module SHALL update exchange status and credit records
4. THE Exchange_Module SHALL support arbitration for both Maker_Bridge swaps and OTC_Order disputes

### Requirement 9: 聊天权限集成

**User Story:** As a user, I want automatic chat permission with my trading counterpart, so that I can communicate about the transaction.

#### Acceptance Criteria

1. WHEN an OTC order is created, THE Exchange_Module SHALL grant bidirectional chat permission between buyer and maker
2. THE Exchange_Module SHALL set chat permission duration to 30 days
3. THE Exchange_Module SHALL include order metadata in chat permission

### Requirement 10: 信用分系统集成

**User Story:** As a platform operator, I want credit tracking for all participants, so that I can maintain service quality.

#### Acceptance Criteria

1. WHEN a maker completes an exchange, THE Exchange_Module SHALL record completion to maker credit
2. WHEN a maker times out, THE Exchange_Module SHALL record timeout to maker credit
3. WHEN arbitration completes, THE Exchange_Module SHALL record dispute result to maker credit
4. WHEN a buyer completes an order, THE Exchange_Module SHALL record completion to buyer credit
5. WHEN a buyer cancels an order, THE Exchange_Module SHALL record cancellation to buyer credit

### Requirement 11: 买家额度管理

**User Story:** As a risk manager, I want buyer quota management, so that I can limit exposure per buyer.

#### Acceptance Criteria

1. WHEN an OTC order is created, THE Exchange_Module SHALL occupy buyer's quota
2. WHEN an OTC order completes or is cancelled, THE Exchange_Module SHALL release buyer's quota
3. IF buyer's quota is insufficient, THEN THE Exchange_Module SHALL reject order creation

### Requirement 12: 统一配置参数

**User Story:** As a runtime developer, I want consolidated configuration, so that I can easily configure the exchange module.

#### Acceptance Criteria

1. THE Exchange_Module SHALL consolidate timeout parameters (SwapTimeout, OcwSwapTimeoutBlocks, OrderTimeout)
2. THE Exchange_Module SHALL consolidate amount limits (MinSwapAmount, MinOrderUsdAmount, MaxOrderUsdAmount)
3. THE Exchange_Module SHALL consolidate first purchase parameters (FirstPurchaseUsdAmount, MaxFirstPurchaseOrdersPerMaker)
4. THE Exchange_Module SHALL provide clear documentation for all configuration parameters

### Requirement 13: 事件统一

**User Story:** As a frontend developer, I want unified event structure, so that I can easily subscribe and handle exchange events.

#### Acceptance Criteria

1. THE Exchange_Module SHALL emit `ExchangeCreated` event with exchange type discriminator
2. THE Exchange_Module SHALL emit `ExchangeStatusChanged` event for all status transitions
3. THE Exchange_Module SHALL emit `ExchangeCompleted` event when exchange reaches terminal state
4. THE Exchange_Module SHALL preserve KYC-related events (KycEnabled, KycDisabled, KycVerificationFailed)

### Requirement 14: 向后兼容性

**User Story:** As a system administrator, I want backward compatibility, so that existing integrations continue to work.

#### Acceptance Criteria

1. THE Exchange_Module SHALL provide migration logic for existing Bridge and OTC storage
2. THE Exchange_Module SHALL maintain existing RPC query interfaces or provide equivalent replacements
3. THE Exchange_Module SHALL document breaking changes and migration steps
4. WHEN migrating, THE Exchange_Module SHALL preserve all existing exchange records and their states
