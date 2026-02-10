# Nexus Discord 群组管理开发设计文档

> 版本: v1.0.0  
> 日期: 2026-02-10  
> 状态: **已实现** (Sprint D1-D6 完成)

## 1. 概述

将 Nexus Bot 系统从 Telegram-only 扩展为多平台架构，首先支持 **Discord**。

### 1.1 现有 Telegram 架构回顾

```
┌─────────────┐    Webhook     ┌──────────────┐    Multicast    ┌──────────────┐
│  Telegram    │ ─────────────→│  nexus-agent  │ ──────────────→│  nexus-node  │
│  Bot API     │               │  (Agent)      │                │  (×N 个)     │
│              │←──── TG API ──│               │←── /v1/execute │              │
└─────────────┘               └──────────────┘                └──────────────┘
```

**关键耦合点（需要解耦）：**

| 组件 | Telegram 硬编码 | 文件 |
|------|----------------|------|
| Agent webhook | `x-telegram-bot-api-secret-token` 验证 | `nexus-agent/src/webhook.rs` |
| Agent types | `TelegramUpdate` 结构体, `telegram_update` 字段 | `nexus-agent/src/types.rs` |
| Agent config | `BOT_TOKEN` → `SHA256(bot_token)` = bot_id_hash | `nexus-agent/src/config.rs` |
| Agent executor | `TelegramExecutor`, `call_tg_api()` | `nexus-agent/src/executor.rs` |
| Agent local_processor | TG Update JSON pointer 解析 | `nexus-agent/src/local_processor.rs` |
| Agent multicaster | `register_telegram_webhook()` | `nexus-agent/src/multicaster.rs` |
| Node types | `telegram_update: Value` 字段 | `nexus-node/src/types.rs` |
| Node rule_engine | `build_context()` 解析 TG JSON, `/message/chat/id` 等 | `nexus-node/src/rule_engine.rs` |
| Node leader | `determine_action()` 解析 TG JSON | `nexus-node/src/leader.rs` |

### 1.2 Discord vs Telegram 核心差异

| 特性 | Telegram | Discord |
|------|----------|---------|
| 事件接收 | Webhook (HTTP POST) | **Gateway (WebSocket)** |
| API 调用 | `https://api.telegram.org/bot{token}/{method}` | `https://discord.com/api/v10/{endpoint}` |
| 认证方式 | URL 路径含 Bot Token | `Authorization: Bot {token}` Header |
| 群组模型 | Chat (group/supergroup) | Guild → Channel (多频道) |
| ID 类型 | `i64` | **Snowflake `u64`** (字符串传输) |
| 权限模型 | ChatPermissions (扁平) | **Role-based 分层权限** (位掩码) |
| 消息引用 | `reply_to_message` | `message_reference` |
| 命令系统 | `/command` 文本解析 | **Slash Commands** (Application Commands API) |
| 成员管理 | banChatMember/restrictChatMember | `PUT /guilds/{id}/bans/{user}`, Role 赋予/移除 |
| 入群审核 | `chat_join_request` event | **Member Screening** / Welcome Screen |
| 频率限制 | 无明确公开 | **明确 Rate Limit** (X-RateLimit-* headers) |
| 媒体类型 | 内嵌消息字段 | **Attachment + Embed** 对象 |

## 2. 架构设计

### 2.1 多平台抽象层

**核心原则：** 引入 `PlatformEvent` 统一事件模型，Agent 和 Node 不再直接操作平台特定 JSON。

```
                        ┌─────────────────────────┐
                        │   PlatformEvent (统一)    │
                        │  ├─ platform: String      │
                        │  ├─ guild_id / chat_id    │
                        │  ├─ channel_id (Discord)  │
                        │  ├─ sender_id: String      │
                        │  ├─ text: String           │
                        │  ├─ is_command: bool       │
                        │  ├─ command / args         │
                        │  ├─ reply_to_user_id       │
                        │  ├─ reply_to_message_id    │
                        │  ├─ is_join_event: bool    │
                        │  ├─ raw_event: Value       │
                        │  └─ ...                    │
                        └─────────────────────────┘
                                    ▲
                    ┌───────────────┴───────────────┐
                    │                               │
         ┌──────────────────┐            ┌──────────────────┐
         │ TelegramAdapter   │            │ DiscordAdapter    │
         │ parse(Value)      │            │ parse(Value)      │
         │ → PlatformEvent   │            │ → PlatformEvent   │
         └──────────────────┘            └──────────────────┘
```

### 2.2 双平台整合架构（TG Webhook + Discord Gateway 共存）

Telegram 用 HTTP Webhook，Discord 用 WebSocket Gateway——**两者事件接收机制完全不同，但可以整合在同一个 Agent 进程中**。

#### 2.2.1 为什么可以整合

| 维度 | Telegram | Discord | 整合方式 |
|------|----------|---------|---------|
| 事件接收 | Axum HTTP Handler | tokio WebSocket task | **两者并行运行，各自产生事件** |
| 事件格式 | `TelegramUpdate` JSON | Gateway Dispatch JSON | **各自解析为统一 `PlatformEvent`** |
| 签名+多播 | 相同 | 相同 | **共用 `sign_and_multicast()`** |
| 本地处理 | 相同 | 相同 | **共用 `LocalProcessor`** |
| 共识流程 | 相同 | 相同 | **共用 Node 的 gossip + rule_engine** |
| 执行出口 | `call_tg_api()` | `call_discord_api()` | **按 `platform` 字段分发到不同 Executor** |

> **关键洞察：** 事件入口不同，但进入系统后的数据流 100% 相同。抽象层切入点在「原始事件 → PlatformEvent」这一步。

#### 2.2.2 Agent 进程内部架构

