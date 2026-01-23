# OCW + TEE 通用架构设计

> 文档日期: 2026-01-23  
> 版本: 1.0  
> 状态: 设计方案

---

## 概述

本文档定义 **OCW + TEE 通用架构**，为所有占卜模块提供统一的隐私计算基础设施。

### 适用模块

| 模块 | 中文名 | 计算复杂度 | 敏感数据 |
|------|--------|------------|----------|
| `bazi` | 八字 | 中 | 出生时间 |
| `qimen` | 奇门遁甲 | 高 | 占问事宜 |
| `meihua` | 梅花易数 | 低 | 起卦数字/时间 |
| `liuyao` | 六爻 | 中 | 摇卦结果 |
| `ziwei` | 紫微斗数 | 高 | 出生时间 |
| `daliuren` | 大六壬 | 高 | 占问时间 |
| `xiaoliuren` | 小六壬 | 低 | 起卦时间 |
| `tarot` | 塔罗 | 低 | 抽牌结果 |

---

## 架构分层

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        应用层（各占卜模块）                               │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐           │
│  │  BaZi   │ │  Qimen  │ │ MeiHua  │ │ LiuYao  │ │  ZiWei  │  ...      │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘           │
│       │           │           │           │           │                 │
│       └───────────┴───────────┴───────────┴───────────┘                 │
│                               │                                          │
│                               ▼                                          │
├─────────────────────────────────────────────────────────────────────────┤
│                      通用层（pallet-divination-ocw-tee）                 │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │ 请求管理    │  │ OCW 调度    │  │ TEE 通信    │  │ IPFS 存储   │    │
│  │ (Pending)   │  │ (Scheduler) │  │ (Client)    │  │ (Uploader)  │    │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘    │
│                                                                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │ 隐私模式    │  │ 密钥管理    │  │ 重试机制    │  │ 事件通知    │    │
│  │ (Privacy)   │  │ (KeyMgmt)   │  │ (Retry)     │  │ (Events)    │    │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
│                               │                                          │
│                               ▼                                          │
├─────────────────────────────────────────────────────────────────────────┤
│                      基础层（现有模块）                                   │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │ pallet-tee-     │  │ pallet-stardust │  │ pallet-         │         │
│  │ privacy         │  │ -ipfs           │  │ divination-     │         │
│  │ (TEE 节点管理)  │  │ (IPFS 集成)     │  │ privacy         │         │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 通用模块设计

### 模块路径

```
pallets/divination/ocw-tee/
├── Cargo.toml
├── src/
│   ├── lib.rs           # 主模块
│   ├── types.rs         # 通用类型定义
│   ├── traits.rs        # 可扩展 trait
│   ├── ocw.rs           # OCW 调度逻辑
│   ├── tee_client.rs    # TEE HTTP 客户端
│   ├── ipfs.rs          # IPFS 上传/PIN
│   ├── retry.rs         # 重试机制
│   └── tests.rs         # 测试
```

---

## 通用类型定义 (types.rs)

