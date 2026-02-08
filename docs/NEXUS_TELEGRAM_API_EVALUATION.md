# Nexus-Node × Telegram Bot API 功能评估与开发路线图

> 基于 Telegram Bot API 8.x 完整方法列表，评估 nexus-node / nexus-agent 可实现的功能范围，并给出开发优先级排序。

---

## 一、现有实现盘点

### 1.1 nexus-agent 已实现的 TG API 方法（10 个）

| TG API 方法 | ActionType | 位置 |
|---|---|---|
| `sendMessage` | SendMessage | executor.rs:69 |
| `deleteMessage` | DeleteMessage | executor.rs:78 |
| `banChatMember` | BanUser | executor.rs:87 |
| `restrictChatMember`（禁言） | MuteUser | executor.rs:100 |
| `restrictChatMember`（解禁） | UnmuteUser | executor.rs:115 |
| `pinChatMessage` | PinMessage | executor.rs:130 |
| `unpinChatMessage` | UnpinMessage | executor.rs:139 |
| `approveChatJoinRequest` | ApproveChatJoinRequest | executor.rs:148 |
| `declineChatJoinRequest` | DeclineChatJoinRequest | executor.rs:157 |
| （空操作） | NoAction | executor.rs:162 |

### 1.2 nexus-node 已实现的规则引擎（rule_engine.rs）

| 规则 | 触发条件 | 产生动作 |
|---|---|---|
| JoinRequestRule | `chat_join_request` 事件 | ApproveChatJoinRequest |
| CommandRule | `/ban /kick /mute /unmute /pin /del /delete` | 对应 ActionType |
| LinkFilterRule | 消息包含 URL 且群规启用 | DeleteMessage |
| DefaultRule | 兜底 | NoAction |

### 1.3 已有基础设施

- **共识层**: Gossip 8 种消息类型 + M/K 共识 + Leader 选举 + Failover
- **链集成**: ChainCache + ChainClient (subxt) + ChainSubmitter（3 队列批量提交）
- **签名**: Ed25519 双向签名（Leader → Agent 指令签名，Agent → 执行回执签名）
- **多平台**: PlatformAdapter trait + TelegramAdapter / DiscordAdapter / SlackAdapter
- **链上 Pallet**: bot-consensus(140) + bot-registry(141) + bot-group-mgmt(142)

---

## 二、Telegram Bot API 完整方法分类

### 2.1 Getting Updates（获取更新）

| 方法 | 说明 | 与 Nexus 的关系 |
|---|---|---|
| `getUpdates` | 长轮询获取更新 | ❌ 不需要 — 使用 Webhook 模式 |
| `setWebhook` | 设置 Webhook | ✅ 已实现（multicaster.rs: register_telegram_webhook） |
| `deleteWebhook` | 删除 Webhook | ⬜ 待实现（清理/迁移时需要） |
| `getWebhookInfo` | 查询 Webhook 状态 | ⬜ 可选（运维诊断） |

### 2.2 Available Methods — 消息发送（核心）

| 方法 | 说明 | 优先级 | 状态 |
|---|---|---|---|
| `sendMessage` | 发送文本消息 | P0 | ✅ 已实现 |
| `forwardMessage` | 转发消息 | P2 | ⬜ |
| `copyMessage` | 复制消息（无转发标记） | P2 | ⬜ |
| `sendPhoto` | 发送图片 | P1 | ⬜ |
| `sendAudio` | 发送音频 | P2 | ⬜ |
| `sendDocument` | 发送文件 | P1 | ⬜ |
| `sendVideo` | 发送视频 | P2 | ⬜ |
| `sendAnimation` | 发送 GIF | P3 | ⬜ |
| `sendVoice` | 发送语音 | P3 | ⬜ |
| `sendVideoNote` | 发送视频圆圈 | P3 | ⬜ |
| `sendMediaGroup` | 发送媒体组（相册） | P2 | ⬜ |
| `sendLocation` | 发送位置 | P3 | ⬜ |
| `sendVenue` | 发送场所 | P3 | ⬜ |
| `sendContact` | 发送联系人 | P3 | ⬜ |
| `sendPoll` | 发送投票 | P1 | ⬜ |
| `sendDice` | 发送骰子 | P3 | ⬜ |
| `sendChatAction` | 发送"正在输入"等状态 | P2 | ⬜ |
| `sendSticker` | 发送贴纸 | P3 | ⬜ |
| `sendPaidMedia` | 发送付费媒体 | P3 | ⬜ |

