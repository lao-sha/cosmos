# 小六壬排盘 Pallet (pallet-xiaoliuren)

小六壬区块链占卜系统 - 中国传统掐指速算术的链上实现

## 概述

小六壬又称"诸葛亮马前课"或"掐指速算"，是中国古代流传的一种简易占卜术。本模块在 Substrate 区块链上实现完整的小六壬排盘功能，通过六宫（大安、留连、速喜、赤口、小吉、空亡）来预测吉凶。

## 架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                    pallet-xiaoliuren                            │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │ 时间起课    │  │ 数字起课    │  │ 随机起课    │  手动指定    │
│  │ divine_by   │  │ divine_by   │  │ divine      │  divine      │
│  │ _time       │  │ _number     │  │ _random     │  _manual     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘              │
│         │                │                │                     │
│         └────────────────┼────────────────┘                     │
│                          ▼                                      │
│              ┌───────────────────────┐                          │
│              │    algorithm.rs       │                          │
│              │  ┌─────────────────┐  │                          │
│              │  │ 月宫 → 日宫 → 时宫│  │                          │
│              │  │  (三宫计算)     │  │                          │
│              │  └─────────────────┘  │                          │
│              └───────────┬───────────┘                          │
│                          ▼                                      │
│              ┌───────────────────────┐                          │
│              │     XiaoLiuRenPan     │                          │
│              │  ┌─────────────────┐  │                          │
│              │  │ SanGong (三宫)  │  │                          │
│              │  │ - yue_gong      │  │                          │
│              │  │ - ri_gong       │  │                          │
│              │  │ - shi_gong      │  │                          │
│              │  └─────────────────┘  │                          │
│              └───────────┬───────────┘                          │
│                          ▼                                      │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Storage                               │   │
│  │  Pans | UserPans | PublicPans | DailyDivinationCount    │   │
│  └─────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                    DivinationProvider                           │
│            (集成 pallet-divination-common)                      │
└─────────────────────────────────────────────────────────────────┘
```

## 功能特性

- **时间起课**：按农历月日时起课（传统方法）
- **数字起课**：活数起课法，三个数字计算三宫
- **随机起课**：使用链上随机数生成
- **手动指定**：直接指定三宫结果
- **AI 智能解读**：IPFS 存储解读结果
- **课盘管理**：公开/私有设置、用户统计

## 六宫详解

| 六宫 | 五行 | 天将 | 方位 | 吉凶 | 含义 |
|------|------|------|------|------|------|
| 大安 | 木 | 青龙 | 东方 | 大吉 | 身不动时，吉祥安康 |
| 留连 | 水 | 玄武 | 北方 | 凶 | 人未归时，延迟纠缠 |
| 速喜 | 火 | 朱雀 | 南方 | 吉 | 人即至时，快速喜庆 |
| 赤口 | 金 | 白虎 | 西方 | 凶 | 官事凶时，口舌是非 |
| 小吉 | 木 | 六合 | 东南 | 吉 | 人来喜时，和合吉利 |
| 空亡 | 土 | 勾陈 | 中央 | 凶 | 音信稀时，无果忧虑 |

### 六宫卦辞

#### 大安
> 大安事事昌，求谋在东方，失物去不远。宅舍保平安，行人身未动，病者主无妨，将军回田野，仔细更推详。

**详解**：身不动时，五行属木，颜色青色，方位东方。临青龙。有静止、心安、吉祥之含义。

#### 留连
> 留连事难成，求谋日未明，官事只宜缓。去者来回程，失物南方见，急讨方遂心。更需防口舌，人事且平平。

**详解**：人未归时，五行属水，颜色黑色，方位北方。临玄武。有暗味不明、延迟、纠缠、拖延之含义。

#### 速喜
> 速喜喜来临，求财向南行，失物申未午。逢人路上寻，官事有福德，病者无祸侵，田宅六畜吉，行人有音信。

**详解**：人即至时，五行属火，颜色红色，方位南方。临朱雀。有快速、喜庆、吉利之含义。指时机已到。

#### 赤口
> 赤口主口舌，官非切要防，失物急去寻，行人有惊慌。鸡犬多作怪，病者出西方，更须防咀咒，恐怕染瘟殃。

**详解**：官事凶时，五行属金，颜色白色，方位西方。临白虎。有不吉、惊恐、凶险、口舌是非之含义。

#### 小吉
> 小吉最吉昌，路上好商量，阴人来报喜。失物在坤方，行人立便至，交易甚是强，凡事皆和合，病者祈上苍。

**详解**：人来喜时，五行属木，临六合。有和合、吉利之含义。

#### 空亡
> 空亡事不祥，阴人多乖张，求财无利益。行人有灾殃，失物寻不见，官事有刑伤。病人逢暗鬼，祈解可安康。

**详解**：音信稀时，五行属土，颜色黄色，方位中央。临勾陈。有不吉、无结果、忧虑之含义。

## 起课算法

### 时间起课（传统方法）

按农历月日时起课：

1. **月宫**：从大安起正月，顺数至所求月份
   - 月宫 = (农历月 - 1) % 6

2. **日宫**：从月宫起初一，顺数至所求日期
   - 日宫 = (月宫索引 + 农历日 - 1) % 6

3. **时宫**：从日宫起子时，顺数至所求时辰
   - 时宫 = (日宫索引 + 时辰序号 - 1) % 6

### 数字起课（活数起课法）

取三个数字 x、y、z：

- 月宫 = (x - 1) % 6
- 日宫 = (x + y - 2) % 6
- 时宫 = (x + y + z - 3) % 6

### 随机起课

使用链上随机数生成三个数字（1-60 范围），然后按数字起课法计算。

## 存储结构

| 存储项 | 类型 | 描述 |
|--------|------|------|
| `NextPanId` | `u64` | 下一个课盘 ID |
| `Pans` | `Map<u64, XiaoLiuRenPan>` | 课盘详情存储 |
| `UserPans` | `Map<AccountId, Vec<u64>>` | 用户课盘索引 |
| `PublicPans` | `Vec<u64>` | 公开课盘列表 |
| `DailyDivinationCount` | `DoubleMap<AccountId, Day, u32>` | 每日起课计数 |
| `AiInterpretationRequests` | `Map<u64, AccountId>` | AI 解读请求队列 |
| `UserStatsStorage` | `Map<AccountId, UserStats>` | 用户统计数据 |

## 可调用函数

### `divine_by_time`

时间起课 - 使用农历月日时起课。

**参数**：
- `lunar_month`: u8 - 农历月份（1-12）
- `lunar_day`: u8 - 农历日期（1-30）
- `hour`: u8 - 当前小时（0-23）
- `question_cid`: Option<BoundedVec<u8>> - 问题 IPFS CID（可选）
- `is_public`: bool - 是否公开

### `divine_by_number`

数字起课 - 活数起课法。

**参数**：
- `x`: u8 - 第一个数字（≥1）
- `y`: u8 - 第二个数字（≥1）
- `z`: u8 - 第三个数字（≥1）
- `question_cid`: Option<BoundedVec<u8>> - 问题 IPFS CID（可选）
- `is_public`: bool - 是否公开

### `divine_random`

随机起课 - 使用链上随机数。

**参数**：
- `question_cid`: Option<BoundedVec<u8>> - 问题 IPFS CID（可选）
- `is_public`: bool - 是否公开

### `divine_manual`

手动指定起课 - 直接指定三宫。

**参数**：
- `yue_index`: u8 - 月宫索引（0-5）
- `ri_index`: u8 - 日宫索引（0-5）
- `shi_index`: u8 - 时宫索引（0-5）
- `question_cid`: Option<BoundedVec<u8>> - 问题 IPFS CID（可选）
- `is_public`: bool - 是否公开

### `request_ai_interpretation`

请求 AI 解读 - 需支付费用。

**参数**：
- `pan_id`: u64 - 课盘 ID

### `submit_ai_interpretation`

提交 AI 解读结果 - 仅限授权节点。

**参数**：
- `pan_id`: u64 - 课盘 ID
- `interpretation_cid`: BoundedVec<u8> - 解读内容 IPFS CID

### `set_pan_visibility`

更改课盘公开状态。

**参数**：
- `pan_id`: u64 - 课盘 ID
- `is_public`: bool - 是否公开

## 配置参数

| 参数 | 类型 | 描述 |
|------|------|------|
| `MaxUserPans` | u32 | 每用户最大课盘数（默认 1000） |
| `MaxPublicPans` | u32 | 公开课盘最大数（默认 10000） |
| `MaxCidLen` | u32 | IPFS CID 最大长度（默认 64） |
| `DailyFreeDivinations` | u32 | 每日免费起课次数（默认 3） |
| `MaxDailyDivinations` | u32 | 每日最大起课次数（默认 100） |
| `AiInterpretationFee` | Balance | AI 解读费用（默认 5 DUST） |

## 事件

| 事件 | 描述 |
|------|------|
| `PanCreated` | 新课盘创建成功 |
| `AiInterpretationRequested` | AI 解读请求已提交 |
| `AiInterpretationSubmitted` | AI 解读结果已提交 |
| `PanVisibilityChanged` | 课盘公开状态已更改 |

## 错误类型

| 错误 | 描述 |
|------|------|
| `PanNotFound` | 课盘不存在 |
| `NotOwner` | 非课盘所有者 |
| `DailyLimitExceeded` | 每日起课次数超限 |
| `InvalidLunarMonth` | 无效的农历月份 |
| `InvalidLunarDay` | 无效的农历日期 |
| `InvalidHour` | 无效的时辰 |
| `UserPansFull` | 用户课盘列表已满 |
| `PublicPansFull` | 公开课盘列表已满 |
| `InsufficientFee` | AI 解读费用不足 |
| `AiRequestAlreadyExists` | AI 解读请求已存在 |
| `AiRequestNotFound` | AI 解读请求不存在 |
| `InvalidParams` | 无效的起课参数 |
| `NumberMustBePositive` | 数字起课参数必须大于 0 |

## 与 pallet-divination-common 集成

本模块实现了 `DivinationProvider` trait，支持：

- 结果查询（`result_exists`, `result_creator`）
- 稀有度计算（`rarity_data`）
- NFT 兼容（`is_nftable`, `mark_as_nfted`）
- 结果摘要（`result_summary`）

### 稀有度计算规则

| 情况 | 稀有度分数 |
|------|-----------|
| 纯宫（三宫相同） | 90 |
| 全吉（三宫皆吉） | 70 |
| 全凶（三宫皆凶） | 60 |
| 普通 | 30 |

## 使用示例

```rust
// 时间起课（农历六月初五辰时）
XiaoLiuRen::divine_by_time(
    origin,
    6,     // 农历六月
    5,     // 初五
    8,     // 8点（辰时）
    None,  // 问题 CID
    true,  // 公开
)?;

