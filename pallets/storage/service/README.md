# pallet-storage-service

> 路径：`pallets/storage/service/`

IPFS 存储服务核心模块，提供去中心化内容固定（Pin）、运营者管理、分层存储策略和自动计费功能。

## 设计理念

- **多副本冗余**：多运营者节点确保高可用性
- **分层存储**：Critical/Standard/Temporary 三级策略
- **自动化**：OCW 健康巡检、故障迁移、周期扣费
- **经济激励**：保证金 + SLA 统计 + 奖励分配

## 核心功能

### 1. Pin 管理
| 功能 | 说明 |
|------|------|
| 多层级请求 | Critical / Standard / Temporary |
| 自动分配 | 基于 Layer、优先级、容量选择运营者 |
| 多副本冗余 | 按层级配置副本数（5/3/1） |
| 状态追踪 | Requested → Pinning → Pinned → Failed |

### 2. 运营者管理
| 功能 | 说明 |
|------|------|
| 三层分类 | Layer1(Core) / Layer2(Community) / Layer3(External) |
| 保证金 | 锁定最低保证金，离开时退还 |
| 状态管理 | Active / Suspended / Banned |
| SLA 统计 | 成功率、健康度、容量使用率 |

### 3. 分层策略（PinTier）
| 层级 | 副本数 | 巡检周期 | 费率 | 适用场景 |
|------|-------|---------|------|---------|
| Critical | 5 | 6小时 | 1.5x | 证据、法律文件 |
| Standard | 3 | 24小时 | 1.0x | 一般内容 |
| Temporary | 1 | 7天 | 0.5x | 临时数据 |

### 4. 计费机制
- **扣费顺序**：IpfsPool配额 → UserFunding → IpfsPool兜底 → 宽限期
- **扣费周期**：默认每周（100,800 块）
- **宽限期**：默认360天，期间服务继续但不接受新Pin

## 主要类型

### SubjectType（业务域）
```rust
pub enum SubjectType {
    Evidence,        // 证据（最高优先级）
    OtcOrder,        // OTC订单
    DivinationMarket,// 命理服务市场
    DivinationNft,   // 命理NFT
    DivinationAi,    // AI解读
    Chat,            // 聊天消息（⚠️临时）
    Livestream,      // 直播间（⚠️临时）
    Swap,            // Swap兑换
    Arbitration,     // 仲裁证据
    UserProfile,     // 用户档案
    Matchmaking,     // 婚恋模块
    General,         // 通用存储
    Custom(Vec<u8>), // 自定义域
}
```

### OperatorLayer（运营者分层）
```rust
pub enum OperatorLayer {
    Layer1,  // Core - 团队运营，最高优先级
    Layer2,  // Community - 社区运营
    Layer3,  // External - 第三方运营
}
```

## Extrinsics

### 用户接口
| 方法 | call_index | 说明 |
|------|-----------|------|
| `request_pin_for_subject` | 10 | 为 Subject 固定 CID，四层扣费 |
| `fund_user_account` | 21 | 为用户资金账户充值 |
| `fund_subject_account` | 9 | ⚠️ 已废弃，用 `fund_user_account` |

### 运营者接口
| 方法 | call_index | 说明 |
|------|-----------|------|
| `join_operator` | 3 | 注册运营者，锁定保证金 |
| `update_operator` | 4 | 更新运营者信息 |
| `leave_operator` | 5 | 注销运营者（宽限期后退还保证金）|
| `pause_operator` | 22 | 暂停接单 |
| `resume_operator` | 23 | 恢复接单 |
| `report_probe` | 7 | OCW 上报心跳 |
| `operator_claim_rewards` | 16 | 领取奖励 |

### OCW 接口
| 方法 | call_index | 说明 |
|------|-----------|------|
| `mark_pinned` | 1 | 上报 Pin 成功 |
| `mark_pin_failed` | 2 | 上报 Pin 失败 |

### 治理接口
| 方法 | call_index | 说明 |
|------|-----------|------|
| `charge_due` | 11 | 处理到期扣费 |
| `set_billing_params` | 12 | 设置计费参数 |
| `distribute_to_operators` | 13 | 分配收益给运营者 |
| `set_replicas_config` | 14 | 设置副本数配置 |
| `update_tier_config` | 15 | 更新分层配置 |
| `emergency_pause_billing` | 17 | 紧急暂停计费 |
| `resume_billing` | 18 | 恢复计费 |
| `set_storage_layer_config` | 19 | 设置存储层配置 |
| `set_operator_layer` | 20 | 设置运营者层级 |
| `set_operator_status` | 6 | 设置运营者状态 |
| `slash_operator` | 8 | 惩罚运营者 |


## Trait 接口

### IpfsPinner（供其他 pallet 调用）
```rust
pub trait IpfsPinner<AccountId, Balance> {
    fn pin_cid_for_subject(
        caller: AccountId,
        subject_type: SubjectType,
        subject_id: u64,
        cid: Vec<u8>,
        tier: Option<PinTier>,
    ) -> DispatchResult;
    
    fn unpin_cid(caller: AccountId, cid: Vec<u8>) -> DispatchResult;
}
```

### ContentRegistry（新业务 pallet 集成）
```rust
pub trait ContentRegistry {
    fn register_content(
        domain: Vec<u8>,
        subject_id: u64,
        cid: Vec<u8>,
        tier: PinTier,
    ) -> DispatchResult;
    
    fn unregister_content(domain: Vec<u8>, cid: Vec<u8>) -> DispatchResult;
}
```

## 配置参数

| 参数 | 说明 | 默认值 |
|------|------|-------|
| `MinOperatorBond` | 最小保证金 | 100 UNIT |
| `MinCapacityGiB` | 最小容量 | 10 GiB |
| `PricePerGiBWeek` | 每 GiB 周单价 | 1e9 |
| `BillingPeriodBlocks` | 扣费周期 | 100,800 (~7天) |
| `GraceBlocks` | 宽限期 | 5,184,000 (~360天) |
| `MaxChargePerBlock` | 每块最大扣费数 | 50 |

## 集成示例

```rust
// 在业务 pallet 中使用
pub trait Config: frame_system::Config {
    type IpfsPinner: IpfsPinner<Self::AccountId, Self::Balance>;
}

// 调用示例
T::IpfsPinner::pin_cid_for_subject(
    who,
    SubjectType::Evidence,
    subject_id,
    cid,
    Some(PinTier::Critical),
)?;
```

## 相关模块

- `@/home/xiaodong/桌面/cosmos/pallets/storage/lifecycle/` - 存储生命周期管理
- `@/home/xiaodong/桌面/cosmos/pallets/dispute/evidence/` - 证据存证（依赖 IpfsPinner）
- `@/home/xiaodong/桌面/cosmos/pallets/trading/otc/` - OTC 交易（依赖 IpfsPinner）