```rust
//! # OCW + TEE 通用类型定义
//!
//! 本模块定义所有占卜模块共享的类型。

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use frame_support::traits::ConstU32;
use scale_info::TypeInfo;
use sp_std::prelude::*;

// ==================== 隐私模式 ====================

/// 隐私模式（所有占卜模块通用）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum PrivacyMode {
    /// 公开模式：链上存储索引，IPFS 明文
    #[default]
    Public = 0,
    /// 加密模式：链上存储索引，IPFS 加密
    Encrypted = 1,
    /// 私密模式：链上不存储索引，IPFS 加密
    Private = 2,
}

impl PrivacyMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Encrypted => "encrypted",
            Self::Private => "private",
        }
    }
    
    pub fn requires_tee(&self) -> bool {
        !matches!(self, Self::Public)
    }
}

// ==================== 请求状态 ====================

/// 请求状态（所有占卜模块通用）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum RequestStatus {
    /// 待处理
    #[default]
    Pending = 0,
    /// 处理中
    Processing = 1,
    /// 已完成
    Completed = 2,
    /// 失败（可重试）
    Failed = 3,
    /// 超时
    Timeout = 4,
}

// ==================== 加密数据 ====================

/// 加密数据（所有占卜模块通用）
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxLen))]
pub struct EncryptedData<MaxLen: frame_support::traits::Get<u32>> {
    /// 密文
    pub ciphertext: BoundedVec<u8, MaxLen>,
    /// 随机数
    pub nonce: [u8; 24],
    /// 发送方公钥
    pub sender_pubkey: [u8; 32],
}

/// 默认加密数据长度（256 字节，适用于大部分模块）
pub type DefaultEncryptedData = EncryptedData<ConstU32<256>>;

/// 大型加密数据长度（512 字节，适用于奇门等复杂输入）
pub type LargeEncryptedData = EncryptedData<ConstU32<512>>;

// ==================== 生成信息 ====================

/// 生成信息（所有占卜模块通用）
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum GenerationInfo<AccountId: Clone + Encode + Decode + TypeInfo + MaxEncodedLen> {
    /// OCW 生成（公开模式）
    Ocw,
    /// TEE 生成（加密/私密模式）
    Tee {
        /// TEE 节点
        node: AccountId,
        /// 计算证明
        proof: ComputationProof,
    },
}

/// 计算证明
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ComputationProof {
    /// MRENCLAVE（Enclave 代码哈希）
    pub mrenclave: [u8; 32],
    /// 计算时间戳
    pub timestamp: u64,
    /// Enclave 签名
    pub signature: [u8; 64],
}

// ==================== 占卜类型 ====================

/// 占卜类型枚举（用于 TEE 路由）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum DivinationType {
    /// 八字
    BaZi = 0,
    /// 奇门遁甲
    QiMen = 1,
    /// 梅花易数
    MeiHua = 2,
    /// 六爻
    LiuYao = 3,
    /// 紫微斗数
    ZiWei = 4,
    /// 大六壬
    DaLiuRen = 5,
    /// 小六壬
    XiaoLiuRen = 6,
    /// 塔罗
    Tarot = 7,
}

impl DivinationType {
    /// 获取 TEE HTTP 端点路径
    pub fn tee_endpoint(&self) -> &'static str {
        match self {
            Self::BaZi => "/compute/bazi",
            Self::QiMen => "/compute/qimen",
            Self::MeiHua => "/compute/meihua",
            Self::LiuYao => "/compute/liuyao",
            Self::ZiWei => "/compute/ziwei",
            Self::DaLiuRen => "/compute/daliuren",
            Self::XiaoLiuRen => "/compute/xiaoliuren",
            Self::Tarot => "/compute/tarot",
        }
    }
    
    /// 获取推荐超时时间（区块数）
    pub fn recommended_timeout(&self) -> u32 {
        match self {
            Self::BaZi => 100,       // ~10 分钟
            Self::QiMen => 150,      // ~15 分钟（计算复杂）
            Self::MeiHua => 80,      // ~8 分钟
            Self::LiuYao => 100,     // ~10 分钟
            Self::ZiWei => 150,      // ~15 分钟（计算复杂）
            Self::DaLiuRen => 150,   // ~15 分钟（计算复杂）
            Self::XiaoLiuRen => 60,  // ~6 分钟（简单）
            Self::Tarot => 60,       // ~6 分钟（简单）
        }
    }
}

// ==================== 通用链上存储 ====================

/// 通用链上存储结构（模板）
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(Index))]
pub struct DivinationOnChain<AccountId, BlockNumber, Index>
where
    AccountId: Clone + Encode + Decode + TypeInfo + MaxEncodedLen,
    BlockNumber: Clone + Encode + Decode + TypeInfo + MaxEncodedLen,
    Index: Clone + Encode + Decode + TypeInfo + MaxEncodedLen,
{
    /// 所有者
    pub owner: AccountId,
    
    /// 隐私模式
    pub privacy_mode: PrivacyMode,
    
    /// 占卜类型特定索引（Private 模式下为 None）
    /// 例如：BaZi 的 SiZhuIndex，QiMen 的 ChartIndex
    pub type_index: Option<Index>,
    
    /// JSON 清单 CID（IPFS）
    pub manifest_cid: BoundedVec<u8, ConstU32<64>>,
    
    /// 清单哈希（用于验证完整性）
    pub manifest_hash: [u8; 32],
    
    /// 生成方式
    pub generation: GenerationInfo<AccountId>,
    
    /// 版本号
    pub version: u32,
    
    /// 创建区块
    pub created_at: BlockNumber,
    
    /// 更新区块
    pub updated_at: BlockNumber,
}

// ==================== 通用待处理请求 ====================

/// 通用待处理请求结构（模板）
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(InputData))]
pub struct PendingRequest<AccountId, BlockNumber, InputData>
where
    AccountId: Clone + Encode + Decode + TypeInfo + MaxEncodedLen,
    BlockNumber: Clone + Encode + Decode + TypeInfo + MaxEncodedLen,
    InputData: Clone + Encode + Decode + TypeInfo + MaxEncodedLen,
{
    /// 请求者
    pub requester: AccountId,
    
    /// 占卜类型
    pub divination_type: DivinationType,
    
    /// 输入数据（明文或密文）
    pub input_data: InputData,
    
    /// 用户公钥（用于加密返回结果）
    pub user_pubkey: Option<[u8; 32]>,
    
    /// 隐私模式
    pub privacy_mode: PrivacyMode,
    
    /// 分配的 TEE 节点
    pub assigned_node: Option<AccountId>,
    
    /// 请求状态
    pub status: RequestStatus,
    
    /// 重试次数
    pub retry_count: u8,
    
    /// 创建区块
    pub created_at: BlockNumber,
}

// ==================== TEE 响应 ====================

/// TEE 计算响应（通用）
#[derive(Clone, Debug, Encode, Decode)]
pub struct TeeComputeResponse {
    /// 加密的 JSON 清单
    pub encrypted_manifest: Vec<u8>,
    /// 清单哈希
    pub manifest_hash: [u8; 32],
    /// 类型特定索引（编码后）
    pub type_index: Option<Vec<u8>>,
    /// 计算证明
    pub computation_proof: ComputationProof,
    /// Enclave 签名
    pub enclave_signature: [u8; 64],
}
```

---

## 通用 Trait 定义 (traits.rs)

