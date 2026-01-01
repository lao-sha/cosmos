# 大六壬排盘 Pallet (pallet-daliuren)

大六壬区块链占卜系统 - 中国古代三式之首的链上实现

## 概述

大六壬是中国古代三式之一（太乙、奇门、六壬），以天人合一、阴阳五行为理论基础，通过起课、定三传来预测吉凶。本模块在 Substrate 区块链上实现完整的大六壬排盘功能，支持时间起课、随机起课、手动指定三种起课方式。

## 架构图

```
┌────────────────────────────────────────────────────────────────────────┐
│                        pallet-daliuren                                 │
├────────────────────────────────────────────────────────────────────────┤
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐            │
│  │  时间起课      │  │  随机起课      │  │  手动指定      │            │
│  │ divine_by_time │  │ divine_random  │  │ divine_manual  │            │
│  └───────┬────────┘  └───────┬────────┘  └───────┬────────┘            │
│          │                   │                   │                     │
│          └───────────────────┼───────────────────┘                     │
│                              ▼                                         │
│              ┌───────────────────────────────┐                         │
│              │       algorithm.rs            │                         │
│              │  ┌─────────────────────────┐  │                         │
│              │  │ 1. 天盘计算             │  │                         │
│              │  │ 2. 天将盘计算           │  │                         │
│              │  │ 3. 四课起法             │  │                         │
│              │  │ 4. 三传取法（九种课式） │  │                         │
│              │  │ 5. 空亡计算             │  │                         │
│              │  └─────────────────────────┘  │                         │
│              └───────────────┬───────────────┘                         │
│                              ▼                                         │
│              ┌───────────────────────────────┐                         │
│              │        DaLiuRenPan            │                         │
│              │  ┌─────────────────────────┐  │                         │
│              │  │ • tian_pan (天盘)       │  │                         │
│              │  │ • tian_jiang_pan (天将) │  │                         │
│              │  │ • si_ke (四课)          │  │                         │
│              │  │ • san_chuan (三传)      │  │                         │
│              │  │ • ke_shi (课式)         │  │                         │
│              │  │ • ge_ju (格局)          │  │                         │
│              │  │ • xun_kong (空亡)       │  │                         │
│              │  └─────────────────────────┘  │                         │
│              └───────────────┬───────────────┘                         │
│                              ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                        Storage                                   │  │
│  │  Pans | UserPans | PublicPans | DailyPanCount | AiRequests      │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

## 功能特性

- **时间起课**：根据年月日时干支、月将、占时进行传统起课
- **随机起课**：使用链上随机数生成月将和占时
- **手动指定**：完全手动指定所有参数，用于复盘或教学
- **式盘存储**：完整保存天盘、四课、三传等信息
- **AI 智能解读**：支持 AI 解读请求和结果存储
- **公开/私有管理**：式盘可设置公开或私有

## 大六壬核心概念

### 天盘

月将加占时，天盘顺时针旋转。十二地支在天盘上的位置由月将和占时决定。

```
月将加时辰，天盘顺时转。
干支起四课，克贼定三传。
```

### 四课

| 课位 | 名称 | 取法 |
|------|------|------|
| 第一课 | 干阳神 | 日干寄宫上神 |
| 第二课 | 干阴神 | 干阳神上神 |
| 第三课 | 支阳神 | 日支上神 |
| 第四课 | 支阴神 | 支阳神上神 |

### 三传

三传是大六壬的核心，分为初传、中传、末传，根据九种课式推导：
- **初传**：发用之神，事情的起始
- **中传**：经过之神，事情的发展
- **末传**：归宿之神，事情的结果

### 十二天将

| 天将 | 五行 | 吉凶 | 象意 |
|------|------|------|------|
| 贵人 | 土 | 吉 | 尊贵、官员、贵人相助 |
| 螣蛇 | 火 | 凶 | 惊恐、怪异、缠绕 |
| 朱雀 | 火 | 凶 | 口舌、文书、信息 |
| 六合 | 木 | 吉 | 和合、婚姻、中介 |
| 勾陈 | 土 | 凶 | 争斗、官司、纠缠 |
| 青龙 | 木 | 吉 | 喜庆、财利、进取 |
| 天空 | 土 | 凶 | 空虚、欺诈、虚假 |
| 白虎 | 金 | 凶 | 凶险、疾病、丧事 |
| 太常 | 土 | 吉 | 饮食、衣服、喜庆 |
| 玄武 | 水 | 凶 | 盗贼、阴私、暗昧 |
| 太阴 | 金 | 吉 | 阴私、暗中、女贵 |
| 天后 | 水 | 吉 | 后妃、阴贵、女性 |

### 十二月将

| 月将 | 地支 | 节气范围 |
|------|------|----------|
| 神后 | 子 | 大雪后 |
| 大吉 | 丑 | 小寒后 |
| 功曹 | 寅 | 立春后 |
| 太冲 | 卯 | 惊蛰后 |
| 天罡 | 辰 | 清明后 |
| 太乙 | 巳 | 立夏后 |
| 胜光 | 午 | 芒种后 |
| 小吉 | 未 | 小暑后 |
| 传送 | 申 | 立秋后 |
| 从魁 | 酉 | 白露后 |
| 河魁 | 戌 | 寒露后 |
| 登明 | 亥 | 立冬后 |

## 九种课式（取三传方法）

### 1. 贼克课

四课中有下克上（贼）或上克下（克）关系时使用。

- **元首课**：一上克下，取克者为初传
- **重审课**：一下贼上，取贼者为初传

### 2. 比用课

贼克多于一个时，取与日干阴阳相同者。

### 3. 涉害课

比用仍有多个时，计算涉害深度，取最深者。

- **涉害格**：深度不同，取最深
- **见机格**：深度相同，从孟发用（寅申巳亥）
- **察微格**：深度相同，从仲发用（子午卯酉）
- **复等格**：全同，阳干取干阳神，阴干取支阳神

### 4. 遥克课

四课无贼克时，取二三四课中克日干者或被日干克者。

### 5. 昂星课

四课俱全无克时使用。

- **虎视格**：阳干取酉上神
- **冬蛇掩目**：阴干取酉所临

### 6. 别责课

三课不备时使用。

- 阳干：取干合神上神
- 阴干：取支前四位

### 7. 八专课

八专日（甲寅、庚申、丁未、己未）特殊取法。

### 8. 伏吟课

天盘与地盘相同时使用。

- **自任格**：阳干取干阳神所刑
- **自信格**：阴干取支阳神所刑

### 9. 返吟课

天盘与地盘六冲时使用。

- 有贼克则用贼克
- **无依格**：无贼克取驿马为初传

## 起课流程图

```
用户发起起课请求
        │
        ▼
