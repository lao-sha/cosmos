# Nexus 综合开发路线图

> **基于三份设计文档，统一规划从 Sprint 7 到 Sprint 16 的完整开发计划。**
>
> - `NEXUS_LAYERED_STORAGE_DESIGN.md` — 全节点同步架构
> - `NEXUS_NODE_REWARD_DESIGN.md` — 订阅费+通胀混合奖励
> - `NEXUS_TELEGRAM_API_EVALUATION.md` — TG API 功能评估与扩展

---

## 一、当前状态

### 1.1 已完成 (Sprint 1-6)

| Sprint | 内容 | 状态 |
|---|---|---|
| 1 | pallet-bot-consensus + pallet-bot-registry | ✅ 38 tests |
| 2 | nexus-agent（签名、多播、Webhook） | ✅ 5 tests |
| 3 | nexus-node（验签、Gossip、状态机） | ✅ 29 tests |
| 4 | Leader 执行 + pallet-bot-group-mgmt + TG Executor | ✅ 11 tests |
| 5 | Chain Submitter + 安全（重放保护、限流） | ✅ |
| 6 | Rule Engine + 多平台适配器 | ✅ |
| **合计** | **3 pallet + 2 binary + 完整管线** | **85 tests** |

### 1.2 已实现的 TG API (10 个)

```
sendMessage, deleteMessage, banChatMember, restrictChatMember(mute/unmute),
pinChatMessage, unpinChatMessage, approveChatJoinRequest, declineChatJoinRequest
```

### 1.3 待建设的三大模块

```
┌─────────────────────────────────────────────────────────────────┐
│                      Nexus 待建设全景                            │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────────┐ │
│  │  全节点同步    │  │  节点奖励     │  │  TG API 功能扩展       │ │
│  │              │  │              │  │                         │ │
│  │  GroupConfig  │  │  订阅+通胀   │  │  Sprint 7-12           │ │
│  │  Gossip 同步  │  │  Era 奖励   │  │  34 个新 API 方法       │ │
│  │  Web DApp    │  │  claim 领取  │  │  6 个新 Rule            │ │
│  │              │  │              │  │                         │ │
│  │  9-10 天     │  │  8.5 天      │  │  15-21 天              │ │
│  └──────────────┘  └──────────────┘  └────────────────────────┘ │
│                                                                  │
│  原始合计: 32.5 ~ 39.5 天                                       │
│  合并优化后: 28 ~ 33 天                                          │
└─────────────────────────────────────────────────────────────────┘
```

---

## 二、依赖关系

```
                    ┌───────────────────┐
                    │ Phase 1: 基础重构  │
                    │ (Pallet精简+类型)  │
                    └────────┬──────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
    ┌─────────────┐  ┌─────────────┐  ┌──────────────┐
    │ Phase 2     │  │ Phase 3     │  │ Phase 4      │
    │ 全节点同步   │  │ 节点奖励    │  │ TG API 补全  │
    │ (Gossip+API)│  │ (订阅+Era)  │  │ (Sprint 7-8) │
    └──────┬──────┘  └──────┬──────┘  └──────┬───────┘
           │                │                │
           │         ┌──────┘                │
           ▼         ▼                       ▼
    ┌─────────────────────┐        ┌──────────────┐
    │ Phase 5             │        │ Phase 6      │
    │ Web DApp            │        │ TG API 增强  │
    │ (配置+订阅+状态)    │        │ (Sprint 9-10)│
    └─────────────────────┘        └──────┬───────┘
                                          │
                                   ┌──────┴───────┐
                                   │ Phase 7      │
                                   │ 高级功能      │
                                   │ (Sprint 11-12)│
                                   └──────────────┘
```

**关键依赖：**
- Phase 2 (全节点同步) **必须在** Phase 4 (TG API 补全) **之前** — RuleEngine 需要 GroupConfig
- Phase 3 (节点奖励) **可与** Phase 2 **并行** — 独立的 pallet 扩展
- Phase 4 **必须在** Phase 1 **之后** — ActionType 重构先行
- Phase 5 (Web DApp) **依赖** Phase 2 + Phase 3 — 需要配置 API + 订阅 API

---

## 三、Sprint 规划

### Phase 1: 基础重构 (Sprint 7) — 3 天

> 为后续所有工作打好地基。

