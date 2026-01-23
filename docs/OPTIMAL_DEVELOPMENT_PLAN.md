# 最优开发方案 - 综合分析报告

> 分析日期: 2026-01-23  
> 版本: 1.0  
> 状态: 最终方案

---

## 一、文档总览

### 1.1 现有文档清单

| 文档 | 核心内容 | 状态 |
|------|----------|------|
| `BAZI_JSON_MANIFEST_DESIGN.md` | 八字 OCW+TEE 架构设计 | ✅ 完整 |
| `QIMEN_JSON_MANIFEST_DESIGN.md` | 奇门 OCW+TEE 架构设计 | ✅ 完整 |
| `OCW_TEE_COMMON_DESIGN.md` | 通用架构抽象设计 | ✅ 完整 |
| `OCW_TEE_COMMON_DESIGN_ANALYSIS.md` | 通用架构深度分析 | ✅ 完整 |

### 1.2 文档关系图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           文档依赖关系                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │              OCW_TEE_COMMON_DESIGN_ANALYSIS.md                   │    │
│  │                    (深度分析报告)                                │    │
│  └───────────────────────────┬─────────────────────────────────────┘    │
│                              │ 分析                                      │
│                              ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                  OCW_TEE_COMMON_DESIGN.md                        │    │
│  │                    (通用架构设计)                                │    │
│  └───────────────────────────┬─────────────────────────────────────┘    │
│                              │ 抽象自                                    │
│              ┌───────────────┴───────────────┐                          │
│              ▼                               ▼                           │
│  ┌─────────────────────┐        ┌─────────────────────┐                 │
│  │ BAZI_JSON_MANIFEST  │        │ QIMEN_JSON_MANIFEST │                 │
│  │    _DESIGN.md       │        │    _DESIGN.md       │                 │
│  │  (八字专用设计)     │        │  (奇门专用设计)     │                 │
│  └─────────────────────┘        └─────────────────────┘                 │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 二、核心发现

### 2.1 设计一致性分析

| 维度 | 八字 | 奇门 | 通用设计 | 一致性 |
|------|------|------|----------|--------|
| **隐私模式** | Public/Encrypted/Private | Public/Encrypted/Private | Public/Encrypted/Private | ✅ 一致 |
| **请求状态** | Pending/Processing/Completed/Failed/Timeout | 同左 | 同左 | ✅ 一致 |
| **OCW 流程** | 6 Phase | 6 Phase | 通用抽象 | ✅ 一致 |
| **TEE 调用** | HTTP API | HTTP API | 通用 Trait | ✅ 一致 |
| **IPFS 存储** | CID + Hash | CID + Hash | 通用接口 | ✅ 一致 |
| **重试机制** | MAX_RETRY=3 | MAX_RETRY=3 | MAX_RETRY=3 | ✅ 一致 |

### 2.2 关键差异点

| 维度 | 八字 | 奇门 | 影响 |
|------|------|------|------|
| **链上索引** | `SiZhuIndex` | `QimenChartIndex` | 需要泛型设计 |
| **输入大小** | 256 bytes | 512 bytes | 需要可配置 |
| **计算超时** | 100 区块 | 150 区块 | 需要按类型配置 |
| **敏感数据** | 出生时间 | 占问事宜 | 隐私风险不同 |
| **JSON 结构** | sizhu/dayun/wuxing | palaces/analysis | 各自定义 |

### 2.3 分析报告关键发现

**来自 `OCW_TEE_COMMON_DESIGN_ANALYSIS.md`**：

#### ✅ 优势
1. 架构清晰，三层职责分明
2. 代码复用率高（预计节省 73%）
3. 扩展性好，新模块易于集成
4. 类型安全，使用 Rust 类型系统保证

#### ⚠️ 需要改进
1. 与现有 `pallet-divination-privacy` 的 `PrivacyMode` 不完全一致（`Partial` vs `Encrypted`）
2. 加密数据格式差异（nonce 长度 12 vs 24，auth_tag 有无）
3. OCW 性能限制（大量请求可能超时）
4. HTTP 请求安全性（TLS 验证）

---

## 三、最优开发方案

### 3.1 方案概述

