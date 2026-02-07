# Costik · Telegram Bot 混合代理架构设计（方案 C）

> 群主持有 Token × 轻量本地代理 × 项目方路由网关 × 多后端验证 × Cosmos 链上授权
>
> **核心原则：项目方永远不接触 Bot Token 明文**

---

## 0. 方案对比与选型

| 维度 | 方案 A: 项目方全托管 | 方案 B: 群主全自建 | **方案 C: 混合架构 ✅** |
|------|--------------------|--------------------|------------------------|
| Bot Token 谁持有 | 项目方（TEE 内） | 群主 | **群主** |
| 项目方能否接触 Token | 能（TEE 内可读） | 不能 | **不能** |
| 群主需要服务器吗 | 不需要 | 需要完整后端 | **需要最小 VPS / Docker** |
| 群主运维复杂度 | 零 | 高 | **极低（一键 Docker）** |
| 安全信任模型 | 信任项目方 | 零信任 | **零信任** |
| 扩展性 | 项目方统一扩展 | 群主各自扩展 | **路由层项目方扩展** |
| 消息完整性保证 | 项目方签名 | 群主签名 | **群主签名，项目方二次签名** |

**选择方案 C 的核心理由：**
1. **零信任安全** — Bot Token 明文永远只在群主自己的机器上，项目方代码不可能泄露
2. **群主低门槛** — 只需 `docker run` 一条命令，无需理解后端开发
3. **职责分离** — 群主负责 Token 安全 + Telegram 对接，项目方负责路由 + 链上授权 + 多后端扇出
4. **可审计** — 群主本地代理是开源的，群主可以审计代码确认 Token 不会外泄

---

## 1. 需求分析

### 1.1 角色定义

| 角色 | 说明 |
|------|------|
| **群主 (Owner)** | 拥有 Telegram 群组 + Bot Token，运行轻量本地代理 |
| **本地代理 (Local Agent)** | 群主服务器上的 Docker 容器，持有 Token，接收 Webhook，签名转发 |
| **项目方路由网关 (Router Gateway)** | 项目方运营，**不接触 Token**，负责验证签名 + 链上授权 + 多后端扇出 |
| **后端节点 (Backend Node)** | 多个独立后端，各自验证消息合法性后执行业务逻辑 |

### 1.2 核心流程

```
群主服务器                                项目方路由网关             多个后端
┌─────────────────────┐                 ┌──────────────┐
│ Local Agent (Docker) │                 │ Router GW    │
│                     │                 │              │
│ 持有 Bot Token ✅    │                 │ 无 Token ✅   │
│                     │                 │              │
│ Telegram ──Webhook──▶│                 │              │
│                     │                 │              │
│ ① 验证 Telegram 消息 │                 │              │
│ ② Ed25519 签名      │── 签名消息 ─────▶│              │
│ ③ 附加链上地址+nonce │                 │ ③ 验签        │
│                     │                 │ ④ 查链上授权   │
│                     │                 │ ⑤ 校验 IP     │         ┌──────────┐
│                     │                 │ ⑥ 扇出转发 ───────────▶│ Backend A│
│                     │                 │             ────────▶│ Backend B│
│                     │                 │             ────────▶│ Backend C│
└─────────────────────┘                 └──────────────┘        └──────────┘
```

### 1.3 安全需求

| 需求 | 说明 |
|------|------|
| **Token 零信任** | Bot Token 明文只存在群主本地代理，项目方永远无法获取 |
| **消息来源验证** | 群主本地代理用 Ed25519 私钥签名，项目方 + 后端用链上公钥验签 |
| **数据完整性** | 签名覆盖完整消息体，任何篡改导致验签失败 |
| **群主 IP 绑定** | 群主本地代理的出口 IP 哈希上链，路由网关校验来源 IP |
| **防重放攻击** | 每条消息附带单调递增 sequence + timestamp |
| **链上授权** | 群主的服务资格、公钥、IP 白名单均由 Cosmos pallet 管理 |

---

## 2. 系统架构

