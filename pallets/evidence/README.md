# Pallet Evidence（统一证据管理系统）

## 📋 模块概述

`pallet-evidence` 是 Stardust 区块链的**统一证据管理系统**，提供链上证据提交、IPFS 内容固定、私密内容加密、访问控制、密钥轮换、CID 去重、限频控制等完整的证据管理功能。支持 Plain（明文）和 Commit（承诺哈希）两种模式，满足不同业务场景的隐私保护需求。

### 设计理念

- **CID 化设计（Phase 1.5）**：链上仅存储单一 `content_cid` 引用，实际内容存 IPFS，降低 74.5% 存储成本
- **双模式支持**：Plain 模式适用于公开证据，Commit 模式适用于隐私保护场景（KYC、OTC 等）
- **低耦合架构**：通过 trait 适配器（`EvidenceAuthorizer`）实现模块间解耦
- **自动化集成**：与 `pallet-stardust-ipfs` 集成，自动 pin 证据 CID 到 IPFS

### 核心特性

- ✅ **Phase 1.5 CID 化设计**：链上只存储单一 content_cid，实际内容存 IPFS，降低 74.5% 存储成本
- ✅ **双模式支持**：Plain 模式（公开证据）+ Commit 模式（承诺哈希）
- ✅ **私密内容管理**：端到端加密、访问控制、密钥轮换、CID 去重
- ✅ **IPFS 自动 Pin**：证据 CID 自动固定到 IPFS，确保内容持久化
- ✅ **限频控制**：账户级 + 目标级双重限频，防止滥用
- ✅ **CID 加密验证**：L-4 修复，私密内容强制 CID 加密验证
- ✅ **命名空间隔离**：支持多域证据管理（OTC、KYC 等）
- ✅ **存储膨胀防护**：自动归档 90 天前的旧证据，存储降低约 75%

---

## 🔑 核心功能

### 1. Plain 模式：公开证据提交

#### `commit`（提交证据）- call_index(0)

**调用方**：授权账户（通过 `EvidenceAuthorizer` 验证）

**功能**：提交公开证据，生成 `EvidenceId` 并落库。

**Phase 1.5 存储优化**：

| 版本 | 存储方式 | 存储成本（10 张图片） | 优化幅度 |
|-----|---------|---------------------|---------|
| 旧版 | 链上存储所有 CID 数组（imgs, vids, docs） | 840 字节 | - |
| 新版 | 链上只存储单一 content_cid | 214 字节 | 降低 74.5% ⭐ |

**IPFS 内容格式（JSON）**：

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
    "memo": "可选文字说明"
  },
  "metadata": {
    "created_at": 1234567890,
    "owner": "5GrwvaEF...",
    "encryption": {
      "enabled": true,
      "scheme": "aes256-gcm",
      "key_bundles": {...}
    }
  }
}
```

**处理流程**：

1. 验证权限（EvidenceAuthorizer）
2. 限频检查（账户级 + 目标级）
3. 检查主体配额（MaxPerSubjectTarget）
4. 验证 CID 格式、去重（使用 `media_utils::IpfsHelper`）
5. 可选全局 CID 去重（EnableGlobalCidDedup）
6. 生成 EvidenceId
7. 创建证据记录，存储到链上
8. 自动 Pin content_cid 到 IPFS（使用 `pin_cid_for_subject`）
9. 触发 `EvidenceCommitted` 事件

**函数签名**：

```rust
#[pallet::call_index(0)]
#[pallet::weight(T::WeightInfo::commit(imgs.len() as u32, vids.len() as u32, docs.len() as u32))]
pub fn commit(
    origin: OriginFor<T>,
    domain: u8,                                    // 域代码（业务域标识）
    target_id: u64,                                // 目标 ID（如 order_id）
    imgs: Vec<BoundedVec<u8, T::MaxCidLen>>,       // 图片 CID 列表
    vids: Vec<BoundedVec<u8, T::MaxCidLen>>,       // 视频 CID 列表
    docs: Vec<BoundedVec<u8, T::MaxCidLen>>,       // 文档 CID 列表
    memo: Option<BoundedVec<u8, T::MaxMemoLen>>,   // 可选文字说明
) -> DispatchResult
```

---

### 2. Commit 模式：承诺哈希提交

#### `commit_hash`（仅登记承诺哈希）- call_index(1)

**调用方**：授权账户

**功能**：仅登记承诺哈希，不在链上存储任何明文/可逆 CID。

**使用场景**：
- **KYC 证据**：链上只存承诺哈希，链下验证
- **OTC 订单证据**：防止泄露敏感信息
- **隐私保护场景**：需要证明存在但不公开内容

**承诺哈希计算**：

```
commit = blake2b256(ns || subject_id || cid_enc || salt || ver)
```

**处理流程**：

1. 验证权限（EvidenceAuthorizer）
2. 防重：承诺哈希唯一（CommitIndex）
3. 限频检查
4. 检查主体配额（MaxPerSubjectNs）
5. 生成 EvidenceId
6. 创建证据记录，存储承诺哈希
7. 触发 `EvidenceCommittedV2` 事件

**函数签名**：

```rust
#[pallet::call_index(1)]
#[pallet::weight(T::WeightInfo::commit_hash())]
pub fn commit_hash(
    origin: OriginFor<T>,
    ns: [u8; 8],                                   // 8 字节命名空间（如 b"kyc_____", b"otc_ord_"）
    subject_id: u64,                               // 业务主体 id（如订单号、账户短码）
    commit: H256,                                  // 承诺哈希
    memo: Option<BoundedVec<u8, T::MaxMemoLen>>,   // 可选文字说明
) -> DispatchResult
```

**命名空间示例**：

| 命名空间 | 业务场景 | 说明 |
|---------|---------|------|
| `b"kyc_____"` | KYC 验证 | 用户身份认证证据 |
| `b"otc_ord_"` | OTC 订单 | 订单交易证据 |
| `b"arb_case"` | 仲裁案件 | 仲裁证据提交 |
| `b"evid___"` | 通用证据 | 默认证据命名空间 |

---

### 3. 证据链接/取消链接

#### `link`（链接证据到目标）- call_index(2)

**调用方**：授权账户

**功能**：为目标链接已存在的证据（允许复用）。

**函数签名**：

```rust
#[pallet::call_index(2)]
#[pallet::weight(T::WeightInfo::link())]
pub fn link(
    origin: OriginFor<T>,
    domain: u8,        // 域代码
    target_id: u64,    // 目标 ID
    id: u64,           // 证据 ID
) -> DispatchResult
```

#### `link_by_ns`（按命名空间链接）- call_index(3)

**功能**：V2 版本，按命名空间与主体链接证据。

```rust
#[pallet::call_index(3)]
#[pallet::weight(T::WeightInfo::link_by_ns())]
pub fn link_by_ns(
    origin: OriginFor<T>,
    ns: [u8; 8],       // 命名空间
    subject_id: u64,   // 主体 ID
    id: u64,           // 证据 ID
) -> DispatchResult
```

#### `unlink`（取消链接）- call_index(4)

**调用方**：授权账户

**功能**：取消目标与证据的链接。

```rust
#[pallet::call_index(4)]
#[pallet::weight(T::WeightInfo::unlink())]
pub fn unlink(
    origin: OriginFor<T>,
    domain: u8,        // 域代码
    target_id: u64,    // 目标 ID
    id: u64,           // 证据 ID
) -> DispatchResult
```

#### `unlink_by_ns`（按命名空间取消链接）- call_index(5)

```rust
#[pallet::call_index(5)]
#[pallet::weight(T::WeightInfo::unlink_by_ns())]
pub fn unlink_by_ns(
    origin: OriginFor<T>,
    ns: [u8; 8],       // 命名空间
    subject_id: u64,   // 主体 ID
    id: u64,           // 证据 ID
) -> DispatchResult
```

---

### 4. 私密内容管理

#### `register_public_key`（注册用户公钥）- call_index(6)

**调用方**：用户

**功能**：注册用户公钥，用于加密密钥包。

**支持的密钥类型**：

| key_type | 密钥类型 | 长度要求 | 用途 |
|----------|---------|---------|------|
| 1 | RSA-2048 | 270-512 字节 | 通用加密，兼容性好 |
| 2 | Ed25519 | 32 字节 | 高性能，Substrate 原生 |
| 3 | ECDSA-P256 | 33 或 65 字节 | 椭圆曲线，安全高效 |

**函数签名**：

```rust
#[pallet::call_index(6)]
#[pallet::weight(10_000)]
pub fn register_public_key(
    origin: OriginFor<T>,
    key_data: BoundedVec<u8, T::MaxKeyLen>,  // 公钥数据
    key_type: u8,                            // 密钥类型（1-3）
) -> DispatchResult
```

#### `store_private_content`（存储私密内容）- call_index(7)

**调用方**：授权账户

**功能**：存储私密内容（端到端加密）。

**处理流程**：

1. 验证权限（EvidenceAuthorizer）
2. **CID 加密验证**（使用 `cid_validator::DefaultCidValidator::is_encrypted`）
3. CID 格式验证（使用 `media_utils::IpfsHelper::validate_cid`）
4. CID 去重检查（PrivateContentByCid）
5. 验证创建者有加密密钥
6. 验证所有授权用户已注册公钥
7. 生成 content_id
8. 创建私密内容记录
9. 存储到链上
10. 触发 `PrivateContentStored` 事件

**访问策略类型**：

```rust
pub enum AccessPolicy<T: Config> {
    /// 仅创建者可访问
    OwnerOnly,

