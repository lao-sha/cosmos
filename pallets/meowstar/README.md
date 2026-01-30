# 喵星宇宙 (Meowstar Universe) - Substrate Pallets

## 概述

喵星宇宙的链上游戏核心模块，基于 Substrate FRAME 开发。包含宠物系统、战斗系统、质押系统、DAO 治理和 NFT 市场。

## Pallets 总览

| Pallet | Index | 功能 | 状态 |
|--------|-------|------|------|
| `pallet-meowstar-pet` | 100 | 宠物系统 | ✅ 完成 |
| `pallet-meowstar-battle` | 101 | 战斗系统 | ✅ 完成 |
| `pallet-meowstar-staking` | 102 | 质押系统 | ✅ 完成 |
| `pallet-meowstar-governance` | 103 | DAO 治理 | ✅ 完成 |
| `pallet-meowstar-marketplace` | 104 | NFT 市场 | ✅ 完成 |

---

## Pallets 详情

### pallet-meowstar-pet

宠物系统 Pallet，实现：

- **宠物铸造 (孵化)**: `hatch_pet()` - 消耗 COS 创建随机宠物
- **升级系统**: `level_up()` - 消耗 COS 提升等级和属性
- **进化系统**: `evolve()` - 达到等级阈值后进化，可改变元素
- **转移**: `transfer()` - NFT 转移
- **重命名**: `rename()` - 修改宠物名称

#### 数据结构

```rust
pub struct Pet<T: Config> {
    pub id: u64,
    pub owner: T::AccountId,
    pub name: BoundedVec<u8, ConstU32<32>>,
    pub element: Element,      // Normal/Fire/Water/Shadow/Light
    pub rarity: Rarity,        // Common/Rare/Epic/Legendary/Mythic
    pub level: u32,
    pub experience: u64,
    pub attributes: PetAttributes,
    pub evolution_stage: u8,   // 0-4
    pub created_at: BlockNumberFor<T>,
    pub gene_hash: [u8; 32],
}

pub struct PetAttributes {
    pub health: u32,
    pub attack: u32,
    pub defense: u32,
    pub speed: u32,
    pub critical_rate: u32,
    pub critical_damage: u32,
}
```

#### 稀有度概率

| 稀有度 | 概率 | 属性倍率 |
|--------|------|----------|
| Common | 60% | 1.0x |
| Rare | 25% | 1.2x |
| Epic | 10% | 1.5x |
| Legendary | 4% | 2.0x |
| Mythic | 1% | 3.0x |

### pallet-meowstar-battle

战斗系统 Pallet，实现：

- **PVE 战斗**: `start_pve_battle()` - 对战 AI
- **PVP 匹配**: `join_pvp_queue()` - 加入匹配队列
- **行动提交**: `submit_action()` - 提交战斗行动
- **离开队列**: `leave_queue()` - 退出匹配
- **投降**: `surrender()` - 放弃战斗

#### 战斗流程

```
1. 开始战斗 -> 创建 Battle 实例
2. 每回合提交行动 -> 计算伤害
3. HP 归零或达到最大回合 -> 结算
4. 更新 ELO 评分 -> 发放奖励
```

#### ELO 评分系统

- 初始分数: 1000
- K 因子: 32
- 胜利: +积分，连胜加成
- 失败: -积分，连胜归零

## 配置示例

```rust
// runtime/src/configs/mod.rs

parameter_types! {
    pub const HatchingFee: Balance = 10 * UNIT;
    pub const LevelUpBaseFee: Balance = UNIT / 10;
    pub const EvolutionFeeMultiplier: u32 = 50;
    pub const MaxPetsPerAccount: u32 = 100;
    pub const BattleEntryFee: Balance = UNIT / 2;
    pub const MaxTurns: u32 = 50;
    pub const BattleTimeout: BlockNumber = 100;
}

impl pallet_meowstar_pet::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Randomness = RandomnessCollectiveFlip;
    type MaxPetsPerAccount = MaxPetsPerAccount;
    type HatchingFee = HatchingFee;
    type LevelUpBaseFee = LevelUpBaseFee;
    type EvolutionFeeMultiplier = EvolutionFeeMultiplier;
    type WeightInfo = pallet_meowstar_pet::weights::SubstrateWeight<Runtime>;
}

impl pallet_meowstar_battle::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type BattleRandomness = RandomnessCollectiveFlip;
    type BattleEntryFee = BattleEntryFee;
    type MaxTurns = MaxTurns;
    type BattleTimeout = BattleTimeout;
    type BattleWeightInfo = pallet_meowstar_battle::weights::SubstrateWeight<Runtime>;
}
```