### 2.1 总体架构

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                  Costik · 混合代理架构（方案 C）                                 │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  群主侧 (Owner Side) — 群主自有服务器 / VPS                                    │
│  ┌────────────────────────────────────────────────────────────────────┐      │
│  │                                                                    │      │
│  │  群主 A 的 VPS              群主 B 的 VPS              群主 C ...   │      │
│  │  ┌──────────────────┐      ┌──────────────────┐                   │      │
│  │  │ Local Agent      │      │ Local Agent      │                   │      │
│  │  │ (Docker 容器)     │      │ (Docker 容器)     │                   │      │
│  │  │                  │      │                  │                   │      │
│  │  │ ┌──────────────┐ │      │ ┌──────────────┐ │                   │      │
│  │  │ │ Bot Token    │ │      │ │ Bot Token    │ │  Token 明文只     │      │
│  │  │ │ (明文, 本地)  │ │      │ │ (明文, 本地)  │ │  存在群主机器上   │      │
│  │  │ └──────────────┘ │      │ └──────────────┘ │                   │      │
│  │  │ ┌──────────────┐ │      │ ┌──────────────┐ │                   │      │
│  │  │ │ Ed25519 私钥  │ │      │ │ Ed25519 私钥  │ │  对应公钥注册    │      │
│  │  │ │ (签名消息用)   │ │      │ │ (签名消息用)   │ │  在链上         │      │
│  │  │ └──────────────┘ │      │ └──────────────┘ │                   │      │
│  │  │                  │      │                  │                   │      │
│  │  │ Telegram ──WH──▶ │      │ Telegram ──WH──▶ │                   │      │
│  │  │ 验证 → 签名 → 转发│      │ 验证 → 签名 → 转发│                   │      │
│  │  └────────┬─────────┘      └────────┬─────────┘                   │      │
│  │           │                         │                             │      │
│  └───────────┼─────────────────────────┼─────────────────────────────┘      │
│              │  HTTPS (签名消息)         │                                    │
│              └───────────┬─────────────┘                                    │
│                          ▼                                                   │
│  项目方侧 (Provider Side) — 项目方服务器集群                                    │
│  ┌────────────────────────────────────────────────────────────────────┐      │
│  │              Router Gateway (路由网关)                               │      │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌────────────┐     │      │
│  │  │ Signature │  │ Chain     │  │ Message   │  │ Rate       │     │      │
│  │  │ Verifier  │  │ Auth      │  │ Fan-out   │  │ Limiter    │     │      │
│  │  │ Ed25519   │  │ Checker   │  │ Router    │  │ 频率控制    │     │      │
│  │  │ 验签      │  │ 链上授权   │  │ 多后端扇出 │  │ 防滥用     │     │      │
│  │  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘  └─────┬──────┘     │      │
│  │        │              │              │              │             │      │
│  │  ┌─────▼──────────────▼──────────────▼──────────────▼──────┐      │      │
│  │  │              IP Verification + Nonce Dedup Engine        │      │      │
│  │  │  来源 IP 校验 + Sequence 去重 + 后端独立 HMAC 二次签名     │      │      │
│  │  └────────────────────────┬────────────────────────────────┘      │      │
│  │                           │                                       │      │
│  │  **注意: 路由网关全程不接触 Bot Token**                              │      │
│  └───────────────────────────┼───────────────────────────────────────┘      │
│                              │                                               │
│               ┌──────────────┼──────────────┐                                │
│               ▼              ▼              ▼                                 │
│  后端层 (Backend Nodes) — 各项目自有服务器                                      │
│  ┌────────────────┐ ┌────────────────┐ ┌────────────────┐                    │
│  │  Backend A     │ │  Backend B     │ │  Backend C     │                    │
│  │  ┌──────────┐  │ │  ┌──────────┐  │ │  ┌──────────┐  │                    │
│  │  │ 群主签名  │  │ │  │ 群主签名  │  │ │  │ 群主签名  │  │                    │
│  │  │ 验证     │  │ │  │ 验证     │  │ │  │ 验证     │  │                    │
│  │  │ 网关签名  │  │ │  │ 网关签名  │  │ │  │ 网关签名  │  │                    │
│  │  │ 验证     │  │ │  │ 验证     │  │ │  │ 验证     │  │                    │
│  │  │ 链上授权  │  │ │  │ 链上授权  │  │ │  │ 链上授权  │  │                    │
│  │  │ 查询     │  │ │  │ 查询     │  │ │  │ 查询     │  │                    │
│  │  └──────────┘  │ │  └──────────┘  │ │  └──────────┘  │                    │
│  │  ┌──────────┐  │ │  ┌──────────┐  │ │  ┌──────────┐  │                    │
│  │  │ 业务逻辑  │  │ │  │ 业务逻辑  │  │ │  │ 业务逻辑  │  │                    │
│  │  └──────────┘  │ │  └──────────┘  │ │  └──────────┘  │                    │
│  └────────────────┘ └────────────────┘ └────────────────┘                    │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│  链上层 (Cosmos / Substrate)                                                  │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐        │
│  │ pallet-      │ │ pallet-      │ │ pallet-      │ │ pallet-      │        │
│  │ bot-registry │ │ bot-auth     │ │ bot-billing  │ │ bot-audit    │        │
│  │ Bot注册      │ │ 授权管理     │ │ 计费结算      │ │ 审计日志     │        │
│  │ 公钥托管     │ │ IP白名单     │ │ 服务购买      │ │ 消息摘要存证  │        │
│  │ 后端绑定     │ │ 链上凭证     │ │ 余额扣减      │ │              │        │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘        │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 组件职责（三层分离）

#### 群主侧 — Local Agent

| 组件 | 职责 |
|------|------|
| **Webhook Receiver** | 接收 Telegram Webhook POST，用 Bot Token 派生的 Secret Token 验证来源 |
| **Telegram Verifier** | 确认消息确实来自 Telegram（非伪造），群主侧是唯一能做此验证的地方 |
| **Ed25519 Signer** | 用本地私钥对完整消息体签名，证明"这条消息经过群主代理认证" |
| **Forwarder** | 将签名后的消息包 HTTPS POST 到项目方路由网关 |

#### 项目方侧 — Router Gateway

| 组件 | 职责 |
|------|------|
| **Signature Verifier** | 用链上注册的群主公钥验证 Ed25519 签名 |
| **Chain Auth Checker** | 查询链上：群主授权是否有效、服务是否过期 |
| **IP Verifier** | 校验请求来源 IP 与链上注册 IP 哈希匹配 |
| **Nonce Dedup** | 检查 sequence 单调递增，拒绝重放消息 |
| **Message Fan-out** | 并发扇出到群主绑定的多个后端，为每个后端附加独立 HMAC |
| **Rate Limiter** | 按群主/Bot 维度限流，防止滥用 |

#### 链上层 — Cosmos Pallets

| Pallet | 职责 |
|--------|------|
| **pallet-bot-registry** | Bot 注册、**群主 Ed25519 公钥托管**（非 Token）、后端 URL 绑定 |
| **pallet-bot-auth** | IP 白名单（哈希）、授权凭证、服务有效期管理 |
| **pallet-bot-billing** | 服务购买、按量/按时计费、余额管理 |
| **pallet-bot-audit** | 消息摘要存证（非明文）、异常行为记录 |

---

## 3. 消息转发协议（两跳架构）

消息经过两跳传递：**Telegram → 群主 Local Agent → 项目方路由网关 → 多后端**

### 3.1 第一跳：Telegram → Local Agent

Telegram 将消息 POST 到**群主自己的服务器**：

