# Nexus · 去中心化多节点验证架构（方案 D）

> 群主持有 Token × 本地代理 × 随机多播到项目节点 × 节点互通共识 × 单节点无法造假
>
> **核心原则：无中心网关、无单点信任、多节点交叉验证**

---

## 0. 方案演进

| 方案 | 架构 | 信任模型 | 弱点 |
|------|------|---------|------|
| A | 项目方全托管网关 | 信任项目方 | 项目方可读 Token |
| C | 群主本地代理 + 项目方路由网关 | Token 零信任，但路由依赖项目方 | **路由网关是中心单点** — 可拒绝服务、可选择性丢弃、可伪称消息不存在 |
| **D** | **群主本地代理 + 随机多播 + 节点互通共识** | **完全去中心化** | **群主需 VPS，节点需组网** |

### 方案 C 的残留问题

```
方案 C 中，项目方路由网关虽然不接触 Token，但仍是中心节点：

群主 Agent ──── 唯一路径 ────▶ 项目方路由网关 ────▶ 多后端

问题 1: 路由网关宕机 → 所有消息中断
问题 2: 路由网关选择性丢弃某些消息 → 群主无法发现
问题 3: 路由网关伪造 "消息已投递" 的回执 → 群主被欺骗
问题 4: 路由网关与某个后端串谋 → 只给特定后端发消息
```

### 方案 D 的解决思路

```
去掉中心路由网关，群主 Agent 直接随机发给 N 个项目节点：

群主 Agent ──┬── 随机选 ──▶ 项目节点 1 ──┐
             ├── 随机选 ──▶ 项目节点 3 ──┤── 节点间互通 gossip
             └── 随机选 ──▶ 项目节点 7 ──┘── 比对消息哈希
                                            │
                                    M-of-N 共识确认
                                            │
                                    消息被视为"已确认"
```

---

## 1. 架构总览

### 1.1 角色定义

| 角色 | 数量 | 说明 |
|------|------|------|
| **群主 (Owner)** | 多个 | 拥有 Bot Token，运行 Local Agent |
| **Local Agent** | 每群主 1 个 | Docker 容器，接收 Webhook，签名，随机多播 |
| **项目节点 (Project Node)** | N 个（如 7-21 个） | 独立运营的业务后端，组成验证网络 |
| **链上 (On-chain)** | 1 条链 | 管理注册、授权、节点列表、消息确认记录 |

### 1.2 核心架构图

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    Nexus · 方案 D 去中心化架构                            │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  群主侧                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐     │
│  │  群主 A 的 VPS           群主 B 的 VPS           群主 C ...      │     │
│  │  ┌────────────────┐    ┌────────────────┐                      │     │
│  │  │ Local Agent    │    │ Local Agent    │                      │     │
│  │  │ Bot Token ✅   │    │ Bot Token ✅   │                      │     │
│  │  │ Ed25519 私钥   │    │ Ed25519 私钥   │                      │     │
│  │  │                │    │                │                      │     │
│  │  │ TG→验证→签名   │    │ TG→验证→签名   │                      │     │
│  │  │  ↓ 随机多播     │    │  ↓ 随机多播     │                      │     │
│  │  └──┬───┬───┬─────┘    └──┬───┬───┬─────┘                      │     │
│  │     │   │   │             │   │   │                            │     │
│  └─────┼───┼───┼─────────────┼───┼───┼────────────────────────────┘     │
│        │   │   │             │   │   │                                   │
│        ▼   ▼   ▼             ▼   ▼   ▼                                   │
│  项目节点网络 (Project Node Network)                                       │
│  ┌──────────────────────────────────────────────────────────────────┐    │
│  │                                                                  │    │
│  │   Node 1      Node 2      Node 3      Node 4      ...  Node N   │    │
│  │   ┌─────┐    ┌─────┐    ┌─────┐    ┌─────┐         ┌─────┐    │    │
│  │   │ 收到 │    │ 收到 │    │     │    │ 收到 │         │     │    │    │
│  │   │ msg  │    │ msg  │    │     │    │ msg  │         │     │    │    │
│  │   └──┬──┘    └──┬──┘    └──┬──┘    └──┬──┘         └──┬──┘    │    │
│  │      │          │          │          │               │       │    │
│  │      ◀──── gossip 互通消息哈希 ────▶                          │    │
│  │      │          │          │          │               │       │    │
│  │   ┌──▼──┐    ┌──▼──┐    ┌──▼──┐    ┌──▼──┐         ┌──▼──┐    │    │
│  │   │验签  │    │验签  │    │补拉  │    │验签  │         │补拉  │    │    │
│  │   │IP校验│    │IP校验│    │消息  │    │IP校验│         │消息  │    │    │
│  │   │共识  │    │共识  │    │验签  │    │共识  │         │验签  │    │    │
│  │   │确认  │    │确认  │    │共识  │    │确认  │         │共识  │    │    │
│  │   └──┬──┘    └──┬──┘    └──┬──┘    └──┬──┘         └──┬──┘    │    │
│  │      │          │          │          │               │       │    │
│  │      ▼          ▼          ▼          ▼               ▼       │    │
│  │   业务逻辑    业务逻辑    业务逻辑    业务逻辑         业务逻辑   │    │
│  │                                                                  │    │
│  └──────────────────────────────────────────────────────────────────┘    │
│                                                                          │
├──────────────────────────────────────────────────────────────────────────┤
│  链上层 (Nexus / Substrate)                                              │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐   │
│  │ pallet-      │ │ pallet-      │ │ pallet-      │ │ pallet-      │   │
│  │ bot-registry │ │ bot-auth     │ │ bot-billing  │ │ bot-consensus│   │
│  │ Bot注册      │ │ IP白名单     │ │ 计费结算      │ │ 消息共识确认  │   │
│  │ 公钥托管     │ │ 节点注册     │ │ 服务购买      │ │ 防伪存证      │   │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘   │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 2. 核心协议：随机多播 + Gossip 共识

### 2.1 Local Agent 多播逻辑

```
Telegram Webhook 到达 Local Agent
    │
    ▼
┌────────────────────────────────────────────────────────────┐
│ ① 验证 Telegram Secret Token                               │
│    确认消息来自 Telegram                                     │
└─────────────────┬──────────────────────────────────────────┘
                  ▼
┌────────────────────────────────────────────────────────────┐
│ ② 构造签名消息                                              │
│    payload = {                                              │
│      owner_address, bot_id_hash, sequence, timestamp,       │
│      message_hash: SHA256(telegram_update),                 │
│      telegram_update: { ... }                               │
│    }                                                        │
│    signature = Ed25519.sign(private_key, payload)            │
└─────────────────┬──────────────────────────────────────────┘
                  ▼
┌────────────────────────────────────────────────────────────┐
│ ③ 从链上获取活跃项目节点列表                                  │
│    active_nodes = query_chain(ActiveProjectNodes)            │
│    N = active_nodes.len()  // 例如 15 个                     │
└─────────────────┬──────────────────────────────────────────┘
                  ▼
┌────────────────────────────────────────────────────────────┐
│ ④ 随机选择 K 个节点 (K = max(3, N/3 + 1))                   │
│    使用 VRF 或 message_hash 作为随机种子                      │
│    selected = deterministic_random_select(                   │
│      seed = SHA256(message_hash + sequence),                │
│      pool = active_nodes,                                   │
│      count = K                                              │
│    )                                                        │
│    注意: 选择算法是确定性的（所有节点可独立计算出相同结果）       │
└─────────────────┬──────────────────────────────────────────┘
                  ▼
┌────────────────────────────────────────────────────────────┐
│ ⑤ 并发发送到 K 个节点                                        │
│    for node in selected:                                    │
│      POST https://{node.endpoint}/v1/message                │
│        X-Nexus-Signature: {Ed25519 签名}                   │
│        X-Nexus-Owner: {owner_address}                      │
│        Body: {payload}                                      │
│    收集响应，记录成功/失败                                     │
└─────────────────┬──────────────────────────────────────────┘
                  ▼
┌────────────────────────────────────────────────────────────┐
│ ⑥ 确认至少 M 个节点返回 ACK (M = ceil(K * 2/3))             │
│    如果成功数 < M → 补发到额外节点直到达到 M                   │
│    如果总失败 → 缓存消息，稍后重试                             │
└────────────────────────────────────────────────────────────┘
```

**确定性随机选择的关键**: 所有节点都能用相同的 `seed = SHA256(message_hash + sequence)` 和相同的节点列表，独立算出 "这条消息应该发给哪 K 个节点"。这意味着：
- 收到消息的节点可以验证 "我确实应该收到这条消息"
- 未收到消息的节点可以主动向应该收到的节点拉取

### 2.2 项目节点收到消息后的处理

```
项目节点 i 收到消息
    │
    ▼
┌────────────────────────────────────────────────────────────┐
│ ① 签名验证                                                  │
│    从链上缓存查 owner_address → agent_public_key             │
│    验证 Ed25519 签名 → 确认消息来自群主 Local Agent           │
└─────────────────┬──────────────────────────────────────────┘
                  ▼
┌────────────────────────────────────────────────────────────┐
│ ② 链上授权验证                                               │
│    检查 owner 服务未过期 + Bot 状态 Active                    │
└─────────────────┬──────────────────────────────────────────┘
                  ▼
┌────────────────────────────────────────────────────────────┐
│ ③ 来源 IP 验证                                               │
│    计算 SHA256(source_ip + salt)                              │
│    比对链上 ip_whitelist                                      │
└─────────────────┬──────────────────────────────────────────┘
                  ▼
┌────────────────────────────────────────────────────────────┐
│ ④ 确定性节点选择验证                                          │
│    用 seed = SHA256(message_hash + sequence) 重新计算         │
│    "这条消息应该发给哪 K 个节点？"                              │
│    验证自己在目标列表中 → 否则拒绝（防止群主绕过随机选择）       │
└─────────────────┬──────────────────────────────────────────┘
                  ▼
┌────────────────────────────────────────────────────────────┐
│ ⑤ 序列号 + 防重放                                            │
│    检查 sequence > last_seen_sequence[owner]                 │
│    检查 timestamp 在合理范围内 (±60s)                         │
└─────────────────┬──────────────────────────────────────────┘
                  ▼
┌────────────────────────────────────────────────────────────┐
│ ⑥ 本地暂存 + 发起 Gossip                                     │
│    状态: PENDING (等待共识)                                   │
│    向其他 K-1 个应收节点广播:                                  │
│    "我收到了 msg_hash=xxx 来自 owner=yyy, seq=zzz"           │
└─────────────────┬──────────────────────────────────────────┘
                  ▼
           等待共识...（见 2.3）
```

### 2.3 节点间 Gossip 共识协议

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Gossip 共识流程                                     │
│                                                                      │
│  参与者: K 个应收节点（由确定性随机选择算出）                            │
│  确认门槛: M = ceil(K * 2/3)                                         │
│                                                                      │
│  时间轴 ────────────────────────────────────────────────────▶        │
│                                                                      │
│  T₀: 群主 Agent 发送消息到 K 个节点                                   │
│       │                                                              │
│       ▼                                                              │
│  T₁: 各节点独立验签、验 IP、验授权（< 50ms）                           │
│       │                                                              │
│       ▼                                                              │
│  T₂: 各节点向其他 K-1 个节点广播 MessageSeen 声明:                     │
│       ┌──────────────────────────────────────────┐                   │
│       │ MessageSeen {                            │                   │
│       │   msg_id: SHA256(owner + sequence),       │                   │
│       │   msg_hash: SHA256(telegram_update),      │                   │
│       │   owner: owner_address,                   │                   │
│       │   sequence: 42,                           │                   │
│       │   node_id: self.id,                       │                   │
│       │   node_signature: Ed25519.sign(...)       │                   │
│       │ }                                        │                   │
│       └──────────────────────────────────────────┘                   │
│       │                                                              │
│       ▼                                                              │
│  T₃: 每个节点收集 MessageSeen 声明，计数:                               │
│       seen_count[msg_id] += 1                                        │
│       │                                                              │
│       ├── seen_count >= M → 消息确认 ✅ (CONFIRMED)                   │
│       │   → 进入业务逻辑处理                                          │
│       │   → (可选) 提交确认摘要上链                                    │
│       │                                                              │
│       ├── seen_count < M 且超时 (如 5s) → 主动拉取                    │
│       │   → 向声称 seen 的节点请求完整消息                              │
│       │   → 收到后独立验签，然后广播自己的 seen                         │
│       │                                                              │
│       └── 超时 (如 30s) 仍未达到 M → 消息标记 TIMEOUT                  │
│           → 上报链上异常事件                                          │
│                                                                      │
│  T₄: 共识达成后，所有节点拥有相同的确认消息                              │
│       → 各节点独立执行业务逻辑                                         │
│       → 消息哈希 + 确认签名可上链存证                                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 2.4 消息状态机

```
                   ┌──────────┐
                   │ RECEIVED │ ← 节点收到消息，验签通过
                   └────┬─────┘
                        │ 广播 MessageSeen
                        ▼
                   ┌──────────┐
                   │ PENDING  │ ← 等待其他节点确认
                   └────┬─────┘
                        │
              ┌─────────┼──────────┐
              ▼                    ▼
        ┌───────────┐       ┌───────────┐
        │ CONFIRMED │       │  TIMEOUT  │
        │ (M/K 达标) │       │ (超时未达标)│
        └─────┬─────┘       └─────┬─────┘
              │                   │
              ▼                   ▼
        ┌───────────┐       ┌───────────┐
        │  业务处理   │       │ 上报异常   │
        │  链上存证   │       │ 链上记录   │
        └───────────┘       └───────────┘
```

---

## 3. 防单节点造假分析

### 3.1 攻击场景与防御

```
┌────────────────────────────────────────────────────────────────────┐
│                  单节点造假攻击分析                                   │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  攻击 1: 恶意节点伪造一条不存在的消息                                 │
│  ──────────────────────────────────────                            │
│  攻击者: Node X 凭空编造 "群主 A 说了 xxx"                           │
│                                                                    │
│  防御:                                                             │
│  ① Node X 无法伪造群主的 Ed25519 签名 → 其他节点验签失败              │
│  ② 即使 Node X 广播 MessageSeen，其他节点要求拉取原始消息并验签        │
│  ③ 验签失败 → 拒绝确认 → 达不到 M-of-K 门槛                        │
│  ④ 结论: ❌ 攻击失败                                               │
│                                                                    │
│  攻击 2: 恶意节点篡改消息内容                                        │
│  ──────────────────────────────────────                            │
│  攻击者: Node X 收到真消息，修改内容后广播给其他节点                    │
│                                                                    │
│  防御:                                                             │
│  ① msg_hash = SHA256(telegram_update) 包含在签名中                  │
│  ② 其他节点独立验签，发现内容与签名不匹配 → 拒绝                      │
│  ③ 诚实节点广播的 msg_hash 与 Node X 不同 → Node X 被识别为异常      │
│  ④ 结论: ❌ 攻击失败                                               │
│                                                                    │
│  攻击 3: 恶意节点选择性丢弃消息（不转发 gossip）                       │
│  ──────────────────────────────────────                            │
│  攻击者: Node X 收到消息后不广播 MessageSeen                         │
│                                                                    │
│  防御:                                                             │
│  ① 群主 Agent 发给了 K 个节点，只需 M 个确认                         │
│  ② 只要 K - M + 1 个节点诚实就够（例如 K=5, M=4, 容忍 1 个恶意）     │
│  ③ 其他诚实节点的 gossip 不受 Node X 影响                            │
│  ④ 结论: ❌ 攻击失败（除非恶意节点超过 1/3）                          │
│                                                                    │
│  攻击 4: 多个恶意节点串谋伪造消息                                     │
│  ──────────────────────────────────────                            │
│  攻击者: f 个恶意节点联合伪造                                        │
│                                                                    │
│  防御:                                                             │
│  ① 需要 M = ceil(K*2/3) 个节点确认                                 │
│  ② 即使 f 个恶意节点声称 seen，仍需伪造签名                          │
│  ③ Ed25519 签名不可伪造 → 诚实节点拉取验签会失败                      │
│  ④ 安全条件: f < K/3 (拜占庭容错经典结论)                            │
│  ⑤ 结论: ❌ 只要恶意节点 < 1/3，攻击失败                             │
│                                                                    │
│  攻击 5: 恶意节点伪装"确定性选中"                                     │
│  ──────────────────────────────────────                            │
│  攻击者: Node X 不在目标列表中，冒充收到消息                           │
│                                                                    │
│  防御:                                                             │
│  ① 确定性选择算法: 所有人用相同 seed 计算出相同目标列表                 │
│  ② Node X 不在列表中 → 其他节点拒绝接受 Node X 的 MessageSeen        │
│  ③ 结论: ❌ 攻击失败                                               │
│                                                                    │
│  攻击 6: 群主 Agent 作恶，给不同节点发不同内容                         │
│  ──────────────────────────────────────                            │
│  攻击者: 群主向 Node 1 发 "msg_A"，向 Node 2 发 "msg_B"             │
│                                                                    │
│  防御:                                                             │
│  ① 两条消息有相同 sequence 但不同 msg_hash                           │
│  ② Gossip 阶段 Node 1 和 Node 2 交换 msg_hash → 发现不一致          │
│  ③ 触发 Equivocation 检测 → 两条消息和签名作为证据上链                │
│  ④ 群主被惩罚（Slash 押金或暂停服务）                                 │
│  ⑤ 结论: ❌ 攻击被检测并惩罚                                        │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### 3.2 安全参数设计

| 参数 | 推荐值 | 说明 |
|------|--------|------|
| **N** (总节点数) | 7-21 | 项目节点总数 |
| **K** (每次发送节点数) | max(3, N/3+1) | 群主每条消息发给的节点数 |
| **M** (确认门槛) | ceil(K × 2/3) | 需要多少节点确认才视为共识达成 |
| **f** (容忍恶意数) | floor((K-1)/3) | 最多容忍的恶意节点数 |
| **gossip 超时** | 5 秒 | 超时后主动拉取 |
| **最终超时** | 30 秒 | 超时未达共识 → 标记异常 |

**典型配置示例**:

| N=7 | N=15 | N=21 |
|-----|------|------|
| K=3, M=2, f=0 | K=6, M=4, f=1 | K=8, M=6, f=2 |
| 最低安全 | 推荐 | 高安全 |

---

## 4. 消息协议

### 4.1 Local Agent → 项目节点

```
POST https://{node_endpoint}/v1/message
Content-Type: application/json
X-Nexus-Owner-Signature: {Ed25519 签名 hex}
X-Nexus-Owner-Address: 5GrwvaEF...