    /// 指定用户列表
    SharedWith(AuthorizedUsers<T>),

    /// 定时访问（到期后自动撤销）
    TimeboxedAccess {
        users: AuthorizedUsers<T>,
        expires_at: BlockNumberFor<T>,
    },

    /// 治理控制
    GovernanceControlled,

    /// 基于角色的访问（扩展用）
    RoleBased(BoundedVec<u8, ConstU32<32>>),
}
```

**函数签名**：

```rust
#[pallet::call_index(7)]
#[pallet::weight(10_000)]
pub fn store_private_content(
    origin: OriginFor<T>,
    ns: [u8; 8],                                    // 命名空间
    subject_id: u64,                                // 主体 ID
    cid: BoundedVec<u8, T::MaxCidLen>,              // IPFS CID（加密内容）
    content_hash: H256,                             // 内容哈希
    encryption_method: u8,                          // 加密方法（1=AES256-GCM, 2=XChaCha20-Poly1305）
    access_policy: AccessPolicy<T>,                 // 访问策略
    encrypted_keys: EncryptedKeyBundles<T>,         // 加密密钥包
) -> DispatchResult
```

#### `grant_access`（授予访问权限）- call_index(8)

**调用方**：创建者

**功能**：授予用户访问私密内容的权限。

**函数签名**：

```rust
#[pallet::call_index(8)]
#[pallet::weight(10_000)]
pub fn grant_access(
    origin: OriginFor<T>,
    content_id: u64,                                // 内容 ID
    user: T::AccountId,                             // 被授权用户
    encrypted_key: BoundedVec<u8, ConstU32<512>>,   // 加密密钥包
) -> DispatchResult
```

#### `revoke_access`（撤销访问权限）- call_index(9)

**调用方**：创建者

**功能**：撤销用户访问权限。

**注意**：不能撤销自己的权限。

**函数签名**：

```rust
#[pallet::call_index(9)]
#[pallet::weight(10_000)]
pub fn revoke_access(
    origin: OriginFor<T>,
    content_id: u64,       // 内容 ID
    user: T::AccountId,    // 被撤销用户
) -> DispatchResult
```

#### `rotate_content_keys`（轮换内容加密密钥）- call_index(10)

**调用方**：创建者

**功能**：轮换内容加密密钥（重新加密内容）。

**使用场景**：
- 用户公钥泄露时重新加密
- 定期安全维护
- 调整授权用户列表

**函数签名**：

```rust
#[pallet::call_index(10)]
#[pallet::weight(10_000)]
pub fn rotate_content_keys(
    origin: OriginFor<T>,
    content_id: u64,                                // 内容 ID
    new_content_hash: H256,                         // 重新加密后的内容哈希
    new_encrypted_keys: BoundedVec<
        (T::AccountId, BoundedVec<u8, ConstU32<512>>),
        T::MaxAuthorizedUsers
    >,                                              // 新的加密密钥包
) -> DispatchResult
```

---

### 5. 限频控制

#### 账户级限频

**机制**：滑动窗口限频

**参数**：
- `WindowBlocks`: 窗口大小（块数）
- `MaxPerWindow`: 窗口内最多提交次数

**工作原理**：

```
窗口 1: [区块 0 - 100]   → 提交 5 次，通过
窗口 2: [区块 101 - 200] → 提交 15 次，超限（MaxPerWindow=10），拒绝
窗口 3: [区块 201 - 300] → 窗口重置，提交 3 次，通过
```

**实现逻辑**：

```rust
fn touch_window(who: &T::AccountId, now: BlockNumberFor<T>) -> Result<(), Error<T>> {
    AccountWindows::<T>::mutate(who, |w| {
        let wb = T::WindowBlocks::get();
        // 超过窗口大小，重置窗口
        if now.saturating_sub(w.window_start) >= wb {
            w.window_start = now;
            w.count = 0;
        }
    });
    let info = AccountWindows::<T>::get(who);
    // 检查是否超过窗口限制
    ensure!(info.count < T::MaxPerWindow::get(), Error::<T>::RateLimited);
    // 增加计数
    AccountWindows::<T>::mutate(who, |w| {
        w.count = w.count.saturating_add(1);
    });
    Ok(())
}
```

#### 目标级配额

**机制**：每个目标最多允许的证据数量

**参数**：
- `MaxPerSubjectTarget`: 每个目标最多证据数（Plain 模式）
- `MaxPerSubjectNs`: 每个命名空间主体最多证据数（Commit 模式）

---

### 6. CID 去重机制

#### 局部去重（必须）

**范围**：单次提交的 imgs/vids/docs 内部

**规则**：不允许重复 CID

**实现**（使用 `media_utils::IpfsHelper` 进行规范验证）：

```rust
fn validate_cid_vec(list: &Vec<BoundedVec<u8, T::MaxCidLen>>) -> Result<(), Error<T>> {
    let mut set: BTreeSet<Vec<u8>> = BTreeSet::new();
    for cid in list.iter() {
        if cid.is_empty() {
            return Err(Error::<T>::InvalidCidFormat);
        }
        // 转换为字符串进行IPFS规范验证
        let cid_str = core::str::from_utf8(cid.as_slice())
            .map_err(|_| Error::<T>::InvalidCidFormat)?;
        // 使用 media_utils 的 IpfsHelper 进行规范验证
        IpfsHelper::validate_cid(cid_str)
            .map_err(|_| Error::<T>::InvalidCidFormat)?;
        // 检查重复
        let v: Vec<u8> = cid.clone().into_inner();
        if !set.insert(v) {
            return Err(Error::<T>::DuplicateCid);
        }
    }
    Ok(())
}
```

#### 全局去重（可选）

**开关**：`EnableGlobalCidDedup`

**机制**：
- 计算 CID 的 blake2_256 哈希
- 检查 `CidHashIndex` 是否存在
- 首次出现时写入索引

---

### 7. 存储膨胀防护：证据归档

#### 自动归档机制

**功能**：自动归档 90 天前的旧证据，将完整记录转换为精简摘要，释放链上存储。

**归档条件**：
- 证据创建时间超过 90 天（1,296,000 区块，按 6 秒/块计算）
- 通过 `on_idle` hook 在空闲时间自动处理

**存储优化效果**：

| 指标 | 原始 Evidence | ArchivedEvidence | 节省 |
|------|--------------|------------------|------|
| 单条记录 | ~200 字节 | ~50 字节 | **75%** |
| 1万条证据 | 2 MB | 500 KB | **1.5 MB** |

**ArchivedEvidence 结构**：

```rust
pub struct ArchivedEvidence {
    /// 证据ID
    pub id: u64,
    /// 所属域
    pub domain: u8,
    /// 目标ID
    pub target_id: u64,
    /// 内容哈希摘要（blake2_256(content_cid)）
    pub content_hash: H256,
    /// 内容类型 (0=Image, 1=Video, 2=Document, 3=Mixed, 4=Text)
    pub content_type: u8,
    /// 创建时间（区块号）
    pub created_at: u32,
    /// 归档时间（区块号）
    pub archived_at: u32,
    /// 年月（YYMM格式，便于按月统计）
    pub year_month: u16,
}
```

**on_idle 处理逻辑**：

```rust
fn on_idle(_now: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
    // 每次最多归档 10 条证据
    let archived = Self::archive_old_evidences(10);
    // ...
}
```

---

## 📊 数据结构

### Evidence（证据记录）

```rust
pub struct Evidence<AccountId, BlockNumber, MaxContentCidLen, MaxSchemeLen> {
    /// 证据唯一 ID
    pub id: u64,