```
                    ┌─────────── 同一个 Agent 进程 ──────────────┐
                    │                                            │
  TG Bot API ──HTTP POST──→ [Axum Webhook Handler]              │
                    │              │                              │
                    │         parse_tg_event()                   │
                    │              │                              │
                    │              ▼                              │
                    │     ┌──────────────┐                       │
                    │     │ PlatformEvent │ (统一格式)             │
                    │     └──────┬───────┘                       │
                    │            │                               │
  Discord ──WSS──→ [Gateway Task (tokio::spawn)]                │
                    │              │                              │
                    │         parse_dc_event()                   │
                    │              │                              │
                    │              ▼                              │
                    │     ┌──────────────┐                       │
                    │     │ PlatformEvent │ (统一格式)             │
                    │     └──────┬───────┘                       │
                    │            │                               │
                    │    ┌───────▼────────┐                      │
                    │    │ event_channel  │ (mpsc::Sender)       │
                    │    │ (TG + DC 汇聚) │                      │
                    │    └───────┬────────┘                      │
                    │            │                               │
                    │            ▼                               │
                    │  ┌─────────────────────┐                   │
                    │  │ Shared Pipeline:     │                   │
                    │  │  1. LocalProcessor   │ ← 反垃圾/欢迎等   │
                    │  │  2. sign_message()   │ ← Ed25519 签名    │
                    │  │  3. multicast_nodes()│ ← 多播到 Node     │
                    │  └─────────────────────┘                   │
                    │                                            │
                    │  ┌─────────────────────┐                   │
                    │  │ /v1/execute Handler  │ ← Leader 回调     │
                    │  │  match platform {    │                   │
                    │  │   "telegram" → TGExe │                   │
                    │  │   "discord"  → DCExe │                   │
                    │  │  }                   │                   │
                    │  └─────────────────────┘                   │
                    └────────────────────────────────────────────┘
```

#### 2.2.3 事件汇聚通道

两个事件源（TG Webhook + Discord Gateway）通过 **同一个 `mpsc` channel** 汇聚：

```rust
// main.rs 启动逻辑
let (event_tx, mut event_rx) = mpsc::channel::<PlatformEvent>(2000);

// ── Telegram 入口 ──
// Webhook Handler 内部:
//   parse TG JSON → PlatformEvent → event_tx.send(event).await

// ── Discord 入口 ──
// Gateway Task 内部:
//   parse Gateway Dispatch → PlatformEvent → event_tx.send(event).await

// ── 统一消费循环 ──
tokio::spawn(async move {
    while let Some(event) = event_rx.recv().await {
        // 1. 本地快速路径 (anti-flood, blacklist, welcome...)
        let local_result = local_processor.process(&event).await;
        if local_result.should_forward {
            // 2. 签名
            let signed = signer.sign_event(&event);
            // 3. 多播到 Node
            multicaster.send_to_nodes(&signed).await;
        }
    }
});
```

#### 2.2.4 Executor 分发

Leader Node 通过 `/v1/execute` 回调 Agent 时，Agent 根据 `platform` 字段选择 Executor：

```rust
// webhook.rs — handle_execute()
pub async fn handle_execute(
    State(state): State<Arc<AppState>>,
    Json(action): Json<ExecuteAction>,
) -> impl IntoResponse {
    // 从 action 中获取 platform 信息（通过 bot_id_hash 查 BotRegistration）
    let platform = state.get_bot_platform(&action.bot_id_hash);

    let result = match platform.as_str() {
        "telegram" => state.tg_executor.as_ref()
            .expect("TG executor not configured")
            .execute(&action, &state.key_manager).await,
        "discord" => state.dc_executor.as_ref()
            .expect("Discord executor not configured")
            .execute(&action, &state.key_manager).await,
        _ => ExecuteResult {
            action_id: action.action_id,
            success: false,
            error: Some(format!("Unknown platform: {}", platform)),
            agent_receipt: None,
        },
    };

    Json(result)
}
```

#### 2.2.5 AppState 扩展

```rust
pub struct AppState {
    // 通用
    pub config: AgentConfig,
    pub key_manager: KeyManager,
    pub sequence_manager: SequenceManager,
    pub config_store: ConfigStore,
    pub local_store: LocalStore,
    pub local_processor: LocalProcessor,
    pub node_list: Vec<String>,

    // 平台 Executor（按需初始化）
    pub tg_executor: Option<TelegramExecutor>,
    pub dc_executor: Option<DiscordExecutor>,

    // 事件通道（TG Webhook + Discord Gateway 共用）
    pub event_tx: mpsc::Sender<PlatformEvent>,

    // Discord 专用
    pub dc_interaction_acker: Option<DiscordInteractionAcker>,
}
```

#### 2.2.6 三种启动模式

```rust
// main.rs
match config.platform {
    PlatformMode::Telegram => {
        // 仅 Axum HTTP: /webhook + /v1/execute + /health
        // tg_executor = Some(...), dc_executor = None
        register_telegram_webhook(&config).await?;
        start_axum_server(state).await?;
    }
    PlatformMode::Discord => {
        // Axum HTTP: /v1/execute + /health (无 /webhook)
        // + tokio::spawn(discord_gateway_task)
        // tg_executor = None, dc_executor = Some(...)
        let gateway = tokio::spawn(run_discord_gateway(config.clone(), event_tx));
        let http = tokio::spawn(start_axum_server(state));
        tokio::try_join!(gateway, http)?;
    }
    PlatformMode::Both => {
        // Axum HTTP: /webhook + /v1/execute + /health
        // + tokio::spawn(discord_gateway_task)
        // tg_executor = Some(...), dc_executor = Some(...)
        register_telegram_webhook(&config).await?;
        let gateway = tokio::spawn(run_discord_gateway(config.clone(), event_tx));
        let http = tokio::spawn(start_axum_server(state));
        tokio::try_join!(gateway, http)?;
    }
}
```

#### 2.2.7 Node 侧整合

Node 不关心事件来源，只关心 `SignedMessage` 的 `platform` 字段：

