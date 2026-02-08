# pallet-entity-disclosure

> 📋 Entity 财务披露模块 — 多级别披露要求与内幕交易控制 (Phase 6)

## 概述

`pallet-entity-disclosure` 实现实体财务信息披露功能，支持多级别披露要求、13 种披露类型、内幕人员管理和黑窗口期交易控制。

### 核心功能

- **多级别披露** — Basic / Standard / Enhanced / Full，自动计算下次披露截止
- **13 种披露类型** — 年报、季报、月报、重大事件、关联交易、股权变动等
- **内幕人员管理** — Owner / Admin / Auditor / Advisor / MajorHolder 五种角色
- **黑窗口期** — 披露后自动/手动开启交易限制窗口
- **更正链** — 通过 `previous_id` 串联披露更正历史
- **违规追踪** — 逾期披露、黑窗口期交易、未披露重大事件

## 披露级别

| 级别 | 要求 | 间隔 | 配置常量 |
|------|------|------|----------|
| Basic | 年度简报 | `BasicDisclosureInterval` | ~1 年 |
| Standard | 季度报告 | `StandardDisclosureInterval` | ~3 个月 |
| Enhanced | 月度报告 + 重大事件 | `EnhancedDisclosureInterval` | ~1 个月 |
| Full | 实时披露 | 0（无固定间隔） | 即时 |

## 数据结构

### DisclosureRecord — 披露记录

```rust
pub struct DisclosureRecord<AccountId, BlockNumber, MaxCidLen> {
    pub id: u64,                                    // 披露 ID
    pub entity_id: u64,                             // 实体 ID
    pub disclosure_type: DisclosureType,            // 披露类型
    pub content_cid: BoundedVec<u8, MaxCidLen>,     // 内容 IPFS CID
    pub summary_cid: Option<BoundedVec<u8, MaxCidLen>>, // 摘要 CID
    pub discloser: AccountId,                       // 披露者
    pub disclosed_at: BlockNumber,                  // 披露时间
    pub status: DisclosureStatus,                   // 状态
    pub previous_id: Option<u64>,                   // 前一版本（更正链）
    pub verifier: Option<AccountId>,                // 验证者
    pub verified_at: Option<BlockNumber>,           // 验证时间
}
```

### DisclosureConfig — 实体披露配置

```rust
pub struct DisclosureConfig<BlockNumber> {
    pub level: DisclosureLevel,                // 披露级别
    pub insider_trading_control: bool,         // 是否启用内幕交易控制
    pub blackout_period_before: BlockNumber,   // 披露前黑窗口期
    pub blackout_period_after: BlockNumber,    // 披露后黑窗口期
    pub next_required_disclosure: BlockNumber, // 下次必须披露时间
    pub last_disclosure: BlockNumber,          // 上次披露时间
    pub violation_count: u32,                  // 连续违规次数
}
```

### InsiderRecord — 内幕人员

```rust
pub struct InsiderRecord<AccountId, BlockNumber> {
    pub account: AccountId,
    pub role: InsiderRole,     // Owner / Admin / Auditor / Advisor / MajorHolder
    pub added_at: BlockNumber,
    pub active: bool,
}
```

### 枚举类型

**DisclosureType（13 种）：**

| 类型 | 说明 |
|------|------|
| `AnnualReport` | 年度财务报告 |
| `QuarterlyReport` | 季度财务报告 |
| `MonthlyReport` | 月度财务报告 |
| `MaterialEvent` | 重大事件公告 |
| `RelatedPartyTransaction` | 关联交易披露 |
| `OwnershipChange` | 股权/代币变动 |
| `ManagementChange` | 管理层变动 |
| `BusinessChange` | 业务变更 |
| `RiskWarning` | 风险提示 |
| `DividendAnnouncement` | 分红公告 |
| `TokenIssuance` | 代币发行公告 |
| `Buyback` | 回购公告 |
| `Other` | 其他 |

**DisclosureStatus：** Pending → Published → Withdrawn / Corrected

**ViolationType：** LateDisclosure / BlackoutTrading / UndisclosedMaterialEvent

## Runtime 配置

