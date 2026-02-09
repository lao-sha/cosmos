use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn, debug};

use crate::types::{SignedMessage, MulticastResult, NodeSendResult, NodeInfo};
use crate::AppState;

/// 确定性随机多播到 K 个节点
///
/// 算法（文档第 2 章）:
/// 1. seed = SHA256(message_hash + sequence)
/// 2. sorted_nodes = ActiveNodeList.sorted()
/// 3. 确定性 Fisher-Yates 取前 K 个
/// 4. 并发 POST 到 K 个节点
pub async fn multicast_to_nodes(
    state: &Arc<AppState>,
    message: &SignedMessage,
) -> anyhow::Result<MulticastResult> {
    // 提前释放读锁，避免在多播期间（最长 MULTICAST_TIMEOUT_MS）阻塞节点列表更新
    let (k, target_nodes, total) = {
        let nodes = state.nodes.read().await;
        if nodes.is_empty() {
            warn!("活跃节点列表为空，跳过多播");
            return Ok(MulticastResult {
                success_count: 0,
                failure_count: 0,
                timeout_count: 0,
                details: vec![],
            });
        }
        let k = select_k(nodes.len());
        let targets = deterministic_select(&nodes, &message.message_hash, message.sequence, k);
        let total = nodes.len();
        (k, targets, total)
    }; // RwLock 在此释放

    info!(
        total_nodes = total,
        k,
        targets = ?target_nodes.iter().map(|n| &n.node_id).collect::<Vec<_>>(),
        "确定性选择 {} 个目标节点",
        k
    );

    // 并发发送
    let timeout = std::time::Duration::from_millis(state.config.multicast_timeout_ms);
    let client = &state.http_client;
    let message_json = serde_json::to_vec(message)?;

    let mut handles = Vec::with_capacity(target_nodes.len());

    for node in &target_nodes {
        let client = client.clone();
        let url = format!("{}/v1/message", node.endpoint);
        let body = message_json.clone();
        let node_id = node.node_id.clone();
        let timeout = timeout;

        handles.push(tokio::spawn(async move {
            let start = Instant::now();
            let result = client
                .post(&url)
                .header("content-type", "application/json")
                .body(body)
                .timeout(timeout)
                .send()
                .await;

            let latency_ms = start.elapsed().as_millis() as u64;

            match result {
                Ok(resp) if resp.status().is_success() => {
                    debug!(node_id, latency_ms, "节点响应成功");
                    NodeSendResult {
                        node_id,
                        success: true,
                        latency_ms,
                        error: None,
                    }
                }
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    warn!(node_id, status, latency_ms, "节点响应非 2xx");
                    NodeSendResult {
                        node_id,
                        success: false,
                        latency_ms,
                        error: Some(format!("HTTP {}", status)),
                    }
                }
                Err(e) => {
                    let is_timeout = e.is_timeout();
                    warn!(node_id, error = %e, is_timeout, "发送失败");
                    NodeSendResult {
                        node_id,
                        success: false,
                        latency_ms,
                        error: Some(if is_timeout {
                            "timeout".to_string()
                        } else {
                            e.to_string()
                        }),
                    }
                }
            }
        }));
    }

    // 收集结果
    let mut details = Vec::with_capacity(handles.len());
    for handle in handles {
        match handle.await {
            Ok(result) => details.push(result),
            Err(e) => {
                warn!(error = %e, "发送任务 panic");
            }
        }
    }

    let success_count = details.iter().filter(|d| d.success).count();
    let timeout_count = details.iter().filter(|d| {
        d.error.as_deref() == Some("timeout")
    }).count();
    let failure_count = details.len() - success_count;

    Ok(MulticastResult {
        success_count,
        failure_count,
        timeout_count,
        details,
    })
}

/// 确定 K 值（发送给多少个节点）
///
/// K = min(ceil(N * 2/3), N)，至少 3 个
fn select_k(total_nodes: usize) -> usize {
    if total_nodes <= 3 {
        return total_nodes;
    }
    let two_thirds = (total_nodes * 2 + 2) / 3; // ceil(N * 2/3)
    two_thirds.min(total_nodes).max(3)
}

