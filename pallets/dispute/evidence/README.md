# pallet-evidence

> 路径：`pallets/dispute/evidence/`

统一证据管理系统，提供链上证据提交、IPFS 存储、私密内容加密、访问控制等功能。

## 设计理念

- **CID 化设计**：链上仅存 `content_cid`，实际内容存 IPFS，降低 74.5% 存储成本
- **双模式支持**：Plain（公开）+ Commit（承诺哈希）
- **低耦合架构**：通过 `EvidenceAuthorizer` trait 实现模块解耦
- **自动 IPFS Pin**：与 `pallet-storage-service` 集成

## 核心功能

| 功能 | 说明 |
|------|------|
| 证据提交 | Plain 模式公开证据 / Commit 模式承诺哈希 |
| 私密内容 | 端到端加密、访问控制、密钥轮换 |
| CID 去重 | 全局 + 目标级去重，防止重复存储 |
| 限频控制 | 账户级 + 目标级双重限频 |
| 证据链接 | 复用已有证据到新目标 |
| 存储归档 | 90天自动归档，降低约75%存储 |

## Extrinsics

### 证据提交
| 方法 | call_index | 说明 |
|------|-----------|------|
| `commit` | 0 | 提交公开证据（imgs/vids/docs CID） |
| `commit_hash` | 1 | 提交承诺哈希（Commit 模式） |
| `append_evidence` | 11 | 追加补充证据 |
| `update_evidence_manifest` | 12 | 修改窗口内更新证据清单 |

### 证据链接
| 方法 | call_index | 说明 |
|------|-----------|------|
| `link` | 2 | 链接已有证据到目标 |
| `link_by_ns` | 3 | 按命名空间链接 |
| `unlink` | 4 | 取消链接 |
| `unlink_by_ns` | 5 | 按命名空间取消链接 |

### 私密内容管理
| 方法 | call_index | 说明 |
|------|-----------|------|
| `register_public_key` | 6 | 注册用户公钥 |
| `store_private_content` | 7 | 存储加密私密内容 |
| `grant_access` | 8 | 授予访问权限 |
| `revoke_access` | 9 | 撤销访问权限 |
| `rotate_content_keys` | 10 | 轮换加密密钥 |

## Trait 接口

### EvidenceAuthorizer（权限验证）
```rust
pub trait EvidenceAuthorizer<AccountId> {
    fn can_commit(who: &AccountId, domain: u8, target_id: u64) -> bool;
    fn can_link(who: &AccountId, domain: u8, target_id: u64, evidence_id: u64) -> bool;
}
```

### EvidenceQuery（证据查询）
```rust
pub trait EvidenceQuery {
    fn get_evidence(id: u64) -> Option<EvidenceRecord>;
    fn list_by_target(domain: u8, target_id: u64) -> Vec<u64>;
}
```

## 主要类型

### EvidenceRecord（证据记录）
```rust
pub struct EvidenceRecord<T: Config> {
    pub id: u64,
    pub owner: T::AccountId,
    pub domain: u8,
    pub target_id: u64,
    pub content_cid: BoundedVec<u8, T::MaxCidLen>,
    pub created_at: BlockNumberFor<T>,
    pub status: EvidenceStatus,
}
```

### EvidenceStatus
```rust
pub enum EvidenceStatus {
    Active,    // 有效
    Locked,    // 仲裁锁定
    Archived,  // 已归档
}
```

## 配置参数

| 参数 | 说明 | 默认值 |
|------|------|-------|
| `MaxCidLen` | CID 最大长度 | 64 |
| `MaxMemoLen` | 备注最大长度 | 256 |
| `MaxPerAccountPerDay` | 每账户每日限额 | 10 |
| `MaxPerSubjectTarget` | 每目标最大证据数 | 50 |
| `EvidenceEditWindow` | 修改窗口（块数） | 100 |

## IPFS 内容格式

```json
{
  "version": "1.0",
  "evidence_id": 123,
  "domain": 2,
  "target_id": 456,
  "content": {
    "images": ["QmXxx1", "QmXxx2"],
    "videos": ["QmYyy1"],
    "documents": ["QmZzz1"],
    "memo": "文字说明"
  }
}
```

## 集成示例

```rust
// 在业务 pallet 中实现 EvidenceAuthorizer
impl<T: Config> EvidenceAuthorizer<T::AccountId> for Pallet<T> {
    fn can_commit(who: &T::AccountId, domain: u8, target_id: u64) -> bool {
        // 验证是否为订单参与方
        Self::is_order_participant(target_id, who)
    }
}
```

## 相关模块

- `@/home/xiaodong/桌面/cosmos/pallets/storage/service/` - IPFS 存储服务
- `@/home/xiaodong/桌面/cosmos/pallets/dispute/arbitration/` - 仲裁系统（引用证据）
