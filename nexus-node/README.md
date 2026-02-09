# Nexus Node v0.1.0 — 去中心化项目节点

Nexus 去中心化多节点验证架构的 Project Node 组件。接收 Agent 多播的签名消息，通过 Gossip 协议在多节点间达成 M/K 共识，由 Leader 节点执行最终动作。

## 核心功能

- **消息验证** — 五步验证（时效 → Ed25519 验签 → Bot 活跃检查 → 公钥匹配 → 目标节点验证）
- **Gossip 共识** — 11 种消息类型 + M/K 投票（M = ceil(K×2/3)）
- **消息状态机** — HeardViaSeen → Received → Confirmed → Executing → Completed / Timeout / Failed
- **Equivocation 检测** — 同一消息不同哈希 → 自动广播警报
- **Pull 补偿** — 通过 Seen 得知但未持有原始消息 → 自动拉取
- **Leader 执行** — 确定性 Round-Robin 选举 (leader_idx = sequence % K) + Backup 超时接管
- **规则引擎** — 10 条可插拔规则链 + 3 个多平台适配器 (Telegram/Discord/Slack)
- **配置命令** — 10 种配置修改命令 (通过共识更新 GroupConfig) + 2 种查询命令 (本地直接回复)
- **群配置同步** — Agent 签名配置 → Gossip 广播 → 全节点同步 + 本地 JSON 持久化
- **链上交互** — subxt 动态 API: 缓存刷新 / 区块订阅 / 三队列批量提交
- **序列号追踪** — 重放保护（±10 容忍窗口）

## 架构概览

```
                              ┌───────────────┐
                              │   Substrate    │
                              │    Chain       │
                              └───┬───────┬───┘
                           读取缓存│       │批量提交
                    ┌─────────────┘       └──────────────┐
                    ▼                                     ▼
┌──────────┐    ┌──────────────────────────────────────────────┐
│  Agent   │───▶│                 Nexus Node                   │
│(签名消息) │    │  ┌──────┐  ┌────────┐  ┌───────┐  ┌──────┐ │
└──────────┘    │  │ API  │→│Verifier│→│Gossip │→│Leader│ │
     ◀──────────│  └──────┘  └────────┘  │Engine │  └──┬───┘ │
   (执行指令)    │                         └───┬───┘     │      │
                │    ┌───────────┐  ┌─────────┘  ┌─────┘      │
                │    │ChainCache │  │RuleEngine│  │ChainSubmit│ │
                │    └───────────┘  └──────────┘  └───────────┘ │
                └──────────────────────────────────────────────┘
                         ▲         Gossip WebSocket        ▲
                         └──────── 其他 Node 节点 ──────────┘
```

## 环境变量

| 变量 | 必填 | 默认值 | 说明 |
|------|------|--------|------|
| `NODE_ID` | ✅ | - | 节点唯一标识 (hex) |
| `LISTEN_PORT` | - | `8080` | HTTP API 端口 |
| `GOSSIP_PORT` | - | `9090` | Gossip WebSocket 端口 |
| `CHAIN_RPC` | - | `ws://127.0.0.1:9944` | Substrate RPC 端点 |
| `SIGNING_KEY_PATH` | - | `/data/node.key` | 节点 Ed25519 私钥路径 |
| `DATA_DIR` | - | `/data` | 数据目录（含 `configs/` 子目录） |
| `BOT_REGISTRATIONS` | - | - | Bot 缓存: `hash:pubkey:platform,...` |
| `NODE_LIST` | - | - | 节点列表: `id@endpoint,...` |
| `AGENT_ENDPOINT` | - | `http://localhost:8443` | Agent 执行端点（Leader 回传指令） |
| `RUST_LOG` | - | `nexus_node=info` | 日志级别 |

## Docker

```bash
docker build -t nexus-node .

docker run -d \
  --name nexus-node-1 \
  -p 8080:8080 -p 9090:9090 \
  -v node1-data:/data \
  -e NODE_ID="node_001" \
  -e NODE_LIST="node_001@http://node1:8080,node_002@http://node2:8080,node_003@http://node3:8080" \
  nexus-node
```

镜像基于 `rust:1.82-slim` 编译 + `debian:bookworm-slim` 运行，内置 healthcheck。

## HTTP API

| 端点 | 方法 | 说明 |
|------|------|------|
| `/v1/message` | POST | 接收 Agent 签名消息（SignedMessage JSON） |
| `/v1/status/{msg_id}` | GET | 查询消息状态（调试用） |
| `/health` | GET | 健康检查（node_id / uptime / active_messages / messages_processed） |

## 消息流程

