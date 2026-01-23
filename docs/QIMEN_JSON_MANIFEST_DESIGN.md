# 奇门遁甲数据 JSON 清单存储设计

> 文档日期: 2026-01-23  
> 版本: 1.0 (OCW + TEE 结合架构)  
> 状态: 设计方案

---

## 概述

本文档定义奇门遁甲数据 JSON 清单存储方案，采用 **OCW + TEE 结合架构**：
- **OCW**: 负责请求调度、TEE 调用、IPFS 存储、链上提交
- **TEE**: 负责隐私计算（解密、奇门排盘、加密、签名）

### 与八字方案的差异

| 维度 | 八字 (BaZi) | 奇门遁甲 (Qimen) |
|------|-------------|------------------|
| **输入** | 出生时间 | 起卦时间 + 占问事宜 |
| **敏感数据** | 出生时间 | 占问事宜（问题内容） |
| **计算复杂度** | 中等 | **高**（九宫 + 三盘叠加） |
| **数据量** | ~500 bytes | ~800 bytes |
| **链上索引** | sizhu_index | chart_index（局数 + 遁类型） |

---

## 架构总览

```
┌─────────────────────────────────────────────────────────────────────────┐
│  OCW + TEE 结合架构（奇门遁甲）                                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐          │
│  │  前端    │───>│  链上    │───>│   OCW    │───>│   TEE    │          │
│  │ (加密)   │    │ (存储)   │    │ (调度)   │    │ (排盘)   │          │
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
| **TEE** | 解密输入、奇门排盘、生成 JSON、加密输出 | ✅ 是（隔离） |
| **IPFS** | 存储加密 JSON | ❌ 否 |

---

## 隐私模式

用户可选择隐私级别：

| 模式 | 链上公开 | IPFS 内容 | 适用场景 |
|------|----------|-----------|----------|
| **Public** | chart_index + question_type | 明文 JSON | 公开占卜展示 |
| **Encrypted** | chart_index + question_type | 加密 JSON | 个人隐私占卜 |
| **Private** | 无 | 加密 JSON | 最高隐私需求 |

```rust
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum PrivacyMode {
    /// 公开模式：链上存储 chart_index，IPFS 明文
    Public = 0,
    /// 加密模式：链上存储 chart_index，IPFS 加密
    Encrypted = 1,
    /// 私密模式：链上不存储 chart_index，IPFS 加密
    Private = 2,
}
```

---

## 链上存储结构

```rust
/// 统一的奇门链上存储（支持所有隐私模式）
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct QimenOnChain<T: Config> {
    /// 所有者
    pub owner: T::AccountId,
    
    /// 隐私模式
    pub privacy_mode: PrivacyMode,
    
    /// 排盘索引（Private 模式下为 None）
    /// 包含：阴阳遁(1) + 局数(4) + 排盘类型(3) = 8 bits
    pub chart_index: Option<QimenChartIndex>,
    
    /// 问事类型（Private 模式下为 None）
    pub question_type: Option<QuestionType>,
    
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

/// 奇门排盘索引（最小化链上存储）
#[derive(Clone, Copy, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct QimenChartIndex {
    /// 阴阳遁类型 (0=阳遁, 1=阴遁)
    pub dun_type: DunType,
    /// 局数 (1-9)
    pub ju_number: u8,
    /// 排盘类型 (时家/日家/月家/年家)
    pub qimen_type: QimenType,
    /// 排盘方法 (转盘/飞盘)
    pub pan_method: PanMethod,
    /// 值符星
    pub zhi_fu_xing: JiuXing,
    /// 值使门
    pub zhi_shi_men: BaMen,
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
| **Public** | ~110 bytes | chart_index + question_type + cid + hash |
| **Encrypted** | ~160 bytes | 同上 + TEE proof |
| **Private** | ~150 bytes | 无 chart_index + TEE proof |

---

## 待处理请求结构

```rust
/// 待处理的奇门请求
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct PendingQimenRequest<T: Config> {
    /// 请求者
    pub requester: T::AccountId,
    /// 输入数据（Public 模式为明文，其他为密文）
    pub input_data: QimenInputData,
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
pub enum QimenInputData {
    /// 明文输入（Public 模式）
    Plaintext(QimenInputPlain),
    /// 加密输入（Encrypted/Private 模式）
    Encrypted(EncryptedData),
}

/// 明文奇门输入
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct QimenInputPlain {
    /// 起卦时间戳（秒）
    pub timestamp: u64,
    /// 问事类型
    pub question_type: QuestionType,
    /// 占问事宜（可选，最多 128 字节）
    pub question: Option<BoundedVec<u8, ConstU32<128>>>,
    /// 命主姓名（可选，最多 32 字节）
    pub name: Option<BoundedVec<u8, ConstU32<32>>>,
    /// 命主性别（可选）
    pub gender: Option<Gender>,
    /// 命主出生年份（可选，用于年命分析）
    pub birth_year: Option<u16>,
    /// 排盘类型（默认时家奇门）
    pub qimen_type: QimenType,
    /// 排盘方法（默认转盘）
    pub pan_method: PanMethod,
}

/// 加密数据
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct EncryptedData {
    pub ciphertext: BoundedVec<u8, ConstU32<512>>,  // 奇门输入较大
    pub nonce: [u8; 24],
    pub sender_pubkey: [u8; 32],
}

/// 请求状态
#[derive(Clone, Copy, Debug, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq)]
pub enum RequestStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Timeout,
}
```

---

## JSON 清单结构

### 明文模式（Public）

```json
{
  "version": "1.0",
  "schema": "qimen-manifest-v1",
  "created_at": "2026-01-23T10:30:00Z",
  
  "metadata": {
    "privacy_mode": "public",
    "generated_by": "ocw",
    "qimen_type": "时家奇门",
    "pan_method": "转盘"
  },
  
  "requester": {
    "name": "张三",
    "gender": "男",
    "birth_year": 1990
  },
  
  "question": {
    "type": "career",
    "type_name": "事业工作",
    "content": "今年能否升职加薪？"
  },
  
  "time_info": {
    "timestamp": 1737614400,
    "datetime": "2026-01-23 10:30:00",
    "year_ganzhi": { "gan": "丙", "zhi": "午", "ganzhi": "丙午" },
    "month_ganzhi": { "gan": "辛", "zhi": "丑", "ganzhi": "辛丑" },
    "day_ganzhi": { "gan": "甲", "zhi": "子", "ganzhi": "甲子" },
    "hour_ganzhi": { "gan": "己", "zhi": "巳", "ganzhi": "己巳" },
    "jie_qi": "大寒"
  },
  
  "chart_info": {
    "dun_type": "阳遁",
    "san_yuan": "中元",
    "ju_number": 5,
    "zhi_fu_xing": { "name": "天禽", "is_auspicious": true },
    "zhi_shi_men": { "name": "开门", "is_auspicious": true }
  },
  
  "palaces": [
    {
      "gong": 1,
      "gong_name": "坎一宫",
      "direction": "北",
      "tian_pan_gan": { "gan": "戊", "is_san_qi": false },
      "di_pan_gan": { "gan": "癸", "is_san_qi": false },
      "xing": { "name": "天蓬", "is_auspicious": false },
      "men": { "name": "休门", "is_auspicious": true },
      "shen": { "name": "值符", "is_auspicious": true },
      "is_xun_kong": false,
      "is_ma_xing": false,
      "is_fu_yin": false,
      "is_fan_yin": false
    },
    {
      "gong": 2,
      "gong_name": "坤二宫",
      "direction": "西南"
    },
    {
      "gong": 3,
      "gong_name": "震三宫",
      "direction": "东"
    },
    {
      "gong": 4,
      "gong_name": "巽四宫",
      "direction": "东南"
    },
    {
      "gong": 5,
      "gong_name": "中五宫",
      "direction": "中",
      "men": null,
      "shen": null
    },
    {
      "gong": 6,
      "gong_name": "乾六宫",
      "direction": "西北"
    },
    {
      "gong": 7,
      "gong_name": "兑七宫",
      "direction": "西"
    },
    {
      "gong": 8,
      "gong_name": "艮八宫",
      "direction": "东北"
    },
    {
      "gong": 9,
      "gong_name": "离九宫",
      "direction": "南"
    }
  ],
  
  "analysis": {
    "ge_ju": { "type": "正格", "is_auspicious": null },
    "fortune": { "level": "中吉", "score": 70 },
    "yong_shen": {
      "primary": "日干",
      "palace": 3,
      "status": "得力"
    },
    "summary": "日干落震三宫，得吉门吉星相助，事业有望顺遂。"
  }
}
```

### 加密模式（Encrypted / Private）

```json
{
  "version": "1.0",
  "schema": "qimen-manifest-encrypted-v1",
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
const qimenInput = {
  timestamp: Math.floor(Date.now() / 1000),  // 起卦时间
  question_type: QuestionType.Career,
  question: "今年能否升职加薪？",
  name: "张三",
  gender: Gender.Male,
  birth_year: 1990,
  qimen_type: QimenType.ShiJia,  // 时家奇门
  pan_method: PanMethod.ZhuanPan,  // 转盘
};

// ========== Public 模式：明文提交 ==========
async function createQimenPublic(input: QimenInput) {
  await api.tx.qimen.createQimenPublic(
    input.timestamp,
    input.question_type,
    input.question,
    input.name,
    input.gender,
    input.birth_year,
    input.qimen_type,
    input.pan_method,
  ).signAndSend(account);
}

// ========== Encrypted/Private 模式：加密提交 ==========
async function createQimenPrivate(input: QimenInput, privacyMode: PrivacyMode) {
  // 1. 获取 TEE 节点公钥
  const teeNodes = await api.query.teePrivacy.activeNodes.entries();
  const enclavePubkey = teeNodes[0][1].enclave_pubkey;

  // 2. 生成/加载用户密钥对
  const userKeyPair = await getUserKeyPair(account);

  // 3. ECDH 加密输入
  const plaintext = JSON.stringify(input);
  const nonce = randomBytes(24);
  const encrypted = nacl.box(
    new TextEncoder().encode(plaintext),
    nonce,
    enclavePubkey,
    userKeyPair.secretKey
  );

  // 4. 提交链上请求
  await api.tx.qimen.createQimen(
    { ciphertext: Array.from(encrypted), nonce: Array.from(nonce), sender_pubkey: Array.from(userKeyPair.publicKey) },
    Array.from(userKeyPair.publicKey),
    privacyMode,
  ).signAndSend(account);
}

// ========== 统一入口 ==========
async function createQimen(input: QimenInput, privacyMode: PrivacyMode) {
  if (privacyMode === PrivacyMode.Public) {
    await createQimenPublic(input);
  } else {
    await createQimenPrivate(input, privacyMode);
  }
}
```

### Phase 2: 链上处理（Runtime）

```rust
// ========== Public 模式：明文提交 ==========
#[pallet::call_index(0)]
pub fn create_qimen_public(
    origin: OriginFor<T>,
    timestamp: u64,
    question_type: QuestionType,
    question: Option<BoundedVec<u8, ConstU32<128>>>,
    name: Option<BoundedVec<u8, ConstU32<32>>>,
    gender: Option<Gender>,
    birth_year: Option<u16>,
    qimen_type: QimenType,
    pan_method: PanMethod,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    let request_id = Self::next_request_id();
    
    PendingRequests::<T>::insert(request_id, PendingQimenRequest {
        requester: who.clone(),
        input_data: QimenInputData::Plaintext(QimenInputPlain {
            timestamp, question_type, question, name, gender, birth_year, qimen_type, pan_method,
        }),
        user_pubkey: None,
        privacy_mode: PrivacyMode::Public,
        assigned_node: None,
        status: RequestStatus::Pending,
        retry_count: 0,
        created_at: Self::current_block(),
    });
    
    Self::deposit_event(Event::QimenRequestSubmitted {
        request_id,
        requester: who,
        privacy_mode: PrivacyMode::Public,
        assigned_node: None,
    });
    
    Ok(())
}

// ========== Encrypted/Private 模式：加密提交 ==========
#[pallet::call_index(1)]
pub fn create_qimen(
    origin: OriginFor<T>,
    encrypted_input: EncryptedData,
    user_pubkey: [u8; 32],
    privacy_mode: PrivacyMode,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    ensure!(privacy_mode != PrivacyMode::Public, Error::<T>::UsePublicExtrinsic);
    
    let request_id = Self::next_request_id();
    let assigned_node = Self::select_active_tee_node()?;
    
    PendingRequests::<T>::insert(request_id, PendingQimenRequest {
        requester: who.clone(),
        input_data: QimenInputData::Encrypted(encrypted_input),
        user_pubkey: Some(user_pubkey),
        privacy_mode,
        assigned_node: Some(assigned_node.clone()),
        status: RequestStatus::Pending,
        retry_count: 0,
        created_at: Self::current_block(),
    });
    
    Self::deposit_event(Event::QimenRequestSubmitted {
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
const MAX_RETRY_COUNT: u8 = 3;

fn offchain_worker(block_number: BlockNumberFor<T>) {
    if block_number % T::OcwInterval::get() != Zero::zero() {
        return;
    }
    
    for (request_id, request) in PendingRequests::<T>::iter() {
        if request.status == RequestStatus::Completed {
            continue;
        }
        
        if Self::is_request_timeout(&request, block_number) {
            Self::submit_status_update(request_id, RequestStatus::Timeout);
            continue;
        }
        
        if request.retry_count >= MAX_RETRY_COUNT {
            Self::submit_status_update(request_id, RequestStatus::Failed);
            continue;
        }
        
        if request.status == RequestStatus::Pending {
            Self::submit_status_update(request_id, RequestStatus::Processing);
        }
        
        let result = match request.privacy_mode {
            PrivacyMode::Public => {
                Self::process_public_request(&request)
            },
            _ => {
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
                Self::submit_retry_increment(request_id);
            }
        }
    }
}

/// OCW 调用 TEE 节点 HTTP API
fn process_tee_request(request: &PendingQimenRequest<T>) -> Result<ProcessResult, &'static str> {
    let tee_node = TeeNodes::<T>::get(&request.assigned_node.as_ref().unwrap())
        .ok_or("TEE node not found")?;
    
    let encrypted_input = match &request.input_data {
        QimenInputData::Encrypted(data) => data,
        _ => return Err("Expected encrypted input"),
    };
    
    let user_pubkey = request.user_pubkey.as_ref().ok_or("User pubkey required")?;
    
    // HTTP 调用 TEE Enclave
    let tee_response = Self::call_tee_http(
        &tee_node.endpoint,
        encrypted_input,
        user_pubkey,
        request.privacy_mode,
    )?;
    
    // 上传加密 JSON 到 IPFS
    let cid = Self::upload_to_ipfs(&tee_response.encrypted_manifest)?;
    Self::request_pin(&cid)?;
    
    Ok(ProcessResult {
        manifest_cid: cid,
        manifest_hash: tee_response.manifest_hash,
        chart_index: tee_response.chart_index,
        question_type: tee_response.question_type,
        proof: Some(tee_response.computation_proof),
    })
}

fn call_tee_http(
    endpoint: &str,
    encrypted_input: &EncryptedData,
    user_pubkey: &[u8; 32],
    privacy_mode: PrivacyMode,
) -> Result<TeeComputeResponse, &'static str> {
    let request = http::Request::post(
        &format!("{}/compute/qimen", endpoint),  // 奇门专用端点
        serde_json::json!({
            "encrypted_input": {
                "ciphertext": hex::encode(&encrypted_input.ciphertext),
                "nonce": hex::encode(&encrypted_input.nonce),
                "sender_pubkey": hex::encode(&encrypted_input.sender_pubkey),
            },
            "user_pubkey": hex::encode(user_pubkey),
            "privacy_mode": privacy_mode as u8,
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

fn compute_qimen_in_enclave(
    encrypted_input: &[u8],
    nonce: &[u8],
    sender_pubkey: &[u8; 32],
    user_pubkey: &[u8; 32],
    privacy_mode: PrivacyMode,
) -> EnclaveResult {
    // 1. ECDH 解密输入
    let shared_key = ecdh_derive_key(ENCLAVE_PRIVKEY, sender_pubkey);
    let plaintext = aes_gcm_decrypt(&shared_key, nonce, encrypted_input)?;
    let input: QimenInput = serde_json::from_slice(&plaintext)?;
    
    // 2. 奇门排盘计算（所有计算在 Enclave 内存中进行）
    let ganzhi = calculate_ganzhi_from_timestamp(input.timestamp);
    let jie_qi = calculate_jie_qi(input.timestamp);
    let dun_type = determine_dun_type(&jie_qi);
    let san_yuan = calculate_san_yuan(input.timestamp, &jie_qi);
    let ju_number = calculate_ju_number(&dun_type, &san_yuan, &jie_qi);
    let zhi_fu_xing = calculate_zhi_fu_xing(&ganzhi.hour, ju_number);
    let zhi_shi_men = calculate_zhi_shi_men(&ganzhi.hour, ju_number);
    let palaces = arrange_palaces(
        &dun_type, ju_number, &ganzhi, &zhi_fu_xing, &input.pan_method
    );
    
    // 3. 格局分析
    let ge_ju = analyze_ge_ju(&palaces);
    let fortune = calculate_fortune(&palaces, &zhi_fu_xing, &zhi_shi_men);
    let yong_shen_status = analyze_yong_shen(&palaces, &ganzhi, input.question_type);
    
    // 4. 生成 JSON 清单
    let manifest = serde_json::json!({
        "version": "1.0",
        "schema": "qimen-manifest-v1",
        "metadata": {
            "privacy_mode": privacy_mode.name(),
            "qimen_type": input.qimen_type.name(),
            "pan_method": input.pan_method.name(),
        },
        "requester": {
            "name": input.name,
            "gender": input.gender.map(|g| g.name()),
            "birth_year": input.birth_year,
        },
        "question": {
            "type": input.question_type.name(),
            "content": input.question,
        },
        "time_info": ganzhi_to_json(&ganzhi, &jie_qi),
        "chart_info": {
            "dun_type": dun_type.name(),
            "san_yuan": san_yuan.name(),
            "ju_number": ju_number,
            "zhi_fu_xing": xing_to_json(&zhi_fu_xing),
            "zhi_shi_men": men_to_json(&zhi_shi_men),
        },
        "palaces": palaces_to_json(&palaces),
        "analysis": {
            "ge_ju": ge_ju,
            "fortune": fortune,
            "yong_shen": yong_shen_status,
        }
    });
    
    // 5. 用用户公钥加密输出
    let output_key = ecdh_derive_key(ENCLAVE_PRIVKEY, user_pubkey);
    let output_nonce = generate_random_nonce();
    let encrypted_manifest = aes_gcm_encrypt(&output_key, &output_nonce, manifest.to_string().as_bytes());
    
    // 6. 计算哈希
    let manifest_hash = sha256(manifest.to_string().as_bytes());
    
    // 7. Enclave 签名
    let signature = enclave_sign(&[
        &chart_index.encode(),
        &manifest_hash,
    ]);
    
    // 8. 生成计算证明
    let proof = ComputationProof {
        mrenclave: MRENCLAVE,
        timestamp: current_time(),
        signature,
    };
    
    // 9. 生成 chart_index（根据 privacy_mode）
    let chart_index = match privacy_mode {
        PrivacyMode::Private => None,
        _ => Some(QimenChartIndex {
            dun_type,
            ju_number,
            qimen_type: input.qimen_type,
            pan_method: input.pan_method,
            zhi_fu_xing,
            zhi_shi_men,
        }),
    };
    
    EnclaveResult {
        encrypted_manifest: EncryptedManifest {
            algorithm: "AES-256-GCM",
            nonce: output_nonce,
            enclave_pubkey: ENCLAVE_PUBKEY,
            ciphertext: encrypted_manifest,
        },
        chart_index,
        question_type: match privacy_mode {
            PrivacyMode::Private => None,
            _ => Some(input.question_type),
        },
        manifest_hash,
        computation_proof: proof,
    }
}
```

### Phase 5: 结果提交（OCW → 链上）

```rust
#[pallet::call_index(2)]
pub fn confirm_qimen_result(
    origin: OriginFor<T>,
    request_id: u64,
    manifest_cid: Vec<u8>,
    manifest_hash: [u8; 32],
    chart_index: Option<QimenChartIndex>,
    question_type: Option<QuestionType>,
    computation_proof: Option<ComputationProof>,
    enclave_signature: Option<[u8; 64]>,
) -> DispatchResult {
    ensure_none(origin)?;
    
    let request = PendingRequests::<T>::take(request_id)
        .ok_or(Error::<T>::RequestNotFound)?;
    
    // 验证 TEE 签名和证明
    if let Some(proof) = &computation_proof {
        Self::verify_enclave_signature(
            &request.assigned_node.unwrap(),
            &chart_index,
            &manifest_hash,
            &enclave_signature.unwrap(),
        )?;
        Self::verify_computation_proof(proof)?;
    }
    
    let chart_id = Self::next_chart_id(&request.requester);
    
    let cid_bounded: BoundedVec<u8, ConstU32<64>> = manifest_cid
        .try_into()
        .map_err(|_| Error::<T>::CidTooLong)?;
    
    QimenCharts::<T>::insert(&request.requester, chart_id, QimenOnChain {
        owner: request.requester.clone(),
        privacy_mode: request.privacy_mode,
        chart_index,
        question_type,
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
    
    Self::deposit_event(Event::QimenCreated {
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
async function viewQimenChart(chartId: number) {
  // 1. 获取链上数据
  const onChain = await api.query.qimen.qimenCharts(account.address, chartId);
  
  // 2. 从 IPFS 获取加密清单
  const response = await fetch(`https://ipfs.io/ipfs/${onChain.manifest_cid}`);
  const encryptedManifest = await response.json();
  
  // 3. 如果是加密模式，解密
  if (onChain.privacy_mode !== 'Public') {
    const decrypted = nacl.box.open(
      new Uint8Array(encryptedManifest.ciphertext),
      new Uint8Array(encryptedManifest.nonce),
      new Uint8Array(encryptedManifest.enclave_pubkey),
      userKeyPair.secretKey
    );
    
    if (!decrypted) {
      throw new Error('Decryption failed - wrong key?');
    }
    
    return JSON.parse(Buffer.from(decrypted).toString());
  }
  
  return encryptedManifest;
}
```

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

### 奇门特有隐私风险

| 风险ID | 位置 | 描述 | 严重度 | 缓解措施 |
|--------|------|------|--------|----------|
| **Q1** | 链上 | 占问事宜泄露（如"能否升职"） | **高** | 加密存储 |
| **Q2** | 链上 | question_type 暴露用户关注点 | 中 | Private 模式 |
| **Q3** | 链上 | chart_index 可推断起卦时间 | 中 | Private 模式 |
| **Q4** | IPFS | 姓名、问题等敏感信息 | **高** | 端到端加密 |

### chart_index 隐私问题

```
⚠️ 隐私风险（低于八字）

