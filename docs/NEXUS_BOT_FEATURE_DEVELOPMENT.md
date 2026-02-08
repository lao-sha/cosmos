# Nexus Bot 功能开发指南

> **基于 6 个 Telegram 群管项目深度分析 + Nexus 现有架构评估的最优开发方案。**
>
> **核心结论：不需要重新架构，在现有 `Rule` trait 扩展点上逐步填充功能。**

---

## 一、架构现状与设计原则

### 1.1 现有架构（不动）

```
Telegram ──Webhook──→ Agent ──multicast──→ Node × K
                        │                    │
                        │                    ├─ RuleEngine.evaluate()
                        │                    ├─ Gossip 共识 (M/K)
                        │                    └─ Leader → Agent /v1/execute
                        │                              │
                        ◄──────────────────────────────┘
                        │
                        └─ TelegramExecutor.execute() → TG API
```

### 1.2 新增：双模式执行

```
Agent 收到 TG Update
       │
  ┌────┴───────────────────────────────────────────────────┐
  │  本地快速路径 (Agent 直接处理, <10ms)                     │
  │                                                         │
  │  ├─ AntifloodCheck  — 内存计数器，超限 → 自动 mute       │
  │  ├─ BlacklistCheck  — 本地词表匹配 → 删除消息             │
  │  ├─ SpamCheck       — 多层检测 → 删除 + mute             │
  │  ├─ WelcomeSend     — 新成员 → 发送欢迎消息               │
  │  ├─ QueryResponse   — /info /id /rules → 直接回复         │
  │  └─ 事后异步: 提交审计记录到 Node 网络                     │
  │                                                         │
  └────┬───────────────────────────────────────────────────┘
       │ (管理类操作仍走共识)
  ┌────┴───────────────────────────────────────────────────┐
  │  共识路径 (Node M/K 确认, 1-3s)                          │
  │                                                         │
  │  ├─ /ban /mute /kick /unban /unmute                     │
  │  ├─ /warn (累积 + 自动升级)                               │
  │  ├─ /promote /demote                                    │
  │  ├─ /gban (全局封禁 → 链上 extrinsic)                    │
  │  ├─ /lock /unlock                                       │
  │  └─ GroupConfig 变更                                     │
  └─────────────────────────────────────────────────────────┘
```

### 1.3 设计原则

| # | 原则 | 说明 |
|---|---|---|
| 1 | **只加不改** | 新功能 = 新 `impl Rule`，不修改现有 Rule 代码 |
| 2 | **配置驱动** | 所有功能的开关和参数都在 `GroupConfig` 中，通过 API 配置 |
| 3 | **快慢分离** | 检测类（antiflood/spam/blacklist）走本地快速路径；管理类走共识 |
| 4 | **审计优先** | 本地快速路径执行的动作也异步提交审计记录 |
| 5 | **参考不抄** | 从 6 个项目借鉴设计模式，但用 Rust 惯用方式重新实现 |

---

## 二、参考项目知识图谱

> 以下是从 6 个项目中提取的最佳实践，直接映射到 Nexus 实现。

### 2.1 FallenRobot → Nexus 功能映射

| FallenRobot 模块 | 核心逻辑 | Nexus 实现位置 |
|---|---|---|
| `antiflood.py` — 5 种处罚模式 | 内存 `CHAT_FLOOD` 字典 + SQL 持久化阈值 | `AntifloodRule` + `LocalStore` |
| `bans.py` — 权限前置检查 | `chat_status.py` 的 `@user_admin` 装饰器 | `CommandRule` 扩展权限验证 |
| `warns.py` — 累积警告 + 自动升级 | SQL warns 表 + `warn_limit` 配置 | `WarnRule` + `LocalStore` |
| `locks.py` — 15 种消息类型锁定 | `LOCK_TYPES` 字典 + `restrictChatMember` | `LockRule` + `GroupConfig.lock_types` |
| `welcome.py` — 多媒体欢迎 | SQL 存储 welcome_type + content | `WelcomeRule` + `GroupConfig.welcome_message` |
| `global_bans.py` — 跨群封禁 | 遍历所有群 + SQL gban 表 | 链上 `pallet-bot-group-mgmt` extrinsic |
| `blacklist.py` — 关键词过滤 | SQL blacklist 表 + 正则匹配 | `BlacklistRule` + `GroupConfig.blacklist_words` |

### 2.2 tg-spam → Nexus 反垃圾映射

