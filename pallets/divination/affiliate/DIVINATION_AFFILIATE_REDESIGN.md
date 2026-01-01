# Pallet Affiliate 占卜专用改造方案

**版本**: v1.0  
**日期**: 2025-12-31  
**状态**: 设计阶段  
**目标**: 将通用联盟计酬系统改造为占卜业务专用的计酬系统

---

## 📋 执行摘要

当前的 `pallet-affiliate` 是一个通用的联盟计酬系统，支持多种业务场景（会员购买、供奉、交易等）。本方案将其改造为**占卜业务专用**的计酬系统，聚焦于占卜服务的特殊需求。

### 核心改造目标

1. ✅ **业务聚焦** - 专注于占卜服务计酬，移除无关功能
2. ✅ **占卜类型适配** - 支持不同占卜类型的差异化计酬
3. ✅ **隐私模式集成** - 与占卜隐私模式深度集成
4. ✅ **简化架构** - 移除复杂的治理和周结算，采用即时分成
5. ✅ **降低维护成本** - 精简代码，提高可维护性

---

## 🎯 占卜业务特点分析

### 占卜服务的独特性

| 特点 | 说明 | 对计酬的影响 |
|------|------|-------------|
| **服务类型多样** | 奇门、紫微、六爻、梅花、塔罗等 | 需要支持不同类型的差异化计酬 |
| **价格差异大** | 免费预览 → 付费解盘 → 高级咨询 | 不同价格档位的分成比例可能不同 |
| **隐私敏感** | Public/Partial/Private 三种模式 | 隐私模式可能影响计酬策略 |
| **即时性强** | 用户付费后立即获得服务 | 适合即时分成，不适合周结算 |
| **无供奉时长** | 占卜是一次性服务 | 不需要活跃期管理 |
| **无会员体系** | 占卜服务独立于会员系统 | 不需要会员专用分配逻辑 |

### 当前系统的冗余功能

| 功能模块 | 当前用途 | 占卜业务是否需要 | 处理方案 |
|---------|---------|-----------------|---------|
| **周结算** | 批量结算，节省Gas | ❌ 不需要 | 移除 |
| **混合模式** | 前N层即时，后M层周结算 | ❌ 不需要 | 移除 |
| **活跃期管理** | 供奉时长管理 | ❌ 不需要 | 移除 |
| **直推活跃数** | 动态层数调整 | ❌ 不需要 | 移除 |
| **会员专用分配** | 会员费100%分配 | ❌ 不需要 | 移除 |
| **系统费用扣除** | 销毁/国库/存储 | ⚠️ 可选 | 简化为可配置 |
| **治理系统** | 全民投票修改比例 | ❌ 过于复杂 | 移除，改为管理员配置 |
| **年费价格治理** | 会员年费投票 | ❌ 不需要 | 移除 |

---

## 🔧 改造方案设计

### 方案一：精简改造（推荐）⭐

**核心思路**：保留推荐关系和即时分成，移除所有冗余功能。

#### 保留的功能

1. **推荐关系管理**（referral pallet）
   - 推荐人绑定
   - 推荐码管理
   - 推荐链查询（15层）

2. **即时分成**（instant.rs）
   - 实时转账
   - 立即到账
   - 15层分成

3. **资金托管**（escrow.rs）
   - 独立托管账户
   - 资金存取

4. **配置管理**（简化版）
   - 占卜类型分成比例配置
   - 价格档位分成比例配置

#### 移除的功能

1. ❌ 周结算模块（weekly.rs）
2. ❌ 混合模式
3. ❌ 活跃期管理
4. ❌ 直推活跃数统计
5. ❌ 会员专用分配
6. ❌ 治理系统（governance.rs）
7. ❌ 年费价格治理

#### 新增的功能

1. ✅ **占卜类型配置**
   - 每种占卜类型独立的分成比例
   - 支持动态添加新占卜类型

2. ✅ **价格档位配置**
   - 不同价格区间的差异化分成
   - 例如：0-10 DUST（5层），10-100 DUST（10层），100+ DUST（15层）

3. ✅ **隐私模式集成**
   - Public 模式：标准分成
   - Partial 模式：标准分成
   - Private 模式：可选更高分成（激励隐私保护）

4. ✅ **占卜服务分配接口**
   - 专门为占卜服务设计的分配接口
   - 自动识别占卜类型和价格档位

---

## 📦 新的数据结构设计

### 占卜类型枚举

```rust
/// 占卜类型
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
pub enum DivinationType {
    Qimen = 0,      // 奇门遁甲
    Ziwei = 1,      // 紫微斗数
    Liuyao = 2,     // 六爻
    Xiaoliuren = 3, // 小六壬
    Daliuren = 4,   // 大六壬
    Meihua = 5,     // 梅花易数
    Tarot = 6,      // 塔罗牌
    Bazi = 7,       // 八字
}
```

