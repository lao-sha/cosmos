# Storage Pallets

存储服务模块组，包含 IPFS 存储管理和数据生命周期管理功能。

## 模块结构

```
storage/
├── service/     # 存储服务核心 (pallet-storage-service)
└── lifecycle/   # 存储生命周期管理 (pallet-storage-lifecycle)
```

## 模块说明

### service (存储服务)

**功能**：
- IPFS Pin 管理（创建、续期、取消）
- 存储运营商注册与管理
- 计费与配额系统
- 健康巡检队列

**主要类型**：
- `PinRecord` - Pin 记录
- `Operator` - 运营商信息
- `BillingRecord` - 计费记录

### lifecycle (生命周期管理)

**功能**：
- 分级归档框架 (Active → L1 → L2 → Purge)
- 自动过期处理
- 可配置的归档延迟

**主要 Trait**：
- `ArchivableData` - 可归档数据接口

## 配置参数

### StorageService
| 参数 | 说明 |
|------|------|
| `MinOperatorBond` | 运营商最低押金 |
| `MinCapacityGiB` | 最小存储容量 |
| `DefaultBillingPeriod` | 默认计费周期 |

### StorageLifecycle
| 参数 | 说明 |
|------|------|
| `L1ArchiveDelay` | Active → L1 延迟 |
| `L2ArchiveDelay` | L1 → L2 延迟 |
| `PurgeDelay` | L2 → Purge 延迟 |
| `EnablePurge` | 是否启用清除 |

## 依赖关系

```
Evidence ──────► StorageService (IpfsPinner)
Arbitration ───► StorageService + StorageLifecycle
Trading/OTC ───► StorageService + StorageLifecycle
Trading/Swap ──► StorageService + StorageLifecycle
```
