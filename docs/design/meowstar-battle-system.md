# 喵星宇宙 (Meowstar Universe) - 战斗系统深度设计

## 1. 战斗系统架构

```
┌─────────────────────────────────────────────────────────┐
│                    战斗系统架构                          │
├─────────────────────────────────────────────────────────┤
│  匹配层 -> 战斗层 -> 结算层                              │
│    │         │         │                                │
│  ELO匹配   回合引擎   奖励计算                           │
│  队列管理   伤害计算   积分更新                           │
│  超时处理   状态管理   记录存储                           │
└─────────────────────────────────────────────────────────┘
```

## 2. 战斗流程

### 2.1 四阶段流程

1. **匹配阶段**: 加入队列 → ELO匹配 → 确认对手 → 锁定阵容
2. **准备阶段**: 阵容展示 → 策略选择 → 生成战斗种子
3. **战斗阶段**: 回合开始 → 行动选择 → 行动执行 → 状态更新 → 胜负判定
4. **结算阶段**: 胜负确认 → 积分计算 → 奖励发放 → 记录存储

### 2.2 行动顺序

```
优先级判定：
- 切换宠物: +6
- 使用道具: +5
- 先制技能: +1 到 +4
- 普通技能: 0
- 后制技能: -1 到 -4

Final_Order = Priority * 1000 + Speed + Random(0-99)
```

## 3. 属性系统

| 属性 | 缩写 | 作用 |
|------|------|------|
| 生命值 | HP | 生存能力 |
| 攻击力 | ATK | 物理伤害 |
| 防御力 | DEF | 物理减免 |
| 特攻 | SP_ATK | 特殊伤害 |
| 特防 | SP_DEF | 特殊减免 |
| 速度 | SPD | 行动顺序 |
| 暴击率 | CRIT | 暴击概率 |
| 暴击伤害 | CRIT_DMG | 暴击倍率 |

### 元素克制

| 攻击\防御 | 火 | 水 | 暗影 | 光明 |
|-----------|-----|-----|------|------|
| 火 | 1.0x | 0.5x | 1.5x | 1.0x |
| 水 | 1.5x | 1.0x | 1.0x | 0.5x |
| 暗影 | 0.5x | 1.0x | 1.0x | 1.5x |
| 光明 | 1.0x | 1.5x | 0.5x | 1.0x |

## 4. 伤害计算公式

```
Step 1: Base_Damage = Skill_Power * ATK / 100
Step 2: Defense_Factor = DEF / (DEF + 500 + Level * 5)
Step 3: After_Defense = Base_Damage * (1 - Defense_Factor)
Step 4: After_Element = After_Defense * Element_Multiplier
Step 5: After_Crit = After_Element * (is_crit ? Crit_Damage : 1.0)
Step 6: Final_Damage = After_Crit * Random(0.85-1.0)
```

## 5. 技能系统

### 5.1 技能类型

| 类型 | 描述 |
|------|------|
| Physical | 物理攻击，基于ATK |
| Special | 特殊攻击，基于SP_ATK |
| Status | 状态技能，施加效果 |
| Support | 辅助技能，增益/回复 |
| Ultimate | 终极技能，高威力高消耗 |

### 5.2 效果类型

**控制效果**: 眩晕、冰冻、沉默、睡眠、混乱、嘲讽

**增益效果**: 攻击提升、防御提升、速度提升、暴击提升、护盾、回复

**减益效果**: 攻击降低、防御降低、速度降低、中毒、灼烧、易伤

## 6. 战斗AI逻辑

### 6.1 AI决策树

```
1. 生存检查: HP < 30% → 优先回复/切换
2. 控制检查: 对手可被控制 → 使用控制技能
3. 克制检查: 有克制优势 → 使用克制技能
4. 伤害最大化: 选择最高伤害技能
5. 默认行动: 普通攻击
```

### 6.2 AI难度等级

| 等级 | 决策深度 | 预判回合 | 失误率 |
|------|----------|----------|--------|
| Easy | 1层 | 0 | 30% |
| Normal | 2层 | 1 | 15% |
| Hard | 3层 | 2 | 5% |
| Expert | 4层 | 3 | 0% |

## 7. 链上验证

### 7.1 随机数生成

```rust
// 使用 VRF 生成战斗种子
battle_seed = VRF(block_hash, battle_id, player1, player2)

// 每回合随机数
turn_random = hash(battle_seed, turn_number, action_index)
```

### 7.2 防作弊机制

- **承诺-揭示**: 双方先提交行动哈希，再揭示
- **超时惩罚**: 超时自动判负
- **重放验证**: 所有行动可链上重放验证
- **异常检测**: 检测不可能的伤害值

## 8. 数据结构

```rust
pub struct Battle<T: Config> {
    pub id: u64,
    pub player1: T::AccountId,
    pub player2: T::AccountId,
    pub status: BattleStatus,
    pub current_turn: u32,
    pub battle_seed: [u8; 32],
    pub winner: Option<T::AccountId>,
    pub started_at: BlockNumberFor<T>,
}

pub enum BattleStatus {
    Matching,
    Preparing,
    InProgress,
    Finished,
    Cancelled,
}

pub struct BattleAction {
    pub action_type: ActionType,
    pub skill_id: Option<u32>,
    pub target: Option<u8>,
}
```

## 9. 奖励系统

| 结果 | 积分变化 | 代币奖励 | 经验奖励 |
|------|----------|----------|----------|
| 胜利 | +15~30 | 100 MEOW | 基础*1.5 |
| 失败 | -10~20 | 20 MEOW | 基础*0.5 |
| 平局 | 0 | 50 MEOW | 基础*1.0 |

积分变化公式 (ELO):
```
K = 32
Expected = 1 / (1 + 10^((Opponent_Rating - My_Rating) / 400))
New_Rating = Old_Rating + K * (Result - Expected)
```

---

*文档版本: v1.0 | 最后更新: 2026-01-30*
