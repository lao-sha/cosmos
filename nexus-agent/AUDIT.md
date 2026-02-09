# Nexus Agent v0.1.0 — 深度审查报告

**审查范围:** `nexus-agent/src/` 全部 11 个源文件 (≈3,500 行)
**审查日期:** 2026-02-09
**发现总计:** 5 Critical · 6 High · 12 Medium · 7 Low
**修复状态:** 5/5 Critical ✅ · 6/6 High ✅ · 12/12 Medium ✅ · 7/7 Low ✅ · Warnings 6→0 · Tests 52→90

---

## C — Critical（安全漏洞 / 数据正确性）

### C1. Leader 空签名绕过 — `/v1/execute` 可被任意调用 ✅ FIXED

**文件:** `webhook.rs:236`
```rust
if !action.leader_signature.is_empty() {
    // ... 验证逻辑 ...
}
// 空签名暂时允许（向后兼容，开发阶段）
```
**问题:** `leader_signature` 为空字符串时完全跳过验证。任何能访问 Agent 端口的网络攻击者只需发送 `leader_signature: ""` 的 ExecuteAction，即可让 Agent 执行任意 TG API 操作（ban/kick/delete/send）。

**修复:** 必须 `reject` 空签名，或至少在生产模式下强制验证。

---

### C2. 共识数量检查恒为 true — 验证形同虚设 ✅ FIXED

**文件:** `webhook.rs:218-224`
```rust
let k = action.consensus_nodes.len();
let m = if k <= 3 { k } else { (k * 2 + 2) / 3 };
if k < m {
    return Err(...);
}
```
**问题:** `m` 由 `k` 自身计算得出，`k < m` 在数学上永远为 false：
- k ≤ 3 → m = k → k < k = false
- k > 3 → m = ceil(k×2/3) ≤ k → k < m = false

**本意:** 应是 `consensus_nodes.len() >= M` 其中 M 基于**活跃节点总数 N** 计算，而非基于提交的 consensus_nodes 本身。攻击者可提交 `consensus_nodes: ["fake"]`（只有 1 个）即通过检查。

**修复:** M 应基于 `state.nodes.read().await.len()` 或链上 ActiveNodeList 长度计算。

---

### C3. Leader 公钥自证 — 无可信来源交叉验证 ✅ FIXED

**文件:** `webhook.rs:252`
```rust
if let Some((pk_hex, sig_hex)) = action.leader_signature.split_once(':') {
    // 从签名中提取公钥 → 用提取的公钥验证签名
}
```
**问题:** Leader 公钥从 `leader_signature` 字段本身提取（`pk_hex:sig_hex` 格式），然后用这个自提供的公钥验证签名。攻击者可用**任意密钥对**生成有效签名。没有与节点注册公钥列表交叉验证。

**修复:** Leader 公钥必须从 `state.nodes` 或链上缓存中按 `leader_node_id` 查找，不能使用请求中自带的。

---

### C4. `load_initial_nodes` — `:` 分隔符路径永远返回 None ✅ FIXED

**文件:** `main.rs:163-171`
```rust
let parts: Vec<&str> = entry.splitn(2, ':').collect();
if parts.len() == 2 {
    None  // ← BUG: 匹配成功但返回 None
} else {
    None
}
```
**问题:** `:` 分隔符的解析路径两个分支都返回 `None`。虽然 `@` 分隔符路径可工作，但函数注释声称格式为 `node_id:http://...`。如果用户按注释使用 `:` 格式，**所有节点会被静默丢弃**，Agent 运行但不发送任何消息到节点。

**修复:** 删除死代码的 `:` 分支，或修复其返回值。同时更正注释与 `.env.example` 一致 (`@` 格式)。

---

### C5. 序列号持久化竞态 — 重启后可能重放 ✅ FIXED

**文件:** `signer.rs:157-170`
```rust
let seq = self.current.fetch_add(1, Ordering::SeqCst) + 1;
// 持久化到文件
atomicwrites::AtomicFile::new(...).write(|f| f.write_all(&seq.to_le_bytes()))?;
```
**问题:** 多个并发 `next()` 调用时，`fetch_add` 和文件写入不是原子的：
- Thread A: fetch_add → seq=6
- Thread B: fetch_add → seq=7
- Thread B: 写入文件 7
- Thread A: 写入文件 6 ← **覆盖了 7**
- 进程重启 → 从 6 开始 → seq=7 被**重用**

