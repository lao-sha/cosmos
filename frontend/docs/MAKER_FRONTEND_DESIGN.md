# 做市商管理模块页面设计方案

## 概述

本文档描述星尘玄鉴 App 中"做市商管理"功能的页面设计方案，基于 `pallets/trading/maker` 模块实现。

## 业务背景

### 模块架构

```
pallets/trading/maker/
├── 做市商申请与审核
├── 押金管理（锁定/解锁/动态调整）
├── 提现管理（冷却期）
├── 溢价配置
├── 服务暂停/恢复
└── 押金扣除与申诉
```

### 核心业务规则

| 规则 | 说明 |
|------|------|
| 押金金额 | 1000 USD 等值 DUST |
| 押金补充阈值 | 950 USD |
| 押金补充目标 | 1050 USD |
| 提现冷却期 | 7 天 |
| 申诉时限 | 7 天 |
| 溢价范围 | -5% ~ +5% (±500 基点) |
| 价格检查间隔 | 每小时 |

### 价格数据来源

押金以 USD 计价，需要 DUST/USD 实时价格。价格数据由 `pallet-trading-pricing` 模块提供：

```
┌─────────────────────────────────────────────────────────┐
│                    Pricing Pallet                       │
├─────────────────────────────────────────────────────────┤
│  数据来源:                                              │
│  ├── OTC 订单成交价格 (滑动窗口 100万 DUST)            │
│  └── Bridge 兑换成交价格 (滑动窗口 100万 DUST)         │
│                                                         │
│  价格算法:                                              │
│  ├── 加权平均价格 = (OTC总USDT + Bridge总USDT)         │
│  │                  / (OTC总DUST + Bridge总DUST)        │
│  └── 简单平均价格 = (OTC均价 + Bridge均价) / 2         │
│                                                         │
│  冷启动保护:                                            │
│  ├── 阈值: 1亿 DUST                                    │
│  ├── 默认价格: 0.000001 USDT/DUST                      │
│  └── 达到阈值后永久退出冷启动                          │
│                                                         │
│  汇率数据 (Offchain Worker):                           │
│  ├── CNY/USD 汇率 (每24小时更新)                       │
│  └── API: exchangerate-api.com                         │
└─────────────────────────────────────────────────────────┘
```

| 价格类型 | 用途 | 精度 |
|----------|------|------|
| `get_dust_market_price_weighted()` | 押金估值、清算价格 | 10^6 (1 = 0.000001 USDT) |
| `get_memo_reference_price()` | 前端显示、价格偏离检查 | 10^6 |
| `get_cny_usdt_rate()` | CNY 金额显示 | 10^6 (7200000 = 7.2 CNY/USD) |

### 做市商状态流转

```
DepositLocked → PendingReview → Active (运营中)
      ↓              ↓
  Cancelled      Rejected
                     ↓
                 Expired
```

---

## 页面结构

### 1. 做市商入口页面

**路径**: `/maker` 或 `/profile/maker`

**功能**:
- 判断用户是否已是做市商
- 显示做市商状态概览
- 申请成为做市商入口

**UI 组件**:

```
┌─────────────────────────────────────┐
│  ← 做市商中心                       │
├─────────────────────────────────────┤
│                                     │
│  ┌─────────────────────────────┐   │
│  │  💼 成为做市商               │   │
│  │                              │   │
│  │  提供 OTC 交易服务           │   │
│  │  赚取交易溢价收益            │   │
│  │                              │   │
│  │  押金要求: 1000 USD 等值DUST │   │
│  │  提现冷却: 7 天              │   │
│  │                              │   │
│  │  [立即申请]                  │   │
│  └─────────────────────────────┘   │
│                                     │
│  ─────── 做市商权益 ───────        │
│                                     │
│  ┌─────────────────────────────┐   │
│  │ 💰 交易溢价                  │   │
│  │ 自定义买卖溢价，赚取差价     │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │ 🛡️ 押金保障                  │   │
│  │ 动态押金机制，价格波动自动   │   │
│  │ 调整，保障交易安全           │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │ ⭐ 信用体系                  │   │
│  │ 建立信用评分，获得更多订单   │   │
│  └─────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

### 2. 做市商申请流程

#### 2.1 锁定押金页面

**路径**: `/maker/apply/deposit`

**功能**:
- 显示押金要求
- 检查账户余额
- 锁定押金

**UI 组件**:

```
┌─────────────────────────────────────┐
│  ← 申请做市商 (1/3)                 │
├─────────────────────────────────────┤
│                                     │
│  第一步：锁定押金                   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  押金要求                    │   │
│  │                              │   │
│  │  1000 USD 等值 DUST          │   │
│  │  ≈ 10,000 DUST               │   │
│  │  (按当前价格 0.10 USD/DUST)  │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  您的余额                    │   │
│  │  15,000 DUST                 │   │
│  │  ✅ 余额充足                 │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  💡 押金说明                 │   │
│  │  • 押金将被锁定，不可交易    │   │
│  │  • 价格波动时可能需要补充    │   │
│  │  • 提现需要 7 天冷却期       │   │
│  │  • 违规行为将扣除押金        │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │       [锁定押金]             │   │
│  └─────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

