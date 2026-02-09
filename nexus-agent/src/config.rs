use serde::Deserialize;

/// Agent 全局配置（从环境变量加载）
#[derive(Clone, Deserialize)]
pub struct AgentConfig {
    /// Telegram Bot Token（永不上链，只在本地内存）
    pub bot_token: String,

    /// Webhook 监听端口
    #[serde(default = "default_webhook_port")]
    pub webhook_port: u16,

    /// Webhook 外部 URL（用于 setWebhook）
    /// 例如: https://my-server.com:8443
    pub webhook_url: String,

    /// Webhook secret token（验证来自 Telegram 的请求）
    #[serde(default = "default_secret_token")]
    pub webhook_secret: String,

    /// 链上 RPC 端点
    #[serde(default = "default_chain_rpc")]
    pub chain_rpc: String,

    /// 数据目录（密钥、序列号等持久化文件）
    #[serde(default = "default_data_dir")]
    pub data_dir: String,

    /// 多播超时（毫秒）
    #[serde(default = "default_multicast_timeout_ms")]
    pub multicast_timeout_ms: u64,

    /// /v1/execute 端点的 Bearer Token 认证（可选，设置后强制验证）
    #[serde(default)]
    pub execute_token: Option<String>,

    /// bot_id_hash（SHA256(bot_token)，启动时自动计算）
    #[serde(skip)]
    pub bot_id_hash: [u8; 32],
}

fn default_webhook_port() -> u16 {
    8443
}

fn default_secret_token() -> String {
    hex::encode(rand::random::<[u8; 16]>())
}

fn default_chain_rpc() -> String {
    "ws://127.0.0.1:9944".to_string()
}

fn default_data_dir() -> String {
    "/data".to_string()
}

fn default_multicast_timeout_ms() -> u64 {
    3000
}

impl std::fmt::Debug for AgentConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentConfig")
            .field("bot_token", &"[REDACTED]")
            .field("webhook_port", &self.webhook_port)
            .field("webhook_url", &self.webhook_url)
            .field("webhook_secret", &"[REDACTED]")
            .field("chain_rpc", &self.chain_rpc)
            .field("data_dir", &self.data_dir)
            .field("multicast_timeout_ms", &self.multicast_timeout_ms)
            .field("execute_token", &self.execute_token.as_ref().map(|_| "[REDACTED]"))
            .field("bot_id_hash", &hex::encode(self.bot_id_hash))
            .finish()
    }
}

impl AgentConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> anyhow::Result<Self> {
        let bot_token = std::env::var("BOT_TOKEN")
            .map_err(|_| anyhow::anyhow!("BOT_TOKEN 环境变量未设置"))?;

        let webhook_port = std::env::var("WEBHOOK_PORT")
            .unwrap_or_else(|_| "8443".to_string())
            .parse::<u16>()
            .unwrap_or(8443);

        let webhook_url = std::env::var("WEBHOOK_URL")
            .map_err(|_| anyhow::anyhow!("WEBHOOK_URL 环境变量未设置（必填，Telegram 需要可达的外部 URL）"))?;

        let webhook_secret = std::env::var("WEBHOOK_SECRET")
            .unwrap_or_else(|_| default_secret_token());

        let chain_rpc = std::env::var("CHAIN_RPC")
            .unwrap_or_else(|_| default_chain_rpc());

        let data_dir = std::env::var("DATA_DIR")
            .unwrap_or_else(|_| default_data_dir());

        let multicast_timeout_ms = std::env::var("MULTICAST_TIMEOUT_MS")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u64>()
            .unwrap_or(3000);

        let execute_token = std::env::var("EXECUTE_TOKEN").ok()
            .filter(|s| !s.is_empty());

        // 计算 bot_id_hash = SHA256(bot_token)
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(bot_token.as_bytes());
        let hash = hasher.finalize();
        let mut bot_id_hash = [0u8; 32];
        bot_id_hash.copy_from_slice(&hash);

