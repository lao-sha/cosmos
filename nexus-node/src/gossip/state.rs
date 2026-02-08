use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{info, debug, warn};

use crate::types::{MessageState, MessageStatus, SeenRecord, SignedMessage, VotePayload};

/// Gossip 全局状态管理
///
/// 管理所有消息的状态机，线程安全
pub struct GossipState {
    /// 消息状态: msg_id → MessageState
    messages: DashMap<String, MessageState>,

    /// 已处理消息计数
    pub messages_processed: AtomicU64,
}

impl GossipState {
    pub fn new() -> Self {
        Self {
            messages: DashMap::new(),
            messages_processed: AtomicU64::new(0),
        }
    }

    /// 生成消息 ID: bot_id_hash[..16] + "_" + sequence
    pub fn make_msg_id(bot_id_hash: &str, sequence: u64) -> String {
        let prefix = if bot_id_hash.len() >= 16 {
            &bot_id_hash[..16]
        } else {
            bot_id_hash
        };
        format!("{}_{}", prefix, sequence)
    }

    /// 收到 Agent 原始消息 → 创建或更新状态
    pub fn on_agent_message(
        &self,
        msg_id: &str,
        message: SignedMessage,
        target_nodes: Vec<String>,
        leader: Option<String>,
        backups: Vec<String>,
    ) -> MessageStatus {
        let now = chrono::Utc::now().timestamp_millis() as u64;

        if let Some(mut entry) = self.messages.get_mut(msg_id) {
            // 已通过 Seen 得知，现在收到原始消息
            if entry.status == MessageStatus::HeardViaSeen {
                entry.original_message = Some(message);
                entry.status = MessageStatus::Received;
                entry.target_nodes = target_nodes;
                entry.leader = leader;
                entry.backups = backups;
                debug!(msg_id, "消息状态: HeardViaSeen → Received");
            }
            entry.status
        } else {
            // 全新消息
            let state = MessageState {
                msg_id: msg_id.to_string(),
                status: MessageStatus::Received,
                original_message: Some(message),
                seen_nodes: std::collections::HashMap::new(),
                target_nodes,
                decision_votes: Vec::new(),
                leader,
                backups,
                created_at: now,
                pull_timer_started: false,
            };
            self.messages.insert(msg_id.to_string(), state);
            debug!(msg_id, "新消息: → Received");
            MessageStatus::Received
        }
    }

    /// 收到 Gossip Seen → 记录 + 返回是否需要 Pull
    pub fn on_seen(
        &self,
        msg_id: &str,
        node_id: &str,
        msg_hash: &str,
    ) -> (MessageStatus, bool) {
        let now = chrono::Utc::now().timestamp_millis() as u64;

        if let Some(mut entry) = self.messages.get_mut(msg_id) {
            // 记录 Seen
            entry.seen_nodes.insert(node_id.to_string(), SeenRecord {
                node_id: node_id.to_string(),
                msg_hash: msg_hash.to_string(),
                seen_at: now,
            });

            (entry.status, false)
        } else {
            // 从未见过此消息 → 创建 HeardViaSeen + 需要 Pull
            let mut seen_nodes = std::collections::HashMap::new();
            seen_nodes.insert(node_id.to_string(), SeenRecord {
                node_id: node_id.to_string(),
                msg_hash: msg_hash.to_string(),
                seen_at: now,
            });

            let state = MessageState {
                msg_id: msg_id.to_string(),
                status: MessageStatus::HeardViaSeen,
                original_message: None,
                seen_nodes,
                target_nodes: Vec::new(),
                decision_votes: Vec::new(),
                leader: None,
                backups: Vec::new(),
                created_at: now,
                pull_timer_started: false,
            };
            self.messages.insert(msg_id.to_string(), state);
            debug!(msg_id, from = node_id, "新消息通过 Seen 得知 → HeardViaSeen (需要 Pull)");

            (MessageStatus::HeardViaSeen, true)
        }
    }