链上公开 chart_index = [dun_type, ju_number, qimen_type, ...]

攻击者可推断:
1. dun_type → 大致季节（阳遁=冬至~夏至，阴遁=夏至~冬至）
2. ju_number → 特定日期范围
3. question_type → 用户关注领域

推断精度: 相对较低，不如八字精确

解决方案: 敏感占卜使用 Private 模式
```

### 隐私保护等级

| 模式 | chart_index 泄露 | question 泄露 | 综合隐私 |
|------|------------------|---------------|----------|
| **Public** | 🔴 是 | 🔴 是 | ⭐ 20% |
| **Encrypted** | 🔴 是 | ✅ 否 | ⭐⭐⭐ 60% |
| **Private** | ✅ 否 | ✅ 否 | ⭐⭐⭐⭐⭐ 95% |

---

## 配置参数

```rust
parameter_types! {
    /// 请求超时（区块数，~15 分钟，奇门计算更复杂）
    pub const RequestTimeout: u32 = 150;
    
    /// OCW 处理间隔（每 5 个区块）
    pub const OcwInterval: u32 = 5;
    
    /// TEE 节点最大数量
    pub const MaxTeeNodes: u32 = 100;
    
    /// 默认 PIN 策略
    pub const DefaultPinPolicy: PinPolicy = PinPolicy::KeepRecent(3);
    
    /// 最大问题长度
    pub const MaxQuestionLen: u32 = 128;
    
    /// 最大姓名长度
    pub const MaxNameLen: u32 = 32;
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
│  │   │  • 奇门排盘计算（九宫三盘叠加）                 │    │    │
│  │   │  • 占问事宜解密与处理                          │    │    │
│  │   │  • 格局分析与吉凶判断                          │    │    │
│  │   │  • JSON 清单生成与加密                         │    │    │
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

## 实现路线图

### Phase 1: 基础架构 (2周)

1. 定义统一的 `QimenOnChain` 链上结构
2. 定义 `PendingQimenRequest` 待处理结构
3. 定义 JSON Manifest Schema
4. 实现 `create_qimen_public` extrinsic
5. 实现 `create_qimen` extrinsic（加密模式）
6. 实现 `confirm_qimen_result` extrinsic

### Phase 2: OCW 实现 (2周)

1. OCW 轮询待处理请求
2. OCW 处理 Public 模式（直接计算）
3. OCW 调用 TEE HTTP API（奇门端点）
4. IPFS 上传 + PIN 服务集成
5. 超时/重试机制

### Phase 3: TEE 集成 (3周)

1. TEE Enclave 奇门排盘算法移植
2. Enclave JSON 生成 + 加密
3. 计算证明生成与验证
4. TEE 节点 HTTP API 服务（`/compute/qimen`）
5. 格局分析与吉凶判断逻辑

### Phase 4: 前端集成 (2周)

1. ECDH 密钥协商
2. 加密/解密 UI
3. 隐私模式选择
4. 奇门盘面展示组件
5. 端到端测试

### Phase 5: 优化与清理 (1周)

1. 性能优化（九宫计算较复杂）
2. 数据迁移工具
3. 文档完善

---

## 方案选择建议

| 场景 | 推荐模式 | 理由 |
|------|----------|------|
| **公开展示** | Public | 简单高效，任何人可查看 |
| **个人占卜** | Encrypted | 链上保留 chart_index 用于分析，问题加密 |
| **敏感问题** | Private | 涉及隐私的占问（如健康、官司） |
| **商业服务** | Private + TEE | 合规性要求，完整审计能力 |

---

## 与八字模块一致性

| 模块 | 链上索引 | 敏感数据 | 生成方式 |
|------|----------|----------|----------|
| **BaZi** | `sizhu_index` | 出生时间 | OCW + TEE |
| **Qimen** | `chart_index` | 占问事宜 | OCW + TEE |

---

## 总结

**OCW + TEE 结合架构（奇门遁甲）**：

```
┌────────────────────────────────────────────────────────────────┐
│  核心设计原则                                                   │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. OCW 负责调度，TEE 负责计算                                  │
│     - OCW: 轮询、中继、IPFS、提交                              │
│     - TEE: 解密、排盘、加密、签名                              │
│                                                                 │
│  2. 三种隐私模式满足不同需求                                    │
│     - Public: 公开占卜展示                                     │
│     - Encrypted: 个人隐私占卜                                  │
│     - Private: 敏感问题占卜                                    │
│                                                                 │
│  3. 端到端加密保护用户数据                                      │
│     - 前端加密 → TEE 计算 → 用户解密                           │
│     - OCW 全程只接触密文                                       │
│                                                                 │
│  4. 占问事宜是核心隐私风险点                                    │
│     - 问题内容可能包含敏感信息                                  │
│     - Encrypted 模式加密存储                                   │
│     - Private 模式完全保护                                      │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

---

## 参考

- `pallets/divination/qimen/src/types.rs` - 奇门遁甲数据类型定义
- `pallets/divination/qimen/src/algorithm.rs` - 奇门排盘算法
- `pallets/divination/tee-privacy/src/lib.rs` - TEE 隐私计算模块
- `docs/BAZI_JSON_MANIFEST_DESIGN.md` - 八字 JSON 清单设计（参考）
- IPFS CID v1 规范: https://docs.ipfs.tech/concepts/content-addressing/