┌───────────────────┐
│ 检查每日起课限制  │
│ 收取起课费用      │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│   准备起课参数    │
├───────────────────┤
│ • 年月日时干支    │
│ • 月将            │
│ • 占时            │
│ • 昼夜            │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ Step 1: 计算天盘  │
│ 月将加占时旋转    │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ Step 2: 计算天将盘│
│ 贵人起法 + 顺逆布 │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ Step 3: 起四课    │
│ 干阳→干阴→支阳→支阴│
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ Step 4: 定三传    │
│ 九种课式选择      │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ Step 5: 计算空亡  │
│ 日干支旬空        │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ 存储 DaLiuRenPan  │
│ 发送 PanCreated   │
└───────────────────┘
```

## 存储结构

| 存储项 | 类型 | 描述 |
|--------|------|------|
| `NextPanId` | `u64` | 下一个式盘 ID |
| `Pans` | `Map<u64, DaLiuRenPan>` | 式盘详情存储 |
| `UserPans` | `DoubleMap<AccountId, u64, bool>` | 用户式盘索引 |
| `PublicPans` | `Map<u64, BlockNumber>` | 公开式盘索引 |
| `DailyPanCount` | `DoubleMap<AccountId, Day, u32>` | 每日起课计数 |
| `AiInterpretationRequests` | `Map<u64, BlockNumber>` | AI 解读请求队列 |
| `UserStatsStorage` | `Map<AccountId, UserStats>` | 用户统计数据 |

## 数据结构

### DaLiuRenPan（式盘）

```rust
pub struct DaLiuRenPan {
    // 基本信息
    pub id: u64,
    pub creator: AccountId,
    pub created_at: BlockNumber,
    pub method: DivinationMethod,
    pub question_cid: Option<BoundedVec<u8, MaxCidLen>>,