```
POST https://{群主VPS_IP}:{port}/webhook/{secret_path}
Content-Type: application/json
X-Telegram-Bot-Api-Secret-Token: {secret_token}

{
  "update_id": 123456,
  "message": {
    "message_id": 789,
    "from": { "id": 111, "first_name": "Alice" },
    "chat": { "id": -100123456, "type": "supergroup" },
    "text": "Hello world",
    "date": 1738850400
  }
}
```

**关键区别**: Webhook 指向群主自己的 IP，而非项目方。`secret_token` 由 Bot Token 派生，**只有持有 Token 的群主能验证**。

### 3.2 Local Agent 内部处理

```
收到 Telegram Webhook
    │
    ▼
┌─────────────────────────────────────────────────┐
│ ① 验证 X-Telegram-Bot-Api-Secret-Token          │
│    (用 Bot Token 派生值比对，确认来自 Telegram)     │
└─────────────────┬───────────────────────────────┘
                  ▼
┌─────────────────────────────────────────────────┐
│ ② 构造签名消息体                                  │
│    payload = {                                   │
│      owner_address: "5GrwvaEF...",               │
│      bot_id_hash: SHA256(bot_token)[0..16],      │
│      sequence: 自增序列号,                        │
│      timestamp: 当前 UTC 秒,                     │
│      telegram_update: { 原始 Update },           │
│    }                                             │
└─────────────────┬───────────────────────────────┘
                  ▼
┌─────────────────────────────────────────────────┐
│ ③ Ed25519 签名                                   │
│    signature = Ed25519.sign(private_key, payload) │
│    (私钥只存在本地，对应公钥已注册在链上)              │
└─────────────────┬───────────────────────────────┘
                  ▼
┌─────────────────────────────────────────────────┐
│ ④ HTTPS POST 到项目方路由网关                      │
│    POST https://router.costik.network/v1/ingest  │
└─────────────────────────────────────────────────┘
```

### 3.3 第二跳：Local Agent → 路由网关

Local Agent 发送到项目方路由网关的消息格式：

```
POST https://router.costik.network/v1/ingest
Content-Type: application/json
X-Costik-Owner-Signature: {Ed25519 签名 (hex)}
X-Costik-Owner-Address: 5GrwvaEF...

{
  "version": "1.0",
  "owner_address": "5GrwvaEF...",
  "bot_id_hash": "a1b2c3d4...",
  "sequence": 42,
  "timestamp": 1738850401,
  "telegram_update": {
    "update_id": 123456,
    "message": {
      "message_id": 789,
      "from": { "id": 111, "first_name": "Alice" },
      "chat": { "id": -100123456, "type": "supergroup" },
      "text": "Hello world",
      "date": 1738850400
    }
  }
}
```

**注意**：消息体中**没有 Bot Token**，只有 `bot_id_hash`。项目方看到的是哈希，无法反推 Token。

### 3.4 第三跳：路由网关 → 多后端

路由网关验证后，为每个后端附加二次签名并扇出：

```
POST https://backend-a.example.com/costik/incoming
Content-Type: application/json
X-Costik-Owner-Signature: {群主原始 Ed25519 签名 (透传)}
X-Costik-Gateway-Signature: HMAC-SHA256({backend_a_secret}, body)
X-Costik-Gateway-Id: gw-001

{
  "version": "1.0",
  "owner_envelope": {
    "owner_address": "5GrwvaEF...",
    "bot_id_hash": "a1b2c3d4...",
    "sequence": 42,
    "timestamp": 1738850401,
    "source_ip_hash": "sha256(群主出口IP + salt)"
  },
  "telegram_update": {
    "update_id": 123456,
    "message": { ... }
  },
  "gateway_metadata": {
    "gateway_id": "gw-001",
    "relayed_at": 1738850401,
    "owner_signature_verified": true,
    "chain_auth_verified": true,
    "chain_auth_block": 12345,
    "chain_auth_expiry": 54321
  }
}
```

**双重签名设计**:
- `X-Costik-Owner-Signature` — 群主 Ed25519 签名（后端可用链上公钥独立验证）
- `X-Costik-Gateway-Signature` — 路由网关 HMAC 签名（后端确认消息经过路由网关）

### 3.5 后端验证流程（6 层验证）

每个后端收到消息后，**独立执行以下 6 层验证**：

```
收到消息
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│ 第 1 层：网关签名验证                                       │
│ ① 取 X-Costik-Gateway-Signature                          │
│ ② 用本后端与网关的 shared_secret 对 body 做 HMAC-SHA256    │
│ ③ 比对签名 → 确认消息经过可信路由网关                         │
└─────────────────┬────────────────────────────────────────┘
                  ▼
┌──────────────────────────────────────────────────────────┐
│ 第 2 层：群主签名验证 ⭐ (方案 C 新增)                       │
│ ① 取 X-Costik-Owner-Signature                            │
│ ② 从链上查询 owner_address 对应的 Ed25519 公钥              │
│ ③ 用公钥验证签名覆盖的 owner_envelope + telegram_update    │
│ → 确认消息确实由群主本地代理签发，未被任何中间方篡改            │
└─────────────────┬────────────────────────────────────────┘
                  ▼
┌──────────────────────────────────────────────────────────┐
│ 第 3 层：时效性 + 防重放                                    │
│ ① 检查 timestamp 与当前时间差 < 60 秒                      │
│ ② 检查 sequence 对该 owner 单调递增                        │
│ ③ 同一 (owner, sequence) 只接受一次                        │
└─────────────────┬────────────────────────────────────────┘
                  ▼
┌──────────────────────────────────────────────────────────┐
│ 第 4 层：来源验证                                          │
│ ① 验证 gateway_id 在已知网关列表中                          │
│ ② 验证请求来源 IP 属于路由网关 IP 池                         │
│ ③ 验证 source_ip_hash 与链上群主注册 IP 匹配               │
└─────────────────┬────────────────────────────────────────┘
                  ▼
┌──────────────────────────────────────────────────────────┐
│ 第 5 层：链上授权验证                                       │
│ ① 验证 chain_auth_expiry > 当前区块                        │
│ ② (可选) 直接 RPC 查询链上状态二次确认                       │
│ ③ 验证 owner_address 的服务套餐包含本后端                    │
└─────────────────┬────────────────────────────────────────┘
                  ▼
┌──────────────────────────────────────────────────────────┐
│ 第 6 层：数据完整性                                         │
│ ① 验证 telegram_update 结构合法 (必须有 update_id)         │
│ ② 验证 message.date 与 timestamp 差值合理 (< 10s)         │
│ ③ 验证 bot_id_hash 与后端期望的 Bot 匹配                    │
└─────────────────┬────────────────────────────────────────┘
                  ▼
           进入业务逻辑
```

