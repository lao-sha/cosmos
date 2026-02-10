# Nexus Blockchain

Nexus 是一个基于 [Polkadot SDK (Substrate)](https://github.com/nicories/polkadot-sdk) 构建的 Layer-1 区块链，提供去中心化商业实体管理、P2P 交易、佣金分销、争议仲裁、以及去中心化社群机器人（Nexus Bot）等完整 Web3 商业基础设施。

## 系统架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Nexus 生态系统                               │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │                    Substrate 区块链 (L1)                       │  │
│  │  node/ (节点)  +  runtime/ (运行时, 40+ pallets)               │  │
│  │  Aura 出块 · GRANDPA 终局 · 6s 出块 · Wasm Runtime            │  │
│  └────────────────────────────────────────────────────────────────┘  │
│         │                    │                    │                   │
│  ┌──────┴──────┐   ┌────────┴────────┐   ┌──────┴──────────┐       │
│  │ Entity 商业  │   │  P2P 交易系统    │   │  Nexus Bot 系统  │       │
│  │ 15 pallets  │   │  7 pallets       │   │  3 pallets       │       │
│  │ 实体·店铺·  │   │  做市·定价·信用  │   │  + nexus-agent   │       │
│  │ 代币·治理·  │   │  P2P·TRC20验证   │   │  + nexus-node    │       │
│  │ 会员·佣金   │   │                  │   │  去中心化群管理   │       │
│  └─────────────┘   └─────────────────┘   └─────────────────┘       │
│         │                    │                                       │
│  ┌──────┴──────┐   ┌────────┴────────┐   ┌─────────────────┐       │
│  │ 争议解决     │   │  去中心化存储    │   │  Node Operator   │       │
│  │ 3 pallets   │   │  2 pallets       │   │  LLM 驱动的      │       │
│  │ 托管·证据·  │   │  IPFS 存储·     │   │  节点自动运维     │       │
│  │ 仲裁        │   │  生命周期管理    │   │                  │       │
│  └─────────────┘   └─────────────────┘   └─────────────────┘       │
└─────────────────────────────────────────────────────────────────────┘
```

## 核心功能

### Entity 商业系统 (15 个 Pallet)

完整的去中心化商业实体管理平台：

| 模块 | Pallet | 功能 |
|------|--------|------|
| **实体注册** | `entity-registry` | 实体创建、审核、暂停、关闭、转让 |
| **店铺管理** | `entity-shop` | 主店铺/子店铺、独立运营状态、统计级联 |
| **实体代币** | `entity-token` | 代币铸造、分红、锁仓、转账限制、白/黑名单 |
| **实体治理** | `entity-governance` | 提案、投票（时间加权）、委员会、否决权 |
| **会员体系** | `entity-member` | 多级会员、推荐链、自动升级、团队统计 |
| **佣金引擎** | `commission-core` + 4 插件 | 插件化分佣：推荐链、级差、单线、提现 |
| **服务管理** | `entity-service` | 服务/商品上架与管理 |
| **交易处理** | `entity-transaction` | 订单创建、支付、完成 |
| **评价系统** | `entity-review` | 订单评价、店铺评分聚合 |
| **内部市场** | `entity-market` | 实体代币 NXS/USDT 双通道交易、TWAP、熔断 |
| **信息披露** | `entity-disclosure` | 内幕人管理、交易窗口期控制 |
| **KYC** | `entity-kyc` | 多级身份认证 |
| **代币销售** | `entity-tokensale` | 多轮次 Token Sale、白名单、退款、提现 |

**佣金插件架构：**
- `commission-common` — 共享类型 + `CommissionPlugin` trait
- `commission-core` — 调度引擎、存储、提现（4 种模式）
- `commission-referral` — 5 种推荐链模式（直推/多级/固定/首单/复购）
- `commission-level-diff` — 级差佣金（全局 + 自定义等级）
- `commission-single-line` — 单线上下级佣金

### P2P 交易系统 (7 个 Pallet)

统一的 P2P 交易模块（合并 OTC 买单 + Swap 卖单）：

| Pallet | 功能 |
|--------|------|
| `trading-p2p` | 买单（法币→NXS）+ 卖单（NXS→法币），USDT 链上验证 |
| `trading-maker` | 做市商注册、审核、额度管理 |
| `trading-pricing` | CNY/USD 汇率预言机、价格保护 |
| `trading-credit` | 买方/卖方信用评分 |
| `trading-common` | 共享类型与接口 |
| `trading-trc20-verifier` | TRC20 USDT 链上交易验证（OCW） |

### Nexus Bot — 去中心化社群管理 (3 Pallet + 2 Binary)

去中心化的多平台社群机器人系统，核心流程：**Telegram Webhook → Agent 签名 → 多播 → Node 共识 → Leader 执行 → 上链审计**

| 组件 | 描述 |
|------|------|
| `pallet-bot-consensus` | 节点注册/质押/退出、消息确认、作恶检测/惩罚、Era 奖励（订阅+通胀） |
| `pallet-bot-registry` | Bot 注册、社区平台绑定（TG/Discord/Slack/Matrix/Farcaster）、用户身份绑定 |
| `pallet-bot-group-mgmt` | 群组规则配置、操作审计日志、加群策略 |
| **nexus-agent** | 群主本地代理 — Webhook 接收、Ed25519 签名、确定性多播、本地反垃圾处理、20 个 TG API 方法执行 |
| **nexus-node** | 项目节点 — 4 层验签、Gossip 共识（M/K=2/3）、规则引擎（25 条命令）、Leader 选举/执行、批量上链 |

**已支持的 Telegram API (20 个)：**
`sendMessage` `deleteMessage` `deleteMessages` `banChatMember` `unbanChatMember` `restrictChatMember` `setChatPermissions` `pinChatMessage` `unpinChatMessage` `approveChatJoinRequest` `declineChatJoinRequest` `promoteChatMember` `getChatMember` `getChatAdministrators` `getChat` `answerCallbackQuery` `editMessageText` `editMessageReplyMarkup` `sendPoll` `stopPoll`

### 争议解决 (3 个 Pallet)

| Pallet | 功能 |
|--------|------|
| `escrow` | 多方资金托管、条件释放 |
| `evidence` | 链上证据存证（关联 IPFS CID） |
| `arbitration` | 仲裁委员会裁决、部分退款/全额退款/释放 |

### 去中心化存储 (2 个 Pallet)

| Pallet | 功能 |
|--------|------|
| `storage-service` | IPFS 文件存储注册与检索 |
| `storage-lifecycle` | 存储生命周期管理、过期清理 |

### 链上治理

4 个委员会实例（基于 `pallet-collective` + `pallet-membership`）：
- **技术委员会** — 协议升级与技术决策
- **仲裁委员会** — 争议裁决
- **财务委员会** — 国库资金管理
- **内容委员会** — 内容审核

### Node Operator — LLM 驱动的节点运维

`node-operator/` — 独立 Rust 二进制，使用 LLM 实现自动化节点运维：
- 云服务器自动部署与管理
- SSH 远程运维
- 节点健康监控与自动修复
- 审批流程（人工确认关键操作）

## 项目结构

```
cosmos/
├── node/                        # Substrate 区块链节点
│   └── src/                     #   CLI、RPC、共识配置
├── runtime/                     # WASM 运行时 (40+ pallets)
│   └── src/
│       ├── lib.rs               #   区块参数、类型、pallet 注册
│       ├── configs/             #   各 pallet Config 实现
│       └── genesis_config_presets/ # 创世配置
├── pallets/                     # 自定义 Pallet 模块
│   ├── entity/                  #   Entity 商业系统 (15 pallets)
│   │   ├── common/              #     共享类型与 trait
│   │   ├── registry/            #     实体注册
│   │   ├── shop/                #     店铺管理
│   │   ├── token/               #     实体代币
│   │   ├── governance/          #     实体治理
│   │   ├── member/              #     会员体系
│   │   ├── commission/          #     佣金引擎 (core + 4 插件)
│   │   ├── service/             #     服务管理
│   │   ├── transaction/         #     交易处理
│   │   ├── market/              #     内部市场
│   │   ├── review/              #     评价系统
│   │   ├── disclosure/          #     信息披露
│   │   ├── kyc/                 #     KYC 认证
│   │   └── tokensale/           #     代币销售
│   ├── trading/                 #   P2P 交易系统 (7 pallets)
│   │   ├── common/              #     共享类型
│   │   ├── pricing/             #     汇率预言机
│   │   ├── credit/              #     信用评分
│   │   ├── maker/               #     做市商
│   │   ├── p2p/                 #     P2P 统一交易
│   │   └── trc20-verifier/      #     TRC20 验证 (OCW)
│   ├── dispute/                 #   争议解决 (3 pallets)
│   │   ├── escrow/              #     资金托管
│   │   ├── evidence/            #     证据存证
│   │   └── arbitration/         #     仲裁裁决
│   ├── storage/                 #   去中心化存储 (2 pallets)
│   │   ├── service/             #     IPFS 存储
│   │   └── lifecycle/           #     生命周期
│   └── nexus/                   #   Nexus Bot 链上 (3 pallets)
│       ├── bot-consensus/       #     节点共识与奖励
│       ├── bot-registry/        #     Bot 注册与绑定
│       └── bot-group-mgmt/      #     群组规则与审计
├── nexus-agent/                 # Nexus Bot Agent (独立二进制)
│   └── src/
│       ├── main.rs              #     入口、Axum 服务
│       ├── webhook.rs           #     Webhook 处理、配置更新
│       ├── executor.rs          #     Telegram API 执行 (20 方法)
│       ├── multicaster.rs       #     确定性多播
│       ├── signer.rs            #     Ed25519 签名管理
│       ├── local_processor.rs   #     本地反垃圾 (6 层检查链)
│       ├── local_store.rs       #     本地状态 (洪泛计数、缓存)
│       ├── group_config.rs      #     群组配置管理
│       └── rate_limiter.rs      #     滑动窗口限流
├── nexus-node/                  # Nexus Bot Node (独立二进制)
│   └── src/
│       ├── main.rs              #     入口、Axum 服务
│       ├── rule_engine.rs       #     规则引擎 (25 条命令)
│       ├── verifier.rs          #     4 层消息验证
│       ├── leader.rs            #     Leader 选举与执行
│       ├── gossip/              #     Gossip 协议 (8 种消息)
│       ├── chain_submitter.rs   #     批量上链 (3 队列)
│       ├── chain_client.rs      #     Substrate 链交互 (subxt)
│       └── chain_cache.rs       #     链上数据缓存
├── node-operator/               # LLM 节点运维 (独立二进制)
│   └── src/
│       ├── main.rs              #     入口
│       ├── agent.rs             #     LLM Agent 逻辑
│       ├── cloud_provider.rs    #     云服务商接口
│       ├── ssh.rs               #     SSH 远程执行
│       └── approval.rs          #     人工审批流程
├── nexus-deploy/                # Docker Compose 集成测试
│   ├── docker-compose.yml       #     1 链 + 1 Agent + 3 Node
│   ├── test-e2e.sh              #     E2E 测试脚本
│   └── test-chain.ts            #     链上集成测试
├── frontend/                    # React Native + Expo 前端
├── scripts/                     # Polkadot.js 测试脚本
├── docs/                        # 架构文档与路线图
├── env-setup/                   # 开发环境 (Nix + rust-toolchain)
├── media-utils/                 # 媒体工具库
├── Cargo.toml                   # Workspace 配置
└── Dockerfile                   # 区块链节点 Docker 镜像
```

## Runtime Pallet 索引

| 索引 | Pallet | 类别 |
|------|--------|------|
| 0 | System | 基础 |
| 1 | Timestamp | 基础 |
| 2 | Aura | 共识 (出块) |
| 3 | Grandpa | 共识 (终局) |
| 4 | Balances | 基础 |
| 5 | TransactionPayment | 基础 |
| 6 | Sudo | 管理 |
| 50-55 | TradingPricing / Credit / Maker / P2p | 交易 |
| 60-65 | Escrow / StorageService / Evidence / Arbitration / StorageLifecycle | 争议与存储 |
| 70-77 | Technical / Arbitration / Treasury / Content Committee + Membership | 治理 |
| 90 | Contracts | 智能合约 |
| 110 | Assets | 资产 |
| 120-135 | EntityRegistry / Shop / Service / Transaction / Review / Token / Governance / Member / CommissionCore / Market / Disclosure / Kyc / TokenSale / Referral / LevelDiff / SingleLine | Entity 商业 |
| 140-142 | BotConsensus / BotRegistry / BotGroupMgmt | Nexus Bot |

## 快速开始

### 环境要求

- **Rust** stable（含 `wasm32-unknown-unknown` target）
- **Node.js** 18+（用于测试脚本）
- **Docker & Docker Compose**（可选，用于集成测试）

### 使用 Nix 设置环境

```bash
cd env-setup
nix develop
```

### 构建与运行

```bash
# 构建区块链节点
cargo build --release

# 启动开发链 (单节点模式)
./target/release/nexus-node --dev

# 连接 Polkadot.js Apps
# https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944
```

### 运行测试

```bash
# 运行全部 pallet 测试
cargo test

# 运行特定 pallet 测试
cargo test -p pallet-trading-p2p
cargo test -p pallet-entity-token
cargo test -p pallet-bot-consensus

# Nexus Bot 组件测试 (独立 workspace)
cd nexus-agent && cargo test
cd nexus-node && cargo test
```

### Docker 部署

```bash
# 构建区块链节点镜像
docker build . -t nexus-node

# 运行单节点开发链
docker run -p 9944:9944 -p 30333:30333 nexus-node --dev --rpc-external

# Nexus Bot 多节点集成测试 (1 链 + 1 Agent + 3 Node)
cd nexus-deploy
docker-compose up -d
```

## 链参数

| 参数 | 值 |
|------|-----|
| **代币符号** | NXS |
| **精度** | 12 位小数 (1 NXS = 10^12 单位) |
| **出块时间** | 6 秒 |
| **出块共识** | Aura |
| **终局共识** | GRANDPA |
| **SS58 格式** | 42 |
| **Existential Deposit** | 0.001 NXS |
| **Runtime 名称** | nexus |
| **Spec 版本** | 100 |

## 技术栈

### 区块链层
- **Polkadot SDK (Substrate)** — Runtime 框架
- **Rust** — 全部链上逻辑与节点实现
- **FRAME** — Pallet 开发框架
- **Aura + GRANDPA** — 混合共识
- **pallet-contracts** — ink! 智能合约支持

### Nexus Bot 层
- **Axum 0.7** — HTTP 服务框架
- **Tokio** — 异步运行时
- **ed25519-dalek 2** — 消息签名
- **tokio-tungstenite** — WebSocket Gossip
- **subxt 0.38** — Substrate 链交互（动态 API）
- **DashMap** — 并发状态管理

### Node Operator
- **Tokio + Reqwest** — 异步 HTTP
- **LLM 集成** — 自动化运维决策

### 前端
- **React Native + Expo 54** — 跨平台移动应用
- **TypeScript 5.9** — 类型系统
- **@polkadot/api** — Substrate RPC 客户端
- **Zustand + TanStack Query** — 状态管理

## 文档

| 文档 | 路径 | 内容 |
|------|------|------|
| 开发路线图 | `docs/NEXUS_DEVELOPMENT_ROADMAP.md` | Sprint 规划与依赖关系 |
| 节点奖励设计 | `docs/NEXUS_NODE_REWARD_DESIGN.md` | 订阅费+通胀混合奖励模型 |
| 分层存储设计 | `docs/NEXUS_LAYERED_STORAGE_DESIGN.md` | 全节点配置同步架构 |
| TG API 评估 | `docs/NEXUS_TELEGRAM_API_EVALUATION.md` | Telegram API 功能覆盖分析 |
| Bot 功能开发 | `docs/NEXUS_BOT_FEATURE_DEVELOPMENT.md` | Bot 功能迭代计划 |
| Agent 威胁模型 | `docs/AGENT_THREAT_MODEL.md` | 安全威胁分析与对策 |
| AI 自主运行 | `docs/AI_AUTONOMOUS_OPERATION.md` | AI 驱动的自动化运维方案 |
| 前端技术栈 | `docs/TECH_STACK.md` | React Native + Expo 技术选型 |

## 许可证

MIT-0
