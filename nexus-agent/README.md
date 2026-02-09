# Nexus Agent v0.1.0 — 群主本地代理

Nexus 去中心化多节点验证架构的 Local Agent 组件。运行在群主本地，接收 Telegram Webhook、Ed25519 签名消息、确定性多播到项目节点网络，并执行 Leader 节点回传的管理指令。

## 核心功能

- **Webhook 接收** — Telegram Bot API setWebhook + secret token 验证
- **Ed25519 签名** — 密钥永不上链，sign_message (结构化) + sign_raw (原始数据)
- **确定性多播** — SHA256 seed + Fisher-Yates 选择 K 个节点并发 POST
- **Leader 指令执行** — 验证共识节点数 + Leader 签名 → 调用 14 种 TG API 方法
- **配置命令处理** — 拦截 ConfigUpdate 动作 → 更新 ConfigStore + 签名 + 持久化 + 确认消息 + 广播节点
- **警告系统 E2E** — /warn 累积计数 (LocalStore) + 超限自动升级 (ban/kick/mute) + /unwarn /warns /resetwarns
- **群配置管理** — GET/POST API + Ed25519 钱包签名认证 + 本地 JSON 持久化 + 全节点广播
- **本地快速路径 (LocalProcessor)** — 不走共识的高频检测（6 级检测链）:
  - 欢迎消息 — 6 种变量替换 ({first}/{last}/{fullname}/{username}/{id}/{chatname})
  - 防刷屏 (Antiflood) — 滑动窗口计数器 (limit × window)
  - 黑名单词过滤 — 3 种匹配模式 (Exact/Contains/Regex) × 4 种动作
  - 消息类型锁定 — 14 种消息类型 (Photo/Video/Audio/Document/Sticker/Gif/Url/Forward/Voice/Contact/Location/Poll/Game/Inline)
  - 重复消息检测 — 消息指纹哈希 + 60 秒窗口 + 3 次阈值
  - Emoji 垃圾检测 — 7 段 Unicode 范围匹配
- **管理员缓存** — getChatAdministrators 自动刷新 + 5 分钟 TTL
- **审计日志** — 本地快速路径动作签名后异步多播到 Node 网络
- **滑动窗口限流器** — 防 Webhook flood 和恶意 /v1/execute 请求
- **序列号持久化** — AtomicU64 + atomicwrites 原子文件写入（防重放）
- **加盐哈希** — community_id_hash / platform_user_id_hash（owner_account 派生盐值，防暴力碰撞）

## 架构概览

```
                    ┌───────────────────┐
                    │  Telegram Server  │
                    └─────┬───────▲─────┘
                 Webhook  │       │ Bot API
                    ┌─────▼───────┴─────┐
                    │    Nexus Agent     │
                    │ ┌───────────────┐  │
                    │ │ LocalProcessor│──┤──▶ 直接 TG API (不走共识)
                    │ │  (6 级检测)    │  │
                    │ └───────────────┘  │
                    │ ┌───────────────┐  │
                    │ │   Signer      │  │   sign_message / sign_raw
                    │ │  (Ed25519)    │  │
                    │ └───────┬───────┘  │
                    │ ┌───────▼───────┐  │
                    │ │  Multicaster  │──┤──▶ 确定性选择 K 个节点
                    │ │  (并发 POST)  │  │     并发 POST /v1/message
                    │ └───────────────┘  │
                    │ ┌───────────────┐  │
                    │ │TG Executor    │◀─┤──  Leader POST /v1/execute
                    │ │ (14 种 API)   │  │
                    │ └───────────────┘  │
                    │ ┌───────────────┐  │
                    │ │ ConfigStore   │──┤──▶ 广播 ConfigSync → 全节点
                    │ │ (群配置管理)   │  │
                    │ └───────────────┘  │
                    └───────────────────┘
```

## 环境变量

| 变量 | 必填 | 默认值 | 说明 |
|------|------|--------|------|
| `BOT_TOKEN` | ✅ | - | Telegram Bot Token（永不上链，SHA256 后作为 bot_id_hash） |
| `WEBHOOK_URL` | ✅ | - | Webhook 外部 URL，如 `https://my-server.com:8443` |
| `WEBHOOK_PORT` | - | `8443` | Webhook 监听端口 |
| `WEBHOOK_SECRET` | - | 随机生成 | Telegram Webhook secret token（随机 16 字节 hex） |
| `CHAIN_RPC` | - | `ws://127.0.0.1:9944` | Substrate 链 RPC 端点 |
| `DATA_DIR` | - | `/data` | 数据目录（密钥、序列号、群配置） |
| `MULTICAST_TIMEOUT_MS` | - | `3000` | 多播超时（毫秒） |
| `NODES` | - | - | 静态节点列表: `id1@http://host1:port,id2@http://host2:port` |
| `RUST_LOG` | - | `nexus_agent=info` | 日志级别 |