```rust
//! # OCW + TEE 通用 Trait 定义
//!
//! 定义各占卜模块需要实现的 trait。

use crate::types::*;
use frame_support::dispatch::DispatchResult;
use sp_std::prelude::*;

/// 占卜计算 Trait
///
/// 各占卜模块需要实现此 trait 来定义自己的计算逻辑。
pub trait DivinationCompute {
    /// 输入类型（明文）
    type PlainInput: Clone + Encode + Decode + TypeInfo + MaxEncodedLen;
    
    /// 索引类型（链上存储的最小化数据）
    type Index: Clone + Encode + Decode + TypeInfo + MaxEncodedLen;
    
    /// 计算结果类型
    type Result: Clone + Encode + Decode;
    
    /// 占卜类型
    fn divination_type() -> DivinationType;
    
    /// 执行计算（Public 模式下 OCW 直接调用）
    fn compute(input: &Self::PlainInput) -> Result<Self::Result, &'static str>;
    
    /// 从计算结果提取索引
    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index>;
    
    /// 生成 JSON 清单
    fn generate_manifest(
        input: &Self::PlainInput,
        result: &Self::Result,
        privacy_mode: PrivacyMode,
    ) -> Result<Vec<u8>, &'static str>;
}

/// OCW 处理器 Trait
///
/// 通用 OCW 处理逻辑，各模块可复用。
pub trait OcwProcessor<T: frame_system::Config> {
    /// 处理待处理请求
    fn process_pending_requests(block_number: T::BlockNumber);
    
    /// 处理 Public 模式请求
    fn process_public_request<D: DivinationCompute>(
        request_id: u64,
        input: &D::PlainInput,
    ) -> Result<ProcessResult, &'static str>;
    
    /// 处理 TEE 模式请求
    fn process_tee_request(
        request_id: u64,
        divination_type: DivinationType,
        encrypted_input: &EncryptedData<ConstU32<512>>,
        user_pubkey: &[u8; 32],
        privacy_mode: PrivacyMode,
    ) -> Result<ProcessResult, &'static str>;
}

/// 处理结果
#[derive(Clone, Debug)]
pub struct ProcessResult {
    pub manifest_cid: Vec<u8>,
    pub manifest_hash: [u8; 32],
    pub type_index: Option<Vec<u8>>,
    pub proof: Option<ComputationProof>,
}

/// TEE 客户端 Trait
pub trait TeeClient {
    /// 调用 TEE 节点
    fn call_tee(
        endpoint: &str,
        divination_type: DivinationType,
        encrypted_input: &[u8],
        user_pubkey: &[u8; 32],
        privacy_mode: PrivacyMode,
    ) -> Result<TeeComputeResponse, &'static str>;
}

/// IPFS 客户端 Trait
pub trait IpfsClient {
    /// 上传数据到 IPFS
    fn upload(data: &[u8]) -> Result<Vec<u8>, &'static str>;
    
    /// 请求 PIN
    fn pin(cid: &[u8]) -> Result<(), &'static str>;
    
    /// 取消 PIN
    fn unpin(cid: &[u8]) -> Result<(), &'static str>;
}
```

---

## OCW 调度逻辑 (ocw.rs)

```rust
//! # OCW 通用调度逻辑

use crate::traits::*;
use crate::types::*;
use frame_support::traits::Get;
use sp_runtime::offchain::http;

/// 最大重试次数
pub const MAX_RETRY_COUNT: u8 = 3;

/// 通用 OCW 处理器实现
pub struct GenericOcwProcessor<T: Config>(sp_std::marker::PhantomData<T>);

impl<T: Config> GenericOcwProcessor<T> {
    /// 主处理循环
    pub fn offchain_worker(block_number: T::BlockNumber) {
        // 检查处理间隔
        if block_number % T::OcwInterval::get() != Zero::zero() {
            return;
        }
        
        // 遍历所有待处理请求
        for (request_id, request) in PendingRequests::<T>::iter() {
            Self::process_single_request(request_id, request, block_number);
        }
    }
    
    /// 处理单个请求
    fn process_single_request(
        request_id: u64,
        request: PendingRequest<T::AccountId, T::BlockNumber, Vec<u8>>,
        block_number: T::BlockNumber,
    ) {
        // 跳过已完成
        if request.status == RequestStatus::Completed {
            return;
        }
        
        // 检查超时
        let timeout = request.divination_type.recommended_timeout();
        if Self::is_timeout(&request, block_number, timeout) {
            Self::submit_status_update(request_id, RequestStatus::Timeout);
            return;
        }
        
        // 检查重试次数
        if request.retry_count >= MAX_RETRY_COUNT {
            Self::submit_status_update(request_id, RequestStatus::Failed);
            return;
        }
        
        // 标记处理中
        if request.status == RequestStatus::Pending {
            Self::submit_status_update(request_id, RequestStatus::Processing);
        }
        
        // 根据隐私模式处理
        let result = if request.privacy_mode == PrivacyMode::Public {
            Self::process_public(request_id, &request)
        } else {
            Self::process_tee(request_id, &request)
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
    
    /// 处理 TEE 请求（通用）
    fn process_tee(
        request_id: u64,
        request: &PendingRequest<T::AccountId, T::BlockNumber, Vec<u8>>,
    ) -> Result<ProcessResult, &'static str> {
        let tee_node = TeeNodes::<T>::get(&request.assigned_node.as_ref().unwrap())
            .ok_or("TEE node not found")?;
        
        let user_pubkey = request.user_pubkey.as_ref().ok_or("User pubkey required")?;
        
        // 构建 HTTP 请求
        let endpoint = format!(
            "{}{}",
            tee_node.endpoint,
            request.divination_type.tee_endpoint()
        );
        
        let body = serde_json::json!({
            "encrypted_input": hex::encode(&request.input_data),
            "user_pubkey": hex::encode(user_pubkey),
            "privacy_mode": request.privacy_mode as u8,
            "divination_type": request.divination_type as u8,
        });
        
        // 发送请求
        let http_request = http::Request::post(&endpoint, body.to_string().into_bytes());
        let response = http_request.send().map_err(|_| "TEE request failed")?;
        let response_body = response.body().collect::<Vec<u8>>();
        
        let tee_response: TeeComputeResponse = 
            serde_json::from_slice(&response_body).map_err(|_| "Parse failed")?;
        
        // 上传到 IPFS
        let cid = T::IpfsClient::upload(&tee_response.encrypted_manifest)?;
        T::IpfsClient::pin(&cid)?;
        
        Ok(ProcessResult {
            manifest_cid: cid,
            manifest_hash: tee_response.manifest_hash,
            type_index: tee_response.type_index,
            proof: Some(tee_response.computation_proof),
        })
    }
}
```

