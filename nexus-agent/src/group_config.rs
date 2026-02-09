use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

use crate::AppState;

// ═══════════════════════════════════════════════════════════════
// 群配置类型（与 nexus-node/src/types.rs 保持一致）
// ═══════════════════════════════════════════════════════════════

/// 入群审批策略
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum JoinApprovalPolicy {
    #[default]
    AutoApprove,
    ManualApproval,
    CaptchaRequired,
    TokenGating {
        min_balance: u64,
        asset_id: String,
    },
}

/// 防刷屏动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FloodAction {
    Mute,
    Kick,
    Ban,
    DeleteOnly,
}

impl Default for FloodAction {
    fn default() -> Self { Self::Mute }
}

/// 警告超限动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WarnAction {
    Ban,
    Kick,
    Mute,
}

impl Default for WarnAction {
    fn default() -> Self { Self::Ban }
}

/// 黑名单匹配模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlacklistMode {
    Exact,
    Contains,
    Regex,
}

impl Default for BlacklistMode {
    fn default() -> Self { Self::Contains }
}

/// 黑名单触发动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlacklistAction {
    Delete,
    DeleteAndWarn,
    DeleteAndMute,
    DeleteAndBan,
}

impl Default for BlacklistAction {
    fn default() -> Self { Self::Delete }
}

/// 可锁定的消息类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum LockType {
    Audio,
    Video,
    Photo,
    Document,
    Sticker,
    Gif,
    Url,
    Forward,
    Voice,
    Contact,
    Location,
    Poll,
    Game,
    Inline,
}

/// 群配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    #[serde(default)]
    pub version: u64,
    #[serde(default)]
    pub bot_id_hash: String,
    #[serde(default)]
    pub join_policy: JoinApprovalPolicy,
    #[serde(default)]
    pub filter_links: bool,
    #[serde(default)]
    pub restrict_mentions: bool,
    #[serde(default)]
    pub rate_limit_per_minute: u16,
    #[serde(default)]
    pub auto_mute_duration: u64,
    #[serde(default)]
    pub new_member_restrict_duration: u64,
    #[serde(default)]
    pub welcome_message: String,
    #[serde(default)]
    pub whitelist: Vec<String>,
    #[serde(default)]
    pub admins: Vec<String>,
    #[serde(default)]
    pub quiet_hours_start: Option<u8>,
    #[serde(default)]
    pub quiet_hours_end: Option<u8>,
    #[serde(default)]
    pub updated_at: u64,

    // ═══ Bot 功能扩展字段 ═══

    #[serde(default)]
    pub antiflood_limit: u16,
    #[serde(default = "default_antiflood_window")]
    pub antiflood_window: u16,
    #[serde(default)]
    pub antiflood_action: FloodAction,

    #[serde(default = "default_warn_limit")]
    pub warn_limit: u8,
    #[serde(default)]
    pub warn_action: WarnAction,

    #[serde(default)]
    pub blacklist_words: Vec<String>,
    #[serde(default)]
    pub blacklist_mode: BlacklistMode,
    #[serde(default)]
    pub blacklist_action: BlacklistAction,

    #[serde(default)]
    pub lock_types: Vec<LockType>,

    #[serde(default)]
    pub spam_detection_enabled: bool,
    #[serde(default)]
    pub spam_max_emoji: u8,
    #[serde(default)]
    pub spam_first_messages_only: u8,
}

fn default_antiflood_window() -> u16 { 10 }
fn default_warn_limit() -> u8 { 3 }

/// 签名的群配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedGroupConfig {
    pub config: GroupConfig,
    pub signature: String,
    pub signer_public_key: String,
}

// ═══════════════════════════════════════════════════════════════
// API 请求/响应类型
// ═══════════════════════════════════════════════════════════════