### 2.3 Available Methods — 群管理（核心）

| 方法 | 说明 | 优先级 | 状态 |
|---|---|---|---|
| `banChatMember` | 封禁用户 | P0 | ✅ 已实现 |
| `unbanChatMember` | 解封用户 | P0 | ⬜ **紧缺** |
| `restrictChatMember` | 限制用户权限 | P0 | ✅ 已实现（禁言/解禁） |
| `promoteChatMember` | 提升为管理员 | P1 | ⬜ |
| `setChatAdministratorCustomTitle` | 设置管理员头衔 | P2 | ⬜ |
| `banChatSenderChat` | 封禁频道身份 | P2 | ⬜ |
| `unbanChatSenderChat` | 解封频道身份 | P2 | ⬜ |
| `setChatPermissions` | 设置群默认权限 | P1 | ⬜ |
| `exportChatInviteLink` | 导出邀请链接 | P1 | ⬜ |
| `createChatInviteLink` | 创建邀请链接 | P1 | ⬜ |
| `editChatInviteLink` | 编辑邀请链接 | P2 | ⬜ |
| `revokeChatInviteLink` | 撤销邀请链接 | P2 | ⬜ |
| `approveChatJoinRequest` | 批准入群申请 | P0 | ✅ 已实现 |
| `declineChatJoinRequest` | 拒绝入群申请 | P0 | ✅ 已实现 |
| `setChatPhoto` | 设置群头像 | P3 | ⬜ |
| `deleteChatPhoto` | 删除群头像 | P3 | ⬜ |
| `setChatTitle` | 设置群名 | P2 | ⬜ |
| `setChatDescription` | 设置群简介 | P2 | ⬜ |
| `pinChatMessage` | 置顶消息 | P0 | ✅ 已实现 |
| `unpinChatMessage` | 取消置顶 | P0 | ✅ 已实现 |
| `unpinAllChatMessages` | 取消所有置顶 | P2 | ⬜ |
| `leaveChat` | Bot 退出群 | P3 | ⬜ |
| `getChat` | 获取群信息 | P1 | ⬜ |
| `getChatAdministrators` | 获取管理员列表 | P1 | ⬜ |
| `getChatMemberCount` | 获取群成员数 | P2 | ⬜ |
| `getChatMember` | 获取成员信息 | P1 | ⬜ |
| `setChatStickerSet` | 设置群贴纸包 | P3 | ⬜ |
| `deleteChatStickerSet` | 删除群贴纸包 | P3 | ⬜ |

### 2.4 Available Methods — 消息编辑

| 方法 | 说明 | 优先级 | 状态 |
|---|---|---|---|
| `editMessageText` | 编辑消息文本 | P1 | ⬜ |
| `editMessageCaption` | 编辑消息标题 | P2 | ⬜ |
| `editMessageMedia` | 编辑消息媒体 | P2 | ⬜ |
| `editMessageLiveLocation` | 编辑实时位置 | P3 | ⬜ |
| `stopMessageLiveLocation` | 停止实时位置 | P3 | ⬜ |
| `editMessageReplyMarkup` | 编辑回复键盘 | P2 | ⬜ |
| `stopPoll` | 停止投票 | P2 | ⬜ |
| `deleteMessage` | 删除消息 | P0 | ✅ 已实现 |
| `deleteMessages` | 批量删除消息 | P1 | ⬜ |

### 2.5 Forum Topics（论坛话题管理）

| 方法 | 说明 | 优先级 | 状态 |
|---|---|---|---|
| `getForumTopicIconStickers` | 获取话题图标 | P3 | ⬜ |
| `createForumTopic` | 创建话题 | P2 | ⬜ |
| `editForumTopic` | 编辑话题 | P2 | ⬜ |
| `closeForumTopic` | 关闭话题 | P2 | ⬜ |
| `reopenForumTopic` | 重开话题 | P2 | ⬜ |
| `deleteForumTopic` | 删除话题 | P2 | ⬜ |
| `unpinAllForumTopicMessages` | 取消话题所有置顶 | P3 | ⬜ |
| `editGeneralForumTopic` | 编辑通用话题 | P3 | ⬜ |
| `closeGeneralForumTopic` | 关闭通用话题 | P3 | ⬜ |
| `reopenGeneralForumTopic` | 重开通用话题 | P3 | ⬜ |
| `hideGeneralForumTopic` | 隐藏通用话题 | P3 | ⬜ |
| `unhideGeneralForumTopic` | 显示通用话题 | P3 | ⬜ |

