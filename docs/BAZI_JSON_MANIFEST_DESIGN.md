# 八字数据 JSON 清单存储设计

> 文档日期: 2026-01-23  
> 版本: 2.0 (OCW + TEE 结合架构)  
> 状态: 设计方案

---

## 概述

本文档定义八字数据 JSON 清单存储方案，采用 **OCW + TEE 结合架构**：
- **OCW**: 负责请求调度、TEE 调用、IPFS 存储、链上提交
- **TEE**: 负责隐私计算（解密、八字计算、加密、签名）

---

## 架构总览

```
┌─────────────────────────────────────────────────────────────────────────┐
│  OCW + TEE 结合架构                                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐          │
│  │  前端    │───>│  链上    │───>│   OCW    │───>│   TEE    │          │
│  │ (加密)   │    │ (存储)   │    │ (调度)   │    │ (计算)   │          │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘          │
│       │                              │ ▲              │                  │
│       │                              │ │              │                  │
│       │                              ▼ │              │                  │
│       │                         ┌──────────┐         │                  │
│       │                         │  IPFS    │<────────┘                  │
│       │                         │ (存储)   │    加密 JSON               │
│       │                         └──────────┘                            │
│       │                              │                                   │
│       └──────────────────────────────┘                                  │
│              用户解密查看                                                │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 职责划分

| 组件 | 职责 | 接触明文 |
|------|------|----------|
| **前端** | 密钥生成、加密输入、解密结果 | ✅ 是 |
| **链上** | 存储请求、存储结果、事件触发 | ❌ 否 |
| **OCW** | 轮询请求、调用 TEE、上传 IPFS、提交结果 | ❌ 否 |
| **TEE** | 解密输入、八字计算、生成 JSON、加密输出 | ✅ 是（隔离） |
| **IPFS** | 存储加密 JSON | ❌ 否 |

---

## 隐私模式

用户可选择隐私级别：

| 模式 | 链上公开 | IPFS 内容 | 适用场景 |
|------|----------|-----------|----------|
| **Public** | sizhu_index + gender | 明文 JSON | 公开命盘展示 |
| **Encrypted** | sizhu_index + gender | 加密 JSON | 个人隐私命盘 |
| **Private** | 无 | 加密 JSON | 最高隐私需求 |

```rust
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum PrivacyMode {
    /// 公开模式：链上存储 sizhu_index，IPFS 明文
    Public = 0,
    /// 加密模式：链上存储 sizhu_index，IPFS 加密
    Encrypted = 1,
    /// 私密模式：链上不存储 sizhu_index，IPFS 加密
    Private = 2,
}
```

---

## 链上存储结构

```rust
/// 统一的八字链上存储（支持所有隐私模式）
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct BaziOnChain<T: Config> {
    /// 所有者
    pub owner: T::AccountId,
    
    /// 隐私模式
    pub privacy_mode: PrivacyMode,
    
    /// 四柱索引（Private 模式下为 None）
    pub sizhu_index: Option<SiZhuIndex>,
    
    /// 性别（Private 模式下为 None）
    pub gender: Option<Gender>,
    
    /// JSON 清单 CID（IPFS）
    pub manifest_cid: BoundedVec<u8, ConstU32<64>>,
    
    /// 清单哈希（用于验证完整性）
    pub manifest_hash: [u8; 32],
    
    /// 生成方式
    pub generation: GenerationInfo<T>,
    
    /// 版本号（用于更新追溯）
    pub version: u32,
    
    /// 创建区块
    pub created_at: BlockNumberFor<T>,
    
    /// 更新区块
    pub updated_at: BlockNumberFor<T>,
}

/// 生成信息
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum GenerationInfo<T: Config> {
    /// OCW 生成（公开模式）
    Ocw,
    /// TEE 生成（加密/私密模式）
    Tee {
        /// TEE 节点
        node: T::AccountId,
        /// 计算证明
        proof: ComputationProof,
    },
}
```

### 存储大小对比

| 模式 | 链上大小 | 说明 |
|------|----------|------|
| **Public** | ~100 bytes | sizhu_index + gender + cid + hash |
| **Encrypted** | ~150 bytes | 同上 + TEE proof |
| **Private** | ~140 bytes | 无 sizhu_index + TEE proof |

---

## 待处理请求结构

```rust
/// 待处理的八字请求
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct PendingBaziRequest<T: Config> {
    /// 请求者
    pub requester: T::AccountId,
    /// 输入数据（Public 模式为明文，其他为密文）
    pub input_data: InputData,
    /// 用户公钥（用于加密返回结果，Public 模式为 None）
    pub user_pubkey: Option<[u8; 32]>,
    /// 隐私模式
    pub privacy_mode: PrivacyMode,
    /// 分配的 TEE 节点（Public 模式为 None）
    pub assigned_node: Option<T::AccountId>,
    /// 请求状态
    pub status: RequestStatus,
    /// 重试次数
    pub retry_count: u8,
    /// 创建区块
    pub created_at: BlockNumberFor<T>,
}