| # | 任务 | 来源文档 | 预估 |
|---|---|---|---|
| 7.1 | 精简 `pallet-bot-group-mgmt`（移除 GroupRules） | 存储设计 §9.2 | 0.5 天 |
| 7.2 | 哈希碰撞修复 — 加盐哈希 | 存储设计 §7.1 | 0.5 天 |
| 7.3 | 定义 `GroupConfig` + `SignedGroupConfig` + `JoinApprovalPolicy` 类型 | 存储设计 §2 | 0.5 天 |
| 7.4 | ActionType 重构为嵌套枚举 | API评估 §5.1 | 1 天 |
| 7.5 | 查询型 vs 执行型分离（架构调整） | API评估 §5.2 | 0.5 天 |

**改动范围：**
```
pallets/nexus/bot-group-mgmt/src/lib.rs   — 移除 GroupRules 相关
pallets/nexus/bot-registry/src/lib.rs     — 加盐哈希
nexus-node/src/types.rs                    — GroupConfig + ActionType 重构
nexus-agent/src/types.rs                   — 同步类型
nexus-node/src/leader.rs                   — ActionType 适配
nexus-node/src/rule_engine.rs              — ActionType 适配
nexus-agent/src/executor.rs                — ActionType 适配
runtime/src/configs/mod.rs                 — pallet 配置更新
```

**验收标准：**
- [ ] `cargo test -p pallet-bot-group-mgmt` — GroupRules 相关测试移除/更新
- [ ] `cargo check -p nexus-runtime` ✅
- [ ] ActionType 新枚举编译通过

---

### Phase 2: 全节点同步 (Sprint 8) — 4 天

> 实现 GroupConfig 从 Agent 到所有 Node 的签名同步。

| # | 任务 | 来源文档 | 预估 |
|---|---|---|---|
| 8.1 | Gossip 新增 ConfigSync / ConfigPull / ConfigPullResponse | 存储设计 §3.2 | 0.5 天 |
| 8.2 | Agent 配置管理 API（POST/GET /v1/group-config） | 存储设计 §6.2 | 1 天 |
| 8.3 | Agent 认证 API（钱包 challenge + JWT） | 存储设计 §6.1 | 0.5 天 |
| 8.4 | Node ChainCache 扩展 + 配置接收 + 签名验证 | 存储设计 §5.1 | 1 天 |
| 8.5 | RuleEngine 对接 GroupConfig（替代旧 chain 查询） | 存储设计 §5.3 | 0.5 天 |
| 8.6 | 节点启动恢复（ConfigPull + 本地 JSON 持久化） | 存储设计 §3.3 | 0.5 天 |

**改动范围：**
```
nexus-node/src/gossip/engine.rs   — ConfigSync/ConfigPull 处理
nexus-node/src/types.rs           — GossipType 新增 3 个变体
nexus-node/src/chain_cache.rs     — group_configs HashMap
nexus-node/src/rule_engine.rs     — get_rules() 对接 GroupConfig
nexus-agent/src/group_config.rs   — 新文件: 配置管理 API
nexus-agent/src/auth.rs           — 新文件: JWT 认证
nexus-agent/src/webhook.rs        — 路由注册
```

**验收标准：**
- [ ] Agent 接收 GroupConfig → 签名 → Gossip 广播 → Node 验签 → 本地持久化
- [ ] Node 重启后从本地文件恢复 + ConfigPull 获取最新
- [ ] SeenPayload 携带 config_version
- [ ] `cargo test` 全部通过

---

### Phase 3: 节点奖励 (Sprint 9) — 5 天

> 与 Phase 2 **可并行**。实现订阅费 + 通胀混合奖励系统。

| # | 任务 | 来源文档 | 预估 |
|---|---|---|---|
| 9.1 | 定义 Subscription / EraRewardInfo / SubscriptionTier | 奖励设计 §6.1 | 0.5 天 |
| 9.2 | 新增 6 个 Storage 项 + Config 扩展 | 奖励设计 §6.2-6.4 | 0.5 天 |
| 9.3 | 实现 subscribe / deposit / cancel / change_tier | 奖励设计 §6.5 | 1 天 |
| 9.4 | 实现 compute_node_weight | 奖励设计 §5.1 | 0.5 天 |
| 9.5 | 实现 on_era_end Hook（扣费 + 铸币 + 分配） | 奖励设计 §6.6 | 1.5 天 |
| 9.6 | 实现 claim_rewards + Runtime 参数 | 奖励设计 §6.5 | 0.5 天 |
| 9.7 | 单元测试 (≥20 tests) | 奖励设计 §11.1 | 0.5 天 |