    /// 所属域（业务域标识）
    pub domain: u8,

    /// 目标 ID（如 order_id）
    pub target_id: u64,

    /// 证据所有者
    pub owner: AccountId,

    /// Phase 1.5 优化：IPFS 内容 CID
    /// - 指向 IPFS 上的 JSON 文件
    /// - 包含所有图片/视频/文档的 CID 数组
    /// - 链上只存 64 字节 CID 引用
    pub content_cid: BoundedVec<u8, MaxContentCidLen>,

    /// 内容类型标识
    /// - 便于前端快速识别和渲染
    /// - 无需下载 IPFS 内容即可知道类型
    pub content_type: ContentType,

    /// 创建时间（区块号）
    pub created_at: BlockNumber,

    /// Phase 1.5 优化：加密标识
    /// - true: content_cid 指向的内容已加密
    /// - false: 公开内容
    pub is_encrypted: bool,

    /// Phase 1.5 优化：加密方案描述（可选）
    /// - 例如："aes256-gcm", "xchacha20-poly1305"
    /// - 用于解密时选择正确的算法
    pub encryption_scheme: Option<BoundedVec<u8, MaxSchemeLen>>,

    /// 证据承诺（Commit 模式）
    /// 例如 H(ns || subject_id || cid_enc || salt || ver)
    pub commit: Option<H256>,

    /// 命名空间（8 字节），用于授权与分域检索
    pub ns: Option<[u8; 8]>,
}
```

### ContentType（内容类型）

```rust
pub enum ContentType {
    /// 图片证据（单张或多张）
    Image,

    /// 视频证据（单个或多个）
    Video,

    /// 文档证据（单个或多个）
    Document,

    /// 混合类型（图片+视频+文档）
    Mixed,

    /// 纯文本描述
    Text,
}
```

### PrivateContent（私密内容记录）

```rust
pub struct PrivateContent<T: Config> {
    /// 内容 ID
    pub id: u64,

    /// 命名空间
    pub ns: [u8; 8],

    /// 主体 ID
    pub subject_id: u64,

    /// IPFS CID（加密内容）
    pub cid: BoundedVec<u8, T::MaxCidLen>,

    /// 内容哈希（用于验证完整性）
    pub content_hash: H256,

    /// 加密方法标识
    /// 1=AES-256-GCM, 2=ChaCha20-Poly1305
    pub encryption_method: u8,

    /// 创建者
    pub creator: T::AccountId,

    /// 访问控制策略
    pub access_policy: AccessPolicy<T>,

    /// 每个授权用户的加密密钥包
    pub encrypted_keys: EncryptedKeyBundles<T>,

    /// 创建时间
    pub created_at: BlockNumberFor<T>,

    /// 最后更新时间
    pub updated_at: BlockNumberFor<T>,
}
```

### UserPublicKey（用户公钥）

```rust
pub struct UserPublicKey<T: Config> {
    /// 公钥数据（DER 格式）
    pub key_data: BoundedVec<u8, T::MaxKeyLen>,

    /// 密钥类型
    /// 1=RSA-2048, 2=Ed25519, 3=ECDSA-P256
    pub key_type: u8,

    /// 注册时间（区块号）
    pub registered_at: BlockNumberFor<T>,
}
```

### KeyRotationRecord（密钥轮换记录）

```rust
pub struct KeyRotationRecord<T: Config> {
    /// 内容 ID
    pub content_id: u64,

    /// 轮换批次
    pub rotation_round: u32,

    /// 轮换时间
    pub rotated_at: BlockNumberFor<T>,