{
  "version": "2.0",
  "owner_address": "5GrwvaEF...",
  "bot_id_hash": "a1b2c3d4...",
  "sequence": 42,
  "timestamp": 1738850401,
  "message_hash": "SHA256(telegram_update)",
  "target_nodes": ["node1_id", "node3_id", "node7_id"],
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

### 4.2 节点间 Gossip — MessageSeen

```json
{
  "type": "MessageSeen",
  "msg_id": "SHA256(owner_address + sequence)",
  "msg_hash": "SHA256(telegram_update)",
  "owner_address": "5GrwvaEF...",
  "sequence": 42,
  "node_id": "node3",
  "seen_at": 1738850402,
  "node_signature": "Ed25519.sign(node_private_key, msg_id + msg_hash)"
}
```

### 4.3 节点间 Gossip — 拉取请求

```json
{
  "type": "MessagePull",
  "msg_id": "SHA256(owner_address + sequence)",
  "requester_node_id": "node5",
  "reason": "gossip_heard_not_received"
}
```

### 4.4 节点间 Gossip — Equivocation 告警

```json
{
  "type": "EquivocationAlert",
  "owner_address": "5GrwvaEF...",
  "sequence": 42,
  "evidence": {
    "msg_hash_a": "abc123...",
    "msg_hash_b": "def456...",
    "signature_a": "...",
    "signature_b": "...",
    "reporter_node": "node1",
    "reporter_signature": "..."
  }
}
```

---

## 5. 项目节点网络设计

### 5.1 节点通信拓扑

```
采用全连接 Gossip (节点数 < 30 时可行):

   Node 1 ◀───────▶ Node 2
     ▲  ╲              ╱  ▲
     │    ╲            ╱    │
     │      ╲        ╱      │
     │        Node 3        │
     │      ╱        ╲      │
     │    ╱            ╲    │
     ▼  ╱              ╲  ▼
   Node 4 ◀───────▶ Node 5

每个节点维护到所有其他节点的持久连接 (WebSocket / gRPC stream)
消息延迟: < 50ms（同区域），< 200ms（跨区域）
```

### 5.2 节点注册与质押

```rust
// pallets/nexus/bot-consensus/src/lib.rs

/// 项目节点信息
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct ProjectNode<T: Config> {
    pub node_id: BoundedVec<u8, ConstU32<32>>,
    pub operator: T::AccountId,
    /// 节点的 Ed25519 公钥（用于 gossip 签名验证）
    pub node_public_key: [u8; 32],
    /// 节点 API 端点哈希 (URL 明文在链下节点发现服务中)
    pub endpoint_hash: [u8; 32],
    /// 质押金额（Slash 保证金）
    pub stake: BalanceOf<T>,
    /// 节点状态
    pub status: NodeStatus,
    /// 信誉分 (0-10000)
    pub reputation: u16,
    /// 历史统计
    pub messages_confirmed: u64,
    pub messages_missed: u64,
    pub equivocations_reported: u32,
    /// 注册区块
    pub registered_at: BlockNumberFor<T>,
    pub last_active: BlockNumberFor<T>,
}

#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub enum NodeStatus {
    Active,
    Probation,     // 试用期（新节点）
    Suspended,     // 因违规暂停
    Exiting,       // 正在退出（冷却期）
}

#[pallet::storage]
pub type Nodes<T> = StorageMap<
    _, Blake2_128Concat, BoundedVec<u8, ConstU32<32>>, ProjectNode<T>
>;

#[pallet::storage]
/// 活跃节点列表（确定性选择算法用）
pub type ActiveNodeList<T> = StorageValue<
    _, BoundedVec<BoundedVec<u8, ConstU32<32>>, ConstU32<100>>
>;

#[pallet::storage]
/// 消息确认记录（批量上链）
pub type MessageConfirmations<T> = StorageMap<
    _, Blake2_128Concat, [u8; 32],  // msg_id
    MessageConfirmation<T>
>;

/// 消息确认记录
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct MessageConfirmation<T: Config> {
    pub owner: T::AccountId,
    pub sequence: u64,
    pub msg_hash: [u8; 32],
    pub confirmed_by: BoundedVec<BoundedVec<u8, ConstU32<32>>, ConstU32<20>>,
    pub confirmed_at: BlockNumberFor<T>,
}

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 注册项目节点（需质押）
    #[pallet::call_index(0)]
    pub fn register_node(
        origin: OriginFor<T>,
        node_id: BoundedVec<u8, ConstU32<32>>,
        node_public_key: [u8; 32],
        endpoint_hash: [u8; 32],
        stake: BalanceOf<T>,
    ) -> DispatchResult;

    /// 批量提交消息确认（节点定期调用）
    #[pallet::call_index(1)]
    pub fn submit_confirmations(
        origin: OriginFor<T>,
        confirmations: Vec<(
            [u8; 32],  // msg_id
            [u8; 32],  // msg_hash
            Vec<([u8; 32], [u8; 64])>,  // [(node_id, node_signature)]
        )>,
    ) -> DispatchResult;

    /// 提交 Equivocation 证据（任何节点可调用）
    #[pallet::call_index(2)]
    pub fn report_equivocation(
        origin: OriginFor<T>,
        owner: T::AccountId,
        sequence: u64,
        msg_hash_a: [u8; 32],
        signature_a: [u8; 64],
        msg_hash_b: [u8; 32],
        signature_b: [u8; 64],
    ) -> DispatchResult;

    /// 举报节点离线/不响应
    #[pallet::call_index(3)]
    pub fn report_node_offline(
        origin: OriginFor<T>,
        node_id: BoundedVec<u8, ConstU32<32>>,
        evidence_msg_ids: Vec<[u8; 32]>,
    ) -> DispatchResult;

    /// 节点退出（进入冷却期）
    #[pallet::call_index(4)]
    pub fn exit_node(
        origin: OriginFor<T>,
        node_id: BoundedVec<u8, ConstU32<32>>,
    ) -> DispatchResult;
}
```

### 5.3 节点信誉与惩罚

```
┌────────────────────────────────────────────────────────────────┐
│                    信誉系统                                      │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  初始信誉: 5000 / 10000                                        │
│                                                                │
│  加分:                                                         │
│  + 正常确认消息           +1 / 条                               │
│  + 参与共识达成           +2 / 条                               │
│  + 举报恶意行为（验证属实） +100                                 │
│  + 持续在线 24h           +10                                   │
│                                                                │
│  扣分:                                                         │
│  - 未响应 gossip (被点名)   -10 / 次                            │
│  - 超时未确认              -5 / 条                              │
│  - 广播错误 msg_hash       -500 + Slash 5%                     │
│  - Equivocation (参与伪造)  -5000 + Slash 100% + 永久封禁       │
│  - 离线 > 1 小时           -50                                  │
│                                                                │
│  信誉等级:                                                      │
│  8000+ : 可信节点 (参与确定性选择的权重更高)                       │
│  5000+ : 正常节点                                               │
│  3000+ : 观察节点 (降低被选中概率)                                │
│  < 3000: 暂停节点 (不参与消息分发)                                │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

---

## 6. 完整数据流

### 6.1 群主注册流程

```
群主                 前端 App           链上                  群主 VPS
 │                     │                │                      │
 │ 1. @BotFather ──────│────────────────│──────────────────────│
 │    获得 Token        │                │                      │
 │                     │                │                      │
 │ 2. 购买服务套餐 ────▶│                │                      │
 │                     ├── purchase ───▶│                      │
 │                     │               ├── 锁定资金            │
 │                     │  ◀── 成功 ─────┤                      │
 │                     │                │                      │
 │ 3. 启动 Docker ─────│────────────────│──────────────────────▶│
 │    docker run \     │                │                      │
 │    -e TOKEN=xxx \   │                │                      │
 │    -e CHAIN_RPC=wss://.. \           │                      │
 │    nexus-agent     │                │                      │
 │                     │                │                      ├── 生成 Ed25519 密钥对
 │                     │                │                      ├── 显示公钥给群主
 │                     │                │                      │
 │ 4. 注册 Bot 公钥 ──▶│                │                      │
 │    (复制 Agent 显    │                │                      │
 │     示的公钥)        ├── register_bot ▶│                      │
 │                     │  (bot_id_hash, │                      │
 │                     │   public_key)  │                      │
 │                     │               ├── 存储公钥            │
 │                     │               ├── 返回活跃节点列表     │
 │                     │                │                      │
 │ 5. 设置 IP ────────▶│                │                      │
 │                     ├── set_ip ─────▶│                      │
 │                     │                │                      │
 │                     │                │                      ├── 查询节点列表
 │                     │                │                      ├── 调用 TG setWebhook
 │                     │                │                      ├── 指向自己的 VPS IP
 │  ◀── 注册完成 ──────│────────────────│──────────────────────┤
 │  "Agent 已就绪"      │                │                      │
```

### 6.2 消息从 Telegram 到业务处理完整流程

```
时间轴 ──────────────────────────────────────────────────────────────────▶

T₀: Telegram 用户在群内发消息
    │
    ▼
T₁: Telegram → 群主 VPS (Webhook, < 100ms)
    │
    ├── Local Agent 收到 Update
    ├── 验证 Telegram Secret Token ✅
    ├── 构造 payload + Ed25519 签名
    ├── 计算 seed = SHA256(msg_hash + sequence)
    ├── 从链上缓存取活跃节点列表 (N=15)
    ├── 确定性随机选择 K=6 个节点
    │
    ▼
T₂: Local Agent → K 个项目节点 (并发 HTTPS, < 100ms)
    │
    ├──▶ Node 2:  收到 → 验签✅ → 验 IP✅ → 暂存 PENDING
    ├──▶ Node 5:  收到 → 验签✅ → 验 IP✅ → 暂存 PENDING
    ├──▶ Node 7:  收到 → 验签✅ → 验 IP✅ → 暂存 PENDING
    ├──▶ Node 9:  收到 → 验签✅ → 验 IP✅ → 暂存 PENDING
    ├──▶ Node 12: 超时 (网络问题)
    └──▶ Node 14: 收到 → 验签✅ → 验 IP✅ → 暂存 PENDING
    │
    ▼
T₃: 节点间 Gossip (< 200ms)
    │
    ├── Node 2 广播 MessageSeen 给 [5,7,9,12,14]
    ├── Node 5 广播 MessageSeen 给 [2,7,9,12,14]
    ├── Node 7 广播 MessageSeen 给 [2,5,9,12,14]
    ├── Node 9 广播 MessageSeen 给 [2,5,7,12,14]
    ├── Node 14 广播 MessageSeen 给 [2,5,7,9,12]
    │
    ├── Node 12 未收到原始消息，但收到了 5 个 MessageSeen
    ├── Node 12 向 Node 2 发起 MessagePull
    ├── Node 12 收到消息 → 验签✅ → 广播 MessageSeen
    │
    ▼
T₄: 共识达成 (M=4, 实际 seen=6) (< 500ms from T₀)
    │
    ├── 所有 6 个节点: 状态 PENDING → CONFIRMED ✅
    ├── 各节点独立执行业务逻辑
    ├── 消息确认摘要加入批量上链队列
    │
    ▼
T₅: 定期上链 (每 N 条或每 M 秒)
    │
    ├── 某节点调用 submit_confirmations()
    ├── 链上记录: msg_id + msg_hash + 确认节点列表
    └── 不可篡改的消息投递证明
```

---

## 7. Local Agent 设计

### 7.1 群主一键部署

```bash
# 群主只需运行一条命令
docker run -d \
  --name nexus-agent \
  --restart unless-stopped \
  -e BOT_TOKEN="123456:ABC-DEF" \
  -e CHAIN_RPC="wss://rpc.nexus.network" \
  -e OWNER_MNEMONIC="word1 word2 ... word12" \
  -p 8443:8443 \
  -v nexus-data:/data \
  ghcr.io/nexus/local-agent:latest

# 首次启动时自动:
# 1. 生成 Ed25519 密钥对 → /data/agent.key
# 2. 显示公钥 → 群主复制到前端注册
# 3. 等待链上注册完成后，自动调用 setWebhook
# 4. 开始接收和转发消息
```

### 7.2 Agent 内部结构

```
┌──────────────────────────────────────────────────────┐
│ Local Agent (Docker Container, ~20MB)                 │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │  Webhook Server (Axum, port 8443)              │  │
│  │  - 接收 Telegram POST                          │  │
│  │  - 验证 Secret Token                           │  │
│  │  - TLS 自签名证书 (Telegram 支持)               │  │
│  └───────────────┬────────────────────────────────┘  │
│                  │                                    │
│  ┌───────────────▼────────────────────────────────┐  │
│  │  Message Processor                             │  │
│  │  - 构造 payload                                │  │
│  │  - Ed25519 签名                                │  │
│  │  - 计算确定性节点选择                            │  │
│  └───────────────┬────────────────────────────────┘  │
│                  │                                    │
│  ┌───────────────▼────────────────────────────────┐  │
│  │  Multi-caster                                  │  │
│  │  - 并发 HTTPS POST 到 K 个节点                  │  │
│  │  - 收集 ACK/NACK                               │  │
│  │  - 失败补发 (备用节点)                           │  │
│  └───────────────┬────────────────────────────────┘  │
│                  │                                    │
│  ┌───────────────▼────────────────────────────────┐  │
│  │  Chain Watcher                                 │  │
│  │  - 订阅链上节点列表变化                          │  │
│  │  - 缓存活跃节点列表 (30s TTL)                    │  │
│  │  - 监听自身授权状态                              │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │  Local Storage (/data/)                        │  │
│  │  - agent.key (Ed25519 私钥)                     │  │
│  │  - sequence.dat (当前序列号)                     │  │
│  │  - retry_queue.db (失败重试队列)                 │  │
│  └────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────┘
```

### 7.3 最低 VPS 要求

| 资源 | 最低要求 | 说明 |
|------|---------|------|
| CPU | 1 vCPU | 签名 + HTTP 转发很轻量 |
| RAM | 128 MB | Agent 本身 ~20MB，余量给 OS |
| 磁盘 | 1 GB | 密钥 + 队列 + 日志 |
| 网络 | 1 Mbps | 每条消息 ~2KB × K 节点 |
| IP | 固定公网 IP | Telegram Webhook 需要 |
| 系统 | Linux + Docker | 任何便宜 VPS 即可 |
| **月成本** | **$3-5** | 最廉价 VPS 即可胜任 |

---

## 8. 对比总结

### 方案 A / C / D 全面对比

| 维度 | A: 项目方全托管 | C: 混合 (中心路由) | **D: 去中心化多节点** |
|------|----------------|-------------------|----------------------|
| Token 安全 | TEE 内 (可读) | 群主本地 | **群主本地** |
| 中心单点 | 路由网关 | 路由网关 | **无** |
| 消息路由 | 项目方控制 | 项目方控制 | **群主直发 + 节点共识** |
| 单节点造假 | 网关可造假 | 网关可选择性丢弃 | **不可能 (Ed25519 + M/K 共识)** |
| 消息审查抗性 | 低 (项目方可审查) | 低 (路由网关可审查) | **高 (K 节点中任意 M 个确认)** |
| 群主运维 | 零 | 一个 Docker | **一个 Docker** |
| 网络复杂度 | 简单 | 简单 | **中等 (节点组网)** |
| 延迟 | ~200ms | ~200ms | **~500ms (含 gossip)** |
| 拜占庭容错 | 无 | 无 | **f < K/3** |
| 可证明投递 | 项目方说了算 | 项目方说了算 | **链上多节点签名存证** |

### 方案 D 的核心创新

```
传统架构:                          方案 D:

群主 ──▶ 中心网关 ──▶ 多后端        群主 ──┬──▶ 节点 1 ─┐
                                        ├──▶ 节点 3 ─┤── gossip 互验
         单点故障 ✗                      └──▶ 节点 7 ─┘── M/K 共识
         单点信任 ✗
         单点审查 ✗                  无单点故障 ✅
                                    无单点信任 ✅
                                    抗审查 ✅
                                    可证明投递 ✅
```

---

## 9. 风险与应对

| 风险 | 影响 | 应对措施 |
|------|------|---------|
| 节点数量不足 (N<7) | 安全性降低 | 初期可用 N=3, K=3, M=2 最低配置；激励更多节点加入 |
| 节点全部在同一运营商 | 集中化风险 | 链上要求节点运营商多样性 (同一 operator 最多 N/3 个节点) |
| gossip 网络分区 | 共识无法达成 | 超时后上链记录异常；分区恢复后自动同步 |
| 群主 VPS 被 DDoS | Bot 离线 | 群主可更换 VPS IP (链上更新)；消息缓存在 Telegram 侧 |
| 节点合谋 (>1/3) | 理论上可审查消息 | 节点质押 + Slash 机制提高作恶成本；社区监督 |
| 确定性选择被预测 | 攻击者只攻击目标节点 | seed 包含 message_hash，攻击者无法预知未来消息的哈希 |
| 节点频繁上下线 | 节点列表不稳定 | 链上节点列表按 epoch 更新（如每 100 区块），epoch 内稳定 |

---

## 10. 实施路线

| 阶段 | 里程碑 | 说明 |
|------|--------|------|
| **P0** | Local Agent + 3 节点 (无 gossip) | 最小验证：群主签名 + 多节点独立验签 |
| **P1** | 节点间 gossip + 共识 | 加入 MessageSeen + M/K 确认 |
| **P2** | 链上确认存证 | submit_confirmations 批量上链 |
| **P3** | Equivocation 检测 + Slash | 防群主双发 + 节点作恶惩罚 |
| **P4** | 信誉系统 + 动态节点管理 | 自动降权/恢复，新节点试用期 |
| **P5** | 完整去中心化 | 节点开放注册，质押准入 |

---

## 11. Telegram Bot API 数据分析 — 群主发送到节点的完整数据

### 11.1 Telegram Webhook Update 对象总览

Telegram 通过 Webhook POST 到群主 Local Agent 的数据是一个 **Update** 对象。Local Agent 验证后将其签名转发给项目节点。以下是 Update 包含的所有可能类型：

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Telegram Update 对象                               │
│                                                                      │
│  {                                                                   │
│    "update_id": 123456789,          ← 全局递增 ID，保证有序           │
│                                                                      │
│    // ─── 以下字段互斥，每个 Update 只包含其中一个 ───                │
│                                                                      │
│    "message": {...},                ← 新消息（文本/图片/文件/等）      │
│    "edited_message": {...},         ← 消息被编辑                     │
│    "channel_post": {...},           ← 频道新帖                       │
│    "edited_channel_post": {...},    ← 频道帖被编辑                   │
│    "business_connection": {...},    ← 商业账户连接                   │
│    "business_message": {...},       ← 商业消息                       │
│    "edited_business_message":{...}, ← 商业消息编辑                   │
│    "deleted_business_messages":{},  ← 商业消息删除                   │
│    "message_reaction": {...},       ← 消息表情反应                   │
│    "message_reaction_count": {...}, ← 匿名表情反应计数               │
│    "inline_query": {...},           ← 内联查询                       │
│    "chosen_inline_result": {...},   ← 内联查询结果被选               │
│    "callback_query": {...},         ← 按钮回调                       │
│    "shipping_query": {...},         ← 配送查询（支付）               │
│    "pre_checkout_query": {...},     ← 预结账查询（支付）             │
│    "purchased_paid_media": {...},   ← 付费媒体购买                   │
│    "poll": {...},                   ← 投票状态更新                   │
│    "poll_answer": {...},            ← 投票回答                       │
│    "my_chat_member": {...},         ← Bot 自身成员状态变化            │
│    "chat_member": {...},            ← 其他成员状态变化               │
│    "chat_join_request": {...},      ← 入群申请                       │
│    "chat_boost": {...},             ← 频道助力                       │
│    "removed_chat_boost": {...}      ← 频道助力移除                   │
│  }                                                                   │
└──────────────────────────────────────────────────────────────────────┘
```

### 11.2 Message 对象 — 最核心、最丰富的数据

每条消息（`message` / `edited_message` / `channel_post`）包含的完整数据：

```json
{
  "message_id": 789,
  "message_thread_id": 12,           // 话题组的话题 ID
  "date": 1738850400,                // Unix 时间戳
  "edit_date": 1738850500,           // 编辑时间（仅 edited_message）

  // ─── 来源信息 ───
  "from": {                          // 发送者
    "id": 123456789,                 // Telegram 用户唯一 ID
    "is_bot": false,
    "first_name": "Alice",
    "last_name": "Wang",             // 可选
    "username": "alice_wang",        // 可选
    "language_code": "zh-hans",      // 用户语言
    "is_premium": true               // 是否 Premium 用户
  },
  "sender_chat": {...},              // 代表频道/群组发送时
  "sender_boost_count": 3,           // 发送者的助力数
  "sender_business_bot": {...},      // 商业 Bot 代发

  // ─── 聊天信息 ───
  "chat": {
    "id": -1001234567890,            // 群/频道的唯一 ID
    "type": "supergroup",            // private/group/supergroup/channel
    "title": "Nexus 开发者群",
    "username": "nexus_dev",        // 公开群的用户名
    "is_forum": true                 // 是否话题群组
  },

  // ─── 消息内容（互斥，每条消息只有一种主类型） ───
  "text": "Hello world",             // 纯文本
  "entities": [                      // 文本中的实体（@提及、URL、命令等）
    { "type": "bot_command", "offset": 0, "length": 6 },
    { "type": "mention", "offset": 7, "length": 10 },
    { "type": "url", "offset": 18, "length": 25 },
    { "type": "bold", "offset": 44, "length": 5 },
    { "type": "text_link", "offset": 50, "length": 4, "url": "https://..." }
  ],

  "photo": [                         // 图片（多个分辨率）
    { "file_id": "xxx", "file_unique_id": "yyy", "width": 90, "height": 90, "file_size": 1234 },
    { "file_id": "xxx", "file_unique_id": "yyy", "width": 320, "height": 320, "file_size": 12345 },
    { "file_id": "xxx", "file_unique_id": "yyy", "width": 800, "height": 800, "file_size": 54321 }
  ],
  "video": {                         // 视频
    "file_id": "xxx", "file_unique_id": "yyy",
    "width": 1920, "height": 1080, "duration": 30,
    "thumbnail": {...}, "file_name": "demo.mp4", "file_size": 5000000
  },
  "document": {                      // 文件/文档
    "file_id": "xxx", "file_unique_id": "yyy",
    "file_name": "report.pdf", "mime_type": "application/pdf", "file_size": 102400
  },
  "audio": {...},                    // 音频
  "voice": {...},                    // 语音消息
  "video_note": {...},               // 圆形视频消息
  "animation": {...},                // GIF 动图
  "sticker": {                       // 贴纸
    "file_id": "xxx", "type": "regular",
    "width": 512, "height": 512, "emoji": "😀",
    "set_name": "my_sticker_set"
  },
  "contact": {                       // 联系人分享
    "phone_number": "+86...", "first_name": "Bob", "user_id": 999
  },
  "location": {                      // 位置
    "latitude": 31.2304, "longitude": 121.4737
  },
  "venue": {                         // 地点（位置 + 名称）
    "location": {...}, "title": "星巴克", "address": "南京路100号"
  },
  "dice": {                          // 骰子/随机
    "emoji": "🎲", "value": 5
  },
  "poll": {                          // 投票
    "id": "xxx", "question": "最佳方案？",
    "options": [
      {"text": "方案 A", "voter_count": 3},
      {"text": "方案 D", "voter_count": 12}
    ],
    "total_voter_count": 15, "is_anonymous": true, "type": "regular"
  },

  // ─── 转发/引用信息 ───
  "forward_origin": {                // 转发来源
    "type": "user",                  // user/hidden_user/chat/channel
    "sender_user": {...},
    "date": 1738840000
  },
  "reply_to_message": {...},         // 回复的原始消息（完整 Message 对象）
  "quote": {                         // 引用的文本片段
    "text": "被引用的内容",
    "entities": [...]
  },
  "external_reply": {...},           // 来自其他聊天的回复

  // ─── 群组管理事件 ───
  "new_chat_members": [{...}],       // 新成员加入
  "left_chat_member": {...},         // 成员离开
  "new_chat_title": "新群名",        // 群名变更
  "new_chat_photo": [...],           // 群头像变更
  "delete_chat_photo": true,         // 群头像删除
  "group_chat_created": true,        // 群组创建
  "pinned_message": {...},           // 消息被置顶
  "forum_topic_created": {...},      // 话题创建
  "forum_topic_closed": {...},       // 话题关闭
  "forum_topic_reopened": {...},     // 话题重新开启
  "forum_topic_edited": {...},       // 话题编辑

  // ─── 支付 ───
  "invoice": {                       // 发票（支付请求）
    "title": "Premium 服务",
    "description": "30天 Pro 套餐",
    "currency": "USD",
    "total_amount": 999               // 最小单位（分）
  },
  "successful_payment": {            // 支付成功
    "currency": "USD",
    "total_amount": 999,
    "invoice_payload": "plan_pro_30d",
    "telegram_payment_charge_id": "xxx",
    "provider_payment_charge_id": "yyy"
  },

  // ─── 媒体组 ───
  "media_group_id": "12345",         // 多张图片/视频作为一组

  // ─── Bot 命令特殊字段 ───
  "caption": "图片说明文字",          // 媒体的文字描述
  "caption_entities": [...]           // caption 中的实体
}
```

### 11.3 其他重要 Update 类型

#### callback_query — 内联键盘按钮回调

```json
{
  "update_id": 123456790,
  "callback_query": {
    "id": "unique_callback_id",
    "from": { "id": 123456789, "first_name": "Alice", ... },
    "message": { ... },              // 包含按钮的原始消息
    "chat_instance": "xxx",
    "data": "vote_option_A"           // 按钮携带的数据（最大 64 字节）
  }
}
```

#### chat_member — 成员状态变化

```json
{
  "update_id": 123456791,
  "chat_member": {
    "chat": { "id": -1001234567890, "type": "supergroup", "title": "..." },
    "from": { "id": 999, ... },       // 谁触发的变更
    "date": 1738850400,
    "old_chat_member": {
      "user": { "id": 888, ... },
      "status": "member"               // member/administrator/creator/restricted/left/kicked
    },
    "new_chat_member": {
      "user": { "id": 888, ... },
      "status": "administrator",
      "can_manage_chat": true,
      "can_delete_messages": true,
      "can_restrict_members": true,
      "can_promote_members": false,
      "can_manage_video_chats": true,
      "can_pin_messages": true
    }
  }
}
```

#### chat_join_request — 入群申请

```json
{
  "update_id": 123456792,
  "chat_join_request": {
    "chat": { "id": -1001234567890, ... },
    "from": { "id": 777, "first_name": "NewUser", ... },
    "user_chat_id": 777,
    "date": 1738850400,
    "bio": "Nexus 开发者",
    "invite_link": {
      "invite_link": "https://t.me/+abc123",
      "creator": { "id": 111, ... },
      "creates_join_request": true,
      "name": "开发者邀请链接"
    }
  }
}
```

---

## 12. 基于 Telegram 数据可实现的功能矩阵

### 12.1 功能分类总览

```
┌──────────────────────────────────────────────────────────────────────────┐
│              Telegram 数据 → 项目节点可实现的功能                          │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ 第一层: 消息处理 (基于 message / edited_message)                  │    │
│  │                                                                 │    │
│  │  ✅ Bot 命令路由 (/start, /help, /buy, /balance, ...)           │    │
│  │  ✅ 自然语言处理 + AI 对话 (text 字段)                           │    │
│  │  ✅ 关键词自动回复 / 内容过滤                                    │    │
│  │  ✅ 多语言检测和翻译 (language_code)                             │    │
│  │  ✅ 媒体文件处理 (photo/video/document → file_id 下载)           │    │
│  │  ✅ 消息编辑追踪 (edit_date)                                    │    │
│  │  ✅ 话题管理 (message_thread_id → forum 话题路由)                │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ 第二层: 用户交互 (基于 callback_query / inline_query)             │    │
│  │                                                                 │    │
│  │  ✅ 内联键盘交互 (按钮投票/确认/选择)                             │    │
│  │  ✅ 内联查询 (在其他聊天中 @bot 搜索)                             │    │
│  │  ✅ 多步骤工作流 (callback_data 状态机)                          │    │
│  │  ✅ 游戏 / 小程序 (Web App 回调)                                │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ 第三层: 群组管理 (基于 chat_member / chat_join_request / 事件)     │    │
│  │                                                                 │    │
│  │  ✅ 自动审批/拒绝入群申请 (chat_join_request)                    │    │
│  │  ✅ 新成员欢迎消息 (new_chat_members)                           │    │
│  │  ✅ 成员离开记录 (left_chat_member)                              │    │
│  │  ✅ 权限变更审计 (chat_member 新旧状态对比)                       │    │
│  │  ✅ 话题创建/关闭/编辑监控 (forum_topic_*)                       │    │
│  │  ✅ 反垃圾/反机器人验证 (新成员验证流程)                          │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ 第四层: 支付与商业 (基于 invoice / successful_payment / business)  │    │
│  │                                                                 │    │
│  │  ✅ Telegram 原生支付 (Stars / 第三方支付网关)                    │    │
│  │  ✅ 订阅制服务 (invoice_payload 标识套餐)                        │    │
│  │  ✅ 付费内容 (purchased_paid_media)                             │    │
│  │  ✅ 商业消息代理 (business_message → CRM 集成)                   │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ 第五层: 数据分析 (基于消息元数据聚合)                               │    │
│  │                                                                 │    │
│  │  ✅ 群活跃度统计 (消息频率/用户参与度)                             │    │
│  │  ✅ 用户画像 (语言/Premium/发言频率/表情偏好)                     │    │
│  │  ✅ 热门话题追踪 (text + entities 分析)                          │    │
│  │  ✅ 表情反应分析 (message_reaction / message_reaction_count)      │    │
│  │  ✅ 投票统计 (poll / poll_answer)                               │    │
│  │  ✅ 转发溯源 (forward_origin)                                   │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### 12.2 方案 D 中项目节点的具体功能实现

结合去中心化多节点架构，项目节点收到的数据可以支持以下场景：

```
┌─────────────────────────────────────────────────────────────────────┐
│ 场景 1: 去中心化 Bot 命令处理                                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  用户发送: /swap ETH USDT 100                                       │
│                                                                     │
│  项目节点收到:                                                       │
│  {                                                                  │
│    "text": "/swap ETH USDT 100",                                    │
│    "entities": [{"type":"bot_command","offset":0,"length":5}],      │
│    "from": {"id": 123456789, "username": "alice"}                   │
│  }                                                                  │
│                                                                     │
│  多节点共识处理:                                                     │
│  ① K 个节点都解析出相同的命令: swap(ETH→USDT, 100)                   │
│  ② 节点 gossip 确认命令一致 (防止单节点篡改参数)                      │
│  ③ M/K 达成共识后，执行链上交易                                      │
│  ④ 任何一个节点的回复结果需其他节点验证后才发出                        │
│                                                                     │
│  可实现: DeFi 交易Bot / 链上治理投票Bot / NFT 购买Bot                 │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ 场景 2: 去中心化入群审核 (链上身份验证)                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  新用户申请入群:                                                     │
│  {                                                                  │
│    "chat_join_request": {                                           │
│      "from": {"id": 777, "username": "new_user"},                   │
│      "invite_link": {"name": "holder_only_link"}                    │
│    }                                                                │
│  }                                                                  │
│                                                                     │
│  多节点共识审核:                                                     │
│  ① 各节点独立查询链上: new_user 是否持有 ≥100 ATOM？                 │
│  ② 节点 gossip 比对查询结果                                         │
│  ③ M/K 节点确认 "是/否" → 发送 approve/decline                      │
│  ④ 防止单节点放行不符条件的用户                                      │
│                                                                     │
│  可实现: Token-gated 社群 / NFT 持有者专属群 / DAO 成员群             │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ 场景 3: 可信投票与治理                                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  用户点击内联按钮投票:                                                │
│  {                                                                  │
│    "callback_query": {                                              │
│      "from": {"id": 123, ...},                                      │
│      "data": "vote_proposal_42_yes"                                 │
│    }                                                                │
│  }                                                                  │
│                                                                     │
│  多节点共识记票:                                                     │
│  ① 各节点记录: user_123 对 proposal_42 投了 yes                      │
│  ② gossip 确认所有节点收到相同投票                                    │
│  ③ 投票结果上链存证 — 不可篡改                                       │
│  ④ 防止单节点篡改票数或伪造投票                                      │
│                                                                     │
│  可实现: DAO 提案投票 / 社区决策 / 链上治理                           │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ 场景 4: 支付确认与订阅管理                                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  用户完成 Telegram Stars 支付:                                       │
│  {                                                                  │
│    "successful_payment": {                                          │
│      "currency": "XTR",                                             │
│      "total_amount": 100,                                           │
│      "invoice_payload": "premium_30d_user_123",                     │
│      "telegram_payment_charge_id": "charge_xxx"                     │
│    }                                                                │
│  }                                                                  │
│                                                                     │
│  多节点共识确认:                                                     │
│  ① 各节点验证支付凭证真实性                                          │
│  ② 共识确认后，链上激活用户权益                                       │
│  ③ 防止单节点伪造 "支付成功" 白嫖服务                                 │
│                                                                     │
│  可实现: 付费订阅 / 链上会员 / NFT 铸造支付                           │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ 场景 5: 不可篡改的消息存证                                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  任何群消息:                                                         │
│  {                                                                  │
│    "message": {                                                     │
│      "from": {"id": 123, "username": "alice"},                      │
│      "chat": {"id": -100xxx, "title": "合同讨论群"},                 │
│      "text": "我同意按照方案 B 执行",                                 │
│      "date": 1738850400                                             │
│    }                                                                │
│  }                                                                  │
│                                                                     │
│  多节点存证:                                                         │
│  ① K 个节点都收到完全相同的消息                                       │
│  ② 消息哈希 + 多节点签名上链                                         │
│  ③ 日后可证明 "alice 在 xxxx 时间确实说了 yyy"                        │
│  ④ 单节点无法伪造或删除已确认的消息记录                                │
│                                                                     │
│  可实现: 合同/承诺存证 / 争议仲裁证据 / 审计日志                      │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ 场景 6: 群组管理自动化                                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  可用的管理事件数据:                                                  │
│                                                                     │
│  • new_chat_members     → 新人验证、发欢迎消息、分配角色              │
│  • left_chat_member     → 记录退出、更新成员统计                     │
│  • chat_member 状态变更  → 权限变更审计、防止越权                     │
│  • forum_topic_created  → 自动分类、设置话题权限                     │
│  • pinned_message       → 同步置顶到其他渠道                        │
│  • message.entities     → 检测垃圾链接/广告，自动删除                │
│                                                                     │
│  多节点共识优势:                                                     │
│  • 管理动作 (踢人/禁言) 需 M/K 节点同意才执行                        │
│  • 防止单节点恶意踢人或解除限制                                       │
│  • 管理日志链上可审计                                                │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 12.3 关键数据字段与用途速查表

| 字段 | 类型 | 用途 |
|------|------|------|
| `update_id` | int | 消息排序、缺失检测、去重 |
| `from.id` | int | **用户唯一标识** — 链上身份绑定、权限查询 |
| `from.username` | string | 人类可读标识 — 显示、@提及 |
| `from.language_code` | string | 多语言路由、本地化回复 |
| `from.is_premium` | bool | Premium 用户特权判断 |
| `chat.id` | int | **群组唯一标识** — 路由到正确的业务逻辑 |
| `chat.type` | string | 区分私聊/群组/超级群/频道 |
| `chat.is_forum` | bool | 是否话题群组 — 决定回复方式 |
| `message_thread_id` | int | 话题 ID — 按话题路由业务逻辑 |
| `text` | string | **核心内容** — 命令解析、NLP、关键词匹配 |
| `entities` | array | 识别命令/链接/提及/格式 — 结构化解析 |
| `photo[].file_id` | string | 图片下载标识 — 需通过 Bot API 获取实际文件 |
| `document.file_id` | string | 文件下载标识 — 文档处理/存储 |
| `callback_query.data` | string | **按钮回调数据** (max 64B) — 驱动交互流程 |
| `reply_to_message` | Message | 引用关系 — 上下文感知回复 |
| `forward_origin` | object | 转发来源 — 内容溯源 |
| `new_chat_members` | array | 新成员 — 触发验证/欢迎流程 |
| `successful_payment` | object | 支付成功 — 触发权益发放 |
| `chat_join_request` | object | 入群申请 — 触发审核流程 |
| `message_reaction` | object | 表情反应 — 情绪分析/轻量投票 |
| `poll_answer` | object | 投票结果 — 社区决策 |

### 12.4 数据流中的注意事项

```
┌──────────────────────────────────────────────────────────────────┐
│                    重要限制和注意                                   │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. 文件内容不在 Update 中                                        │
│     ─────────────────────                                        │
│     photo/video/document 只有 file_id（元数据）                   │
│     实际文件需要通过 Telegram Bot API 下载:                        │
│     GET https://api.telegram.org/bot<token>/getFile?file_id=xxx  │
│                                                                  │
│     方案 D 影响:                                                  │
│     • 只有群主 Local Agent 能下载（持有 Token）                    │
│     • 项目节点只收到 file_id，无法直接下载                         │
│     • 方案: Agent 下载后将文件内容附加到转发消息中                  │
│       或上传到 IPFS，节点收到 CID                                 │
│                                                                  │
│  2. file_id 绑定 Bot                                              │
│     ─────────────────                                             │
│     每个 Bot 获得的 file_id 不同，不能跨 Bot 使用                  │
│     file_unique_id 是通用的但不能用于下载                          │
│                                                                  │
│  3. 消息大小上限                                                   │
│     ─────────────                                                 │
│     • 文本消息: 最长 4096 UTF-8 字符                               │
│     • Caption: 最长 1024 字符                                     │
│     • callback_data: 最长 64 字节                                 │
│     • 文件下载: Bot API 限制 20MB (标准) / 50MB (Local API)       │
│                                                                  │
│  4. 隐私模式 (Privacy Mode)                                       │
│     ────────────────────                                          │
│     • 默认开启: Bot 在群组中只收到 /命令 和 @提及 的消息            │
│     • 关闭后: Bot 收到群组中的所有消息                              │
│     • 群主需在 @BotFather 设置 /setprivacy → Disable              │
│     • 方案 D 建议: 根据功能需求决定是否关闭                        │
│       - 命令型 Bot: 保持开启（更安全）                             │
│       - 全量消息分析: 需关闭                                      │
│                                                                  │
│  5. Update 不保证一定送达                                          │
│     ─────────────────                                             │
│     • Telegram 最多重试 Webhook 若干次                             │
│     • 如果群主 VPS 持续不可达，消息会丢失（24h 后过期）             │
│     • 方案 D 缓解: Agent 维护 sequence 号，节点可检测缺失          │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### 12.5 方案 D 中 Local Agent 对 Telegram 数据的预处理

```
┌──────────────────────────────────────────────────────────────────┐
│ Local Agent 在转发前对 Telegram Update 的处理                      │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  原始 Telegram Update                                            │
│        │                                                         │
│        ▼                                                         │
│  ① 验证真实性 (Secret Token)                                     │
│        │                                                         │
│        ▼                                                         │
│  ② 媒体预处理 (可选):                                             │
│     • 有 photo/video/document → 用 Token 调用 getFile 下载       │
│     • 上传到 IPFS → 获得 CID                                     │
│     • 或直接 base64 编码附加到消息中 (小文件)                      │
│     • 替换 file_id 为 IPFS CID 或 base64                         │
│        │                                                         │
│        ▼                                                         │
│  ③ 隐私过滤 (可选):                                               │
│     • 群主可配置是否转发敏感字段                                   │
│     • 例如: 去除 from.last_name / 脱敏 phone_number              │
│        │                                                         │
│        ▼                                                         │
│  ④ 构造转发 payload:                                              │
│     {                                                            │
│       owner_address, bot_id_hash, sequence, timestamp,           │
│       message_hash,                                              │
│       telegram_update: { 预处理后的 Update },                     │
│       media_attachments: [                                       │
│         { type: "photo", cid: "Qm...", size: 54321 }            │
│       ]                                                          │
│     }                                                            │
│        │                                                         │
│        ▼                                                         │
│  ⑤ Ed25519 签名 → 随机多播到 K 个节点                             │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 13. 群组管理自动化 — 深度分析

### 13.1 总体架构：去中心化群管理

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    去中心化群组管理架构                                      │
│                                                                          │
│  Telegram 群组                                                           │
│  ┌────────────────────────────────────────────────────────────────┐     │
│  │  用户行为产生事件:                                               │     │
│  │  • 发消息 → message                                             │     │
│  │  • 入群/退群 → chat_member / chat_join_request                  │     │
│  │  • 点按钮 → callback_query                                     │     │
│  │  • 发垃圾/违规 → message (需节点判定)                            │     │
│  └────────────────────┬───────────────────────────────────────────┘     │
│                       │ Webhook                                          │
│                       ▼                                                  │
│  群主 Local Agent                                                        │
│  ┌────────────────────────────────────────────────────────────────┐     │
│  │  • 验证 Telegram 来源                                           │     │
│  │  • Ed25519 签名                                                │     │
│  │  • 随机多播到 K 个项目节点                                       │     │
│  └────────────────────┬───────────────────────────────────────────┘     │
│                       │                                                  │
│           ┌───────────┼───────────┐                                     │
│           ▼           ▼           ▼                                     │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                      │
│  │  Node 1     │ │  Node 2     │ │  Node K     │                      │
│  │             │ │             │ │             │                      │
│  │ ① 验签     │ │ ① 验签     │ │ ① 验签     │                      │
│  │ ② 规则引擎  │ │ ② 规则引擎  │ │ ② 规则引擎  │                      │
│  │ ③ 判定动作  │ │ ③ 判定动作  │ │ ③ 判定动作  │                      │
│  │ ④ gossip   │ │ ④ gossip   │ │ ④ gossip   │                      │
│  │    判定结果  │ │    判定结果  │ │    判定结果  │                      │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘                      │
│         │               │               │                               │
│         └───────────────┼───────────────┘                               │
│                         │                                                │
│                  M/K 共识达成                                             │
│                         │                                                │
│                         ▼                                                │
│              ┌─────────────────────┐                                    │
│              │ 执行管理动作         │                                    │
│              │ (通过 Telegram API)  │                                    │
│              │ 由指定 Leader 节点   │                                    │
│              │ 代为执行             │                                    │
│              └──────────┬──────────┘                                    │
│                         │                                                │
│                         ▼                                                │
│              ┌─────────────────────┐                                    │
│              │ 链上记录管理日志      │                                    │
│              │ (action_hash +      │                                    │
│              │  多节点确认签名)     │                                    │
│              └─────────────────────┘                                    │
└──────────────────────────────────────────────────────────────────────────┘
```

**关键问题: 谁执行 Telegram API 调用？**

```
管理动作 (踢人/禁言/删消息) 需要调用 Telegram Bot API:
  POST https://api.telegram.org/bot<TOKEN>/banChatMember
  POST https://api.telegram.org/bot<TOKEN>/deleteMessage

只有群主 Local Agent 持有 Token → 只有 Agent 能执行 API 调用

解决方案: 节点共识后，将"管理指令"发回给群主 Local Agent 执行

  项目节点 (共识后) ───── 管理指令 (带多节点签名) ─────▶ Local Agent
                                                          │
                                                          ├── 验证 M/K 节点签名
                                                          ├── 确认指令在授权范围内
                                                          ├── 调用 Telegram Bot API
                                                          └── 返回执行结果
```

### 13.2 管理事件分类与完整处理流程

#### 13.2.1 入群管理

```
┌──────────────────────────────────────────────────────────────────────┐
│                    入群管理完整流程                                     │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  触发事件: chat_join_request (需开启入群审核)                          │
│           或 new_chat_members (直接加入)                              │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │ Telegram Update:                                            │    │
│  │ {                                                           │    │
│  │   "chat_join_request": {                                    │    │
│  │     "chat": {"id": -100xxx, "title": "DAO Members"},        │    │
│  │     "from": {                                               │    │
│  │       "id": 777,                                            │    │
│  │       "username": "new_user",                               │    │
│  │       "is_premium": false                                   │    │
│  │     },                                                      │    │
│  │     "date": 1738850400,                                     │    │
│  │     "bio": "Web3 developer",                                │    │
│  │     "invite_link": {                                        │    │
│  │       "invite_link": "https://t.me/+abc123",                │    │
│  │       "name": "holder_only",                                │    │
│  │       "creator": {"id": 111}                                │    │
│  │     }                                                       │    │
│  │   }                                                         │    │
│  │ }                                                           │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  节点处理流程:                                                        │
│                                                                      │
│  ┌──────────────────────────────────────────────────┐               │
│  │ Step 1: 规则匹配 — 该邀请链接绑定了什么规则？      │               │
│  │                                                  │               │
│  │ invite_link.name = "holder_only"                  │               │
│  │ → 查链上 GroupRules: 需持有 ≥100 ATOM             │               │
│  └───────────────────────┬──────────────────────────┘               │
│                          ▼                                           │
│  ┌──────────────────────────────────────────────────┐               │
│  │ Step 2: 链上身份解析                               │               │
│  │                                                  │               │
│  │ 方式 A: 用户提前在 Nexus DApp 绑定了             │               │
│  │         TG user_id → 链上地址                     │               │
│  │         查 UserBinding(tg_id=777) → 5Gxyz...      │               │
│  │                                                  │               │
│  │ 方式 B: 未绑定 → 发送验证链接让用户绑定            │               │
│  │         approveChatJoinRequest 延迟，等用户绑定    │               │
│  └───────────────────────┬──────────────────────────┘               │
│                          ▼                                           │
│  ┌──────────────────────────────────────────────────┐               │
│  │ Step 3: 多节点独立查询链上资产                     │               │
│  │                                                  │               │
│  │ Node 1: query balance(5Gxyz) = 150 ATOM ✅        │               │
│  │ Node 2: query balance(5Gxyz) = 150 ATOM ✅        │               │
│  │ Node K: query balance(5Gxyz) = 150 ATOM ✅        │               │
│  └───────────────────────┬──────────────────────────┘               │
│                          ▼                                           │
│  ┌──────────────────────────────────────────────────┐               │
│  │ Step 4: Gossip 判定结果                            │               │
│  │                                                  │               │
│  │ 各节点广播:                                       │               │
│  │ {                                                │               │
│  │   type: "JoinDecision",                           │               │
│  │   msg_id: "xxx",                                  │               │
│  │   tg_user_id: 777,                                │               │
│  │   decision: "APPROVE",                            │               │
│  │   reason: "balance=150 >= threshold=100",         │               │
│  │   chain_block: 12345,                             │               │
│  │   node_signature: "..."                           │               │
│  │ }                                                │               │
│  └───────────────────────┬──────────────────────────┘               │
│                          ▼                                           │
│  ┌──────────────────────────────────────────────────┐               │
│  │ Step 5: M/K 共识 → 执行                           │               │
│  │                                                  │               │
│  │ M 个节点都判定 APPROVE                             │               │
│  │ → Leader 节点向 Agent 发送管理指令:                │               │
│  │                                                  │               │
│  │ {                                                │               │
│  │   action: "approveChatJoinRequest",               │               │
│  │   params: { chat_id: -100xxx, user_id: 777 },    │               │
│  │   consensus_proof: {                              │               │
│  │     approvals: [node1_sig, node2_sig, ...],       │               │
│  │     decision: "APPROVE",                          │               │
│  │     msg_id: "xxx"                                 │               │
│  │   }                                              │               │
│  │ }                                                │               │
│  │                                                  │               │
│  │ Agent 验证 M 个签名 → 调用 Telegram API            │               │
│  └──────────────────────────────────────────────────┘               │
│                                                                      │
│  入群审核规则类型 (链上配置):                                          │
│                                                                      │
│  ┌────────────────────────────────────────────────────────┐         │
│  │ 规则类型              │ 链上验证内容                     │         │
│  ├───────────────────────┼─────────────────────────────────┤         │
│  │ Token 持仓门槛         │ balance(addr) >= threshold      │         │
│  │ NFT 持有验证          │ nft_owner(collection, addr)     │         │
│  │ DAO 成员验证          │ is_member(dao_id, addr)         │         │
│  │ 质押量验证            │ staked(addr) >= min_stake       │         │
│  │ 链上信誉分            │ reputation(addr) >= min_score   │         │
│  │ 白名单               │ is_whitelisted(addr)            │         │
│  │ 推荐人验证            │ has_referrer(addr, valid_ref)   │         │
│  │ 复合条件 (AND/OR)     │ rule_A AND rule_B               │         │
│  └───────────────────────┴─────────────────────────────────┘         │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 13.2.2 反垃圾 / 反机器人验证

```
┌──────────────────────────────────────────────────────────────────────┐
│                    反垃圾自动化                                        │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  触发事件: message (每条群消息)                                        │
│                                                                      │
│  节点收到消息后执行规则引擎:                                            │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    反垃圾规则引擎                               │   │
│  │                                                              │   │
│  │  ① 黑名单检测 (O(1) 查表)                                    │   │
│  │     from.id ∈ global_blacklist? → 立即删除 + 踢出              │   │
│  │                                                              │   │
│  │  ② 新人限制 (join_date < 24h)                                │   │
│  │     新人发链接? → 删除 + 警告                                  │   │
│  │     新人发 @提及? → 删除 + 警告                                │   │
│  │     entities 包含 url/text_link/mention? → 检查               │   │
│  │                                                              │   │
│  │  ③ 内容检测                                                  │   │
│  │     ┌──────────────────────────────────────────────────┐    │   │
│  │     │ 检测项       │ 使用字段          │ 判定方式       │    │   │
│  │     ├──────────────┼──────────────────┼───────────────┤    │   │
│  │     │ 垃圾链接     │ entities[url]     │ 域名黑名单    │    │   │
│  │     │ 广告关键词   │ text              │ 关键词表匹配  │    │   │
│  │     │ 色情/暴力    │ text              │ AI 分类模型   │    │   │
│  │     │ 钓鱼链接     │ entities[text_link]│ URL 信誉库    │    │   │
│  │     │ 刷屏         │ 频率统计           │ >5条/10s      │    │   │
│  │     │ 转发刷屏     │ forward_origin     │ 频率 + 来源   │    │   │
│  │     │ 贴纸刷屏     │ sticker 频率       │ >3个/10s      │    │   │
│  │     │ 仿冒管理员   │ from.username      │ 相似度检测    │    │   │
│  │     └──────────────┴──────────────────┴───────────────┘    │   │
│  │                                                              │   │
│  │  ④ 行为模式检测                                              │   │
│  │     加群后立刻发消息 (< 5s)? → 高概率机器人                    │   │
│  │     同内容群发到多个群? → 需跨群情报共享                        │   │
│  │     深夜集中发送? → 异常时段标记                                │   │
│  │                                                              │   │
│  │  ⑤ 判定输出                                                  │   │
│  │     SAFE     → 放行                                          │   │
│  │     WARN     → 删消息 + 发警告                                │   │
│  │     MUTE     → 禁言 N 分钟                                   │   │
│  │     BAN      → 踢出 + 封禁                                   │   │
│  │     REVIEW   → 人工审核队列                                   │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  多节点共识的关键作用:                                                 │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                                                              │   │
│  │  为什么反垃圾需要多节点共识？                                  │   │
│  │                                                              │   │
│  │  问题: 单节点可能被恶意操控                                    │   │
│  │  • 恶意节点故意将正常消息判为垃圾 → 审查特定用户               │   │
│  │  • 恶意节点故意放行垃圾消息 → 纵容广告商                       │   │
│  │  • 恶意节点篡改规则引擎参数 → 降低/提高检测阈值               │   │
│  │                                                              │   │
│  │  共识解决:                                                    │   │
│  │  • M/K 节点独立判定，多数一致才执行                            │   │
│  │  • 判定分歧时进入人工审核而非直接执行                          │   │
│  │  • 所有判定记录上链，可事后审计                                │   │
│  │                                                              │   │
│  │  分级共识 (优化延迟):                                         │   │
│  │  • BAN (高危动作)  → 需 M/K 完全共识，~500ms                  │   │
│  │  • MUTE (中危动作) → 需 M/K 完全共识，~500ms                  │   │
│  │  • WARN (低危动作) → 任意 2 节点一致即可，~200ms               │   │
│  │  • DELETE (最轻)   → 任意 2 节点一致即可，~200ms               │   │
│  │                                                              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 13.2.3 新成员欢迎与验证

```
┌──────────────────────────────────────────────────────────────────────┐
│                    新成员验证流程 (CAPTCHA + 链上)                      │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  触发: new_chat_members                                              │
│                                                                      │
│  时间轴 ──────────────────────────────────────────────────▶         │
│                                                                      │
│  T₀: 新用户加入群组                                                   │
│      │                                                               │
│      ▼                                                               │
│  T₁: 节点共识: 限制新用户权限 (restrict)                               │
│      │ Agent 执行: restrictChatMember(user, can_send_messages=false)  │
│      │                                                               │
│      ▼                                                               │
│  T₂: Agent 发送欢迎消息 + 验证按钮                                     │
│      │                                                               │
│      │ "欢迎 @new_user! 请在 120 秒内完成验证:"                       │
│      │                                                               │
│      │ ┌─────────────────────────────────────────────────┐          │
│      │ │ [🧮 数学验证: 3+7=?]  [🔗 绑定钱包]  [🎯 点击验证] │          │
│      │ └─────────────────────────────────────────────────┘          │
│      │                                                               │
│      ▼                                                               │
│  T₃: 用户点击按钮 → callback_query                                    │
│      │                                                               │
│      │ 情况 A: 数学验证                                               │
│      │ callback_data = "captcha_answer_10"                            │
│      │ → 节点比对: 3+7=10 ✅                                         │
│      │                                                               │
│      │ 情况 B: 绑定钱包                                               │
│      │ callback_data = "bind_wallet"                                  │
│      │ → Agent 发送绑定链接 → 用户签名证明拥有地址                     │
│      │ → 节点查链上: 地址满足入群条件?                                 │
│      │                                                               │
│      │ 情况 C: 简单点击验证 (最低安全级)                               │
│      │ callback_data = "verify_human_xxxxx"                           │
│      │ → 验证通过                                                    │
│      │                                                               │
│      ▼                                                               │
│  T₄: 多节点共识确认验证结果                                            │
│      │                                                               │
│      ├── 验证通过: restrictChatMember(user, 恢复权限)                  │
│      │             + 发送 "验证成功" 消息                              │
│      │                                                               │
│      └── 超时未验证 (120s):                                           │
│           → 节点共识: banChatMember(user) 踢出                        │
│           → 可配置: 踢出 vs 保留但持续限制                             │
│                                                                      │
│  验证难度分级 (群主链上配置):                                           │
│  ┌─────────────────────────────────────────────────────────┐        │
│  │ Level 1: 点击按钮 (防最低级 Bot)                         │        │
│  │ Level 2: 数学题 / 图片选择 (防中级 Bot)                   │        │
│  │ Level 3: 绑定钱包签名 (Web3 身份验证)                     │        │
│  │ Level 4: Token 持仓验证 (Token-gated)                    │        │
│  │ Level 5: NFT 持有 + 链上行为分析 (高级 DAO)               │        │
│  │ Level 6: 人工审核 (邀请制群组)                            │        │
│  └─────────────────────────────────────────────────────────┘        │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 13.2.4 权限管理与角色系统

```
┌──────────────────────────────────────────────────────────────────────┐
│                    链上驱动的动态权限管理                                │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  传统群管理: 群主手动设管理员，权限固定                                  │
│  方案 D:    链上条件驱动，权限动态变化，多节点共识执行                    │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    动态角色系统                                │   │
│  │                                                              │   │
│  │  角色定义 (链上存储, 群主配置):                                │   │
│  │                                                              │   │
│  │  ┌───────────────────────────────────────────────────────┐  │   │
│  │  │ 角色         │ 链上条件                │ TG 权限        │  │   │
│  │  ├──────────────┼────────────────────────┼───────────────┤  │   │
│  │  │ 🐋 鲸鱼      │ balance ≥ 10000 ATOM   │ 管理员权限    │  │   │
│  │  │ 💎 钻石持有者 │ 持有 Diamond NFT       │ 可置顶/删消息 │  │   │
│  │  │ 🏛️ DAO 委员  │ is_council_member      │ 管理员权限    │  │   │
│  │  │ ⭐ 活跃贡献者 │ reputation ≥ 8000      │ 可邀请新人    │  │   │
│  │  │ 🆕 新人      │ join_time < 7d         │ 限制发链接    │  │   │
│  │  │ 📵 观察者     │ balance < 10 ATOM      │ 只读          │  │   │
│  │  └──────────────┴────────────────────────┴───────────────┘  │   │
│  │                                                              │   │
│  │  动态权限更新:                                                │   │
│  │                                                              │   │
│  │  链上事件: Alice 卖出 ATOM, 余额从 15000 → 5000               │   │
│  │       │                                                      │   │
│  │       ▼                                                      │   │
│  │  节点监听链上变化:                                            │   │
│  │  "Alice 不再满足鲸鱼条件 (≥10000)"                            │   │
│  │       │                                                      │   │
│  │       ▼                                                      │   │
│  │  M/K 节点共识确认                                             │   │
│  │       │                                                      │   │
│  │       ▼                                                      │   │
│  │  指令 → Agent: promoteChatMember(alice, 降低权限)             │   │
│  │                                                              │   │
│  │  反向: Alice 买入 ATOM, 余额 → 12000                         │   │
│  │  → 自动恢复鲸鱼权限                                          │   │
│  │                                                              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  权限变更审计 (基于 chat_member Update):                               │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  每次权限变更 → Telegram 发送 chat_member Update               │   │
│  │  → 节点记录变更日志:                                          │   │
│  │                                                              │   │
│  │  {                                                           │   │
│  │    user: 888,                                                │   │
│  │    old_status: "administrator",                               │   │
│  │    new_status: "member",                                      │   │
│  │    changed_by: "nexus_bot",                                  │   │
│  │    reason: "balance_below_threshold",                         │   │
│  │    consensus_proof: { ... },                                  │   │
│  │    chain_block: 12345,                                        │   │
│  │    timestamp: 1738850400                                      │   │
│  │  }                                                           │   │
│  │                                                              │   │
│  │  → 日志哈希批量上链 → 不可篡改的权限变更历史                   │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 13.2.5 话题 (Forum Topics) 自动管理

```
┌──────────────────────────────────────────────────────────────────────┐
│                    话题群组自动化管理                                    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  适用: chat.is_forum = true 的超级群 (Telegram 话题功能)              │
│                                                                      │
│  触发事件:                                                            │
│  • forum_topic_created — 新话题创建                                   │
│  • forum_topic_edited  — 话题名称/图标编辑                             │
│  • forum_topic_closed  — 话题关闭                                     │
│  • forum_topic_reopened — 话题重新开启                                 │
│  • message.message_thread_id — 消息属于哪个话题                        │
│                                                                      │
│  自动化功能:                                                          │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ 功能 1: 按链上提案自动创建话题                                 │   │
│  │                                                              │   │
│  │ 链上事件: new_proposal(id=42, title="扩容方案")                │   │
│  │     │                                                        │   │
│  │     ▼                                                        │   │
│  │ 节点共识 → Agent 执行:                                        │   │
│  │ createForumTopic(chat_id, name="📋 提案#42: 扩容方案")         │   │
│  │ → 自动在该话题发布提案内容 + 投票按钮                          │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ 功能 2: 话题内消息路由                                         │   │
│  │                                                              │   │
│  │ 不同话题 → 不同业务逻辑:                                      │   │
│  │                                                              │   │
│  │ thread_id=1 "💰 交易"  → 解析 /swap /buy /sell 命令          │   │
│  │ thread_id=2 "🗳️ 治理"  → 解析投票和提案讨论                   │   │
│  │ thread_id=3 "❓ 问答"  → AI 自动回答 + 知识库检索             │   │
│  │ thread_id=4 "🔔 公告"  → 只允许管理员发言，自动转发到其他渠道  │   │
│  │ thread_id=5 "🐛 反馈"  → 自动创建 issue，追踪状态              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ 功能 3: 话题生命周期管理                                       │   │
│  │                                                              │   │
│  │ • 提案投票结束 → 自动关闭对应话题                              │   │
│  │ • 话题 7 天无人发言 → 自动归档                                │   │
│  │ • 新 epoch 开始 → 自动创建当期讨论话题                        │   │
│  │ • 话题消息超 1000 条 → 创建续集话题 + 关闭旧话题              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 13.3 管理指令协议与回传机制

```
┌──────────────────────────────────────────────────────────────────────┐
│             节点 → Agent 管理指令协议                                   │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  指令格式:                                                            │
│                                                                      │
│  POST https://{agent_host}/v1/admin-action                           │
│  Content-Type: application/json                                      │
│  X-Nexus-Consensus-Proof: {多节点签名聚合}                           │
│                                                                      │
│  {                                                                   │
│    "action_id": "SHA256(action_type + params + timestamp)",          │
│    "action_type": "banChatMember",                                   │
│    "params": {                                                       │
│      "chat_id": -1001234567890,                                      │
│      "user_id": 777,                                                 │
│      "until_date": 0,          // 0 = 永久                          │
│      "revoke_messages": true                                         │
│    },                                                                │
│    "trigger": {                                                      │
│      "update_id": 123456789,                                         │
│      "msg_hash": "原始消息哈希",                                      │
│      "rule_matched": "spam_link_detected"                            │
│    },                                                                │
│    "consensus": {                                                    │
│      "decision": "BAN",                                              │
│      "total_nodes": 6,                                               │
│      "agree_nodes": 5,                                               │
│      "disagree_nodes": 1,                                            │
│      "node_signatures": [                                            │
│        {"node_id": "node1", "sig": "...", "decision": "BAN"},        │
│        {"node_id": "node2", "sig": "...", "decision": "BAN"},        │
│        ...                                                           │
│      ]                                                               │
│    },                                                                │
│    "timestamp": 1738850400                                           │
│  }                                                                   │
│                                                                      │
│  Agent 验证:                                                          │
│  ① 验证 M/K 节点签名 (查链上公钥)                                     │
│  ② 验证 action_type 在群主授权的动作范围内                              │
│  ③ 验证 trigger.msg_hash 确实是之前转发过的消息                        │
│  ④ 调用 Telegram Bot API 执行                                         │
│  ⑤ 返回执行结果给发送指令的节点                                        │
│                                                                      │
│  支持的管理动作:                                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ 动作                      │ Telegram API              │ 危险级 │  │
│  ├───────────────────────────┼──────────────────────────┼────────┤  │
│  │ 删除消息                  │ deleteMessage             │ 低     │  │
│  │ 发送消息/回复             │ sendMessage               │ 低     │  │
│  │ 发送带按钮消息            │ sendMessage + reply_markup│ 低     │  │
│  │ 置顶消息                  │ pinChatMessage            │ 低     │  │
│  │ 限制用户 (禁言)           │ restrictChatMember        │ 中     │  │
│  │ 解除限制                  │ restrictChatMember        │ 中     │  │
│  │ 踢出用户                  │ banChatMember             │ 高     │  │
│  │ 解除封禁                  │ unbanChatMember           │ 高     │  │
│  │ 提升管理员                │ promoteChatMember         │ 高     │  │
│  │ 降级管理员                │ promoteChatMember         │ 高     │  │
│  │ 审批入群申请              │ approveChatJoinRequest    │ 中     │  │
│  │ 拒绝入群申请              │ declineChatJoinRequest    │ 中     │  │
│  │ 创建话题                  │ createForumTopic          │ 低     │  │
│  │ 关闭/重开话题             │ closeForumTopic           │ 低     │  │
│  │ 设置群头像/名称           │ setChatPhoto/Title        │ 高     │  │
│  └───────────────────────────┴──────────────────────────┴────────┘  │
│                                                                      │
│  群主权限控制 (链上配置):                                               │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │                                                                │  │
│  │  群主可精确控制哪些动作允许节点自动执行:                          │  │
│  │                                                                │  │
│  │  {                                                             │  │
│  │    "auto_allowed": [                                           │  │
│  │      "deleteMessage",     // 删消息自动执行                     │  │
│  │      "restrictChatMember", // 禁言自动执行                      │  │
│  │      "sendMessage",        // 发消息自动执行                    │  │
│  │      "approveChatJoinRequest" // 审批入群自动执行               │  │
│  │    ],                                                          │  │
│  │    "require_owner_confirm": [                                  │  │
│  │      "banChatMember",     // 踢人需群主确认                     │  │
│  │      "promoteChatMember"  // 提升管理员需群主确认               │  │
│  │    ],                                                          │  │
│  │    "forbidden": [                                              │  │
│  │      "setChatPhoto",      // 禁止自动改群头像                   │  │
│  │      "setChatTitle"       // 禁止自动改群名                     │  │
│  │    ]                                                           │  │
│  │  }                                                             │  │
│  │                                                                │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 13.4 链上群管理 Pallet 设计

```rust
// pallets/nexus/bot-group-mgmt/src/lib.rs

/// 群组管理规则 (链上存储, 群主配置)
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct GroupConfig<T: Config> {
    pub owner: T::AccountId,
    pub chat_id_hash: [u8; 16],

    /// 入群验证规则
    pub join_rules: BoundedVec<JoinRule<T>, ConstU32<10>>,

    /// 反垃圾规则配置
    pub spam_config: SpamConfig,

    /// 角色系统
    pub roles: BoundedVec<Role<T>, ConstU32<20>>,

    /// 节点自动执行权限
    pub auto_actions: BoundedVec<ActionPermission, ConstU32<20>>,

    /// 新人验证级别 (1-6)
    pub verification_level: u8,

    /// 新人限制时长 (区块数)
    pub newbie_restriction_blocks: BlockNumberFor<T>,
}

/// 入群规则
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub enum JoinRule<T: Config> {
    /// Token 持仓门槛
    TokenBalance {
        asset_id: u32,
        min_balance: BalanceOf<T>,
    },
    /// NFT 持有
    NftOwnership {
        collection_id: u32,
    },
    /// 链上信誉
    ReputationScore {
        min_score: u16,
    },
    /// 白名单
    Whitelist {
        list_id: u32,
    },
    /// 推荐人
    Referral {
        required_referrer_reputation: u16,
    },
    /// 复合条件
    And(Box<JoinRule<T>>, Box<JoinRule<T>>),
    Or(Box<JoinRule<T>>, Box<JoinRule<T>>),
}

/// 反垃圾配置
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct SpamConfig {
    /// 新人多久内不能发链接 (秒)
    pub newbie_link_cooldown: u32,
    /// 刷屏阈值 (N条/M秒)
    pub flood_threshold_count: u8,
    pub flood_threshold_seconds: u8,
    /// 是否启用 AI 内容检测
    pub ai_content_filter: bool,
    /// 域名黑名单 (哈希列表)
    pub domain_blacklist_hash: [u8; 32],
}

/// 动态角色
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct Role<T: Config> {
    pub role_id: u8,
    pub name_hash: [u8; 16],
    /// 满足此条件获得该角色
    pub condition: JoinRule<T>,
    /// TG 权限映射
    pub tg_permissions: TgPermissions,
}

#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct TgPermissions {
    pub can_send_messages: bool,
    pub can_send_media: bool,
    pub can_send_polls: bool,
    pub can_add_web_page_previews: bool,
    pub can_invite_users: bool,
    pub can_pin_messages: bool,
    pub can_manage_topics: bool,
    pub can_delete_messages: bool,
    pub can_restrict_members: bool,
    pub can_promote_members: bool,
}

