# 服务提供者档案模块 (pallet-divination-profile)

## 概述

服务提供者档案模块用于管理占卜服务提供者（命理师）的个人档案、资质认证、服务评价等信息。

> **注意**：本模块目前处于规划阶段，相关功能暂时分布在以下模块中：
> - `pallet-divination-privacy`：服务提供者注册、类型管理、信誉系统
> - `pallet-divination-market`：提供者详细资料、资质证书、案例展示

## 当前实现位置

### privacy 模块中的服务提供者功能

```rust
// 服务提供者类型
pub enum ServiceProviderType {
    MingLiShi = 0,    // 命理师
    AiService = 1,    // AI 服务
    FamilyMember = 2, // 家族成员
    Research = 3,     // 研究机构
}

// 服务提供者基础信息
pub struct ServiceProvider<BlockNumber> {
    pub provider_type: ServiceProviderType,
    pub public_key: [u8; 32],      // X25519 公钥
    pub reputation: u8,             // 信誉分 (0-100)
    pub is_active: bool,
    pub registered_at: BlockNumber,
    pub completed_services: u32,
}
```

存储项：
- `ServiceProviders`: 服务提供者信息映射
- `ProvidersByType`: 按类型索引的提供者列表

### market 模块中的档案功能

```rust
// 服务提供者详细资料
pub struct ProviderProfile<BlockNumber, MaxDetailLen, MaxCidLen> {
    pub introduction_cid: Option<BoundedVec<u8, MaxCidLen>>,  // 自我介绍 IPFS CID
    pub experience_years: u8,                                  // 从业年限
    pub background: Option<BoundedVec<u8, MaxDetailLen>>,     // 师承背景
    pub motto: Option<BoundedVec<u8, ConstU32<256>>>,         // 服务理念
    pub expertise_description: Option<BoundedVec<u8, MaxDetailLen>>,  // 擅长领域
    pub working_hours: Option<BoundedVec<u8, ConstU32<128>>>, // 工作时间
    pub avg_response_time: Option<u32>,                        // 平均响应时间
    pub accepts_appointment: bool,                             // 是否接受预约
    pub banner_cid: Option<BoundedVec<u8, MaxCidLen>>,        // 背景图 CID
    pub updated_at: BlockNumber,
}

// 资质证书
pub struct Certificate<BlockNumber, MaxNameLen, MaxCidLen> {
    pub id: u32,
    pub name: BoundedVec<u8, MaxNameLen>,
    pub cert_type: CertificateType,
    pub issuer: Option<BoundedVec<u8, MaxNameLen>>,
    pub image_cid: BoundedVec<u8, MaxCidLen>,
    pub issued_at: Option<BlockNumber>,
    pub is_verified: bool,
    pub uploaded_at: BlockNumber,
}

// 证书类型
pub enum CertificateType {
    Education = 0,      // 学历证书
    Professional = 1,   // 专业资格
    Association = 2,    // 行业协会认证
    Apprenticeship = 3, // 师承证明
    Award = 4,          // 获奖证书
    Other = 5,
}
```

存储项：
- `ProviderProfiles`: 提供者详细资料
- `ProviderCertificates`: 资质证书

## 规划功能

### 核心功能

1. **档案管理**
   - 创建/更新服务提供者档案
   - 头像、背景图上传（IPFS）
   - 自我介绍（支持富文本）
   - 从业经历和师承背景

2. **资质认证**
   - 上传资质证书
   - 管理员审核认证
   - 证书有效期管理
   - 认证徽章展示

3. **服务评价**
   - 订单完成后评价
   - 评分系统（1-5星）
   - 评价内容审核
   - 信誉分计算

4. **档案展示**
   - 个人主页
   - 服务套餐展示
   - 历史案例展示
   - 客户评价展示

5. **保证金机制**
   - 注册需缴纳保证金
   - 违规扣除保证金
   - 退出返还保证金

### 数据结构规划