```rust
// rule_engine.rs — build_context() 改造
fn build_context(message: &SignedMessage, chain_cache: Option<&ChainCache>) -> RuleContext {
    match message.platform.as_str() {
        "telegram" => TelegramPlatform::build_context(message, chain_cache),
        "discord"  => DiscordPlatform::build_context(message, chain_cache),
        _ => RuleContext::default(),
    }
}

// leader.rs — determine_action() 改造
fn determine_action(&self, message: &SignedMessage) -> (ActionType, i64, Value) {
    match message.platform.as_str() {
        "telegram" => TelegramPlatform::determine_action(message),
        "discord"  => DiscordPlatform::determine_action(message),
        _ => (ActionType::NoAction, 0, json!({})),
    }
}
```

> **整合总结：** TG Webhook 和 Discord Gateway 在同一进程中并行运行，通过 `mpsc` channel 汇聚为统一的 `PlatformEvent` 流。后续签名、多播、共识、规则引擎全部共用。仅在最终 API 调用时按 `platform` 分发。这种设计使得未来添加 Slack/Matrix 等平台只需增加一个 Adapter + Executor + Gateway/Webhook 入口。

### 2.3 新增文件结构

```
nexus-agent/
├── src/
│   ├── platform/
│   │   ├── mod.rs              # PlatformAdapter trait + PlatformEvent
│   │   ├── telegram.rs         # TelegramAdapter (从现有代码提取)
│   │   └── discord.rs          # DiscordAdapter (新增)
│   ├── executor/
│   │   ├── mod.rs              # PlatformExecutor trait + ActionType (从现有提取)
│   │   ├── telegram.rs         # TelegramExecutor (从现有 executor.rs 提取)
│   │   └── discord.rs          # DiscordExecutor (新增)
│   ├── gateway/
│   │   └── discord.rs          # Discord Gateway WebSocket 连接管理
│   ├── webhook.rs              # 保留 Telegram webhook + 新增通用入口
│   ├── config.rs               # 扩展支持 DISCORD_BOT_TOKEN
│   └── main.rs                 # 根据配置启动 TG webhook / Discord gateway / 两者
│
nexus-node/
├── src/
│   ├── platform/
│   │   ├── mod.rs              # PlatformAdapter trait (Node 侧)
│   │   ├── telegram.rs         # TG JSON → RuleContext
│   │   └── discord.rs          # Discord JSON → RuleContext
│   ├── rule_engine.rs          # 改用 PlatformEvent 构建 RuleContext
│   ├── leader.rs               # determine_action 改用 PlatformEvent
│   └── types.rs                # SignedMessage.telegram_update → platform_event
```

### 2.3 SignedMessage 演进

**当前:**
```rust
pub struct SignedMessage {
    pub owner_public_key: String,
    pub bot_id_hash: String,
    pub sequence: u64,
    pub timestamp: u64,
    pub message_hash: String,
    pub telegram_update: serde_json::Value,  // ← TG 硬编码
    pub owner_signature: String,
    pub platform: String,                     // ← 已有但未使用
}
```

**目标:**
```rust
pub struct SignedMessage {
    pub owner_public_key: String,
    pub bot_id_hash: String,
    pub sequence: u64,
    pub timestamp: u64,
    pub message_hash: String,
    pub platform_event: serde_json::Value,   // ← 重命名，平台无关
    pub owner_signature: String,
    pub platform: String,                     // "telegram" | "discord"
}
```

> **兼容性:** `telegram_update` → `platform_event` 是 breaking change，但 Agent 和 Node 是配套部署的，不存在跨版本兼容问题。可用 `#[serde(alias = "telegram_update")]` 过渡。

## 3. Discord 具体实现设计

### 3.1 Discord Gateway 连接

Discord 不使用 Webhook（Webhook 仅用于发消息到频道），**必须通过 Gateway WebSocket** 接收事件。

```rust
// nexus-agent/src/gateway/discord.rs

pub struct DiscordGateway {
    token: String,
    intents: u64,
    ws_sender: Option<SplitSink<...>>,
    heartbeat_interval: u64,
    sequence: Option<u64>,         // Gateway sequence (用于 resume)
    session_id: Option<String>,
    bot_id_hash: String,
}

impl DiscordGateway {
    /// 连接 Gateway 并开始接收事件
    pub async fn connect_and_run(
        &mut self,
        event_tx: mpsc::Sender<DiscordEvent>,
    ) -> anyhow::Result<()>;

    /// 心跳循环
    async fn heartbeat_loop(&self, interval: u64);

    /// 处理 Gateway 事件分发
    async fn handle_dispatch(
        &mut self,
        event_name: &str,
        data: serde_json::Value,
        event_tx: &mpsc::Sender<DiscordEvent>,
    );

    /// 断线重连 (Resume)
    async fn resume(&mut self) -> anyhow::Result<()>;
}
```

**Gateway Intents（需要的权限位）：**

```rust
const INTENTS: u64 =
    (1 << 0)  // GUILDS — guild create/update/delete, channel CRUD, role CRUD
  | (1 << 1)  // GUILD_MEMBERS — member add/remove/update (Privileged!)
  | (1 << 9)  // GUILD_MESSAGES — message create/update/delete
  | (1 << 13) // GUILD_MESSAGE_REACTIONS
  | (1 << 15) // MESSAGE_CONTENT (Privileged! — 需要在 Developer Portal 开启)
  | (1 << 20) // AUTO_MODERATION_EXECUTION
  ;
```

> ⚠️ `GUILD_MEMBERS` 和 `MESSAGE_CONTENT` 是 **Privileged Intents**，需要在 Discord Developer Portal 手动开启。Bot 超过 100 个 Guild 后需要申请 verification。

### 3.2 Discord 事件映射

