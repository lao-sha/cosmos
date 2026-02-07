/// 节点全局配置
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// 节点 ID (hex)
    pub node_id: String,

    /// API 监听端口
    pub listen_port: u16,

    /// Gossip 监听端口 (WebSocket)
    pub gossip_port: u16,

    /// 链上 RPC 端点
    pub chain_rpc: String,

    /// 节点 Ed25519 私钥路径
    pub signing_key_path: String,

    /// 数据目录
    pub data_dir: String,
}

impl NodeConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let node_id = std::env::var("NODE_ID")
            .map_err(|_| anyhow::anyhow!("NODE_ID 环境变量未设置"))?;

        let listen_port = std::env::var("LISTEN_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);

        let gossip_port = std::env::var("GOSSIP_PORT")
            .unwrap_or_else(|_| "9090".to_string())
            .parse::<u16>()
            .unwrap_or(9090);

        let chain_rpc = std::env::var("CHAIN_RPC")
            .unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string());

        let signing_key_path = std::env::var("SIGNING_KEY_PATH")
            .unwrap_or_else(|_| "/data/node.key".to_string());

        let data_dir = std::env::var("DATA_DIR")
            .unwrap_or_else(|_| "/data".to_string());

        Ok(Self {
            node_id,
            listen_port,
            gossip_port,
            chain_rpc,
            signing_key_path,
            data_dir,
        })
    }
}