/// 输入数据（区分明文和密文）
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum InputData {
    /// 明文输入（Public 模式）
    Plaintext(BaziInputPlain),
    /// 加密输入（Encrypted/Private 模式）
    Encrypted(EncryptedData),
}

/// 明文八字输入
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct BaziInputPlain {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub gender: Gender,
    pub longitude: Option<i32>,
}

/// 加密数据
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct EncryptedData {
    pub ciphertext: BoundedVec<u8, ConstU32<256>>,
    pub nonce: [u8; 24],
    pub sender_pubkey: [u8; 32],
}

/// 请求状态
#[derive(Clone, Copy, Debug, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq)]
pub enum RequestStatus {
    /// 待处理
    Pending,
    /// 处理中
    Processing,
    /// 已完成
    Completed,
    /// 失败（可重试）
    Failed,
    /// 超时
    Timeout,
}
```

---

## JSON 清单结构

### 明文模式（Public）

```json
{
  "version": "1.0",
  "schema": "bazi-manifest-v1",
  "created_at": "2026-01-23T08:42:00Z",
  
  "metadata": {
    "name": "张三命盘",
    "privacy_mode": "public",
    "generated_by": "ocw"
  },
  
  "sizhu": {
    "year": {
      "gan": "甲", "gan_index": 0,
      "zhi": "子", "zhi_index": 0,
      "ganzhi": "甲子", "ganzhi_index": 0,
      "nayin": "海中金",
      "canggan": [{ "gan": "癸", "type": "主气", "weight": 100, "shishen": "正印" }]
    },
    "month": { ... },
    "day": { ... },
    "hour": { ... },
    "rizhu": { "gan": "戊", "wuxing": "土" }
  },
  
  "dayun": {
    "qiyun_age": 3,
    "qiyun_year": 1987,
    "is_shun": true,
    "steps": [...]
  },
  
  "analysis": {
    "wuxing_strength": { "jin": 120, "mu": 180, "shui": 90, "huo": 150, "tu": 200, "total": 740 },
    "xiyong_shen": "水",
    "jiyong_shen": "火",
    "body_strength": "身旺"
  }
}
```

### 加密模式（Encrypted / Private）

```json
{
  "version": "1.0",
  "schema": "bazi-manifest-encrypted-v1",
  "algorithm": "AES-256-GCM",
  "nonce": "base64_nonce...",
  "enclave_pubkey": "base64_pubkey...",
  "ciphertext": "base64_encrypted_full_manifest..."
}
```

---

## 完整流程详解

### Phase 1: 用户提交请求（前端）

```typescript
const birthData = {
  year: 1990, month: 5, day: 15, hour: 10,
  gender: 0,  // 0=male, 1=female
  longitude: null,
};

// ========== Public 模式：明文提交 ==========
async function createBaziPublic(birthData: BirthInput) {
  // 无需加密，直接提交明文
  await api.tx.bazi.createBaziPublic(
    birthData.year,
    birthData.month,
    birthData.day,
    birthData.hour,
    birthData.gender,
    birthData.longitude,
  ).signAndSend(account);
}

// ========== Encrypted/Private 模式：加密提交 ==========
async function createBaziPrivate(birthData: BirthInput, privacyMode: PrivacyMode) {
  // 1. 获取 TEE 节点公钥
  const teeNodes = await api.query.teePrivacy.activeNodes.entries();
  const enclavePubkey = teeNodes[0][1].enclave_pubkey;

  // 2. 生成/加载用户密钥对（见"密钥管理"章节）
  const userKeyPair = await getUserKeyPair(account);

  // 3. ECDH 加密出生时间
  const plaintext = JSON.stringify(birthData);
  const nonce = randomBytes(24);
  const encrypted = nacl.box(
    new TextEncoder().encode(plaintext),
    nonce,
    enclavePubkey,
    userKeyPair.secretKey
  );

  // 4. 提交链上请求
  await api.tx.bazi.createBazi(
    { ciphertext: Array.from(encrypted), nonce: Array.from(nonce), sender_pubkey: Array.from(userKeyPair.publicKey) },
    Array.from(userKeyPair.publicKey),  // 用于加密返回结果
    privacyMode,
  ).signAndSend(account);
}