### 占卜分成配置

```rust
/// 占卜分成配置
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
pub struct DivinationRewardConfig {
    /// 占卜类型
    pub divination_type: DivinationType,
    
    /// 基础分成比例（15层）
    pub base_percents: LevelPercents,
    
    /// 是否启用价格档位
    pub enable_price_tiers: bool,
    
    /// 价格档位配置（可选）
    pub price_tiers: Option<BoundedVec<PriceTier, ConstU32<5>>>,
    
    /// 隐私模式加成（可选）
    pub privacy_bonus: Option<PrivacyBonus>,
}

/// 价格档位
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
pub struct PriceTier {
    /// 最低价格（包含）
    pub min_price: Balance,
    
    /// 最高价格（不包含，0表示无上限）
    pub max_price: Balance,
    
    /// 分配层数（1-15）
    pub levels: u8,
    
    /// 分成比例（可选，不设置则使用base_percents）
    pub percents: Option<LevelPercents>,
}

/// 隐私模式加成
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
pub struct PrivacyBonus {
    /// Private 模式额外加成（百分比，0-50）
    pub private_bonus: u8,
    
    /// Partial 模式额外加成（百分比，0-20）
    pub partial_bonus: u8,
}
```

### 存储项设计

```rust
// === 占卜配置存储 ===

/// 占卜类型分成配置
#[pallet::storage]
pub type DivinationConfigs<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    DivinationType,
    DivinationRewardConfig,
    ValueQuery,
    DefaultDivinationConfig,
>;

/// 系统费用配置（可选）
#[pallet::storage]
pub type SystemFeeConfig<T: Config> = StorageValue<
    _,
    SystemFees,
    ValueQuery,
    DefaultSystemFees,
>;

/// 系统费用结构
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
pub struct SystemFees {
    /// 销毁比例（0-100）
    pub burn_percent: u8,
    
    /// 国库比例（0-100）
    pub treasury_percent: u8,
    
    /// 存储比例（0-100）
    pub storage_percent: u8,
    
    /// 是否启用系统费用
    pub enabled: bool,
}
```

---

## 🔌 新的接口设计

### 核心分配接口

```rust
/// 占卜服务分配接口
/// 
/// 参数：
/// - buyer: 购买者（占卜用户）
/// - divination_type: 占卜类型
/// - amount: 支付金额
/// - privacy_mode: 隐私模式（0=Public, 1=Partial, 2=Private）
/// 
/// 返回：实际分配总额
#[pallet::call_index(20)]
#[pallet::weight(10_000)]
pub fn distribute_divination_rewards(
    origin: OriginFor<T>,
    buyer: T::AccountId,
    divination_type: DivinationType,
    amount: BalanceOf<T>,
    privacy_mode: u8,
) -> DispatchResult {
    T::AdminOrigin::ensure_origin(origin)?;
    
    let distributed = Self::do_distribute_divination_rewards(
        &buyer,
        divination_type,
        amount,
        privacy_mode,
    )?;
    
    Self::deposit_event(Event::DivinationRewardDistributed {
        buyer,
        divination_type,
        amount,
        distributed,
    });
    
    Ok(())
}
```

### 配置管理接口

```rust
/// 设置占卜类型分成配置
#[pallet::call_index(21)]
#[pallet::weight(10_000)]
pub fn set_divination_config(
    origin: OriginFor<T>,
    divination_type: DivinationType,
    config: DivinationRewardConfig,
) -> DispatchResult {
    T::AdminOrigin::ensure_origin(origin)?;
    
    // 验证配置有效性
    Self::validate_divination_config(&config)?;
    
    DivinationConfigs::<T>::insert(divination_type, config);
    
    Self::deposit_event(Event::DivinationConfigUpdated {
        divination_type,
    });
    
    Ok(())
}

/// 设置系统费用配置
#[pallet::call_index(22)]
#[pallet::weight(10_000)]
pub fn set_system_fees(
    origin: OriginFor<T>,
    fees: SystemFees,
) -> DispatchResult {
    T::AdminOrigin::ensure_origin(origin)?;
    
    // 验证总比例不超过100%
    ensure!(
        fees.burn_percent + fees.treasury_percent + fees.storage_percent <= 100,
        Error::<T>::InvalidSystemFees
    );
    
    SystemFeeConfig::<T>::put(fees);
    
    Self::deposit_event(Event::SystemFeesUpdated);
    
    Ok(())
}
```

---

## 🎨 分配逻辑实现

### 核心分配流程

