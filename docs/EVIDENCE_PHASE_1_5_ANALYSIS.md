# Evidence 多媒体 Phase 1.5 深度分析报告

> 生成日期: 2026-01-22  
> 分析范围: `pallets/evidence/src/lib.rs` 核心逻辑

---

## 📊 现状分析

### 当前临时方案

```
@/home/xiaodong/桌面/stardust/pallets/evidence/src/lib.rs:553-565
```

**问题描述**：

| 行号 | 问题 | 影响 |
|------|------|------|
| 553-565 | 仅使用第一个媒体CID作为content_cid | 丢失其他媒体引用 |
| 573 | 强制使用 `ContentType::Mixed` | 无法区分内容类型 |
| 575 | 假设 `is_encrypted: false` | 无法支持加密证据 |
| 585-592 | 仅对content_cid去重 | 未去重所有媒体CID |
| 595-611 | 仅pin content_cid | 未pin所有媒体CID |

### 临时方案流程

```
用户提交: imgs=[A,B,C], vids=[D], docs=[E,F]
                    ↓
临时处理: content_cid = A (仅第一个图片)
                    ↓
存储: Evidence { content_cid: A, ... }
                    ↓
问题: B,C,D,E,F 丢失，无法追溯
```

---

## 🔮 Phase 1.5 完整设计

### 目标架构

```
用户提交: imgs=[A,B,C], vids=[D], docs=[E,F]
                    ↓
Step 1: 构建 JSON 清单文件
{
  "version": "1.0",
  "evidence_id": 123,
  "content": {
    "images": ["QmA", "QmB", "QmC"],
    "videos": ["QmD"],
    "documents": ["QmE", "QmF"]
  },
  "metadata": {
    "created_at": 1234567890,
    "owner": "5Grw...",
    "total_size": 52428800
  }
}
                    ↓
Step 2: 上传 JSON 到 IPFS → content_cid = QmManifest
                    ↓
Step 3: 存储 Evidence { content_cid: QmManifest, ... }
                    ↓
Step 4: Pin 所有 CID [QmA, QmB, QmC, QmD, QmE, QmF, QmManifest]
                    ↓
Step 5: 去重所有 CID Hash
```

### 核心数据结构

```rust
/// 证据清单结构（IPFS JSON）
#[derive(Serialize, Deserialize)]
pub struct EvidenceManifest {
    /// 版本号
    pub version: String,
    /// 证据ID
    pub evidence_id: u64,
    /// 内容
    pub content: ManifestContent,
    /// 元数据
    pub metadata: ManifestMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct ManifestContent {
    pub images: Vec<String>,
    pub videos: Vec<String>,
    pub documents: Vec<String>,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ManifestMetadata {
    pub created_at: u64,
    pub owner: String,
    pub total_size: Option<u64>,
    pub encryption: Option<EncryptionInfo>,
}
```

---

## 🛠️ 实现方案

### 方案 A: OCW 异步上传（推荐）

**原理**：链上仅存储临时引用，OCW 异步构建 JSON 并上传

```
1. 用户调用 commit_evidence_v2(imgs, vids, docs)
2. 链上生成 pending_id，存储到 PendingManifests
3. OCW 读取 pending，构建 JSON，上传 IPFS
4. OCW 调用 finalize_manifest(pending_id, content_cid)
5. 链上更新 Evidence.content_cid
```

**优点**：
- 链上逻辑简单
- 不阻塞交易
- 可重试

**缺点**：
- 需要 OCW 支持
- 证据创建有延迟

### 方案 B: 前端预构建（简单）

**原理**：前端负责构建 JSON 并上传，仅传递 content_cid

```
1. 前端构建 JSON 清单
2. 前端上传到 IPFS 获得 content_cid
3. 前端调用 commit_evidence(content_cid)
4. 链上直接存储
```

**优点**：
- 链上改动最小
- 无需 OCW

**缺点**：
- 信任前端
- 无法验证 JSON 内容

### 方案 C: 混合验证（平衡）