### 2.6 Callback & Inline

| 方法 | 说明 | 优先级 | 状态 |
|---|---|---|---|
| `answerCallbackQuery` | 响应 Inline 键盘回调 | P1 | ⬜ |
| `answerInlineQuery` | 响应 Inline 搜索 | P2 | ⬜ |
| `answerWebAppQuery` | 响应 WebApp 交互 | P3 | ⬜ |

### 2.7 Bot 自身管理

| 方法 | 说明 | 优先级 | 状态 |
|---|---|---|---|
| `getMe` | 获取 Bot 信息 | P1 | ⬜ |
| `logOut` | 登出 | P3 | ⬜ |
| `close` | 关闭 | P3 | ⬜ |
| `setMyCommands` | 设置命令菜单 | P1 | ⬜ |
| `deleteMyCommands` | 删除命令菜单 | P2 | ⬜ |
| `getMyCommands` | 查询命令菜单 | P2 | ⬜ |
| `setMyName` | 设置 Bot 名称 | P3 | ⬜ |
| `getMyName` | 获取 Bot 名称 | P3 | ⬜ |
| `setMyDescription` | 设置 Bot 描述 | P3 | ⬜ |
| `getMyDescription` | 获取 Bot 描述 | P3 | ⬜ |
| `setMyShortDescription` | 设置简短描述 | P3 | ⬜ |
| `getMyShortDescription` | 获取简短描述 | P3 | ⬜ |
| `setChatMenuButton` | 设置菜单按钮 | P2 | ⬜ |
| `getChatMenuButton` | 获取菜单按钮 | P3 | ⬜ |
| `setMyDefaultAdministratorRights` | 设置默认管理权限 | P2 | ⬜ |
| `getMyDefaultAdministratorRights` | 获取默认管理权限 | P3 | ⬜ |

### 2.8 用户信息

| 方法 | 说明 | 优先级 | 状态 |
|---|---|---|---|
| `getUserProfilePhotos` | 获取用户头像 | P2 | ⬜ |
| `getFile` | 下载文件 | P2 | ⬜ |

### 2.9 Payments（支付）

| 方法 | 说明 | 优先级 | 状态 |
|---|---|---|---|
| `sendInvoice` | 发送收款单 | P2 | ⬜ |
| `createInvoiceLink` | 创建收款链接 | P2 | ⬜ |
| `answerShippingQuery` | 响应配送查询 | P3 | ⬜ |
| `answerPreCheckoutQuery` | 响应预结算查询 | P3 | ⬜ |
| `sendStarTransaction` | 发送 Telegram Stars 交易 | P3 | ⬜ |

### 2.10 Stickers（贴纸）

| 方法 | 说明 | 优先级 | 状态 |
|---|---|---|---|
| `sendSticker` | 发送贴纸 | P3 | ⬜ |
| `getStickerSet` | 获取贴纸包 | P3 | ⬜ |
| `getCustomEmojiStickers` | 获取自定义 Emoji | P3 | ⬜ |
| `uploadStickerFile` | 上传贴纸文件 | P3 | ⬜ |
| `createNewStickerSet` | 创建贴纸包 | P3 | ⬜ |
| `addStickerToSet` | 添加贴纸到包 | P3 | ⬜ |
| `setStickerPositionInSet` | 调整贴纸位置 | P3 | ⬜ |
| `deleteStickerFromSet` | 删除贴纸 | P3 | ⬜ |
| `setStickerSetTitle` | 设置贴纸包标题 | P3 | ⬜ |
| `setStickerSetThumbnail` | 设置贴纸包缩略图 | P3 | ⬜ |
| `deleteStickerSet` | 删除贴纸包 | P3 | ⬜ |

### 2.11 Games（游戏）

| 方法 | 说明 | 优先级 | 状态 |
|---|---|---|---|
| `sendGame` | 发送游戏 | P3 | ⬜ |
| `setGameScore` | 设置游戏分数 | P3 | ⬜ |
| `getGameHighScores` | 获取游戏高分 | P3 | ⬜ |

---