**方案 C 的核心优势**: 第 2 层的群主签名验证是端到端的 — 即使路由网关被攻破或作恶，后端仍然可以独立验证消息来自群主本地代理，**不依赖对项目方的信任**。

---

## 4. Cosmos Pallet 设计

### 4.1 pallet-bot-registry — Bot 注册与公钥管理

**方案 C 核心变化**: 链上**不存储任何 Token 信息**（连加密的都不存），只存群主的 Ed25519 公钥和 Bot ID 哈希。

```rust
// pallets/costik/bot-registry/src/lib.rs

/// Bot 注册信息（方案 C：零 Token 设计）
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct BotRegistration<T: Config> {
    pub owner: T::AccountId,
    /// 群主 Local Agent 的 Ed25519 公钥（用于验签）
    /// 注意：这不是 Substrate 账户密钥，而是 Local Agent 专用的签名密钥
    pub agent_public_key: [u8; 32],
    /// Bot Token 的 SHA256 哈希（仅用于路由匹配，无法反推 Token）
    pub bot_id_hash: [u8; 16],
    /// Telegram Bot 的数字 ID（公开信息，非敏感）
    pub telegram_bot_id: u64,
    /// 绑定的后端 URL 哈希列表（URL 明文由路由网关链下管理）
    pub backend_url_hashes: BoundedVec<[u8; 32], ConstU32<10>>,
    /// 状态
    pub status: BotStatus,
    /// 消息序列号高水位（防重放，链上记录最后确认的 sequence）
    pub last_confirmed_sequence: u64,
    /// 注册区块
    pub registered_at: BlockNumberFor<T>,
    /// 服务到期区块
    pub service_expires_at: BlockNumberFor<T>,
}

/// 后端注册信息（链上只存哈希，URL 明文由路由网关链下保存）
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct BackendRegistration {
    /// 后端 URL 的 SHA256 哈希
    pub url_hash: [u8; 32],
    /// 是否启用
    pub enabled: bool,
    /// 后端与路由网关之间的 HMAC 密钥哈希（用于链上验证绑定关系）
    pub secret_binding_hash: [u8; 32],
}

#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub enum BotStatus {
    /// 活跃
    Active,
    /// 已暂停（欠费或违规）
    Suspended,
    /// 已注销
    Deregistered,
}

#[pallet::storage]
/// Bot 注册信息（bot_id_hash → BotRegistration）
pub type Bots<T> = StorageMap<_, Blake2_128Concat, [u8; 16], BotRegistration<T>>;

#[pallet::storage]
/// 群主拥有的 Bot 列表
pub type OwnerBots<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId, BoundedVec<[u8; 16], ConstU32<20>>
>;

#[pallet::storage]
/// 群主 Ed25519 公钥 → 链上账户 的反向索引（路由网关用于快速查找）
pub type PublicKeyOwner<T> = StorageMap<
    _, Blake2_128Concat, [u8; 32], T::AccountId
>;

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 群主注册 Bot
    /// - 不需要提交 Token，只需 Token 哈希 + Agent 公钥
    /// - 群主在本地生成 Ed25519 密钥对，公钥上链，私钥留本地
    #[pallet::call_index(0)]
    pub fn register_bot(
        origin: OriginFor<T>,
        telegram_bot_id: u64,
        bot_id_hash: [u8; 16],
        agent_public_key: [u8; 32],
    ) -> DispatchResult;

    /// 轮换 Agent 签名密钥（私钥泄露时使用）
    #[pallet::call_index(1)]
    pub fn rotate_agent_key(
        origin: OriginFor<T>,
        bot_id_hash: [u8; 16],
        new_public_key: [u8; 32],
    ) -> DispatchResult;

    /// 添加后端 URL 哈希
    #[pallet::call_index(2)]
    pub fn add_backend(
        origin: OriginFor<T>,
        bot_id_hash: [u8; 16],
        backend: BackendRegistration,
    ) -> DispatchResult;

    /// 移除后端
    #[pallet::call_index(3)]
    pub fn remove_backend(
        origin: OriginFor<T>,
        bot_id_hash: [u8; 16],
        url_hash: [u8; 32],
    ) -> DispatchResult;

    /// 注销 Bot
    #[pallet::call_index(4)]
    pub fn deregister_bot(
        origin: OriginFor<T>,
        bot_id_hash: [u8; 16],
    ) -> DispatchResult;
}
```

**链上存储 vs 链下存储对比**:

| 数据 | 存储位置 | 原因 |
|------|---------|------|
| Bot Token 明文 | ❌ **不存任何地方**（只在群主 Docker 环境变量中） | 零信任核心 |
| Bot Token 哈希 | 链上 (`bot_id_hash`) | 路由匹配，不可逆 |
| Ed25519 公钥 | 链上 (`agent_public_key`) | 所有参与方需要验签 |
| Ed25519 私钥 | 群主本地 (`/data/agent.key`) | 只有群主能签名 |
| 后端 URL 明文 | 路由网关链下数据库 | 避免链上暴露后端地址 |
| 后端 URL 哈希 | 链上 (`backend_url_hashes`) | 链上验证绑定关系 |
| HMAC 共享密钥 | 路由网关 + 各后端（链下） | 网关与后端之间的通道认证 |

