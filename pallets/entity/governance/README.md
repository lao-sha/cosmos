# pallet-entity-governance

> 实体代币治理模块 — 多模式去中心化决策系统

- **Runtime Pallet Index**: 125
- **版本**: v0.2.0

## 概述

`pallet-entity-governance` 实现基于实体代币的多模式治理系统。支持 6 种治理模式、42 种提案类型、4 级分层阈值、委员会投票、管理员否决权，以及闪电贷防护快照机制。

### 核心能力

- **6 种治理模式** — None / Advisory / DualTrack / Committee / FullDAO / Tiered
- **42 种提案类型** — 商品、店铺、代币、财务、返佣、提现、会员等级、治理参数、社区
- **分层治理阈值** — Operational / Significant / Critical / Constitutional 四级
- **委员会治理** — 委员会成员管理、最小批准数
- **管理员否决权** — DualTrack / Advisory 模式下可否决通过的提案
- **闪电贷防护** — 快照区块 + 首次持有时间校验

## 架构

```
┌────────────────────────────────────────────────────────────────────┐
│                    pallet-entity-governance                        │
│                       (Runtime Index: 125)                         │
├────────────────────────────────────────────────────────────────────┤
│  提案创建 → 快照锁定 → 代币投票 → 结果判定 → 执行/否决             │
│  治理配置 → 分层阈值 → 委员会管理                                   │
└───────┬──────────┬──────────────┬──────────────┬──────────────────┘
        │          │              │              │
   EntityProvider  ShopProvider   TokenProvider  CommissionProvider
        │          │              │         + MemberProvider
        ▼          ▼              ▼              ▼
   entity-registry entity-shop  entity-token  commission-core
                                              + entity-member
```

### 依赖 Trait

| Trait | 来源 | 用途 |
|-------|------|------|
| `EntityProvider` | pallet-entity-common | 实体所有权查询 |
| `ShopProvider` | pallet-entity-common | 店铺存在性、所有权、暂停/恢复操作 |
| `EntityTokenProvider` | pallet-entity-common | 代币余额、总供应量、启用状态、TokenType 查询 |
| `CommissionProvider` | pallet-entity-commission | 返佣模式/费率/提现配置的链上写入 |
| `MemberProvider` | pallet-entity-commission | 自定义等级/升级模式/升级规则的链上写入 |

## 治理模式

6 种模式定义在 `pallet-entity-common::GovernanceMode`，每个实体可独立配置：

| 模式 | 说明 | 管理员否决 | 推荐实体类型 |
|------|------|-----------|-------------|
| **None** | 无治理，管理员全权控制 | - | Merchant, ServiceProvider |
| **Advisory** | 咨询式，投票仅作建议，可否决 | 可 | Community |
| **DualTrack** | 双轨制，代币投票 + 管理员否决权 | 可 | Enterprise, Project |
| **Committee** | 委员会投票决策 | 否 | Fund |
| **FullDAO** | 完全 DAO，纯代币投票 | 否 | DAO |
| **Tiered** | 分层治理，不同级别不同阈值 | 否 | - |

### 分层阈值（ProposalLevel）

Tiered 模式下，不同级别的提案使用不同通过阈值：

| 级别 | 说明 | 默认阈值 |
|------|------|---------|
| `Operational` | 日常运营 | 50% |
| `Significant` | 重要决策 | 60% |
| `Critical` | 重大变更 | 67% |
| `Constitutional` | 宪法级 | 75% |

## 数据结构

### GovernanceConfig — 实体治理配置

```rust
pub struct GovernanceConfig<AccountId, Balance, BlockNumber, MaxCommitteeSize> {
    pub mode: GovernanceMode,                              // 治理模式
    pub voting_period: BlockNumber,                        // 投票期（0 = 使用全局默认）
    pub execution_delay: BlockNumber,                      // 执行延迟（0 = 使用全局默认）
    pub operational_threshold: u8,                         // 日常运营阈值（百分比）
    pub significant_threshold: u8,                         // 重要决策阈值
    pub critical_threshold: u8,                            // 重大变更阈值
    pub constitutional_threshold: u8,                      // 宪法级阈值
    pub quorum_threshold: u8,                              // 法定人数阈值
    pub proposal_threshold: u16,                           // 提案创建门槛（基点）
    pub admin_veto_enabled: bool,                          // 管理员否决权
    pub required_token_type: Option<TokenType>,            // 需要的代币类型
    pub min_committee_approval: u8,                        // 最小批准数
}
```