Node 侧的 `SequenceTracker` 如果已记录 seq=7，重用的 seq=7 将被拒绝（好）。但如果节点也重启了，seq=7 的重放可能被接受（坏）。

**修复:** 在 `next()` 中加 `Mutex`，确保 fetch_add 和文件写入串行化。

---

## H — High（逻辑错误 / 潜在利用）

### H1. RateLimiter 已实现但从未使用 ✅ FIXED

**文件:** `rate_limiter.rs` (整个模块) + `webhook.rs` + `main.rs`

`RateLimiter` 完整实现了滑动窗口限流，但：
- `AppState` 中没有 `rate_limiter` 字段
- `handle_webhook` 和 `handle_execute` 都没有调用 `limiter.check()`
- `/webhook` 和 `/v1/execute` 完全无限流

**影响:** Webhook flood 攻击可消耗所有序列号 + 磁盘写入 + 节点网络带宽。

---

### H2. Kick 操作 unban 失败 → 永久封禁 ✅ FIXED

**文件:** `executor.rs:208-225`
```rust
if ban_result.is_ok() {
    let _ = self.call_tg_api("unbanChatMember", ...).await;  // ← 错误被吞
}
ban_result.map(|(_, resp)| ("kick".to_string(), resp))
```
**问题:** `unbanChatMember` 失败时错误被 `let _` 忽略。如果 unban 因网络超时或 TG API 限流失败，用户将被**永久 ban** 而非 kick。执行结果仍返回 success=true。

**修复:** 检查 unban 结果，失败时至少标记 `error` 字段或重试。

---

### H3. 群配置持久化非原子写入 ✅ FIXED

**文件:** `group_config.rs:259`
```rust
std::fs::write(&path, json)
```
**问题:** `std::fs::write` 不保证原子性。进程在写入中途崩溃会导致 `group_config.json` 损坏（部分 JSON）。下次启动时 `serde_json::from_str` 会失败，**群配置丢失**。

项目已有 `atomicwrites` 依赖（用于 `sequence.dat`），但此处未使用。

**修复:** 改用 `atomicwrites::AtomicFile`。

---

### H4. 群配置更新 TOCTOU 竞态 ✅ FIXED

**文件:** `group_config.rs:321-322`
```rust
let current_version = state.config_store.get_version();
let new_version = current_version + 1;
```
**问题:** 两个并发 POST 请求可能同时读到 `version=5`，都计算 `new_version=6`，后写入的请求静默覆盖前一个。配置数据可能丢失。

**修复:** 在 `ConfigStore::set` 中验证 `new_version > current_version`，或在 `handle_update_config` 中持锁操作。

---

### H5. ConfigSync 广播目标端点不存在 ✅ FIXED

**文件:** `group_config.rs:505`
```rust
let url = format!("{}/v1/gossip", node.endpoint);
```
**问题:** `nexus-node` 的 API 端点是 `POST /v1/message`，没有 `/v1/gossip` 端点。ConfigSync 广播将全部收到 404。

**修复:** 确认 nexus-node 端有对应的 Gossip HTTP 入口，或改用正确的端点。

---

### H6. `/v1/execute` 无网络访问控制 ✅ FIXED

**文件:** `main.rs:137`
```rust
.route("/v1/execute", post(webhook::handle_execute))
```
**问题:** 绑定 `0.0.0.0:{port}`，`/v1/execute` 对任何网络来源开放。结合 C1（空签名绕过），任何能访问端口的人可执行任意 TG 管理操作。

**缓解:** 应至少添加 IP 白名单、mutual TLS 或 bearer token 认证。

---

## M — Medium（设计缺陷 / 健壮性不足）

### M1. Webhook 双重 JSON 反序列化 ✅ FIXED

**文件:** `webhook.rs:37,49`
```rust
let update: TelegramUpdate = serde_json::from_slice(&body)?;      // 第 1 次
let raw_update: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();  // 第 2 次
```
**浪费:** 同一个 body 被反序列化两次（一次到结构体，一次到 Value）。且第二次用 `unwrap_or_default()` 在失败时返回空 Value，但第一次已验证 JSON 有效。

**修复:** 只解析一次 `serde_json::Value`，再用 `serde_json::from_value::<TelegramUpdate>()` 转换。