| tg-spam 检测层 | 算法 | Nexus 实现 |
|---|---|---|
| CAS API | HTTP 查询 Combot 黑名单 | `SpamDetectorRule` 子检查器 |
| Stop Words | 关键词精确/模糊匹配 | 复用 `BlacklistRule` |
| Similarity | 与已知垃圾样本文本相似度 | `SpamDetectorRule` 子检查器 |
| Classifier | 朴素贝叶斯 spam/ham 分类 | 可选：Agent 侧插件 |
| Meta Checks | emoji 计数、多语言混合、异常空格 | `SpamDetectorRule` 子检查器 |
| Duplicate | 重复消息检测（窗口+阈值） | `SpamDetectorRule` 子检查器 |
| OpenAI Veto | AI 二次确认 | 可选：Agent 侧 HTTP 调用 |
| Lua Plugin | 用户自定义脚本 | 远期：WASM 插件 |

### 2.3 tgbot → Nexus 权限映射

| tgbot 权限层 | Nexus 对应 |
|---|---|
| `OWNER_ID` | `GroupConfig.admins[0]` (owner) |
| `SUDO_USERS` | Agent Ed25519 公钥持有者 |
| `SUPPORT_USERS` | `GroupConfig.admins` 列表 |
| `WHITELIST_USERS` | `GroupConfig.whitelist` 列表 |
| `is_admin()` | `RuleContext` + TG `getChatAdministrators` 缓存 |

---

## 三、GroupConfig 扩展规范

### 3.1 新增字段

在 `nexus-node/src/types.rs` 和 `nexus-agent/src/group_config.rs` 的 `GroupConfig` 中新增：

```rust
pub struct GroupConfig {
    // ═══ 现有字段（不动）═══
    pub version: u64,
    pub bot_id_hash: String,
    pub join_policy: JoinApprovalPolicy,
    pub filter_links: bool,
    pub restrict_mentions: bool,
    pub rate_limit_per_minute: u16,
    pub auto_mute_duration: u64,
    pub new_member_restrict_duration: u64,
    pub welcome_message: String,
    pub whitelist: Vec<String>,
    pub admins: Vec<String>,
    pub quiet_hours_start: Option<u8>,
    pub quiet_hours_end: Option<u8>,
    pub updated_at: u64,

    // ═══ 新增字段 ═══

    // --- 防刷屏 ---
    /// 防刷屏阈值（N 条/时间窗口，0 = 关闭）
    pub antiflood_limit: u16,
    /// 防刷屏时间窗口（秒）
    pub antiflood_window: u16,
    /// 防刷屏动作
    pub antiflood_action: FloodAction,

    // --- 警告系统 ---
    /// 警告上限（达到后执行 warn_action）
    pub warn_limit: u8,
    /// 超限动作
    pub warn_action: WarnAction,

    // --- 黑名单 ---
    /// 黑名单关键词列表
    pub blacklist_words: Vec<String>,
    /// 黑名单匹配模式
    pub blacklist_mode: BlacklistMode,
    /// 黑名单触发动作
    pub blacklist_action: BlacklistAction,

    // --- 锁定系统 ---
    /// 锁定的消息类型列表
    pub lock_types: Vec<LockType>,

    // --- 反垃圾 ---
    /// 反垃圾检测开关
    pub spam_detection_enabled: bool,
    /// 反垃圾：最大 emoji 数
    pub spam_max_emoji: u8,
    /// 反垃圾：相似度阈值 (0.0-1.0)
    pub spam_similarity_threshold: f32,
    /// 反垃圾：只检查新成员的前 N 条消息 (0 = 检查所有)
    pub spam_first_messages_only: u8,
}
```

### 3.2 新增枚举类型

```rust
/// 防刷屏动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FloodAction {
    Mute,       // 禁言（默认）
    Kick,       // 踢出
    Ban,        // 封禁
    DeleteOnly, // 仅删除消息
}

/// 警告超限动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarnAction {
    Ban,   // 封禁（默认）
    Kick,  // 踢出
    Mute,  // 禁言
}

/// 黑名单匹配模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlacklistMode {
    Exact,     // 精确匹配
    Contains,  // 包含匹配（默认）
    Regex,     // 正则匹配
}

/// 黑名单触发动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlacklistAction {
    Delete,          // 仅删除（默认）
    DeleteAndWarn,   // 删除 + 警告
    DeleteAndMute,   // 删除 + 禁言
    DeleteAndBan,    // 删除 + 封禁
}

/// 可锁定的消息类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockType {
    Audio,
    Video,
    Photo,
    Document,
    Sticker,
    Gif,
    Url,
    Forward,
    Voice,
    Contact,
    Location,
    Poll,
    Game,
    Inline,
}
```

### 3.3 GroupConfig API 扩展

`POST /v1/group-config` 请求体新增对应字段（均为 `Option<T>`，增量更新）。

