# pallet-entity-disclosure

实体财务披露模块。

## 概述

本模块实现实体财务信息披露功能，支持多级别披露要求和内幕交易控制。

## 功能特性

- **多级别披露**: Basic / Standard / Enhanced / Full
- **多种披露类型**: 年报、季报、重大事件、关联交易等
- **内幕人员管理**: 记录和管理内幕人员
- **黑窗口期控制**: 限制敏感时期交易
- **披露历史追溯**: 完整的披露记录链

## 披露级别

| 级别 | 要求 | 间隔 |
|------|------|------|
| Basic | 年度简报 | 1 年 |
| Standard | 季度报告 | 3 个月 |
| Enhanced | 月度报告 + 重大事件 | 1 个月 |
| Full | 实时披露 | 即时 |

## 披露类型

```rust
pub enum DisclosureType {
    AnnualReport,              // 年度报告
    QuarterlyReport,           // 季度报告
    MonthlyReport,             // 月度报告
    MaterialEvent,             // 重大事件
    RelatedPartyTransaction,   // 关联交易
    OwnershipChange,           // 股权变动
    ManagementChange,          // 管理层变动
    BusinessChange,            // 业务变更
    RiskWarning,               // 风险警示
    DividendAnnouncement,      // 分红公告
    TokenIssuance,             // 代币发行
    Buyback,                   // 回购公告
    Other,                     // 其他
}
```

## Extrinsics

| 函数 | 说明 |
|------|------|
| `configure_disclosure` | 配置披露设置 |
| `publish_disclosure` | 发布披露 |
| `withdraw_disclosure` | 撤回披露 |
| `correct_disclosure` | 更正披露 |
| `add_insider` | 添加内幕人员 |
| `remove_insider` | 移除内幕人员 |
| `start_blackout` | 开始黑窗口期 |
| `end_blackout` | 结束黑窗口期 |

## 存储项

| 名称 | 说明 |
|------|------|
| `Disclosures` | 披露记录 |
| `DisclosureConfigs` | 实体披露配置 |
| `EntityDisclosures` | 实体披露历史索引 |
| `Insiders` | 内幕人员列表 |
| `BlackoutPeriods` | 黑窗口期状态 |

## 辅助函数

```rust
// 检查是否在黑窗口期
fn is_in_blackout(entity_id: u64) -> bool

// 检查是否是内幕人员
fn is_insider(entity_id: u64, account: &AccountId) -> bool

// 检查内幕人员能否交易
fn can_insider_trade(entity_id: u64, account: &AccountId) -> bool

// 检查披露是否逾期
fn is_disclosure_overdue(entity_id: u64) -> bool
```

## 版本

- v0.1.0 (Phase 6): 初始版本