/// 管理动作日志 (链上存证)
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct AdminActionLog<T: Config> {
    pub action_hash: [u8; 32],
    pub chat_id_hash: [u8; 16],
    pub target_user_hash: [u8; 16],
    pub action_type: ActionType,
    pub reason_hash: [u8; 32],
    pub consensus_count: u8,
    pub executed_at: BlockNumberFor<T>,
}

#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub enum ActionType {
    Approve,
    Decline,
    Ban,
    Unban,
    Mute,
    Unmute,
    Promote,
    Demote,
    DeleteMessage,
    Warn,
}

#[pallet::storage]
pub type GroupConfigs<T> = StorageMap<
    _, Blake2_128Concat, [u8; 16], GroupConfig<T>  // chat_id_hash → config
>;

#[pallet::storage]
/// TG user_id → 链上地址绑定
pub type UserBindings<T> = StorageMap<
    _, Blake2_128Concat, u64, T::AccountId
>;

#[pallet::storage]
/// 管理动作日志 (最近 1000 条)
pub type ActionLogs<T> = StorageMap<
    _, Blake2_128Concat, [u8; 16],  // chat_id_hash
    BoundedVec<AdminActionLog<T>, ConstU32<1000>>
>;

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 群主设置群组管理规则
    #[pallet::call_index(0)]
    pub fn set_group_config(
        origin: OriginFor<T>,
        chat_id_hash: [u8; 16],
        config: GroupConfig<T>,
    ) -> DispatchResult;

    /// 用户绑定 TG ID → 链上地址 (需签名证明)
    #[pallet::call_index(1)]
    pub fn bind_telegram_user(
        origin: OriginFor<T>,
        tg_user_id: u64,
        proof_signature: [u8; 64],
    ) -> DispatchResult;

    /// 节点批量提交管理动作日志
    #[pallet::call_index(2)]
    pub fn submit_action_logs(
        origin: OriginFor<T>,
        chat_id_hash: [u8; 16],
        logs: Vec<AdminActionLog<T>>,
    ) -> DispatchResult;

    /// 更新入群规则
    #[pallet::call_index(3)]
    pub fn update_join_rules(
        origin: OriginFor<T>,
        chat_id_hash: [u8; 16],
        rules: BoundedVec<JoinRule<T>, ConstU32<10>>,
    ) -> DispatchResult;

    /// 更新角色系统
    #[pallet::call_index(4)]
    pub fn update_roles(
        origin: OriginFor<T>,
        chat_id_hash: [u8; 16],
        roles: BoundedVec<Role<T>, ConstU32<20>>,
    ) -> DispatchResult;
}
```

### 13.5 完整场景时序图

```
完整场景: 垃圾用户加群 → 验证失败 → 自动踢出

时间轴 ──────────────────────────────────────────────────────────▶

T₀  │ Spammer 点击邀请链接加群
    │
T₁  │ Telegram → Agent: new_chat_members(user_id=999)
    │ Agent → K 个节点: 签名转发
    │
T₂  │ 节点共识: restrictChatMember(999, 禁言)
    │ → Agent 执行限制
    │ 节点共识: sendMessage("验证: 3+7=?", buttons=[8,10,12])
    │ → Agent 发送验证消息
    │
T₃  │ ... 等待 120 秒 ...
    │
    │ 情况 A: Spammer 不理验证
T₄a │ 节点: 超时检测 → 共识: banChatMember(999)
    │ → Agent 执行踢出
    │ → 节点: 链上记录 ActionLog{BAN, reason="verification_timeout"}
    │
    │ 情况 B: Spammer 点击了错误答案
T₄b │ Telegram → Agent: callback_query(data="captcha_answer_8")
    │ Agent → K 节点: 签名转发
    │ 节点: 3+7≠8 → 共识: 发送 "答案错误，还剩 2 次机会"
    │ ... 连续错误 3 次 ...
    │ 节点共识: banChatMember(999)
    │ → Agent 执行踢出
    │
    │ 情况 C: Spammer 通过验证但随后发垃圾
T₄c │ callback_query(data="captcha_answer_10") → 验证通过
    │ 节点共识: restrictChatMember(999, 恢复权限)
T₅c │ Spammer 发送: "🔥 Join this amazing crypto pump group! t.me/scam"
    │ Agent → K 节点: 签名转发
    │ 节点规则引擎:
    │   entities 含 url → 检测域名 → t.me/scam 在黑名单
    │   用户是新人 (join < 24h) + 发链接 → 高风险
    │ 节点共识: deleteMessage + warn
    │ → Agent 删除消息 + 发送警告
T₆c │ Spammer 再发: "Best signals here → @scam_channel"
    │ 节点: 第 2 次违规 → 共识: mute(999, 24h)
    │ → Agent 禁言 24 小时
    │ 如第 3 次: → 共识: ban(999)

时间轴 ──────────────────────────────────────────────────────────▶

完整场景: 正常用户加群 → 绑定钱包 → 获得动态角色

T₀  │ Alice 点击邀请链接
T₁  │ new_chat_members(Alice, id=123)
    │ 节点共识: restrict + 发送验证消息
    │
T₂  │ Alice 点击 [🔗 绑定钱包]
    │ callback_query(data="bind_wallet")
    │ 节点共识: sendMessage("请访问 https://app.nexus.io/bind?tg=123")
    │
T₃  │ Alice 在 DApp 连接钱包，签名证明:
    │   sign("I am Telegram user 123", private_key) → signature
    │ DApp 调用链上: bind_telegram_user(tg_id=123, signature)
    │
T₄  │ 节点监听链上事件: UserBound(tg_id=123, addr=5Gxyz)
    │ 节点查询: balance(5Gxyz) = 15000 ATOM
    │ 匹配角色: 🐋 鲸鱼 (≥10000 ATOM)
    │
T₅  │ 节点共识:
    │ ① restrictChatMember(123, 恢复全部权限)
    │ ② promoteChatMember(123, 管理员权限)
    │ ③ sendMessage("🐋 欢迎鲸鱼 Alice! 你已获得管理员权限")
    │ → Agent 执行
    │
T₆  │ 日后: Alice 链上余额变化
    │ 节点持续监控 → 动态调整权限
```

### 13.6 群管理功能总览表

| 功能类别 | 具体功能 | 触发数据 | 多节点共识作用 |
|----------|---------|----------|--------------|
| **入群审核** | Token-gated 入群 | `chat_join_request` | 多节点独立查链上余额，防单节点放水 |
| | NFT 持有验证 | `chat_join_request` | 多节点独立查 NFT ownership |
| | CAPTCHA 人机验证 | `callback_query` | 多节点确认答案一致 |
| | 钱包绑定验证 | `callback_query` → 链上事件 | 多节点验证签名和链上绑定 |
| **反垃圾** | 链接检测 | `message.entities[url]` | 防单节点误判/纵容 |
| | 关键词过滤 | `message.text` | 多节点使用相同规则，结果可审计 |
| | 刷屏限制 | 消息频率统计 | 多节点独立计数，一致才禁言 |
| | AI 内容分析 | `message.text` | 多节点独立推理，多数决 |
| **动态权限** | 链上资产驱动角色 | 链上余额变化事件 | 多节点独立查链，防篡改 |
| | 信誉分驱动权限 | `pallet-bot-consensus.reputation` | 链上数据，无争议 |
| | 新人冷却期 | 加入时间 + 消息事件 | 节点共识判定冷却结束 |
| **话题管理** | 提案自动创建话题 | 链上 `new_proposal` 事件 | 多节点确认同一提案 |
| | 话题自动归档 | 消息时间统计 | 多节点一致判定超时 |
| | 话题内消息路由 | `message.message_thread_id` | 不同话题路由到不同业务 |
| **群信息** | 活跃度统计 | 消息聚合 | 多节点独立统计，交叉校验 |
| | 成员变动追踪 | `chat_member` | 日志哈希上链，不可篡改 |
| | 投票与治理 | `callback_query` / `poll_answer` | 多节点记票防篡改 |
| **安全** | 管理动作审计 | 所有管理指令 | 链上日志，群主可审计 |
| | 群主权限兜底 | `require_owner_confirm` 配置 | 高危动作需群主二次确认 |
| | 异常告警 | 节点判定分歧 | 分歧时暂停动作 + 通知群主 |

---

## 14. Leader 节点选举与执行机制 — 深度分析

### 14.1 为什么需要 Leader？

```
核心问题:

  M/K 节点共识达成后，谁来执行管理指令？

  ❌ 所有节点都发 → Agent 收到 M 条重复指令 → 重复执行 (踢人踢 M 次?)
  ❌ 随机一个节点发 → 该节点宕机则无人执行
  ❌ 没有协调 → 多个节点同时发，Agent 并发处理产生竞态

  ✅ 选出一个 Leader → 由 Leader 唯一负责向 Agent 发送执行指令
     其他节点作为 Backup → Leader 失败时接替