> 注：委员会成员独立存储在 `CommitteeMembers` StorageMap 中，不再嵌入 GovernanceConfig。

### Proposal — 提案

```rust
pub struct Proposal<T: Config> {
    pub id: ProposalId,                              // 提案 ID (u64)
    pub shop_id: u64,                                // 店铺 ID
    pub proposer: T::AccountId,                      // 提案者
    pub proposal_type: ProposalType<BalanceOf<T>>,   // 提案类型
    pub title: BoundedVec<u8, T::MaxTitleLength>,    // 标题
    pub description_cid: Option<BoundedVec<u8, T::MaxCidLength>>, // 描述 CID
    pub status: ProposalStatus,                      // 状态
    pub created_at: BlockNumberFor<T>,               // 创建时间
    pub snapshot_block: BlockNumberFor<T>,            // 快照区块（闪电贷防护）
    pub voting_start: BlockNumberFor<T>,             // 投票开始
    pub voting_end: BlockNumberFor<T>,               // 投票结束
    pub execution_time: Option<BlockNumberFor<T>>,   // 执行时间
    pub yes_votes: BalanceOf<T>,                     // 赞成票
    pub no_votes: BalanceOf<T>,                      // 反对票
    pub abstain_votes: BalanceOf<T>,                 // 弃权票
}
```

### VoteRecord — 投票记录

```rust
pub struct VoteRecord<AccountId, Balance, BlockNumber> {
    pub voter: AccountId,       // 投票者
    pub vote: VoteType,         // 投票类型 (Yes/No/Abstain)
    pub weight: Balance,        // 投票权重
    pub voted_at: BlockNumber,  // 投票时间
}
```

### ProposalStatus — 提案状态

```
Created → Voting → Passed → Executed
                 → Failed
                 → Cancelled (提案者/店主取消, 或被否决)
         Expired
```

| 状态 | 说明 |
|------|------|
| `Created` | 已创建，等待投票 |
| `Voting` | 投票中 |
| `Passed` | 投票通过，等待执行 |
| `Failed` | 投票未通过 |
| `Queued` | 排队等待执行 |
| `Executed` | 已执行 |
| `Cancelled` | 已取消 / 被否决 |
| `Expired` | 已过期 |

## 提案类型（共 42 种）

### 商品管理类 (4)

| 类型 | 说明 | 执行方式 |
|------|------|---------|
| `PriceChange` | 商品价格调整 | 事件记录 |
| `ProductListing` | 新商品上架 | 链下 CID 解析 |
| `ProductDelisting` | 商品下架 | 事件记录 |
| `InventoryAdjustment` | 库存调整 | 事件记录 |

### 店铺运营类 (5)

| 类型 | 说明 | 执行方式 |
|------|------|---------|
| `Promotion` | 促销活动 | 事件记录 |
| `ShopNameChange` | 修改店铺名称 | 链下确认 |
| `ShopDescriptionChange` | 修改店铺描述 | 链下确认 |
| `ShopPause` | 暂停店铺营业 | **链上执行** `ShopProvider::pause_shop` |
| `ShopResume` | 恢复店铺营业 | **链上执行** `ShopProvider::resume_shop` |

### 代币经济类 (5)

| 类型 | 说明 | 执行方式 |
|------|------|---------|
| `TokenConfigChange` | 代币配置修改 | 事件记录 |
| `TokenMint` | 增发代币 | 链下执行 |
| `TokenBurn` | 销毁代币 | 事件记录 |
| `AirdropDistribution` | 空投分发 | 链下执行 |
| `Dividend` | 分红提案 | 事件记录 |

### 财务管理类 (4)

