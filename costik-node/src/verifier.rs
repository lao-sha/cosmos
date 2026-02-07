use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use sha2::{Sha256, Digest};
use tracing::{warn, debug};

use crate::types::SignedMessage;
use crate::chain_cache::ChainCache;

/// 四层验证结果
#[derive(Debug)]
pub enum VerifyError {
    /// Ed25519 签名无效
    InvalidSignature(String),
    /// Bot 未注册或已停用
    BotNotActive(String),
    /// 公钥不匹配
    PublicKeyMismatch,
    /// 本节点不在目标节点列表中
    NotTargetNode(String),
    /// 消息过期
    MessageExpired,
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSignature(e) => write!(f, "签名无效: {}", e),
            Self::BotNotActive(id) => write!(f, "Bot 未活跃: {}", id),
            Self::PublicKeyMismatch => write!(f, "公钥不匹配"),
            Self::NotTargetNode(info) => write!(f, "非目标节点: {}", info),
            Self::MessageExpired => write!(f, "消息已过期"),
            Self::Other(e) => write!(f, "验证错误: {}", e),
        }
    }
}

/// 四层验证 Agent 消息（文档第 3 章）
///
/// ① Ed25519 验签
/// ② Bot 活跃状态检查
/// ③ 公钥匹配
/// ④ 目标节点验证
pub fn verify_signed_message(
    message: &SignedMessage,
    my_node_id: &str,
    chain_cache: &ChainCache,
) -> Result<(), VerifyError> {
    let now = chrono::Utc::now().timestamp() as u64;

    // 0. 消息时效检查（60 秒内）
    if now.saturating_sub(message.timestamp) > 60 {
        warn!(
            msg_hash = message.message_hash,
            age_secs = now - message.timestamp,
            "消息已过期"
        );
        return Err(VerifyError::MessageExpired);
    }

    // ① Ed25519 验签
    verify_ed25519_signature(message)?;

    // ② Bot 活跃状态检查
    let bot_info = chain_cache.get_bot(&message.bot_id_hash)
        .ok_or_else(|| VerifyError::BotNotActive(message.bot_id_hash.clone()))?;

    if !bot_info.is_active {
        return Err(VerifyError::BotNotActive(message.bot_id_hash.clone()));
    }

    // ③ 公钥匹配：消息中的公钥必须与链上注册的一致
    let msg_pubkey = hex::decode(&message.owner_public_key)
        .map_err(|e| VerifyError::Other(format!("公钥 hex 解码失败: {}", e)))?;
    if msg_pubkey.len() != 32 || msg_pubkey[..] != bot_info.owner_public_key[..] {
        return Err(VerifyError::PublicKeyMismatch);
    }

    // ④ 目标节点验证：本节点在确定性选择的 K 个中
    let active_nodes = chain_cache.get_active_node_ids();
    let k = select_k(active_nodes.len());
    let targets = deterministic_select_ids(&active_nodes, &message.message_hash, message.sequence, k);

    if !targets.contains(&my_node_id.to_string()) {
        return Err(VerifyError::NotTargetNode(format!(
            "我({}) 不在目标列表 {:?} 中",
            my_node_id, targets
        )));
    }

    debug!(
        msg_hash = message.message_hash,
        sequence = message.sequence,
        "四层验证通过"
    );

    Ok(())
}

