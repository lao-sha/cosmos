use std::sync::Arc;
use std::collections::VecDeque;
use tokio::sync::Mutex;
use tracing::{info, warn, debug};

// SignedMessage used by ConfirmationEntry and ActionLogEntry fields

/// 批量上链提交器
///
/// 职责:
/// 1. 收集已确认的消息确认记录
/// 2. 批量提交到链上 (pallet-bot-consensus::submit_confirmations)
/// 3. 收集 Leader 执行结果 → 提交到链上 (pallet-bot-group-mgmt::log_action)
/// 4. 检测 Equivocation → 自动提交举报
///
/// 当前实现: 内存队列 + 定时批量提交（需 subxt 集成）
pub struct ChainSubmitter {
    /// 待提交的消息确认
    confirmation_queue: Mutex<VecDeque<ConfirmationEntry>>,
    /// 待提交的动作日志
    action_log_queue: Mutex<VecDeque<ActionLogEntry>>,
    /// 待提交的 Equivocation 举报
    equivocation_queue: Mutex<VecDeque<EquivocationEntry>>,
    /// 节点 ID
    node_id: String,
    /// 批量大小
    batch_size: usize,
    /// 提交间隔（秒）
    submit_interval_secs: u64,
}

/// 消息确认条目
#[derive(Debug, Clone)]
pub struct ConfirmationEntry {
    pub bot_id_hash: [u8; 32],
    pub msg_hash: [u8; 32],
    pub sequence: u64,
    pub timestamp: u64,
    pub confirmed_at_ms: u64,
}

/// 动作日志条目
#[derive(Debug, Clone)]
pub struct ActionLogEntry {
    pub community_id_hash: [u8; 32],
    pub action_type: u8,
    pub target_user_hash: [u8; 32],
    pub executor_node_hash: [u8; 32],
    pub consensus_count: u8,
    pub sequence: u64,
    pub msg_hash: [u8; 32],
}

/// Equivocation 举报条目
#[derive(Debug, Clone)]
pub struct EquivocationEntry {
    pub owner_public_key: [u8; 32],
    pub sequence: u64,
    pub msg_hash_a: [u8; 32],
    pub signature_a: [u8; 64],
    pub msg_hash_b: [u8; 32],
    pub signature_b: [u8; 64],
}

/// 批量提交结果
#[derive(Debug)]
pub struct SubmitResult {
    pub confirmations_submitted: usize,
    pub action_logs_submitted: usize,
    pub equivocations_submitted: usize,
    pub errors: Vec<String>,
}

impl ChainSubmitter {
    pub fn new(node_id: String) -> Self {
        Self {
            confirmation_queue: Mutex::new(VecDeque::new()),
            action_log_queue: Mutex::new(VecDeque::new()),
            equivocation_queue: Mutex::new(VecDeque::new()),
            node_id,
            batch_size: 50,
            submit_interval_secs: 6, // 每个区块提交一次
        }
    }

    /// 队列容量上限（防止 OOM）
    const MAX_QUEUE_SIZE: usize = 10_000;

    /// 添加消息确认到队列
    pub async fn queue_confirmation(&self, entry: ConfirmationEntry) {
        let mut queue = self.confirmation_queue.lock().await;
        if queue.len() >= Self::MAX_QUEUE_SIZE {
            warn!(queue_size = queue.len(), "确认队列已满，丢弃最旧条目");
            queue.pop_front();
        }
        queue.push_back(entry);
        debug!(queue_size = queue.len(), "确认已入队");
    }

    /// 添加动作日志到队列
    pub async fn queue_action_log(&self, entry: ActionLogEntry) {
        let mut queue = self.action_log_queue.lock().await;
        if queue.len() >= Self::MAX_QUEUE_SIZE {
            warn!(queue_size = queue.len(), "动作日志队列已满，丢弃最旧条目");
            queue.pop_front();
        }
        queue.push_back(entry);
        debug!(queue_size = queue.len(), "动作日志已入队");
    }

    /// 添加 Equivocation 举报到队列（优先级最高）
    pub async fn queue_equivocation(&self, entry: EquivocationEntry) {
        let mut queue = self.equivocation_queue.lock().await;
        if queue.len() >= Self::MAX_QUEUE_SIZE {
            warn!(queue_size = queue.len(), "举报队列已满，丢弃最旧条目");
            queue.pop_front();
        }
        queue.push_back(entry);
        warn!(queue_size = queue.len(), "Equivocation 举报已入队");
    }