---

## 四、Agent 本地状态层设计

### 4.1 LocalStore 结构

新增文件：`nexus-agent/src/local_store.rs`

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;

/// Agent 本地状态存储
///
/// 用于高频、低延迟的检测操作（防刷屏、警告计数等）。
/// 不需要共识，但会异步提交审计记录到 Node 网络。
pub struct LocalStore {
    /// 防刷屏计数器: (chat_id, user_id) → FloodCounter
    flood_counters: RwLock<HashMap<(i64, i64), FloodCounter>>,

    /// 警告计数: (chat_id, user_id) → warn_count
    warn_counts: RwLock<HashMap<(i64, i64), u8>>,

    /// 管理员缓存: chat_id → (admin_list, expire_time)
    admin_cache: RwLock<HashMap<i64, AdminCacheEntry>>,

    /// 最近消息指纹: chat_id → Vec<(user_id, hash, time)>
    /// 用于重复消息检测
    recent_messages: RwLock<HashMap<i64, Vec<MessageFingerprint>>>,
}

/// 防刷屏计数器
struct FloodCounter {
    count: u16,
    window_start: Instant,
}

/// 管理员缓存条目
struct AdminCacheEntry {
    admin_ids: Vec<i64>,
    cached_at: Instant,
    ttl_seconds: u64,
}

/// 消息指纹（用于重复检测）
struct MessageFingerprint {
    user_id: i64,
    text_hash: u64,
    timestamp: Instant,
}
```

### 4.2 LocalStore 核心方法

```rust
impl LocalStore {
    /// 检查并更新防刷屏计数
    /// 返回 true = 触发防刷屏
    pub fn check_flood(&self, chat_id: i64, user_id: i64, limit: u16, window_secs: u16) -> bool;

    /// 增加警告计数，返回当前计数
    pub fn add_warn(&self, chat_id: i64, user_id: i64) -> u8;

    /// 减少警告计数
    pub fn remove_warn(&self, chat_id: i64, user_id: i64) -> u8;

    /// 重置警告
    pub fn reset_warns(&self, chat_id: i64, user_id: i64);

    /// 获取警告数
    pub fn get_warns(&self, chat_id: i64, user_id: i64) -> u8;

    /// 检查是否为管理员（带 TTL 缓存）
    pub fn is_admin_cached(&self, chat_id: i64, user_id: i64) -> Option<bool>;

    /// 更新管理员缓存
    pub fn set_admin_cache(&self, chat_id: i64, admin_ids: Vec<i64>);

    /// 记录消息指纹，返回重复次数
    pub fn record_message(&self, chat_id: i64, user_id: i64, text: &str) -> u32;

    /// 清理过期数据（定时任务调用）
    pub fn cleanup_expired(&self);
}
```

### 4.3 集成到 AppState

```rust
// nexus-agent/src/main.rs
pub struct AppState {
    // ... 现有字段 ...
    pub local_store: LocalStore,  // 新增
}
```

---

## 五、Rule 扩展实现规范

### 5.1 规则链完整设计

```
                        RuleEngine.evaluate()
                              │
    ┌─────────────────────────┼─────────────────────────────┐
    │                         │                             │
    ▼                         ▼                             ▼
JoinRequestRule         CommandRule              (非命令消息)
 ├─ AutoApprove         ├─ /ban → Ban             │
 ├─ TokenGating(TODO)   ├─ /mute → Mute           ▼
 └─ Captcha(TODO)       ├─ /unmute → Unmute    AntifloodRule ──(新)
                        ├─ /kick → Ban          │ 超限 → Mute/Ban
                        ├─ /pin → Pin           ▼
                        ├─ /del → Delete       BlacklistRule ──(新)
                        ├─ /warn → Warn(新)     │ 命中 → Delete+Warn
                        ├─ /unwarn → (新)       ▼
                        ├─ /warns → (新)       SpamDetectorRule ──(新)
                        ├─ /lock → (新)         │ 垃圾 → Delete+Mute
                        ├─ /unlock → (新)       ▼
                        ├─ /welcome → (新)     LockRule ──(新)
                        ├─ /rules → (新)        │ 类型不允许 → Delete
                        └─ /help → Send         ▼
                                              WelcomeRule ──(新)
                                                │ 新成员 → Send welcome
                                                ▼
                                              LinkFilterRule (现有)
                                                │
                                                ▼
                                              DefaultRule (现有)
                                                │
                                                ▼
                                              NoAction
