# Pallet Evidence（统一证据管理系统）

## 📋 模块概述

`pallet-evidence` 是 Stardust 区块链的**统一证据管理系统**，提供链上证据提交、IPFS 内容固定、私密内容加密、访问控制、密钥轮换、CID 去重、限频控制等完整的证据管理功能。支持 Plain（明文）和 Commit（承诺哈希）两种模式，满足不同业务场景的隐私保护需求。

### 设计理念

- **CID 化设计（Phase 1.5）**：链上仅存储单一 `content_cid` 引用，实际内容存 IPFS，降低 74.5% 存储成本
- **双模式支持**：Plain 模式适用于公开证据，Commit 模式适用于隐私保护场景（KYC、OTC 等）
- **低耦合架构**：通过 trait 适配器（`EvidenceAuthorizer`、`FamilyVerifier`）实现模块间解耦
- **自动化集成**：与 `pallet-stardust-ipfs` 集成，自动 pin 证据 CID 到 IPFS

### 核心特性

- ✅ **Phase 1.5 CID 化设计**：链上只存储单一 content_cid，实际内容存 IPFS，降低 74.5% 存储成本
- ✅ **双模式支持**：Plain 模式（公开证据）+ Commit 模式（承诺哈希）
- ✅ **私密内容管理**：端到端加密、访问控制、密钥轮换、CID 去重
- ✅ **IPFS 自动 Pin**：证据 CID 自动固定到 IPFS，确保内容持久化
- ✅ **家庭关系验证**：基于 FamilyVerifier 的访问控制
- ✅ **限频控制**：账户级 + 目标级双重限频，防止滥用
- ✅ **CID 加密验证**：L-4 修复，除特殊场景外强制 CID 加密
- ✅ **命名空间隔离**：支持多域证据管理（墓地、逝者、OTC、KYC 等）

---

## 🔑 核心功能

### 1. Plain 模式：公开证据提交

#### `commit`（提交证据）

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
    "images": ["QmXxx1", "QmXxx2", ...],
    "videos": ["QmYyy1", ...],
    "documents": ["QmZzz1", ...],
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
4. 验证 CID 格式、去重
5. 可选全局 CID 去重（EnableGlobalCidDedup）
6. 生成 EvidenceId
7. 打包内容到 IPFS，获取 content_cid
8. 创建证据记录，存储到链上
9. 自动 Pin content_cid 到 IPFS
10. 触发 `EvidenceCommitted` 事件

**函数签名**：

```rust
pub fn commit(
    origin: OriginFor<T>,
    domain: u8,                                    // 域代码（1=Grave, 2=Deceased, ...）
    target_id: u64,                                // 目标 ID（如 deceased_id）
    imgs: Vec<BoundedVec<u8, T::MaxCidLen>>,       // 图片 CID 列表
    vids: Vec<BoundedVec<u8, T::MaxCidLen>>,       // 视频 CID 列表
    docs: Vec<BoundedVec<u8, T::MaxCidLen>>,       // 文档 CID 列表
    memo: Option<BoundedVec<u8, T::MaxMemoLen>>,   // 可选文字说明
) -> DispatchResult
```

**权重计算**：

```rust
#[pallet::weight(T::WeightInfo::commit(imgs.len() as u32, vids.len() as u32, docs.len() as u32))]
```

---

### 2. Commit 模式：承诺哈希提交

#### `commit_hash`（仅登记承诺哈希）

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

#### `link`（链接证据到目标）

**调用方**：授权账户

**功能**：为目标链接已存在的证据（允许复用）。

**使用场景**：
- 多个墓地共享同一证据
- 跨域证据复用
- 证据关联管理

**函数签名**：

```rust
pub fn link(
    origin: OriginFor<T>,
    domain: u8,        // 域代码
    target_id: u64,    // 目标 ID
    id: u64,           // 证据 ID
) -> DispatchResult
```

#### `unlink`（取消链接）

**调用方**：授权账户

**功能**：取消目标与证据的链接。

**函数签名**：

```rust
pub fn unlink(
    origin: OriginFor<T>,
    domain: u8,        // 域代码
    target_id: u64,    // 目标 ID
    id: u64,           // 证据 ID
) -> DispatchResult
```

#### `link_by_ns` / `unlink_by_ns`（按命名空间链接/取消链接）