| 类型 | 说明 | 执行方式 |
|------|------|---------|
| `TreasurySpend` | 金库支出 | 链下执行 |
| `FeeAdjustment` | 手续费调整 | 事件记录 |
| `RevenueShare` | 收益分配比例 | 事件记录 |
| `RefundPolicy` | 退款政策调整 | 事件记录 |

### 治理参数类 (3)

| 类型 | 说明 | 执行方式 |
|------|------|---------|
| `VotingPeriodChange` | 投票期调整 | **链上执行** 更新 GovernanceConfig |
| `QuorumChange` | 法定人数调整 | **链上执行** 更新 GovernanceConfig |
| `ProposalThresholdChange` | 提案门槛调整 | **链上执行** 更新 GovernanceConfig |

### 返佣配置类 (9)

| 类型 | 说明 | 执行方式 |
|------|------|---------|
| `CommissionModesChange` | 启用/禁用返佣模式 | **链上执行** `CommissionProvider` |
| `DirectRewardChange` | 直推奖励费率 | **链上执行** `CommissionProvider` |
| `MultiLevelChange` | 多级分销配置 | 链下 CID 解析 |
| `LevelDiffChange` | 等级差价配置（5 级费率） | **链上执行** `CommissionProvider` |
| `CustomLevelDiffChange` | 自定义等级极差 | 链下 CID 解析 |
| `FixedAmountChange` | 固定金额配置 | **链上执行** `CommissionProvider` |
| `FirstOrderChange` | 首单奖励配置 | **链上执行** `CommissionProvider` |
| `RepeatPurchaseChange` | 复购奖励配置 | **链上执行** `CommissionProvider` |
| `SingleLineChange` | 单线收益配置 | 待扩展 CommissionProvider |

### 提现配置类 (2)

| 类型 | 说明 | 执行方式 |
|------|------|---------|
| `WithdrawalConfigChange` | 分级提现配置 | **链上执行** `CommissionProvider` |
| `MinRepurchaseRateChange` | 全局最低复购比例底线 | **链上执行** `CommissionProvider` |

### 会员等级体系类 (7)

| 类型 | 说明 | 执行方式 |
|------|------|---------|
| `AddCustomLevel` | 添加自定义等级 | **链上执行** `MemberProvider` |
| `UpdateCustomLevel` | 更新自定义等级 | **链上执行** `MemberProvider` |
| `RemoveCustomLevel` | 删除自定义等级 | **链上执行** `MemberProvider` |
| `SetUpgradeMode` | 设置升级模式 (Auto/Manual/PeriodReset) | **链上执行** `MemberProvider` |
| `EnableCustomLevels` | 启用/禁用自定义等级 | **链上执行** `MemberProvider` |
| `AddUpgradeRule` | 添加升级规则 | 链下 CID 解析 |
| `RemoveUpgradeRule` | 删除升级规则 | 待扩展 MemberProvider |

### 社区类 (3)

| 类型 | 说明 | 执行方式 |
|------|------|---------|
| `CommunityEvent` | 社区活动 | 仅记录 |
| `RuleSuggestion` | 规则建议 | 仅记录 |
| `General` | 通用提案 | 仅记录 |

## 存储项

| 存储 | 类型 | Key | 说明 |
|------|------|-----|------|
| `NextProposalId` | ValueQuery | - | 下一个提案 ID |
| `Proposals` | StorageMap | proposal_id | 提案详情 |
| `ShopProposals` | StorageMap | shop_id | 店铺活跃提案列表 (BoundedVec) |
| `VoteRecords` | StorageDoubleMap | (proposal_id, account) | 投票记录 |
| `FirstHoldTime` | StorageDoubleMap | (shop_id, account) | 用户首次持有代币时间 |
| `VotingPowerSnapshot` | StorageDoubleMap | (proposal_id, account) | 投票权快照余额 |
| `GovernanceConfigs` | StorageMap | entity_id | 实体治理配置 |
| `CommitteeMembers` | StorageMap | entity_id | 委员会成员列表 (BoundedVec) |

## Extrinsics

### call_index(0) — create_proposal

创建治理提案。