**改动范围：**
```
pallets/nexus/bot-consensus/src/lib.rs  — 主要改动 (新增 ~400 行)
runtime/src/configs/mod.rs              — 新增 7 个参数
```

**验收标准：**
- [ ] subscribe → deposit → on_era_end → claim_rewards 完整流程
- [ ] 通胀铸币正确分配
- [ ] 欠费 → PastDue → Suspended 状态转换
- [ ] MaxRewardShare 上限生效
- [ ] MinUptimeForReward 门槛生效
- [ ] `cargo test -p pallet-bot-consensus` ≥ 39 tests (原 19 + 新 20)
- [ ] `cargo check -p nexus-runtime` ✅

---

### Phase 4: TG API 群管补全 (Sprint 10) — 3 天

> 依赖 Phase 1 (ActionType 重构) + Phase 2 (GroupConfig)。

| # | 任务 | 来源文档 | 预估 |
|---|---|---|---|
| 10.1 | 新增 5 个 ActionType: Unban/GetMember/GetAdmins/DeleteBatch/SetPerms | API评估 §Sprint7 | 1 天 |
| 10.2 | 新增 3 个命令: /unban /perms /warn | API评估 §Sprint7 | 0.5 天 |
| 10.3 | 权限前置检查（Leader 执行前 getChatMember） | API评估 §Sprint7 | 0.5 天 |
| 10.4 | Inline 键盘交互: answerCallbackQuery + editMessageText | API评估 §Sprint8 | 1 天 |

**新增 TG API 方法 (10 个)：**
```
unbanChatMember, getChatMember, getChatAdministrators,
deleteMessages, setChatPermissions,
answerCallbackQuery, editMessageText, editMessageReplyMarkup,
sendPoll, stopPoll
```

**验收标准：**
- [ ] /ban → /unban 完整闭环
- [ ] Leader 执行前自动检查目标非管理员
- [ ] Inline 键盘回调正确处理
- [ ] 累计 TG API: 10 → 20

---

### Phase 5: Web DApp (Sprint 11) — 5 天

> 依赖 Phase 2 (配置 API) + Phase 3 (订阅 API)。合并两份文档的前端需求。

| # | 任务 | 来源文档 | 预估 |
|---|---|---|---|
| 11.1 | 项目初始化 (React + TailwindCSS + polkadot.js) | — | 0.5 天 |
| 11.2 | 钱包连接 + 签名认证 | 存储设计 §6.1 | 0.5 天 |
| 11.3 | 群配置管理页面（所有 GroupConfig 字段） | 存储设计 §8 | 1.5 天 |
| 11.4 | 订阅管理页面（订阅/充值/升级/取消） | 奖励设计 §11.1 | 1 天 |
| 11.5 | 节点状态 + 同步状态 + 奖励看板 | 两份文档 | 1 天 |
| 11.6 | 加盐哈希前端实现 (JS) | 存储设计 §7.1 | 0.5 天 |

**技术栈：**
```
nexus-web/
├── src/
│   ├── pages/
│   │   ├── Dashboard.tsx        ← Bot 总览 + 节点状态
│   │   ├── GroupConfig.tsx      ← 群规则配置表单
│   │   ├── Subscription.tsx     ← 订阅管理
│   │   └── NodeRewards.tsx      ← 奖励看板 (节点运营者)
│   ├── hooks/
│   │   ├── useWallet.ts         ← polkadot.js 钱包
│   │   ├── useAgent.ts          ← Agent API 调用
│   │   └── useChain.ts          ← 链上数据查询
│   └── utils/
│       └── hash.ts              ← 加盐哈希 (JS)
├── package.json
└── tailwind.config.js
```

**验收标准：**
- [ ] 钱包登录 → JWT 获取 → 配置提交 → Agent 签名 → Node 同步 完整流程
- [ ] 订阅/充值/取消 链上交易正确
- [ ] 同步状态实时显示

---

### Phase 6: TG API 增强 (Sprint 12-13) — 5 天

> 与 Phase 5 **可并行**。

**Sprint 12 — Bot 初始化 + 媒体 (2.5 天)：**