**功能**：V2 版本，按命名空间与主体链接/取消链接。

**函数签名**：

```rust
pub fn link_by_ns(
    origin: OriginFor<T>,
    ns: [u8; 8],       // 命名空间
    subject_id: u64,   // 主体 ID
    id: u64,           // 证据 ID
) -> DispatchResult

pub fn unlink_by_ns(
    origin: OriginFor<T>,
    ns: [u8; 8],       // 命名空间
    subject_id: u64,   // 主体 ID
    id: u64,           // 证据 ID
) -> DispatchResult
```

---

### 4. 私密内容管理

#### `register_public_key`（注册用户公钥）

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
pub fn register_public_key(
    origin: OriginFor<T>,
    key_data: BoundedVec<u8, T::MaxKeyLen>,  // 公钥数据
    key_type: u8,                            // 密钥类型（1-3）
) -> DispatchResult
```

#### `store_private_content`（存储私密内容）

**调用方**：授权账户

**功能**：存储私密内容（端到端加密）。

**处理流程**：

1. 验证权限（EvidenceAuthorizer）
2. CID 去重检查（PrivateContentByCid）
3. 验证创建者有加密密钥
4. 验证所有授权用户已注册公钥
5. 家庭成员访问策略验证（FamilyVerifier）
6. 生成 content_id
7. 创建私密内容记录
8. 存储到链上
9. 触发 `PrivateContentStored` 事件

**访问策略类型**：

```rust
pub enum AccessPolicy<T: Config> {
    /// 仅创建者可访问
    OwnerOnly,

    /// 指定用户列表
    SharedWith(AuthorizedUsers<T>),

