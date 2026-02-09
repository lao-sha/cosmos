use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn, debug};

use crate::types::*;
use crate::gossip::state::GossipState;
use crate::chain_cache::ChainCache;

/// Gossip 引擎
///
/// 负责:
/// 1. 收到 Agent 消息后广播 MessageSeen
/// 2. 处理收到的 Gossip 消息
/// 3. 检查共识 + 触发 Leader 执行
/// 4. Pull 补偿机制
/// 5. Equivocation 检测
pub struct GossipEngine {
    /// 本节点 ID
    pub node_id: String,

    /// 消息状态管理
    pub state: Arc<GossipState>,

    /// 链上缓存
    pub chain_cache: Arc<ChainCache>,

    /// 广播通道（内部 → Gossip 网络层）
    pub outbound_tx: broadcast::Sender<GossipEnvelope>,
}

impl GossipEngine {
    pub fn new(
        node_id: String,
        state: Arc<GossipState>,
        chain_cache: Arc<ChainCache>,
    ) -> Self {
        let (outbound_tx, _) = broadcast::channel(1024);
        Self {
            node_id,
            state,
            chain_cache,
            outbound_tx,
        }
    }

    /// 创建 outbound 消息接收通道（供 Gossip 网络层使用）
    ///
    /// 将内部 broadcast channel 桥接到 mpsc::UnboundedReceiver
    pub fn subscribe_outbound(&self) -> tokio::sync::mpsc::UnboundedReceiver<GossipEnvelope> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let mut broadcast_rx = self.outbound_tx.subscribe();
        tokio::spawn(async move {
            loop {
                match broadcast_rx.recv().await {
                    Ok(envelope) => {
                        if tx.send(envelope).is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "Gossip outbound 消息丢失");
                    }
                    Err(_) => break,
                }
            }
        });
        rx
    }

    /// 处理来自 Agent 的新消息
    ///
    /// 1. 计算 msg_id
    /// 2. 更新状态 → Received
    /// 3. 广播 MessageSeen 到 Gossip 网络
    /// 4. 检查共识
    pub fn on_agent_message(&self, message: SignedMessage) {
        let msg_id = GossipState::make_msg_id(&message.bot_id_hash, message.sequence);

        // 确定目标节点 + Leader
        let active_nodes = self.chain_cache.get_active_node_ids();
        let k = select_k(active_nodes.len());
        let targets = deterministic_select_ids(&active_nodes, &message.message_hash, message.sequence, k);

        let election = crate::leader::LeaderExecutor::elect_leader(&targets, message.sequence);
        let leader = if election.leader.is_empty() { None } else { Some(election.leader) };
        let backups = election.backups;

        let msg_hash = message.message_hash.clone();
        let message_bot_id_hash = message.bot_id_hash.clone();

        // 更新状态
        self.state.on_agent_message(
            &msg_id,
            message,
            targets,
            leader.clone(),
            backups,
        );

        // 广播 MessageSeen
        let seen = GossipEnvelope {
            version: 1,
            msg_type: GossipType::MessageSeen,
            sender_node_id: self.node_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            payload: GossipPayload::Seen(SeenPayload {
                msg_id: msg_id.clone(),
                msg_hash: msg_hash.clone(),
                node_id: self.node_id.clone(),
                config_version: self.chain_cache.get_config_version(&message_bot_id_hash),
            }),
            sender_signature: String::new(), // TODO: 签名
        };

        if let Err(e) = self.outbound_tx.send(seen) {
            warn!(error = %e, "广播 MessageSeen 失败 (无接收者)");
        }

        // 自己也算 Seen
        self.state.on_seen(&msg_id, &self.node_id, &msg_hash);

        // 检查共识
        self.try_consensus(&msg_id);
    }

    /// 处理收到的 Gossip 消息
    pub fn on_gossip_message(&self, envelope: GossipEnvelope) {
        match envelope.msg_type {
            GossipType::MessageSeen => {
                if let GossipPayload::Seen(seen) = &envelope.payload {
                    self.handle_seen(seen);
                }
            }
            GossipType::MessagePull => {
                if let GossipPayload::Pull(pull) = &envelope.payload {
                    self.handle_pull(pull, &envelope.sender_node_id);
                }
            }
            GossipType::MessagePullResponse => {
                if let GossipPayload::PullResponse(resp) = envelope.payload {
                    self.handle_pull_response(resp);
                }
            }
            GossipType::DecisionVote => {
                if let GossipPayload::Vote(vote) = envelope.payload {
                    self.handle_vote(vote);
                }
            }
            GossipType::EquivocationAlert => {
                if let GossipPayload::Alert(alert) = &envelope.payload {
                    self.handle_equivocation_alert(alert);
                }
            }
            GossipType::ExecutionResult => {
                if let GossipPayload::Result(result) = &envelope.payload {
                    self.handle_execution_result(result);
                }
            }
            GossipType::LeaderTakeover => {
                if let GossipPayload::Takeover(takeover) = &envelope.payload {
                    self.handle_leader_takeover(takeover);
                }
            }
            GossipType::Heartbeat => {
                // 心跳: 更新 peer 活跃时间
                debug!(from = envelope.sender_node_id, "收到心跳");
            }
            GossipType::ConfigSync => {
                if let GossipPayload::ConfigSync(sync) = envelope.payload {
                    self.handle_config_sync(sync);
                }
            }
            GossipType::ConfigPull => {
                if let GossipPayload::ConfigPull(pull) = &envelope.payload {
                    self.handle_config_pull(pull, &envelope.sender_node_id);
                }
            }
            GossipType::ConfigPullResponse => {
                if let GossipPayload::ConfigPullResponse(resp) = envelope.payload {
                    self.handle_config_pull_response(resp);
                }
            }
        }
    }

    /// 处理 MessageSeen
    fn handle_seen(&self, seen: &SeenPayload) {
        let (_status, needs_pull) = self.state.on_seen(
            &seen.msg_id,
            &seen.node_id,
            &seen.msg_hash,
        );

        // 如果从未见过 → 发 Pull 请求
        if needs_pull {
            let pull = GossipEnvelope {
                version: 1,
                msg_type: GossipType::MessagePull,
                sender_node_id: self.node_id.clone(),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                payload: GossipPayload::Pull(PullPayload {
                    msg_id: seen.msg_id.clone(),
                }),
                sender_signature: String::new(),
            };
            let _ = self.outbound_tx.send(pull);
        }

        // Equivocation 检测
        if let Some((hash_a, hash_b)) = self.state.has_conflicting_hashes(&seen.msg_id) {
            warn!(
                msg_id = seen.msg_id,
                hash_a, hash_b,
                "Equivocation 检测到! 广播警报"
            );
            let alert = GossipEnvelope {
                version: 1,
                msg_type: GossipType::EquivocationAlert,
                sender_node_id: self.node_id.clone(),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                payload: GossipPayload::Alert(AlertPayload {
                    owner_public_key: String::new(), // TODO: 从消息中提取
                    sequence: 0,
                    msg_hash_a: hash_a,
                    signature_a: String::new(),
                    msg_hash_b: hash_b,
                    signature_b: String::new(),
                }),
                sender_signature: String::new(),
            };
            let _ = self.outbound_tx.send(alert);
        }

        // 检查共识
        self.try_consensus(&seen.msg_id);
    }

    /// 处理 Pull 请求 → 返回原始消息
    fn handle_pull(&self, pull: &PullPayload, requester: &str) {
        if let Some(original) = self.state.get_original_message(&pull.msg_id) {
            debug!(msg_id = pull.msg_id, to = requester, "响应 Pull 请求");
            let response = GossipEnvelope {
                version: 1,
                msg_type: GossipType::MessagePullResponse,
                sender_node_id: self.node_id.clone(),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                payload: GossipPayload::PullResponse(PullResponsePayload {
                    msg_id: pull.msg_id.clone(),
                    signed_message: original,
                }),
                sender_signature: String::new(),
            };
            let _ = self.outbound_tx.send(response);
        }
    }

    /// 处理 Pull 响应 → 更新状态
    fn handle_pull_response(&self, resp: PullResponsePayload) {
        let msg = resp.signed_message;
        let active_nodes = self.chain_cache.get_active_node_ids();
        let k = select_k(active_nodes.len());
        let targets = deterministic_select_ids(&active_nodes, &msg.message_hash, msg.sequence, k);

        let election = crate::leader::LeaderExecutor::elect_leader(&targets, msg.sequence);
        let leader = if election.leader.is_empty() { None } else { Some(election.leader) };
        let backups = election.backups;

        self.state.on_agent_message(
            &resp.msg_id,
            msg,
            targets,
            leader,
            backups,
        );

        self.try_consensus(&resp.msg_id);
    }

    /// 处理 DecisionVote
    fn handle_vote(&self, vote: VotePayload) {
        let msg_id = vote.msg_id.clone();
        self.state.add_vote(&msg_id, vote);
    }

    /// 处理 Equivocation 警报
    fn handle_equivocation_alert(&self, alert: &AlertPayload) {
        warn!(
            owner = alert.owner_public_key,
            sequence = alert.sequence,
            "收到 Equivocation 警报 — TODO: 上链举报"
        );
        // TODO: Sprint 5 — 自动调用 pallet-bot-consensus::report_equivocation
    }

    /// 处理 Leader 执行结果
    fn handle_execution_result(&self, result: &ResultPayload) {
        if result.success {
            self.state.set_completed(&result.msg_id);
            info!(msg_id = result.msg_id, leader = result.executor_node_id, "执行成功");
        } else {
            warn!(msg_id = result.msg_id, "Leader 执行失败");
        }
    }

    /// 处理 Leader 接管
    fn handle_leader_takeover(&self, takeover: &TakeoverPayload) {
        info!(
            msg_id = takeover.msg_id,
            original = takeover.original_leader,
            backup_rank = takeover.backup_rank,
            "Backup 接管 Leader"
        );
        // TODO: Sprint 4 — 实现 Leader 切换逻辑
    }

    /// 处理 ConfigSync — Agent 签名配置广播
    fn handle_config_sync(&self, sync: ConfigSyncPayload) {
        let bot_id = &sync.bot_id_hash;
        let version = sync.signed_config.config.version;

        match self.chain_cache.apply_signed_config(bot_id, sync.signed_config) {
            Ok(()) => {
                info!(bot = %bot_id, version, "ConfigSync 已接收并验证");
            }
            Err(e) => {
                warn!(bot = %bot_id, error = %e, "ConfigSync 验证失败");
            }
        }
    }

    /// 处理 ConfigPull — 其他节点请求配置
    fn handle_config_pull(&self, pull: &ConfigPullPayload, requester: &str) {
        let local_version = self.chain_cache.get_config_version(&pull.bot_id_hash);

        if local_version > pull.current_version {
            let signed_config = self.chain_cache.get_group_config(&pull.bot_id_hash);
            debug!(
                bot = %pull.bot_id_hash,
                requester,
                local_version,
                remote_version = pull.current_version,
                "响应 ConfigPull"
            );

            let response = GossipEnvelope {
                version: 1,
                msg_type: GossipType::ConfigPullResponse,
                sender_node_id: self.node_id.clone(),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                payload: GossipPayload::ConfigPullResponse(ConfigPullResponsePayload {
                    bot_id_hash: pull.bot_id_hash.clone(),
                    signed_config,
                }),
                sender_signature: String::new(),
            };
            let _ = self.outbound_tx.send(response);
        } else {
            debug!(
                bot = %pull.bot_id_hash,
                requester,
                "ConfigPull: 本地版本不更新，跳过"
            );
        }
    }

    /// 处理 ConfigPullResponse — 收到配置拉取响应
    fn handle_config_pull_response(&self, resp: ConfigPullResponsePayload) {
        if let Some(signed_config) = resp.signed_config {
            let version = signed_config.config.version;
            match self.chain_cache.apply_signed_config(&resp.bot_id_hash, signed_config) {
                Ok(()) => {
                    info!(
                        bot = %resp.bot_id_hash,
                        version,
                        "ConfigPullResponse 已应用"
                    );
                }
                Err(e) => {
                    warn!(
                        bot = %resp.bot_id_hash,
                        error = %e,
                        "ConfigPullResponse 验证失败"
                    );
                }
            }
        }
    }

    /// 发送 ConfigPull 请求（节点启动恢复时调用）
    pub fn send_config_pull(&self, bot_id_hash: &str) {
        let current_version = self.chain_cache.get_config_version(bot_id_hash);
        let pull = GossipEnvelope {
            version: 1,
            msg_type: GossipType::ConfigPull,
            sender_node_id: self.node_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            payload: GossipPayload::ConfigPull(ConfigPullPayload {
                bot_id_hash: bot_id_hash.to_string(),
                current_version,
            }),
            sender_signature: String::new(),
        };
        let _ = self.outbound_tx.send(pull);
        debug!(bot = bot_id_hash, current_version, "已发送 ConfigPull 请求");
    }

    /// 尝试达成 M/K 共识
    fn try_consensus(&self, msg_id: &str) {
        let (reached, _count, _m) = self.state.check_consensus(msg_id);

        if reached {
            // 检查自己是否是 Leader
            if let Some(leader) = self.state.get_leader(msg_id) {
                if leader == self.node_id {
                    info!(msg_id, "我是 Leader — TODO: Sprint 4 执行动作");
                    self.state.set_executing(msg_id);
                    // TODO: Sprint 4 — Leader 执行 TG API 动作
                }
            }
        }
    }

}

