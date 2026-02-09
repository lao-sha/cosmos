# Nexus Node v0.1.0 — 深度审查报告

**审查范围:** `nexus-node/src/` 全部 11 个源文件 + gossip/ 子模块 (≈5,200 行)
**审查日期:** 2026-02-09
**发现总计:** 5 Critical · 6 High · 12 Medium · 7 Low
**修复状态:** 0/5 Critical · 0/6 High · 0/12 Medium · 0/7 Low

---

## C — Critical（安全漏洞 / 数据正确性）

### C1. `GossipPayload` 使用 `#[serde(untagged)]` — 反序列化歧义

**文件:** `types.rs:151`
```rust
#[serde(untagged)]
pub enum GossipPayload {
    Seen(SeenPayload),
    Pull(PullPayload),
    ...
}
```

`untagged` 枚举按变体顺序尝试反序列化。多个载荷类型有重叠字段名（如 `msg_id`），可能导致 `PullPayload` 被错误解析为 `SeenPayload`。恶意节点可构造歧义载荷。

**修复:** 改为内部标签或利用 `GossipEnvelope.msg_type` 字段手动分派反序列化。

### C2. Gossip 消息未签名 — 任何节点可伪造消息

**文件:** `gossip/engine.rs` 全文

所有 outbound `GossipEnvelope` 的 `sender_signature` 均为空字符串。`on_gossip_message` 从不验证签名。攻击者可伪造:
- `MessageSeen` → 操纵共识投票
- `ExecutionResult` → 虚假标记消息完成
- `LeaderTakeover` → 劫持执行权
- `EquivocationAlert` → 陷害诚实节点

**修复:** 在 GossipEngine 中添加信封签名 + 验签逻辑。(结构性变更，标记为 TODO)

### C3. `ChainCache` 所有 `RwLock` 使用 `.unwrap()` — 锁中毒会级联 panic

**文件:** `chain_cache.rs` (约 15 处)
```rust
self.bots.read().unwrap()
self.nodes.write().unwrap()
self.group_configs.read().unwrap()
```

任何线程在持锁期间 panic，后续所有访问都会 panic → 节点崩溃。

**修复:** 全部替换为 `.unwrap_or_else(|e| e.into_inner())`

### C4. `load_or_generate_signer` 回退到 `dev::alice()` — 不安全的默认密钥

**文件:** `main.rs:211`
```rust
let kp = subxt_signer::sr25519::dev::alice();
```

密钥文件缺失或损坏时，静默使用 Alice dev 密钥。生产环境所有无密钥节点共享同一身份。

**修复:** 生成随机密钥并持久化，而非使用 dev 密钥。

### C5. `fetch_active_node_list` 返回空列表 — 链客户端未实际解码数据

**文件:** `chain_client.rs:63-81`
```rust
let mut node_ids = Vec::new();
// SCALE: Vec prefix = compact length ...
// 简化: 仅记录日志，实际解码需要 codec
Ok(node_ids) // 永远返回空!
```

`fetch_bot` 和 `fetch_node` 同样返回硬编码占位值（全零公钥、"Active" 状态、reputation=5000）。链上数据读取完全不可用。

**修复:** 使用 `codec::Decode` 正确解码 SCALE 数据。(结构性变更，标记为 TODO + warn 日志)

---

## H — High（逻辑缺陷 / 可利用漏洞）

### H1. `SequenceTracker::check_and_update` TOCTOU 竞态

**文件:** `chain_submitter.rs:274-305`
```rust
match self.last_seen.get(&key) {
    Some(last) => {
        let last_val = *last;
        if sequence > last_val {
            drop(last);
            self.last_seen.insert(key, sequence); // 竞态窗口!
```

`get()` 后 `drop()` 再 `insert()`，两个并发调用可能都通过检查。

**修复:** 使用 `DashMap::entry()` API 实现原子操作。

### H2. Leader 选举不一致 — engine 总选第一个，executor 用 round-robin

**文件:** `gossip/engine.rs:84` vs `leader.rs:135`

engine: `let leader = targets.first().cloned();` (永远选 index 0)
executor: `let leader_idx = (sequence as usize) % k;` (round-robin)

导致 engine 认为的 Leader 和实际 Leader 不一致。

**修复:** engine 中也使用 `LeaderExecutor::elect_leader()` 的结果。

### H3. Gossip 网络层不转发消息到 GossipEngine — gossip 层完全失效

**文件:** `gossip/network.rs:131, :213`
```rust
// TODO: 转发到 GossipEngine 处理
```

WebSocket 收到的消息被解析但丢弃，gossip 共识无法工作。

**修复:** 需要传入 GossipEngine 引用或 channel。(结构性变更，标记为 TODO)