---

## 各模块集成示例

### BaZi 模块集成

```rust
// pallets/divination/bazi/src/lib.rs

use pallet_divination_ocw_tee::{
    traits::{DivinationCompute, ProcessResult},
    types::{DivinationType, PrivacyMode, EncryptedData, GenerationInfo},
};

/// 八字计算实现
impl DivinationCompute for Pallet<T> {
    type PlainInput = BaziInputPlain;
    type Index = SiZhuIndex;
    type Result = BaziChart;
    
    fn divination_type() -> DivinationType {
        DivinationType::BaZi
    }
    
    fn compute(input: &Self::PlainInput) -> Result<Self::Result, &'static str> {
        // 调用现有的八字计算逻辑
        let sizhu = calculate_sizhu(input.year, input.month, input.day, input.hour);
        let dayun = calculate_dayun(&sizhu, input.gender, input.year);
        let wuxing = calculate_wuxing_strength(&sizhu);
        
        Ok(BaziChart { sizhu, dayun, wuxing, ... })
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
    ) -> Result<Vec<u8>, &'static str> {
        let manifest = serde_json::json!({
            "version": "1.0",
            "schema": "bazi-manifest-v1",
            "sizhu": sizhu_to_json(&result.sizhu),
            "dayun": dayun_to_json(&result.dayun),
            // ...
        });
        
        Ok(manifest.to_string().into_bytes())
    }
}

// Extrinsic 简化
#[pallet::call_index(0)]
pub fn create_bazi_public(
    origin: OriginFor<T>,
    year: u16, month: u8, day: u8, hour: u8,
    gender: Gender,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // 使用通用模块创建请求
    pallet_divination_ocw_tee::Pallet::<T>::create_request(
        who,
        DivinationType::BaZi,
        InputData::Plaintext(BaziInputPlain { year, month, day, hour, gender, longitude: None }),
        None,  // user_pubkey
        PrivacyMode::Public,
    )
}

#[pallet::call_index(1)]
pub fn create_bazi(
    origin: OriginFor<T>,
    encrypted_input: EncryptedData<ConstU32<256>>,
    user_pubkey: [u8; 32],
    privacy_mode: PrivacyMode,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    ensure!(privacy_mode != PrivacyMode::Public, Error::<T>::UsePublicExtrinsic);
    
    pallet_divination_ocw_tee::Pallet::<T>::create_request(
        who,
        DivinationType::BaZi,
        InputData::Encrypted(encrypted_input),
        Some(user_pubkey),
        privacy_mode,
    )
}
```

### QiMen 模块集成

```rust
// pallets/divination/qimen/src/lib.rs

impl DivinationCompute for Pallet<T> {
    type PlainInput = QimenInputPlain;
    type Index = QimenChartIndex;
    type Result = QimenChart;
    
    fn divination_type() -> DivinationType {
        DivinationType::QiMen
    }
    
    fn compute(input: &Self::PlainInput) -> Result<Self::Result, &'static str> {
        // 调用现有的奇门排盘逻辑
        let ganzhi = calculate_ganzhi_from_timestamp(input.timestamp);
        let palaces = arrange_palaces(...);
        
        Ok(QimenChart { ganzhi, palaces, ... })
    }
    
    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index> {
        match privacy_mode {
            PrivacyMode::Private => None,
            _ => Some(QimenChartIndex {
                dun_type: result.dun_type,
                ju_number: result.ju_number,
                // ...
            }),
        }
    }
    
    // ...
}
```

---

## 前端通用 SDK

