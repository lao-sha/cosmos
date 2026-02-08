use serde::Deserialize;

/// Agent 全局配置（从环境变量加载）
#[derive(Debug, Clone, Deserialize)]
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
            .unwrap_or_else(|_| format!("https://localhost:{}", webhook_port));

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
            bot_id_hash,
        })
    }

    /// 获取 bot_id_hash 的 hex 字符串
    pub fn bot_id_hash_hex(&self) -> String {
        hex::encode(self.bot_id_hash)
    }
}