/// POST /v1/group-config 请求体
#[derive(Debug, Deserialize)]
pub struct UpdateGroupConfigRequest {
    pub join_policy: Option<JoinApprovalPolicy>,
    pub filter_links: Option<bool>,
    pub restrict_mentions: Option<bool>,
    pub rate_limit_per_minute: Option<u16>,
    pub auto_mute_duration: Option<u64>,
    pub new_member_restrict_duration: Option<u64>,
    pub welcome_message: Option<String>,
    pub whitelist: Option<Vec<String>>,
    pub admins: Option<Vec<String>>,
    pub quiet_hours_start: Option<u8>,
    pub quiet_hours_end: Option<u8>,
    // Bot 功能扩展
    pub antiflood_limit: Option<u16>,
    pub antiflood_window: Option<u16>,
    pub antiflood_action: Option<FloodAction>,
    pub warn_limit: Option<u8>,
    pub warn_action: Option<WarnAction>,
    pub blacklist_words: Option<Vec<String>>,
    pub blacklist_mode: Option<BlacklistMode>,
    pub blacklist_action: Option<BlacklistAction>,
    pub lock_types: Option<Vec<LockType>>,
    pub spam_detection_enabled: Option<bool>,
    pub spam_max_emoji: Option<u8>,
    pub spam_first_messages_only: Option<u8>,
    /// 钱包签名认证 (hex): sign(bot_id_hash + timestamp)
    pub auth_signature: String,
    /// 签名时间戳
    pub auth_timestamp: u64,
    /// 签名者公钥 (hex)
    pub auth_public_key: String,
}

/// GET /v1/group-config 响应
#[derive(Debug, Serialize)]
pub struct GroupConfigResponse {
    pub config: Option<GroupConfig>,
    pub version: u64,
    pub signed: bool,
}

/// POST /v1/group-config 响应
#[derive(Debug, Serialize)]
pub struct UpdateGroupConfigResponse {
    pub success: bool,
    pub version: u64,
    pub error: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// 配置存储（Agent 本地）
// ═══════════════════════════════════════════════════════════════

use std::sync::RwLock;
use std::path::PathBuf;

pub struct ConfigStore {
    current: RwLock<Option<SignedGroupConfig>>,
    data_dir: PathBuf,
}

impl ConfigStore {
    pub fn new(data_dir: &str) -> Self {
        let store = Self {
            current: RwLock::new(None),
            data_dir: PathBuf::from(data_dir),
        };
        store.load_from_disk();
        store
    }

    pub fn get(&self) -> Option<SignedGroupConfig> {
        self.current.read().unwrap().clone()
    }

    pub fn get_version(&self) -> u64 {
        self.current.read().unwrap()
            .as_ref()
            .map(|c| c.config.version)
            .unwrap_or(0)
    }

    /// 设置群配置（CAS 版本检查，防止并发覆盖）
    ///
    /// `expected_version`: 调用方读取时的版本号，必须与当前版本匹配
    pub fn set(&self, config: SignedGroupConfig, expected_version: u64) -> Result<(), String> {
        let mut guard = self.current.write()
            .unwrap_or_else(|e| e.into_inner());
        let current_ver = guard.as_ref().map(|c| c.config.version).unwrap_or(0);
        if current_ver != expected_version {
            return Err(format!(
                "版本冲突: 期望 {}, 当前 {} (可能被并发请求更新)",
                expected_version, current_ver
            ));
        }
        if let Err(e) = self.persist_to_disk(&config) {
            warn!(error = %e, "群配置持久化失败");
        }
        *guard = Some(config);
        Ok(())
    }

    fn persist_to_disk(&self, config: &SignedGroupConfig) -> Result<(), String> {
        let path = self.data_dir.join("group_config.json");
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Create dir failed: {}", e))?;
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Serialize failed: {}", e))?;
        atomicwrites::AtomicFile::new(
            &path,
            atomicwrites::OverwriteBehavior::AllowOverwrite,
        )
        .write(|f| {
            use std::io::Write;
            f.write_all(json.as_bytes())
        })
        .map_err(|e| format!("Atomic write failed: {}", e))?;
        Ok(())
    }

