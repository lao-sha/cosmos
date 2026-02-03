# pallet-entity-sale

实体代币发售模块。

## 概述

本模块实现实体代币公开发售（Token Sale / IEO）功能，支持多种发售模式和锁仓机制。

## 功能特性

- **多种发售模式**: 固定价格、荷兰拍卖、白名单、FCFS、抽签
- **多资产支付**: 支持原生代币和多种资产支付
- **锁仓解锁**: 支持线性、阶梯等多种解锁方式
- **KYC 集成**: 可配置 KYC 级别要求
- **白名单管理**: 支持定向分配

## 发售模式

| 模式 | 说明 |
|------|------|
| FixedPrice | 固定价格发售 |
| DutchAuction | 荷兰拍卖（价格递减） |
| WhitelistAllocation | 白名单定向分配 |
| FCFS | 先到先得 |
| Lottery | 抽签发售 |

## 锁仓类型

```rust
pub enum VestingType {
    None,      // 无锁仓
    Linear,    // 线性解锁
    Cliff,     // 阶梯解锁
    Custom,    // 自定义
}
```

## Extrinsics

| 函数 | 说明 |
|------|------|
| `create_sale_round` | 创建发售轮次 |
| `add_payment_option` | 添加支付选项 |
| `set_vesting_config` | 设置锁仓配置 |
| `configure_dutch_auction` | 配置荷兰拍卖 |
| `add_to_whitelist` | 添加白名单 |
| `start_sale` | 开始发售 |
| `subscribe` | 认购 |
| `end_sale` | 结束发售 |
| `claim_tokens` | 领取代币 |
| `unlock_tokens` | 解锁代币 |
| `cancel_sale` | 取消发售 |

## 存储项

| 名称 | 说明 |
|------|------|
| `SaleRounds` | 发售轮次 |
| `EntityRounds` | 实体发售索引 |
| `Subscriptions` | 认购记录 |
| `RoundParticipants` | 轮次参与者 |
| `RaisedFunds` | 已募集资金 |

## 锁仓配置示例

```rust
VestingConfig {
    vesting_type: VestingType::Linear,
    initial_unlock_bps: 2000,      // 20% 初始解锁
    cliff_duration: 30 * DAYS,     // 30 天悬崖期
    total_duration: 365 * DAYS,    // 1 年总解锁期
    unlock_interval: 30 * DAYS,    // 每月解锁一次
}
```

## 辅助函数

```rust
// 获取当前价格（支持荷兰拍卖）
fn get_current_price(round_id: u64, asset_id: Option<AssetId>) -> Option<Balance>

// 获取用户认购信息
fn get_subscription(round_id: u64, account: &AccountId) -> Option<Subscription>

// 获取可解锁数量
fn get_unlockable_amount(round_id: u64, account: &AccountId) -> Balance
```

## 版本

- v0.1.0 (Phase 8): 初始版本