```
Agent POST /v1/message
       │
       ▼
  五步验证 (verifier.rs)
  ① 时效检查 (60s)
  ② Ed25519 验签 (sign_data = pk + bot_hash + seq_le + ts_le + msg_hash)
  ③ Bot 活跃状态 (ChainCache)
  ④ 公钥匹配 (消息公钥 == 链上注册公钥)
  ⑤ 目标节点 (确定性选择 K 个节点, 本节点在列表中)
       │
       ▼
  GossipEngine.on_agent_message()
  → 状态 Received + 广播 MessageSeen
       │
       ▼
  收集其他节点 Seen → check_consensus()
  M = ceil(K×2/3), 一致 hash 数 >= M → Confirmed
       │
       ▼
  Leader 执行 (leader_idx = seq % K)
  → POST Agent /v1/execute → 广播 ExecutionResult
       │
       ▼
  Completed / Failed (Backup 超时接管)
```

## Gossip 协议

### 消息类型 (11 种)

| 类型 | 方向 | 载荷 | 说明 |
|------|------|------|------|
| `MessageSeen` | 广播 | msg_id, msg_hash, config_version | 节点确认收到消息 |
| `MessagePull` | 单播 | msg_id | 请求拉取未收到的原始消息 |
| `MessagePullResponse` | 回复 | msg_id, signed_message | Pull 响应（携带完整 SignedMessage） |
| `DecisionVote` | 广播 | msg_id, action_type, voter_signature | 动作投票 |
| `EquivocationAlert` | 广播 | owner_pk, seq, hash_a/b, sig_a/b | 矛盾行为警报 |
| `ExecutionResult` | 广播 | msg_id, success, agent_receipt | Leader 执行结果 |
| `LeaderTakeover` | 广播 | msg_id, original_leader, backup_rank | Backup 接管 |
| `Heartbeat` | 广播 | node_id, messages_processed | 心跳 |
| `ConfigSync` | 广播 | bot_id_hash, signed_config | 群配置同步 |
| `ConfigPull` | 单播 | bot_id_hash, current_version | 请求拉取群配置 |
| `ConfigPullResponse` | 回复 | bot_id_hash, signed_config? | 群配置拉取响应 |

### 消息状态机

```
HeardViaSeen ──Pull──▶ Received ──M/K共识──▶ Confirmed ──Leader──▶ Executing ──▶ Completed
                                                                       │
                                                                       └──▶ Failed
                                                                       └──▶ Timeout
```

### 网络层

- **服务端:** WebSocket 监听 `GOSSIP_PORT`，接受其他节点连接
- **客户端:** 每 10s 从 ChainCache 发现新节点并主动连接
- **端点推导:** `http://host:808X` → `ws://host:909X`

## 群配置同步

```
Agent 签名配置 ──▶ Gossip ConfigSync 广播 ──▶ 全节点
                                               │
                      验证流程:                  ▼
                      ① Bot 注册查找            ChainCache
                      ② 签名者公钥 == Bot owner  │
                      ③ Ed25519 签名验证         ▼
                      ④ 版本号单调递增           持久化 → DATA_DIR/configs/{bot_id_hash}.json
                                               │
                                               ▼
                                          RuleEngine 实时读取
```

**启动恢复:** 节点启动时从 `DATA_DIR/configs/` 加载本地 JSON + 向邻居发送 `ConfigPull` 补偿。

## 动作类型 (ActionType)

嵌套枚举，分四类:

| 分类 | 变体 | 需要共识 |
|------|------|---------|
| **Message** | Send / Delete / DeleteBatch / Pin / Unpin | ✅ |
| **Admin** | Ban / Unban / Mute / Unmute / ApproveJoinRequest / DeclineJoinRequest / SetPermissions / Kick / Promote / Demote | ✅ |
| **Query** | GetChatMember / GetAdmins / GetChat / GetMe | ❌ |
| **ConfigUpdate** | AddBlacklistWord / RemoveBlacklistWord / LockType / UnlockType / SetWelcome / SetFloodLimit / SetWarnLimit / SetWarnAction | ✅ |
| **NoAction** | - | ❌ |

## 规则引擎

10 条可插拔规则链，按优先级顺序评估（首个匹配即返回）:

| # | 规则 | 说明 |
|---|------|------|
| 1 | **JoinRequestRule** | 入群申请（根据 `join_policy`: AutoApprove / ManualApproval / CaptchaRequired / TokenGating） |
| 2 | **AdminPermissionRule** | 权限前置检查 — 非管理员执行 admin/config 命令 → 拒绝（19 种命令受保护） |
| 3 | **CommandRule** | 25 种命令: /ban /unban /mute /unmute /kick /warn /unwarn /warns /resetwarns /pin /del /help /rules /info /id + 10 种配置命令 (/blacklist /unblacklist /blacklists /lock /unlock /locks /welcome /flood /setwarnlimit /setwarnaction) |
| 4 | **AntifloodRule** | 防刷屏标记（实际计数在 Agent LocalStore，Node 侧验证） |
| 5 | **SpamDetectorRule** | 反垃圾检测: emoji 过多 / 全大写 >80% / Latin+Cyrillic 多语言混排 |
| 6 | **BlacklistRule** | 黑名单词过滤 — 3 种匹配模式 (Exact/Contains/Regex) × 4 种动作 (Delete/Warn/Mute/Ban) |
| 7 | **LockRule** | 消息类型锁定 — 14 种类型: Photo/Video/Audio/Document/Sticker/Gif/Url/Forward/Voice/Contact/Location/Poll/Game/Inline |
| 8 | **WelcomeRule** | 欢迎消息 — 6 种变量替换: {first} {last} {fullname} {username} {id} {chatname} |
| 9 | **LinkFilterRule** | 链接过滤（检测 http:// / https:// / t.me/） |
| 10 | **DefaultRule** | 兜底 → NoAction |

## 多平台适配器

| 适配器 | 事件解析 | API 映射 |
|--------|---------|---------|
| **TelegramAdapter** | Telegram Update JSON → RuleContext | sendMessage / deleteMessage / banChatMember / restrictChatMember / approveChatJoinRequest / pinChatMessage |
| **DiscordAdapter** | Gateway Event (MESSAGE_CREATE / GUILD_MEMBER_ADD) → RuleContext | POST /channels/messages, DELETE /channels/messages, PUT /guilds/bans, PATCH /guilds/members (timeout), PUT /channels/pins |
| **SlackAdapter** | Events API (message / team_join) → RuleContext | chat.postMessage / chat.delete / conversations.kick / pins.add |

`AdapterRegistry` 统一管理所有适配器，通过 `platform_name()` 查找。

## GroupConfig 字段

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `version` | u64 | - | 配置版本号（单调递增） |
| `bot_id_hash` | String | - | Bot ID 哈希 |
| `join_policy` | JoinApprovalPolicy | - | 入群策略 |
| `filter_links` | bool | - | 是否过滤链接 |
| `restrict_mentions` | bool | - | 是否限制 @everyone |
| `rate_limit_per_minute` | u16 | - | 限流（0=不限） |
| `auto_mute_duration` | u64 | - | 触发限流后禁言秒数 |
| `new_member_restrict_duration` | u64 | - | 新成员限制时长（秒） |
| `welcome_message` | String | - | 欢迎消息（空=不发送） |
| `whitelist` | Vec\<String\> | - | 白名单用户 |
| `admins` | Vec\<String\> | - | 管理员列表 |
| `quiet_hours_start/end` | Option\<u8\> | None | 静默时段（UTC 小时） |
| `antiflood_limit` | u16 | 0 | 防刷屏阈值（0=关闭） |
| `antiflood_window` | u16 | 10 | 防刷屏时间窗口（秒） |
| `antiflood_action` | FloodAction | Mute | 触发动作 |
| `warn_limit` | u8 | 3 | 警告上限 |
| `warn_action` | WarnAction | Ban | 超限动作 |
| `blacklist_words` | Vec\<String\> | [] | 黑名单关键词 |
| `blacklist_mode` | BlacklistMode | Contains | 匹配模式 |
| `blacklist_action` | BlacklistAction | Delete | 触发动作 |
| `lock_types` | Vec\<LockType\> | [] | 锁定的消息类型 |
| `spam_detection_enabled` | bool | false | 反垃圾检测开关 |
| `spam_max_emoji` | u8 | 0 | 最大 emoji 数（0=不限） |
| `spam_first_messages_only` | u8 | 0 | 只检查新成员前 N 条（0=全部） |

## 链上批量提交

`ChainSubmitter` 维护三个内存队列，按优先级定时批量提交:

| 优先级 | 队列 | 目标 Pallet |
|--------|------|------------|
| 🔴 最高 | Equivocation 举报 | `pallet-bot-consensus::report_equivocation` |
| 🟡 中等 | 动作日志 | `pallet-bot-group-mgmt::log_action` |
| 🟢 普通 | 消息确认 | `pallet-bot-consensus::submit_confirmations` |

- **批量大小:** 50 条/次
- **提交间隔:** 6 秒（每个区块一次）
- **失败处理:** 重新入队（队首）