```rust
fn create_proposal(
    origin: OriginFor<T>,
    shop_id: u64,
    proposal_type: ProposalType<BalanceOf<T>>,
    title: Vec<u8>,
    description_cid: Option<Vec<u8>>,
) -> DispatchResult
```

- **权限**: 持有店铺代币 >= 总供应量 × `MinProposalThreshold` (默认 1%)
- **校验**: 店铺存在、治理模式 ≠ None、代币启用、参数有效性、活跃提案数 < MaxActiveProposals
- **快照**: `snapshot_block` 设为当前区块
- **H2 参数验证**: 费率/比例类字段 ≤ 10000 (basis points)，百分比 ≤ 100，RevenueShare 之和 ≤ 10000

### call_index(1) — vote

对提案投票。

```rust
fn vote(
    origin: OriginFor<T>,
    proposal_id: ProposalId,
    vote: VoteType,
) -> DispatchResult
```

- **权限**: 持有店铺代币且 `FirstHoldTime <= snapshot_block`
- **投票权重**: `min(当前余额, 快照余额)`
- **校验**: 代币 TokenType 具有投票权 (`has_voting_power()`)、未重复投票、投票期内
- **快照**: 首次投票时锁定当前余额到 `VotingPowerSnapshot`

### call_index(2) — finalize_voting

结束投票并计算结果。任何人可调用（投票期结束后）。

- **法定人数**: `总投票 >= 总供应量 × QuorumThreshold%`
- **通过阈值**: `赞成票 > 总投票 × PassThreshold%`
- 通过后设置 `execution_time = now + ExecutionDelay`

### call_index(3) — execute_proposal

执行通过的提案。任何人可调用（执行时间到达后）。

- 根据 `ProposalType` 调用对应的 Provider 方法
- 链上可直接执行的提案类型立即生效
- 需要链下解析的提案发出 `ProposalExecutionNote` 事件

### call_index(4) — cancel_proposal

取消提案。

- **权限**: 提案者或店主
- **限制**: 仅 Created / Voting 状态

### call_index(5) — configure_governance

配置实体治理模式。

```rust
fn configure_governance(
    origin: OriginFor<T>,
    entity_id: u64,
    mode: GovernanceMode,
    voting_period: Option<BlockNumberFor<T>>,
    quorum_threshold: Option<u8>,
    proposal_threshold: Option<u16>,
    admin_veto_enabled: Option<bool>,
) -> DispatchResult
```

- **权限**: 实体（店铺）所有者

### call_index(6) — set_tiered_thresholds

设置分层治理的四级阈值。

```rust
fn set_tiered_thresholds(
    origin: OriginFor<T>,
    entity_id: u64,
    operational: u8,
    significant: u8,
    critical: u8,
    constitutional: u8,
) -> DispatchResult
```

- **权限**: 实体所有者（通过 EntityProvider 验证）

### call_index(7) — add_committee_member

添加委员会成员。

- **权限**: 实体所有者
- **限制**: 不能重复、不能超过 MaxCommitteeSize

### call_index(8) — remove_committee_member

移除委员会成员。

- **权限**: 实体所有者

### call_index(9) — veto_proposal

管理员否决提案。

```rust
fn veto_proposal(
    origin: OriginFor<T>,
    proposal_id: ProposalId,
) -> DispatchResult
```

- **权限**: 店铺所有者
- **限制**: `admin_veto_enabled == true` 且模式为 DualTrack 或 Advisory
- **适用状态**: Voting 或 Passed

## Events

| 事件 | 字段 | 说明 |
|------|------|------|
| `ProposalCreated` | proposal_id, shop_id, proposer, title | 提案已创建 |
| `Voted` | proposal_id, voter, vote, weight | 已投票 |
| `ProposalPassed` | proposal_id | 提案通过 |
| `ProposalFailed` | proposal_id | 提案未通过 |
| `ProposalExecuted` | proposal_id | 提案已执行 |
| `ProposalCancelled` | proposal_id | 提案已取消 |
| `GovernanceConfigUpdated` | entity_id, mode | 治理配置已更新 |
| `CommitteeMemberAdded` | entity_id, member | 委员会成员已添加 |
| `CommitteeMemberRemoved` | entity_id, member | 委员会成员已移除 |
| `ProposalVetoed` | proposal_id, by | 提案被否决 |
| `ProposalExecutionNote` | proposal_id, note | 执行备注（链下执行） |

