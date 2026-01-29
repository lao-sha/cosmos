# pallet-storage-lifecycle

存储生命周期管理模块，提供分级归档框架。

## 模块概述

本模块为 Substrate 链上数据提供统一的生命周期管理机制，通过分级归档策略有效降低链上存储成本。模块定义了 `ArchivableData` trait 用于数据生命周期管理，支持三级存储层次，并在 `on_idle` 中自动处理归档任务。

### 核心功能

- **分级归档**：支持三级存储层次（活跃 → L1归档 → L2归档 → 清除）
- **自动处理**：在 `on_idle` 中自动处理归档任务
- **可扩展 Trait**：通过 `ArchivableData` trait 支持任意数据类型
- **批次管理**：记录归档批次信息，便于追踪和审计
- **统计分析**：实时统计归档数量和节省的存储空间

### 架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                    存储生命周期管理                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌────────────┐    L1延迟     ┌────────────┐    L2延迟          │
│  │  活跃数据   │ ──────────▶  │  L1归档    │ ──────────▶        │
│  │  (完整)    │              │  (~50-80%) │                    │
│  └────────────┘              └────────────┘                    │
│                                    │                            │
│                               L2延迟                            │
│                                    ▼                            │
│                              ┌────────────┐    清除延迟          │
│                              │  L2归档    │ ──────────▶ 清除    │
│                              │  (~90%+)   │                     │
│                              └────────────┘                     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 归档策略

| 归档级别 | 描述 | 存储节省 | 保留内容 |
|---------|------|---------|---------|
| **Active** | 活跃数据 | 0% | 完整存储 |
| **ArchivedL1** | 一级归档 | 50-80% | 核心字段，压缩存储 |
| **ArchivedL2** | 二级归档 | 90%+ | 仅统计摘要 |
| **Purged** | 已清除 | 100% | 仅永久统计 |

## Extrinsics

本模块为基础框架模块，不直接提供外部可调用函数（extrinsics）。归档逻辑通过以下方式触发：

- **自动归档**：在 `on_idle` hook 中自动处理
- **程序化调用**：其他模块通过 `StorageLifecycleManager` 的方法进行归档操作

## 存储项

| 存储项 | 键类型 | 值类型 | 描述 |
|--------|--------|--------|------|
| `ArchiveCursor` | `BoundedVec<u8, 32>` | `u64` | 归档游标，记录每种数据类型当前处理到的ID |
| `ArchiveBatches` | `BoundedVec<u8, 32>` | `BoundedVec<ArchiveBatch, 100>` | 归档批次记录，保留最近100个批次 |
| `ArchiveStats` | `BoundedVec<u8, 32>` | `ArchiveStatistics` | 归档统计信息 |

## 事件

| 事件 | 参数 | 描述 |
|------|------|------|
| `ArchivedToL1` | `data_type`, `count`, `saved_bytes` | 数据已归档到L1级别 |
| `ArchivedToL2` | `data_type`, `count`, `saved_bytes` | 数据已归档到L2级别 |
| `DataPurged` | `data_type`, `count` | 数据已被清除 |
| `BatchCompleted` | `data_type`, `batch_id`, `level` | 归档批次处理完成 |

## 错误

| 错误 | 描述 |
|------|------|
| `DataTypeTooLong` | 数据类型标识过长（超过32字节） |
| `BatchQueueFull` | 归档批次队列已满（超过100个批次） |
| `DataNotFound` | 指定的数据不存在 |
| `InvalidArchiveState` | 数据当前状态不允许进行归档操作 |

## 配置参数

| 参数 | 类型 | 描述 | 建议值 |
|------|------|------|--------|
| `RuntimeEvent` | `Event` | 运行时事件类型 | - |
| `L1ArchiveDelay` | `u32` | L1归档延迟（区块数），数据完成后多久可以归档到L1 | ~7天（约100,800块） |
| `L2ArchiveDelay` | `u32` | L2归档延迟（区块数），L1归档后多久可以转为L2 | ~30天（约432,000块） |
| `PurgeDelay` | `u32` | 清除延迟（区块数），L2归档后多久可以清除 | ~90天（约1,296,000块） |
| `EnablePurge` | `bool` | 是否启用清除功能 | `false` |
| `MaxBatchSize` | `u32` | 每次 `on_idle` 最大处理数量 | `100` |

## 核心类型

### ArchivableData Trait

所有需要生命周期管理的数据类型都应实现此 Trait：