`SequenceTracker` 提供序列号重放保护: sequence > last → 有效, == last → 重放, < last 且差值 ≤10 → 容忍。

## 链客户端 (subxt)

使用 subxt 动态 API（无需编译时 metadata），支持:

- **Storage 读取:** BotRegistry::Bots, BotConsensus::ActiveNodeList, BotConsensus::Nodes
- **交易提交:** submit_confirmations, log_action, report_equivocation
- **区块订阅:** subscribe_finalized_blocks（每 100 个区块刷新缓存）
- **签名密钥:** SR25519 (从 `DATA_DIR/node_signer.key` 加载，缺失时使用 Alice dev key)

连接失败不阻止启动 — 降级为静态缓存模式。

## 文件结构

```
nexus-node/
├── src/
│   ├── main.rs               # 入口 + Axum 服务器 + 链客户端 + 后台任务
│   ├── config.rs              # NodeConfig (6 字段, 从环境变量加载)
│   ├── types.rs               # ActionType(嵌套枚举) + SignedMessage + GossipEnvelope
│   │                          #   + GroupConfig(23字段) + SignedGroupConfig
│   │                          #   + ConfigUpdateAction(8变体)
│   │                          #   + 6 种枚举(FloodAction/WarnAction/BlacklistMode/...)
│   │                          #   + MessageState + BotInfoCache + NodeInfoCache
│   ├── api.rs                 # 3 个 HTTP handler (message/status/health)
│   ├── verifier.rs            # 五步验证 + 确定性节点选择 (SHA256 seed + Fisher-Yates)
│   ├── chain_cache.rs         # 链上缓存 (Bot/Node/GroupConfig) + Ed25519 配置验签
│   │                          #   + JSON 持久化 + 磁盘恢复
│   ├── chain_client.rs        # subxt 动态 API (storage读取 + 交易提交 + 区块订阅)
│   ├── chain_submitter.rs     # 三队列批量提交 + 优先级 flush + SequenceTracker
│   ├── leader.rs              # LeaderExecutor (Round-Robin选举 + POST Agent /v1/execute)
│   │                          #   + FailoverManager (Backup 超时接管)
│   ├── rule_engine.rs         # RuleEngine (10条规则链) + 3 个 PlatformAdapter
│   │                          #   + AdapterRegistry
│   └── gossip/
│       ├── mod.rs             # 模块声明
│       ├── state.rs           # GossipState (DashMap状态机 + M/K共识 + Equivocation检测 + GC)
│       ├── engine.rs          # GossipEngine (11种消息路由 + ConfigSync + Pull补偿)
│       └── network.rs         # WebSocket 网络层 (服务端 + 客户端 + 对等发现)
├── .env.example               # 环境变量模板
├── Cargo.toml                 # 依赖: axum 0.7, tokio-tungstenite 0.24, ed25519-dalek 2,
│                              #   subxt 0.38, dashmap 6, regex 1, reqwest 0.12, ...
├── Dockerfile                 # 多阶段构建 (rust:1.82-slim → debian:bookworm-slim)
└── README.md
```

## 依赖

| 类别 | 依赖 | 版本 |
|------|------|------|
| HTTP 服务 | axum | 0.7 |
| HTTP 客户端 | reqwest | 0.12 |
| WebSocket | tokio-tungstenite | 0.24 |
| 签名验证 | ed25519-dalek | 2 |
| 哈希 | sha2 | 0.10 |
| 链客户端 | subxt / subxt-signer | 0.38 |
| SCALE 编码 | parity-scale-codec | 3.6 |
| 并发状态 | dashmap | 6 |
| 正则 | regex | 1 |
| 序列化 | serde / serde_json / bincode | 1.0 / 1.0 / 1.3 |

## 测试

```bash
cargo test    # 81 tests
```

| 模块 | 测试数 | 说明 |
|------|--------|------|
| verifier | 3 | K 值计算 + 确定性选择 + Ed25519 签名往返 |
| chain_cache | 7 | 签名配置验证 + 版本递增 + 持久化恢复 |
| chain_submitter | 5 | 队列 flush + 优先级 + SequenceTracker 重放保护 |
| leader | 6 | Round-Robin 选举 + determine_action (入群/命令/普通) |
| gossip/state | 6 | 状态机流转 + M/K 共识 + Equivocation + GC |
| gossip/engine | 3 | Agent 消息处理 + Seen 触发 Pull + 共识流程 |
| rule_engine | 51 | 10 条规则 + 3 平台适配器 + 22 E2E (含 13 配置命令) + 4 bench |
| **总计** | **81** | |