**采用"通用架构 + 模块适配"策略**：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         最优开发方案                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Phase 0: 设计评审与统一 (1周)                                          │
│     ↓                                                                    │
│  Phase 1: 通用模块实现 (3周)                                            │
│     ↓                                                                    │
│  Phase 2: 八字模块迁移 (2周)                                            │
│     ↓                                                                    │
│  Phase 3: 奇门模块迁移 (2周)                                            │
│     ↓                                                                    │
│  Phase 4: 其他模块迁移 (4周)                                            │
│     ↓                                                                    │
│  Phase 5: 前端 SDK (2周)                                                │
│     ↓                                                                    │
│  Phase 6: 测试与优化 (2周)                                              │
│                                                                          │
│  总计: 16周                                                              │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 详细实施计划

---

## Phase 0: 设计评审与统一 (1周)

### 目标
- 统一设计决策
- 解决文档间的差异
- 确定技术选型

### 任务清单

#### P0-1: 统一隐私模式定义
```rust
// 最终统一方案
pub enum PrivacyMode {
    /// 公开模式：链上存储索引，IPFS 明文
    Public = 0,
    /// 加密模式：链上存储索引，IPFS 加密（原 Partial）
    Encrypted = 1,
    /// 私密模式：链上不存储索引，IPFS 加密
    Private = 2,
}

// 兼容性别名（迁移期间使用）
pub type Partial = Encrypted;
```

#### P0-2: 统一加密数据格式
```rust
// 最终统一方案（兼容 X25519 + AES-256-GCM）
pub struct EncryptedData<MaxLen: Get<u32>> {
    /// 密文
    pub ciphertext: BoundedVec<u8, MaxLen>,
    /// 随机数（24 字节，兼容 XSalsa20）
    pub nonce: [u8; 24],
    /// 发送方公钥（用于 ECDH）
    pub sender_pubkey: [u8; 32],
}

// 为兼容现有 12 字节 nonce 的 AES-GCM，提供转换函数
impl<MaxLen: Get<u32>> EncryptedData<MaxLen> {
    pub fn from_aes_gcm(data: &[u8], nonce_12: [u8; 12], pubkey: [u8; 32]) -> Self {
        let mut nonce_24 = [0u8; 24];
        nonce_24[..12].copy_from_slice(&nonce_12);
        Self {
            ciphertext: data.to_vec().try_into().unwrap(),
            nonce: nonce_24,
            sender_pubkey: pubkey,
        }
    }
}
```

#### P0-3: 统一计算证明格式
```rust
// 最终统一方案（合并两种设计的优点）
pub struct ComputationProof {
    /// MRENCLAVE（Enclave 代码哈希，用于身份验证）
    pub mrenclave: [u8; 32],
    /// 输入哈希（用于数据完整性验证）
    pub input_hash: [u8; 32],
    /// 输出哈希（用于数据完整性验证）
    pub output_hash: [u8; 32],
    /// 计算时间戳
    pub timestamp: u64,
    /// Enclave 签名
    pub signature: [u8; 64],
}
```

#### P0-4: 确定模块路径
```
pallets/divination/ocw-tee/
├── Cargo.toml
├── src/
│   ├── lib.rs           # 主模块入口
│   ├── types.rs         # 通用类型定义
│   ├── traits.rs        # 可扩展 trait
│   ├── ocw.rs           # OCW 调度逻辑
│   ├── tee_client.rs    # TEE HTTP 客户端
│   ├── ipfs.rs          # IPFS 上传/PIN
│   ├── retry.rs         # 重试机制
│   ├── error.rs         # 统一错误处理 ← 新增
│   ├── config.rs        # 配置参数 ← 新增
│   ├── events.rs        # 事件定义 ← 新增
│   └── tests.rs         # 测试
```

### 交付物
- [ ] 统一的类型定义文档
- [ ] 技术选型决策记录
- [ ] 迁移兼容性方案

---

## Phase 1: 通用模块实现 (3周)

### 目标
- 实现 `pallet-divination-ocw-tee` 核心功能
- 提供可复用的基础设施

### Week 1: 类型和 Trait

#### 任务
1. **types.rs**: 实现所有通用类型
2. **traits.rs**: 实现核心 Trait
3. **error.rs**: 实现统一错误处理

```rust
// error.rs
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum OcwTeeError {
    // TEE 相关
    TeeNodeNotFound,
    TeeNodeOffline,
    TeeRequestFailed(BoundedVec<u8, ConstU32<128>>),
    TeeResponseInvalid,
    TeeSignatureInvalid,
    
    // IPFS 相关
    IpfsUploadFailed,
    IpfsPinFailed,
    IpfsUnpinFailed,
    
    // 请求相关
    RequestNotFound,
    RequestAlreadyProcessed,
    RequestTimeout,
    MaxRetriesExceeded,
    
    // 加密相关
    EncryptionFailed,
    DecryptionFailed,
    InvalidPublicKey,
    
    // 其他
    InvalidInput,
    InternalError,
}
```