#### 2.2 提交资料页面

**路径**: `/maker/apply/info`

**功能**:
- 填写实名信息
- 填写收款信息
- 设置溢价参数

**UI 组件**:

```
┌─────────────────────────────────────┐
│  ← 申请做市商 (2/3)                 │
├─────────────────────────────────────┤
│                                     │
│  第二步：提交资料                   │
│                                     │
│  ⏱️ 请在 58:30 内完成提交           │
│                                     │
│  实名信息                           │
│  ┌─────────────────────────────┐   │
│  │  真实姓名                    │   │
│  │  [    张三    ]              │   │
│  │                              │   │
│  │  身份证号                    │   │
│  │  [  110101199001011234  ]    │   │
│  │                              │   │
│  │  出生日期                    │   │
│  │  [    1990-01-01    ]        │   │
│  └─────────────────────────────┘   │
│                                     │
│  收款信息                           │
│  ┌─────────────────────────────┐   │
│  │  TRON 地址 (TRC20)           │   │
│  │  [  TYASr5UV6HEcXatwdF...  ] │   │
│  │                              │   │
│  │  微信号                      │   │
│  │  [    wechat_12345    ]      │   │
│  └─────────────────────────────┘   │
│                                     │
│  EPAY 配置 (可选)                   │
│  ┌─────────────────────────────┐   │
│  │  商户号                      │   │
│  │  [    12345678    ]          │   │
│  │                              │   │
│  │  密钥                        │   │
│  │  [    ********    ]          │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │       [提交资料]             │   │
│  └─────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

#### 2.3 等待审核页面

**路径**: `/maker/apply/pending`

**功能**:
- 显示审核状态
- 显示预计审核时间
- 取消申请入口

**UI 组件**:

```
┌─────────────────────────────────────┐
│  ← 申请做市商 (3/3)                 │
├─────────────────────────────────────┤
│                                     │
│           ⏳                        │
│       等待审核中                    │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  申请编号: #12345            │   │
│  │  提交时间: 2025-01-11 10:30  │   │
│  │  预计审核: 24 小时内         │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  审核流程                    │   │
│  │                              │   │
│  │  ✅ 押金已锁定               │   │
│  │  ✅ 资料已提交               │   │
│  │  ⏳ 平台审核中               │   │
│  │  ○ 审核通过                  │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  💡 审核说明                 │   │
│  │  • 审核通过后即可开始服务    │   │
│  │  • 审核驳回将退还押金        │   │
│  │  • 如需取消可点击下方按钮    │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │       [取消申请]             │   │
│  └─────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

### 3. 做市商控制台

#### 3.1 控制台首页

**路径**: `/maker/dashboard`

**功能**:
- 显示做市商状态概览
- 押金状态监控
- 快捷操作入口

**UI 组件**:

```
┌─────────────────────────────────────┐
│  ← 做市商控制台                     │
├─────────────────────────────────────┤
│                                     │
│  ┌─────────────────────────────┐   │
│  │  👤 做市商 #12345            │   │
│  │  张*三 | 在线 🟢             │   │
│  │  ⭐ 4.9 | 已服务 1,280 用户  │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  💰 押金状态                 │   │
│  │                              │   │
│  │  当前押金: 10,500 DUST       │   │
│  │  USD价值: $1,050             │   │
│  │  状态: ✅ 正常               │   │
│  │                              │   │
│  │  [补充押金]    [申请提现]    │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  📊 今日统计                 │   │
│  │                              │   │
│  │  订单数: 12                  │   │
│  │  交易额: 2,500 USDT          │   │
│  │  收益: 25 USDT               │   │
│  └─────────────────────────────┘   │
│                                     │
│  快捷操作                           │
│  ┌──────────┐  ┌──────────┐        │
│  │ 📝 订单  │  │ ⚙️ 设置  │        │
│  └──────────┘  └──────────┘        │
│  ┌──────────┐  ┌──────────┐        │
│  │ 📜 记录  │  │ ⚠️ 申诉  │        │
│  └──────────┘  └──────────┘        │
│                                     │
└─────────────────────────────────────┘
```

#### 3.2 押金管理页面

**路径**: `/maker/deposit`

**功能**:
- 查看押金详情
- 补充押金
- 申请提现
- 查看扣除记录

**UI 组件**:

```
┌─────────────────────────────────────┐
│  ← 押金管理                         │
├─────────────────────────────────────┤
│                                     │
│  ┌─────────────────────────────┐   │
│  │  押金概览                    │   │
│  │                              │   │
│  │  当前押金                    │   │
│  │  10,500 DUST                 │   │
│  │  ≈ $1,050 USD                │   │
│  │                              │   │
│  │  ┌─────────────────────┐    │   │
│  │  │████████████░░░░░░░░│    │   │
│  │  └─────────────────────┘    │   │
│  │  目标: $1,000 | 阈值: $950  │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  [补充押金]    [申请提现]    │   │
│  └─────────────────────────────┘   │
│                                     │
│  押金变动记录                       │
│  ┌─────────────────────────────┐   │
│  │ 📥 补充押金                  │   │
│  │ +500 DUST | 01-10 14:30     │   │
│  ├─────────────────────────────┤   │
│  │ 📤 扣除 - OTC超时            │   │
│  │ -50 DUST | 01-09 10:15      │   │
│  │ [查看详情] [申诉]            │   │
│  ├─────────────────────────────┤   │
│  │ 📥 初始押金                  │   │
│  │ +10,000 DUST | 01-01 09:00  │   │
│  └─────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

#### 3.3 补充押金页面

**路径**: `/maker/deposit/replenish`

**功能**:
- 显示当前押金状态
- 计算需补充金额
- 执行补充操作

**UI 组件**:

```
┌─────────────────────────────────────┐
│  ← 补充押金                         │
├─────────────────────────────────────┤
│                                     │
│  ┌─────────────────────────────┐   │
│  │  当前状态                    │   │
│  │                              │   │
│  │  当前押金: 9,200 DUST        │   │
│  │  USD价值: $920               │   │
│  │  状态: ⚠️ 低于阈值           │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  补充计算                    │   │
│  │                              │   │
│  │  目标价值: $1,050            │   │
│  │  当前价值: $920              │   │
│  │  需补充: $130                │   │
│  │                              │   │
│  │  ≈ 1,300 DUST                │   │
│  │  (按当前价格 0.10 USD/DUST)  │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  您的可用余额                │   │
│  │  5,000 DUST                  │   │
│  │  ✅ 余额充足                 │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │       [确认补充]             │   │
│  └─────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

#### 3.4 申请提现页面

**路径**: `/maker/deposit/withdraw`

**功能**:
- 输入提现金额
- 显示冷却期说明
- 提交提现申请

**UI 组件**:

```
┌─────────────────────────────────────┐
│  ← 申请提现                         │
├─────────────────────────────────────┤
│                                     │
│  ┌─────────────────────────────┐   │
│  │  可提现押金                  │   │
│  │  10,500 DUST                 │   │
│  │  ≈ $1,050 USD                │   │
│  └─────────────────────────────┘   │
│                                     │
│  提现金额                           │
│  ┌─────────────────────────────┐   │
│  │  [    5,000    ] DUST        │   │
│  │  ≈ $500 USD                  │   │
│  └─────────────────────────────┘   │
│                                     │
│  快捷金额                           │
│  [25%] [50%] [75%] [全部]          │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  ⚠️ 提现说明                 │   │
│  │                              │   │
│  │  • 提现需要 7 天冷却期       │   │
│  │  • 冷却期内可取消提现        │   │
│  │  • 提现后押金可能低于阈值    │   │
│  │  • 押金不足将暂停服务        │   │
│  └─────────────────────────────┘   │
│                                     │
│  提现后押金: 5,500 DUST ($550)     │
│  状态: ⚠️ 低于目标值               │
│                                     │
│  ┌─────────────────────────────┐   │
│  │       [提交申请]             │   │
│  └─────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

#### 3.5 提现进度页面

**路径**: `/maker/deposit/withdraw/status`

**功能**:
- 显示提现进度
- 冷却期倒计时
- 取消/执行提现

**UI 组件**:

```
┌─────────────────────────────────────┐
│  ← 提现进度                         │
├─────────────────────────────────────┤
│                                     │
│  ┌─────────────────────────────┐   │
│  │  提现申请                    │   │
│  │                              │   │
│  │  金额: 5,000 DUST            │   │
│  │  申请时间: 01-11 10:30       │   │
│  │  状态: ⏳ 冷却期中           │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  冷却期倒计时                │   │
│  │                              │   │
│  │      6天 12:30:45            │   │
│  │                              │   │
│  │  可执行时间: 01-18 10:30     │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  进度                        │   │
│  │                              │   │
│  │  ✅ 申请已提交               │   │
│  │  ⏳ 冷却期中 (7天)           │   │
│  │  ○ 可执行提现                │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │       [取消提现]             │   │
│  └─────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

冷却期结束后：