    /// 批量提交（定时调用）
    ///
    /// 优先级: Equivocation > ActionLog > Confirmation
    pub async fn flush(&self) -> SubmitResult {
        let mut result = SubmitResult {
            confirmations_submitted: 0,
            action_logs_submitted: 0,
            equivocations_submitted: 0,
            errors: vec![],
        };

        // 1. 提交 Equivocation（立即，不等批量）
        {
            let mut queue = self.equivocation_queue.lock().await;
            while let Some(entry) = queue.pop_front() {
                match self.submit_equivocation(&entry).await {
                    Ok(_) => {
                        result.equivocations_submitted += 1;
                        info!(
                            sequence = entry.sequence,
                            "Equivocation 举报已提交"
                        );
                    }
                    Err(e) => {
                        result.errors.push(format!("equivocation: {}", e));
                        // 重新入队
                        queue.push_front(entry);
                        break;
                    }
                }
            }
        }

        // 2. 提交动作日志
        {
            let mut queue = self.action_log_queue.lock().await;
            let drain_len = self.batch_size.min(queue.len());
            let batch: Vec<_> = queue.drain(..drain_len).collect();
            if !batch.is_empty() {
                match self.submit_action_logs(&batch).await {
                    Ok(_) => {
                        result.action_logs_submitted = batch.len();
                        debug!(count = batch.len(), "动作日志批量提交成功");
                    }
                    Err(e) => {
                        result.errors.push(format!("action_logs: {}", e));
                        // 重新入队
                        for entry in batch.into_iter().rev() {
                            queue.push_front(entry);
                        }
                    }
                }
            }
        }

        // 3. 提交消息确认
        {
            let mut queue = self.confirmation_queue.lock().await;
            let drain_len = self.batch_size.min(queue.len());
            let batch: Vec<_> = queue.drain(..drain_len).collect();
            if !batch.is_empty() {
                match self.submit_confirmations(&batch).await {
                    Ok(_) => {
                        result.confirmations_submitted = batch.len();
                        debug!(count = batch.len(), "确认批量提交成功");
                    }
                    Err(e) => {
                        result.errors.push(format!("confirmations: {}", e));
                        for entry in batch.into_iter().rev() {
                            queue.push_front(entry);
                        }
                    }
                }
            }
        }

        if result.confirmations_submitted > 0
            || result.action_logs_submitted > 0
            || result.equivocations_submitted > 0
        {
            info!(
                confirmations = result.confirmations_submitted,
                action_logs = result.action_logs_submitted,
                equivocations = result.equivocations_submitted,
                errors = result.errors.len(),
                "批量提交完成"
            );
        }

        result
    }

    /// 提交消息确认到链上
    ///
    /// 调用: pallet-bot-consensus::submit_confirmations(confirmations)
    async fn submit_confirmations(&self, _batch: &[ConfirmationEntry]) -> anyhow::Result<()> {
        // TODO: 使用 subxt 构造并提交交易
        // let api = subxt::OnlineClient::new().await?;
        // let tx = api.tx().pallet_bot_consensus().submit_confirmations(batch);
        // api.sign_and_submit(&tx, &signer).await?;
        debug!(count = _batch.len(), "TODO: subxt submit_confirmations");
        Ok(())
    }

    /// 提交动作日志到链上
    ///
    /// 调用: pallet-bot-group-mgmt::log_action(...)
    async fn submit_action_logs(&self, _batch: &[ActionLogEntry]) -> anyhow::Result<()> {
        // TODO: 使用 subxt 构造并提交交易
        debug!(count = _batch.len(), "TODO: subxt log_action");
        Ok(())
    }

    /// 提交 Equivocation 举报到链上
    ///
    /// 调用: pallet-bot-consensus::report_equivocation(...)
    async fn submit_equivocation(&self, _entry: &EquivocationEntry) -> anyhow::Result<()> {
        // TODO: 使用 subxt 构造并提交交易
        debug!(sequence = _entry.sequence, "TODO: subxt report_equivocation");
        Ok(())
    }

    /// 获取队列大小
    pub async fn queue_sizes(&self) -> (usize, usize, usize) {
        let c = self.confirmation_queue.lock().await.len();
        let a = self.action_log_queue.lock().await.len();
        let e = self.equivocation_queue.lock().await.len();
        (c, a, e)
    }

    /// 启动定时提交循环
    pub fn spawn_submit_loop(self: Arc<Self>) {
        let interval = self.submit_interval_secs;
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(
                std::time::Duration::from_secs(interval),
            );
            loop {
                tick.tick().await;
                self.flush().await;
            }
        });
    }
}

/// 序列号追踪器（重放保护）
///
/// 每个 (bot_id_hash, owner_public_key) 维护最近 seen 的序列号窗口
pub struct SequenceTracker {
    /// (bot_id_hash_hex) → 最后已知序列号
    last_seen: dashmap::DashMap<String, u64>,
}

impl SequenceTracker {
    pub fn new() -> Self {
        Self {
            last_seen: dashmap::DashMap::new(),
        }
    }