    /// 家庭成员（关联逝者ID）
    FamilyMembers(u64),

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

#### `grant_access`（授予访问权限）

**调用方**：创建者

**功能**：授予用户访问私密内容的权限。

**函数签名**：

```rust
pub fn grant_access(
    origin: OriginFor<T>,
    content_id: u64,                                // 内容 ID
    user: T::AccountId,                             // 被授权用户
    encrypted_key: BoundedVec<u8, ConstU32<512>>,   // 加密密钥包
) -> DispatchResult
```

#### `revoke_access`（撤销访问权限）

**调用方**：创建者

**功能**：撤销用户访问权限。

**注意**：不能撤销自己的权限。

**函数签名**：

```rust
pub fn revoke_access(
    origin: OriginFor<T>,
    content_id: u64,       // 内容 ID
    user: T::AccountId,    // 被撤销用户
) -> DispatchResult
```

#### `rotate_content_keys`（轮换内容加密密钥）

**调用方**：创建者

**功能**：轮换内容加密密钥（重新加密内容）。

**使用场景**：
- 用户公钥泄露时重新加密
- 定期安全维护
- 调整授权用户列表

**函数签名**：

```rust
pub fn rotate_content_keys(
    origin: OriginFor<T>,
    content_id: u64,                                                              // 内容 ID
    new_content_hash: H256,                                                       // 重新加密后的内容哈希
    new_encrypted_keys: BoundedVec<(T::AccountId, BoundedVec<u8, ConstU32<512>>), T::MaxAuthorizedUsers>,  // 新的加密密钥包
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

**机制**：每个目标（如墓地、逝者）最多允许的证据数量

**参数**：
- `MaxPerSubjectTarget`: 每个目标最多证据数（Plain 模式）
- `MaxPerSubjectNs`: 每个命名空间主体最多证据数（Commit 模式）

**用途**：防止单个目标被刷证据

---

### 6. CID 去重机制

#### 局部去重（必须）

**范围**：单次提交的 imgs/vids/docs 内部

**规则**：不允许重复 CID

**实现**：

```rust
fn validate_cid_vec(list: &Vec<BoundedVec<u8, T::MaxCidLen>>) -> Result<(), Error<T>> {
    let mut set: BTreeSet<Vec<u8>> = BTreeSet::new();
    for cid in list.iter() {
        // 检查 CID 格式
        if cid.is_empty() {
            return Err(Error::<T>::InvalidCidFormat);
        }
        // 检查可见 ASCII（0x21..=0x7E）
        for b in cid.iter() {
            if *b < 0x21 || *b > 0x7E {
                return Err(Error::<T>::InvalidCidFormat);
            }
        }
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

**用途**：
- Plain 模式：防止重复上传相同证据
- 节省 IPFS 存储空间

**实现**：

```rust
fn ensure_global_cid_unique(list_groups: [&Vec<BoundedVec<u8, T::MaxCidLen>>; 3]) -> Result<(), Error<T>> {
    if !T::EnableGlobalCidDedup::get() {
        return Ok(());
    }
    for list in list_groups.into_iter() {
        for cid in list.iter() {
            let h = H256::from(blake2_256(&cid.clone().into_inner()));
            if CidHashIndex::<T>::get(h).is_some() {
                return Err(Error::<T>::DuplicateCidGlobal);
            }
        }
    }
    Ok(())
}
```

---

## 📊 数据结构

### Evidence（证据记录）

```rust
pub struct Evidence<AccountId, BlockNumber, MaxContentCidLen, MaxSchemeLen> {
    /// 证据唯一 ID
    pub id: u64,

    /// 所属域（0=Default, 1=Grave, 2=Deceased, ...）
    pub domain: u8,

    /// 目标 ID（如 deceased_id）
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
    pub encrypted_keys: BoundedVec<
        (T::AccountId, BoundedVec<u8, T::MaxKeyLen>),
        T::MaxAuthorizedUsers
    >,

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

    /// 家庭关系验证失败
    FamilyVerificationFailed,

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
    type FamilyVerifier = FamilyVerifierAdapter;

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

## 💻 使用示例

### Rust 代码示例

#### 示例 1：提交公开证据（Plain 模式）

```rust
use frame_support::dispatch::DispatchResult;
use sp_runtime::traits::StaticLookup;

// 准备图片 CID
let img_cids = vec![
    BoundedVec::try_from(b"QmImage1".to_vec()).unwrap(),
    BoundedVec::try_from(b"QmImage2".to_vec()).unwrap(),
];

// 提交证据
let result = Evidence::commit(
    RuntimeOrigin::signed(owner_account),
    2,                  // domain: Deceased
    deceased_id,        // target_id
    img_cids,           // imgs
    vec![],             // vids (空)
    vec![],             // docs (空)
    None,               // memo (无)
)?;

// 监听事件
System::assert_has_event(
    Event::Evidence(pallet_evidence::Event::EvidenceCommitted {
        id: evidence_id,
        domain: 2,
        target_id: deceased_id,
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

let mut preimage = Vec::new();
preimage.extend_from_slice(&ns);
preimage.extend_from_slice(&subject_id.to_le_bytes());
preimage.extend_from_slice(cid_enc);
preimage.extend_from_slice(salt);
preimage.extend_from_slice(&ver.to_le_bytes());

let commit = H256::from(blake2_256(&preimage));

// 提交承诺哈希
let result = Evidence::commit_hash(
    RuntimeOrigin::signed(submitter),
    ns,
    subject_id,
    commit,
    None,  // memo (无)
)?;

// 监听事件
System::assert_has_event(
    Event::Evidence(pallet_evidence::Event::EvidenceCommittedV2 {
        id: evidence_id,
        ns,
        subject_id,
        owner: submitter,
    })
);
```

#### 示例 3：注册公钥并存储私密内容

```rust
use sp_core::crypto::Ss58Codec;

// 步骤 1: 注册用户公钥
let rsa_public_key = /* RSA-2048 公钥 DER 格式 */;
let key_data = BoundedVec::try_from(rsa_public_key).unwrap();

Evidence::register_public_key(
    RuntimeOrigin::signed(user_account),
    key_data,
    1,  // key_type: RSA-2048
)?;

// 步骤 2: 准备加密内容
let encrypted_content_cid = BoundedVec::try_from(b"enc-QmEncryptedContent".to_vec()).unwrap();
let content_hash = H256::from(blake2_256(b"original_content"));

// 步骤 3: 准备访问策略（家庭成员）
let access_policy = AccessPolicy::FamilyMembers(deceased_id);

// 步骤 4: 准备加密密钥包
let encrypted_key = /* 使用用户公钥加密的 AES 密钥 */;
let encrypted_keys = BoundedVec::try_from(vec![
    (user_account.clone(), BoundedVec::try_from(encrypted_key).unwrap()),
]).unwrap();

// 步骤 5: 存储私密内容
Evidence::store_private_content(
    RuntimeOrigin::signed(creator_account),
    *b"priv_med",      // ns: 私密医疗记录
    deceased_id,        // subject_id
    encrypted_content_cid,
    content_hash,
    1,                  // encryption_method: AES256-GCM
    access_policy,
    encrypted_keys,
)?;

// 监听事件
System::assert_has_event(
    Event::Evidence(pallet_evidence::Event::PrivateContentStored {
        content_id,
        ns: *b"priv_med",
        subject_id: deceased_id,
        cid: encrypted_content_cid,
        creator: creator_account,
    })
);
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

// 监听事件
System::assert_has_event(
    Event::Evidence(pallet_evidence::Event::KeysRotated {
        content_id,
        rotation_round: 1,
        rotated_by: creator_account,
    })
);
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
    2,              // domain: Deceased
    deceased_id,    // target_id
    0,              // start_id
    100,            // limit
);
println!("Evidence IDs: {:?}", evidence_ids);

// 查询证据数量
let count = Evidence::count_by_target(2, deceased_id);
println!("Evidence count: {}", count);

// 查询私密内容
let private_content = Evidence::private_contents(content_id).unwrap();
println!("Creator: {:?}", private_content.creator);
println!("Access Policy: {:?}", private_content.access_policy);

// 检查访问权限
let can_access = Evidence::can_access_private_content(content_id, &user_account);
println!("Can access: {}", can_access);

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
  2,                                   // domain: Deceased
  deceasedId,                          // target_id
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
        console.log(`Evidence committed: ID=${id.toNumber()}, Domain=${domain}, Target=${targetId}`);
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

await commitHashTx.signAndSend(submitter, ({ status }) => {
  if (status.isInBlock) {
    console.log(`Commit hash transaction in block`);
  }
});
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
  console.log('Encryption Scheme:', ev.encryptionScheme.toHuman());
}

// 查询目标的所有证据
const evidenceEntries = await api.query.evidence.evidenceByTarget.entries([2, deceasedId]);
const evidenceIds = evidenceEntries.map(([key, _]) => key.args[1].toNumber());
console.log('Evidence IDs:', evidenceIds);

// 查询证据数量
const count = await api.query.evidence.evidenceCountByTarget([2, deceasedId]);
console.log('Evidence count:', count.toNumber());
```

#### 示例 4：注册公钥并存储私密内容

```typescript
import { generateKeyPair } from 'crypto';
import { promisify } from 'util';

// 生成 RSA-2048 密钥对
const generateKeyPairAsync = promisify(generateKeyPair);
const { publicKey } = await generateKeyPairAsync('rsa', {
  modulusLength: 2048,
  publicKeyEncoding: { type: 'spki', format: 'der' },
});

// 注册公钥
const registerKeyTx = api.tx.evidence.registerPublicKey(
  Array.from(publicKey),
  1  // key_type: RSA-2048
);
await registerKeyTx.signAndSend(userAccount);

// 存储私密内容
const storePrivateTx = api.tx.evidence.storePrivateContent(
  [112, 114, 105, 118, 95, 109, 101, 100], // ns: "priv_med"
  deceasedId,
  'enc-QmEncryptedContent',
  contentHash,
  1,  // encryption_method: AES256-GCM
  { FamilyMembers: deceasedId },  // access_policy
  [
    [userAccount.address, encryptedKeyBytes]
  ]
);
await storePrivateTx.signAndSend(creatorAccount);
```

#### 示例 5：授予和撤销访问权限

```typescript
// 授予访问权限
const grantAccessTx = api.tx.evidence.grantAccess(
  contentId,
  newUserAccount.address,
  encryptedKeyForNewUser
);
await grantAccessTx.signAndSend(creatorAccount);

// 撤销访问权限
const revokeAccessTx = api.tx.evidence.revokeAccess(
  contentId,
  oldUserAccount.address
);
await revokeAccessTx.signAndSend(creatorAccount);
```

---

## 🎯 Plain 模式 vs Commit 模式

### Plain 模式（公开证据）

**特点**：
- 证据内容可查询（通过 content_cid）
- 支持全局 CID 去重（可选）
- 自动 Pin 到 IPFS
- 适合公开透明场景

**使用场景**：
- 墓地照片证据
- 逝者档案文档
- 纪念馆供奉记录
- 公开仲裁证据

**调用方法**：`commit(domain, target_id, imgs, vids, docs, memo)`

**存储索引**：`EvidenceByTarget<(domain, target_id), evidence_id>`

---

### Commit 模式（承诺哈希）

**特点**：
- 链上只存储承诺哈希
- 无法通过链上数据反推原始内容
- 防止承诺哈希重复提交
- 适合隐私保护场景

**使用场景**：
- KYC 身份认证证据
- OTC 订单交易证据
- 隐私医疗记录
- 敏感仲裁证据

**调用方法**：`commit_hash(ns, subject_id, commit, memo)`

**存储索引**：
- `EvidenceByNs<(ns, subject_id), evidence_id>`
- `CommitIndex<commit_hash, evidence_id>`（防重）

---

### 对比表

| 维度 | Plain 模式 | Commit 模式 |
|-----|----------|------------|
| **链上存储** | content_cid（可查询） | commit_hash（不可逆） |
| **隐私保护** | 低（内容可查） | 高（仅承诺哈希） |
| **CID 去重** | 支持（可选） | 不适用 |
| **IPFS Pin** | 自动 Pin | 不 Pin（无 CID） |
| **防重机制** | CidHashIndex | CommitIndex |
| **查询索引** | EvidenceByTarget | EvidenceByNs |
| **配额参数** | MaxPerSubjectTarget | MaxPerSubjectNs |
| **适用场景** | 公开证据 | 隐私证据 |
| **典型用途** | 墓地照片、纪念馆记录 | KYC、OTC、医疗记录 |

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
       → 上传加密内容到 IPFS → 获得 CID
       → 为每个授权用户用其公钥加密 AES 密钥
       → 提交到链上（store_private_content）
```

**链上存储**：
- 加密内容 CID
- 内容哈希（用于完整性验证）
- 加密方法标识（1=AES256-GCM）
- 访问策略
- 每个用户的加密密钥包

#### 3. 用户访问私密内容

```
用户 → 查询链上加密密钥包（get_encrypted_key_for_user）
    → 使用自己的私钥解密 AES 密钥
    → 从 IPFS 下载加密内容（通过 CID）
    → 使用 AES 密钥解密内容
    → 验证内容哈希
```

### 访问控制策略

#### OwnerOnly（仅创建者）

```rust
AccessPolicy::OwnerOnly
```

**适用场景**：个人私密日记、遗嘱草稿

---

#### SharedWith（指定用户列表）

```rust
AccessPolicy::SharedWith(vec![user1, user2, user3])
```

**适用场景**：与特定用户分享的照片、家庭文档

---

#### FamilyMembers（家庭成员）

```rust
AccessPolicy::FamilyMembers(deceased_id)
```

**验证逻辑**：
```rust
T::FamilyVerifier::is_family_member(&user, deceased_id)
```

**适用场景**：逝者的医疗记录、家庭照片、遗嘱

---

#### TimeboxedAccess（限时访问）

```rust
AccessPolicy::TimeboxedAccess {
    users: vec![user1, user2],
    expires_at: block_number + 1000,  // 1000 个块后过期
}
```

**适用场景**：临时分享、限时查看权限

---

#### GovernanceControlled（治理控制）

```rust
AccessPolicy::GovernanceControlled
```

**适用场景**：仲裁证据、法律文档（需要治理投票才能访问）

---

#### RoleBased（基于角色）

```rust
AccessPolicy::RoleBased(b"admin".to_vec())
```

**适用场景**：企业文档、组织内部资料

---

### 密钥轮换机制

**触发场景**：
- 用户公钥泄露
- 定期安全维护
- 调整授权用户列表

**轮换流程**：

```
创建者 → 生成新的 AES 密钥
       → 使用新密钥重新加密内容
       → 上传新加密内容到 IPFS → 获得新 CID
       → 为所有用户用新密钥生成新的加密密钥包
       → 调用 rotate_content_keys
       → 链上记录轮换历史（KeyRotationHistory）
```

**轮换历史**：

```rust
KeyRotationRecord {
    content_id: 123,
    rotation_round: 2,  // 第 2 次轮换
    rotated_at: block_number,
    rotated_by: creator_account,
}
```

---

## 🛡️ 访问控制策略

### 权限检查逻辑

```rust
pub fn can_access_private_content(content_id: u64, user: &T::AccountId) -> bool {
    if let Some(content) = PrivateContents::<T>::get(content_id) {
        // 1. 检查是否是创建者
        if &content.creator == user {
            return true;
        }

        // 2. 检查访问策略
        match &content.access_policy {
            AccessPolicy::OwnerOnly => false,

            AccessPolicy::SharedWith(users) => {
                users.iter().any(|u| u == user)
            }

            AccessPolicy::FamilyMembers(deceased_id) => {
                T::FamilyVerifier::is_family_member(user, *deceased_id)
            }

            AccessPolicy::TimeboxedAccess { users, expires_at } => {
                let now = <frame_system::Pallet<T>>::block_number();
                now <= *expires_at && users.iter().any(|u| u == user)
            }

            AccessPolicy::GovernanceControlled => {
                // TODO: 实现治理权限检查
                false
            }

            AccessPolicy::RoleBased(_role) => {
                // TODO: 实现基于角色的权限检查
                false
            }
        }
    } else {
        false
    }
}
```

### 授权管理最佳实践

#### 1. 最小权限原则

只授予必要的用户访问权限，避免过度授权。

```rust
// 好的做法：只授予直系亲属
AccessPolicy::SharedWith(vec![spouse, child1, child2])

// 不好的做法：授予所有联系人
AccessPolicy::SharedWith(all_contacts)  // ❌
```

#### 2. 定期审查权限

定期检查授权用户列表，撤销不必要的权限。

```typescript
// 查询私密内容
const content = await api.query.evidence.privateContents(contentId);

// 检查授权用户
const authorizedUsers = content.unwrap().encryptedKeys.map(([user, _]) => user.toString());
console.log('Authorized users:', authorizedUsers);

// 撤销不再需要的权限
for (const user of usersToRevoke) {
  await api.tx.evidence.revokeAccess(contentId, user).signAndSend(creator);
}
```

#### 3. 使用限时访问

对于临时分享，使用 `TimeboxedAccess` 策略。

```rust
AccessPolicy::TimeboxedAccess {
    users: vec![temp_user],
    expires_at: current_block + 1000,  // 约 100 分钟后过期（6s/块）
}
```

#### 4. 密钥轮换

定期轮换密钥，或在用户公钥泄露时立即轮换。

```typescript
// 每 3 个月轮换一次
const rotationInterval = 30 * 24 * 60 * 10;  // 30 天，每块 6s

if (blocksSinceLastRotation >= rotationInterval) {
  // 重新加密内容
  const newEncryptedContent = await reEncryptContent(content);
  const newContentHash = blake2AsHex(newEncryptedContent);

  // 为所有用户生成新的密钥包
  const newEncryptedKeys = await generateNewKeyBundles(authorizedUsers);

  // 轮换密钥
  await api.tx.evidence.rotateContentKeys(
    contentId,
    newContentHash,
    newEncryptedKeys
  ).signAndSend(creator);
}
```

---

## 🔗 集成说明

### 与 pallet-stardust-ipfs 集成

**自动 Pin 机制**：

```rust
// 证据提交时自动 Pin
let cid_vec: Vec<u8> = ev.content_cid.clone().into_inner();
if let Err(e) = T::IpfsPinner::pin_cid_for_deceased(
    who.clone(),
    deceased_id_u64,
    cid_vec,
    None,  // 使用默认 Standard 层级（3 副本）
) {
    log::warn!(
        target: "evidence",
        "Auto-pin content cid failed for evidence {:?}: {:?}",
        id,
        e
    );
}
```

**配置示例**：

```rust
impl pallet_evidence::Config for Runtime {
    type IpfsPinner = StardustIpfs;
    type Balance = Balance;
    type DefaultStoragePrice = ConstU128<1_000_000_000_000>;  // 1 DUST/副本/月
}
```

---

### 与 pallet-deceased 集成

**家庭关系验证**：

```rust
// FamilyVerifier trait 实现
pub struct FamilyVerifierAdapter;

impl pallet_evidence::FamilyRelationVerifier<AccountId> for FamilyVerifierAdapter {
    fn is_family_member(user: &AccountId, deceased_id: u64) -> bool {
        // 调用 pallet-deceased 的家庭关系检查
        if let Some(deceased) = Deceased::deceased_records(deceased_id) {
            deceased.family_members.contains(user)
        } else {
            false
        }
    }

    fn is_authorized_for_deceased(user: &AccountId, deceased_id: u64) -> bool {
        // 检查是否是创建者或管理员
        if let Some(deceased) = Deceased::deceased_records(deceased_id) {
            &deceased.creator == user || deceased.admins.contains(user)
        } else {
            false
        }
    }
}
```

**配置示例**：

```rust
impl pallet_evidence::Config for Runtime {
    type FamilyVerifier = FamilyVerifierAdapter;
}
```

---

### 与 pallet-arbitration 集成

**仲裁证据提交**：

```rust
// 仲裁案件证据（Commit 模式）
pub fn submit_arbitration_evidence(
    origin: OriginFor<T>,
    case_id: u64,
    evidence_commit: H256,
) -> DispatchResult {
    let submitter = ensure_signed(origin)?;

    // 提交证据承诺哈希
    Evidence::commit_hash(
        origin,
        *b"arb_case",  // ns: 仲裁案件
        case_id,       // subject_id
        evidence_commit,
        None,
    )?;

    // 记录到仲裁案件
    ArbitrationCases::<T>::mutate(case_id, |case| {
        if let Some(c) = case {
            c.evidence_ids.push(evidence_id);
        }
    });

    Ok(())
}
```

---

### 与 pallet-otc-order 集成

**OTC 订单证据（Commit 模式）**：

```rust
// OTC 订单支付证据
pub fn submit_payment_proof(
    origin: OriginFor<T>,
    order_id: u64,
    payment_proof_commit: H256,
) -> DispatchResult {
    let buyer = ensure_signed(origin)?;

    // 提交支付证据承诺哈希
    Evidence::commit_hash(
        origin,
        *b"otc_ord_",  // ns: OTC 订单
        order_id,      // subject_id
        payment_proof_commit,
        None,
    )?;

    // 更新订单状态
    OtcOrders::<T>::mutate(order_id, |order| {
        if let Some(o) = order {
            o.payment_proof_id = Some(evidence_id);
            o.status = OrderStatus::PendingVerification;
        }
    });

    Ok(())
}
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

---

### 2. CID 格式规范

**格式要求**：
- 非空
- 全部为可见 ASCII（0x21..=0x7E）
- 无重复（同次提交）

**推荐格式**：
```
QmXxx...  (IPFS CIDv0)
bafxxx... (IPFS CIDv1)
bagxxx... (IPFS CIDv1 base32)
```

**加密 CID 前缀**（L-4 修复）：
```
enc-QmXxx...       (通用加密前缀)
sealed-bafxxx...   (密封加密)
priv-bagxxx...     (私有加密)
encrypted-cidxxx   (完整单词前缀)
```

---

### 3. 限频策略建议

**账户级限频**：
- 普通用户：600 块（≈1 小时）最多 10 次
- VIP 用户：600 块最多 100 次
- 管理员：不限制（或极高限额）

**目标级配额**：
- 普通墓地：最多 100 条证据
- 高级墓地：最多 1000 条证据
- 纪念馆：最多 10000 条证据

---

### 4. IPFS 存储优化

**Phase 1.5 CID 化设计**：
- ✅ 链上只存储 content_cid（64 字节）
- ✅ 实际内容存 IPFS（JSON 格式）
- ✅ 降低 74.5% 存储成本

**IPFS JSON 结构**：
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
    "memo": "证据说明"
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

---

### 5. 私密内容安全建议

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

**加密方法**：
- ✅ 优先使用 AES-256-GCM（加密+认证）
- ✅ 或使用 XChaCha20-Poly1305（高性能）
- ✅ 验证内容哈希（完整性检查）
- ❌ 不要使用弱加密算法（如 DES、RC4）

---

### 6. 错误处理

**常见错误及解决方案**：

| 错误 | 原因 | 解决方案 |
|-----|------|---------|
| `NotAuthorized` | 权限不足 | 检查 EvidenceAuthorizer 配置 |
| `RateLimited` | 限频超限 | 等待窗口重置或升级账户权限 |
| `TooManyForSubject` | 配额超限 | 清理旧证据或扩大配额 |
| `DuplicateCid` | CID 重复 | 检查提交的 CID 列表 |
| `DuplicateCidGlobal` | 全局 CID 重复 | 关闭全局去重或使用新 CID |
| `InvalidCidFormat` | CID 格式错误 | 检查 CID 格式（非空、可见 ASCII） |
| `CommitAlreadyExists` | 承诺哈希重复 | 修改 salt 或 ver 重新计算 |
| `PublicKeyNotRegistered` | 用户未注册公钥 | 先调用 register_public_key |
| `AccessDenied` | 无权访问 | 联系创建者授予权限 |
| `FamilyVerificationFailed` | 家庭关系验证失败 | 检查 FamilyVerifier 配置 |

---

### 7. 性能优化建议

**查询优化**：
```rust
// ❌ 不好的做法：遍历所有证据
let all_evidences = Evidences::<T>::iter().collect::<Vec<_>>();

// ✅ 好的做法：使用索引查询
let evidence_ids = Evidence::list_ids_by_target(domain, target_id, 0, 100);
```

**批量操作**：
```typescript
// ❌ 不好的做法：逐个提交
for (const cid of cids) {
  await api.tx.evidence.commit(domain, targetId, [cid], [], [], null).signAndSend(owner);
}

// ✅ 好的做法：批量提交
await api.tx.evidence.commit(domain, targetId, cids, [], [], null).signAndSend(owner);
```

**限制查询范围**：
```typescript
// ❌ 不好的做法：查询所有证据
const allEvidences = await api.query.evidence.evidenceByTarget.entries([domain, targetId]);

// ✅ 好的做法：分页查询
const page1 = await api.rpc.evidence.listIdsByTarget(domain, targetId, 0, 100);
const page2 = await api.rpc.evidence.listIdsByTarget(domain, targetId, 100, 100);
```

---

### 8. 测试建议

**单元测试**：
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
```

**集成测试**：
```typescript
describe('Evidence Pallet', () => {
  it('should commit evidence and auto-pin to IPFS', async () => {
    // 提交证据
    const tx = api.tx.evidence.commit(2, deceasedId, ['QmImage1'], [], [], null);
    await tx.signAndSend(owner);

    // 验证证据已创建
    const evidence = await api.query.evidence.evidences(0);
    expect(evidence.isSome).toBe(true);

    // 验证 IPFS 自动 Pin
    const pinStatus = await api.query.stardustIpfs.pinRecords('QmImage1');
    expect(pinStatus.isSome).toBe(true);
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
   - 前端打包：imgs/vids/docs → JSON → IPFS → content_cid
   - 链端接收：content_cid（64 字节）
3. ⏳ 实现 IPFS JSON 解析功能
   - 前端查询：content_cid → IPFS → JSON → 解析 imgs/vids/docs
4. ⏳ 更新自动 Pin 逻辑
   - Pin content_cid 本身
   - 解析 JSON，Pin 所有媒体 CID
5. ⏳ 前端 UI 适配
   - 上传流程：选择文件 → 上传 IPFS → 打包 JSON → 提交 content_cid
   - 查看流程：查询 content_cid → 下载 JSON → 解析并展示

---

### 潜在改进方向

1. **zkSNARK 零知识证明**
   - 证明拥有证据但不公开内容
   - 适用于 KYC、合规检查

2. **多签授权**
   - 多个管理员共同管理私密内容
   - 适用于企业文档、遗产管理

3. **链上治理集成**
   - 通过投票决定访问权限
   - 适用于敏感仲裁证据

4. **跨链证据验证**
   - 支持跨链证据互认
   - 适用于多链生态

5. **AI 内容审核**
   - 自动检测违规内容
   - 保护平台合规性

---

## 📚 相关文档

- [Polkadot SDK 文档](https://docs.substrate.io/)
- [IPFS 文档](https://docs.ipfs.tech/)
- [pallet-stardust-ipfs README](../stardust-ipfs/README.md)
- [pallet-deceased README](../deceased/README.md)
- [pallet-arbitration README](../arbitration/README.md)
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

**最后更新**：2025-11-11
**版本**：v0.1.0
**维护者**：Stardust Team