```rust
// 档案状态
pub enum ProfileStatus {
    Pending,    // 待审核
    Active,     // 活跃
    Suspended,  // 暂停
    Banned,     // 封禁
}

// 完整档案信息
pub struct FullProfile<AccountId, Balance, BlockNumber> {
    pub account: AccountId,
    pub status: ProfileStatus,
    pub deposit: Balance,
    
    // 基础信息
    pub name: BoundedVec<u8, MaxNameLen>,
    pub avatar_cid: Option<BoundedVec<u8, MaxCidLen>>,
    pub bio: BoundedVec<u8, MaxBioLen>,
    
    // 专业信息
    pub specialties: BoundedVec<DivinationType, MaxSpecialties>,
    pub experience_years: u8,
    pub background: Option<BoundedVec<u8, MaxDetailLen>>,
    
    // 统计信息
    pub total_orders: u32,
    pub completed_orders: u32,
    pub total_rating: u32,
    pub rating_count: u32,
    
    // 时间戳
    pub created_at: BlockNumber,
    pub updated_at: BlockNumber,
}

// 评价记录
pub struct Rating<AccountId, BlockNumber> {
    pub rater: AccountId,
    pub order_id: u64,
    pub score: u8,           // 1-5
    pub comment: Option<BoundedVec<u8, MaxCommentLen>>,
    pub timestamp: BlockNumber,
}
```

### Extrinsics 规划

```rust
// 用户调用
fn create_profile(origin, name, bio, specialties) -> DispatchResult;
fn update_profile(origin, updates) -> DispatchResult;
fn upload_certification(origin, cert_type, name, image_cid) -> DispatchResult;
fn submit_rating(origin, provider, order_id, score, comment) -> DispatchResult;
fn withdraw_deposit(origin) -> DispatchResult;

// 治理调用
fn approve_certification(origin, provider, cert_id) -> DispatchResult;
fn reject_certification(origin, provider, cert_id, reason) -> DispatchResult;
fn suspend_profile(origin, provider, reason) -> DispatchResult;
fn unsuspend_profile(origin, provider) -> DispatchResult;
fn ban_profile(origin, provider, reason) -> DispatchResult;
```

### 配置参数规划

| 参数 | 类型 | 说明 |
|------|------|------|
| `ProfileDeposit` | `Balance` | 档案保证金 (10 DUST) |
| `MaxSpecialties` | `u32` | 最大专业数 |
| `MaxCertifications` | `u32` | 最大资质数 |
| `MinRatingScore` | `u8` | 最小评分 (1) |
| `MaxRatingScore` | `u8` | 最大评分 (5) |
| `MaxCommentLength` | `u32` | 最大评论长度 |
| `MaxNameLength` | `u32` | 最大名称长度 |
| `MaxBioLength` | `u32` | 最大简介长度 |

## 使用示例

### 当前：通过 privacy 模块注册服务提供者

```rust
// 注册为命理师
Privacy::register_provider(
    origin,
    ServiceProviderType::MingLiShi,
    public_key,
)?;

// 查询服务提供者
let provider = Privacy::service_providers(account);

// 按类型获取提供者列表
let masters = Privacy::providers_by_type(ServiceProviderType::MingLiShi);
```

### 当前：通过 market 模块管理档案

```rust
// 更新提供者资料
Market::update_provider_profile(
    origin,
    introduction_cid,
    experience_years,
    background,
    motto,
)?;

// 上传资质证书
Market::upload_certificate(
    origin,
    name,
    cert_type,
    issuer,
    image_cid,
)?;
```

## 相关模块

- [pallet-divination-privacy](../privacy/README.md) - 隐私保护与服务提供者注册
- [pallet-divination-market](../market/README.md) - 服务市场与档案管理
- [pallet-divination-common](../common/README.md) - 公共类型定义

## 开发计划

1. **Phase 1**：整合现有功能
   - 从 privacy 和 market 模块提取档案相关代码
   - 统一数据结构和接口

2. **Phase 2**：完善档案功能
   - 实现完整的档案生命周期管理
   - 添加保证金机制
   - 实现评价系统

3. **Phase 3**：增强功能
   - 档案搜索和排序
   - 推荐算法
   - 档案认证徽章

## 版本历史

- v0.0.1 - 规划阶段，功能分布在 privacy 和 market 模块中