```typescript
// @stardust/divination-sdk

import nacl from 'tweetnacl';

/**
 * 通用占卜客户端
 */
export class DivinationClient {
  private api: ApiPromise;
  private keyPair: nacl.BoxKeyPair;
  
  constructor(api: ApiPromise, keyPair: nacl.BoxKeyPair) {
    this.api = api;
    this.keyPair = keyPair;
  }
  
  /**
   * 创建占卜请求（通用）
   */
  async create<T>(
    divinationType: DivinationType,
    input: T,
    privacyMode: PrivacyMode,
  ): Promise<string> {
    if (privacyMode === PrivacyMode.Public) {
      return this.createPublic(divinationType, input);
    } else {
      return this.createPrivate(divinationType, input, privacyMode);
    }
  }
  
  /**
   * 创建公开请求
   */
  private async createPublic<T>(
    divinationType: DivinationType,
    input: T,
  ): Promise<string> {
    const extrinsic = this.getPublicExtrinsic(divinationType, input);
    return this.signAndSend(extrinsic);
  }
  
  /**
   * 创建加密请求
   */
  private async createPrivate<T>(
    divinationType: DivinationType,
    input: T,
    privacyMode: PrivacyMode,
  ): Promise<string> {
    // 1. 获取 TEE 公钥
    const enclavePubkey = await this.getTeePublicKey();
    
    // 2. 加密输入
    const plaintext = JSON.stringify(input);
    const nonce = nacl.randomBytes(24);
    const encrypted = nacl.box(
      new TextEncoder().encode(plaintext),
      nonce,
      enclavePubkey,
      this.keyPair.secretKey
    );
    
    // 3. 构建加密数据
    const encryptedData = {
      ciphertext: Array.from(encrypted),
      nonce: Array.from(nonce),
      sender_pubkey: Array.from(this.keyPair.publicKey),
    };
    
    // 4. 提交
    const extrinsic = this.getPrivateExtrinsic(
      divinationType,
      encryptedData,
      Array.from(this.keyPair.publicKey),
      privacyMode,
    );
    
    return this.signAndSend(extrinsic);
  }
  
  /**
   * 查看占卜结果（通用）
   */
  async view(
    divinationType: DivinationType,
    chartId: number,
  ): Promise<any> {
    // 1. 获取链上数据
    const onChain = await this.getOnChainData(divinationType, chartId);
    
    // 2. 从 IPFS 获取清单
    const manifest = await this.fetchFromIpfs(onChain.manifest_cid);
    
    // 3. 如果加密，解密
    if (onChain.privacy_mode !== PrivacyMode.Public) {
      return this.decryptManifest(manifest);
    }
    
    return manifest;
  }
  
  /**
   * 解密清单
   */
  private decryptManifest(encryptedManifest: any): any {
    const decrypted = nacl.box.open(
      new Uint8Array(encryptedManifest.ciphertext),
      new Uint8Array(encryptedManifest.nonce),
      new Uint8Array(encryptedManifest.enclave_pubkey),
      this.keyPair.secretKey
    );
    
    if (!decrypted) {
      throw new Error('Decryption failed');
    }
    
    return JSON.parse(new TextDecoder().decode(decrypted));
  }
  
  // ... 辅助方法
}

// ========== 类型特定客户端 ==========

/**
 * 八字客户端
 */
export class BaziClient extends DivinationClient {
  async createBazi(input: BaziInput, privacyMode: PrivacyMode) {
    return this.create(DivinationType.BaZi, input, privacyMode);
  }
  
  async viewBazi(chartId: number) {
    return this.view(DivinationType.BaZi, chartId);
  }
}

/**
 * 奇门客户端
 */
export class QimenClient extends DivinationClient {
  async createQimen(input: QimenInput, privacyMode: PrivacyMode) {
    return this.create(DivinationType.QiMen, input, privacyMode);
  }
  
  async viewQimen(chartId: number) {
    return this.view(DivinationType.QiMen, chartId);
  }
}

// ... 其他模块客户端
```

---

## 配置参数

```rust
// runtime/src/lib.rs

parameter_types! {
    /// OCW 处理间隔（每 5 个区块）
    pub const OcwInterval: u32 = 5;
    
    /// 最大重试次数
    pub const MaxRetryCount: u8 = 3;
    
    /// TEE 节点最大数量
    pub const MaxTeeNodes: u32 = 100;
    
    /// 默认 PIN 策略
    pub const DefaultPinPolicy: PinPolicy = PinPolicy::KeepRecent(3);
}

impl pallet_divination_ocw_tee::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OcwInterval = OcwInterval;
    type MaxRetryCount = MaxRetryCount;
    type TeePrivacy = TeePrivacy;
    type IpfsClient = StardustIpfs;
}
```

---

## 实现路线图

### Phase 1: 通用模块基础 (2周)

1. 创建 `pallet-divination-ocw-tee` 模块
2. 实现通用类型定义
3. 实现通用 trait
4. 实现 OCW 调度逻辑
5. 实现 TEE 客户端
6. 实现 IPFS 客户端

### Phase 2: 八字模块迁移 (1周)

1. 八字模块实现 `DivinationCompute` trait
2. 迁移现有 extrinsic
3. 测试验证

### Phase 3: 奇门模块迁移 (1周)

1. 奇门模块实现 `DivinationCompute` trait
2. 迁移现有 extrinsic
3. 测试验证

### Phase 4: 其他模块迁移 (3周)

1. 梅花易数
2. 六爻
3. 紫微斗数
4. 大六壬
5. 小六壬
6. 塔罗

### Phase 5: 前端 SDK (2周)

1. 通用 DivinationClient
2. 各模块特定客户端
3. 文档和示例

---

## 总结

**通用 OCW + TEE 架构优势**：

