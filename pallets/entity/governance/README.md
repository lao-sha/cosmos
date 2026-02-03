# pallet-entity-governance

> 🏛️ Entity 店铺代币治理模块 - 去中心化店铺决策系统

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Substrate](https://img.shields.io/badge/Substrate-polkadot--sdk-blue)](https://github.com/paritytech/polkadot-sdk)

## 📖 概述

`pallet-entity-governance` 是 Entity 商城系统的店铺治理模块，实现基于店铺代币的去中心化决策机制。代币持有者可以参与店铺运营决策，实现社区驱动的店铺管理。

### 核心功能

- 📝 **提案创建** - 代币持有者发起治理提案
- 🗳️ **代币投票** - 代币加权投票机制
- 🛡️ **店主否决** - 双轨制下店主保留否决权
- ⚡ **提案执行** - 通过的提案自动执行
- 🔒 **安全机制** - 法定人数、通过阈值、执行延迟

## 🏗️ 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                  pallet-entity-governance                    │
│                      (治理模块)                                  │
├─────────────────────────────────────────────────────────────────┤
│  • 提案创建与管理                                                │
│  • 代币加权投票                                                  │
│  • 店主否决权                                                    │
│  • 提案执行                                                      │
└─────────────────────────────────────────────────────────────────┘
         │                              │
         │ ShopProvider                 │ ShopTokenProvider
         ▼                              ▼
┌─────────────────────┐    ┌─────────────────────────────────────┐
│  pallet-entity   │    │       pallet-entity-token        │
│      -shop          │    │           (代币模块)                 │
│    (店铺模块)        │    │  • 代币余额查询                      │
│  • 店铺存在性验证    │    │  • 总供应量查询                      │
│  • 店主身份验证      │    │  • 代币启用状态                      │
└─────────────────────┘    └─────────────────────────────────────┘
```

## 🎯 治理模式

本模块采用 **纯代币投票 (TokenVoteOnly)** 模式，所有提案完全由代币持有者投票决定，店主无否决权。

### 提案类型（共 22 种）

```
┌─────────────────────────────────────────────────────────────────┐
│                   纯代币投票 (TokenVoteOnly)                    │
│                   所有提案由代币持有者决定                         │
├─────────────────────────────────────────────────────────────────┤
│  商品管理 (4)                                                   │
│  • PriceChange / ProductListing / ProductDelisting              │
│  • InventoryAdjustment                                          │
├─────────────────────────────────────────────────────────────────┤
│  店铺运营 (5)                                                   │
│  • Promotion / ShopNameChange / ShopDescriptionChange           │
│  • ShopPause / ShopResume                                       │
├─────────────────────────────────────────────────────────────────┤
│  代币经济 (5)                                                   │
│  • TokenConfigChange / TokenMint / TokenBurn                    │
│  • AirdropDistribution / Dividend                               │
├─────────────────────────────────────────────────────────────────┤
│  财务管理 (4)                                                   │
│  • TreasurySpend / FeeAdjustment / RevenueShare / RefundPolicy  │
├─────────────────────────────────────────────────────────────────┤
│  治理参数 (3)                                                   │
│  • VotingPeriodChange / QuorumChange / ProposalThresholdChange  │
├─────────────────────────────────────────────────────────────────┤
│  社区类 (3)                                                     │
│  • CommunityEvent / RuleSuggestion / General                    │
└─────────────────────────────────────────────────────────────────┘
```

## 📦 安装

### Cargo.toml

```toml
[dependencies]
pallet-entity-governance = { path = "pallets/entity/governance", default-features = false }

[features]
std = [
    "pallet-entity-governance/std",
]
```

## ⚙️ Runtime 配置

```rust
parameter_types! {
    /// 投票期: 7 天（假设 6 秒一个块）
    pub const GovernanceVotingPeriod: BlockNumber = 100800;
    /// 执行延迟: 2 天
    pub const GovernanceExecutionDelay: BlockNumber = 28800;
    /// 通过阈值: 50%
    pub const GovernancePassThreshold: u8 = 50;
    /// 法定人数: 10%
    pub const GovernanceQuorumThreshold: u8 = 10;
    /// 创建提案所需最低代币持有比例: 1% (100 基点)
    pub const GovernanceMinProposalThreshold: u16 = 100;
}

impl pallet_entity_governance::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type ShopProvider = EntityShop;
    type TokenProvider = EntityTokenProvider;
    type VotingPeriod = GovernanceVotingPeriod;
    type ExecutionDelay = GovernanceExecutionDelay;
    type PassThreshold = GovernancePassThreshold;
    type QuorumThreshold = GovernanceQuorumThreshold;
    type MinProposalThreshold = GovernanceMinProposalThreshold;
    type MaxTitleLength = ConstU32<128>;
    type MaxCidLength = ConstU32<64>;
    type MaxActiveProposals = ConstU32<10>;
}
```

### 配置参数说明

| 参数 | 类型 | 说明 | 示例值 |
|------|------|------|--------|
| `VotingPeriod` | BlockNumber | 投票期长度 | 100800 (~7天) |
| `ExecutionDelay` | BlockNumber | 执行延迟（否决窗口） | 28800 (~2天) |
| `PassThreshold` | u8 | 通过阈值（百分比） | 50 (50%) |
| `QuorumThreshold` | u8 | 法定人数阈值（百分比） | 10 (10%) |
| `MinProposalThreshold` | u16 | 创建提案最低持有比例（基点） | 100 (1%) |
| `MaxTitleLength` | u32 | 提案标题最大长度 | 128 |
| `MaxCidLength` | u32 | CID 最大长度 | 64 |
| `MaxActiveProposals` | u32 | 每店铺最大活跃提案数 | 10 |

## 📊 数据结构

### ProposalStatus - 提案状态

```rust
pub enum ProposalStatus {
    Created,    // 已创建，等待投票
    Voting,     // 投票中
    Passed,     // 投票通过
    Failed,     // 投票未通过
    Queued,     // 排队等待执行
    Executed,   // 已执行
    Cancelled,  // 已取消
    Expired,    // 已过期
}
```

### VoteType - 投票类型

```rust
pub enum VoteType {
    Yes,      // 赞成
    No,       // 反对
    Abstain,  // 弃权
}
```

### ProposalType - 提案类型（共 39 种）

#### 商品管理类 (4)

| 类型 | 说明 | 参数 |
|------|------|------|
| `PriceChange` | 商品价格调整 | `product_id`, `new_price` |
| `ProductListing` | 新商品上架 | `product_cid` |
| `ProductDelisting` | 商品下架 | `product_id` |
| `InventoryAdjustment` | 库存调整 | `product_id`, `new_inventory` |

#### 店铺运营类 (5)

| 类型 | 说明 | 参数 |
|------|------|------|
| `Promotion` | 促销活动 | `discount_rate`, `duration_blocks` |
| `ShopNameChange` | 修改店铺名称 | `new_name` |
| `ShopDescriptionChange` | 修改店铺描述 | `description_cid` |
| `ShopPause` | 暂停店铺营业 | - |
| `ShopResume` | 恢复店铺营业 | - |

#### 代币经济类 (5)

| 类型 | 说明 | 参数 |
|------|------|------|
| `TokenConfigChange` | 代币配置修改 | `reward_rate`, `exchange_rate` |
| `TokenMint` | 增发代币 | `amount`, `recipient_cid` |
| `TokenBurn` | 销毁代币 | `amount` |
| `AirdropDistribution` | 空投分发 | `airdrop_cid`, `total_amount` |
| `Dividend` | 分红提案 | `rate` |

#### 财务管理类 (4)

| 类型 | 说明 | 参数 |
|------|------|------|
| `TreasurySpend` | 店铺金库支出 | `amount`, `recipient_cid`, `reason_cid` |
| `FeeAdjustment` | 手续费调整 | `new_fee_rate` |
| `RevenueShare` | 收益分配比例 | `owner_share`, `token_holder_share` |
| `RefundPolicy` | 退款政策调整 | `policy_cid` |

#### 治理参数类 (3)

| 类型 | 说明 | 参数 |
|------|------|------|
| `VotingPeriodChange` | 投票期调整 | `new_period_blocks` |
| `QuorumChange` | 法定人数调整 | `new_quorum` |
| `ProposalThresholdChange` | 提案门槛调整 | `new_threshold` |

#### 返佣配置类 (9) 🆕

| 类型 | 说明 | 参数 |
|------|------|------|
| `CommissionModesChange` | 启用/禁用返佣模式 | `modes` (位标志) |
| `DirectRewardChange` | 直推奖励配置 | `rate` |
| `MultiLevelChange` | 多级分销配置 | `levels_cid`, `max_total_rate` |
| `LevelDiffChange` | 等级差价配置 | `normal_rate`, `silver_rate`, `gold_rate`, `platinum_rate`, `diamond_rate` |
| `CustomLevelDiffChange` | 自定义等级极差配置 | `rates_cid`, `max_depth` |
| `FixedAmountChange` | 固定金额配置 | `amount` |
| `FirstOrderChange` | 首单奖励配置 | `amount`, `rate`, `use_amount` |
| `RepeatPurchaseChange` | 复购奖励配置 | `rate`, `min_orders` |
| `SingleLineChange` | 单线收益配置 | `upline_rate`, `downline_rate`, `base_upline_levels`, `base_downline_levels`, `max_upline_levels`, `max_downline_levels` |

#### 分级提现配置类 (1) 🆕

| 类型 | 说明 | 参数 |
|------|------|------|
| `WithdrawalConfigChange` | 分级提现配置 | `tier_configs_cid`, `enabled`, `shopping_balance_generates_commission` |

#### 会员等级体系类 (7) 🆕

| 类型 | 说明 | 参数 |
|------|------|------|
| `AddCustomLevel` | 添加自定义等级 | `level_id`, `name`, `threshold`, `discount_rate`, `commission_bonus` |
| `UpdateCustomLevel` | 更新自定义等级 | `level_id`, `name?`, `threshold?`, `discount_rate?`, `commission_bonus?` |
| `RemoveCustomLevel` | 删除自定义等级 | `level_id` |
| `SetUpgradeMode` | 设置升级模式 | `mode` (0=Auto, 1=Manual, 2=PeriodReset) |
| `EnableCustomLevels` | 启用/禁用自定义等级 | `enabled` |
| `AddUpgradeRule` | 添加升级规则 | `rule_cid` |
| `RemoveUpgradeRule` | 删除升级规则 | `rule_id` |

#### 社区类 (3)

| 类型 | 说明 | 参数 |
|------|------|------|
| `CommunityEvent` | 社区活动 | `event_cid` |
| `RuleSuggestion` | 规则建议 | `suggestion_cid` |
| `General` | 通用提案 | `title_cid`, `content_cid` |

### Proposal - 提案结构

```rust
pub struct Proposal<T: Config> {
    pub id: ProposalId,                              // 提案 ID
    pub shop_id: u64,                                // 店铺 ID
    pub proposer: T::AccountId,                      // 提案者
    pub proposal_type: ProposalType<BalanceOf<T>>,   // 提案类型
    pub title: BoundedVec<u8, T::MaxTitleLength>,    // 标题
    pub description_cid: Option<BoundedVec<u8, T::MaxCidLength>>, // 描述 CID
    pub status: ProposalStatus,                      // 状态
    pub created_at: BlockNumberFor<T>,               // 创建时间
    pub voting_start: BlockNumberFor<T>,             // 投票开始
    pub voting_end: BlockNumberFor<T>,               // 投票结束
    pub execution_time: Option<BlockNumberFor<T>>,   // 执行时间
    pub yes_votes: BalanceOf<T>,                     // 赞成票
    pub no_votes: BalanceOf<T>,                      // 反对票
    pub abstain_votes: BalanceOf<T>,                 // 弃权票
}
```

## 🔧 Extrinsics

### 1. create_proposal

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

**权限要求：**
- 持有店铺代币 ≥ 总供应量的 `MinProposalThreshold`（默认 1%）

**示例：**
```javascript
// 创建价格调整提案
api.tx.entityGovernance.createProposal(
    1,  // shop_id
    { PriceChange: { product_id: 100, new_price: 50000000000 } },
    "调整商品价格",
    "QmXxx..."  // IPFS CID
)
```

### 2. vote

对提案投票。

```rust
fn vote(
    origin: OriginFor<T>,
    proposal_id: ProposalId,
    vote: VoteType,
) -> DispatchResult
```

**权限要求：**
- 持有店铺代币（投票权重 = 代币余额）

**示例：**
```javascript
api.tx.entityGovernance.vote(1, { Yes: null })
```

### 3. finalize_voting

结束投票并计算结果。

```rust
fn finalize_voting(
    origin: OriginFor<T>,
    proposal_id: ProposalId,
) -> DispatchResult
```

**权限要求：**
- 任何人（投票期结束后）

**判定逻辑：**
1. 检查法定人数：`总投票 ≥ 总供应量 × QuorumThreshold%`
2. 检查通过阈值：`赞成票 > 总投票 × PassThreshold%`

### 4. execute_proposal

执行通过的提案。

```rust
fn execute_proposal(
    origin: OriginFor<T>,
    proposal_id: ProposalId,
) -> DispatchResult
```

**权限要求：**
- 任何人（执行时间到达后）

### 5. cancel_proposal

取消提案。

```rust
fn cancel_proposal(
    origin: OriginFor<T>,
    proposal_id: ProposalId,
) -> DispatchResult
```

**权限要求：**
- 提案者或店主
- 提案状态为 Created 或 Voting

## 📡 Events

| 事件 | 说明 | 字段 |
|------|------|------|
| `ProposalCreated` | 提案已创建 | proposal_id, shop_id, proposer, title |
| `Voted` | 已投票 | proposal_id, voter, vote, weight |
| `ProposalPassed` | 提案已通过 | proposal_id |
| `ProposalFailed` | 提案未通过 | proposal_id |
| `ProposalExecuted` | 提案已执行 | proposal_id |
| `ProposalCancelled` | 提案已取消 | proposal_id |

## ❌ Errors

| 错误 | 说明 |
|------|------|
| `ShopNotFound` | 店铺不存在 |
| `NotShopOwner` | 不是店主 |
| `TokenNotEnabled` | 店铺代币未启用 |
| `ProposalNotFound` | 提案不存在 |
| `InsufficientTokensForProposal` | 代币不足以创建提案 |
| `TooManyActiveProposals` | 已达到最大活跃提案数 |
| `InvalidProposalStatus` | 提案状态不允许此操作 |
| `AlreadyVoted` | 已经投过票 |
| `NoVotingPower` | 没有投票权 |
| `VotingEnded` | 投票期已结束 |
| `VotingNotEnded` | 投票期未结束 |
| `ExecutionTimeNotReached` | 执行时间未到 |
| `CannotCancel` | 无权取消 |

## 🔌 依赖接口

### ShopTokenProvider Trait

治理模块依赖此 trait 查询代币信息：

```rust
pub trait ShopTokenProvider<AccountId, Balance> {
    /// 获取用户在店铺的代币余额
    fn token_balance(shop_id: u64, holder: &AccountId) -> Balance;
    