// ========== 统一入口 ==========
async function createBazi(birthData: BirthInput, privacyMode: PrivacyMode) {
  if (privacyMode === PrivacyMode.Public) {
    await createBaziPublic(birthData);
  } else {
    await createBaziPrivate(birthData, privacyMode);
  }
}
```

### Phase 2: 链上处理（Runtime）

```rust
// ========== Public 模式：明文提交 ==========
#[pallet::call_index(0)]
pub fn create_bazi_public(
    origin: OriginFor<T>,
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    gender: Gender,
    longitude: Option<i32>,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    let request_id = Self::next_request_id();
    
    // Public 模式：明文存储，无需 TEE
    PendingRequests::<T>::insert(request_id, PendingBaziRequest {
        requester: who.clone(),
        input_data: InputData::Plaintext(BaziInputPlain { year, month, day, hour, gender, longitude }),
        user_pubkey: None,
        privacy_mode: PrivacyMode::Public,
        assigned_node: None,
        status: RequestStatus::Pending,
        retry_count: 0,
        created_at: Self::current_block(),
    });
    
    Self::deposit_event(Event::BaziRequestSubmitted {
        request_id,
        requester: who,
        privacy_mode: PrivacyMode::Public,
        assigned_node: None,
    });
    
    Ok(())
}

// ========== Encrypted/Private 模式：加密提交 ==========
#[pallet::call_index(1)]
pub fn create_bazi(
    origin: OriginFor<T>,
    encrypted_input: EncryptedData,
    user_pubkey: [u8; 32],
    privacy_mode: PrivacyMode,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // 验证隐私模式
    ensure!(privacy_mode != PrivacyMode::Public, Error::<T>::UsePublicExtrinsic);
    
    let request_id = Self::next_request_id();
    
    // 分配 TEE 节点
    let assigned_node = Self::select_active_tee_node()?;
    
    PendingRequests::<T>::insert(request_id, PendingBaziRequest {
        requester: who.clone(),
        input_data: InputData::Encrypted(encrypted_input),
        user_pubkey: Some(user_pubkey),
        privacy_mode,
        assigned_node: Some(assigned_node.clone()),
        status: RequestStatus::Pending,
        retry_count: 0,
        created_at: Self::current_block(),
    });
    
    Self::deposit_event(Event::BaziRequestSubmitted {
        request_id,
        requester: who,
        privacy_mode,
        assigned_node: Some(assigned_node),
    });
    
    Ok(())
}
```

### Phase 3: OCW 调度 + TEE 计算

```rust
/// 最大重试次数
const MAX_RETRY_COUNT: u8 = 3;

fn offchain_worker(block_number: BlockNumberFor<T>) {
    if block_number % T::OcwInterval::get() != Zero::zero() {
        return;
    }
    
    for (request_id, request) in PendingRequests::<T>::iter() {
        // 跳过已完成或已超过重试次数的请求
        if request.status == RequestStatus::Completed {
            continue;
        }
        
        // 检查超时
        if Self::is_request_timeout(&request, block_number) {
            Self::submit_status_update(request_id, RequestStatus::Timeout);
            continue;
        }
        
        // 检查重试次数
        if request.retry_count >= MAX_RETRY_COUNT {
            log::error!("Request {} exceeded max retries", request_id);
            Self::submit_status_update(request_id, RequestStatus::Failed);
            continue;
        }
        
        // 标记为处理中
        if request.status == RequestStatus::Pending {
            Self::submit_status_update(request_id, RequestStatus::Processing);
        }
        
        // 根据隐私模式处理
        let result = match request.privacy_mode {
            PrivacyMode::Public => {
                // OCW 直接计算（明文）
                Self::process_public_request(&request)
            },
            _ => {
                // 调用 TEE 节点处理（加密）
                Self::process_tee_request(&request)
            },
        };
        
        match result {
            Ok(process_result) => {
                Self::submit_result(request_id, process_result);
            },
            Err(e) => {
                log::warn!("Request {} failed (retry {}): {:?}", 
                    request_id, request.retry_count, e);
                // 增加重试计数，下次 OCW 周期会重试
                Self::submit_retry_increment(request_id);
            }
        }
    }
}

/// 提交状态更新（unsigned transaction）
fn submit_status_update(request_id: u64, status: RequestStatus) {
    let call = Call::update_request_status { request_id, status };
    let _ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into());
}

/// 提交重试计数增加
fn submit_retry_increment(request_id: u64) {
    let call = Call::increment_retry_count { request_id };
    let _ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into());
}

// ========== 新增 Extrinsic ==========

