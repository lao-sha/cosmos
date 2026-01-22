# 占卜服务市场联盟计酬集成设计

**创建日期**: 2026-01-20  
**状态**: ✅ 已实现  
**关联模块**: `pallet-divination-market`, `pallet-affiliate`

## 1. 背景

`pallet-divination-market` 实现了去中心化的占卜服务交易市场，支持命理师注册、服务套餐、订单系统、评价机制等完整功能。

当前订单支付流程：
```
客户支付 → 平台抽成（10-20%） → 命理师收入（80-90%）
```

**问题**：如果客户是通过推荐链来的，推荐人无法获得分成奖励。

## 2. 联盟计酬系统概述

`pallet-affiliate` 提供统一的联盟计酬解决方案：

### 2.1 核心功能

| 功能 | 说明 |
|------|------|
| 推荐关系管理 | 15层推荐链，永久绑定 |
| 即时分成 | 实时转账，立即到账 |
| 周结算 | 记账分配，周期结算 |
| 混合模式 | 前N层即时 + 后M层周结算 |

### 2.2 已提供接口

```rust
// pallet-affiliate/src/types.rs
pub trait AffiliateDistributor<AccountId, Balance, BlockNumber> {
    /// 分配联盟奖励
    fn distribute_rewards(
        buyer: &AccountId,
        amount: Balance,
        target: Option<(u8, u64)>,  // (层数, 目标订单ID)
    ) -> Result<Balance, DispatchError>;
}
```

## 3. 集成方案

### 3.1 订单支付流程（集成后）

```
┌─────────────────────────────────────────────────────────────┐
│                    客户支付 100 DUST                         │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │   平台抽成 15 DUST     │ ← 根据命理师等级（10-20%）
              │      (15%)             │
              └───────────┬────────────┘
                          │
            ┌─────────────┴─────────────┐
            ▼                           ▼
   ┌─────────────────┐        ┌─────────────────┐
   │  平台留存 50%   │        │  联盟分成 50%   │
   │   7.5 DUST      │        │   7.5 DUST      │
   └─────────────────┘        └────────┬────────┘
                                       │
                                       ▼
                         ┌─────────────────────────┐
                         │  15层推荐链分配         │
                         │  L1: 20% = 1.5 DUST    │
                         │  L2: 15% = 1.125 DUST  │
                         │  L3: 10% = 0.75 DUST   │
                         │  ...                    │
                         └─────────────────────────┘

              ┌────────────────────────┐
              │  命理师收入 85 DUST    │
              │      (85%)             │
              └────────────────────────┘
```

### 3.2 分成来源选项

| 方案 | 分成来源 | 命理师收入 | 平台收入 | 推荐链收入 |
|------|----------|------------|----------|------------|
| **A（推荐）** | 从平台抽成中拨出 | 85% | 7.5% | 7.5% |
| B | 从订单总额额外扣除 | 80% | 15% | 5% |
| C | 命理师让利 | 82% | 15% | 3% |

**推荐方案A**：平台承担推荐成本，激励推荐增长。

### 3.3 触发条件

联盟分成仅在以下条件满足时触发：

1. **客户有推荐人**：`ReferralChain::get(customer).is_some()`
2. **订单状态为已完成**：`OrderStatus::Completed`
3. **无争议/退款**：排除 `Disputed` / `Refunded` 状态

## 4. 技术实现

### 4.1 Config 扩展

```rust
// pallet-divination-market/src/lib.rs
#[pallet::config]
pub trait Config: frame_system::Config + pallet_timestamp::Config {
    // ... 现有配置 ...
    
    /// 🆕 联盟分成接口
    type AffiliateDistributor: pallet_affiliate::types::AffiliateDistributor<
        Self::AccountId, 
        BalanceOf<Self>,
        BlockNumberFor<Self>
    >;
    
    /// 🆕 平台抽成中用于联盟分成的比例（基点，5000 = 50%）
    #[pallet::constant]
    type AffiliateFeeRatio: Get<u16>;
}
```

### 4.2 订单完成时分配

```rust
// 在 complete_order 或 deliver_interpretation 函数中
fn do_complete_order(order_id: u64) -> DispatchResult {
    let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
    
    // 计算平台抽成
    let provider = Providers::<T>::get(&order.provider).ok_or(Error::<T>::ProviderNotFound)?;
    let platform_fee_rate = provider.tier.platform_fee_rate();
    let platform_fee = order.amount * platform_fee_rate / 10000;
    
    // 🆕 联盟分成（从平台抽成中扣除）
    let affiliate_ratio = T::AffiliateFeeRatio::get();  // 5000 = 50%
    let affiliate_amount = platform_fee * affiliate_ratio / 10000;
    
    if !affiliate_amount.is_zero() {
        // 调用联盟分配
        let _ = T::AffiliateDistributor::distribute_rewards(
            &order.customer,
            affiliate_amount.saturated_into(),
            Some((15, order_id)),  // 15层分配
        );
    }
    
    // 平台实际留存
    let platform_retain = platform_fee.saturating_sub(affiliate_amount);
    
    // 命理师收入
    let provider_income = order.amount.saturating_sub(platform_fee);
    
    // ... 转账逻辑 ...
    
    Ok(())
}
```

### 4.3 Runtime 配置

```rust
// runtime/src/lib.rs
impl pallet_divination_market::Config for Runtime {
    // ... 现有配置 ...
    
    type AffiliateDistributor = Affiliate;  // pallet-affiliate
    type AffiliateFeeRatio = ConstU16<5000>;  // 50%
}
```

## 5. 存储影响

### 5.1 新增存储项

```rust
/// 累计联盟分成金额（用于统计）
#[pallet::storage]
pub type TotalAffiliateDistributed<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;
```

### 5.2 Order 结构扩展（可选）

```rust
pub struct Order<...> {
    // ... 现有字段 ...
    
    /// 🆕 联盟分成金额（记录用）
    pub affiliate_distributed: Balance,
}
```

## 6. 事件

```rust
/// 联盟奖励已分配
AffiliateRewardDistributed {
    order_id: u64,
    customer: T::AccountId,
    total_distributed: BalanceOf<T>,
},
```

## 7. 实现详情（2026-01-20）

### 已完成配置

| 决策项 | 实现值 |
|--------|--------|
| 是否集成联盟计酬 | ✅ 是 |
| 分成来源 | 方案A（平台抽成） |
| 联盟比例参数 | `AffiliateFeeRatio`（可配置） |
| 结算模式 | 即时（通过 `AffiliateDistributor`） |
| 分配层数 | 15层 |

### 代码变更

1. ✅ **Cargo.toml**: 添加 `pallet-affiliate` 依赖
2. ✅ **Config 扩展**: 添加 `AffiliateDistributor` 和 `AffiliateFeeRatio`
3. ✅ **存储**: 添加 `TotalAffiliateDistributed` 统计
4. ✅ **事件**: 添加 `AffiliateRewardDistributed`
5. ✅ **订单完成逻辑**: `submit_interpretation()` 中集成联盟分配

### Runtime 配置 ✅

```rust
// runtime/src/configs/mod.rs
impl pallet_divination_market::Config for Runtime {
    // ... 现有配置 ...
    type AffiliateDistributor = Affiliate;
    type AffiliateFeeRatio = ConstU16<5000>;  // 50% 平台抽成用于联盟分成
}
```

## 9. 参考

- `pallet-affiliate/src/distribute.rs` - 统一分配入口
- `pallet-affiliate/src/types.rs` - `AffiliateDistributor` trait
- `pallet-trading-otc` - OTC 订单联盟分成参考实现