    // 时间干支
    pub year_gz: (TianGan, DiZhi),
    pub month_gz: (TianGan, DiZhi),
    pub day_gz: (TianGan, DiZhi),
    pub hour_gz: (TianGan, DiZhi),

    // 起课参数
    pub yue_jiang: DiZhi,   // 月将
    pub zhan_shi: DiZhi,    // 占时
    pub is_day: bool,       // 昼夜

    // 式盘数据
    pub tian_pan: TianPan,           // 天盘
    pub tian_jiang_pan: TianJiangPan,// 天将盘
    pub si_ke: SiKe,                 // 四课
    pub san_chuan: SanChuan,         // 三传

    // 课式与格局
    pub ke_shi: KeShiType,  // 课式类型
    pub ge_ju: GeJuType,    // 格局类型

    // 其他
    pub xun_kong: (DiZhi, DiZhi),    // 空亡
    pub is_public: bool,
    pub ai_interpretation_cid: Option<BoundedVec<u8, MaxCidLen>>,
}
```

### SanChuan（三传）

```rust
pub struct SanChuan {
    pub chu: DiZhi,           // 初传
    pub zhong: DiZhi,         // 中传
    pub mo: DiZhi,            // 末传
    pub chu_jiang: TianJiang, // 初传天将
    pub zhong_jiang: TianJiang,
    pub mo_jiang: TianJiang,
    pub chu_qin: LiuQin,      // 初传六亲
    pub zhong_qin: LiuQin,
    pub mo_qin: LiuQin,
    pub chu_dun: Option<TianGan>,  // 初传遁干
    pub zhong_dun: Option<TianGan>,
    pub mo_dun: Option<TianGan>,
}
```

## 可调用函数

### `divine_by_time`

时间起课 - 根据干支历进行起课。

**参数**：
- `year_gz`: (u8, u8) - 年干支（天干索引, 地支索引）
- `month_gz`: (u8, u8) - 月干支
- `day_gz`: (u8, u8) - 日干支
- `hour_gz`: (u8, u8) - 时干支
- `yue_jiang`: u8 - 月将（地支索引）
- `zhan_shi`: u8 - 占时（地支索引）
- `is_day`: bool - 是否昼占
- `question_cid`: Option<BoundedVec<u8>> - 问题 IPFS CID（可选）

### `divine_random`

随机起课 - 使用链上随机数。

**参数**：
- `day_gz`: (u8, u8) - 日干支
- `question_cid`: Option<BoundedVec<u8>> - 问题 IPFS CID（可选）

### `divine_manual`

手动指定 - 完全手动指定所有参数。

**参数**：同 `divine_by_time`

### `request_ai_interpretation`

请求 AI 解读 - 需支付费用。

**参数**：
- `pan_id`: u64 - 式盘 ID

### `submit_ai_interpretation`

提交 AI 解读结果 - 仅限授权节点。

**参数**：
- `pan_id`: u64 - 式盘 ID
- `interpretation_cid`: BoundedVec<u8> - 解读内容 IPFS CID

### `set_pan_visibility`

更改式盘公开状态。

**参数**：
- `pan_id`: u64 - 式盘 ID
- `is_public`: bool - 是否公开

## 配置参数

| 参数 | 类型 | 描述 |
|------|------|------|
| `MaxCidLen` | u32 | IPFS CID 最大长度（默认 64） |
| `MaxDailyDivinations` | u32 | 每日最大起课次数（默认 100） |
| `DivinationFee` | Balance | 起课费用（默认 1 DUST） |
| `AiInterpretationFee` | Balance | AI 解读费用（默认 5 DUST） |

## 事件

| 事件 | 描述 |
|------|------|
| `PanCreated` | 式盘已创建（包含课式和格局信息） |
| `AiInterpretationRequested` | AI 解读已请求 |
| `AiInterpretationSubmitted` | AI 解读已提交 |
| `PanVisibilityChanged` | 式盘可见性已更改 |

## 错误类型

| 错误 | 描述 |
|------|------|
| `PanNotFound` | 式盘不存在 |
| `NotAuthorized` | 无权操作 |
| `DailyLimitExceeded` | 超出每日限额 |
| `InsufficientBalance` | 余额不足 |
| `CidTooLong` | CID 过长 |
| `AiInterpretationAlreadyRequested` | AI 解读已请求 |
| `AiInterpretationNotRequested` | AI 解读未请求 |
| `AiInterpretationAlreadySubmitted` | AI 解读已完成 |
| `InvalidGanZhi` | 无效的干支组合 |
| `InvalidYueJiang` | 无效的月将 |
| `InvalidZhanShi` | 无效的占时 |

## 天干地支对照

### 十天干

| 索引 | 天干 | 五行 | 阴阳 |
|------|------|------|------|
| 0 | 甲 | 木 | 阳 |
| 1 | 乙 | 木 | 阴 |
| 2 | 丙 | 火 | 阳 |
| 3 | 丁 | 火 | 阴 |
| 4 | 戊 | 土 | 阳 |
| 5 | 己 | 土 | 阴 |
| 6 | 庚 | 金 | 阳 |
| 7 | 辛 | 金 | 阴 |
| 8 | 壬 | 水 | 阳 |
| 9 | 癸 | 水 | 阴 |

### 十二地支

| 索引 | 地支 | 五行 | 时辰 | 类型 |
|------|------|------|------|------|
| 0 | 子 | 水 | 23:00-01:00 | 仲 |
| 1 | 丑 | 土 | 01:00-03:00 | 季 |
| 2 | 寅 | 木 | 03:00-05:00 | 孟 |
| 3 | 卯 | 木 | 05:00-07:00 | 仲 |
| 4 | 辰 | 土 | 07:00-09:00 | 季 |
| 5 | 巳 | 火 | 09:00-11:00 | 孟 |
| 6 | 午 | 火 | 11:00-13:00 | 仲 |
| 7 | 未 | 土 | 13:00-15:00 | 季 |
| 8 | 申 | 金 | 15:00-17:00 | 孟 |
| 9 | 酉 | 金 | 17:00-19:00 | 仲 |
| 10 | 戌 | 土 | 19:00-21:00 | 季 |
| 11 | 亥 | 水 | 21:00-23:00 | 孟 |

## 天干寄宫表

| 天干 | 寄宫 |
|------|------|
| 甲 | 寅 |
| 乙 | 辰 |
| 丙 | 巳 |
| 丁 | 未 |
| 戊 | 巳 |
| 己 | 未 |
| 庚 | 申 |
| 辛 | 戌 |
| 壬 | 亥 |
| 癸 | 丑 |

## 贵人起法

### 昼贵人表

| 天干 | 昼贵 |
|------|------|
| 甲 | 未 |
| 乙 | 申 |
| 丙 | 酉 |
| 丁 | 亥 |
| 戊 | 丑 |
| 己 | 子 |
| 庚 | 丑 |
| 辛 | 寅 |
| 壬 | 卯 |
| 癸 | 巳 |

### 夜贵人表

| 天干 | 夜贵 |
|------|------|
| 甲 | 丑 |
| 乙 | 子 |
| 丙 | 亥 |
| 丁 | 酉 |
| 戊 | 未 |
| 己 | 申 |
| 庚 | 未 |
| 辛 | 午 |
| 壬 | 巳 |
| 癸 | 卯 |

## 六亲对照

| 六亲 | 关系 | 象意 |
|------|------|------|
| 兄弟 | 同我者 | 朋友、同事、竞争 |
| 父母 | 生我者 | 长辈、文书、保护 |
| 官鬼 | 克我者 | 官员、压力、疾病 |
| 妻财 | 我克者 | 财物、妻子、收益 |
| 子孙 | 我生者 | 后代、福德、消解 |

## 使用示例

```rust
// 时间起课（甲子日，月将午，占时子，昼占）
DaLiuRen::divine_by_time(
    origin,
    (0, 0),   // 年干支：甲子
    (0, 0),   // 月干支：甲子
    (0, 0),   // 日干支：甲子
    (0, 0),   // 时干支：甲子
    6,        // 月将：午
    0,        // 占时：子
    true,     // 昼占
    None,     // 问题 CID
)?;

