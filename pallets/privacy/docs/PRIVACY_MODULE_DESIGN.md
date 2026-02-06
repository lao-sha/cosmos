# Cosmos 隐私计算模块设计方案

> 基于 Phala Network 深度分析，结合 Cosmos 项目特点设计

## 目录

1. [Phala Network 深度分析](#1-phala-network-深度分析)
2. [Cosmos 隐私模块总体设计](#2-cosmos-隐私模块总体设计)
3. [Pallet 子模块详细设计](#3-pallet-子模块详细设计)
4. [与现有模块集成方案](#4-与现有模块集成方案)
5. [实施路线图](#5-实施路线图)

---

## 1. Phala Network 深度分析

### 1.1 项目定位

Phala Network 是基于 Substrate 构建的去中心化隐私云计算协议，核心理念是**将计算从链上剥离到链下安全工作节点（TEE）**，区块链仅负责共识、注册、调度和验证，实际计算在可信执行环境中完成。

### 1.2 核心架构：三组件模型

Phala 的系统由三个核心组件协同运作：

```
┌─────────────────────────────────────────────────────────┐
│                    用户 / DApp                           │
│                       │                                  │
│              ┌────────▼────────┐                         │
│              │   Blockchain    │  ← Substrate 链         │
│              │  (共识 + 注册)   │    记录命令、注册Worker  │
│              └────────┬────────┘    治理、代币、质押       │
│                       │                                  │
│              ┌────────▼────────┐                         │
│              │     pherry      │  ← 中继桥               │
│              │   (数据桥接)     │    链上↔链下消息转发     │
│              └────────┬────────┘    交易验证与同步        │
│                       │                                  │
│              ┌────────▼────────┐                         │
│              │    pRuntime     │  ← TEE 运行时            │
│              │  (机密计算引擎)  │    Intel SGX/TDX 飞地    │
│              │                 │    执行机密合约          │
│              └─────────────────┘                         │
└─────────────────────────────────────────────────────────┘
```

**1) Blockchain（区块链层）**
- 基于 Substrate FRAME 构建
- **不执行**机密计算本身，只记录调用命令和状态摘要
- 运行 Worker 注册表（phala-registry）、计算调度（phala-computation）、质押池（phala-stakepool）、消息队列（phala-mq）等 pallet
- 通过 Remote Attestation 验证 Worker 的 TEE 真实性

**2) pherry（中继桥）**
- 链上与链下的双向消息桥
- 将区块链上的合约调用命令转发给 pRuntime
- 将 pRuntime 的计算结果/状态更新提交回链上
- pRuntime **不信任** pherry，会独立验证每个区块和交易

**3) pRuntime（隐私运行时）**
- 运行在 Intel SGX / TDX 等 TEE 硬件内的安全飞地
- 执行用 ink! 编写的机密智能合约（Phat Contract）
- 数据在飞地内解密计算，外部无法窥探
- 支持 HTTP 请求、跨链调用等链下扩展能力

### 1.3 链上 Pallet 体系

Phala 的 Substrate 链上模块经过多次重构（Issue #208），拆分为职责清晰的 pallet 组：

| Pallet | 职责 |
|--------|------|
| **phala-registry** | Worker/Gatekeeper 注册、TEE 远程证明验证、Worker 生命周期管理 |
| **phala-computation** | 计算任务调度、Worker 分配、算力评分（Performance Score） |
| **phala-stakepool** | 质押池管理、委托质押、奖励分配、佣金设置 |
| **phala-mq** | 链上消息队列，链上↔TEE 的命令/结果通道 |
| **pallet-assets** | 代币资产管理（复用 Substrate 标准模块） |

### 1.4 密钥层级体系

Phala 设计了精密的密钥分层管理：

```
RootKey（根密钥，由 Gatekeeper 集体生成/MPC 分片）
├── WorkerKey（sr25519，每个 Worker 的身份密钥）
├── EcdhKey（用于端到端加密通信，建议定期轮换）
├── ContractKey（每个合约的独立密钥，加密合约状态）
└── Application Keys（派生）
    ├── CA Key（应用证书密钥）
    ├── Disk Encryption Key（磁盘加密）
    ├── Environment Encryption Key（环境变量加密）
    └── ECDSA Key（签名操作）
```

**关键安全特性：**
- **前向/后向保密**：密钥轮换确保历史和未来数据安全
- **MPC 分片**：根密钥通过 Shamir 秘密共享分布到多节点，单节点泄露不影响全局
- **密钥轮换**：支持 RootKey Share 轮换和完整 RootKey 轮换

### 1.5 Gatekeeper 机制

- Gatekeeper 是**最高置信度**的特殊 Worker 节点
- 通过 NPoS 机制在链上选举产生
- 职责：管理根密钥、分发合约密钥、监控 Worker 健康
- 安全措施：端点不公开、不可部署合约、要求更高质押

### 1.6 dstack 平台（新一代零信任架构）

Phala 最新推出的 dstack 是对早期架构的全面升级：

| 组件 | 功能 |
|------|------|
| **dstack-os** | 硬件抽象层 + 最小化 OS，消除不同 TEE 硬件差异，减少攻击面 |
| **dstack-kms** | 区块链控制的密钥管理服务，替代硬件绑定的加密方案 |
| **dstack-gateway** | TEE 控制的域名管理 + 反向代理，自动 TLS 证书 |
| **dstack-ingress** | 自定义域名的灵活方案，支持 TLS 直通 |

**核心创新：**
- **可移植机密容器**：工作负载可在不同 TEE 实例间迁移，不绑定硬件
- **去中心化代码管理**：通过智能合约（KmsAuth + AppAuth）治理应用生命周期
- **可验证域名管理**：Zero Trust TLS 协议，证书完全由 TEE 控制

### 1.7 Phat Contract（机密智能合约）

Phat Contract 是 Phala 的编程模型，超越传统智能合约的能力：

- **完整 ink! 兼容**：使用 Rust + ink! 编写，Wasm 编译
- **CPU 密集计算**：在 TEE 内全速执行，不受链上 gas 限制
- **网络访问**：可发起 HTTP/HTTPS 请求（Oracle、API 调用）
- **跨链互操作**：Phat Offchain Rollup SDK 连接 EVM/Substrate 链
- **机密状态**：合约状态加密存储，仅 TEE 内可解密

### 1.8 经济模型

- **质押池（StakePool）**：Worker 运营者创建，委托者质押 PHA 赚取奖励
- **Vault**：聚合多个质押池的委托机制
- **奖励计算**：基于 Worker 算力评分（Performance Score）× 质押乘数
- **惩罚机制**：Worker 恶意行为（伪造结果）被其他 Worker 检测后，链上 slash 质押

---

## 2. Cosmos 隐私模块总体设计

### 2.1 设计理念

借鉴 Phala 的 **"链上调度 + 链下计算"** 分层架构，但根据 Cosmos 的实际场景做适配简化：

- Cosmos 不需要通用隐私云计算平台，而是需要**面向特定业务的隐私保护能力**
- 聊天端到端加密、占卜隐私保护、交易匿名、用户数据安全是核心诉求
- 采用 **"链上密钥管理 + 链上访问控制 + 链下隐私计算验证"** 的混合方案

### 2.2 模块命名

```
pallets/privacy/            → 隐私计算模块根目录
```

### 2.3 架构概览

```
┌──────────────────────────────────────────────────────────────────┐
│                         Cosmos 隐私计算架构                       │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│   链上层 (Substrate Pallets)                                      │
│   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │
│   │ privacy- │ │ privacy- │ │ privacy- │ │   privacy-       │   │
│   │ common   │ │ keystore │ │ access   │ │   credential     │   │
│   │ 公共类型  │ │ 密钥管理  │ │ 访问控制  │ │   隐私凭证       │   │
│   └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────────────┘   │
│        │            │            │              │                 │
│   ┌────▼────────────▼────────────▼──────────────▼─────────┐      │
│   │              privacy-compute                          │      │
│   │           机密计算调度 & 验证                            │      │
│   └───────────────────────┬───────────────────────────────┘      │
│                           │                                      │
├───────────────────────────┼──────────────────────────────────────┤
│   链下层 (Off-chain)       │                                      │
│                           ▼                                      │
│   ┌─────────────────────────────────────────────────────────┐    │
│   │              Off-chain Worker (TEE / MPC)                │    │
│   │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐ │    │
│   │  │ 加密聊天  │ │ 匿名匹配  │ │ 隐私占卜  │ │ 数据安全   │ │    │
│   │  │ 引擎     │ │ 引擎     │ │ 引擎     │ │ 存储引擎   │ │    │
│   │  └──────────┘ └──────────┘ └──────────┘ └────────────┘ │    │
│   └─────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
```

### 2.4 子模块划分

| 子模块 | Crate 名称 | 路径 | 职责 |
|--------|-----------|------|------|
| **common** | `pallet-privacy-common` | `pallets/privacy/common/` | 公共类型、Trait、错误定义、加密原语接口 |
| **keystore** | `pallet-privacy-keystore` | `pallets/privacy/keystore/` | 链上密钥注册/轮换/派生、Worker 密钥管理 |
| **access** | `pallet-privacy-access` | `pallets/privacy/access/` | 数据访问控制策略、权限授予/撤销 |
| **credential** | `pallet-privacy-credential` | `pallets/privacy/credential/` | 隐私凭证发行/验证、匿名身份、ZK 证明集成 |
| **compute** | `pallet-privacy-compute` | `pallets/privacy/compute/` | 链下计算任务提交/调度、结果验证、Worker 注册 |

---

## 3. Pallet 子模块详细设计

### 3.1 privacy-common

公共基础模块，定义所有隐私子模块共享的类型和接口。

```rust
// pallets/privacy/common/src/lib.rs

/// 加密算法类型
pub enum CryptoScheme {
    X25519,           // ECDH 密钥交换
    Sr25519,          // 签名密钥
    Aes256Gcm,        // 对称加密
    ChaCha20Poly1305, // 流式加密
}

/// 隐私等级
pub enum PrivacyLevel {
    Public,           // 公开
    Protected,        // 受保护（需授权访问）
    Confidential,     // 机密（端到端加密）
    Anonymous,        // 匿名（零知识证明）
}

/// Worker 节点状态（借鉴 Phala phala-registry）
pub enum WorkerStatus {
    Registered,       // 已注册
    Attested,         // 已通过远程证明
    Active,           // 活跃计算中
    Cooling,          // 冷却期
    Retired,          // 已退出
}

/// 机密计算任务
pub struct ComputeTask<AccountId, BlockNumber> {
    pub id: TaskId,
    pub submitter: AccountId,
    pub task_type: TaskType,
    pub encrypted_input: Vec<u8>,
    pub privacy_level: PrivacyLevel,
    pub deadline: BlockNumber,
    pub reward: Balance,
}

/// 任务类型（对应 Cosmos 业务场景）
pub enum TaskType {
    EncryptedChat,         // 加密聊天消息处理
    AnonymousMatching,     // 匿名缘分匹配
    ConfidentialDivination,// 隐私占卜计算
    PrivateTrading,        // 隐私交易撮合
    DataSeal,              // 数据密封存储
}

/// 访问控制策略
pub struct AccessPolicy<AccountId> {
    pub resource_id: ResourceId,
    pub owner: AccountId,
    pub granted_accounts: Vec<AccountId>,
    pub expiry: Option<BlockNumber>,
    pub privacy_level: PrivacyLevel,
}
```

### 3.2 privacy-keystore

借鉴 Phala 的密钥层级体系和 dstack-kms 的设计。

**核心职责：**
- 用户公钥注册与管理
- 端到端加密密钥交换协调
- 密钥轮换与过期管理
- Worker 节点身份密钥注册

```rust
// pallets/privacy/keystore/src/lib.rs

#[pallet::storage]
/// 用户加密公钥注册表 (AccountId => EncryptionPublicKey)
pub type UserKeys<T> = StorageMap<_, Blake2_128Concat, T::AccountId, UserKeyInfo<T>>;

#[pallet::storage]
/// Worker 节点密钥注册表（借鉴 Phala phala-registry 的 WorkerKey）
pub type WorkerKeys<T> = StorageMap<_, Blake2_128Concat, WorkerId, WorkerKeyInfo<T>>;

#[pallet::storage]
/// 密钥轮换记录
pub type KeyRotationLog<T> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<KeyRotation<T>>>;

/// 用户密钥信息
pub struct UserKeyInfo<T: Config> {
    pub encryption_key: [u8; 32],     // X25519 公钥，用于端到端加密
    pub signing_key: [u8; 32],        // Sr25519 公钥，用于签名验证
    pub registered_at: BlockNumberFor<T>,
    pub last_rotated: BlockNumberFor<T>,
    pub key_version: u32,
}

/// Worker 密钥信息（对标 Phala 的 WorkerKey + EcdhKey）
pub struct WorkerKeyInfo<T: Config> {
    pub identity_key: [u8; 32],       // Worker 身份密钥
    pub ecdh_key: [u8; 32],           // ECDH 通信密钥
    pub attestation_report: Vec<u8>,  // TEE 远程证明报告
    pub confidence_level: u8,         // 置信等级 (1-5)
    pub registered_at: BlockNumberFor<T>,
}

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 注册用户加密公钥
    pub fn register_key(origin, encryption_key: [u8; 32], signing_key: [u8; 32]) -> DispatchResult;

    /// 轮换密钥（借鉴 Phala 的 Key Rotation 机制）
    pub fn rotate_key(origin, new_encryption_key: [u8; 32]) -> DispatchResult;

    /// 注册 Worker 节点（借鉴 Phala phala-registry）
    pub fn register_worker(origin, identity_key: [u8; 32], ecdh_key: [u8; 32], attestation: Vec<u8>) -> DispatchResult;

    /// 验证 Worker 远程证明
    pub fn verify_attestation(origin, worker_id: WorkerId, report: Vec<u8>) -> DispatchResult;
}
```

### 3.3 privacy-access

数据访问控制层，管理谁可以访问什么数据，借鉴 Phala 的 AppAuth 合约理念。

**核心职责：**
- 数据资源注册与所有权管理
- 基于策略的访问授权（类似 Phala KmsAuth 的角色）
- 授权记录上链（不可篡改审计轨迹）
- 与 entity/kyc 模块协同

```rust
// pallets/privacy/access/src/lib.rs

#[pallet::storage]
/// 资源访问策略表
pub type Policies<T> = StorageMap<_, Blake2_128Concat, ResourceId, AccessPolicy<T>>;

#[pallet::storage]
/// 授权记录（谁授权谁访问什么）
pub type Grants<T> = StorageDoubleMap<
    _,
    Blake2_128Concat, ResourceId,
    Blake2_128Concat, T::AccountId,
    GrantInfo<T>,
>;

#[pallet::storage]
/// 数据分类标签
pub type DataClassification<T> = StorageMap<_, Blake2_128Concat, ResourceId, PrivacyLevel>;

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 注册隐私数据资源
    pub fn register_resource(origin, resource_id: ResourceId, privacy_level: PrivacyLevel) -> DispatchResult;

    /// 授予访问权限
    pub fn grant_access(origin, resource_id: ResourceId, grantee: T::AccountId, expiry: Option<BlockNumberFor<T>>) -> DispatchResult;

    /// 撤销访问权限
    pub fn revoke_access(origin, resource_id: ResourceId, grantee: T::AccountId) -> DispatchResult;

    /// 批量授权（用于群组场景）
    pub fn batch_grant(origin, resource_id: ResourceId, grantees: Vec<T::AccountId>) -> DispatchResult;

    /// 验证访问权限（可被其他 pallet 调用）
    pub fn check_access(resource_id: ResourceId, accessor: T::AccountId) -> bool;
}
```

### 3.4 privacy-credential

隐私凭证模块，支持匿名身份和可验证声明，用于匿名匹配、隐私交易等场景。

**设计灵感：**
- Phala 的 Zero Trust 理念：不信任任何单一实体
- 可验证凭证（Verifiable Credential）标准
- 零知识证明用于"证明属性而不泄露数据"

```rust
// pallets/privacy/credential/src/lib.rs

/// 凭证类型
pub enum CredentialType {
    AgeRange,          // 年龄范围证明（用于匹配）
    LocationRegion,    // 地区证明（不泄露精确位置）
    IdentityVerified,  // 已验证身份（不泄露身份细节）
    ReputationScore,   // 信誉分范围证明
    MembershipTier,    // 会员等级证明
}

/// 隐私凭证
pub struct PrivacyCredential<T: Config> {
    pub credential_type: CredentialType,
    pub issuer: T::AccountId,          // 颁发者
    pub holder_commitment: [u8; 32],   // 持有者的承诺值（不暴露身份）
    pub proof: Vec<u8>,                // ZK 证明数据
    pub issued_at: BlockNumberFor<T>,
    pub expires_at: Option<BlockNumberFor<T>>,
    pub revoked: bool,
}

#[pallet::storage]
/// 凭证注册表（commitment => credential）
pub type Credentials<T> = StorageMap<_, Blake2_128Concat, [u8; 32], PrivacyCredential<T>>;

#[pallet::storage]
/// 吊销列表
pub type RevocationList<T> = StorageMap<_, Blake2_128Concat, [u8; 32], BlockNumberFor<T>>;

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 颁发隐私凭证（由可信颁发者调用）
    pub fn issue_credential(origin, holder_commitment: [u8; 32], credential_type: CredentialType, proof: Vec<u8>) -> DispatchResult;

    /// 验证凭证（链上验证 ZK 证明）
    pub fn verify_credential(origin, commitment: [u8; 32], proof: Vec<u8>) -> DispatchResult;

    /// 吊销凭证
    pub fn revoke_credential(origin, commitment: [u8; 32]) -> DispatchResult;

    /// 匿名展示凭证（用于匹配等场景，仅证明满足条件）
    pub fn present_credential(origin, commitment: [u8; 32], challenge: [u8; 32], response: Vec<u8>) -> DispatchResult;
}
```

### 3.5 privacy-compute

机密计算调度与验证模块，借鉴 Phala 的 phala-computation + phala-mq 设计。

**核心职责：**
- Worker 节点注册与管理（简化版 phala-registry）
- 计算任务提交与分配
- 计算结果验证（哈希承诺 + 多 Worker 交叉验证）
- 奖惩机制

```rust
// pallets/privacy/compute/src/lib.rs

#[pallet::storage]
/// Worker 注册表
pub type Workers<T> = StorageMap<_, Blake2_128Concat, WorkerId, WorkerInfo<T>>;

#[pallet::storage]
/// 待处理任务队列
pub type TaskQueue<T> = StorageValue<_, Vec<ComputeTask<T>>>;

#[pallet::storage]
/// 任务结果（TaskId => 加密结果）
pub type TaskResults<T> = StorageMap<_, Blake2_128Concat, TaskId, TaskResult<T>>;

#[pallet::storage]
/// Worker 质押（简化版 StakePool）
pub type WorkerStake<T> = StorageMap<_, Blake2_128Concat, WorkerId, BalanceOf<T>>;

/// Worker 信息
pub struct WorkerInfo<T: Config> {
    pub owner: T::AccountId,
    pub status: WorkerStatus,
    pub performance_score: u32,       // 算力评分（借鉴 Phala）
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub stake: BalanceOf<T>,
    pub registered_at: BlockNumberFor<T>,
}

/// 任务结果
pub struct TaskResult<T: Config> {
    pub task_id: TaskId,
    pub worker_id: WorkerId,
    pub encrypted_output: Vec<u8>,
    pub output_hash: [u8; 32],        // 结果哈希（用于验证）
    pub completed_at: BlockNumberFor<T>,
}

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 提交机密计算任务
    pub fn submit_task(origin, task_type: TaskType, encrypted_input: Vec<u8>, privacy_level: PrivacyLevel) -> DispatchResult;

    /// Worker 领取任务
    pub fn claim_task(origin, worker_id: WorkerId, task_id: TaskId) -> DispatchResult;

    /// Worker 提交计算结果
    pub fn submit_result(origin, task_id: TaskId, encrypted_output: Vec<u8>, output_hash: [u8; 32]) -> DispatchResult;

    /// 质疑计算结果（触发多 Worker 验证，借鉴 Phala 的 Slash 机制）
    pub fn challenge_result(origin, task_id: TaskId) -> DispatchResult;

    /// Worker 质押
    pub fn stake(origin, worker_id: WorkerId, amount: BalanceOf<T>) -> DispatchResult;

    /// Worker 解除质押
    pub fn unstake(origin, worker_id: WorkerId, amount: BalanceOf<T>) -> DispatchResult;
}
```

---

## 4. 与现有模块集成方案

### 4.1 Chat 模块集成（端到端加密聊天）

```
用户A                 privacy-keystore              Chat 模块                用户B
  │                       │                            │                      │
  ├─ register_key() ──────▶                            │                      │
  │                       │                            │    ◀── register_key() ┤
  │                       │                            │                      │
  ├─ 查询B的公钥 ─────────▶                            │                      │
  │  ◀── 返回B的X25519公钥 ┤                            │                      │
  │                       │                            │                      │
  ├─ ECDH派生共享密钥 ────│──── send_message(加密) ────▶│                      │
  │                       │                            ├──── 转发加密消息 ─────▶│
  │                       │                            │                      │
  │                       │                            │  B用共享密钥解密 ◀─────┤
```

**集成点：**
- `pallet-chat-core` 调用 `pallet-privacy-keystore` 获取收发双方公钥
- 消息体使用 X25519 + AES-256-GCM 加密后上链
- `pallet-privacy-access` 控制群聊密钥的分发权限

### 4.2 Social/Matchmaking 模块集成（匿名缘分匹配）

```
用户A                 privacy-credential            Matchmaking 模块          用户B
  │                       │                            │                      │
  ├─ issue_credential ────▶  (年龄范围凭证)             │                      │
  ├─ issue_credential ────▶  (地区凭证)                │                      │
  │                       │                            │                      │
  ├─ present_credential ──▶──── 匿名匹配请求 ──────────▶│                      │
  │                       │    (只有ZK证明，不含真实数据) │                      │
  │                       │                            │  ◀── 匿名匹配请求 ───┤
  │                       │                            │                      │
  │                       │    ◀──── 匹配成功通知 ──────┤                      │
  │                       │    (双方同意后才揭示身份)     │                      │
```

**集成点：**
- `pallet-matchmaking-profile` 不再明文存储用户详细信息
- 用 `pallet-privacy-credential` 的 ZK 凭证替代：只证明"25-30岁"而不泄露确切年龄
- 匹配算法在 `pallet-privacy-compute` 的链下 Worker 中运行

### 4.3 Trading 模块集成（隐私交易）

**集成点：**
- `pallet-trading-otc` 的大额 OTC 交易可选隐私模式
- 交易金额和参与方通过 `pallet-privacy-access` 控制可见性
- 交易撮合计算可提交至 `pallet-privacy-compute` 在链下完成

### 4.4 Entity/KYC 模块集成

**集成点：**
- `pallet-entity-kyc` 完成 KYC 后，颁发 `pallet-privacy-credential` 的 IdentityVerified 凭证
- 用户在其他场景出示凭证，无需重复暴露 KYC 材料
- 实现 "一次验证，到处使用" 的隐私友好型身份体系

### 4.5 Storage 模块集成（加密存储）

**集成点：**
- `pallet-storage-service` 存储的文件通过 `pallet-privacy-keystore` 的密钥加密
- `pallet-privacy-access` 管理文件的共享权限
- 密钥轮换时，加密文件可通过 Worker 在 TEE 中重新加密

---

## 5. 实施路线图

### Phase 1：基础设施（4-6 周）

| 任务 | 优先级 | 依赖 |
|------|--------|------|
| 创建 `pallets/privacy/common/` 公共类型定义 | P0 | 无 |
| 实现 `pallets/privacy/keystore/` 密钥注册与管理 | P0 | common |
| 实现 `pallets/privacy/access/` 基础访问控制 | P0 | common |
| 单元测试和集成测试 | P0 | 上述全部 |

**里程碑：** 用户可注册加密公钥，数据可设置隐私级别和访问权限。

### Phase 2：隐私凭证（3-4 周）

| 任务 | 优先级 | 依赖 |
|------|--------|------|
| 实现 `pallets/privacy/credential/` 凭证发行与验证 | P1 | common, keystore |
| 集成简化版 ZK 验证（Groth16 或 Bulletproofs） | P1 | credential |
| 与 `pallet-entity-kyc` 集成 | P1 | credential |
| 与 `pallet-matchmaking-*` 集成 | P1 | credential |

**里程碑：** 匿名匹配功能可用，KYC 一次验证到处使用。

### Phase 3：机密计算（4-6 周）

| 任务 | 优先级 | 依赖 |
|------|--------|------|
| 实现 `pallets/privacy/compute/` 任务调度 | P2 | common, keystore |
| 开发链下 Worker 运行时（可先用 Offchain Worker 原型） | P2 | compute |
| Worker 质押与奖惩机制 | P2 | compute |
| 与 Chat/Trading 模块集成 | P2 | compute, access |

**里程碑：** 端到端加密聊天、隐私交易撮合可用。

### Phase 4：进阶能力（持续迭代）

| 任务 | 优先级 | 依赖 |
|------|--------|------|
| TEE 远程证明验证（Intel SGX/TDX） | P3 | compute |
| MPC 多方密钥生成（借鉴 Phala 的 Shamir 方案） | P3 | keystore |
| 密钥自动轮换策略 | P3 | keystore |
| GPU TEE 支持（AI 隐私推理，对接 Meowstar AI） | P3 | compute |

---

## 附录 A：目录结构

```
pallets/privacy/
├── README.md
├── Cargo.toml               # workspace 虚拟清单
├── common/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs            # 公共类型、Trait、错误
├── keystore/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs            # Pallet 定义
│       ├── types.rs          # 密钥相关类型
│       ├── weights.rs        # 权重
│       └── tests.rs          # 单元测试
├── access/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── policy.rs         # 策略引擎
│       ├── weights.rs
│       └── tests.rs
├── credential/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── verifier.rs       # ZK 证明验证器
│       ├── weights.rs
│       └── tests.rs
└── compute/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── scheduler.rs      # 任务调度器
        ├── weights.rs
        └── tests.rs
```

## 附录 B：Phala vs Cosmos 隐私模块对比

| 维度 | Phala Network | Cosmos 隐私模块 |
|------|--------------|----------------|
| **定位** | 通用隐私云计算平台 | 面向特定业务的隐私保护 |
| **TEE 依赖** | 核心依赖 Intel SGX/TDX | 可选，Phase 1-2 不强制要求 |
| **合约模型** | Phat Contract (ink! + Wasm) | 复用 Substrate Offchain Worker，逐步引入 TEE |
| **密钥管理** | dstack-kms + Gatekeeper 集群 | 简化版 keystore pallet + 未来 MPC |
| **经济模型** | PHA 质押 + StakePool + Vault | COS 质押 + 简化版 Worker 奖励 |
| **跨链** | Polkadot 平行链 + Offchain Rollup | 暂不涉及，专注链内隐私 |
| **ZK 证明** | 不是核心（偏 TEE） | 凭证模块核心使用 ZK |
| **代码管理** | 链上治理（KmsAuth + AppAuth） | 简化版访问控制 pallet |

## 附录 C：关键技术选型

| 组件 | 推荐方案 | 备选方案 |
|------|---------|---------|
| **密钥交换** | X25519 (Curve25519 ECDH) | sr25519 |
| **对称加密** | AES-256-GCM | ChaCha20-Poly1305 |
| **ZK 证明** | Groth16 (snarkjs 兼容) | Bulletproofs (无需可信设置) |
| **哈希** | Blake2b-256 (Substrate 原生) | SHA3-256 |
| **链下计算** | Substrate Offchain Worker (Phase 1) | TEE Worker (Phase 3+) |
| **MPC** | Shamir Secret Sharing (t-of-n) | FROST 签名 |