```

### 14.2 Leader 选举算法

#### 14.2.1 确定性轮转 Leader (推荐方案)

```
┌──────────────────────────────────────────────────────────────────────┐
│                    确定性 Leader 选举                                   │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  核心思想: 用消息本身的属性确定性地选出 Leader                          │
│  所有节点用相同算法独立计算，结果一致，无需额外通信                       │
│                                                                      │
│  算法:                                                                │
│                                                                      │
│  // 输入: 本次消息的 K 个目标节点（已按 ID 排序）                       │
│  // 输入: 消息 sequence 号                                            │
│  fn select_leader(                                                   │
│      target_nodes: &[NodeId; K],  // 确定性随机选出的 K 个节点         │
│      sequence: u64,                                                  │
│  ) -> (NodeId, Vec<NodeId>) {                                        │
│      let leader_index = sequence % K;                                │
│      let leader = target_nodes[leader_index];                        │
│      let backups = target_nodes                                      │
│          .iter()                                                     │
│          .filter(|n| **n != leader)                                  │
│          .collect();                                                 │
│      (leader, backups)  // 返回 Leader + 有序 backup 列表             │
│  }                                                                   │
│                                                                      │
│  示例 (K=5, 节点 [N2, N5, N7, N9, N14]):                              │
│                                                                      │
│  sequence=40 → 40%5=0 → Leader=N2,  Backup=[N5,N7,N9,N14]           │
│  sequence=41 → 41%5=1 → Leader=N5,  Backup=[N2,N7,N9,N14]           │
│  sequence=42 → 42%5=2 → Leader=N7,  Backup=[N2,N5,N9,N14]           │
│  sequence=43 → 43%5=3 → Leader=N9,  Backup=[N2,N5,N7,N14]           │
│  sequence=44 → 44%5=4 → Leader=N14, Backup=[N2,N5,N7,N9]            │
│  sequence=45 → 45%5=0 → Leader=N2   (轮回)                           │
│                                                                      │
│  特性:                                                                │
│  ✅ 零通信开销 — 不需要额外的选举消息                                  │
│  ✅ 确定性 — 所有节点独立算出相同结果                                  │
│  ✅ 公平轮转 — 每个节点轮流当 Leader，负载均衡                         │
│  ✅ 可验证 — Agent 也能算出谁是 Leader，拒绝非 Leader 的指令            │
│  ✅ 有序 backup — Leader 失败时，按固定顺序递补                        │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 14.2.2 基于信誉加权的 Leader 选举 (高级方案)

```
┌──────────────────────────────────────────────────────────────────────┐
│                    信誉加权 Leader 选举                                 │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  动机: 简单轮转不考虑节点质量                                          │
│  • 低信誉节点当 Leader → 执行失败概率高                                │
│  • 高延迟节点当 Leader → 整体延迟变高                                  │
│  • 频繁离线节点当 Leader → 需要频繁 failover                           │
│                                                                      │
│  改进算法:                                                            │
│                                                                      │
│  fn select_leader_weighted(                                          │
│      target_nodes: &[NodeId; K],                                     │
│      sequence: u64,                                                  │
│      reputations: &HashMap<NodeId, u16>,  // 链上信誉分               │
│  ) -> (NodeId, Vec<NodeId>) {                                        │
│                                                                      │
│      // 1. 计算加权概率                                               │
│      let weights: Vec<u64> = target_nodes                            │
│          .iter()                                                     │
│          .map(|n| reputations[n] as u64)                             │
│          .collect();                                                 │
│      let total_weight = weights.iter().sum::<u64>();                  │
│                                                                      │
│      // 2. 用 sequence 作为确定性随机源                                │
│      let seed = hash(sequence.to_le_bytes());                        │
│      let pick = u64::from_le_bytes(seed[0..8]) % total_weight;       │
│                                                                      │
│      // 3. 加权选择                                                   │
│      let mut acc = 0;                                                │
│      for (i, w) in weights.iter().enumerate() {                      │
│          acc += w;                                                   │
│          if pick < acc {                                             │
│              let leader = target_nodes[i];                            │
│              // backup 按信誉降序排列                                  │
│              let backups = sort_by_reputation_desc(                   │
│                  target_nodes.except(leader)                          │
│              );                                                      │
│              return (leader, backups);                                │
│          }                                                           │
│      }                                                               │
│  }                                                                   │
│                                                                      │
│  效果:                                                                │
│  • 信誉 9000 的节点被选为 Leader 的概率是 3000 的 3 倍                 │
│  • 但低信誉节点仍有机会 (避免垄断)                                     │
│  • backup 按信誉排序 → 高信誉节点优先接替                              │
│                                                                      │
│  示例 (K=3, 信誉=[N2:9000, N7:6000, N14:3000]):                       │
│  total = 18000                                                       │
│  N2 当 Leader 概率 = 9000/18000 = 50%                                │
│  N7 当 Leader 概率 = 6000/18000 = 33%                                │
│  N14 当 Leader 概率 = 3000/18000 = 17%                               │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 14.3 Leader 执行流程

```
时间轴 ──────────────────────────────────────────────────────────▶

T₀: M/K 共识达成
    │
    ▼
T₁: 所有 K 个节点独立计算 Leader (结果一致)
    │
    ├── 我是 Leader → 进入执行路径
    └── 我不是 Leader → 进入监督路径
    │
    ▼
┌──────────────────────────────┐  ┌────────────────────────────────┐
│ Leader 执行路径               │  │ Backup 监督路径                  │
│                              │  │                                │
│ T₂: 构造管理指令              │  │ T₂: 启动 Leader 超时计时器      │
│     附加共识证明 (M 个签名)   │  │     timeout = 3 秒              │
│     │                        │  │     │                          │
│     ▼                        │  │     │                          │
│ T₃: POST → Agent             │  │     │ 等待 Leader 执行结果广播  │
│     /v1/admin-action          │  │     │                          │
│     │                        │  │     │                          │
│     ▼                        │  │     │                          │
│ T₄: Agent 验证:              │  │     │                          │
│     ① 验证发送者是 Leader     │  │     │                          │
│       (Agent 也算 Leader)    │  │     │                          │
│     ② 验证 M/K 签名          │  │     │                          │
│     ③ 验证 action 在授权内   │  │     │                          │
│     ④ 调用 Telegram API      │  │     │                          │
│     ⑤ 返回结果给 Leader      │  │     │                          │
│     │                        │  │     │                          │
│     ▼                        │  │     │                          │
│ T₅: Leader 收到结果           │  │     │                          │
│     广播 ExecutionResult      │  │     ▼                          │
│     给所有 K-1 个节点    ─────│──│──▶ 收到 ExecutionResult        │
│     │                        │  │     │                          │
│     │                        │  │     ├── 成功 → 记录日志         │
│     │                        │  │     │   取消超时计时器           │
│     │                        │  │     │                          │
│     │                        │  │     └── 失败 → 见 14.4 failover│
│     ▼                        │  │                                │
│ T₆: 日志上链                 │  │                                │
│     action_hash + result     │  │                                │
└──────────────────────────────┘  └────────────────────────────────┘
```

### 14.4 Leader 失败与 Failover 机制

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Leader Failover 完整流程                            │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  失败场景:                                                            │
│  ① Leader 宕机 — 共识达成后 Leader 突然离线                           │
│  ② Leader 网络故障 — Leader 无法连接 Agent                            │
│  ③ Leader 拒绝执行 — 恶意 Leader 故意不发指令                         │
│  ④ Agent 拒绝 Leader — Agent 验证失败 (签名/权限)                     │
│  ⑤ Telegram API 失败 — API 限流/网络问题                             │
│                                                                      │
│  Failover 流程:                                                       │
│                                                                      │
│  T₀: 共识达成，Leader = N7，Backup = [N2, N5, N9, N14]               │
│      │                                                               │
│      ▼                                                               │
│  T₁: 所有 backup 启动 Leader 超时计时器                                │
│      deadline = now + LEADER_TIMEOUT (默认 3 秒)                      │
│      │                                                               │
│      │  ┌────────────────────────────────────────┐                   │
│      │  │ 正常路径: Leader N7 在 3s 内广播结果     │                   │
│      │  │ → 所有 backup 取消计时器 → 流程结束      │                   │
│      │  └────────────────────────────────────────┘                   │
│      │                                                               │
│      │  ┌────────────────────────────────────────┐                   │
│      │  │ 异常路径: 3 秒超时，未收到结果           │                   │
│      │  └────────────────────┬───────────────────┘                   │
│      │                       ▼                                       │
│  T₂: Backup #1 (N2) 接替成为新 Leader                                 │
│      │                                                               │
│      ├── N2 向其他 backup 广播 LeaderTakeover:                        │
│      │   {                                                           │
│      │     type: "LeaderTakeover",                                   │
│      │     original_leader: "N7",                                    │
│      │     reason: "timeout_3s",                                     │
│      │     new_leader: "N2",                                         │
│      │     backup_rank: 1,                                           │
│      │     takeover_signature: "..."                                 │
│      │   }                                                           │
│      │                                                               │
│      ├── N2 向 Agent 发送管理指令                                      │
│      │   Header 附加: X-Nexus-Leader-Takeover: true                 │
│      │                X-Nexus-Original-Leader: N7                   │
│      │                X-Nexus-Takeover-Rank: 1                      │
│      │                                                               │
│      ├── Agent 验证:                                                  │
│      │   ① N2 确实是 backup_rank=1 (Agent 独立计算验证)               │
│      │   ② 超时时间合理 (Agent 也有计时)                              │
│      │   ③ 原始 Leader N7 确实未执行 (幂等性检查)                     │
│      │   ④ 执行指令                                                  │
│      │                                                               │
│      └── N2 广播执行结果                                               │
│      │                                                               │
│      │  ┌────────────────────────────────────────┐                   │
│      │  │ 如果 N2 也超时 (再过 3s)?              │                   │
│      │  │ → Backup #2 (N5) 接替                  │                   │
│      │  │ → 以此类推直到 Backup #K-1              │                   │
│      │  │ → 全部超时 → 标记 ACTION_FAILED 上链    │                   │
│      │  └────────────────────────────────────────┘                   │
│      │                                                               │
│      ▼                                                               │
│  T₃: Leader 失败上报                                                   │
│      节点调用链上 report_leader_failure():                             │
│      - 记录: N7 在 msg_id=xxx 上 Leader 超时                         │
│      - N7 信誉 -20                                                   │
│      - 连续 3 次 Leader 超时 → 信誉 -200 + 降为观察节点               │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 14.4.1 幂等性保障 — 防止重复执行

```
┌──────────────────────────────────────────────────────────────────────┐
│                    幂等性设计 (防重复执行)                               │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  问题: Leader 实际执行成功了，但广播结果时网络断了                       │
│  → Backup 以为 Leader 失败 → Backup 也执行 → 重复执行!                │
│                                                                      │
│  例: 踢人踢了两次 (无害但浪费)                                         │
│  例: 发消息发了两次 (用户困惑)                                         │
│  例: 审批入群审批了两次 (无害)                                         │
│                                                                      │
│  解决方案: Agent 侧幂等性表                                            │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Agent 维护最近 1000 条已执行动作的表:                         │   │
│  │                                                              │   │
│  │  executed_actions: HashMap<ActionId, ExecutionResult>          │   │
│  │                                                              │   │
│  │  收到指令时:                                                  │   │
│  │  1. 计算 action_id = SHA256(action_type + params + msg_id)   │   │
│  │  2. 查 executed_actions:                                      │   │
│  │     - 已存在 → 直接返回之前的结果 (不重复调用 TG API)          │   │
│  │     - 不存在 → 执行 → 记录结果到 executed_actions              │   │
│  │                                                              │   │
│  │  效果:                                                        │   │
│  │  • Leader 执行成功但广播失败 → Backup 重试 → Agent 返回缓存    │   │
│  │  • 同一动作无论谁发、发几次，只执行一次                        │   │
│  │  • action_id 绑定 msg_id → 不同消息的同类动作不会冲突          │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  Telegram API 本身的幂等性:                                           │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ API                  │ 重复调用效果          │ 幂等？          │   │
│  ├──────────────────────┼─────────────────────┼────────────────┤   │
│  │ banChatMember        │ 第二次返回成功(无害)   │ ✅ 天然幂等    │   │
│  │ unbanChatMember      │ 第二次返回成功(无害)   │ ✅ 天然幂等    │   │
│  │ restrictChatMember   │ 覆盖前次限制设置      │ ✅ 天然幂等    │   │
│  │ deleteMessage        │ 第二次返回错误(已删)   │ ⚠️ 需处理错误  │   │
│  │ sendMessage          │ 第二次会重复发送!      │ ❌ 非幂等      │   │
│  │ promoteChatMember    │ 覆盖前次权限设置      │ ✅ 天然幂等    │   │
│  │ approveChatJoinReq   │ 第二次返回错误(已处理) │ ⚠️ 需处理错误  │   │
│  │ createForumTopic     │ 第二次会重复创建!      │ ❌ 非幂等      │   │
│  └──────────────────────┴─────────────────────┴────────────────┘   │
│                                                                      │
│  对于非幂等 API (sendMessage, createForumTopic):                      │
│  Agent 的 executed_actions 表是唯一防线                                │
│  → 必须保证 action_id 精确去重                                        │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 14.5 防止 Leader 作恶

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Leader 作恶分析与防御                                │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  攻击 1: Leader 篡改指令内容                                          │
│  ──────────────────────────                                          │
│  共识: BAN user_777                                                  │
│  Leader 篡改为: BAN user_888 (踢错人)                                │
│                                                                      │
│  防御:                                                                │
│  ① Agent 收到指令时，验证 consensus.node_signatures                  │
│  ② M 个节点的签名覆盖了原始 action_type + params                     │
│  ③ Leader 篡改 params → 签名验证失败 → Agent 拒绝执行                │
│  ④ 结论: ❌ 攻击失败                                                 │
│                                                                      │
│  攻击 2: Leader 不执行 (消极罢工)                                     │
│  ──────────────────────────                                          │
│  共识达成后，Leader 故意不发指令                                       │
│                                                                      │
│  防御:                                                                │
│  ① Backup 节点 3 秒超时 → 自动接替 (见 14.4)                         │
│  ② Leader 被上报: report_leader_failure → 信誉 -20                   │
│  ③ 连续罢工 → 信誉骤降 → 降为观察节点 → 不再被选为 Leader             │
│  ④ 结论: ❌ 攻击效果有限 (最多延迟 3 秒)                              │
│                                                                      │
│  攻击 3: Leader 执行但不广播结果                                      │
│  ──────────────────────────                                          │
│  Leader 执行了指令，但不告诉其他节点                                    │
│                                                                      │
│  防御:                                                                │
│  ① Backup 超时 → 尝试接替执行                                        │
│  ② Agent 幂等性表 → 不会重复执行                                      │
│  ③ Backup 执行后收到 "already executed" → 知道 Leader 实际执行了       │
│  ④ Backup 上报 Leader 未广播结果 → 信誉 -10                          │
│  ⑤ 结论: ❌ 指令仍被执行，Leader 受惩罚                               │
│                                                                      │
│  攻击 4: Leader 伪造执行结果                                          │
│  ──────────────────────────                                          │
│  Leader 没有执行，但广播 "执行成功"                                     │
│                                                                      │
│  防御:                                                                │
│  ① Leader 广播的 ExecutionResult 需包含 Agent 的回执签名:             │
│     {                                                                │
│       action_id: "xxx",                                              │
│       result: "success",                                             │
│       telegram_response: { ... },                                    │
│       agent_receipt_signature: Ed25519.sign(                         │
│         agent_key, action_id + result                                │
│       )                                                              │
│     }                                                                │
│  ② 其他节点验证 agent_receipt_signature                               │
│  ③ Leader 没有 Agent 私钥 → 无法伪造回执                              │
│  ④ 结论: ❌ 攻击失败                                                 │
│                                                                      │
│  攻击 5: Leader 执行额外指令 (越权)                                    │
│  ──────────────────────────                                          │
│  共识只决定 BAN user_777，Leader 额外发 BAN user_888                  │
│                                                                      │
│  防御:                                                                │
│  ① Agent 对每条指令独立验证 M/K 共识签名                               │
│  ② 额外指令没有 M 个节点的签名 → Agent 拒绝                           │
│  ③ Agent 记录所有收到的指令 (包括被拒绝的) → 可审计                    │
│  ④ 结论: ❌ 攻击失败                                                 │
│                                                                      │
│  攻击 6: Leader 与 Agent 串谋                                         │
│  ──────────────────────────                                          │
│  Leader 是恶意节点 + Agent 也被入侵                                    │
│                                                                      │
│  防御:                                                                │
│  ① Agent 在群主本地 → 群主被入侵是群主自己的责任                       │
│  ② 但: 其他节点有完整的共识记录 + 实际执行日志                         │
│  ③ 如果 Agent 执行了无共识的指令 → 链上日志缺失对应共识                 │
│  ④ 审计发现: 链上 action_log 有记录但无 consensus_proof → 异常         │
│  ⑤ 结论: ⚠️ 可事后发现，但无法实时阻止                                │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 14.6 Leader 选举的完整状态机

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                    Leader 执行状态机                                  │
│                                                                     │
│  ┌──────────────┐                                                  │
│  │ CONSENSUS    │ ← M/K 节点共识达成                                │
│  │ REACHED      │                                                  │
│  └──────┬───────┘                                                  │
│         │ 计算 Leader                                               │
│         ▼                                                          │
│  ┌──────────────┐         ┌──────────────┐                         │
│  │ I_AM_LEADER  │         │ I_AM_BACKUP  │                         │
│  └──────┬───────┘         └──────┬───────┘                         │
│         │                        │                                  │
│         │ 发送指令到 Agent        │ 启动超时计时器 (3s)               │
│         ▼                        ▼                                  │
│  ┌──────────────┐         ┌──────────────┐                         │
│  │ EXECUTING    │         │ WATCHING     │                         │
│  └──────┬───────┘         └──────┬───────┘                         │
│         │                        │                                  │
│    ┌────┴────┐              ┌────┴────┐                            │
│    ▼         ▼              ▼         ▼                            │
│ ┌──────┐ ┌──────┐    ┌──────────┐ ┌──────────┐                    │
│ │ OK   │ │ FAIL │    │ GOT      │ │ TIMEOUT  │                    │
│ │      │ │      │    │ RESULT   │ │ (3s)     │                    │
│ └──┬───┘ └──┬───┘    └────┬─────┘ └────┬─────┘                    │
│    │        │             │             │                           │
│    ▼        ▼             ▼             ▼                           │
│ 广播      重试 1 次    记录日志      我是 Backup #1?                │
│ Result    │            完成         ┌────┴────┐                    │
│ │         ▼                        是        否                    │
│ │      ┌──────┐                    ▼         ▼                    │
│ │      │ FAIL │                接替为       等待                   │
│ │      │ 2次  │                新 Leader    Backup #1              │
│ │      └──┬───┘                    │        的结果                 │
│ │         ▼                        ▼         │                    │
│ │      标记失败               EXECUTING      │                    │
│ │      上报链上               (同左侧流程)   │                    │
│ ▼                                            ▼                    │
│ 完成                                    级联 failover              │
│                                         直到执行成功               │
│                                         或全部失败                 │
└─────────────────────────────────────────────────────────────────────┘
```

### 14.7 Agent 侧 Leader 验证

```
┌──────────────────────────────────────────────────────────────────────┐
│             Agent 如何验证 "你真的是 Leader"？                          │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Agent 收到管理指令时的验证流程:                                       │
│                                                                      │
│  POST /v1/admin-action                                               │
│  X-Nexus-Node-Id: N7                                                │
│  X-Nexus-Node-Signature: {...}                                      │
│  X-Nexus-Leader-Takeover: false                                     │
│                                                                      │
│  Agent 验证:                                                          │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ Step 1: 从指令中提取 msg_id + sequence                       │   │
│  │                                                              │   │
│  │ Step 2: 从共识签名中提取 target_nodes 列表                    │   │
│  │         (Agent 也缓存了活跃节点列表，可独立计算)               │   │
│  │                                                              │   │
│  │ Step 3: 独立计算 Leader                                      │   │
│  │         expected_leader = target_nodes[sequence % K]          │   │
│  │                                                              │   │
│  │ Step 4: 比对                                                  │   │
│  │                                                              │   │
│  │   if sender == expected_leader:                               │   │
│  │       ✅ 是 Leader → 执行                                    │   │
│  │                                                              │   │
│  │   elif X-Nexus-Leader-Takeover == true:                      │   │
│  │       // 是 Backup 接替                                       │   │
│  │       takeover_rank = header.Takeover-Rank                    │   │
│  │       expected_backup = backup_list[takeover_rank - 1]        │   │
│  │       if sender == expected_backup:                            │   │
│  │           // 验证时间: 至少等了 rank × LEADER_TIMEOUT         │   │
│  │           min_wait = takeover_rank × 3s                       │   │
│  │           if now - consensus_time >= min_wait:                 │   │
│  │               ✅ 合法接替 → 执行                              │   │
│  │           else:                                               │   │
│  │               ❌ 过早接替 → 拒绝                              │   │
│  │       else:                                                   │   │
│  │           ❌ 不是合法 backup → 拒绝                           │   │
│  │                                                              │   │
│  │   else:                                                       │   │
│  │       ❌ 非 Leader 非合法 backup → 拒绝                       │   │
│  │                                                              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  时间窗口约束 (防止过早/过晚 takeover):                                │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  时间线:                                                      │   │
│  │                                                              │   │
│  │  T₀         T₀+3s       T₀+6s       T₀+9s       T₀+30s     │   │
│  │  │           │           │           │           │           │   │
│  │  ▼           ▼           ▼           ▼           ▼           │   │
│  │  共识达成    Leader超时   Backup#1超时 Backup#2超时 最终超时   │   │
│  │  │           │           │           │           │           │   │
│  │  └─ Leader ──┘           │           │           │           │   │
│  │  只有 Leader              └─ Backup#1 ┘           │           │   │
│  │  能执行                   只有 Backup#1            └─ Backup#2│   │
│  │                           能执行                   能执行     │   │
│  │                                                              │   │
│  │  T₀+30s 后: 全部超时 → ACTION_FAILED → 链上记录             │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 14.8 ExecutionResult 协议

```json
// Leader 广播给所有 Backup 节点的执行结果

{
  "type": "ExecutionResult",
  "action_id": "SHA256(action_type + params + msg_id)",
  "msg_id": "原始消息 ID",
  "leader_node_id": "N7",
  "was_takeover": false,

  "result": {
    "status": "success",
    "telegram_response": {
      "ok": true,
      "result": true
    },
    "executed_at": 1738850402
  },

  // Agent 的回执签名 (Leader 无法伪造)
  "agent_receipt": {
    "action_id": "xxx",
    "status": "success",
    "agent_signature": "Ed25519.sign(agent_key, action_id + status + timestamp)"
  },

  "leader_signature": "Ed25519.sign(leader_key, full_result_hash)"
}
```

### 14.9 链上 Leader 相关存储

```rust
// pallets/nexus/bot-consensus/src/lib.rs (扩展)

/// Leader 执行统计
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub struct LeaderStats {
    /// 成功执行次数
    pub successful_leads: u64,
    /// 超时次数 (被 backup 接替)
    pub timeouts: u64,
    /// 连续超时次数 (当前)
    pub consecutive_timeouts: u8,
    /// 作为 Backup 接替成功次数
    pub successful_takeovers: u64,
    /// 上次成功执行的区块
    pub last_successful_lead: BlockNumberFor<T>,
}

#[pallet::storage]
pub type LeaderStatsMap<T> = StorageMap<
    _, Blake2_128Concat, BoundedVec<u8, ConstU32<32>>,  // node_id
    LeaderStats
>;

#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 上报 Leader 超时 (由 takeover 节点调用)
    #[pallet::call_index(10)]
    pub fn report_leader_timeout(
        origin: OriginFor<T>,
        failed_leader: BoundedVec<u8, ConstU32<32>>,
        msg_id: [u8; 32],
        takeover_node: BoundedVec<u8, ConstU32<32>>,
        takeover_proof: TakeoverProof,
    ) -> DispatchResult {
        // 验证 takeover 合法性
        // 更新 failed_leader 的 LeaderStats.timeouts
        // 更新 failed_leader 的 reputation (-20)
        // 如果 consecutive_timeouts >= 3:
        //   reputation -= 200
        //   node.status = Probation
    }
}
```

### 14.10 Leader 机制总结

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Leader 机制设计要点                                  │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────┬────────────────────────────────────────┐   │
│  │ 设计维度             │ 方案                                   │   │
│  ├─────────────────────┼────────────────────────────────────────┤   │
│  │ 选举方式             │ 确定性轮转 (sequence % K)              │   │
│  │                     │ 可选: 信誉加权随机                      │   │
│  │ 选举通信开销         │ 零 (所有人独立算出相同结果)             │   │
│  │ Failover 方式        │ 有序 Backup 超时接替 (3s 间隔)        │   │
│  │ 最大 failover 延迟   │ K × 3s (全部失败) → ACTION_FAILED     │   │
│  │ 幂等性保障           │ Agent 侧 executed_actions 表          │   │
│  │ 防篡改指令           │ M/K 共识签名覆盖完整指令内容           │   │
│  │ 防伪造结果           │ Agent 回执签名 (Leader 无法伪造)       │   │
│  │ 防消极罢工           │ 超时 → Backup 接替 → 信誉惩罚         │   │
│  │ 防越权执行           │ Agent 逐条验证共识签名                 │   │
│  │ 审计追踪             │ 链上 LeaderStats + ActionLogs         │   │
│  │ 公平性              │ 轮转保证每个节点均等承担 Leader 职责     │   │
│  └─────────────────────┴────────────────────────────────────────┘   │
│                                                                      │
│  安全保证:                                                            │
│  • Leader 篡改指令 → 签名验证失败 → Agent 拒绝 ✅                    │
│  • Leader 不执行   → 3s 后 Backup 接替 ✅                            │
│  • Leader 伪造结果 → Agent 回执签名验证失败 ✅                        │
│  • Leader 越权     → Agent 逐条验证共识签名 ✅                        │
│  • Leader 串谋 Agent → 事后审计可发现 ⚠️ (唯一弱点，限于群主被入侵)  │
│                                                                      │
│  性能指标:                                                            │
│  • 正常执行延迟: 共识后 ~200ms (Leader 直接执行)                      │
│  • 首次 failover: +3s                                                │
│  • 第 N 次 failover: +N×3s                                           │
│  • 全部失败 (K=5): 最长 15s 后标记失败                                │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 15. Gossip 协议 — 深度分析

### 15.1 Gossip 在方案 D 中的定位

```
方案 D 的三个通信阶段:

阶段 1: 群主 Agent → K 个节点 (单播/多播, HTTPS)
         ↑ 已有详细设计 (第 2 章)

阶段 2: K 个节点之间 Gossip 互通 (本章重点)
         用途: 消息确认、共识达成、拉取缺失、异常检测

阶段 3: Leader → Agent 回传指令 (单播, HTTPS)
         ↑ 已有详细设计 (第 14 章)

本章深入分析阶段 2: 节点间 Gossip 协议的完整设计
```

### 15.2 网络拓扑

#### 15.2.1 两层拓扑结构

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Gossip 网络拓扑                                      │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  层 1: 全局持久连接 (Background Mesh)                                 │
│  ─────────────────────────────────                                   │
│  所有 N 个活跃节点之间维护持久 WebSocket 连接                           │
│  用途: 心跳检测、节点发现、链上事件同步                                 │
│                                                                      │
│     N1 ◀───────▶ N2 ◀───────▶ N3                                    │
│      ▲ ╲          ▲          ╱ ▲                                    │
│      │   ╲         │        ╱   │                                    │
│      │     ╲       │      ╱     │                                    │
│      │       ╲     │    ╱       │                                    │
│      │         ╲   │  ╱         │                                    │
│      ▼           ╲ ▼╱           ▼                                    │
│     N4 ◀───────▶ N5 ◀───────▶ N6                                    │
│      ▲           ╱ ▲╲           ▲                                    │
│      │         ╱   │  ╲         │                                    │
│      │       ╱     │    ╲       │                                    │
│      │     ╱       │      ╲     │                                    │
│      │   ╱         │        ╲   │                                    │
│      ▼ ╱           ▼          ╲ ▼                                    │
│     N7 ◀───────▶ N8 ◀───────▶ N9                                    │
│                                                                      │
│  连接数: N×(N-1)/2                                                   │
│  N=9 → 36 条连接 (可承受)                                            │
│  N=21 → 210 条连接 (仍可承受，每条 WebSocket 开销极低)                 │
│                                                                      │
│                                                                      │
│  层 2: 消息级按需 Gossip (Per-Message Scope)                          │
│  ──────────────────────────────────────                              │
│  每条消息只在 K 个目标节点之间 gossip                                  │
│  不涉及非目标节点 → 大幅减少无关流量                                    │
│                                                                      │
│  例: K=5, 目标节点 = [N2, N5, N7, N9, N14]                           │
│                                                                      │
│  只有这 5 个节点互相 gossip:                                          │
│                                                                      │
│     N2 ◀────▶ N5                                                    │
│      ▲ ╲      ╱ ▲                                                    │
│      │   ╲  ╱   │                                                    │
│      │    N7    │          N1,N3,N4,N6,N8 等非目标节点                 │
│      │   ╱  ╲   │          不参与此消息的 gossip                       │
│      ▼ ╱      ╲ ▼          (节省带宽)                                │
│     N9 ◀────▶ N14                                                   │
│                                                                      │
│  连接复用: 层 2 的 gossip 消息复用层 1 的 WebSocket 连接               │
│  → 无需为每条消息建新连接                                              │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 15.2.2 连接管理

```
┌──────────────────────────────────────────────────────────────────────┐
│                    WebSocket 连接管理                                   │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  建立连接:                                                            │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  1. 节点启动时，从链上读取 ActiveNodeList                       │ │
│  │  2. 从链下节点发现服务获取各节点 endpoint URL                   │ │
│  │  3. 向所有其他活跃节点发起 WebSocket 连接:                      │ │
│  │     wss://{node_endpoint}/v1/gossip                            │ │
│  │  4. 双向认证: 连接时交换 Ed25519 签名的握手消息                 │ │
│  │     { node_id, timestamp, nonce, signature }                   │ │
│  │  5. 验证对方公钥在链上 ActiveNodeList 中                        │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  连接维护:                                                            │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  • 心跳: 每 30 秒 Ping/Pong                                    │ │
│  │  • 断线重连: 指数退避 (1s, 2s, 4s, 8s, max 60s)                │ │
│  │  • 连接池: 每个节点维护到所有其他节点的连接 HashMap               │ │
│  │    connections: HashMap<NodeId, WebSocketStream>                │ │
│  │  • 连接状态: Connected / Reconnecting / Dead                   │ │
│  │  • Dead 判定: 连续 3 次重连失败 → 标记 Dead → 上报链上          │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  节点加入/退出:                                                       │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  链上事件: NodeRegistered(new_node_id)                          │ │
│  │  → 所有现有节点向新节点发起连接                                  │ │
│  │  → 新节点向所有现有节点发起连接                                  │ │
│  │                                                                │ │
│  │  链上事件: NodeExited(old_node_id)                              │ │
│  │  → 所有节点关闭到该节点的连接                                    │ │
│  │  → 清理连接池                                                  │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 15.3 Gossip 消息类型完整定义

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Gossip 消息类型枚举                                  │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  所有 gossip 消息共享信封:                                             │
│  {                                                                   │
│    "envelope": {                                                     │
│      "version": "1.0",                                              │
│      "msg_type": "MessageSeen | MessagePull | ...",                  │
│      "sender_node_id": "N7",                                        │
│      "timestamp": 1738850402,                                        │
│      "sender_signature": "Ed25519.sign(...)"                         │
│    },                                                                │
│    "payload": { ... }  // 类型特定                                    │
│  }                                                                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 类型 1: MessageSeen — 消息确认声明

```json
{
  "envelope": { "msg_type": "MessageSeen", ... },
  "payload": {
    "msg_id": "SHA256(owner_address + sequence)",
    "msg_hash": "SHA256(telegram_update)",
    "owner_address": "5GrwvaEF...",
    "sequence": 42,
    "bot_id_hash": "a1b2c3d4",
    "seen_at": 1738850402,
    "verification_result": {
      "signature_valid": true,
      "ip_valid": true,
      "auth_valid": true,
      "target_node_valid": true
    }
  }
}