#[pallet::call_index(10)]
#[pallet::weight(Weight::from_parts(10_000_000, 0))]
pub fn update_request_status(
    origin: OriginFor<T>,
    request_id: u64,
    status: RequestStatus,
) -> DispatchResult {
    ensure_none(origin)?;
    
    PendingRequests::<T>::mutate(request_id, |req| {
        if let Some(r) = req {
            r.status = status;
        }
    });
    
    // 如果是超时或失败，触发事件通知用户
    if status == RequestStatus::Timeout || status == RequestStatus::Failed {
        if let Some(request) = PendingRequests::<T>::get(request_id) {
            Self::deposit_event(Event::BaziRequestFailed {
                request_id,
                requester: request.requester,
                reason: status,
            });
        }
    }
    
    Ok(())
}

#[pallet::call_index(11)]
#[pallet::weight(Weight::from_parts(10_000_000, 0))]
pub fn increment_retry_count(
    origin: OriginFor<T>,
    request_id: u64,
) -> DispatchResult {
    ensure_none(origin)?;
    
    PendingRequests::<T>::mutate(request_id, |req| {
        if let Some(r) = req {
            r.retry_count = r.retry_count.saturating_add(1);
            r.status = RequestStatus::Pending;  // 重置为待处理
        }
    });
    
    Ok(())
}

/// OCW 调用 TEE 节点 HTTP API
fn process_tee_request(request: &PendingBaziRequest<T>) -> Result<ProcessResult, &'static str> {
    let tee_node = TeeNodes::<T>::get(&request.assigned_node.as_ref().unwrap())
        .ok_or("TEE node not found")?;
    
    // 提取加密输入
    let encrypted_input = match &request.input_data {
        InputData::Encrypted(data) => data,
        _ => return Err("Expected encrypted input"),
    };
    
    let user_pubkey = request.user_pubkey.as_ref().ok_or("User pubkey required")?;
    
    // 1. HTTP 调用 TEE Enclave（传递 privacy_mode）
    let tee_response = Self::call_tee_http(
        &tee_node.endpoint,
        encrypted_input,
        user_pubkey,
        request.privacy_mode,  // 传递隐私模式
    )?;
    
    // OCW 只能看到加密数据，无法解密
    // tee_response 包含:
    // - encrypted_manifest: 用户公钥加密的 JSON
    // - sizhu_index: 公开的四柱索引（Private 模式下为 None）
    // - computation_proof: TEE 计算证明
    // - enclave_signature: Enclave 签名
    
    // 2. 上传加密 JSON 到 IPFS
    let cid = Self::upload_to_ipfs(&tee_response.encrypted_manifest)?;
    
    // 3. 请求 PIN
    Self::request_pin(&cid)?;
    
    Ok(ProcessResult {
        manifest_cid: cid,
        manifest_hash: tee_response.manifest_hash,
        sizhu_index: tee_response.sizhu_index,
        proof: Some(tee_response.computation_proof),
    })
}

fn call_tee_http(
    endpoint: &str,
    encrypted_input: &EncryptedData,
    user_pubkey: &[u8; 32],
    privacy_mode: PrivacyMode,  // 传递隐私模式给 TEE
) -> Result<TeeComputeResponse, &'static str> {
    let request = http::Request::post(
        &format!("{}/compute/bazi", endpoint),
        serde_json::json!({
            "encrypted_input": {
                "ciphertext": hex::encode(&encrypted_input.ciphertext),
                "nonce": hex::encode(&encrypted_input.nonce),
                "sender_pubkey": hex::encode(&encrypted_input.sender_pubkey),
            },
            "user_pubkey": hex::encode(user_pubkey),
            "privacy_mode": privacy_mode as u8,  // 0=Public, 1=Encrypted, 2=Private
        }).to_string().into_bytes(),
    );
    
    let response = request.send().map_err(|_| "TEE request failed")?;
    let body = response.body().collect::<Vec<u8>>();
    serde_json::from_slice(&body).map_err(|_| "Parse failed")
}
```

### Phase 4: TEE Enclave 内部计算

```rust
// 此代码运行在 TEE Enclave 内部（安全隔离区）