    /// 轮换者
    pub rotated_by: T::AccountId,
}
```

### WindowInfo（限频窗口信息）

```rust
pub struct WindowInfo<BlockNumber> {
    /// 窗口起始区块
    pub window_start: BlockNumber,
    /// 窗口内提交计数
    pub count: u32,
}
```

### ArchiveStatistics（归档统计）

```rust
pub struct ArchiveStatistics {
    /// 已归档证据总数
    pub total_archived: u64,
    /// 释放的存储字节数（估算）
    pub bytes_saved: u64,
    /// 最后归档时间
    pub last_archive_block: u32,
}
```

---

## 🗄️ 存储项

### 证据存储

| 存储项 | 类型 | 说明 |
|-------|------|-----|
| `NextEvidenceId` | `StorageValue<u64>` | 下一个证据 ID（自增） |
| `Evidences` | `StorageMap<u64, Evidence>` | 证据主存储（ID → Evidence） |
| `EvidenceByTarget` | `StorageDoubleMap<(u8, u64), u64, ()>` | 按目标索引证据（domain, target_id → evidence_id） |
| `EvidenceByNs` | `StorageDoubleMap<([u8; 8], u64), u64, ()>` | 按命名空间索引证据（ns, subject_id → evidence_id） |
| `CommitIndex` | `StorageMap<H256, u64>` | 承诺哈希到 EvidenceId 的唯一索引 |
| `CidHashIndex` | `StorageMap<H256, u64>` | Plain 模式全局 CID 去重索引（blake2_256(cid) → evidence_id） |

### 配额与限频

| 存储项 | 类型 | 说明 |
|-------|------|-----|
| `EvidenceCountByTarget` | `StorageMap<(u8, u64), u32>` | 每主体（domain, target）下的证据提交计数 |
| `EvidenceCountByNs` | `StorageMap<([u8; 8], u64), u32>` | 每主体（ns, subject_id）下的证据提交计数 |
| `AccountWindows` | `StorageMap<AccountId, WindowInfo>` | 账户限频窗口存储（窗口起点与计数） |

### 私密内容存储

| 存储项 | 类型 | 说明 |
|-------|------|-----|
| `NextPrivateContentId` | `StorageValue<u64>` | 下一个私密内容 ID（自增） |
| `PrivateContents` | `StorageMap<u64, PrivateContent>` | 私密内容主存储（content_id → PrivateContent） |
| `PrivateContentByCid` | `StorageMap<BoundedVec<u8>, u64>` | 按 CID 索引私密内容（支持去重和快速查找） |
| `PrivateContentBySubject` | `StorageDoubleMap<([u8; 8], u64), u64, ()>` | 按主体索引私密内容（ns, subject_id → content_id） |
| `UserPublicKeys` | `StorageMap<AccountId, UserPublicKey>` | 用户公钥存储 |
| `KeyRotationHistory` | `StorageDoubleMap<u64, u32, KeyRotationRecord>` | 密钥轮换历史（content_id, rotation_round → record） |

### 归档存储

| 存储项 | 类型 | 说明 |
|-------|------|-----|
| `ArchivedEvidences` | `StorageMap<u64, ArchivedEvidence>` | 归档证据存储（精简摘要） |
| `EvidenceArchiveCursor` | `StorageValue<u64>` | 归档游标（已扫描到的证据ID） |
| `ArchiveStats` | `StorageValue<ArchiveStatistics>` | 归档统计信息 |

---

## 📡 事件定义

### 证据事件（Plain 模式）

```rust
/// 证据已提交
EvidenceCommitted {
    id: u64,
    domain: u8,
    target_id: u64,
    owner: T::AccountId,
}

/// 证据已链接
EvidenceLinked {
    domain: u8,
    target_id: u64,
    id: u64,
}

/// 证据已取消链接
EvidenceUnlinked {
    domain: u8,
    target_id: u64,
    id: u64,
}
```

### 证据事件（Commit 模式）

```rust
/// 证据已提交（V2）
EvidenceCommittedV2 {
    id: u64,
    ns: [u8; 8],
    subject_id: u64,
    owner: T::AccountId,
}

/// 证据已链接（V2）
EvidenceLinkedV2 {
    ns: [u8; 8],
    subject_id: u64,
    id: u64,
}

/// 证据已取消链接（V2）
EvidenceUnlinkedV2 {
    ns: [u8; 8],
    subject_id: u64,
    id: u64,
}
```

### 限频与配额事件

```rust
/// 因限频或配额被限制
EvidenceThrottled(
    T::AccountId,
    u8,  // reason_code: 1=RateLimited, 2=Quota
)

/// 达到主体配额上限
EvidenceQuotaReached(
    u8,   // 0=target, 1=ns
    u64,  // subject_id or target_id
)
```

### 私密内容事件

```rust
/// 私密内容已存储
PrivateContentStored {
    content_id: u64,
    ns: [u8; 8],
    subject_id: u64,
    cid: BoundedVec<u8, T::MaxCidLen>,
    creator: T::AccountId,
}

/// 访问权限已授予
AccessGranted {
    content_id: u64,
    user: T::AccountId,
    granted_by: T::AccountId,
}

/// 访问权限已撤销
AccessRevoked {
    content_id: u64,
    user: T::AccountId,
    revoked_by: T::AccountId,
}

/// 密钥已轮换
KeysRotated {
    content_id: u64,
    rotation_round: u32,
    rotated_by: T::AccountId,
}

/// 用户公钥已注册
PublicKeyRegistered {
    user: T::AccountId,
    key_type: u8,
}
```

### 归档事件

```rust
/// 证据已归档
EvidenceArchived {
    id: u64,
    domain: u8,
    target_id: u64,
}
```

---

## ❌ 错误定义

```rust
pub enum Error<T> {
    /// 权限不足（命名空间或账户不被授权）
    NotAuthorized,

    /// 未找到目标对象
    NotFound,

    /// 私密内容未找到
    PrivateContentNotFound,

    /// 用户公钥未注册
    PublicKeyNotRegistered,

    /// 无权访问此内容
    AccessDenied,

    /// CID 已存在（去重检查）
    CidAlreadyExists,

    /// 授权用户数量过多
    TooManyAuthorizedUsers,

    /// 无效的加密密钥格式
    InvalidEncryptedKey,

    /// 密钥类型不支持
    UnsupportedKeyType,

    /// 图片数量超过上限
    TooManyImages,

    /// 视频数量超过上限
    TooManyVideos,

    /// 文档数量超过上限
    TooManyDocs,

    /// CID 长度或格式非法（非可见 ASCII 或为空）
    InvalidCidFormat,

    /// 发现重复的 CID 输入
    DuplicateCid,

    /// 提交的承诺已存在（防重）
    CommitAlreadyExists,

    /// 证据命名空间与当前操作命名空间不匹配
    NamespaceMismatch,

    /// 账号在窗口内达到提交上限
    RateLimited,

    /// 该主体已达到最大证据条数
    TooManyForSubject,

    /// 全局 CID 去重命中（Plain 模式）
    DuplicateCidGlobal,
}
```

---

## ⚙️ 配置参数

### Config Trait 定义

```rust
#[pallet::config]
pub trait Config: frame_system::Config + TypeInfo + core::fmt::Debug {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    
    // Phase 1.5优化：新的泛型参数（CID化版本）
    /// 内容CID最大长度（IPFS CID，建议64字节）
    #[pallet::constant]
    type MaxContentCidLen: Get<u32>;
    /// 加密方案描述最大长度（建议32字节）
    #[pallet::constant]
    type MaxSchemeLen: Get<u32>;
    