### pallet-meowstar-staking

质押系统 Pallet，实现：

- **质押**: `stake(amount, lock_period)` - 质押 COS 代币
- **解除质押**: `unstake(stake_id)` - 解锁后取回质押
- **领取收益**: `claim_rewards(stake_id)` - 领取质押收益
- **充值奖励池**: `fund_reward_pool(amount)` - 管理员充值

#### 锁定期选项

| 类型 | 锁定时间 | APY | 投票权重 |
|------|----------|-----|----------|
| Flexible | 0 | 5% | 1.0x |
| Days30 | 30天 | 12% | 1.2x |
| Days90 | 90天 | 20% | 1.5x |
| Days180 | 180天 | 30% | 2.0x |
| Days365 | 365天 | 50% | 3.0x |

### pallet-meowstar-governance

DAO 治理 Pallet，实现：

- **创建提案**: `create_proposal(type, title_hash, desc_hash)` - 创建治理提案
- **投票**: `vote(proposal_id, approve)` - 对提案投票
- **结算提案**: `finalize_proposal(proposal_id)` - 投票结束后结算
- **执行提案**: `execute_proposal(proposal_id)` - 执行通过的提案
- **紧急暂停**: `emergency_pause()` / `emergency_unpause()` - 紧急控制

#### 提案类型

- `General` - 普通提案
- `ParameterChange` - 参数修改
- `TreasurySpend` - 国库支出
- `Emergency` - 紧急提案

### pallet-meowstar-marketplace

NFT 市场 Pallet，实现：

- **固定价格挂单**: `list_fixed_price(pet_id, price, duration)` - 固定价格出售
- **拍卖挂单**: `list_auction(pet_id, starting_price, duration)` - 拍卖出售
- **取消挂单**: `cancel_listing(listing_id)` - 取消出售
- **购买**: `buy(listing_id)` - 购买固定价格商品
- **出价**: `place_bid(listing_id, amount)` - 拍卖出价
- **结束拍卖**: `end_auction(listing_id)` - 结束拍卖并结算

---

## 开发状态

- [x] pallet-meowstar-pet - ✅ 完成
- [x] pallet-meowstar-battle - ✅ 完成
- [x] pallet-meowstar-staking - ✅ 完成
- [x] pallet-meowstar-governance - ✅ 完成
- [x] pallet-meowstar-marketplace - ✅ 完成
- [x] Runtime 集成 - ✅ 完成
- [x] 节点编译 - ✅ 完成
- [x] 前端集成 - ✅ 完成
- [ ] 单元测试
- [ ] Benchmarks

## 目录结构

```
pallets/meowstar/
├── README.md
├── pet/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── weights.rs
│       ├── mock.rs
│       └── tests.rs
├── battle/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── weights.rs
├── staking/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── weights.rs
├── governance/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── weights.rs
└── marketplace/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        └── weights.rs
```

## 前端集成

前端代码位于 `frontend/` 目录：

```
frontend/
├── app/meowstar/           # Expo Router 页面
│   ├── index.tsx           # 主页
│   ├── pets.tsx            # 宠物列表
│   ├── pet/[id].tsx        # 宠物详情
│   ├── battle.tsx          # 战斗
│   ├── marketplace.tsx     # 市场
│   ├── staking.tsx         # 质押
│   ├── governance.tsx      # 治理
│   └── chat.tsx            # AI 陪伴
├── src/services/
│   ├── meowstar.ts         # 链上服务
│   └── aiCompanion.ts      # AI 陪伴服务
├── src/hooks/
│   ├── useMeowstar.ts      # React Hooks
│   └── useAICompanion.ts   # AI Hooks
└── src/stores/
    └── meowstarStore.ts    # Zustand Store
```

---

*版本: v0.2.0*
*更新: 2026-01-30*