```

### 5.2 AntifloodRule

**参考来源：** FallenRobot `antiflood.py` (5 种处罚模式) + tg-spam `DuplicateDetection`

```rust
/// 防刷屏规则
///
/// 在 Agent 本地快速路径执行。
/// 逻辑：统计每个用户在时间窗口内的消息数，超过阈值触发动作。
///
/// 参考: FallenRobot/modules/antiflood.py
///   - CHAT_FLOOD 内存字典 (chat_id → {user_id → [count, limit]})
///   - 5 种 flood_type: ban/kick/mute/tban/tmute
///
/// Nexus 改进:
///   - 使用 LocalStore 替代全局字典
///   - 动作由 GroupConfig.antiflood_action 配置
///   - 白名单豁免 (GroupConfig.whitelist + admins)
pub struct AntifloodRule;

impl Rule for AntifloodRule {
    fn name(&self) -> &str { "antiflood" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        // 1. 检查开关: antiflood_limit > 0
        // 2. 跳过命令和系统消息
        // 3. 跳过白名单和管理员
        // 4. 调用 local_store.check_flood()
        // 5. 超限 → 根据 antiflood_action 返回 Mute/Ban/Kick
        // 6. 同时删除触发消息
        todo!()
    }
}
```

**GroupConfig 依赖字段：**
- `antiflood_limit` — 阈值 (默认 0 = 关闭)
- `antiflood_window` — 时间窗口秒数 (默认 10)
- `antiflood_action` — FloodAction 枚举

**LocalStore 依赖方法：**
- `check_flood(chat_id, user_id, limit, window)`

---

### 5.3 WarnRule

**参考来源：** FallenRobot `warns.py` (18K 行) + tgbot `warns.py`

```rust
/// 警告系统规则
///
/// 命令触发 → 共识路径执行。
/// Agent LocalStore 维护 warn 计数，达到上限自动升级为 ban/kick/mute。
///
/// 参考: FallenRobot/modules/warns.py
///   - /warn → 增加警告 + 检查上限 → 超限自动 ban/kick
///   - /warns → 查看当前警告数
///   - /resetwarns → 重置警告
///   - warn_limit 可配置 (默认 3)
///   - warn_action: ban 或 kick
///
/// 支持的命令:
///   /warn  [reply]          — 警告用户
///   /unwarn [reply]         — 移除一次警告
///   /warns [reply]          — 查看警告数
///   /resetwarns [reply]     — 重置警告
///   /setwarnlimit <N>       — 设置警告上限
///   /setwarnaction <action> — 设置超限动作
pub struct WarnRule;

impl Rule for WarnRule {
    fn name(&self) -> &str { "warn" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        // 仅处理 /warn /unwarn /warns /resetwarns 命令
        // /warn:
        //   1. 检查权限: 发送者必须是 admin
        //   2. 检查目标: reply_to_user_id 不能是 admin/whitelist
        //   3. local_store.add_warn() → 获取当前计数
        //   4. 如果 count >= warn_limit → 返回 warn_action 对应的动作
        //   5. 否则 → 返回 Send (回复警告消息)
        todo!()
    }
}
```

**GroupConfig 依赖字段：**
- `warn_limit` — 上限 (默认 3)
- `warn_action` — WarnAction 枚举

**LocalStore 依赖方法：**
- `add_warn()`, `remove_warn()`, `get_warns()`, `reset_warns()`

---

### 5.4 BlacklistRule

**参考来源：** FallenRobot `blacklist.py` + tg-spam `stop_words`

```rust
/// 黑名单关键词过滤规则
///
/// 在 Agent 本地快速路径执行。
/// 检查消息文本是否包含黑名单关键词。
///
/// 参考: FallenRobot/modules/blacklist.py
///   - SQL blacklist 表 (chat_id, trigger)
///   - 支持正则匹配
///   - 触发后删除消息 + 可选 warn/mute/ban
///
/// 参考: tg-spam/lib/tgspam/detector.go
///   - stopWords []string 精确匹配
///   - tokenizedSpam 相似度匹配
///
/// 支持的命令:
///   /blacklist <word>     — 添加黑名单词
///   /unblacklist <word>   — 移除黑名单词
///   /blacklists           — 查看当前黑名单
pub struct BlacklistRule;