// 数字起课（1, 2, 3）
XiaoLiuRen::divine_by_number(
    origin,
    1, 2, 3,
    None,
    false,
)?;

// 随机起课
XiaoLiuRen::divine_random(
    origin,
    Some(question_cid),
    true,
)?;

// 请求 AI 解读
XiaoLiuRen::request_ai_interpretation(origin, pan_id)?;
```

## 十二时辰对照

| 时辰 | 时段 | 序号 |
|------|------|------|
| 子时 | 23:00-01:00 | 1 |
| 丑时 | 01:00-03:00 | 2 |
| 寅时 | 03:00-05:00 | 3 |
| 卯时 | 05:00-07:00 | 4 |
| 辰时 | 07:00-09:00 | 5 |
| 巳时 | 09:00-11:00 | 6 |
| 午时 | 11:00-13:00 | 7 |
| 未时 | 13:00-15:00 | 8 |
| 申时 | 15:00-17:00 | 9 |
| 酉时 | 17:00-19:00 | 10 |
| 戌时 | 19:00-21:00 | 11 |
| 亥时 | 21:00-23:00 | 12 |

## 三宫分析

- **月宫**：代表事情的起因或背景
- **日宫**：代表事情的经过或现状
- **时宫**：代表事情的结果或未来（最重要）

### 五行关系

| 关系 | 含义 | 吉凶影响 |
|------|------|---------|
| 相生 | 日宫生时宫 | +1 |
| 比和 | 日宫同时宫 | +1 |
| 相克 | 日宫克时宫 | -1 |
| 被克 | 时宫克日宫 | -1 |
| 泄气 | 时宫生日宫 | 0 |

## 测试

运行测试：

```bash
SKIP_WASM_BUILD=1 cargo test -p pallet-xiaoliuren
```

测试覆盖：
- 时间起课算法验证
- 数字起课算法验证
- 随机起课功能测试
- 手动指定功能测试
- AI 解读请求/提交测试
- 课盘可见性管理测试
- 每日限制功能测试
- 用户统计更新测试

## 版本历史

- **v0.1.0** (2025-12-01)：初始版本
  - 实现四种起课方式
  - 集成 AI 解读功能
  - 实现 DivinationProvider trait
  - 完整测试覆盖（35 个测试）

## 起课流程图

```
用户发起起课请求
        │
        ▼