触发: 节点收到群主消息并验证通过后立即广播
目标: 发给本消息的其他 K-1 个目标节点
大小: ~300 bytes
```

#### 类型 2: MessagePull — 消息拉取请求

```json
{
  "envelope": { "msg_type": "MessagePull", ... },
  "payload": {
    "msg_id": "SHA256(owner_address + sequence)",
    "reason": "gossip_heard_not_received",
    "heard_from_nodes": ["N2", "N5"],
    "pull_target": "N2"
  }
}

触发: 节点通过 gossip 得知某消息存在，但自己没有收到原始消息
目标: 发给声称已 seen 的某个节点
大小: ~200 bytes
```

#### 类型 3: MessagePullResponse — 消息拉取响应

```json
{
  "envelope": { "msg_type": "MessagePullResponse", ... },
  "payload": {
    "msg_id": "xxx",
    "original_message": {
      "owner_address": "5GrwvaEF...",
      "bot_id_hash": "a1b2c3d4",
      "sequence": 42,
      "timestamp": 1738850401,
      "message_hash": "SHA256(telegram_update)",
      "telegram_update": { ... },
      "owner_signature": "xxx"
    }
  }
}

触发: 收到 MessagePull 请求
目标: 发给请求方
大小: ~2-10 KB (含完整 Telegram Update)
```

#### 类型 4: DecisionVote — 管理动作投票

```json
{
  "envelope": { "msg_type": "DecisionVote", ... },
  "payload": {
    "msg_id": "xxx",
    "action_id": "SHA256(action_type + params + msg_id)",
    "action_type": "banChatMember",
    "params": { "chat_id": -100123, "user_id": 777 },
    "decision": "AGREE",
    "reason": "spam_link_detected",
    "rule_engine_output": {
      "matched_rules": ["url_blacklist", "newbie_link"],
      "confidence": 0.95
    }
  }
}

触发: 节点规则引擎产出管理动作判定后
目标: 发给本消息的其他 K-1 个目标节点
大小: ~400 bytes
```

#### 类型 5: EquivocationAlert — 双发告警

```json
{
  "envelope": { "msg_type": "EquivocationAlert", ... },
  "payload": {
    "owner_address": "5GrwvaEF...",
    "sequence": 42,
    "evidence": {
      "variant_a": { "msg_hash": "abc...", "signature": "..." },
      "variant_b": { "msg_hash": "def...", "signature": "..." }
    },
    "detected_by": "N7"
  }
}

触发: 节点发现同一 (owner, sequence) 有两个不同的 msg_hash
目标: 广播给所有 N 个节点 (不限于 K 个)
大小: ~500 bytes
```

#### 类型 6: ExecutionResult — Leader 执行结果

```json
{
  "envelope": { "msg_type": "ExecutionResult", ... },
  "payload": {
    "action_id": "xxx",
    "msg_id": "xxx",
    "leader_node_id": "N7",
    "was_takeover": false,
    "result": { "status": "success", "telegram_response": {...} },
    "agent_receipt": { "agent_signature": "..." }
  }
}

触发: Leader 执行管理指令后
目标: 发给本消息的其他 K-1 个目标节点
大小: ~500 bytes
```

#### 类型 7: LeaderTakeover — Backup 接替通知

```json
{
  "envelope": { "msg_type": "LeaderTakeover", ... },
  "payload": {
    "action_id": "xxx",
    "original_leader": "N7",
    "reason": "timeout_3s",
    "new_leader": "N2",
    "backup_rank": 1
  }
}

触发: Backup 节点因 Leader 超时而接替
目标: 发给本消息的其他 K-1 个目标节点
大小: ~250 bytes
```

#### 类型 8: Heartbeat — 心跳 (层 1)

```json
{
  "envelope": { "msg_type": "Heartbeat", ... },
  "payload": {
    "latest_block": 12345,
    "active_gossip_count": 3,
    "uptime_seconds": 86400,
    "load": 0.2
  }
}

触发: 每 30 秒
目标: 所有已连接节点
大小: ~150 bytes
```

### 15.4 Gossip 传播算法

#### 15.4.1 消息确认的完整传播流程

```
┌──────────────────────────────────────────────────────────────────────┐
│                    MessageSeen 传播时序                                │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  场景: K=5, 目标=[N2, N5, N7, N9, N14], 群主发消息 seq=42            │
│                                                                      │
│  时间轴 (毫秒) ──────────────────────────────────────────▶          │
│                                                                      │
│  T=0ms    群主 Agent 并发 POST 到 5 个节点                            │
│                                                                      │
│  T=50ms   N2 收到消息 → 验签 ✅                                      │
│           N5 收到消息 → 验签 ✅                                      │
│           N9 收到消息 → 验签 ✅                                      │
│           N14 收到消息 → 验签 ✅                                     │
│           N7 未收到 (网络延迟)                                        │
│           │                                                          │
│           ▼                                                          │
│  T=60ms   N2 广播 MessageSeen → [N5, N7, N9, N14]                   │
│           N5 广播 MessageSeen → [N2, N7, N9, N14]                   │
│           N9 广播 MessageSeen → [N2, N5, N7, N14]                   │
│           N14 广播 MessageSeen → [N2, N5, N7, N9]                   │
│           │                                                          │
│           ▼                                                          │
│  T=80ms   所有节点互相收到 MessageSeen:                               │
│                                                                      │
│           N2 的 seen 表: {N2:✅, N5:✅, N9:✅, N14:✅} = 4/5         │
│           N5 的 seen 表: {N2:✅, N5:✅, N9:✅, N14:✅} = 4/5         │
│           N9 的 seen 表: {N2:✅, N5:✅, N9:✅, N14:✅} = 4/5         │
│           N14 的 seen 表: {N2:✅, N5:✅, N9:✅, N14:✅} = 4/5        │
│           │                                                          │
│           │  M = ceil(5 × 2/3) = 4                                   │
│           │  seen_count = 4 ≥ M = 4 → 共识达成 ✅                    │
│           │                                                          │
│           ▼                                                          │
│  T=90ms   N7 收到了 4 个 MessageSeen (来自 N2,N5,N9,N14)            │
│           但 N7 自己没有原始消息                                      │
│           │                                                          │
│           ▼                                                          │
│  T=95ms   N7 发送 MessagePull → N2 (请求完整消息)                    │
│           │                                                          │
│           ▼                                                          │
│  T=110ms  N2 回复 MessagePullResponse (含完整消息)                    │
│           N7 验签 ✅ → 广播 MessageSeen → [N2,N5,N9,N14]            │
│           │                                                          │
│           ▼                                                          │
│  T=130ms  所有节点 seen 表: {N2,N5,N7,N9,N14} = 5/5                 │
│           N7 也达成共识                                               │
│                                                                      │
│  总耗时: ~130ms (含一次 Pull 补偿)                                    │
│  无 Pull 时: ~80ms                                                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 15.4.2 节点本地状态机

```
每个节点为每条消息维护的状态:

┌──────────────────────────────────────────────────────────────────────┐
│  struct MessageState {                                               │
│      msg_id: [u8; 32],                                              │
│      status: MessageStatus,                                          │
│                                                                      │
│      // 原始消息 (可能为 None，需 Pull)                               │
│      original_message: Option<SignedMessage>,                        │
│                                                                      │
│      // seen 表: 哪些节点声称看到了这条消息                            │
│      seen_nodes: HashMap<NodeId, SeenRecord>,                        │
│                                                                      │
│      // 管理动作投票表 (如果需要执行管理动作)                           │
│      decision_votes: HashMap<NodeId, DecisionVote>,                  │
│                                                                      │
│      // 目标节点列表 (确定性计算)                                      │
│      target_nodes: Vec<NodeId>,                                      │
│      leader: NodeId,                                                 │
│      backups: Vec<NodeId>,                                           │
│                                                                      │
│      // 时间戳                                                        │
│      first_seen_at: Instant,                                         │
│      consensus_reached_at: Option<Instant>,                          │
│      execution_completed_at: Option<Instant>,                        │
│                                                                      │
│      // 超时计时器                                                    │
│      pull_timer: Option<Timer>,        // 5s 后主动 Pull              │
│      consensus_timer: Option<Timer>,   // 30s 后标记 Timeout         │
│      leader_timer: Option<Timer>,      // 3s 后 Leader failover     │
│  }                                                                   │
│                                                                      │
│  enum MessageStatus {                                                │
│      HeardViaSeen,    // 只收到 gossip，没有原始消息                   │
│      Received,        // 收到原始消息，验签通过                        │
│      Pending,         // 已广播 Seen，等待共识                        │
│      Confirmed,       // M/K 共识达成                                │
│      Executing,       // Leader 正在执行管理指令                      │
│      Completed,       // 执行完成                                    │
│      Timeout,         // 30s 超时未达共识                             │
│      Failed,          // 执行失败                                    │
│  }                                                                   │
│                                                                      │
│  struct SeenRecord {                                                 │
│      node_id: NodeId,                                                │
│      msg_hash: [u8; 32],                                             │
│      seen_at: u64,                                                   │
│      signature: [u8; 64],                                            │
│  }                                                                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 15.4.3 消息处理引擎

```
┌──────────────────────────────────────────────────────────────────────┐
│                    节点消息处理引擎 (伪代码)                            │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  fn on_gossip_message(msg: GossipMessage) {                          │
│      match msg.envelope.msg_type {                                   │
│                                                                      │
│          "MessageSeen" => {                                          │
│              let state = get_or_create_state(msg.payload.msg_id);    │
│                                                                      │
│              // 验证发送者在目标节点列表中                              │
│              if !state.target_nodes.contains(msg.sender) {           │
│                  return; // 拒绝非目标节点的 seen                     │
│              }                                                       │
│                                                                      │
│              // 验证发送者签名                                        │
│              if !verify_node_signature(msg) {                        │
│                  return;                                              │
│              }                                                       │
│                                                                      │
│              // 记录 seen                                             │
│              state.seen_nodes.insert(msg.sender, SeenRecord{...});   │
│                                                                      │
│              // 检查 msg_hash 一致性                                  │
│              if has_conflicting_hashes(state) {                      │
│                  // 发现 equivocation!                                │
│                  broadcast_equivocation_alert(state);                 │
│                  return;                                              │
│              }                                                       │
│                                                                      │
│              // 如果我没有原始消息，启动 Pull 计时器                    │
│              if state.original_message.is_none()                     │
│                  && state.status == HeardViaSeen                      │
│                  && state.pull_timer.is_none() {                     │
│                  state.pull_timer = Some(Timer::new(500ms));         │
│                  // 500ms 后如果 Agent 的消息还没到，主动 Pull         │
│              }                                                       │
│                                                                      │
│              // 检查是否达到共识                                       │
│              check_consensus(state);                                  │
│          }                                                           │
│                                                                      │
│          "MessagePull" => {                                          │
│              let state = get_state(msg.payload.msg_id);              │
│              if let Some(original) = state.original_message {        │
│                  send_to(msg.sender, MessagePullResponse {           │
│                      original_message: original                      │
│                  });                                                  │
│              }                                                       │
│          }                                                           │
│                                                                      │
│          "MessagePullResponse" => {                                  │
│              let state = get_or_create_state(msg.payload.msg_id);    │
│              let original = msg.payload.original_message;            │
│                                                                      │
│              // 独立验证群主签名                                       │
│              if !verify_owner_signature(original) {                  │
│                  return; // 签名无效，可能是恶意节点篡改               │
│              }                                                       │
│                                                                      │
│              state.original_message = Some(original);                │
│              state.status = Received;                                │
│              state.pull_timer = None; // 取消 Pull 计时器             │
│                                                                      │
│              // 广播自己的 MessageSeen                                │
│              broadcast_seen(state);                                   │
│              state.status = Pending;                                 │
│                                                                      │
│              check_consensus(state);                                  │
│          }                                                           │
│                                                                      │
│          "DecisionVote" => {                                         │
│              let state = get_state(msg.payload.msg_id);              │
│              state.decision_votes.insert(msg.sender, msg.payload);   │
│              check_decision_consensus(state);                        │
│          }                                                           │
│                                                                      │
│          "ExecutionResult" => {                                      │
│              let state = get_state(msg.payload.msg_id);              │
│              // 验证 Agent 回执签名                                   │
│              if verify_agent_receipt(msg.payload.agent_receipt) {    │
│                  state.status = Completed;                           │
│                  state.leader_timer = None;                          │
│                  log_completion(state);                               │
│              }                                                       │
│          }                                                           │
│                                                                      │
│          _ => { /* 其他类型类似处理 */ }                               │
│      }                                                               │
│  }                                                                   │
│                                                                      │
│  fn check_consensus(state: &mut MessageState) {                      │
│      let K = state.target_nodes.len();                               │
│      let M = (K * 2 + 2) / 3;  // ceil(K * 2/3)                    │
│      let consistent_count = count_consistent_seen(state);           │
│                                                                      │
│      if consistent_count >= M && state.status == Pending {           │
│          state.status = Confirmed;                                   │
│          state.consensus_reached_at = Some(Instant::now());         │
│                                                                      │
│          // 运行规则引擎，判断是否需要管理动作                          │
│          if let Some(action) = run_rule_engine(state) {              │
│              // 广播 DecisionVote                                    │
│              broadcast_decision_vote(state, action);                 │
│          }                                                           │
│                                                                      │
│          // 如果我是 Leader → 准备执行                                │
│          if state.leader == self.node_id {                           │
│              // 等待 DecisionVote 达到 M/K → 执行                    │
│          } else {                                                    │
│              // 启动 Leader 超时计时器                                │
│              state.leader_timer = Some(Timer::new(3s));              │
│          }                                                           │
│      }                                                               │
│  }                                                                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 15.5 延迟优化

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Gossip 延迟优化策略                                  │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  目标: 从群主消息到达 → 共识达成 < 200ms (无 Pull 场景)               │
│                                                                      │
│  策略 1: 立即广播 (Zero-Wait Seen)                                    │
│  ──────────────────────────                                          │
│  节点收到原始消息并验签后，立即广播 MessageSeen                        │
│  不等待其他节点的 Seen → 最快的节点先发                                │
│  │                                                                   │
│  延迟贡献: ~10ms (验签) + ~20ms (WebSocket 发送)                     │
│                                                                      │
│  策略 2: 并发验证 + 广播                                              │
│  ──────────────────────────                                          │
│  验签和广播可以流水线化:                                               │
│  ① 收到消息 → 开始验签                                               │
│  ② 验签通过 → 同时: 广播 Seen + 运行规则引擎                         │
│  ③ 规则引擎出结果 → 广播 DecisionVote                                │
│  │                                                                   │
│  而不是串行: 验签 → 广播 → 规则引擎 → 广播                            │
│                                                                      │
│  策略 3: 延迟 Pull (Lazy Pull)                                       │
│  ──────────────────────────                                          │
│  收到 MessageSeen 但没有原始消息时:                                    │
│  • 不立即 Pull → 群主的原始消息可能还在路上 (网络延迟)                 │
│  • 等 500ms → 如果还没收到，再 Pull                                  │
│  • 避免不必要的 Pull 流量                                             │
│  │                                                                   │
│  延迟影响: 最差 +500ms (如果消息真的丢了)                              │
│                                                                      │
│  策略 4: Seen 聚合 (Seen Aggregation)                                 │
│  ──────────────────────────                                          │
│  问题: K=10 时，每个节点发 Seen 给其他 9 个 → 90 条消息               │
│  优化: 收到其他节点的 Seen 时，附加到自己的 Seen 中转发:               │
│  │                                                                   │
│  Round 1: N2 发 Seen{seen_by:[N2]}                                   │
│  Round 2: N5 收到 N2 的 Seen，发 Seen{seen_by:[N5, N2]}              │
│  Round 3: N7 收到 N5 的 Seen，发 Seen{seen_by:[N7, N5, N2]}          │
│  │                                                                   │
│  效果: 一条 Seen 消息携带多个节点的确认 → 减少消息数                   │
│  注意: 只聚合确认计数，不替代各节点的独立签名                          │
│                                                                      │
│  策略 5: 消息优先级队列                                                │
│  ──────────────────────────                                          │
│  WebSocket 上的消息按优先级排队:                                       │
│                                                                      │
│  优先级 1 (最高): EquivocationAlert                                  │
│  优先级 2: MessageSeen / DecisionVote                                │
│  优先级 3: ExecutionResult / LeaderTakeover                          │
│  优先级 4: MessagePull / MessagePullResponse                         │
│  优先级 5 (最低): Heartbeat                                          │
│                                                                      │
│  高优先级消息可以中断低优先级消息的发送                                 │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 15.6 带宽分析与控制

```
┌──────────────────────────────────────────────────────────────────────┐
│                    带宽消耗估算                                        │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  场景: N=15 个节点，K=6，群主每秒 10 条消息                            │
│                                                                      │
│  每条消息产生的 gossip 流量 (单节点视角):                               │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ 消息类型          │ 发送数量   │ 单条大小  │ 合计               │ │
│  ├───────────────────┼───────────┼──────────┼───────────────────┤ │
│  │ 收到原始消息       │ 1 (入站)  │ ~2 KB    │ 2 KB (入)          │ │
│  │ 广播 MessageSeen  │ K-1=5 (出)│ 300 B    │ 1.5 KB (出)        │ │
│  │ 收到 MessageSeen  │ K-1=5 (入)│ 300 B    │ 1.5 KB (入)        │ │
│  │ 广播 DecisionVote │ K-1=5 (出)│ 400 B    │ 2 KB (出)          │ │
│  │ 收到 DecisionVote │ K-1=5 (入)│ 400 B    │ 2 KB (入)          │ │
│  │ ExecutionResult   │ 1 (入/出) │ 500 B    │ 0.5 KB             │ │
│  ├───────────────────┼───────────┼──────────┼───────────────────┤ │
│  │ 每条消息合计       │           │          │ ~10 KB             │ │
│  │ 10 msg/s × 10KB  │           │          │ ~100 KB/s          │ │
│  │ + 心跳等开销       │           │          │ ~5 KB/s            │ │
│  ├───────────────────┼───────────┼──────────┼───────────────────┤ │
│  │ 单节点总带宽       │           │          │ ~105 KB/s          │ │
│  │                   │           │          │ ≈ 0.8 Mbps         │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  结论: 带宽需求非常低，任何 VPS 都能满足                                │
│                                                                      │
│  带宽控制策略:                                                        │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                                                                │ │
│  │  ① 消息去重: 相同 msg_id 的 Seen 只处理一次                    │ │
│  │     → 避免 gossip 风暴                                         │ │
│  │                                                                │ │
│  │  ② TTL 限制: 每条 gossip 消息有 TTL=1                          │ │
│  │     节点不转发收到的 Seen (只广播自己的)                         │ │
│  │     → 目标节点间直接通信，不会指数扩散                           │ │
│  │                                                                │ │
│  │  ③ 速率限制: 每节点每秒最多处理 1000 条 gossip                  │ │
│  │     超出丢弃 + 告警                                            │ │
│  │                                                                │ │
│  │  ④ 消息过期: gossip 消息 timestamp > 60s 前 → 丢弃             │ │
│  │     → 防止旧消息反复传播                                       │ │
│  │                                                                │ │
│  │  ⑤ 按消息级别限流:                                              │ │
│  │     原始消息: 不限 (必须处理)                                    │ │
│  │     Seen/Vote: 每消息每节点只接受 1 条                          │ │
│  │     Pull: 每秒每节点最多 10 条                                  │ │
│  │     Heartbeat: 每 30 秒 1 条                                   │ │
│  │                                                                │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 15.7 网络分区与恢复

```
┌──────────────────────────────────────────────────────────────────────┐
│                    网络分区场景分析                                      │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  场景 1: 单节点隔离                                                   │
│  ──────────────────                                                  │
│                                                                      │
│  正常: [N2, N5, N7, N9, N14] 全连通                                  │
│  分区: N7 与所有其他节点断连                                           │
│                                                                      │
│     N2 ◀───▶ N5            N7 (孤立)                                │
│      ▲       ▲                                                      │
│      │       │                                                       │
│      ▼       ▼                                                       │
│     N9 ◀───▶ N14                                                    │
│                                                                      │
│  影响:                                                                │
│  • K=5, M=4: 剩余 4 个节点仍可达成共识 ✅                            │
│  • N7 无法参与 → 不影响系统运行                                       │
│  • N7 的 Leader 任务 → 超时 → Backup 接替                             │
│                                                                      │
│  恢复:                                                                │
│  • N7 恢复连接后，通过 gossip 收到缺失消息的 Seen                     │
│  • N7 Pull 缺失消息 → 同步状态                                       │
│  • N7 不需要重新处理已完成的消息                                       │
│                                                                      │
│                                                                      │
│  场景 2: 双分区 (脑裂)                                                │
│  ──────────────────                                                  │
│                                                                      │
│  K=5, M=4                                                            │
│  分区 A: [N2, N5]          分区 B: [N7, N9, N14]                     │
│                                                                      │
│  影响:                                                                │
│  • 分区 A: 2 个节点 < M=4 → 无法达成共识 ❌                          │
│  • 分区 B: 3 个节点 < M=4 → 无法达成共识 ❌                          │
│  • 两边都无法达成共识 → 消息进入 TIMEOUT 状态                         │
│  • 这是正确的行为: 不确定状态时不执行动作 (安全优先)                   │
│                                                                      │
│  恢复:                                                                │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  分区恢复后:                                                  │   │
│  │                                                              │   │
│  │  T₀: 网络恢复，节点重新连接                                   │   │
│  │       │                                                      │   │
│  │       ▼                                                      │   │
│  │  T₁: 节点交换 Heartbeat → 发现分区恢复                        │   │
│  │       │                                                      │   │
│  │       ▼                                                      │   │
│  │  T₂: 对每条 TIMEOUT 状态的消息:                                │   │
│  │       ① 交换 MessageSeen → 合并两边的 seen 表                 │   │
│  │       ② 如果合并后 seen_count ≥ M → 延迟确认                  │   │
│  │       ③ 如果消息已过期 (>60s) → 标记 EXPIRED，不再处理        │   │
│  │       │                                                      │   │
│  │       ▼                                                      │   │
│  │  T₃: 延迟确认的消息:                                          │   │
│  │       • 时效性要求低的 (删消息/踢人) → 仍然执行                │   │
│  │       • 时效性要求高的 (CAPTCHA 验证) → 可能已过期，跳过       │   │
│  │       • 由节点规则引擎判断是否仍有意义执行                      │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│                                                                      │
│  场景 3: 群主 Agent 到部分节点不通                                     │
│  ──────────────────                                                  │
│                                                                      │
│  群主 Agent → N2 ✅ → N5 ❌ → N7 ✅ → N9 ❌ → N14 ✅                │
│  (3/5 收到原始消息)                                                   │
│                                                                      │
│  处理:                                                                │
│  • N2, N7, N14 广播 Seen                                             │
│  • N5, N9 收到 Seen 但没有原始消息 → Pull                             │
│  • Pull 成功 → 5/5 节点都有消息 → 正常共识                            │
│  • 这正是 gossip + Pull 机制的价值                                    │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 15.8 安全性分析

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Gossip 层安全分析                                    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  攻击 1: 伪造 MessageSeen                                             │
│  ───────────────────────                                             │
│  恶意节点 N9 伪造 "N2 说看到了消息 X"                                  │
│  防御: 每条 Seen 都有发送者的 Ed25519 签名                             │
│        N9 没有 N2 的私钥 → 无法伪造 N2 的 Seen ❌                     │
│                                                                      │
│  攻击 2: Gossip 洪泛 (Flood)                                         │
│  ───────────────────────                                             │
│  恶意节点发送大量无效 gossip 消息                                      │
│  防御:                                                                │
│  ① 速率限制: 每节点每秒 1000 条上限                                   │
│  ② 签名验证前置: 无效签名立即丢弃 → 零计算开销                        │
│  ③ msg_id 去重: 已处理的 msg_id 不重复处理                            │
│  ④ 持续违规 → 断开连接 → 上报链上                                    │
│                                                                      │
│  攻击 3: 选择性不转发 (Censorship)                                    │
│  ───────────────────────                                             │
│  恶意节点收到消息但不广播 Seen                                         │
│  防御:                                                                │
│  ① 群主 Agent 直接发给 K 个节点 → 恶意节点只能审查自己                 │
│  ② 其他 K-1 个诚实节点的 gossip 不受影响                              │
│  ③ 只要诚实节点 ≥ M → 共识仍然达成                                   │
│  ④ 恶意节点的 "沉默" 会被其他节点发现:                                │
│     其他节点有 Seen 但恶意节点没有 → 可统计举报                        │
│                                                                      │
│  攻击 4: 延迟注入 (Delay Attack)                                     │
│  ───────────────────────                                             │
│  恶意节点故意延迟广播 Seen (比如延迟 10 秒)                            │
│  防御:                                                                │
│  ① timestamp 验证: Seen.seen_at 与当前时间差 > 5s → 标记延迟          │
│  ② 不影响共识: 其他 K-1 个节点的 Seen 不受影响                        │
│  ③ 持续延迟 → 信誉扣分                                               │
│                                                                      │
│  攻击 5: Pull 响应投毒                                                │
│  ───────────────────────                                             │
│  恶意节点在 PullResponse 中返回篡改的消息                              │
│  防御:                                                                │
│  ① Pull 方收到消息后独立验证群主 Ed25519 签名                         │
│  ② 签名不匹配 → 丢弃 → 向其他节点重新 Pull                           │
│  ③ 恶意节点被标记 → 以后不再向其 Pull                                 │
│                                                                      │
│  攻击 6: Sybil 攻击 (伪造大量节点)                                    │
│  ───────────────────────                                             │
│  攻击者注册大量节点，试图控制 >1/3 节点                                │
│  防御:                                                                │
│  ① 节点注册需质押 → 经济门槛                                         │
│  ② 同一 operator 最多 N/3 个节点 (链上强制)                           │
│  ③ 新节点有试用期 (Probation) → 信誉需积累                            │
│  ④ 确定性随机选择 → 攻击者不能保证自己的节点被选中                     │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 15.9 Gossip 实现技术选型

```
┌──────────────────────────────────────────────────────────────────────┐
│                    实现技术选型                                        │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────┬──────────────────┬──────────────────────────┐  │
│  │ 组件             │ 技术选型          │ 理由                      │  │
│  ├─────────────────┼──────────────────┼──────────────────────────┤  │
│  │ 传输层           │ WebSocket (TLS)  │ 全双工、持久连接、低开销  │  │
│  │ 序列化           │ bincode / CBOR   │ 比 JSON 小 50%+，更快    │  │
│  │ 签名             │ Ed25519          │ 与链上一致，快速验签      │  │
│  │ 哈希             │ BLAKE2b-256      │ 比 SHA256 快，Substrate  │  │
│  │                 │                  │ 生态原生支持              │  │
│  │ 异步运行时       │ tokio            │ Rust 异步标准             │  │
│  │ 连接管理         │ tokio-tungstenite│ 异步 WebSocket            │  │
│  │ 状态存储         │ 内存 HashMap      │ 低延迟，消息生命周期短   │  │
│  │ 持久化           │ RocksDB (可选)    │ 异常重启后恢复未完成消息 │  │
│  │ 指标监控         │ Prometheus       │ gossip 延迟/吞吐量指标   │  │
│  └─────────────────┴──────────────────┴──────────────────────────┘  │
│                                                                      │
│  核心数据结构:                                                        │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                                                                │ │
│  │  struct GossipEngine {                                         │ │
│  │      node_id: NodeId,                                          │ │
│  │      signing_key: Ed25519PrivateKey,                           │ │
│  │                                                                │ │
│  │      // 连接池                                                  │ │
│  │      connections: RwLock<HashMap<NodeId, WsConnection>>,       │ │
│  │                                                                │ │
│  │      // 消息状态表 (msg_id → MessageState)                      │ │
│  │      messages: RwLock<HashMap<[u8;32], MessageState>>,          │ │
│  │                                                                │ │
│  │      // 链上数据缓存                                            │ │
│  │      chain_cache: ChainCache,   // 节点列表、公钥、规则         │ │
│  │                                                                │ │
│  │      // 规则引擎                                                │ │
│  │      rule_engine: RuleEngine,                                  │ │
│  │                                                                │ │
│  │      // 指标收集                                                │ │
│  │      metrics: GossipMetrics,                                   │ │
│  │  }                                                             │ │
│  │                                                                │ │
│  │  impl GossipEngine {                                           │ │
│  │      async fn start(&self) {                                   │ │
│  │          tokio::join!(                                         │ │
│  │              self.accept_incoming_connections(),                │ │
│  │              self.connect_to_peers(),                           │ │
│  │              self.process_incoming_messages(),                  │ │
│  │              self.run_timers(),          // 超时检测             │ │
│  │              self.subscribe_chain_events(), // 链上事件          │ │
│  │              self.gc_expired_messages(), // 清理过期消息         │ │
│  │          );                                                    │ │
│  │      }                                                         │ │
│  │  }                                                             │ │
│  │                                                                │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 15.10 Gossip 监控指标

```
┌──────────────────────────────────────────────────────────────────────┐
│                    关键运维指标                                        │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────┬──────────────┬───────────────────┐ │
│  │ 指标名                      │ 正常范围      │ 告警阈值           │ │
│  ├─────────────────────────────┼──────────────┼───────────────────┤ │
│  │ gossip_seen_latency_ms      │ 20-100ms     │ > 500ms           │ │
│  │ gossip_consensus_latency_ms │ 50-200ms     │ > 1000ms          │ │
│  │ gossip_pull_rate             │ < 5%         │ > 20%             │ │
│  │ gossip_pull_success_rate     │ > 99%        │ < 90%             │ │
│  │ gossip_connected_peers       │ N-1          │ < N×0.8           │ │
│  │ gossip_messages_per_sec      │ 10-100       │ > 1000            │ │
│  │ gossip_bandwidth_kbps        │ 50-200       │ > 1000            │ │
│  │ gossip_consensus_timeout_rate│ < 1%         │ > 5%              │ │
│  │ gossip_equivocation_alerts   │ 0            │ > 0 (即时告警)    │ │
│  │ gossip_decision_disagreements│ < 2%         │ > 10%             │ │
│  └─────────────────────────────┴──────────────┴───────────────────┘ │
│                                                                      │
│  gossip_pull_rate 含义:                                               │
│  "有多少比例的消息需要通过 Pull 获取而非直接从 Agent 收到"              │
│  高 pull_rate → Agent 到节点的网络不稳定 或 节点处理能力不足           │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 15.11 Gossip 协议总结

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Gossip 设计要点总结                                  │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────┬────────────────────────────────────────┐   │
│  │ 维度                │ 设计                                    │   │
│  ├─────────────────────┼────────────────────────────────────────┤   │
│  │ 网络拓扑            │ 两层: 全局 mesh + 消息级 K 节点范围     │   │
│  │ 传输协议            │ WebSocket (TLS) + bincode 序列化        │   │
│  │ 消息类型            │ 8 种: Seen/Pull/PullResp/Vote/Result/  │   │
│  │                     │ Takeover/Equivocation/Heartbeat         │   │
│  │ 传播范围            │ 严格限制在 K 个目标节点内 (非全网广播)  │   │
│  │ 共识算法            │ 收集 M-of-K 个一致的 Seen → CONFIRMED  │   │
│  │ 缺失补偿            │ Lazy Pull (500ms 延迟后主动拉取)        │   │
│  │ 延迟目标            │ 无 Pull: ~80ms, 有 Pull: ~130ms        │   │
│  │ 带宽消耗            │ ~100 KB/s (10 msg/s, K=6)              │   │
│  │ 分区容忍            │ 单节点隔离无影响; 脑裂时安全暂停        │   │
│  │ 安全验证            │ 每条 gossip 独立签名 + 去重 + 速率限制  │   │
│  │ 状态管理            │ 内存 HashMap, 60s 过期自动清理           │   │
│  └─────────────────────┴────────────────────────────────────────┘   │
│                                                                      │
│  关键设计决策:                                                        │
│  1. 消息级范围限制 → 带宽 O(K²) 而非 O(N²)                           │
│  2. Lazy Pull → 避免不必要的拉取流量                                  │
│  3. 签名信封 → 每条 gossip 可独立验证真实性                           │
│  4. Agent 回执签名 → ExecutionResult 不可伪造                         │
│  5. 优先级队列 → 关键消息不被低优先级阻塞                             │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 16. 全局流程延时核算