impl Rule for BlacklistRule {
    fn name(&self) -> &str { "blacklist" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        // 1. 获取 GroupConfig.blacklist_words
        // 2. 根据 blacklist_mode 匹配:
        //    - Exact: 精确匹配
        //    - Contains: text.to_lowercase().contains(word)
        //    - Regex: regex::Regex 匹配
        // 3. 匹配成功 → 根据 blacklist_action:
        //    - Delete: 删除消息
        //    - DeleteAndWarn: 删除 + warn
        //    - DeleteAndMute: 删除 + mute
        //    - DeleteAndBan: 删除 + ban
        todo!()
    }
}
```

**GroupConfig 依赖字段：**
- `blacklist_words` — 词表
- `blacklist_mode` — 匹配模式
- `blacklist_action` — 触发动作

---

### 5.5 LockRule

**参考来源：** FallenRobot `locks.py` (22K 行)

```rust
/// 消息类型锁定规则
///
/// 在 Agent 本地快速路径执行。
/// 检查消息类型是否在锁定列表中。
///
/// 参考: FallenRobot/modules/locks.py
///   LOCK_TYPES = {
///     "audio", "voice", "contact", "video", "document",
///     "photo", "gif", "url", "bots", "forward", "game",
///     "location", "rtl", "button", "egame", "phone",
///     "dice", "wperm", "pin", "info", "invite",
///     "anonchannel", "forwardchannel", "forwardbot", "sticker"
///   }
///
/// 支持的命令:
///   /lock <type>     — 锁定消息类型
///   /unlock <type>   — 解锁消息类型
///   /locks           — 查看当前锁定列表
pub struct LockRule;

impl Rule for LockRule {
    fn name(&self) -> &str { "lock" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        // 1. 获取 GroupConfig.lock_types
        // 2. 检测消息类型 (从 telegram_update JSON 提取):
        //    - photo: update.message.photo is_some
        //    - video: update.message.video is_some
        //    - audio: update.message.audio is_some
        //    - document: update.message.document is_some
        //    - sticker: update.message.sticker is_some
        //    - gif: update.message.animation is_some
        //    - voice: update.message.voice is_some
        //    - forward: update.message.forward_from is_some
        //    - url: 文本含 http/https/t.me (已有 LinkFilterRule)
        //    - contact: update.message.contact is_some
        //    - location: update.message.location is_some
        //    - poll: update.message.poll is_some
        //    - game: update.message.game is_some
        // 3. 类型在锁定列表中 → Delete
        todo!()
    }
}
```

---

### 5.6 WelcomeRule

**参考来源：** FallenRobot `welcome.py` (42K 行)

```rust
/// 欢迎消息规则
///
/// 在 Agent 本地快速路径执行（只读操作）。
/// 新成员加入时发送欢迎消息。
///
/// 参考: FallenRobot/modules/welcome.py
///   - 支持 text/sticker/photo/audio/voice/video/document
///   - 支持变量替换: {first}, {last}, {fullname}, {username}, {id}, {chatname}
///   - 支持 Markdown/HTML 格式化
///
/// Nexus 简化: MVP 仅支持文本欢迎 + 变量替换
pub struct WelcomeRule;

impl Rule for WelcomeRule {
    fn name(&self) -> &str { "welcome" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        // 1. 检测是否为 new_chat_members 事件
        // 2. 获取 GroupConfig.welcome_message (空 = 不发送)
        // 3. 变量替换:
        //    {first} → 用户名
        //    {chatname} → 群名
        //    {id} → user_id
        // 4. 返回 Message(Send) 动作
        todo!()
    }
}
```

---

### 5.7 SpamDetectorRule

**参考来源：** tg-spam 多层检测引擎

```rust
/// 反垃圾检测规则
///
/// 在 Agent 本地快速路径执行。
/// 多层检测链，任一层命中即判定为垃圾。
///
/// 参考: tg-spam/lib/tgspam/detector.go
///   7 层检测: CAS → StopWords → Similarity → Classifier
///            → MetaChecks → Duplicate → OpenAI
///
/// Nexus MVP 实现 3 层:
///   1. 黑名单词匹配 (复用 BlacklistRule 结果)
///   2. 元数据检查 (emoji 过多、多语言混合)
///   3. 重复消息检测 (LocalStore)
///
/// 远期扩展:
///   4. CAS API 查询
///   5. 贝叶斯分类器
///   6. AI 二次确认
pub struct SpamDetectorRule;

impl Rule for SpamDetectorRule {
    fn name(&self) -> &str { "spam_detector" }

    fn evaluate(&self, ctx: &RuleContext) -> Option<RuleAction> {
        // 1. 检查开关: spam_detection_enabled
        // 2. 跳过管理员和白名单
        // 3. 检查 spam_first_messages_only (可选: 只检查新用户前 N 条)
        // 4. 子检查器链:
        //    a. emoji_check: emoji 数 > spam_max_emoji → spam
        //    b. duplicate_check: local_store.record_message() 重复次数 > 3 → spam
        //    c. multi_lang_check: 检测混合语言 → spam
        // 5. 任一命中 → Delete + Mute
        todo!()
    }
}
```

---

## 六、ActionType 扩展

### 6.1 新增管理动作

```rust
// nexus-node/src/types.rs — AdminAction 扩展
pub enum AdminAction {
    // 现有
    Ban,
    Unban,
    Mute,
    Unmute,
    ApproveJoinRequest,
    DeclineJoinRequest,
    SetPermissions,

