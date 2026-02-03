# Entity 模块 (pallet-entity)

> 🏪 COSMOS 实体管理系统 - 支持多类型实体、通证发行、治理、KYC 和代币发售

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Substrate](https://img.shields.io/badge/Substrate-polkadot--sdk-blue)](https://github.com/paritytech/polkadot-sdk)

## 概述

Entity 模块是 COSMOS 平台的核心业务模块套件，从 Entity 重构而来，提供通用实体管理能力。支持店铺、餐饮、投资基金、DAO、合作社等多种实体类型。

## 模块架构

```
pallets/entity/
├── common/          # 公共类型和 Trait
├── registry/        # 实体注册管理
├── token/           # 实体通证管理
├── governance/      # 实体治理
├── member/          # 会员管理
├── commission/      # 返佣管理
├── market/          # 代币交易市场
├── service/         # 商品/服务管理
├── transaction/     # 订单/交易管理
├── review/          # 评价管理
├── disclosure/      # 财务披露 (Phase 6)
├── kyc/             # KYC/AML 认证 (Phase 7)
└── sale/            # 代币发售 (Phase 8)
```

## 子模块说明

| 模块 | 说明 | 状态 |
|------|------|------|
| [common](./common/README.md) | 公共类型定义 | ✅ |
| [registry](./registry/README.md) | 实体注册、生命周期、多管理员 | ✅ |
| [token](./token/README.md) | 通证发行、分红、锁仓 | ✅ |
| [governance](./governance/README.md) | 多模式治理、提案、投票 | ✅ |
| [member](./member/README.md) | 会员等级、推荐关系 | ✅ |
| [commission](./commission/README.md) | 多模式返佣、分级提现 | ✅ |
| [market](./market/README.md) | P2P 代币交易、TWAP 预言机 | ✅ |
| [service](./service/README.md) | 商品/服务 CRUD | ✅ |
| [transaction](./transaction/README.md) | 订单流程、托管 | ✅ |
| [review](./review/README.md) | 评价系统 | ✅ |
| [disclosure](./disclosure/README.md) | 财务披露、内幕交易控制 | ✅ |
| [kyc](./kyc/README.md) | KYC/AML 认证 | ✅ |
| [sale](./sale/README.md) | 代币发售、锁仓解锁 | ✅ |

## 核心类型

### 实体类型 (EntityType)

```rust
pub enum EntityType {
    Shop,           // 店铺
    Restaurant,     // 餐饮
    Investment,     // 投资基金
    DAO,            // 去中心化自治组织
    Cooperative,    // 合作社
    Project,        // 项目/众筹
    Club,           // 俱乐部
    Custom(u8),     // 自定义
}
```

### 通证类型 (TokenType)

```rust
pub enum TokenType {
    Utility,        // 实用型（消费积分）
    Equity,         // 权益型（分红权）
    Governance,     // 治理型（投票权）
    Membership,     // 会员型
    Hybrid,         // 混合型
}
```

### 治理模式 (GovernanceMode)

```rust
pub enum GovernanceMode {
    None,           // 无治理
    Advisory,       // 咨询式
    DualTrack,      // 双轨制
    Committee,      // 委员会制
    FullDAO,        // 完全 DAO
    Tiered,         // 分层治理
}
```

## 主要功能

### 实体管理
- 多类型实体注册和管理
- 多管理员支持
- 实体类型升级
- 运营资金管理

### 通证系统
- 多种通证类型
- 分红机制
- 锁仓和解锁
- 类型变更

### 治理系统
- 6 种治理模式
- 分层投票阈值
- 委员会管理
- 管理员否决权

### 合规功能
- 财务信息披露
- KYC/AML 认证
- 内幕交易控制
- 高风险国家管理

### 代币发售
- 多种发售模式（固定价格、荷兰拍卖等）
- 多资产支付
- 锁仓解锁机制
- KYC 集成

## 快速开始

### 依赖配置

```toml
[dependencies]
pallet-entity-common = { path = "pallets/entity/common", default-features = false }
pallet-entity-registry = { path = "pallets/entity/registry", default-features = false }
# ... 其他子模块
```

### 测试

```bash
# 测试所有子模块
cargo test -p pallet-entity-*

# 测试特定模块
cargo test -p pallet-entity-disclosure
cargo test -p pallet-entity-kyc
cargo test -p pallet-entity-sale
```

## 文档

- [API 文档](../../docs/design/entity-module-api.md)
- [前端集成指南](../../docs/design/entity-frontend-integration.md)
- [重构设计文档](../../docs/design/entity-token-refactor-plan.md)

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1.0 | 2026-01-31 | 从 Entity 拆分 |
| v0.2.0 | 2026-02-01 | Phase 1-4: 扩展类型支持 |
| v0.3.0 | 2026-02-02 | Phase 5: 治理增强 |
| v0.4.0 | 2026-02-03 | Phase 6-8: 披露/KYC/发售 |

## 许可证

MIT License