        Ok(Self {
            bot_token,
            webhook_port,
            webhook_url,
            webhook_secret,
            chain_rpc,
            data_dir,
            multicast_timeout_ms,
            execute_token,
            bot_id_hash,
        })
    }

    /// 获取 bot_id_hash 的 hex 字符串
    pub fn bot_id_hash_hex(&self) -> String {
        hex::encode(self.bot_id_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // 环境变量测试需要串行化（共享进程环境）
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env<F: FnOnce() -> R, R>(vars: &[(&str, &str)], remove: &[&str], f: F) -> R {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // 保存旧值
        let mut all_keys: Vec<&str> = vars.iter().map(|(k, _)| *k).collect();
        all_keys.extend_from_slice(remove);
        let saved: Vec<_> = all_keys.iter()
            .map(|k| (*k, std::env::var(k).ok()))
            .collect();
        // 设置新值
        for (k, v) in vars {
            std::env::set_var(k, v);
        }
        for k in remove {
            std::env::remove_var(k);
        }
        let result = f();
        // 恢复
        for (k, old) in saved {
            match old {
                Some(v) => std::env::set_var(k, v),
                None => std::env::remove_var(k),
            }
        }
        result
    }

    #[test]
    fn test_from_env_missing_bot_token() {
        with_env(&[], &["BOT_TOKEN", "WEBHOOK_URL"], || {
            let result = AgentConfig::from_env();
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("BOT_TOKEN"));
        });
    }

    #[test]
    fn test_from_env_missing_webhook_url() {
        with_env(
            &[("BOT_TOKEN", "test:token123")],
            &["WEBHOOK_URL"],
            || {
                let result = AgentConfig::from_env();
                assert!(result.is_err());
                assert!(result.unwrap_err().to_string().contains("WEBHOOK_URL"));
            },
        );
    }

    #[test]
    fn test_from_env_defaults() {
        with_env(
            &[("BOT_TOKEN", "test:token"), ("WEBHOOK_URL", "https://example.com")],
            &["WEBHOOK_PORT", "CHAIN_RPC", "DATA_DIR", "MULTICAST_TIMEOUT_MS", "EXECUTE_TOKEN", "WEBHOOK_SECRET"],
            || {
                let config = AgentConfig::from_env().unwrap();
                assert_eq!(config.webhook_port, 8443);
                assert_eq!(config.chain_rpc, "ws://127.0.0.1:9944");
                assert_eq!(config.data_dir, "/data");
                assert_eq!(config.multicast_timeout_ms, 3000);
                assert!(config.execute_token.is_none());
                assert!(!config.webhook_secret.is_empty(), "应自动生成 secret");
            },
        );
    }

    #[test]
    fn test_from_env_custom_values() {
        with_env(
            &[
                ("BOT_TOKEN", "123:ABC"),
                ("WEBHOOK_URL", "https://my.host:443"),
                ("WEBHOOK_PORT", "9999"),
                ("CHAIN_RPC", "wss://rpc.example.com"),
                ("DATA_DIR", "/tmp/test"),
                ("MULTICAST_TIMEOUT_MS", "5000"),
                ("EXECUTE_TOKEN", "secret_bearer"),
            ],
            &[],
            || {
                let config = AgentConfig::from_env().unwrap();
                assert_eq!(config.bot_token, "123:ABC");
                assert_eq!(config.webhook_url, "https://my.host:443");
                assert_eq!(config.webhook_port, 9999);
                assert_eq!(config.chain_rpc, "wss://rpc.example.com");
                assert_eq!(config.data_dir, "/tmp/test");
                assert_eq!(config.multicast_timeout_ms, 5000);
                assert_eq!(config.execute_token, Some("secret_bearer".into()));
            },
        );
    }

    #[test]
    fn test_bot_id_hash_deterministic() {
        with_env(
            &[("BOT_TOKEN", "same_token"), ("WEBHOOK_URL", "https://a.com")],
            &["EXECUTE_TOKEN"],
            || {
                let c1 = AgentConfig::from_env().unwrap();
                let c2 = AgentConfig::from_env().unwrap();
                assert_eq!(c1.bot_id_hash, c2.bot_id_hash);
                assert_eq!(c1.bot_id_hash_hex().len(), 64);
            },
        );
    }

    #[test]
    fn test_debug_redacts_secrets() {
        with_env(
            &[("BOT_TOKEN", "SUPER_SECRET_TOKEN"), ("WEBHOOK_URL", "https://a.com")],
            &["EXECUTE_TOKEN"],
            || {
                let config = AgentConfig::from_env().unwrap();
                let debug_str = format!("{:?}", config);
                assert!(!debug_str.contains("SUPER_SECRET_TOKEN"), "Debug 不应包含 bot_token");
                assert!(debug_str.contains("[REDACTED]"), "Debug 应显示 [REDACTED]");
            },
        );
    }

    #[test]
    fn test_execute_token_empty_becomes_none() {
        with_env(
            &[("BOT_TOKEN", "t"), ("WEBHOOK_URL", "https://a.com"), ("EXECUTE_TOKEN", "")],
            &[],
            || {
                let config = AgentConfig::from_env().unwrap();
                assert!(config.execute_token.is_none(), "空字符串应为 None");
            },
        );
    }
}