    // 新增
    /// 踢出用户（ban + 立即 unban）
    Kick,
    /// 提升为管理员
    Promote,
    /// 降级管理员
    Demote,
}
```

### 6.2 Executor 扩展

在 `nexus-agent/src/executor.rs` 的 `execute()` 方法中新增对应分支：

```rust
AdminAction::Kick => {
    // 1. banChatMember
    // 2. unbanChatMember (only_if_banned: true)
}
AdminAction::Promote => {
    // promoteChatMember with permissions
}
AdminAction::Demote => {
    // promoteChatMember with all false
}
```

---

## 七、Webhook 快速路径扩展

### 7.1 handle_webhook 修改

当前 `webhook.rs` 的 `handle_webhook` 是纯转发模式。需要在转发前插入本地快速路径：

```rust
pub async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> StatusCode {
    // ... 现有: 验证 secret, 解析 update, 签名 ...

    // ═══ 新增: 本地快速路径 ═══
    let local_actions = state.local_processor.process(&update, &state).await;
    for action in &local_actions {
        // 直接执行（不走共识）
        state.executor.execute_local(action, &state.key_manager).await;
    }

    // ═══ 现有: 异步多播到节点（管理类操作由节点决策）═══
    if should_forward_to_nodes(&update, &local_actions) {
        tokio::spawn(async move {
            crate::multicaster::multicast_to_nodes(&state_clone, &signed_message).await;
        });
    }

    StatusCode::OK
}
```

### 7.2 LocalProcessor

新增文件：`nexus-agent/src/local_processor.rs`

```rust
/// 本地快速路径处理器
///
/// 在 Webhook 收到消息后、转发到 Node 之前执行。
/// 处理不需要共识的高频操作。
pub struct LocalProcessor {
    antiflood: AntifloodChecker,
    blacklist: BlacklistChecker,
    spam: SpamChecker,
    lock: LockChecker,
    welcome: WelcomeChecker,
}