```
┌─────────────────────────────────────┐
│  ← 提现进度                         │
├─────────────────────────────────────┤
│                                     │
│  ┌─────────────────────────────┐   │
│  │  ✅ 冷却期已结束             │   │
│  │                              │   │
│  │  金额: 5,000 DUST            │   │
│  │  可立即执行提现              │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │       [执行提现]             │   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │       [取消提现]             │   │
│  └─────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

#### 3.6 做市商设置页面

**路径**: `/maker/settings`

**功能**:
- 设置溢价参数
- 设置最小交易金额
- 暂停/恢复服务
- 更新收款信息

**UI 组件**:

```
┌─────────────────────────────────────┐
│  ← 做市商设置                       │
├─────────────────────────────────────┤
│                                     │
│  服务状态                           │
│  ┌─────────────────────────────┐   │
│  │  当前状态: 🟢 服务中         │   │
│  │  [暂停服务]                  │   │
│  └─────────────────────────────┘   │
│                                     │
│  溢价设置                           │
│  ┌─────────────────────────────┐   │
│  │  买入溢价 (Bridge)           │   │
│  │  [  +1.0%  ] (-5% ~ +5%)     │   │
│  │                              │   │
│  │  卖出溢价 (OTC)              │   │
│  │  [  +0.5%  ] (-5% ~ +5%)     │   │
│  └─────────────────────────────┘   │
│                                     │
│  交易限额                           │
│  ┌─────────────────────────────┐   │
│  │  最小交易金额                │   │
│  │  [    20    ] USD            │   │
│  └─────────────────────────────┘   │
│                                     │
│  收款信息                           │
│  ┌─────────────────────────────┐   │
│  │  TRON 地址                   │   │
│  │  TYASr5UV6HEcXatwdF...       │   │
│  │  [修改]                      │   │
│  │                              │   │
│  │  微信号                      │   │
│  │  wechat_12345                │   │
│  │  [修改]                      │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │       [保存设置]             │   │
│  └─────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

### 4. 押金扣除与申诉

#### 4.1 扣除记录页面

**路径**: `/maker/penalties`

**功能**:
- 查看所有扣除记录
- 筛选扣除类型
- 申诉入口

**UI 组件**:

```
┌─────────────────────────────────────┐
│  ← 扣除记录                         │
├─────────────────────────────────────┤
│                                     │
│  筛选: [全部▼] [本月▼]              │
│                                     │
│  ┌─────────────────────────────┐   │
│  │ ⚠️ OTC订单超时               │   │
│  │ 订单 #12345                  │   │
│  │ 扣除: 500 DUST ($50)         │   │
│  │ 时间: 01-10 14:30            │   │
│  │ 状态: 未申诉                 │   │
│  │ [查看详情] [申诉]            │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │ ⚠️ Bridge兑换超时            │   │
│  │ 兑换 #67890                  │   │
│  │ 扣除: 300 DUST ($30)         │   │
│  │ 时间: 01-08 09:15            │   │
│  │ 状态: 申诉中                 │   │
│  │ [查看详情]                   │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │ ❌ 争议仲裁败诉              │   │
│  │ 案件 #11111                  │   │
│  │ 扣除: 700 DUST ($70)         │   │
│  │ 时间: 01-05 16:45            │   │
│  │ 状态: 申诉驳回               │   │
│  │ [查看详情]                   │   │
│  └─────────────────────────────┘   │
│                                     │
│  本月累计扣除: 1,500 DUST ($150)   │
│                                     │
└─────────────────────────────────────┘
```

#### 4.2 扣除详情页面

**路径**: `/maker/penalties/[penaltyId]`

**功能**:
- 显示扣除详情
- 显示相关订单/兑换信息
- 申诉操作

**UI 组件**:

```
┌─────────────────────────────────────┐
│  ← 扣除详情                         │
├─────────────────────────────────────┤
│                                     │
│  ┌─────────────────────────────┐   │
│  │  扣除编号: #P12345           │   │
│  │  类型: OTC订单超时           │   │
│  │  状态: 未申诉                │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  扣除金额                    │   │
│  │                              │   │
│  │  500 DUST                    │   │
│  │  ≈ $50 USD                   │   │
│  │                              │   │
│  │  扣除时间: 01-10 14:30       │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  关联订单                    │   │
│  │                              │   │
│  │  订单号: #12345              │   │
│  │  买家: 0x1234...5678         │   │
│  │  金额: 100 USDT              │   │
│  │  超时时长: 2 小时            │   │
│  │                              │   │
│  │  [查看订单详情]              │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  💡 扣除原因                 │   │
│  │                              │   │
│  │  买家已付款超过 2 小时，     │   │
│  │  做市商未及时释放 DUST，     │   │
│  │  触发超时扣除机制。          │   │
│  └─────────────────────────────┘   │
│                                     │
│  申诉截止: 01-17 14:30 (6天后)     │
│                                     │
│  ┌─────────────────────────────┐   │
│  │       [发起申诉]             │   │
│  └─────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

#### 4.3 申诉页面

**路径**: `/maker/penalties/[penaltyId]/appeal`

**功能**:
- 填写申诉理由
- 上传证据
- 提交申诉

**UI 组件**:

```
┌─────────────────────────────────────┐
│  ← 发起申诉                         │
├─────────────────────────────────────┤
│                                     │
│  ┌─────────────────────────────┐   │
│  │  扣除编号: #P12345           │   │
│  │  扣除金额: 500 DUST ($50)    │   │
│  │  扣除原因: OTC订单超时       │   │
│  └─────────────────────────────┘   │
│                                     │
│  申诉理由                           │
│  ┌─────────────────────────────┐   │
│  │  [                          ]│   │
│  │  [  请详细说明申诉理由...   ]│   │
│  │  [                          ]│   │
│  │  [                          ]│   │
│  └─────────────────────────────┘   │
│                                     │
│  上传证据                           │
│  ┌─────────────────────────────┐   │
│  │  📎 点击上传证据文件         │   │
│  │  支持图片、PDF (最大10MB)    │   │
│  └─────────────────────────────┘   │
│                                     │
│  已上传:                            │
│  ┌─────────────────────────────┐   │
│  │ 📄 转账截图.png    [删除]    │   │
│  │ 📄 聊天记录.pdf    [删除]    │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  ⚠️ 申诉须知                 │   │
│  │  • 申诉将由平台仲裁员审核    │   │
│  │  • 审核周期约 3-7 个工作日   │   │
│  │  • 申诉成功将退还扣除金额    │   │
│  │  • 恶意申诉将加重处罚        │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │       [提交申诉]             │   │
│  └─────────────────────────────┘   │
│                                     │
└─────────────────────────────────────┘
```

---

## 数据模型

### 做市商信息

```typescript
interface MakerApplication {
  id: number;
  owner: string;                    // 账户地址
  deposit: bigint;                  // 押金金额 (DUST)
  status: ApplicationStatus;        // 申请状态
  direction: Direction;             // 业务方向
  tronAddress: string;              // TRC20 收款地址
  publicCid: string;                // 公开资料 IPFS CID
  privateCid: string;               // 私密资料 IPFS CID
  buyPremiumBps: number;            // 买入溢价 (基点)
  sellPremiumBps: number;           // 卖出溢价 (基点)
  minAmount: bigint;                // 最小交易金额
  createdAt: number;                // 创建时间 (Unix秒)
  infoDeadline: number;             // 资料提交截止时间
  reviewDeadline: number;           // 审核截止时间
  servicePaused: boolean;           // 服务暂停状态
  usersServed: number;              // 已服务用户数
  maskedFullName: string;           // 脱敏姓名
  maskedIdCard: string;             // 脱敏身份证
  maskedBirthday: string;           // 脱敏生日
  wechatId: string;                 // 微信号
  epayNo?: string;                  // EPAY 商户号
  targetDepositUsd: number;         // 目标押金USD价值
  lastPriceCheck: number;           // 上次价格检查区块
  depositWarning: boolean;          // 押金不足警告
}

enum ApplicationStatus {
  DepositLocked = 'DepositLocked',
  PendingReview = 'PendingReview',
  Active = 'Active',
  Rejected = 'Rejected',
  Cancelled = 'Cancelled',
  Expired = 'Expired',
}

enum Direction {
  Buy = 0,      // 仅买入 (Bridge)
  Sell = 1,     // 仅卖出 (OTC)
  BuyAndSell = 2, // 双向
}
```

### 提现请求

```typescript
interface WithdrawalRequest {
  amount: bigint;           // 提现金额
  requestedAt: number;      // 申请时间 (Unix秒)
  executableAt: number;     // 可执行时间 (Unix秒)
  status: WithdrawalStatus; // 请求状态
}

enum WithdrawalStatus {
  Pending = 'Pending',
  Executed = 'Executed',
  Cancelled = 'Cancelled',
}
```

### 扣除记录

```typescript
interface PenaltyRecord {
  id: number;
  makerId: number;
  penaltyType: PenaltyType;
  deductedAmount: bigint;       // 扣除的DUST数量
  usdValue: number;             // 扣除时的USD价值
  beneficiary?: string;         // 受益人账户
  deductedAt: number;           // 扣除区块号
  appealed: boolean;            // 是否已申诉
  appealResult?: boolean;       // 申诉结果
}

type PenaltyType = 
  | { type: 'OtcTimeout'; orderId: number; timeoutHours: number }
  | { type: 'BridgeTimeout'; swapId: number; timeoutHours: number }
  | { type: 'ArbitrationLoss'; caseId: number; lossAmount: number }
  | { type: 'LowCreditScore'; currentScore: number; daysBelowThreshold: number }
  | { type: 'MaliciousBehavior'; behaviorType: number; evidenceCid: string };
```

---

## API 调用

### 1. 锁定押金

```typescript
await api.tx.maker.lockDeposit().signAndSend(account);
```

### 2. 提交资料

```typescript
await api.tx.maker.submitInfo(
  realName,           // 真实姓名
  idCardNumber,       // 身份证号
  birthday,           // 生日 (YYYY-MM-DD)
  tronAddress,        // TRON 地址
  wechatId,           // 微信号
  epayNo,             // EPAY 商户号 (可选)
  epayKey,            // EPAY 密钥 (可选)
).signAndSend(account);
```

### 3. 取消申请

```typescript
await api.tx.maker.cancelMaker().signAndSend(account);
```

### 4. 补充押金

```typescript
await api.tx.maker.replenishDeposit().signAndSend(account);
```

### 5. 申请提现

```typescript
await api.tx.maker.requestWithdrawal(amount).signAndSend(account);
```

### 6. 执行提现

```typescript
await api.tx.maker.executeWithdrawal().signAndSend(account);
```

### 7. 取消提现

```typescript
await api.tx.maker.cancelWithdrawal().signAndSend(account);
```

### 8. 申诉扣除

```typescript
await api.tx.maker.appealPenalty(
  penaltyId,
  evidenceCid,        // 证据 IPFS CID
).signAndSend(account);
```

### 9. 查询做市商信息

```typescript
// 通过账户查询做市商ID
const makerId = await api.query.maker.accountToMaker(account);