    /// 检查序列号是否有效（防重放）
    ///
    /// 规则:
    /// 1. sequence > last_seen → 有效
    /// 2. sequence == last_seen → 重放
    /// 3. sequence < last_seen → 重放
    /// 4. 允许小窗口跳跃（乱序容忍）
    pub fn check_and_update(&self, bot_id_hash: &str, sequence: u64) -> bool {
        let key = bot_id_hash.to_string();

        // 使用 entry API 避免 TOCTOU 竞态
        let entry = self.last_seen.entry(key);
        match entry {
            dashmap::mapref::entry::Entry::Occupied(mut occ) => {
                let last_val = *occ.get();
                if sequence > last_val {
                    occ.insert(sequence);
                    true
                } else if sequence == last_val {
                    warn!(bot_id_hash, sequence, "重放检测: 序列号重复");
                    false
                } else {
                    // 旧序列号 — 允许小窗口 (±10)
                    if last_val - sequence <= 10 {
                        debug!(bot_id_hash, sequence, last = last_val, "旧序列号但在容忍窗口内");
                        true
                    } else {
                        warn!(bot_id_hash, sequence, last = last_val, "重放检测: 序列号过旧");
                        false
                    }
                }
            }
            dashmap::mapref::entry::Entry::Vacant(vac) => {
                vac.insert(sequence);
                true
            }
        }
    }

    /// 获取某 bot 的最后序列号
    pub fn last_sequence(&self, bot_id_hash: &str) -> Option<u64> {
        self.last_seen.get(bot_id_hash).map(|v| *v)
    }

    /// 清理不活跃的条目（防止内存泄漏）
    ///
    /// 保留最近活跃的 max_entries 条，按 sequence 值排序保留最大的
    pub fn gc(&self, max_entries: usize) {
        if self.last_seen.len() <= max_entries {
            return;
        }
        // 收集所有条目并按 sequence 排序
        let mut entries: Vec<(String, u64)> = self.last_seen
            .iter()
            .map(|r| (r.key().clone(), *r.value()))
            .collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1)); // 降序
        // 保留 top max_entries，删除其余
        for (key, _) in entries.into_iter().skip(max_entries) {
            self.last_seen.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_queue_and_flush() {
        let submitter = Arc::new(ChainSubmitter::new("test_node".into()));

        // 入队 3 条确认
        for i in 0..3 {
            submitter.queue_confirmation(ConfirmationEntry {
                bot_id_hash: [i; 32],
                msg_hash: [i + 10; 32],
                sequence: i as u64,
                timestamp: 1700000000,
                confirmed_at_ms: 1700000000000,
            }).await;
        }

        let (c, a, e) = submitter.queue_sizes().await;
        assert_eq!(c, 3);
        assert_eq!(a, 0);
        assert_eq!(e, 0);

        // flush
        let result = submitter.flush().await;
        assert_eq!(result.confirmations_submitted, 3);
        assert!(result.errors.is_empty());

        // 队列应该为空
        let (c, _, _) = submitter.queue_sizes().await;
        assert_eq!(c, 0);
    }

    #[tokio::test]
    async fn test_equivocation_priority() {
        let submitter = Arc::new(ChainSubmitter::new("test_node".into()));

        submitter.queue_confirmation(ConfirmationEntry {
            bot_id_hash: [1; 32],
            msg_hash: [2; 32],
            sequence: 1,
            timestamp: 0,
            confirmed_at_ms: 0,
        }).await;

        submitter.queue_equivocation(EquivocationEntry {
            owner_public_key: [3; 32],
            sequence: 42,
            msg_hash_a: [4; 32],
            signature_a: [5; 64],
            msg_hash_b: [6; 32],
            signature_b: [7; 64],
        }).await;

        let result = submitter.flush().await;
        // equivocation 应该被提交
        assert_eq!(result.equivocations_submitted, 1);
        assert_eq!(result.confirmations_submitted, 1);
    }

    #[test]
    fn test_sequence_tracker_normal_flow() {
        let tracker = SequenceTracker::new();

        assert!(tracker.check_and_update("bot_1", 1));
        assert!(tracker.check_and_update("bot_1", 2));
        assert!(tracker.check_and_update("bot_1", 3));
        assert_eq!(tracker.last_sequence("bot_1"), Some(3));
    }

    #[test]
    fn test_sequence_tracker_replay() {
        let tracker = SequenceTracker::new();

        assert!(tracker.check_and_update("bot_1", 50));
        // 重复
        assert!(!tracker.check_and_update("bot_1", 50));
        // 过旧（超出容忍窗口）
        assert!(!tracker.check_and_update("bot_1", 1)); // 50-1=49 > 10
        // 但窗口内旧值 OK
        assert!(tracker.check_and_update("bot_1", 45)); // 50-45=5 <= 10
    }

    #[test]
    fn test_sequence_tracker_independent_bots() {
        let tracker = SequenceTracker::new();

        assert!(tracker.check_and_update("bot_a", 1));
        assert!(tracker.check_and_update("bot_b", 1)); // 不同 bot，同序列号 OK
        assert!(tracker.check_and_update("bot_a", 2));
        assert_eq!(tracker.last_sequence("bot_a"), Some(2));
        assert_eq!(tracker.last_sequence("bot_b"), Some(1));
    }
}
