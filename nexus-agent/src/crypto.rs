//! # 加盐哈希工具
//!
//! 修复 community_id_hash 和 platform_user_id_hash 的哈希碰撞漏洞。
//! 原方案 `SHA256(chat_id)` 不安全（chat_id 范围 ~10^13，可暴力碰撞）。
//!
//! 新方案：加盐哈希，盐值由 owner_account 派生，不可预测。
//!
//! 详见 `docs/NEXUS_LAYERED_STORAGE_DESIGN.md` 第七节。

use sha2::{Sha256, Digest};

/// 计算社区 ID 哈希（加盐）
///
/// ```text
/// salt = SHA256(owner_account + "nexus-community-salt")
/// hash = SHA256(platform + chat_id_le_bytes + salt)
/// ```
///
/// ## 参数
/// - `platform`: 平台名称（如 "telegram", "discord"）
/// - `chat_id`: 平台侧的群组/频道 ID
/// - `owner_account`: 群主的链上账户公钥（32 字节）
pub fn compute_community_id_hash(
    platform: &str,
    chat_id: i64,
    owner_account: &[u8; 32],
) -> [u8; 32] {
    let salt = Sha256::new()
        .chain_update(owner_account)
        .chain_update(b"nexus-community-salt")
        .finalize();

    // 长度前缀防止域分离歧义 (e.g. "ab"+"c..." vs "abc"+"...")
    let result = Sha256::new()
        .chain_update(&(platform.len() as u32).to_le_bytes())
        .chain_update(platform.as_bytes())
        .chain_update(&chat_id.to_le_bytes())
        .chain_update(&salt)
        .finalize();

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// 计算平台用户 ID 哈希（加盐）
///
/// ```text
/// salt = SHA256(owner_account + "nexus-user-salt")
/// hash = SHA256(platform + user_id_str + salt)
/// ```
///
/// ## 参数
/// - `platform`: 平台名称
/// - `user_id`: 平台侧用户 ID（字符串形式，支持各平台格式）
/// - `owner_account`: 所属社区群主的链上账户公钥（32 字节）
pub fn compute_platform_user_id_hash(
    platform: &str,
    user_id: &str,
    owner_account: &[u8; 32],
) -> [u8; 32] {
    let salt = Sha256::new()
        .chain_update(owner_account)
        .chain_update(b"nexus-user-salt")
        .finalize();

    // 长度前缀防止域分离歧义
    let result = Sha256::new()
        .chain_update(&(platform.len() as u32).to_le_bytes())
        .chain_update(platform.as_bytes())
        .chain_update(&(user_id.len() as u32).to_le_bytes())
        .chain_update(user_id.as_bytes())
        .chain_update(&salt)
        .finalize();

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn community_hash_deterministic() {
        let owner = [1u8; 32];
        let h1 = compute_community_id_hash("telegram", -1001234567890, &owner);
        let h2 = compute_community_id_hash("telegram", -1001234567890, &owner);
        assert_eq!(h1, h2);
    }

    #[test]
    fn community_hash_different_owners() {
        let owner_a = [1u8; 32];
        let owner_b = [2u8; 32];
        let h1 = compute_community_id_hash("telegram", -1001234567890, &owner_a);
        let h2 = compute_community_id_hash("telegram", -1001234567890, &owner_b);
        assert_ne!(h1, h2, "different owners should produce different hashes");
    }

    #[test]
    fn community_hash_different_platforms() {
        let owner = [1u8; 32];
        let h1 = compute_community_id_hash("telegram", 12345, &owner);
        let h2 = compute_community_id_hash("discord", 12345, &owner);
        assert_ne!(h1, h2);
    }

    #[test]
    fn community_hash_different_chat_ids() {
        let owner = [1u8; 32];
        let h1 = compute_community_id_hash("telegram", 111, &owner);
        let h2 = compute_community_id_hash("telegram", 222, &owner);
        assert_ne!(h1, h2);
    }

    #[test]
    fn user_hash_deterministic() {
        let owner = [1u8; 32];
        let h1 = compute_platform_user_id_hash("telegram", "123456789", &owner);
        let h2 = compute_platform_user_id_hash("telegram", "123456789", &owner);
        assert_eq!(h1, h2);
    }

    #[test]
    fn user_hash_different_owners() {
        let owner_a = [1u8; 32];
        let owner_b = [2u8; 32];
        let h1 = compute_platform_user_id_hash("telegram", "123", &owner_a);
        let h2 = compute_platform_user_id_hash("telegram", "123", &owner_b);
        assert_ne!(h1, h2);
    }

    #[test]
    fn user_hash_different_users() {
        let owner = [1u8; 32];
        let h1 = compute_platform_user_id_hash("telegram", "111", &owner);
        let h2 = compute_platform_user_id_hash("telegram", "222", &owner);
        assert_ne!(h1, h2);
    }
}