// 查询做市商详情
const makerApp = await api.query.maker.makerApplications(makerId);

// 查询提现请求
const withdrawal = await api.query.maker.withdrawalRequests(makerId);

// 查询扣除记录
const penalties = await api.query.maker.makerPenalties(makerId);
const penaltyRecord = await api.query.maker.penaltyRecords(penaltyId);
```

### 10. 查询价格数据

```typescript
// 获取 DUST 市场统计信息
const marketStats = await api.query.pricing.getMarketStats();
// 返回: { otcPrice, bridgePrice, weightedPrice, simpleAvgPrice, ... }

// 获取加权平均价格 (用于押金估值)
const weightedPrice = await api.call.pricingApi.getDustMarketPriceWeighted();

// 获取 CNY/USDT 汇率
const cnyRate = await api.query.pricing.cnyUsdtRate();

// 检查是否处于冷启动阶段
const coldStartExited = await api.query.pricing.coldStartExited();
```

---

## 状态管理

### 价格数据 Hook

```typescript
// hooks/useDustPrice.ts
interface DustPriceState {
  weightedPrice: number;      // 加权平均价格 (精度 10^6)
  simpleAvgPrice: number;     // 简单平均价格
  otcPrice: number;           // OTC 均价
  bridgePrice: number;        // Bridge 均价
  cnyRate: number;            // CNY/USD 汇率
  isColdStart: boolean;       // 是否处于冷启动
  isLoading: boolean;
  error: Error | null;
}

function useDustPrice(): DustPriceState {
  const [state, setState] = useState<DustPriceState>({
    weightedPrice: 0,
    simpleAvgPrice: 0,
    otcPrice: 0,
    bridgePrice: 0,
    cnyRate: 7_200_000, // 默认 7.2
    isColdStart: true,
    isLoading: true,
    error: null,
  });

  useEffect(() => {
    const fetchPrice = async () => {
      try {
        const [marketStats, coldStartExited, cnyRate] = await Promise.all([
          api.query.pricing.getMarketStats(),
          api.query.pricing.coldStartExited(),
          api.query.pricing.cnyUsdtRate(),
        ]);

        setState({
          weightedPrice: marketStats.weightedPrice.toNumber(),
          simpleAvgPrice: marketStats.simpleAvgPrice.toNumber(),
          otcPrice: marketStats.otcPrice.toNumber(),
          bridgePrice: marketStats.bridgePrice.toNumber(),
          cnyRate: cnyRate.cnyRate.toNumber() || 7_200_000,
          isColdStart: !coldStartExited.toHuman(),
          isLoading: false,
          error: null,
        });
      } catch (error) {
        setState(prev => ({ ...prev, isLoading: false, error }));
      }
    };

    fetchPrice();
    // 每分钟刷新价格
    const interval = setInterval(fetchPrice, 60_000);
    return () => clearInterval(interval);
  }, []);

  return state;
}

// 计算押金 USD 价值
function calculateDepositUsdValue(
  depositDust: bigint,
  weightedPrice: number
): number {
  // USD = DUST数量 × 价格 / 10^12 / 10^6
  return Number(depositDust * BigInt(weightedPrice) / BigInt(1e18));
}

// 计算所需 DUST 数量
function calculateRequiredDust(
  targetUsd: number,
  weightedPrice: number
): bigint {
  // DUST = USD × 10^12 × 10^6 / 价格
  return BigInt(Math.ceil(targetUsd * 1e18 / weightedPrice));
}
```

### Zustand Store

```typescript
// stores/makerStore.ts
interface MakerState {
  // 做市商信息
  makerId: number | null;
  makerApp: MakerApplication | null;
  isLoading: boolean;
  
  // 提现状态
  withdrawalRequest: WithdrawalRequest | null;
  
  // 扣除记录
  penalties: PenaltyRecord[];
  
  // 押金状态
  depositUsdValue: number;
  needsReplenishment: boolean;
  