| Discord Gateway Event | 映射到 PlatformEvent | 对应 Telegram 事件 |
|----------------------|---------------------|-------------------|
| `MESSAGE_CREATE` | text / command 消息 | `message` |
| `INTERACTION_CREATE` (type=2) | Slash Command | `/command` 文本 |
| `INTERACTION_CREATE` (type=3) | Button/Select 交互 | `callback_query` |
| `GUILD_MEMBER_ADD` | 入群事件 | `chat_join_request` |
| `GUILD_MEMBER_REMOVE` | 离群事件 | `chat_member` (left) |
| `GUILD_BAN_ADD` | 被封禁 | — |
| `MESSAGE_DELETE` | 消息删除通知 | — |
| `MESSAGE_UPDATE` | 消息编辑 | `edited_message` |

### 3.3 Discord REST API 映射

| 功能 | Telegram API | Discord REST API |
|------|-------------|-----------------|
| 发消息 | `sendMessage` | `POST /channels/{id}/messages` |
| 删消息 | `deleteMessage` | `DELETE /channels/{id}/messages/{id}` |
| 批量删消息 | `deleteMessages` | `POST /channels/{id}/messages/bulk-delete` (≤100, ≤14天) |
| 封禁用户 | `banChatMember` | `PUT /guilds/{id}/bans/{user}` |
| 解封用户 | `unbanChatMember` | `DELETE /guilds/{id}/bans/{user}` |
| 禁言用户 | `restrictChatMember` | `PUT /guilds/{id}/members/{user}` + `communication_disabled_until` 字段 (Timeout) |
| 解除禁言 | `restrictChatMember` (restore) | `PUT /guilds/{id}/members/{user}` + `communication_disabled_until: null` |
| 踢出用户 | ban + unban | `DELETE /guilds/{id}/members/{user}` |
| 置顶消息 | `pinChatMessage` | `PUT /channels/{id}/pins/{msg}` |
| 取消置顶 | `unpinChatMessage` | `DELETE /channels/{id}/pins/{msg}` |
| 设置权限 | `setChatPermissions` | `PUT /channels/{id}/permissions/{overwrite}` (Permission Overwrite) |
| 管理角色 | `promoteChatMember` | `PUT /guilds/{id}/members/{user}` + `roles` 数组 |
| 创建邀请 | `createChatInviteLink` | `POST /channels/{id}/invites` |
| 获取成员信息 | `getChatMember` | `GET /guilds/{id}/members/{user}` |
| 注册命令 | `setMyCommands` | `PUT /applications/{id}/guilds/{guild}/commands` (Slash Commands) |
| 发送嵌入 | — (无原生支持) | `POST /channels/{id}/messages` + `embeds` 数组 |
| 发起投票 | `sendPoll` | 嵌入 + Reaction 模拟 / 使用 Discord Poll API (2024+) |
| 设置频道话题 | `setChatDescription` | `PATCH /channels/{id}` + `topic` 字段 |
| 设置服务器名 | `setChatTitle` | `PATCH /guilds/{id}` + `name` 字段 |

### 3.4 DiscordExecutor 设计

```rust
// nexus-agent/src/executor/discord.rs

pub struct DiscordExecutor {
    bot_token: String,
    application_id: String,
    client: reqwest::Client,
}

impl DiscordExecutor {
    pub async fn execute(
        &self,
        action: &ExecuteAction,
        key_manager: &KeyManager,
    ) -> ExecuteResult;

    /// 调用 Discord REST API
    /// 自动处理 Rate Limit (429 → 等待 retry_after 重试)
    async fn call_discord_api(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> anyhow::Result<(String, serde_json::Value)>;
}
```

**Rate Limit 处理策略：**

```rust
/// Discord Rate Limit 处理
/// 429 Too Many Requests → 读取 retry_after → 等待 → 重试（最多 3 次）
/// X-RateLimit-Remaining = 0 → 预防性等待至 X-RateLimit-Reset
async fn call_discord_api(...) -> Result<...> {
    for attempt in 0..3 {
        let resp = self.client.request(method, &url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .json(&body)
            .send().await?;

        if resp.status() == 429 {
            let retry_after = resp.json::<Value>()["retry_after"]
                .as_f64().unwrap_or(1.0);
            tokio::time::sleep(Duration::from_secs_f64(retry_after)).await;
            continue;
        }
        // ...
    }
}
```

### 3.5 Discord 特有功能

#### 3.5.1 Slash Commands 注册

Discord 推荐使用 Slash Commands 而非文本前缀命令。Bot 启动时需注册：

```rust
/// 注册 Guild-scoped Slash Commands
async fn register_slash_commands(
    &self,
    guild_id: &str,
) -> anyhow::Result<()> {
    let commands = vec![
        json!({"name": "ban", "description": "Ban a user", "options": [
            {"name": "user", "type": 6, "description": "Target user", "required": true},
            {"name": "reason", "type": 3, "description": "Ban reason"}
        ]}),
        json!({"name": "mute", "description": "Timeout a user", "options": [
            {"name": "user", "type": 6, "description": "Target user", "required": true},
            {"name": "duration", "type": 4, "description": "Duration in seconds", "required": true}
        ]}),
        json!({"name": "kick", "description": "Kick a user", "options": [
            {"name": "user", "type": 6, "description": "Target user", "required": true}
        ]}),
        json!({"name": "warn", "description": "Warn a user", "options": [
            {"name": "user", "type": 6, "description": "Target user", "required": true},
            {"name": "reason", "type": 3, "description": "Warn reason"}
        ]}),
        json!({"name": "config", "description": "Bot configuration", "options": [
            {"name": "blacklist", "type": 1, "description": "Manage blacklist", "options": [
                {"name": "action", "type": 3, "description": "add/remove/list", "required": true},
                {"name": "word", "type": 3, "description": "Word to add/remove"}
            ]},
            {"name": "flood", "type": 1, "description": "Set flood limit", "options": [
                {"name": "limit", "type": 4, "description": "Messages per window"}
            ]},
            {"name": "welcome", "type": 1, "description": "Set welcome message", "options": [
                {"name": "message", "type": 3, "description": "Welcome text"}
            ]}
        ]}),
        // ... 更多命令
    ];

    self.call_discord_api(
        Method::PUT,
        &format!("/applications/{}/guilds/{}/commands", self.application_id, guild_id),
        Some(json!(commands)),
    ).await?;

    Ok(())
}
```