### 4.2 pallet-bot-auth — 授权与 IP 管理

```rust
// pallets/costik/bot-auth/src/lib.rs

/// 群主授权凭证
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct OwnerAuthorization<T: Config> {
    pub owner: T::AccountId,
    /// 授权的 IP 白名单（哈希存储，保护隐私）
    pub ip_whitelist: BoundedVec<[u8; 32], ConstU32<10>>,
    /// 授权生效区块
    pub valid_from: BlockNumberFor<T>,
    /// 授权到期区块
    pub valid_until: BlockNumberFor<T>,
    /// 授权等级（决定可绑定后端数、消息频率等）
    pub tier: ServiceTier,
    /// 最近一次验证的区块
    pub last_verified: BlockNumberFor<T>,
}

#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub enum ServiceTier {
    /// 基础版：1 Bot, 3 后端, 100 msg/min
    Basic,
    /// 专业版：5 Bot, 10 后端, 1000 msg/min
    Pro,
    /// 企业版：20 Bot, 无限后端, 10000 msg/min
    Enterprise,
}

/// IP 变更记录（用于审计）
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct IpChangeRecord<T: Config> {
    pub old_ip_hash: [u8; 32],
    pub new_ip_hash: [u8; 32],
    pub changed_at: BlockNumberFor<T>,
    pub reason_hash: [u8; 32],  // 变更原因哈希
}

#[pallet::storage]
/// 群主授权信息
pub type Authorizations<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId, OwnerAuthorization<T>
>;

#[pallet::storage]
/// IP 变更历史（审计用）
pub type IpChangeHistory<T> = StorageMap<
    _, Blake2_128Concat, T::AccountId,
    BoundedVec<IpChangeRecord<T>, ConstU32<50>>
>;

#[pallet::storage]
/// 网关白名单（授权的代理网关节点）
pub type GatewayWhitelist<T> = StorageValue<
    _, BoundedVec<GatewayInfo<T>, ConstU32<100>>
>;

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 群主设置 IP 白名单
    #[pallet::call_index(0)]
    pub fn set_ip_whitelist(
        origin: OriginFor<T>,
        ip_hashes: Vec<[u8; 32]>,
    ) -> DispatchResult;

    /// 群主更新 IP（需提供旧 IP 哈希验证）
    #[pallet::call_index(1)]
    pub fn update_ip(
        origin: OriginFor<T>,
        old_ip_hash: [u8; 32],
        new_ip_hash: [u8; 32],
    ) -> DispatchResult;

    /// 验证群主授权状态（代理网关/后端调用，通过 RPC）
    /// 返回链上 Merkle 证明供后端离线验证
    #[pallet::call_index(2)]
    pub fn verify_authorization(
        origin: OriginFor<T>,
        owner: T::AccountId,
        ip_hash: [u8; 32],
    ) -> DispatchResult;

    /// 管理员注册代理网关节点
    #[pallet::call_index(10)]
    pub fn register_gateway(
        origin: OriginFor<T>,
        gateway_info: GatewayInfo<T>,
    ) -> DispatchResult;
}
```

### 4.3 pallet-bot-billing — 计费与结算

```rust
// pallets/costik/bot-billing/src/lib.rs

/// 服务套餐
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct ServicePlan<T: Config> {
    pub plan_id: u32,
    pub tier: ServiceTier,
    pub price_per_period: BalanceOf<T>,        // 每周期价格
    pub period_blocks: BlockNumberFor<T>,       // 计费周期（区块数）
    pub max_bots: u8,
    pub max_backends_per_bot: u8,
    pub max_messages_per_minute: u32,
}

/// 群主账单
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct OwnerBilling<T: Config> {
    pub owner: T::AccountId,
    pub plan_id: u32,
    pub balance: BalanceOf<T>,                  // 预充值余额
    pub total_spent: BalanceOf<T>,
    pub current_period_start: BlockNumberFor<T>,
    pub current_period_messages: u64,           // 本周期消息数
    pub auto_renew: bool,
}

#[pallet::storage]
pub type Plans<T> = StorageMap<_, Blake2_128Concat, u32, ServicePlan<T>>;

#[pallet::storage]
pub type Billings<T> = StorageMap<_, Blake2_128Concat, T::AccountId, OwnerBilling<T>>;

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 购买服务套餐
    #[pallet::call_index(0)]
    pub fn purchase_service(
        origin: OriginFor<T>,
        plan_id: u32,
        periods: u32,  // 购买周期数
    ) -> DispatchResult;

    /// 充值余额
    #[pallet::call_index(1)]
    pub fn top_up(
        origin: OriginFor<T>,
        amount: BalanceOf<T>,
    ) -> DispatchResult;

    /// 提取剩余余额（注销时）
    #[pallet::call_index(2)]
    pub fn withdraw_balance(
        origin: OriginFor<T>,
    ) -> DispatchResult;
}

#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_initialize(n: BlockNumberFor<T>) -> Weight {
        // 定期检查服务到期，自动续费或暂停
        Self::process_billing_cycle(n);
        Weight::zero()
    }
}
```

---

## 5. 安全架构（方案 C 深度分析）

### 5.1 Bot Token 保护 — 零泄露设计