```rust
impl<T: Config> Pallet<T> {
    /// 占卜服务分配实现
    pub fn do_distribute_divination_rewards(
        buyer: &T::AccountId,
        divination_type: DivinationType,
        gross_amount: BalanceOf<T>,
        privacy_mode: u8,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if gross_amount.is_zero() {
            return Ok(BalanceOf::<T>::zero());
        }
        
        // 1. 获取占卜配置
        let config = DivinationConfigs::<T>::get(divination_type);
        
        // 2. 扣除系统费用（如果启用）
        let distributable = Self::deduct_system_fees_if_enabled(buyer, gross_amount)?;
        
        // 3. 确定分配层数和比例
        let (levels, percents) = Self::determine_distribution_params(
            &config,
            distributable,
            privacy_mode,
        );
        
        // 4. 执行即时分成
        let distributed = Self::do_instant_distribute_with_config(
            buyer,
            distributable,
            levels,
            &percents,
        );
        
        Ok(distributed)
    }
    
    /// 确定分配参数（层数和比例）
    fn determine_distribution_params(
        config: &DivinationRewardConfig,
        amount: BalanceOf<T>,
        privacy_mode: u8,
    ) -> (u8, LevelPercents) {
        let mut levels = 15u8;
        let mut percents = config.base_percents.clone();
        
        // 1. 根据价格档位调整
        if config.enable_price_tiers {
            if let Some(tiers) = &config.price_tiers {
                for tier in tiers.iter() {
                    if amount >= tier.min_price.into() 
                        && (tier.max_price == 0 || amount < tier.max_price.into()) 
                    {
                        levels = tier.levels;
                        if let Some(tier_percents) = &tier.percents {
                            percents = tier_percents.clone();
                        }
                        break;
                    }
                }
            }
        }
        
        // 2. 根据隐私模式加成
        if let Some(bonus) = &config.privacy_bonus {
            let bonus_percent = match privacy_mode {
                2 => bonus.private_bonus,  // Private
                1 => bonus.partial_bonus,  // Partial
                _ => 0,                    // Public
            };
            
            if bonus_percent > 0 {
                percents = Self::apply_bonus(&percents, bonus_percent);
            }
        }
        
        (levels, percents)
    }
    
    /// 应用加成
    fn apply_bonus(percents: &LevelPercents, bonus: u8) -> LevelPercents {
        let mut result = percents.clone();
        for percent in result.iter_mut() {
            let bonus_amount = (*percent as u16 * bonus as u16) / 100;
            *percent = (*percent as u16 + bonus_amount).min(100) as u8;
        }
        result
    }
}
```

---

## 📊 默认配置示例

### 奇门遁甲配置

```rust
DivinationRewardConfig {
    divination_type: DivinationType::Qimen,
    base_percents: vec![30, 25, 15, 10, 7, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1],
    enable_price_tiers: true,
    price_tiers: Some(vec![
        PriceTier {
            min_price: 0,
            max_price: 10_000_000_000_000,  // 10 DUST
            levels: 5,
            percents: Some(vec![40, 30, 20, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        },
        PriceTier {
            min_price: 10_000_000_000_000,
            max_price: 100_000_000_000_000,  // 100 DUST
            levels: 10,
            percents: None,  // 使用 base_percents
        },
        PriceTier {
            min_price: 100_000_000_000_000,
            max_price: 0,  // 无上限
            levels: 15,
            percents: None,
        },
    ]),
    privacy_bonus: Some(PrivacyBonus {
        private_bonus: 20,  // Private 模式额外 20%
        partial_bonus: 10,  // Partial 模式额外 10%
    }),
}
```

### 塔罗牌配置（简化版）

```rust
DivinationRewardConfig {
    divination_type: DivinationType::Tarot,
    base_percents: vec![25, 20, 15, 10, 8, 5, 3, 3, 2, 2, 1, 1, 1, 1, 1],
    enable_price_tiers: false,  // 不启用价格档位
    price_tiers: None,
    privacy_bonus: None,  // 不启用隐私加成
}
```

---

## 🔄 迁移步骤

### Phase 1: 代码精简（2天）

1. **移除冗余模块**
   - [ ] 删除 `weekly.rs`
   - [ ] 删除 `governance.rs`
   - [ ] 删除 `lib.rs` 中的治理相关存储和接口
   - [ ] 删除 `types.rs` 中的 `SettlementMode::Weekly` 和 `Hybrid`

2. **简化 distribute.rs**
   - [ ] 移除 `do_distribute_rewards`（通用分配）
   - [ ] 移除 `do_distribute_membership_rewards`（会员分配）
   - [ ] 移除 `distribute_by_mode`（模式路由）
   - [ ] 保留 `deduct_system_fees`，改为可选

3. **简化 instant.rs**
   - [ ] 保留核心即时分成逻辑
   - [ ] 移除活跃期相关代码