┌───────────────────┐
│ 检查每日起课限制  │
│ (DailyLimitCheck) │
└────────┬──────────┘
         │ 通过
         ▼
┌───────────────────┐
│   选择起课方式    │
├───────────────────┤
│ ① 时间起课       │──→ 输入：农历月/日/时
│ ② 数字起课       │──→ 输入：三个数字(x,y,z)
│ ③ 随机起课       │──→ 使用链上随机数
│ ④ 手动指定       │──→ 输入：三宫索引(0-5)
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│   算法计算三宫    │
│ ┌───────────────┐ │
│ │月宫→日宫→时宫 │ │
│ └───────────────┘ │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ 创建 XiaoLiuRenPan│
│   存储到链上      │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  发送 PanCreated  │
│      事件         │
└────────┬──────────┘
         │
         ▼
   ┌─────┴─────┐
   │ 可选步骤  │
   └─────┬─────┘
         │
         ▼
┌───────────────────┐
│ 请求 AI 解读      │
│ (支付 5 DUST)     │
└───────────────────┘
```

## 前端集成示例

### Polkadot.js API 调用

```typescript
import { ApiPromise, WsProvider } from '@polkadot/api';

// 连接到节点
const wsProvider = new WsProvider('ws://localhost:9944');
const api = await ApiPromise.create({ provider: wsProvider });