    // 旧版泛型参数（保留以向后兼容旧API）
    #[pallet::constant]
    type MaxCidLen: Get<u32>;
    #[pallet::constant]
    type MaxImg: Get<u32>;
    #[pallet::constant]
    type MaxVid: Get<u32>;
    #[pallet::constant]
    type MaxDoc: Get<u32>;
    #[pallet::constant]
    type MaxMemoLen: Get<u32>;
    #[pallet::constant]
    type MaxAuthorizedUsers: Get<u32>;
    #[pallet::constant]
    type MaxKeyLen: Get<u32>;
    #[pallet::constant]
    type EvidenceNsBytes: Get<[u8; 8]>;
    
    /// 授权验证器
    type Authorizer: EvidenceAuthorizer<Self::AccountId>;
    
    #[pallet::constant]
    type MaxPerSubjectTarget: Get<u32>;
    #[pallet::constant]
    type MaxPerSubjectNs: Get<u32>;
    #[pallet::constant]
    type WindowBlocks: Get<BlockNumberFor<Self>>;
    #[pallet::constant]
    type MaxPerWindow: Get<u32>;
    #[pallet::constant]
    type EnableGlobalCidDedup: Get<bool>;
    #[pallet::constant]
    type MaxListLen: Get<u32>;
    
    type WeightInfo: WeightInfo;
    
    // IPFS自动Pin相关配置
    /// IPFS自动pin提供者
    type IpfsPinner: pallet_stardust_ipfs::IpfsPinner<Self::AccountId, Self::Balance>;
    /// 余额类型（用于IPFS存储费用支付）
    type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
    /// 默认IPFS存储单价（每副本每月）
    #[pallet::constant]
    type DefaultStoragePrice: Get<Self::Balance>;
}
```

### Runtime 配置示例

```rust
parameter_types! {
    pub const EvidenceMaxCidLen: u32 = 64;
    pub const EvidenceMaxImg: u32 = 20;
    pub const EvidenceMaxVid: u32 = 5;
    pub const EvidenceMaxDoc: u32 = 5;
    pub const EvidenceMaxMemoLen: u32 = 64;
    pub const EvidenceNsBytes: [u8; 8] = *b"evid___ ";
}

impl pallet_evidence::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    // Phase 1.5 优化参数
    type MaxContentCidLen = ConstU32<64>;    // 内容 CID 最大长度
    type MaxSchemeLen = ConstU32<32>;        // 加密方案名称最大长度

    // 旧版兼容参数
    type MaxCidLen = EvidenceMaxCidLen;
    type MaxImg = EvidenceMaxImg;
    type MaxVid = EvidenceMaxVid;
    type MaxDoc = EvidenceMaxDoc;
    type MaxMemoLen = EvidenceMaxMemoLen;
    type EvidenceNsBytes = EvidenceNsBytes;

    // 授权与验证
    type Authorizer = AllowAllEvidenceAuthorizer;

    // 配额与限频
    type MaxPerSubjectTarget = ConstU32<10_000>;
    type MaxPerSubjectNs = ConstU32<10_000>;
    type WindowBlocks = ConstU32<600>;           // 600 块 ≈ 1 小时（6s/块）
    type MaxPerWindow = ConstU32<100>;

    // CID 去重
    type EnableGlobalCidDedup = ConstBool<false>;

    // 查询限制
    type MaxListLen = ConstU32<512>;

    // 权重
    type WeightInfo = pallet_evidence::weights::SubstrateWeight<Runtime>;

    // 私密内容参数
    type MaxAuthorizedUsers = ConstU32<64>;
    type MaxKeyLen = ConstU32<4096>;

    // IPFS 自动 Pin
    type IpfsPinner = StardustIpfs;
    type Balance = Balance;
    type DefaultStoragePrice = ConstU128<1_000_000_000_000>;  // 1 DUST/副本/月
}
```

### 参数说明

| 参数 | 默认值 | 说明 |
|-----|-------|------|
| `MaxContentCidLen` | 64 | 内容 CID 最大长度（IPFS CID） |
| `MaxSchemeLen` | 32 | 加密方案描述最大长度 |
| `MaxCidLen` | 64 | CID 最大长度（旧版兼容） |
| `MaxImg` | 20 | 最多图片数（旧版兼容） |
| `MaxVid` | 5 | 最多视频数（旧版兼容） |
| `MaxDoc` | 5 | 最多文档数（旧版兼容） |
| `MaxMemoLen` | 64 | 备注最大长度 |
| `MaxPerSubjectTarget` | 10,000 | 每个目标最多证据数 |
| `MaxPerSubjectNs` | 10,000 | 每个命名空间主体最多证据数 |
| `WindowBlocks` | 600 | 限频窗口大小（块）≈ 1 小时 |
| `MaxPerWindow` | 100 | 窗口内最多提交次数 |
| `EnableGlobalCidDedup` | false | 是否启用全局 CID 去重 |
| `MaxListLen` | 512 | 查询列表最大长度 |
| `MaxAuthorizedUsers` | 64 | 私密内容最多授权用户数 |
| `MaxKeyLen` | 4096 | 加密密钥最大长度（支持 RSA-2048） |
| `DefaultStoragePrice` | 1 DUST | 默认 IPFS 存储单价（每副本每月） |

---

## 🔧 辅助函数

### 承诺哈希计算与验证

```rust
/// 计算 Evidence 承诺哈希
/// 使用 media_utils::HashHelper 计算标准格式的承诺哈希:
/// H(ns || subject_id || cid || salt || version)
pub fn compute_evidence_commitment(
    ns: &[u8; 8],
    subject_id: u64,
    cid: &[u8],
    salt: &[u8],
    version: u32,
) -> H256

/// 验证承诺哈希是否正确
pub fn verify_evidence_commitment(
    ns: &[u8; 8],
    subject_id: u64,
    cid: &[u8],
    salt: &[u8],
    version: u32,
    expected_commit: &H256,
) -> bool

/// 验证单个 CID 格式
pub fn validate_single_cid(cid: &[u8]) -> Result<(), Error<T>>

/// 验证内容完整性
pub fn verify_content_integrity(content_data: &[u8], cid: &str) -> bool
```

### 私密内容查询

```rust
/// 检查用户是否有访问特定私密内容的权限
pub fn can_access_private_content(content_id: u64, user: &T::AccountId) -> bool

/// 获取用户的加密密钥包
pub fn get_encrypted_key_for_user(
    content_id: u64,
    user: &T::AccountId,
) -> Option<BoundedVec<u8, T::MaxKeyLen>>

/// 通过CID查找私密内容
pub fn get_private_content_by_cid(
    cid: &BoundedVec<u8, T::MaxCidLen>,
) -> Option<PrivateContent<T>>

/// 获取主体下的所有私密内容ID
pub fn get_private_content_ids_by_subject(ns: [u8; 8], subject_id: u64) -> Vec<u64>
```

### 证据查询

```rust
/// 按 (domain, target) 分页列出 evidence id
pub fn list_ids_by_target(
    domain: u8,
    target_id: u64,
    start_id: u64,
    limit: u32,
) -> Vec<u64>

/// 按 (ns, subject_id) 分页列出 evidence id
pub fn list_ids_by_ns(
    ns: [u8; 8],
    subject_id: u64,
    start_id: u64,
    limit: u32,
) -> Vec<u64>

