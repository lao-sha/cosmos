# pallet-entity-kyc

> 🔐 Entity KYC/AML 认证模块 — 多级别认证与合规性检查 (Phase 7)

## 概述

`pallet-entity-kyc` 实现用户和实体的 KYC（了解你的客户）和 AML（反洗钱）认证功能，支持多级别认证、多认证提供者、风险评分和高风险国家管理。

### 核心功能

- **5 级认证** — None / Basic / Standard / Enhanced / Institutional
- **4 种提供者类型** — Internal / ThirdParty / Government / Financial
- **风险评分** — 0-100 风险评分系统（未认证用户默认 100）
- **高风险国家** — 可配置最多 50 个 ISO 3166-1 alpha-2 国家代码
- **认证有效期** — 按级别配置，自动过期检查
- **实体 KYC 要求** — 可配置最低级别、强制性、宽限期、风险阈值

## KYC 级别

| 级别 | 要求 | 配置常量 | 可比较 |
|------|------|----------|--------|
| None | 未认证 | - | ✅ (最低) |
| Basic | 邮箱/手机验证 | `BasicKycValidity` | ✅ |
| Standard | 身份证件 | `StandardKycValidity` | ✅ |
| Enhanced | 地址 + 资金来源 | `EnhancedKycValidity` | ✅ |
| Institutional | 企业文件 + 受益人 | `EnhancedKycValidity` | ✅ (最高) |

> KycLevel 实现 `PartialOrd + Ord`，支持 `>=` 比较。

## 数据结构

### KycRecord — 用户认证记录

```rust
pub struct KycRecord<AccountId, BlockNumber, MaxCidLen> {
    pub account: AccountId,                          // 用户账户
    pub level: KycLevel,                             // 申请级别
    pub status: KycStatus,                           // 当前状态
    pub provider: Option<AccountId>,                 // 审核提供者
    pub data_cid: Option<BoundedVec<u8, MaxCidLen>>, // 认证数据 IPFS CID（加密）
    pub submitted_at: Option<BlockNumber>,           // 提交时间
    pub verified_at: Option<BlockNumber>,            // 审核时间
    pub expires_at: Option<BlockNumber>,             // 过期时间
    pub rejection_reason: Option<RejectionReason>,   // 拒绝原因
    pub rejection_details_cid: Option<BoundedVec<u8, MaxCidLen>>, // 拒绝详情 CID
    pub country_code: Option<[u8; 2]>,               // ISO 3166-1 alpha-2
    pub risk_score: u8,                              // 风险评分 0-100
}
```

### KycProvider — 认证提供者

```rust
pub struct KycProvider<AccountId, MaxNameLen> {
    pub account: AccountId,
    pub name: BoundedVec<u8, MaxNameLen>,
    pub provider_type: ProviderType,    // Internal / ThirdParty / Government / Financial
    pub max_level: KycLevel,            // 支持的最高认证级别
    pub active: bool,
    pub verifications_count: u64,       // 已完成认证数
    pub deposit: u128,
}
```

### EntityKycRequirement — 实体 KYC 要求

```rust
pub struct EntityKycRequirement {
    pub min_level: KycLevel,               // 最低 KYC 级别
    pub mandatory: bool,                   // 是否强制要求
    pub grace_period: u32,                 // 宽限期（区块数）
    pub allow_high_risk_countries: bool,   // 是否允许高风险国家
    pub max_risk_score: u8,                // 最大允许风险评分
}
```

### 枚举类型

**KycStatus：** NotSubmitted → Pending → Approved / Rejected / Expired / Revoked

**VerificationType（10 种）：** Email / Phone / IdentityDocument / AddressProof / SourceOfFunds / BusinessRegistration / BeneficialOwner / FinancialStatements / FaceVerification / VideoVerification

**RejectionReason（8 种）：** UnclearDocument / ExpiredDocument / InformationMismatch / SuspiciousActivity / SanctionedEntity / HighRiskCountry / ForgedDocument / Other

## Runtime 配置

```rust
impl pallet_entity_kyc::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxCidLength = ConstU32<64>;
    type MaxProviderNameLength = ConstU32<64>;
    type MaxProviders = ConstU32<10>;
    type BasicKycValidity = ...;     // ~1 年（区块数）
    type StandardKycValidity = ...;  // ~6 个月
    type EnhancedKycValidity = ...;  // ~1 年
    type AdminOrigin = EnsureRoot<AccountId>;
}
```

## Extrinsics