## Errors

| 错误 | 说明 |
|------|------|
| `ShopNotFound` | 店铺不存在 |
| `NotShopOwner` | 不是店主 |
| `TokenNotEnabled` | 代币未启用 |
| `ProposalNotFound` | 提案不存在 |
| `InsufficientTokensForProposal` | 代币不足以创建提案 |
| `TooManyActiveProposals` | 已达最大活跃提案数 |
| `InvalidProposalStatus` | 状态不允许此操作 |
| `AlreadyVoted` | 已投过票 |
| `NoVotingPower` | 没有投票权 |
| `VotingEnded` | 投票期已结束 |
| `VotingNotEnded` | 投票期未结束 |
| `ExecutionTimeNotReached` | 执行时间未到 |
| `TitleTooLong` | 标题过长 |
| `CidTooLong` | CID 过长 |
| `CannotCancel` | 无权取消 |
| `GovernanceModeNotAllowed` | 治理模式不允许此操作 |
| `NotAdmin` | 不是管理员 |
| `NotCommitteeMember` | 不是委员会成员 |
| `CommitteeMemberExists` | 委员会成员已存在 |
| `CommitteeMemberNotFound` | 委员会成员不存在 |
| `CommitteeFull` | 委员会已满 |
| `ProposalAlreadyVetoed` | 提案已被否决 |
| `NoVetoRight` | 无否决权 |
| `TokenTypeNoVotingPower` | 代币类型不具有投票权 |
| `InvalidParameter` | 参数无效（费率超范围等） |

## Runtime 配置

```rust
parameter_types! {
    pub const GovernanceVotingPeriod: BlockNumber = 100800;      // 7 天
    pub const GovernanceExecutionDelay: BlockNumber = 28800;     // 2 天
    pub const GovernancePassThreshold: u8 = 50;                 // 50%
    pub const GovernanceQuorumThreshold: u8 = 10;               // 10%
    pub const GovernanceMinProposalThreshold: u16 = 100;        // 1% (基点)
}

impl pallet_entity_governance::Config for Runtime {
    type Balance = Balance;
    type EntityProvider = EntityRegistry;
    type ShopProvider = EntityShop;
    type TokenProvider = EntityTokenProvider;
    type CommissionProvider = EntityCommissionProvider;
    type MemberProvider = EntityMemberProvider;
    type VotingPeriod = GovernanceVotingPeriod;
    type ExecutionDelay = GovernanceExecutionDelay;
    type PassThreshold = GovernancePassThreshold;
    type QuorumThreshold = GovernanceQuorumThreshold;
    type MinProposalThreshold = GovernanceMinProposalThreshold;
    type MaxTitleLength = ConstU32<128>;
    type MaxCidLength = ConstU32<64>;
    type MaxActiveProposals = ConstU32<10>;
    type MaxCommitteeSize = ConstU32<20>;
}
```

### 配置参数说明

| 参数 | 类型 | 说明 | 值 |
|------|------|------|-----|
| `VotingPeriod` | BlockNumber | 投票期长度 | 100800 (~7天) |
| `ExecutionDelay` | BlockNumber | 执行延迟 | 28800 (~2天) |
| `PassThreshold` | u8 | 通过阈值 (%) | 50 |
| `QuorumThreshold` | u8 | 法定人数阈值 (%) | 10 |
| `MinProposalThreshold` | u16 | 提案创建门槛 (基点) | 100 (1%) |
| `MaxTitleLength` | u32 | 标题最大长度 | 128 |
| `MaxCidLength` | u32 | CID 最大长度 | 64 |
| `MaxActiveProposals` | u32 | 每店铺最大活跃提案数 | 10 |
| `MaxCommitteeSize` | u32 | 委员会最大成员数 | 20 |

## 安全机制

### 1. 闪电贷防护（快照机制）