/// 获取主体证据数量
pub fn count_by_target(domain: u8, target_id: u64) -> u32
pub fn count_by_ns(ns: [u8; 8], subject_id: u64) -> u32
```

### 归档函数

```rust
/// 归档旧证据（每次最多处理 max_count 条）
/// 归档条件：证据创建时间超过 90 天（1_296_000 区块）
pub fn archive_old_evidences(max_count: u32) -> u32
```

---

## 🔗 Trait 定义

### EvidenceAuthorizer（授权适配接口）

```rust
/// 授权适配接口：由 runtime 实现并桥接到 pallet-authorizer
pub trait EvidenceAuthorizer<AccountId> {
    /// 校验某账户是否在给定命名空间下被授权提交/链接证据
    fn is_authorized(ns: [u8; 8], who: &AccountId) -> bool;
}
```

### EvidenceProvider（只读查询接口）

```rust
/// 只读查询 trait：供其他 pallet 低耦合读取证据
pub trait EvidenceProvider<AccountId> {
    fn get(id: u64) -> Option<()>;
}
```

### PrivateContentProvider（私密内容查询接口）

```rust
/// 私密内容查询接口：供其他 pallet 使用
pub trait PrivateContentProvider<AccountId> {
    /// 检查用户是否可以访问指定的私密内容
    fn can_access(content_id: u64, user: &AccountId) -> bool;
    /// 获取用户的解密密钥
    fn get_decryption_key(content_id: u64, user: &AccountId) -> Option<Vec<u8>>;
}
```

---

## 💻 使用示例

### Rust 代码示例

#### 示例 1：提交公开证据（Plain 模式）

```rust
use frame_support::dispatch::DispatchResult;

// 准备图片 CID
let img_cids = vec![
    BoundedVec::try_from(b"QmImage1".to_vec()).unwrap(),
    BoundedVec::try_from(b"QmImage2".to_vec()).unwrap(),
];

// 提交证据
let result = Evidence::commit(
    RuntimeOrigin::signed(owner_account),
    1,                  // domain: OTC
    order_id,           // target_id
    img_cids,           // imgs
    vec![],             // vids (空)
    vec![],             // docs (空)
    None,               // memo (无)
)?;

// 监听事件
System::assert_has_event(
    Event::Evidence(pallet_evidence::Event::EvidenceCommitted {
        id: evidence_id,
        domain: 1,
        target_id: order_id,
        owner: owner_account,
    })
);
```

#### 示例 2：提交承诺哈希（Commit 模式）

```rust
use sp_core::{blake2_256, H256};

// 计算承诺哈希
let ns = *b"otc_ord_";
let subject_id = order_id;
let cid_enc = b"enc-QmEncryptedContent";
let salt = b"random_salt_12345678";
let ver = 1u32;

// 使用模块提供的辅助函数计算承诺哈希
let commit = Evidence::compute_evidence_commitment(
    &ns,
    subject_id,
    cid_enc,
    salt,
    ver,
);

// 提交承诺哈希
let result = Evidence::commit_hash(
    RuntimeOrigin::signed(submitter),
    ns,
    subject_id,
    commit,
    None,  // memo (无)
)?;
```

#### 示例 3：注册公钥并存储私密内容

```rust
// 步骤 1: 注册用户公钥
let rsa_public_key = /* RSA-2048 公钥 DER 格式 */;
let key_data = BoundedVec::try_from(rsa_public_key).unwrap();

Evidence::register_public_key(
    RuntimeOrigin::signed(user_account),
    key_data,
    1,  // key_type: RSA-2048
)?;

// 步骤 2: 准备加密内容（CID 必须带加密前缀）
let encrypted_content_cid = BoundedVec::try_from(b"enc-QmEncryptedContent".to_vec()).unwrap();
let content_hash = H256::from(blake2_256(b"original_content"));

// 步骤 3: 准备访问策略（指定用户）
let access_policy = AccessPolicy::SharedWith(authorized_users);

// 步骤 4: 准备加密密钥包
let encrypted_key = /* 使用用户公钥加密的 AES 密钥 */;
let encrypted_keys = BoundedVec::try_from(vec![
    (user_account.clone(), BoundedVec::try_from(encrypted_key).unwrap()),
]).unwrap();

// 步骤 5: 存储私密内容
Evidence::store_private_content(
    RuntimeOrigin::signed(creator_account),
    *b"priv_otc",      // ns: OTC订单私密内容
    order_id,           // subject_id
    encrypted_content_cid,
    content_hash,
    1,                  // encryption_method: AES256-GCM
    access_policy,
    encrypted_keys,
)?;
```

#### 示例 4：授予和撤销访问权限

```rust
// 授予访问权限
let new_user_encrypted_key = /* 使用 new_user 公钥加密的密钥 */;

Evidence::grant_access(
    RuntimeOrigin::signed(creator_account),
    content_id,
    new_user_account,
    BoundedVec::try_from(new_user_encrypted_key).unwrap(),
)?;

// 撤销访问权限
Evidence::revoke_access(
    RuntimeOrigin::signed(creator_account),
    content_id,
    old_user_account,
)?;
```

#### 示例 5：密钥轮换

```rust
// 重新加密内容，生成新的哈希和密钥包
let new_content_hash = H256::from(blake2_256(b"re_encrypted_content"));

let new_encrypted_keys = BoundedVec::try_from(vec![
    (user1.clone(), BoundedVec::try_from(encrypted_key_1).unwrap()),
    (user2.clone(), BoundedVec::try_from(encrypted_key_2).unwrap()),
]).unwrap();

// 轮换密钥
Evidence::rotate_content_keys(
    RuntimeOrigin::signed(creator_account),
    content_id,
    new_content_hash,
    new_encrypted_keys,
)?;
```

#### 示例 6：查询证据

```rust
// 查询单个证据
let evidence = Evidence::evidences(evidence_id).unwrap();
println!("Owner: {:?}", evidence.owner);
println!("Content CID: {:?}", String::from_utf8_lossy(&evidence.content_cid));
println!("Content Type: {:?}", evidence.content_type);
println!("Is Encrypted: {}", evidence.is_encrypted);

// 查询目标的所有证据 ID
let evidence_ids = Evidence::list_ids_by_target(
    1,              // domain: OTC
    order_id,       // target_id
    0,              // start_id
    100,            // limit
);

// 查询证据数量
let count = Evidence::count_by_target(1, order_id);

// 查询私密内容
let private_content = Evidence::private_contents(content_id).unwrap();

// 检查访问权限
let can_access = Evidence::can_access_private_content(content_id, &user_account);

// 获取加密密钥包
if let Some(encrypted_key) = Evidence::get_encrypted_key_for_user(content_id, &user_account) {
    println!("Encrypted key: {:?}", encrypted_key);
}
```

---

### TypeScript/JavaScript 代码示例（Polkadot.js API）

#### 示例 1：提交公开证据

```typescript
import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';

// 连接到节点
const provider = new WsProvider('ws://localhost:9944');
const api = await ApiPromise.create({ provider });

// 准备账户
const keyring = new Keyring({ type: 'sr25519' });
const owner = keyring.addFromUri('//Alice');