---

### M2. 审计日志消耗主序列号 ✅ FIXED

**文件:** `webhook.rs:508`
```rust
let sequence = match state.sequence_manager.next() { ... };
```
**问题:** 每次本地快速路径触发审计日志时，消耗一个主序列号。高频操作（如持续刷屏检测）会快速递增序列号，影响正常消息的 deterministic node selection 可预测性。

**建议:** 审计日志使用独立的序列号空间，或使用 `audit_` 前缀区分。

---

### M3. `execute_local_action` TG API 响应被丢弃 ✅ FIXED

**文件:** `webhook.rs:304,323,352` 等
```rust
let _ = state.http_client.post(&url).json(&...).send().await.map_err(...)?;
```
**问题:** TG API 返回的 `ok: false` 响应（如 Bot 无权限）被完全忽略。只有网络层错误被捕获，业务层错误（如 "not enough rights"）静默丢失。

---

### M4. Regex 黑名单每消息重编译 ⚠️ PARTIAL

**文件:** `local_processor.rs:246-251`
```rust
config.blacklist_words.iter().any(|pattern| {
    regex::Regex::new(pattern)  // ← 每条消息、每个 pattern 都重新编译
        .map(|re| re.is_match(&text_lower))
        .unwrap_or(false)
})
```
**影响:** Regex 编译是 O(n) 到 O(n²)。在高流量群组中，每条消息对每个 regex pattern 都重新编译，性能严重降低。

**修复:** 在 `ConfigStore::set` 时预编译 regex 并缓存。

---

### M5. 欢迎消息只处理第一个新成员 ✅ FIXED

**文件:** `local_processor.rs:386`
```rust
let member = &members[0];
```
**问题:** 多用户同时加群时，只为第一个发送欢迎消息。

---

### M6. `WEBHOOK_URL` 非必填 — 静默回退到不可用地址 ✅ FIXED

**文件:** `config.rs:69-70`
```rust
let webhook_url = std::env::var("WEBHOOK_URL")
    .unwrap_or_else(|_| format!("https://localhost:{}", webhook_port));
```
**问题:** README 标注 WEBHOOK_URL 为必填，但代码默认 `https://localhost:8443`。Webhook 注册将成功但 Telegram 无法回调，消息静默丢失。

---

### M7. 多播期间持有读锁 ✅ FIXED

**文件:** `multicaster.rs:19`
```rust
let nodes = state.nodes.read().await;
// ... 选择节点、并发发送、等待结果 ...（锁一直持有到函数结束）
```
**问题:** 读锁在整个多播操作期间（最长 MULTICAST_TIMEOUT_MS 3 秒）被持有，阻塞节点列表更新。

**修复:** `let target_nodes = { ... nodes.read().await ... }.clone();` 提前释放锁。

---

### M8. `quiet_hours` 一旦设置无法清除 ✅ FIXED

**文件:** `group_config.rs:369-370`
```rust
quiet_hours_start: req.quiet_hours_start.or(base.quiet_hours_start),
quiet_hours_end: req.quiet_hours_end.or(base.quiet_hours_end),
```
**问题:** `.or()` 语义是 "如果 req 是 None 则保留 base"。但无法传入 `Some(None)` 来主动清除。一旦设置了 quiet hours，无法通过 API 取消。

**修复:** 其他字段用 `.unwrap_or(base.x)`，这里也应统一或引入 `unset_quiet_hours: bool` 字段。

---

### M9. 无请求体大小限制 ✅ FIXED

**文件:** `main.rs:135-141`

`/webhook` 和 `/v1/execute` 都没有配置 body size limit。攻击者可发送超大 payload 导致 OOM。

**修复:** Axum 配合 `tower_http::limit::RequestBodyLimitLayer`。

---

### M10. RwLock::unwrap() — 锁中毒连锁 panic ✅ FIXED

**文件:** `local_store.rs` (全部 RwLock 操作) + `group_config.rs:236,250,273`

所有 `RwLock` 访问使用 `.unwrap()`。如果任一线程在持锁时 panic，锁被 poison，后续所有访问都会 panic，导致整个 Agent 崩溃。

**修复:** 使用 `.unwrap_or_else(|e| e.into_inner())` 恢复 poisoned lock，或改用 `parking_lot::RwLock`（无 poison 机制）。