fn compute_bazi_in_enclave(
    encrypted_input: &[u8],
    nonce: &[u8],
    sender_pubkey: &[u8; 32],
    user_pubkey: &[u8; 32],  // 用于加密输出
    privacy_mode: PrivacyMode,  // 隐私模式（决定是否返回 sizhu_index）
) -> EnclaveResult {
    // 1. ECDH 解密输入
    let shared_key = ecdh_derive_key(ENCLAVE_PRIVKEY, sender_pubkey);
    let plaintext = aes_gcm_decrypt(&shared_key, nonce, encrypted_input)?;
    let input: BaziInput = serde_json::from_slice(&plaintext)?;
    
    // 2. 八字计算（所有计算在 Enclave 内存中进行）
    let sizhu = calculate_sizhu(input.year, input.month, input.day, input.hour);
    let dayun = calculate_dayun(&sizhu, input.gender, input.year);
    let wuxing = calculate_wuxing_strength(&sizhu);
    let xiyong = determine_xiyong_shen(&sizhu, &wuxing);
    
    // 3. 生成 JSON 清单
    let manifest = serde_json::json!({
        "version": "1.0",
        "schema": "bazi-manifest-v1",
        "sizhu": sizhu_to_json(&sizhu),
        "dayun": dayun_to_json(&dayun),
        "analysis": { "wuxing_strength": wuxing, "xiyong_shen": xiyong }
    });
    
    // 4. 用用户公钥加密输出
    let output_key = ecdh_derive_key(ENCLAVE_PRIVKEY, user_pubkey);
    let output_nonce = generate_random_nonce();
    let encrypted_manifest = aes_gcm_encrypt(&output_key, &output_nonce, manifest.to_string().as_bytes());
    
    // 5. 计算哈希
    let manifest_hash = sha256(manifest.to_string().as_bytes());
    
    // 6. Enclave 签名
    let signature = enclave_sign(&[
        &sizhu.to_index().encode(),
        &manifest_hash,
    ]);
    
    // 7. 生成计算证明
    let proof = ComputationProof {
        mrenclave: MRENCLAVE,
        timestamp: current_time(),
        signature,
    };
    
    EnclaveResult {
        encrypted_manifest: EncryptedManifest {
            algorithm: "AES-256-GCM",
            nonce: output_nonce,
            enclave_pubkey: ENCLAVE_PUBKEY,
            ciphertext: encrypted_manifest,
        },
        // 根据 privacy_mode 决定是否返回 sizhu_index
        sizhu_index: match privacy_mode {
            PrivacyMode::Private => None,  // Private 模式不返回，保护隐私
            _ => Some(sizhu.to_index()),   // 其他模式返回用于链上计算
        },
        gender: match privacy_mode {
            PrivacyMode::Private => None,
            _ => Some(input.gender),
        },
        manifest_hash,
        computation_proof: proof,
    }
}
```

### Phase 5: 结果提交（OCW → 链上）

```rust
#[pallet::call_index(1)]
pub fn confirm_bazi_result(
    origin: OriginFor<T>,
    request_id: u64,
    manifest_cid: Vec<u8>,
    manifest_hash: [u8; 32],
    sizhu_index: Option<SiZhuIndex>,
    gender: Option<Gender>,
    computation_proof: Option<ComputationProof>,
    enclave_signature: Option<[u8; 64]>,
) -> DispatchResult {
    // OCW unsigned transaction
    ensure_none(origin)?;
    
    // 1. 获取并验证请求
    let request = PendingRequests::<T>::take(request_id)
        .ok_or(Error::<T>::RequestNotFound)?;
    
    // 2. 如果是 TEE 生成，验证签名和证明
    if let Some(proof) = &computation_proof {
        Self::verify_enclave_signature(
            &request.assigned_node.unwrap(),
            &sizhu_index,
            &manifest_hash,
            &enclave_signature.unwrap(),
        )?;
        Self::verify_computation_proof(proof)?;
    }
    
    // 3. 生成命盘 ID
    let chart_id = Self::next_chart_id(&request.requester);
    
    // 4. 存储最终数据
    let cid_bounded: BoundedVec<u8, ConstU32<64>> = manifest_cid
        .try_into()
        .map_err(|_| Error::<T>::CidTooLong)?;
    
    BaziCharts::<T>::insert(&request.requester, chart_id, BaziOnChain {
        owner: request.requester.clone(),
        privacy_mode: request.privacy_mode,
        sizhu_index,
        gender,
        manifest_cid: cid_bounded,
        manifest_hash,
        generation: match computation_proof {
            Some(proof) => GenerationInfo::Tee {
                node: request.assigned_node.unwrap(),
                proof,
            },
            None => GenerationInfo::Ocw,
        },
        version: 1,
        created_at: Self::current_block(),
        updated_at: Self::current_block(),
    });
    
    // 5. 触发完成事件
    Self::deposit_event(Event::BaziCreated {
        request_id,
        owner: request.requester,
        chart_id,
        privacy_mode: request.privacy_mode,
    });
    
    Ok(())
}
```

### Phase 6: 用户解密查看（前端）

```typescript
async function viewBaziChart(chartId: number) {
  // 1. 获取链上数据
  const onChain = await api.query.bazi.baziCharts(account.address, chartId);
  
  // 2. 从 IPFS 获取加密清单
  const response = await fetch(`https://ipfs.io/ipfs/${onChain.manifest_cid}`);
  const encryptedManifest = await response.json();
  
  // 3. 如果是加密模式，解密
  if (onChain.privacy_mode !== 'Public') {
    const decrypted = nacl.box.open(
      new Uint8Array(encryptedManifest.ciphertext),
      new Uint8Array(encryptedManifest.nonce),
      new Uint8Array(encryptedManifest.enclave_pubkey),
      userKeyPair.secretKey  // 用户私钥
    );
    
    if (!decrypted) {
      throw new Error('Decryption failed - wrong key?');
    }
    
    return JSON.parse(Buffer.from(decrypted).toString());
  }
  
  // 公开模式直接返回
  return encryptedManifest;
}
```

---

## 前端密钥管理

用户私钥用于解密 IPFS 上的加密清单，需要安全持久化存储。

### 密钥派生方案（推荐）

```typescript
import { Keyring } from '@polkadot/keyring';
import { mnemonicToMiniSecret } from '@polkadot/util-crypto';
import nacl from 'tweetnacl';