    /// 获取店铺代币总供应量
    fn total_supply(shop_id: u64) -> Balance;
    
    /// 检查店铺代币是否启用
    fn is_enabled(shop_id: u64) -> bool;
}
```

## 💡 使用流程

### 完整治理流程

```
┌─────────────────────────────────────────────────────────────────┐
│                        治理流程                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. 创建提案                                                     │
│     └── 代币持有者调用 create_proposal                           │
│         └── 需持有 ≥1% 代币                                      │
│                                                                 │
│  2. 投票期 (7 天)                                                │
│     └── 代币持有者调用 vote                                      │
│         └── 投票权重 = 代币余额                                  │
│                                                                 │
│  3. 结束投票                                                     │
│     └── 任何人调用 finalize_voting                               │
│         ├── 法定人数 ≥10%？                                      │
│         │   └── 否 → 提案失败                                    │
│         └── 赞成票 >50%？                                        │
│             ├── 是 → 提案通过                                    │
│             └── 否 → 提案失败                                    │
│                                                                 │
│  4. 执行延迟 (2 天)                                              │
│     └── 等待执行时间到达                                          │
│                                                                 │
│  5. 执行提案                                                     │
│     └── 任何人调用 execute_proposal                              │
│         └── 执行提案内容                                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 示例场景