### Week 2: OCW 调度逻辑

#### 任务
1. **ocw.rs**: 实现通用 OCW 处理器
2. **retry.rs**: 实现重试机制
3. **config.rs**: 实现配置管理

```rust
// config.rs
pub trait OcwTeeConfig: frame_system::Config {
    /// OCW 处理间隔（区块数）
    type OcwInterval: Get<BlockNumberFor<Self>>;
    
    /// 最大重试次数
    type MaxRetryCount: Get<u8>;
    
    /// 每区块最大处理请求数（性能优化）
    type MaxRequestsPerBlock: Get<u32>;
    
    /// TEE 请求超时（毫秒）
    type TeeRequestTimeout: Get<u64>;
    
    /// IPFS 客户端
    type IpfsClient: IpfsClient;
    
    /// TEE 节点管理
    type TeeNodeManager: TeeNodeManager<Self::AccountId>;
}
```

### Week 3: TEE 和 IPFS 客户端

#### 任务
1. **tee_client.rs**: 实现 TEE HTTP 客户端
2. **ipfs.rs**: 实现 IPFS 客户端
3. **集成测试**

```rust
// tee_client.rs
pub struct TeeHttpClient;

impl TeeHttpClient {
    pub fn call<T: Serialize, R: DeserializeOwned>(
        endpoint: &str,
        divination_type: DivinationType,
        request: &T,
        timeout_ms: u64,
    ) -> Result<R, OcwTeeError> {
        let url = format!("{}{}", endpoint, divination_type.tee_endpoint());
        
        let body = serde_json::to_vec(request)
            .map_err(|_| OcwTeeError::InvalidInput)?;
        
        let http_request = http::Request::post(&url, body)
            .add_header("Content-Type", "application/json")
            .deadline(sp_io::offchain::timestamp().add(Duration::from_millis(timeout_ms)));
        
        let response = http_request.send()
            .map_err(|_| OcwTeeError::TeeRequestFailed(b"HTTP error".to_vec().try_into().unwrap()))?;
        
        if response.code != 200 {
            return Err(OcwTeeError::TeeRequestFailed(
                format!("HTTP {}", response.code).into_bytes().try_into().unwrap()
            ));
        }
        
        let body = response.body().collect::<Vec<u8>>();
        serde_json::from_slice(&body)
            .map_err(|_| OcwTeeError::TeeResponseInvalid)
    }
}
```

### 交付物
- [ ] `pallet-divination-ocw-tee` 完整实现
- [ ] 单元测试覆盖率 > 80%
- [ ] API 文档

---

## Phase 2: 八字模块迁移 (2周)

### 目标
- 将八字模块迁移到通用架构
- 验证通用架构可行性

### Week 1: Trait 实现

```rust
// pallets/divination/bazi/src/ocw_integration.rs

impl<T: Config> DivinationCompute for Pallet<T> {
    type PlainInput = BaziInputPlain;
    type Index = SiZhuIndex;
    type Result = BaziChart;
    
    fn divination_type() -> DivinationType {
        DivinationType::BaZi
    }
    
    fn compute(input: &Self::PlainInput) -> Result<Self::Result, OcwTeeError> {
        // 复用现有计算逻辑
        let sizhu = Self::calculate_sizhu(input.year, input.month, input.day, input.hour)?;
        let dayun = Self::calculate_dayun(&sizhu, input.gender, input.year)?;
        let wuxing = Self::calculate_wuxing_strength(&sizhu);
        let xiyong = Self::determine_xiyong_shen(&sizhu, &wuxing);
        
        Ok(BaziChart {
            sizhu,
            dayun,
            wuxing_strength: wuxing,
            xiyong_shen: xiyong,
            // ...
        })
    }
    
    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index> {
        match privacy_mode {
            PrivacyMode::Private => None,
            _ => Some(result.sizhu.to_index()),
        }
    }
    
    fn generate_manifest(
        input: &Self::PlainInput,
        result: &Self::Result,
        privacy_mode: PrivacyMode,
    ) -> Result<Vec<u8>, OcwTeeError> {
        let manifest = BaziManifest {
            version: "1.0".to_string(),
            schema: "bazi-manifest-v1".to_string(),
            privacy_mode: privacy_mode.name().to_string(),
            sizhu: result.sizhu.to_json(),
            dayun: result.dayun.to_json(),
            analysis: BaziAnalysis {
                wuxing_strength: result.wuxing_strength.clone(),
                xiyong_shen: result.xiyong_shen.clone(),
            },
        };
        
        serde_json::to_vec(&manifest)
            .map_err(|_| OcwTeeError::InternalError)
    }
}
```

