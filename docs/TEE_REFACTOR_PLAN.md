# 方案 A：完全依赖 TEE 重构设计

## 概述

本文档定义了将八字（及其他占卜模块）从"前端加密 + 链上存储"模式迁移到"完全依赖 TEE"模式的最优方案。

## 现有架构分析

### 当前数据结构

```rust
// 现有：EncryptedBaziChart（需要移除）
pub struct EncryptedBaziChart<T: Config> {
    pub owner: T::AccountId,
    pub sizhu_index: SiZhuIndex,           // 8 bytes 明文
    pub gender: Gender,                     // 1 byte 明文
    pub encrypted_data: BoundedVec<u8, 256>, // ❌ TEE 替代
    pub data_hash: [u8; 32],                // ❌ TEE 证明替代
    pub created_at: u32,
}
```

### 当前存储

```rust
// 现有存储（需要重构）
#[pallet::storage]
pub type EncryptedBaziCharts<T> = StorageMap<_, Blake2_128Concat, u64, EncryptedBaziChart<T>>;

#[pallet::storage]
pub type UserEncryptedCharts<T> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<u64, MaxChartsPerAccount>>;
```

## 新架构设计

### 核心原则

1. **链上数据最小化** - 只存储必要的引用和索引
2. **TEE 全程保护** - 敏感数据在 TEE 内处理，链上从不出现明文
3. **IPFS 存储结果** - 完整结果加密存储在 IPFS
4. **向后兼容** - 保留 `SiZhuIndex` 支持链上查询

### 新数据结构

```rust
// ==================== 新增：通用占卜结果 ====================

/// 占卜结果链上存储（所有占卜模块通用）
/// 
/// 已在 ocw-tee/types.rs 中定义为 DivinationOnChain
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct DivinationOnChain<AccountId, BlockNumber, Index> {
    /// 所有者
    pub owner: AccountId,
    /// 隐私模式
    pub privacy_mode: PrivacyMode,
    /// 类型特定索引（可选，Encrypted 模式有）
    pub type_index: Option<BoundedVec<u8, ConstU32<256>>>,
    /// IPFS 清单 CID
    pub manifest_cid: BoundedVec<u8, ConstU32<64>>,
    /// 清单哈希（用于验证）
    pub manifest_hash: [u8; 32],
    /// 生成信息（OCW 或 TEE）
    pub generation: GenerationInfo<AccountId>,
    /// 版本号
    pub version: u8,
    /// 创建时间
    pub created_at: BlockNumber,
    /// 更新时间
    pub updated_at: BlockNumber,
}

/// 生成信息
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum GenerationInfo<AccountId> {
    /// OCW 生成（公开模式）
    Ocw,
    /// TEE 生成（加密/私密模式）
    Tee {
        /// 执行节点
        node: AccountId,
        /// 计算证明
        proof: ComputationProof,
    },
}

/// TEE 计算证明
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ComputationProof {
    /// Enclave MRENCLAVE
    pub mr_enclave: [u8; 32],
    /// 输入哈希
    pub input_hash: [u8; 32],
    /// 输出哈希
    pub output_hash: [u8; 32],
    /// 计算时间戳
    pub timestamp: u64,
    /// Enclave 签名
    pub signature: [u8; 64],
}
```

### 八字专用索引

```rust
// ==================== 八字专用：保留 SiZhuIndex ====================

/// 四柱索引（8 bytes）
/// 
/// 保留此结构用于链上查询，如"查找所有甲子日生人"
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct SiZhuIndex {
    pub year_gan: u8,   // 年干 (0-9)
    pub year_zhi: u8,   // 年支 (0-11)
    pub month_gan: u8,  // 月干 (0-9)
    pub month_zhi: u8,  // 月支 (0-11)
    pub day_gan: u8,    // 日干 (0-9)
    pub day_zhi: u8,    // 日支 (0-11)
    pub hour_gan: u8,   // 时干 (0-9)
    pub hour_zhi: u8,   // 时支 (0-11)
}

/// 八字结果索引（编码后存入 type_index）
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct BaziTypeIndex {
    /// 四柱索引
    pub sizhu: SiZhuIndex,
    /// 性别
    pub gender: Gender,
}
```