    /// 检查 M/K 共识是否达成
    ///
    /// M = ceil(K * 2/3)
    /// 返回 (是否达成, 一致节点数, 需要的最小数)
    pub fn check_consensus(&self, msg_id: &str) -> (bool, usize, usize) {
        if let Some(mut entry) = self.messages.get_mut(msg_id) {
            if entry.status == MessageStatus::Confirmed
                || entry.status == MessageStatus::Executing
                || entry.status == MessageStatus::Completed
            {
                return (true, 0, 0);
            }

            let k = entry.target_nodes.len();
            if k == 0 {
                return (false, 0, 0);
            }

            let m = (k * 2 + 2) / 3; // ceil(K * 2/3)

            // 获取主要 msg_hash（出现最多的）
            let original_hash = entry.original_message.as_ref()
                .map(|m| m.message_hash.clone());

            let consistent_count = if let Some(ref hash) = original_hash {
                entry.seen_nodes.values()
                    .filter(|s| s.msg_hash == *hash)
                    .count()
                    + 1 // 加上自己（自己也看到了原始消息）
            } else {
                // 没有原始消息，只看 Seen 中最多的 hash
                let mut hash_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
                for seen in entry.seen_nodes.values() {
                    *hash_counts.entry(&seen.msg_hash).or_insert(0) += 1;
                }
                hash_counts.values().max().copied().unwrap_or(0)
            };

            if consistent_count >= m {
                entry.status = MessageStatus::Confirmed;
                self.messages_processed.fetch_add(1, Ordering::Relaxed);
                info!(msg_id, consistent_count, m, k, "M/K 共识达成 → Confirmed");
                (true, consistent_count, m)
            } else {
                debug!(msg_id, consistent_count, m, k, "共识未达成");
                (false, consistent_count, m)
            }
        } else {
            (false, 0, 0)
        }
    }

    /// 检测 Equivocation（同一消息不同哈希）
    pub fn has_conflicting_hashes(&self, msg_id: &str) -> Option<(String, String)> {
        if let Some(entry) = self.messages.get(msg_id) {
            let hashes: std::collections::HashSet<&str> = entry.seen_nodes.values()
                .map(|s| s.msg_hash.as_str())
                .collect();

            if hashes.len() > 1 {
                let mut iter = hashes.into_iter();
                let a = iter.next().unwrap().to_string();
                let b = iter.next().unwrap().to_string();
                warn!(msg_id, hash_a = a, hash_b = b, "Equivocation 检测到!");
                return Some((a, b));
            }
        }
        None
    }

    /// 获取消息状态
    pub fn get_status(&self, msg_id: &str) -> Option<MessageStatus> {
        self.messages.get(msg_id).map(|e| e.status)
    }

    /// 获取原始消息（Pull 响应用）
    pub fn get_original_message(&self, msg_id: &str) -> Option<SignedMessage> {
        self.messages.get(msg_id)
            .and_then(|e| e.original_message.clone())
    }

    /// 设置状态为 Executing
    pub fn set_executing(&self, msg_id: &str) {
        if let Some(mut entry) = self.messages.get_mut(msg_id) {
            entry.status = MessageStatus::Executing;
        }
    }

    /// 设置状态为 Completed
    pub fn set_completed(&self, msg_id: &str) {
        if let Some(mut entry) = self.messages.get_mut(msg_id) {
            entry.status = MessageStatus::Completed;
        }
    }

    /// 设置状态为 Failed
    pub fn set_failed(&self, msg_id: &str) {
        if let Some(mut entry) = self.messages.get_mut(msg_id) {
            entry.status = MessageStatus::Failed;
        }
    }

    /// 添加 DecisionVote
    pub fn add_vote(&self, msg_id: &str, vote: VotePayload) {
        if let Some(mut entry) = self.messages.get_mut(msg_id) {
            entry.decision_votes.push(vote);
        }
    }

    /// 获取消息的 Leader
    pub fn get_leader(&self, msg_id: &str) -> Option<String> {
        self.messages.get(msg_id).and_then(|e| e.leader.clone())
    }

    /// 设置 Pull 定时器已启动
    pub fn set_pull_timer_started(&self, msg_id: &str) {
        if let Some(mut entry) = self.messages.get_mut(msg_id) {
            entry.pull_timer_started = true;
        }
    }