```
┌────────────────────────────────────────────────────────────────┐
│  代码复用                                                       │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. 通用类型：PrivacyMode, RequestStatus, EncryptedData        │
│     - 所有模块共享，无需重复定义                                │
│                                                                 │
│  2. 通用逻辑：OCW 调度、TEE 通信、IPFS 上传                     │
│     - 一次实现，多处复用                                        │
│                                                                 │
│  3. 通用 Trait：DivinationCompute                               │
│     - 各模块只需实现计算逻辑                                    │
│     - 框架处理请求管理、重试、超时                              │
│                                                                 │
│  4. 前端 SDK：统一 API                                          │
│     - create(type, input, privacy)                             │
│     - view(type, chartId)                                      │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

**代码量对比**：

| 模块 | 无通用架构 | 有通用架构 | 节省 |
|------|------------|------------|------|
| 八字 | ~1000 行 | ~300 行 | 70% |
| 奇门 | ~1200 行 | ~400 行 | 67% |
| 其他6个模块 | ~6000 行 | ~1500 行 | 75% |
| **总计** | ~8200 行 | ~2200 行 | **73%** |

---

## 模块注册机制（插件化设计）

为支持任意数量的占卜模块动态扩展，采用**注册表模式**。

### 设计原则

1. **开放封闭原则**：对扩展开放，对修改封闭
2. **零代码侵入**：新模块无需修改通用层代码
3. **运行时注册**：模块在 runtime 配置时自动注册
4. **类型安全**：编译时检查模块实现

### 架构图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         模块注册机制                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    DivinationRegistry                            │    │
│  │                    (模块注册表)                                  │    │
│  ├─────────────────────────────────────────────────────────────────┤    │
│  │  register<M: DivinationModule>()                                │    │
│  │  get_module(type: DivinationType) -> Option<&dyn Module>        │    │
│  │  list_modules() -> Vec<DivinationType>                          │    │
│  │  is_registered(type: DivinationType) -> bool                    │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                              │                                           │
│              ┌───────────────┼───────────────┬───────────────┐          │
│              ▼               ▼               ▼               ▼          │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ ┌────┐    │
│  │ BaZiModule      │ │ QiMenModule     │ │ ZiWeiModule     │ │ ...│    │
│  │ impl Module     │ │ impl Module     │ │ impl Module     │ │    │    │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘ └────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 核心 Trait 定义

```rust
//! # 模块注册机制
//!
//! 支持任意占卜模块的动态注册和扩展。

use crate::types::*;
use frame_support::dispatch::DispatchResult;
use sp_std::prelude::*;

/// 占卜模块 Trait（所有模块必须实现）
///
/// 这是模块注册的核心接口，定义了模块需要提供的所有能力。
pub trait DivinationModule<T: Config> {
    /// 模块唯一标识
    const MODULE_ID: DivinationType;
    
    /// 模块名称（用于日志和调试）
    const MODULE_NAME: &'static str;
    
    /// 模块版本
    const VERSION: u32;
    
    /// 输入类型（明文）
    type PlainInput: Clone + Encode + Decode + TypeInfo + MaxEncodedLen;
    
    /// 索引类型（链上存储的最小化数据）
    type Index: Clone + Encode + Decode + TypeInfo + MaxEncodedLen;
    
    /// 计算结果类型
    type Result: Clone + Encode + Decode;
    
    /// 配置类型（模块特定配置）
    type ModuleConfig: ModuleConfiguration;
    
    // ==================== 核心方法 ====================
    
    /// 执行计算
    fn compute(input: &Self::PlainInput) -> Result<Self::Result, ModuleError>;
    
    /// 从计算结果提取索引
    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index>;
    
    /// 生成 JSON 清单
    fn generate_manifest(
        input: &Self::PlainInput,
        result: &Self::Result,
        privacy_mode: PrivacyMode,
    ) -> Result<Vec<u8>, ModuleError>;
    
    /// 验证输入有效性
    fn validate_input(input: &Self::PlainInput) -> Result<(), ModuleError>;
    
    // ==================== 可选方法（有默认实现）====================
    
    /// 获取推荐超时时间（区块数）
    fn recommended_timeout() -> u32 {
        Self::MODULE_ID.recommended_timeout()
    }
    
    /// 获取最大输入大小
    fn max_input_size() -> u32 {
        256
    }
    
    /// 是否支持批量处理
    fn supports_batch() -> bool {
        false
    }
    
    /// 获取 TEE 端点路径
    fn tee_endpoint() -> &'static str {
        Self::MODULE_ID.tee_endpoint()
    }
    
    /// 模块初始化钩子（runtime 启动时调用）
    fn on_initialize() -> Weight {
        Weight::zero()
    }
    
    /// 模块清理钩子（runtime 关闭时调用）
    fn on_finalize() {
        // 默认无操作
    }
}

/// 模块配置 Trait
pub trait ModuleConfiguration {
    /// 获取配置值
    fn get<V: Decode>(key: &str) -> Option<V>;
    
    /// 设置配置值
    fn set<V: Encode>(key: &str, value: V);
}

/// 模块错误类型
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub enum ModuleError {
    /// 输入无效
    InvalidInput(BoundedVec<u8, ConstU32<128>>),
    /// 计算失败
    ComputationFailed(BoundedVec<u8, ConstU32<128>>),
    /// 序列化失败
    SerializationFailed,
    /// 模块未注册
    ModuleNotRegistered,
    /// 配置错误
    ConfigurationError,
    /// 其他错误
    Other(BoundedVec<u8, ConstU32<128>>),
}
```

### 模块注册表实现

```rust
//! # 模块注册表
//!
//! 管理所有已注册的占卜模块。

use sp_std::collections::btree_map::BTreeMap;

/// 模块注册表
pub struct DivinationRegistry<T: Config> {
    /// 已注册模块
    modules: BTreeMap<DivinationType, Box<dyn ModuleHandler<T>>>,
}