```
方案 C 的 Token 生命周期：

┌─────────────────────────────────────────────────────────────────┐
│                    Token 只存在于群主控制域                        │
│                                                                 │
│  ① 群主在 @BotFather 获得 Token                                  │
│       │                                                         │
│       ▼                                                         │
│  ② 群主将 Token 写入 Local Agent 的环境变量                       │
│     docker run -e BOT_TOKEN="123456:ABC-DEF" costik-agent       │
│       │                                                         │
│       ▼                                                         │
│  ③ Local Agent 启动时：                                          │
│     - Token 加载到进程内存                                       │
│     - 计算 bot_id_hash = SHA256(Token)[0..16]                   │
│     - 调用 Telegram setWebhook API（需要 Token 明文）              │
│     - 用 Token 派生 secret_token 供 Webhook 验证                  │
│       │                                                         │
│       ▼                                                         │
│  ④ 运行期间：                                                    │
│     - Token 始终在内存中（用于验证每个 Webhook 请求）               │
│     - Token 永远不通过网络发出                                    │
│     - Token 不写入任何日志                                       │
│     - 只有 bot_id_hash 出现在转发消息中                           │
│       │                                                         │
│  ⑤ 谁能接触到 Token？                                            │
│     ✅ 群主自己                                                  │
│     ✅ 群主 VPS 上的 Docker 进程                                  │
│     ❌ 项目方路由网关                                             │
│     ❌ 后端节点                                                  │
│     ❌ 链上存储                                                  │
│     ❌ 任何第三方                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**对比方案 A（项目方全托管）**:

| 维度 | 方案 A | **方案 C** |
|------|--------|-----------|
| Token 传输 | 群主 → 项目方 (网络传输) | **永不传输** |
| Token 存储 | TEE 加密 (仍可在 TEE 内读取) | **只在群主本地内存** |
| 项目方员工能否获取 | 理论上能 (TEE 有侧信道风险) | **不可能** |
| 数据库泄露风险 | 密文可能被暴力破解 | **不存在** |
| 群主换项目方 | 需要迁移 Token | **无需迁移，Token 从未离开** |

### 5.2 Ed25519 签名链 — 端到端信任

```
信任链:

群主 Local Agent                路由网关                    后端
┌─────────────┐              ┌──────────┐              ┌──────────┐
│ Ed25519 私钥 │──── 签名 ──▶ │ 透传签名  │──── 转发 ──▶ │ 验证签名  │
│ (本地保管)   │              │ (不篡改)  │              │ (链上公钥) │
└─────────────┘              └──────────┘              └──────────┘

签名覆盖范围:
signature = Ed25519.sign(private_key, CONCAT(
    owner_address,          // 群主链上地址
    bot_id_hash,            // Bot 标识
    sequence,               // 单调递增序列号
    timestamp,              // UTC 时间戳
    SHA256(telegram_update)  // Telegram 消息体的哈希
))

验签方式:
- 路由网关: 从链上缓存 PublicKeyOwner 表查公钥，验签
- 后端: 从链上 RPC 或缓存查公钥，独立验签
- 任何人: 只要有链上公钥，都可以验证消息来源
```

**核心安全属性**:
1. **不可伪造** — 只有持有私钥的群主 Local Agent 能产生有效签名
2. **不可篡改** — 签名覆盖完整消息内容，路由网关无法修改
3. **不可否认** — 群主不能否认已签名的消息
4. **端到端** — 后端验签不依赖路由网关的诚实性

### 5.3 IP 绑定验证 — 三层校验

```
群主注册时                          消息转发时
┌──────────────────┐             ┌──────────────────────────────────┐
│ 群主 VPS          │             │                                  │
│ 出口 IP: 1.2.3.4  │             │  Local Agent (IP: 1.2.3.4)       │
│                  │             │       │                          │
│ ip_hash =        │             │       │ POST (来源 IP: 1.2.3.4)  │
│  SHA256("1.2.3.4" │             │       ▼                          │
│    + owner_salt)  │             │  路由网关                         │
│       │          │             │  ① 取请求来源 IP = 1.2.3.4       │
│       ▼          │             │  ② 计算 SHA256(IP + salt)         │
│ 链上 set_ip_      │             │  ③ 查链上 ip_whitelist            │
│ whitelist()      │             │  ④ 比对 → 匹配 ✅                │
└──────────────────┘             │  ⑤ 附加 source_ip_hash 到转发消息 │
                                 │       │                          │
                                 │       ▼                          │
                                 │  后端                            │
                                 │  ⑥ 取 source_ip_hash             │
                                 │  ⑦ 查链上 ip_whitelist 比对 ✅    │
                                 └──────────────────────────────────┘
