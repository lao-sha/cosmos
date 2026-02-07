use std::collections::HashMap;
use std::sync::RwLock;
use tracing::info;

use crate::types::{BotInfoCache, NodeInfoCache};

/// 链上数据缓存（从链上读取，本地缓存）
///
/// TODO: Sprint 3 Week 3 — 使用 subxt 订阅链上事件自动刷新
/// 当前: 从环境变量加载静态数据（开发用）
pub struct ChainCache {
    /// Bot 注册信息: bot_id_hash → BotInfoCache
    bots: RwLock<HashMap<String, BotInfoCache>>,

    /// 活跃节点列表: node_id → NodeInfoCache
    nodes: RwLock<HashMap<String, NodeInfoCache>>,
}

impl ChainCache {
    pub fn new() -> Self {
        Self {
            bots: RwLock::new(HashMap::new()),
            nodes: RwLock::new(HashMap::new()),
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
}