/**
 * 从 Substrate 账户派生 ECDH 密钥对
 * 优点：用户无需额外记忆密钥，与账户绑定
 */
async function deriveKeyPairFromAccount(mnemonic: string, accountIndex: number = 0): Promise<nacl.BoxKeyPair> {
  // 1. 从助记词派生种子
  const seed = mnemonicToMiniSecret(mnemonic);
  
  // 2. 使用派生路径生成八字专用密钥
  const derivationPath = `bazi/encryption/${accountIndex}`;
  const derivedSeed = await crypto.subtle.digest(
    'SHA-256',
    new TextEncoder().encode(`${seed.toString()}:${derivationPath}`)
  );
  
  // 3. 生成 X25519 密钥对
  return nacl.box.keyPair.fromSecretKey(new Uint8Array(derivedSeed));
}

/**
 * 获取或创建用户密钥对
 */
async function getUserKeyPair(account: InjectedAccountWithMeta): Promise<nacl.BoxKeyPair> {
  const storageKey = `bazi_keypair_${account.address}`;
  
  // 1. 尝试从 localStorage 加载
  const cached = localStorage.getItem(storageKey);
  if (cached) {
    const { publicKey, secretKey } = JSON.parse(cached);
    return {
      publicKey: new Uint8Array(publicKey),
      secretKey: new Uint8Array(secretKey),
    };
  }
  
  // 2. 如果用户使用助记词登录，派生密钥
  if (account.meta.source === 'mnemonic') {
    const mnemonic = await promptUserForMnemonic();
    const keyPair = await deriveKeyPairFromAccount(mnemonic);
    
    // 缓存到 localStorage（可选：加密存储）
    localStorage.setItem(storageKey, JSON.stringify({
      publicKey: Array.from(keyPair.publicKey),
      secretKey: Array.from(keyPair.secretKey),
    }));
    
    return keyPair;
  }
  
  // 3. 硬件钱包用户：生成独立密钥对
  const keyPair = nacl.box.keyPair();
  
  // ⚠️ 警告用户备份密钥
  await showBackupKeyDialog(keyPair);
  
  localStorage.setItem(storageKey, JSON.stringify({
    publicKey: Array.from(keyPair.publicKey),
    secretKey: Array.from(keyPair.secretKey),
  }));
  
  return keyPair;
}
```

### 密钥恢复机制

```typescript
/**
 * 密钥恢复选项
 */
enum KeyRecoveryMethod {
  // 从助记词重新派生
  FromMnemonic = 'mnemonic',
  // 从备份文件导入
  FromBackup = 'backup',
  // 重新生成（旧数据将无法解密）
  Regenerate = 'regenerate',
}