#### 3.5.2 Interaction Response (Discord 要求 3 秒内响应)

Discord Slash Command 交互**必须在 3 秒内回复**，否则交互失效。

```rust
/// 快速确认交互（Deferred Response）
/// Discord 要求 3 秒内响应，复杂操作先发 Deferred 再跟进
async fn ack_interaction(&self, interaction_id: &str, interaction_token: &str) {
    // Type 5 = Deferred Channel Message with Source
    self.call_discord_api(
        Method::POST,
        &format!("/interactions/{}/{}/callback", interaction_id, interaction_token),
        Some(json!({"type": 5})),
    ).await.ok();
}

/// 后续编辑交互响应
async fn followup_interaction(
    &self,
    interaction_token: &str,
    content: &str,
) {
    self.call_discord_api(
        Method::POST,
        &format!("/webhooks/{}/{}", self.application_id, interaction_token),
        Some(json!({"content": content})),
    ).await.ok();
}
```

#### 3.5.3 Role-based 权限管理

Discord 使用 Role 体系管理权限，与 Telegram 的扁平权限不同：

```rust
/// Discord 权限位
pub mod discord_permissions {
    pub const KICK_MEMBERS: u64         = 1 << 1;
    pub const BAN_MEMBERS: u64          = 1 << 2;
    pub const ADMINISTRATOR: u64        = 1 << 3;
    pub const MANAGE_CHANNELS: u64      = 1 << 4;
    pub const MANAGE_GUILD: u64         = 1 << 5;
    pub const SEND_MESSAGES: u64        = 1 << 11;
    pub const MANAGE_MESSAGES: u64      = 1 << 13;
    pub const MENTION_EVERYONE: u64     = 1 << 17;
    pub const MANAGE_ROLES: u64         = 1 << 28;
    pub const MODERATE_MEMBERS: u64     = 1 << 40;  // Timeout
}

/// 检查用户是否拥有指定权限
fn has_permission(member_permissions: u64, required: u64) -> bool {
    (member_permissions & discord_permissions::ADMINISTRATOR != 0)
        || (member_permissions & required == required)
}
```

#### 3.5.4 Embed 富文本消息

Discord 的 Embed 功能远强于 Telegram Markdown，可用于规则展示、警告消息等：

```rust
/// 构建 Embed 消息
fn build_warn_embed(user_tag: &str, count: u8, limit: u8, reason: &str) -> Value {
    json!({
        "embeds": [{
            "title": "⚠️ Warning",
            "color": 0xFF9900,  // 橙色
            "fields": [
                {"name": "User", "value": user_tag, "inline": true},
                {"name": "Warnings", "value": format!("{}/{}", count, limit), "inline": true},
                {"name": "Reason", "value": reason},
            ],
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }]
    })
}
```

### 3.6 Guild vs Channel 模型适配

**Telegram:** 一个 Bot 对应一个 Group (chat_id)。

**Discord:** 一个 Bot 对应一个 Guild (server)，Guild 下有多个 Channel。

```
Discord Guild (≈ TG Group)
├── #general          (text channel)
├── #announcements    (text channel)
├── #bot-commands     (text channel) ← Bot 命令聚焦频道
├── Voice Channel 1   (voice channel)
└── ...
```

**设计决策：**

1. **`chat_id` 映射:** 
   - Telegram: `chat_id` = Group ID (`i64`)
   - Discord: `chat_id` = Channel ID (`u64` as `i64`)，Guild ID 额外携带
   
2. **GroupConfig 作用域:**
   - Telegram: 1 config per Group
   - Discord: **1 config per Guild**（全服务器共享配置），频道级配置可选扩展

3. **RuleContext 扩展:**
   ```rust
   pub struct RuleContext {
       // ... 现有字段
       /// Discord Guild ID (None for Telegram)
       pub guild_id: Option<String>,
       /// Discord Channel ID (None for Telegram)
       pub channel_id: Option<String>,
   }
   ```

## 4. PlatformAdapter Trait 设计

### 4.1 Agent 侧

```rust
// nexus-agent/src/platform/mod.rs

/// 平台事件（统一格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformEvent {
    /// 平台类型
    pub platform: String,
    /// 群组/服务器 ID（TG: chat_id, Discord: guild_id）
    pub group_id: String,
    /// 频道 ID（Discord 专用，TG 为空）
    pub channel_id: String,
    /// 发送者 ID
    pub sender_id: String,
    /// 消息文本
    pub text: String,
    /// 消息 ID
    pub message_id: String,
    /// 是否为命令
    pub is_command: bool,
    /// 命令名（不含前缀）
    pub command: Option<String>,
    /// 命令参数
    pub command_args: Option<String>,
    /// 回复目标用户 ID
    pub reply_to_user_id: Option<String>,
    /// 回复目标消息 ID
    pub reply_to_message_id: Option<String>,
    /// 是否为入群事件
    pub is_join_event: bool,
    /// 入群用户 ID
    pub join_user_id: Option<String>,
    /// 是否为交互事件（Discord Slash Command / TG Callback）
    pub is_interaction: bool,
    /// 交互 ID（Discord interaction_id / TG callback_query_id）
    pub interaction_id: Option<String>,
    /// 交互 token（Discord interaction_token）
    pub interaction_token: Option<String>,
    /// 原始平台事件 JSON（保留完整数据）
    pub raw_event: serde_json::Value,
}

/// 平台适配器 Trait (Agent 侧)
pub trait PlatformAdapter: Send + Sync {
    /// 平台名称
    fn platform_name(&self) -> &str;

    /// 解析原始事件为统一格式
    fn parse_event(&self, raw: &serde_json::Value) -> Option<PlatformEvent>;

    /// 计算 bot_id_hash
    fn compute_bot_id_hash(token: &str) -> [u8; 32];
}

/// 平台执行器 Trait (Agent 侧)
#[async_trait::async_trait]
pub trait PlatformExecutor: Send + Sync {
    /// 执行动作
    async fn execute(
        &self,
        action: &ExecuteAction,
        key_manager: &KeyManager,
    ) -> ExecuteResult;

    /// 快速确认交互（Discord 3 秒限制）
    async fn ack_interaction(
        &self,
        _interaction_id: &str,
        _interaction_token: &str,
    ) -> anyhow::Result<()> {
        Ok(()) // Telegram 不需要，默认空实现
    }

    /// 注册平台命令
    async fn register_commands(&self, _group_id: &str) -> anyhow::Result<()> {
        Ok(()) // 可选
    }
}
```