## Docker

```bash
docker build -t nexus-agent .

docker run -d \
  --name nexus-agent \
  -p 8443:8443 \
  -v nexus-data:/data \
  -e BOT_TOKEN="your_bot_token" \
  -e WEBHOOK_URL="https://your-server.com:8443" \
  -e NODES="node1@http://node1:8080,node2@http://node2:8080,node3@http://node3:8080" \
  nexus-agent
```

镜像基于 `rust:1.82-slim` 编译 + `debian:bookworm-slim` 运行，内置 healthcheck (`/health`)。

### 本地开发

```bash
export BOT_TOKEN="your_bot_token"
export WEBHOOK_URL="https://your-ngrok-url"
export DATA_DIR="./data"
export NODES="node1@http://localhost:8081,node2@http://localhost:8082"

cargo run
```

## HTTP API

| 端点 | 方法 | 说明 |
|------|------|------|
| `/webhook` | POST | Telegram Webhook 接收（验证 X-Telegram-Bot-Api-Secret-Token） |
| `/v1/execute` | POST | Leader 节点管理指令执行（验证 bot_id_hash + 共识数 + Leader 签名） |
| `/v1/group-config` | GET | 获取当前群配置（返回 config + version + signed） |
| `/v1/group-config` | POST | 更新群配置（Ed25519 钱包签名认证 + 合并更新 + 广播） |
| `/health` | GET | 健康检查（bot_id_hash / public_key / sequence / uptime / nodes_count） |

## 首次启动

1. Agent 自动生成 Ed25519 密钥对 → `$DATA_DIR/agent.key`（32 字节种子，权限 600）
2. 控制台输出公钥 (hex) → 复制注册到链上
3. 链上调用 `pallet-bot-registry::register_bot()` 注册公钥
4. Agent 自动调用 `setWebhook`（3 次重试，allowed_updates: message/edited_message/callback_query/chat_member/chat_join_request）
5. 加载本地群配置 `$DATA_DIR/group_config.json`（如存在）
6. 启动 LocalStore 定时清理（每 60 秒）

## 消息流程

```
Telegram 用户发消息
       │
       ▼
Telegram Server → POST /webhook → Agent
       │
       ▼
  ① 验证 secret token (X-Telegram-Bot-Api-Secret-Token)
  ② 解析 TelegramUpdate (message / edited_message / callback_query / chat_member / chat_join_request)
       │
       ├──▶ 本地快速路径 (异步): LocalProcessor.process() → execute_local_action() → TG API
       │    └── 审计日志: submit_audit_log() → 签名 → 多播到 Node
       │
       ▼
  ③ 签名: sign_data = pk + bot_hash + seq_le + ts_le + msg_hash → Ed25519
  ④ 构造 SignedMessage (owner_public_key, bot_id_hash, sequence, timestamp, message_hash, telegram_update, owner_signature, platform)
       │
       ▼
  ⑤ 确定性选择 K 个节点 (seed = SHA256(msg_hash + seq_le), Fisher-Yates)
  ⑥ 并发 POST /v1/message → K 个节点 (timeout: MULTICAST_TIMEOUT_MS)
       │
       ▼
  ⑦ 返回 200 OK (不阻塞 Webhook 响应)
```

## 群配置认证机制

群配置 POST API 使用 Ed25519 钱包签名认证:

1. 客户端签名: `sign(bot_id_hash_hex_bytes + timestamp_le_bytes)`
2. Agent 验证: 签名者公钥 == Agent 公钥 (Bot owner)
3. 时间戳窗口: ±5 分钟

```json
// POST /v1/group-config 请求体中的认证字段
{
  "auth_signature": "<hex-encoded Ed25519 signature>",
  "auth_public_key": "<hex-encoded public key>",
  "auth_timestamp": 1234567890
}
```

## 群配置同步流程