| # | 任务 | 来源文档 | 预估 |
|---|---|---|---|
| 12.1 | getMe + setMyCommands（Bot 启动自检 + 命令注册） | API评估 §Sprint9 | 0.5 天 |
| 12.2 | sendPhoto / sendDocument / sendMediaGroup | API评估 §Sprint9 | 1 天 |
| 12.3 | WelcomeRule（入群欢迎消息 + 媒体） | API评估 §Sprint9 | 0.5 天 |
| 12.4 | sendChatAction + getChat | API评估 §Sprint9 | 0.5 天 |

**Sprint 13 — 邀请链接 + 管理员 (2.5 天)：**

| # | 任务 | 来源文档 | 预估 |
|---|---|---|---|
| 13.1 | createChatInviteLink / exportChatInviteLink / revokeChatInviteLink | API评估 §Sprint10 | 1 天 |
| 13.2 | promoteChatMember + setChatAdministratorCustomTitle | API评估 §Sprint10 | 0.5 天 |
| 13.3 | setChatTitle / setChatDescription / unpinAllChatMessages | API评估 §Sprint10 | 0.5 天 |
| 13.4 | InviteRule（邀请链接管理命令） | API评估 §Sprint10 | 0.5 天 |

**新增 TG API 方法 (15 个)，累计: 20 → 35**

---

### Phase 7: 高级功能 (Sprint 14-16) — 8 天

> 可选阶段，根据产品需求决定优先级。

**Sprint 14 — 论坛话题管理 (2.5 天)：**

| # | 任务 | 来源文档 | 预估 |
|---|---|---|---|
| 14.1 | createForumTopic / editForumTopic / closeForumTopic / reopenForumTopic / deleteForumTopic | API评估 §Sprint11 | 1.5 天 |
| 14.2 | message_thread_id 贯穿所有消息发送方法 | API评估 §Sprint11 | 0.5 天 |
| 14.3 | ForumRule + GroupConfig 新增 forum_enabled | API评估 §Sprint11 | 0.5 天 |

**Sprint 15 — Webhook 事件扩展 (2 天)：**

| # | 任务 | 来源文档 | 预估 |
|---|---|---|---|
| 15.1 | 支持 edited_message / channel_post / poll_answer | API评估 §5.3 | 1 天 |
| 15.2 | 支持 my_chat_member / inline_query | API评估 §5.3 | 1 天 |

**Sprint 16 — 支付集成 (3.5 天)：**

| # | 任务 | 来源文档 | 预估 |
|---|---|---|---|
| 16.1 | sendInvoice / createInvoiceLink | API评估 §Sprint12 | 1 天 |
| 16.2 | answerShippingQuery / answerPreCheckoutQuery | API评估 §Sprint12 | 1 天 |
| 16.3 | 链上 Entity/Shop 集成（订单记录 + 佣金触发） | API评估 §Sprint12 | 1.5 天 |

**新增 TG API 方法 (9 个)，累计: 35 → 44**

---

## 四、时间线总览

```
Week 1          Week 2          Week 3          Week 4          Week 5          Week 6
│               │               │               │               │               │
├─ Phase 1 ─────┤               │               │               │               │
│ Sprint 7 (3d) │               │               │               │               │
│ 基础重构       │               │               │               │               │
│               ├─ Phase 2 ─────┤               │               │               │
│               │ Sprint 8 (4d) │               │               │               │
│               │ 全节点同步     │               │               │               │
│               │               │               │               │               │
│               ├─ Phase 3 ─────┼──┤            │               │               │
│               │ Sprint 9 (5d)    │            │               │               │
│               │ 节点奖励 (并行) │            │               │               │
│               │               │  │            │               │               │
│               │               │  ├─ Phase 4 ──┤               │               │
│               │               │  │ Sprint10(3d)│               │               │
│               │               │  │ TG群管补全  │               │               │
│               │               │  │            │               │               │
│               │               │  │            ├─ Phase 5 ─────┤               │
│               │               │  │            │ Sprint 11 (5d)│               │
│               │               │  │            │ Web DApp      │               │
│               │               │  │            │               │               │
│               │               │  │            ├─ Phase 6 ─────┤               │
│               │               │  │            │ Sprint12-13(5d)│              │
│               │               │  │            │ TG API增强(并行)│             │
│               │               │  │            │               │               │
│               │               │  │            │               ├─ Phase 7 ─────┤
│               │               │  │            │               │ Sprint14-16   │
│               │               │  │            │               │ 高级功能 (8d) │
│               │               │  │            │               │ (可选)        │
```

