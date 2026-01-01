# 占卜模块存储押金与数据删除机制分析方案

> **文档版本**: 1.0.0
> **创建日期**: 2025-12-28
> **状态**: 设计阶段

---

## 一、模块存储结构汇总

### 1.1 核心占卜模块（8个）

| 模块 | 存储项数量 | 主要存储项 | 现有删除能力 | 估算单条数据大小 |
|------|-----------|-----------|-------------|-----------------|
| **bazi** | 15+ | ChartById, UserCharts, InterpretationCache, MultiKeyEncryptedChartById, UserEncryptionKeys, ServiceProviders, ProviderGrants, EncryptedData, OwnerKeyBackup | ✅ delete_bazi_chart, delete_multi_key_encrypted_chart | 500-2000 bytes |
| **qimen** | 8 | ChartById, UserCharts, InterpretationCache, EncryptedChartById, UserEncryptedCharts, EncryptedData, OwnerKeyBackup, ChartCounter | ✅ delete_chart | 400-1500 bytes |
| **daliuren** | 8 | ChartById, UserCharts, InterpretationCache, EncryptedData, OwnerKeyBackup | ✅ delete_chart | 300-1200 bytes |
| **liuyao** | 8 | ChartById, UserCharts, InterpretationCache, EncryptedData, OwnerKeyBackup | ✅ delete_chart | 350-1300 bytes |
| **meihua** | 9 | ChartById, UserCharts, InterpretationCache, EncryptedData, OwnerKeyBackup | ✅ delete_chart | 300-1000 bytes |
| **tarot** | 8 | ChartById, UserCharts, InterpretationCache, EncryptedData, OwnerKeyBackup | ✅ delete_chart | 400-1200 bytes |
| **xiaoliuren** | 10 | ChartById, UserCharts, InterpretationCache, EncryptedData, OwnerKeyBackup | ✅ delete_chart | 200-800 bytes |
| **ziwei** | 9 | ChartById, UserCharts, InterpretationCache, EncryptedData, OwnerKeyBackup | ✅ delete_chart | 800-3000 bytes |

### 1.2 服务与基础设施模块（6个）

| 模块 | 存储项数量 | 主要存储项 | 现有删除能力 | 估算单条数据大小 |
|------|-----------|-----------|-------------|-----------------|
| **ai** | 14 | Requests, Results, Oracles, ActiveOracles, Disputes, UserRequests, OracleQueue, ModelConfigs, OracleModelSupports, Stats, TypeStats | ❌ 无用户删除函数 | 200-1000 bytes |
| **market** | 20+ | Providers, Packages, Orders, FollowUps, Reviews, ProviderBalances, Withdrawals, BountyQuestions, BountyAnswers, CustomerOrders, ProviderOrders | ❌ 无用户删除函数（有管理员功能） | 300-2000 bytes |
| **nft** | 13 | Nfts, ResultNftMapping, UserNfts, Listings, Offers, NftOffers, Collections, UserCollections, CollectionNfts, NftStatistics | ⚠️ 有burn但非完整删除 | 400-1500 bytes |
| **privacy** | 10+ | UserPrivacySettings, AuthorizationRecords, AccessLogs, PrivacyPolicies | ❌ 无删除函数 | 100-500 bytes |
| **almanac** | 5 | AlmanacData, OcwConfigStorage, DataAuthorities, DataStats, LastUpdatedDate | ✅ remove_almanac (Root only) | 200-800 bytes |
| **common** | 0 | 仅类型定义，无存储项 | N/A | N/A |

---

## 二、存储押金机制设计

### 2.1 设计原则

1. **公平性**: 用户支付的押金应与其占用的存储空间成正比
2. **激励性**: 鼓励用户及时删除不再需要的数据
3. **可持续性**: 押金水平应能覆盖长期存储成本
4. **灵活性**: 支持不同隐私模式的差异化定价

### 2.2 押金计算公式

#### 基础公式

```
存储押金 = 基础费率 × 数据大小系数 × 隐私模式系数 × 持久性系数
```

#### 参数定义

| 参数 | 说明 | 取值范围 |
|-----|------|---------|
| **基础费率** | 每 KB 存储的基础押金 | 建议: 0.01 DUST / KB |
| **数据大小系数** | 实际存储 KB 数（向上取整） | ceil(bytes / 1024) |
| **隐私模式系数** | 根据加密复杂度调整 | Public=1.0, Partial=1.2, Private=1.5 |
| **持久性系数** | 长期存储额外成本 | 1.0（标准）, 0.5（临时数据） |