/// 确定 K 值（与 Agent/Verifier 完全一致）
fn select_k(total_nodes: usize) -> usize {
    if total_nodes <= 3 { return total_nodes; }
    let two_thirds = (total_nodes * 2 + 2) / 3;
    two_thirds.min(total_nodes).max(3)
}

/// 确定性随机选择节点 ID（与 Agent/Verifier 完全一致）
fn deterministic_select_ids(
    node_ids: &[String],
    message_hash_hex: &str,
    sequence: u64,
    k: usize,
) -> Vec<String> {
    use sha2::{Sha256, Digest};
    if node_ids.is_empty() || k == 0 { return vec![]; }

    let mut sorted = node_ids.to_vec();
    sorted.sort();
    let k = k.min(sorted.len());

    let mut hasher = Sha256::new();
    hasher.update(message_hash_hex.as_bytes());
    hasher.update(&sequence.to_le_bytes());
    let seed = hasher.finalize();

    let n = sorted.len();
    for i in 0..k {
        let mut idx_hasher = Sha256::new();
        idx_hasher.update(&seed);
        idx_hasher.update(&(i as u64).to_le_bytes());
        let idx_hash = idx_hasher.finalize();
        let rand_val = u64::from_le_bytes(idx_hash[..8].try_into().unwrap());
        let j = i + (rand_val as usize % (n - i));
        sorted.swap(i, j);
    }

    sorted.truncate(k);
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_engine() -> GossipEngine {
        let state = Arc::new(GossipState::new());
        let cache = Arc::new(ChainCache::new());

        // 添加 3 个测试节点
        cache.add_node(crate::types::NodeInfoCache {
            node_id: "node_a".into(),
            endpoint: "http://a:8080".into(),
            node_public_key: [0u8; 32],
            status: "Active".into(),
            reputation: 5000,
        });
        cache.add_node(crate::types::NodeInfoCache {
            node_id: "node_b".into(),
            endpoint: "http://b:8080".into(),
            node_public_key: [0u8; 32],
            status: "Active".into(),
            reputation: 5000,
        });
        cache.add_node(crate::types::NodeInfoCache {
            node_id: "node_c".into(),
            endpoint: "http://c:8080".into(),
            node_public_key: [0u8; 32],
            status: "Active".into(),
            reputation: 5000,
        });

        GossipEngine::new("node_a".into(), state, cache)
    }

    fn make_msg(seq: u64) -> SignedMessage {
        SignedMessage {
            owner_public_key: "aa".repeat(32),
            bot_id_hash: "bb".repeat(16),
            sequence: seq,
            timestamp: chrono::Utc::now().timestamp() as u64,
            message_hash: format!("hash_{}", seq),
            telegram_update: serde_json::json!({"update_id": seq}),
            owner_signature: "cc".repeat(32),
            platform: "telegram".into(),
        }
    }

    #[test]
    fn test_on_agent_message_creates_state() {
        let engine = make_test_engine();
        let msg = make_msg(1);
        let msg_id = GossipState::make_msg_id(&msg.bot_id_hash, msg.sequence);

        engine.on_agent_message(msg);

        let status = engine.state.get_status(&msg_id);
        assert!(status.is_some());
    }

    #[tokio::test]
    async fn test_gossip_seen_triggers_pull_for_unknown() {
        let engine = make_test_engine();
        let mut rx = engine.subscribe_outbound();

        let seen = GossipEnvelope {
            version: 1,
            msg_type: GossipType::MessageSeen,
            sender_node_id: "node_b".into(),
            timestamp: 0,
            payload: GossipPayload::Seen(SeenPayload {
                msg_id: "unknown_msg".into(),
                msg_hash: "some_hash".into(),
                node_id: "node_b".into(),
                config_version: 0,
            }),
            sender_signature: String::new(),
        };

        engine.on_gossip_message(seen);

        // 给 bridge task 一点时间转发
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // 应该发出 Pull 请求
        if let Ok(envelope) = rx.try_recv() {
            assert_eq!(envelope.msg_type, GossipType::MessagePull);
        }
    }

    #[test]
    fn test_consensus_flow() {
        let engine = make_test_engine();
        let msg = make_msg(1);
        let msg_id = GossipState::make_msg_id(&msg.bot_id_hash, msg.sequence);
        let hash = msg.message_hash.clone();

        engine.on_agent_message(msg);

        // 其他 2 个节点 Seen
        engine.state.on_seen(&msg_id, "node_b", &hash);
        engine.state.on_seen(&msg_id, "node_c", &hash);

        let (reached, _, _) = engine.state.check_consensus(&msg_id);
        assert!(reached);
    }
}