// 时间起课
const timeDivination = api.tx.xiaoLiuRen.divineByTime(
  6,      // 农历六月
  15,     // 十五日
  14,     // 14点（未时）
  null,   // 问题CID（可选）
  true    // 是否公开
);
await timeDivination.signAndSend(account);

// 数字起课
const numberDivination = api.tx.xiaoLiuRen.divineByNumber(
  3,      // 第一个数字
  6,      // 第二个数字
  9,      // 第三个数字
  null,   // 问题CID
  false   // 私有
);
await numberDivination.signAndSend(account);

// 随机起课
const randomDivination = api.tx.xiaoLiuRen.divineRandom(
  'QmXxx...', // IPFS问题CID
  true        // 公开
);
await randomDivination.signAndSend(account);

// 查询用户的所有课盘
const userPans = await api.query.xiaoLiuRen.userPans(accountId);
console.log('用户课盘列表:', userPans.toHuman());

// 查询具体课盘详情
const pan = await api.query.xiaoLiuRen.pans(panId);
console.log('课盘详情:', pan.toHuman());

// 请求AI解读
const aiRequest = api.tx.xiaoLiuRen.requestAiInterpretation(panId);
await aiRequest.signAndSend(account);
```

### React Hooks 封装

```typescript
import { useQuery, useMutation } from '@tanstack/react-query';
import { useApi } from './useApi';

// 查询用户课盘列表
export function useUserPans(accountId: string) {
  const api = useApi();

  return useQuery({
    queryKey: ['xiaoliuren', 'userPans', accountId],
    queryFn: async () => {
      const pans = await api.query.xiaoLiuRen.userPans(accountId);
      return pans.toHuman();
    },
  });
}

// 时间起课
export function useDivineByTime() {
  const api = useApi();

  return useMutation({
    mutationFn: async ({
      lunarMonth,
      lunarDay,
      hour,
      questionCid,
      isPublic,
    }: {
      lunarMonth: number;
      lunarDay: number;
      hour: number;
      questionCid?: string;
      isPublic: boolean;
    }) => {
      const tx = api.tx.xiaoLiuRen.divineByTime(
        lunarMonth,
        lunarDay,
        hour,
        questionCid ?? null,
        isPublic
      );
      return tx;
    },
  });
}
```

## 谋事数对照表

每宫都有对应的谋事数，用于进一步细化占断：

| 六宫 | 谋事数 | 用法 |
|------|--------|------|
| 大安 | 1, 5, 7 | 事情在1、5、7日/月有结果 |
| 留连 | 2, 8, 10 | 事情在2、8、10日/月有结果 |
| 速喜 | 3, 6, 9 | 事情在3、6、9日/月有结果 |
| 赤口 | 4, 7, 10 | 事情在4、7、10日/月有结果 |
| 小吉 | 1, 5, 7 | 事情在1、5、7日/月有结果 |
| 空亡 | 3, 6, 9 | 事情在3、6、9日/月有结果 |

## 相关模块

- **pallet-divination-common**：通用占卜类型和 trait 定义
- **pallet-divination-nft**：占卜结果 NFT 铸造
- **pallet-divination-ai**：AI 解读服务
- **pallet-divination-market**：占卜服务市场

## 许可证

Unlicense