```

**三层 IP 验证**:
1. **路由网关层**: 收到请求时，取 TCP 连接的真实来源 IP，计算哈希比对链上
2. **转发附加层**: 网关将 `source_ip_hash` 附加到消息信封中
3. **后端验证层**: 后端查链上 `ip_whitelist`，验证 `source_ip_hash` 在白名单中

**IP 变更场景**:

| 场景 | 处理方式 |
|------|---------|
| 群主 VPS 换 IP | 群主发链上交易 `update_ip(old_hash, new_hash)`，需签名确认 |
| 动态 IP | 群主购买固定 IP 的 VPS，或使用 DDNS + 定期更新链上记录 |
| 群主用多个 VPS | 白名单支持多个 IP 哈希 (`BoundedVec<[u8;32], ConstU32<10>>`) |
| 攻击者伪造 IP | TCP 层无法伪造回程 IP，HTTPS 握手会失败 |

### 5.4 防攻击矩阵（方案 C）

| 攻击类型 | 攻击描述 | 防御措施 |
|----------|---------|---------|
| **伪造 Telegram Webhook** | 攻击者向 Local Agent 发假消息 | Secret Token 校验（只有 Telegram + 持有 Token 的群主能生成） |
| **伪造 Local Agent 消息** | 攻击者冒充群主发消息到路由网关 | Ed25519 签名验证 — 无私钥无法伪造 |
| **路由网关篡改消息** | 项目方修改消息内容再扇出 | 后端独立验证群主 Ed25519 签名 — 任何篡改导致验签失败 |
| **消息重放** | 截获消息重复投递 | sequence 单调递增 + timestamp 60s 窗口 + 去重表 |
| **Bot Token 泄露** | 攻击者获取 Token | Token 只在群主本地，攻击面仅限群主 VPS 被入侵 |
| **私钥泄露** | 攻击者获取 Agent 私钥 | 链上 `rotate_agent_key()` 即时轮换，旧签名立即失效 |
| **IP 欺骗** | 攻击者伪装群主 IP | TCP 三次握手 + TLS 阻止 IP 欺骗 |
| **DDoS 路由网关** | 大量请求淹没网关 | Rate Limiter + 签名验证前置（无效签名零成本丢弃） |
| **中间人攻击** | 截获 Local Agent → 网关通信 | HTTPS/TLS 1.3 + 可选证书固定 |
| **后端枚举** | 攻击者获知所有后端地址 | 后端 URL 明文只在路由网关链下数据库，链上只存哈希 |
| **群主 VPS 被入侵** | 攻击者控制群主服务器 | 损失限于该群主的 Bot Token + Agent 私钥，不影响其他群主 |
| **路由网关被入侵** | 攻击者控制项目方网关 | 无法获取任何 Token；可篡改路由但后端会验签失败；可拒绝服务但无法伪造 |

### 5.5 故障隔离分析

```
┌──────────────────────────────────────────────────────────────────┐
│                     故障影响范围对比                                │
├──────────────────┬───────────────────┬───────────────────────────┤
│ 故障点            │ 方案 A 影响        │ 方案 C 影响               │
├──────────────────┼───────────────────┼───────────────────────────┤
│ 群主 A 的 VPS 宕机│ 无（托管在项目方）  │ 仅群主 A 的 Bot 离线      │
│ 项目方网关宕机     │ 所有群主全部离线    │ 所有消息无法扇出到后端    │
│                  │ + 所有 Token 不可用 │ 但 Token 安全、群主 Agent │
│                  │                   │ 可缓存消息等待恢复         │
│ 项目方数据库泄露   │ 加密 Token 密文泄露 │ 仅后端 URL + HMAC 密钥   │
│                  │ (有被破解风险)      │ (无 Token，零影响)        │
│ 群主 A 的 VPS 被黑│ 无直接影响         │ 群主 A 的 Token + 私钥泄露 │
│                  │ (Token 在项目方)    │ 不影响其他群主             │
│ 链上数据泄露      │ 加密 Token 密文     │ 仅公钥 + IP 哈希 + Bot 哈希│
│                  │                   │ (全部为公开或不可逆数据)    │
└──────────────────┴───────────────────┴───────────────────────────┘
```

---

## 6. 数据流详解

### 6.1 群主注册完整流程

```
群主                      前端 App              链上                  代理网关
 │                          │                   │                      │
 │  1. @BotFather 创建 Bot   │                   │                      │
 │     获得 Bot Token        │                   │                      │
 │                          │                   │                      │
 │  2. 打开 Costik 前端 ─────▶│                   │                      │
 │  3. 连接钱包              │                   │                      │
 │  4. 选择套餐 + 付款 ──────▶│                   │                      │
 │                          ├── purchase_service ▶│                      │
 │                          │                   ├── 锁定资金            │
 │                          │  ◀── 购买成功 ─────┤                      │
 │                          │                   │                      │
 │  5. 填入 Bot Token ──────▶│                   │                      │
 │  6. 填入后端 URL ─────────▶│                   │                      │
 │                          ├── 端到端加密 Token ─────────────────────────▶│
 │                          │                   │                      ├── TEE 加密
 │                          │                   │  ◀── register_bot ───┤
 │                          │                   ├── 存储加密 Token      │
 │                          │                   │                      │
 │  7. 设置 IP 白名单 ──────▶│                   │                      │
 │                          ├── set_ip_whitelist ▶│                      │
 │                          │                   ├── 存储 ip_hash       │
 │                          │                   │                      │
 │  8. 设置 Webhook ─────────────────────────────────────────────────────▶│
 │     (Telegram API)       │                   │                      ├── 生成 Webhook URL
 │                          │                   │                      ├── 调用 Telegram API
 │  ◀── 注册完成 ────────────┤                   │                      │     setWebhook
```

### 6.2 消息转发完整流程

```
时间轴 ──────────────────────────────────────────────────────────────────▶

T₀: Telegram 用户在群内发消息
    │
    ▼
T₁: Telegram 服务器 POST Webhook 到代理网关 (< 100ms)
    │
    ├── 代理网关收到 Update
    ├── 1. 根据 URL path 的 bot_id_hash 查找对应 Bot
    ├── 2. 验证 Telegram Secret Token (X-Telegram-Bot-Api-Secret-Token)
    ├── 3. 查询链上：Bot 状态 == Active? 服务未过期?
    ├── 4. 查询链上：群主 IP 在白名单中?
    ├── 5. 检查 Rate Limit
    │
    ▼
T₂: 代理网关构造签名消息 (< 10ms)
    │
    ├── 为每个后端生成独立的 HMAC 签名（使用各自的 shared_secret）
    ├── 附加 envelope（timestamp, nonce, auth_proof）
    │
    ▼
T₃: 并发扇出到 N 个后端 (< 50ms)
    │
    ├──▶ Backend A: POST /costik/incoming  →  独立 5 层验证  →  业务逻辑
    ├──▶ Backend B: POST /costik/incoming  →  独立 5 层验证  →  业务逻辑
    └──▶ Backend C: POST /costik/incoming  →  独立 5 层验证  →  业务逻辑
    │
    ▼
T₄: 代理网关收集投递结果 (< 200ms)
    │
    ├── 记录每个后端的响应状态
    ├── 失败的后端：加入重试队列（最多 3 次，指数退避）
    ├── 消息摘要上链存证（批量，每 N 条或每 M 秒）