### 4.2 Node 侧

```rust
// nexus-node/src/platform/mod.rs

/// Node 侧平台适配器
pub trait NodePlatformAdapter: Send + Sync {
    /// 从 SignedMessage 提取 RuleContext
    fn build_context(
        &self,
        message: &SignedMessage,
        chain_cache: Option<&ChainCache>,
    ) -> RuleContext;

    /// 从 SignedMessage 判断动作类型 (用于 Leader)
    fn determine_action(
        &self,
        message: &SignedMessage,
    ) -> (ActionType, i64, serde_json::Value);
}
```

## 5. Agent 配置扩展

### 5.1 环境变量

```bash
# ═══ 通用 ═══
PLATFORM=telegram           # "telegram" | "discord" | "both"
CHAIN_RPC=ws://127.0.0.1:9944
DATA_DIR=/data
EXECUTE_TOKEN=secret

# ═══ Telegram 专用（PLATFORM=telegram 或 both 时必填）═══
BOT_TOKEN=123:ABC
WEBHOOK_URL=https://my-server.com:8443
WEBHOOK_PORT=8443
WEBHOOK_SECRET=xxx

# ═══ Discord 专用（PLATFORM=discord 或 both 时必填）═══
DISCORD_BOT_TOKEN=MTIzNDU2Nzg5MDEy...
DISCORD_APPLICATION_ID=123456789012
DISCORD_GATEWAY_INTENTS=33283         # 可选，默认包含必需 intents
DISCORD_COMMAND_CHANNEL=bot-commands  # 可选，限制命令到特定频道
```

### 5.2 Config 结构扩展

```rust
pub struct AgentConfig {
    // 通用
    pub platform: PlatformMode, // Telegram, Discord, Both
    pub chain_rpc: String,
    pub data_dir: String,
    pub execute_token: Option<String>,

    // Telegram
    pub telegram: Option<TelegramConfig>,

    // Discord
    pub discord: Option<DiscordConfig>,
}

pub enum PlatformMode {
    Telegram,
    Discord,
    Both,
}

pub struct TelegramConfig {
    pub bot_token: String,
    pub webhook_port: u16,
    pub webhook_url: String,
    pub webhook_secret: String,
    pub bot_id_hash: [u8; 32],
}

pub struct DiscordConfig {
    pub bot_token: String,
    pub application_id: String,
    pub gateway_intents: u64,
    pub bot_id_hash: [u8; 32],  // SHA256(discord_bot_token)
    pub command_channel: Option<String>,
}
```

## 6. Agent main.rs 启动流程

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AgentConfig::from_env()?;

    // 根据平台模式启动
    match config.platform {
        PlatformMode::Telegram => {
            // 现有逻辑：注册 Webhook + 启动 HTTP 服务器
            start_telegram_agent(config).await?;
        }
        PlatformMode::Discord => {
            // 新逻辑：连接 Gateway + 启动 HTTP 服务器 (仅 /v1/execute + /health)
            start_discord_agent(config).await?;
        }
        PlatformMode::Both => {
            // 两者并行
            let tg = tokio::spawn(start_telegram_agent(config.clone()));
            let dc = tokio::spawn(start_discord_agent(config.clone()));
            tokio::try_join!(tg, dc)?;
        }
    }

    Ok(())
}