**原理**：前端构建 JSON，链上验证结构

```
1. 前端构建 JSON 并上传 → content_cid
2. 前端调用 commit_evidence_verified(content_cid, imgs_hash, vids_hash, docs_hash)
3. 链上存储 hash 用于后续验证
4. OCW 异步验证 JSON 内容与 hash 匹配
5. 验证通过后标记为 verified
```

---

## 📋 实现步骤

### Phase 1.5.1: 基础重构

```rust
// 新增存储：待处理清单
#[pallet::storage]
pub type PendingManifests<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64, // evidence_id
    PendingManifest<T>,
>;

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct PendingManifest<T: Config> {
    pub imgs: BoundedVec<BoundedVec<u8, T::MaxCidLen>, T::MaxMediaCount>,
    pub vids: BoundedVec<BoundedVec<u8, T::MaxCidLen>, T::MaxMediaCount>,
    pub docs: BoundedVec<BoundedVec<u8, T::MaxCidLen>, T::MaxMediaCount>,
    pub created_at: BlockNumberFor<T>,
}
```

### Phase 1.5.2: OCW 实现

```rust
// off-chain worker
fn offchain_worker(block_number: BlockNumberFor<Self>) {
    for (evidence_id, pending) in PendingManifests::<T>::iter() {
        // 1. 构建 JSON
        let manifest = build_manifest(&pending);
        
        // 2. 上传 IPFS
        let content_cid = upload_to_ipfs(&manifest)?;
        
        // 3. Pin 所有 CID
        pin_all_cids(&pending, &content_cid)?;
        
        // 4. 提交结果
        submit_manifest_result(evidence_id, content_cid)?;
    }
}
```

### Phase 1.5.3: 验证与清理

```rust
// 清理已完成的 pending
#[pallet::call]
fn finalize_manifest(
    origin: OriginFor<T>,
    evidence_id: u64,
    content_cid: BoundedVec<u8, T::MaxContentCidLen>,
) -> DispatchResult {
    // 验证 OCW 签名
    let who = ensure_signed(origin)?;
    ensure!(Self::is_ocw_signer(&who), Error::<T>::NotOcwSigner);
    
    // 更新 Evidence
    Evidences::<T>::try_mutate(evidence_id, |ev| {
        let ev = ev.as_mut().ok_or(Error::<T>::EvidenceNotFound)?;
        ev.content_cid = content_cid;
        Ok(())
    })?;
    
    // 清理 pending
    PendingManifests::<T>::remove(evidence_id);
    
    Ok(())
}
```

---

## 📊 存储成本对比

| 场景 | 当前方案 | Phase 1.5 |
|------|----------|-----------|
| 10张图片 | ~840 字节 | ~214 字节 |
| 5张图片+2视频+3文档 | ~840 字节 | ~214 字节 |
| 链上存储 | 所有 CID | 仅 1 个 content_cid |
| IPFS 存储 | 无清单 | JSON 清单 + 媒体 |

**存储降低: ~74.5%**

---

## 🚀 推荐实施路径

### 短期（1-2周）

1. **采用方案 B（前端预构建）**
   - 前端实现 JSON 构建和上传
   - 链上仅验证 content_cid 格式
   - 最小改动，快速上线

### 中期（1个月）

2. **增加 OCW 验证**
   - OCW 异步验证 JSON 结构
   - 标记验证状态
   - 增强可信度

### 长期（Phase 2）

3. **完整 OCW 流程**
   - 链上仅存储原始 CID 列表
   - OCW 完全负责 JSON 构建
   - 去中心化验证

---

## ✅ 检查清单

- [ ] 定义 EvidenceManifest JSON schema
- [ ] 前端实现 JSON 构建
- [ ] 前端实现 IPFS 上传
- [ ] 链上添加 content_cid 验证
- [ ] 实现所有 CID 的 pin 逻辑
- [ ] 实现所有 CID 的去重逻辑
- [ ] OCW 验证逻辑（可选）
- [ ] 测试覆盖

---

**报告结束**