## 三、功能分层评估

基于 Nexus 作为**去中心化社群管理 Bot 网络**的定位，将 Telegram API 功能按业务价值分层：

### Tier 1 — 核心群管功能（已实现 + 紧急补全）

> 目标：完整的群管理闭环。这些是所有社群 Bot 的基本能力。

**已完成 (10 ActionType):**
- sendMessage, deleteMessage, banChatMember, restrictChatMember(mute/unmute)
- pinChatMessage, unpinChatMessage, approveChatJoinRequest, declineChatJoinRequest

**紧急缺失:**
- `unbanChatMember` — 没有解封就无法完成 ban→unban 闭环
- `deleteMessages` — 批量删除（清理垃圾消息场景）
- `getChatMember` — 执行前权限检查（避免 ban 管理员）
- `getChatAdministrators` — 管理员鉴权

### Tier 2 — 增强群管功能

> 目标：丰富交互方式，支持更复杂的社群管理场景。

- `answerCallbackQuery` + `editMessageText` + `editMessageReplyMarkup` — **Inline 键盘交互**（投票确认、规则配置 UI）
- `sendPhoto` / `sendDocument` — 发送欢迎图片、规则文档
- `sendPoll` — 社群投票（治理提案）
- `setChatPermissions` — 批量设置群权限（全员禁言/解禁模式）
- `promoteChatMember` — 动态管理员管理
- `createChatInviteLink` / `exportChatInviteLink` — 邀请链接管理
- `getChat` — 群信息查询（用于规则引擎上下文）
- `setMyCommands` — 注册 Bot 命令菜单（/help /rules /ban 等）
- `getMe` — Bot 自检（启动时验证 token）
- `sendChatAction` — 改善 UX（显示"正在处理"）

### Tier 3 — 论坛话题管理

> 目标：支持 Telegram 超级群的论坛模式。

- `createForumTopic` / `editForumTopic` / `closeForumTopic` / `reopenForumTopic` / `deleteForumTopic`
- `unpinAllForumTopicMessages`
- 所有 `*GeneralForumTopic` 方法
- 注意：需要 `message_thread_id` 参数贯穿所有消息发送方法

### Tier 4 — 支付与高级功能

> 目标：支持链上治理与链下支付集成。

- `sendInvoice` / `createInvoiceLink` — 与链上 Entity/Shop 系统打通
- `answerShippingQuery` / `answerPreCheckoutQuery` — 支付流程
- `answerInlineQuery` — Inline 搜索模式
- `answerWebAppQuery` — Mini App 集成

### Tier 5 — 低优先级 / 按需实现

- Stickers 全系列（除非特定社群需求）
- Games 全系列
- `sendLocation` / `sendVenue` / `sendContact`
- `sendDice`
- Bot 名称/描述管理（运维工具，非核心）

---

## 四、建议开发顺序（Sprint 规划）

### Sprint 7 — 群管闭环补全 + 鉴权前置

**目标：** 补全 ban/unban 闭环，增加执行前权限检查。

**新增 ActionType:**
```
UnbanUser          → unbanChatMember
GetChatMember      → getChatMember        (查询型)
GetAdmins          → getChatAdministrators (查询型)
DeleteMessages     → deleteMessages        (批量)
SetChatPermissions → setChatPermissions
```

**代码改动范围:**
1. `nexus-agent/src/executor.rs` — 新增 5 个 TG API 调用
2. `nexus-node/src/leader.rs` — ActionType 枚举新增 5 个变体
3. `nexus-node/src/rule_engine.rs` — CommandRule 新增 `/unban` `/perms` 命令
4. `nexus-node/src/rule_engine.rs` — TelegramAdapter 新增 API 映射
5. 新增: **权限前置检查** — Leader 执行前先 getChatMember 验证目标不是管理员

**新增命令:**
- `/unban` — 解封用户（回复目标消息或 `/unban @username`）
- `/perms` — 查看/设置群权限
- `/warn` — 警告用户（3 次警告自动 ban，需要本地计数器）

**预估工作量：** 2-3 天

---

### Sprint 8 — Inline 键盘交互 + 消息编辑

**目标：** 支持交互式确认流程（ban 确认、投票、规则配置）。

**新增 ActionType:**
```
AnswerCallbackQuery  → answerCallbackQuery
EditMessageText      → editMessageText
EditMessageMarkup    → editMessageReplyMarkup
SendPoll             → sendPoll
StopPoll             → stopPoll
```