### Week 2: Extrinsic 迁移和测试

```rust
// 简化后的 extrinsic
#[pallet::call_index(0)]
pub fn create_bazi_public(
    origin: OriginFor<T>,
    year: u16, month: u8, day: u8, hour: u8,
    gender: Gender,
    longitude: Option<i32>,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // 委托给通用模块
    pallet_divination_ocw_tee::Pallet::<T>::create_request(
        who,
        DivinationType::BaZi,
        BaziInputPlain { year, month, day, hour, gender, longitude }.encode(),
        None,
        PrivacyMode::Public,
    )
}

#[pallet::call_index(1)]
pub fn create_bazi(
    origin: OriginFor<T>,
    encrypted_input: DefaultEncryptedData,
    user_pubkey: [u8; 32],
    privacy_mode: PrivacyMode,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    ensure!(privacy_mode != PrivacyMode::Public, Error::<T>::UsePublicExtrinsic);
    
    pallet_divination_ocw_tee::Pallet::<T>::create_request(
        who,
        DivinationType::BaZi,
        encrypted_input.encode(),
        Some(user_pubkey),
        privacy_mode,
    )
}
```

### 交付物
- [ ] 八字模块完成迁移
- [ ] 集成测试通过
- [ ] 性能基准测试

---

## Phase 3: 奇门模块迁移 (2周)

### 目标
- 将奇门模块迁移到通用架构
- 验证复杂计算场景

### Week 1: Trait 实现

```rust
// pallets/divination/qimen/src/ocw_integration.rs

impl<T: Config> DivinationCompute for Pallet<T> {
    type PlainInput = QimenInputPlain;
    type Index = QimenChartIndex;
    type Result = QimenChart;
    
    fn divination_type() -> DivinationType {
        DivinationType::QiMen
    }
    
    fn compute(input: &Self::PlainInput) -> Result<Self::Result, OcwTeeError> {
        // 复用现有排盘逻辑
        let ganzhi = Self::calculate_ganzhi_from_timestamp(input.timestamp)?;
        let jie_qi = Self::calculate_jie_qi(input.timestamp)?;
        let dun_type = Self::determine_dun_type(&jie_qi);
        let san_yuan = Self::calculate_san_yuan(input.timestamp, &jie_qi)?;
        let ju_number = Self::calculate_ju_number(&dun_type, &san_yuan, &jie_qi)?;
        let zhi_fu_xing = Self::calculate_zhi_fu_xing(&ganzhi.hour, ju_number)?;
        let zhi_shi_men = Self::calculate_zhi_shi_men(&ganzhi.hour, ju_number)?;
        let palaces = Self::arrange_palaces(
            &dun_type, ju_number, &ganzhi, &zhi_fu_xing, &input.pan_method
        )?;
        
        Ok(QimenChart {
            ganzhi,
            jie_qi,
            dun_type,
            san_yuan,
            ju_number,
            zhi_fu_xing,
            zhi_shi_men,
            palaces,
            // ...
        })
    }
    
    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index> {
        match privacy_mode {
            PrivacyMode::Private => None,
            _ => Some(QimenChartIndex {
                dun_type: result.dun_type,
                ju_number: result.ju_number,
                qimen_type: result.qimen_type,
                pan_method: result.pan_method,
                zhi_fu_xing: result.zhi_fu_xing,
                zhi_shi_men: result.zhi_shi_men,
            }),
        }
    }
    
    // ...
}
```

### Week 2: 测试和优化

- 复杂计算性能测试
- 超时处理验证
- 九宫数据完整性测试

### 交付物
- [ ] 奇门模块完成迁移
- [ ] 性能优化（复杂计算场景）
- [ ] 文档更新

---

## Phase 4: 其他模块迁移 (4周)

### 目标
- 迁移剩余 6 个占卜模块
- 验证通用架构的广泛适用性

### 迁移顺序（按复杂度）

