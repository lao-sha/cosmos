# TEE 隐私计算方案设计文档

> 版本: 1.2.0
> 日期: 2026-01-06
> 状态: 设计阶段

## 目录

- [1. 概述](#1-概述)
- [2. 模块独立性设计](#2-模块独立性设计)
- [3. 架构设计](#3-架构设计)
- [4. TEE 选型](#4-tee-选型)
- [5. 核心模块设计](#5-核心模块设计)
- [6. 数据流程](#6-数据流程)
- [7. API 设计](#7-api-设计)
- [8. 安全机制](#8-安全机制)
- [9. 部署方案](#9-部署方案)
- [10. 测试计划](#10-测试计划)
- [11. 经济激励机制](#11-经济激励机制)
- [12. 故障转移机制](#12-故障转移机制)
- [13. DCAP 认证支持](#13-dcap-认证支持)
- [14. 批处理优化](#14-批处理优化)
- [15. Enclave 升级机制](#15-enclave-升级机制)
- [16. ARM TrustZone 支持](#16-arm-trustzone-支持)
- [17. 审计日志系统](#17-审计日志系统)
- [18. 性能基准测试](#18-性能基准测试)

---

## 1. 概述

### 1.1 背景

Stardust 占卜系统采用**双轨隐私架构**：

1. **纯密码学方案** (`pallet-divination-privacy`) - 已实现，基于 X25519 + XChaCha20-Poly1305
2. **TEE 方案** (`pallet-tee-privacy`) - 本文档描述，基于可信执行环境

两种方案**相互独立**，可单独部署或同时使用，为不同安全需求的用户提供选择。

### 1.2 目标

| 目标 | 描述 |
|------|------|
| 隐私计算 | 在 TEE 内执行占卜计算，节点无法获取明文数据 |
| 密钥安全 | 密钥在 TEE 内生成和管理，永不暴露 |
| 可验证性 | 提供远程认证，证明计算在真实 TEE 内执行 |
| 数据主权 | 用户完全控制数据访问权限 |
| **模块独立** | 与纯密码学方案完全解耦，可独立部署 |

### 1.3 范围

本方案涵盖以下占卜模块的 TEE 隐私计算支持：

- `pallet-bazi` - 八字命理
- `pallet-meihua` - 梅花易数
- `pallet-qimen` - 奇门遁甲
- `pallet-liuyao` - 六爻占卜
- `pallet-ziwei` - 紫微斗数
- `pallet-tarot` - 塔罗占卜

---

## 2. 模块独立性设计

### 2.1 双轨隐私架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Stardust 双轨隐私架构                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│                    ┌─────────────────────────┐                          │
│                    │   divination-common     │                          │
│                    │   (公共类型/Trait)      │                          │
│                    │                         │                          │
│                    │  • PrivacyMode 枚举     │                          │
│                    │  • PrivacyProvider Trait│                          │
│                    │  • DivinationType       │                          │
│                    └────────────┬────────────┘                          │
│                                 │                                       │
│              ┌──────────────────┴──────────────────┐                    │
│              │                                     │                    │
│              ▼                                     ▼                    │
│   ┌─────────────────────────┐       ┌─────────────────────────┐        │
│   │  pallet-divination-     │       │   pallet-tee-privacy    │        │
│   │  privacy                │       │                         │        │
│   │  (纯密码学方案)          │       │   (TEE 方案)            │        │
│   │                         │       │                         │        │
│   │  ✓ X25519 密钥交换      │       │  ✓ Intel SGX / ARM TZ   │        │
│   │  ✓ XChaCha20-Poly1305   │       │  ✓ 远程认证             │        │
│   │  ✓ 前端加密/解密        │       │  ✓ Enclave 内计算       │        │
│   │  ✓ 授权管理             │       │  ✓ 密钥永不出 Enclave   │        │
│   │  ✓ 无硬件依赖           │       │  ✓ 需要 TEE 节点        │        │
│   │                         │       │                         │        │
│   │  状态: ✅ 已实现         │       │  状态: 📝 设计中        │        │
│   └─────────────────────────┘       └─────────────────────────┘        │
│              │                                     │                    │
│              │         相互独立，无依赖            │                    │
│              │                                     │                    │
│              └──────────────────┬──────────────────┘                    │
│                                 │                                       │
│                                 ▼                                       │
│                    ┌─────────────────────────┐                          │
│                    │        Runtime          │                          │
│                    │      (按需集成)         │                          │
│                    └─────────────────────────┘                          │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 两种方案对比

| 特性 | pallet-divination-privacy | pallet-tee-privacy |
|------|---------------------------|-------------------|
| **加密方式** | X25519 + XChaCha20-Poly1305 | TEE Enclave 内加密 |
| **计算位置** | 前端 / Runtime API | TEE Enclave |
| **密钥管理** | 用户本地存储 | Enclave 内管理 |
| **硬件要求** | 无 | Intel SGX / ARM TrustZone |
| **可验证性** | 无 | 远程认证 (Attestation) |
| **部署成本** | 低 | 高 |
| **可用性** | 100% 用户可用 | 依赖 TEE 节点 |
| **适用场景** | 日常隐私保护 | 高安全需求 |
| **状态** | ✅ 已实现 | 📝 设计中 |

### 2.3 独立部署场景

| 部署场景 | divination-privacy | tee-privacy | 说明 |
|---------|-------------------|-------------|------|
| **初期上线** | ✅ 启用 | ❌ 不部署 | 快速上线，无硬件依赖 |
| **TEE 就绪后** | ✅ 保留 | ✅ 启用 | 两者并存，用户选择 |
| **纯 TEE 环境** | ❌ 可移除 | ✅ 启用 | 企业私有链场景 |
| **降级模式** | ✅ 启用 | ⚠️ 故障 | TEE 不可用时自动降级 |

### 2.4 公共接口设计

两个模块通过 `divination-common` 中定义的公共 Trait 实现统一接口：

```rust
// pallets/divination/common/src/privacy.rs

/// 隐私模式枚举（两种方案共用）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum PrivacyMode {
    /// 公开模式 - 所有数据明文
    Public = 0,
    /// 部分加密 - 计算数据明文，敏感数据加密
    Partial = 1,
    /// 完全加密 - 所有数据加密
    Private = 2,
}

/// 隐私提供者 Trait（抽象接口）
/// 两种方案都实现此 Trait，占卜模块通过此接口调用
pub trait PrivacyProvider<AccountId, BlockNumber> {
    /// 存储加密数据
    fn store_encrypted(
        divination_type: DivinationType,
        result_id: u64,
        owner: &AccountId,
        encrypted_data: EncryptedData,
        mode: PrivacyMode,
    ) -> DispatchResult;

    /// 授权访问
    fn grant_access(
        divination_type: DivinationType,
        result_id: u64,
        owner: &AccountId,
        grantee: &AccountId,
        encrypted_key: Vec<u8>,
        role: AccessRole,
        expires_at: Option<BlockNumber>,
    ) -> DispatchResult;

    /// 撤销授权
    fn revoke_access(
        divination_type: DivinationType,
        result_id: u64,
        owner: &AccountId,
        grantee: &AccountId,
    ) -> DispatchResult;

    /// 检查访问权限
    fn check_access(
        divination_type: DivinationType,
        result_id: u64,
        accessor: &AccountId,
    ) -> bool;
}
```

### 2.5 Runtime 集成配置

```rust
// runtime/src/lib.rs

// ==================== 方案 1：只用纯密码学 ====================
impl pallet_meihua::Config for Runtime {
    type PrivacyProvider = pallet_divination_privacy::Pallet<Runtime>;
    // ...
}

// ==================== 方案 2：只用 TEE ====================
impl pallet_meihua::Config for Runtime {
    type PrivacyProvider = pallet_tee_privacy::Pallet<Runtime>;
    // ...
}

// ==================== 方案 3：混合使用（推荐） ====================
/// 混合隐私适配器 - 根据用户选择路由到不同方案
pub struct HybridPrivacyAdapter<T>(PhantomData<T>);

impl<T: Config> PrivacyProvider<T::AccountId, T::BlockNumber> for HybridPrivacyAdapter<T>
where
    T: pallet_divination_privacy::Config + pallet_tee_privacy::Config,
{
    fn store_encrypted(
        divination_type: DivinationType,
        result_id: u64,
        owner: &T::AccountId,
        encrypted_data: EncryptedData,
        mode: PrivacyMode,
    ) -> DispatchResult {
        // 根据加密数据中的标记判断使用哪种方案
        if encrypted_data.is_tee_encrypted() {
            pallet_tee_privacy::Pallet::<T>::store_encrypted(...)
        } else {
            pallet_divination_privacy::Pallet::<T>::store_encrypted(...)
        }
    }
    // ...
}

impl pallet_meihua::Config for Runtime {
    type PrivacyProvider = HybridPrivacyAdapter<Runtime>;
    // ...
}
```

### 2.6 前端统一接口

```typescript
// frontend/src/lib/privacy/index.ts

/**
 * 统一隐私服务接口
 */
interface PrivacyService {
  encrypt(data: string, recipients: string[]): Promise<EncryptedData>;
  decrypt(encrypted: EncryptedData, privateKey?: Uint8Array): Promise<string>;
  grantAccess(params: GrantAccessParams): Promise<void>;
}

/**
 * 纯密码学实现
 */
class CryptoPrivacyService implements PrivacyService {
  async encrypt(data: string, recipients: string[]) {
    // 前端 X25519 + XChaCha20 加密
    return CryptoEncryption.encryptForMultipleRecipients(data, recipients);
  }
  
  async decrypt(encrypted: EncryptedData, privateKey: Uint8Array) {
    return CryptoEncryption.decrypt(encrypted, privateKey);
  }
}

/**
 * TEE 实现
 */
class TeePrivacyService implements PrivacyService {
  async encrypt(data: string, recipients: string[]) {
    // 提交到 TEE 节点加密
    return api.tx.teePrivacy.submitEncryptRequest(data, recipients);
  }
  
  async decrypt(encrypted: EncryptedData) {
    // 请求 TEE 节点解密
    return api.call.teePrivacyApi.decrypt(encrypted);
  }
}

/**
 * 工厂函数：根据用户选择和系统状态返回对应服务
 */
export async function getPrivacyService(
  preferTee: boolean = false
): Promise<PrivacyService> {
  if (preferTee) {
    // 检查是否有可用的 TEE 节点
    const teeAvailable = await checkTeeNodesAvailable();
    if (teeAvailable) {
      return new TeePrivacyService();
    }
    console.warn('TEE 节点不可用，降级到纯密码学方案');
  }
  return new CryptoPrivacyService();
}
```

---

## 3. 架构设计

### 3.1 模块结构

```
pallets/
├── tee-privacy/                    # TEE 隐私计算核心模块
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # Pallet 主入口
│       ├── types.rs                # 类型定义
│       ├── traits.rs               # Trait 接口
│       ├── enclave/                # Enclave 相关
│       │   ├── mod.rs
│       │   ├── attestation.rs      # 远程认证
│       │   ├── keys.rs             # 密钥管理
│       │   └── crypto.rs           # 加密操作
│       ├── ocw/                    # Off-chain Worker
│       │   ├── mod.rs
│       │   └── processor.rs        # 请求处理器
│       └── benchmarking.rs
│
├── tee-enclave/                    # Enclave 应用 (独立编译)
│   ├── Cargo.toml
│   ├── enclave.edl                 # Enclave 接口定义
│   └── src/
│       ├── lib.rs
│       ├── ecalls.rs               # Enclave 调用入口
│       ├── crypto.rs               # 密码学实现
│       ├── divination/             # 占卜计算
│       │   ├── mod.rs
│       │   ├── bazi.rs
│       │   ├── meihua.rs
│       │   ├── qimen.rs
│       │   ├── liuyao.rs
│       │   └── ziwei.rs
│       └── attestation.rs          # 认证生成
│
└── divination/                     # 现有占卜模块 (需改造)
    ├── bazi/
    ├── meihua/
    ├── qimen/
    └── ...
```

---

## 4. TEE 选型

### 4.1 支持的 TEE 平台

| 平台 | 优先级 | 说明 |
|------|--------|------|
| Intel SGX | P0 | 主要支持平台，生态成熟 |
| ARM TrustZone | P1 | 移动端和边缘节点支持 |
| AMD SEV | P2 | 云环境支持 |
| RISC-V Keystone | P3 | 未来扩展 |

### 4.2 Intel SGX 技术栈

```
┌─────────────────────────────────────┐
│         Rust SGX SDK                │
│  (Apache Teaclave SGX SDK)          │
├─────────────────────────────────────┤
│         Intel SGX SDK               │
├─────────────────────────────────────┤
│         SGX Driver                  │
├─────────────────────────────────────┤
│     Intel CPU (SGX Enabled)         │
└─────────────────────────────────────┘
```

### 4.3 开发依赖

```toml
# pallets/tee-enclave/Cargo.toml

[dependencies]
sgx_tstd = { git = "https://github.com/apache/incubator-teaclave-sgx-sdk" }
sgx_tcrypto = { git = "https://github.com/apache/incubator-teaclave-sgx-sdk" }
sgx_tse = { git = "https://github.com/apache/incubator-teaclave-sgx-sdk" }
sgx_rand = { git = "https://github.com/apache/incubator-teaclave-sgx-sdk" }

# 密码学
ring = { version = "0.16", default-features = false }
x25519-dalek = { version = "2.0", default-features = false }
aes-gcm = { version = "0.10", default-features = false }

# 序列化
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde_json = { version = "1.0", default-features = false }
```

---

## 5. 核心模块设计

### 5.1 类型定义

```rust
// pallets/tee-privacy/src/types.rs

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use scale_info::TypeInfo;

/// TEE 节点信息
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TeeNode<AccountId> {
    /// 节点账户
    pub account: AccountId,
    /// Enclave 公钥
    pub enclave_pubkey: [u8; 32],
    /// 远程认证报告
    pub attestation: TeeAttestation,
    /// 注册时间
    pub registered_at: u64,
    /// 状态
    pub status: TeeNodeStatus,
}

/// TEE 节点状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum TeeNodeStatus {
    /// 待验证
    Pending,
    /// 活跃
    Active,
    /// 暂停
    Suspended,
    /// 已注销
    Deregistered,
}

/// 远程认证报告
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TeeAttestation {
    /// TEE 类型
    pub tee_type: TeeType,
    /// MRENCLAVE (Enclave 度量值)
    pub mr_enclave: [u8; 32],
    /// MRSIGNER (签名者度量值)
    pub mr_signer: [u8; 32],
    /// ISV Product ID
    pub isv_prod_id: u16,
    /// ISV SVN (安全版本号)
    pub isv_svn: u16,
    /// 报告数据
    pub report_data: [u8; 64],
    /// IAS 签名 (Intel Attestation Service)
    pub ias_signature: BoundedVec<u8, ConstU32<512>>,
    /// 认证时间
    pub timestamp: u64,
}

/// TEE 类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum TeeType {
    IntelSgx,
    ArmTrustZone,
    AmdSev,
    RiscVKeystone,
}

/// 隐私计算请求
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct TeeComputeRequest<AccountId, BlockNumber> {
    /// 请求 ID
    pub id: u64,
    /// 请求者
    pub requester: AccountId,
    /// 计算类型
    pub compute_type: ComputeType,
    /// 加密输入数据
    pub encrypted_input: EncryptedData,
    /// 指定的 TEE 节点 (可选)
    pub assigned_node: Option<AccountId>,
    /// 创建区块
    pub created_at: BlockNumber,
    /// 超时区块
    pub timeout_at: BlockNumber,
    /// 状态
    pub status: RequestStatus,
}

/// 计算类型
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum ComputeType {
    /// 八字计算
    BaZi(BaZiParams),
    /// 梅花易数
    MeiHua(MeiHuaParams),
    /// 奇门遁甲
    QiMen(QiMenParams),
    /// 六爻
    LiuYao(LiuYaoParams),
    /// 紫微斗数
    ZiWei(ZiWeiParams),
    /// 塔罗
    Tarot(TarotParams),
}

/// 八字参数
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct BaZiParams {
    /// 出生年
    pub year: u16,
    /// 出生月
    pub month: u8,
    /// 出生日
    pub day: u8,
    /// 出生时辰
    pub hour: u8,
    /// 性别
    pub gender: Gender,
    /// 经度 (用于真太阳时)
    pub longitude: Option<i32>,
}

/// 梅花易数参数
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct MeiHuaParams {
    /// 起卦方式
    pub method: MeiHuaMethod,
    /// 起卦数据
    pub data: BoundedVec<u8, ConstU32<64>>,
    /// 时间戳
    pub timestamp: u64,
}

/// 奇门遁甲参数
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct QiMenParams {
    /// 排盘时间
    pub datetime: u64,
    /// 排盘类型
    pub pan_type: QiMenPanType,
    /// 排盘方法
    pub method: QiMenMethod,
    /// 问事类型
    pub question_type: Option<QuestionType>,
}

/// 加密数据
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct EncryptedData {
    /// 密文
    pub ciphertext: BoundedVec<u8, ConstU32<65536>>,
    /// 临时公钥 (ECDH)
    pub ephemeral_pubkey: [u8; 32],
    /// Nonce
    pub nonce: [u8; 12],
    /// 认证标签
    pub auth_tag: [u8; 16],
}

/// 计算结果
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct TeeComputeResult<AccountId> {
    /// 请求 ID
    pub request_id: u64,
    /// 执行节点
    pub executor: AccountId,
    /// 加密结果
    pub encrypted_output: EncryptedData,
    /// 计算证明
    pub computation_proof: ComputationProof,
    /// 完成时间
    pub completed_at: u64,
}

/// 计算证明
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ComputationProof {
    /// 输入数据哈希
    pub input_hash: [u8; 32],
    /// 输出数据哈希
    pub output_hash: [u8; 32],
    /// Enclave 签名
    pub enclave_signature: [u8; 64],
    /// 时间戳
    pub timestamp: u64,
}

/// 请求状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum RequestStatus {
    /// 待处理
    Pending,
    /// 处理中
    Processing,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 已超时
    Timeout,
}
```

### 5.2 Pallet 接口

```rust
// pallets/tee-privacy/src/lib.rs

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 认证验证器
        type AttestationVerifier: AttestationVerifier;

        /// 时间提供者
        type TimeProvider: UnixTime;

        /// 最大请求数
        #[pallet::constant]
        type MaxPendingRequests: Get<u32>;

        /// 请求超时区块数
        #[pallet::constant]
        type RequestTimeout: Get<Self::BlockNumber>;

        /// 认证有效期 (秒)
        #[pallet::constant]
        type AttestationValidity: Get<u64>;

        /// 权重信息
        type WeightInfo: WeightInfo;
    }

    // ==================== 存储 ====================

    /// 下一个请求 ID
    #[pallet::storage]
    #[pallet::getter(fn next_request_id)]
    pub type NextRequestId<T> = StorageValue<_, u64, ValueQuery>;

    /// TEE 节点注册表
    #[pallet::storage]
    #[pallet::getter(fn tee_nodes)]
    pub type TeeNodes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        TeeNode<T::AccountId>,
        OptionQuery,
    >;

    /// 活跃 TEE 节点列表
    #[pallet::storage]
    #[pallet::getter(fn active_nodes)]
    pub type ActiveNodes<T: Config> = StorageValue<
        _,
        BoundedVec<T::AccountId, ConstU32<100>>,
        ValueQuery,
    >;

    /// 待处理请求
    #[pallet::storage]
    #[pallet::getter(fn pending_requests)]
    pub type PendingRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        TeeComputeRequest<T::AccountId, T::BlockNumber>,
        OptionQuery,
    >;

    /// 计算结果
    #[pallet::storage]
    #[pallet::getter(fn compute_results)]
    pub type ComputeResults<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        TeeComputeResult<T::AccountId>,
        OptionQuery,
    >;

    /// 用户请求索引
    #[pallet::storage]
    #[pallet::getter(fn user_requests)]
    pub type UserRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<1000>>,
        ValueQuery,
    >;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// TEE 节点已注册
        TeeNodeRegistered {
            node: T::AccountId,
            enclave_pubkey: [u8; 32],
        },
        /// TEE 节点已注销
        TeeNodeDeregistered {
            node: T::AccountId,
        },
        /// 认证已更新
        AttestationUpdated {
            node: T::AccountId,
            timestamp: u64,
        },
        /// 计算请求已提交
        ComputeRequestSubmitted {
            request_id: u64,
            requester: T::AccountId,
            compute_type: ComputeType,
        },
        /// 计算已完成
        ComputeCompleted {
            request_id: u64,
            executor: T::AccountId,
        },
        /// 计算失败
        ComputeFailed {
            request_id: u64,
            reason: FailureReason,
        },
        /// 请求超时
        RequestTimeout {
            request_id: u64,
        },
    }

    // ==================== 错误 ====================

    #[pallet::error]
    pub enum Error<T> {
        /// 节点已注册
        NodeAlreadyRegistered,
        /// 节点未注册
        NodeNotRegistered,
        /// 认证无效
        InvalidAttestation,
        /// 认证已过期
        AttestationExpired,
        /// 请求不存在
        RequestNotFound,
        /// 请求已处理
        RequestAlreadyProcessed,
        /// 无权限
        Unauthorized,
        /// 超过最大请求数
        TooManyRequests,
        /// 无可用节点
        NoAvailableNodes,
        /// 证明验证失败
        ProofVerificationFailed,
        /// 签名无效
        InvalidSignature,
        /// 数据太大
        DataTooLarge,
    }

    // ==================== 调用 ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 注册 TEE 节点
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_tee_node())]
        pub fn register_tee_node(
            origin: OriginFor<T>,
            enclave_pubkey: [u8; 32],
            attestation: TeeAttestation,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                !TeeNodes::<T>::contains_key(&who),
                Error::<T>::NodeAlreadyRegistered
            );

            // 验证远程认证
            T::AttestationVerifier::verify(&attestation)
                .map_err(|_| Error::<T>::InvalidAttestation)?;

            // 检查认证是否过期
            let now = T::TimeProvider::now().as_secs();
            ensure!(
                now.saturating_sub(attestation.timestamp) < T::AttestationValidity::get(),
                Error::<T>::AttestationExpired
            );

            let node = TeeNode {
                account: who.clone(),
                enclave_pubkey,
                attestation,
                registered_at: now,
                status: TeeNodeStatus::Active,
            };

            TeeNodes::<T>::insert(&who, node);

            // 添加到活跃节点列表
            ActiveNodes::<T>::try_mutate(|nodes| {
                nodes.try_push(who.clone())
            }).map_err(|_| Error::<T>::TooManyRequests)?;

            Self::deposit_event(Event::TeeNodeRegistered {
                node: who,
                enclave_pubkey,
            });

            Ok(())
        }

        /// 更新认证报告
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::update_attestation())]
        pub fn update_attestation(
            origin: OriginFor<T>,
            attestation: TeeAttestation,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            TeeNodes::<T>::try_mutate(&who, |maybe_node| {
                let node = maybe_node.as_mut().ok_or(Error::<T>::NodeNotRegistered)?;

                // 验证新认证
                T::AttestationVerifier::verify(&attestation)
                    .map_err(|_| Error::<T>::InvalidAttestation)?;

                node.attestation = attestation.clone();

                Self::deposit_event(Event::AttestationUpdated {
                    node: who.clone(),
                    timestamp: attestation.timestamp,
                });

                Ok(())
            })
        }

        /// 提交计算请求
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::submit_compute_request())]
        pub fn submit_compute_request(
            origin: OriginFor<T>,
            compute_type: ComputeType,
            encrypted_input: EncryptedData,
            assigned_node: Option<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查是否有可用节点
            let active_nodes = ActiveNodes::<T>::get();
            ensure!(!active_nodes.is_empty(), Error::<T>::NoAvailableNodes);

            // 如果指定了节点，验证其状态
            if let Some(ref node) = assigned_node {
                let tee_node = TeeNodes::<T>::get(node)
                    .ok_or(Error::<T>::NodeNotRegistered)?;
                ensure!(
                    tee_node.status == TeeNodeStatus::Active,
                    Error::<T>::NodeNotRegistered
                );
            }

            let request_id = NextRequestId::<T>::mutate(|id| {
                let current = *id;
                *id = id.saturating_add(1);
                current
            });

            let current_block = <frame_system::Pallet<T>>::block_number();
            let timeout_at = current_block.saturating_add(T::RequestTimeout::get());

            let request = TeeComputeRequest {
                id: request_id,
                requester: who.clone(),
                compute_type: compute_type.clone(),
                encrypted_input,
                assigned_node,
                created_at: current_block,
                timeout_at,
                status: RequestStatus::Pending,
            };

            PendingRequests::<T>::insert(request_id, request);

            // 更新用户请求索引
            UserRequests::<T>::try_mutate(&who, |requests| {
                requests.try_push(request_id)
            }).map_err(|_| Error::<T>::TooManyRequests)?;

            Self::deposit_event(Event::ComputeRequestSubmitted {
                request_id,
                requester: who,
                compute_type,
            });

            Ok(())
        }

        /// 提交计算结果 (仅 TEE 节点可调用)
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::submit_compute_result())]
        pub fn submit_compute_result(
            origin: OriginFor<T>,
            request_id: u64,
            encrypted_output: EncryptedData,
            computation_proof: ComputationProof,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证调用者是注册的 TEE 节点
            let tee_node = TeeNodes::<T>::get(&who)
                .ok_or(Error::<T>::NodeNotRegistered)?;
            ensure!(
                tee_node.status == TeeNodeStatus::Active,
                Error::<T>::Unauthorized
            );

            // 获取并验证请求
            let mut request = PendingRequests::<T>::get(request_id)
                .ok_or(Error::<T>::RequestNotFound)?;
            ensure!(
                request.status == RequestStatus::Pending ||
                request.status == RequestStatus::Processing,
                Error::<T>::RequestAlreadyProcessed
            );

            // 如果指定了节点，验证是否匹配
            if let Some(ref assigned) = request.assigned_node {
                ensure!(assigned == &who, Error::<T>::Unauthorized);
            }

            // 验证计算证明
            Self::verify_computation_proof(
                &computation_proof,
                &tee_node.enclave_pubkey,
            )?;

            let now = T::TimeProvider::now().as_secs();

            let result = TeeComputeResult {
                request_id,
                executor: who.clone(),
                encrypted_output,
                computation_proof,
                completed_at: now,
            };

            // 更新请求状态
            request.status = RequestStatus::Completed;
            PendingRequests::<T>::insert(request_id, request);

            // 存储结果
            ComputeResults::<T>::insert(request_id, result);

            Self::deposit_event(Event::ComputeCompleted {
                request_id,
                executor: who,
            });

            Ok(())
        }

        /// 注销 TEE 节点
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::deregister_tee_node())]
        pub fn deregister_tee_node(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            TeeNodes::<T>::try_mutate(&who, |maybe_node| {
                let node = maybe_node.as_mut().ok_or(Error::<T>::NodeNotRegistered)?;
                node.status = TeeNodeStatus::Deregistered;
                Ok::<(), Error<T>>(())
            })?;

            // 从活跃列表移除
            ActiveNodes::<T>::mutate(|nodes| {
                nodes.retain(|n| n != &who);
            });

            Self::deposit_event(Event::TeeNodeDeregistered { node: who });

            Ok(())
        }
    }

    // ==================== Hooks ====================

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 每个区块检查超时请求
        fn on_initialize(now: T::BlockNumber) -> Weight {
            let mut weight = Weight::zero();

            // 检查并处理超时请求
            for (request_id, mut request) in PendingRequests::<T>::iter() {
                if request.status == RequestStatus::Pending && now >= request.timeout_at {
                    request.status = RequestStatus::Timeout;
                    PendingRequests::<T>::insert(request_id, request);

                    Self::deposit_event(Event::RequestTimeout { request_id });

                    weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                }
            }

            weight
        }

        /// Off-chain Worker 入口
        fn offchain_worker(block_number: T::BlockNumber) {
            // TEE 节点在此处理待处理请求
            if let Err(e) = Self::process_pending_requests() {
                log::error!("TEE offchain worker error: {:?}", e);
            }
        }
    }

    // ==================== 内部函数 ====================

    impl<T: Config> Pallet<T> {
        /// 验证计算证明
        fn verify_computation_proof(
            proof: &ComputationProof,
            enclave_pubkey: &[u8; 32],
        ) -> DispatchResult {
            // 构造待验证消息
            let mut message = Vec::new();
            message.extend_from_slice(&proof.input_hash);
            message.extend_from_slice(&proof.output_hash);
            message.extend_from_slice(&proof.timestamp.to_le_bytes());

            // 验证 Enclave 签名
            let valid = Self::verify_ed25519_signature(
                &message,
                &proof.enclave_signature,
                enclave_pubkey,
            );

            ensure!(valid, Error::<T>::ProofVerificationFailed);

            Ok(())
        }

        /// 验证 Ed25519 签名
        fn verify_ed25519_signature(
            message: &[u8],
            signature: &[u8; 64],
            public_key: &[u8; 32],
        ) -> bool {
            // 使用 sp_core 的签名验证
            use sp_core::ed25519::{Public, Signature};

            let public = Public::from_raw(*public_key);
            let sig = Signature::from_raw(*signature);

            sp_io::crypto::ed25519_verify(&sig, message, &public)
        }

        /// 处理待处理请求 (Off-chain Worker)
        #[cfg(feature = "std")]
        fn process_pending_requests() -> Result<(), &'static str> {
            use sp_runtime::offchain::storage::StorageValueRef;

            // 检查本节点是否是 TEE 节点
            let node_key = StorageValueRef::persistent(b"tee::node_key");
            let enclave_key = match node_key.get::<[u8; 32]>() {
                Ok(Some(key)) => key,
                _ => return Ok(()), // 非 TEE 节点，跳过
            };

            // 获取待处理请求
            for (request_id, request) in PendingRequests::<T>::iter() {
                if request.status != RequestStatus::Pending {
                    continue;
                }

                // 在 Enclave 内处理请求
                match Self::process_in_enclave(&request, &enclave_key) {
                    Ok((encrypted_output, proof)) => {
                        // 提交结果到链上
                        Self::submit_result_unsigned(request_id, encrypted_output, proof)?;
                    }
                    Err(e) => {
                        log::error!("Enclave processing failed: {:?}", e);
                    }
                }
            }

            Ok(())
        }

        /// 在 Enclave 内处理请求
        #[cfg(feature = "tee-enclave")]
        fn process_in_enclave(
            request: &TeeComputeRequest<T::AccountId, T::BlockNumber>,
            enclave_key: &[u8; 32],
        ) -> Result<(EncryptedData, ComputationProof), &'static str> {
            // 调用 Enclave 处理
            // 实际实现在 tee-enclave crate 中
            tee_enclave::process_request(request, enclave_key)
        }
    }
}
```

### 5.3 Enclave 实现

```rust
// pallets/tee-enclave/src/lib.rs

#![no_std]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_tstd as std;

use std::vec::Vec;
use sgx_tcrypto::*;
use sgx_tse::*;

mod crypto;
mod divination;
mod attestation;

pub use crypto::*;
pub use divination::*;
pub use attestation::*;

/// Enclave 密钥管理器
pub struct EnclaveKeyManager {
    /// Sealing 密钥 (用于持久化)
    sealing_key: [u8; 16],
    /// 签名密钥对
    signing_keypair: Ed25519KeyPair,
    /// ECDH 密钥对
    ecdh_keypair: X25519KeyPair,
}

impl EnclaveKeyManager {
    /// 初始化密钥管理器
    pub fn init() -> SgxResult<Self> {
        // 从 SGX Sealing 派生密钥
        let sealing_key = Self::derive_sealing_key()?;

        // 生成签名密钥对
        let signing_keypair = Ed25519KeyPair::generate()?;

        // 生成 ECDH 密钥对
        let ecdh_keypair = X25519KeyPair::generate()?;

        Ok(Self {
            sealing_key,
            signing_keypair,
            ecdh_keypair,
        })
    }

    /// 派生 Sealing 密钥
    fn derive_sealing_key() -> SgxResult<[u8; 16]> {
        let key_policy = SGX_KEYPOLICY_MRENCLAVE;
        let attribute_mask = sgx_attributes_t {
            flags: TSEAL_DEFAULT_FLAGSMASK,
            xfrm: 0,
        };

        let key_request = sgx_key_request_t {
            key_name: SGX_KEYSELECT_SEAL,
            key_policy,
            isv_svn: 0,
            reserved1: 0,
            cpu_svn: sgx_cpu_svn_t::default(),
            attribute_mask,
            key_id: sgx_key_id_t::default(),
            misc_mask: TSEAL_DEFAULT_MISCMASK,
            config_svn: 0,
            reserved2: [0; 434],
        };

        let mut key = [0u8; 16];
        unsafe {
            sgx_get_key(&key_request, &mut key)?;
        }

        Ok(key)
    }

    /// 获取 Enclave 公钥
    pub fn get_public_key(&self) -> [u8; 32] {
        self.ecdh_keypair.public_key()
    }

    /// ECDH 密钥协商
    pub fn ecdh_shared_secret(&self, peer_pubkey: &[u8; 32]) -> SgxResult<[u8; 32]> {
        self.ecdh_keypair.shared_secret(peer_pubkey)
    }

    /// 签名数据
    pub fn sign(&self, data: &[u8]) -> SgxResult<[u8; 64]> {
        self.signing_keypair.sign(data)
    }
}

/// 处理计算请求
pub fn process_request(
    encrypted_input: &EncryptedData,
    compute_type: &ComputeType,
    key_manager: &EnclaveKeyManager,
) -> SgxResult<(EncryptedData, ComputationProof)> {
    // 1. 解密输入数据
    let plaintext = crypto::decrypt_input(
        encrypted_input,
        key_manager,
    )?;

    // 2. 计算输入哈希
    let input_hash = crypto::sha256(&plaintext);

    // 3. 执行占卜计算
    let result = match compute_type {
        ComputeType::BaZi(params) => {
            divination::bazi::calculate(&plaintext, params)?
        }
        ComputeType::MeiHua(params) => {
            divination::meihua::calculate(&plaintext, params)?
        }
        ComputeType::QiMen(params) => {
            divination::qimen::calculate(&plaintext, params)?
        }
        ComputeType::LiuYao(params) => {
            divination::liuyao::calculate(&plaintext, params)?
        }
        ComputeType::ZiWei(params) => {
            divination::ziwei::calculate(&plaintext, params)?
        }
        ComputeType::Tarot(params) => {
            divination::tarot::calculate(&plaintext, params)?
        }
    };

    // 4. 序列化结果
    let result_bytes = result.encode();

    // 5. 计算输出哈希
    let output_hash = crypto::sha256(&result_bytes);

    // 6. 加密结果
    let encrypted_output = crypto::encrypt_output(
        &result_bytes,
        &encrypted_input.ephemeral_pubkey,
        key_manager,
    )?;

    // 7. 生成计算证明
    let timestamp = get_trusted_time()?;
    let proof_data = [
        input_hash.as_slice(),
        output_hash.as_slice(),
        &timestamp.to_le_bytes(),
    ].concat();

    let signature = key_manager.sign(&proof_data)?;

    let proof = ComputationProof {
        input_hash,
        output_hash,
        enclave_signature: signature,
        timestamp,
    };

    Ok((encrypted_output, proof))
}
```

### 5.4 加密模块

```rust
// pallets/tee-enclave/src/crypto.rs

use sgx_tcrypto::*;
use std::vec::Vec;

/// AES-256-GCM 加密
pub fn aes_gcm_encrypt(
    plaintext: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
    aad: &[u8],
) -> SgxResult<(Vec<u8>, [u8; 16])> {
    let mut ciphertext = vec![0u8; plaintext.len()];
    let mut mac = [0u8; 16];

    rsgx_aes_gcm_encrypt(
        key,
        plaintext,
        nonce,
        aad,
        &mut ciphertext,
        &mut mac,
    )?;

    Ok((ciphertext, mac))
}

/// AES-256-GCM 解密
pub fn aes_gcm_decrypt(
    ciphertext: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
    aad: &[u8],
    mac: &[u8; 16],
) -> SgxResult<Vec<u8>> {
    let mut plaintext = vec![0u8; ciphertext.len()];

    rsgx_aes_gcm_decrypt(
        key,
        ciphertext,
        nonce,
        aad,
        mac,
        &mut plaintext,
    )?;

    Ok(plaintext)
}

/// 解密输入数据
pub fn decrypt_input(
    encrypted: &EncryptedData,
    key_manager: &EnclaveKeyManager,
) -> SgxResult<Vec<u8>> {
    // ECDH 密钥协商
    let shared_secret = key_manager.ecdh_shared_secret(&encrypted.ephemeral_pubkey)?;

    // 派生加密密钥
    let enc_key = derive_encryption_key(&shared_secret, &encrypted.ephemeral_pubkey);

    // AES-GCM 解密
    aes_gcm_decrypt(
        &encrypted.ciphertext,
        &enc_key,
        &encrypted.nonce,
        &[],
        &encrypted.auth_tag,
    )
}

/// 加密输出数据
pub fn encrypt_output(
    plaintext: &[u8],
    requester_pubkey: &[u8; 32],
    key_manager: &EnclaveKeyManager,
) -> SgxResult<EncryptedData> {
    // 生成临时密钥对
    let ephemeral = X25519KeyPair::generate()?;

    // ECDH 密钥协商
    let shared_secret = ephemeral.shared_secret(requester_pubkey)?;

    // 派生加密密钥
    let enc_key = derive_encryption_key(&shared_secret, &ephemeral.public_key());

    // 生成随机 nonce
    let nonce = generate_random_nonce()?;

    // AES-GCM 加密
    let (ciphertext, auth_tag) = aes_gcm_encrypt(plaintext, &enc_key, &nonce, &[])?;

    Ok(EncryptedData {
        ciphertext: ciphertext.try_into().map_err(|_| sgx_status_t::SGX_ERROR_UNEXPECTED)?,
        ephemeral_pubkey: ephemeral.public_key(),
        nonce,
        auth_tag,
    })
}

/// 派生加密密钥 (HKDF)
fn derive_encryption_key(shared_secret: &[u8; 32], info: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};

    // 简化的 HKDF-Expand
    let mut hasher = Sha256::new();
    hasher.update(shared_secret);
    hasher.update(info);
    hasher.update(&[0x01]);

    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// SHA-256 哈希
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hash = [0u8; 32];
    rsgx_sha256_slice(data, &mut hash).unwrap();
    hash
}

/// 生成随机 nonce
fn generate_random_nonce() -> SgxResult<[u8; 12]> {
    let mut nonce = [0u8; 12];
    rsgx_read_rand(&mut nonce)?;
    Ok(nonce)
}
```

---

## 6. 数据流程

### 6.1 计算请求流程

```
┌─────────┐                ┌─────────┐                ┌─────────┐
│  用户    │                │ Substrate│               │ TEE节点  │
│ 客户端   │                │   节点   │               │ Enclave │
└────┬────┘                └────┬────┘               └────┬────┘
     │                          │                         │
     │ 1. 获取 Enclave 公钥     │                         │
     │─────────────────────────>│                         │
     │                          │                         │
     │ 2. 返回公钥+认证报告     │                         │
     │<─────────────────────────│                         │
     │                          │                         │
     │ 3. 验证认证报告          │                         │
     │ (本地或 IAS)             │                         │
     │                          │                         │
     │ 4. 用公钥加密输入数据    │                         │
     │                          │                         │
     │ 5. 提交计算请求          │                         │
     │─────────────────────────>│                         │
     │                          │                         │
     │                          │ 6. 存储请求到链上       │
     │                          │─────────────────────────│
     │                          │                         │
     │                          │ 7. OCW 获取待处理请求   │
     │                          │<────────────────────────│
     │                          │                         │
     │                          │                         │ 8. Enclave 内
     │                          │                         │    - 解密输入
     │                          │                         │    - 执行计算
     │                          │                         │    - 加密输出
     │                          │                         │    - 生成证明
     │                          │                         │
     │                          │ 9. 提交计算结果         │
     │                          │<────────────────────────│
     │                          │                         │
     │                          │ 10. 验证证明并存储      │
     │                          │                         │
     │ 11. 查询结果             │                         │
     │─────────────────────────>│                         │
     │                          │                         │
     │ 12. 返回加密结果         │                         │
     │<─────────────────────────│                         │
     │                          │                         │
     │ 13. 本地解密查看         │                         │
     │                          │                         │
```

### 6.2 客户端加密流程

```python
# 伪代码示例

def submit_divination_request(birth_data, enclave_pubkey):
    # 1. 生成临时 ECDH 密钥对
    ephemeral_private, ephemeral_public = x25519_generate_keypair()

    # 2. 计算共享密钥
    shared_secret = x25519_ecdh(ephemeral_private, enclave_pubkey)

    # 3. 派生加密密钥
    enc_key = hkdf_expand(shared_secret, ephemeral_public, 32)

    # 4. 生成随机 nonce
    nonce = random_bytes(12)

    # 5. 序列化输入数据
    plaintext = serialize(birth_data)

    # 6. AES-256-GCM 加密
    ciphertext, auth_tag = aes_gcm_encrypt(plaintext, enc_key, nonce)

    # 7. 构造加密数据
    encrypted_data = EncryptedData(
        ciphertext=ciphertext,
        ephemeral_pubkey=ephemeral_public,
        nonce=nonce,
        auth_tag=auth_tag
    )

    # 8. 提交到链上
    submit_extrinsic("submit_compute_request", encrypted_data)
```

---

## 7. API 设计

### 7.1 RPC 接口

```rust
// rpc/tee-privacy/src/lib.rs

#[rpc(server)]
pub trait TeePrivacyApi<BlockHash, AccountId> {
    /// 获取活跃 TEE 节点列表
    #[method(name = "teePrivacy_getActiveNodes")]
    fn get_active_nodes(
        &self,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<TeeNodeInfo<AccountId>>>;

    /// 获取指定节点的 Enclave 公钥和认证信息
    #[method(name = "teePrivacy_getNodeAttestation")]
    fn get_node_attestation(
        &self,
        node: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<Option<TeeAttestation>>;

    /// 获取请求状态
    #[method(name = "teePrivacy_getRequestStatus")]
    fn get_request_status(
        &self,
        request_id: u64,
        at: Option<BlockHash>,
    ) -> RpcResult<Option<RequestStatusInfo>>;

    /// 获取计算结果
    #[method(name = "teePrivacy_getComputeResult")]
    fn get_compute_result(
        &self,
        request_id: u64,
        at: Option<BlockHash>,
    ) -> RpcResult<Option<TeeComputeResult<AccountId>>>;

    /// 获取用户的请求历史
    #[method(name = "teePrivacy_getUserRequests")]
    fn get_user_requests(
        &self,
        user: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u64>>;

    /// 验证认证报告 (辅助接口)
    #[method(name = "teePrivacy_verifyAttestation")]
    fn verify_attestation(
        &self,
        attestation: TeeAttestation,
    ) -> RpcResult<AttestationVerifyResult>;
}
```

### 7.2 Extrinsic 接口

| 函数 | 描述 | 调用者 |
|------|------|--------|
| `register_tee_node` | 注册 TEE 节点 | TEE 节点 |
| `update_attestation` | 更新认证报告 | TEE 节点 |
| `deregister_tee_node` | 注销节点 | TEE 节点 |
| `submit_compute_request` | 提交计算请求 | 用户 |
| `submit_compute_result` | 提交计算结果 | TEE 节点 |
| `cancel_request` | 取消请求 | 用户 |

### 7.3 占卜模块适配接口

```rust
// pallets/divination/bazi/src/lib.rs

impl<T: Config> Pallet<T> {
    /// 创建隐私八字计算请求
    pub fn create_private_bazi(
        origin: OriginFor<T>,
        encrypted_input: EncryptedData,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        // 构造计算类型
        let compute_type = ComputeType::BaZi(BaZiParams::default());

        // 调用 TEE Privacy 模块
        pallet_tee_privacy::Pallet::<T>::submit_compute_request(
            origin,
            compute_type,
            encrypted_input,
            None, // 自动分配节点
        )
    }

    /// 获取八字计算结果
    pub fn get_bazi_result(
        origin: OriginFor<T>,
        request_id: u64,
    ) -> Result<Option<EncryptedData>, DispatchError> {
        let who = ensure_signed(origin)?;

        // 验证请求所有权
        let request = pallet_tee_privacy::PendingRequests::<T>::get(request_id)
            .ok_or(Error::<T>::RequestNotFound)?;
        ensure!(request.requester == who, Error::<T>::Unauthorized);

        // 获取结果
        let result = pallet_tee_privacy::ComputeResults::<T>::get(request_id);
        Ok(result.map(|r| r.encrypted_output))
    }
}
```

---

## 8. 安全机制

### 8.1 远程认证

```
┌────────────────────────────────────────────────────────────────┐
│                        远程认证流程                             │
└────────────────────────────────────────────────────────────────┘

1. Enclave 生成 Quote
   ┌─────────────────────┐
   │     Enclave         │
   │  ┌───────────────┐  │
   │  │ REPORT        │  │
   │  │ - MRENCLAVE   │  │
   │  │ - MRSIGNER    │  │
   │  │ - User Data   │  │
   │  └───────────────┘  │
   └──────────┬──────────┘
              │
              ▼
   ┌─────────────────────┐
   │   Quoting Enclave   │
   │   (Intel 提供)       │
   │  ┌───────────────┐  │
   │  │ QUOTE         │  │
   │  │ - REPORT      │  │
   │  │ - EPID 签名   │  │
   │  └───────────────┘  │
   └──────────┬──────────┘
              │
2. 发送 Quote 到 IAS
              │
              ▼
   ┌─────────────────────┐
   │ Intel Attestation   │
   │     Service         │
   │  ┌───────────────┐  │
   │  │ 验证 Quote    │  │
   │  │ 签发认证报告   │  │
   │  └───────────────┘  │
   └──────────┬──────────┘
              │
3. 验证 IAS 签名
              │
              ▼
   ┌─────────────────────┐
   │   链上验证模块       │
   │  - 验证 IAS 签名    │
   │  - 检查 MRENCLAVE   │
   │  - 检查有效期       │
   └─────────────────────┘
```

### 8.2 安全边界

| 保护对象 | 保护措施 |
|----------|----------|
| 用户输入数据 | Enclave 公钥加密，仅 Enclave 内可解密 |
| 计算过程 | 在 Enclave 内执行，外部不可见 |
| 计算结果 | 用用户公钥加密，仅用户可解密 |
| Enclave 密钥 | SGX Sealing 保护，与硬件绑定 |
| 代码完整性 | MRENCLAVE 度量值验证 |

### 8.3 威胁模型

| 威胁 | 缓解措施 |
|------|----------|
| 恶意节点运营商 | TEE 隔离，运营商无法访问 Enclave 内数据 |
| 侧信道攻击 | 使用常量时间算法，避免分支泄露 |
| 回滚攻击 | 使用单调计数器，检测回滚 |
| 伪造认证 | 链上验证 IAS 签名，检查 MRENCLAVE |
| 重放攻击 | 请求 ID 唯一，结果与请求绑定 |

### 8.4 密钥管理

```
┌─────────────────────────────────────────────────────────────┐
│                     密钥层次结构                              │
└─────────────────────────────────────────────────────────────┘

                    ┌──────────────────┐
                    │  SGX Root Key    │ ← 硬件级别
                    │  (不可导出)       │
                    └────────┬─────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
     ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
     │ Sealing Key  │ │ Report Key   │ │ Launch Key   │
     │ (数据持久化)  │ │ (本地认证)   │ │ (Enclave启动)│
     └──────┬───────┘ └──────────────┘ └──────────────┘
            │
            ▼
     ┌──────────────┐
     │ Master Key   │ ← 从 Sealing Key 派生
     │ (Enclave级)  │
     └──────┬───────┘
            │
     ┌──────┴──────┐
     ▼             ▼
┌─────────┐  ┌─────────┐
│ ECDH    │  │ Signing │
│ KeyPair │  │ KeyPair │
└─────────┘  └─────────┘
```

---

## 9. 部署方案

### 9.1 TEE 节点要求

**硬件要求:**
- CPU: Intel Xeon (支持 SGX) 或 AMD EPYC (支持 SEV)
- 内存: 16GB+
- EPC 内存: 256MB+ (SGX)
- 存储: 100GB SSD

**软件要求:**
- OS: Ubuntu 20.04/22.04 LTS
- SGX Driver: 2.17+
- SGX SDK: 2.18+
- Rust: 1.70+

### 9.2 节点部署

```bash
# 1. 安装 SGX 驱动
wget https://download.01.org/intel-sgx/sgx-linux/2.18/distro/ubuntu20.04-server/sgx_linux_x64_driver_2.11.0_2d2b795.bin
chmod +x sgx_linux_x64_driver_2.11.0_2d2b795.bin
sudo ./sgx_linux_x64_driver_2.11.0_2d2b795.bin

# 2. 安装 SGX SDK
wget https://download.01.org/intel-sgx/sgx-linux/2.18/distro/ubuntu20.04-server/sgx_linux_x64_sdk_2.18.100.3.bin
chmod +x sgx_linux_x64_sdk_2.18.100.3.bin
sudo ./sgx_linux_x64_sdk_2.18.100.3.bin

# 3. 编译节点
git clone https://github.com/lao-sha/stardust.git
cd stardust
cargo build --release --features tee-enclave

# 4. 启动节点
./target/release/stardust-node \
    --chain mainnet \
    --tee-enabled \
    --enclave-path ./target/release/enclave.signed.so \
    --ias-api-key $IAS_API_KEY
```

### 9.3 节点注册

```bash
# 生成并提交远程认证
./target/release/stardust-cli tee register \
    --suri "//TeeNode" \
    --ias-url https://api.trustedservices.intel.com/sgx/attestation/v4
```

### 9.4 监控配置

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'stardust-tee'
    static_configs:
      - targets: ['localhost:9615']
    metrics_path: /metrics

# 关键指标
# - tee_requests_pending: 待处理请求数
# - tee_requests_completed: 已完成请求数
# - tee_enclave_uptime: Enclave 运行时间
# - tee_attestation_age: 认证报告年龄
```

---

## 10. 测试计划

### 10.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_ok, assert_noop};

    #[test]
    fn test_register_tee_node() {
        new_test_ext().execute_with(|| {
            let node = account(1);
            let pubkey = [1u8; 32];
            let attestation = mock_attestation();

            assert_ok!(TeePrivacy::register_tee_node(
                RuntimeOrigin::signed(node.clone()),
                pubkey,
                attestation,
            ));

            assert!(TeeNodes::<Test>::contains_key(&node));
            assert!(ActiveNodes::<Test>::get().contains(&node));
        });
    }

    #[test]
    fn test_submit_compute_request() {
        new_test_ext().execute_with(|| {
            // 注册 TEE 节点
            setup_tee_node();

            let user = account(2);
            let encrypted_input = mock_encrypted_data();

            assert_ok!(TeePrivacy::submit_compute_request(
                RuntimeOrigin::signed(user.clone()),
                ComputeType::BaZi(BaZiParams::default()),
                encrypted_input,
                None,
            ));

            let request_id = NextRequestId::<Test>::get() - 1;
            let request = PendingRequests::<Test>::get(request_id).unwrap();

            assert_eq!(request.requester, user);
            assert_eq!(request.status, RequestStatus::Pending);
        });
    }

    #[test]
    fn test_compute_result_verification() {
        new_test_ext().execute_with(|| {
            // 设置请求
            let request_id = setup_pending_request();

            let node = get_tee_node();
            let result = mock_compute_result();
            let proof = mock_computation_proof();

            assert_ok!(TeePrivacy::submit_compute_result(
                RuntimeOrigin::signed(node),
                request_id,
                result,
                proof,
            ));

            let stored = ComputeResults::<Test>::get(request_id).unwrap();
            assert!(stored.computation_proof.enclave_signature.len() > 0);
        });
    }

    #[test]
    fn test_request_timeout() {
        new_test_ext().execute_with(|| {
            let request_id = setup_pending_request();

            // 推进到超时区块
            run_to_block(REQUEST_TIMEOUT + 1);

            let request = PendingRequests::<Test>::get(request_id).unwrap();
            assert_eq!(request.status, RequestStatus::Timeout);
        });
    }
}
```

### 10.2 集成测试

```bash
# 测试脚本

#!/bin/bash

echo "=== TEE Privacy 集成测试 ==="

# 1. 启动本地测试网
./target/release/stardust-node --dev &
NODE_PID=$!
sleep 5

# 2. 启动 TEE 模拟器
./scripts/start-sgx-simulator.sh &
SGX_PID=$!
sleep 3

# 3. 注册 TEE 节点
echo "注册 TEE 节点..."
./scripts/register-tee-node.sh

# 4. 提交测试请求
echo "提交计算请求..."
REQUEST_ID=$(./scripts/submit-test-request.sh)
echo "Request ID: $REQUEST_ID"

# 5. 等待计算完成
echo "等待计算..."
sleep 30

# 6. 验证结果
echo "验证结果..."
./scripts/verify-result.sh $REQUEST_ID

# 清理
kill $NODE_PID $SGX_PID
```

### 10.3 安全测试

| 测试项 | 描述 | 预期结果 |
|--------|------|----------|
| 认证伪造 | 提交伪造的认证报告 | 被拒绝 |
| 密钥泄露 | 尝试从外部读取 Enclave 密钥 | 无法访问 |
| 结果篡改 | 篡改计算结果 | 证明验证失败 |
| 重放攻击 | 重放旧的计算结果 | 请求 ID 不匹配 |
| 侧信道 | 时间侧信道分析 | 无信息泄露 |

---

## 11. 经济激励机制

### 11.1 节点质押与奖励

TEE 节点需要质押代币以参与网络，并通过完成计算获得奖励。

```rust
// pallets/tee-privacy/src/economics.rs

use frame_support::pallet_prelude::*;

/// 节点质押信息
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct NodeStake<Balance, BlockNumber> {
    /// 质押金额
    pub amount: Balance,
    /// 质押开始区块
    pub staked_at: BlockNumber,
    /// 解锁中的金额
    pub unlocking: Balance,
    /// 解锁完成区块
    pub unlock_at: Option<BlockNumber>,
    /// 累计奖励
    pub total_rewards: Balance,
    /// 可提取奖励
    pub claimable_rewards: Balance,
}

/// 计算奖励信息
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ComputeReward<Balance> {
    /// 基础奖励
    pub base_reward: Balance,
    /// 计算类型乘数 (basis points, 10000 = 1x)
    pub type_multiplier: u32,
    /// 紧急奖励 (快速响应加成)
    pub urgency_bonus: Balance,
}

// ==================== 存储 ====================

/// 节点质押存储
#[pallet::storage]
#[pallet::getter(fn node_stakes)]
pub type NodeStakes<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    NodeStake<BalanceOf<T>, T::BlockNumber>,
    OptionQuery,
>;

/// 计算类型奖励配置
#[pallet::storage]
#[pallet::getter(fn compute_rewards)]
pub type ComputeRewardConfig<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    ComputeType,
    ComputeReward<BalanceOf<T>>,
    OptionQuery,
>;

/// 最小质押金额
#[pallet::storage]
#[pallet::getter(fn min_stake)]
pub type MinStake<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

/// 解锁周期（区块数）
#[pallet::storage]
#[pallet::getter(fn unlock_period)]
pub type UnlockPeriod<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

// ==================== 调用 ====================

impl<T: Config> Pallet<T> {
    /// 质押代币成为 TEE 节点
    #[pallet::call_index(10)]
    #[pallet::weight(T::WeightInfo::stake())]
    pub fn stake(
        origin: OriginFor<T>,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        ensure!(amount >= MinStake::<T>::get(), Error::<T>::InsufficientStake);

        // 锁定代币
        T::Currency::reserve(&who, amount)?;

        let current_block = <frame_system::Pallet<T>>::block_number();

        NodeStakes::<T>::mutate(&who, |maybe_stake| {
            if let Some(stake) = maybe_stake {
                stake.amount = stake.amount.saturating_add(amount);
            } else {
                *maybe_stake = Some(NodeStake {
                    amount,
                    staked_at: current_block,
                    unlocking: Zero::zero(),
                    unlock_at: None,
                    total_rewards: Zero::zero(),
                    claimable_rewards: Zero::zero(),
                });
            }
        });

        Self::deposit_event(Event::Staked { node: who, amount });

        Ok(())
    }

    /// 申请解除质押
    #[pallet::call_index(11)]
    #[pallet::weight(T::WeightInfo::unstake())]
    pub fn unstake(
        origin: OriginFor<T>,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        NodeStakes::<T>::try_mutate(&who, |maybe_stake| {
            let stake = maybe_stake.as_mut().ok_or(Error::<T>::NotStaked)?;

            ensure!(stake.amount >= amount, Error::<T>::InsufficientStake);

            stake.amount = stake.amount.saturating_sub(amount);
            stake.unlocking = stake.unlocking.saturating_add(amount);

            let current_block = <frame_system::Pallet<T>>::block_number();
            stake.unlock_at = Some(current_block.saturating_add(UnlockPeriod::<T>::get()));

            // 如果质押不足，移出活跃列表
            if stake.amount < MinStake::<T>::get() {
                ActiveNodes::<T>::mutate(|nodes| {
                    nodes.retain(|n| n != &who);
                });
            }

            Self::deposit_event(Event::UnstakeRequested { node: who.clone(), amount });

            Ok(())
        })
    }

    /// 提取已解锁的质押
    #[pallet::call_index(12)]
    #[pallet::weight(T::WeightInfo::withdraw_stake())]
    pub fn withdraw_stake(origin: OriginFor<T>) -> DispatchResult {
        let who = ensure_signed(origin)?;

        NodeStakes::<T>::try_mutate(&who, |maybe_stake| {
            let stake = maybe_stake.as_mut().ok_or(Error::<T>::NotStaked)?;

            let current_block = <frame_system::Pallet<T>>::block_number();

            if let Some(unlock_at) = stake.unlock_at {
                ensure!(current_block >= unlock_at, Error::<T>::StillLocked);

                let amount = stake.unlocking;
                stake.unlocking = Zero::zero();
                stake.unlock_at = None;

                // 解锁代币
                T::Currency::unreserve(&who, amount);

                Self::deposit_event(Event::StakeWithdrawn { node: who.clone(), amount });
            }

            Ok(())
        })
    }

    /// 提取奖励
    #[pallet::call_index(13)]
    #[pallet::weight(T::WeightInfo::claim_rewards())]
    pub fn claim_rewards(origin: OriginFor<T>) -> DispatchResult {
        let who = ensure_signed(origin)?;

        NodeStakes::<T>::try_mutate(&who, |maybe_stake| {
            let stake = maybe_stake.as_mut().ok_or(Error::<T>::NotStaked)?;

            let rewards = stake.claimable_rewards;
            ensure!(!rewards.is_zero(), Error::<T>::NoRewardsToClaim);

            stake.claimable_rewards = Zero::zero();

            // 从奖励池转账
            T::Currency::transfer(
                &Self::reward_pool_account(),
                &who,
                rewards,
                ExistenceRequirement::KeepAlive,
            )?;

            Self::deposit_event(Event::RewardsClaimed { node: who.clone(), amount: rewards });

            Ok(())
        })
    }

    /// 内部：发放计算奖励
    pub(crate) fn distribute_compute_reward(
        executor: &T::AccountId,
        compute_type: &ComputeType,
        response_blocks: T::BlockNumber,
    ) -> DispatchResult {
        let reward_config = ComputeRewardConfig::<T>::get(compute_type)
            .unwrap_or_default();

        // 计算奖励：基础奖励 * 类型乘数 + 紧急奖励
        let mut reward = reward_config.base_reward
            .saturating_mul(reward_config.type_multiplier.into())
            / 10000u32.into();

        // 快速响应加成（响应越快，奖励越高）
        if response_blocks < T::FastResponseThreshold::get() {
            reward = reward.saturating_add(reward_config.urgency_bonus);
        }

        // 更新节点奖励
        NodeStakes::<T>::mutate(executor, |maybe_stake| {
            if let Some(stake) = maybe_stake {
                stake.total_rewards = stake.total_rewards.saturating_add(reward);
                stake.claimable_rewards = stake.claimable_rewards.saturating_add(reward);
            }
        });

        Self::deposit_event(Event::RewardDistributed {
            node: executor.clone(),
            amount: reward,
        });

        Ok(())
    }
}
```

### 11.2 惩罚机制

```rust
/// 惩罚类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum SlashReason {
    /// 计算超时
    ComputeTimeout,
    /// 提交无效结果
    InvalidResult,
    /// 认证过期未更新
    AttestationExpired,
    /// 恶意行为
    MaliciousBehavior,
}

/// 惩罚比例配置 (basis points)
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct SlashConfig {
    /// 超时惩罚比例
    pub timeout_slash: u32,  // 500 = 5%
    /// 无效结果惩罚比例
    pub invalid_result_slash: u32,  // 1000 = 10%
    /// 认证过期惩罚比例
    pub attestation_expired_slash: u32,  // 200 = 2%
    /// 恶意行为惩罚比例
    pub malicious_slash: u32,  // 10000 = 100%
}

impl<T: Config> Pallet<T> {
    /// 执行惩罚
    pub(crate) fn slash_node(
        node: &T::AccountId,
        reason: SlashReason,
    ) -> DispatchResult {
        let slash_config = SlashConfig::<T>::get();

        let slash_rate = match reason {
            SlashReason::ComputeTimeout => slash_config.timeout_slash,
            SlashReason::InvalidResult => slash_config.invalid_result_slash,
            SlashReason::AttestationExpired => slash_config.attestation_expired_slash,
            SlashReason::MaliciousBehavior => slash_config.malicious_slash,
        };

        NodeStakes::<T>::try_mutate(node, |maybe_stake| {
            let stake = maybe_stake.as_mut().ok_or(Error::<T>::NotStaked)?;

            let slash_amount = stake.amount
                .saturating_mul(slash_rate.into())
                / 10000u32.into();

            stake.amount = stake.amount.saturating_sub(slash_amount);

            // 将惩罚金额转入国库
            T::Currency::slash_reserved(node, slash_amount);

            // 如果质押不足最小值，移出活跃列表
            if stake.amount < MinStake::<T>::get() {
                ActiveNodes::<T>::mutate(|nodes| {
                    nodes.retain(|n| n != node);
                });

                TeeNodes::<T>::mutate(node, |maybe_node| {
                    if let Some(n) = maybe_node {
                        n.status = TeeNodeStatus::Suspended;
                    }
                });
            }

            Self::deposit_event(Event::NodeSlashed {
                node: node.clone(),
                reason,
                amount: slash_amount,
            });

            Ok(())
        })
    }
}
```

---

## 12. 故障转移机制

### 12.1 超时优化

使用有序存储避免遍历所有请求：

```rust
// pallets/tee-privacy/src/timeout.rs

use sp_std::collections::btree_map::BTreeMap;

/// 超时队列 - 按超时区块排序
#[pallet::storage]
#[pallet::getter(fn timeout_queue)]
pub type TimeoutQueue<T: Config> = StorageValue<
    _,
    BoundedBTreeMap<T::BlockNumber, BoundedVec<u64, ConstU32<100>>, ConstU32<1000>>,
    ValueQuery,
>;

impl<T: Config> Pallet<T> {
    /// 添加请求到超时队列
    pub(crate) fn add_to_timeout_queue(
        request_id: u64,
        timeout_at: T::BlockNumber,
    ) -> DispatchResult {
        TimeoutQueue::<T>::try_mutate(|queue| {
            if let Some(requests) = queue.get_mut(&timeout_at) {
                requests.try_push(request_id).map_err(|_| Error::<T>::TooManyRequests)?;
            } else {
                let mut requests = BoundedVec::new();
                requests.try_push(request_id).map_err(|_| Error::<T>::TooManyRequests)?;
                queue.try_insert(timeout_at, requests).map_err(|_| Error::<T>::TooManyRequests)?;
            }
            Ok(())
        })
    }

    /// 从超时队列移除请求
    pub(crate) fn remove_from_timeout_queue(
        request_id: u64,
        timeout_at: T::BlockNumber,
    ) {
        TimeoutQueue::<T>::mutate(|queue| {
            if let Some(requests) = queue.get_mut(&timeout_at) {
                requests.retain(|id| *id != request_id);
                if requests.is_empty() {
                    queue.remove(&timeout_at);
                }
            }
        });
    }
}

#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    /// 优化的超时检查 - 只处理当前区块超时的请求
    fn on_initialize(now: T::BlockNumber) -> Weight {
        let mut weight = Weight::zero();

        // 获取当前区块需要处理的超时请求
        let timeout_requests = TimeoutQueue::<T>::mutate(|queue| {
            queue.remove(&now).unwrap_or_default()
        });

        for request_id in timeout_requests.iter() {
            if let Some(mut request) = PendingRequests::<T>::get(request_id) {
                if request.status == RequestStatus::Pending ||
                   request.status == RequestStatus::Processing {
                    // 尝试故障转移
                    if Self::try_failover(*request_id, &mut request).is_err() {
                        // 故障转移失败，标记为超时
                        request.status = RequestStatus::Timeout;
                        PendingRequests::<T>::insert(request_id, request.clone());

                        // 惩罚未完成的节点
                        if let Some(ref node) = request.assigned_node {
                            let _ = Self::slash_node(node, SlashReason::ComputeTimeout);
                        }

                        Self::deposit_event(Event::RequestTimeout { request_id: *request_id });
                    }

                    weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 2));
                }
            }
        }

        weight
    }
}
```

### 12.2 请求故障转移

```rust
// pallets/tee-privacy/src/failover.rs

/// 故障转移配置
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct FailoverConfig<BlockNumber> {
    /// 最大重试次数
    pub max_retries: u8,
    /// 重试间隔（区块数）
    pub retry_interval: BlockNumber,
    /// 是否启用自动故障转移
    pub auto_failover: bool,
}

/// 请求重试信息
#[pallet::storage]
#[pallet::getter(fn request_retries)]
pub type RequestRetries<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,
    u8,
    ValueQuery,
>;

impl<T: Config> Pallet<T> {
    /// 尝试故障转移
    pub(crate) fn try_failover(
        request_id: u64,
        request: &mut TeeComputeRequest<T::AccountId, T::BlockNumber>,
    ) -> DispatchResult {
        let config = FailoverConfig::<T>::get();

        ensure!(config.auto_failover, Error::<T>::FailoverDisabled);

        let retries = RequestRetries::<T>::get(request_id);
        ensure!(retries < config.max_retries, Error::<T>::MaxRetriesExceeded);

        // 获取新的可用节点
        let failed_node = request.assigned_node.clone();
        let new_node = Self::select_available_node(&failed_node)?;

        // 更新请求
        request.assigned_node = Some(new_node.clone());
        request.status = RequestStatus::Pending;

        let current_block = <frame_system::Pallet<T>>::block_number();
        let new_timeout = current_block.saturating_add(T::RequestTimeout::get());
        request.timeout_at = new_timeout;

        // 更新存储
        PendingRequests::<T>::insert(request_id, request.clone());
        RequestRetries::<T>::insert(request_id, retries.saturating_add(1));

        // 添加到新的超时队列
        Self::add_to_timeout_queue(request_id, new_timeout)?;

        Self::deposit_event(Event::RequestFailover {
            request_id,
            from_node: failed_node,
            to_node: new_node,
            retry_count: retries + 1,
        });

        Ok(())
    }

    /// 选择可用节点（排除失败节点）
    fn select_available_node(
        exclude: &Option<T::AccountId>,
    ) -> Result<T::AccountId, DispatchError> {
        let active_nodes = ActiveNodes::<T>::get();

        // 过滤掉失败节点和认证过期节点
        let now = T::TimeProvider::now().as_secs();

        let available: Vec<_> = active_nodes
            .iter()
            .filter(|node| {
                // 排除失败节点
                if let Some(excluded) = exclude {
                    if *node == excluded {
                        return false;
                    }
                }

                // 检查节点状态和认证
                if let Some(tee_node) = TeeNodes::<T>::get(node) {
                    tee_node.status == TeeNodeStatus::Active &&
                    now.saturating_sub(tee_node.attestation.timestamp) < T::AttestationValidity::get()
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        ensure!(!available.is_empty(), Error::<T>::NoAvailableNodes);

        // 简单随机选择（可改进为负载均衡）
        let index = Self::random_index(available.len());
        Ok(available[index].clone())
    }

    /// 节点故障处理
    pub fn on_node_failure(node: T::AccountId) -> DispatchResult {
        // 更新节点状态
        TeeNodes::<T>::mutate(&node, |maybe_node| {
            if let Some(n) = maybe_node {
                n.status = TeeNodeStatus::Suspended;
            }
        });

        // 从活跃列表移除
        ActiveNodes::<T>::mutate(|nodes| {
            nodes.retain(|n| n != &node);
        });

        // 获取该节点的所有待处理请求
        let pending: Vec<_> = PendingRequests::<T>::iter()
            .filter(|(_, req)| req.assigned_node.as_ref() == Some(&node))
            .map(|(id, _)| id)
            .collect();

        // 重新分配请求
        for request_id in pending {
            if let Some(mut request) = PendingRequests::<T>::get(request_id) {
                let _ = Self::try_failover(request_id, &mut request);
            }
        }

        Self::deposit_event(Event::NodeFailure { node });

        Ok(())
    }
}
```

---

## 13. DCAP 认证支持

### 13.1 DCAP vs EPID

| 特性 | EPID (传统) | DCAP (推荐) |
|------|-------------|-------------|
| 认证服务 | Intel IAS (中心化) | 本地验证 (去中心化) |
| 网络依赖 | 需要连接 Intel 服务器 | 无网络依赖 |
| 隐私性 | Intel 可追踪 | 完全匿名 |
| 延迟 | 高 (网络请求) | 低 (本地验证) |
| 适用场景 | 传统部署 | 区块链/去中心化 |

### 13.2 DCAP 认证实现

```rust
// pallets/tee-privacy/src/attestation/dcap.rs

use sgx_dcap_quoteverify_rs::*;

/// DCAP 认证报告
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct DcapAttestation {
    /// Quote 数据
    pub quote: BoundedVec<u8, ConstU32<8192>>,
    /// 附加数据 (PCK 证书链)
    pub collateral: QuoteCollateral,
    /// 用户自定义数据
    pub report_data: [u8; 64],
    /// 时间戳
    pub timestamp: u64,
}

/// 抵押品数据
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct QuoteCollateral {
    /// PCK 证书链
    pub pck_crl_issuer_chain: BoundedVec<u8, ConstU32<4096>>,
    /// Root CA CRL
    pub root_ca_crl: BoundedVec<u8, ConstU32<2048>>,
    /// PCK CRL
    pub pck_crl: BoundedVec<u8, ConstU32<2048>>,
    /// TCB 信息
    pub tcb_info: BoundedVec<u8, ConstU32<4096>>,
    /// TCB 信息签名
    pub tcb_info_signature: BoundedVec<u8, ConstU32<512>>,
    /// QE Identity
    pub qe_identity: BoundedVec<u8, ConstU32<2048>>,
    /// QE Identity 签名
    pub qe_identity_signature: BoundedVec<u8, ConstU32<512>>,
}

/// DCAP 验证器
pub struct DcapVerifier;

impl DcapVerifier {
    /// 验证 DCAP Quote
    pub fn verify(
        attestation: &DcapAttestation,
        expected_mr_enclave: &[u8; 32],
    ) -> Result<bool, AttestationError> {
        // 1. 验证 Quote 格式
        let quote = Quote::parse(&attestation.quote)
            .map_err(|_| AttestationError::InvalidQuoteFormat)?;

        // 2. 验证 MRENCLAVE
        if quote.report_body.mr_enclave != *expected_mr_enclave {
            return Err(AttestationError::MrEnclaveMismatch);
        }

        // 3. 验证 Report Data
        if quote.report_body.report_data[..32] != attestation.report_data[..32] {
            return Err(AttestationError::ReportDataMismatch);
        }

        // 4. 验证 Quote 签名
        let collateral = attestation.collateral.to_sgx_collateral();
        let verification_result = sgx_qv_verify_quote(
            &attestation.quote,
            Some(&collateral),
            attestation.timestamp as i64,
        ).map_err(|_| AttestationError::QuoteVerificationFailed)?;

        // 5. 检查验证结果
        match verification_result.quote_verification_result {
            SgxQlQvResult::Ok => Ok(true),
            SgxQlQvResult::ConfigNeeded |
            SgxQlQvResult::OutOfDate |
            SgxQlQvResult::OutOfDateConfigNeeded => {
                // 可接受的 TCB 状态（取决于安全策略）
                Ok(true)
            }
            _ => Err(AttestationError::TcbStatusInvalid),
        }
    }
}

/// 链上 DCAP 验证
impl<T: Config> Pallet<T> {
    /// 注册节点（DCAP 模式）
    #[pallet::call_index(20)]
    #[pallet::weight(T::WeightInfo::register_tee_node_dcap())]
    pub fn register_tee_node_dcap(
        origin: OriginFor<T>,
        enclave_pubkey: [u8; 32],
        dcap_attestation: DcapAttestation,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        ensure!(
            !TeeNodes::<T>::contains_key(&who),
            Error::<T>::NodeAlreadyRegistered
        );

        // 获取预期的 MRENCLAVE
        let expected_mr_enclave = ExpectedMrEnclave::<T>::get()
            .ok_or(Error::<T>::MrEnclaveNotSet)?;

        // DCAP 验证
        DcapVerifier::verify(&dcap_attestation, &expected_mr_enclave)
            .map_err(|_| Error::<T>::InvalidAttestation)?;

        // 转换为通用认证格式
        let attestation = TeeAttestation {
            tee_type: TeeType::IntelSgxDcap,
            mr_enclave: expected_mr_enclave,
            mr_signer: Self::extract_mr_signer(&dcap_attestation.quote),
            isv_prod_id: Self::extract_isv_prod_id(&dcap_attestation.quote),
            isv_svn: Self::extract_isv_svn(&dcap_attestation.quote),
            report_data: dcap_attestation.report_data,
            ias_signature: BoundedVec::new(), // DCAP 不使用 IAS
            timestamp: dcap_attestation.timestamp,
        };

        let node = TeeNode {
            account: who.clone(),
            enclave_pubkey,
            attestation,
            registered_at: T::TimeProvider::now().as_secs(),
            status: TeeNodeStatus::Active,
        };

        TeeNodes::<T>::insert(&who, node);
        ActiveNodes::<T>::try_mutate(|nodes| nodes.try_push(who.clone()))?;

        Self::deposit_event(Event::TeeNodeRegistered {
            node: who,
            enclave_pubkey,
        });

        Ok(())
    }
}
```

---

## 14. 批处理优化

### 14.1 批量请求接口

```rust
// pallets/tee-privacy/src/batch.rs

/// 批量计算请求
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct BatchComputeRequest<AccountId, BlockNumber> {
    /// 批次 ID
    pub batch_id: u64,
    /// 请求者
    pub requester: AccountId,
    /// 子请求列表
    pub requests: BoundedVec<ComputeRequestItem, ConstU32<50>>,
    /// 创建区块
    pub created_at: BlockNumber,
    /// 状态
    pub status: BatchStatus,
}

/// 单个请求项
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct ComputeRequestItem {
    /// 计算类型
    pub compute_type: ComputeType,
    /// 加密输入
    pub encrypted_input: EncryptedData,
}

/// 批次状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum BatchStatus {
    /// 待处理
    Pending,
    /// 处理中
    Processing { completed: u8, total: u8 },
    /// 已完成
    Completed,
    /// 部分失败
    PartialFailed { failed_indices: BoundedVec<u8, ConstU32<50>> },
}

/// 批次结果存储
#[pallet::storage]
#[pallet::getter(fn batch_results)]
pub type BatchResults<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,
    BoundedVec<Option<EncryptedData>, ConstU32<50>>,
    OptionQuery,
>;

impl<T: Config> Pallet<T> {
    /// 提交批量计算请求
    #[pallet::call_index(30)]
    #[pallet::weight(T::WeightInfo::submit_batch_request(requests.len() as u32))]
    pub fn submit_batch_request(
        origin: OriginFor<T>,
        requests: BoundedVec<ComputeRequestItem, ConstU32<50>>,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        ensure!(!requests.is_empty(), Error::<T>::EmptyBatch);

        let active_nodes = ActiveNodes::<T>::get();
        ensure!(!active_nodes.is_empty(), Error::<T>::NoAvailableNodes);

        let batch_id = NextBatchId::<T>::mutate(|id| {
            let current = *id;
            *id = id.saturating_add(1);
            current
        });

        let current_block = <frame_system::Pallet<T>>::block_number();

        let batch = BatchComputeRequest {
            batch_id,
            requester: who.clone(),
            requests: requests.clone(),
            created_at: current_block,
            status: BatchStatus::Pending,
        };

        PendingBatches::<T>::insert(batch_id, batch);

        // 初始化结果存储
        let empty_results: BoundedVec<Option<EncryptedData>, ConstU32<50>> =
            requests.iter().map(|_| None).collect::<Vec<_>>().try_into().unwrap();
        BatchResults::<T>::insert(batch_id, empty_results);

        // 批量收费（有折扣）
        let total_fee = Self::calculate_batch_fee(&requests)?;
        T::Currency::transfer(&who, &Self::fee_account(), total_fee, ExistenceRequirement::KeepAlive)?;

        Self::deposit_event(Event::BatchSubmitted {
            batch_id,
            requester: who,
            count: requests.len() as u8,
        });

        Ok(())
    }

    /// 提交批量结果
    #[pallet::call_index(31)]
    #[pallet::weight(T::WeightInfo::submit_batch_result(results.len() as u32))]
    pub fn submit_batch_result(
        origin: OriginFor<T>,
        batch_id: u64,
        results: BoundedVec<(u8, EncryptedData, ComputationProof), ConstU32<50>>,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        // 验证调用者是 TEE 节点
        let tee_node = TeeNodes::<T>::get(&who)
            .ok_or(Error::<T>::NodeNotRegistered)?;
        ensure!(tee_node.status == TeeNodeStatus::Active, Error::<T>::Unauthorized);

        let mut batch = PendingBatches::<T>::get(batch_id)
            .ok_or(Error::<T>::BatchNotFound)?;

        // 验证并存储结果
        BatchResults::<T>::try_mutate(batch_id, |maybe_results| {
            let stored_results = maybe_results.as_mut().ok_or(Error::<T>::BatchNotFound)?;

            for (index, encrypted_output, proof) in results.iter() {
                let idx = *index as usize;
                ensure!(idx < stored_results.len(), Error::<T>::InvalidIndex);

                // 验证计算证明
                Self::verify_computation_proof(proof, &tee_node.enclave_pubkey)?;

                stored_results[idx] = Some(encrypted_output.clone());
            }

            // 更新批次状态
            let completed_count = stored_results.iter().filter(|r| r.is_some()).count();
            if completed_count == stored_results.len() {
                batch.status = BatchStatus::Completed;
            } else {
                batch.status = BatchStatus::Processing {
                    completed: completed_count as u8,
                    total: stored_results.len() as u8,
                };
            }

            PendingBatches::<T>::insert(batch_id, batch);

            Ok(())
        })?;

        Self::deposit_event(Event::BatchResultSubmitted {
            batch_id,
            executor: who,
            count: results.len() as u8,
        });

        Ok(())
    }

    /// 计算批量费用（有折扣）
    fn calculate_batch_fee(
        requests: &[ComputeRequestItem],
    ) -> Result<BalanceOf<T>, DispatchError> {
        let base_fee = T::BaseFee::get();
        let count = requests.len() as u32;

        // 批量折扣：10个以上 10%，20个以上 20%，30个以上 30%
        let discount_rate = if count >= 30 {
            70u32  // 70%
        } else if count >= 20 {
            80u32
        } else if count >= 10 {
            90u32
        } else {
            100u32
        };

        let total = base_fee
            .saturating_mul(count.into())
            .saturating_mul(discount_rate.into())
            / 100u32.into();

        Ok(total)
    }
}
```

---

## 15. Enclave 升级机制

### 15.1 平滑升级流程

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Enclave 平滑升级流程                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  阶段 1: 准备                                                            │
│  ┌─────────────┐                                                        │
│  │ 1. 发布新版 │ → 治理提案 → 投票通过                                   │
│  │    Enclave  │                                                        │
│  └─────────────┘                                                        │
│         ↓                                                               │
│  阶段 2: 过渡期                                                          │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ 旧版 Enclave (MRENCLAVE_OLD)  ──────────────────────────────│        │
│  │   ✓ 继续处理请求                                             │        │
│  │   ✓ 接受新注册                                               │        │
│  ├─────────────────────────────────────────────────────────────┤        │
│  │ 新版 Enclave (MRENCLAVE_NEW)  ──────────────────────────────│        │
│  │   ✓ 开始接受注册                                             │        │
│  │   ✓ 开始处理请求                                             │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                               │
│  阶段 3: 迁移完成                                                        │
│  ┌─────────────┐                                                        │
│  │ 2. 废弃旧版 │ → 旧版节点强制升级或退出                                │
│  │    Enclave  │                                                        │
│  └─────────────┘                                                        │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 15.2 升级存储与治理

```rust
// pallets/tee-privacy/src/upgrade.rs

/// Enclave 版本信息
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct EnclaveVersion {
    /// 版本号
    pub version: u32,
    /// MRENCLAVE
    pub mr_enclave: [u8; 32],
    /// MRSIGNER
    pub mr_signer: [u8; 32],
    /// 最低 ISV SVN
    pub min_isv_svn: u16,
    /// 激活区块
    pub activated_at: u64,
    /// 废弃区块（None 表示当前版本）
    pub deprecated_at: Option<u64>,
    /// 强制升级截止区块
    pub force_upgrade_at: Option<u64>,
}

/// 当前接受的 MRENCLAVE 列表
#[pallet::storage]
#[pallet::getter(fn accepted_mr_enclaves)]
pub type AcceptedMrEnclaves<T: Config> = StorageValue<
    _,
    BoundedVec<EnclaveVersion, ConstU32<5>>,
    ValueQuery,
>;

/// 预发布版本（治理通过后等待激活）
#[pallet::storage]
#[pallet::getter(fn pending_version)]
pub type PendingVersion<T: Config> = StorageValue<
    _,
    EnclaveVersion,
    OptionQuery,
>;

impl<T: Config> Pallet<T> {
    /// 提议新版本（需要治理）
    #[pallet::call_index(40)]
    #[pallet::weight(T::WeightInfo::propose_version())]
    pub fn propose_version(
        origin: OriginFor<T>,
        new_version: EnclaveVersion,
    ) -> DispatchResult {
        // 需要技术委员会或治理通过
        T::VersionOrigin::ensure_origin(origin)?;

        // 验证版本号递增
        let current_versions = AcceptedMrEnclaves::<T>::get();
        if let Some(latest) = current_versions.last() {
            ensure!(
                new_version.version > latest.version,
                Error::<T>::InvalidVersion
            );
        }

        PendingVersion::<T>::put(new_version.clone());

        Self::deposit_event(Event::VersionProposed {
            version: new_version.version,
            mr_enclave: new_version.mr_enclave,
        });

        Ok(())
    }

    /// 激活新版本
    #[pallet::call_index(41)]
    #[pallet::weight(T::WeightInfo::activate_version())]
    pub fn activate_version(origin: OriginFor<T>) -> DispatchResult {
        T::VersionOrigin::ensure_origin(origin)?;

        let mut new_version = PendingVersion::<T>::take()
            .ok_or(Error::<T>::NoPendingVersion)?;

        new_version.activated_at = T::TimeProvider::now().as_secs();

        AcceptedMrEnclaves::<T>::try_mutate(|versions| {
            versions.try_push(new_version.clone())
        })?;

        Self::deposit_event(Event::VersionActivated {
            version: new_version.version,
        });

        Ok(())
    }

    /// 废弃旧版本
    #[pallet::call_index(42)]
    #[pallet::weight(T::WeightInfo::deprecate_version())]
    pub fn deprecate_version(
        origin: OriginFor<T>,
        version: u32,
        force_upgrade_blocks: T::BlockNumber,
    ) -> DispatchResult {
        T::VersionOrigin::ensure_origin(origin)?;

        AcceptedMrEnclaves::<T>::try_mutate(|versions| {
            let version_entry = versions
                .iter_mut()
                .find(|v| v.version == version)
                .ok_or(Error::<T>::VersionNotFound)?;

            let now = T::TimeProvider::now().as_secs();
            version_entry.deprecated_at = Some(now);

            let current_block = <frame_system::Pallet<T>>::block_number();
            version_entry.force_upgrade_at = Some(
                current_block.saturating_add(force_upgrade_blocks).saturated_into()
            );

            Ok(())
        })?;

        Self::deposit_event(Event::VersionDeprecated { version });

        Ok(())
    }

    /// 验证 MRENCLAVE 是否被接受
    pub fn is_mr_enclave_accepted(mr_enclave: &[u8; 32]) -> bool {
        let versions = AcceptedMrEnclaves::<T>::get();
        let now = T::TimeProvider::now().as_secs();

        versions.iter().any(|v| {
            v.mr_enclave == *mr_enclave &&
            v.deprecated_at.map_or(true, |d| d > now)
        })
    }
}

#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_initialize(now: T::BlockNumber) -> Weight {
        // ... 超时处理 ...

        // 检查强制升级
        Self::check_force_upgrades(now);

        Weight::zero()
    }
}

impl<T: Config> Pallet<T> {
    /// 检查并处理强制升级
    fn check_force_upgrades(now: T::BlockNumber) {
        let versions = AcceptedMrEnclaves::<T>::get();
        let now_u64: u64 = now.saturated_into();

        for version in versions.iter() {
            if let Some(force_at) = version.force_upgrade_at {
                if now_u64 >= force_at {
                    // 暂停使用旧版本的节点
                    Self::suspend_old_version_nodes(&version.mr_enclave);
                }
            }
        }
    }

    fn suspend_old_version_nodes(old_mr_enclave: &[u8; 32]) {
        for (account, node) in TeeNodes::<T>::iter() {
            if node.attestation.mr_enclave == *old_mr_enclave {
                TeeNodes::<T>::mutate(&account, |n| {
                    if let Some(node) = n {
                        node.status = TeeNodeStatus::Suspended;
                    }
                });

                ActiveNodes::<T>::mutate(|nodes| {
                    nodes.retain(|n| n != &account);
                });

                Self::deposit_event(Event::NodeSuspendedForUpgrade { node: account.clone() });
            }
        }
    }
}
```

---

## 16. ARM TrustZone 支持

### 16.1 架构对比

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Intel SGX vs ARM TrustZone                            │
├──────────────────────────────┬──────────────────────────────────────────┤
│        Intel SGX             │           ARM TrustZone                   │
├──────────────────────────────┼──────────────────────────────────────────┤
│                              │                                          │
│  ┌──────────────────────┐    │    ┌──────────────────────┐              │
│  │    Normal World      │    │    │    Normal World      │              │
│  │  ┌────────────────┐  │    │    │  ┌────────────────┐  │              │
│  │  │  Application   │  │    │    │  │  Rich OS       │  │              │
│  │  └───────┬────────┘  │    │    │  │  (Linux/Android)│ │              │
│  │          │           │    │    │  └───────┬────────┘  │              │
│  │  ┌───────▼────────┐  │    │    └──────────┼──────────┘              │
│  │  │   Enclave      │◄─┼────┼───────────────┼──────────►              │
│  │  │  (隔离内存)    │  │    │    ┌──────────▼──────────┐              │
│  │  └────────────────┘  │    │    │    Secure World     │              │
│  └──────────────────────┘    │    │  ┌────────────────┐  │              │
│                              │    │  │  Trusted App   │  │              │
│  特点：                       │    │  │  (TA)         │  │              │
│  • 应用级隔离                 │    │  └───────┬────────┘  │              │
│  • 多 Enclave 并行            │    │  ┌───────▼────────┐  │              │
│  • 内存加密                   │    │  │  Trusted OS    │  │              │
│  • EPID/DCAP 认证             │    │  │  (OP-TEE)     │  │              │
│                              │    │  └────────────────┘  │              │
│                              │    └──────────────────────┘              │
│                              │                                          │
│                              │    特点：                                 │
│                              │    • 系统级隔离                           │
│                              │    • 单一安全世界                         │
│                              │    • ARM 处理器原生支持                   │
│                              │    • GlobalPlatform TEE 认证              │
│                              │                                          │
└──────────────────────────────┴──────────────────────────────────────────┘
```

### 16.2 TrustZone TA 实现

```rust
// pallets/tee-enclave-tz/src/lib.rs

//! ARM TrustZone Trusted Application 实现

#![no_std]

use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session,
    Error, ErrorKind, Parameters, Result,
};

mod crypto;
mod divination;

/// TA UUID
pub const TA_UUID: &str = "8aaaf200-2450-11e4-abe2-0002a5d5c51b";

/// 命令 ID
#[repr(u32)]
pub enum Command {
    /// 初始化密钥
    InitKeys = 0,
    /// 获取公钥
    GetPublicKey = 1,
    /// 处理计算请求
    ProcessRequest = 2,
    /// 生成认证报告
    GenerateAttestation = 3,
}

/// TA 状态
pub struct TaContext {
    /// 签名密钥对
    signing_key: Option<Ed25519KeyPair>,
    /// ECDH 密钥对
    ecdh_key: Option<X25519KeyPair>,
    /// 是否已初始化
    initialized: bool,
}

static mut TA_CONTEXT: TaContext = TaContext {
    signing_key: None,
    ecdh_key: None,
    initialized: false,
};

#[ta_create]
fn create() -> Result<()> {
    // TA 创建时初始化
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut Parameters) -> Result<()> {
    // 会话打开
    Ok(())
}

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> {
    match Command::try_from(cmd_id) {
        Ok(Command::InitKeys) => cmd_init_keys(params),
        Ok(Command::GetPublicKey) => cmd_get_public_key(params),
        Ok(Command::ProcessRequest) => cmd_process_request(params),
        Ok(Command::GenerateAttestation) => cmd_generate_attestation(params),
        Err(_) => Err(Error::new(ErrorKind::BadParameters)),
    }
}

fn cmd_init_keys(params: &mut Parameters) -> Result<()> {
    unsafe {
        if TA_CONTEXT.initialized {
            return Ok(());
        }

        // 从安全存储加载或生成密钥
        let signing_key = crypto::load_or_generate_signing_key()?;
        let ecdh_key = crypto::load_or_generate_ecdh_key()?;

        TA_CONTEXT.signing_key = Some(signing_key);
        TA_CONTEXT.ecdh_key = Some(ecdh_key);
        TA_CONTEXT.initialized = true;
    }

    Ok(())
}

fn cmd_get_public_key(params: &mut Parameters) -> Result<()> {
    let mut output = params.param_mut::<MemRef>(0)?;

    unsafe {
        let ecdh_key = TA_CONTEXT.ecdh_key.as_ref()
            .ok_or(Error::new(ErrorKind::BadState))?;

        let pubkey = ecdh_key.public_key();
        output.buffer_mut().copy_from_slice(&pubkey);
    }

    Ok(())
}

fn cmd_process_request(params: &mut Parameters) -> Result<()> {
    // 获取输入
    let input = params.param::<MemRef>(0)?;
    let compute_type = params.param::<Value>(1)?.a();
    let mut output = params.param_mut::<MemRef>(2)?;
    let mut proof = params.param_mut::<MemRef>(3)?;

    unsafe {
        let ecdh_key = TA_CONTEXT.ecdh_key.as_ref()
            .ok_or(Error::new(ErrorKind::BadState))?;
        let signing_key = TA_CONTEXT.signing_key.as_ref()
            .ok_or(Error::new(ErrorKind::BadState))?;

        // 解密输入
        let plaintext = crypto::decrypt_input(input.buffer(), ecdh_key)?;

        // 执行计算
        let result = match compute_type {
            0 => divination::bazi::calculate(&plaintext)?,
            1 => divination::meihua::calculate(&plaintext)?,
            2 => divination::qimen::calculate(&plaintext)?,
            3 => divination::liuyao::calculate(&plaintext)?,
            4 => divination::ziwei::calculate(&plaintext)?,
            5 => divination::tarot::calculate(&plaintext)?,
            _ => return Err(Error::new(ErrorKind::BadParameters)),
        };

        // 加密输出
        let encrypted_result = crypto::encrypt_output(&result, ecdh_key)?;
        output.buffer_mut()[..encrypted_result.len()].copy_from_slice(&encrypted_result);

        // 生成证明
        let computation_proof = crypto::generate_proof(
            &plaintext,
            &result,
            signing_key,
        )?;
        proof.buffer_mut()[..computation_proof.len()].copy_from_slice(&computation_proof);
    }

    Ok(())
}

fn cmd_generate_attestation(params: &mut Parameters) -> Result<()> {
    let report_data = params.param::<MemRef>(0)?;
    let mut attestation = params.param_mut::<MemRef>(1)?;

    // 生成 GlobalPlatform 认证报告
    let att = crypto::generate_gp_attestation(report_data.buffer())?;
    attestation.buffer_mut()[..att.len()].copy_from_slice(&att);

    Ok(())
}

#[ta_close_session]
fn close_session() {
    // 会话关闭
}

#[ta_destroy]
fn destroy() {
    // TA 销毁时清理
    unsafe {
        TA_CONTEXT.signing_key = None;
        TA_CONTEXT.ecdh_key = None;
        TA_CONTEXT.initialized = false;
    }
}
```

### 16.3 TrustZone 节点注册

```rust
// pallets/tee-privacy/src/trustzone.rs

/// TrustZone 认证报告
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct TrustZoneAttestation {
    /// TA UUID
    pub ta_uuid: [u8; 16],
    /// TA 版本
    pub ta_version: u32,
    /// 设备 ID 哈希
    pub device_id_hash: [u8; 32],
    /// 时间戳
    pub timestamp: u64,
    /// 自定义数据
    pub report_data: [u8; 64],
    /// 签名 (由 OEM 私钥签名)
    pub signature: BoundedVec<u8, ConstU32<256>>,
    /// OEM 证书链
    pub cert_chain: BoundedVec<u8, ConstU32<4096>>,
}

impl<T: Config> Pallet<T> {
    /// 注册 TrustZone 节点
    #[pallet::call_index(50)]
    #[pallet::weight(T::WeightInfo::register_trustzone_node())]
    pub fn register_trustzone_node(
        origin: OriginFor<T>,
        enclave_pubkey: [u8; 32],
        tz_attestation: TrustZoneAttestation,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        ensure!(
            !TeeNodes::<T>::contains_key(&who),
            Error::<T>::NodeAlreadyRegistered
        );

        // 验证 TrustZone 认证
        Self::verify_trustzone_attestation(&tz_attestation)?;

        // 验证 TA UUID 在白名单中
        let expected_uuid = ExpectedTaUuid::<T>::get()
            .ok_or(Error::<T>::TaUuidNotSet)?;
        ensure!(
            tz_attestation.ta_uuid == expected_uuid,
            Error::<T>::InvalidTaUuid
        );

        // 转换为通用格式
        let attestation = TeeAttestation {
            tee_type: TeeType::ArmTrustZone,
            mr_enclave: tz_attestation.device_id_hash,  // 使用设备哈希作为标识
            mr_signer: [0u8; 32],  // TrustZone 不适用
            isv_prod_id: 0,
            isv_svn: tz_attestation.ta_version as u16,
            report_data: tz_attestation.report_data,
            ias_signature: tz_attestation.signature,
            timestamp: tz_attestation.timestamp,
        };

        let node = TeeNode {
            account: who.clone(),
            enclave_pubkey,
            attestation,
            registered_at: T::TimeProvider::now().as_secs(),
            status: TeeNodeStatus::Active,
        };

        TeeNodes::<T>::insert(&who, node);
        ActiveNodes::<T>::try_mutate(|nodes| nodes.try_push(who.clone()))?;

        Self::deposit_event(Event::TrustZoneNodeRegistered {
            node: who,
            ta_uuid: tz_attestation.ta_uuid,
        });

        Ok(())
    }

    /// 验证 TrustZone 认证
    fn verify_trustzone_attestation(
        attestation: &TrustZoneAttestation,
    ) -> DispatchResult {
        // 1. 验证证书链
        let root_cert = TrustedOemRootCert::<T>::get()
            .ok_or(Error::<T>::RootCertNotSet)?;

        Self::verify_cert_chain(&attestation.cert_chain, &root_cert)?;

        // 2. 验证签名
        let message = Self::construct_tz_message(attestation);
        Self::verify_tz_signature(&message, &attestation.signature, &attestation.cert_chain)?;

        // 3. 验证时间戳
        let now = T::TimeProvider::now().as_secs();
        ensure!(
            now.saturating_sub(attestation.timestamp) < T::AttestationValidity::get(),
            Error::<T>::AttestationExpired
        );

        Ok(())
    }
}
```

---

## 17. 审计日志系统

### 17.1 审计事件

```rust
// pallets/tee-privacy/src/audit.rs

/// 审计事件类型
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum AuditEventType {
    /// 节点注册
    NodeRegistered { node: AccountId, tee_type: TeeType },
    /// 节点注销
    NodeDeregistered { node: AccountId },
    /// 认证更新
    AttestationUpdated { node: AccountId },
    /// 请求提交
    RequestSubmitted { request_id: u64, requester: AccountId, compute_type: ComputeType },
    /// 请求完成
    RequestCompleted { request_id: u64, executor: AccountId, duration_blocks: u32 },
    /// 请求失败
    RequestFailed { request_id: u64, reason: FailureReason },
    /// 故障转移
    FailoverTriggered { request_id: u64, from_node: AccountId, to_node: AccountId },
    /// 节点惩罚
    NodeSlashed { node: AccountId, reason: SlashReason, amount: Balance },
    /// 版本升级
    VersionUpgrade { from_version: u32, to_version: u32 },
    /// 安全告警
    SecurityAlert { alert_type: SecurityAlertType, details: Vec<u8> },
}

/// 安全告警类型
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum SecurityAlertType {
    /// 认证验证失败
    AttestationVerificationFailed,
    /// 计算证明无效
    InvalidComputationProof,
    /// 可疑行为
    SuspiciousBehavior,
    /// 侧信道攻击检测
    SideChannelDetected,
}

/// 审计日志条目
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct AuditLogEntry<BlockNumber> {
    /// 序号
    pub seq: u64,
    /// 区块号
    pub block_number: BlockNumber,
    /// 时间戳
    pub timestamp: u64,
    /// 事件类型
    pub event_type: AuditEventType,
    /// 哈希（用于完整性验证）
    pub hash: [u8; 32],
    /// 前一条目哈希（链式验证）
    pub prev_hash: [u8; 32],
}

/// 审计日志存储
#[pallet::storage]
#[pallet::getter(fn audit_logs)]
pub type AuditLogs<T: Config> = StorageMap<
    _,
    Twox64Concat,
    u64,  // 序号
    AuditLogEntry<T::BlockNumber>,
    OptionQuery,
>;

/// 最新审计序号
#[pallet::storage]
#[pallet::getter(fn latest_audit_seq)]
pub type LatestAuditSeq<T: Config> = StorageValue<_, u64, ValueQuery>;

/// 审计日志根哈希（Merkle 根）
#[pallet::storage]
#[pallet::getter(fn audit_root)]
pub type AuditRoot<T: Config> = StorageValue<_, [u8; 32], ValueQuery>;

impl<T: Config> Pallet<T> {
    /// 记录审计事件
    pub(crate) fn record_audit_event(event_type: AuditEventType) {
        let seq = LatestAuditSeq::<T>::mutate(|s| {
            let current = *s;
            *s = s.saturating_add(1);
            current
        });

        let prev_hash = if seq > 0 {
            AuditLogs::<T>::get(seq - 1)
                .map(|e| e.hash)
                .unwrap_or([0u8; 32])
        } else {
            [0u8; 32]
        };

        let current_block = <frame_system::Pallet<T>>::block_number();
        let timestamp = T::TimeProvider::now().as_secs();

        // 计算条目哈希
        let hash = Self::compute_audit_hash(seq, current_block, timestamp, &event_type, &prev_hash);

        let entry = AuditLogEntry {
            seq,
            block_number: current_block,
            timestamp,
            event_type,
            hash,
            prev_hash,
        };

        AuditLogs::<T>::insert(seq, entry);

        // 定期更新 Merkle 根
        if seq % 100 == 0 {
            Self::update_audit_root();
        }
    }

    /// 计算审计哈希
    fn compute_audit_hash(
        seq: u64,
        block_number: T::BlockNumber,
        timestamp: u64,
        event_type: &AuditEventType,
        prev_hash: &[u8; 32],
    ) -> [u8; 32] {
        use sp_core::hashing::blake2_256;

        let mut data = Vec::new();
        data.extend_from_slice(&seq.to_le_bytes());
        data.extend_from_slice(&block_number.encode());
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(&event_type.encode());
        data.extend_from_slice(prev_hash);

        blake2_256(&data)
    }

    /// 更新审计 Merkle 根
    fn update_audit_root() {
        // 简化实现：使用最新哈希作为根
        // 生产环境应使用完整 Merkle 树
        let latest = LatestAuditSeq::<T>::get();
        if let Some(entry) = AuditLogs::<T>::get(latest.saturating_sub(1)) {
            AuditRoot::<T>::put(entry.hash);
        }
    }

    /// 验证审计日志完整性
    pub fn verify_audit_integrity(from_seq: u64, to_seq: u64) -> bool {
        let mut prev_hash = if from_seq > 0 {
            AuditLogs::<T>::get(from_seq - 1)
                .map(|e| e.hash)
                .unwrap_or([0u8; 32])
        } else {
            [0u8; 32]
        };

        for seq in from_seq..=to_seq {
            if let Some(entry) = AuditLogs::<T>::get(seq) {
                // 验证前一哈希
                if entry.prev_hash != prev_hash {
                    return false;
                }

                // 验证当前哈希
                let computed = Self::compute_audit_hash(
                    seq,
                    entry.block_number,
                    entry.timestamp,
                    &entry.event_type,
                    &prev_hash,
                );

                if entry.hash != computed {
                    return false;
                }

                prev_hash = entry.hash;
            } else {
                return false;
            }
        }

        true
    }
}
```

### 17.2 审计查询 RPC

```rust
// rpc/tee-privacy/src/audit.rs

#[rpc(server)]
pub trait TeeAuditApi<BlockHash> {
    /// 获取审计日志
    #[method(name = "teeAudit_getLogs")]
    fn get_logs(
        &self,
        from_seq: u64,
        to_seq: u64,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<AuditLogEntry>>;

    /// 验证审计完整性
    #[method(name = "teeAudit_verifyIntegrity")]
    fn verify_integrity(
        &self,
        from_seq: u64,
        to_seq: u64,
        at: Option<BlockHash>,
    ) -> RpcResult<bool>;

    /// 获取审计统计
    #[method(name = "teeAudit_getStats")]
    fn get_stats(
        &self,
        at: Option<BlockHash>,
    ) -> RpcResult<AuditStats>;

    /// 按事件类型过滤
    #[method(name = "teeAudit_filterByType")]
    fn filter_by_type(
        &self,
        event_type: String,
        limit: u32,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<AuditLogEntry>>;
}

/// 审计统计
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_entries: u64,
    pub nodes_registered: u64,
    pub requests_completed: u64,
    pub requests_failed: u64,
    pub failovers_triggered: u64,
    pub slashes_executed: u64,
    pub security_alerts: u64,
}
```

---

## 18. 性能基准测试

### 18.1 基准测试框架

```rust
// pallets/tee-privacy/src/benchmarking.rs

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_tee_node() {
        let caller: T::AccountId = whitelisted_caller();
        let enclave_pubkey = [1u8; 32];
        let attestation = mock_attestation::<T>();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()), enclave_pubkey, attestation);

        assert!(TeeNodes::<T>::contains_key(&caller));
    }

    #[benchmark]
    fn update_attestation() {
        let caller: T::AccountId = setup_tee_node::<T>();
        let new_attestation = mock_attestation::<T>();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()), new_attestation);
    }

    #[benchmark]
    fn submit_compute_request() {
        setup_tee_node::<T>();
        let caller: T::AccountId = whitelisted_caller();
        let compute_type = ComputeType::BaZi(BaZiParams::default());
        let encrypted_input = mock_encrypted_data();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), compute_type, encrypted_input, None);
    }

    #[benchmark]
    fn submit_compute_result() {
        let node = setup_tee_node::<T>();
        let request_id = setup_pending_request::<T>();
        let encrypted_output = mock_encrypted_data();
        let proof = mock_computation_proof::<T>(&node);

        #[extrinsic_call]
        _(RawOrigin::Signed(node), request_id, encrypted_output, proof);
    }

    #[benchmark]
    fn stake() {
        let caller: T::AccountId = whitelisted_caller();
        let amount = T::MinStake::get();
        T::Currency::make_free_balance_be(&caller, amount * 2u32.into());

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()), amount);

        assert!(NodeStakes::<T>::contains_key(&caller));
    }

    #[benchmark]
    fn submit_batch_request(n: Linear<1, 50>) {
        setup_tee_node::<T>();
        let caller: T::AccountId = whitelisted_caller();

        let requests: BoundedVec<_, _> = (0..n)
            .map(|_| ComputeRequestItem {
                compute_type: ComputeType::MeiHua(MeiHuaParams::default()),
                encrypted_input: mock_encrypted_data(),
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        T::Currency::make_free_balance_be(&caller, u32::MAX.into());

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), requests);
    }

    #[benchmark]
    fn process_timeout_queue(n: Linear<1, 100>) {
        // 设置 n 个待超时请求
        for i in 0..n {
            setup_timeout_request::<T>(i as u64);
        }

        let now = frame_system::Pallet::<T>::block_number();

        #[block]
        {
            Pallet::<T>::on_initialize(now);
        }
    }
}
```

### 18.2 性能基准数据

| 操作 | 时间复杂度 | 读 | 写 | 基准 Gas |
|------|------------|-----|-----|----------|
| `register_tee_node` | O(1) | 3 | 3 | 50,000 |
| `update_attestation` | O(1) | 2 | 1 | 30,000 |
| `submit_compute_request` | O(1) | 3 | 4 | 60,000 |
| `submit_compute_result` | O(1) | 4 | 3 | 70,000 |
| `stake` | O(1) | 2 | 2 | 40,000 |
| `submit_batch_request` | O(n) | 2 + n | 3 + n | 40,000 + 20,000*n |
| `on_initialize` (超时) | O(k) | k | 2k | 10,000*k |

*注: k = 当前区块超时请求数*

### 18.3 Enclave 性能测试

```bash
#!/bin/bash
# scripts/benchmark-enclave.sh

echo "=== Enclave 性能基准测试 ==="

# 测试加密性能
echo "1. AES-256-GCM 加密性能"
./enclave-bench encrypt --iterations 10000 --size 1KB
./enclave-bench encrypt --iterations 1000 --size 10KB
./enclave-bench encrypt --iterations 100 --size 100KB

# 测试 ECDH 性能
echo "2. X25519 ECDH 性能"
./enclave-bench ecdh --iterations 10000

# 测试签名性能
echo "3. Ed25519 签名性能"
./enclave-bench sign --iterations 10000 --size 256B

# 测试占卜计算性能
echo "4. 占卜计算性能"
./enclave-bench compute --type bazi --iterations 1000
./enclave-bench compute --type meihua --iterations 1000
./enclave-bench compute --type qimen --iterations 100
./enclave-bench compute --type ziwei --iterations 100

# 测试认证生成性能
echo "5. 远程认证生成性能"
./enclave-bench attestation --iterations 100

# 输出结果
echo "=== 测试完成 ==="
```

---

## 附录

### A. 术语表

| 术语 | 英文 | 说明 |
|------|------|------|
| 可信执行环境 | TEE | Trusted Execution Environment |
| 远程认证 | RA | Remote Attestation |
| 安全区 | Enclave | SGX 安全隔离区域 |
| 度量值 | Measurement | MRENCLAVE/MRSIGNER |
| 密封 | Sealing | 数据持久化加密 |

### B. 参考文档

- [Intel SGX Developer Reference](https://download.01.org/intel-sgx/sgx-linux/2.18/docs/)
- [Apache Teaclave SGX SDK](https://github.com/apache/incubator-teaclave-sgx-sdk)
- [Substrate Off-chain Workers](https://docs.substrate.io/reference/how-to-guides/offchain-workers/)

### C. 更新日志

| 版本 | 日期 | 变更 |
|------|------|------|
| 1.2.0 | 2026-01-06 | 新增第 11-18 节：经济激励机制（质押/奖励/惩罚）、故障转移机制（超时队列优化）、DCAP 认证支持、批处理优化、Enclave 升级机制、ARM TrustZone 支持、审计日志系统、性能基准测试 |
| 1.1.0 | 2026-01-06 | 明确模块独立性设计：pallet-tee-privacy 与 pallet-divination-privacy 相互独立，可单独部署或同时使用；新增第 2 节"模块独立性设计"；移除迁移方案（两模块无迁移关系） |
| 1.0.0 | 2026-01-01 | 初始版本 |

---

*文档结束*
