# pallet-entity-tokensale

> 🎯 Entity 代币发售模块 — 多模式公开发售与锁仓机制 (Phase 8)

## 概述

`pallet-entity-tokensale` 实现实体代币公开发售（Token Sale / IEO）功能，支持 5 种发售模式、多资产支付、锁仓解锁和 KYC 集成。

### 核心功能

- **5 种发售模式** — 固定价格、荷兰拍卖、白名单分配、先到先得、抽签
- **多资产支付** — 支持原生 NXS 和其他链上资产
- **锁仓解锁** — None / Linear / Cliff / Custom，支持初始解锁比例和悬崖期
- **KYC 集成** — 可配置最低 KYC 级别要求
- **白名单管理** — 定向分配模式下限制参与者范围
- **多轮发售** — 一个实体可创建多个发售轮次

## 发售模式

| 模式 | 说明 | 价格机制 |
|------|------|----------|
| `FixedPrice` | 固定价格发售（默认） | 恒定价格 |
| `DutchAuction` | 荷兰拍卖 | 价格从 start_price 线性递减到 end_price |
| `WhitelistAllocation` | 白名单定向分配 | 固定价格，仅白名单可参与 |
| `FCFS` | 先到先得 | 固定价格，售完即止 |
| `Lottery` | 抽签发售 | 固定价格，随机分配 |

## 数据结构

### SaleRound — 发售轮次

```rust
pub struct SaleRound<...> {
    pub id: u64,                     // 轮次 ID
    pub entity_id: u64,              // 实体 ID
    pub mode: SaleMode,              // 发售模式
    pub status: RoundStatus,         // 状态
    pub total_supply: Balance,       // 代币总量
    pub sold_amount: Balance,        // 已售数量
    pub remaining_amount: Balance,   // 剩余数量
    pub participants_count: u32,     // 参与人数
    pub payment_options: BoundedVec<PaymentConfig>,  // 支付选项
    pub vesting_config: VestingConfig,               // 锁仓配置
    pub whitelist: BoundedVec<AccountId>,             // 白名单
    pub kyc_required: bool,          // 是否需要 KYC
    pub min_kyc_level: u8,           // 最低 KYC 级别 (0-4)
    pub start_block: BlockNumber,    // 开始时间
    pub end_block: BlockNumber,      // 结束时间
    pub dutch_start_price: Option<Balance>,  // 荷兰拍卖起始价
    pub dutch_end_price: Option<Balance>,    // 荷兰拍卖结束价
    pub creator: AccountId,          // 创建者
    pub created_at: BlockNumber,     // 创建时间
}
```

### VestingConfig — 锁仓配置

```rust
pub struct VestingConfig<BlockNumber> {
    pub vesting_type: VestingType,     // None / Linear / Cliff / Custom
    pub initial_unlock_bps: u16,       // 初始解锁比例（基点，1000 = 10%）
    pub cliff_duration: BlockNumber,   // 悬崖期（区块数）
    pub total_duration: BlockNumber,   // 总解锁期（区块数）
    pub unlock_interval: BlockNumber,  // 解锁间隔（区块数）
}
```

### PaymentConfig — 支付选项

```rust
pub struct PaymentConfig<AssetId, Balance> {
    pub asset_id: Option<AssetId>,          // None = 原生 NXS
    pub price: Balance,                     // 单价
    pub min_purchase: Balance,              // 最小购买量
    pub max_purchase_per_account: Balance,  // 每人最大购买量
    pub enabled: bool,                      // 是否启用
}
```

### Subscription — 认购记录

```rust
pub struct Subscription<AccountId, Balance, BlockNumber, AssetId> {
    pub subscriber: AccountId,        // 认购者
    pub round_id: u64,                // 轮次 ID
    pub amount: Balance,              // 认购数量
    pub payment_asset: Option<AssetId>, // 支付资产
    pub payment_amount: Balance,      // 支付金额
    pub subscribed_at: BlockNumber,   // 认购时间
    pub claimed: bool,                // 是否已领取
    pub unlocked_amount: Balance,     // 已解锁数量
    pub last_unlock_at: BlockNumber,  // 上次解锁时间
}
```

### 枚举类型

**RoundStatus：** NotStarted → WhitelistOpen → Active → SoldOut / Ended / Cancelled → Settling → Completed

## Runtime 配置

```rust
impl pallet_entity_tokensale::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetId = u64;
    type MaxPaymentOptions = ConstU32<5>;
    type MaxWhitelistSize = ConstU32<1000>;
    type MaxRoundsHistory = ConstU32<50>;
    type MaxSubscriptionsPerRound = ConstU32<10000>;
}
```

## Extrinsics