### 16.1 端到端延时总览

```
┌──────────────────────────────────────────────────────────────────────┐
│           完整延时链路 (用户发消息 → 管理动作生效)                       │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  阶段          组件                     延时          累计            │
│  ─────────────────────────────────────────────────────────────────  │
│                                                                      │
│  ① TG 内部    用户发消息                                              │
│       │        → Telegram 服务器处理    ~50-200ms                    │
│       │        → Webhook POST 到 Agent                               │
│       ▼                                 ──────────                   │
│                                         ~100ms        T=100ms        │
│                                                                      │
│  ② Agent      收到 Webhook                                           │
│       │        → 验证 Telegram 来源     ~5ms                         │
│       │        → 构造签名消息           ~2ms                         │
│       │        → Ed25519 签名           ~1ms                         │
│       │        → 查链上节点列表(缓存)   ~0ms                         │
│       │        → 确定性随机选 K 节点    ~0ms                         │
│       ▼                                 ──────────                   │
│                                         ~10ms         T=110ms        │
│                                                                      │
│  ③ 网络传输   Agent → K 个节点                                       │
│       │        (并发 HTTPS POST)                                     │
│       │        Agent 在 VPS (如美国)                                 │
│       │        节点分布全球                                           │
│       ▼                                 ──────────                   │
│                                         ~30-80ms      T=160ms        │
│                                                                      │
│  ④ 节点验证   收到消息                                                │
│       │        → Ed25519 验签           ~1ms                         │
│       │        → IP 白名单校验          ~0ms                         │
│       │        → 链上授权验证(缓存)     ~1ms                         │
│       │        → 消息哈希计算           ~0ms                         │
│       ▼                                 ──────────                   │
│                                         ~5ms          T=165ms        │
│                                                                      │
│  ⑤ Gossip     广播 MessageSeen                                       │
│       │        → WebSocket 发送到 K-1   ~5ms                         │
│       │        → 对方节点收到+验签      ~15ms                        │
│       │        → 收集 M-of-K Seen       ~10ms                        │
│       ▼                                 ──────────                   │
│                                         ~30ms         T=195ms        │
│                                                                      │
│  ⑥ 规则引擎   消息共识确认后                                          │
│       │        → 运行反垃圾规则         ~5ms                         │
│       │        → 关键词/黑名单匹配      ~1ms                         │
│       │        → AI 分类 (如果启用)     ~50-200ms                    │
│       │        → 链上查询 (如果需要)    ~10ms                        │
│       ▼                                 ──────────                   │
│                                         ~10ms (无AI)  T=205ms        │
│                                         ~200ms (有AI) T=395ms        │
│                                                                      │
│  ⑦ 决策共识   广播 DecisionVote                                      │
│       │        → WebSocket 发送         ~5ms                         │
│       │        → 收集 M-of-K Vote       ~20ms                        │
│       ▼                                 ──────────                   │
│                                         ~30ms         T=235ms (无AI) │
│                                                                      │
│  ⑧ Leader     计算 Leader (零开销)                                    │
│    执行        → 构造管理指令           ~2ms                         │
│       │        → POST 到 Agent          ~30-80ms (网络)              │
│       │        → Agent 验证签名         ~5ms                         │
│       │        → Agent 调用 TG API      ~100-300ms                   │
│       ▼                                 ──────────                   │
│                                         ~200ms        T=435ms (无AI) │
│                                                                      │
│  ⑨ 结果       Leader 广播 Result                                     │
│    广播        → 节点记录日志           ~5ms                         │
│       │        → 批量上链 (异步)        ~6s (出块时间)                │
│       ▼                                 ──────────                   │
│                                         ~10ms         T=445ms (无AI) │
│                                         (链上异步)                    │
│                                                                      │
│  ═══════════════════════════════════════════════════════════════════  │
│  端到端总延时:                                                        │
│                                                                      │
│  最佳情况 (纯消息确认, 无管理动作):      ~200ms                       │
│  典型情况 (简单规则 + 管理动作):         ~450ms                       │
│  含 AI 分析:                             ~650ms                       │
│  含 1 次 Pull 补偿:                      +500ms                      │
│  含 1 次 Leader failover:                +3000ms                     │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 16.2 各阶段延时详细拆解

#### 阶段 ①: Telegram → Agent (Webhook)

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Telegram Webhook 延时分析                           │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  路径: 用户手机 → Telegram 服务器 → Webhook POST → Agent VPS         │
│                                                                      │
│  ┌──────────────────────────┬────────────┬──────────────────────┐   │
│  │ 子步骤                   │ 延时        │ 说明                  │   │
│  ├──────────────────────────┼────────────┼──────────────────────┤   │
│  │ 用户 → Telegram 服务器   │ 30-100ms   │ 取决于用户网络质量    │   │
│  │ Telegram 内部处理        │ 10-30ms    │ Bot API 路由           │   │
│  │ Telegram → Agent HTTPS  │ 20-100ms   │ TG 服务器在多地区部署  │   │
│  │ TLS 握手 (首次)          │ 0ms        │ 持久连接，无需重握手   │   │
│  ├──────────────────────────┼────────────┼──────────────────────┤   │
│  │ 合计                     │ 50-200ms   │ 典型 ~100ms            │   │
│  └──────────────────────────┴────────────┴──────────────────────┘   │
│                                                                      │
│  优化:                                                                │
│  • Agent VPS 选择与 Telegram 服务器同区域 (欧洲/美东)                  │
│  • Telegram 使用 HTTP/2 持久连接推送 Webhook                          │
│  • 此阶段延时不可控 (Telegram 侧)                                     │
│                                                                      │
│  Telegram 官方文档:                                                   │
│  "Webhooks are usually delivered within 1-2 seconds"                 │
│  实测: 大多数情况 100-500ms，高峰期可达 1-2s                          │
│                                                                      │
│  ⚠️ 这是整条链路中最不可控的部分                                      │
│  最差情况可达 2000ms (Telegram 服务器拥堵)                             │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 阶段 ②: Agent 本地处理

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Agent 本地处理延时                                   │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌────────────────────────────┬───────────┬─────────────────────┐   │
│  │ 操作                       │ 延时       │ 算法复杂度           │   │
│  ├────────────────────────────┼───────────┼─────────────────────┤   │
│  │ 解析 HTTP body (JSON)      │ ~1ms      │ O(n), n=消息大小     │   │
│  │ 验证 Telegram secret_token │ ~0.1ms    │ O(1) 字符串比较       │   │
│  │ 计算 message_hash (SHA256) │ ~0.1ms    │ O(n)                 │   │
│  │ 序列号递增 + 构造信封       │ ~0.1ms    │ O(1)                 │   │
│  │ Ed25519 签名               │ ~0.5ms    │ O(1), 固定大小       │   │
│  │ 查缓存: 活跃节点列表       │ ~0.1ms    │ O(1) 内存读取         │   │
│  │ 确定性随机选 K 个节点       │ ~0.1ms    │ O(K)                 │   │
│  │ 序列化签名消息 (bincode)    │ ~0.5ms    │ O(n)                 │   │
│  ├────────────────────────────┼───────────┼─────────────────────┤   │
│  │ Agent 处理合计              │ ~3-5ms    │                      │   │
│  └────────────────────────────┴───────────┴─────────────────────┘   │
│                                                                      │
│  瓶颈: 无                                                            │
│  Agent 处理全在内存中，CPU 密集操作只有 Ed25519 签名 (~0.5ms)         │
│  单核即可处理 >1000 msg/s                                             │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 阶段 ③: Agent → 节点 网络传输

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Agent → 节点 网络延时                                │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Agent 并发发送到 K 个节点 (等最慢的到达):                              │
│                                                                      │
│  ┌───────────────────────────────┬──────────────────────────────┐   │
│  │ Agent → 节点位置              │ 单程延时 (RTT/2)              │   │
│  ├───────────────────────────────┼──────────────────────────────┤   │
│  │ 同数据中心                    │ ~1ms                          │   │
│  │ 同区域 (如美东→美西)          │ ~20ms                         │   │
│  │ 跨洲 (如美东→欧洲)           │ ~40ms                         │   │
│  │ 跨洲 (如美东→亚洲)           │ ~80ms                         │   │
│  └───────────────────────────────┴──────────────────────────────┘   │
│                                                                      │
│  并发发送的关键: 延时取决于最慢的节点                                  │
│                                                                      │
│  K=5 节点分布:                                                        │
│  N2(美东, 20ms) N5(欧洲, 40ms) N7(亚洲, 80ms)                       │
│  N9(美西, 30ms) N14(欧洲, 45ms)                                      │
│                                                                      │
│  所有节点收到的最早时间: T+20ms (N2)                                  │
│  所有节点收到的最晚时间: T+80ms (N7)                                  │
│  M=4 个节点收到的时间:   T+45ms (第4快的 N14)                         │
│                                                                      │
│  → 对共识来说，延时 = 第 M 快的节点到达时间                           │
│  → 典型 ~40-60ms                                                     │
│                                                                      │
│  附: HTTPS 开销                                                       │
│  ┌────────────────────────────┬─────────────────────────────────┐   │
│  │ TLS 握手 (首次)            │ +1 RTT (~40-80ms)               │   │
│  │ TLS 会话复用               │ +0 RTT (Agent 维护连接池)       │   │
│  │ HTTP/2 多路复用            │ 多条消息共用同一连接             │   │
│  │ 请求体大小 (~2KB)          │ 1 个 TCP 包即可传完              │   │
│  └────────────────────────────┴─────────────────────────────────┘   │
│                                                                      │
│  优化: Agent 维护到每个节点的 HTTP/2 长连接池                         │
│  → TLS 握手只发生一次                                                 │
│  → 后续请求零额外握手开销                                             │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 阶段 ④-⑤: 节点验证 + Gossip 共识

```
┌──────────────────────────────────────────────────────────────────────┐
│                    验证 + Gossip 共识延时                               │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌────────────────────────────┬──────────┬──────────────────────┐   │
│  │ 操作                       │ 延时      │ 说明                  │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ Ed25519 验签               │ ~1ms     │ CPU 操作              │   │
│  │ IP 白名单校验              │ ~0.1ms   │ HashSet 查表           │   │
│  │ 授权检查 (缓存)            │ ~0.5ms   │ 链上数据本地缓存      │   │
│  │ 消息哈希计算               │ ~0.1ms   │ SHA256                │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ 节点验证小计                │ ~2-5ms   │                      │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ 构造 MessageSeen           │ ~0.5ms   │ 签名+序列化           │   │
│  │ WebSocket 发送 (K-1 节点)  │ ~2ms     │ 并发，非阻塞           │   │
│  │ 对方接收+验签              │ ~5-15ms  │ 取决于节点间延迟       │   │
│  │ 收集 M 个 Seen             │ ~10-20ms │ 等第 M 个节点响应     │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ Gossip 共识小计             │ ~20-40ms │                      │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ 阶段 ④+⑤ 合计              │ ~25-45ms │ 典型 ~35ms           │   │
│  └────────────────────────────┴──────────┴──────────────────────┘   │
│                                                                      │
│  Gossip 延时主要受节点间网络延迟影响:                                  │
│                                                                      │
│  节点全在同一区域:     Gossip ~15ms                                   │
│  节点跨 2 个区域:      Gossip ~25ms                                   │
│  节点全球分布:          Gossip ~40ms                                   │
│                                                                      │
│  含 Pull 的情况 (节点未收到原始消息):                                   │
│  +500ms (Lazy Pull 等待) + ~30ms (Pull 往返)                         │
│  → 共识延时: ~570ms                                                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 阶段 ⑥: 规则引擎

```
┌──────────────────────────────────────────────────────────────────────┐
│                    规则引擎延时                                        │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌────────────────────────────┬──────────┬──────────────────────┐   │
│  │ 规则类型                   │ 延时      │ 说明                  │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ 黑名单查表                 │ ~0.1ms   │ HashSet O(1)          │   │
│  │ 关键词匹配                 │ ~0.5ms   │ Aho-Corasick 多模匹配 │   │
│  │ 域名黑名单 (URL 提取+查表) │ ~1ms     │ 正则+HashSet           │   │
│  │ 刷屏频率检测               │ ~0.1ms   │ 滑动窗口计数器         │   │
│  │ 新人时间检查               │ ~0.1ms   │ 比较 join_date         │   │
│  │ 仿冒管理员检测             │ ~1ms     │ 编辑距离计算            │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ 基础规则引擎合计            │ ~3-5ms   │ 绝大多数消息           │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ AI 内容分类 (可选)          │ 50-200ms │ 本地推理或 API 调用    │   │
│  │ 链上查询 (Token 持仓等)     │ 5-20ms   │ 本地轻节点 or RPC     │   │
│  │ 外部 API (URL 信誉查询)     │ 50-500ms │ 第三方服务             │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ 高级规则引擎最差情况        │ ~500ms   │ AI + 链上 + 外部 API  │   │
│  └────────────────────────────┴──────────┴──────────────────────┘   │
│                                                                      │
│  优化: 规则引擎与 Gossip 并行                                         │
│  • Seen 广播后立即运行规则引擎 (不等共识)                              │
│  • 共识达成时规则引擎通常已完成                                        │
│  • 实际增加的延时 = max(gossip, rule_engine) - gossip                 │
│  • 基础规则: max(35ms, 5ms) - 35ms = 0ms (规则更快，不增加延时)      │
│  • AI 规则: max(35ms, 200ms) - 35ms = +165ms                        │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 阶段 ⑦: 决策共识

```
┌──────────────────────────────────────────────────────────────────────┐
│                    DecisionVote 共识延时                                │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  前提: 只有需要管理动作时才有此阶段                                    │
│  纯消息确认 (无管理动作) → 跳过此阶段                                  │
│                                                                      │
│  ┌────────────────────────────┬──────────┬──────────────────────┐   │
│  │ 操作                       │ 延时      │ 说明                  │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ 构造 DecisionVote + 签名   │ ~1ms     │ CPU 操作              │   │
│  │ WebSocket 广播 (K-1)       │ ~2ms     │ 并发                  │   │
│  │ 对方接收 + 验签            │ ~5-15ms  │ 节点间延迟             │   │
│  │ 收集 M 个一致 Vote         │ ~10-20ms │ 等第 M 个              │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ 决策共识合计                │ ~20-40ms │ 典型 ~30ms            │   │
│  └────────────────────────────┴──────────┴──────────────────────┘   │
│                                                                      │
│  优化: 与规则引擎 + MessageSeen 流水线化                               │
│  • 节点收到足够 Seen 的同时规则引擎已出结果                            │
│  • 立即广播 DecisionVote                                              │
│  • 其他节点的 Vote 也几乎同时到达                                     │
│  • 实际决策共识耗时 ≈ Gossip 一轮延时                                 │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 阶段 ⑧: Leader 执行

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Leader 执行延时 (关键路径)                           │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌────────────────────────────┬──────────┬──────────────────────┐   │
│  │ 操作                       │ 延时      │ 说明                  │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ 计算 Leader (确定性)        │ ~0ms     │ sequence % K          │   │
│  │ 构造管理指令 + 签名         │ ~2ms     │ CPU 操作              │   │
│  │ Leader → Agent HTTPS       │ 30-80ms  │ 取决于 Leader 位置    │   │
│  │ Agent 验证 Leader 身份      │ ~3ms     │ 计算+比对              │   │
│  │ Agent 验证 M/K 共识签名     │ ~5ms     │ M 次 Ed25519 verify   │   │
│  │ Agent 验证授权范围          │ ~1ms     │ 查配置                 │   │
│  │ Agent 查幂等性表            │ ~0.1ms   │ HashMap lookup         │   │
│  │ Agent 调用 Telegram API     │ 100-300ms│ ⚠️ 最大延时贡献者      │   │
│  │ Agent 构造回执 + 签名       │ ~1ms     │ CPU 操作              │   │
│  │ Agent → Leader 响应         │ 30-80ms  │ HTTP 响应              │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ Leader 执行合计             │ 170-470ms│ 典型 ~250ms           │   │
│  └────────────────────────────┴──────────┴──────────────────────┘   │
│                                                                      │
│  ⚠️ Telegram API 是整条链路中延时最大的单一组件                        │
│                                                                      │
│  Telegram API 延时实测:                                               │
│  ┌──────────────────────────┬───────────────────────────────────┐   │
│  │ API                      │ 典型延时                           │   │
│  ├──────────────────────────┼───────────────────────────────────┤   │
│  │ sendMessage              │ 100-200ms                         │   │
│  │ deleteMessage            │ 80-150ms                          │   │
│  │ banChatMember            │ 100-200ms                         │   │
│  │ restrictChatMember       │ 100-200ms                         │   │
│  │ approveChatJoinRequest   │ 150-300ms                         │   │
│  │ promoteChatMember        │ 100-200ms                         │   │
│  │ API 限流时 (429)         │ 1000-30000ms (需等待重试)         │   │
│  └──────────────────────────┴───────────────────────────────────┘   │
│                                                                      │
│  Failover 额外延时:                                                   │
│  Leader 超时 → +3000ms (每级 failover)                               │
│  最差 K=5: 5 × 3s = +15000ms                                        │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 16.3 场景化延时总表

```
┌──────────────────────────────────────────────────────────────────────┐
│                    各场景端到端延时汇总                                  │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  场景                       │ 阶段组合              │ 总延时   │   │
│  ├─────────────────────────────┼──────────────────────┼─────────┤   │
│  │                                                              │   │
│  │  ━━━ 纯消息确认 (无管理动作) ━━━                              │   │
│  │                                                              │   │
│  │  最佳: 同区域节点,无 Pull    │ ①+②+③+④+⑤           │ ~200ms  │   │
│  │  典型: 跨区域节点,无 Pull    │ ①+②+③+④+⑤           │ ~300ms  │   │
│  │  含 Pull: 1 节点需补拉      │ ①+②+③+④+⑤+Pull      │ ~800ms  │   │
│  │                                                              │   │
│  │  ━━━ 简单管理动作 (删消息/禁言) ━━━                           │   │
│  │                                                              │   │
│  │  最佳: 同区域,基础规则       │ ①+②+③+④+⑤+⑥+⑦+⑧    │ ~400ms  │   │
│  │  典型: 跨区域,基础规则       │ ①+②+③+④+⑤+⑥+⑦+⑧    │ ~550ms  │   │
│  │  含 AI 分析                 │ ①+②+③+④+⑤+⑥ᴬᴵ+⑦+⑧  │ ~750ms  │   │
│  │  含 1 次 Pull               │ 上述 +Pull            │ +500ms  │   │
│  │  含 1 次 failover           │ 上述 +failover        │ +3000ms │   │
│  │                                                              │   │
│  │  ━━━ 入群审核 (链上查询) ━━━                                  │   │
│  │                                                              │   │
│  │  已绑定钱包,无 Pull          │ ①+②+③+④+⑤+⑥ᶜʰᵃⁱⁿ+⑦+⑧│ ~600ms │   │
│  │  未绑定钱包 (需等用户操作)   │ 上述 + 用户操作时间    │ ~10-60s │   │
│  │                                                              │   │
│  │  ━━━ 新人 CAPTCHA 验证 ━━━                                    │   │
│  │                                                              │   │
│  │  发送验证消息                │ ①+②+③+④+⑤+⑦+⑧        │ ~500ms  │   │
│  │  等待用户点击按钮            │ 用户操作时间            │ 1-120s  │   │
│  │  验证答案 + 解除限制         │ 同"简单管理动作"        │ ~500ms  │   │
│  │  总计 (含用户操作)           │                        │ 2-121s  │   │
│  │                                                              │   │
│  │  ━━━ 异常/降级场景 ━━━                                        │   │
│  │                                                              │   │
│  │  Telegram Webhook 延迟       │ ①(慢) + 正常           │ +2000ms │   │
│  │  网络分区 (K/2 节点不通)     │ 无法达成共识           │ TIMEOUT │   │
│  │  全部 Leader failover       │ K × 3s                 │ +15s    │   │
│  │  Telegram API 限流 (429)    │ 等待重试               │ +1-30s  │   │
│  │                                                              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 16.4 延时瓶颈排名

```
┌──────────────────────────────────────────────────────────────────────┐
│                    延时瓶颈 TOP 5                                      │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  排名  组件                  典型延时     占比     可控性              │
│  ────────────────────────────────────────────────────────────────   │
│  #1    Telegram API 调用     100-300ms    40-55%   ❌ 不可控          │
│  #2    Telegram Webhook      50-200ms     20-30%   ❌ 不可控          │
│  #3    Agent ↔ 节点网络      30-80ms      10-15%   ⚠️ 可通过选址优化  │
│  #4    Gossip 共识            20-40ms      5-10%    ⚠️ 节点分布影响   │
│  #5    节点间决策共识          20-40ms      5-10%    ⚠️ 节点分布影响   │
│  ──    Agent 本地处理         3-5ms        <1%      ✅ 极快           │
│  ──    节点验证               2-5ms        <1%      ✅ 极快           │
│                                                                      │
│  结论:                                                                │
│  • ~60-70% 延时来自 Telegram (Webhook + API)，完全不可控              │
│  • ~20-30% 延时来自网络传输，可通过节点选址优化                        │
│  • <5% 延时来自本地计算 (签名/验证/规则)，性能充裕                     │
│                                                                      │
│  与传统中心化 Bot 对比:                                                │
│  ┌────────────────────────────┬──────────┬──────────────────────┐   │
│  │ 架构                       │ 典型延时  │ 额外开销              │   │
│  ├────────────────────────────┼──────────┼──────────────────────┤   │
│  │ 传统中心化 Bot              │ ~250ms   │ 基准                  │   │
│  │ 方案 D (无管理动作)         │ ~300ms   │ +50ms (+20%)          │   │
│  │ 方案 D (含管理动作)         │ ~550ms   │ +300ms (+120%)        │   │
│  └────────────────────────────┴──────────┴──────────────────────┘   │
│                                                                      │
│  方案 D 的去中心化共识额外开销:                                        │
│  • 纯消息确认: +50ms (~20%) ← 几乎无感知                             │
│  • 含管理动作: +300ms (~120%) ← 可接受，主要来自回传+TG API           │
│  • 用户感知: 从发消息到看到 Bot 回复/动作 < 1 秒                      │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 16.5 延时优化建议

```
┌──────────────────────────────────────────────────────────────────────┐
│                    优化建议 (按收益排序)                                │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  优化 1: Agent VPS 选址 (收益: -50ms)                                │
│  ────────────────────────────────                                    │
│  将 Agent VPS 部署在 Telegram 服务器附近 (欧洲/美东)                   │
│  Webhook 延时: 200ms → 100ms                                        │
│  Agent→TG API: 300ms → 150ms                                        │
│                                                                      │
│  优化 2: 节点集中部署 (收益: -30ms)                                   │
│  ────────────────────────────────                                    │
│  鼓励节点部署在低延迟区域 (不强制)                                     │
│  节点间 Gossip: 40ms → 15ms                                         │
│  Agent→节点: 80ms → 30ms                                            │
│  注意: 过度集中会降低去中心化程度                                      │
│                                                                      │
│  优化 3: HTTP/2 长连接池 (收益: -20ms)                                │
│  ────────────────────────────────                                    │
│  Agent 维护到每个节点的 HTTP/2 持久连接                                │
│  消除 TLS 握手延迟 (首次 ~80ms → 0ms)                                │
│                                                                      │
│  优化 4: 并行流水线 (收益: -30ms)                                     │
│  ────────────────────────────────                                    │
│  验签 → 同时: 广播 Seen + 规则引擎 + 计算 Leader                     │
│  而非串行: 验签 → 广播 → 等共识 → 规则 → 投票 → 执行                  │
│                                                                      │
│  优化 5: 投机执行 (高级, 收益: -200ms, 风险: 需回滚)                  │
│  ────────────────────────────────                                    │
│  对低风险动作 (如 deleteMessage):                                      │
│  Leader 不等 DecisionVote 共识，直接执行                               │
│  同时并行收集 Vote                                                    │
│  如果 Vote 结果与执行不一致 → 回滚 (对幂等 API 无需回滚)              │
│  仅适用于天然幂等的 API (ban/restrict/delete)                         │
│  ⚠️ 牺牲安全性换取延迟，需谨慎                                        │
│                                                                      │
│  优化后理论最佳延时:                                                   │
│  ┌──────────────────────────────┬────────────────────────────────┐  │
│  │ 场景                         │ 优化后延时                      │  │
│  ├──────────────────────────────┼────────────────────────────────┤  │
│  │ 纯消息确认                   │ ~150ms                         │  │
│  │ 简单管理动作                  │ ~350ms                         │  │
│  │ 含 AI                        │ ~550ms                         │  │
│  └──────────────────────────────┴────────────────────────────────┘  │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 16.6 延时对用户体验的影响

```
┌──────────────────────────────────────────────────────────────────────┐
│                    用户体验分级                                        │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┬──────────────┬──────────────────────────────────┐ │
│  │ 延时范围      │ 用户感知      │ 适用场景                         │ │
│  ├──────────────┼──────────────┼──────────────────────────────────┤ │
│  │ < 200ms      │ 即时          │ 删消息、发确认回复               │ │
│  │ 200-500ms    │ 快速响应      │ 踢人、禁言、CAPTCHA 发送        │ │
│  │ 500ms-1s     │ 正常          │ 入群审核、权限变更               │ │
│  │ 1-3s         │ 略有等待      │ 含 AI 分析的复杂判断             │ │
│  │ 3-10s        │ 明显等待      │ Leader failover 场景            │ │
│  │ > 10s        │ 超时感        │ 多次 failover / 网络问题         │ │
│  └──────────────┴──────────────┴──────────────────────────────────┘ │
│                                                                      │
│  方案 D 的典型延时 (~550ms) 属于 "快速响应" 级别                       │
│  用户在群里发垃圾消息 → 半秒内被删除并收到警告                         │
│  → 体验与传统中心化 Bot 几乎一致                                      │
│                                                                      │
│  唯一明显差异: 回传执行环节 (+200ms)                                   │
│  传统 Bot: Webhook → 处理 → 直接调 API                                │
│  方案 D:   Webhook → Agent → 节点共识 → 回传 Agent → 调 API          │
│  多了 "节点共识 + 回传" 两步，但换来的是去中心化信任                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 17. 多平台扩展分析 — 不仅仅是 Telegram

### 17.1 核心洞察：方案 D 的平台无关性

```
方案 D 数据流中，"Telegram" 只出现在两个端点:

  ① 入口: Telegram Webhook → Agent
  ② 出口: Agent → Telegram API

中间核心链路完全平台无关:
  签名 → 多播 → Gossip → 共识 → 规则 → 决策 → Leader → 回传

  ┌─────────┐     ┌──────────────────────────────┐     ┌─────────┐
  │ 平台     │     │      平台无关核心             │     │ 平台    │
  │ 适配器   │ ──▶ │  签名→多播→Gossip→共识→Leader │ ──▶ │ 适配器  │
  │ (入口)   │     │  (节点网络完全复用)           │     │ (出口)  │
  └─────────┘     └──────────────────────────────┘     └─────────┘
  TG/DC/Slack      不关心事件来源                       TG/DC/Slack

结论: 方案 D 天然支持多平台，只需增加平台适配器
```

### 17.2 主流平台 Bot 能力对比

| 能力 | Telegram | Discord | Slack | 微信(企业) | Matrix |
|------|----------|---------|-------|-----------|--------|
| **事件推送** | Webhook (HTTP) | WebSocket Gateway | Events API (HTTP) | 回调URL | HTTP/WebSocket |
| **消息收发** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **踢人/封禁** | ✅ ban/kick | ✅ ban/kick/timeout | ⚠️ 有限 | ❌ | ✅ |
| **消息删除** | ✅ | ✅ | ✅ | ❌ | ✅ |
| **权限管理** | ✅ promote/restrict | ✅ 角色+频道覆盖 | ⚠️ 有限 | ❌ | ✅ |
| **入群审核** | ✅ approve/decline | ✅ 验证级别 | ✅ 邀请审批 | ⚠️ | ✅ |
| **按钮交互** | ✅ Inline Keyboard | ✅ Components | ✅ Block Kit | ⚠️ 卡片 | ✅ |
| **频道/话题** | ✅ Forum Topics | ✅ Channel/Thread | ✅ Channel/Thread | ❌ | ✅ Space |
| **Slash 命令** | ✅ | ✅ | ✅ | ❌ | ✅ |
| **速率限制** | 30 msg/s | 50 req/s | Tier-based | 严格 | 宽松 |
| **适配难度** | ★☆☆☆☆ 已完成 | ★★☆☆☆ 容易 | ★★★☆☆ 中等 | ★★★★★ 困难 | ★★☆☆☆ 容易 |

### 17.3 Discord 适配深度分析

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Discord 适配要点                                    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  差异 1: 事件推送方式                                                 │
│  Telegram: Webhook (被动接收 HTTP POST)                              │
│  Discord:  WebSocket Gateway (主动连接, 持久双向通道)                 │
│  → Agent 需维护 wss://gateway.discord.gg 长连接                     │
│  → 需处理 heartbeat, reconnect, resume 生命周期                      │
│                                                                      │
│  差异 2: 群组模型                                                     │
│  Telegram: 群组 (扁平)                                               │
│  Discord:  Guild → Channel → Thread (层级)                           │
│  映射: TG 群组 ↔ DC Guild, TG 话题 ↔ DC Channel                     │
│                                                                      │
│  差异 3: 权限模型                                                     │
│  Telegram: 扁平权限 (can_send_messages, can_pin 等)                  │
│  Discord:  角色层级 + 频道覆盖 (53位 BitFlags, 更精细)                │
│  → 方案 D 动态角色系统天然适配 Discord Role 分配                     │
│                                                                      │
│  管理 API 对照:                                                       │
│  ┌──────────────────┬────────────────────┬──────────────────────┐   │
│  │ 功能              │ Telegram            │ Discord              │   │
│  ├──────────────────┼────────────────────┼──────────────────────┤   │
│  │ 踢人             │ banChatMember       │ PUT /guilds/bans/{}  │   │
│  │ 禁言             │ restrictChatMember  │ PATCH members/ +     │   │
│  │                  │                    │ timeout              │   │
│  │ 删消息           │ deleteMessage       │ DELETE /messages/{}  │   │
│  │ 分配角色         │ promoteChatMember   │ PUT /members/roles/  │   │
│  │ 发消息           │ sendMessage         │ POST /messages       │   │
│  │ 创建频道         │ createForumTopic    │ POST /channels       │   │
│  └──────────────────┴────────────────────┴──────────────────────┘   │
│                                                                      │
│  结论: Discord 管理能力与 TG 完全对等甚至更强                         │
│  适配工作量: ~1-2 周                                                  │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 17.4 多平台统一架构

