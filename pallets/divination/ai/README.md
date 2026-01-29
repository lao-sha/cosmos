# 通用玄学 AI 解读 Pallet (pallet-divination-ai)

通用 AI 智能解读模块 - 为所有玄学占卜系统提供统一的 AI 解读服务

## 概述

本模块实现了基于链下预言机的 AI 智能解读系统，支持多种玄学系统的 AI 解读请求、预言机管理、结果处理和费用分配。

## 架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                     pallet-divination-ai                        │
│              (通用 AI 解读、预言机管理、争议处理)                  │
└──────────────────────────┬──────────────────────────────────────┘
                           │ DivinationProvider trait
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                Runtime: CombinedDivinationProvider              │
├─────────┬─────────┬─────────┬─────────┬─────────┬───────────────┤
│ Meihua  │  Bazi   │ Liuyao  │  Qimen  │  Ziwei  │    Tarot      │
│ 梅花易数 │  八字   │  六爻   │  奇门   │  紫微   │    塔罗       │
└─────────┴─────────┴─────────┴─────────┴─────────┴───────────────┘
```

## 核心功能

### 1. 解读请求

- 用户为占卜结果请求 AI 解读
- 支持多种解读类型（基础/详细/专业）
- 费用根据占卜类型和解读类型动态计算

### 2. 预言机管理

- 预言机节点注册与质押
- 支持的占卜类型和解读类型声明
- 模型版本管理
- 评分和信誉系统

### 3. 结果处理

- 预言机接收并处理请求
- 提交解读结果（IPFS CID）
- 用户评分反馈

### 4. 争议处理

- 用户可对解读结果发起争议
- 仲裁员裁决机制
- 退款和惩罚机制

### 5. 费用分配

- 预言机获得解读费用
- 平台抽成进入国库
- 争议押金机制

## 存储结构

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `NextRequestId` | `u64` | 下一个请求 ID |
| `NextDisputeId` | `u64` | 下一个争议 ID |
| `Requests` | `Map<u64, Request>` | 解读请求存储 |
| `Results` | `Map<u64, Result>` | 解读结果存储 |
| `Oracles` | `Map<AccountId, Oracle>` | 预言机节点存储 |
| `ActiveOracles` | `Vec<AccountId>` | 活跃预言机列表 |
| `Disputes` | `Map<u64, Dispute>` | 争议存储 |
| `ModelConfigs` | `Map<DivinationType, Config>` | 模型配置 |

## 可调用函数

### 用户接口

| 函数 | 说明 |
|------|------|
| `request_interpretation` | 请求 AI 解读 |
| `rate_result` | 对解读结果评分 |

### 预言机接口

| 函数 | 说明 |
|------|------|
| `register_oracle` | 注册预言机节点 |
| `unregister_oracle` | 注销预言机节点 |
| `pause_oracle` | 暂停预言机 |
| `resume_oracle` | 恢复预言机 |
| `accept_request` | 接收解读请求 |
| `submit_result` | 提交解读结果 |
| `report_failure` | 报告处理失败 |

### 争议接口

| 函数 | 说明 |
|------|------|
| `create_dispute` | 创建争议 |
| `resolve_dispute` | 解决争议（仲裁员） |

## 配置参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `BaseInterpretationFee` | 基础解读费用 | 10 COS |
| `MinOracleStake` | 预言机最低质押 | 1000 COS |
| `DisputeDeposit` | 争议押金 | 50 COS |
| `RequestTimeout` | 请求超时（区块数） | 1200 |
| `ProcessingTimeout` | 处理超时（区块数） | 3600 |
| `DisputePeriod` | 争议期限（区块数） | 7200 |

## 解读类型

| 类型 | 说明 | 费用倍数 |
|------|------|----------|
| `Basic` | 基础解读 | 1x |
| `Detailed` | 详细解读 | 2x |
| `Professional` | 专业解读 | 3x |
| `Comprehensive` | 综合解读 | 5x |

## 事件

| 事件 | 说明 |
|------|------|
| `InterpretationRequested` | 解读请求已创建 |
| `RequestAccepted` | 预言机已接收请求 |
| `ResultSubmitted` | 解读结果已提交 |
| `RequestFailed` | 请求处理失败 |
| `ResultRated` | 用户已评分 |
| `OracleRegistered` | 预言机已注册 |
| `OracleUnregistered` | 预言机已注销 |
| `DisputeCreated` | 争议已创建 |
| `DisputeResolved` | 争议已解决 |

## 使用示例

```rust
// 请求 AI 解读
DivinationAi::request_interpretation(
    origin,
    DivinationType::Meihua,  // 占卜类型
    hexagram_id,             // 占卜结果 ID
    InterpretationType::Detailed, // 解读类型
    None,                    // 额外上下文
)?;

// 预言机接收请求
DivinationAi::accept_request(oracle_origin, request_id)?;

// 提交解读结果
DivinationAi::submit_result(
    oracle_origin,
    request_id,
    b"QmXxx...".to_vec(),    // 内容 IPFS CID
    None,                    // 摘要 CID
    b"gpt-4".to_vec(),       // 模型版本
    b"zh".to_vec(),          // 语言
)?;

// 用户评分
DivinationAi::rate_result(origin, request_id, 5)?;
```

## 测试

```bash
cargo test -p pallet-divination-ai
```

## 许可证

MIT License
