# Cosmos 项目 AI 自主运行 — 可行性分析报告

> 文档版本：v2.0  
> 日期：2026-02-07  
> 范围：评估 Cosmos 项目引入 AI 自主运行能力的技术可行性、架构方案和实施路线  
> v2.0 变更：聚焦 AI 客服为核心产品方向，统一架构描述与实施路线图，移除不可行方案（TEE/Leader Node 调用 LLM）

---

## 目录

1. [项目愿景](#1-项目愿景)
2. [现有基础设施盘点](#2-现有基础设施盘点)
3. [行业参考：区块链 × AI 项目分析](#3-行业参考区块链--ai-项目分析)
4. [三大核心目标分析](#4-三大核心目标分析)
   - 4.1 AI 自动运维（AI Ops Agent）
   - 4.2 AI 自动获客（AI Sales Agent）
   - 4.3 AI 自动部署节点（AI Infra Agent）
5. [整体架构设计](#5-整体架构设计)
6. [安全与治理模型](#6-安全与治理模型)
7. [实施路线图](#7-实施路线图)
8. [风险评估与缓解](#8-风险评估与缓解)
9. [结论与建议](#9-结论与建议)
10. [Nexus 官方代理 — 产品需求](#10-nexus-官方代理--产品需求)
    - 10.1~10.8 核心功能、技术架构、商业模型、实施计划
    - 10.9 自建代理广告激励方案
11. [AI 客服 — 深度实现方案](#11-ai-客服--深度实现方案)
    - 11.1 目标与定位
    - 11.2 消息处理全流程
    - 11.3 核心模块设计（规则引擎 / AI Handler / 知识库 / 安全过滤 / 对话缓存）
    - 11.4 知识库三层架构
    - 11.5 LLM 调用策略（三级模型 + 成本估算）
    - 11.6 链上存储与配置
    - 11.7 群主操作命令
    - 11.8 效果追踪与自动学习
    - 11.9 与 Commission 系统集成
    - 11.10 实施里程碑
    - 11.11 双重角色分析：店铺产品客服 × 群客服
    - 11.12 调用区块链 AI 项目大模型 — 可行性分析
    - 11.13 Nexus Agent 群主自保管 Key — 可行性分析

---

## 1. 项目愿景

**核心目标**：让 Cosmos 区块链网络中的 Entity（实体/商家）能够通过 AI 实现：

1. **自动运营**（AI Ops）— AI 持续监控和维护节点健康，自动诊断修复
2. **自动获客**（AI Sales）— AI 在社群平台代表 Entity 智能推荐服务、回答咨询
3. **自动扩缩**（AI Infra）— AI 根据网络负载自动部署/缩减节点，管理预算

**差异化定位**：与其他 AI+区块链项目不同，Cosmos 不是"用区块链做 AI 训练/推理市场"，而是"让 AI 成为区块链商业实体的自主运营者"。AI 的每个决策通过去中心化共识验证、上链存证，解决"谁来监督 AI"的信任问题。

---

## 2. 现有基础设施盘点

### 2.1 链上 Pallet 矩阵

| 类别 | Pallet | 说明 | AI 可利用点 |
|------|--------|------|------------|
| **实体管理** | `pallet-entity-registry` | Entity 生命周期、运营资金健康 | AI 自动注册/管理 Entity |
| | `pallet-entity-shop` | Shop 生命周期、积分系统 | AI 管理多 Shop |
| | `pallet-entity-common` | EntityType/GovernanceMode/Status | AI 读取治理规则 |
| | `pallet-entity-member` | 会员等级（Inherit/Independent/Hybrid） | AI 个性化推荐 |
| | `pallet-entity-service` | 服务目录管理 | AI 了解"卖什么" |
| **佣金系统** | `pallet-commission-core` | 调度引擎、提现/复购（4 种模式）、购物余额 | AI 获客自动产生佣金 |
| | `pallet-commission-referral` | 5 种推荐链模式 | AI 利用推荐关系获客 |
| | `pallet-commission-level-diff` | 级差佣金 | AI 策略优化 |
| | `pallet-commission-single-line` | 单线佣金 | AI 策略优化 |
| **Bot 共识** | `pallet-bot-consensus` (idx=140) | 节点注册/质押/Equivocation/信誉 | AI 决策多节点验证 |
| | `pallet-bot-registry` (idx=141) | Bot 注册 + 5 平台身份绑定 | AI Agent 链上身份 |
| | `pallet-bot-group-mgmt` (idx=142) | 群规则引擎 + ActionLog | AI 行为规则约束 |
| **交易** | `pallet-trading` | OTC/Swap/Maker/Pricing/Credit/Escrow | AI 自动交易/定价 |
| **存储** | `pallet-storage-service` | IPFS Pin + 生命周期 | AI 知识库存储 |
| **仲裁** | `pallet-arbitration` | 投诉/证据/裁决 | AI 异常行为仲裁 |
| **隐私** | `pallet-privacy-*` | ZK 凭证 + 隐私计算 | 保护 AI 数据 |

### 2.2 链下组件

#### node-operator（LLM 驱动的节点运维 Agent）

**位置**: `node-operator/`（独立 binary，排除在 workspace 外）

**模块清单**:

| 模块 | 文件 | 能力 |
|------|------|------|
| **Agent 核心** | `agent.rs` | LLM Tool-calling 循环（单轮 + 多轮对话） |
| **LLM 客户端** | `llm_client.rs` | 支持 Claude / OpenAI，通过 `LLM_PROVIDER` 切换 |
| **工具注册表** | `tools.rs` | 本地工具 6 个 + 远程工具 7 个 + 云工具 9 个 |
| **安全审批** | `approval.rs` | PendingOperation + RiskLevel 三级审批 |
| **SSH 管理** | `ssh.rs` | 远程服务器连接与命令执行 |
| **远程工具** | `remote_tools.rs` | deploy_node / start_node / stop_node / ssh_execute / ansible |
| **云服务** | `cloud_provider.rs` | Vultr / DigitalOcean / LNVPS 统一接口 |
| **云工具** | `cloud_tools.rs` | create_server / destroy_server / auto_deploy / estimate_cost |
| **Lightning** | `lightning.rs` | LNbits 客户端：付款/收款/余额查询/自动付款 |

**关键设计**:
- `ToolRegistry` — 可插拔工具系统，LLM 只能调用注册过的工具
- `BudgetManager` — 月度预算上限控制（默认 $100/月）
- `LightningPaymentManager` — 自动付款开关 + 金额上限（默认 100,000 sats ≈ $30）
- `ApprovalManager` — 高危操作需人工审批

#### nexus-agent（去中心化 Bot 本地代理）

**位置**: `nexus-agent/`（独立 binary）

**能力**:
- Telegram Webhook 接收 → Ed25519 签名 → 确定性节点选择 → 并行多播
- TelegramExecutor：10 种 TG API 方法（sendMessage / deleteMessage / banChatMember 等）
- 滑动窗口限速器

#### nexus-node（去中心化验证节点）

**位置**: `nexus-node/`（独立 binary）

**能力**:
- 4 层验证：Ed25519 签名 → Bot 活跃检查 → 公钥匹配 → 目标节点检查
- Gossip 协议：8 种消息类型，M/K 共识（M = ceil(K×2/3)）
- 规则引擎：可插拔 Rule trait 链（JoinRequest → Command → LinkFilter → Default）
- 多平台适配器：`PlatformAdapter` trait（已实现 Telegram / Discord / Slack）
- 链上提交器：3 优先队列（equivocation > confirmation > action_log），批次提交
- Leader 选举：Round-robin，FailoverManager
- subxt 链客户端：订阅 finalized blocks，每 100 块刷新缓存

### 2.3 成熟度评估

```
┌─────────────────┬────────────┬────────────────────────────┐
│ 组件             │ 成熟度     │ AI 自主运行就绪度           │
├─────────────────┼────────────┼────────────────────────────┤
│ Entity/Shop     │ ⭐⭐⭐⭐    │ 高 — 完整商业闭环           │
│ Commission      │ ⭐⭐⭐⭐    │ 高 — 4 种提现模式 + 奖励    │
│ Nexus Bot      │ ⭐⭐⭐⭐    │ 高 — 85 测试全过            │
│ node-operator   │ ⭐⭐⭐     │ 中高 — 需加自动循环         │
│ Trading         │ ⭐⭐⭐⭐    │ 中 — 需 AI 定价策略         │
│ Storage/Privacy │ ⭐⭐⭐     │ 低 — AI 知识库需扩展        │
└─────────────────┴────────────┴────────────────────────────┘
```

---

## 3. 行业参考：区块链 × AI 项目分析

### 3.1 去中心化 AI 推理/训练市场

| 项目 | 方向 | 优点 | 缺点 | 与 Cosmos 的关系 |
|------|------|------|------|-----------------|
| **Bittensor (TAO)** | 去中心化 AI 模型训练激励 | 开创性的子网竞争机制 | 质量评估困难、GPU 门槛高 | 无直接关系，不同赛道 |
| **Ritual** | 链上 AI 推理验证 | 可验证推理 + EVM 集成 | 早期阶段、推理成本高 | 可参考验证推理模式 |
| **Gensyn** | 去中心化 GPU 训练 | 大规模分布式训练 | 训练一致性、网络带宽 | 无直接关系 |
| **Together AI** | AI 推理服务（部分去中心化） | 高性能、易用 | 半中心化 | 可作为推理后端 |

### 3.2 AI Agent 框架

| 项目 | 方向 | 优点 | 缺点 | 与 Cosmos 的关系 |
|------|------|------|------|-----------------|
| **Autonolas (OLAS)** | 链上 AI Agent 注册 + 协调 | Agent 经济模型完整 | 复杂度高、落地场景少 | **最接近**：Agent 注册/协调 |
| **Fetch.ai** | 自主经济 Agent | 多 Agent 通信框架 | 生态小、实际应用少 | 可参考 Agent 通信 |
| **SingularityNET** | AI 服务市场 | 品牌知名度 | 实际使用率低 | 不同方向 |
| **AI16Z/Eliza** | 社交 AI Agent 框架 | 开源、社区活跃 | 纯 Agent 框架，无链上治理 | **可集成**为 LLM 调用层 |

### 3.3 AI + 运维/基础设施

| 项目 | 方向 | 优点 | 缺点 | 与 Cosmos 的关系 |
|------|------|------|------|-----------------|
| **Akash Network** | 去中心化云计算 | 真实可用的 GPU/CPU 市场 | 供应不稳定 | 可作为云服务商之一 |
| **io.net** | 分布式 GPU 网络 | 大规模 GPU 聚合 | 早期、质量参差不齐 | 无直接关系 |
| **Spheron** | 去中心化计算 | Kubernetes 友好 | 小众 | 可作为部署后端 |

### 3.4 Cosmos 的独特定位

```
                    其他项目                     Cosmos 项目
                ┌──────────────┐            ┌──────────────────┐
                │  AI 训练/推理 │            │  AI 自主运营商家   │
                │  去中心化市场 │            │  去中心化验证决策   │
                │              │            │                  │
核心价值:       │  计算资源交易 │            │  商业实体自治运营   │
                │  模型质量竞争 │            │  决策审计可追溯     │
                └──────────────┘            └──────────────────┘

关键差异:
- 不做 AI 训练/推理市场（留给 Bittensor/Ritual）
- 做 AI 驱动的商业实体自主运营
- 每个 AI 决策通过 Nexus 多节点共识验证
- AI 的"钱包"由链上治理规则约束
```

---

## 4. 三大核心目标分析

### 4.1 AI 自动运维（AI Ops Agent）

**可行性：⭐⭐⭐⭐⭐ — 最高，已有 80% 基础**

#### 4.1.1 现有能力

`node-operator` 已具备完整的 LLM → Tool 调用管道：

```
用户/定时器 触发
    ↓
NodeOperatorAgent.chat()
    ↓
LLM 理解意图 → 选择 Tool
    ↓
ToolRegistry.execute()
    ↓
┌─────────────────────────────────────────────────┐
│ 本地工具:                                        │
│   get_node_status → RPC system_health/syncState │
│   diagnose_node   → 连接性/对等/资源检查         │
│   get_node_logs   → journalctl 日志             │
│   check_system_resources → CPU/内存/磁盘        │
│   generate_chain_spec → 配置建议                │
│                                                 │
│ 远程工具:                                        │
│   ssh_execute → 远程命令（需审批）               │
│   deploy_node → 部署到远程服务器（需审批）       │
│   start_node / stop_node → 启停服务             │
│   generate_ansible_playbook → 批量部署          │
│                                                 │
│ 云工具:                                          │
│   create_cloud_server → Vultr/DO/LNVPS          │
│   auto_deploy_node → 端到端自动部署             │
│   estimate_cost → 费用预估                      │
│   get_lightning_balance → BTC 余额查询          │
│   pay_lightning_invoice → BTC 自动付款          │
└─────────────────────────────────────────────────┘
    ↓
LLM 总结结果 → 返回用户
```

#### 4.1.2 缺失项与实施方案

| 缺失项 | 难度 | 实施方案 |
|--------|------|---------|
| **自动触发循环** | 低 | 在 `node-operator/src/main.rs` 加入 `tokio::spawn` 定时任务，每 N 分钟调用 `agent.chat("检查所有节点状态")` |
| **链上事件监听** | 中 | 复用 `nexus-node` 的 `chain_client.rs` subxt 框架，监听 `NodeOffline` / `EquivocationDetected` 事件触发运维 |
| **自愈决策链** | 中 | 定义决策树：检测异常 → 尝试重启 → 失败则迁移 → 记录到链上 |
| **多 Agent 协调** | 高 | 多个 Ops Agent 通过 Nexus 共识协调：同一异常只由一个 Agent 处理 |
| **决策审计上链** | 低 | 复用 `pallet-bot-group-mgmt` 的 `log_action` extrinsic 记录运维决策 |

#### 4.1.3 架构变化

```
Before (当前):
  运维人员 → CLI → node-operator → 执行

After (目标):
  定时循环 ─────────────┐
  链上事件监听 ──────────┤
  异常阈值告警 ──────────┤
                        ↓
                 OpsAgent (扩展 NodeOperatorAgent)
                        ↓
                 LLM 分析 + Tool 调用
                        ↓
              ┌─────────┴─────────┐
              │ 低危操作           │ 高危操作
              │ (查看日志/状态)    │ (重启/迁移/删除)
              │ → 自动执行         │ → Nexus 共识验证
              │                   │ → 链上审批 (多签)
              │                   │ → 执行 + 上链存证
              └─────────┬─────────┘
                        ↓
                 执行结果上链
```

---

### 4.2 AI 自动获客（AI Sales Agent）

**可行性：⭐⭐⭐ — 中等，需要新建核心模块，但框架已有**

#### 4.2.1 商业闭环分析

```
Entity Owner 创建 Entity + Shop + Service（链上）
        ↓
Entity Owner 配置 AI Sales Agent（链上注册 Bot）
        ↓
AI Agent 被部署到社群平台（TG/Discord/Slack）
        ↓
AI Agent 行为循环:
  1. 监听群组消息 → 识别潜在客户需求
  2. 根据 Entity 知识库生成回复/推荐
  3. 引导用户到 Entity Shop 完成交易
  4. 客户注册为 Entity Member → 推荐链激活
        ↓
交易完成 → pallet-commission 自动结算佣金
        ↓
Entity 资金池增长 → AI Agent 运营费用自动支付
```

#### 4.2.2 可复用的现有组件

| 现有组件 | 在 AI 获客中的角色 |
|---------|-------------------|
| `pallet-bot-registry` (5 平台绑定) | AI Sales Agent 的链上身份注册 |
| `pallet-bot-consensus` (多节点验证) | AI 回复内容经多节点验证后才发送 |
| `pallet-bot-group-mgmt` (规则引擎) | 约束 AI 营销行为（频率/内容/合规） |
| `nexus-node/rule_engine.rs` (Rule trait 链) | 扩展为 AI 获客规则：需求识别 → 推荐匹配 → 回复生成 |
| `nexus-node/PlatformAdapter` trait | 已实现 TG/Discord/Slack 适配器，直接复用 |
| `pallet-entity-service` | AI 读取 Entity 的服务目录 |
| `pallet-commission-referral` | AI 带来的客户自动进入推荐链 |
| `pallet-entity-member` | 新客户自动注册为 Member |

#### 4.2.3 需新建的模块

**a) AI 获客 Agent（off-chain 服务）**

```rust
// 新建: nexus-agent/src/sales_agent.rs (概念设计)

pub struct SalesAgent {
    /// LLM 客户端 (复用 node-operator 的 llm_client)
    llm: Box<dyn LlmClient>,
    /// Entity 知识库 (从 IPFS 加载)
    knowledge_base: EntityKnowledgeBase,
    /// 平台适配器
    platform: Box<dyn PlatformAdapter>,
    /// 行为约束规则
    behavior_rules: SalesBehaviorRules,
}

impl SalesAgent {
    /// 处理群消息 → 决定是否回复
    async fn process_message(&self, msg: &PlatformMessage) -> Option<SalesAction> {
        // 1. 规则引擎检查：是否在允许时间/频率内
        // 2. LLM 分析：消息是否包含潜在需求
        // 3. LLM 生成：个性化回复/推荐
        // 4. Nexus 共识验证：多节点确认回复内容合规
        // 5. 执行发送
    }
}
```

**b) Entity 知识库（链上索引 + IPFS 存储）**

```
pallet-entity-knowledge (新 pallet，可选)
    ├── 服务目录（从 pallet-entity-service 同步）
    ├── FAQ 文档（IPFS CID，由 Entity Owner 上传）
    ├── 推荐话术模板
    └── 禁止词/敏感词列表
```

**c) 行为合规约束（扩展 pallet-bot-group-mgmt）**

```
新增链上规则:
    ├── max_replies_per_hour: u16       // 每小时最多回复数
    ├── min_interval_seconds: u32       // 最短回复间隔
    ├── forbidden_keywords: Vec<String> // 禁止词
    ├── require_consensus: bool         // 是否需要多节点验证
    └── auto_disclosure: bool           // 是否自动声明为 AI
```

#### 4.2.4 获客效果归因

```
AI 回复消息 → 附带 referral_code (链接到 Entity Shop)
    ↓
用户点击链接 → 进入 Shop → 注册 Member
    ↓
pallet-entity-member::auto_register()
    ├── bind_referrer(ai_agent_account)  // AI 作为推荐人
    └── initial_level = 0
    ↓
用户下单 → pallet-commission-core::process_order()
    ├── ReferralPlugin: AI Agent 获得推荐佣金
    └── Entity Shop 获得销售收入
    ↓
AI Agent 佣金 → 自动支付运营成本 (LLM API / 服务器)
```

#### 4.2.5 关键风险

| 风险 | 等级 | 缓解措施 |
|------|------|---------|
| 平台封号（TG/Discord 反自动营销） | 🔴 高 | 严格限速 + 拟人化回复 + 声明 AI 身份 |
| 用户反感垃圾营销 | 🔴 高 | 只在需求匹配时回复 + 用户可屏蔽 |
| LLM 幻觉导致虚假宣传 | 🟡 中 | Nexus 多节点验证 + 知识库 RAG |
| 法律合规（GDPR/广告法） | 🟡 中 | 自动声明 AI 身份 + 不收集用户数据 |
| 冷启动（无客户历史数据） | 🟡 中 | 初期由 Entity Owner 提供种子知识 |

---

### 4.3 AI 自动部署节点（AI Infra Agent）

**可行性：⭐⭐⭐⭐ — 高，已有 70% 基础**

#### 4.3.1 现有部署管道

```
node-operator 已实现:

CloudClient (统一接口)
    ├── VultrClient    → API: 区域/套餐/OS/创建/删除
    ├── DigitalOceanClient → API: 同上
    └── LnvpsClient    → API: 同上 + Lightning Invoice 支付
        ↓
CreateServerRequest { name, region, plan, os, ssh_keys }
        ↓
Server { id, ip_address, status, monthly_cost }
        ↓
SSH 连接 → 部署脚本 → Substrate 节点上线
        ↓
健康检查 → 注册到链上
```

#### 4.3.2 自动扩缩容设计

```
                    ┌───────────────────────┐
                    │  链上指标监控           │
                    │  - ActiveNodeList.len()│
                    │  - 平均出块时间        │
                    │  - 交易队列深度        │
                    │  - 节点在线率          │
                    └──────────┬────────────┘
                               │
                    ┌──────────▼────────────┐
                    │  扩缩容决策引擎        │
                    │                       │
                    │  if nodes < min_nodes: │
                    │    → 扩容              │
                    │  if nodes > max_nodes  │
                    │    && load < threshold:│
                    │    → 缩容              │
                    │  if node_offline:      │
                    │    → 替换              │
                    └──────────┬────────────┘
                               │
              ┌────────────────┼────────────────┐
              ↓                ↓                ↓
         扩容流程          替换流程          缩容流程
    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
    │ 1. 选区域     │  │ 1. 诊断故障  │  │ 1. 选择节点   │
    │ 2. 选套餐     │  │ 2. 尝试修复  │  │ 2. 迁移状态   │
    │ 3. 预算检查   │  │ 3. 修复失败  │  │ 3. 退出共识   │
    │ 4. 创建服务器 │  │    → 新建替换│  │ 4. 销毁服务器 │
    │ 5. 部署节点   │  │ 4. 迁移数据  │  │ 5. 释放质押   │
    │ 6. 注册共识   │  │ 5. 更新缓存  │  │ 6. 更新预算   │
    └──────────────┘  └──────────────┘  └──────────────┘
```

#### 4.3.3 自动支付流程（LNVPS + Lightning）

这是 Cosmos 的独特优势 — **AI 可以用 Bitcoin Lightning 自动付费购买服务器，无需 KYC**：

```
链上 Entity 资金池 (COSMOS token)
        ↓ (需要 Bridge，未实现)
Lightning Network 钱包 (LNbits)
        ↓
LightningPaymentManager.auto_pay_if_enabled()
    ├── 检查: auto_pay_enabled == true
    ├── 检查: invoice.amount_sats <= max_auto_pay_sats (默认 100k sats)
    ├── 检查: balance >= invoice.amount_sats
    └── 执行: LnbitsClient.pay_invoice(bolt11)
        ↓
LNVPS 确认支付 → 创建 VM → 返回 IP
        ↓
SSH 部署 Cosmos 节点 → 注册到链上
```

#### 4.3.4 缺失项

| 缺失项 | 难度 | 说明 |
|--------|------|------|
| 扩缩容策略 pallet | 中 | `pallet-infra-policy`: 链上参数控制 min/max nodes、预算上限 |
| 自动注册 extrinsic | 低 | 新节点调用 `pallet-bot-consensus::register_node` + 质押 |
| COSMOS ↔ Lightning Bridge | 高 | 链上资金兑换为 Lightning sats（可先用手动充值） |
| 节点健康评分 | 中 | 综合可用率、延迟、共识参与度给节点打分 |
| 跨区域调度 | 中 | 根据地理分布和延迟优化节点位置 |

---

## 5. 整体架构设计

### 5.1 产品架构总览

**核心产品方向**（按优先级排序）：

| 优先级 | 产品 | 基于 | 状态 |
|--------|------|------|------|
| **P0** | AI 客服 Bot | nexus-agent + nexus-node + LLM | 详见 Section 11 |
| **P1** | Nexus 官方代理 @NexusBot | nexus-official-agent | 详见 Section 10 |
| **P2** | AI 自动运维 | node-operator + 自动循环 | 详见 4.1 |
| **P3** | AI 自动部署节点 | node-operator + cloud_provider | 详见 4.3 |
| **P4** | AI 主动获客 | AI 客服扩展 | 详见 4.2（远期） |

### 5.2 系统架构图

```
┌──────────────────────────────────────────────────────────────────┐
│                    Cosmos Blockchain (Substrate)                   │
│                                                                   │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐           │
│  │ Entity 系统  │  │ Bot 共识系统  │  │ Trading 系统   │           │
│  │ registry     │  │ consensus    │  │ otc/swap/maker │           │
│  │ shop         │  │ registry     │  │ pricing/credit │           │
│  │ member       │  │ group-mgmt   │  │ escrow         │           │
│  │ service      │  │              │  │                │           │
│  │ commission   │  │              │  │                │           │
│  └──────┬──────┘  └──────┬───────┘  └───────┬───────┘           │
│         │                │                   │                    │
│         └────────────────┼───────────────────┘                    │
│                          │  subxt 订阅 + RPC 查询                 │
└──────────────────────────┼────────────────────────────────────────┘
                           │
            ┌──────────────┼──────────────┐
            │              │              │
     ┌──────▼──────┐ ┌────▼────┐ ┌───────▼────────┐
     │ nexus-node │ │ nexus- │ │ node-operator  │
     │  (×K 节点)  │ │ agent   │ │ (运维/部署 AI) │
     │             │ │ (群主   │ │                │
     │ 规则引擎    │ │  本地   │ │ LLM Tool-call  │
     │ Gossip 共识 │ │  代理)  │ │ SSH/Cloud/Pay  │
     │ Leader 执行 │ │         │ │ 预算管理       │
     │ AI Handler  │ │ Webhook │ │                │
     │ 链上提交    │ │ 签名    │ └────────────────┘
     └──────┬──────┘ │ 多播    │
            │        │ TG 执行 │
            │        └────┬────┘
            │             │
            └──────┬──────┘
                   │
     ┌─────────────▼─────────────┐
     │      AI 客服数据流         │
     │                           │
     │  TG 用户提问               │
     │    → Agent 签名多播        │
     │    → Node 共识验证         │
     │    → Leader: 规则匹配      │
     │    → Leader: LLM 推理      │
     │      (知识库 RAG + 对话)   │
     │    → 回传 Agent 执行       │
     │    → TG 发送回复           │
     │    → 链上存证              │
     └───────────────────────────┘
```

### 5.3 关键组件关系

```
群主 (Entity Owner)
  │
  ├── 自建模式: 运行 nexus-agent (本地 VPS)
  │     └── 自保管 BOT_TOKEN + Ed25519 Key (详见 11.13)
  │
  └── 官方模式: 拉 @NexusBot 入群 (详见 Section 10)
        └── 零部署，Nexus 集群托管

nexus-agent ←──── TG Webhook ────→ Telegram Server
  │
  │ Ed25519 签名 + 确定性多播
  ↓
nexus-node (×K)
  │ 4 层验证 → Gossip 共识 → M/K 确认
  ↓
Leader Node
  │ 规则引擎匹配 → AI Handler (LLM 调用)
  │ 知识库: 链上商品(L1) + IPFS文档(L2) + 历史QA(L3)
  ↓
nexus-agent ←── POST /v1/execute
  │ TelegramExecutor.sendMessage()
  ↓
用户看到 AI 客服回复
```

### 5.4 数据流

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  外部输入    │     │  AI 决策     │     │  执行 + 存证 │
├─────────────┤     ├─────────────┤     ├─────────────┤
│ 链上事件     │────→│ LLM 分析     │────→│ Nexus 验证  │
│ 平台消息     │     │ Tool 调用    │     │ 链上审批     │
│ 定时任务     │     │ 规则匹配     │     │ 执行动作     │
│ 阈值告警     │     │ 预算校验     │     │ 结果上链     │
└─────────────┘     └─────────────┘     └─────────────┘
```

---

## 6. 安全与治理模型

### 6.1 三层安全架构

```
Layer 1: AI 自身约束
    ├── Tool 白名单（只能调用注册的工具）
    ├── 预算硬上限（BudgetManager.monthly_limit）
    ├── 单次操作金额上限（max_auto_pay_sats）
    └── 行为规则引擎（频率/内容/时间限制）

Layer 2: Nexus 共识验证
    ├── 高危决策需 M/K 节点确认（M = ceil(K×2/3)）
    ├── Equivocation 检测（AI 不一致行为被惩罚）
    ├── Leader 轮换（避免单点操控）
    └── 结果上链不可篡改

Layer 3: 链上治理
    ├── Entity Owner 可随时暂停 AI Agent
    ├── Governance 设置全局参数（GlobalMinRepurchaseRate 等）
    ├── 仲裁系统处理 AI 引发的纠纷
    └── 紧急暂停机制（多签触发）
```

### 6.2 操作风险分级

| 风险级别 | 操作示例 | 执行方式 |
|---------|---------|---------|
| 🟢 **低** | 查看节点状态、读取日志、查询余额 | AI 直接执行 |
| 🟡 **中** | 重启节点、发送群消息、创建 Invoice | Nexus M/K 共识后执行 |
| 🔴 **高** | 创建/删除服务器、转账、修改链上配置 | 共识 + Entity Owner 审批 |
| ⚫ **禁止** | 修改治理参数、操作用户钱包、绕过审计 | AI 无权限 |

### 6.3 AI 不可触碰的红线

1. **不能直接操作用户资金** — AI 只能操作 Entity 运营资金池
2. **不能修改链上治理参数** — AI 只有建议权
3. **不能绕过 Nexus 共识** — 所有副作用操作必须经过验证
4. **不能隐藏决策** — 所有决策必须上链存证
5. **不能超出预算** — BudgetManager 是硬性检查，不是 LLM 判断

---

## 7. 实施路线图

> 以下路线图已按 v2.0 产品优先级重新排列。AI 客服为最高优先级产品方向，详细里程碑见 Section 11.10。

### Phase 1: AI 客服 — 最小可用（P0，3 周）

**最高商业价值，可直接面向 Entity Owner 交付**

- [ ] 抽取 `cosmos-llm-client` 共享 crate（复用 `node-operator/src/llm_client.rs`）
- [ ] 实现 `AiCustomerServiceRule`（nexus-node 规则引擎新规则）
- [ ] 实现 `AiHandler`（Leader 节点 LLM 推理 + RAG）
- [ ] 实现 `KnowledgeManager` Layer 1（链上商品目录）
- [ ] 实现 `ConversationCache`（内存对话缓存）
- [ ] 实现 `SafetyFilter`（基础安全过滤）
- [ ] 群角色系统（`GroupCsRole`）— 从第一天支持按群设角色
- [ ] `/ai_on`、`/ai_off`、`/ai_role` 命令

**验证标准**: AI 能根据链上商品目录回答价格/库存问题，准确率 > 80%

### Phase 2: AI 客服 — 知识库 + 学习（P0，4 周）

- [ ] `KnowledgeManager` Layer 2（IPFS 文档索引 + 向量搜索）
- [ ] `KnowledgeManager` Layer 3（历史 QA 积累 + 管理员反馈信号）
- [ ] IPFS 文档标签系统 + 角色知识域过滤
- [ ] `pallet-bot-group-mgmt` 扩展 AI 配置字段
- [ ] 三级模型路由策略（DeepSeek / Haiku / Sonnet）
- [ ] 集成 Hyperbolic 作为去中心化 LLM 备选（详见 11.12）
- [ ] 完整命令集：`/ai_knowledge`、`/ai_test`、`/ai_correct`、`/ai_status`
- [ ] 商品链接附带 referral code → Commission 系统归因

**验证标准**: AI 回答在一周内通过学习明显提升准确率（> 90%），LLM 成本可追踪

### Phase 3: Nexus 官方代理（P1，4 周）

**零门槛增长引擎 — 详见 Section 10**

- [ ] 新建 `nexus-official-agent/` 项目
- [ ] `GroupRegistry`：Bot 入群/退群自动注册
- [ ] `MemberCollector`：会员信息采集
- [ ] `StatsEngine`：活跃度评分、增长趋势
- [ ] 完整命令集：`/stats`、`/members`、`/growth`、`/inactive`、`/export`
- [ ] 反垃圾过滤 + 新成员欢迎 + 入群验证
- [ ] `/bind` 命令：TG user_id ↔ 链上 AccountId 绑定

**验证标准**: Bot 入群后 1 分钟内开始采集，单实例稳定服务 500+ 群

### Phase 4: AI 自动运维 + 节点部署（P2，4 周）

**改动最小、风险最低 — 详见 4.1 和 4.3**

- [ ] `node-operator` 加入定时监控循环（5 分钟/30 分钟/24 小时）
- [ ] 链上事件驱动（订阅 `NodeOffline` / `EquivocationDetected`）
- [ ] 自愈决策链（检测 → 诊断 → 重启 → 迁移）
- [ ] 自动扩缩容（检测节点不足/过多 → 创建/销毁服务器）
- [ ] 预算守护 + 决策审计上链

**验证标准**: 节点宕机后 5 分钟内自动检测、10 分钟内尝试修复

### Phase 5: 广告网络 + AI 主动获客（P3，远期）

- [ ] 官方代理广告投放系统（详见 10.5）
- [ ] 自建代理广告激励（详见 10.9）
- [ ] AI 主动获客：需求识别 → 智能推荐 → 推荐链激活（详见 4.2）
- [ ] 链上资金 → Lightning Bridge（可选）

---

## 8. 风险评估与缓解

### 8.1 技术风险

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| LLM 幻觉导致错误操作 | 中 | 高 | Tool 白名单 + Nexus 共识 + 金额上限 |
| LLM API 不可用 | 中 | 中 | 多 Provider 切换（Claude ↔ OpenAI） |
| 链上交易费暴涨 | 低 | 中 | 批次提交（ChainSubmitter 已支持） |
| subxt 断连 | 中 | 低 | 自动重连 + 静态缓存回退（已实现） |

### 8.2 商业风险

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| 平台封号 | 中 | 高 | 限速 + 拟人 + 声明 AI |
| 用户反感 | 中 | 中 | 只响应匹配需求 + 可屏蔽 |
| 竞争对手抄袭 | 中 | 低 | Nexus 共识验证是技术壁垒 |
| 监管变化 | 低 | 高 | 保守合规 + 支持区域关闭 |

### 8.3 安全风险

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| API Key 泄露 | 低 | 极高 | 环境变量隔离 + 定期轮换 + 权限最小化 |
| AI 被提示注入攻击 | 中 | 高 | 输入清洗 + 系统提示硬编码 |
| 恶意 Entity 滥用 AI | 中 | 中 | 链上信誉系统 + 仲裁 |
| 共识节点合谋 | 低 | 高 | M/K 阈值 + 质押罚没 |

---

## 9. 结论与建议

### 9.1 核心结论

**Cosmos 项目实现 AI 自主运行是可行且合理的**，理由如下：

1. **技术基座完备** — 已有 LLM Agent 框架（node-operator）、去中心化共识（Nexus 85 测试）、云部署 API（3 云服务商 + Lightning 支付）、完整商业实体系统（Entity/Shop/Member/Commission）
2. **差异化明确** — 不做 AI 训练/推理市场，做 AI 驱动的商业实体自主运营 + 决策链上可审计
3. **AI 客服是最佳切入点** — 复用现有 nexus-agent/nexus-node 管道，3 周可交付 MVP，直接面向 Entity Owner 产生商业价值

### 9.2 独特竞争优势

- **去中心化 AI 决策审计** — AI 的每个回复都经多节点共识验证、不可篡改地上链
- **群主自保管 Key** — 类似 Bitcoin 钱包的自主权模型，单点被攻破不影响其他群主（详见 11.13）
- **完整商业闭环** — AI 客服回答 → 商品链接（referral）→ 下单 → 佣金分配，全链路链上化
- **角色化知识隔离** — 同一 Bot 在不同群扮演不同角色，知识库按标签精准过滤（详见 11.11）
- **去中心化 LLM 备选** — 可集成 Hyperbolic/Bittensor 降低成本 + 增强 Web3 叙事（详见 11.12）

### 9.3 建议优先级

```
推荐路线（v2.0 更新）:

Phase 1: AI 客服 MVP（3 周）     → 最高商业价值，直接面向 Entity Owner
Phase 2: AI 客服完善（4 周）     → 知识库 + 自动学习 + 去中心化 LLM
Phase 3: 官方代理 @NexusBot（4 周）→ 零门槛增长引擎
Phase 4: AI 运维 + 节点部署（4 周） → 基础设施自动化
Phase 5: 广告网络 + 获客（远期）   → 商业模式闭环
```

### 9.4 下一步行动

1. 抽取 `cosmos-llm-client` 共享 crate，统一 LLM 调用接口
2. 实现 `AiCustomerServiceRule` + `AiHandler`，完成 AI 客服最小管道
3. 确定首批测试 Entity（知识库种子数据）
4. 部署测试环境：1 Agent + 3 Nodes + DeepSeek LLM

---

## 10. Nexus 官方代理 — 产品需求

### 10.1 背景与动机

当前 `nexus-agent` 是**群主自建代理**模式：群主需要自行部署 Docker、配置 `BOT_TOKEN`、管理服务器。这对非技术型群主来说门槛过高，限制了项目的推广和用户增长。

**核心思路**：Nexus 官方运营一个托管的 Telegram Bot（`@NexusBot`），群主**只需将官方 Bot 拉入群聊**，即可**免费**获得群聊会员信息采集与管理服务。群主零配置、零部署、零成本。

### 10.2 产品定位

```
当前模式（nexus-agent — 自建代理）:
  群主 → 部署服务器 → 配置 BOT_TOKEN → 启动 Docker → 注册链上
  适用: 技术型群主 / Entity 运营方 / 高级用户

新增模式（Nexus Official Bot — 官方代理）:
  群主 → 拉 @NexusBot 入群 → 完成
  适用: 所有群主 / 零门槛 / 免费增长引擎
```

两种模式**共存互补**，不互斥：
- **官方代理**：免费基础服务，快速获客，数据由 Nexus 集群托管
- **自建代理**：高级功能，数据完全自主，适合有隐私需求的 Entity

### 10.3 核心功能需求

#### F1: 一键入群激活

| 项目 | 说明 |
|------|------|
| **触发方式** | 群主/管理员将 `@NexusBot` 添加到 Telegram 群组 |
| **激活流程** | Bot 检测 `chat_member_updated` 事件 → 自动绑定群组 → 发送欢迎消息 |
| **权限要求** | Bot 需获得群管理员权限（读消息 + 获取成员列表） |
| **退出机制** | 群主踢出 Bot 即停止服务，自动清理该群数据 |

#### F2: 群聊会员信息采集

Bot 入群后自动采集和持续更新以下信息：

| 数据类别 | 字段 | 来源 | 更新频率 |
|---------|------|------|---------|
| **群基本信息** | 群 ID、群名、群类型（group/supergroup）、成员总数 | TG API `getChat` | 入群时 + 每日 |
| **成员列表** | user_id、username、first_name、is_bot、status | TG API `getChatAdministrators` + 消息事件 | 实时 |
| **成员活跃度** | 最后发言时间、发言频次、活跃天数 | 消息事件统计 | 实时 |
| **入退群记录** | 加入时间、退出时间、邀请人 | `chat_member_updated` 事件 | 实时 |
| **消息统计** | 总消息数、日均消息数、高峰时段 | 消息事件聚合 | 每小时 |
| **成员角色** | creator / administrator / member / restricted / left | TG API | 变更时 |

**隐私原则**：
- 不采集消息内容正文（只统计元数据：时间、发送者、类型）
- 不采集群外用户信息
- 群主可选择开启/关闭特定采集项
- 遵守 Telegram ToS 和 GDPR

#### F3: 群主数据面板

群主通过 Bot 命令查看群聊会员数据：

| 命令 | 功能 | 输出示例 |
|------|------|---------|
| `/stats` | 群概览 | 成员数、日活、周活、消息趋势 |
| `/members` | 活跃成员 Top 20 | 排名、昵称、活跃度评分 |
| `/growth` | 增长趋势 | 近 7/30 天新增、流失、净增 |
| `/inactive` | 沉默用户 | 超过 N 天未发言的成员列表 |
| `/peak` | 高峰分析 | 每日消息分布、最活跃时段 |
| `/export` | 数据导出 | CSV/JSON 格式导出（限群主） |
| `/settings` | 采集设置 | 开关各项采集功能 |
| `/help` | 帮助 | 命令列表和使用说明 |

#### F4: 会员信息与链上 Entity 关联

**核心价值**：将 Telegram 群成员与链上 Entity Member 系统打通

```
Telegram 群成员 (user_id: 12345, username: @alice)
        ↓ Bot 采集
Nexus 会员数据库
        ↓ 群主授权关联
pallet-entity-member (链上 Member 记录)
        ↓
佣金系统 / 推荐链 / 会员等级

关联流程:
1. 群成员在群内发送 /bind <wallet_address> 或点击 Mini App 链接
2. Bot 记录 TG user_id ↔ 链上 AccountId 映射
3. 群主可在面板中查看哪些成员已绑定链上身份
4. 已绑定成员自动成为 Entity Member → 进入推荐链
```

#### F5: 智能群管辅助（免费增值）

| 功能 | 说明 | 依赖 |
|------|------|------|
| **新成员欢迎** | 可定制欢迎语 + 群规提醒 | `chat_member_updated` |
| **反垃圾过滤** | 识别广告/诈骗消息并删除 | 复用 `nexus-node/rule_engine.rs` 的 `LinkFilterRule` |
| **入群验证** | 简单验证码/问答防机器人 | 复用 `JoinRequestRule` |
| **关键词提醒** | 群主设定关键词，触发时通知 | 新增 `KeywordAlertRule` |
| **定时公告** | 定时发送群公告 | 新增定时任务 |

### 10.4 技术架构

#### 10.4.1 与现有组件的关系

```
┌─────────────────────────────────────────────────────────────┐
│                   Nexus Official Bot                        │
│                                                             │
│  ┌──────────────────────────────────────────────────┐       │
│  │  官方代理服务 (nexus-official-agent)             │       │
│  │                                                  │       │
│  │  复用 nexus-agent 核心:                          │       │
│  │  ├── webhook.rs      — Webhook 接收              │       │
│  │  ├── signer.rs       — Ed25519 签名              │       │
│  │  ├── multicaster.rs  — 多播到 Nexus 节点        │       │
│  │  ├── executor.rs     — TG API 执行               │       │
│  │  │                                               │       │
│  │  新增模块:                                        │       │
│  │  ├── member_collector.rs  — 会员信息采集引擎     │       │
│  │  ├── stats_engine.rs      — 统计聚合引擎         │       │
│  │  ├── command_handler.rs   — 群主命令处理         │       │
│  │  ├── group_registry.rs    — 多群管理注册表       │       │
│  │  ├── data_store.rs        — 会员数据持久化       │       │
│  │  └── chain_linker.rs      — TG ↔ 链上身份关联   │       │
│  └──────────────────────────────────────────────────┘       │
│                          │                                   │
│                          ↓                                   │
│  ┌──────────────────────────────────────────────────┐       │
│  │  Nexus Node 集群 (共识验证)                      │       │
│  │  - 管理动作(ban/delete) 需经共识验证              │       │
│  │  - 数据采集动作无需共识（只读操作）               │       │
│  └──────────────────────────────────────────────────┘       │
│                          │                                   │
│                          ↓                                   │
│  ┌──────────────────────────────────────────────────┐       │
│  │  Cosmos 链上                                      │       │
│  │  - pallet-bot-registry: 官方 Bot 注册             │       │
│  │  - pallet-entity-member: 会员关联                 │       │
│  │  - pallet-bot-group-mgmt: 群规则/行为日志        │       │
│  └──────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

#### 10.4.2 多群管理架构

与自建代理（一个 Bot Token 对应一个群）不同，官方代理用**一个 Bot Token 服务所有群**：

```rust
// 概念设计: group_registry.rs

/// 已注册群组信息
pub struct RegisteredGroup {
    /// Telegram chat_id
    pub chat_id: i64,
    /// 群名称
    pub chat_title: String,
    /// 群主 TG user_id
    pub owner_user_id: i64,
    /// 关联的链上 Entity ID（可选）
    pub entity_id: Option<u64>,
    /// 采集配置
    pub collection_config: CollectionConfig,
    /// 激活时间
    pub activated_at: chrono::DateTime<chrono::Utc>,
    /// 状态
    pub status: GroupStatus, // Active / Paused / Removed
}

/// 采集配置（群主可定制）
pub struct CollectionConfig {
    /// 是否采集活跃度统计
    pub collect_activity: bool,      // 默认 true
    /// 是否采集入退群记录
    pub collect_join_leave: bool,    // 默认 true
    /// 是否采集消息统计（不含正文）
    pub collect_msg_stats: bool,     // 默认 true
    /// 是否启用反垃圾
    pub enable_anti_spam: bool,      // 默认 true
    /// 是否启用入群验证
    pub enable_join_verify: bool,    // 默认 false
}

/// 全局群组注册表
pub struct GroupRegistry {
    groups: DashMap<i64, RegisteredGroup>,  // chat_id → group
    db: sled::Db,                           // 持久化
}
```

#### 10.4.3 会员数据采集引擎

```rust
// 概念设计: member_collector.rs

/// 群成员快照
pub struct MemberSnapshot {
    pub user_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub is_bot: bool,
    pub role: MemberRole,          // Creator/Admin/Member/Restricted
    pub joined_at: Option<DateTime<Utc>>,
    pub last_seen: DateTime<Utc>,
    pub message_count: u64,
    pub active_days: u32,
    /// 链上关联的 AccountId（如果已绑定）
    pub chain_account: Option<String>,
}

/// 群统计聚合
pub struct GroupStats {
    pub total_members: u32,
    pub active_today: u32,         // 今日发言人数
    pub active_week: u32,          // 本周发言人数
    pub messages_today: u64,
    pub messages_week: u64,
    pub new_members_week: u32,     // 本周新增
    pub left_members_week: u32,    // 本周流失
    pub peak_hour: u8,             // 消息最多的小时 (0-23)
    pub top_members: Vec<(i64, u64)>, // (user_id, msg_count) Top N
}
```

#### 10.4.4 数据存储方案

| 层级 | 存储 | 数据 | 说明 |
|------|------|------|------|
| **热数据** | 内存 (DashMap) | 当前在线群列表、实时消息计数 | 毫秒级查询 |
| **温数据** | 嵌入式 DB (sled/SQLite) | 成员快照、历史统计、采集配置 | 持久化、无外部依赖 |
| **冷数据** | IPFS (复用 pallet-storage-service) | 历史导出、长期归档 | 可选，链上索引 |
| **链上** | Substrate 链上存储 | 会员绑定关系、群注册记录 | 不可篡改 |

### 10.5 商业模型

#### 10.5.1 核心理念：永久免费 + 广告变现

**所有群主永久免费使用**，不设付费墙。Bot 通过在群内定期投放广告信息来覆盖运营成本并盈利。

```
群主拉 @NexusBot 入群
    ↓
免费获得全部群管功能（会员采集/统计/反垃圾/群管辅助）
    ↓
Bot 定期在群内发送广告消息（Entity 投放的推广信息）
    ↓
广告主 (Entity) 付费投放 → Nexus 收取广告费
    ↓
广告费覆盖运营成本 + 利润
    ↓
群主如果也是 Entity → 可以在其他群投放自己的广告
```

**群主的交换**：用群内的广告位换取免费的群管工具 + 会员数据。

#### 10.5.2 广告投放规则

| 参数 | 默认值 | 群主可调 | 说明 |
|------|--------|---------|------|
| **每日广告次数** | 2 条/天 | ✅ (1-5) | 群主可降低/增加频率 |
| **投放时段** | 10:00-22:00 | ✅ | 避免深夜打扰 |
| **最短间隔** | 4 小时 | ❌ | 硬性限制，防止刷屏 |
| **广告格式** | 文字 + 链接/图片 | ❌ | 统一格式，标注"[广告]" |
| **广告内容审核** | 平台审核 + AI 过滤 | ❌ | 禁止违法/色情/诈骗 |
| **群主免广告** | 付费 Entity 可选 | — | 绑定 Entity 并付费可去广告 |

#### 10.5.3 广告来源与匹配

```
广告主 (Entity)
    ↓
通过 Nexus 广告后台提交广告:
  - 广告内容（文案 + 链接/图片）
  - 目标群类别（行业/规模/地区/语言）
  - 投放预算（CPM 按千次展示付费）
  - 投放时段
    ↓
Nexus 广告引擎 (链上 + 链下):
  - 审核广告内容（AI 过滤 + 人工审核）
  - 匹配目标群（群画像 ↔ 广告标签）
  - 排期调度（优先高出价 + 轮换）
    ↓
@NexusBot 在匹配的群内发送广告
    ↓
记录展示/点击/转化数据 → 反馈给广告主
```

#### 10.5.4 广告投放监控机制

**核心问题**：如何确保广告确实在群内发送了？如何防止伪造投放数据？

##### 监控架构

```
┌──────────────────────────────────────────────────────────┐
│                  广告投放监控系统                          │
│                                                          │
│  ┌─────────────────────────────────────────────────┐     │
│  │  Layer 1: 发送端记录                             │     │
│  │                                                 │     │
│  │  Bot 每次发送广告时记录:                          │     │
│  │  ├── ad_id        — 广告唯一 ID                  │     │
│  │  ├── chat_id      — 目标群 ID                    │     │
│  │  ├── message_id   — TG 返回的消息 ID（发送凭证） │     │
│  │  ├── sent_at      — 发送时间戳                   │     │
│  │  ├── ad_content   — 广告内容 hash                │     │
│  │  └── status       — sent / failed / deleted      │     │
│  └─────────────────────────────────────────────────┘     │
│                          │                                │
│  ┌─────────────────────────────────────────────────┐     │
│  │  Layer 2: Nexus 共识验证                        │     │
│  │                                                 │     │
│  │  广告发送指令经 Nexus 多节点共识:                │     │
│  │  1. Bot 生成广告发送请求 → 签名 → 多播到 K 节点  │     │
│  │  2. M/K 节点确认发送合法性（频率/内容/时段）     │     │
│  │  3. Leader 节点执行发送 → TG API sendMessage     │     │
│  │  4. 返回 message_id 作为发送凭证                 │     │
│  │  5. 发送记录经共识后上链                         │     │
│  │                                                 │     │
│  │  防伪保证:                                       │     │
│  │  - 不经共识的广告发送 = Equivocation → 罚没      │     │
│  │  - 伪造 message_id → TG API 可验证（getChat）   │     │
│  │  - 多节点独立记录，篡改需攻破 M 个节点           │     │
│  └─────────────────────────────────────────────────┘     │
│                          │                                │
│  ┌─────────────────────────────────────────────────┐     │
│  │  Layer 3: 链上存证与审计                         │     │
│  │                                                 │     │
│  │  pallet-bot-group-mgmt::log_action 记录:         │     │
│  │  ├── ActionType::AdDelivery                     │     │
│  │  ├── ad_id + chat_id + message_id               │     │
│  │  ├── block_number (不可篡改时间戳)               │     │
│  │  └── consensus_proof (M/K 签名)                 │     │
│  │                                                 │     │
│  │  广告主可链上查询:                                │     │
│  │  - 自己的广告在哪些群投放了                       │     │
│  │  - 每条广告的发送凭证（message_id）              │     │
│  │  - 投放时间和频率是否符合预期                    │     │
│  └─────────────────────────────────────────────────┘     │
│                          │                                │
│  ┌─────────────────────────────────────────────────┐     │
│  │  Layer 4: 效果追踪                               │     │
│  │                                                 │     │
│  │  展示量 (Impression):                            │     │
│  │  ├── 群成员数 × 活跃率 = 预估曝光量              │     │
│  │  └── 基于群活跃度数据（Bot 已采集）              │     │
│  │                                                 │     │
│  │  点击量 (Click):                                 │     │
│  │  ├── 广告链接附带 tracking 参数                  │     │
│  │  │   例: https://entity.example/ad?ref=nexus    │     │
│  │  │       &ad_id=xxx&chat_id=yyy                  │     │
│  │  └── 通过 redirect 服务统计点击次数              │     │
│  │                                                 │     │
│  │  转化量 (Conversion):                            │     │
│  │  ├── 用户点击 → 注册 Entity Member               │     │
│  │  └── 链上可追溯（referral_code 归因）            │     │
│  └─────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────┘
```

##### 广告投放数据结构

```rust
// 概念设计: ad_monitor.rs

/// 广告投放记录
pub struct AdDeliveryRecord {
    /// 广告 ID
    pub ad_id: String,
    /// 广告主 Entity ID
    pub advertiser_entity_id: u64,
    /// 目标群 chat_id
    pub chat_id: i64,
    /// TG 返回的 message_id（发送凭证，不可伪造）
    pub message_id: i64,
    /// 广告内容 hash（SHA-256）
    pub content_hash: [u8; 32],
    /// 发送时间
    pub sent_at: DateTime<Utc>,
    /// 投放状态
    pub status: AdStatus,
    /// 共识证明（M/K 节点签名）
    pub consensus_proof: Vec<NodeSignature>,
}

pub enum AdStatus {
    /// 已发送（TG API 返回成功）
    Sent,
    /// 发送失败（TG API 返回错误）
    Failed { reason: String },
    /// 被群主删除
    DeletedByAdmin,
    /// 已过期（超过展示周期）
    Expired,
}

/// 广告效果统计
pub struct AdPerformance {
    pub ad_id: String,
    /// 投放群数
    pub groups_delivered: u32,
    /// 预估曝光（群成员数 × 活跃率之和）
    pub estimated_impressions: u64,
    /// 链接点击数
    pub clicks: u64,
    /// 转化数（注册 Entity Member）
    pub conversions: u32,
    /// 广告主已花费
    pub spent: u128,
    /// CPM（千次展示成本）
    pub effective_cpm: f64,
}
```

##### 群主广告管理命令

| 命令 | 功能 | 说明 |
|------|------|------|
| `/ad_settings` | 广告设置面板 | 调整每日广告数量/时段 |
| `/ad_freq <1-5>` | 设置每日广告频率 | 默认 2 条/天 |
| `/ad_hours <HH-HH>` | 设置投放时段 | 如 `/ad_hours 10-20` |
| `/ad_history` | 查看广告投放记录 | 近 7 天在本群投放的广告列表 |
| `/ad_block <ad_id>` | 屏蔽特定广告主 | 该广告主的广告不再投放到本群 |
| `/ad_report <ad_id>` | 举报违规广告 | 触发审核流程 |

##### 防作弊机制

| 作弊类型 | 检测方式 | 处罚 |
|---------|---------|------|
| **伪造投放** — 未发送但报告已发送 | TG API `getMessages` 抽查验证 + Nexus 共识 | 广告费退还 + 节点罚没 |
| **重复投放** — 同一广告多次发送 | message_id 唯一性校验 + 频率硬限制 | 自动过滤重复 |
| **刷量** — 向空群/僵尸群投放 | 群活跃度评分 < 阈值的群不投放 | 不计入展示量 |
| **群主删广告** — 发送后立即删除 | 监听 `message_deleted` 事件 | 记录删除率，高删除率群降权 |
| **广告主刷点击** | 点击去重（IP + User-Agent + 时间窗口） | 过滤异常点击 |

#### 10.5.5 收入模型

```
广告主 (Entity) 充值广告预算
    ↓
按 CPM（千次展示）或 CPC（单次点击）扣费
    ↓
收入分配:
    ├── 70% → Nexus 平台运营（服务器/开发/审核）
    ├── 20% → 群主激励（吸引更多群主加入）
    │         群主可选择: 提现 / 兑换增值功能 / 转为链上资产
    └── 10% → Nexus 节点运营者（共识验证激励）
```

**群主激励**：群主不仅免费用，还能通过接受广告获得收益。群越活跃、成员越多，分到的广告收入越高。

| 群规模 | 预估月广告收入（群主份额 20%） |
|--------|------------------------------|
| 100 人群 | $2-5/月 |
| 500 人群 | $10-25/月 |
| 2000 人群 | $40-100/月 |
| 10000 人群 | $200-500/月 |

#### 10.5.6 免费版 vs Entity 绑定版

| 功能 | 免费版（接受广告） | Entity 绑定版（可选去广告） |
|------|-------------------|--------------------------|
| 群成员统计 | ✅ | ✅ |
| 活跃度排名 | ✅ 全部 | ✅ 全部 |
| 入退群记录 | ✅ 全部 | ✅ 全部 |
| 数据导出 | ✅ CSV/JSON | ✅ CSV/JSON |
| 反垃圾过滤 | ✅ 基础 | ✅ AI 高级 |
| 入群验证 | ✅ 验证码 | ✅ 自定义问答 |
| 定时公告 | ✅ 2 条/天 | ✅ 无限制 |
| 链上会员关联 | ❌ | ✅ |
| AI 客服/获客 | ❌ | ✅ |
| 群内广告 | 2 条/天 | 可选去除（付费） |
| 广告分成收入 | ✅ 20% 分成 | 可选接受广告赚收入 |
| 自己投放广告 | ❌ | ✅ 投放到其他群 |

### 10.6 实施计划

#### Sprint 1: 基础框架（2 周）

- [ ] 新建 `nexus-official-agent/` 项目（复用 `nexus-agent` 的核心模块）
- [ ] 实现 `GroupRegistry`：Bot 入群/退群自动注册/注销
- [ ] 实现 `MemberCollector`：通过 `chat_member_updated` + 消息事件采集成员数据
- [ ] 数据持久化：sled 嵌入式 DB
- [ ] 基础命令：`/stats`、`/members`、`/help`

**验证标准**: Bot 入群后 1 分钟内开始采集，`/stats` 返回准确的成员数和消息统计

#### Sprint 2: 统计与管理（2 周）

- [ ] `StatsEngine`：活跃度评分算法、增长趋势计算、高峰时段分析
- [ ] 完整命令集：`/growth`、`/inactive`、`/peak`、`/settings`
- [ ] 反垃圾过滤：复用 `LinkFilterRule` + 基础广告词库
- [ ] 新成员欢迎：可定制模板
- [ ] 入群验证：简单验证码

**验证标准**: 群主通过命令获取完整的群分析报告，反垃圾误报率 < 5%

#### Sprint 3: 链上关联（3 周）

- [ ] `/bind` 命令：TG user_id ↔ 链上 AccountId 绑定
- [ ] 链上提交：通过 subxt 调用 `pallet-entity-member::register_member`
- [ ] `pallet-bot-registry` 注册官方 Bot 身份
- [ ] 群主可在链上关联群组到 Entity
- [ ] 已绑定成员数据同步到链上

**验证标准**: 群成员通过 `/bind` 成功绑定链上身份，并出现在 Entity Member 列表中

#### Sprint 4: 增值与优化（2 周）

- [ ] `/export` 数据导出（CSV/JSON）
- [ ] 关键词提醒功能
- [ ] 定时公告功能
- [ ] 多群管理优化（单 Bot 支撑 1000+ 群）
- [ ] 性能优化：消息处理吞吐 > 1000 msg/s

**验证标准**: 单实例稳定服务 500+ 群，`/export` 正确导出全部成员数据

### 10.7 与自建代理的区分

```
┌─────────────────────┬────────────────────────┬─────────────────────────┐
│                     │ 自建代理 nexus-agent  │ 官方代理 @NexusBot     │
├─────────────────────┼────────────────────────┼─────────────────────────┤
│ 部署方式             │ 群主自行部署 Docker     │ 无需部署，拉入群即用     │
│ Bot Token           │ 群主自己的 Bot          │ Nexus 官方 Bot         │
│ 数据主权             │ 完全自主（本地存储）    │ Nexus 集群托管         │
│ 费用                │ 服务器成本自付          │ 免费基础版              │
│ 功能范围             │ 完整 Nexus 协议功能    │ 会员采集 + 群管基础功能  │
│ 共识验证             │ 管理动作经多节点验证    │ 管理动作经多节点验证     │
│ 适用场景             │ Entity 运营方/技术用户  │ 所有群主/快速推广       │
│ 链上关联             │ 原生支持               │ 付费版支持              │
│ AI 获客/客服         │ Phase 1/2 支持         │ 付费版支持              │
│ 可定制性             │ 高（源码级）            │ 中（命令配置）          │
└─────────────────────┴────────────────────────┴─────────────────────────┘
```

### 10.8 关键指标（KPI）

| 指标 | 3 个月目标 | 6 个月目标 | 12 个月目标 |
|------|-----------|-----------|------------|
| 服务群数 | 100 | 1,000 | 10,000 |
| 采集会员数 | 10,000 | 100,000 | 1,000,000 |
| 链上绑定率 | 5% | 10% | 20% |
| 日活群 (DAG) | 50 | 500 | 5,000 |
| Entity 转化 | 10 | 100 | 500 |

### 10.9 自建代理广告激励方案 — 可行性分析

#### 10.9.1 核心思路

当前 `nexus-agent` 自建代理的群主需要自付服务器成本。如果群主**主动接受 Nexus 广告网络的广告投放**，就能获得以下优惠/回报：

```
自建代理群主 opt-in 接受广告
    ↓
Nexus 广告网络向该群投放广告（通过群主的 Bot 发送）
    ↓
群主获得优惠:
    ├── 链上服务费折扣（质押费/交易手续费减免）
    ├── 广告收入分成（比官方代理群主更高，因为自付服务器成本）
    ├── 增值功能免费解锁（AI 客服/获客/高级分析）
    └── Nexus 积分/Token 奖励
```

#### 10.9.2 可行性分析

##### 技术可行性：⭐⭐⭐⭐ — 高

**已有基础**：

| 组件 | 复用方式 | 改动量 |
|------|---------|--------|
| `nexus-agent/webhook.rs` | 已有 Webhook 接收 + TG API 发送管道 | 低 — 新增广告消息类型 |
| `nexus-agent/executor.rs` | `TelegramExecutor` 已支持 `sendMessage` | 零改动 |
| `nexus-node/rule_engine.rs` | 扩展 Rule 链增加 `AdDeliveryRule` | 中 — 新增规则 |
| `pallet-bot-consensus` | 广告发送经共识验证 | 低 — 复用现有共识流程 |
| `pallet-bot-group-mgmt` | `log_action(AdDelivery)` 记录投放 | 低 — 新增 ActionType |

**需新增的模块**：

```rust
// nexus-agent 新增: ad_receiver.rs（概念设计）

/// 广告接收器 — 自建代理从 Nexus 网络拉取匹配的广告
pub struct AdReceiver {
    /// Nexus 广告 API 端点
    ad_api_endpoint: String,
    /// 本群的广告配置
    ad_config: AdConfig,
    /// 投放调度器
    scheduler: AdScheduler,
}

pub struct AdConfig {
    /// 是否 opt-in 接受广告
    pub enabled: bool,                    // 默认 false
    /// 每日广告上限
    pub max_daily_ads: u8,                // 默认 2
    /// 允许投放时段
    pub allowed_hours: (u8, u8),          // 默认 (10, 22)
    /// 屏蔽的广告主列表
    pub blocked_advertisers: Vec<u64>,
    /// 允许的广告类别
    pub allowed_categories: Vec<AdCategory>,
}

impl AdReceiver {
    /// 定时从 Nexus 广告网络拉取匹配的广告
    async fn poll_ads(&self) -> Vec<AdContent> {
        // 1. 向 Nexus 广告 API 请求匹配广告
        //    带上群画像（规模/类别/语言/活跃度）
        // 2. 过滤：排除屏蔽广告主 + 类别不匹配
        // 3. 返回待投放广告列表
    }

    /// 投放广告（经 Nexus 共识后执行）
    async fn deliver_ad(&self, ad: &AdContent) -> AdDeliveryResult {
        // 1. 签名广告发送请求
        // 2. 多播到 K 个 Nexus 节点
        // 3. M/K 共识确认 → Leader 回传执行指令
        // 4. TelegramExecutor.send_message(ad.content)
        // 5. 记录 message_id → 上报 Nexus 网络
    }
}
```

**数据流**：

```
Nexus 广告网络 (中心化广告库)
        │
        │  广告 API: GET /ads/match?group_size=500&lang=zh&category=crypto
        ↓
nexus-agent (AdReceiver)
        │
        │  签名 → 多播到 K 个 nexus-node
        ↓
Nexus 共识验证 (M/K 确认)
        │
        │  Leader 回传执行指令
        ↓
nexus-agent (TelegramExecutor)
        │
        │  sendMessage → Telegram 群
        ↓
记录 message_id → 上报广告网络 → 链上存证
        │
        ↓
Nexus 网络结算 → 优惠/收入到群主链上账户
```

##### 商业合理性：⭐⭐⭐⭐⭐ — 非常高

**多方共赢**：

| 参与者 | 收益 |
|--------|------|
| **群主（自建代理）** | 服务器成本被广告收入覆盖甚至盈利；免费解锁高级功能 |
| **广告主（Entity）** | 接触自建代理群的高价值用户（技术型群主 = 高质量社群） |
| **Nexus 网络** | 扩大广告库存（自建代理群 + 官方代理群）；增加网络参与者 |
| **群成员** | 可能看到有价值的广告（经群主筛选 + AI 匹配） |

**关键优势 — 相比官方代理群主的分成更高**：

```
官方代理群主:
  - 零成本使用 → 广告分成 20%
  - 群主不承担任何成本

自建代理群主:
  - 自付服务器成本 → 广告分成 40%（更高！）
  - 理由: 群主自己承担了基础设施成本，应获得更多回报
  - 额外激励: 高级功能免费解锁
```

##### 激励设计

**阶梯式优惠**：

| 广告接受等级 | 每日广告数 | 优惠/回报 |
|-------------|-----------|----------|
| **关闭** (默认) | 0 条 | 无优惠，正常付费 |
| **轻度** | 1 条/天 | 链上交易手续费 -20%；广告收入 40% 分成 |
| **标准** | 2 条/天 | 手续费 -40%；AI 客服免费解锁；收入 40% 分成 |
| **深度** | 3-5 条/天 | 手续费 -60%；全功能免费解锁；收入 40% 分成 + 额外 Token 奖励 |

**预估收入对比**：

| 群规模 | 自建代理月服务器成本 | 广告月收入（标准级，40% 分成） | 净收益 |
|--------|--------------------|-----------------------------|--------|
| 200 人群 | ~$5/月 (VPS) | $4-10/月 | 接近覆盖成本 |
| 500 人群 | ~$5/月 | $10-25/月 | 盈利 $5-20 |
| 2000 人群 | ~$10/月 | $40-100/月 | 盈利 $30-90 |
| 10000 人群 | ~$20/月 | $200-500/月 | 盈利 $180-480 |

**结论**：500 人以上的群，广告收入不仅覆盖服务器成本，还有可观盈利。

#### 10.9.3 与现有架构的融合

##### 广告投放通道对比

```
┌──────────────────────────────────────────────────────────────────┐
│                                                                  │
│  官方代理通道:                     自建代理通道:                    │
│                                                                  │
│  Nexus 广告网络                   Nexus 广告网络                 │
│       │                                 │                        │
│       ↓                                 ↓                        │
│  @NexusBot 直接发送               nexus-agent 拉取广告          │
│  (Nexus 控制发送)                 (群主控制发送)                  │
│       │                                 │                        │
│       ↓                                 ↓                        │
│  群主无法控制广告内容              群主可过滤广告类别/广告主        │
│  (只能调频率/时段)                 (更高自主权)                    │
│       │                                 │                        │
│       ↓                                 ↓                        │
│  广告分成 20%                      广告分成 40%                   │
│  (Nexus 承担基础设施)             (群主自付基础设施)              │
│                                                                  │
│  两个通道共享同一个广告库、共识验证、链上存证系统                    │
└──────────────────────────────────────────────────────────────────┘
```

##### 链上激励记录

```
pallet-bot-consensus 扩展:
    ├── 已有: 节点注册 / 质押 / 信誉评分
    └── 新增: AdOptInConfig 存储
              ├── bot_id → AdConfig (opt-in 级别)
              ├── bot_id → AdRewardAccrued (累计广告奖励)
              └── bot_id → AdDeliveryCount (投放计数)

结算流程:
    1. nexus-agent 投放广告 → 链上记录 AdDelivery
    2. 每个结算周期（如每周）:
       - 统计 bot_id 的有效投放数
       - 计算广告收入分成
       - 自动转账到群主链上账户
       - 自动应用手续费折扣
```

##### 群主操作体验

```bash
# 在 nexus-agent 配置中 opt-in 广告（环境变量方式）
AD_ENABLED=true
AD_MAX_DAILY=2
AD_HOURS="10-22"
AD_BLOCKED_CATEGORIES="gambling,adult"

# 或通过 Telegram Bot 命令
/ad_optin standard     # 启用标准级广告接受
/ad_optout             # 关闭广告
/ad_earnings           # 查看广告收入
/ad_config             # 修改广告设置
```

#### 10.9.4 风险与挑战

| 风险 | 等级 | 缓解措施 |
|------|------|---------|
| 群主 opt-in 后立即删除广告消息 | 🟡 中 | 监听 `message_deleted` 事件；连续删除 → 降低分成/暂停 |
| 群主伪造投放数据 | 🟡 中 | 所有投放经 Nexus M/K 共识验证 + `message_id` 链上存证 |
| 广告内容不适合群主社群 | 🟢 低 | 群主可设置类别过滤 + 屏蔽特定广告主 |
| 群成员反感广告 → 退群 | 🟡 中 | 限制频率（最多 5 条/天）+ 群主自主控制 |
| 自建代理群数量少导致广告库存不足 | 🟡 中 | 与官方代理共享广告库；自建代理群是高价值补充 |
| 广告分成计算争议 | 🟢 低 | 全部链上存证，透明可审计 |

#### 10.9.5 可行性结论

**可行性评分：⭐⭐⭐⭐（高）**

| 维度 | 评分 | 理由 |
|------|------|------|
| **技术可行性** | ⭐⭐⭐⭐ | 复用现有 webhook/executor/共识管道，只需新增 `AdReceiver` 模块 |
| **商业合理性** | ⭐⭐⭐⭐⭐ | 多方共赢：群主降低成本甚至盈利，广告主获高质量流量，网络扩大库存 |
| **用户体验** | ⭐⭐⭐⭐ | opt-in 机制 + 群主自主过滤 = 尊重群主权益 |
| **与现有架构兼容** | ⭐⭐⭐⭐ | 共享广告库/共识/存证系统，自建代理只多一个拉取+执行模块 |
| **激励设计** | ⭐⭐⭐⭐⭐ | 阶梯式优惠清晰：广告越多 → 折扣越大 → 群主自行权衡 |

**核心价值**：让自建代理群主从"纯成本消耗"变为"可盈利的广告渠道"，同时保持数据自主权和更高的控制力。这与官方代理形成**互补生态**：

```
官方代理: 零门槛 + 免费 + 广告换功能 → 快速增长用户量
自建代理: 高自主权 + 高分成 + 广告换优惠 → 服务高价值群主
    ↓
共同构成 Nexus 广告网络的供给侧
    ↓
广告主只需在一个后台投放 → 同时触达两类群
```

#### 10.9.6 实施优先级

建议在 **Sprint 2（统计与管理）** 之后、**Sprint 3（链上关联）** 中一并实现：

- [ ] nexus-agent 新增 `AdReceiver` 模块 + `AdConfig` 环境变量
- [ ] nexus-agent 新增 `/ad_optin`、`/ad_optout`、`/ad_earnings` 命令
- [ ] nexus-node 规则引擎新增 `AdDeliveryRule`
- [ ] `pallet-bot-consensus` 新增 `AdOptInConfig` 存储 + 结算逻辑
- [ ] 广告 API 后端（链下服务）：广告库 + 匹配 + 分发
- [ ] 链上结算：每周自动计算分成 + 转账

---

## 11. AI 客服 — 深度实现方案

### 11.1 目标与定位

**核心目标**：让每个 Entity 的 Telegram/Discord 群拥有一个 7×24 小时的 AI 客服，能够：

1. **回答产品问题** — 基于 Entity 知识库（RAG）精准回复
2. **引导下单** — 识别购买意图 → 推荐商品 → 引导到 Shop 完成交易
3. **售后支持** — 查询订单状态、处理常见问题
4. **自动学习** — 从群主/管理员的回复中持续优化知识库

**不做什么**：
- 不代替群管功能（反垃圾/入群验证已由 `nexus-node/rule_engine.rs` 处理）
- 不主动群发营销（那是 Phase 5 AI 主动获客的范畴）
- 不处理涉及资金操作的请求（仅展示信息，操作需用户自行完成）

### 11.2 消息处理全流程

```
用户在群里发消息: "你们这个 VPN 套餐 A 多少钱？支持几个设备？"
        │
        ↓
┌───────────────────────────────────────────────────────────────┐
│ Step 1: nexus-agent 接收 Telegram Webhook                    │
│                                                               │
│ webhook.rs: POST /webhook                                     │
│   → 验证 secret → 解析 TelegramUpdate → Ed25519 签名          │
│   → 构造 SignedMessage → 确定性选择 K 个节点 → 并发多播       │
└───────────────────┬───────────────────────────────────────────┘
                    │
                    ↓
┌───────────────────────────────────────────────────────────────┐
│ Step 2: nexus-node 接收 + 四层验证                            │
│                                                               │
│ api.rs: POST /v1/message                                      │
│   → Ed25519 签名验证 → Bot 活跃检查 → 公钥匹配 → 目标节点检查 │
│   → 送入 GossipEngine                                         │
└───────────────────┬───────────────────────────────────────────┘
                    │
                    ↓
┌───────────────────────────────────────────────────────────────┐
│ Step 3: 规则引擎评估 (rule_engine.rs)                          │
│                                                               │
│ 规则链: JoinRequestRule → CommandRule → LinkFilterRule         │
│       → ★ AiCustomerServiceRule (新增) ★                      │
│       → DefaultRule                                           │
│                                                               │
│ AiCustomerServiceRule:                                        │
│   1. 检查群是否启用 AI 客服                                    │
│   2. 检查消息是否为用户提问（排除命令/bot消息/媒体文件）       │
│   3. 匹配: 返回 ActionType::AiReply + 原始消息文本            │
└───────────────────┬───────────────────────────────────────────┘
                    │
                    ↓
┌───────────────────────────────────────────────────────────────┐
│ Step 4: Gossip 共识 (gossip/engine.rs)                        │
│                                                               │
│ M/K 节点确认消息合法 → 达成共识                                │
│ Leader 节点被选出（round-robin: leader_idx = seq % K）        │
└───────────────────┬───────────────────────────────────────────┘
                    │
                    ↓
┌───────────────────────────────────────────────────────────────┐
│ Step 5: Leader 执行 AI 推理 (leader.rs + ★新增 ai_handler★)   │
│                                                               │
│ Leader 节点（且仅 Leader）执行 LLM 调用:                       │
│   1. 从 Entity 知识库加载 RAG 上下文                           │
│   2. 构造 LLM prompt (system + 知识库 + 对话历史 + 用户问题)  │
│   3. 调用 LLM API (Claude/OpenAI/DeepSeek)                    │
│   4. 获取回复文本                                              │
│   5. 安全过滤（去除幻觉/敏感信息）                             │
│                                                               │
│ 返回: ExecuteAction {                                         │
│   action_type: SendMessage,                                   │
│   params: { "text": "套餐A 价格...", "reply_to_message_id": N }│
│ }                                                             │
└───────────────────┬───────────────────────────────────────────┘
                    │
                    ↓
┌───────────────────────────────────────────────────────────────┐
│ Step 6: 回传 Agent 执行 (executor.rs)                          │
│                                                               │
│ Leader → POST /v1/execute → nexus-agent                      │
│ TelegramExecutor.call_tg_api("sendMessage", {                 │
│   chat_id, text, reply_to_message_id                          │
│ })                                                            │
│ → 签名回执 → 返回 Leader → 链上存证                           │
└───────────────────────────────────────────────────────────────┘

最终效果:
  Bot 在群内回复: "套餐 A 价格为 $9.99/月，支持同时 5 台设备连接。
  详情请查看: https://shop.example.com/product/123?ref=nexus_ai"
```

### 11.3 核心模块设计

#### 11.3.1 AiCustomerServiceRule（nexus-node 规则引擎扩展）

```rust
// nexus-node/src/rule_engine.rs 新增规则

/// AI 客服规则 — 在 LinkFilterRule 之后、DefaultRule 之前插入
pub struct AiCustomerServiceRule {
    /// 已启用 AI 客服的群列表（从链上缓存）
    enabled_groups: Arc<DashMap<i64, AiServiceConfig>>,
}

/// 群级 AI 客服配置（链上存储，本地缓存）
#[derive(Debug, Clone)]
pub struct AiServiceConfig {
    /// 关联的 Entity ID
    pub entity_id: u64,
    /// 关联的 Shop ID
    pub shop_id: u64,
    /// 知识库 IPFS CID
    pub knowledge_base_cid: Option<String>,
    /// 回复语言
    pub language: String,       // "zh" / "en" / "auto"
    /// 回复风格
    pub tone: AiTone,           // Professional / Friendly / Casual
    /// 最大回复长度
    pub max_reply_length: u16,  // 默认 500 字符
    /// 是否自动声明 AI 身份
    pub auto_disclosure: bool,  // 默认 true
    /// 冷却时间（同一用户连续提问最短间隔秒数）
    pub cooldown_seconds: u32,  // 默认 10
    /// 每小时最大回复数
    pub max_replies_per_hour: u16, // 默认 30
}

#[derive(Debug, Clone)]
pub enum AiTone {
    Professional,  // 正式商务
    Friendly,      // 亲切友好
    Casual,        // 随意轻松
}

impl Rule for AiCustomerServiceRule {
    fn name(&self) -> &str { "ai_customer_service" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        // 1. 排除非文本消息
        if ctx.text.is_empty() || ctx.is_command || ctx.is_join_request || ctx.is_callback {
            return None;
        }

        // 2. 排除 Bot 消息（避免自问自答）
        let is_bot = ctx.message.telegram_update
            .pointer("/message/from/is_bot")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if is_bot { return None; }

        // 3. 检查群是否启用 AI 客服
        let config = self.enabled_groups.get(&ctx.chat_id)?;

        // 4. 判断消息是否像"提问"（简单启发式 + 可选 LLM 预判断）
        if !Self::looks_like_question(&ctx.text) {
            return None;
        }

        // 5. 频率限制检查
        // ... (基于 cooldown_seconds + max_replies_per_hour)

        // 6. 返回 AI 回复动作
        Some(RuleAction {
            action_type: ActionType::SendMessage,
            chat_id: ctx.chat_id,
            params: serde_json::json!({
                "ai_request": true,
                "user_text": ctx.text,
                "user_id": ctx.sender_id,
                "entity_id": config.entity_id,
                "shop_id": config.shop_id,
                "knowledge_cid": config.knowledge_base_cid,
                "language": config.language,
                "tone": format!("{:?}", config.tone),
                "max_length": config.max_reply_length,
                "auto_disclosure": config.auto_disclosure,
                "reply_to_message_id": ctx.message.telegram_update
                    .pointer("/message/message_id")
                    .and_then(|v| v.as_i64()),
            }),
            reason: "ai_customer_service".into(),
        })
    }
}

impl AiCustomerServiceRule {
    /// 启发式判断消息是否为提问
    fn looks_like_question(text: &str) -> bool {
        let text = text.trim();

        // 包含问号
        if text.contains('?') || text.contains('？') { return true; }

        // 中文疑问词
        let cn_keywords = ["怎么", "如何", "什么", "哪个", "哪里", "多少",
                          "能不能", "可以", "是否", "有没有", "支持",
                          "价格", "费用", "多久", "几个", "推荐"];
        for kw in &cn_keywords {
            if text.contains(kw) { return true; }
        }

        // 英文疑问词
        let en_keywords = ["how", "what", "where", "when", "which", "can",
                          "does", "is there", "price", "cost", "support"];
        let lower = text.to_lowercase();
        for kw in &en_keywords {
            if lower.contains(kw) { return true; }
        }

        // @ 提及 Bot（用户主动 @Bot 提问）
        if text.contains("@") { return true; }

        false
    }
}
```

#### 11.3.2 AI Handler（Leader 节点新增模块）

```rust
// nexus-node/src/ai_handler.rs（新建）

use crate::knowledge_base::KnowledgeBase;

/// AI 客服处理器 — 仅在 Leader 节点上运行
pub struct AiHandler {
    /// LLM 客户端（复用 node-operator 的 llm_client）
    llm: Box<dyn LlmClient>,
    /// 知识库管理器
    knowledge_manager: KnowledgeManager,
    /// 对话历史缓存（chat_id+user_id → 最近 N 轮对话）
    conversation_cache: DashMap<(i64, i64), VecDeque<ConversationTurn>>,
    /// 安全过滤器
    safety_filter: SafetyFilter,
}

/// 单轮对话
struct ConversationTurn {
    user_message: String,
    ai_reply: String,
    timestamp: DateTime<Utc>,
}

impl AiHandler {
    /// 处理 AI 客服请求
    pub async fn handle(&self, params: &serde_json::Value) -> Result<String> {
        let user_text = params["user_text"].as_str().unwrap_or("");
        let entity_id = params["entity_id"].as_u64().unwrap_or(0);
        let shop_id = params["shop_id"].as_u64().unwrap_or(0);
        let knowledge_cid = params["knowledge_cid"].as_str();
        let language = params["language"].as_str().unwrap_or("auto");
        let tone = params["tone"].as_str().unwrap_or("Friendly");
        let max_length = params["max_length"].as_u64().unwrap_or(500);
        let auto_disclosure = params["auto_disclosure"].as_bool().unwrap_or(true);
        let user_id = params["user_id"].as_i64().unwrap_or(0);
        let chat_id = params.get("chat_id")
            .and_then(|v| v.as_i64()).unwrap_or(0);

        // 1. 加载知识库上下文 (RAG)
        let knowledge_context = self.retrieve_knowledge(
            entity_id, shop_id, knowledge_cid, user_text
        ).await?;

        // 2. 获取对话历史（最近 5 轮）
        let history = self.get_conversation_history(chat_id, user_id, 5);

        // 3. 构造 LLM 消息
        let messages = self.build_messages(
            user_text, &knowledge_context, &history,
            language, tone, max_length, auto_disclosure,
        );

        // 4. 调用 LLM
        let response = self.llm.chat(messages, None).await?;

        // 5. 安全过滤
        let filtered = self.safety_filter.filter(&response.content)?;

        // 6. 缓存对话
        self.cache_conversation(chat_id, user_id, user_text, &filtered);

        Ok(filtered)
    }

    /// RAG 知识检索
    async fn retrieve_knowledge(
        &self,
        entity_id: u64,
        shop_id: u64,
        knowledge_cid: Option<&str>,
        query: &str,
    ) -> Result<String> {
        let mut context_parts = Vec::new();

        // 层级 1: 链上商品目录（实时）
        let products = self.knowledge_manager
            .get_product_catalog(shop_id).await?;
        if !products.is_empty() {
            context_parts.push(format!("## 商品目录\n{}", products));
        }

        // 层级 2: IPFS 知识库文档（FAQ/产品详情）
        if let Some(cid) = knowledge_cid {
            let docs = self.knowledge_manager
                .search_ipfs_knowledge(cid, query, 5).await?;
            if !docs.is_empty() {
                context_parts.push(format!("## 相关知识\n{}", docs));
            }
        }

        // 层级 3: 历史 QA 对（从成功回答中学习）
        let similar_qa = self.knowledge_manager
            .search_similar_qa(entity_id, query, 3).await?;
        if !similar_qa.is_empty() {
            context_parts.push(format!("## 历史成功回答\n{}", similar_qa));
        }

        Ok(context_parts.join("\n\n"))
    }

    /// 构造 LLM 消息
    fn build_messages(
        &self,
        user_text: &str,
        knowledge: &str,
        history: &[ConversationTurn],
        language: &str,
        tone: &str,
        max_length: u64,
        auto_disclosure: bool,
    ) -> Vec<Message> {
        let disclosure = if auto_disclosure {
            "\n- 在回复末尾简短声明: '（我是 AI 客服，如需人工帮助请 @管理员）'"
        } else { "" };

        let lang_instruction = match language {
            "zh" => "用中文回复。",
            "en" => "Reply in English.",
            _ => "用用户提问的语言回复。",
        };

        let tone_instruction = match tone {
            "Professional" => "保持正式、专业的语气。",
            "Friendly" => "保持亲切友好的语气，像朋友一样交谈。",
            "Casual" => "用轻松随意的语气。",
            _ => "保持友好的语气。",
        };

        let system_prompt = format!(
            "你是一个商家的 AI 客服助手。\n\
            \n\
            ## 规则\n\
            - 只根据以下【知识库】中的信息回答问题\n\
            - 如果知识库中没有相关信息，回复：'抱歉，这个问题我暂时无法回答，请联系管理员'\n\
            - 不要编造任何商品信息（价格/功能/库存）\n\
            - 不要讨论政治、宗教、色情等敏感话题\n\
            - 回复长度不超过 {} 字符\n\
            - {}\n\
            - {}{}\n\
            \n\
            ## 知识库\n\
            {}\n",
            max_length, lang_instruction, tone_instruction, disclosure, knowledge
        );

        let mut messages = vec![Message::system(system_prompt)];

        // 历史对话
        for turn in history {
            messages.push(Message::user(&turn.user_message));
            messages.push(Message::assistant(&turn.ai_reply));
        }

        // 当前问题
        messages.push(Message::user(user_text));

        messages
    }
}
```

#### 11.3.3 知识库管理器

```rust
// nexus-node/src/knowledge_base.rs（新建）

/// 知识库管理器
///
/// 三层知识来源:
/// 1. 链上商品目录 (pallet-entity-service) — 实时
/// 2. IPFS 文档 (pallet-storage-service pin) — Entity Owner 上传
/// 3. 历史 QA 缓存 — 从成功客服对话中自动积累
pub struct KnowledgeManager {
    /// subxt 链客户端（读取链上商品数据）
    chain_client: Arc<ChainClient>,
    /// IPFS 网关
    ipfs_gateway: String,
    /// 文本向量化（用于相似度搜索）
    embedder: Option<Box<dyn TextEmbedder>>,
    /// 向量数据库（本地 sled + 向量索引）
    vector_store: VectorStore,
    /// 商品目录缓存
    product_cache: DashMap<u64, (Vec<ProductSummary>, Instant)>,
}

/// 商品摘要（从链上 pallet-entity-service 读取）
#[derive(Debug, Clone)]
pub struct ProductSummary {
    pub id: u64,
    pub name: String,          // 从 IPFS name_cid 解析
    pub price: String,         // 格式化价格
    pub status: String,        // "在售" / "下架"
    pub stock: String,         // "有货" / "缺货" / "N件"
    pub category: String,
    pub shop_url: String,      // 商品链接（含 referral）
}

/// 向量存储（轻量级本地实现）
pub struct VectorStore {
    db: sled::Db,
    /// entity_id → Vec<(embedding, text_chunk, metadata)>
    index: DashMap<u64, Vec<VectorEntry>>,
}

struct VectorEntry {
    embedding: Vec<f32>,       // 文本向量
    text: String,              // 原始文本块
    source: KnowledgeSource,   // 来源标记
    score: f32,                // 相似度分数（查询时填充）
}

enum KnowledgeSource {
    ProductCatalog,            // 链上商品
    IpfsDocument { cid: String, section: String },
    HistoricalQA { question: String, answer: String },
}

impl KnowledgeManager {
    /// 从链上读取商品目录 → 格式化为文本
    pub async fn get_product_catalog(&self, shop_id: u64) -> Result<String> {
        // 缓存 5 分钟
        if let Some(cached) = self.product_cache.get(&shop_id) {
            if cached.1.elapsed() < Duration::from_secs(300) {
                return Ok(Self::format_products(&cached.0));
            }
        }

        // 通过 subxt 读取 pallet-entity-service::Products 存储
        // Product { id, name_cid, price, stock, status, category }
        let products = self.chain_client
            .query_products_by_shop(shop_id).await?;

        // 解析 IPFS CID → 商品名称
        let mut summaries = Vec::new();
        for product in &products {
            let name = self.resolve_ipfs_text(&product.name_cid).await
                .unwrap_or_else(|_| format!("商品#{}", product.id));

            summaries.push(ProductSummary {
                id: product.id,
                name,
                price: format!("{} COS", product.price),
                status: if product.status == "Active" { "在售" } else { "下架" }.into(),
                stock: if product.stock == 0 { "不限量" }
                       else { &format!("{}件", product.stock) }.into(),
                category: product.category.clone(),
                shop_url: format!("https://shop.example/{}/product/{}?ref=ai",
                                  shop_id, product.id),
            });
        }

        self.product_cache.insert(shop_id, (summaries.clone(), Instant::now()));
        Ok(Self::format_products(&summaries))
    }

    fn format_products(products: &[ProductSummary]) -> String {
        products.iter()
            .filter(|p| p.status == "在售")
            .map(|p| format!(
                "- 【{}】{} | 库存: {} | 链接: {}",
                p.name, p.price, p.stock, p.shop_url
            ))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 搜索 IPFS 知识库文档（向量相似度）
    pub async fn search_ipfs_knowledge(
        &self, cid: &str, query: &str, top_k: usize
    ) -> Result<String> {
        // 1. 确保文档已下载并索引
        self.ensure_indexed(cid).await?;

        // 2. 向量化查询
        let query_embedding = self.embed_text(query).await?;

        // 3. 相似度搜索 top_k
        let results = self.vector_store.search(cid, &query_embedding, top_k);

        Ok(results.iter()
            .map(|r| r.text.clone())
            .collect::<Vec<_>>()
            .join("\n---\n"))
    }
}
```

#### 11.3.4 安全过滤器

```rust
// nexus-node/src/safety_filter.rs（新建）

/// AI 回复安全过滤器
pub struct SafetyFilter {
    /// 禁止词列表
    forbidden_words: Vec<String>,
    /// 价格校验（防止 LLM 幻觉编造价格）
    price_validator: Option<PriceValidator>,
}

impl SafetyFilter {
    pub fn filter(&self, reply: &str) -> Result<String> {
        let mut filtered = reply.to_string();

        // 1. 禁止词过滤
        for word in &self.forbidden_words {
            if filtered.to_lowercase().contains(&word.to_lowercase()) {
                return Ok("抱歉，这个问题我暂时无法回答，请联系管理员。".into());
            }
        }

        // 2. 长度截断
        if filtered.chars().count() > 2000 {
            filtered = filtered.chars().take(2000).collect();
            filtered.push_str("...");
        }

        // 3. 外部链接过滤（只保留自家 shop 链接）
        // 防止 LLM 生成竞品链接
        filtered = Self::filter_external_links(&filtered);

        Ok(filtered)
    }

    fn filter_external_links(text: &str) -> String {
        // 保留 shop.example 域名链接，移除其他外部链接
        // 实际实现用正则替换
        text.to_string()
    }
}
```

#### 11.3.5 对话历史管理

```rust
// nexus-node/src/conversation_cache.rs（新建）

/// 对话缓存 — 维护每个用户的最近对话上下文
///
/// 存储: 内存 DashMap + 定时持久化到 sled
/// Key: (chat_id, user_id)
/// Value: VecDeque<ConversationTurn>, 最多保留 10 轮
/// TTL: 30 分钟无新消息自动清除
pub struct ConversationCache {
    cache: DashMap<(i64, i64), ConversationSession>,
    db: sled::Db,
}

struct ConversationSession {
    turns: VecDeque<ConversationTurn>,
    last_active: Instant,
    total_turns: u64,
}

impl ConversationCache {
    /// 获取历史对话（最近 N 轮）
    pub fn get_history(&self, chat_id: i64, user_id: i64, n: usize)
        -> Vec<ConversationTurn>
    {
        self.cache.get(&(chat_id, user_id))
            .map(|session| {
                session.turns.iter()
                    .rev().take(n).rev()
                    .cloned().collect()
            })
            .unwrap_or_default()
    }

    /// 记录新一轮对话
    pub fn record(&self, chat_id: i64, user_id: i64,
                  user_msg: &str, ai_reply: &str)
    {
        self.cache.entry((chat_id, user_id))
            .and_modify(|session| {
                session.turns.push_back(ConversationTurn {
                    user_message: user_msg.to_string(),
                    ai_reply: ai_reply.to_string(),
                    timestamp: Utc::now(),
                });
                if session.turns.len() > 10 {
                    session.turns.pop_front();
                }
                session.last_active = Instant::now();
                session.total_turns += 1;
            })
            .or_insert_with(|| ConversationSession {
                turns: {
                    let mut vd = VecDeque::new();
                    vd.push_back(ConversationTurn {
                        user_message: user_msg.to_string(),
                        ai_reply: ai_reply.to_string(),
                        timestamp: Utc::now(),
                    });
                    vd
                },
                last_active: Instant::now(),
                total_turns: 1,
            });
    }

    /// 定时 GC: 清除 30 分钟未活跃的会话
    pub fn gc(&self) {
        let threshold = Instant::now() - Duration::from_secs(1800);
        self.cache.retain(|_, session| session.last_active > threshold);
    }
}
```

### 11.4 知识库三层架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Entity 知识库三层架构                      │
│                                                             │
│  ┌─────────────────────────────────────────────────┐        │
│  │  Layer 1: 链上商品目录（实时、结构化）            │        │
│  │                                                 │        │
│  │  来源: pallet-entity-service::Products          │        │
│  │  内容: 商品名/价格/库存/状态/类别               │        │
│  │  更新: 每 5 分钟刷新缓存                        │        │
│  │  优先级: 最高 — 价格等核心信息以链上为准         │        │
│  │                                                 │        │
│  │  Product {                                      │        │
│  │    id: 42, name_cid: "Qm...", price: 999,      │        │
│  │    stock: 100, status: Active,                  │        │
│  │    category: Digital                            │        │
│  │  }                                              │        │
│  └─────────────────────────────────────────────────┘        │
│                          │                                   │
│  ┌─────────────────────────────────────────────────┐        │
│  │  Layer 2: IPFS 知识文档（半静态、非结构化）      │        │
│  │                                                 │        │
│  │  来源: Entity Owner 上传到 IPFS，CID 注册链上   │        │
│  │  内容:                                          │        │
│  │  ├── FAQ.md — 常见问题解答                      │        │
│  │  ├── product_details.md — 详细产品说明          │        │
│  │  ├── shipping_policy.md — 配送政策              │        │
│  │  ├── refund_policy.md — 退款政策                │        │
│  │  └── custom_prompts.md — 自定义回复模板         │        │
│  │                                                 │        │
│  │  索引方式: 文本分块 → 向量化 → 本地向量存储     │        │
│  │  更新: Owner 更新 CID 时自动重新索引            │        │
│  └─────────────────────────────────────────────────┘        │
│                          │                                   │
│  ┌─────────────────────────────────────────────────┐        │
│  │  Layer 3: 历史 QA 对（动态、持续积累）           │        │
│  │                                                 │        │
│  │  来源: AI 成功回复 + 管理员手动确认/纠正         │        │
│  │  积累方式:                                       │        │
│  │  ├── AI 回复后管理员未删除 → 视为正确，加入库    │        │
│  │  ├── 管理员回复同一问题 → 覆盖 AI 答案          │        │
│  │  └── /ai_correct 命令 → 手动添加 QA 对          │        │
│  │                                                 │        │
│  │  存储: 本地 sled + 向量索引                     │        │
│  │  检索: 用户新问题 → 向量相似度 → 匹配历史 QA   │        │
│  └─────────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

### 11.5 LLM 调用策略

#### 11.5.1 三级模型策略（成本优化）

| 级别 | 模型 | 用途 | 每次成本 | 延迟 |
|------|------|------|---------|------|
| **L1 快速** | DeepSeek-Chat | 简单 FAQ（知识库命中率高） | ~$0.001 | ~0.5s |
| **L2 标准** | Claude 3.5 Haiku | 复杂问题 + 多轮对话 | ~$0.005 | ~1.5s |
| **L3 高级** | Claude 3.5 Sonnet | 需要推理/比较/建议的问题 | ~$0.02 | ~3s |

**路由逻辑**：

```
用户提问
    ↓
知识库检索命中率 > 0.9 → L1 DeepSeek（直接格式化答案）
    ↓ 否
问题长度 < 50 字 且 无对话历史 → L2 Haiku
    ↓ 否
多轮对话 或 比较类问题 → L3 Sonnet
```

#### 11.5.2 LLM 客户端复用

```
node-operator/src/llm_client.rs 已有:
├── LlmClient trait (async fn chat)
├── ClaudeClient (claude-3-5-sonnet)
├── OpenAIClient (gpt-4-turbo)
├── DeepSeekClient (deepseek-chat，支持自定义 base_url)
└── create_llm_client() 工厂函数

复用方式:
├── 抽取 llm_client.rs 为共享 crate: cosmos-llm-client
├── nexus-node 依赖 cosmos-llm-client
└── 三级模型同时初始化，按路由选择
```

#### 11.5.3 月度成本预估

| 群规模 | 日均提问数 | 模型分布 (L1/L2/L3) | 月成本 |
|--------|----------|---------------------|--------|
| 100 人群 | ~10 | 60/30/10 | ~$1.5/月 |
| 500 人群 | ~30 | 50/35/15 | ~$6/月 |
| 2000 人群 | ~80 | 45/35/20 | ~$20/月 |
| 10000 人群 | ~200 | 40/35/25 | ~$60/月 |

### 11.6 链上存储与配置

#### 11.6.1 pallet-bot-group-mgmt 扩展

```rust
// 扩展 GroupRules 结构体，新增 AI 客服配置字段

/// 群规则（扩展后）
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct GroupRules<MaxCidLen: Get<u32>> {
    // === 现有字段 ===
    pub join_policy: JoinApprovalPolicy,
    pub rate_limit_per_minute: u16,
    pub auto_mute_duration: u64,
    pub filter_links: bool,
    pub restrict_mentions: bool,

    // === 新增: AI 客服配置 ===
    /// 是否启用 AI 客服
    pub ai_service_enabled: bool,
    /// 关联的 Shop ID
    pub ai_shop_id: u64,
    /// 知识库文档 IPFS CID
    pub ai_knowledge_cid: BoundedVec<u8, MaxCidLen>,
    /// 回复语言 (0=auto, 1=zh, 2=en)
    pub ai_language: u8,
    /// 回复风格 (0=Professional, 1=Friendly, 2=Casual)
    pub ai_tone: u8,
    /// 最大回复长度
    pub ai_max_reply_length: u16,
    /// 是否自动声明 AI 身份
    pub ai_auto_disclosure: bool,
    /// 每小时最大回复数
    pub ai_max_replies_per_hour: u16,
}
```

#### 11.6.2 ActionType 扩展

```rust
// pallet-bot-group-mgmt 新增 ActionType 变体

pub enum ActionType {
    // === 现有 ===
    SendMessage, DeleteMessage, BanUser, MuteUser,
    UnmuteUser, PinMessage, ApproveChatJoinRequest, DeclineChatJoinRequest,
    // === 新增 ===
    AiReply,           // AI 客服回复
    AiKnowledgeUpdate, // 知识库更新
}
```

### 11.7 群主/管理员操作命令

| 命令 | 功能 | 说明 |
|------|------|------|
| `/ai_on` | 启用 AI 客服 | 需管理员权限 |
| `/ai_off` | 关闭 AI 客服 | 立即停止 AI 回复 |
| `/ai_status` | 查看状态 | 知识库大小/今日回复数/成本/准确率 |
| `/ai_tone <风格>` | 设置回复风格 | professional / friendly / casual |
| `/ai_lang <语言>` | 设置回复语言 | auto / zh / en |
| `/ai_correct <问题> \| <正确答案>` | 手动添加 QA | 补充/纠正知识库 |
| `/ai_delete <问题关键词>` | 删除 QA 对 | 移除错误的历史回答 |
| `/ai_knowledge` | 查看知识库 | 列出已索引的文档和 QA 数量 |
| `/ai_test <问题>` | 测试 AI 回复 | 仅管理员可见回复（不发群） |
| `/ai_limit <N>` | 设置每小时上限 | 频率控制 |

### 11.8 效果追踪与自动学习

#### 11.8.1 效果指标

```
每次 AI 回复记录:
├── 回复延迟（从收到消息到发出回复）
├── 用户后续行为:
│   ├── 继续追问 → 上一轮可能不够好
│   ├── 点击商品链接 → 有效引导
│   ├── 无后续消息 → 可能已解决
│   └── 管理员删除 AI 回复 → 回答错误
├── 管理员反馈:
│   ├── 未删除 → 正确（弱信号）
│   ├── 删除 → 错误（强信号）
│   └── 自己回答了同一问题 → 更好的答案（最强信号）
└── 商品转化:
    └── 用户点击 referral 链接 → 下单 → 归因到 AI 客服
```

#### 11.8.2 自动学习循环

```
AI 回复 → 管理员未删除 → 弱正样本，写入历史 QA
    ↓
AI 回复 → 管理员删除 → 负样本，标记为"错误回答"
    ↓
用户提问 → 管理员自己回复 → 强正样本:
  1. 提取管理员回复作为"标准答案"
  2. 关联用户的原始问题
  3. 写入历史 QA，优先级最高
  4. 下次类似问题优先使用管理员答案
    ↓
/ai_correct 命令 → 人工标注正样本，最高优先级
```

### 11.9 与 Commission 系统集成

```
AI 客服回复中的商品链接:
  https://shop.example/{shop_id}/product/{product_id}?ref=ai_cs_{entity_id}

用户点击 → 进入 Shop → 下单
    ↓
pallet-commission-core::process_order()
    ↓
referral_code = "ai_cs_{entity_id}"
    → 识别为 AI 客服引流
    → 佣金归属: Entity 运营账户（而非个人推荐人）
    → 可选: AI 客服产生的佣金用于抵扣 LLM API 成本
```

### 11.10 实施里程碑

> 以下里程碑对应主路线图（Section 7）的 **Phase 1 + Phase 2**。Milestone 1~2 = Phase 1，Milestone 3~4 = Phase 2。

#### Milestone 1: 最小可用（3 周）

- [ ] 抽取 `cosmos-llm-client` 共享 crate
- [ ] 实现 `AiCustomerServiceRule`（规则引擎新规则）
- [ ] 实现 `AiHandler`（Leader 节点 LLM 调用）
- [ ] 实现 `KnowledgeManager` Layer 1（链上商品目录）
- [ ] 实现 `ConversationCache`（内存对话缓存）
- [ ] 实现 `SafetyFilter`（基础安全过滤）
- [ ] `/ai_on`、`/ai_off` 命令

**验证标准**: AI 能根据链上商品目录回答价格/库存问题，准确率 > 80%

#### Milestone 2: 知识库完善（2 周）

- [ ] 实现 `KnowledgeManager` Layer 2（IPFS 文档索引 + 向量搜索）
- [ ] 文本向量化（DeepSeek Embedding / 本地模型）
- [ ] `pallet-bot-group-mgmt` 扩展 AI 配置字段
- [ ] Entity Owner 上传知识文档流程
- [ ] `/ai_knowledge`、`/ai_test` 命令

**验证标准**: AI 能根据 IPFS FAQ 文档回答产品详情问题

#### Milestone 3: 自动学习（2 周）

- [ ] 实现 `KnowledgeManager` Layer 3（历史 QA 积累）
- [ ] 管理员行为追踪（删除/自己回复 → 反馈信号）
- [ ] `/ai_correct`、`/ai_delete` 命令
- [ ] 三级模型路由策略
- [ ] 效果指标面板 `/ai_status`

**验证标准**: AI 回答在一周内通过学习明显提升准确率（> 90%）

#### Milestone 4: 商业闭环（2 周）

- [ ] 商品链接自动附带 referral code
- [ ] Commission 系统识别 AI 客服引流
- [ ] LLM 成本追踪 + 与广告/佣金收入对比
- [ ] AI 客服 ROI Dashboard

### 11.11 双重角色分析：店铺产品客服 × 群客服

#### 11.11.1 问题本质

一个 Entity 通常有多个 Telegram 群，每个群的定位和客服需求截然不同：

| 群类型 | 典型定位 | 客服需求 | 示例问题 |
|--------|---------|---------|---------|
| **售前咨询群** | 新客引流 | 产品介绍、价格对比、购买引导 | "套餐 A 和 B 有什么区别？" |
| **售后服务群** | 已购用户 | 使用教程、故障排查、退款政策 | "账号登不上去怎么办？" |
| **VIP 会员群** | 高价值用户 | 专属优惠、新品预告、优先支持 | "有没有老客续费折扣？" |
| **社区讨论群** | 用户交流 | 话题引导、活动通知、氛围维护 | "下次抽奖活动什么时候？" |
| **代理/分销群** | B 端合作 | 佣金政策、素材分发、培训 | "二级代理分成比例是多少？" |
| **技术支持群** | 开发者 | API 文档、接入指南、Bug 反馈 | "Webhook 回调格式是什么？" |

**核心矛盾**：同一个 Bot（`nexus-agent`）+ 同一套 LLM 调用链路，需要在不同群里扮演不同的"角色"，回答完全不同领域的问题。

#### 11.11.2 可行性分析

##### ✅ 技术上完全可行

**理由 1：群级配置已天然支持**

11.3.1 节设计的 `AiServiceConfig` 是**按 chat_id 存储**的，每个群独立配置：

```
enabled_groups: DashMap<i64 /* chat_id */, AiServiceConfig>
```

只需扩展 `AiServiceConfig`，从"一个配置控一切"变为"一个角色定义 + 对应知识域"：

```rust
pub struct AiServiceConfig {
    // === 现有字段（不变）===
    pub entity_id: u64,
    pub shop_id: u64,
    pub knowledge_base_cid: Option<String>,
    pub language: String,
    pub tone: AiTone,
    pub max_reply_length: u16,
    pub auto_disclosure: bool,
    pub cooldown_seconds: u32,
    pub max_replies_per_hour: u16,

    // === 新增：群角色定义 ===
    /// 群客服角色类型
    pub role: GroupCsRole,
    /// 角色自定义系统提示词（IPFS CID，可选覆盖默认）
    pub custom_system_prompt_cid: Option<String>,
    /// 知识域范围（控制 RAG 检索范围）
    pub knowledge_scope: KnowledgeScope,
    /// 是否允许推荐商品（售后群可能不需要）
    pub allow_product_recommendation: bool,
    /// 是否需要追踪转化（社区群可能不需要 referral）
    pub track_conversion: bool,
}

/// 群客服角色
pub enum GroupCsRole {
    /// 售前客服 — 侧重产品介绍、价格对比、购买引导
    PreSales,
    /// 售后客服 — 侧重使用帮助、故障排查、退款流程
    AfterSales,
    /// VIP 专属 — 专属优惠、优先响应、新品预告
    VipSupport,
    /// 社区管理 — 话题引导、活动通知、氛围维护
    Community,
    /// 代理支持 — 佣金政策、素材分发、培训
    AgentSupport,
    /// 技术支持 — API 文档、接入指南
    TechSupport,
    /// 通用 — 全功能，不限定角色
    General,
}

/// 知识域范围
pub struct KnowledgeScope {
    /// 是否包含链上商品目录（Layer 1）
    pub include_product_catalog: bool,
    /// 是否包含 IPFS 知识文档（Layer 2）
    pub include_ipfs_docs: bool,
    /// 指定 IPFS 文档子集（空 = 全部）
    pub ipfs_doc_tags: Vec<String>,   // e.g. ["after_sales", "refund"]
    /// 是否包含历史 QA（Layer 3）
    pub include_historical_qa: bool,
    /// 历史 QA 来源限制（空 = 所有群的 QA）
    pub qa_source_chat_ids: Vec<i64>, // 空 = 不限制
}
```

**理由 2：LLM 的角色切换零成本**

不同群的 AI 客服本质上只是 **system prompt 不同** + **RAG 知识域不同**：

```
售前咨询群 system prompt:
  "你是 [Entity名] 的售前顾问。重点介绍产品优势、对比差异、引导用户下单。
   遇到售后问题请引导到售后群..."

售后服务群 system prompt:
  "你是 [Entity名] 的售后工程师。帮助用户解决使用问题、处理退款咨询。
   不推荐新产品。遇到购买需求引导到售前群..."

社区群 system prompt:
  "你是 [Entity名] 社区助手。回答社区活动、规则相关问题。
   商品问题简短回答后引导到客服群..."
```

LLM 天然支持通过 system prompt 切换角色，**无需多个模型实例、无需额外训练**。

**理由 3：现有架构无需大改**

消息处理全流程（11.2 节的 6 步）完全不变，只需在 **Step 5 Leader 执行 AI 推理** 时，根据 `chat_id` 查到的 `AiServiceConfig` 来：
1. 选择对应的 system prompt 模板（或自定义 prompt）
2. 控制 RAG 知识检索范围
3. 决定是否附加商品链接

##### ✅ 产品上合理且必要

**理由 1：一个 Bot 多角色 ≠ 一锅端**

```
错误理解: 一个 Bot 在所有群里回答一样的内容
正确理解: 一个 Bot 实例 × 多个群级配置 = 每个群独立的 AI 客服角色

类比:
├── 一个人类客服团队 → 售前组、售后组、VIP 组
├── 同一个公司品牌 → 不同部门的服务标准
└── 同一个 Bot Token → 不同群里不同行为
     这在 Telegram Bot 中完全正常，群主控制 Bot 在该群的行为
```

**理由 2：降低群主管理成本**

如果每个群需要不同的 Bot → 群主要注册多个 Bot → 管理多套 Token/Agent → 不现实。

```
方案 A（不可行）: 每种角色一个 Bot
  售前咨询群 → @entity_presales_bot → nexus-agent-1
  售后服务群 → @entity_aftersales_bot → nexus-agent-2
  社区群     → @entity_community_bot → nexus-agent-3
  ❌ 群主需管理 3 个 Bot，注册 3 次，运行 3 个 Agent

方案 B（推荐）: 一个 Bot + 群级角色配置
  售前咨询群 → @entity_bot (role=PreSales)
  售后服务群 → @entity_bot (role=AfterSales)
  社区群     → @entity_bot (role=Community)
  ✅ 群主只管一个 Bot，通过 /ai_role 命令按群设角色
```

**理由 3：知识库自然分层覆盖**

```
                    Entity 级知识（所有群共享）
                    ├── 品牌介绍
                    ├── 联系方式
                    └── 通用政策
                         │
        ┌────────────────┼────────────────┐
        ↓                ↓                ↓
  售前群知识           售后群知识        社区群知识
  ├── 商品目录         ├── 使用教程      ├── 活动日历
  ├── 价格表           ├── 故障排查 FAQ  ├── 社区规则
  ├── 优惠活动         ├── 退款流程      ├── 抽奖规则
  └── 竞品对比         └── 升级指南      └── 贡献者名单
```

三层知识库（11.4 节）天然支持这种分层：
- **Layer 1 链上商品** → 售前群全量展示，售后群仅展示已购商品，社区群不展示
- **Layer 2 IPFS 文档** → 通过 `ipfs_doc_tags` 过滤，每个群只加载对应标签的文档
- **Layer 3 历史 QA** → 通过 `qa_source_chat_ids` 控制，售后群只学习售后群的历史 QA

#### 11.11.3 关键设计：角色驱动的 Prompt 构造

```rust
// ai_handler.rs 扩展：根据群角色构造不同 system prompt

impl AiHandler {
    fn build_role_prompt(&self, config: &AiServiceConfig) -> String {
        let entity_name = self.get_entity_name(config.entity_id);

        let role_instruction = match &config.role {
            GroupCsRole::PreSales => format!(
                "你是【{}】的售前顾问。\n\
                ## 你的职责\n\
                - 详细介绍产品功能和优势\n\
                - 对比不同套餐/方案的区别\n\
                - 推荐最适合用户需求的产品\n\
                - 引导用户通过链接下单\n\
                ## 边界\n\
                - 售后问题简短安慰后引导到售后群\n\
                - 不讨论竞品的负面信息\n\
                - 价格以知识库为准，不编造优惠",
                entity_name
            ),
            GroupCsRole::AfterSales => format!(
                "你是【{}】的售后工程师。\n\
                ## 你的职责\n\
                - 帮助用户解决产品使用问题\n\
                - 提供故障排查步骤（按知识库流程）\n\
                - 解答退款/换货政策\n\
                - 无法解决时引导联系人工客服\n\
                ## 边界\n\
                - 不推荐新产品或升级（除非用户主动问）\n\
                - 不承诺知识库未列出的退款条件\n\
                - 涉及账号安全问题必须转人工",
                entity_name
            ),
            GroupCsRole::VipSupport => format!(
                "你是【{}】的 VIP 专属顾问。\n\
                ## 你的职责\n\
                - 优先、详细地回答所有问题\n\
                - 主动告知专属优惠和新品预告\n\
                - 处理售前+售后全流程\n\
                ## 风格\n\
                - 尊称用户，语气尊贵感\n\
                - 回复更详细（长度上限可放宽）",
                entity_name
            ),
            GroupCsRole::Community => format!(
                "你是【{}】社区助手。\n\
                ## 你的职责\n\
                - 回答社区规则、活动信息\n\
                - 引导新成员了解社区\n\
                - 维护良好讨论氛围\n\
                ## 边界\n\
                - 商品问题简短回答后引导到客服群\n\
                - 不参与争论，保持中立\n\
                - 鼓励用户互助讨论",
                entity_name
            ),
            GroupCsRole::AgentSupport => format!(
                "你是【{}】的代理支持助手。\n\
                ## 你的职责\n\
                - 解答佣金政策和分成规则\n\
                - 提供推广素材和话术\n\
                - 解答代理注册和升级流程\n\
                ## 边界\n\
                - 具体佣金数字以链上数据为准\n\
                - 涉及结算问题转人工财务",
                entity_name
            ),
            GroupCsRole::TechSupport => format!(
                "你是【{}】的技术支持工程师。\n\
                ## 你的职责\n\
                - 提供 API 文档和接入指南\n\
                - 帮助排查技术问题\n\
                - 解答开发者常见问题\n\
                ## 风格\n\
                - 使用技术术语，给出代码示例\n\
                - 提供链接到详细文档",
                entity_name
            ),
            GroupCsRole::General => format!(
                "你是【{}】的 AI 客服助手。\n\
                根据用户问题类型灵活回答，涵盖产品、售后、社区等全方面。",
                entity_name
            ),
        };

        role_instruction
    }
}
```

#### 11.11.4 知识域隔离机制

```
┌─────────────────────────────────────────────────────────────────────┐
│                 知识检索流程（按群角色过滤）                          │
│                                                                     │
│  用户提问: "怎么申请退款？"                                         │
│  当前群角色: AfterSales                                             │
│       │                                                             │
│       ↓                                                             │
│  ┌─────────────────────────────────────────────────┐                │
│  │ Layer 1: 链上商品目录                            │                │
│  │                                                 │                │
│  │ config.knowledge_scope.include_product_catalog   │                │
│  │   = true (售后群也需要知道用户买了什么)          │                │
│  │   但 format 不同:                               │                │
│  │   售前群: "【VPN套餐A】$9.99/月... → 购买链接"  │                │
│  │   售后群: "【VPN套餐A】有效期/续费/使用说明"    │                │
│  └───────────────────────┬─────────────────────────┘                │
│                          ↓                                          │
│  ┌─────────────────────────────────────────────────┐                │
│  │ Layer 2: IPFS 文档（标签过滤）                   │                │
│  │                                                 │                │
│  │ config.knowledge_scope.ipfs_doc_tags             │                │
│  │   = ["after_sales", "refund", "troubleshoot"]   │                │
│  │                                                 │                │
│  │ 只检索带以上标签的文档:                          │                │
│  │   ✅ refund_policy.md (tag: after_sales, refund)│                │
│  │   ✅ troubleshoot.md (tag: troubleshoot)        │                │
│  │   ❌ pricing.md (tag: pre_sales) — 被过滤       │                │
│  │   ❌ promo_events.md (tag: community) — 被过滤  │                │
│  └───────────────────────┬─────────────────────────┘                │
│                          ↓                                          │
│  ┌─────────────────────────────────────────────────┐                │
│  │ Layer 3: 历史 QA（来源过滤）                     │                │
│  │                                                 │                │
│  │ config.knowledge_scope.qa_source_chat_ids        │                │
│  │   = [-1001234567] (只用本群的历史 QA)           │                │
│  │                                                 │                │
│  │ 只检索来自售后群的历史成功回答:                  │                │
│  │   ✅ "怎么退款" → "请在设置-订单中申请..."      │                │
│  │   ❌ "多少钱" → "$9.99/月" (来自售前群) — 过滤  │                │
│  └─────────────────────────────────────────────────┘                │
│                                                                     │
│  最终 RAG 上下文 = Layer1(售后视角) + Layer2(售后标签) + Layer3(本群) │
└─────────────────────────────────────────────────────────────────────┘
```

#### 11.11.5 IPFS 知识文档的标签系统

Entity Owner 上传知识文档时，需为每个文档打标签。文档 manifest 格式：

```json
{
  "entity_id": 42,
  "version": "2026-02-07",
  "documents": [
    {
      "cid": "QmXyz...",
      "title": "产品详情与价格",
      "tags": ["pre_sales", "pricing"],
      "language": "zh"
    },
    {
      "cid": "QmAbc...",
      "title": "使用教程",
      "tags": ["after_sales", "tutorial"],
      "language": "zh"
    },
    {
      "cid": "QmDef...",
      "title": "退款政策",
      "tags": ["after_sales", "refund"],
      "language": "zh"
    },
    {
      "cid": "QmGhi...",
      "title": "社区活动日历",
      "tags": ["community", "events"],
      "language": "zh"
    },
    {
      "cid": "QmJkl...",
      "title": "代理佣金政策",
      "tags": ["agent", "commission"],
      "language": "zh"
    },
    {
      "cid": "QmMno...",
      "title": "API 接入文档",
      "tags": ["tech", "api"],
      "language": "en"
    }
  ]
}
```

整个 manifest 上传到 IPFS，其 CID 注册到链上 `GroupRules.ai_knowledge_cid`。

**标签与角色的默认映射**（群主可覆盖）：

| 角色 | 默认加载的文档标签 |
|------|-------------------|
| PreSales | `pre_sales`, `pricing`, `promo` |
| AfterSales | `after_sales`, `refund`, `tutorial`, `troubleshoot` |
| VipSupport | 全部（VIP 享受完整知识库） |
| Community | `community`, `events`, `rules` |
| AgentSupport | `agent`, `commission`, `training` |
| TechSupport | `tech`, `api`, `sdk` |
| General | 全部 |

#### 11.11.6 群间知识共享与隔离的平衡

```
问: 售后群的 AI 学到了一个好的退款流程回答，售前群能用吗？
答: 取决于配置。

策略 A: 严格隔离（默认）
  每个群只学习自己群的 QA
  优点: 角色不串戏
  缺点: 知识积累慢

策略 B: Entity 级共享 + 标签过滤（推荐）
  所有群的 QA 共存于 Entity 知识池
  每条 QA 自动打标签（来源群角色 + 问题类型）
  检索时按当前群角色过滤标签
  优点: 知识积累快，相关 QA 可跨群复用
  缺点: 需要准确的自动标签

策略 C: 手动共享
  群主通过 /ai_share <qa_id> <target_group> 手动跨群共享 QA
  优点: 完全可控
  缺点: 管理成本高
```

**推荐策略 B**，实现方式：

```rust
/// 历史 QA 记录
struct HistoricalQA {
    question: String,
    answer: String,
    source_chat_id: i64,
    source_role: GroupCsRole,       // 来源群角色
    auto_tags: Vec<String>,         // 自动识别的标签
    confidence: f32,                // 置信度
    created_at: DateTime<Utc>,
    verified: bool,                 // 管理员确认
}

/// 检索时过滤逻辑
fn filter_qa(qa: &HistoricalQA, current_role: &GroupCsRole) -> bool {
    // 1. 同角色的 QA → 直接可用
    if &qa.source_role == current_role { return true; }

    // 2. 通用 QA（高置信度 + 已验证）→ 可跨角色
    if qa.verified && qa.confidence > 0.95 { return true; }

    // 3. 相关角色的 QA（如 VIP 群可用售前+售后的 QA）
    match current_role {
        GroupCsRole::VipSupport => true,  // VIP 看所有
        GroupCsRole::General => true,     // 通用看所有
        GroupCsRole::PreSales => qa.auto_tags.iter()
            .any(|t| ["pricing", "product", "promo"].contains(&t.as_str())),
        GroupCsRole::AfterSales => qa.auto_tags.iter()
            .any(|t| ["refund", "tutorial", "troubleshoot"].contains(&t.as_str())),
        _ => false,
    }
}
```

#### 11.11.7 群主操作流程

```
群主拉 Bot 进群后:

1. /ai_on                          → 启用 AI 客服（默认 General 角色）
2. /ai_role presales               → 设置为售前客服
3. 上传知识文档到 IPFS（带标签）
4. /ai_knowledge set QmManifest... → 绑定知识文档 manifest
5. /ai_test "套餐 A 多少钱？"      → 测试 AI 回复
6. 满意后正式启用

换一个群:
1. /ai_on
2. /ai_role aftersales             → 设置为售后客服
3. /ai_knowledge set QmManifest... → 绑定同一个 manifest（自动按标签过滤）
4. /ai_test "怎么退款？"           → 测试
```

新增命令：

| 命令 | 功能 | 说明 |
|------|------|------|
| `/ai_role <角色>` | 设置群角色 | presales / aftersales / vip / community / agent / tech / general |
| `/ai_role` | 查看当前角色 | 显示角色 + 关联知识标签 |
| `/ai_scope` | 查看知识范围 | 显示当前群加载了哪些文档 |
| `/ai_scope add <tag>` | 扩展知识范围 | 手动添加文档标签 |
| `/ai_scope remove <tag>` | 缩小知识范围 | 移除不需要的文档标签 |

#### 11.11.8 合理性论证

##### 为什么不做成"一个全能 AI"？

```
方案: 不区分角色，让 AI 自己判断该怎么回答
问题:
  1. 上下文污染 — 售后群里 AI 推荐新产品 → 用户反感
  2. 知识膨胀 — 所有文档都塞进 RAG → 检索精度下降
  3. 人格分裂 — 同一个问题在不同群回答风格不一致
  4. 成本浪费 — 每次都检索全量知识 → 更多 token → 更贵

角色隔离方案:
  1. 明确边界 — 售后群只回答售后问题，越界自动引导
  2. 精准检索 — 只搜索相关文档 → 命中率高 → L1 模型即可
  3. 一致体验 — 每个群有固定人设，用户有明确预期
  4. 成本优化 — 知识域小 → prompt 短 → 便宜
```

##### 投入产出比

| 改动项 | 工作量 | 价值 |
|--------|--------|------|
| `AiServiceConfig` 增加 role + scope 字段 | 0.5 天 | 支撑所有角色定义 |
| 角色 prompt 模板（7 种） | 1 天 | 每个群有专属 AI 人设 |
| 知识域过滤逻辑 | 1 天 | RAG 精准度提升 30%+ |
| IPFS 文档标签系统 | 1 天 | Entity Owner 管理知识结构 |
| 历史 QA 跨群共享过滤 | 1 天 | 知识积累速度 ×3 |
| `/ai_role` `/ai_scope` 命令 | 0.5 天 | 群主自助配置 |
| **合计** | **5 天** | **每个群变成专业客服** |

仅增加约 **1 周开发量**（嵌入 Milestone 1），即可将"通用 AI 客服"升级为"专业角色化 AI 客服"。

#### 11.11.9 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| 群主不会配置角色 | 中 | 默认 General 也能用 | 提供一键模板 + 创建群时自动推荐角色 |
| 角色边界不够清晰，AI 越界回答 | 中 | 体验不一致 | system prompt 明确写 "超出范围→引导到对应群" |
| 知识文档无标签 | 高 | 无法按角色过滤 | 无标签时退化为全量检索（= 不开标签功能） |
| 多群 QA 自动标签不准 | 中 | 知识串群 | 默认严格隔离，跨群共享需管理员验证 |

#### 11.11.10 结论

**可行性**: ✅ **完全可行**
- 技术上只需扩展 `AiServiceConfig` + prompt 模板 + 知识域过滤，架构无需变更
- LLM 天然支持角色切换，零额外推理成本
- 现有三层知识库（链上/IPFS/历史QA）通过标签过滤即可实现知识域隔离

**合理性**: ✅ **非常合理且必要**
- 一个 Bot Token + 群级角色配置 >>> 多个 Bot 分别管理
- 角色隔离提升 RAG 精准度和用户体验
- 仅增加 ~5 天开发量，ROI 极高
- 与竞品（Intercom/Zendesk 多团队工作区）理念一致，但完全去中心化

**建议**: 将角色系统作为 **Milestone 1 的核心特性**（而非后续迭代），因为从第一天就让群主选择角色，比后续再改默认行为的迁移成本低得多。

### 11.12 调用区块链 AI 项目大模型 — 可行性分析

#### 11.12.1 问题

11.5 节设计的 LLM 调用策略依赖中心化 API（Claude / OpenAI / DeepSeek）。作为一个去中心化项目，Nexus 能否调用其他**区块链 AI 项目**提供的去中心化大模型推理服务？

#### 11.12.2 当前区块链 AI 推理项目全景

| 项目 | 代币 | 核心能力 | 推理 API 方式 | 成熟度 |
|------|------|---------|-------------|--------|
| **Bittensor** | TAO | 去中心化"神经互联网"，Subnet 架构 | 通过 Subnet Validator 转发请求到 Miner 节点 | ⭐⭐⭐⭐ 生产可用 |
| **Ritual** | — | AI 执行层，Infernet 协议 | 智能合约触发 → Infernet Node 执行推理 → 结果上链 | ⭐⭐⭐ 主网已上线 |
| **Hyperbolic** | — | 去中心化 GPU 云 + 推理服务 | OpenAI 兼容 REST API（最易集成） | ⭐⭐⭐⭐ 生产可用 |
| **Cuckoo AI** | CAI | 全栈去中心化 AI 平台 | Coordinator Node API → Miner 执行 → 结果返回 | ⭐⭐ 早期 |
| **Gensyn** | — | 去中心化训练协议 | 专注训练，**不提供推理 API** | ❌ 不适用 |
| **Akash Network** | AKT | 去中心化云计算 | 租用 GPU → 自己部署模型（IaaS 层） | ⭐⭐⭐ 需自运维 |
| **io.net** | IO | GPU 聚合层 | 租用 GPU 集群，自己部署推理服务 | ⭐⭐⭐ 需自运维 |
| **SingularityNET** | AGIX→ASI | AI 服务市场 | gRPC/REST API 调用市场中的 AI 服务 | ⭐⭐ 服务质量参差 |

#### 11.12.3 可集成性分析

##### 方案 A：Hyperbolic — 最佳首选（OpenAI 兼容 API）

**为什么是首选**：Hyperbolic 提供与 OpenAI **完全兼容** 的 REST API，意味着现有 `LlmClient` trait 几乎零改动即可对接。

```rust
// 现有 OpenAIClient 改动量 ≈ 0
// 只需将 base_url 从 "https://api.openai.com" 改为 "https://api.hyperbolic.xyz"

pub enum LlmProvider {
    Claude,
    OpenAI,
    DeepSeek,
    Hyperbolic,   // 新增 — OpenAI 兼容，复用 OpenAIClient
}

impl LlmProvider {
    pub fn from_env() -> Result<Self> {
        let provider = std::env::var("LLM_PROVIDER")
            .unwrap_or_else(|_| "claude".to_string());
        match provider.to_lowercase().as_str() {
            "claude" | "anthropic" => Ok(Self::Claude),
            "openai" | "gpt" => Ok(Self::OpenAI),
            "deepseek" => Ok(Self::DeepSeek),
            "hyperbolic" => Ok(Self::Hyperbolic), // 新增
            _ => Err(anyhow!("Unknown LLM provider: {}", provider)),
        }
    }
}

// HyperbolicClient 实现 — 直接复用 OpenAI 客户端逻辑
pub struct HyperbolicClient {
    inner: OpenAIClient, // 复用，仅 base_url 不同
}

impl HyperbolicClient {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("HYPERBOLIC_API_KEY")?;
        Ok(Self {
            inner: OpenAIClient::with_base_url(
                api_key,
                "https://api.hyperbolic.xyz/v1".to_string(),
            ),
        })
    }
}

#[async_trait]
impl LlmClient for HyperbolicClient {
    async fn chat(&self, messages: Vec<Message>, tools: Option<Vec<ToolDef>>) -> Result<LlmResponse> {
        self.inner.chat(messages, tools).await
    }
}
```

**优势**：
- **零协议适配** — OpenAI 兼容 API，无需学习新 SDK
- **模型丰富** — 支持 Llama 3.1 405B、Mixtral、DeepSeek 等开源模型
- **价格优势** — 去中心化 GPU 供给，比 OpenAI 便宜 50%~80%
- **去中心化叙事** — Nexus 使用去中心化 AI 基础设施，品牌契合

**劣势**：
- 延迟可能比中心化 API 高 200-500ms（可接受，客服场景非实时）
- 服务质量 SLA 不如 OpenAI/Anthropic 成熟
- 模型质量上限受限于开源模型（暂无 GPT-4o / Claude 3.5 级别）

##### 方案 B：Bittensor Subnet — 最"Web3 原生"

Bittensor 基于 Substrate 构建（与 Cosmos 同为 Substrate 生态），技术栈最亲近。

**调用方式**：

```
Nexus Leader Node
    │
    ↓ (HTTP/WebSocket)
Bittensor Validator Node (Subnet 1: Text Prompting)
    │
    ↓ (Bittensor Axon Protocol)
Bittensor Miner (运行 LLM 模型)
    │
    ↓
返回推理结果
```

**集成方案**：

```rust
pub struct BittensorClient {
    /// Bittensor Validator 的 Axon endpoint
    validator_endpoint: String,
    /// 调用者的 hotkey（Bittensor 身份）
    hotkey: String,
    http_client: reqwest::Client,
}

#[async_trait]
impl LlmClient for BittensorClient {
    async fn chat(&self, messages: Vec<Message>, _tools: Option<Vec<ToolDef>>) -> Result<LlmResponse> {
        // Bittensor 的 prompting subnet 接受标准消息格式
        let payload = serde_json::json!({
            "messages": messages.iter().map(|m| {
                serde_json::json!({"role": &m.role, "content": &m.content})
            }).collect::<Vec<_>>(),
            "timeout": 30,
        });

        let resp = self.http_client
            .post(&format!("{}/chat", self.validator_endpoint))
            .header("Authorization", format!("Bearer {}", self.hotkey))
            .json(&payload)
            .send()
            .await?;

        let body: serde_json::Value = resp.json().await?;
        Ok(LlmResponse {
            content: body["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            tool_calls: vec![],
        })
    }
}
```

**优势**：
- **Substrate 亲缘** — Bittensor 的 Subtensor 链基于 Substrate，可考虑未来 XCM 跨链
- **Yuma 共识** — Miner 竞争出最优回答，质量有去中心化保障
- **TAO 代币经济** — 可考虑 COS↔TAO 的跨链支付
- **Subnet 专业化** — 可创建/使用专门的"客服 AI" Subnet

**劣势**：
- 需要运行或委托 Bittensor Validator 节点（入门门槛高）
- 延迟较高（Miner 竞争 + Validator 评分 → 3-10 秒）
- API 非标准化，需定制适配
- Miner 质量参差不齐

##### 方案 C：Ritual Infernet — 最"可验证"

Ritual 的核心价值是**可验证推理**（Verifiable Inference），每次 LLM 调用都可以产生密码学证明。

```
Nexus Leader Node
    │
    ↓ (Infernet SDK / REST)
Ritual Infernet Node
    │ (执行推理 + 生成 ZK/TEE 证明)
    ↓
返回: { result: "...", proof: "0x..." }
```

**对 Nexus 的潜在价值**（远期可选增强）：

```
当前 Nexus 信任模型:
  Leader Node 执行 AI 推理 → 返回结果 → 结果质量由市场竞争保障
  (Entity Owner 通过用户评价和切换 Node 来淘汰劣质服务)

如果未来需要更强信任保障 (Ritual 集成):
  Leader Node 调用 Infernet → 返回结果 + ZK Proof
  Leader 广播: { ai_reply: "...", ritual_proof: "0x..." }
  其他节点: 验证 proof → 确认 AI 确实处理了这个问题
  → 可选的信任增强层，非必须
```

**优势**：
- **可验证推理** — 可选的信任增强层，密码学证明 AI 确实处理了请求
- **链上可组合** — 智能合约可直接触发 AI 推理
- 支持 Claude / GPT / 开源模型

**劣势**：
- ZK 证明生成开销大 → 每次调用成本 +30%~100%
- 客服场景是否需要可验证？（过度工程的风险）
- 项目较新，生态还在早期

##### 方案 D：Akash / io.net — 自托管开源模型

不调用别人的 API，而是在去中心化 GPU 市场上**租用 GPU 自己部署模型**。

```
Nexus Node Operator
    │ (租用 Akash/io.net GPU)
    ↓
自己部署的 Llama 3.1 70B / DeepSeek V3 实例
    │ (vLLM / TGI 推理框架)
    ↓
完全私有的 OpenAI 兼容 API
```

**优势**：
- **完全自主** — 不依赖任何第三方 API
- **数据隐私** — 推理数据不离开自己的 GPU
- **成本可控** — Akash GPU 价格约为 AWS 的 30%
- **模型可定制** — 可微调专属客服模型

**劣势**：
- 运维复杂度高（模型部署、更新、扩容）
- 需要持续的 GPU 租赁支出（即使没有请求）
- 小规模时不经济（GPU 利用率低）

#### 11.12.4 推荐策略：分层混合架构

```
                    Nexus AI 客服 LLM 调用策略
                    
    ┌──────────────────────────────────────────────────────┐
    │                   LlmRouter                          │
    │   根据请求特征 + 成本 + 延迟要求 自动路由            │
    └───────┬───────────┬──────────────┬──────────────┬────┘
            │           │              │              │
            ↓           ↓              ↓              ↓
    ┌───────────┐ ┌──────────┐ ┌────────────┐ ┌───────────┐
    │ 中心化 API│ │Hyperbolic│ │ Bittensor  │ │  Ritual   │
    │ Claude    │ │(OpenAI   │ │ (Subnet)   │ │(可验证)   │
    │ OpenAI    │ │ 兼容)    │ │            │ │           │
    │ DeepSeek  │ │          │ │            │ │           │
    └───────────┘ └──────────┘ └────────────┘ └───────────┘
    Phase 1 默认    Phase 2      Phase 3       Phase 4
    成熟稳定       成本优化     Web3 叙事      可验证推理
```

**Phase 1（MVP）**: 仅中心化 API（当前已设计，DeepSeek + Haiku + Sonnet 三级）
**Phase 2（成本优化）**: 加入 Hyperbolic 作为 L1 模型的替代
  - 简单问题 → Hyperbolic（Llama 70B，便宜 80%）
  - 复杂问题 → Claude Haiku / Sonnet（质量保证）
**Phase 3（Web3 叙事）**: 接入 Bittensor Prompting Subnet
  - 面向 Web3 社区的营销亮点："100% 去中心化 AI 客服"
  - TAO 代币支付推理费用 → 跨链 Token 经济故事
**Phase 4（可验证）**: 对关键场景使用 Ritual 可验证推理
  - 涉及价格/政策的回答 → Ritual proof → 链上存证
  - 争议解决时可验证 AI 确实给出了某回答

#### 11.12.5 LlmRouter 实现

```rust
/// LLM 路由器 — 根据策略自动选择 provider
pub struct LlmRouter {
    /// 可用的 LLM 客户端（按优先级排序）
    clients: Vec<(LlmProvider, Box<dyn LlmClient>)>,
    /// 路由策略
    strategy: RoutingStrategy,
    /// 各 provider 的实时健康状态
    health: DashMap<LlmProvider, ProviderHealth>,
}

pub enum RoutingStrategy {
    /// 固定优先级（按配置顺序尝试，失败 fallback）
    Priority,
    /// 成本最优（估算 token 数 → 选最便宜的能处理的 provider）
    CostOptimal,
    /// 质量最优（复杂问题用强模型，简单问题用弱模型）
    QualityTiered {
        /// 问题复杂度评分阈值
        complexity_threshold: f32,
    },
    /// 混合（默认）: 简单→去中心化，复杂→中心化，关键→可验证
    Hybrid,
}

struct ProviderHealth {
    /// 最近 100 次调用的成功率
    success_rate: f32,
    /// 平均延迟 (ms)
    avg_latency_ms: u64,
    /// 最后一次成功时间
    last_success: DateTime<Utc>,
    /// 连续失败次数（用于熔断）
    consecutive_failures: u32,
}

impl LlmRouter {
    pub async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolDef>>,
        hint: &RequestHint,
    ) -> Result<LlmResponse> {
        let ordered_providers = match &self.strategy {
            RoutingStrategy::Priority => self.priority_order(),
            RoutingStrategy::CostOptimal => self.cost_order(&messages),
            RoutingStrategy::QualityTiered { complexity_threshold } => {
                let complexity = Self::estimate_complexity(&messages);
                if complexity > *complexity_threshold {
                    // 复杂问题 → 中心化强模型优先
                    self.quality_order()
                } else {
                    // 简单问题 → 去中心化便宜模型优先
                    self.cost_order(&messages)
                }
            }
            RoutingStrategy::Hybrid => {
                if hint.requires_verification {
                    // 关键场景 → Ritual 优先
                    self.verifiable_order()
                } else if hint.is_simple_qa {
                    // 简单 QA → Hyperbolic/Bittensor
                    self.decentralized_order()
                } else {
                    // 默认 → 质量优先
                    self.quality_order()
                }
            }
        };

        // 按顺序尝试，支持自动 fallback
        for (provider, client) in ordered_providers {
            if self.is_circuit_broken(&provider) {
                continue; // 跳过已熔断的 provider
            }
            match client.chat(messages.clone(), tools.clone()).await {
                Ok(resp) => {
                    self.record_success(&provider);
                    return Ok(resp);
                }
                Err(e) => {
                    self.record_failure(&provider, &e);
                    tracing::warn!("Provider {:?} failed: {}, trying next", provider, e);
                    continue;
                }
            }
        }
        Err(anyhow!("All LLM providers failed"))
    }

    /// 估算问题复杂度 (0.0 ~ 1.0)
    fn estimate_complexity(messages: &[Message]) -> f32 {
        let last_msg = messages.last().map(|m| &m.content).unwrap_or(&String::new());
        let mut score = 0.0f32;

        // 长问题通常更复杂
        if last_msg.len() > 200 { score += 0.2; }
        if last_msg.len() > 500 { score += 0.2; }

        // 多轮对话更复杂
        let turn_count = messages.iter().filter(|m| m.role == "user").count();
        if turn_count > 3 { score += 0.2; }

        // 包含技术关键词
        let tech_keywords = ["API", "对接", "报错", "代码", "配置", "架构"];
        if tech_keywords.iter().any(|k| last_msg.contains(k)) {
            score += 0.2;
        }

        // 包含比较/分析关键词
        let analysis_keywords = ["对比", "区别", "优缺点", "建议", "方案"];
        if analysis_keywords.iter().any(|k| last_msg.contains(k)) {
            score += 0.2;
        }

        score.min(1.0)
    }
}
```

#### 11.12.6 跨链代币支付

使用去中心化 AI 服务需要用对应代币支付。Nexus 如何处理？

```
方案 A: Node Operator 自行管理多币种（推荐初期）
  - Node Operator 在各链上持有 TAO / CAI 等代币
  - 推理成本由 Operator 承担，通过 Entity 服务费回收
  - 简单直接，无跨链复杂度

方案 B: 链上代理支付（中期）
  - Entity Owner 用 COS 支付 AI 服务费
  - Nexus 链上 Oracle 获取 COS/TAO 汇率
  - 自动 swap → 支付给 Bittensor/Hyperbolic
  - 需要跨链桥或 DEX 集成

方案 C: IBC 跨链结算（远期）
  - 如果 Bittensor 支持 IBC（Substrate → Cosmos IBC）
  - COS ↔ TAO 原生跨链转账 + 推理费结算
  - 最优雅，但依赖生态互通
```

**推荐路径**：Phase 1-2 用方案 A（Node Operator 自理），Phase 3+ 探索方案 B。

#### 11.12.7 与现有架构的对接点

```
现有架构                        去中心化 AI 对接点
─────────                      ──────────────────
node-operator/
  src/llm_client.rs
    LlmClient trait  ←────────  新增 HyperbolicClient
    LlmProvider enum ←────────  新增 Hyperbolic/Bittensor/Ritual 枚举值
    (新增) LlmRouter ←────────  智能路由 + fallback + 熔断

nexus-node/
  src/leader.rs
    AiHandler ←───────────────  调用 LlmRouter 而非单个 client
    (新增) RequestHint ←──────  标注请求特征（简单/复杂/需验证）

pallet-bot-group-mgmt/
  GroupRules.ai_config ←──────  新增 preferred_provider 字段
                                群主可指定优先使用哪个 AI provider
```

#### 11.12.8 风险与缓解

| 风险 | 严重度 | 缓解措施 |
|------|--------|---------|
| 去中心化 AI 延迟过高（>5s） | 中 | LlmRouter 熔断机制 + 自动 fallback 到中心化 API |
| 去中心化 AI 回答质量不稳定 | 高 | SafetyFilter 已有（11.3.4 节），质量不过关则丢弃并重试其他 provider |
| Bittensor Miner 返回恶意内容 | 中 | SafetyFilter + 链上记录，如连续低质量则自动屏蔽该 provider |
| 代币价格波动导致成本不可控 | 中 | 设置每日/每月成本上限，超出则降级到免费模型 |
| 去中心化 AI 项目停摆/不维护 | 低 | 多 provider 架构，任一停摆不影响服务 |
| API 格式变动 | 低 | LlmClient trait 抽象已隔离，适配层变更不影响业务 |

#### 11.12.9 结论

**能否调用？** ✅ **完全可以，且现有 `LlmClient` trait 架构天然支持**

```
集成难度排名（从易到难）:

1. Hyperbolic  ★☆☆☆☆  OpenAI 兼容 API，改 base_url 即可
2. Akash 自托管 ★★★☆☆  需部署模型，但 API 自己定义（OpenAI 兼容）
3. Bittensor   ★★★☆☆  需适配 Axon 协议，消息格式需转换
4. Cuckoo AI   ★★★☆☆  需适配 Coordinator API
5. Ritual      ★★★★☆  需集成 Infernet SDK + proof 验证逻辑
6. SingularityNET ★★★★☆  gRPC 适配 + 服务质量不确定
```

**是否应该做？** ✅ **应该，但分阶段**

```
Phase 1 (MVP):     中心化 API only          → 保证质量和速度
Phase 2 (+2 周):   + Hyperbolic              → 降本 50%+，1 天集成
Phase 3 (+4 周):   + Bittensor Subnet        → Web3 叙事，3 天集成
Phase 4 (远期):    + Ritual 可验证推理        → 信任升级，5 天集成
```

**核心价值**：

1. **成本降低** — Hyperbolic 开源模型价格是 OpenAI 的 1/5
2. **品牌契合** — "去中心化商业平台 + 去中心化 AI" = 纯正 Web3 叙事
3. **抗审查** — 不依赖单一 AI 供应商，避免 API Key 被封风险
4. **生态联动** — COS 与 TAO / CAI 的代币经济互通可能性
5. **信任增强** — Ritual 可验证推理提供可选的密码学信任保障

**建议**：Phase 2 首先集成 Hyperbolic（1 天工作量），作为 L1 廉价模型的替代。这既能立刻降低成本，又能在市场推广中宣传"去中心化 AI 基础设施"。

### 11.13 Nexus Agent 群主本地代理 — 自保管 Key 的合理性与可行性分析

#### 11.13.1 问题定义

Nexus 架构的核心设计是：**每个群主自己运行 `nexus-agent`，在本地保管所有敏感 Key**。这与传统 SaaS 模式（平台方统一托管所有 Bot）形成鲜明对比。

```
传统 SaaS 模式（中心化）:              Nexus 模式（群主自保管）:

  群主A ──→ ┌──────────────┐            群主A ──→ 自己的 Agent A（本地/VPS）
  群主B ──→ │  平台服务器    │            群主B ──→ 自己的 Agent B（本地/VPS）
  群主C ──→ │ 托管所有Bot   │            群主C ──→ 自己的 Agent C（本地/VPS）
             │              │                        │        │        │
             │ 所有BOT_TOKEN │                        ↓        ↓        ↓
             │ 集中在这里！   │               nexus-node 去中心化共识网络
             └──────────────┘
             
  风险: 单点泄露 = 全部沦陷       风险: 每个 Agent 独立，互不影响
```

**核心问题**：群主自己跑 Agent、自己保管 Key，这在技术上可行吗？在产品上合理吗？

#### 11.13.2 群主需要保管哪些 Key

基于 `nexus-agent/src/config.rs` 和 `nexus-agent/src/signer.rs` 的实际实现：

| Key | 来源 | 敏感度 | 泄露后果 | 保管位置 |
|-----|------|--------|---------|---------|
| **BOT_TOKEN** | Telegram @BotFather 创建时获得 | 🔴 极高 | 攻击者可完全控制 Bot：发消息、踢人、读历史 | `.env` 环境变量 |
| **Ed25519 私钥** | `nexus-agent` 首次启动自动生成 | 🔴 极高 | 可伪造签名消息，欺骗共识网络 | `$DATA_DIR/agent.key` (权限 600) |
| **WEBHOOK_SECRET** | 自动生成或手动设置 | 🟡 中等 | 可伪造 Telegram Webhook 请求 | `.env` 环境变量 |
| **LLM API Key** | （可选）群主自有的 Hyperbolic/OpenAI Key | 🟠 较高 | 攻击者消耗群主的 LLM 额度 | `.env` 环境变量或内存 |

```
nexus-agent 本地保管的 Key 清单:

$DATA_DIR/
├── agent.key          ← Ed25519 私钥 (32 bytes, chmod 600)
└── sequence.dat       ← 序列号 (防重放, 8 bytes)

.env:
├── BOT_TOKEN=xxx      ← Telegram Bot Token (永不上链)
├── WEBHOOK_SECRET=xxx ← Webhook 验证密钥
└── HYPERBOLIC_API_KEY ← (可选) LLM API Key
```

#### 11.13.3 可行性分析

##### A. 技术可行性 — ✅ 完全可行

**A1. 部署门槛已极低**

```bash
# 群主只需一条 Docker 命令：
docker run -d \
  --name my-nexus-agent \
  -p 8443:8443 \
  -v nexus-data:/data \
  -e BOT_TOKEN="123456:ABC-DEF..." \
  -e WEBHOOK_URL="https://my-server.com:8443" \
  -e NODES="node1@http://n1:8080,node2@http://n2:8080,node3@http://n3:8080" \
  nexus-agent

# 首次启动自动完成:
# ✅ 生成 Ed25519 密钥对 → /data/agent.key
# ✅ 初始化序列号 → /data/sequence.dat
# ✅ 注册 Telegram Webhook (3 次重试)
# ✅ 输出公钥 → 群主复制到链上注册
```

对比行业标准：
- **Telegram Bot** 本身就设计为群主自己创建、自己持有 Token
- **Matterbridge / Heisenbridge** 等开源桥接也是用户自部署
- **Mastodon 实例** 的运营模式更重（数据库 + 存储），nexus-agent 只需要 <100MB 内存

**A2. Key 自动生成，群主无需懂密码学**

```rust
// nexus-agent/src/signer.rs — 首次启动自动生成
pub fn load_or_generate(data_dir: &str) -> anyhow::Result<Self> {
    let key_path = Path::new(data_dir).join("agent.key");
    if key_path.exists() {
        // 加载已有密钥
    } else {
        // 自动生成 Ed25519 密钥对
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        std::fs::write(&key_path, signing_key.as_bytes())?;
        // chmod 600
        #[cfg(unix)]
        std::fs::set_permissions(&key_path, Permissions::from_mode(0o600))?;
        
        warn!("⚠️  请将以上公钥注册到链上");
    }
}
```

群主只需要做的事：
1. 在 Telegram @BotFather 创建 Bot → 获得 `BOT_TOKEN`
2. 有一台公网可访问的服务器（VPS / 家庭宽带 + 端口映射 / Cloudflare Tunnel）
3. 运行 `docker run` 一条命令
4. 将输出的公钥提交到链上注册（`pallet-bot-registry::register_bot`）

**群主不需要**：理解 Ed25519 / SHA256 / 密钥派生 / 签名算法

**A3. 运维成本极低**

```
nexus-agent 资源消耗:

CPU:    < 1% (仅在收到 Webhook 时活跃)
内存:   ~20MB (Rust 编译，无 GC)
磁盘:   ~15MB (二进制) + 40 bytes (数据文件)
带宽:   与群消息量成正比，通常 < 1MB/天
```

最低配置 VPS：1 核 512MB（$2.5~5/月）即可运行，甚至可以跑在树莓派上。

**A4. 离线恢复简单**

```
备份:  只需备份 $DATA_DIR/ (32+8 = 40 bytes)
恢复:  docker run ... -v /backup/data:/data → 立即恢复身份
迁移:  scp -r /data new-server:/data → docker run → 完成
```

##### B. 安全可行性 — ✅ 优于中心化方案

**B1. 爆破半径最小化**

```
中心化 SaaS 被攻破:
  攻击者获得 1 台服务器 → 控制 所有 Bot (数千/数万群)
  
Nexus Agent 被攻破:
  攻击者获得群主A的 VPS → 控制 群主A的 1 个 Bot (1~几个群)
  群主B、C 完全不受影响
```

**B2. 已有的多层安全机制**

```
                    nexus-agent 安全层次

Layer 1: Telegram Webhook 验证
  ┌─────────────────────────────────────────────────┐
  │ X-Telegram-Bot-Api-Secret-Token == WEBHOOK_SECRET│
  │ → 只有 Telegram 服务器知道这个 secret            │
  └──────────────────────────┬──────────────────────┘
                             ↓
Layer 2: Ed25519 消息签名
  ┌─────────────────────────────────────────────────┐
  │ sign(pubkey + bot_id_hash + seq + ts + msg_hash)│
  │ → 私钥永不离开本地，签名不可伪造                 │
  └──────────────────────────┬──────────────────────┘
                             ↓
Layer 3: 序列号防重放
  ┌─────────────────────────────────────────────────┐
  │ sequence 原子递增 + 持久化                       │
  │ → nexus-node 有 ±10 容忍窗口 (SequenceTracker) │
  └──────────────────────────┬──────────────────────┘
                             ↓
Layer 4: nexus-node 4 层验证
  ┌─────────────────────────────────────────────────┐
  │ ① Ed25519 签名验证                              │
  │ ② Bot 活跃状态检查 (链上 pallet-bot-registry)   │
  │ ③ 公钥匹配检查 (签名公钥 == 链上注册的公钥)      │
  │ ④ 目标节点检查 (本节点是否在选定节点列表中)       │
  └──────────────────────────┬──────────────────────┘
                             ↓
Layer 5: 多节点共识
  ┌─────────────────────────────────────────────────┐
  │ M/K 节点共识 (M = ceil(K*2/3))                  │
  │ → 即使 Agent 被控制，伪造消息也需要骗过多数节点   │
  └─────────────────────────────────────────────────┘
```

**B3. BOT_TOKEN 泄露的影响被共识层限制**

即使 `BOT_TOKEN` 泄露：
- 攻击者可以直接调 Telegram API → 但这不经过共识验证
- 链上 `ActionLog` 不会有记录 → 其他 Node 不承认
- 攻击者无法获得 `agent.key` → 无法伪造签名消息欺骗共识网络
- **最坏情况**：攻击者直接操控 Bot 发消息/踢人（绕过共识）→ 但这与传统中心化方案下的风险完全相同

```
BOT_TOKEN 泄露时的攻击面对比:

                    中心化 SaaS        Nexus Agent
能操控 Bot？          ✅ 能               ✅ 能 (相同)
操控有链上记录？       N/A (无链)         ❌ 无 (因为不走共识)
影响其他群主？         ✅ 可能             ❌ 不可能
群主能自行换 Token？   ❌ 需联系平台       ✅ 自己去 @BotFather revoke
```

**B4. Ed25519 私钥泄露的风险与缓解**

```
agent.key 泄露 → 攻击者可以:
  1. 伪造 SignedMessage → 欺骗 nexus-node
  2. 触发错误的共识决策
  
缓解:
  1. 文件权限 600 (仅 owner 可读)
  2. Docker volume 隔离
  3. 发现泄露 → 链上 deactivate_bot → 立即失效
  4. 重新生成密钥 + 重新注册 → 5 分钟恢复
  
链上吊销流程 (pallet-bot-registry):
  ① 群主调用 deactivate_bot(bot_id) → Bot 立即停用
  ② 所有 Node 拒绝该 Bot 的签名消息
  ③ 群主生成新密钥 → register_bot() + update_bot_public_key()
  ④ 恢复服务
```

##### C. 运营可行性 — ✅ 对目标用户群完全可行

**C1. Nexus 的目标用户画像**

```
Nexus 的目标用户不是普通消费者，而是:
  
  ┌─────────────────────────────────────────────────┐
  │ Entity Owner（实体主） — 商家/社群运营者           │
  │                                                   │
  │ 特征:                                             │
  │ - 已在运营 Telegram 群（有技术基础）               │
  │ - 已有 Bot（从 @BotFather 创建过）                │
  │ - 多数已有 VPS（用于其他服务）                    │
  │ - 对数据主权有诉求（不想把 Token 交给第三方）      │
  │ - 有能力运行 docker run 命令                      │
  └─────────────────────────────────────────────────┘
```

**C2. 与行业标杆对比**

| 产品 | 模式 | 用户需要做什么 | 用户群 |
|------|------|--------------|--------|
| **Shopify** | SaaS 托管 | 注册 → 配置 → 使用 | 普通商家 |
| **WooCommerce** | 自部署 | 租 VPS → 装 WordPress → 装插件 | 技术商家 |
| **Matterbridge** | 自部署 | 编译 → 配置 → 运行 | 技术用户 |
| **Mastodon** | 自部署 | 租 VPS → docker-compose → 运维 | 社区运营者 |
| **Nexus Agent** | 自部署 | 租 VPS → docker run → 注册公钥 | Entity Owner |

Nexus Agent 的部署复杂度**低于 Mastodon、低于 WooCommerce**，与 Matterbridge 相当。目标用户完全有能力运营。

##### D. 经济可行性 — ✅ 成本可控

```
群主月度运营成本:

VPS (1核512MB):              $2.5 ~ $5/月
域名 + SSL (可选):           $0 (Cloudflare 免费)
LLM API (按量, 可选):        $0 ~ $5/月 (取决于群活跃度)
带宽:                        几乎为 0 (包含在 VPS 中)
────────────────────────────────────────
总计:                        $2.5 ~ $10/月

对比:
- 传统 SaaS 群管 Bot:        $10 ~ $50/月
- 自建 Bot 服务器:           $5 ~ $20/月 (无共识验证)
```

**群主同时获得**：
- 完全的数据主权（消息不经过第三方）
- 去中心化共识验证（动作可信）
- 链上佣金/商品管理
- AI 客服能力（通过 Node Operator 的 LLM）

#### 11.13.4 合理性分析

##### R1. 与 Telegram Bot 设计哲学一致

```
Telegram 的设计哲学:
  - Bot Token 由群主自己持有
  - Webhook URL 由群主自己的服务器接收
  - Telegram 不托管 Bot 逻辑

Nexus 的设计哲学:
  - BOT_TOKEN 由群主自己保管           ← 与 TG 一致
  - Webhook 由群主本地 Agent 接收       ← 与 TG 一致
  - 消息处理逻辑由去中心化共识网络执行  ← 在 TG 基础上增强
```

Nexus Agent 本质上就是**群主本来就需要运行的 Bot 服务器**，只是多了 Ed25519 签名 + 共识网络多播。群主的额外负担接近于零。

##### R2. "不信任平台方"是 Web3 核心价值

```
中心化风险:

  1. 平台方跑路 → 所有 Bot 失效
  2. 平台方被攻破 → 所有 Token 泄露
  3. 平台方审查 → 你的 Bot 被封
  4. 平台方涨价 → 你没有替代方案
  5. 平台方数据泄露 → 所有消息被第三方获取

Nexus 自保管:

  1. Nexus 团队跑路 → 你的 Agent 继续运行（只要 Node 在）
  2. 你的 Agent 被攻破 → 只影响你一个
  3. 无人可审查你的 Bot → 主权在你
  4. 成本透明（VPS + LLM）→ 不受单方定价
  5. 消息不经过平台方 → 隐私保护
```

##### R3. Key 分离设计合理

```
Nexus 的密钥分离架构:

  ┌──────────────────────────────────────────────────────────────────┐
  │                         角色分离                                  │
  │                                                                  │
  │  Entity Owner (群主):                                             │
  │    持有: BOT_TOKEN, agent.key (Ed25519)                          │
  │    职责: 接收消息 → 签名 → 转发给共识网络                          │
  │    不持有: 链上私钥 (钱包)、Node 运营权                            │
  │                                                                  │
  │  Node Operator (节点运营商):                                       │
  │    持有: node.key (Ed25519), LLM_API_KEY, Staking 资金            │
  │    职责: 共识验证 → Leader 执行 → 链上提交                         │
  │    不持有: BOT_TOKEN (不能直接操控任何 Bot)                        │
  │                                                                  │
  │  链上 Runtime:                                                    │
  │    持有: 所有人的公钥 (不含私钥)、群规则、佣金配置                  │
  │    不持有: 任何私钥、任何 Token、任何 API Key                      │
  └──────────────────────────────────────────────────────────────────┘
```

**关键洞察**：没有任何单一角色持有全部密钥。
- 群主有 BOT_TOKEN 但不能绕过共识执行动作（链上无记录）
- Node 有 LLM Key 但不能操控 Bot（没有 BOT_TOKEN）
- 链上有规则但没有任何私钥

这比中心化方案更安全，因为**攻击面被分散到多个独立实体**。

##### R4. 渐进式体验降级方案

对于确实不想自己运行 Agent 的群主，可以提供官方托管模式（保持架构一致性）：

```
                    Agent 运行模式光谱
                    
  ← 更自主 (Web3 原教旨)                    更便捷 (Web2 体验) →
  
  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
  │ 模式1:      │  │ 模式2:      │  │ 模式3:      │  │ 模式4:      │
  │ 自有服务器   │  │ VPS 一键部署│  │ 官方托管    │  │ 共享 Agent  │
  │             │  │             │  │             │  │             │
  │ 群主自己的   │  │ 官方提供    │  │ 官方服务器   │  │ 多群主共享  │
  │ 物理机/VPS  │  │ 一键部署脚本│  │ 运行群主的   │  │ 一个 Agent  │
  │             │  │ (Terraform) │  │ Agent 实例   │  │ (多租户)    │
  │ 主权: ⭐⭐⭐│  │ 主权: ⭐⭐⭐│  │ 主权: ⭐⭐  │  │ 主权: ⭐    │
  │ 门槛: 最高  │  │ 门槛: 低    │  │ 门槛: 零    │  │ 门槛: 零    │
  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘
       ↑                 ↑                                    ↑
    目标模式         推荐起步模式                         临时体验模式
```

```bash
# 模式 2: 官方一键部署脚本（推荐起步）
curl -sSL https://install.nexus.network | bash -s -- \
  --bot-token "123456:ABC-DEF..." \
  --domain "my-bot.nexus.network" \
  --auto-ssl

# 脚本自动完成:
# ✅ 安装 Docker
# ✅ 拉取 nexus-agent 镜像
# ✅ 配置 Caddy 反向代理 + 自动 HTTPS
# ✅ 启动 Agent
# ✅ 输出公钥 + 链上注册指引
```

#### 11.13.5 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| 群主 VPS 宕机 | 中 | Bot 离线，群消息不处理 | Docker restart=always + 健康检查告警 |
| BOT_TOKEN 泄露 | 低 | Bot 被恶意操控 | 发现后 @BotFather revoke + 重新绑定 |
| agent.key 泄露 | 极低 | 伪造签名欺骗共识 | 链上 deactivate_bot + 重新生成密钥 |
| 群主不会运维 | 中 | 无法恢复故障 | 一键部署脚本 + 官方托管备选 |
| VPS 被黑客入侵 | 低 | 所有本地 Key 泄露 | 最小权限 Docker + Key 文件 600 + 及时更新 |
| 群主放弃运营 | 中 | Bot 永久离线 | 链上 Entity 转让 + Agent 迁移流程 |

**最关键的缓解**：所有风险都是**单群主隔离**的。群主 A 的 Agent 出问题，群主 B、C 完全不受影响。这是对中心化方案最本质的改进。

#### 11.13.6 结论

**群主本地自保管 Key 既合理又可行**：

1. **技术可行** — 一条 `docker run` 命令，20MB 内存，$2.5/月 VPS，首次启动自动生成密钥
2. **安全可行** — 5 层安全机制（Webhook 验证 → Ed25519 签名 → 序列号防重放 → Node 4 层验证 → M/K 共识），爆破半径最小化为单个群主
3. **运营可行** — 目标用户是 Entity Owner（已有 TG 群运营经验），复杂度低于 Mastodon / WooCommerce
4. **经济可行** — 月成本 $2.5~10，低于传统 SaaS 群管方案
5. **哲学合理** — 与 Telegram Bot 设计一致，与 Web3 "Not Your Keys, Not Your Bot" 理念一致
6. **密钥分离** — 群主持有 Bot 身份 Key，Node 持有 AI 能力 Key，链上只有公钥，无单点故障

**一句话总结**：就像 Bitcoin 钱包让用户自保管私钥一样，Nexus Agent 让群主自保管 Bot Token —— 这不是负担，而是主权。

