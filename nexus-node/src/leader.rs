use std::sync::Arc;
use tracing::{info, warn, debug, error};

use crate::types::*;
use crate::types::{ActionType, MessageAction, AdminAction};
use crate::gossip::state::GossipState;
use crate::chain_cache::ChainCache;

/// Leader 执行器
///
/// 职责:
/// 1. 共识达成后，判断自己是否为 Leader
/// 2. 根据消息内容决定动作类型
/// 3. 向 Agent 的 /v1/execute 发送执行指令
/// 4. 等待执行结果
/// 5. 广播 ExecutionResult
/// 6. Failover: 超时后 Backup 接管
pub struct LeaderExecutor {
    pub node_id: String,
    pub state: Arc<GossipState>,
    pub chain_cache: Arc<ChainCache>,
    pub http_client: reqwest::Client,
    /// Leader 执行超时（毫秒）
    pub execute_timeout_ms: u64,
    /// 节点 Ed25519 签名密钥
    pub signing_key: Option<ed25519_dalek::SigningKey>,
}

/// 执行动作
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecuteAction {
    /// 动作 ID（唯一）
    pub action_id: String,
    /// 动作类型
    pub action_type: ActionType,
    /// 目标 Bot ID hash
    pub bot_id_hash: String,
    /// 目标 chat_id
    pub chat_id: i64,
    /// 动作参数
    pub params: serde_json::Value,
    /// Leader 节点签名 (hex)
    pub leader_signature: String,
    /// Leader 节点 ID
    pub leader_node_id: String,
    /// 共识证据: 参与 Seen 的节点列表
    pub consensus_nodes: Vec<String>,
    /// 目标平台（telegram/discord），默认 telegram 以兼容旧版
    #[serde(default = "default_execute_platform")]
    pub platform: String,
}

fn default_execute_platform() -> String {
    "telegram".to_string()
}

/// 执行结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecuteResult {
    pub action_id: String,
    pub success: bool,
    pub error: Option<String>,
    /// Agent 回执签名
    pub agent_receipt: Option<String>,
}

/// Leader 选举结果
#[derive(Debug, Clone)]
pub struct ElectionResult {
    /// Leader 节点 ID
    pub leader: String,
    /// Backup 节点列表（按优先级排序）
    pub backups: Vec<String>,
}

impl LeaderExecutor {
    pub fn new(
        node_id: String,
        state: Arc<GossipState>,
        chain_cache: Arc<ChainCache>,
    ) -> Self {
        Self {
            node_id,
            state,
            chain_cache,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            execute_timeout_ms: 5000,
            signing_key: None,
        }
    }

    pub fn with_signing_key(mut self, key: ed25519_dalek::SigningKey) -> Self {
        self.signing_key = Some(key);
        self
    }

    /// V4: 生成 Leader 签名
    ///
    /// 签名数据 = SHA256(action_id + bot_id_hash + action_type_str + chat_id_le)
    /// 返回格式: "pubkey_hex:signature_hex"
    fn sign_action(&self, action_id: &str, bot_id_hash: &str, action_type: &ActionType, chat_id: i64) -> String {
        use sha2::{Sha256, Digest};
        use ed25519_dalek::Signer;

        let Some(ref sk) = self.signing_key else {
            return String::new();
        };

        let action_type_str = format!("{:?}", action_type);
        let mut hasher = Sha256::new();
        hasher.update(action_id.as_bytes());
        hasher.update(bot_id_hash.as_bytes());
        hasher.update(action_type_str.as_bytes());
        hasher.update(&chat_id.to_le_bytes());
        let sign_data = hasher.finalize();

        let sig = sk.sign(&sign_data);
        let pk = sk.verifying_key();
        format!("{}:{}", hex::encode(pk.as_bytes()), hex::encode(sig.to_bytes()))
    }

    /// Leader 选举：确定性 Round-Robin
    ///
    /// 算法（文档第 14 章）:
    /// leader_index = sequence % K
    /// targets[leader_index] 为 Leader，其余为 Backup
    pub fn elect_leader(
        target_nodes: &[String],
        sequence: u64,
    ) -> ElectionResult {
        if target_nodes.is_empty() {
            return ElectionResult {
                leader: String::new(),
                backups: vec![],
            };
        }

        let k = target_nodes.len();
        let leader_idx = (sequence as usize) % k;
        let leader = target_nodes[leader_idx].clone();

        let mut backups = Vec::with_capacity(k - 1);
        for (i, node) in target_nodes.iter().enumerate() {
            if i != leader_idx {
                backups.push(node.clone());
            }
        }

        ElectionResult { leader, backups }
    }