### 新存储设计

```rust
// ==================== 新存储（替代旧存储） ====================

/// 占卜结果存储（通用）
/// 
/// 所有占卜模块共用，由 ocw-tee pallet 管理
#[pallet::storage]
pub type CompletedResults<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,  // request_id
    DivinationOnChain<T::AccountId, BlockNumberFor<T>, ConstU32<256>>,
>;

/// 用户请求索引
#[pallet::storage]
pub type UserRequests<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<u64, T::MaxRequestsPerUser>,
>;

// ==================== 八字专用索引（可选） ====================

/// 按日柱索引（用于查询"同日柱"的八字）
/// 
/// 仅 Encrypted 模式有此索引，Private 模式无
#[pallet::storage]
pub type BaziByDayPillar<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u8,  // day_gz (0-59)
    BoundedVec<u64, ConstU32<1000>>,  // request_ids
>;
```

## 隐私模式对比

| 模式 | 输入处理 | 计算位置 | 链上存储 | IPFS 存储 |
|------|---------|---------|---------|----------|
| **Public** | 明文提交 | OCW | `SiZhuIndex` + CID | 明文 JSON |
| **Encrypted** | 加密提交 | TEE | `SiZhuIndex` + CID + 证明 | 加密 JSON |
| **Private** | 加密提交 | TEE | CID + 证明（无索引） | 加密 JSON |

## 数据流

### Public 模式

```
用户 → 明文输入 → OCW 计算 → 生成 JSON → 上传 IPFS → 存储 CID + SiZhuIndex
```

### Encrypted 模式

```
用户 → 加密输入 → TEE 解密 → TEE 计算 → 生成加密 JSON → 上传 IPFS 
    → 存储 CID + SiZhuIndex + TEE 证明
```

### Private 模式

```
用户 → 加密输入 → TEE 解密 → TEE 计算 → 生成加密 JSON → 上传 IPFS 
    → 存储 CID + TEE 证明（无索引）
```

## 迁移方案

### Phase 1: 添加新结构（已完成）

- [x] `DivinationOnChain` 在 `ocw-tee/types.rs`
- [x] `GenerationInfo` 和 `ComputationProof`
- [x] `TeePrivacyIntegration` trait

### Phase 2: 重构 bazi pallet

#### 2.1 移除旧结构

```rust
// ❌ 移除
pub struct EncryptedBaziChart<T: Config> { ... }

// ❌ 移除存储
pub type EncryptedBaziCharts<T> = StorageMap<...>;
pub type UserEncryptedCharts<T> = StorageMap<...>;

// ❌ 移除事件
Event::EncryptedBaziChartCreated { ... }
Event::EncryptedBaziChartDeleted { ... }

// ❌ 移除 extrinsics
fn create_encrypted_bazi_chart(...) { ... }
fn delete_encrypted_bazi_chart(...) { ... }
```

#### 2.2 保留的结构

```rust
// ✅ 保留
pub struct SiZhuIndex { ... }  // 用于链上查询
pub struct BaziTypeIndex { ... }  // 编码后存入 type_index

// ✅ 保留 Runtime API
fn calculate_bazi_from_index(sizhu_index: SiZhuIndex, gender: Gender) -> BaziChart;
```

#### 2.3 新增集成

```rust
// ✅ 新增：通过 ocw-tee 创建八字请求
impl pallet_divination_ocw_tee::traits::DivinationModule for BaziModule {
    type PlainInput = BaziInput;
    type Index = BaziTypeIndex;
    type Result = BaziChart;
    
    fn compute(input: &Self::PlainInput) -> Result<Self::Result, ModuleError>;
    fn generate_index(result: &Self::Result) -> Self::Index;
}
```

### Phase 3: 更新前端 SDK

```typescript
// 旧 API（废弃）
await baziPallet.createEncryptedBaziChart(encryptedData, sizhuIndex, gender);

// 新 API
await client.bazi.createEncrypted({
  year: 1990, month: 5, day: 15, hour: 10,
  gender: 'male',
});
// 自动：加密 → 提交到 ocw-tee → TEE 计算 → 结果存 IPFS
```