**业务场景:**
1. **操作确认:** `/ban` → 发送确认消息+Inline 键盘 → 用户点击"确认" → `answerCallbackQuery` + `banChatMember`
2. **社群投票:** `/vote "提案内容"` → `sendPoll` → 结果自动上链
3. **规则配置:** `/settings` → 发送设置面板 → 键盘交互修改群规 → `editMessageText` 更新面板

**代码改动范围:**
1. `nexus-agent/src/executor.rs` — 新增 5 个方法
2. `nexus-node/src/leader.rs` — callback_query 事件分发逻辑
3. `nexus-node/src/rule_engine.rs` — 新增 CallbackRule（解析 callback_data）
4. 新增: **回调状态管理** — 记录 pending 确认操作（msg_id → 待执行 action）

**预估工作量：** 3-4 天

---

### Sprint 9 — Bot 初始化 + 命令菜单 + 媒体发送

**目标：** 提升 Bot 的自描述性和多媒体交互能力。

**新增 ActionType:**
```
GetMe               → getMe
SetMyCommands        → setMyCommands
SendPhoto            → sendPhoto
SendDocument         → sendDocument
SendMediaGroup       → sendMediaGroup
SendChatAction       → sendChatAction
GetChat              → getChat
```

**业务场景:**
1. **Bot 启动自检:** Agent 启动时调用 `getMe` 验证 token
2. **命令注册:** Agent 启动时调用 `setMyCommands` 注册所有可用命令
3. **欢迎消息:** 入群时发送带图片/文档的欢迎消息
4. **群规发布:** `/rules` → 发送群规文档（PDF/图片）
5. **打字状态:** 长时间处理时显示 "typing..."

**预估工作量：** 2-3 天

---

### Sprint 10 — 邀请链接管理 + 管理员操作

**目标：** 完善管理工具链。

**新增 ActionType:**
```
CreateInviteLink     → createChatInviteLink
ExportInviteLink     → exportChatInviteLink
RevokeInviteLink     → revokeChatInviteLink
PromoteMember        → promoteChatMember
SetAdminTitle        → setChatAdministratorCustomTitle
SetChatTitle         → setChatTitle
SetChatDescription   → setChatDescription
UnpinAll             → unpinAllChatMessages
```

**业务场景:**
1. **邀请链接:** `/invite` → 生成带有效期/人数限制的邀请链接，链上记录
2. **管理员管理:** `/promote @user` → 提升管理员（权限从链上群规配置读取）
3. **群信息维护:** `/title "新群名"`, `/desc "新描述"` → 管理员设置群信息

**预估工作量：** 2-3 天

---

### Sprint 11 — 论坛话题管理

**目标：** 支持 Telegram 超级群论坛模式。

**新增 ActionType:**
```
CreateForumTopic     → createForumTopic
EditForumTopic       → editForumTopic
CloseForumTopic      → closeForumTopic
ReopenForumTopic     → reopenForumTopic
DeleteForumTopic     → deleteForumTopic
```

**改动:**
- 所有消息发送 ActionType 新增 `message_thread_id` 参数
- RuleContext 新增 `forum_topic_id` 字段
- 群规新增 `forum_enabled` 标记

**预估工作量：** 2-3 天

---

### Sprint 12 — 支付集成（与链上系统打通）

**目标：** Telegram Payments + 链上 Entity/Shop 系统集成。

**新增 ActionType:**
```
SendInvoice              → sendInvoice
CreateInvoiceLink        → createInvoiceLink
AnswerShippingQuery      → answerShippingQuery
AnswerPreCheckoutQuery   → answerPreCheckoutQuery
```

**业务场景:**
1. **商品购买:** 用户通过 Bot 发起支付 → `sendInvoice` → TG 支付流程 → `answerPreCheckoutQuery` → 链上记录订单
2. **收款链接:** 商家通过 `/invoice` 创建收款链接 → 分享到群内
3. **佣金触发:** 支付完成事件 → 触发链上佣金分配（commission-core）

**预估工作量：** 4-5 天（含链上集成）

---

## 五、架构改进建议

### 5.1 ActionType 重构

当前 ActionType 是扁平枚举，随着方法增加会膨胀。建议重构为分类枚举：

