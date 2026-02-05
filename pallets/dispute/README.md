# Dispute 模块组

> 路径：`pallets/dispute/`

COSMOS 争议解决基础设施，提供资金托管、证据管理、仲裁裁决功能，支持 12 个业务域的争议处理。

## 目录结构

```
pallets/dispute/
├── escrow/       # 资金托管 (pallet-escrow)
├── evidence/     # 证据管理 (pallet-evidence)
└── arbitration/  # 仲裁裁决 (pallet-arbitration)
```

## 子模块

| 模块 | 功能 | 依赖 |
|------|------|------|
| **escrow** | 资金锁定、释放、退款、分账 | 无 |
| **evidence** | 证据提交、IPFS Pin、访问控制 | pallet-storage-service |
| **arbitration** | 争议登记、仲裁裁决、投诉系统 | escrow, evidence |

## 依赖关系

```
           ┌─────────────┐
           │ Arbitration │
           └──────┬──────┘
                  │
        ┌─────────┴─────────┐
        ▼                   ▼
  ┌──────────┐       ┌──────────┐
  │  Escrow  │       │ Evidence │
  └──────────┘       └──────────┘
```

## 业务域（8字节标识）

| 域标识 | 业务 | 域标识 | 业务 |
|--------|------|--------|------|
| `otc_ord_` | OTC 交易 | `livstrm_` | 直播 |
| `divine__` | 占卜服务 | `chat____` | 聊天 |
| `chatgrp_` | 群组 | `maker___` | 做市商 |
| `nft_trd_` | NFT 交易 | `swap____` | Swap 交换 |
| `contact_` | 联系人 | `member__` | 会员 |
| `affiliat` | 推荐分成 | `credit__` | 信用系统 |

## 争议解决流程

```
1. evidence::commit          → 发起方提交证据
2. arbitration::dispute_with_two_way_deposit → 发起仲裁（锁押金）
3. evidence::commit          → 应诉方提交证据  
4. arbitration::respond_to_dispute → 应诉（锁押金）
5. arbitration::arbitrate    → 仲裁员裁决
6. escrow::split_partial     → 资金分账
```

## 相关文档

- `@/home/xiaodong/桌面/cosmos/pallets/dispute/escrow/README.md`
- `@/home/xiaodong/桌面/cosmos/pallets/dispute/evidence/README.md`
- `@/home/xiaodong/桌面/cosmos/pallets/dispute/arbitration/README.md`