#### 各模块押金估算

| 模块 | 隐私模式 | 估算存储(KB) | 押金计算 | 预计押金(DUST) |
|------|---------|-------------|---------|---------------|
| bazi (Public) | Public | 1.5 | 0.01 × 1.5 × 1.0 × 1.0 | 0.015 |
| bazi (Partial) | Partial | 2.0 | 0.01 × 2.0 × 1.2 × 1.0 | 0.024 |
| bazi (Private) | Private | 2.5 | 0.01 × 2.5 × 1.5 × 1.0 | 0.0375 |
| qimen | Partial | 1.5 | 0.01 × 1.5 × 1.2 × 1.0 | 0.018 |
| ziwei | Partial | 3.0 | 0.01 × 3.0 × 1.2 × 1.0 | 0.036 |
| xiaoliuren | Public | 0.8 | 0.01 × 0.8 × 1.0 × 1.0 | 0.008 |
| ai (请求) | N/A | 1.0 | 0.01 × 1.0 × 1.0 × 0.5 | 0.005 |
| market (订单) | N/A | 2.0 | 0.01 × 2.0 × 1.0 × 1.0 | 0.02 |
| nft | N/A | 1.5 | 0.01 × 1.5 × 1.0 × 1.0 | 0.015 |

### 2.3 押金管理机制

#### 押金状态流转

```
创建数据 → 锁定押金 → [数据存在期间押金冻结]
                          ↓
                    用户删除数据
                          ↓
                    返还押金（扣除手续费）
```

#### 押金返还规则

| 删除时机 | 返还比例 | 说明 |
|---------|---------|------|
| 30天内删除 | 100% | 全额返还，鼓励及时清理 |
| 30天后删除 | 90% | 扣除 10% 作为存储成本 |

扣除的 10% 进入国库（Treasury）。

---

## 三、数据删除能力分析

### 3.1 现有删除能力评估

#### ✅ 已具备完整删除能力（8个模块）

| 模块 | 删除函数 | 权限控制 | 关联数据清理 |
|------|---------|---------|-------------|
| bazi | `delete_bazi_chart`, `delete_multi_key_encrypted_chart` | 仅所有者 | ✅ 用户列表同步更新 |
| qimen | `delete_chart` | 仅所有者 | ✅ 用户列表同步更新 |
| daliuren | `delete_chart` | 仅所有者 | ✅ 用户列表同步更新 |
| liuyao | `delete_chart` | 仅所有者 | ✅ 用户列表同步更新 |
| meihua | `delete_chart` | 仅所有者 | ✅ 用户列表同步更新 |
| tarot | `delete_chart` | 仅所有者 | ✅ 用户列表同步更新 |
| xiaoliuren | `delete_chart` | 仅所有者 | ✅ 用户列表同步更新 |
| ziwei | `delete_chart` | 仅所有者 | ✅ 用户列表同步更新 |

#### ⚠️ 需要增强删除能力（4个模块）

| 模块 | 当前状态 | 需要添加的功能 | 复杂度 |
|------|---------|---------------|-------|
| **ai** | 无用户删除函数 | `cancel_request`, `cleanup_expired_results` | 中等 |
| **market** | 仅管理员功能 | `cancel_order`(用户), `close_bounty`(用户) | 较高 |
| **nft** | 有burn但不完整 | `delete_listing`, `cancel_offer`, `dissolve_collection` | 中等 |
| **privacy** | 无删除函数 | `revoke_authorization`, `clear_access_logs` | 低 |

#### ⚠️ 管理员专用删除（1个模块）

| 模块 | 删除函数 | 建议 |
|------|---------|-----|
| **almanac** | `remove_almanac` (Root only) | 保持现状，系统数据不需用户删除 |

### 3.2 删除功能缺失分析

#### 3.2.1 AI 模块

**现状问题**：
- 解读请求 (`Requests`) 创建后无法删除
- 解读结果 (`Results`) 永久存储
- 用户请求列表 (`UserRequests`) 只增不减

**建议新增函数**：

```rust
/// 取消解读请求（仅限待处理状态）
#[pallet::call_index(10)]
pub fn cancel_request(origin: OriginFor<T>, request_id: u64) -> DispatchResult;

/// 删除已完成的解读结果（仅限请求者）
#[pallet::call_index(11)]
pub fn delete_result(origin: OriginFor<T>, request_id: u64) -> DispatchResult;

/// 清理过期数据（Root权限，定期执行）
#[pallet::call_index(12)]
pub fn cleanup_expired(origin: OriginFor<T>, before_block: BlockNumber) -> DispatchResult;
```