### Phase 2: 新功能开发（3天）

1. **占卜类型配置**
   - [ ] 定义 `DivinationType` 枚举
   - [ ] 定义 `DivinationRewardConfig` 结构
   - [ ] 定义 `PriceTier` 和 `PrivacyBonus` 结构
   - [ ] 添加 `DivinationConfigs` 存储

2. **新分配接口**
   - [ ] 实现 `distribute_divination_rewards`
   - [ ] 实现 `do_distribute_divination_rewards`
   - [ ] 实现 `determine_distribution_params`
   - [ ] 实现 `apply_bonus`

3. **配置管理接口**
   - [ ] 实现 `set_divination_config`
   - [ ] 实现 `set_system_fees`
   - [ ] 添加配置验证逻辑

### Phase 3: 集成测试（2天）

1. **单元测试**
   - [ ] 测试不同占卜类型的分配
   - [ ] 测试价格档位逻辑
   - [ ] 测试隐私模式加成
   - [ ] 测试系统费用扣除

2. **集成测试**
   - [ ] 与 Qimen 模块集成测试
   - [ ] 与 Ziwei 模块集成测试
   - [ ] 与 Privacy 模块集成测试

3. **文档更新**
   - [ ] 更新 README.md
   - [ ] 添加配置示例
   - [ ] 添加前端调用示例

---

## 📈 改造效果对比

### 代码量对比

| 模块 | 改造前 | 改造后 | 减少 |
|------|--------|--------|------|
| lib.rs | 1656 行 | ~800 行 | -52% |
| weekly.rs | ~300 行 | 0 行 | -100% |
| governance.rs | ~500 行 | 0 行 | -100% |
| distribute.rs | ~150 行 | ~200 行 | +33% |
| types.rs | ~100 行 | ~200 行 | +100% |
| **总计** | ~2700 行 | ~1200 行 | **-56%** |

### 功能对比

| 功能 | 改造前 | 改造后 |
|------|--------|--------|
| 结算模式 | 3种（Weekly/Instant/Hybrid） | 1种（Instant） |
| 配置管理 | 治理投票 | 管理员配置 |
| 占卜类型支持 | 无 | 8种（可扩展） |
| 价格档位 | 无 | 支持 |
| 隐私模式集成 | 无 | 支持 |
| 维护复杂度 | 高 | 低 |

---

## 🚀 前端集成示例

### 创建占卜并分配奖励

```typescript
// 1. 用户创建占卜（例如奇门遁甲）
const chartId = await api.tx.qimen
  .createChartEncrypted(
    solarYear,
    solarMonth,
    solarDay,
    solarHour,
    solarMinute,
    1  // PrivacyMode::Partial
  )
  .signAndSend(user);

// 2. 用户支付解盘费用（例如 50 DUST）
const amount = 50_000_000_000_000;  // 50 DUST

// 3. 系统自动分配联盟奖励
await api.tx.affiliate
  .distributeDivinationRewards(
    user.address,
    0,  // DivinationType::Qimen
    amount,
    1   // PrivacyMode::Partial
  )
  .signAndSend(adminAccount);
```

### 查询占卜配置

```typescript
// 查询奇门遁甲的分成配置
const config = await api.query.affiliate.divinationConfigs(0);  // Qimen

console.log('基础分成比例:', config.basePercents.toJSON());
console.log('价格档位:', config.priceTiers.toJSON());
console.log('隐私加成:', config.privacyBonus.toJSON());
```

---

## ⚠️ 注意事项

### 向后兼容性

1. **推荐关系数据**：完全兼容，无需迁移
2. **托管账户余额**：完全兼容，无需迁移
3. **周结算数据**：需要在移除前完成所有待结算周期
4. **治理提案**：需要在移除前完成或取消所有活跃提案

### 迁移风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 周结算数据丢失 | 高 | 迁移前完成所有结算 |
| 治理提案中断 | 中 | 提前公告，完成或取消提案 |
| 前端调用失败 | 高 | 提供兼容层或同步更新前端 |

---

## 📝 总结

### 改造优势

1. ✅ **代码量减少 56%**，维护成本大幅降低
2. ✅ **专注占卜业务**，功能更加精准
3. ✅ **支持差异化计酬**，适应不同占卜类型
4. ✅ **隐私模式集成**，激励用户使用隐私保护
5. ✅ **即时分成**，用户体验更好

### 推荐实施

**推荐采用方案一（精简改造）**，理由：
- 占卜业务特点决定了不需要复杂的周结算和治理
- 即时分成更符合占卜服务的即时性特点
- 代码精简后更易维护和扩展
- 差异化配置满足不同占卜类型的需求

---

**维护者**: Stardust Team  
**最后更新**: 2025-12-31  
**版本**: v1.0