impl LocalProcessor {
    /// 处理一条 Update，返回需要立即执行的动作列表
    pub async fn process(
        &self,
        update: &serde_json::Value,
        state: &AppState,
    ) -> Vec<LocalAction> {
        let mut actions = vec![];
        let config = state.config_store.get().map(|c| c.config);

        // 按优先级顺序执行检查
        if let Some(a) = self.antiflood.check(update, &config, &state.local_store) {
            actions.push(a);
        }
        if let Some(a) = self.blacklist.check(update, &config) {
            actions.push(a);
        }
        if let Some(a) = self.spam.check(update, &config, &state.local_store) {
            actions.push(a);
        }
        if let Some(a) = self.lock.check(update, &config) {
            actions.push(a);
        }
        if let Some(a) = self.welcome.check(update, &config) {
            actions.push(a);
        }

        actions
    }
}
```

---

## 八、命令注册规范

### 8.1 完整命令表

| 命令 | 权限 | 路径 | Rule | Sprint |
|---|---|---|---|---|
| `/ban` | admin | 共识 | CommandRule (现有) | ✅ |
| `/mute <seconds>` | admin | 共识 | CommandRule (现有) | ✅ |
| `/unmute` | admin | 共识 | CommandRule (现有) | ✅ |
| `/kick` | admin | 共识 | CommandRule (现有) | ✅ |
| `/pin` | admin | 共识 | CommandRule (现有) | ✅ |
| `/del` | admin | 共识 | CommandRule (现有) | ✅ |
| `/unban` | admin | 共识 | CommandRule 扩展 | S10 |
| `/warn` | admin | 共识 | WarnRule | **S-Bot1** |
| `/unwarn` | admin | 共识 | WarnRule | **S-Bot1** |
| `/warns` | any | 本地 | WarnRule | **S-Bot1** |
| `/resetwarns` | admin | 共识 | WarnRule | **S-Bot1** |
| `/setwarnlimit <N>` | admin | 共识 | → GroupConfig API | **S-Bot1** |
| `/flood <N>` | admin | 共识 | → GroupConfig API | **S-Bot1** |
| `/blacklist <word>` | admin | 共识 | → GroupConfig API | **S-Bot2** |
| `/unblacklist <word>` | admin | 共识 | → GroupConfig API | **S-Bot2** |
| `/blacklists` | any | 本地 | BlacklistRule | **S-Bot2** |
| `/lock <type>` | admin | 共识 | → GroupConfig API | **S-Bot2** |
| `/unlock <type>` | admin | 共识 | → GroupConfig API | **S-Bot2** |
| `/locks` | any | 本地 | LockRule | **S-Bot2** |
| `/welcome <text>` | admin | 共识 | → GroupConfig API | **S-Bot2** |
| `/rules` | any | 本地 | 直接回复 | **S-Bot3** |
| `/help` | any | 本地 | 直接回复 | **S-Bot3** |
| `/id` | any | 本地 | 直接回复 | **S-Bot3** |
| `/info` | any | 本地 | Query → Agent | **S-Bot3** |

---

## 九、开发 Sprint 规划

> 以下 Sprint 与现有路线图 (`NEXUS_DEVELOPMENT_ROADMAP.md`) **并行**。
> Bot 功能开发可在 Phase 2 (Sprint 8 全节点同步) 完成后立即启动。

### Sprint-Bot1: 基础功能层 (3 天)

> **前置依赖：Sprint 8 (全节点同步) 完成**

| # | 任务 | 预估 | 改动文件 |
|---|---|---|---|
| B1.1 | `LocalStore` 实现 (防刷屏计数器 + warn 计数) | 0.5d | `nexus-agent/src/local_store.rs` (新) |
| B1.2 | `AntifloodRule` 实现 | 0.5d | `nexus-node/src/rule_engine.rs` |
| B1.3 | `WarnRule` 实现 (/warn /unwarn /warns /resetwarns) | 1d | `nexus-node/src/rule_engine.rs` |
| B1.4 | `LocalProcessor` 框架 + webhook 集成 | 0.5d | `nexus-agent/src/local_processor.rs` (新), `webhook.rs` |
| B1.5 | GroupConfig 新增 antiflood + warn 字段 | 0.5d | `types.rs` (两处), `group_config.rs` |

**验收标准：**
- [ ] 消息刷屏 → 自动 mute
- [ ] /warn → 警告计数增加 → 超限自动 ban
- [ ] /warns → 显示当前警告数
- [ ] GroupConfig.antiflood_limit 和 warn_limit 可通过 API 配置
- [ ] 单元测试 ≥ 8 个

---

### Sprint-Bot2: 过滤 + 锁定 + 欢迎 (3 天)

| # | 任务 | 预估 | 改动文件 |
|---|---|---|---|
| B2.1 | `BlacklistRule` 实现 (3 种匹配模式) | 0.5d | `rule_engine.rs` |
| B2.2 | `LockRule` 实现 (14 种消息类型检测) | 0.5d | `rule_engine.rs` |
| B2.3 | `WelcomeRule` 实现 (文本 + 变量替换) | 0.5d | `rule_engine.rs` |
| B2.4 | GroupConfig 新增 blacklist + lock + welcome 扩展字段 | 0.5d | `types.rs`, `group_config.rs` |
| B2.5 | 命令解析扩展 (/blacklist /lock /unlock /welcome) | 0.5d | `rule_engine.rs` CommandRule |
| B2.6 | Executor 扩展: Kick / Promote / Demote | 0.5d | `executor.rs` |

**验收标准：**
- [ ] 黑名单词 → 消息被删除
- [ ] lock photo → 发送图片被删除
- [ ] 新成员加入 → 收到欢迎消息
- [ ] /kick → ban + 立即 unban
- [ ] 单元测试 ≥ 10 个

---

### Sprint-Bot3: 反垃圾 + 查询命令 (3 天)

| # | 任务 | 预估 | 改动文件 |
|---|---|---|---|
| B3.1 | `SpamDetectorRule` MVP (emoji + 重复 + 多语言) | 1d | `rule_engine.rs`, `local_store.rs` |
| B3.2 | 查询命令实现 (/help /rules /id /info) | 0.5d | `rule_engine.rs` |
| B3.3 | 管理员缓存 (getChatAdministrators + TTL) | 0.5d | `local_store.rs`, `local_processor.rs` |
| B3.4 | 权限前置检查 (admin 命令验证发送者权限) | 0.5d | `rule_engine.rs` |
| B3.5 | 审计日志异步提交 (本地快速路径 → Node 审计) | 0.5d | `local_processor.rs`, `webhook.rs` |

**验收标准：**
- [ ] emoji 刷屏消息 → 自动删除 + mute
- [ ] 重复消息 > 3 次 → 自动删除
- [ ] /help → 返回命令列表
- [ ] 非管理员执行 /ban → 被拒绝
- [ ] 本地快速路径执行的动作有审计日志
- [ ] 单元测试 ≥ 8 个

---

### Sprint-Bot4: 集成测试 + 优化 (2 天)

| # | 任务 | 预估 | 改动文件 |
|---|---|---|---|
| B4.1 | E2E 测试: Agent 收消息 → 本地检测 → 共识执行 | 0.5d | `tests/` |
| B4.2 | 性能基准: 100 msg/s 压力下 LocalProcessor 延迟 | 0.5d | `benches/` |
| B4.3 | LocalStore cleanup 定时任务 | 0.5d | `main.rs`, `local_store.rs` |
| B4.4 | 文档更新 + README | 0.5d | `README.md` |

---

## 十、时间线总览

```
Sprint 8 (全节点同步) ──完成──→ Bot 功能开发启动
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
   Sprint-Bot1 (3d)       Sprint 9 (并行)        Sprint 10 (并行)
   防刷屏 + 警告           节点奖励               TG API 补全
        │                       │                       │
        ▼                       │                       │
   Sprint-Bot2 (3d)             │                       │
   过滤 + 锁定 + 欢迎          │                       │
        │                       │                       │
        ▼                       ▼                       ▼
   Sprint-Bot3 (3d)       Sprint 11 (Web DApp)
   反垃圾 + 查询
        │
        ▼
   Sprint-Bot4 (2d)
   集成测试
