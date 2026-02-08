use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;
use tracing::{info, warn, debug, error};

use crate::types::{BotInfoCache, NodeInfoCache, SignedGroupConfig, GroupConfig};

/// 链上数据缓存（从链上读取，本地缓存）
///
/// TODO: Sprint 3 Week 3 — 使用 subxt 订阅链上事件自动刷新
/// 当前: 从环境变量加载静态数据（开发用）
pub struct ChainCache {
    /// Bot 注册信息: bot_id_hash → BotInfoCache
    bots: RwLock<HashMap<String, BotInfoCache>>,

    /// 活跃节点列表: node_id → NodeInfoCache
    nodes: RwLock<HashMap<String, NodeInfoCache>>,

    /// 群配置缓存: bot_id_hash → SignedGroupConfig
    group_configs: RwLock<HashMap<String, SignedGroupConfig>>,

    /// 本地持久化目录
    data_dir: PathBuf,
}

impl ChainCache {
    pub fn new() -> Self {
        let data_dir = std::env::var("DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp/nexus-node"));
        Self {
            bots: RwLock::new(HashMap::new()),
            nodes: RwLock::new(HashMap::new()),
            group_configs: RwLock::new(HashMap::new()),
            data_dir,
        }
    }

    /// 使用指定 data_dir 创建（测试用）
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        Self {
            bots: RwLock::new(HashMap::new()),
            nodes: RwLock::new(HashMap::new()),
            group_configs: RwLock::new(HashMap::new()),
            data_dir,
        }
    }

    /// 从环境变量初始化（开发用）
    ///
    /// BOT_REGISTRATIONS=bot_id_hash:owner_pubkey:platform,...
    /// NODE_LIST=node_id:endpoint:pubkey,...
    pub fn load_from_env(&self) {
        // 加载 Bot 注册
        if let Ok(bots_str) = std::env::var("BOT_REGISTRATIONS") {
            let mut bots = self.bots.write().unwrap();
            for entry in bots_str.split(',') {
                let parts: Vec<&str> = entry.split(':').collect();
                if parts.len() >= 2 {
                    let bot_id_hash = parts[0].to_string();
                    let owner_pk_hex = parts[1];
                    let platform = parts.get(2).unwrap_or(&"telegram").to_string();

                    let mut pk = [0u8; 32];
                    if let Ok(bytes) = hex::decode(owner_pk_hex) {
                        if bytes.len() == 32 {
                            pk.copy_from_slice(&bytes);
                        }
                    }

                    bots.insert(bot_id_hash.clone(), BotInfoCache {
                        bot_id_hash,
                        owner_public_key: pk,
                        platform,
                        is_active: true,
                    });
                }
            }
            info!(count = bots.len(), "Bot 注册缓存已加载");
        }

        // 加载节点列表
        if let Ok(nodes_str) = std::env::var("NODE_LIST") {
            let mut nodes = self.nodes.write().unwrap();
            for entry in nodes_str.split(',') {
                let parts: Vec<&str> = entry.split('@').collect();
                if parts.len() >= 2 {
                    let node_id = parts[0].to_string();
                    let endpoint = parts[1].to_string();

                    nodes.insert(node_id.clone(), NodeInfoCache {
                        node_id: node_id.clone(),
                        endpoint,
                        node_public_key: [0u8; 32],
                        status: "Active".to_string(),
                        reputation: 5000,
                    });
                }
            }
            info!(count = nodes.len(), "节点列表缓存已加载");
        }
    }

    /// 获取 Bot 信息
    pub fn get_bot(&self, bot_id_hash: &str) -> Option<BotInfoCache> {
        self.bots.read().unwrap().get(bot_id_hash).cloned()
    }

    /// 获取活跃节点 ID 列表（排序后）
    pub fn get_active_node_ids(&self) -> Vec<String> {
        let nodes = self.nodes.read().unwrap();
        let mut ids: Vec<String> = nodes.keys().cloned().collect();
        ids.sort();
        ids
    }

    /// 获取节点信息
    pub fn get_node(&self, node_id: &str) -> Option<NodeInfoCache> {
        self.nodes.read().unwrap().get(node_id).cloned()
    }

    /// 获取所有节点信息
    pub fn get_all_nodes(&self) -> Vec<NodeInfoCache> {
        self.nodes.read().unwrap().values().cloned().collect()
    }

    /// 注册 Bot（手动添加，开发调试用）
    pub fn register_bot(&self, info: BotInfoCache) {
        self.bots.write().unwrap().insert(info.bot_id_hash.clone(), info);
    }

    /// 添加节点
    pub fn add_node(&self, info: NodeInfoCache) {
        self.nodes.write().unwrap().insert(info.node_id.clone(), info);
    }

    /// 移除节点
    pub fn remove_node(&self, node_id: &str) {
        self.nodes.write().unwrap().remove(node_id);
    }

    // ═══════════════════════════════════════════════════════════════
    // 群配置管理 (Sprint 8)
    // ═══════════════════════════════════════════════════════════════

    /// 接收并验证签名群配置
    ///
    /// 验证流程:
    /// 1. 根据 bot_id_hash 查找 Bot 注册信息
    /// 2. 验证签名者公钥 == Bot owner_public_key
    /// 3. 验证 Ed25519 签名 (config JSON → signature)
    /// 4. 版本号单调递增检查
    /// 5. 存入缓存 + 持久化到本地 JSON
    pub fn apply_signed_config(&self, bot_id_hash: &str, signed_config: SignedGroupConfig) -> Result<(), String> {
        // 1. 查找 Bot 注册信息
        let bot_info = self.get_bot(bot_id_hash)
            .ok_or_else(|| format!("Bot {} not found in cache", bot_id_hash))?;

        // 2. 验证签名者公钥
        let signer_pk_bytes = hex::decode(&signed_config.signer_public_key)
            .map_err(|e| format!("Invalid signer public key hex: {}", e))?;
        if signer_pk_bytes.len() != 32 {
            return Err("Signer public key must be 32 bytes".into());
        }
        if signer_pk_bytes[..] != bot_info.owner_public_key[..] {
            return Err("Signer public key does not match Bot owner".into());
        }

        // 3. 验证 Ed25519 签名
        self.verify_config_signature(&signed_config)?;

        // 4. 版本号单调递增
        {
            let configs = self.group_configs.read().unwrap();
            if let Some(existing) = configs.get(bot_id_hash) {
                if signed_config.config.version <= existing.config.version {
                    return Err(format!(
                        "Config version {} <= existing {}",
                        signed_config.config.version, existing.config.version
                    ));
                }
            }
        }

        // 5. 存入缓存
        info!(
            bot = bot_id_hash,
            version = signed_config.config.version,
            "群配置已更新"
        );
        self.group_configs.write().unwrap()
            .insert(bot_id_hash.to_string(), signed_config.clone());

        // 6. 持久化到本地 JSON
        if let Err(e) = self.persist_config(bot_id_hash, &signed_config) {
            warn!(bot = bot_id_hash, error = %e, "群配置持久化失败");
        }

        Ok(())
    }

    /// 获取群配置
    pub fn get_group_config(&self, bot_id_hash: &str) -> Option<SignedGroupConfig> {
        self.group_configs.read().unwrap().get(bot_id_hash).cloned()
    }

    /// 获取群配置版本号
    pub fn get_config_version(&self, bot_id_hash: &str) -> u64 {
        self.group_configs.read().unwrap()
            .get(bot_id_hash)
            .map(|c| c.config.version)
            .unwrap_or(0)
    }

    /// 获取所有群配置的 bot_id_hash 列表
    pub fn get_config_bot_ids(&self) -> Vec<String> {
        self.group_configs.read().unwrap().keys().cloned().collect()
    }

    /// 验证 Ed25519 签名
    fn verify_config_signature(&self, signed_config: &SignedGroupConfig) -> Result<(), String> {
        use ed25519_dalek::{Verifier, VerifyingKey, Signature};

        let pk_bytes = hex::decode(&signed_config.signer_public_key)
            .map_err(|e| format!("Invalid public key hex: {}", e))?;
        let pk_array: [u8; 32] = pk_bytes.try_into()
            .map_err(|_| "Public key must be 32 bytes")?;
        let verifying_key = VerifyingKey::from_bytes(&pk_array)
            .map_err(|e| format!("Invalid Ed25519 public key: {}", e))?;

        let sig_bytes = hex::decode(&signed_config.signature)
            .map_err(|e| format!("Invalid signature hex: {}", e))?;
        let sig_array: [u8; 64] = sig_bytes.try_into()
            .map_err(|_| "Signature must be 64 bytes")?;
        let signature = Signature::from_bytes(&sig_array);

        let config_json = serde_json::to_string(&signed_config.config)
            .map_err(|e| format!("Config serialization error: {}", e))?;

        verifying_key.verify(config_json.as_bytes(), &signature)
            .map_err(|e| format!("Signature verification failed: {}", e))?;

        Ok(())
    }

    /// 持久化群配置到本地 JSON 文件
    fn persist_config(&self, bot_id_hash: &str, signed_config: &SignedGroupConfig) -> Result<(), String> {
        let config_dir = self.data_dir.join("configs");
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Create config dir failed: {}", e))?;

        let path = config_dir.join(format!("{}.json", bot_id_hash));
        let json = serde_json::to_string_pretty(signed_config)
            .map_err(|e| format!("Serialize config failed: {}", e))?;
        std::fs::write(&path, json)
            .map_err(|e| format!("Write config file failed: {}", e))?;

        debug!(bot = bot_id_hash, path = %path.display(), "群配置已持久化");
        Ok(())
    }

    /// 从本地 JSON 恢复群配置（节点启动时调用）
    pub fn load_configs_from_disk(&self) {
        let config_dir = self.data_dir.join("configs");
        if !config_dir.exists() {
            debug!("无本地群配置目录，跳过恢复");
            return;
        }

        let entries = match std::fs::read_dir(&config_dir) {
            Ok(e) => e,
            Err(e) => {
                warn!(error = %e, "读取群配置目录失败");
                return;
            }
        };

        let mut count = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            match std::fs::read_to_string(&path) {
                Ok(json) => match serde_json::from_str::<SignedGroupConfig>(&json) {
                    Ok(config) => {
                        let bot_id = config.config.bot_id_hash.clone();
                        self.group_configs.write().unwrap()
                            .insert(bot_id.clone(), config);
                        count += 1;
                        debug!(bot = bot_id, "从本地恢复群配置");
                    }
                    Err(e) => warn!(path = %path.display(), error = %e, "解析群配置 JSON 失败"),
                },
                Err(e) => warn!(path = %path.display(), error = %e, "读取群配置文件失败"),
            }
        }

        if count > 0 {
            info!(count, "已从本地恢复群配置");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GroupConfig, SignedGroupConfig, JoinApprovalPolicy, BotInfoCache};
    use ed25519_dalek::{SigningKey, Signer};
    use tempfile::TempDir;

    fn make_signing_key() -> SigningKey {
        let secret = [42u8; 32];
        SigningKey::from_bytes(&secret)
    }

    fn make_group_config(bot_id: &str, version: u64) -> GroupConfig {
        GroupConfig {
            version,
            bot_id_hash: bot_id.to_string(),
            join_policy: JoinApprovalPolicy::AutoApprove,
            filter_links: true,
            restrict_mentions: false,
            rate_limit_per_minute: 10,
            auto_mute_duration: 300,
            new_member_restrict_duration: 0,
            welcome_message: "Welcome!".into(),
            whitelist: vec![],
            admins: vec![],
            quiet_hours_start: None,
            quiet_hours_end: None,
            updated_at: 1000,
            antiflood_limit: 0,
            antiflood_window: 10,
            antiflood_action: crate::types::FloodAction::default(),
            warn_limit: 3,
            warn_action: crate::types::WarnAction::default(),
            blacklist_words: vec![],
            blacklist_mode: crate::types::BlacklistMode::default(),
            blacklist_action: crate::types::BlacklistAction::default(),
            lock_types: vec![],
            spam_detection_enabled: false,
            spam_max_emoji: 0,
            spam_first_messages_only: 0,
        }
    }

    fn sign_config(key: &SigningKey, config: &GroupConfig) -> SignedGroupConfig {
        let json = serde_json::to_string(config).unwrap();
        let sig = key.sign(json.as_bytes());
        SignedGroupConfig {
            config: config.clone(),
            signature: hex::encode(sig.to_bytes()),
            signer_public_key: hex::encode(key.verifying_key().as_bytes()),
        }
    }

    fn setup_cache_with_bot(key: &SigningKey) -> (ChainCache, TempDir) {
        let tmp = TempDir::new().unwrap();
        let cache = ChainCache::with_data_dir(tmp.path().to_path_buf());
        let pk = *key.verifying_key().as_bytes();
        cache.register_bot(BotInfoCache {
            bot_id_hash: "bot_abc".into(),
            owner_public_key: pk,
            platform: "telegram".into(),
            is_active: true,
        });
        (cache, tmp)
    }

    #[test]
    fn test_apply_signed_config_success() {
        let key = make_signing_key();
        let (cache, _tmp) = setup_cache_with_bot(&key);
        let config = make_group_config("bot_abc", 1);
        let signed = sign_config(&key, &config);

        assert!(cache.apply_signed_config("bot_abc", signed).is_ok());
        assert_eq!(cache.get_config_version("bot_abc"), 1);

        let stored = cache.get_group_config("bot_abc").unwrap();
        assert_eq!(stored.config.filter_links, true);
        assert_eq!(stored.config.welcome_message, "Welcome!");
    }

    #[test]
    fn test_version_must_increase() {
        let key = make_signing_key();
        let (cache, _tmp) = setup_cache_with_bot(&key);

        let c1 = make_group_config("bot_abc", 1);
        let s1 = sign_config(&key, &c1);
        assert!(cache.apply_signed_config("bot_abc", s1).is_ok());

        // Same version → rejected
        let c1b = make_group_config("bot_abc", 1);
        let s1b = sign_config(&key, &c1b);
        let err = cache.apply_signed_config("bot_abc", s1b).unwrap_err();
        assert!(err.contains("version"));

        // Lower version → rejected
        let c0 = make_group_config("bot_abc", 0);
        let s0 = sign_config(&key, &c0);
        assert!(cache.apply_signed_config("bot_abc", s0).is_err());

        // Higher version → accepted
        let c2 = make_group_config("bot_abc", 2);
        let s2 = sign_config(&key, &c2);
        assert!(cache.apply_signed_config("bot_abc", s2).is_ok());
        assert_eq!(cache.get_config_version("bot_abc"), 2);
    }

    #[test]
    fn test_wrong_signer_rejected() {
        let owner_key = make_signing_key();
        let (cache, _tmp) = setup_cache_with_bot(&owner_key);

        // Sign with a different key
        let wrong_key = SigningKey::from_bytes(&[99u8; 32]);
        let config = make_group_config("bot_abc", 1);
        let signed = sign_config(&wrong_key, &config);

        let err = cache.apply_signed_config("bot_abc", signed).unwrap_err();
        assert!(err.contains("does not match"));
    }

    #[test]
    fn test_invalid_signature_rejected() {
        let key = make_signing_key();
        let (cache, _tmp) = setup_cache_with_bot(&key);

        let config = make_group_config("bot_abc", 1);
        let mut signed = sign_config(&key, &config);
        // Corrupt signature
        signed.signature = hex::encode([0u8; 64]);

        let err = cache.apply_signed_config("bot_abc", signed).unwrap_err();
        assert!(err.contains("Signature verification failed"));
    }

    #[test]
    fn test_unknown_bot_rejected() {
        let key = make_signing_key();
        let (cache, _tmp) = setup_cache_with_bot(&key);

        let config = make_group_config("bot_unknown", 1);
        let signed = sign_config(&key, &config);

        let err = cache.apply_signed_config("bot_unknown", signed).unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_persist_and_recover() {
        let key = make_signing_key();
        let (cache, tmp) = setup_cache_with_bot(&key);

        let config = make_group_config("bot_abc", 5);
        let signed = sign_config(&key, &config);
        cache.apply_signed_config("bot_abc", signed).unwrap();

        // Verify JSON file was created
        let json_path = tmp.path().join("configs/bot_abc.json");
        assert!(json_path.exists());

        // Create a fresh cache from same dir → load_configs_from_disk
        let cache2 = ChainCache::with_data_dir(tmp.path().to_path_buf());
        assert_eq!(cache2.get_config_version("bot_abc"), 0); // not loaded yet
        cache2.load_configs_from_disk();
        assert_eq!(cache2.get_config_version("bot_abc"), 5);
        assert_eq!(
            cache2.get_group_config("bot_abc").unwrap().config.welcome_message,
            "Welcome!"
        );
    }

    #[test]
    fn test_get_config_bot_ids() {
        let key = make_signing_key();
        let (cache, _tmp) = setup_cache_with_bot(&key);

        assert!(cache.get_config_bot_ids().is_empty());

        let c1 = make_group_config("bot_abc", 1);
        let s1 = sign_config(&key, &c1);
        cache.apply_signed_config("bot_abc", s1).unwrap();

        let ids = cache.get_config_bot_ids();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&"bot_abc".to_string()));
    }
}