```rust
pub trait ArchivableData: Encode + Decode + Clone {
    /// 一级归档类型（精简摘要，~50-80%压缩）
    type ArchivedL1: Encode + Decode + Clone + MaxEncodedLen;
    /// 二级归档类型（最小摘要，~90%+压缩）
    type ArchivedL2: Encode + Decode + Clone + MaxEncodedLen;
    /// 永久统计类型
    type PermanentStats: Encode + Decode + Clone + MaxEncodedLen + Default;

    /// 获取数据ID
    fn get_id(&self) -> u64;
    
    /// 判断是否可以归档到 L1
    /// - `now`: 当前区块号
    /// - `l1_delay`: L1归档延迟（区块数）
    fn can_archive_l1(&self, now: u64, l1_delay: u64) -> bool;
    
    /// 转换为一级归档
    fn to_archived_l1(&self) -> Self::ArchivedL1;
    
    /// 判断L1归档是否可以转为L2
    /// - `archived`: L1归档数据
    /// - `now`: 当前区块号
    /// - `l2_delay`: L2归档延迟（区块数）
    fn can_archive_l2(archived: &Self::ArchivedL1, now: u64, l2_delay: u64) -> bool;
    
    /// 从一级归档转换为二级归档
    fn l1_to_l2(archived: &Self::ArchivedL1) -> Self::ArchivedL2;
    
    /// 更新永久统计
    fn update_stats(stats: &mut Self::PermanentStats, archived: &Self::ArchivedL1);
}
```

### ArchiveLevel 枚举

```rust
pub enum ArchiveLevel {
    Active,     // 活跃数据（完整存储）
    ArchivedL1, // 一级归档（精简存储）
    ArchivedL2, // 二级归档（最小存储）
    Purged,     // 已清除（仅统计）
}
```

**方法**：
- `to_u8()` - 转换为 u8 值（0-3）
- `from_u8(val)` - 从 u8 值转换

### ArchiveRecord 结构

```rust
pub struct ArchiveRecord {
    pub data_id: u64,        // 数据ID
    pub level: ArchiveLevel, // 归档级别
    pub archived_at: u64,    // 归档时间（区块号）
    pub original_size: u32,  // 原始大小（字节）
    pub archived_size: u32,  // 归档后大小（字节）
}
```

### ArchiveBatch 结构

```rust
pub struct ArchiveBatch {
    pub batch_id: u64,     // 批次ID
    pub id_start: u64,     // 数据ID范围起始
    pub id_end: u64,       // 数据ID范围结束
    pub count: u32,        // 归档数量
    pub archived_at: u64,  // 归档时间（区块号）
    pub level: u8,         // 归档级别 (0=Active, 1=L1, 2=L2, 3=Purged)
}
```

### ArchiveStatistics 结构

```rust
pub struct ArchiveStatistics {
    pub total_l1_archived: u64,  // 总归档到L1数量
    pub total_l2_archived: u64,  // 总归档到L2数量
    pub total_purged: u64,       // 总清除数量
    pub total_bytes_saved: u64,  // 节省的存储字节数
    pub last_archive_at: u64,    // 最后归档时间（区块号）
}
```

### StorageLifecycleManager

存储生命周期管理器，提供分级归档的核心逻辑：

| 方法 | 描述 |
|------|------|
| `new()` | 创建新的管理器实例 |
| `process_archival(now, max_to_process, data_type)` | 处理分级归档（在 on_idle 中调用） |
| `record_batch(data_type, id_start, id_end, count, level, now)` | 记录归档批次 |
| `update_cursor(data_type, cursor)` | 更新归档游标 |
| `get_cursor(data_type)` | 获取归档游标 |
| `record_bytes_saved(data_type, bytes)` | 更新节省的存储统计 |

## 辅助函数

| 函数 | 参数 | 描述 |
|------|------|------|
| `block_to_year_month` | `block_number: u32`, `blocks_per_day: u32` | 区块号转换为YYMM格式（假设创世区块是2024年1月） |
| `amount_to_tier` | `amount: u64` | 金额转换为档位（0-5） |

**金额档位对照表**：

| 档位 | 金额范围 |
|------|---------|
| 0 | < 100 |
| 1 | 100 - 999 |
| 2 | 1,000 - 9,999 |
| 3 | 10,000 - 99,999 |
| 4 | 100,000 - 999,999 |
| 5 | ≥ 1,000,000 |

## 使用示例

### 1. 实现 ArchivableData Trait