```
群主 POST /v1/group-config
       │
       ▼
  验证认证 → 合并配置字段 (26 个可选字段 merge) → version++
       │
       ▼
  Agent sign_raw(config_json) → SignedGroupConfig
       │
       ├──▶ 本地存储: ConfigStore.set() → DATA_DIR/group_config.json
       │
       └──▶ 异步广播: POST /v1/gossip → 所有 Node (ConfigSync 载荷)
```

## 本地快速路径 (LocalProcessor)

6 级检测链，在 Webhook 收到消息后立即执行（不走 Node 共识）:

| # | 检测 | 条件 | 触发动作 | 说明 |
|---|------|------|---------|------|
| 1 | **欢迎消息** | new_chat_members + welcome_message 非空 | SendMessage | 不受豁免限制，6 种变量替换 |
| 2 | **防刷屏** | antiflood_limit > 0 && 超限 | Mute/Ban/Kick/Delete | 触发后立即返回（跳过后续检查） |
| 3 | **黑名单词** | blacklist_words 匹配 | Delete/Warn/Mute/Ban | 3 种匹配模式 × 4 种动作 |
| 4 | **消息类型锁定** | lock_types 包含消息类型 | DeleteMessage | 14 种类型检测 |
| 5 | **重复消息** | spam_detection_enabled && 60s 内 ≥3 次相同内容 | DeleteMessage | 消息指纹哈希窗口 |
| 6 | **Emoji 过多** | spam_max_emoji > 0 && emoji 数超限 | DeleteMessage | 7 段 Unicode 范围 |

**豁免规则:** 管理员、白名单用户、命令消息 (`/` 开头) 跳过第 2-6 步检查。

**LocalActionType:** DeleteMessage / MuteUser / BanUser / KickUser / SendMessage → 直接调用 TG API。

## LocalStore（本地状态存储）

| 存储 | 键 | 说明 |
|------|---|------|
| **flood_counters** | (chat_id, user_id) | 滑动窗口计数器 + 过期时间 |
| **warn_counts** | (chat_id, user_id) | 警告计数（add/remove/reset/get） |
| **admin_cache** | chat_id | 管理员 ID 列表 + TTL 300s |
| **recent_messages** | chat_id | 消息指纹 Vec（user_id + text_hash + timestamp） |

定时清理（60 秒）: flood 计数器 >60s / admin 缓存过期 / 消息指纹 >5 分钟。

## Telegram API 方法

### TelegramExecutor（Leader 指令，需共识）

| ActionType | TG API 方法 | 说明 |
|------------|------------|------|
| Message::Send | `sendMessage` | 发送消息 |
| Message::Delete | `deleteMessage` | 删除消息 |
| Message::DeleteBatch | `deleteMessages` | 批量删除（TODO） |
| Message::Pin | `pinChatMessage` | 置顶消息 |
| Message::Unpin | `unpinChatMessage` | 取消置顶 |
| Admin::Ban | `banChatMember` | 封禁用户 |
| Admin::Unban | `unbanChatMember` | 解封用户 |
| Admin::Mute | `restrictChatMember` | 禁言（can_send_messages: false） |
| Admin::Unmute | `restrictChatMember` | 解除禁言（恢复全部权限） |
| Admin::ApproveJoinRequest | `approveChatJoinRequest` | 通过入群申请 |
| Admin::DeclineJoinRequest | `declineChatJoinRequest` | 拒绝入群申请 |
| Admin::SetPermissions | `setChatPermissions` | 设置群权限（TODO） |
| Admin::Kick | `banChatMember` + `unbanChatMember` | 踢出（ban 后立即 unban） |
| Admin::Promote | `promoteChatMember` | 提升管理员（赋予权限） |
| Admin::Demote | `promoteChatMember` | 降级管理员（撤销权限） |
| Query::GetChatMember | `getChatMember` | 获取成员信息 |
| Query::GetAdmins | `getChatAdministrators` | 获取管理员列表 |
| Query::GetChat | `getChat` | 获取群信息 |
| Query::GetMe | `getMe` | 获取 Bot 信息 |

**签名回执:** 每次执行成功后生成 `agent_signature = pk_hex:sign(action_id + method + SHA256(tg_response))`

### 本地快速路径（不走共识）

| 动作 | TG API 方法 |
|------|------------|
| DeleteMessage | `deleteMessage` |
| MuteUser | `restrictChatMember` + 可选 `deleteMessage` |
| BanUser | `banChatMember` + 可选 `deleteMessage` |
| KickUser | `banChatMember` + `unbanChatMember` |
| SendMessage | `sendMessage` |