```rust
impl pallet_entity_disclosure::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type EntityProvider = EntityRegistry;
    type MaxCidLength = ConstU32<64>;
    type MaxInsiders = ConstU32<20>;
    type MaxDisclosureHistory = ConstU32<100>;
    type BasicDisclosureInterval = ...;     // ~1 年（区块数）
    type StandardDisclosureInterval = ...;  // ~3 个月
    type EnhancedDisclosureInterval = ...;  // ~1 个月
    type MajorHolderThreshold = ConstU16<500>; // 5%
}
```

## Extrinsics

| Index | 函数 | 权限 | 说明 |
|-------|------|------|------|
| 0 | `configure_disclosure(entity_id, level, insider_control, blackout_before, blackout_after)` | Entity owner | 配置披露设置 |
| 1 | `publish_disclosure(entity_id, type, content_cid, summary_cid)` | Entity owner | 发布披露，自动触发黑窗口期 |
| 2 | `withdraw_disclosure(disclosure_id)` | Owner / discloser | 撤回已发布披露 |
| 3 | `correct_disclosure(old_id, content_cid, summary_cid)` | Entity owner | 更正披露（创建新版，旧版标记 Corrected） |
| 4 | `add_insider(entity_id, account, role)` | Entity owner | 添加内幕人员 |
| 5 | `remove_insider(entity_id, account)` | Entity owner | 移除内幕人员（标记 active=false） |
| 6 | `start_blackout(entity_id, duration)` | Entity owner | 手动开始黑窗口期 |
| 7 | `end_blackout(entity_id)` | Entity owner | 手动结束黑窗口期 |

## Storage

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `NextDisclosureId` | `StorageValue<u64>` | 自增披露 ID |
| `Disclosures` | `StorageMap<u64, DisclosureRecord>` | 披露记录 |
| `DisclosureConfigs` | `StorageMap<u64, DisclosureConfig>` | 实体披露配置 |
| `EntityDisclosures` | `StorageMap<u64, BoundedVec<u64>>` | 实体披露历史索引 |
| `Insiders` | `StorageMap<u64, BoundedVec<InsiderRecord>>` | 内幕人员列表 |
| `BlackoutPeriods` | `StorageMap<u64, (BlockNumber, BlockNumber)>` | 黑窗口期 (start, end) |

## Events

| 事件 | 说明 |
|------|------|
| `DisclosurePublished` | 披露已发布 |
| `DisclosureWithdrawn` | 披露已撤回 |
| `DisclosureCorrected` | 披露已更正（old_id → new_id） |
| `DisclosureConfigUpdated` | 披露配置已更新 |
| `InsiderAdded` | 内幕人员已添加 |
| `InsiderRemoved` | 内幕人员已移除 |
| `BlackoutStarted` | 黑窗口期已开始 |
| `BlackoutEnded` | 黑窗口期已结束 |
| `DisclosureViolation` | 披露违规 |

## Errors

| 错误 | 说明 |
|------|------|
| `EntityNotFound` | 实体不存在 |
| `NotAdmin` | 不是管理员 |
| `DisclosureNotFound` | 披露不存在 |
| `CidTooLong` | CID 过长 |
| `HistoryFull` | 历史记录已满 |
| `InsiderExists` | 内幕人员已存在 |
| `InsiderNotFound` | 内幕人员不存在 |
| `InsidersFull` | 内幕人员列表已满 |
| `InBlackoutPeriod` | 黑窗口期内禁止交易 |
| `InvalidDisclosureStatus` | 无效的披露状态 |
| `InsufficientDisclosureLevel` | 披露级别不满足要求 |
| `DisclosureIntervalNotReached` | 披露间隔未到 |

## 辅助函数

```rust
impl<T: Config> Pallet<T> {
    /// 计算下次必须披露时间
    pub fn calculate_next_disclosure(level, now) -> BlockNumber;
    /// 检查是否在黑窗口期内
    pub fn is_in_blackout(entity_id: u64) -> bool;
    /// 检查是否是内幕人员
    pub fn is_insider(entity_id: u64, account: &AccountId) -> bool;
    /// 检查内幕人员能否交易（非内幕人员始终 true）
    pub fn can_insider_trade(entity_id: u64, account: &AccountId) -> bool;
    /// 获取实体披露级别
    pub fn get_disclosure_level(entity_id: u64) -> DisclosureLevel;
    /// 检查披露是否逾期
    pub fn is_disclosure_overdue(entity_id: u64) -> bool;
}
```

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1.0 | 2026-02-03 | Phase 6 初始版本 |