// 提交证据
const commitTx = api.tx.evidence.commit(
  1,                                   // domain: OTC
  orderId,                             // target_id
  ['QmImage1', 'QmImage2'],            // imgs
  [],                                  // vids
  [],                                  // docs
  null                                 // memo
);

await commitTx.signAndSend(owner, ({ status, events }) => {
  if (status.isInBlock) {
    console.log(`Transaction included in block ${status.asInBlock}`);

    // 查找 EvidenceCommitted 事件
    events.forEach(({ event }) => {
      if (api.events.evidence.EvidenceCommitted.is(event)) {
        const [id, domain, targetId, ownerAccount] = event.data;
        console.log(`Evidence committed: ID=${id.toNumber()}`);
      }
    });
  }
});
```

#### 示例 2：提交承诺哈希

```typescript
import { blake2AsHex } from '@polkadot/util-crypto';

// 计算承诺哈希
const ns = new Uint8Array([111, 116, 99, 95, 111, 114, 100, 95]); // "otc_ord_"
const subjectId = 12345;
const cidEnc = new TextEncoder().encode('enc-QmEncryptedContent');
const salt = new TextEncoder().encode('random_salt_12345678');
const ver = 1;

const preimage = new Uint8Array([
  ...ns,
  ...new Uint8Array(new BigUint64Array([BigInt(subjectId)]).buffer),
  ...cidEnc,
  ...salt,
  ...new Uint8Array(new Uint32Array([ver]).buffer),
]);

const commit = blake2AsHex(preimage, 256);

// 提交承诺哈希
const commitHashTx = api.tx.evidence.commitHash(
  ns,
  subjectId,
  commit,
  null
);

await commitHashTx.signAndSend(submitter);
```

#### 示例 3：查询证据

```typescript
// 查询单个证据
const evidence = await api.query.evidence.evidences(evidenceId);
if (evidence.isSome) {
  const ev = evidence.unwrap();
  console.log('Owner:', ev.owner.toString());
  console.log('Content CID:', ev.contentCid.toUtf8());
  console.log('Content Type:', ev.contentType.toString());
  console.log('Is Encrypted:', ev.isEncrypted.toHuman());
}

// 查询目标的所有证据
const evidenceEntries = await api.query.evidence.evidenceByTarget.entries([1, orderId]);
const evidenceIds = evidenceEntries.map(([key, _]) => key.args[1].toNumber());

// 查询证据数量
const count = await api.query.evidence.evidenceCountByTarget([1, orderId]);
```

#### 示例 4：注册公钥并存储私密内容

```typescript
// 注册公钥
const registerKeyTx = api.tx.evidence.registerPublicKey(
  Array.from(publicKeyDer),
  1  // key_type: RSA-2048
);
await registerKeyTx.signAndSend(userAccount);

// 存储私密内容（CID 必须带加密前缀）
const storePrivateTx = api.tx.evidence.storePrivateContent(
  [112, 114, 105, 118, 95, 111, 116, 99], // ns: "priv_otc"
  orderId,
  'enc-QmEncryptedContent',  // 加密前缀
  contentHash,
  1,  // encryption_method: AES256-GCM
  { SharedWith: [userAccount.address] },
  [[userAccount.address, encryptedKeyBytes]]
);
await storePrivateTx.signAndSend(creatorAccount);
```

---

## 🎯 Plain 模式 vs Commit 模式

### 对比表

| 维度 | Plain 模式 | Commit 模式 |
|-----|----------|------------|
| **链上存储** | content_cid（可查询） | commit_hash（不可逆） |
| **隐私保护** | 低（内容可查） | 高（仅承诺哈希） |
| **CID 去重** | 支持（CidHashIndex） | 不支持 |
| **IPFS Pin** | 自动 Pin | 不 Pin（无 CID） |
| **防重机制** | CidHashIndex | CommitIndex |
| **查询索引** | EvidenceByTarget | EvidenceByNs |
| **配额参数** | MaxPerSubjectTarget | MaxPerSubjectNs |
| **适用场景** | 公开证据 | 隐私证据 |
| **典型用途** | 订单证据、公开记录 | KYC、OTC、医疗记录 |

---

## 🔐 私密内容加密机制

### 端到端加密流程

#### 1. 用户注册公钥

```
用户 → 生成非对称密钥对（RSA-2048/Ed25519/ECDSA）
    → 提交公钥到链上（register_public_key）
    → 链上存储：UserPublicKeys<AccountId, UserPublicKey>
```

#### 2. 创建者存储私密内容

```
创建者 → 生成随机 AES 密钥（256-bit）
       → 使用 AES 加密原始内容
       → 上传加密内容到 IPFS → 获得 CID（必须带加密前缀）
       → 为每个授权用户用其公钥加密 AES 密钥
       → 提交到链上（store_private_content）
```

#### 3. 用户访问私密内容

```
用户 → 查询链上加密密钥包（get_encrypted_key_for_user）
    → 使用自己的私钥解密 AES 密钥
    → 从 IPFS 下载加密内容（通过 CID）
    → 使用 AES 密钥解密内容
    → 验证内容哈希
```

### 访问控制策略

| 策略 | 说明 | 适用场景 |
|-----|------|---------|
| `OwnerOnly` | 仅创建者可访问 | 个人私密日记、遗嘱草稿 |
| `SharedWith` | 指定用户列表 | 与特定用户分享的照片、家庭文档 |
| `TimeboxedAccess` | 限时访问 | 临时分享、限时查看权限 |
| `GovernanceControlled` | 治理控制 | 仲裁证据、法律文档 |
| `RoleBased` | 基于角色 | 企业文档、组织内部资料 |

---

## 🔗 集成说明

### 与 pallet-stardust-ipfs 集成

**自动 Pin 机制**：

```rust
// 证据提交时自动 Pin（使用 pin_cid_for_subject）
let cid_vec: Vec<u8> = ev.content_cid.clone().into_inner();
if let Err(e) = T::IpfsPinner::pin_cid_for_subject(
    who.clone(),
    pallet_stardust_ipfs::SubjectType::Evidence,
    id,  // 使用 evidence_id
    cid_vec,
    None,  // 使用默认层级
) {
    log::warn!(
        target: "evidence",
        "Auto-pin content cid failed for evidence {:?}: {:?}",
        id,
        e
    );
}
```

### 与 media_utils 集成

**CID 验证**：

```rust
// 使用 media_utils::IpfsHelper 进行 CID 格式验证
let cid_str = core::str::from_utf8(cid.as_slice())
    .map_err(|_| Error::<T>::InvalidCidFormat)?;
IpfsHelper::validate_cid(cid_str)
    .map_err(|_| Error::<T>::InvalidCidFormat)?;