| Index | 函数 | 权限 | 说明 |
|-------|------|------|------|
| 0 | `submit_kyc(level, data_cid, country_code)` | 任意用户 | 提交 KYC 申请（已有 Pending 时拒绝） |
| 1 | `approve_kyc(account, risk_score)` | 认证提供者 | 批准 KYC，设置有效期和风险评分 |
| 2 | `reject_kyc(account, reason, details_cid)` | 认证提供者 | 拒绝 KYC，记录原因 |
| 3 | `revoke_kyc(account, reason)` | AdminOrigin | 撤销已通过的 KYC |
| 4 | `register_provider(account, name, type, max_level)` | AdminOrigin | 注册认证提供者 |
| 5 | `remove_provider(account)` | AdminOrigin | 移除认证提供者 |
| 6 | `set_entity_requirement(entity_id, min_level, mandatory, grace_period, allow_high_risk, max_risk_score)` | AdminOrigin | 设置实体 KYC 要求 |
| 7 | `update_high_risk_countries(countries)` | AdminOrigin | 更新高风险国家列表（最多 50 个） |

## Storage

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `KycRecords` | `StorageMap<AccountId, KycRecord>` | 用户 KYC 记录 |
| `Providers` | `StorageMap<AccountId, KycProvider>` | 认证提供者 |
| `ProviderCount` | `StorageValue<u32>` | 活跃提供者数量 |
| `EntityRequirements` | `StorageMap<u64, EntityKycRequirement>` | 实体 KYC 要求 |
| `PendingVerifications` | `StorageMap<AccountId, BoundedVec<AccountId>>` | 提供者待审核队列 |
| `HighRiskCountries` | `StorageValue<BoundedVec<[u8;2]>>` | 高风险国家列表 |

## Events

| 事件 | 说明 |
|------|------|
| `KycSubmitted` | KYC 已提交 |
| `KycApproved` | KYC 已通过（含 expires_at） |
| `KycRejected` | KYC 已拒绝（含 reason） |
| `KycExpired` | KYC 已过期 |
| `KycRevoked` | KYC 已撤销 |
| `ProviderRegistered` | 提供者已注册 |
| `ProviderRemoved` | 提供者已移除 |
| `EntityRequirementSet` | 实体 KYC 要求已设置 |
| `HighRiskCountriesUpdated` | 高风险国家已更新 |

## Errors

| 错误 | 说明 |
|------|------|
| `KycNotFound` | KYC 记录不存在 |
| `KycAlreadyPending` | 已有待审核的 KYC |
| `KycAlreadyApproved` | KYC 已通过 |
| `ProviderNotFound` | 提供者不存在 |
| `ProviderAlreadyExists` | 提供者已存在 |
| `NotAProvider` | 不是认证提供者 |
| `ProviderNotActive` | 提供者不活跃 |
| `Unauthorized` | 无权限 |
| `CidTooLong` / `NameTooLong` | 长度超限 |
| `MaxProvidersReached` | 达到最大提供者数量 |
| `InvalidKycStatus` / `InvalidKycLevel` | 状态/级别无效 |
| `InsufficientKycLevel` | KYC 级别不满足要求 |
| `HighRiskCountry` | 高风险国家 |
| `RiskScoreTooHigh` | 风险评分过高 |
| `KycExpired` | KYC 已过期 |
| `ProviderLevelNotSupported` | 提供者不支持此级别 |

## 辅助函数

```rust
impl<T: Config> Pallet<T> {
    /// 获取 KYC 有效期（按级别不同）
    pub fn get_validity_period(level: KycLevel) -> BlockNumber;
    /// 检查用户是否满足 KYC 要求（含过期检查）
    pub fn meets_kyc_requirement(account: &AccountId, min_level: KycLevel) -> bool;
    /// 获取用户当前 KYC 级别（仅 Approved 状态）
    pub fn get_kyc_level(account: &AccountId) -> KycLevel;
    /// 检查用户是否来自高风险国家
    pub fn is_high_risk_country(account: &AccountId) -> bool;
    /// 综合检查用户能否参与实体活动（级别+国家+风险+过期）
    pub fn can_participate_in_entity(account: &AccountId, entity_id: u64) -> bool;
    /// 获取用户风险评分（未认证返回 100）
    pub fn get_risk_score(account: &AccountId) -> u8;
}
```

## 隐私说明

- KYC 数据通过 IPFS CID 引用，实际数据加密存储在链下
- 链上只存储认证状态、级别、风险评分等元数据
- 符合 GDPR 数据最小化原则

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1.0 | 2026-02-03 | Phase 7 初始版本 |