    fn load_from_disk(&self) {
        let path = self.data_dir.join("group_config.json");
        if !path.exists() {
            return;
        }
        match std::fs::read_to_string(&path) {
            Ok(json) => match serde_json::from_str::<SignedGroupConfig>(&json) {
                Ok(config) => {
                    info!(version = config.config.version, "从本地恢复群配置");
                    *self.current.write().unwrap() = Some(config);
                }
                Err(e) => warn!(error = %e, "解析本地群配置失败"),
            },
            Err(e) => warn!(error = %e, "读取本地群配置失败"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// API Handlers
// ═══════════════════════════════════════════════════════════════

/// GET /v1/group-config — 获取当前群配置
pub async fn handle_get_config(
    State(state): State<Arc<AppState>>,
) -> Json<GroupConfigResponse> {
    let signed = state.config_store.get();
    let version = signed.as_ref().map(|c| c.config.version).unwrap_or(0);
    Json(GroupConfigResponse {
        config: signed.as_ref().map(|c| c.config.clone()),
        version,
        signed: signed.is_some(),
    })
}

/// POST /v1/group-config — 更新群配置
///
/// 流程:
/// 1. 验证请求者身份 (Ed25519 签名 == Bot owner)
/// 2. 合并配置字段
/// 3. Agent 签名新配置
/// 4. 存储 + 广播到所有 Node (via multicast)
pub async fn handle_update_config(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateGroupConfigRequest>,
) -> (StatusCode, Json<UpdateGroupConfigResponse>) {
    // 1. 验证请求者身份
    if let Err(e) = verify_config_auth(&state, &req) {
        warn!(error = %e, "群配置更新认证失败");
        return (StatusCode::FORBIDDEN, Json(UpdateGroupConfigResponse {
            success: false,
            version: 0,
            error: Some(e),
        }));
    }

    // 2. 合并配置
    let current_version = state.config_store.get_version();
    let new_version = current_version + 1;
    let bot_id_hash = state.config.bot_id_hash_hex();
    let now = chrono::Utc::now().timestamp() as u64;

    let base = state.config_store.get()
        .map(|c| c.config)
        .unwrap_or_else(|| GroupConfig {
            version: 0,
            bot_id_hash: bot_id_hash.clone(),
            join_policy: JoinApprovalPolicy::AutoApprove,
            filter_links: false,
            restrict_mentions: false,
            rate_limit_per_minute: 0,
            auto_mute_duration: 300,
            new_member_restrict_duration: 0,
            welcome_message: String::new(),
            whitelist: vec![],
            admins: vec![],
            quiet_hours_start: None,
            quiet_hours_end: None,
            updated_at: now,
            antiflood_limit: 0,
            antiflood_window: default_antiflood_window(),
            antiflood_action: FloodAction::default(),
            warn_limit: default_warn_limit(),
            warn_action: WarnAction::default(),
            blacklist_words: vec![],
            blacklist_mode: BlacklistMode::default(),
            blacklist_action: BlacklistAction::default(),
            lock_types: vec![],
            spam_detection_enabled: false,
            spam_max_emoji: 0,
            spam_first_messages_only: 0,
        });

    let new_config = GroupConfig {
        version: new_version,
        bot_id_hash: bot_id_hash.clone(),
        join_policy: req.join_policy.unwrap_or(base.join_policy),
        filter_links: req.filter_links.unwrap_or(base.filter_links),
        restrict_mentions: req.restrict_mentions.unwrap_or(base.restrict_mentions),
        rate_limit_per_minute: req.rate_limit_per_minute.unwrap_or(base.rate_limit_per_minute),
        auto_mute_duration: req.auto_mute_duration.unwrap_or(base.auto_mute_duration),
        new_member_restrict_duration: req.new_member_restrict_duration.unwrap_or(base.new_member_restrict_duration),
        welcome_message: req.welcome_message.unwrap_or(base.welcome_message),
        whitelist: req.whitelist.unwrap_or(base.whitelist),
        admins: req.admins.unwrap_or(base.admins),
        // quiet_hours: 值 255 表示清除，None 表示保持原值
        quiet_hours_start: match req.quiet_hours_start {
            Some(255) => None,
            Some(v) => Some(v),
            None => base.quiet_hours_start,
        },
        quiet_hours_end: match req.quiet_hours_end {
            Some(255) => None,
            Some(v) => Some(v),
            None => base.quiet_hours_end,
        },
        updated_at: now,
        antiflood_limit: req.antiflood_limit.unwrap_or(base.antiflood_limit),
        antiflood_window: req.antiflood_window.unwrap_or(base.antiflood_window),
        antiflood_action: req.antiflood_action.unwrap_or(base.antiflood_action),
        warn_limit: req.warn_limit.unwrap_or(base.warn_limit),
        warn_action: req.warn_action.unwrap_or(base.warn_action),
        blacklist_words: req.blacklist_words.unwrap_or(base.blacklist_words),
        blacklist_mode: req.blacklist_mode.unwrap_or(base.blacklist_mode),
        blacklist_action: req.blacklist_action.unwrap_or(base.blacklist_action),
        lock_types: req.lock_types.unwrap_or(base.lock_types),
        spam_detection_enabled: req.spam_detection_enabled.unwrap_or(base.spam_detection_enabled),
        spam_max_emoji: req.spam_max_emoji.unwrap_or(base.spam_max_emoji),
        spam_first_messages_only: req.spam_first_messages_only.unwrap_or(base.spam_first_messages_only),
    };

    // 3. Agent 签名
    let config_json = match serde_json::to_string(&new_config) {
        Ok(j) => j,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(UpdateGroupConfigResponse {
                success: false,
                version: 0,
                error: Some(format!("Config serialization failed: {}", e)),
            }));
        }
    };

    let signature = state.key_manager.sign_raw(config_json.as_bytes());
    let signed_config = SignedGroupConfig {
        config: new_config,
        signature: hex::encode(signature),
        signer_public_key: state.key_manager.public_key_hex(),
    };

    // 4. 存储 (CAS: 检查版本未被并发修改)
    if let Err(e) = state.config_store.set(signed_config.clone(), current_version) {
        warn!(error = %e, "群配置存储失败");
        return (StatusCode::CONFLICT, Json(UpdateGroupConfigResponse {
            success: false,
            version: 0,
            error: Some(e),
        }));
    }
    info!(version = new_version, "群配置已更新 v{}", new_version);

    // 5. 异步广播到所有 Node
    let state_clone = state.clone();
    let bot_id = bot_id_hash.clone();
    let config_for_broadcast = signed_config.clone();
    tokio::spawn(async move {
        broadcast_config_to_nodes(&state_clone, &bot_id, config_for_broadcast).await;
    });

    (StatusCode::OK, Json(UpdateGroupConfigResponse {
        success: true,
        version: new_version,
        error: None,
    }))
}

/// 验证配置更新请求的认证
fn verify_config_auth(
    state: &Arc<AppState>,
    req: &UpdateGroupConfigRequest,
) -> Result<(), String> {
    use ed25519_dalek::{VerifyingKey, Verifier, Signature};

    // 检查时间戳（5 分钟窗口）
    let now = chrono::Utc::now().timestamp() as u64;
    if now.abs_diff(req.auth_timestamp) > 300 {
        return Err("认证时间戳过期 (>5分钟)".into());
    }

    // 验证公钥匹配 Bot owner
    let pk_bytes = hex::decode(&req.auth_public_key)
        .map_err(|e| format!("Invalid auth public key hex: {}", e))?;
    if pk_bytes.len() != 32 {
        return Err("Auth public key must be 32 bytes".into());
    }

    // 验证公钥 == Agent 公钥 (Bot owner)
    let agent_pk = state.key_manager.public_key_hex();
    if req.auth_public_key != agent_pk {
        return Err(format!(
            "Auth public key {} does not match Agent key {}",
            req.auth_public_key, agent_pk
        ));
    }

    // 验证签名: sign(bot_id_hash + timestamp_le_bytes)
    let pk_array: [u8; 32] = pk_bytes.try_into()
        .map_err(|_| "Public key conversion failed")?;
    let verifying_key = VerifyingKey::from_bytes(&pk_array)
        .map_err(|e| format!("Invalid Ed25519 key: {}", e))?;

    let sig_bytes = hex::decode(&req.auth_signature)
        .map_err(|e| format!("Invalid auth signature hex: {}", e))?;
    let sig_array: [u8; 64] = sig_bytes.try_into()
        .map_err(|_| "Signature must be 64 bytes")?;
    let signature = Signature::from_bytes(&sig_array);

    let bot_id = state.config.bot_id_hash_hex();
    let mut sign_data = Vec::new();
    sign_data.extend_from_slice(bot_id.as_bytes());
    sign_data.extend_from_slice(&req.auth_timestamp.to_le_bytes());

    verifying_key.verify(&sign_data, &signature)
        .map_err(|e| format!("Auth signature verification failed: {}", e))?;

    Ok(())
}

/// 广播配置到所有 Node
pub async fn broadcast_config_to_nodes(
    state: &Arc<AppState>,
    bot_id_hash: &str,
    signed_config: SignedGroupConfig,
) {
    let nodes = state.nodes.read().await;
    if nodes.is_empty() {
        warn!("无可用节点，跳过配置广播");
        return;
    }

    // ConfigSync 载荷
    let payload = serde_json::json!({
        "version": 1,
        "msg_type": "ConfigSync",
        "sender_node_id": "agent",
        "timestamp": chrono::Utc::now().timestamp_millis() as u64,
        "payload": {
            "bot_id_hash": bot_id_hash,
            "signed_config": signed_config,
        },
        "sender_signature": ""
    });

    let mut success = 0;
    let mut failure = 0;

    for node in nodes.iter() {
        let url = format!("{}/v1/message", node.endpoint);
        match state.http_client.post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                success += 1;
            }
            Ok(resp) => {
                warn!(node = %node.node_id, status = %resp.status(), "ConfigSync 广播失败");
                failure += 1;
            }
            Err(e) => {
                warn!(node = %node.node_id, error = %e, "ConfigSync 广播失败");
                failure += 1;
            }
        }
    }