| 周次 | 模块 | 复杂度 | 特殊处理 |
|------|------|--------|----------|
| Week 1 | 小六壬 + 塔罗 | 低 | 简单输入 |
| Week 2 | 梅花易数 | 低 | 数字起卦 |
| Week 3 | 六爻 | 中 | 摇卦结果 |
| Week 4 | 紫微斗数 + 大六壬 | 高 | 复杂计算 |

### 每个模块迁移步骤

1. 实现 `DivinationCompute` trait
2. 定义模块特定的 `Index` 类型
3. 实现 JSON Manifest 生成
4. 迁移 extrinsic
5. 编写测试
6. 更新文档

### 交付物
- [ ] 所有 8 个占卜模块完成迁移
- [ ] 统一的测试套件
- [ ] 迁移报告

---

## Phase 5: 前端 SDK (2周)

### 目标
- 提供统一的前端 SDK
- 简化前端集成

### Week 1: 核心 SDK

```typescript
// @stardust/divination-sdk

export class DivinationSDK {
  private api: ApiPromise;
  private keyManager: KeyManager;
  
  constructor(api: ApiPromise) {
    this.api = api;
    this.keyManager = new KeyManager();
  }
  
  // 通用创建方法
  async create<T extends DivinationInput>(
    type: DivinationType,
    input: T,
    privacyMode: PrivacyMode,
  ): Promise<string> {
    if (privacyMode === PrivacyMode.Public) {
      return this.createPublic(type, input);
    }
    return this.createPrivate(type, input, privacyMode);
  }
  
  // 通用查看方法
  async view(
    type: DivinationType,
    chartId: number,
  ): Promise<DivinationResult> {
    const onChain = await this.getOnChainData(type, chartId);
    const manifest = await this.fetchFromIpfs(onChain.manifest_cid);
    
    if (onChain.privacy_mode !== PrivacyMode.Public) {
      return this.decryptManifest(manifest);
    }
    return manifest;
  }
  
  // 密钥管理
  async initKeyPair(account: string): Promise<void> {
    await this.keyManager.init(account);
  }
  
  // ...
}

// 类型特定客户端
export class BaziClient extends DivinationSDK {
  async createBazi(input: BaziInput, privacyMode: PrivacyMode) {
    return this.create(DivinationType.BaZi, input, privacyMode);
  }
}

export class QimenClient extends DivinationSDK {
  async createQimen(input: QimenInput, privacyMode: PrivacyMode) {
    return this.create(DivinationType.QiMen, input, privacyMode);
  }
}

// ... 其他模块客户端
```

### Week 2: 密钥管理和文档

```typescript
// 密钥管理器
export class KeyManager {
  private keyPair: nacl.BoxKeyPair | null = null;
  
  async init(account: string): Promise<void> {
    // 1. 尝试从 localStorage 加载
    const cached = this.loadFromStorage(account);
    if (cached) {
      this.keyPair = cached;
      return;
    }
    
    // 2. 尝试从助记词派生
    const mnemonic = await this.promptForMnemonic();
    if (mnemonic) {
      this.keyPair = await this.deriveFromMnemonic(mnemonic);
      this.saveToStorage(account, this.keyPair);
      return;
    }
    
    // 3. 生成新密钥对
    this.keyPair = nacl.box.keyPair();
    await this.promptBackup(this.keyPair);
    this.saveToStorage(account, this.keyPair);
  }
  
  // ...
}
```

### 交付物
- [ ] `@stardust/divination-sdk` npm 包
- [ ] API 文档
- [ ] 使用示例

---

## Phase 6: 测试与优化 (2周)

### 目标
- 全面测试
- 性能优化
- 文档完善

### Week 1: 测试

#### 单元测试
- 所有 Trait 实现
- 类型转换
- 错误处理

#### 集成测试
- OCW 完整流程
- TEE 通信
- IPFS 上传

#### 压力测试
- 大量请求处理
- 节点故障恢复
- 网络异常处理

### Week 2: 优化和文档

#### 性能优化
- 优先级队列实现
- 批量处理优化
- 缓存机制

#### 文档完善
- 架构文档更新
- API 参考
- 迁移指南
- 故障排除指南

### 交付物
- [ ] 测试报告
- [ ] 性能基准
- [ ] 完整文档

---

## 四、风险管理

