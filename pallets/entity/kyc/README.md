# pallet-entity-kyc

实体 KYC/AML 认证模块。

## 概述

本模块实现用户和实体的 KYC（了解你的客户）和 AML（反洗钱）认证功能。

## 功能特性

- **多级别认证**: None / Basic / Standard / Enhanced / Institutional
- **多提供者支持**: 内部、第三方、政府、金融机构
- **风险评分**: 0-100 风险评分系统
- **高风险国家管理**: 可配置高风险国家列表
- **认证有效期**: 自动过期管理

## KYC 级别

| 级别 | 要求 | 有效期 |
|------|------|--------|
| None | 未认证 | - |
| Basic | 邮箱/手机验证 | 1 年 |
| Standard | 身份证件 | 6 个月 |
| Enhanced | 地址 + 资金来源 | 1 年 |
| Institutional | 企业文件 + 受益人 | 1 年 |

## KYC 状态

```rust
pub enum KycStatus {
    NotSubmitted,  // 未提交
    Pending,       // 待审核
    Approved,      // 已通过
    Rejected,      // 已拒绝
    Expired,       // 已过期
    Revoked,       // 已撤销
}
```

## Extrinsics

| 函数 | 说明 |
|------|------|
| `submit_kyc` | 提交 KYC 申请 |
| `approve_kyc` | 批准 KYC（提供者） |
| `reject_kyc` | 拒绝 KYC（提供者） |
| `revoke_kyc` | 撤销 KYC（管理员） |
| `register_provider` | 注册认证提供者 |
| `remove_provider` | 移除认证提供者 |
| `set_entity_requirement` | 设置实体 KYC 要求 |
| `update_high_risk_countries` | 更新高风险国家 |

## 存储项

| 名称 | 说明 |
|------|------|
| `KycRecords` | 用户 KYC 记录 |
| `Providers` | 认证提供者列表 |
| `ProviderCount` | 活跃提供者数量 |
| `EntityRequirements` | 实体 KYC 要求 |
| `HighRiskCountries` | 高风险国家列表 |

## 辅助函数

```rust
// 检查是否满足 KYC 要求
fn meets_kyc_requirement(account: &AccountId, min_level: KycLevel) -> bool

// 获取用户 KYC 级别
fn get_kyc_level(account: &AccountId) -> KycLevel

// 检查是否高风险国家
fn is_high_risk_country(account: &AccountId) -> bool

// 检查能否参与实体活动
fn can_participate_in_entity(account: &AccountId, entity_id: u64) -> bool

// 获取风险评分
fn get_risk_score(account: &AccountId) -> u8
```

## 隐私说明

- KYC 数据通过 IPFS CID 引用，实际数据加密存储
- 链上只存储认证状态和元数据
- 符合 GDPR 数据最小化原则

## 版本

- v0.1.0 (Phase 7): 初始版本