## Leader 指令验证 (/v1/execute)

Agent 接收 Leader 指令前执行 4 步验证:

1. **bot_id_hash 匹配** — 指令目标 == 本 Agent
2. **共识数量检查** — consensus_nodes.len() >= M (ceil(K×2/3))
3. **Leader 在共识列表中** — leader_node_id ∈ consensus_nodes
4. **Leader Ed25519 签名** — 签名数据 = SHA256(action_id + bot_id_hash + action_type + chat_id_le)

## GroupConfig 字段 (26 个)

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `version` | u64 | - | 配置版本号（单调递增） |
| `bot_id_hash` | String | - | Bot ID 哈希 |
| `join_policy` | JoinApprovalPolicy | AutoApprove | 入群策略 |
| `filter_links` | bool | false | 是否过滤链接 |
| `restrict_mentions` | bool | false | 是否限制 @everyone |
| `rate_limit_per_minute` | u16 | 0 | 限流（0=不限） |
| `auto_mute_duration` | u64 | 300 | 触发限流后禁言秒数 |
| `new_member_restrict_duration` | u64 | 0 | 新成员限制时长（秒） |
| `welcome_message` | String | "" | 欢迎消息（空=不发送） |
| `whitelist` | Vec\<String\> | [] | 白名单用户 ID |
| `admins` | Vec\<String\> | [] | 管理员列表 |
| `quiet_hours_start` | Option\<u8\> | None | 静默时段开始（UTC 小时） |
| `quiet_hours_end` | Option\<u8\> | None | 静默时段结束（UTC 小时） |
| `updated_at` | u64 | - | 最后更新时间戳 |
| `antiflood_limit` | u16 | 0 | 防刷屏阈值（0=关闭） |
| `antiflood_window` | u16 | 10 | 防刷屏时间窗口（秒） |
| `antiflood_action` | FloodAction | Mute | 触发动作 (Mute/Kick/Ban/DeleteOnly) |
| `warn_limit` | u8 | 3 | 警告上限 |
| `warn_action` | WarnAction | Ban | 超限动作 (Ban/Kick/Mute) |
| `blacklist_words` | Vec\<String\> | [] | 黑名单关键词 |
| `blacklist_mode` | BlacklistMode | Contains | 匹配模式 (Exact/Contains/Regex) |
| `blacklist_action` | BlacklistAction | Delete | 触发动作 (Delete/DeleteAndWarn/DeleteAndMute/DeleteAndBan) |
| `lock_types` | Vec\<LockType\> | [] | 锁定的消息类型 |
| `spam_detection_enabled` | bool | false | 反垃圾检测开关 |
| `spam_max_emoji` | u8 | 0 | 最大 emoji 数（0=不限） |
| `spam_first_messages_only` | u8 | 0 | 只检查新成员前 N 条（0=全部） |

## 加盐哈希 (crypto.rs)

防止 `SHA256(chat_id)` 暴力碰撞（chat_id 范围 ~10^13）:

- **community_id_hash:** `SHA256(platform + chat_id_le + SHA256(owner_account + "nexus-community-salt"))`
- **platform_user_id_hash:** `SHA256(platform + user_id_str + SHA256(owner_account + "nexus-user-salt"))`

盐值由 owner_account 派生，不可预测。

## 确定性多播算法

```
K = min(ceil(N×2/3), N)，至少 3 个
seed = SHA256(message_hash_hex + sequence_le_bytes)
sorted_nodes = ActiveNodeList.sort_by(node_id)
Fisher-Yates shuffle (seed 驱动) → 取前 K 个
并发 POST /v1/message → 收集结果 (success / failure / timeout)
```

## 文件结构