// 随机起课
DaLiuRen::divine_random(
    origin,
    (0, 0),   // 日干支：甲子
    Some(question_cid),
)?;

// 请求 AI 解读
DaLiuRen::request_ai_interpretation(origin, pan_id)?;

// 设置式盘公开
DaLiuRen::set_pan_visibility(origin, pan_id, true)?;
```

## 前端集成示例

### Polkadot.js API 调用

```typescript
import { ApiPromise, WsProvider } from '@polkadot/api';

const wsProvider = new WsProvider('ws://localhost:9944');
const api = await ApiPromise.create({ provider: wsProvider });

// 时间起课
const tx = api.tx.daliuren.divineByTime(
  [0, 0],    // 年干支
  [0, 0],    // 月干支
  [0, 0],    // 日干支
  [0, 0],    // 时干支
  6,         // 月将（午）
  0,         // 占时（子）
  true,      // 昼占
  null       // 问题CID
);
await tx.signAndSend(account);

// 随机起课
const randomTx = api.tx.daliuren.divineRandom([0, 0], null);
await randomTx.signAndSend(account);

// 查询式盘
const pan = await api.query.daliuren.pans(panId);
console.log('式盘:', pan.toHuman());

// 查询用户统计
const stats = await api.query.daliuren.userStatsStorage(accountId);
console.log('统计:', stats.toHuman());
```

### React Hooks 封装

```typescript
import { useQuery, useMutation } from '@tanstack/react-query';
import { useApi } from './useApi';