```
┌──────────────────────────────────────────────────────────────────────┐
│                Local Agent (群主 VPS) — 多平台架构                     │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌────────────────────── 平台适配器层 ──────────────────────────┐   │
│  │ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐          │   │
│  │ │ TG Adapter   │ │ DC Adapter   │ │ Slack Adapter│  ...     │   │
│  │ │ Webhook接收  │ │ Gateway WS   │ │ Events API   │          │   │
│  │ │ TG API执行   │ │ Discord API  │ │ Slack API    │          │   │
│  │ └──────┬───────┘ └──────┬───────┘ └──────┬───────┘          │   │
│  └────────┼────────────────┼────────────────┼──────────────────┘   │
│           ▼                ▼                ▼                      │
│  ┌──────────────────────── 统一事件层 ─────────────────────────┐   │
│  │ UnifiedEvent {                                              │   │
│  │   platform: "telegram" | "discord" | "slack",               │   │
│  │   event_type: MessageCreated | MemberJoined | ...,          │   │
│  │   community_id, sender, content, raw_event                  │   │
│  │ }                                                           │   │
│  └──────────────────────────┬──────────────────────────────────┘   │
│                             ▼                                      │
│  ┌──────────────────── 核心处理层 (不变) ──────────────────────┐   │
│  │ Ed25519 签名 → 确定性随机选 K 节点 → 多播                   │   │
│  └──────────────────────────┬──────────────────────────────────┘   │
│                             ▼ 节点共识后回传                       │
│  ┌──────────────────── 统一指令层 ────────────────────────────┐   │
│  │ UnifiedAction { platform, action_type, target_user, ... }   │   │
│  │ → 路由到对应平台适配器执行                                   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  节点网络: 完全复用，零修改                                           │
│  节点收到的 SignedMessage 多一个 platform 字段                        │
│  规则引擎按 platform 加载不同规则集                                   │
│  Gossip / 共识 / Leader 完全不变                                     │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 17.5 统一数据模型

```rust
/// 平台类型
enum Platform { Telegram, Discord, Slack, Matrix, Farcaster }

/// 统一事件类型
enum UnifiedEventType {
    // 消息类
    MessageCreated, MessageEdited, MessageDeleted,
    // 成员类
    MemberJoined, MemberLeft, MemberBanned,
    MemberRoleChanged, JoinRequestCreated,
    // 交互类
    ButtonClicked, CommandInvoked,
    ReactionAdded, ReactionRemoved,
    // 频道/话题类
    ChannelCreated, ChannelDeleted, ChannelUpdated,
    // 投票/表单类
    PollVoted, FormSubmitted,
}

/// 统一用户标识
struct UnifiedUser {
    platform: Platform,
    platform_user_id: String,    // "777"(TG) / "123456789"(DC)
    display_name: Option<String>,
    username: Option<String>,
    is_bot: bool,
    chain_address: Option<AccountId>,  // 链上绑定
}

/// 统一社区标识
struct UnifiedCommunity {
    platform: Platform,
    platform_community_id: String,  // "-100xxx"(TG) / "guild_id"(DC)
    chain_community_id: Option<[u8; 16]>,  // 链上社区ID
}

/// 统一管理动作
enum UnifiedAction {
    SendMessage { channel_id: String, content: String },
    DeleteMessage { channel_id: String, message_id: String },
    BanUser { user_id: String, reason: String, duration: Option<u64> },
    KickUser { user_id: String, reason: String },
    MuteUser { user_id: String, duration: u64 },
    UnmuteUser { user_id: String },
    AssignRole { user_id: String, role_id: String },
    RemoveRole { user_id: String, role_id: String },
    ApproveJoin { user_id: String },
    DenyJoin { user_id: String },
    CreateChannel { name: String, category: Option<String> },
    ArchiveChannel { channel_id: String },
}

/// 平台适配器 trait
trait PlatformAdapter {
    fn parse_event(raw: &[u8]) -> Result<UnifiedEvent>;
    async fn execute_action(action: &UnifiedAction) -> Result<()>;
    fn platform() -> Platform;
}
```

### 17.6 链上 Pallet 扩展 (多平台)

```rust
// pallet-bot-registry 扩展

/// 平台类型 (链上存储)
#[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
pub enum Platform { Telegram, Discord, Slack, Matrix, Farcaster }

/// Bot 注册 (多平台)
pub struct BotRegistration<T: Config> {
    pub owner: T::AccountId,
    pub platform: Platform,
    pub bot_id_hash: [u8; 16],
    pub community_id_hash: [u8; 16],
    pub owner_public_key: [u8; 32],
    pub registered_at: BlockNumberFor<T>,
    pub status: BotStatus,
}

/// 社区跨平台绑定
pub struct CommunityBinding<T: Config> {
    pub chain_community_id: [u8; 16],
    pub platform: Platform,
    pub platform_community_id_hash: [u8; 16],
    pub bound_at: BlockNumberFor<T>,
}

// 存储
#[pallet::storage]
/// 链上社区 → 多平台绑定
pub type CommunityPlatforms<T> = StorageDoubleMap<
    _, Blake2_128Concat, [u8; 16],   // chain_community_id
    Blake2_128Concat, Platform,       // platform
    CommunityBinding<T>,
>;

#[pallet::storage]
/// 用户多平台身份绑定
pub type UserPlatformBindings<T> = StorageDoubleMap<
    _, Blake2_128Concat, T::AccountId,
    Blake2_128Concat, Platform,
    BoundedVec<u8, ConstU32<64>>,     // platform_user_id
>;

// 调用
#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 注册多平台 Bot
    #[pallet::call_index(10)]
    pub fn register_bot_multi_platform(
        origin: OriginFor<T>,
        platform: Platform,
        bot_id_hash: [u8; 16],
        community_id_hash: [u8; 16],
        owner_public_key: [u8; 32],
    ) -> DispatchResult;

    /// 绑定社区到链上 (一个链上社区可跨多平台)
    #[pallet::call_index(11)]
    pub fn bind_community_platform(
        origin: OriginFor<T>,
        chain_community_id: [u8; 16],
        platform: Platform,
        platform_community_id_hash: [u8; 16],
    ) -> DispatchResult;

    /// 用户绑定多平台身份
    #[pallet::call_index(12)]
    pub fn bind_user_platform(
        origin: OriginFor<T>,
        platform: Platform,
        platform_user_id: BoundedVec<u8, ConstU32<64>>,
        proof_signature: [u8; 64],
    ) -> DispatchResult;
}
```

### 17.7 跨平台联动场景

```
┌──────────────────────────────────────────────────────────────────────┐
│                    跨平台联动 — 方案 D 独特优势                        │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  场景 1: 统一身份 — 一个链上地址管理所有平台                           │
│  ──────────────────────────────                                      │
│  Alice 链上地址: 5Gxyz...                                            │
│  ├── Telegram: user_id = 777                                        │
│  ├── Discord:  user_id = 123456789                                  │
│  └── Slack:    user_id = U0123ABC                                   │
│                                                                      │
│  一次绑定 → 三个平台的群组都自动识别身份                               │
│  链上 Token 持仓变化 → 三个平台的权限同时更新                         │
│                                                                      │
│                                                                      │
│  场景 2: 跨平台同步管理                                               │
│  ──────────────────────────────                                      │
│  Alice 在 TG 群发垃圾消息 → 被 BAN                                   │
│  联动: 节点共识后，同时通知两个平台 Agent:                             │
│  ① TG Agent: banChatMember(alice_tg_id)                             │
│  ② DC Agent: PUT /guilds/{guild}/bans/{alice_dc_id}                 │
│                                                                      │
│  链上: UserPlatformBindings(alice) → 获取所有平台 ID → 同步执行      │
│                                                                      │
│                                                                      │
│  场景 3: 跨平台公告同步                                               │
│  ──────────────────────────────                                      │
│  链上提案通过 → 节点共识 → Leader 回传指令:                           │
│  ① TG Agent: sendMessage(tg_chat, "提案#42 已通过!")                 │
│  ② DC Agent: POST /channels/{dc_channel}/messages                   │
│  ③ Slack Agent: POST chat.postMessage                               │
│                                                                      │
│  所有平台同时收到相同公告，由多节点共识保证一致性                       │
│                                                                      │
│                                                                      │
│  场景 4: 链上资产驱动跨平台角色                                        │
│  ──────────────────────────────                                      │
│  链上条件: balance ≥ 10000 ATOM → 鲸鱼角色                           │
│                                                                      │
│  Telegram: promoteChatMember(alice, 特殊权限)                        │
│  Discord:  PUT roles/{whale_role}/members/{alice}                    │
│  → Alice 在两个平台同时获得/失去鲸鱼角色                              │
│                                                                      │
│                                                                      │
│  场景 5: 跨平台投票聚合                                               │
│  ──────────────────────────────                                      │
│  DAO 投票:                                                           │
│  TG 用户 → callback_query 投票                                      │
│  DC 用户 → Button Component 投票                                     │
│  Slack 用户 → Block Kit 投票                                        │
│                                                                      │
│  所有票数由节点统一计数 (链上地址去重)                                 │
│  → 防止同一用户在多个平台重复投票                                     │
│  → 链上记录最终结果                                                   │
│                                                                      │
│                                                                      │
│  场景 6: 跨平台反欺诈                                                 │
│  ──────────────────────────────                                      │
│  攻击者在 TG 群被 BAN 后:                                            │
│  → 查链上 UserPlatformBindings → 发现同一地址绑定了 DC 账号          │
│  → 自动在 DC 服务器也 BAN → 攻击者无法跨平台逃避                     │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 17.8 可行性分析

```
┌──────────────────────────────────────────────────────────────────────┐
│                    多平台扩展可行性评估                                  │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────┬──────────┬──────────┬──────────────────────┐   │
│  │ 评估维度         │ 评分      │ 可行性   │ 说明                  │   │
│  ├─────────────────┼──────────┼──────────┼──────────────────────┤   │
│  │ 架构可行性       │ ★★★★★   │ 完全可行 │ 方案 D 天然平台无关   │   │
│  │                 │          │          │ 核心层零修改           │   │
│  ├─────────────────┼──────────┼──────────┼──────────────────────┤   │
│  │ Telegram 适配   │ ★★★★★   │ 已完成   │ 当前设计的目标平台     │   │
│  ├─────────────────┼──────────┼──────────┼──────────────────────┤   │
│  │ Discord 适配    │ ★★★★☆   │ 容易     │ API 能力全面对等      │   │
│  │                 │          │          │ WebSocket Gateway     │   │
│  │                 │          │          │ 需额外处理             │   │
│  ├─────────────────┼──────────┼──────────┼──────────────────────┤   │
│  │ Slack 适配      │ ★★★☆☆   │ 中等     │ Bot 权限模型不同      │   │
│  │                 │          │          │ OAuth 需定期刷新      │   │
│  │                 │          │          │ 管理能力有限制         │   │
│  ├─────────────────┼──────────┼──────────┼──────────────────────┤   │
│  │ Matrix/Element   │ ★★★★☆   │ 容易     │ 去中心化协议          │   │
│  │                 │          │          │ 理念高度吻合           │   │
│  ├─────────────────┼──────────┼──────────┼──────────────────────┤   │
│  │ Farcaster       │ ★★★★★   │ 极佳     │ 链上原生社交          │   │
│  │                 │          │          │ 无需 Token 适配       │   │
│  ├─────────────────┼──────────┼──────────┼──────────────────────┤   │
│  │ 微信/企业微信    │ ★★☆☆☆   │ 困难     │ API 限制多            │   │
│  │                 │          │          │ 审核严格               │   │
│  │                 │          │          │ 不支持踢人/封禁        │   │
│  ├─────────────────┼──────────┼──────────┼──────────────────────┤   │
│  │ 开发工作量       │          │          │                      │   │
│  │   每个新平台     │          │          │ ~1-2 周 (适配器)      │   │
│  │   核心层修改     │          │          │ 0 (无需修改)          │   │
│  └─────────────────┴──────────┴──────────┴──────────────────────┘   │
│                                                                      │
│  技术可行性详解:                                                      │
│                                                                      │
│  ✅ Agent 侧: 可插拔适配器模式                                       │
│     每个平台一个独立模块，实现 PlatformAdapter trait                  │
│     新平台 = 新增一个 crate，零耦合                                  │
│                                                                      │
│  ✅ 节点侧: 完全不需要知道具体平台                                    │
│     SignedMessage 中增加 platform 字段即可                            │
│     规则引擎可按 platform 分组加载规则集                               │
│     Gossip/共识/Leader 协议零修改                                    │
│                                                                      │
│  ✅ 链上: 增加 Platform enum 即可                                     │
│     现有 Pallet 存储结构稍作扩展                                      │
│     向后兼容: 不指定 platform 默认 Telegram                           │
│                                                                      │
│  ⚠️ Agent 部署考量:                                                   │
│     单平台: Agent 只需一个进程                                        │
│     多平台: 建议每个平台一个 Adapter 线程/进程                        │
│     同一 VPS 上可运行所有 Adapter → 共享签名密钥                      │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 17.9 合理性分析

```
┌──────────────────────────────────────────────────────────────────────┐
│                    多平台扩展合理性论证                                  │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ✅ 合理性 1: Web3 项目的现实需求                                     │
│  ────────────────────────────────                                    │
│  绝大多数 Web3 / DAO 项目同时运营:                                    │
│  • Telegram 群 (亚洲社区为主)                                        │
│  • Discord 服务器 (全球社区为主)                                      │
│  • 有时还有 Slack (开发者社区)                                        │
│                                                                      │
│  痛点: 每个平台独立管理 Bot → 规则不一致、人力浪费                    │
│  方案 D: 一套规则链上配置 → 所有平台自动同步执行                      │
│                                                                      │
│                                                                      │
│  ✅ 合理性 2: 一个链上身份 = 所有平台通行证                            │
│  ────────────────────────────────                                    │
│  用户只需绑定一次链上地址:                                             │
│  bind_user_platform(Telegram, tg_id, proof)                          │
│  bind_user_platform(Discord, dc_id, proof)                           │
│                                                                      │
│  之后 Token-gated 入群、权限管理、投票去重全部自动化                   │
│  不需要在每个平台单独验证身份                                          │
│                                                                      │
│                                                                      │
│  ✅ 合理性 3: 去中心化信任跨平台共享                                   │
│  ────────────────────────────────                                    │
│  方案 D 的多节点共识不仅保护 Telegram:                                │
│  • Discord 管理动作同样需要防止单节点造假                              │
│  • 一套节点网络、一套共识机制 → 保护所有平台                          │
│  • 经济效率: 节点运营成本只需一份                                      │
│                                                                      │
│                                                                      │
│  ✅ 合理性 4: 商业价值倍增                                            │
│  ────────────────────────────────                                    │
│  • 单平台 TAM: Telegram Bot 市场                                     │
│  • 多平台 TAM: 所有社交平台的社区管理市场                              │
│  • 跨平台联动是独特卖点 (竞品难以复制)                                │
│  • pallet-bot-billing 收费可按平台数量递增                            │
│  • 用户粘性: 管理多平台的用户更难迁移                                  │
│                                                                      │
│                                                                      │
│  ✅ 合理性 5: 架构零额外设计成本                                      │
│  ────────────────────────────────                                    │
│  方案 D 已将 "平台" 和 "核心逻辑" 分离:                               │
│  • Agent 负责平台适配 (可插拔)                                        │
│  • 节点网络处理统一消息 (平台无关)                                    │
│  • 链上 Pallet 存储统一规则 (平台无关)                                │
│  • 增加新平台 = 写一个新 Adapter → 核心代码零修改                     │
│                                                                      │
│                                                                      │
│  ⚠️ 风险与应对:                                                       │
│  ────────────────────────────────                                    │
│                                                                      │
│  风险 1: 各平台 API 变动                                              │
│  → 适配器层隔离变更，不影响核心                                       │
│  → 每个适配器独立维护和版本管理                                       │
│                                                                      │
│  风险 2: 平台间能力差异                                               │
│  → 统一动作可降级: Slack 不支持 Ban? → 跳过 + 日志记录                │
│  → UnifiedAction 返回 Unsupported → 链上记录 "该平台不支持此动作"    │
│                                                                      │
│  风险 3: 跨平台身份验证成本                                           │
│  → 绑定需要证明拥有平台账号 (如: 在平台发送特定 code)                 │
│  → 链上验证签名即可，一次绑定永久有效                                 │
│                                                                      │
│  风险 4: Agent 复杂度增加                                             │
│  → 每个 Adapter 独立进程，互不干扰                                    │
│  → 某平台 Adapter 崩溃不影响其他平台                                  │
│  → Docker Compose 部署: 一个容器一个 Adapter                         │
│                                                                      │
│  风险 5: 规则引擎跨平台一致性                                         │
│  → 规则分两层: 通用规则 (所有平台) + 平台专属规则                     │
│  → 通用规则保证一致性，专属规则处理差异                                │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 17.10 多平台实施路线

```
┌──────────────────────────────────────────────────────────────────────┐
│                    多平台扩展实施路线                                    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Phase 1 (当前): Telegram 单平台                                     │
│  ──────────────────────────────                                      │
│  • 完成核心架构 (Agent + 节点网络 + 链上 Pallet)                      │
│  • Telegram 适配器作为参考实现                                        │
│  • 验证方案 D 的可行性和性能                                          │
│                                                                      │
│  Phase 2 (+2月): 引入 PlatformAdapter 抽象                           │
│  ──────────────────────────────                                      │
│  • 将现有 Telegram 代码重构为 PlatformAdapter trait 实现              │
│  • 定义 UnifiedEvent / UnifiedAction 统一数据模型                    │
│  • 链上 Pallet 增加 Platform enum                                    │
│  • SignedMessage 增加 platform 字段                                  │
│                                                                      │
│  Phase 3 (+3月): Discord 适配器                                      │
│  ──────────────────────────────                                      │
│  • 开发 DiscordAdapter (Gateway WebSocket + Discord REST API)        │
│  • Discord 特定规则集                                                 │
│  • 跨平台身份绑定 DApp                                               │
│  • 测试: TG + DC 双平台同步管理                                      │
│                                                                      │
│  Phase 4 (+4月): 跨平台联动                                          │
│  ──────────────────────────────                                      │
│  • 跨平台 BAN 同步                                                   │
│  • 跨平台公告同步                                                    │
│  • 跨平台投票聚合                                                    │
│  • 链上资产驱动的跨平台角色                                           │
│                                                                      │
│  Phase 5 (后续): 更多平台                                             │
│  ──────────────────────────────                                      │
│  • Matrix/Element (去中心化社交，理念吻合)                             │
│  • Farcaster (链上原生，最佳契合)                                     │
│  • Slack (企业场景)                                                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 17.11 多平台扩展总结

```
┌──────────────────────────────────────────────────────────────────────┐
│                    总结                                                │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  可行性: ★★★★★                                                       │
│  方案 D 架构天然平台无关，核心层零修改即可扩展到任意平台               │
│  每个新平台仅需 1-2 周开发 Adapter                                    │
│                                                                      │
│  合理性: ★★★★★                                                       │
│  Web3 项目多平台运营是现实需求                                        │
│  统一身份 + 跨平台联动是独特竞争优势                                  │
│  节点网络成本复用 → 边际成本趋零                                      │
│                                                                      │
│  优先级建议:                                                          │
│  ┌────────────────┬──────────┬──────────────────────────────────┐   │
│  │ 平台             │ 优先级    │ 理由                              │   │
│  ├────────────────┼──────────┼──────────────────────────────────┤   │
│  │ Telegram        │ P0       │ 首发平台，已设计完成               │   │
│  │ Discord         │ P1       │ Web3 第二大社区，API 能力对等      │   │
│  │ Farcaster       │ P2       │ 链上原生，理念最契合               │   │
│  │ Matrix          │ P2       │ 去中心化协议，开源生态             │   │
│  │ Slack           │ P3       │ 企业场景，管理能力有限             │   │
│  │ 微信            │ 暂缓      │ API 限制过多，不建议初期支持       │   │
│  └────────────────┴──────────┴──────────────────────────────────┘   │
│                                                                      │
│  核心结论:                                                            │
│  方案 D 不应该被定位为 "Telegram Bot 代理"                            │
│  而应定位为 "去中心化多平台社区管理协议"                               │
│  Telegram 只是第一个适配的平台                                        │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 18. 开发步骤 — 完整实施计划

### 18.1 整体开发阶段总览

```
┌──────────────────────────────────────────────────────────────────────┐
│                    开发阶段总览 (6 个 Sprint)                          │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Sprint 1 (4周): 链上基础 Pallet                                     │
│  ────────────────────────────────                                    │
│  输出: pallet-bot-registry + pallet-bot-consensus 可编译上线          │
│  验证: 节点可注册、质押、查询活跃列表                                  │
│                                                                      │
│  Sprint 2 (4周): Local Agent MVP                                     │
│  ────────────────────────────────                                    │
│  输出: Agent Docker 镜像，可接收 Webhook 并签名多播                   │
│  验证: TG 消息 → Agent → K 个节点收到签名消息                        │
│                                                                      │
│  Sprint 3 (4周): 节点验证 + Gossip 共识                               │
│  ────────────────────────────────                                    │
│  输出: 节点二进制，可验签、gossip、M/K 共识                           │
│  验证: 3 节点组网，消息确认延迟 <200ms                                │
│                                                                      │
│  Sprint 4 (3周): Leader 执行 + 群管理                                 │
│  ────────────────────────────────                                    │
│  输出: Leader 选举+执行+failover，Agent 验证+调 TG API                │
│  验证: 垃圾消息 → 共识 → Leader 回传 → Agent 删消息，<1s             │
│                                                                      │
│  Sprint 5 (3周): 链上存证 + 安全加固                                  │
│  ────────────────────────────────                                    │
│  输出: 批量上链、Equivocation 检测、信誉系统、Slash                   │
│  验证: 双发攻击被检测并 Slash，信誉分正确变化                         │
│                                                                      │
│  Sprint 6 (3周): 规则引擎 + 多平台基础                                │
│  ────────────────────────────────                                    │
│  输出: 可配置规则引擎、PlatformAdapter 抽象、Discord 适配器原型       │
│  验证: TG+DC 双平台同步管理                                          │
│                                                                      │
│  总计: ~21 周 (约 5 个月)                                             │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 18.2 代码仓库结构

```
cosmos/pallets/nexus/
├── bot-registry/          # Sprint 1: Bot 注册 + 授权
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
├── bot-consensus/         # Sprint 1: 节点注册 + 质押 + 消息确认
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
├── bot-billing/           # Sprint 5: 计费结算
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
├── bot-group-mgmt/        # Sprint 4: 群管理规则 (链上)
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
└── docs/
    ├── BOT_MODULE_DESIGN_PART2.md
    └── BOT_MODULE_DESIGN_PART3.md

nexus-agent/              # Sprint 2: Local Agent (独立仓库)
├── src/
│   ├── main.rs
│   ├── webhook.rs         # Telegram Webhook 接收
│   ├── signer.rs          # Ed25519 签名
│   ├── multicaster.rs     # 确定性多播
│   ├── chain_watcher.rs   # 链上数据订阅
│   ├── command_handler.rs # Sprint 4: 接收+验证 Leader 指令
│   ├── tg_executor.rs     # Sprint 4: 调用 Telegram API
│   ├── idempotency.rs     # Sprint 4: 幂等性表
│   └── adapters/          # Sprint 6: 多平台适配器
│       ├── mod.rs
│       ├── telegram.rs
│       └── discord.rs
├── Dockerfile
├── docker-compose.yml
└── Cargo.toml

nexus-node/               # Sprint 3: 项目节点 (独立仓库)
├── src/
│   ├── main.rs
│   ├── api_server.rs      # 接收 Agent 消息的 HTTP 端点
│   ├── verifier.rs        # 签名/IP/授权 验证
│   ├── gossip/
│   │   ├── mod.rs
│   │   ├── engine.rs      # GossipEngine 核心
│   │   ├── connection.rs  # WebSocket 连接管理
│   │   ├── messages.rs    # 8 种 Gossip 消息类型
│   │   └── state.rs       # MessageState 状态机
│   ├── consensus.rs       # M/K 共识判定
│   ├── leader.rs          # Sprint 4: Leader 选举+执行
│   ├── rule_engine/       # Sprint 6: 规则引擎
│   │   ├── mod.rs
│   │   ├── spam.rs
│   │   ├── keyword.rs
│   │   └── chain_query.rs
│   ├── chain_submitter.rs # Sprint 5: 批量上链
│   └── metrics.rs         # Prometheus 指标
├── Dockerfile
└── Cargo.toml
```

### 18.3 Sprint 1: 链上基础 Pallet (4 周)

```
┌──────────────────────────────────────────────────────────────────────┐
│  Sprint 1 · 链上基础 Pallet                                          │
│  目标: 链上节点注册/质押 + Bot 注册可用                                │
│  文档依据: 第 5 章 (pallet-bot-consensus) + 第 2 章 (pallet-bot-registry)│
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  第 1 周: pallet-bot-consensus 数据结构 + 存储                        │
│  ──────────────────────────────────────────                          │
│  Step 1.1: 创建 pallets/nexus/bot-consensus/ crate                 │
│    ├── Cargo.toml (依赖: frame-support, frame-system, sp-runtime)    │
│    ├── src/lib.rs                                                    │
│    │                                                                 │
│    │  定义数据结构:                                                   │
│    │  • ProjectNode<T> (node_id, operator, node_public_key,          │
│    │    endpoint_hash, stake, status, reputation, 统计字段)           │
│    │  • NodeStatus enum (Active, Probation, Suspended, Exiting)      │
│    │  • MessageConfirmation<T> (owner, sequence, msg_hash,           │
│    │    confirmed_by, confirmed_at)                                  │
│    │                                                                 │
│    │  定义存储:                                                       │
│    │  • Nodes<T>: StorageMap<NodeId → ProjectNode>                   │
│    │  • ActiveNodeList<T>: StorageValue<Vec<NodeId>>                 │
│    │  • MessageConfirmations<T>: StorageMap<msg_id → Confirmation>   │
│    │  • MinStake<T>: StorageValue<Balance>                           │
│    │                                                                 │
│  Step 1.2: 定义 Event 和 Error                                      │
│    • Event: NodeRegistered, NodeExited, NodeSuspended,               │
│             ConfirmationSubmitted, EquivocationReported               │
│    • Error: InsufficientStake, NodeAlreadyExists,                    │
│             NodeNotFound, NotNodeOperator, InvalidSignature          │
│                                                                      │
│  测试: 编译通过，存储可读写                                           │
│                                                                      │
│                                                                      │
│  第 2 周: pallet-bot-consensus 调用 + 逻辑                            │
│  ──────────────────────────────────────────                          │
│  Step 1.3: 实现 register_node()                                     │
│    • 验证质押 ≥ MinStake                                              │
│    • 锁定质押代币 (T::Currency::reserve)                              │
│    • 创建 ProjectNode (status=Probation, reputation=5000)            │
│    • 更新 ActiveNodeList                                             │
│    • 发出 NodeRegistered 事件                                        │
│                                                                      │
│  Step 1.4: 实现 exit_node()                                         │
│    • 验证调用者是 operator                                            │
│    • status → Exiting                                                │
│    • 设置冷却期 (如 100 blocks)                                       │
│    • 冷却期后释放质押 (on_finalize 或 scheduled call)                 │
│                                                                      │
│  Step 1.5: 实现 submit_confirmations() (简化版)                      │
│    • 接受批量消息确认                                                 │
│    • 验证提交者是 Active 节点                                         │
│    • 存储 MessageConfirmation                                        │
│                                                                      │
│  Step 1.6: 实现 report_equivocation() (简化版)                       │
│    • 接受双发证据 (同 sequence 不同 hash)                             │
│    • 验证两个签名都有效                                               │
│    • 存储证据 (惩罚逻辑 Sprint 5 实现)                                │
│                                                                      │
│  测试: 单元测试覆盖所有调用                                           │
│                                                                      │
│                                                                      │
│  第 3 周: pallet-bot-registry                                        │
│  ──────────────────────────────────────────                          │
│  Step 1.7: 创建 pallets/nexus/bot-registry/ crate                  │
│    │                                                                 │
│    │  定义数据结构:                                                   │
│    │  • BotRegistration<T> (owner, platform, bot_id_hash,            │
│    │    community_id_hash, owner_public_key, status)                 │
│    │  • Platform enum (Telegram, Discord, Slack, Matrix, Farcaster)  │
│    │  • BotStatus enum (Active, Suspended, Deactivated)              │
│    │                                                                 │
│    │  定义存储:                                                       │
│    │  • Bots<T>: StorageMap<bot_id_hash → BotRegistration>           │
│    │  • OwnerBots<T>: StorageMap<AccountId → Vec<bot_id_hash>>       │
│    │  • CommunityPlatforms<T>: StorageDoubleMap<community_id,        │
│    │    Platform → CommunityBinding>                                 │
│    │  • UserPlatformBindings<T>: StorageDoubleMap<AccountId,         │
│    │    Platform → platform_user_id>                                 │
│    │                                                                 │
│  Step 1.8: 实现调用                                                  │
│    • register_bot() — 注册 Bot + 存储 Ed25519 公钥                   │
│    • update_bot_public_key() — 更换 Agent 公钥                       │
│    • deactivate_bot() — 停用 Bot                                     │
│    • bind_community_platform() — 社区跨平台绑定                      │
│    • bind_user_platform() — 用户多平台身份绑定                       │
│                                                                      │
│  测试: 单元测试，Bot 注册/查询/停用流程                               │
│                                                                      │
│                                                                      │
│  第 4 周: Runtime 集成 + 集成测试                                     │
│  ──────────────────────────────────────────                          │
│  Step 1.9: 在 runtime/src/lib.rs 中注册两个 Pallet                   │
│    • 配置 pallet_bot_consensus::Config                                │
│    • 配置 pallet_bot_registry::Config                                 │
│    • 分配 PalletIndex                                                │
│                                                                      │
│  Step 1.10: 编写集成测试                                             │
│    • 注册节点 → 查询 ActiveNodeList                                  │
│    • 注册 Bot → 查询 BotRegistration                                 │
│    • 节点退出 → 冷却 → 质押释放                                      │
│                                                                      │
│  Step 1.11: 启动本地 dev chain 验证                                  │
│    • cargo build --release                                           │
│    • 通过 polkadot.js Apps 手动测试外部调用                           │
│                                                                      │
│  ✅ Sprint 1 交付物:                                                  │
│  • pallet-bot-consensus 可编译 + 测试通过                             │
│  • pallet-bot-registry 可编译 + 测试通过                              │
│  • Runtime 集成成功，dev chain 可跑                                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 18.4 Sprint 2: Local Agent MVP (4 周)