#### 3.2.2 Market 模块

**现状问题**：
- 用户无法取消未开始的订单
- 悬赏问题无法关闭
- 服务商包无法删除（只能禁用）

**建议新增函数**：

```rust
/// 取消订单（仅限待接单状态）
#[pallet::call_index(20)]
pub fn cancel_order(origin: OriginFor<T>, order_id: u64) -> DispatchResult;

/// 关闭悬赏问题（仅限提问者，未有采纳答案时）
#[pallet::call_index(21)]
pub fn close_bounty(origin: OriginFor<T>, question_id: u64) -> DispatchResult;

/// 删除服务包（仅限服务商，无待处理订单时）
#[pallet::call_index(22)]
pub fn delete_package(origin: OriginFor<T>, package_id: u32) -> DispatchResult;
```

#### 3.2.3 NFT 模块

**现状问题**：
- 上架后无法取消
- 报价无法撤回
- 系列无法解散

**建议新增函数**：

```rust
/// 取消上架
#[pallet::call_index(15)]
pub fn cancel_listing(origin: OriginFor<T>, nft_id: u64) -> DispatchResult;

/// 撤回报价
#[pallet::call_index(16)]
pub fn cancel_offer(origin: OriginFor<T>, offer_id: u64) -> DispatchResult;

/// 解散系列（所有NFT需先移除）
#[pallet::call_index(17)]
pub fn dissolve_collection(origin: OriginFor<T>, collection_id: u32) -> DispatchResult;
```

#### 3.2.4 Privacy 模块

**现状问题**：
- 授权记录无法撤销
- 访问日志无法清理

**建议新增函数**：

```rust
/// 撤销授权
#[pallet::call_index(5)]
pub fn revoke_authorization(
    origin: OriginFor<T>,
    target: T::AccountId,
    data_type: DivinationType,
) -> DispatchResult;

/// 清理访问日志（用户自己的）
#[pallet::call_index(6)]
pub fn clear_access_logs(origin: OriginFor<T>) -> DispatchResult;
```

---

## 四、实现方案

### 4.1 统一押金 Trait 设计

```rust
/// 存储押金管理 Trait
pub trait StorageDepositManager<AccountId, Balance> {
    /// 计算存储押金
    fn calculate_deposit(data_size: u32, privacy_mode: PrivacyMode) -> Balance;

    /// 锁定存储押金
    fn reserve_deposit(who: &AccountId, amount: Balance) -> DispatchResult;

    /// 返还存储押金
    fn unreserve_deposit(
        who: &AccountId,
        amount: Balance,
        duration_blocks: BlockNumber,
    ) -> Balance;

    /// 获取用户已锁定的押金总额
    fn total_reserved(who: &AccountId) -> Balance;
}
```

### 4.2 通用删除 Trait 设计

```rust
/// 可删除数据 Trait
pub trait Deletable<AccountId, ChartId> {
    /// 检查是否可删除
    fn can_delete(who: &AccountId, chart_id: ChartId) -> bool;

    /// 执行删除
    fn do_delete(who: &AccountId, chart_id: ChartId) -> DispatchResult;

    /// 获取关联数据列表（用于级联删除）
    fn get_related_data(chart_id: ChartId) -> Vec<StorageKey>;
}
```

### 4.3 pallet-divination-common 扩展

在 common 模块中添加统一的押金计算逻辑：

```rust
// pallets/divination/common/src/deposit.rs

/// 押金配置
pub struct DepositConfig<Balance> {
    /// 每 KB 基础费率
    pub base_rate_per_kb: Balance,
    /// 最小押金
    pub minimum_deposit: Balance,
    /// 最大押金
    pub maximum_deposit: Balance,
}

/// 隐私模式系数
pub fn privacy_mode_multiplier(mode: PrivacyMode) -> u32 {
    match mode {
        PrivacyMode::Public => 100,   // 1.0x
        PrivacyMode::Partial => 120,  // 1.2x
        PrivacyMode::Private => 150,  // 1.5x
    }
}

/// 计算存储押金
pub fn calculate_storage_deposit<Balance: From<u32> + Saturating>(
    data_size_bytes: u32,
    privacy_mode: PrivacyMode,
    config: &DepositConfig<Balance>,
) -> Balance {
    let size_kb = (data_size_bytes + 1023) / 1024; // 向上取整
    let multiplier = privacy_mode_multiplier(privacy_mode);

    let deposit = config.base_rate_per_kb
        .saturating_mul(Balance::from(size_kb))
        .saturating_mul(Balance::from(multiplier))
        / Balance::from(100u32);

    deposit.clamp(config.minimum_deposit, config.maximum_deposit)
}
```