/// 确定性随机选择 K 个节点
///
/// seed = SHA256(message_hash_hex + sequence_le_bytes)
/// 使用 seed 做确定性 Fisher-Yates shuffle，取前 K 个
fn deterministic_select(
    nodes: &[NodeInfo],
    message_hash_hex: &str,
    sequence: u64,
    k: usize,
) -> Vec<NodeInfo> {
    use sha2::{Sha256, Digest};

    if nodes.is_empty() || k == 0 {
        return vec![];
    }

    // 按 node_id 排序（确保所有参与者一致）
    let mut sorted: Vec<NodeInfo> = nodes.to_vec();
    sorted.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    let k = k.min(sorted.len());

    // seed = SHA256(message_hash + sequence_le)
    let mut hasher = Sha256::new();
    hasher.update(message_hash_hex.as_bytes());
    hasher.update(&sequence.to_le_bytes());
    let seed = hasher.finalize();

    // 确定性 Fisher-Yates shuffle (取前 K 个)
    let n = sorted.len();
    for i in 0..k {
        // 从 seed 中提取伪随机数
        let mut idx_hasher = Sha256::new();
        idx_hasher.update(&seed);
        idx_hasher.update(&(i as u64).to_le_bytes());
        let idx_hash = idx_hasher.finalize();

        // 从哈希中取 8 字节作为 u64
        let rand_val = u64::from_le_bytes(idx_hash[..8].try_into().unwrap());
        let j = i + (rand_val as usize % (n - i));

        sorted.swap(i, j);
    }

    sorted.truncate(k);
    sorted
}

/// 注册 Telegram Webhook
///
/// 调用 TG API: setWebhook(url, secret_token)
pub async fn register_telegram_webhook(
    client: &reqwest::Client,
    bot_token: &str,
    webhook_url: &str,
    secret_token: &str,
) -> anyhow::Result<()> {
    let url = format!(
        "https://api.telegram.org/bot{}/setWebhook",
        bot_token
    );

    let params = serde_json::json!({
        "url": format!("{}/webhook", webhook_url),
        "secret_token": secret_token,
        "allowed_updates": ["message", "edited_message", "callback_query",
                           "chat_member", "chat_join_request"],
        "drop_pending_updates": false,
    });

    let mut last_error = None;
    for attempt in 1..=3 {
        match client.post(&url).json(&params).send().await {
            Ok(resp) => {
                let body: crate::types::TelegramApiResponse = resp.json().await?;
                if body.ok {
                    info!("Telegram Webhook 注册成功");
                    return Ok(());
                } else {
                    let desc = body.description.unwrap_or_default();
                    warn!(attempt, error = desc, "setWebhook 返回错误");
                    last_error = Some(desc);
                }
            }
            Err(e) => {
                warn!(attempt, error = %e, "setWebhook 请求失败");
                last_error = Some(e.to_string());
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    anyhow::bail!(
        "setWebhook 失败（3 次重试）: {}",
        last_error.unwrap_or_default()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_nodes(count: usize) -> Vec<NodeInfo> {
        (0..count)
            .map(|i| NodeInfo {
                node_id: format!("node_{:03}", i),
                endpoint: format!("http://node{}:8080", i),
                node_public_key: format!("{:064x}", i),
                status: "Active".to_string(),
            })
            .collect()
    }

    #[test]
    fn test_select_k() {
        assert_eq!(select_k(1), 1);
        assert_eq!(select_k(2), 2);
        assert_eq!(select_k(3), 3);
        assert_eq!(select_k(5), 4);    // ceil(5*2/3) = 4
        assert_eq!(select_k(10), 7);   // ceil(10*2/3) = 7
        assert_eq!(select_k(20), 14);  // ceil(20*2/3) = 14
    }

    #[test]
    fn test_deterministic_select_is_deterministic() {
        let nodes = mock_nodes(10);
        let hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";

        let r1 = deterministic_select(&nodes, hash, 42, 5);
        let r2 = deterministic_select(&nodes, hash, 42, 5);

        // 相同输入 → 相同输出
        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.node_id, b.node_id);
        }
    }

    #[test]
    fn test_deterministic_select_different_sequence() {
        let nodes = mock_nodes(10);
        let hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";

        let r1 = deterministic_select(&nodes, hash, 1, 5);
        let r2 = deterministic_select(&nodes, hash, 2, 5);

        // 不同 sequence → 大概率不同的选择
        let ids1: Vec<_> = r1.iter().map(|n| &n.node_id).collect();
        let ids2: Vec<_> = r2.iter().map(|n| &n.node_id).collect();
        // 不强制不同（小概率相同），但长度一致
        assert_eq!(ids1.len(), 5);
        assert_eq!(ids2.len(), 5);
    }

    #[test]
    fn test_deterministic_select_all_nodes_when_k_ge_n() {
        let nodes = mock_nodes(3);
        let hash = "0000000000000000000000000000000000000000000000000000000000000000";

        let result = deterministic_select(&nodes, hash, 1, 5);
        // K > N → 返回所有节点
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_deterministic_select_unique() {
        let nodes = mock_nodes(20);
        let hash = "aaaa";

        let result = deterministic_select(&nodes, hash, 100, 10);
        // 确保没有重复
        let mut ids: Vec<_> = result.iter().map(|n| &n.node_id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 10);
    }
}