## 需要修改的文件

### bazi pallet

| 文件 | 修改内容 |
|------|---------|
| `types.rs` | 移除 `EncryptedBaziChart`，保留 `SiZhuIndex` |
| `lib.rs` | 移除加密相关存储和 extrinsics |
| `ocw_tee.rs` | 已实现 `DivinationModule` trait |

### ocw-tee pallet

| 文件 | 状态 |
|------|------|
| `types.rs` | ✅ 已有 `DivinationOnChain` |
| `traits.rs` | ✅ 已有 `TeePrivacyIntegration` |
| `lib.rs` | ✅ 已重构存储和方法 |

### 其他模块

| 模块 | 修改内容 |
|------|---------|
| `qimen` | 类似 bazi，移除加密结构（如有） |
| `meihua` | 类似 bazi，移除加密结构（如有） |

## 兼容性考虑

### 数据迁移

```rust
// 迁移脚本：将旧 EncryptedBaziChart 转换为新格式
fn migrate_encrypted_charts<T: Config>() {
    for (chart_id, old_chart) in EncryptedBaziCharts::<T>::iter() {
        // 1. 创建新的 DivinationOnChain
        let new_result = DivinationOnChain {
            owner: old_chart.owner,
            privacy_mode: PrivacyMode::Encrypted,
            type_index: Some(BaziTypeIndex {
                sizhu: old_chart.sizhu_index,
                gender: old_chart.gender,
            }.encode().try_into().unwrap()),
            manifest_cid: BoundedVec::default(), // 需要重新上传 IPFS
            manifest_hash: old_chart.data_hash,
            generation: GenerationInfo::Ocw, // 旧数据标记为 OCW
            version: 1,
            created_at: old_chart.created_at.into(),
            updated_at: old_chart.created_at.into(),
        };
        
        // 2. 存储到新位置
        CompletedResults::<T>::insert(chart_id, new_result);
    }
    
    // 3. 清理旧存储
    let _ = EncryptedBaziCharts::<T>::clear(u32::MAX, None);
}
```

### Runtime API 兼容

```rust
// 保持现有 Runtime API 不变
impl BaziChartApi for Runtime {
    // ✅ 保留：基于 SiZhuIndex 计算
    fn calculate_bazi_from_index(
        sizhu_index: SiZhuIndex,
        gender: Gender,
    ) -> FullBaziChartForApi {
        // 实现不变
    }
}
```

## 总结

### 移除的内容

| 类型 | 名称 | 原因 |
|------|------|------|
| 结构体 | `EncryptedBaziChart` | TEE 替代加密存储 |
| 字段 | `encrypted_data` | 结果存 IPFS |
| 字段 | `data_hash` | TEE 证明替代 |
| 存储 | `EncryptedBaziCharts` | 使用 `CompletedResults` |
| 存储 | `UserEncryptedCharts` | 使用 `UserRequests` |
| Extrinsic | `create_encrypted_bazi_chart` | 使用 `ocw-tee::create_encrypted_request` |
| Extrinsic | `delete_encrypted_bazi_chart` | 使用通用删除接口 |

### 保留的内容

| 类型 | 名称 | 原因 |
|------|------|------|
| 结构体 | `SiZhuIndex` | 链上查询索引 |
| 结构体 | `BaziTypeIndex` | 编码存入 `type_index` |
| Runtime API | `calculate_bazi_from_index` | 免费计算接口 |
| 所有计算逻辑 | `calculations/*` | 核心算法不变 |

### 新增的内容

| 类型 | 名称 | 位置 |
|------|------|------|
| 结构体 | `DivinationOnChain` | `ocw-tee/types.rs` |
| 结构体 | `ComputationProof` | `ocw-tee/types.rs` |
| Trait | `TeePrivacyIntegration` | `ocw-tee/traits.rs` |
| Trait 实现 | `DivinationModule for BaziModule` | `bazi/ocw_tee.rs` |