### 4.1 技术风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| OCW 性能不足 | 中 | 高 | 限制每区块处理数，优先级队列 |
| TEE 节点不稳定 | 中 | 高 | 重试机制，节点故障转移 |
| IPFS 上传失败 | 低 | 中 | 重试机制，多节点冗余 |
| 加密兼容性问题 | 中 | 中 | 适配层，渐进迁移 |

### 4.2 进度风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 模块迁移超时 | 中 | 中 | 预留缓冲时间，并行开发 |
| 测试发现重大问题 | 低 | 高 | 早期测试，持续集成 |
| 需求变更 | 低 | 中 | 模块化设计，灵活架构 |

### 4.3 应急预案

1. **OCW 性能问题**：降级为同步处理，限制请求频率
2. **TEE 节点故障**：自动切换备用节点，通知运维
3. **IPFS 不可用**：临时使用链上存储（仅限小数据）

---

## 五、资源需求

### 5.1 人力资源

| 角色 | 人数 | 职责 |
|------|------|------|
| 后端开发 | 2 | Rust pallet 开发 |
| 前端开发 | 1 | SDK 和 UI 集成 |
| 测试工程师 | 1 | 测试和质量保证 |
| 技术文档 | 0.5 | 文档编写 |

### 5.2 基础设施

| 资源 | 数量 | 用途 |
|------|------|------|
| TEE 节点 | 3+ | 生产环境 |
| IPFS 节点 | 2+ | 数据存储 |
| 测试网络 | 1 | 开发测试 |

---

## 六、成功标准

### 6.1 功能标准

- [ ] 所有 8 个占卜模块支持 OCW + TEE 架构
- [ ] 三种隐私模式正常工作
- [ ] 前端 SDK 功能完整

### 6.2 性能标准

- [ ] 单请求处理时间 < 10 秒
- [ ] 每区块处理请求数 > 10
- [ ] 系统可用性 > 99.9%

### 6.3 质量标准

- [ ] 测试覆盖率 > 80%
- [ ] 无 P0/P1 级别 bug
- [ ] 文档完整

---

## 七、总结

### 7.1 方案优势

1. **代码复用率高**：预计节省 70%+ 重复代码
2. **扩展性好**：新模块只需实现 `DivinationCompute` trait
3. **维护成本低**：统一架构，集中维护
4. **类型安全**：Rust 类型系统保证正确性

### 7.2 关键决策

1. **采用通用架构**：而非各模块独立实现
2. **统一隐私模式**：Public/Encrypted/Private
3. **分阶段迁移**：先核心模块，后扩展模块
4. **前端 SDK 统一**：一套 API 支持所有模块

### 7.3 下一步行动

1. **立即**：评审本方案，确认技术选型
2. **本周**：启动 Phase 0，统一设计决策
3. **下周**：启动 Phase 1，开始通用模块开发

---

## 附录

### A. 文档更新建议

| 文档 | 建议 |
|------|------|
| `BAZI_JSON_MANIFEST_DESIGN.md` | 添加"迁移到通用架构"章节 |
| `QIMEN_JSON_MANIFEST_DESIGN.md` | 添加"迁移到通用架构"章节 |
| `OCW_TEE_COMMON_DESIGN.md` | 根据本方案更新实现细节 |
| `OCW_TEE_COMMON_DESIGN_ANALYSIS.md` | 标记为"已采纳建议" |

### B. 相关代码路径

```
pallets/divination/
├── ocw-tee/           # 新增：通用 OCW+TEE 模块
├── bazi/              # 迁移：实现 DivinationCompute
├── qimen/             # 迁移：实现 DivinationCompute
├── meihua/            # 迁移：实现 DivinationCompute
├── liuyao/            # 迁移：实现 DivinationCompute
├── ziwei/             # 迁移：实现 DivinationCompute
├── daliuren/          # 迁移：实现 DivinationCompute
├── xiaoliuren/        # 迁移：实现 DivinationCompute
├── tarot/             # 迁移：实现 DivinationCompute
├── privacy/           # 现有：隐私存储
├── tee-privacy/       # 现有：TEE 节点管理
└── common/            # 现有：公共类型
```

### C. 参考文档

- `docs/BAZI_JSON_MANIFEST_DESIGN.md`
- `docs/QIMEN_JSON_MANIFEST_DESIGN.md`
- `docs/OCW_TEE_COMMON_DESIGN.md`
- `docs/OCW_TEE_COMMON_DESIGN_ANALYSIS.md`

---

**文档完成**