    info!(
        bot = bot_id_hash,
        success,
        failure,
        total = nodes.len(),
        "ConfigSync 广播完成"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signed_config(version: u64) -> SignedGroupConfig {
        SignedGroupConfig {
            config: GroupConfig {
                version,
                bot_id_hash: "test_bot".into(),
                join_policy: JoinApprovalPolicy::AutoApprove,
                filter_links: false,
                restrict_mentions: false,
                rate_limit_per_minute: 0,
                auto_mute_duration: 300,
                new_member_restrict_duration: 0,
                welcome_message: String::new(),
                whitelist: vec![],
                admins: vec![],
                quiet_hours_start: None,
                quiet_hours_end: None,
                updated_at: 0,
                antiflood_limit: 0,
                antiflood_window: 10,
                antiflood_action: FloodAction::default(),
                warn_limit: 3,
                warn_action: WarnAction::default(),
                blacklist_words: vec![],
                blacklist_mode: BlacklistMode::default(),
                blacklist_action: BlacklistAction::default(),
                lock_types: vec![],
                spam_detection_enabled: false,
                spam_max_emoji: 0,
                spam_first_messages_only: 0,
            },
            signature: "sig".into(),
            signer_public_key: "pk".into(),
        }
    }

    // ═══════════════════════════════════════════
    // ConfigStore 基础操作
    // ═══════════════════════════════════════════

    #[test]
    fn test_config_store_empty() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::new(dir.path().to_str().unwrap());
        assert!(store.get().is_none());
        assert_eq!(store.get_version(), 0);
    }