    /// 清理过期消息 (>60s)
    pub fn gc_expired_messages(&self) {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let expired_threshold = 60_000; // 60 seconds

        self.messages.retain(|_, state| {
            now.saturating_sub(state.created_at) < expired_threshold
        });
    }

    /// 获取活跃消息数
    pub fn active_message_count(&self) -> usize {
        self.messages.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signed_msg(hash: &str, seq: u64) -> SignedMessage {
        SignedMessage {
            owner_public_key: "aa".repeat(32),
            bot_id_hash: "bb".repeat(16),
            sequence: seq,
            timestamp: 1700000000,
            message_hash: hash.to_string(),
            telegram_update: serde_json::json!({}),
            owner_signature: "cc".repeat(32),
            platform: "telegram".to_string(),
        }
    }

    #[test]
    fn test_make_msg_id() {
        let id = GossipState::make_msg_id("abcdef1234567890aabbccdd", 42);
        assert_eq!(id, "abcdef1234567890_42");
    }

    #[test]
    fn test_on_agent_message_creates_received() {
        let state = GossipState::new();
        let msg = make_signed_msg("hash1", 1);

        let status = state.on_agent_message(
            "msg_1", msg,
            vec!["n1".into(), "n2".into(), "n3".into()],
            Some("n1".into()),
            vec!["n2".into(), "n3".into()],
        );

        assert_eq!(status, MessageStatus::Received);
        assert_eq!(state.get_status("msg_1"), Some(MessageStatus::Received));
    }

    #[test]
    fn test_seen_then_message_flow() {
        let state = GossipState::new();

        // 先通过 Seen 得知
        let (status, needs_pull) = state.on_seen("msg_1", "node_a", "hash1");
        assert_eq!(status, MessageStatus::HeardViaSeen);
        assert!(needs_pull);

        // 再收到原始消息
        let msg = make_signed_msg("hash1", 1);
        let status = state.on_agent_message(
            "msg_1", msg,
            vec!["n1".into(), "n2".into(), "n3".into()],
            Some("n1".into()),
            vec![],
        );
        assert_eq!(status, MessageStatus::Received);
    }

    #[test]
    fn test_consensus_3_of_3() {
        let state = GossipState::new();
        let msg = make_signed_msg("hash1", 1);

        state.on_agent_message(
            "msg_1", msg,
            vec!["n1".into(), "n2".into(), "n3".into()],
            Some("n1".into()),
            vec![],
        );

        // 添加 Seen 从其他 2 个节点
        state.on_seen("msg_1", "n2", "hash1");
        state.on_seen("msg_1", "n3", "hash1");

        // K=3, M=ceil(3*2/3)=2, 我们有 3 个一致 (自己+n2+n3)
        let (reached, count, m) = state.check_consensus("msg_1");
        assert!(reached, "共识应该达成: count={}, m={}", count, m);
        assert_eq!(state.get_status("msg_1"), Some(MessageStatus::Confirmed));
    }

    #[test]
    fn test_equivocation_detection() {
        let state = GossipState::new();
        let msg = make_signed_msg("hash1", 1);

        state.on_agent_message(
            "msg_1", msg,
            vec!["n1".into(), "n2".into(), "n3".into()],
            None,
            vec![],
        );

        state.on_seen("msg_1", "n2", "hash1");
        state.on_seen("msg_1", "n3", "hash_DIFFERENT"); // 不同的哈希!

        let conflict = state.has_conflicting_hashes("msg_1");
        assert!(conflict.is_some());
    }

    #[test]
    fn test_gc_removes_old_messages() {
        let state = GossipState::new();
        let msg = make_signed_msg("hash1", 1);

        state.on_agent_message("msg_1", msg.clone(), vec![], None, vec![]);

        // 手动设置 created_at 为很久之前
        if let Some(mut entry) = state.messages.get_mut("msg_1") {
            entry.created_at = 0; // epoch
        }

        state.gc_expired_messages();
        assert_eq!(state.active_message_count(), 0);
    }
}