    /// 共识达成后，执行 Leader 职责
    ///
    /// 1. 判断动作类型
    /// 2. 构造 ExecuteAction
    /// 3. POST 到 Agent /v1/execute
    /// 4. 返回结果
    pub async fn execute_as_leader(
        &self,
        msg_id: &str,
        message: &SignedMessage,
        consensus_nodes: Vec<String>,
    ) -> anyhow::Result<ExecuteResult> {
        info!(msg_id, "开始 Leader 执行");

        // 1. 判断动作类型
        let (action_type, chat_id, params) = self.determine_action(message);

        if action_type.is_no_action() {
            debug!(msg_id, "无需执行动作");
            self.state.set_completed(msg_id);
            return Ok(ExecuteResult {
                action_id: msg_id.to_string(),
                success: true,
                error: None,
                agent_receipt: None,
            });
        }

        // 2. 构造 ExecuteAction
        let action = ExecuteAction {
            action_id: msg_id.to_string(),
            action_type: action_type.clone(),
            bot_id_hash: message.bot_id_hash.clone(),
            chat_id,
            params,
            leader_signature: self.sign_action(msg_id, &message.bot_id_hash, &action_type, chat_id),
            leader_node_id: self.node_id.clone(),
            consensus_nodes,
            platform: message.platform.clone(),
        };

        // 3. 查找 Agent 端点（从 Bot 注册信息推断）
        // TODO: Agent 端点应该在链上或通过节点发现获取
        // 当前: 使用环境变量 AGENT_ENDPOINT
        let agent_endpoint = std::env::var("AGENT_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8443".to_string());

        let url = format!("{}/v1/execute", agent_endpoint);

        // 4. POST 到 Agent
        let timeout = std::time::Duration::from_millis(self.execute_timeout_ms);

        match self.http_client
            .post(&url)
            .json(&action)
            .timeout(timeout)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let result: ExecuteResult = resp.json().await.unwrap_or(ExecuteResult {
                    action_id: msg_id.to_string(),
                    success: true,
                    error: None,
                    agent_receipt: None,
                });

                if result.success {
                    self.state.set_completed(msg_id);
                    info!(msg_id, action_type = ?action_type, "Leader 执行成功");
                } else {
                    self.state.set_failed(msg_id);
                    warn!(msg_id, error = ?result.error, "Leader 执行返回失败");
                }

                Ok(result)
            }
            Ok(resp) => {
                let status = resp.status();
                self.state.set_failed(msg_id);
                warn!(msg_id, %status, "Agent 返回非 2xx");
                Ok(ExecuteResult {
                    action_id: msg_id.to_string(),
                    success: false,
                    error: Some(format!("Agent HTTP {}", status)),
                    agent_receipt: None,
                })
            }
            Err(e) => {
                self.state.set_failed(msg_id);
                if e.is_timeout() {
                    warn!(msg_id, "Leader 执行超时 — Backup 可接管");
                } else {
                    error!(msg_id, error = %e, "Leader 执行失败");
                }
                Ok(ExecuteResult {
                    action_id: msg_id.to_string(),
                    success: false,
                    error: Some(e.to_string()),
                    agent_receipt: None,
                })
            }
        }
    }

    /// 根据平台事件内容判断动作类型
    ///
    /// 根据 message.platform 分发到对应的 NodePlatformAdapter，
    /// 默认回退到 Telegram 适配器以兼容旧消息。
    fn determine_action(&self, message: &SignedMessage) -> (ActionType, i64, serde_json::Value) {
        let adapter = crate::platform::get_adapter(&message.platform)
            .unwrap_or_else(|| crate::platform::get_adapter("telegram").unwrap());
        adapter.determine_action(message)
    }
}

/// Failover 管理器
pub struct FailoverManager {
    /// Leader 超时阈值（毫秒）
    pub timeout_ms: u64,
}

impl FailoverManager {
    pub fn new(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }

    /// 检查是否需要 Backup 接管
    ///
    /// 如果共识达成后 timeout_ms 内 Leader 未广播 ExecutionResult，
    /// 则 Backup[0] 应接管
    pub fn should_takeover(
        &self,
        msg_id: &str,
        state: &GossipState,
        my_node_id: &str,
        backups: &[String],
    ) -> Option<u32> {
        let status = state.get_status(msg_id)?;

        if status != MessageStatus::Confirmed && status != MessageStatus::Executing {
            return None;
        }

        // 找到自己在 backup 列表中的排名
        let my_rank = backups.iter().position(|id| id == my_node_id)?;

        // 每个 Backup 等待额外 rank * 2s
        let _total_wait = self.timeout_ms + (my_rank as u64 * 2000);

        // TODO: 检查时间差（需要记录共识达成时间）
        // 当前简化: 由外部定时器调用

        Some(my_rank as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elect_leader_round_robin() {
        let nodes = vec!["a".into(), "b".into(), "c".into()];

        let e1 = LeaderExecutor::elect_leader(&nodes, 0);
        assert_eq!(e1.leader, "a");
        assert_eq!(e1.backups, vec!["b", "c"]);

        let e2 = LeaderExecutor::elect_leader(&nodes, 1);
        assert_eq!(e2.leader, "b");
        assert_eq!(e2.backups, vec!["a", "c"]);

        let e3 = LeaderExecutor::elect_leader(&nodes, 2);
        assert_eq!(e3.leader, "c");
        assert_eq!(e3.backups, vec!["a", "b"]);

        // 循环
        let e4 = LeaderExecutor::elect_leader(&nodes, 3);
        assert_eq!(e4.leader, "a");
    }

    #[test]
    fn test_elect_leader_single_node() {
        let nodes = vec!["only".into()];
        let e = LeaderExecutor::elect_leader(&nodes, 42);
        assert_eq!(e.leader, "only");
        assert!(e.backups.is_empty());
    }

    #[test]
    fn test_elect_leader_empty() {
        let nodes: Vec<String> = vec![];
        let e = LeaderExecutor::elect_leader(&nodes, 0);
        assert!(e.leader.is_empty());
    }

    #[test]
    fn test_determine_action_join_request() {
        let executor = LeaderExecutor::new(
            "node_1".into(),
            Arc::new(GossipState::new()),
            Arc::new(ChainCache::new()),
        );

        let msg = SignedMessage {
            owner_public_key: String::new(),
            bot_id_hash: String::new(),
            sequence: 1,
            timestamp: 0,
            message_hash: String::new(),
            platform_event: serde_json::json!({
                "update_id": 1,
                "chat_join_request": {
                    "chat": {"id": -100123},
                    "from": {"id": 456, "is_bot": false}
                }
            }),
            owner_signature: String::new(),
            platform: "telegram".into(),
        };

        let (action_type, _chat_id, params) = executor.determine_action(&msg);
        assert!(matches!(action_type, ActionType::Admin(AdminAction::ApproveJoinRequest)));
        assert_eq!(params["user_id"], 456);
    }

    #[test]
    fn test_determine_action_ban_command() {
        let executor = LeaderExecutor::new(
            "node_1".into(),
            Arc::new(GossipState::new()),
            Arc::new(ChainCache::new()),
        );

        let msg = SignedMessage {
            owner_public_key: String::new(),
            bot_id_hash: String::new(),
            sequence: 1,
            timestamp: 0,
            message_hash: String::new(),
            platform_event: serde_json::json!({
                "update_id": 1,
                "message": {
                    "message_id": 100,
                    "chat": {"id": -100123, "type": "supergroup"},
                    "text": "/ban",
                    "reply_to_message": {
                        "message_id": 99,
                        "from": {"id": 789, "is_bot": false}
                    }
                }
            }),
            owner_signature: String::new(),
            platform: "telegram".into(),
        };

        let (action_type, chat_id, params) = executor.determine_action(&msg);
        assert!(matches!(action_type, ActionType::Admin(AdminAction::Ban)));
        assert_eq!(chat_id, -100123);
        assert_eq!(params["user_id"], 789);
    }

    #[test]
    fn test_determine_action_plain_message() {
        let executor = LeaderExecutor::new(
            "node_1".into(),
            Arc::new(GossipState::new()),
            Arc::new(ChainCache::new()),
        );

        let msg = SignedMessage {
            owner_public_key: String::new(),
            bot_id_hash: String::new(),
            sequence: 1,
            timestamp: 0,
            message_hash: String::new(),
            platform_event: serde_json::json!({
                "update_id": 1,
                "message": {
                    "message_id": 100,
                    "chat": {"id": -100123, "type": "supergroup"},
                    "text": "hello world"
                }
            }),
            owner_signature: String::new(),
            platform: "telegram".into(),
        };

        let (action_type, _, _) = executor.determine_action(&msg);
        assert!(action_type.is_no_action());
    }
}