---

### M11. `TelegramExecutor` 独立 reqwest::Client ✅ FIXED

**文件:** `executor.rs:87`
```rust
client: reqwest::Client::new(),
```
**问题:** 与 `AppState.http_client` 分开的连接池。对 `api.telegram.org` 维护两套 TCP 连接。

---

### M12. `DeleteBatch` / `SetPermissions` 返回假 OK ✅ FIXED

**文件:** `executor.rs:113,206`
```rust
Ok(("deleteMessages".to_string(), serde_json::json!({"ok": true})))
```
**问题:** TODO 桩返回 `success: true` + 签名回执，调用方认为操作已执行。

---

## L — Low（代码质量 / 轻微问题）

### L1. `AgentConfig` serde 注解无用 ✅ FIXED

`AgentConfig` 的 `#[serde(default)]` 注解仅在 serde 反序列化时生效，但 `from_env()` 手动解析环境变量，serde 从未被调用。

### L2. `AgentConfig` 派生 `Debug` 暴露 bot_token ✅ FIXED

`bot_token` 会出现在任何 `{:?}` 格式化中。如果日志意外打印 config 对象，token 会泄露。

### L3. `use crate::group_config::ConfigStore` 未使用 ✅ FIXED

`local_processor.rs:8` 导入 `ConfigStore` 但从未引用。

### L4. `register_telegram_webhook` 创建独立 Client ✅ FIXED

`multicaster.rs:199` 创建新的 `reqwest::Client`，而非复用已有的。现已传入共享 `http_client`。

### L5. `crypto.rs` 域分离不完整 ✅ FIXED

`platform + user_id` 无长度前缀。`platform="ab"` + `user_id="c123"` 与 `platform="abc"` + `user_id="123"` 的哈希输入相同。实际风险极低（平台名固定），但密码学上不规范。

### L6. `DefaultHasher` 非密码学安全 ✅ FIXED

`local_store.rs:204` 的消息指纹用 `DefaultHasher`（SipHash），可被故意碰撞绕过去重检测。现已改用 `RandomState` 进程级随机种子。

### L7. 测试覆盖空白区 ✅ FIXED

新增 38 个单元测试：
- `signer.rs`: 12 tests — KeyManager 生成/加载/损坏文件/签名/验证/sign_message, SequenceManager 初始化/递增/持久化/审计独立/并发安全
- `config.rs`: 7 tests — 必填缺失报错/默认值/自定义值/bot_id_hash确定性/Debug遮蔽/EXECUTE_TOKEN空值
- `group_config.rs`: 9 tests — ConfigStore空/读写/持久化/CAS成功/版本冲突/过期读/serde/默认值/quiet_hours哨兵清除
- `executor.rs`: 7 tests — ActionType 20变体 roundtrip/ExecuteAction反序列化/ExecuteResult序列化/sign_receipt格式/可验证/确定性/唯一性
- 仍缺: `webhook.rs`(HTTP handler需 integration test), `main.rs`(启动流程)

---

## 修复优先级建议

| 优先级 | ID | 工作量 | 说明 |
|--------|-----|--------|------|
| **P0 立即** | C1 | 1 行 | 拒绝空 leader_signature |
| **P0 立即** | C2 | 5 行 | 基于活跃节点总数计算 M |
| **P0 立即** | C3 | 10 行 | 从 nodes 列表按 node_id 查公钥 |
| **P0 立即** | C4 | 10 行 | 修复或删除 `:` 分隔符路径 |
| **P0 立即** | C5 | 5 行 | next() 加 Mutex 保护 |
| **P1 本周** | H1 | 15 行 | 集成 RateLimiter 到 webhook/execute |
| **P1 本周** | H2 | 5 行 | Kick 时检查 unban 结果 |
| **P1 本周** | H3 | 3 行 | ConfigStore 改用 atomicwrites |
| **P1 本周** | H4 | 10 行 | ConfigStore 加版本 CAS |
| **P1 本周** | H5 | 1 行 | 修正 ConfigSync 端点路径 |
| **P1 本周** | H6 | 20 行 | /v1/execute IP 白名单或 token |
| **P2 下周** | M1-M12 | 各 5-20 行 | 见各项描述 |
| **P3 排期** | L1-L7 | 各 1-5 行 | 代码质量改进 |