```

---

## 7. 代理网关实现要点

### 7.1 技术选型

| 组件 | 技术 | 理由 |
|------|------|------|
| **网关主体** | Rust (Axum) | 高性能、内存安全、与 Substrate 生态一致 |
| **TEE** | Intel SGX / ARM TrustZone | Bot Token 加密、密钥管理 |
| **消息队列** | 内嵌 (tokio channels) | 低延迟，无外部依赖 |
| **重试队列** | Redis / 内存 | 失败消息的指数退避重试 |
| **链上交互** | subxt | Rust 原生 Substrate 客户端 |
| **限流** | 令牌桶 (in-memory) | 按 Bot/群主维度 |

### 7.2 核心数据结构

```rust
/// 代理网关内部的 Bot 配置缓存
struct BotConfig {
    token_hash: [u8; 32],
    owner_address: AccountId,
    backends: Vec<BackendTarget>,
    rate_limit: RateLimit,
    auth_valid_until: BlockNumber,
}

struct BackendTarget {
    url: Url,                          // 从 TEE 解密获得
    shared_secret: [u8; 32],           // 从 TEE 解密获得
    enabled: bool,
    consecutive_failures: u32,          // 连续失败计数
    circuit_breaker_until: Option<Instant>, // 熔断截止时间
}

/// 转发信封
struct ForwardEnvelope {
    version: String,
    gateway_id: String,
    bot_id_hash: String,
    owner_address: String,
    forwarded_at: u64,
    nonce: String,
    source_ip_hash: String,
    telegram_update: serde_json::Value,
    proof: AuthProof,
}

struct AuthProof {
    auth_block: u64,
    auth_expiry_block: u64,
    merkle_proof: Vec<u8>,
}
```

### 7.3 熔断与重试

```
后端响应策略：

  成功 (2xx)  ──▶  计数重置，正常继续
  失败 (5xx)  ──▶  重试队列（1s, 2s, 4s 指数退避，最多 3 次）
  超时 (>3s)  ──▶  同失败处理
  连续 5 次失败 ──▶  熔断 60 秒，期间跳过该后端
  熔断恢复    ──▶  半开状态，放 1 条试探，成功则恢复
```

---

## 8. 链上授权验证的两种模式

### 8.1 在线模式（实时 RPC 查询）

```
后端 ──── RPC query ────▶ 链节点
         is_authorized(owner, ip_hash)?
     ◀── true / false ────
```

- **优点**: 实时准确
- **缺点**: 依赖节点可用性、增加延迟

### 8.2 离线模式（Merkle 证明验证）

```
代理网关在转发时附带 Merkle 证明:

proof = {
    state_root: "0x...",       // 最近区块的状态根
    block_number: 12345,
    storage_proof: [...],       // 从 state_root 到 Authorization storage 的路径
}

后端验证流程:
1. 信任一组已知的链上 Finalized Block Hash（通过轻客户端同步）
2. 验证 state_root 属于某个 Finalized Block
3. 沿 storage_proof 验证到 Authorizations<T> 的叶子节点
4. 解码叶子节点，检查 valid_until > current_block
```

- **优点**: 不依赖节点、可完全离线验证
- **缺点**: 有区块确认延迟（~12s）

### 8.3 推荐：混合模式

```
默认用离线 Merkle 证明（低延迟）
    │
    ├── 证明年龄 < 5 分钟  →  直接通过
    │
    ├── 证明年龄 > 5 分钟  →  退化为在线 RPC 查询
    │
    └── 关键操作（首次消息/IP 变更后）→  强制在线查询
```

---

## 9. 部署拓扑

```
                         ┌──────────────────────────┐
                         │     Telegram Cloud        │
                         │     (Webhook 出站)         │
                         └────────────┬─────────────┘
                                      │
                              ┌───────▼───────┐
                              │  Load Balancer │
                              │  (TLS 终端)    │
                              └───┬───────┬───┘
                                  │       │
                    ┌─────────────▼──┐ ┌──▼─────────────┐
                    │  Gateway Node 1│ │  Gateway Node 2│     (水平扩展)
                    │  (TEE)         │ │  (TEE)         │
                    └─────┬──────────┘ └──────┬─────────┘
                          │                   │
              ┌───────────┼───────────────────┼───────────┐
              │           │    Substrate 链    │           │
              │   ┌───────▼────┐        ┌─────▼───────┐   │
              │   │ Full Node 1│        │ Full Node 2 │   │
              │   └────────────┘        └─────────────┘   │
              └───────────────────────────────────────────┘
                          │
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
    ┌───────────┐   ┌───────────┐   ┌───────────┐
    │ Backend A │   │ Backend B │   │ Backend C │    (各项目自有后端)
    │ (项目 1)  │   │ (项目 2)  │   │ (项目 3)  │
    └───────────┘   └───────────┘   └───────────┘
```

---

## 10. 与现有 Pallet 的集成

| 现有模块 | 集成方式 |
|----------|---------|
| **pallet-commission-core** | 群主推荐新群主使用服务 → 触发佣金分配 |
| **pallet-chat-core** | Bot 转发的消息可选择性存入链上聊天记录 |
| **pallet-dispute** | 服务质量争议 → 走链上仲裁流程 |
| **pallet-affiliate-referral** | 推荐计划：群主邀请群主，获得服务费分成 |

---

## 11. 总结

### 架构核心要点

| 要点 | 设计决策 |
|------|---------|
| **Bot 归属** | 群主自建 Bot，拥有完全控制权，代理网关只做转发 |
| **Token 安全** | TEE 加密存储，明文不出飞地 |
| **多后端扇出** | 一条消息并发推送到多个后端，各后端独立验证 |
| **验证分层** | 5 层验证：签名 → 时效 → 来源 → 链上授权 → 数据完整 |
| **IP 管理** | IP 哈希上链，保护隐私，变更需链上交易 |
| **链上授权** | Cosmos pallet 管理服务生命周期，Merkle 证明支持离线验证 |
| **计费模型** | 预充值 + 按周期扣费，欠费自动暂停 |
| **高可用** | 网关无状态水平扩展，后端熔断+重试 |