```
创建提案 → snapshot_block = now
投票时 → 检查 FirstHoldTime <= snapshot_block
       → 投票权重 = min(当前余额, 快照余额)
```

攻击者无法通过借入代币→投票→归还来操纵投票结果。

### 2. TokenType 投票权检查

投票前校验代币类型的 `has_voting_power()` 方法，确保仅具备投票权的代币类型可参与治理。

### 3. 创建提案门槛

需持有 >= 1% 总供应量的代币才能创建提案，防止垃圾提案。

### 4. 法定人数

总投票需 >= 10% 总供应量，确保足够参与度。

### 5. 执行延迟

通过后需等待 2 天才能执行，给社区反应时间。DualTrack 模式下管理员可在此窗口内否决。

### 6. 活跃提案限制

每店铺最多 10 个活跃提案，防止 DoS 攻击。

### 7. 治理模式检查

`GovernanceMode::None` 下禁止创建提案。无配置时向后兼容（允许使用全局默认参数）。

### 8. 提案参数验证

创建提案时自动校验参数有效性：费率/比例类字段 ≤ 10000 (basis points)，百分比字段 ≤ 100，`RevenueShare` 两项之和 ≤ 10000，`SetUpgradeMode` ≤ 2。

## 治理流程

### 标准流程（FullDAO / Tiered）

```
代币持有者 create_proposal (持有 >= 1%)
    ↓
投票期 (7天) — 代币持有者 vote (权重 = 代币余额)
    ↓
finalize_voting — 法定人数 >= 10% 且 赞成 > 50%
    ↓
执行延迟 (2天)
    ↓
execute_proposal — 链上自动执行 / 链下事件通知
```

### DualTrack 流程

```
代币持有者 create_proposal
    ↓
投票期 — 代币投票
    ↓                              ┐
finalize_voting → Passed           │ 管理员可在任意阶段
    ↓                              │ 调用 veto_proposal 否决
执行延迟 (否决窗口)                 ┘
    ↓
execute_proposal
```

### Advisory 流程

```
代币持有者 create_proposal
    ↓
投票期 — 代币投票
    ↓
finalize_voting → Passed (仅作建议)
    ↓
管理员可选择执行或否决
```

## 依赖

```toml
[dependencies]
pallet-entity-common = { workspace = true }
pallet-entity-registry = { workspace = true }
pallet-entity-token = { workspace = true }
pallet-entity-commission = { workspace = true }

[dev-dependencies]
pallet-balances = { workspace = true }
```

## 测试

```bash
cargo test -p pallet-entity-governance
```

## 待实现功能

| 功能 | 说明 |
|------|------|
| 时间加权投票 | `calculate_voting_power` 当前直接返回余额，未实现持有时间加权 |
| 委托投票 | 将投票权委托给他人 |
| Committee 模式完整实现 | 当前委员会成员管理已完成，投票流程中未区分委员会模式 |
| Tiered 模式完整实现 | ProposalLevel 已定义，但 finalize_voting 未按级别使用不同阈值 |
| 链上直接执行扩展 | 部分提案类型仅发出事件，待集成更多 Provider |
| ~~单元测试~~ | ✅ 已完成 39 个测试 |

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1.0 | 2026-01-31 | 初始版本：5 个 extrinsics，22 种提案类型 |
| v0.2.0 | 2026-02-03 | Phase 5 增强：6 种治理模式、分层阈值、委员会管理、管理员否决、快照防护、42 种提案类型 |
| v0.2.0-audit | 2026-02-09 | 深度审计：C1 移除弃用 RuntimeEvent、H1 治理模式检查、H2 提案参数验证、H3 冗余 remove_from_active、H4 删除死代码 GovernanceTokenProvider、M4 GovernanceConfig 移除 committee 字段、39 个单元测试 |

## 相关模块

- [pallet-entity-common](../common/README.md) — GovernanceMode、EntityProvider、ShopProvider、EntityTokenProvider
- [pallet-entity-token](../token/README.md) — 代币余额查询
- [pallet-entity-commission](../commission/README.md) — CommissionProvider 返佣配置
- [pallet-entity-member](../member/README.md) — MemberProvider 会员等级
