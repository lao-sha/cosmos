# pallet-arbitration

> 路径：`pallets/dispute/arbitration/`

仲裁争议处理系统，提供争议登记、证据管理、仲裁裁决、双向押金、统一投诉等功能，支持 12 个业务域。

## 设计理念

- **域路由架构**：8 字节域标识，多业务统一仲裁
- **双向押金**：发起方/应诉方各锁 15% 订单金额
- **两大子系统**：仲裁系统（资金争议）+ 投诉系统（行为投诉）

## 核心功能

### 仲裁系统（资金争议）
| 功能 | 说明 |
|------|------|
| 争议登记 | 发起仲裁，锁定押金 |
| 证据引用 | 通过 evidence_id 引用 pallet-evidence |
| 应诉机制 | 设置应诉期限，超时视为弃权 |
| 裁决执行 | Release / Refund / Partial 三种方式 |
| 押金罚没 | 败诉罚 30%，部分胜诉各罚 50% |

### 投诉系统（行为投诉）
| 功能 | 说明 |
|------|------|
| 投诉类型 | 56 种类型，覆盖 12 业务域 |
| 响应/申诉 | 被投诉方可提交申诉 |
| 和解机制 | 双方可达成和解 |
| 仲裁升级 | 调解失败可升级到仲裁委员会 |

## Extrinsics

### 仲裁相关
| 方法 | call_index | 说明 |
|------|-----------|------|
| `dispute` | 0 | 发起仲裁（旧版） |
| `arbitrate` | 1 | 仲裁员裁决（治理 Origin） |
| `dispute_with_evidence_id` | 2 | 带证据ID发起仲裁 |
| `append_evidence_id` | 3 | 追加证据引用 |
| `dispute_with_two_way_deposit` | 4 | 双向押金仲裁（推荐） |
| `respond_to_dispute` | 5 | 应诉（锁押金+提交证据） |

### 投诉相关
| 方法 | call_index | 说明 |
|------|-----------|------|
| `file_complaint` | 10 | 发起投诉（需押金） |
| `respond_to_complaint` | 11 | 响应/申诉 |
| `withdraw_complaint` | 12 | 撤销投诉 |
| `settle_complaint` | 13 | 达成和解 |
| `escalate_to_arbitration` | 14 | 升级到仲裁 |
| `resolve_complaint` | 15 | 仲裁裁决投诉（治理 Origin） |

## 主要类型

### Decision（裁决类型）
```rust
pub enum Decision {
    Release,      // 全额释放（卖家胜）
    Refund,       // 全额退款（买家胜）
    Partial(u16), // 按比例分配，bps=0-10000
}
```

### ComplaintStatus（投诉状态）
```rust
pub enum ComplaintStatus {
    Submitted,              // 已提交
    Responded,              // 已响应
    Mediating,              // 调解中
    Arbitrating,            // 仲裁中
    ResolvedComplainantWin, // 投诉方胜
    ResolvedRespondentWin,  // 被投诉方胜
    ResolvedSettlement,     // 和解
    Withdrawn,              // 已撤销
    Expired,                // 已过期
}
```

### 投诉类型示例

| 业务域 | 投诉类型 |
|-------|---------|
| OTC | OtcSellerNotDeliver, OtcBuyerFalseClaim, OtcTradeFraud |
| 直播 | LiveIllegalContent, LiveFalseAdvertising, LiveHarassment |
| 占卜 | DivinePornography, DivineFraud, DivineAbuse |
| 聊天 | ChatHarassment, ChatFraud, ChatIllegalContent |
| NFT | NftSellerNotDeliver, NftCounterfeit, NftTradeFraud |

## Trait 接口

### ArbitrationRouter（域路由）
```rust
pub trait ArbitrationRouter<AccountId> {
    fn can_dispute(domain: [u8; 8], object_id: u64, who: &AccountId) -> bool;
    fn get_escrow_id(domain: [u8; 8], object_id: u64) -> Option<u64>;
    fn get_parties(domain: [u8; 8], object_id: u64) -> Option<(AccountId, AccountId)>;
}
```

## 配置参数

| 参数 | 说明 | 默认值 |
|------|------|-------|
| `DisputeDepositRate` | 押金比例 | 1500 (15%) |
| `ResponseDeadline` | 应诉期限 | 7 天 |
| `ComplaintDeposit` | 投诉押金 | 10 UNIT |
| `LoseDepositPenalty` | 败诉罚没比例 | 3000 (30%) |
| `PartialPenalty` | 部分胜诉罚没 | 5000 (50%) |

## 集成示例

```rust
// OTC 模块实现 ArbitrationRouter
impl<T: Config> ArbitrationRouter<T::AccountId> for Pallet<T> {
    fn can_dispute(domain: [u8; 8], object_id: u64, who: &T::AccountId) -> bool {
        if domain != *b"otc_ord_" { return false; }
        Self::is_order_participant(object_id, who)
    }
    
    fn get_escrow_id(domain: [u8; 8], object_id: u64) -> Option<u64> {
        Orders::<T>::get(object_id).map(|o| o.escrow_id)
    }
}
```

## 相关模块

- `@/home/xiaodong/桌面/cosmos/pallets/dispute/escrow/` - 资金托管
- `@/home/xiaodong/桌面/cosmos/pallets/dispute/evidence/` - 证据管理
