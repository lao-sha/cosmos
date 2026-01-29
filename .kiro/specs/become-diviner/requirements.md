# Requirements Document

## Introduction

"成为占卜师"功能允许用户注册成为占卜服务提供者，在星尘玄鉴平台上提供专业的玄学占卜服务。该功能基于 `pallet-divination-market` 模块，实现占卜师注册、资质审核、服务套餐管理、订单处理、收益提现等完整业务流程。

## Glossary

- **Diviner (占卜师)**: 在平台注册并通过审核的占卜服务提供者
- **Provider (服务提供者)**: 链上存储的占卜师实体，包含基本信息、状态、等级等
- **Package (服务套餐)**: 占卜师创建的服务产品，包含价格、时长、追问次数等
- **Order (订单)**: 用户购买占卜服务产生的交易记录
- **Tier (等级)**: 占卜师的信誉等级，从 Novice 到 Master 共 5 级
- **Specialty (擅长领域)**: 占卜师擅长的问事类型，如事业、感情、财运等
- **DivinationType (占卜类型)**: 支持的占卜术数类型，如梅花易数、八字、六爻等
- **ServiceType (服务类型)**: 服务形式，包括文字解卦、语音解卦、视频解卦、实时咨询
- **Platform_Fee (平台费率)**: 根据占卜师等级收取的服务费，8%-20%
- **Review (评价)**: 用户对占卜服务的多维度评分

## Requirements

### Requirement 1: 占卜师注册

**User Story:** As a user, I want to register as a diviner, so that I can provide divination services on the platform.

#### Acceptance Criteria

1. WHEN a user opens the "become diviner" page, THE System SHALL display the registration form with required fields
2. WHEN a user submits registration with valid information, THE System SHALL create a provider record with Pending status
3. WHEN a user submits registration with invalid information, THE System SHALL display specific validation errors
4. THE System SHALL require the following fields: display name (1-64 chars), bio (1-256 chars), specialties (at least 1), supported divination types (at least 1)
5. WHEN registration is submitted, THE System SHALL lock the minimum deposit (100 COS) from user's balance
6. IF the user has insufficient balance for deposit, THEN THE System SHALL display an error and prevent registration

### Requirement 2: 占卜师状态管理

**User Story:** As a diviner, I want to manage my service status, so that I can control when I accept orders.

#### Acceptance Criteria

1. WHEN a diviner's registration is approved, THE System SHALL change status from Pending to Active
2. WHEN an active diviner pauses service, THE System SHALL change status to Paused and stop showing in available list
3. WHEN a paused diviner resumes service, THE System SHALL change status to Active and show in available list
4. WHEN a diviner deactivates account, THE System SHALL change status to Deactivated and refund deposit
5. WHILE a diviner is in Banned status, THE System SHALL prevent any service operations
6. THE System SHALL display current status prominently on the diviner dashboard

### Requirement 3: 服务套餐管理

**User Story:** As a diviner, I want to create and manage service packages, so that users can purchase my services.

#### Acceptance Criteria

1. WHEN a diviner creates a package, THE System SHALL validate price >= minimum (1 COS)
2. WHEN a diviner creates a package, THE System SHALL require: divination type, service type, name, description, price, follow-up count
3. THE System SHALL limit each diviner to maximum 10 packages
4. WHEN a diviner updates a package, THE System SHALL preserve the package ID and update only changed fields
5. WHEN a diviner removes a package, THE System SHALL mark it as inactive (not delete)
6. THE System SHALL support urgent service option with configurable surcharge (0-100%)

### Requirement 4: 订单处理

**User Story:** As a diviner, I want to receive and process orders, so that I can provide services and earn income.

#### Acceptance Criteria

1. WHEN a new order is created for a diviner, THE System SHALL notify the diviner
2. WHEN a diviner accepts an order, THE System SHALL change order status to Accepted
3. WHEN a diviner rejects an order, THE System SHALL refund the customer and change status to Cancelled
4. IF a diviner does not respond within 2 hours, THEN THE System SHALL auto-cancel and refund
5. WHEN a diviner submits interpretation, THE System SHALL change status to Completed and settle payment
6. THE System SHALL calculate diviner earnings as: total_amount - (total_amount × platform_fee_rate)

### Requirement 5: 追问与回复

**User Story:** As a diviner, I want to answer follow-up questions, so that I can provide complete service to customers.

#### Acceptance Criteria

1. WHEN a customer submits a follow-up, THE System SHALL notify the diviner
2. WHEN a diviner answers a follow-up, THE System SHALL record the response with IPFS CID
3. THE System SHALL enforce the follow-up limit defined in the package
4. WHEN all follow-ups are used, THE System SHALL prevent additional follow-up submissions

### Requirement 6: 评价系统

**User Story:** As a diviner, I want to receive and respond to reviews, so that I can build reputation and improve service.

#### Acceptance Criteria

1. WHEN a customer submits a review, THE System SHALL update the diviner's average ratings
2. THE System SHALL support 4 rating dimensions: overall, accuracy, attitude, response speed (1-5 stars each)
3. WHEN a diviner replies to a review, THE System SHALL display the reply under the review
4. WHEN ratings meet tier upgrade criteria, THE System SHALL automatically upgrade the diviner's tier
5. THE System SHALL display tier badge and statistics on diviner profile

### Requirement 7: 收益管理

**User Story:** As a diviner, I want to view and withdraw my earnings, so that I can receive payment for my services.

#### Acceptance Criteria

1. THE System SHALL display total earnings, available balance, and pending settlements
2. WHEN a diviner requests withdrawal, THE System SHALL transfer funds to their wallet
3. THE System SHALL record all withdrawal history with timestamps and amounts
4. IF withdrawal amount exceeds available balance, THEN THE System SHALL reject the request
5. THE System SHALL display platform fee breakdown for each completed order

### Requirement 8: 占卜师等级体系

**User Story:** As a diviner, I want to progress through tiers, so that I can unlock benefits and lower platform fees.

#### Acceptance Criteria

1. THE System SHALL define 5 tiers: Novice (0), Certified (1), Senior (2), Expert (3), Master (4)
2. THE System SHALL apply platform fees by tier: Novice 20%, Certified 15%, Senior 12%, Expert 10%, Master 8%
3. WHEN a diviner meets upgrade criteria (order count + rating), THE System SHALL auto-upgrade tier
4. THE System SHALL display tier progress and requirements on dashboard
5. THE System SHALL show tier badge on diviner's public profile

### Requirement 9: 占卜师仪表盘

**User Story:** As a diviner, I want a dashboard to manage my business, so that I can efficiently handle all operations.

#### Acceptance Criteria

1. THE Dashboard SHALL display: status, tier, earnings summary, pending orders count
2. THE Dashboard SHALL provide quick access to: orders, packages, reviews, withdrawals
3. THE Dashboard SHALL show recent activity feed with order and review notifications
4. THE Dashboard SHALL display performance metrics: completion rate, average rating, response time
5. WHEN there are pending orders, THE Dashboard SHALL highlight them prominently

### Requirement 10: 占卜师公开资料

**User Story:** As a user, I want to view diviner profiles, so that I can choose the right diviner for my needs.

#### Acceptance Criteria

1. THE Profile SHALL display: name, bio, avatar, tier badge, specialties, supported types
2. THE Profile SHALL show statistics: total orders, average rating, response time
3. THE Profile SHALL list active service packages with prices
4. THE Profile SHALL display recent reviews (with optional anonymity respected)
5. WHEN viewing a profile, THE System SHALL provide a "Book Service" action