// 查询式盘
export function usePan(panId: number) {
  const api = useApi();
  return useQuery({
    queryKey: ['daliuren', 'pan', panId],
    queryFn: async () => {
      const pan = await api.query.daliuren.pans(panId);
      return pan.toHuman();
    },
  });
}

// 时间起课
export function useDivineByTime() {
  const api = useApi();
  return useMutation({
    mutationFn: async (params: DivineByTimeParams) => {
      const tx = api.tx.daliuren.divineByTime(
        params.yearGz,
        params.monthGz,
        params.dayGz,
        params.hourGz,
        params.yueJiang,
        params.zhanShi,
        params.isDay,
        params.questionCid
      );
      return tx;
    },
  });
}
```

## 测试

运行测试：

```bash
SKIP_WASM_BUILD=1 cargo test -p pallet-daliuren
```

测试覆盖：
- 天盘计算算法验证
- 天将盘顺逆布置验证
- 四课起法验证
- 九种课式（贼克、比用、涉害、遥克、昂星、别责、八专、伏吟、返吟）
- 三传取法验证
- 空亡计算验证
- 贵人起法（昼贵/夜贵）验证
- AI 解读请求/提交测试
- 式盘可见性管理测试
- 每日限制功能测试

## 版本历史

- **v0.1.0** (2025-12-01)：初始版本
  - 实现三种起课方式
  - 完整九种课式算法
  - 天将盘自动排布
  - AI 解读集成
  - 完整测试覆盖（43 个测试）

## 相关模块

- **pallet-liuyao**：六爻纳甲排盘系统
- **pallet-qimen**：奇门遁甲排盘系统
- **pallet-ziwei**：紫微斗数排盘系统
- **pallet-meihua**：梅花易数排盘系统
- **pallet-xiaoliuren**：小六壬排盘系统
- **pallet-tarot**：塔罗牌排盘系统

## 许可证

Unlicense