| Index | 函数 | 权限 | 说明 |
|-------|------|------|------|
| 0 | `create_sale_round(entity_id, mode, total_supply, start_block, end_block, kyc_required, min_kyc_level)` | 任意用户 | 创建发售轮次 |
| 1 | `add_payment_option(round_id, asset_id, price, min_purchase, max_per_account)` | 创建者 | 添加支付选项（仅 NotStarted 状态） |
| 2 | `set_vesting_config(round_id, type, initial_bps, cliff, total, interval)` | 创建者 | 设置锁仓配置（仅 NotStarted 状态） |
| 3 | `configure_dutch_auction(round_id, start_price, end_price)` | 创建者 | 配置荷兰拍卖（start > end） |
| 4 | `add_to_whitelist(round_id, accounts)` | 创建者 | 批量添加白名单 |
| 5 | `start_sale(round_id)` | 创建者 | 开始发售 |
| 6 | `subscribe(round_id, amount, payment_asset)` | 任意用户 | 认购（每人每轮一次） |
| 7 | `end_sale(round_id)` | 创建者 | 结束发售 |
| 8 | `claim_tokens(round_id)` | 认购者 | 领取代币（初始解锁部分） |
| 9 | `unlock_tokens(round_id)` | 认购者 | 解锁锁仓代币 |
| 10 | `cancel_sale(round_id)` | 创建者 | 取消发售（NotStarted / Active） |

## Storage

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `NextRoundId` | `StorageValue<u64>` | 自增轮次 ID |
| `SaleRounds` | `StorageMap<u64, SaleRound>` | 发售轮次 |
| `EntityRounds` | `StorageMap<u64, BoundedVec<u64>>` | 实体 → 轮次索引 |
| `Subscriptions` | `StorageDoubleMap<u64, AccountId, Subscription>` | 认购记录 |
| `RoundParticipants` | `StorageMap<u64, BoundedVec<AccountId>>` | 轮次参与者 |
| `RaisedFunds` | `StorageDoubleMap<u64, Option<AssetId>, Balance>` | 已募集资金（按资产分计） |

## Events

| 事件 | 说明 |
|------|------|
| `SaleRoundCreated` | 发售轮次已创建 |
| `SaleRoundStarted` | 发售已开始 |
| `SaleRoundEnded` | 发售已结束（含 sold_amount, participants_count） |
| `SaleRoundCancelled` | 发售已取消 |
| `Subscribed` | 用户已认购（含 amount, payment_amount） |
| `TokensClaimed` | 代币已领取（初始解锁） |
| `TokensUnlocked` | 代币已解锁（锁仓期后） |
| `WhitelistUpdated` | 白名单已更新 |
| `FundsWithdrawn` | 募集资金已提取 |
| `RefundProcessed` | 退款已处理 |

## Errors

| 错误 | 说明 |
|------|------|
| `RoundNotFound` | 轮次不存在 |
| `RoundNotStarted` / `RoundEnded` / `RoundCancelled` | 轮次状态不匹配 |
| `SoldOut` | 已售罄 |
| `InvalidRoundStatus` | 无效轮次状态 |
| `InsufficientBalance` | 余额不足 |
| `ExceedsPurchaseLimit` / `BelowMinPurchase` | 购买量超限/不足 |
| `NotInWhitelist` | 不在白名单中 |
| `InsufficientKycLevel` | KYC 级别不足 |
| `InvalidPaymentAsset` | 无效支付资产 |
| `AlreadySubscribed` / `NotSubscribed` | 认购状态错误 |
| `AlreadyClaimed` | 已领取 |
| `NoTokensToUnlock` / `CliffNotReached` | 解锁条件不满足 |
| `Unauthorized` | 无权限 |
| `WhitelistFull` / `RoundsHistoryFull` / `ParticipantsFull` / `PaymentOptionsFull` | 容量已满 |
| `InvalidDutchAuctionConfig` / `InvalidVestingConfig` | 配置无效 |

## 辅助函数

```rust
impl<T: Config> Pallet<T> {
    /// 获取当前价格（荷兰拍卖返回实时价格，其他返回配置价格）
    pub fn get_current_price(round_id: u64, asset_id: Option<AssetId>) -> Option<Balance>;
    /// 获取用户认购信息
    pub fn get_subscription(round_id: u64, account: &AccountId) -> Option<Subscription>;
    /// 获取用户可解锁代币数量
    pub fn get_unlockable_amount(round_id: u64, account: &AccountId) -> Balance;
}
```

## 锁仓配置示例

```rust
// 线性解锁：20% 初始解锁，30 天悬崖期，1 年总解锁期
VestingConfig {
    vesting_type: VestingType::Linear,
    initial_unlock_bps: 2000,      // 20% 初始解锁
    cliff_duration: 30 * DAYS,     // 30 天悬崖期
    total_duration: 365 * DAYS,    // 1 年总解锁期
    unlock_interval: 30 * DAYS,    // 每月解锁一次
}
```

## 发售生命周期

```
┌────────────────────────────────────────────────────────────────┐
│                      发售轮次生命周期                           │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  1. create_sale_round        → NotStarted                     │
│  2. add_payment_option       → 配置支付（可多次）              │
│  3. set_vesting_config       → 配置锁仓                       │
│  4. configure_dutch_auction  → 配置拍卖（如适用）              │
│  5. add_to_whitelist         → 添加白名单（如适用）            │
│  6. start_sale               → Active                         │
│  7. subscribe                → 用户认购                       │
│  8. end_sale / 售罄          → Ended / SoldOut                │
│  9. claim_tokens             → 领取初始解锁代币               │
│ 10. unlock_tokens            → 悬崖期后持续解锁               │
│                                                                │
│  取消路径: cancel_sale (NotStarted/Active → Cancelled)        │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1.0 | 2026-02-03 | Phase 8 初始版本 |