#### 场景 1：价格调整提案

```
1. 用户 Alice 持有 2% 店铺代币
2. Alice 创建 PriceChange 提案
3. 投票期内：
   - Bob (5% 代币) 投赞成
   - Carol (3% 代币) 投反对
4. 投票结束：赞成 5% > 反对 3%，法定人数 8% < 10%
5. 结果：提案失败（未达法定人数）
```

#### 场景 2：社区活动提案

```
1. 用户 Dave 持有 1.5% 店铺代币
2. Dave 创建 CommunityEvent 提案
3. 投票期内：
   - 15% 代币投赞成
   - 5% 代币投反对
4. 投票结束：法定人数 20% ≥10%，赞成 75% > 50%
5. 结果：提案通过
6. 2 天后执行
```

## 🔐 安全机制

### 1. 创建提案门槛

```rust
// 需持有 ≥1% 代币才能创建提案
let min_threshold = total_supply * MinProposalThreshold / 10000;
ensure!(balance >= min_threshold, Error::<T>::InsufficientTokensForProposal);
```

**目的**：防止垃圾提案

### 2. 法定人数

```rust
// 总投票需 ≥10% 总供应量
let quorum_threshold = total_supply * QuorumThreshold / 100;
if total_votes < quorum_threshold {
    // 提案失败
}
```