async function recoverKeyPair(method: KeyRecoveryMethod): Promise<nacl.BoxKeyPair> {
  switch (method) {
    case KeyRecoveryMethod.FromMnemonic:
      const mnemonic = await promptUserForMnemonic();
      return deriveKeyPairFromAccount(mnemonic);
      
    case KeyRecoveryMethod.FromBackup:
      const file = await promptUserForBackupFile();
      const backup = JSON.parse(await file.text());
      return {
        publicKey: new Uint8Array(backup.publicKey),
        secretKey: new Uint8Array(backup.secretKey),
      };
      
    case KeyRecoveryMethod.Regenerate:
      // ⚠️ 警告：旧的加密数据将无法解密
      if (!await confirmRegenerate()) {
        throw new Error('User cancelled');
      }
      return nacl.box.keyPair();
  }
}
```

### 安全建议

| 场景 | 建议 |
|------|------|
| **助记词用户** | 使用派生密钥，无需额外备份 |
| **硬件钱包用户** | 必须备份密钥文件或助记词 |
| **多设备同步** | 通过备份文件或助记词恢复 |
| **密钥丢失** | 旧数据无法解密，需重新创建命盘 |

---

## 隐私风险分析

### 数据流全链路审计

| 阶段 | 数据状态 | 可见方 | 风险等级 |
|------|----------|--------|----------|
| 前端输入 | 明文 | 用户 | - |
| 前端加密 | 密文 | 用户、链上、OCW | ✅ 安全 |
| 链上存储 | 密文 | 所有人 | ⚠️ P1 |
| OCW 中继 | 密文 | OCW 节点 | ⚠️ P2 |
| TEE 内部 | 明文（隔离） | TEE Enclave | ✅ 安全 |
| IPFS 存储 | 密文 | 所有人 | ✅ 安全 |
| 用户解密 | 明文 | 用户 | - |

### 风险点详细分析

| 风险ID | 位置 | 描述 | 严重度 | 缓解措施 |
|--------|------|------|--------|----------|
| **P1** | 链上 | 加密数据长期存储，未来量子计算可能破解 | 中 | 使用抗量子加密算法 |
| **P2** | OCW | 节点运营者记录元数据（时间、地址、大小） | 低 | 多节点随机分配 |
| **P3** | 网络 | OCW↔TEE 通信被监听 | 低 | TLS + 端到端加密 |
| **P4** | TEE | 侧信道攻击 | 低 | 及时更新微码 |
| **P5** | IPFS | CID 与用户地址关联 | 中 | 使用中继/混淆 |
| **P6** | 链上 | sizhu_index 可反推出生时间 | **高** | **Private 模式** |

### sizhu_index 隐私问题

```
⚠️ 核心隐私风险

链上公开 sizhu_index = [year_gz, month_gz, day_gz, hour_gz]

攻击者可推断:
1. year_gz → 可能年份（60年周期）
2. month_gz → 确定月份范围
3. day_gz → 结合年月确定日期
4. hour_gz → 确定 2 小时窗口

推断精度: 已知年龄 + sizhu_index → 精确到 ±2 小时

解决方案: 使用 Private 模式（不存储 sizhu_index）
```

### 隐私保护等级

| 模式 | sizhu 泄露 | JSON 泄露 | 综合隐私 |
|------|------------|-----------|----------|
| **Public** | 🔴 是 | 🔴 是 | ⭐ 20% |
| **Encrypted** | 🔴 是 | ✅ 否 | ⭐⭐⭐ 60% |
| **Private** | ✅ 否 | ✅ 否 | ⭐⭐⭐⭐⭐ 95% |

---

## IPFS PIN 策略

### 版本管理

```rust
pub enum PinPolicy {
    /// 只 PIN 最新版本
    LatestOnly,
    /// PIN 所有版本
    KeepAll,
    /// 保留最近 N 个版本
    KeepRecent(u32),
}

fn update_manifest_with_pin_policy(
    old_cid: &str,
    new_cid: &str,
    policy: PinPolicy,
) -> Result<(), &'static str> {
    // PIN 新版本
    Self::pin_to_ipfs(new_cid)?;
    
    // 根据策略处理旧版本
    match policy {
        PinPolicy::LatestOnly => Self::unpin_from_ipfs(old_cid)?,
        PinPolicy::KeepAll => {},
        PinPolicy::KeepRecent(n) => Self::cleanup_old_versions(n)?,
    }
    
    Ok(())
}
```

### PIN 服务配置

| 策略 | 说明 | 适用场景 |
|------|------|----------|
| **本地节点** | OCW 调用本地 IPFS | 开发/测试 |
| **Pinata** | 远程 PIN 服务 | 生产环境 |
| **多网关冗余** | 同时 PIN 多个服务 | 高可用需求 |

---

## 配置参数

```rust
parameter_types! {
    /// 请求超时（区块数，~10 分钟）
    pub const RequestTimeout: u32 = 100;
    
    /// OCW 处理间隔（每 5 个区块）
    pub const OcwInterval: u32 = 5;
    
    /// TEE 节点最大数量
    pub const MaxTeeNodes: u32 = 100;
    
    /// 默认 PIN 策略
    pub const DefaultPinPolicy: PinPolicy = PinPolicy::KeepRecent(3);
}
```

---

## TEE 安全模型

```
┌─────────────────────────────────────────────────────────────────┐
│                    TEE 信任边界                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              可信区域 (TCB)                               │    │
│  │   ┌────────────────────────────────────────────────┐    │    │
│  │   │            Intel SGX Enclave                    │    │    │
│  │   │  • CPU 硬件保护                                 │    │    │
│  │   │  • 内存加密 (MEE)                               │    │    │
│  │   │  • 代码完整性验证 (MRENCLAVE)                   │    │    │
│  │   └────────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              不可信区域                                  │    │
│  │  • 操作系统 / OCW 节点                                  │    │
│  │  • 网络传输（需加密）                                   │    │
│  │  • IPFS 存储（加密内容）                                │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 前置条件