async fn start_discord_agent(config: AgentConfig) -> anyhow::Result<()> {
    let dc_config = config.discord.as_ref().unwrap();

    // 1. 连接 Discord Gateway
    let (event_tx, mut event_rx) = mpsc::channel(1000);
    let mut gateway = DiscordGateway::new(
        dc_config.bot_token.clone(),
        dc_config.gateway_intents,
        dc_config.bot_id_hash,
    );

    // 2. Gateway 连接（后台任务，自动重连）
    tokio::spawn(async move {
        loop {
            if let Err(e) = gateway.connect_and_run(event_tx.clone()).await {
                error!(error = %e, "Discord Gateway 断线，5 秒后重连");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    });

    // 3. 事件处理循环
    let discord_adapter = DiscordAdapter::new();
    let discord_executor = DiscordExecutor::new(
        dc_config.bot_token.clone(),
        dc_config.application_id.clone(),
    );

    while let Some(event) = event_rx.recv().await {
        // 解析为 PlatformEvent
        if let Some(platform_event) = discord_adapter.parse_event(&event.data) {
            // 签名 + 多播到 Node
            let signed = sign_and_wrap(&config, &platform_event);
            multicast_to_nodes(&state, &signed).await;

            // 本地快速路径（反垃圾等）
            process_local(&state, &platform_event).await;
        }
    }

    Ok(())
}
```

## 7. 链上 Pallet 变更

### 7.1 pallet-bot-registry — 无需修改

`Platform::Discord` 已存在，`register_bot()` 接受任意 platform。`bot_id_hash` 对于 Discord 同样是 `SHA256(discord_bot_token)`。

### 7.2 pallet-bot-group-mgmt — 无需修改

`ActionType` 枚举是平台无关的（Ban, Mute, DeleteMessage 等），`ActionLog` 使用哈希值存储，不涉及平台特定数据。

### 7.3 pallet-bot-consensus — 无需修改

节点共识机制与平台无关。`SignedMessage` 验证基于 Ed25519 签名和 `bot_id_hash`，与具体平台解耦。

> **结论：链上 Pallet 无需任何修改。** 多平台支持完全在链下层（Agent + Node）实现。

## 8. 开发计划 (Sprint 划分)

### Sprint D1: 平台抽象层重构 (3 天)

**目标：** 提取 TelegramAdapter/TelegramExecutor，引入 PlatformEvent，不破坏现有功能。

1. 创建 `nexus-agent/src/platform/mod.rs` — PlatformEvent + PlatformAdapter trait
2. 创建 `nexus-agent/src/platform/telegram.rs` — 从 `webhook.rs`/`types.rs` 提取 TG 解析逻辑
3. 创建 `nexus-agent/src/executor/mod.rs` — PlatformExecutor trait
4. 重构 `nexus-agent/src/executor/telegram.rs` — 现有 TelegramExecutor 不变，仅实现新 trait
5. 重构 `nexus-node/src/rule_engine.rs` — `build_context()` 改用 `platform` 字段分发
6. 重构 `nexus-node/src/leader.rs` — `determine_action()` 改用 `platform` 分发
7. `SignedMessage.telegram_update` → `platform_event` + `#[serde(alias)]`
8. **验证：** 所有现有测试通过（nexus-agent 90/90, nexus-node 92/92）

### Sprint D2: Discord Gateway 连接 (2 天)

**目标：** 建立与 Discord 的 WebSocket 连接，接收事件。

1. 创建 `nexus-agent/src/gateway/discord.rs` — Gateway 连接 + 心跳 + Resume
2. 扩展 `AgentConfig` — `DISCORD_BOT_TOKEN`, `DISCORD_APPLICATION_ID` 等
3. 扩展 `main.rs` — `PlatformMode` 分发启动
4. 添加依赖：`tokio-tungstenite`, `futures-util`
5. **验证：** 能连接 Discord Gateway 并打印 `READY` 和 `MESSAGE_CREATE` 事件

### Sprint D3: DiscordAdapter + DiscordExecutor (3 天)

**目标：** 实现 Discord 事件解析和 API 调用。

1. 创建 `nexus-agent/src/platform/discord.rs` — DiscordAdapter: parse Gateway events → PlatformEvent
2. 创建 `nexus-agent/src/executor/discord.rs` — DiscordExecutor: 20 个 API 方法实现
3. Rate Limit 处理（429 重试 + bucket 追踪）
4. 创建 `nexus-node/src/platform/discord.rs` — Discord JSON → RuleContext
5. **验证：** 手动测试 /ban, /mute, /kick 等命令

### Sprint D4: Slash Commands + Interaction (2 天)

**目标：** 注册 Discord Slash Commands，处理交互响应。

1. `register_slash_commands()` — 启动时注册全部命令到 Guild
2. Interaction 处理：3 秒内 ACK + 后续 followup
3. Slash Command 参数解析（user mention → user_id, duration 等）
4. Button/Select Menu 交互处理
5. **验证：** Discord 中 `/ban @user` 等 Slash Command 正常工作

### Sprint D5: 完善功能对齐 (2 天)

**目标：** 与 Telegram 功能完全对齐。

1. Welcome message (GUILD_MEMBER_ADD → 发送 Embed)
2. Anti-flood (Discord 侧消息频率检测)
3. Blacklist/Lock (消息内容过滤)
4. Warn 系统 (warn count + auto-escalation)
5. Role 管理命令 (/role add/remove)
6. Embed 格式化（规则展示、警告、帮助等）
7. **验证：** 所有 GroupConfig 功能在 Discord 上可用

### Sprint D6: 测试 + 部署 (2 天)

**目标：** 完整测试覆盖 + Docker 部署支持。

1. Discord 单元测试 (adapter parse, executor API 映射, rate limit)
2. E2E 测试脚本
3. 更新 `nexus-deploy/docker-compose.yml` — Discord Agent 配置
4. 更新文档: README, `.env.example`
5. **验证：** cargo test 全量通过，Docker 部署正常

**总预估工期：14 天**

## 9. Discord 命令对照表

| 命令 | Telegram | Discord Slash Command | 说明 |
|------|----------|----------------------|------|
| 封禁 | `/ban` (reply) | `/ban user:@user reason:xxx` | Discord 用 mention 选择目标 |
| 解封 | `/unban` (reply) | `/unban user:@user` | |
| 禁言 | `/mute 3600` (reply) | `/mute user:@user duration:3600` | Discord Timeout (最长 28 天) |
| 解除禁言 | `/unmute` (reply) | `/unmute user:@user` | |
| 踢出 | `/kick` (reply) | `/kick user:@user` | |
| 警告 | `/warn` (reply) | `/warn user:@user reason:xxx` | |
| 查询警告 | `/warns` | `/warns user:@user` | |
| 删除消息 | `/del` (reply) | 右键菜单 / `/purge count:10` | Discord 支持批量清理 |
| 置顶 | `/pin` (reply) | `/pin message_id:xxx` | |
| 黑名单 | `/blacklist word` | `/config blacklist action:add word:xxx` | 子命令组 |
| 防刷屏 | `/flood 5` | `/config flood limit:5` | |
| 欢迎消息 | `/welcome text` | `/config welcome message:xxx` | |
| 权限 | `/perms send_messages off` | `/perms permission:send_messages value:off` | |
| 投票 | `/poll Q\|A\|B` | `/poll question:Q option1:A option2:B` | |
| 帮助 | `/help` | `/help` | 输出 Embed 格式 |
| 规则 | `/rules` | `/rules` | 输出 Embed 格式 |

## 10. 安全考虑

### 10.1 Token 安全

- Discord Bot Token 与 Telegram Bot Token 同等处理：**永不上链**，仅存在 Agent 内存
- `bot_id_hash = SHA256(discord_bot_token)` 作为链上标识
- Gateway 连接使用 WSS (TLS)

### 10.2 Privileged Intents

- `GUILD_MEMBERS` 和 `MESSAGE_CONTENT` 需要在 Discord Developer Portal 手动开启
- Bot 加入超过 100 个 Guild 后需要 Discord 审核（verification）
- 文档中需要明确说明这些前置要求

### 10.3 Rate Limit 保护

- Discord 有严格的 Rate Limit，违规会导致 IP ban
- 实现 bucket-based rate limit tracking
- 全局限制：50 requests/second
- 每 endpoint 独立限制（通过 `X-RateLimit-Bucket` header 追踪）

### 10.4 Permission Escalation 防护

- 与 Telegram 的 `check_target_not_admin()` 类似
- Discord: 检查目标用户的最高 Role 是否高于 Bot Role
- Bot 无法操作比自己 Role 更高的用户

```rust
/// 检查 Bot 是否有权操作目标用户
async fn check_bot_can_manage(&self, guild_id: &str, target_user_id: &str) -> Result<(), String> {
    let bot_member = self.get_guild_member(guild_id, "@me").await?;
    let target_member = self.get_guild_member(guild_id, target_user_id).await?;

    let bot_top_role = self.get_highest_role_position(guild_id, &bot_member.roles).await?;
    let target_top_role = self.get_highest_role_position(guild_id, &target_member.roles).await?;

    if target_top_role >= bot_top_role {
        return Err("目标用户 Role 等级 >= Bot，无法执行管理操作".into());
    }
    Ok(())
}
```

## 11. 依赖清单

### nexus-agent 新增依赖

```toml
# Discord Gateway (WebSocket)
tokio-tungstenite = { version = "0.24", features = ["native-tls"] }
futures-util = "0.3"

# 可选：Discord 专用库（如果不想手写 Gateway）
# twilight-gateway = "0.15"      # 轻量 Discord Gateway 库
# twilight-http = "0.15"         # 轻量 Discord HTTP 库
# 或 serenity = "0.12"           # 全功能 Discord 框架（较重）
```

**推荐方案：** 手写 Gateway + reqwest HTTP 调用（与 Telegram 实现风格一致，避免引入重框架）。若后续需要加速开发，可考虑 `twilight-gateway` (仅 Gateway 层，不侵入业务逻辑)。

## 12. 风险与 TODO

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| Privileged Intents 审核 | Bot 超 100 guilds 需 Discord 审核 | 提前申请，提供合规说明 |
| Gateway 断线重连 | 丢失事件 | Resume 机制 + 事件序列号追踪 |
| Rate Limit 被封 | IP 全局 ban | 严格遵守 rate limit，添加 bucket tracking |
| Slash Command 注册延迟 | 全局命令最长 1 小时生效 | 使用 Guild-scoped 命令（即时生效） |
| Discord API 版本更新 | v10 → v11 可能 breaking | 锁定 API 版本号 (`/api/v10/`) |
| Message Content Intent 可能收紧 | 未来可能需要额外审核 | 优先使用 Slash Commands，减少对消息文本的依赖 |

## 13. 实现状态总结 (2026-02-10)

### 已完成的 Sprint

| Sprint | 内容 | 状态 |
|--------|------|------|
| D1 | 平台抽象层 (PlatformEvent, PlatformAdapter, PlatformExecutor, NodePlatformAdapter, SignedMessage 重命名, AgentConfig) | ✅ |
| D2 | Discord Gateway (WebSocket + 心跳 + Resume + 事件解析) | ✅ |
| D3.1 | DiscordAdapter — Agent 侧事件解析 | ✅ |
| D3.2 | DiscordExecutor — 20 REST API 方法 + Rate Limit + sign_receipt | ✅ |
| D3.3 | DiscordNodeAdapter — Node 侧事件解析 + determine_action | ✅ |
| D4 | main.rs 集成 — 条件启动 TG/Discord, Gateway 事件循环, 多播 | ✅ |
| D5 | 功能对齐 — platform routing, rule_engine + leader 通过 NodePlatformAdapter 分发 | ✅ |
| D6 | E2E 测试 + Docker 部署配置 | ✅ |

### 新增/修改文件清单

**nexus-agent:**
- `src/platform/mod.rs` — PlatformEvent + PlatformAdapter trait
- `src/platform/telegram.rs` — TelegramAdapter
- `src/platform/discord.rs` — DiscordAdapter
- `src/gateway/mod.rs` + `src/gateway/discord.rs` — DiscordGateway (WebSocket)
- `src/discord_executor.rs` — DiscordExecutor (20 REST API + rate limit)
- `src/executor.rs` — PlatformExecutor trait + ExecuteAction.platform 字段
- `src/config.rs` — PlatformMode + DiscordConfig
- `src/webhook.rs` — platform routing (TG/Discord executor)
- `src/main.rs` — 条件启动 + AppState.discord_executor
- `.env.example` — Discord 环境变量
- `Cargo.toml` — tokio-tungstenite, futures-util, url

**nexus-node:**
- `src/platform/mod.rs` — NodePlatformAdapter trait + get_adapter()
- `src/platform/telegram.rs` — TelegramNodeAdapter
- `src/platform/discord.rs` — DiscordNodeAdapter
- `src/rule_engine.rs` — build_context 通过适配器分发 + BlacklistRule 平台无关 msg_id + CommandRule 支持 Discord slash args
- `src/leader.rs` — determine_action 通过适配器分发 + ExecuteAction.platform 字段

**部署:**
- `nexus-deploy/docker-compose.yml` — PLATFORM/DISCORD_* 环境变量

### 测试统计

- **nexus-agent**: 127 tests ✅
- **nexus-node**: 132 tests ✅ (含 6 个 Discord E2E 测试)