### H4. `endpoint_to_ws` 端口映射使用朴素字符串替换

**文件:** `gossip/network.rs:237-245`
```rust
http_endpoint
    .replace(":8080", ":9090")
    .replace(":8081", ":9091")
```

仅覆盖 4 个端口，且 URL 路径中出现 `:8080` 也会被替换。

**修复:** 使用 `url::Url` 解析后替换端口。(当前先改进为 regex 或解析逻辑)

### H5. `action_type_name` 未知类型默认返回 "Ban"

**文件:** `chain_client.rs:234`
```rust
_ => "Ban", // 任何未知 idx 都变成 Ban!
```

上链时动作类型被错误记录。

**修复:** 返回 panic 或 "Unknown"。

### H6. 配置持久化使用非原子写入

**文件:** `chain_cache.rs:255`
```rust
std::fs::write(&path, json) // 非原子
```

写入中途崩溃会留下损坏文件。

**修复:** 写入临时文件 + rename。

---

## M — Medium（健壮性 / 性能 / 设计问题）

### M1. `NodeConfig` 派生 `Debug` — 日志可能泄露敏感路径

**文件:** `config.rs:2`

### M2. `BlacklistRule` 每条消息重新编译正则

**文件:** `rule_engine.rs:566`
```rust
regex::Regex::new(pattern).map(|re| re.is_match(&text_lower))
```

### M3. `GossipPayload` untagged 序列化不可靠 — 应配合 `msg_type` 分派

### M4. `check_consensus` 使用 `get_mut` 但初始只需读权限

**文件:** `gossip/state.rs:129`

### M5. `ChainSubmitter` 队列无上限 — 可能 OOM

**文件:** `chain_submitter.rs:88-104`

### M6. `SequenceTracker` 从不清理旧条目 — 内存泄漏

**文件:** `chain_submitter.rs:255-312`

### M7. `check_config_version_from_seen` 是空实现

**文件:** `gossip/engine.rs:410-417`

### M8. `handle_seen` 不检查发送者是否为目标节点 — 任意节点可影响共识

**文件:** `gossip/engine.rs:190-239`

### M9. `submit_confirmations/action_logs/equivocation` 全是 TODO stub

**文件:** `chain_submitter.rs:202-227`

### M10. `SequenceTracker` 已定义但未集成到消息处理管线

**文件:** `chain_submitter.rs` — 从未在 `api.rs` 或 `verifier.rs` 中使用

### M11. `discover_and_connect` 无限 spawn 连接任务

**文件:** `gossip/network.rs:183-231`

### M12. `events` 变量未使用

**文件:** `chain_client.rs:138`
```rust
let events = self.api.tx()...  // 未使用
```

---

## L — Low（代码质量 / 测试）

### L1. `api.rs` 导入了未使用的 `error`

**文件:** `api.rs:7`

### L2. `GroupConfig` 基础字段缺少 `#[serde(default)]`

**文件:** `types.rs:445-517`

### L3. `select_k` 和 `deterministic_select_ids` 在 3 处重复

`verifier.rs`, `gossip/engine.rs`, Agent — 应提取为共享 crate。

### L4. 多模块 0 测试覆盖

`config.rs`, `gossip/network.rs`, `chain_client.rs` 无测试。

### L5. `JoinApprovalPolicy` 缺少 `Default` derive

**文件:** `types.rs:426`

### L6. 时间戳单位不一致 — 秒 vs 毫秒

`verifier.rs` 用秒 (`timestamp()`)，`gossip/state.rs` 用毫秒 (`timestamp_millis()`)。

### L7. `NodeConfig` 不验证 `node_id` 格式

`config.rs:25` — 接受任意字符串，应验证 hex 格式。

---

## 修复优先级

| 优先级 | ID | 工作量 | 说明 |
|--------|-----|--------|------|
| **P0** | C3 | 小 | RwLock unwrap → into_inner |
| **P0** | C4 | 小 | dev::alice → 随机生成 + 持久化 |
| **P0** | H1 | 小 | DashMap entry API |
| **P0** | H2 | 小 | engine leader 选举修正 |
| **P0** | H5 | 1 行 | action_type_name 默认值 |
| **P0** | H6 | 小 | 原子写入 |
| **P1** | C1 | 中 | GossipPayload 手动分派 |
| **P1** | C5 | 中 | 链客户端占位值 → warn 日志 |
| **P1** | M1-M12 | 各小 | 代码质量改进 |
| **P2** | C2 | 大 | Gossip 签名 (结构性) |
| **P2** | H3 | 大 | Gossip 网络转发 (结构性) |
| **P2** | H4 | 中 | endpoint 解析 |
| **P3** | L1-L7 | 各小 | 代码质量 + 测试 |