---

## 五、实施路线图

### Phase 1: 基础设施（1-2周）

- [ ] 在 `pallet-divination-common` 中实现押金计算逻辑
- [ ] 定义 `StorageDepositManager` 和 `Deletable` trait
- [ ] 编写单元测试

### Phase 2: 核心模块集成（2-3周）

- [ ] bazi 模块集成押金机制
- [ ] 其他 7 个核心占卜模块集成
- [ ] 验证删除时押金返还逻辑

### Phase 3: 服务模块增强（2-3周）

- [ ] ai 模块添加删除功能
- [ ] market 模块添加删除功能
- [ ] nft 模块完善删除功能
- [ ] privacy 模块添加撤销功能

### Phase 4: 测试与优化（1-2周）

- [ ] 集成测试
- [ ] 性能测试（大规模删除）
- [ ] 边界条件测试
- [ ] 文档更新

---

## 六、风险评估

### 6.1 技术风险

| 风险 | 影响 | 缓解措施 |
|-----|------|---------|
| 删除后数据不可恢复 | 高 | 添加确认机制，考虑软删除过渡期 |
| 押金计算不准确 | 中 | 参数可配置，支持治理调整 |
| 关联数据一致性 | 中 | 删除时确保清理所有关联存储项（Substrate 单线程顺序执行，无并发问题） |

### 6.2 经济风险

| 风险 | 影响 | 缓解措施 |
|-----|------|---------|
| 押金设置过高阻碍使用 | 高 | 从保守值开始，根据反馈调整 |
| 押金设置过低导致滥用 | 中 | 设置最小押金门槛 |
| 批量删除导致国库压力 | 低 | 返还比例机制已考虑 |

### 6.3 兼容性风险

| 风险 | 影响 | 缓解措施 |
|-----|------|---------|
| 前端需要适配 | 中 | 提供清晰的 API 文档和示例 |

> **注意**：主网尚未上线，无需考虑数据迁移问题。所有功能可在主网上线前直接集成。

---

## 七、附录

### A. 存储成本参考

| 项目 | Polkadot | Kusama | 本项目建议 |
|-----|----------|--------|----------|
| 每 byte 押金 | ~0.0001 DOT | ~0.00001 KSM | 0.00001 DUST |
| 最小押金 | 0.01 DOT | 0.001 KSM | 0.001 DUST |

### B. 相关 Substrate 参考

- `pallet-balances`: ReservableCurrency trait 实现
- `pallet-contracts`: 存储押金计算参考
- `pallet-nfts`: 数据删除与押金返还模式

### C. 配置示例

```rust
// runtime/src/lib.rs

parameter_types! {
    /// 每 KB 存储押金
    pub const StorageDepositPerKb: Balance = 10_000_000_000; // 0.01 DUST
    /// 最小存储押金
    pub const MinStorageDeposit: Balance = 1_000_000_000; // 0.001 DUST
    /// 最大存储押金
    pub const MaxStorageDeposit: Balance = 100_000_000_000_000; // 100 DUST
}

impl pallet_bazi_chart::Config for Runtime {
    // ...
    type StorageDepositPerKb = StorageDepositPerKb;
    type MinStorageDeposit = MinStorageDeposit;
    type MaxStorageDeposit = MaxStorageDeposit;
}
```

---

## 八、结论

### 可行性评估

| 方面 | 评估 | 说明 |
|-----|------|-----|
| 技术可行性 | ✅ 高 | Substrate 原生支持，参考实现充分 |
| 经济合理性 | ✅ 高 | 押金机制成熟，参数可调整 |
| 实施复杂度 | ⚠️ 中 | 核心模块已有基础，服务模块需较多开发 |
| 用户体验 | ✅ 高 | 明确的押金规则，自主删除控制 |

### 建议

1. **优先实施核心占卜模块**：这些模块已具备删除能力，仅需添加押金逻辑
2. **分阶段推进服务模块**：ai、market、nft 模块按优先级逐步添加删除功能
3. **保守定价起步**：初始押金设置偏低，根据实际使用情况调整
4. **完善前端交互**：清晰展示押金金额、删除确认、返还预期

---

*文档结束*