```rust
use pallet_storage_lifecycle::{ArchivableData, ArchiveLevel};
use codec::{Encode, Decode};
use frame_support::pallet_prelude::*;

#[derive(Encode, Decode, Clone)]
pub struct MyData {
    pub id: u64,
    pub content: Vec<u8>,
    pub created_at: u64,
    pub metadata: Vec<u8>,
}

#[derive(Encode, Decode, Clone, MaxEncodedLen)]
pub struct MyDataL1 {
    pub id: u64,
    pub content_hash: [u8; 32],
    pub created_at: u64,
}

#[derive(Encode, Decode, Clone, MaxEncodedLen)]
pub struct MyDataL2 {
    pub id: u64,
    pub created_at: u64,
}

#[derive(Encode, Decode, Clone, MaxEncodedLen, Default)]
pub struct MyStats {
    pub total_count: u64,
    pub total_size: u64,
}

impl ArchivableData for MyData {
    type ArchivedL1 = MyDataL1;
    type ArchivedL2 = MyDataL2;
    type PermanentStats = MyStats;

    fn get_id(&self) -> u64 { 
        self.id 
    }

    fn can_archive_l1(&self, now: u64, l1_delay: u64) -> bool {
        now > self.created_at + l1_delay
    }

    fn to_archived_l1(&self) -> Self::ArchivedL1 {
        MyDataL1 {
            id: self.id,
            content_hash: sp_io::hashing::blake2_256(&self.content),
            created_at: self.created_at,
        }
    }

    fn can_archive_l2(archived: &Self::ArchivedL1, now: u64, l2_delay: u64) -> bool {
        now > archived.created_at + l2_delay
    }
    
    fn l1_to_l2(archived: &Self::ArchivedL1) -> Self::ArchivedL2 {
        MyDataL2 {
            id: archived.id,
            created_at: archived.created_at,
        }
    }

    fn update_stats(stats: &mut Self::PermanentStats, _archived: &Self::ArchivedL1) {
        stats.total_count += 1;
    }
}
```

### 2. 在 on_idle 中调用归档

```rust
fn on_idle(_n: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
    use pallet_storage_lifecycle::StorageLifecycleManager;
    
    let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
    let max_process = T::MaxBatchSize::get();
    
    StorageLifecycleManager::<T>::process_archival(
        now,
        max_process,
        b"my_data",
    );
    
    remaining_weight
}
```

### 3. 记录归档批次

```rust
use pallet_storage_lifecycle::{StorageLifecycleManager, ArchiveLevel};
use frame_support::BoundedVec;

// 创建数据类型标识
let data_type: BoundedVec<u8, ConstU32<32>> = 
    b"my_data".to_vec().try_into().expect("data type too long");

// 记录归档批次
StorageLifecycleManager::<T>::record_batch(
    data_type.clone(),
    1,      // id_start
    100,    // id_end
    100,    // count
    ArchiveLevel::ArchivedL1,
    now,
)?;

// 更新游标
StorageLifecycleManager::<T>::update_cursor(data_type.clone(), 100);

// 记录节省的存储空间
StorageLifecycleManager::<T>::record_bytes_saved(&data_type, 50000);
```

### 4. 配置示例

```rust
parameter_types! {
    // 约7天（假设6秒一个块）
    pub const L1ArchiveDelay: u32 = 100_800;
    // 约30天
    pub const L2ArchiveDelay: u32 = 432_000;
    // 约90天
    pub const PurgeDelay: u32 = 1_296_000;
    pub const EnablePurge: bool = false;
    pub const MaxBatchSize: u32 = 100;
}

impl pallet_storage_lifecycle::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type L1ArchiveDelay = L1ArchiveDelay;
    type L2ArchiveDelay = L2ArchiveDelay;
    type PurgeDelay = PurgeDelay;
    type EnablePurge = EnablePurge;
    type MaxBatchSize = MaxBatchSize;
}
```

## 最佳实践

1. **延迟配置**：根据数据访问频率合理配置归档延迟，高频访问数据应设置较长的 L1 延迟
2. **批次大小**：根据链上性能调整 `MaxBatchSize`，避免单次处理过多数据影响出块
3. **清除策略**：谨慎启用 `EnablePurge`，确保重要数据已备份或已提取必要统计信息
4. **存储估算**：预估归档后的存储节省，优化 L1/L2 归档格式设计
5. **数据类型标识**：使用简短且唯一的数据类型标识，不超过32字节

## 许可证

MIT License

## 作者

Cosmos Team