**目的**：确保足够参与度

### 3. 执行延迟

```rust
// 通过后需等待 2 天才能执行
proposal.execution_time = Some(now + ExecutionDelay);
```

**目的**：给用户反应时间

### 4. 活跃提案限制

```rust
// 每店铺最多 10 个活跃提案
ensure!(
    shop_proposals.len() < MaxActiveProposals,
    Error::<T>::TooManyActiveProposals
);
```

**目的**：防止 DoS 攻击

## 🧪 测试

```bash
# 运行单元测试
cargo test -p pallet-entity-governance

# 运行特定测试
cargo test -p pallet-entity-governance test_create_proposal
```

## 🚧 待实现功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 时间加权投票 | 🔜 TODO | 持有时间越长，投票权重越高 |
| 提案执行逻辑 | 🔜 TODO | 各类型提案的具体执行 |
| 委托投票 | 🔜 TODO | 将投票权委托给他人 |
| 快照投票 | 🔜 TODO | 基于提案创建时的余额快照 |

## 📝 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1.0 | 2026-01-31 | 初始版本 |

## 📄 许可证

MIT License

## 🔗 相关链接

- [Entity 设计文档](../../docs/design/entity-token-governance.md)
- [pallet-entity-token](../token/README.md)
- [Substrate 治理文档](https://docs.substrate.io/build/runtime-governance/)
