# Nexus Node — 项目节点

去中心化多节点验证架构（方案 D）的 Project Node 组件。

## 功能

- 接收 Agent 多播的签名消息
- 四层验证（Ed25519 验签 → Bot 活跃检查 → 公钥匹配 → 目标节点验证）
- Gossip 共识（11 种消息类型 + M/K 投票）
- 消息状态机（HeardViaSeen → Received → Confirmed → Executing → Completed）
- Equivocation 检测
- Pull 补偿机制
- Leader 执行 + 轮询选举
- 规则引擎（可插拔 Rule 链 + 多平台适配器: Telegram/Discord/Slack）
- 群配置同步（Agent → Gossip → 全节点，Ed25519 签名验证 + 版本控制）
- 链上批量提交（confirmation / action_log / equivocation 三队列）
- 本地持久化（群配置 JSON 持久化 + 启动恢复）

## 环境变量

| 变量 | 必填 | 默认值 | 说明 |
|------|------|--------|------|
| `NODE_ID` | ✅ | - | 节点唯一标识 |
| `LISTEN_PORT` | - | `8080` | HTTP API 端口 |
| `GOSSIP_PORT` | - | `9090` | Gossip WebSocket 端口 |
| `CHAIN_RPC` | - | `ws://127.0.0.1:9944` | 链上 RPC |
| `SIGNING_KEY_PATH` | - | `/data/node.key` | 节点私钥路径 |
| `DATA_DIR` | - | `/data` | 数据目录（含 `configs/` 子目录） |
| `BOT_REGISTRATIONS` | - | - | Bot 缓存: `hash:pubkey:platform,...` |
| `NODE_LIST` | - | - | 节点列表: `id@endpoint,...` |

## Docker 运行

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

## API 端点

| 端点 | 方法 | 说明 |
|------|------|------|
| `/v1/message` | POST | 接收 Agent 签名消息 |
| `/v1/status/{msg_id}` | GET | 查询消息状态 |
| `/health` | GET | 健康检查 |

## Gossip 消息类型

| 类型 | 方向 | 说明 |
|------|------|------|
| `MessageSeen` | 广播 | 节点确认收到消息（含 config_version） |
| `MessagePull` | 单播 | 请求拉取未收到的消息 |
| `PullResponse` | 回复 | 消息拉取响应 |
| `DecisionVote` | 广播 | M/K 共识投票 |
| `EquivocationAlert` | 广播 | 矛盾行为告警 |
| `ExecutionResult` | 广播 | Leader 执行结果 |
| `LeaderTakeover` | 广播 | Leader 接管 |
| `Heartbeat` | 广播 | 心跳 |
| `ConfigSync` | 广播 | 群配置同步（SignedGroupConfig） |
| `ConfigPull` | 单播 | 请求拉取群配置 |
| `ConfigPullResponse` | 回复 | 群配置拉取响应 |

## 消息流程

```
Agent POST /v1/message → 四层验证 → GossipEngine
                                        ↓
                              广播 MessageSeen → 其他节点
                                        ↓
                              收集 Seen → 检查 M/K 共识
                                        ↓
                              共识达成 → Leader 执行
```

## 群配置同步流程

```
Agent POST /v1/group-config → 签名 → 广播 ConfigSync → 所有节点
                                                          ↓
                                            验证签名 + 版本号 → ChainCache
                                                          ↓
                                            持久化 → DATA_DIR/configs/{bot_id_hash}.json
                                                          ↓
                                            RuleEngine 实时读取 GroupConfig
```

**启动恢复:** 节点启动时从 `DATA_DIR/configs/` 加载本地配置 + 向邻居发送 `ConfigPull` 补偿。

## GroupConfig 字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `version` | u64 | 配置版本号（单调递增） |
| `bot_id_hash` | String | Bot ID 哈希 |
| `join_policy` | JoinApprovalPolicy | 入群策略（AutoApprove/ManualApproval/CaptchaRequired/TokenGating） |
| `filter_links` | bool | 是否过滤链接 |
| `restrict_mentions` | bool | 是否限制 @everyone |
| `rate_limit_per_minute` | u16 | 限流（0=不限） |
| `auto_mute_duration` | u64 | 触发限流后自动禁言秒数 |
| `new_member_restrict_duration` | u64 | 新成员限制时长（秒） |
| `welcome_message` | String | 欢迎消息（空=不发送） |
| `whitelist` | Vec\<String\> | 白名单用户 ID 哈希 |
| `admins` | Vec\<String\> | 管理员列表 |
| `quiet_hours_start/end` | Option\<u8\> | 静默时段（UTC 小时） |

## 规则引擎

可插拔规则链，按优先级顺序评估:

1. **JoinRequestRule** — 入群申请（根据 `join_policy` 自动通过/拒绝）
2. **CommandRule** — 命令解析（/ban /mute /unmute /pin /del）
3. **LinkFilterRule** — 链接过滤（根据 `filter_links` 配置）
4. **DefaultRule** — 兜底（NoAction）

多平台适配器: `TelegramAdapter` / `DiscordAdapter` / `SlackAdapter`

## 文件结构

```
nexus-node/
├── src/
│   ├── main.rs              # 入口 + Axum 服务器 + 启动恢复
│   ├── config.rs            # 配置管理
│   ├── types.rs             # 数据类型 + Gossip 消息 + GroupConfig + ActionType
│   ├── api.rs               # HTTP API 处理
│   ├── verifier.rs          # 四层验证 + Ed25519 验签
│   ├── chain_cache.rs       # 链上数据缓存 + 群配置存储/验签/持久化
│   ├── chain_client.rs      # Substrate 链客户端 (subxt)
│   ├── chain_submitter.rs   # 链上批量提交（三队列 + 优先级）
│   ├── leader.rs            # Leader 执行 + 轮询选举 + 失败转移
│   ├── rule_engine.rs       # 规则引擎 + 多平台适配器
│   └── gossip/
│       ├── mod.rs
│       ├── state.rs         # 消息状态机 + M/K 共识
│       ├── engine.rs        # Gossip 引擎 + 消息路由 + ConfigSync 处理
│       └── network.rs       # WebSocket 网络层
├── Cargo.toml
├── Dockerfile
└── README.md
```

## 测试

```bash
cargo test    # 48 tests
```