  // Actions
  fetchMakerInfo: () => Promise<void>;
  lockDeposit: () => Promise<void>;
  submitInfo: (info: MakerInfoInput) => Promise<void>;
  cancelApplication: () => Promise<void>;
  replenishDeposit: () => Promise<void>;
  requestWithdrawal: (amount: bigint) => Promise<void>;
  executeWithdrawal: () => Promise<void>;
  cancelWithdrawal: () => Promise<void>;
  appealPenalty: (penaltyId: number, evidenceCid: string) => Promise<void>;
  fetchPenalties: () => Promise<void>;
}
```

---

## 文件结构

```
frontend/
├── app/
│   └── maker/
│       ├── index.tsx               # 做市商入口
│       ├── apply/
│       │   ├── deposit.tsx         # 锁定押金
│       │   ├── info.tsx            # 提交资料
│       │   └── pending.tsx         # 等待审核
│       ├── dashboard.tsx           # 控制台首页
│       ├── deposit/
│       │   ├── index.tsx           # 押金管理
│       │   ├── replenish.tsx       # 补充押金
│       │   └── withdraw/
│       │       ├── index.tsx       # 申请提现
│       │       └── status.tsx      # 提现进度
│       ├── settings.tsx            # 做市商设置
│       └── penalties/
│           ├── index.tsx           # 扣除记录列表
│           ├── [penaltyId]/
│           │   ├── index.tsx       # 扣除详情
│           │   └── appeal.tsx      # 申诉页面
├── src/
│   ├── features/
│   │   └── maker/
│   │       ├── components/
│   │       │   ├── DepositStatus.tsx     # 押金状态组件
│   │       │   ├── MakerStatusCard.tsx   # 做市商状态卡片
│   │       │   ├── PenaltyCard.tsx       # 扣除记录卡片
│   │       │   ├── WithdrawalProgress.tsx # 提现进度组件
│   │       │   ├── PremiumSlider.tsx     # 溢价滑块
│   │       │   ├── DepositValueDisplay.tsx # 押金USD价值显示
│   │       │   └── index.ts
│   │       ├── hooks/
│   │       │   ├── useMaker.ts           # 做市商数据
│   │       │   ├── useDeposit.ts         # 押金管理
│   │       │   ├── useWithdrawal.ts      # 提现管理
│   │       │   ├── usePenalties.ts       # 扣除记录
│   │       │   └── useDustPrice.ts       # DUST价格数据
│   │       └── index.ts
│   └── stores/
│       └── makerStore.ts
```

---

## 主题样式

```typescript
// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#C9B07A';
const THEME_BG = '#F5F5F7';

// 状态色
const SUCCESS_COLOR = '#4CD964';
const WARNING_COLOR = '#FF9500';
const ERROR_COLOR = '#FF3B30';
const INFO_COLOR = '#007AFF';

// 做市商状态色
const MAKER_STATUS_COLORS = {
  DepositLocked: '#FF9500',   // 橙色 - 押金已锁定
  PendingReview: '#007AFF',   // 蓝色 - 待审核
  Active: '#4CD964',          // 绿色 - 已激活
  Rejected: '#FF3B30',        // 红色 - 已驳回
  Cancelled: '#8E8E93',       // 灰色 - 已取消
  Expired: '#8E8E93',         // 灰色 - 已过期
};

// 押金状态色
const DEPOSIT_STATUS_COLORS = {
  normal: '#4CD964',          // 绿色 - 正常
  warning: '#FF9500',         // 橙色 - 低于阈值
  critical: '#FF3B30',        // 红色 - 严重不足
};

// 扣除类型色
const PENALTY_TYPE_COLORS = {
  OtcTimeout: '#FF9500',
  BridgeTimeout: '#FF9500',
  ArbitrationLoss: '#FF3B30',
  LowCreditScore: '#FF9500',
  MaliciousBehavior: '#FF3B30',
};
```

---

## 错误处理

### 常见错误码

| 错误码 | 说明 | 用户提示 | 处理方式 |
|--------|------|----------|----------|
| `Maker.MakerAlreadyExists` | 已申请过做市商 | 您已是做市商 | 跳转控制台 |
| `Maker.MakerNotFound` | 做市商不存在 | 做市商信息不存在 | 刷新页面 |
| `Maker.InvalidMakerStatus` | 状态不正确 | 当前状态不允许此操作 | 刷新状态 |
| `Maker.InsufficientDeposit` | 押金不足 | 押金余额不足 | 补充押金 |
| `Maker.InsufficientBalance` | 余额不足 | 账户余额不足 | 充值 |
| `Maker.InvalidTronAddress` | TRON地址无效 | 请输入有效的TRON地址 | 重新输入 |
| `Maker.WithdrawalRequestNotFound` | 提现请求不存在 | 未找到提现申请 | 刷新页面 |
| `Maker.WithdrawalCooldownNotMet` | 冷却期未满 | 提现冷却期未结束 | 等待冷却期 |
| `Maker.PenaltyRecordNotFound` | 扣除记录不存在 | 未找到扣除记录 | 刷新页面 |
| `Maker.AlreadyAppealed` | 已申诉过 | 该记录已申诉 | 查看申诉状态 |
| `Maker.AppealDeadlineExpired` | 申诉期限已过 | 申诉期限已过 | 无法申诉 |

### 押金不足警告

```typescript
// hooks/useDepositWarning.ts
function useDepositWarning(makerApp: MakerApplication) {
  useEffect(() => {
    if (makerApp.depositWarning) {
      showNotification(
        '押金不足警告',
        '您的押金价值已低于阈值，请及时补充以避免服务暂停'
      );
    }
  }, [makerApp.depositWarning]);
}
```

**押金不足警告 UI**:

```
┌─────────────────────────────────────┐
│  ⚠️ 押金不足警告                    │
│                                     │
│  您的押金价值已低于 $950 阈值       │
│  当前价值: $920                     │
│                                     │
│  请及时补充押金，否则可能：         │
│  • 无法接收新订单                   │
│  • 服务被自动暂停                   │
│                                     │
│  [立即补充]        [稍后提醒]       │
└─────────────────────────────────────┘
```

---

## IPFS 集成方案

### 服务选型

项目已集成多端点 IPFS 服务，支持故障转移：

| 服务商 | 用途 | 特点 |
|--------|------|------|
| Pinata | 主要上传端点 | 稳定、免费额度充足 |
| Web3.Storage | 备用上传端点 | 去中心化存储 |
| 多网关下载 | 内容获取 | Cloudflare、IPFS.io 等 |

### 环境配置

```bash
# .env
EXPO_PUBLIC_IPFS_API=https://api.pinata.cloud/pinning/pinFileToIPFS
EXPO_PUBLIC_IPFS_GATEWAY=https://gateway.pinata.cloud/ipfs/
EXPO_PUBLIC_PINATA_API_KEY=your_api_key
EXPO_PUBLIC_PINATA_API_SECRET=your_api_secret
EXPO_PUBLIC_WEB3_STORAGE_TOKEN=your_token
```

### 加密方案

使用 X25519 ECDH + AES-256-GCM 端到端加密：

```typescript
// 1. 生成加密密钥对
import { generateX25519KeyPair } from '@/services/crypto.service';
const { privateKey, publicKey } = generateX25519KeyPair();