    #[test]
    fn test_config_store_set_and_get() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::new(dir.path().to_str().unwrap());

        let config = make_signed_config(1);
        store.set(config.clone(), 0).unwrap();

        let got = store.get().unwrap();
        assert_eq!(got.config.version, 1);
        assert_eq!(store.get_version(), 1);
    }

    #[test]
    fn test_config_store_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        {
            let store = ConfigStore::new(path);
            store.set(make_signed_config(5), 0).unwrap();
        }

        // 重新加载
        let store2 = ConfigStore::new(path);
        assert_eq!(store2.get_version(), 5, "应从磁盘恢复版本号");
    }

    // ═══════════════════════════════════════════
    // CAS (Compare-And-Swap) 版本检查
    // ═══════════════════════════════════════════

    #[test]
    fn test_cas_success() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::new(dir.path().to_str().unwrap());

        // v0 → v1
        store.set(make_signed_config(1), 0).unwrap();
        assert_eq!(store.get_version(), 1);

        // v1 → v2
        store.set(make_signed_config(2), 1).unwrap();
        assert_eq!(store.get_version(), 2);
    }

    #[test]
    fn test_cas_version_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::new(dir.path().to_str().unwrap());

        store.set(make_signed_config(1), 0).unwrap();

        // 试图用 expected_version=0 再次写入 → 冲突
        let result = store.set(make_signed_config(2), 0);
        assert!(result.is_err(), "版本冲突应返回错误");
        assert!(result.unwrap_err().contains("版本冲突"));

        // 原值应保持不变
        assert_eq!(store.get_version(), 1);
    }

    #[test]
    fn test_cas_stale_read() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::new(dir.path().to_str().unwrap());

        store.set(make_signed_config(1), 0).unwrap();
        store.set(make_signed_config(2), 1).unwrap();

        // 试图用过期的 expected_version=1 写入
        let result = store.set(make_signed_config(3), 1);
        assert!(result.is_err());
        assert_eq!(store.get_version(), 2, "CAS 失败不应修改数据");
    }

    // ═══════════════════════════════════════════
    // GroupConfig 序列化
    // ═══════════════════════════════════════════

    #[test]
    fn test_group_config_serde_roundtrip() {
        let config = make_signed_config(42);
        let json = serde_json::to_string(&config).unwrap();
        let back: SignedGroupConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.config.version, 42);
        assert_eq!(back.config.bot_id_hash, "test_bot");
        assert_eq!(back.signature, "sig");
    }

    #[test]
    fn test_group_config_defaults() {
        // 最小 JSON — 所有字段都有 serde(default)，应全部成功
        let json = r#"{}"#;
        let config: GroupConfig = serde_json::from_str(json).unwrap();
        // 基础字段默认值
        assert_eq!(config.version, 0);
        assert!(config.bot_id_hash.is_empty());
        assert!(matches!(config.join_policy, JoinApprovalPolicy::AutoApprove));
        assert!(!config.filter_links);
        assert!(config.quiet_hours_start.is_none());
        // Bot 扩展字段默认值
        assert_eq!(config.antiflood_window, 10, "default_antiflood_window");
        assert_eq!(config.warn_limit, 3, "default_warn_limit");
        assert_eq!(config.antiflood_limit, 0);
        assert!(!config.spam_detection_enabled);
        assert!(config.blacklist_words.is_empty());
        assert!(config.lock_types.is_empty());
    }

    // ═══════════════════════════════════════════
    // quiet_hours sentinel 清除
    // ═══════════════════════════════════════════

    #[test]
    fn test_quiet_hours_clear_sentinel() {
        // 模拟 quiet_hours 合并逻辑中 255 清除行为
        let req_start: Option<u8> = Some(255);
        let base_start: Option<u8> = Some(22);

        let result = match req_start {
            Some(255) => None,
            Some(v) => Some(v),
            None => base_start,
        };
        assert!(result.is_none(), "255 应清除 quiet_hours");
    }

    #[test]
    fn test_quiet_hours_preserve() {
        let req_start: Option<u8> = None;
        let base_start: Option<u8> = Some(22);

        let result = match req_start {
            Some(255) => None,
            Some(v) => Some(v),
            None => base_start,
        };
        assert_eq!(result, Some(22), "None 应保留原值");
    }

    #[test]
    fn test_quiet_hours_set_new() {
        let req_start: Option<u8> = Some(8);
        let base_start: Option<u8> = Some(22);

        let result = match req_start {
            Some(255) => None,
            Some(v) => Some(v),
            None => base_start,
        };
        assert_eq!(result, Some(8), "新值应覆盖");
    }
}