```
┌──────────────────────────────────────────────────────────────────────┐
│  Sprint 2 · Local Agent MVP                                          │
│  目标: Agent 可接收 TG Webhook，签名并多播到 K 个节点                  │
│  文档依据: 第 7 章 (Agent 设计) + 第 4 章 (消息协议) + 第 6 章 (流程) │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  第 1 周: 项目脚手架 + Webhook 接收                                   │
│  ──────────────────────────────────────────                          │
│  Step 2.1: 初始化 nexus-agent 仓库                                  │
│    • cargo init nexus-agent                                         │
│    • 依赖: axum, tokio, ed25519-dalek, reqwest, serde,               │
│            subxt (链上交互), tracing (日志)                            │
│    • Dockerfile: FROM rust:slim → 编译 → runtime 镜像 ~20MB          │
│                                                                      │
│  Step 2.2: 实现 webhook.rs — Telegram Webhook Server                 │
│    • Axum HTTP 服务器 (port 8443)                                    │
│    • TLS 自签名证书生成 (启动时自动)                                  │
│    • POST /webhook 端点                                              │
│    • 验证 X-Telegram-Bot-Api-Secret-Token header                     │
│    • 解析 Telegram Update JSON → 内部 TelegramUpdate 结构            │
│    • 返回 200 OK (Telegram 要求快速响应)                              │
│                                                                      │
│  Step 2.3: 实现 Telegram setWebhook 自动注册                         │
│    • 启动时调用 TG API: setWebhook(url, certificate, secret_token)   │
│    • 错误处理: API 失败 → 重试 3 次 → 退出                           │
│                                                                      │
│  测试: 用 ngrok 暴露本地端口，真实 TG Bot 发消息验证                  │
│                                                                      │
│                                                                      │
│  第 2 周: Ed25519 签名 + 消息构造                                     │
│  ──────────────────────────────────────────                          │
│  Step 2.4: 实现 signer.rs — 密钥管理                                 │
│    • 首次启动: 生成 Ed25519 密钥对 → 保存 /data/agent.key            │
│    • 后续启动: 从文件加载                                             │
│    • 显示公钥 (hex) → 群主复制到前端注册                              │
│                                                                      │
│  Step 2.5: 实现消息签名流程 (文档第 4.1 章)                          │
│    • sequence 原子递增 (持久化到 /data/sequence.dat)                  │
│    • message_hash = SHA256(telegram_update_json)                     │
│    • 构造 SignedMessage:                                              │
│      { owner_address, bot_id_hash, sequence, timestamp,              │
│        message_hash, telegram_update, owner_signature }              │
│    • Ed25519 签名: sign(owner_address + bot_id_hash +                │
│      sequence + timestamp + message_hash)                            │
│                                                                      │
│  测试: 单元测试签名/验签，确保与节点侧验签算法一致                    │
│                                                                      │
│                                                                      │
│  第 3 周: 确定性多播 + 链上订阅                                       │
│  ──────────────────────────────────────────                          │
│  Step 2.6: 实现 chain_watcher.rs — 链上数据订阅                      │
│    • subxt 连接 chain RPC (wss://...)                                │
│    • 订阅 ActiveNodeList 变化                                        │
│    • 本地缓存节点列表 (30s TTL 自动刷新)                              │
│    • 查询自身 Bot 注册状态                                            │
│    • 监听 NodeRegistered/NodeExited 事件 → 更新缓存                  │
│                                                                      │
│  Step 2.7: 实现 multicaster.rs — 确定性随机多播 (文档第 2 章)        │
│    • seed = SHA256(message_hash + sequence)                          │
│    • sorted_nodes = ActiveNodeList.sorted()                          │
│    • target_indices = 确定性随机 Fisher-Yates 取 K 个                 │
│    • 并发 POST 到 K 个节点: reqwest::Client (HTTP/2 连接池)          │
│    • 超时 3s / 节点，失败不重试 (gossip Pull 补偿)                    │
│    • 收集响应: 成功/失败/超时 计数                                    │
│    • 日志记录: 发送给哪些节点，耗时多少                                │
│                                                                      │
│  测试: mock 3 个 HTTP 端点，验证确定性选择算法正确                    │
│                                                                      │
│                                                                      │
│  第 4 周: Docker 打包 + 端到端联调                                    │
│  ──────────────────────────────────────────                          │
│  Step 2.8: Dockerfile + docker-compose.yml                           │
│    • 多阶段构建: builder → runtime (debian-slim)                     │
│    • 环境变量: BOT_TOKEN, CHAIN_RPC, OWNER_MNEMONIC                  │
│    • Volume: /data (密钥、序列号)                                     │
│    • Health check: /health 端点                                      │
│                                                                      │
│  Step 2.9: 端到端测试                                                 │
│    • 启动 dev chain (Sprint 1)                                       │
│    • 链上注册 3 个节点 (mock: 本地 3 个 HTTP 服务器)                  │
│    • 链上注册 Bot                                                     │
│    • 启动 Agent → 接收 TG Webhook → 签名 → 多播到 3 个 mock 节点     │
│    • 验证: mock 节点收到的消息签名可验证通过                          │
│    • 验证: 确定性选择算法两端一致                                     │
│                                                                      │
│  Step 2.10: 编写部署文档 (群主一键部署)                               │
│    • docker run 命令示例                                              │
│    • 环境变量说明                                                     │
│    • 常见问题排查                                                     │
│                                                                      │
│  ✅ Sprint 2 交付物:                                                  │
│  • nexus-agent Docker 镜像                                          │
│  • TG Webhook 接收 + Ed25519 签名 + 确定性多播                      │
│  • 链上节点列表订阅 + 缓存                                           │
│  • 端到端: TG 消息 → Agent → mock 节点 ✅                            │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 18.5 Sprint 3: 节点验证 + Gossip 共识 (4 周)

```
┌──────────────────────────────────────────────────────────────────────┐
│  Sprint 3 · 节点验证 + Gossip 共识                                    │
│  目标: 节点可验签、gossip 互通、达成 M/K 共识                          │
│  文档依据: 第 15 章 (Gossip) + 第 5 章 (节点网络) + 第 2 章 (共识)    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  第 1 周: 节点 HTTP 端点 + 消息验证                                   │
│  ──────────────────────────────────────────                          │
│  Step 3.1: 初始化 nexus-node 仓库                                   │
│    • 依赖: axum, tokio, ed25519-dalek, tokio-tungstenite,            │
│            subxt, serde, tracing, prometheus                         │
│                                                                      │
│  Step 3.2: 实现 api_server.rs — 接收 Agent 消息                      │
│    • POST /v1/message 端点                                           │
│    • 接收 SignedMessage                                               │
│    • 返回 200 OK + ack (快速响应)                                     │
│                                                                      │
│  Step 3.3: 实现 verifier.rs — 四层验证 (文档第 3 章)                 │
│    • ① Ed25519 验签 (owner_public_key 从链上缓存获取)                 │
│    • ② IP 白名单校验 (链上 AuthorizedIPs)                             │
│    • ③ 链上授权状态检查 (Bot 是否 Active)                             │
│    • ④ 目标节点验证: 自己在确定性选择的 K 个中                       │
│    • 任一验证失败 → 拒绝 + 记录原因                                  │
│                                                                      │
│  Step 3.4: 实现链上数据缓存                                          │
│    • subxt 订阅 ActiveNodeList、Bots、公钥                            │
│    • 本地 HashMap 缓存 (30s TTL)                                     │
│    • 验签时直接查缓存 → ~0ms                                         │
│                                                                      │
│  测试: 发送合法/非法消息，验证通过/拒绝                               │
│                                                                      │
│                                                                      │
│  第 2 周: Gossip 引擎 — 连接管理 + 消息类型                          │
│  ──────────────────────────────────────────                          │
│  Step 3.5: 实现 gossip/connection.rs — WebSocket 连接管理            │
│    • 启动时从链上读 ActiveNodeList → 连接所有节点                     │
│    • wss://{node_endpoint}/v1/gossip                                 │
│    • 双向认证握手: { node_id, timestamp, nonce, signature }          │
│    • 连接池: HashMap<NodeId, WebSocketStream>                        │
│    • 心跳: 每 30s Ping/Pong                                         │
│    • 断线重连: 指数退避 (1s, 2s, 4s, 8s, max 60s)                   │
│    • 连接状态: Connected / Reconnecting / Dead                       │
│    • 链上 NodeRegistered → 连接新节点                                │
│    • 链上 NodeExited → 断开+清理                                     │
│                                                                      │
│  Step 3.6: 实现 gossip/messages.rs — 消息类型 (文档 15.3 章)        │
│    • GossipEnvelope { version, msg_type, sender_node_id,             │
│                       timestamp, sender_signature }                  │
│    • GossipPayload enum:                                             │
│      MessageSeen, MessagePull, MessagePullResponse,                  │
│      DecisionVote, EquivocationAlert, ExecutionResult,               │
│      LeaderTakeover, Heartbeat                                       │
│    • bincode 序列化/反序列化                                          │
│    • 签名: Ed25519 签 (envelope + payload)                           │
│                                                                      │
│  测试: 序列化/反序列化 round-trip 测试                                │
│                                                                      │
│                                                                      │
│  第 3 周: Gossip 引擎 — 状态机 + 共识                                │
│  ──────────────────────────────────────────                          │
│  Step 3.7: 实现 gossip/state.rs — MessageState (文档 15.4.2 章)     │
│    • MessageState { msg_id, status, original_message,                │
│      seen_nodes, decision_votes, target_nodes, leader,               │
│      backups, timers... }                                            │
│    • MessageStatus enum: HeardViaSeen → Received → Pending →         │
│      Confirmed → Executing → Completed / Timeout / Failed            │
│    • SeenRecord { node_id, msg_hash, seen_at, signature }            │
│    • 消息过期: 60s 自动清理 (gc_expired_messages)                    │
│                                                                      │
│  Step 3.8: 实现 gossip/engine.rs — 核心处理 (文档 15.4.3 章)        │
│    • on_agent_message() → 验证 → 广播 Seen → check_consensus        │
│    • on_gossip_message() → match msg_type:                           │
│      - MessageSeen → 记录 seen → 检查 hash 一致性 →                  │
│        如果无原始消息启动 Pull Timer (500ms) → check_consensus       │
│      - MessagePull → 回复 PullResponse                               │
│      - MessagePullResponse → 验群主签名 → 存储 → 广播 Seen           │
│    • check_consensus():                                               │
│      M = ceil(K * 2/3), 如果 consistent_seen >= M → CONFIRMED       │
│    • 优先级队列: Equivocation > Seen/Vote > Result > Pull > HB      │
│                                                                      │
│  Step 3.9: 实现 consensus.rs — M/K 共识判定                          │
│    • count_consistent_seen(): 统计 msg_hash 一致的 seen 数           │
│    • has_conflicting_hashes(): 检测 equivocation                     │
│    • 确定性计算 target_nodes (与 Agent 侧算法完全一致)               │
│                                                                      │
│  测试: 3 节点本地组网，模拟消息流转，验证共识达成                     │
│                                                                      │
│                                                                      │
│  第 4 周: 集成测试 + 性能验证                                         │
│  ──────────────────────────────────────────                          │
│  Step 3.10: 3 节点 + Agent 完整联调                                  │
│    • docker-compose: 1 Agent + 3 Node + 1 dev-chain                  │
│    • 真实 TG Bot 发消息 → Agent → 3 节点                             │
│    • 验证: 3/3 节点收到消息并 CONFIRMED                              │
│    • 验证: gossip MessageSeen 正确交换                                │
│    • 验证: 确定性选择两端一致                                         │
│                                                                      │
│  Step 3.11: Pull 补偿测试                                            │
│    • 模拟 1 个节点未收到 Agent 消息 (网络延迟)                        │
│    • 验证: 通过 gossip Seen → Pull → PullResponse 补偿               │
│    • 验证: 该节点最终也达到 CONFIRMED                                 │
│                                                                      │
│  Step 3.12: 延迟基准测试                                             │
│    • 测量: Agent 发送 → 共识达成 耗时                                 │
│    • 目标: 同机 <50ms, 同区域 <100ms, 跨区域 <200ms                  │
│    • Prometheus 指标: gossip_seen_latency_ms,                        │
│      gossip_consensus_latency_ms, gossip_pull_rate                   │
│                                                                      │
│  Step 3.13: Dockerfile + 节点部署文档                                 │
│    • 环境变量: NODE_ID, SIGNING_KEY, CHAIN_RPC, LISTEN_PORT          │
│    • 节点运营者部署指南                                               │
│                                                                      │
│  ✅ Sprint 3 交付物:                                                  │
│  • nexus-node 二进制 + Docker 镜像                                  │
│  • 完整 Gossip 引擎 (8 种消息类型)                                   │
│  • M/K 共识 + Pull 补偿                                              │
│  • 端到端: TG → Agent → 3 节点 gossip → CONFIRMED ✅                │
│  • 延迟基准: 同区域 <100ms ✅                                        │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 18.6 Sprint 4: Leader 执行 + 群管理 (3 周)

```
┌──────────────────────────────────────────────────────────────────────┐
│  Sprint 4 · Leader 执行 + 群管理                                      │
│  目标: 共识后 Leader 回传指令，Agent 验证并执行 TG API 管理动作         │
│  文档依据: 第 14 章 (Leader) + 第 13 章 (群管理) + 第 16 章 (延迟)    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  第 1 周: Leader 选举 + 节点侧执行流程                                │
│  ──────────────────────────────────────────                          │
│  Step 4.1: 实现 leader.rs — Leader 选举 (文档 14.2 章)              │
│    • 确定性轮转: leader = target_nodes[sequence % K]                 │
│    • backups = target_nodes 中排除 leader 的其余节点                  │
│    • backup_rank: 按确定性排序                                       │
│    • 所有节点独立计算 → 结果一致                                      │
│                                                                      │
│  Step 4.2: DecisionVote 流程                                        │
│    • 共识 CONFIRMED 后 → 运行规则引擎 (简单版: 关键词匹配)           │
│    • 规则命中 → 构造 DecisionVote { action_type, params }            │
│    • 广播 DecisionVote → 收集 M/K 投票                               │
│    • M 个投票一致 → 触发执行                                         │
│                                                                      │
│  Step 4.3: Leader 执行流程                                           │
│    • 我是 Leader? → 构造 AdminAction 指令                             │
│    • 附带 M 个 DecisionVote 签名作为 consensus_proof                 │
│    • POST 到 Agent 的 /v1/execute 端点                               │
│    • 等待 Agent 返回 ExecutionResult (含 agent_receipt)               │
│    • 广播 ExecutionResult 给其他 K-1 节点                            │
│                                                                      │
│  Step 4.4: Failover 机制                                             │
│    • 非 Leader 节点: 启动 3s 超时计时器                               │
│    • 超时 → Backup#1 接替 → 广播 LeaderTakeover                     │
│    • Backup#1 也超时 → Backup#2 接替 → ...                           │
│    • 全部失败 → 标记 Failed + 日志                                   │
│                                                                      │
│  测试: 模拟 Leader 正常/超时/恶意，验证 failover 正确触发             │
│                                                                      │
│                                                                      │
│  第 2 周: Agent 侧 — 指令验证 + TG API 执行                         │
│  ──────────────────────────────────────────                          │
│  Step 4.5: 实现 command_handler.rs — 接收 Leader 指令                │
│    • POST /v1/execute 端点                                           │
│    • 接收 AdminAction { action_id, action_type, params,              │
│      consensus_proof, leader_node_id, sequence }                     │
│                                                                      │
│  Step 4.6: Agent 三层验证 (文档 14.7 章)                             │
│    • ① 验证 Leader 身份: 独立计算 expected_leader =                  │
│      target_nodes[sequence % K]，拒绝非法 Leader                     │
│    • ② 验证 Backup 合法性 (如果是 failover):                         │
│      rank 正确 + 等待时间 ≥ rank × 3s                                │
│    • ③ 验证 consensus_proof: M 个 DecisionVote 签名逐一验签          │
│      确保 action_type + params 一致                                  │
│    • ④ 验证授权范围: action_type 在允许列表中                         │
│                                                                      │
│  Step 4.7: 实现 idempotency.rs — 幂等性保障 (文档 14.4 章)          │
│    • executed_actions: HashMap<action_id, ExecutionRecord>            │
│    • 同一 action_id 只执行一次                                       │
│    • 重复请求 → 返回缓存的 ExecutionResult                           │
│    • 过期清理: 1 小时后移除旧记录                                     │
│                                                                      │
│  Step 4.8: 实现 tg_executor.rs — Telegram API 调用                   │
│    • 支持的管理动作:                                                  │
│      - sendMessage (回复/警告)                                       │
│      - deleteMessage (删除垃圾消息)                                   │
│      - banChatMember (封禁)                                          │
│      - restrictChatMember (禁言)                                     │
│      - approveChatJoinRequest (审批入群)                              │
│      - declineChatJoinRequest (拒绝入群)                              │
│    • 使用 Bot Token 调用 TG API                                      │
│    • 构造 AgentReceipt { action_id, result, agent_signature }        │
│    • 返回给 Leader                                                    │
│                                                                      │
│  测试: 真实 TG Bot 测试删消息、踢人、禁言                             │
│                                                                      │
│                                                                      │
│  第 3 周: 群管理 E2E + pallet-bot-group-mgmt                        │
│  ──────────────────────────────────────────                          │
│  Step 4.9: 创建 pallet-bot-group-mgmt (链上群管理规则)                │
│    • GroupRule { rule_type, params, enabled }                         │
│    • 存储: GroupRules<T>: StorageDoubleMap<community_id,              │
│      rule_type → GroupRule>                                          │
│    • 调用: set_group_rule(), remove_group_rule()                     │
│    • ActionLog<T>: 管理动作日志                                      │
│                                                                      │
│  Step 4.10: 群管理 E2E 测试场景                                      │
│    • 场景 A: 垃圾消息 → 关键词命中 → 共识 → Leader 回传 →            │
│      Agent 删消息 + 发警告，全程 <1s                                  │
│    • 场景 B: 新人入群 → 发 CAPTCHA 按钮 → 用户点击 →                 │
│      callback_query → 验证 → 解除限制                                │
│    • 场景 C: Leader 超时 → Backup 接替 → 动作仍执行成功              │
│                                                                      │
│  Step 4.11: Prometheus 指标扩展                                      │
│    • leader_execution_latency_ms                                     │
│    • leader_failover_count                                           │
│    • decision_vote_agreement_rate                                    │
│    • tg_api_latency_ms                                               │
│                                                                      │
│  ✅ Sprint 4 交付物:                                                  │
│  • Leader 选举 + 执行 + failover 完整实现                            │
│  • Agent 三层验证 + 幂等性 + TG API 调用                             │
│  • pallet-bot-group-mgmt 链上群规则                                  │
│  • E2E: 垃圾消息 → 删除，全程 <1s ✅                                 │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 18.7 Sprint 5: 链上存证 + 安全加固 (3 周)

```
┌──────────────────────────────────────────────────────────────────────┐
│  Sprint 5 · 链上存证 + 安全加固                                       │
│  目标: 批量上链、Equivocation Slash、信誉系统、计费                    │
│  文档依据: 第 5.2-5.3 章 (质押/信誉) + 第 3 章 (防攻击) + 第 15.8 章 │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  第 1 周: 批量上链 + Equivocation Slash                               │
│  ──────────────────────────────────────────                          │
│  Step 5.1: 实现 chain_submitter.rs — 批量上链                        │
│    • 消息确认队列: 每收到 CONFIRMED → 加入批量队列                    │
│    • 触发条件: 队列满 50 条 OR 每 30 秒                               │
│    • 调用 pallet-bot-consensus::submit_confirmations()               │
│    • 签名提交: 附带 M 个节点签名                                     │
│    • 失败重试: 指数退避 (最多 3 次)                                   │
│                                                                      │
│  Step 5.2: 完善 report_equivocation() — Slash 逻辑                  │
│    • 链上验证: 同一 (owner, sequence) 两个不同 msg_hash               │
│    • 验证两个 owner_signature 都有效                                  │
│    • 确认 equivocation → Slash owner 质押 (如 10%)                   │
│    • 奖励举报者 (Slash 金额的 50%)                                   │
│    • 发出 EquivocationConfirmed 事件                                 │
│                                                                      │
│  Step 5.3: 节点侧 Equivocation 检测                                 │
│    • gossip/engine.rs: has_conflicting_hashes() 触发时               │
│    • 构造 EquivocationAlert → 广播所有 N 个节点                      │
│    • 提交链上 report_equivocation()                                  │
│                                                                      │
│  测试: 模拟群主双发 → 节点检测 → Slash 执行                          │
│                                                                      │
│                                                                      │
│  第 2 周: 信誉系统 + 节点管理                                         │
│  ──────────────────────────────────────────                          │
│  Step 5.4: 实现信誉评分系统 (文档 5.3 章)                            │
│    • 初始: 5000 / 10000                                              │
│    • 加分: 正常确认 +1, 举报作弊 +100                                │
│    • 减分: 超时未响应 -10, 被举报延迟 -20,                            │
│            Leader 超时 -20, 伪造消息 -5000                            │
│    • pallet 内 update_reputation() 函数                              │
│                                                                      │
│  Step 5.5: 信誉驱动的节点管理                                        │
│    • reputation < 2000 → status=Suspended → 不再被选中               │
│    • reputation < 1000 → 强制退出 + 部分 Slash                       │
│    • 恢复: appeal_suspension() + 社区治理投票                        │
│                                                                      │
│  Step 5.6: 实现 report_node_offline()                                │
│    • 节点提交证据: 在 N 条消息中该节点应参与但未响应                  │
│    • 链上验证: 确定性选择算法确认该节点确实应参与                      │
│    • 连续离线 → 信誉扣分 → 自动 Suspend                             │
│                                                                      │
│  Step 5.7: LeaderStats 扩展 (文档 14.6 章)                          │
│    • pallet 存储: LeaderStats<T> per node                            │
│    • 统计: total_leads, successful, timeout, failed                  │
│    • report_leader_timeout() extrinsic                               │
│    • 连续 3 次 timeout → 信誉 -100                                   │
│                                                                      │
│  测试: 模拟节点离线/Leader 超时 → 信誉变化 → 自动 Suspend            │
│                                                                      │
│                                                                      │
│  第 3 周: 计费 + 安全审计                                             │
│  ──────────────────────────────────────────                          │
│  Step 5.8: 创建 pallet-bot-billing                                   │
│    • ServicePlan { price_per_msg, price_per_action, max_messages }   │
│    • OwnerBilling { balance, messages_used, actions_used }            │
│    • charge_message(): 每条消息确认扣费                               │
│    • charge_action(): 每次管理动作扣费                                │
│    • deposit(): 充值                                                  │
│    • 余额不足 → Bot 暂停                                             │
│                                                                      │
│  Step 5.9: Gossip 安全加固 (文档 15.8 章)                            │
│    • 速率限制: 每节点每秒 1000 条                                     │
│    • 签名验证前置: 无效签名 → 立即丢弃                                │
│    • msg_id 去重: 已处理不重复处理                                    │
│    • 持续违规 → 断开 + 上报                                          │
│    • 消息过期: timestamp > 60s → 丢弃                                │
│                                                                      │
│  Step 5.10: 安全审计 checklist                                       │
│    • [ ] Agent 签名无法伪造                                           │
│    • [ ] 节点无法获取 Bot Token                                       │
│    • [ ] M/K 共识正确阻止少数派                                       │
│    • [ ] Equivocation 必定被检测                                      │
│    • [ ] Leader 无法篡改指令 (consensus_proof)                        │
│    • [ ] Agent 幂等表防重复执行                                       │
│    • [ ] Gossip 洪泛被速率限制                                        │
│                                                                      │
│  ✅ Sprint 5 交付物:                                                  │
│  • 批量上链确认 (链上不可篡改证明)                                    │
│  • Equivocation 检测 + Slash                                         │
│  • 信誉系统 + 自动 Suspend                                           │
│  • pallet-bot-billing 计费模块                                       │
│  • Gossip 安全加固                                                    │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 18.8 Sprint 6: 规则引擎 + 多平台基础 (3 周)

```
┌──────────────────────────────────────────────────────────────────────┐
│  Sprint 6 · 规则引擎 + 多平台基础                                     │
│  目标: 可配置规则引擎 + PlatformAdapter 抽象 + Discord 原型            │
│  文档依据: 第 13 章 (群管理自动化) + 第 17 章 (多平台)                │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  第 1 周: 规则引擎                                                    │
│  ──────────────────────────────────────────                          │
│  Step 6.1: 实现 rule_engine/mod.rs — 规则引擎框架                    │
│    • RuleEngine trait:                                                │
│      fn evaluate(event: &UnifiedEvent, rules: &[GroupRule])          │
│        -> Vec<RuleResult>                                            │
│    • RuleResult { rule_id, action: Option<UnifiedAction>,            │
│      confidence: f32, reason: String }                               │
│    • 规则从链上 pallet-bot-group-mgmt 加载 (缓存)                    │
│                                                                      │
│  Step 6.2: 实现基础规则模块                                          │
│    • spam.rs:                                                         │
│      - 刷屏检测 (滑动窗口: N 条/M 秒)                                │
│      - URL 黑名单                                                    │
│      - 新人 + 链接 = 高风险                                          │
│    • keyword.rs:                                                      │
│      - 关键词黑名单 (Aho-Corasick 多模匹配)                          │
│      - 正则表达式规则                                                 │
│      - 仿冒管理员检测 (编辑距离)                                      │
│    • chain_query.rs:                                                  │
│      - Token 持仓查询 → 角色分配                                     │
│      - NFT 持有验证 → 入群条件                                       │
│                                                                      │
│  Step 6.3: 规则引擎与 Gossip 流水线集成                               │
│    • 消息 CONFIRMED 后并行运行规则引擎                                │
│    • 规则命中 → 构造 DecisionVote → 广播                             │
│    • 规则引擎 <5ms (基础规则)                                        │
│                                                                      │
│  测试: 发各类垃圾消息 → 正确命中规则 → 正确动作                      │
│                                                                      │
│                                                                      │
│  第 2 周: PlatformAdapter 抽象                                        │
│  ──────────────────────────────────────────                          │
│  Step 6.4: 定义统一数据模型 (文档 17.5 章)                           │
│    • 共享 crate: nexus-common                                       │
│    • Platform enum                                                    │
│    • UnifiedEventType enum (MessageCreated, MemberJoined...)         │
│    • UnifiedEvent struct                                              │
│    • UnifiedUser struct                                               │
│    • UnifiedAction enum (SendMessage, BanUser, AssignRole...)        │
│                                                                      │
│  Step 6.5: 定义 PlatformAdapter trait                                │
│    • fn parse_event(raw: &[u8]) -> Result<UnifiedEvent>              │
│    • async fn execute_action(action: &UnifiedAction) -> Result<()>   │
│    • fn platform() -> Platform                                       │
│                                                                      │
│  Step 6.6: 重构 Agent — 将 TG 代码迁移到 TelegramAdapter            │
│    • adapters/telegram.rs: impl PlatformAdapter for TelegramAdapter  │
│    • webhook.rs → 调用 adapter.parse_event()                         │
│    • tg_executor.rs → 调用 adapter.execute_action()                  │
│    • 验证: 重构后所有 Sprint 2-4 测试仍通过                           │
│                                                                      │
│  Step 6.7: 重构 Node — SignedMessage 增加 platform 字段              │
│    • SignedMessage { ..., platform: Platform }                       │
│    • 规则引擎按 platform 加载规则集                                   │
│    • AdminAction 增加 platform 字段                                  │
│    • 验证: 重构后所有 Sprint 3-4 测试仍通过                           │
│                                                                      │
│                                                                      │
│  第 3 周: Discord 适配器原型 + 跨平台联调                             │
│  ──────────────────────────────────────────                          │
│  Step 6.8: 实现 adapters/discord.rs — DiscordAdapter                 │
│    • Discord Gateway WebSocket 连接                                   │
│      - Identify (token + intents)                                    │
│      - Heartbeat (OP 1)                                              │
│      - Reconnect / Resume                                            │
│    • 事件解析: MESSAGE_CREATE → UnifiedEvent::MessageCreated          │
│      GUILD_MEMBER_ADD → UnifiedEvent::MemberJoined                   │
│      INTERACTION_CREATE → UnifiedEvent::ButtonClicked                │
│    • 管理动作执行: Discord REST API                                   │
│      BanUser → PUT /guilds/{}/bans/{}                                │
│      DeleteMessage → DELETE /channels/{}/messages/{}                  │
│      AssignRole → PUT /guilds/{}/members/{}/roles/{}                 │
│      SendMessage → POST /channels/{}/messages                        │
│                                                                      │
│  Step 6.9: 跨平台联调                                                │
│    • docker-compose: 1 Agent(TG+DC) + 3 Node + dev-chain             │
│    • 场景 1: TG 群发垃圾 → 删除 + 在 DC 也 BAN (跨平台联动)         │
│    • 场景 2: DC 服务器发垃圾 → 删除 + 在 TG 也 BAN                  │
│    • 场景 3: 链上提案 → TG + DC 同时收到公告                         │
│                                                                      │
│  Step 6.10: 链上 Pallet 扩展                                         │
│    • pallet-bot-registry: 增加 Platform 参数                         │
│    • CommunityPlatforms 存储                                         │
│    • UserPlatformBindings 存储                                       │
│    • 验证: 同一链上社区绑定 TG + DC                                  │
│                                                                      │
│  ✅ Sprint 6 交付物:                                                  │
│  • 可配置规则引擎 (垃圾/关键词/链上)                                  │
│  • PlatformAdapter 抽象 + TelegramAdapter 重构                       │
│  • DiscordAdapter 原型可工作                                          │
│  • 跨平台联动: TG+DC 双平台管理 ✅                                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 18.9 开发依赖关系图

```
Sprint 1                Sprint 2              Sprint 3
pallet-bot-consensus ──▶ Agent 链上订阅 ──▶ 节点链上缓存
pallet-bot-registry  ──▶ Agent Bot 查询  ──▶ 节点验签

Sprint 1 ──▶ Sprint 2 ──▶ Sprint 3 ──▶ Sprint 4 ──▶ Sprint 5
  链上          Agent        节点         Leader       安全
  Pallet        MVP         Gossip       执行         加固
                                           │
                                           ▼
                                        Sprint 6
                                        规则引擎
                                        多平台

关键路径: Sprint 1 → 2 → 3 → 4 (最短可用路径)
Sprint 5, 6 可与 Sprint 4 部分并行
```

### 18.10 每个 Sprint 的验收标准

| Sprint | 核心验收标准 | 测试方式 |
|--------|-------------|---------|
| **S1** | 链上注册节点+Bot，查询 ActiveNodeList | polkadot.js Apps 手动测试 + 单元测试 |
| **S2** | TG 发消息 → Agent 签名 → 3 个 mock 节点收到 | docker-compose + 真实 TG Bot |
| **S3** | 3 节点 gossip 共识达成，延迟 <200ms | docker-compose + Prometheus 指标 |
| **S4** | 垃圾消息 → 共识 → Leader → Agent 删消息 <1s | E2E 自动化测试 |
| **S5** | 双发攻击 → 检测 → Slash，信誉分正确 | 模拟攻击脚本 |
| **S6** | TG+DC 双平台同步 BAN | docker-compose + 真实 TG+DC Bot |

### 18.11 技术栈汇总

```
┌──────────────────────────────────────────────────────────────────────┐
│                    技术栈                                              │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────────┬─────────────────────────────────────────────┐ │
│  │ 层               │ 技术选型                                     │ │
│  ├──────────────────┼─────────────────────────────────────────────┤ │
│  │ 链上 Pallet      │ Substrate FRAME (Rust)                      │ │
│  │ Agent            │ Rust + Axum + Tokio + ed25519-dalek         │ │
│  │ 节点             │ Rust + Axum + Tokio + tokio-tungstenite     │ │
│  │ 链上交互         │ subxt (Rust Substrate 客户端)                │ │
│  │ Gossip 传输      │ WebSocket (TLS) + bincode 序列化             │ │
│  │ 签名算法         │ Ed25519                                      │ │
│  │ 哈希算法         │ SHA256 (消息) + BLAKE2b (链上)               │ │
│  │ HTTP 客户端      │ reqwest (HTTP/2 连接池)                      │ │
│  │ 状态存储         │ 内存 HashMap (Agent/节点)                    │ │
│  │ 持久存储         │ 文件 (Agent 密钥/序列号)                     │ │
│  │ 监控             │ Prometheus + Grafana                         │ │
│  │ 日志             │ tracing + tracing-subscriber                 │ │
│  │ 容器化           │ Docker + docker-compose                      │ │
│  │ CI/CD            │ GitHub Actions                               │ │
│  └──────────────────┴─────────────────────────────────────────────┘ │
│                                                                      │
│  核心 Rust crate 依赖:                                                │
│  ┌──────────────────┬────────────────────────────────────────────┐  │
│  │ crate            │ 用途                                        │  │
│  ├──────────────────┼────────────────────────────────────────────┤  │
│  │ axum 0.7+        │ HTTP 服务器 (Agent + Node)                  │  │
│  │ tokio 1.x        │ 异步运行时                                  │  │
│  │ ed25519-dalek 2  │ Ed25519 签名/验签                           │  │
│  │ reqwest 0.12+    │ HTTP/2 客户端 (Agent→Node, Leader→Agent)    │  │
│  │ tokio-tungstenite│ WebSocket (Gossip)                          │  │
│  │ subxt 0.35+      │ Substrate 链上交互                          │  │
│  │ serde + bincode  │ 序列化                                      │  │
│  │ sha2             │ SHA256 哈希                                  │  │
│  │ tracing          │ 结构化日志                                  │  │
│  │ prometheus       │ 指标采集                                    │  │
│  │ aho-corasick     │ 多模关键词匹配 (规则引擎)                   │  │
│  └──────────────────┴────────────────────────────────────────────┘  │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```