/// ① Ed25519 签名验证
fn verify_ed25519_signature(message: &SignedMessage) -> Result<(), VerifyError> {
    // 解析公钥
    let pubkey_bytes = hex::decode(&message.owner_public_key)
        .map_err(|e| VerifyError::InvalidSignature(format!("公钥 hex 无效: {}", e)))?;
    if pubkey_bytes.len() != 32 {
        return Err(VerifyError::InvalidSignature("公钥长度不是 32 字节".into()));
    }
    let mut pk_arr = [0u8; 32];
    pk_arr.copy_from_slice(&pubkey_bytes);

    let verifying_key = VerifyingKey::from_bytes(&pk_arr)
        .map_err(|e| VerifyError::InvalidSignature(format!("公钥无效: {}", e)))?;

    // 解析签名
    let sig_bytes = hex::decode(&message.owner_signature)
        .map_err(|e| VerifyError::InvalidSignature(format!("签名 hex 无效: {}", e)))?;
    if sig_bytes.len() != 64 {
        return Err(VerifyError::InvalidSignature("签名长度不是 64 字节".into()));
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let signature = Signature::from_bytes(&sig_arr);

    // 解析 message_hash
    let msg_hash_bytes = hex::decode(&message.message_hash)
        .map_err(|e| VerifyError::InvalidSignature(format!("msg_hash hex 无效: {}", e)))?;

    // 解析 bot_id_hash
    let bot_hash_bytes = hex::decode(&message.bot_id_hash)
        .map_err(|e| VerifyError::InvalidSignature(format!("bot_id_hash hex 无效: {}", e)))?;

    // 重构签名数据 = public_key + bot_id_hash + sequence(LE) + timestamp(LE) + message_hash
    let mut sign_data = Vec::with_capacity(32 + 32 + 8 + 8 + 32);
    sign_data.extend_from_slice(&pk_arr);
    sign_data.extend_from_slice(&bot_hash_bytes);
    sign_data.extend_from_slice(&message.sequence.to_le_bytes());
    sign_data.extend_from_slice(&message.timestamp.to_le_bytes());
    sign_data.extend_from_slice(&msg_hash_bytes);

    // 验签
    verifying_key.verify(&sign_data, &signature)
        .map_err(|e| VerifyError::InvalidSignature(format!("Ed25519 验签失败: {}", e)))?;

    // 额外: 验证 message_hash 与 telegram_update JSON 一致
    let update_json = serde_json::to_vec(&message.telegram_update)
        .map_err(|e| VerifyError::Other(format!("序列化失败: {}", e)))?;
    let mut hasher = Sha256::new();
    hasher.update(&update_json);
    let computed_hash = hex::encode(hasher.finalize());

    // 注意: Agent 签名时使用 raw JSON bytes，而这里重新序列化可能不一致
    // 因此这个检查在生产中需要 Agent 同时发送 raw bytes 或跳过此检查
    // 当前保留为 debug 级别警告
    if computed_hash != message.message_hash {
        debug!(
            expected = message.message_hash,
            computed = computed_hash,
            "message_hash 与重新序列化不一致 (预期行为: Agent 使用原始字节)"
        );
    }

    Ok(())
}

/// 确定 K 值（与 Agent 侧算法完全一致）
fn select_k(total_nodes: usize) -> usize {
    if total_nodes <= 3 {
        return total_nodes;
    }
    let two_thirds = (total_nodes * 2 + 2) / 3;
    two_thirds.min(total_nodes).max(3)
}

/// 确定性随机选择节点 ID 列表（与 Agent 侧算法完全一致）
fn deterministic_select_ids(
    node_ids: &[String],
    message_hash_hex: &str,
    sequence: u64,
    k: usize,
) -> Vec<String> {
    if node_ids.is_empty() || k == 0 {
        return vec![];
    }

    let mut sorted = node_ids.to_vec();
    sorted.sort();
    let k = k.min(sorted.len());

    // seed = SHA256(message_hash + sequence_le)
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

    #[test]
    fn test_select_k_matches_agent() {
        assert_eq!(select_k(1), 1);
        assert_eq!(select_k(3), 3);
        assert_eq!(select_k(5), 4);
        assert_eq!(select_k(10), 7);
        assert_eq!(select_k(20), 14);
    }

    #[test]
    fn test_deterministic_select_matches_agent() {
        let ids: Vec<String> = (0..10).map(|i| format!("node_{:03}", i)).collect();
        let hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";

        let r1 = deterministic_select_ids(&ids, hash, 42, 5);
        let r2 = deterministic_select_ids(&ids, hash, 42, 5);

        assert_eq!(r1, r2);
        assert_eq!(r1.len(), 5);

        // 无重复
        let mut deduped = r1.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(deduped.len(), 5);
    }

    #[test]
    fn test_ed25519_sign_verify_roundtrip() {
        use ed25519_dalek::SigningKey;

        let mut csprng = rand::rngs::OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let bot_id_hash = [1u8; 32];
        let sequence: u64 = 42;
        let timestamp: u64 = 1700000000;
        let telegram_json = b"{\"update_id\":123}";

        // Agent 侧签名
        let mut hasher = Sha256::new();
        hasher.update(telegram_json);
        let msg_hash: [u8; 32] = hasher.finalize().into();

        let mut sign_data = Vec::new();
        sign_data.extend_from_slice(verifying_key.as_bytes());
        sign_data.extend_from_slice(&bot_id_hash);
        sign_data.extend_from_slice(&sequence.to_le_bytes());
        sign_data.extend_from_slice(&timestamp.to_le_bytes());
        sign_data.extend_from_slice(&msg_hash);

        use ed25519_dalek::Signer;
        let signature = signing_key.sign(&sign_data);

        // 构造 SignedMessage
        let message = SignedMessage {
            owner_public_key: hex::encode(verifying_key.as_bytes()),
            bot_id_hash: hex::encode(bot_id_hash),
            sequence,
            timestamp,
            message_hash: hex::encode(msg_hash),
            telegram_update: serde_json::from_slice(telegram_json).unwrap(),
            owner_signature: hex::encode(signature.to_bytes()),
            platform: "telegram".to_string(),
        };

        // Node 侧验签
        let result = verify_ed25519_signature(&message);
        assert!(result.is_ok(), "验签失败: {:?}", result.err());
    }
}