// 2. 加密敏感资料
import { encryptMessage } from '@/services/crypto.service';
const makerInfo = JSON.stringify({
  realName: '张三',
  idCard: '110101199001011234',
  birthday: '1990-01-01',
  // ...
});
const encrypted = await encryptMessage(makerInfo, sharedKey);

// 3. 上传到 IPFS
import { uploadToIpfs } from '@/services/ipfs.service';
const cid = await uploadToIpfs(encrypted);
```

### CID 用途说明

| CID 字段 | 内容 | 加密 | 访问权限 |
|----------|------|------|----------|
| `publicCid` | 公开资料（头像、简介） | 否 | 所有用户 |
| `privateCid` | 实名资料（身份证、银行卡） | 是 | 仅平台审核 |
| `epayKeyCid` | EPAY 密钥 | 是 | 仅做市商本人 |
| `evidenceCid` | 申诉证据（截图、聊天记录） | 否 | 仲裁员 |

### 上传流程

```typescript
// hooks/useMakerIpfs.ts
async function uploadMakerInfo(info: MakerInfoInput): Promise<{
  publicCid: string;
  privateCid: string;
}> {
  // 1. 公开资料（不加密）
  const publicData = JSON.stringify({
    avatar: info.avatar,
    bio: info.bio,
  });
  const publicCid = await uploadToIpfs(
    new TextEncoder().encode(publicData)
  );

  // 2. 私密资料（加密）
  const privateData = JSON.stringify({
    realName: info.realName,
    idCard: info.idCard,
    birthday: info.birthday,
    bankInfo: info.bankInfo,
  });
  
  // 使用平台公钥加密（仅平台可解密）
  const platformPublicKey = await getPlatformPublicKey();
  const sharedKey = await deriveSharedKey(userPrivateKey, platformPublicKey);
  const encrypted = await encryptMessage(privateData, sharedKey);
  const privateCid = await uploadToIpfs(encrypted);

  return { publicCid, privateCid };
}
```

### 下载与解密

```typescript
// 下载并解密私密资料（仅平台审核使用）
async function downloadMakerPrivateInfo(
  privateCid: string,
  makerPublicKey: Uint8Array
): Promise<MakerPrivateInfo> {
  // 1. 从 IPFS 下载
  const encrypted = await downloadFromIpfs(privateCid);
  
  // 2. 使用平台私钥解密
  const sharedKey = await deriveSharedKey(platformPrivateKey, makerPublicKey);
  const decrypted = await decryptMessage(encrypted, sharedKey);
  
  return JSON.parse(decrypted);
}
```

---

## 安全考虑

1. **实名信息脱敏**: 链上只存储脱敏后的姓名、身份证号
2. **敏感信息加密**: 完整资料使用 AES-256-GCM 加密后上传 IPFS
3. **密钥交换**: 使用 X25519 ECDH 进行安全密钥协商
4. **押金保障**: 动态押金机制确保交易安全
5. **冷却期保护**: 7 天提现冷却期防止恶意提现
6. **申诉机制**: 7 天申诉期保障做市商权益
7. **多端点容灾**: IPFS 服务支持多端点故障转移

---

## 后续优化

1. **自动补充**: 押金低于阈值时自动补充
2. **收益统计**: 详细的收益报表和分析
3. **等级体系**: 做市商等级和特权
4. **批量操作**: 批量处理订单功能
5. **API 对接**: 第三方交易平台 API 集成