impl<T: Config> DivinationRegistry<T> {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
        }
    }
    
    /// 注册模块
    pub fn register<M: DivinationModule<T> + 'static>(&mut self) -> Result<(), &'static str> {
        let module_id = M::MODULE_ID;
        
        if self.modules.contains_key(&module_id) {
            return Err("Module already registered");
        }
        
        log::info!(
            "📦 Registering divination module: {} (v{})",
            M::MODULE_NAME,
            M::VERSION
        );
        
        self.modules.insert(module_id, Box::new(ModuleWrapper::<T, M>::new()));
        
        Ok(())
    }
    
    /// 获取模块处理器
    pub fn get(&self, module_id: DivinationType) -> Option<&dyn ModuleHandler<T>> {
        self.modules.get(&module_id).map(|m| m.as_ref())
    }
    
    /// 列出所有已注册模块
    pub fn list(&self) -> Vec<DivinationType> {
        self.modules.keys().cloned().collect()
    }
    
    /// 检查模块是否已注册
    pub fn is_registered(&self, module_id: DivinationType) -> bool {
        self.modules.contains_key(&module_id)
    }
    
    /// 获取已注册模块数量
    pub fn count(&self) -> usize {
        self.modules.len()
    }
}

/// 模块处理器 Trait（类型擦除）
pub trait ModuleHandler<T: Config>: Send + Sync {
    /// 获取模块 ID
    fn module_id(&self) -> DivinationType;
    
    /// 获取模块名称
    fn module_name(&self) -> &'static str;
    
    /// 处理请求（通用入口）
    fn handle_request(
        &self,
        input_data: &[u8],
        privacy_mode: PrivacyMode,
    ) -> Result<ProcessResult, ModuleError>;
    
    /// 获取推荐超时
    fn recommended_timeout(&self) -> u32;
}

/// 模块包装器（桥接泛型和 trait object）
struct ModuleWrapper<T: Config, M: DivinationModule<T>> {
    _phantom: sp_std::marker::PhantomData<(T, M)>,
}

impl<T: Config, M: DivinationModule<T>> ModuleWrapper<T, M> {
    fn new() -> Self {
        Self {
            _phantom: sp_std::marker::PhantomData,
        }
    }
}

impl<T: Config, M: DivinationModule<T>> ModuleHandler<T> for ModuleWrapper<T, M> {
    fn module_id(&self) -> DivinationType {
        M::MODULE_ID
    }
    
    fn module_name(&self) -> &'static str {
        M::MODULE_NAME
    }
    
    fn handle_request(
        &self,
        input_data: &[u8],
        privacy_mode: PrivacyMode,
    ) -> Result<ProcessResult, ModuleError> {
        // 1. 解码输入
        let input = M::PlainInput::decode(&mut &input_data[..])
            .map_err(|_| ModuleError::InvalidInput(b"Decode failed".to_vec().try_into().unwrap()))?;
        
        // 2. 验证输入
        M::validate_input(&input)?;
        
        // 3. 执行计算
        let result = M::compute(&input)?;
        
        // 4. 提取索引
        let type_index = M::extract_index(&result, privacy_mode)
            .map(|idx| idx.encode());
        
        // 5. 生成清单
        let manifest = M::generate_manifest(&input, &result, privacy_mode)?;
        let manifest_hash = sp_io::hashing::blake2_256(&manifest);
        
        Ok(ProcessResult {
            manifest_cid: Vec::new(), // 由调用方上传 IPFS 后填充
            manifest_hash,
            type_index,
            proof: None,
            manifest_data: Some(manifest),
        })
    }
    
    fn recommended_timeout(&self) -> u32 {
        M::recommended_timeout()
    }
}
```

### Runtime 配置示例

```rust
//! # Runtime 模块注册
//!
//! 在 runtime 中注册所有占卜模块。

// runtime/src/lib.rs

use pallet_divination_ocw_tee::{
    DivinationRegistry,
    DivinationModule,
};

/// 初始化模块注册表
pub fn init_divination_registry() -> DivinationRegistry<Runtime> {
    let mut registry = DivinationRegistry::new();
    
    // 注册八字模块
    registry.register::<pallet_divination_bazi::Pallet<Runtime>>()
        .expect("Failed to register BaZi module");
    
    // 注册奇门模块
    registry.register::<pallet_divination_qimen::Pallet<Runtime>>()
        .expect("Failed to register QiMen module");
    
    // 注册梅花易数模块
    registry.register::<pallet_divination_meihua::Pallet<Runtime>>()
        .expect("Failed to register MeiHua module");
    
    // 注册六爻模块
    registry.register::<pallet_divination_liuyao::Pallet<Runtime>>()
        .expect("Failed to register LiuYao module");
    
    // 注册紫微斗数模块
    registry.register::<pallet_divination_ziwei::Pallet<Runtime>>()
        .expect("Failed to register ZiWei module");
    
    // 注册大六壬模块
    registry.register::<pallet_divination_daliuren::Pallet<Runtime>>()
        .expect("Failed to register DaLiuRen module");
    
    // 注册小六壬模块
    registry.register::<pallet_divination_xiaoliuren::Pallet<Runtime>>()
        .expect("Failed to register XiaoLiuRen module");
    
    // 注册塔罗模块
    registry.register::<pallet_divination_tarot::Pallet<Runtime>>()
        .expect("Failed to register Tarot module");
    
    log::info!("✅ Registered {} divination modules", registry.count());
    
    registry
}

// 在 runtime 配置中使用
impl pallet_divination_ocw_tee::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Registry = DivinationRegistry<Runtime>;
    // ...
}
```

### 新模块接入指南

添加新的占卜模块只需 **3 步**：

#### Step 1: 实现 DivinationModule Trait

```rust
// pallets/divination/new-module/src/lib.rs