> ⚠️ **重要**：TEE 方案依赖 `pallet-tee-privacy` 的 IAS 签名验证功能。
> 当前该功能计划在 Phase 4 实现（参见 C-2 漏洞）。
> 在 IAS 验证实现前，TEE 方案的安全性无法完全保证。

---

## 数据更新机制

### 更新流程

```rust
#[pallet::call_index(2)]
pub fn update_bazi_manifest(
    origin: OriginFor<T>,
    chart_id: u64,
    new_encrypted_input: EncryptedData,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // 1. 验证所有者
    let chart = BaziCharts::<T>::get(&who, chart_id)
        .ok_or(Error::<T>::ChartNotFound)?;
    ensure!(chart.owner == who, Error::<T>::NotOwner);
    
    // 2. 创建更新请求（走相同的 OCW + TEE 流程）
    let request_id = Self::create_update_request(chart_id, new_encrypted_input)?;
    
    Self::deposit_event(Event::BaziUpdateRequested {
        chart_id,
        request_id,
        owner: who,
    });
    
    Ok(())
}
```

### 版本追溯

```
更新后的链上存储:

Version 1 → Version 2 → Version 3
   │            │            │
   ▼            ▼            ▼
CID: abc...  CID: def...  CID: ghi... (当前)

链上记录:
- current_cid: ghi...
- version: 3
- previous_cid: def... (可选保留)
```

---

## 实现路线图

### Phase 1: 基础架构 (2周)

1. 定义统一的 `BaziOnChain` 链上结构
2. 定义 `PendingBaziRequest` 待处理结构
3. 定义 JSON Manifest Schema
4. 实现 `create_bazi` extrinsic（支持隐私模式选择）
5. 实现 `confirm_bazi_result` extrinsic

### Phase 2: OCW 实现 (2周)

1. OCW 轮询待处理请求
2. OCW 处理 Public 模式（直接计算）
3. OCW 调用 TEE HTTP API
4. IPFS 上传 + PIN 服务集成
5. 超时/重试机制

### Phase 3: TEE 集成 (3周)

1. 实现 IAS 签名验证（修复 C-2）
2. TEE Enclave 八字计算逻辑
3. Enclave JSON 生成 + 加密
4. 计算证明生成与验证
5. TEE 节点 HTTP API 服务

### Phase 4: 前端集成 (2周)

1. ECDH 密钥协商
2. 加密/解密 UI
3. 隐私模式选择
4. 端到端测试

### Phase 5: 优化与清理 (1周)

1. 性能优化
2. 数据迁移工具
3. 文档完善

---

## 方案选择建议

| 场景 | 推荐模式 | 理由 |
|------|----------|------|
| **公开展示** | Public | 简单高效，任何人可查看 |
| **个人隐私** | Encrypted | 链上保留 sizhu 用于计算，JSON 加密 |
| **最高隐私** | Private | 链上不存储任何可反推数据 |
| **商业服务** | Private + TEE | 合规性要求，完整审计能力 |

---

## 与现有模块一致性

| 模块 | 链上存储 | 链下清单 | 生成方式 |
|------|----------|----------|----------|
| Evidence | `evidence_hash` + `manifest_cid` | JSON 证据清单 | OCW |
| **BaZi** | `sizhu_index?` + `manifest_cid` | JSON 八字清单 | OCW + TEE |

---

## 总结

**OCW + TEE 结合架构**：

```
┌────────────────────────────────────────────────────────────────┐
│  核心设计原则                                                   │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. OCW 负责调度，TEE 负责计算                                  │
│     - OCW: 轮询、中继、IPFS、提交                              │
│     - TEE: 解密、计算、加密、签名                              │
│                                                                 │
│  2. 三种隐私模式满足不同需求                                    │
│     - Public: 公开展示                                         │
│     - Encrypted: 个人隐私                                      │
│     - Private: 最高隐私                                        │
│                                                                 │
│  3. 端到端加密保护用户数据                                      │
│     - 前端加密 → TEE 计算 → 用户解密                           │
│     - OCW 全程只接触密文                                       │
│                                                                 │
│  4. sizhu_index 是核心隐私风险点                                │
│     - Encrypted 模式仍暴露四柱                                  │
│     - Private 模式完全保护                                      │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

---

## 参考

- `pallets/divination/bazi/src/types.rs` - 八字数据类型定义
- `pallets/divination/tee-privacy/src/lib.rs` - TEE 隐私计算模块
- `pallets/evidence/src/lib.rs` - Evidence 清单模式参考
- IPFS CID v1 规范: https://docs.ipfs.tech/concepts/content-addressing/