### 总工期预估

| 路径 | Sprint | 天数 | 说明 |
|---|---|---|---|
| **关键路径** | 7→8→10→11 | 3+4+3+5 = **15 天** | 最短必须时间 |
| **含奖励系统** | +9 (并行) | +2 天延迟 = **17 天** | Phase 3 与 Phase 2 并行，尾部溢出 2 天 |
| **含 TG 增强** | +12-13 (并行) | 0 天增量 | 与 Phase 5 并行 |
| **含高级功能** | +14-16 | +8 天 = **25 天** | 可选 |
| **全部完成** | Sprint 7-16 | **25 工作日 ≈ 5 周** | 1 人全栈 |

---

## 五、里程碑

| 里程碑 | Sprint | 交付物 | 核心指标 |
|---|---|---|---|
| **M1: 架构就绪** | 7 | Pallet 精简 + 类型定义 + ActionType 重构 | `cargo check -p nexus-runtime` ✅ |
| **M2: 配置同步上线** | 8 | Agent→Node 配置同步完整链路 | 群主改配置 → 全节点秒级同步 |
| **M3: 经济系统上线** | 9 | 订阅+通胀+领取 | ≥39 pallet tests |
| **M4: 群管功能完整** | 10 | 20 个 TG API + 权限检查 + Inline 键盘 | ban/unban 闭环 |
| **M5: 产品可用** | 11 | Web DApp (配置+订阅+看板) | 群主端到端可用 |
| **M6: 功能丰富** | 12-13 | 35 个 TG API + 媒体+邀请+管理员 | — |
| **M7: 全功能** | 14-16 | 44 个 TG API + 论坛+支付 | — |

---

## 六、测试计划

### 6.1 单元测试目标

| 模块 | 现有 | 新增 | 目标 |
|---|---|---|---|
| pallet-bot-consensus | 19 | +20 (奖励) | 39 |
| pallet-bot-group-mgmt | 11 | -3 (移除) +2 (更新) | 10 |
| nexus-agent | 7 | +10 (配置API+认证) | 17 |
| nexus-node | 29 | +15 (配置同步+新规则) | 44 |
| **合计** | **85** | **+44** | **~130** |

### 6.2 集成测试

```
E2E 场景（基于 costik-deploy/test-e2e.sh 扩展）:

1. 配置同步 E2E:
   群主 Web DApp → Agent API → Gossip ConfigSync → 3 Node 验签 → 持久化

2. 奖励 E2E:
   subscribe → deposit → 等待 1 Era → on_era_end → claim_rewards

3. 群管 E2E:
   TG 消息 → Agent → Node 共识 → Leader 决策 → Agent 执行 → 链上 ActionLog

4. 新功能 E2E:
   /ban → 确认键盘 → callback → 权限检查 → banChatMember → unbanChatMember
```

---

## 七、风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| ActionType 重构影响面大 | 中 | Sprint 7 延期 | 编译驱动，逐文件替换 |
| Gossip ConfigSync 与现有消息冲突 | 低 | Sprint 8 延期 | 新增独立消息类型，不改现有 |
| on_era_end 逻辑复杂，边界条件多 | 中 | Sprint 9 延期 | 充分单测 (≥20 tests) |
| Web DApp 与 Agent API 联调 | 中 | Sprint 11 延期 | Agent API 先写 mock/test |
| TG API 限流 (429 Too Many Requests) | 中 | 运行时问题 | Agent 已有 rate_limiter |

---

## 八、文档关联

| 文档 | 覆盖 Sprint | 主要内容 |
|---|---|---|
| `NEXUS_LAYERED_STORAGE_DESIGN.md` | 7 (部分), 8, 11 (部分) | 全节点同步架构、数据结构、API |
| `NEXUS_NODE_REWARD_DESIGN.md` | 9, 11 (部分) | 订阅+通胀混合奖励、链上实现 |
| `NEXUS_TELEGRAM_API_EVALUATION.md` | 7 (部分), 10, 12-16 | TG API 扩展、规则引擎扩展 |
| `NEXUS_DEVELOPMENT_ROADMAP.md` | **全部** | **本文档 — 统一路线图** |

---

*文档版本: v1.0 · 2026-02-08*
*规划范围: Sprint 7 — Sprint 16*
*预估总工期: 25 工作日（5 周）*
*优先交付: M1-M5（17 工作日内产品可用）*