```

**承诺哈希计算**：

```rust
// 使用 media_utils::HashHelper 计算承诺哈希
let commit = HashHelper::evidence_commitment(ns, subject_id, cid, salt, version);
```

### 与 cid_validator 模块集成

**私密内容 CID 加密验证**（L-4 修复）：

```rust
// 私密内容必须使用加密 CID
ensure!(
    crate::cid_validator::DefaultCidValidator::is_encrypted(cid_bytes),
    Error::<T>::InvalidCidFormat
);
```

---

## 📌 最佳实践

### 1. 选择合适的模式

**Plain 模式**：
- ✅ 公开透明场景
- ✅ 需要内容可查询
- ✅ 支持 IPFS 自动 Pin
- ❌ 隐私保护需求高

**Commit 模式**：
- ✅ 隐私保护场景
- ✅ 防止内容泄露
- ✅ 链下验证需求
- ❌ 需要链上查询内容

### 2. CID 格式规范

**格式要求**：
- 非空
- 符合 IPFS CID 规范（使用 `media_utils::IpfsHelper` 验证）
- 无重复（同次提交）

**推荐格式**：
```
QmXxx...  (IPFS CIDv0)
bafxxx... (IPFS CIDv1)
bagxxx... (IPFS CIDv1 base32)
```

**加密 CID 前缀**（私密内容必须）：
```
enc-QmXxx...       (通用加密前缀)
sealed-bafxxx...   (密封加密)
priv-bagxxx...     (私有加密)
encrypted-cidxxx   (完整单词前缀)
```

### 3. 限频策略建议

**账户级限频**：
- 普通用户：600 块（≈1 小时）最多 10 次
- VIP 用户：600 块最多 100 次
- 管理员：不限制（或极高限额）

**目标级配额**：
- 普通目标：最多 100 条证据
- 高级目标：最多 1000 条证据
- 特殊目标：最多 10000 条证据

### 4. 私密内容安全建议

**密钥管理**：
- ✅ 使用强随机数生成器生成 AES 密钥
- ✅ 定期轮换密钥（每 3-6 个月）
- ✅ 私钥离线存储，避免泄露
- ❌ 不要在链上存储未加密的密钥

**访问控制**：
- ✅ 遵循最小权限原则
- ✅ 定期审查授权用户列表
- ✅ 使用限时访问（临时分享）
- ❌ 避免过度授权

### 5. 错误处理

**常见错误及解决方案**：

| 错误 | 原因 | 解决方案 |
|-----|------|---------|
| `NotAuthorized` | 权限不足 | 检查 EvidenceAuthorizer 配置 |
| `RateLimited` | 限频超限 | 等待窗口重置或升级账户权限 |
| `TooManyForSubject` | 配额超限 | 清理旧证据或扩大配额 |
| `DuplicateCid` | CID 重复 | 检查提交的 CID 列表 |
| `DuplicateCidGlobal` | 全局 CID 重复 | 关闭全局去重或使用新 CID |
| `InvalidCidFormat` | CID 格式错误 | 检查 CID 格式（IPFS 规范） |
| `CommitAlreadyExists` | 承诺哈希重复 | 修改 salt 或 ver 重新计算 |
| `PublicKeyNotRegistered` | 用户未注册公钥 | 先调用 register_public_key |
| `AccessDenied` | 无权访问 | 联系创建者授予权限 |

---

## 🧪 测试建议

### 单元测试

```rust
#[test]
fn test_commit_evidence() {
    new_test_ext().execute_with(|| {
        // 准备测试数据
        let owner = 1;
        let domain = 2;
        let target_id = 100;
        let imgs = vec![
            BoundedVec::try_from(b"QmImage1".to_vec()).unwrap(),
        ];

        // 提交证据
        assert_ok!(Evidence::commit(
            RuntimeOrigin::signed(owner),
            domain,
            target_id,
            imgs,
            vec![],
            vec![],
            None,
        ));

        // 验证事件
        System::assert_has_event(
            Event::Evidence(crate::Event::EvidenceCommitted {
                id: 0,
                domain,
                target_id,
                owner,
            })
        );

        // 验证存储
        assert!(Evidence::evidences(0).is_some());
    });
}

#[test]
fn test_commit_hash() {
    new_test_ext().execute_with(|| {
        let submitter = 1;
        let ns = *b"test_ns_";
        let subject_id = 100;
        let commit = H256::from([1u8; 32]);

        assert_ok!(Evidence::commit_hash(
            RuntimeOrigin::signed(submitter),
            ns,
            subject_id,
            commit,
            None,
        ));

        // 验证承诺索引
        assert_eq!(CommitIndex::<Test>::get(commit), Some(0));
    });
}

#[test]
fn test_private_content_access() {
    new_test_ext().execute_with(|| {
        // 注册公钥
        let user = 1;
        let key_data = vec![0u8; 32]; // Ed25519 公钥
        assert_ok!(Evidence::register_public_key(
            RuntimeOrigin::signed(user),
            BoundedVec::try_from(key_data).unwrap(),
            2, // Ed25519
        ));

        // 存储私密内容
        // ...

        // 验证访问权限
        assert!(Evidence::can_access_private_content(0, &user));
    });
}
```

### 集成测试

```typescript
describe('Evidence Pallet', () => {
  it('should commit evidence and auto-pin to IPFS', async () => {
    // 提交证据
    const tx = api.tx.evidence.commit(1, orderId, ['QmImage1'], [], [], null);
    await tx.signAndSend(owner);

    // 验证证据已创建
    const evidence = await api.query.evidence.evidences(0);
    expect(evidence.isSome).toBe(true);
  });

  it('should archive old evidences', async () => {
    // 等待归档条件满足（90天）
    // 验证归档统计
    const stats = await api.query.evidence.archiveStats();
    expect(stats.totalArchived.toNumber()).toBeGreaterThan(0);
  });
});
```

---

## 🚀 未来扩展

### Phase 2 完整实施计划

**目标**：完全实现 Phase 1.5 CID 化设计

**待完成**：
1. ✅ 定义 Evidence 结构（content_cid, content_type, is_encrypted, encryption_scheme）
2. ⏳ 实现 IPFS JSON 打包功能
3. ⏳ 实现 IPFS JSON 解析功能
4. ⏳ 更新自动 Pin 逻辑（Pin content_cid 及其包含的所有媒体 CID）
5. ⏳ 前端 UI 适配

### 潜在改进方向

1. **zkSNARK 零知识证明**：证明拥有证据但不公开内容
2. **多签授权**：多个管理员共同管理私密内容
3. **链上治理集成**：通过投票决定访问权限
4. **跨链证据验证**：支持跨链证据互认
5. **AI 内容审核**：自动检测违规内容

---

## 📚 相关文档

- [Polkadot SDK 文档](https://docs.substrate.io/)
- [IPFS 文档](https://docs.ipfs.tech/)
- [pallet-stardust-ipfs README](../stardust-ipfs/README.md)
- [Stardust 项目总览](../../README.md)

---

## 🤝 贡献指南

欢迎贡献代码、报告问题或提出改进建议。

**贡献流程**：
1. Fork 本仓库
2. 创建特性分支（`git checkout -b feature/your-feature`）
3. 提交更改（`git commit -m "Add your feature"`）
4. 推送到分支（`git push origin feature/your-feature`）
5. 创建 Pull Request

**代码规范**：
- 所有源代码修改需要**详细的中文函数级注释**
- 更新对应的 README.md 文件
- 添加单元测试和集成测试
- 确保 `cargo test` 和 `cargo clippy` 通过

---

## 📄 许可证

Unlicense

---

**最后更新**：2025-01-15
**版本**：v0.2.0
**维护者**：Stardust Team