use pallet_divination_ocw_tee::{
    DivinationModule, DivinationType, PrivacyMode, ModuleError,
};

impl<T: Config> DivinationModule<T> for Pallet<T> {
    const MODULE_ID: DivinationType = DivinationType::NewModule;
    const MODULE_NAME: &'static str = "NewModule";
    const VERSION: u32 = 1;
    
    type PlainInput = NewModuleInput;
    type Index = NewModuleIndex;
    type Result = NewModuleResult;
    type ModuleConfig = DefaultModuleConfig;
    
    fn compute(input: &Self::PlainInput) -> Result<Self::Result, ModuleError> {
        // 实现计算逻辑
        todo!()
    }
    
    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index> {
        match privacy_mode {
            PrivacyMode::Private => None,
            _ => Some(result.to_index()),
        }
    }
    
    fn generate_manifest(
        input: &Self::PlainInput,
        result: &Self::Result,
        privacy_mode: PrivacyMode,
    ) -> Result<Vec<u8>, ModuleError> {
        // 生成 JSON 清单
        todo!()
    }
    
    fn validate_input(input: &Self::PlainInput) -> Result<(), ModuleError> {
        // 验证输入
        Ok(())
    }
}
```

#### Step 2: 添加 DivinationType 枚举值

```rust
// pallet-divination-ocw-tee/src/types.rs

pub enum DivinationType {
    BaZi = 0,
    QiMen = 1,
    // ...
    NewModule = 8,  // 新增
}

impl DivinationType {
    pub fn tee_endpoint(&self) -> &'static str {
        match self {
            // ...
            Self::NewModule => "/compute/newmodule",
        }
    }
}
```

#### Step 3: 在 Runtime 注册

```rust
// runtime/src/lib.rs

registry.register::<pallet_divination_newmodule::Pallet<Runtime>>()
    .expect("Failed to register NewModule");
```

**完成！** 新模块自动获得：
- ✅ OCW 调度支持
- ✅ TEE 隐私计算
- ✅ IPFS 存储
- ✅ 重试机制
- ✅ 超时处理
- ✅ 前端 SDK 支持

---

## 扩展点设计

### 1. 自定义计算超时

```rust
impl<T: Config> DivinationModule<T> for Pallet<T> {
    // ...
    
    fn recommended_timeout() -> u32 {
        200  // 覆盖默认值
    }
}
```

### 2. 自定义输入验证

```rust
fn validate_input(input: &Self::PlainInput) -> Result<(), ModuleError> {
    if input.year < 1900 || input.year > 2100 {
        return Err(ModuleError::InvalidInput(
            b"Year out of range".to_vec().try_into().unwrap()
        ));
    }
    Ok(())
}
```

### 3. 批量处理支持

```rust
impl<T: Config> DivinationModule<T> for Pallet<T> {
    fn supports_batch() -> bool {
        true
    }
    
    fn compute_batch(inputs: &[Self::PlainInput]) -> Result<Vec<Self::Result>, ModuleError> {
        inputs.iter().map(Self::compute).collect()
    }
}
```

### 4. 模块特定事件

```rust
// 模块可以定义自己的事件
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    /// 八字计算完成
    BaziComputed {
        owner: T::AccountId,
        chart_id: u64,
        sizhu_index: SiZhuIndex,
    },
}
```

### 5. 模块特定存储

```rust
// 模块可以有自己的额外存储
#[pallet::storage]
pub type BaziCharts<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,
    BaziOnChain<T>,
    OptionQuery,
>;
```

---

## 模块生命周期

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         模块生命周期                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. 编译时                                                               │
│     ┌─────────────────────────────────────────────────────────────┐     │
│     │ impl DivinationModule<T> for Pallet<T>                      │     │
│     │   - 类型检查                                                │     │
│     │   - Trait 约束验证                                          │     │
│     └─────────────────────────────────────────────────────────────┘     │
│                              │                                           │
│                              ▼                                           │
│  2. Runtime 初始化                                                       │
│     ┌─────────────────────────────────────────────────────────────┐     │
│     │ registry.register::<Pallet<Runtime>>()                      │     │
│     │   - 注册到全局注册表                                        │     │
│     │   - 调用 on_initialize()                                    │     │
│     └─────────────────────────────────────────────────────────────┘     │
│                              │                                           │
│                              ▼                                           │
│  3. 请求处理                                                             │
│     ┌─────────────────────────────────────────────────────────────┐     │
│     │ OCW 调度器                                                  │     │
│     │   - 根据 divination_type 查找模块                           │     │
│     │   - 调用 handle_request()                                   │     │
│     │   - 处理结果/错误                                           │     │
│     └─────────────────────────────────────────────────────────────┘     │
│                              │                                           │
│                              ▼                                           │
│  4. Runtime 关闭                                                         │
│     ┌─────────────────────────────────────────────────────────────┐     │
│     │ 调用 on_finalize()                                          │     │
│     │   - 清理资源                                                │     │
│     └─────────────────────────────────────────────────────────────┘     │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 参考

- `docs/BAZI_JSON_MANIFEST_DESIGN.md` - 八字 JSON 清单设计
- `docs/QIMEN_JSON_MANIFEST_DESIGN.md` - 奇门 JSON 清单设计
- `docs/OPTIMAL_DEVELOPMENT_PLAN.md` - 最优开发方案
- `pallets/divination/tee-privacy/src/lib.rs` - TEE 隐私计算模块
- `pallets/stardust-ipfs/src/lib.rs` - IPFS 集成模块