```

**Bot 功能总工期：11 工作日**
**与主线路线图并行，不增加关键路径**

---

## 十一、文件变更清单

### 新增文件

| 文件 | 用途 |
|---|---|
| `nexus-agent/src/local_store.rs` | Agent 本地状态存储 |
| `nexus-agent/src/local_processor.rs` | 本地快速路径处理器 |

### 修改文件

| 文件 | 改动内容 |
|---|---|
| `nexus-node/src/types.rs` | GroupConfig 扩展字段 + 新枚举类型 + AdminAction 扩展 |
| `nexus-node/src/rule_engine.rs` | 新增 6 个 Rule 实现 |
| `nexus-agent/src/group_config.rs` | GroupConfig 扩展字段 + API 字段 |
| `nexus-agent/src/executor.rs` | Kick/Promote/Demote 执行逻辑 |
| `nexus-agent/src/webhook.rs` | 插入本地快速路径 |
| `nexus-agent/src/main.rs` | AppState 新增 local_store + local_processor |

### 不动文件

| 文件 | 说明 |
|---|---|
| `nexus-node/src/gossip/` | Gossip 协议不变 |
| `nexus-node/src/leader.rs` | Leader 选举不变 |
| `nexus-node/src/chain_cache.rs` | 链缓存不变 |
| `nexus-node/src/chain_client.rs` | 链客户端不变 |
| `nexus-agent/src/signer.rs` | 签名模块不变 |
| `nexus-agent/src/multicaster.rs` | 多播模块不变 |
| `pallets/nexus/` | Pallet 不变 |

---

## 十二、测试规范

### 12.1 每个 Rule 必须有的测试

```rust
#[cfg(test)]
mod tests {
    // 1. 功能开启时匹配成功 → 正确动作
    #[test] fn test_rule_triggers_when_enabled() {}

    // 2. 功能关闭时 → None (跳过)
    #[test] fn test_rule_skips_when_disabled() {}

    // 3. 管理员豁免
    #[test] fn test_rule_exempts_admin() {}

    // 4. 白名单豁免
    #[test] fn test_rule_exempts_whitelist() {}

    // 5. 边界条件 (阈值恰好等于 limit)
    #[test] fn test_rule_boundary_condition() {}
}
```

### 12.2 测试数量目标

| 模块 | 目标测试数 |
|---|---|
| AntifloodRule | 6 |
| WarnRule | 8 |
| BlacklistRule | 6 |
| LockRule | 6 |
| WelcomeRule | 4 |
| SpamDetectorRule | 6 |
| LocalStore | 10 |
| LocalProcessor | 4 |
| **合计** | **≥ 50** |

---

## 十三、关联文档

| 文档 | 关系 |
|---|---|
| `NEXUS_DEVELOPMENT_ROADMAP.md` | 主线路线图，Bot 功能 Sprint 与之并行 |
| `NEXUS_LAYERED_STORAGE_DESIGN.md` | GroupConfig 同步机制 (Sprint 8 前置依赖) |
| `NEXUS_TELEGRAM_API_EVALUATION.md` | TG API 方法评估 |
| `AGENT_THREAT_MODEL.md` | 安全威胁模型 (本地快速路径的信任边界) |

---

*文档版本: v1.0 · 2026-02-08*
*适用范围: Sprint-Bot1 — Sprint-Bot4*
*预估工期: 11 工作日，与主线并行*
*前置依赖: Sprint 8 (全节点同步) 完成*