```
nexus-agent/
├── src/
│   ├── main.rs               # 入口: Axum 服务器 + AppState(9 字段)
│   │                          #   + LocalStore 60s 清理定时器
│   │                          #   + setWebhook 注册
│   ├── config.rs              # AgentConfig (7 字段, 从环境变量加载)
│   │                          #   + bot_id_hash = SHA256(bot_token)
│   ├── signer.rs              # KeyManager (Ed25519 签名: sign_message + sign_raw)
│   │                          #   + SequenceManager (AtomicU64 + atomicwrites 持久化)
│   ├── types.rs               # TelegramUpdate/Message/User/Chat/CallbackQuery
│   │                          #   + SignedMessage + NodeInfo + MulticastResult
│   │                          #   + HealthResponse
│   ├── webhook.rs             # POST /webhook (5 步: secret → parse → local → sign → multicast)
│   │                          #   + POST /v1/execute (4 步验证 + ConfigUpdate/Warn 拦截 + TG API 执行)
│   │                          #   + handle_config_update (10 种配置命令 → ConfigStore 更新)
│   │                          #   + handle_warn_action (warn 计数 + 超限自动升级)
│   │                          #   + execute_local_action (5 种本地动作 → TG API)
│   │                          #   + refresh_admin_cache (getChatAdministrators)
│   │                          #   + submit_audit_log (签名 → 多播审计记录)
│   │                          #   + GET /health
│   ├── multicaster.rs         # 确定性多播 (SHA256 seed + Fisher-Yates)
│   │                          #   + select_k + deterministic_select
│   │                          #   + register_telegram_webhook (3 次重试)
│   ├── executor.rs            # TelegramExecutor (20 种 ActionType → 14 种 TG API)
│   │                          #   + ExecuteAction/ExecuteResult/ActionType(嵌套枚举)
│   │                          #   + ConfigUpdateAction(8变体) + ConfigUpdate处理
│   │                          #   + 签名回执 sign(action_id + method + SHA256(response))
│   ├── group_config.rs        # GroupConfig (26 字段) + SignedGroupConfig
│   │                          #   + ConfigStore (RwLock + JSON 持久化)
│   │                          #   + POST/GET handler + 认证验证
│   │                          #   + broadcast_config_to_nodes (ConfigSync)
│   │                          #   + 6 种枚举 (JoinApprovalPolicy/FloodAction/WarnAction/
│   │                          #     BlacklistMode/BlacklistAction/LockType)
│   ├── local_store.rs         # LocalStore (4 个 RwLock<HashMap>)
│   │                          #   + flood/warn/admin_cache/message_fingerprint
│   │                          #   + cleanup_expired (60s 定时)
│   ├── local_processor.rs     # LocalProcessor (6 级检测链)
│   │                          #   + LocalAction + LocalActionType (5 种)
│   │                          #   + 管理员/白名单/命令豁免
│   ├── rate_limiter.rs        # RateLimiter (AtomicU64 + Mutex<Instant>)
│   └── crypto.rs              # 加盐哈希: community_id_hash + platform_user_id_hash
├── .env.example               # 环境变量模板
├── Cargo.toml                 # 依赖: axum 0.7, reqwest 0.12, ed25519-dalek 2,
│                              #   sha2 0.10, regex 1, atomicwrites 0.4, ...
├── Dockerfile                 # 多阶段构建 (rust:1.82-slim → debian:bookworm-slim)
└── README.md
```

## 依赖

| 类别 | 依赖 | 版本 |
|------|------|------|
| HTTP 服务 | axum | 0.7 |
| CORS/Trace | tower-http | 0.5 |
| HTTP 客户端 | reqwest (rustls-tls) | 0.12 |
| Ed25519 签名 | ed25519-dalek | 2 |
| SHA256 哈希 | sha2 | 0.10 |
| 序列化 | serde / serde_json | 1.0 |
| 随机数 | rand | 0.8 |
| 日志 | tracing / tracing-subscriber | 0.1 / 0.3 |
| 错误处理 | thiserror / anyhow | 1.0 |
| 环境变量 | dotenvy | 0.15 |
| Hex 编码 | hex | 0.4 |
| 时间 | chrono | 0.4 |
| 原子文件写入 | atomicwrites | 0.4 |
| 正则表达式 | regex | 1 |

## 测试

```bash
cargo test    # 90 tests
```

| 模块 | 测试数 | 说明 |
|------|--------|------|
| local_processor | 27 | 12 unit + 11 E2E + 4 bench (1000 msg <500ms) |
| local_store | 11 | flood/warn/admin_cache/duplicate/cleanup |
| group_config | 4 | ConfigStore set/get + CAS 版本冲突 + 持久化 |
| signer | 14 | 密钥生成/加载 + 签名验签 + 序列号持久化 + 并发安全 |
| executor | 1 | 签名回执可验证 |
| crypto | 7 | 加盐哈希确定性 + 不同 owner/platform/chat/user |
| multicaster | 5 | K 值计算 + 确定性选择 + 唯一性 |
| rate_limiter | 2 | 窗口内允许 + 超限阻止 |
| **总计** | **90** | |