```rust
pub enum ActionType {
    // 消息操作
    Message(MessageAction),
    // 群管操作
    Admin(AdminAction),
    // 交互操作
    Interactive(InteractiveAction),
    // 查询操作（只读，无需共识）
    Query(QueryAction),
    // 论坛操作
    Forum(ForumAction),
    // 支付操作
    Payment(PaymentAction),
    // 无操作
    NoAction,
}

pub enum MessageAction {
    Send, Edit, Delete, DeleteBatch, Forward, Copy,
    SendPhoto, SendDocument, SendVideo, SendPoll, SendSticker,
    // ...
}

pub enum AdminAction {
    Ban, Unban, Mute, Unmute, Promote, Demote,
    SetPermissions, CreateInviteLink, RevokeInviteLink,
    SetTitle, SetDescription, SetPhoto,
    Pin, Unpin, UnpinAll,
    // ...
}
```

### 5.2 查询型 vs 执行型分离

当前所有操作都走共识流程（gossip → M/K → Leader → Agent）。但查询型操作（getChat, getChatMember, getChatAdministrators）不修改状态，不需要共识。

建议：
- **执行型操作：** 完整共识流程（现有路径）
- **查询型操作：** 任意节点直接请求 Agent → 结果缓存到 ChainCache
- **前置检查操作：** Leader 在执行前主动查询（不走共识）

### 5.3 Webhook 事件扩展

当前 nexus-agent 只处理 `message` 和 `chat_join_request` 类型的 Update。需要扩展：

```
已支持:
  ✅ message
  ✅ chat_join_request
  ✅ callback_query (已解析但 NoAction)
  ✅ chat_member (已解析但 NoAction)

待支持:
  ⬜ edited_message         — 编辑消息事件
  ⬜ channel_post           — 频道消息
  ⬜ edited_channel_post    — 频道编辑
  ⬜ inline_query           — Inline 查询
  ⬜ chosen_inline_result   — Inline 结果选择
  ⬜ shipping_query         — 配送查询
  ⬜ pre_checkout_query     — 预结算查询
  ⬜ poll / poll_answer     — 投票事件
  ⬜ my_chat_member         — Bot 自身成员变动
  ⬜ chat_boost / removed_chat_boost — 群加速事件
```

### 5.4 规则引擎扩展点

每个新 Sprint 的规则可以作为独立 Rule 实现插入规则链：

```
Sprint 7:  PermissionCheckRule (前置权限检查)
Sprint 8:  CallbackRule (处理 Inline 键盘回调)
Sprint 9:  WelcomeRule (入群欢迎消息 + 媒体)
Sprint 10: InviteRule (邀请链接管理命令)
Sprint 11: ForumRule (论坛话题操作命令)
Sprint 12: PaymentRule (支付流程处理)
```

---

## 六、统计概览

| 指标 | 数量 |
|---|---|
| Telegram Bot API 总方法数 | ~100+ |
| 已实现 | 10 (约 10%) |
| Sprint 7-12 规划实现 | ~40 |
| 低优先级/按需实现 | ~50 |
| 规划覆盖率（排除贴纸/游戏等） | ~70% |

| Sprint | 新增方法 | 累计 | 预估天数 |
|---|---|---|---|
| Sprint 7 | 5 | 15 | 2-3 |
| Sprint 8 | 5 | 20 | 3-4 |
| Sprint 9 | 7 | 27 | 2-3 |
| Sprint 10 | 8 | 35 | 2-3 |
| Sprint 11 | 5 | 40 | 2-3 |
| Sprint 12 | 4 | 44 | 4-5 |
| **总计** | **34** | **44** | **15-21 天** |

---

## 七、关键决策点

1. **ActionType 是否重构为嵌套枚举？** → 建议 Sprint 7 开始前执行，否则后续每个 Sprint 都在扩展扁平枚举
2. **查询型操作是否走共识？** → 建议不走共识，直接 Agent 调用 + 缓存
3. **Forum 话题支持是否为必须？** → 取决于目标社群是否使用论坛模式
4. **支付集成优先级？** → 取决于与 Entity/Shop 链上系统的集成时间线
5. **Inline 模式（answerInlineQuery）是否需要？** → 如需实现群内搜索/快速操作则需要

---

*文档生成时间: 2026-02-08*
*基于: Telegram Bot API 8.x + nexus-node v0.1.0 + nexus-agent v0.1.0*
