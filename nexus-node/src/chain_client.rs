use subxt::{OnlineClient, SubstrateConfig, dynamic::Value};
use subxt_signer::sr25519::Keypair;
use tracing::{info, warn, debug};

use crate::types::{BotInfoCache, NodeInfoCache};

/// Substrate 链客户端
///
/// 使用 subxt dynamic API — 不需要编译时 metadata 生成
pub struct ChainClient {
    api: OnlineClient<SubstrateConfig>,
    signer: Keypair,
}

impl ChainClient {
    /// 连接到链
    pub async fn connect(rpc_url: &str, signer: Keypair) -> anyhow::Result<Self> {
        info!(url = rpc_url, "正在连接到 Substrate 链...");
        let api = OnlineClient::<SubstrateConfig>::from_url(rpc_url).await?;

        let chain = api.genesis_hash();
        let runtime_version = api.runtime_version();
        info!(
            genesis = ?chain,
            spec_version = runtime_version.spec_version,
            "链连接成功"
        );

        Ok(Self { api, signer })
    }

    // ═══════════════════════════════════════════════════════════════
    // Storage 读取
    // ═══════════════════════════════════════════════════════════════

    /// 从链上读取 Bot 注册信息
    pub async fn fetch_bot(&self, bot_id_hash: &[u8; 32]) -> anyhow::Result<Option<BotInfoCache>> {
        let query = subxt::dynamic::storage(
            "BotRegistry", "Bots",
            vec![Value::from_bytes(bot_id_hash)],
        );
        let result = self.api.storage().at_latest().await?.fetch(&query).await?;

        match result {
            Some(val) => {
                // 解码为 JSON 字符串，再提取字段
                let decoded = val.to_value()?;
                let json_str = format!("{:?}", decoded);
                debug!(raw = json_str, "Bot 原始数据");

                warn!("fetch_bot 返回占位值 — SCALE 解码未实现，公钥为全零");
                Ok(Some(BotInfoCache {
                    bot_id_hash: hex::encode(bot_id_hash),
                    owner_public_key: [0u8; 32], // TODO: 从 decoded 提取
                    platform: "telegram".into(),
                    is_active: true,
                }))
            }
            None => Ok(None),
        }
    }

    /// 从链上读取活跃节点列表
    pub async fn fetch_active_node_list(&self) -> anyhow::Result<Vec<[u8; 32]>> {
        let query = subxt::dynamic::storage(
            "BotConsensus", "ActiveNodeList", vec![],
        );
        let result = self.api.storage().at_latest().await?.fetch(&query).await?;

        match result {
            Some(val) => {
                // 原始 SCALE 编码的 bytes → 手动解码 Vec<[u8;32]>
                let raw = val.encoded();
                let node_ids = Vec::new();
                // SCALE: Vec prefix = compact length, then N * 32 bytes
                // 简化: 仅记录日志，实际解码需要 codec
                warn!(raw_len = raw.len(), "fetch_active_node_list 返回空列表 — SCALE 解码未实现");
                Ok(node_ids)
            }
            None => Ok(vec![]),
        }
    }

    /// 从链上读取单个节点信息
    pub async fn fetch_node(&self, node_id: &[u8; 32]) -> anyhow::Result<Option<NodeInfoCache>> {
        let query = subxt::dynamic::storage(
            "BotConsensus", "Nodes",
            vec![Value::from_bytes(node_id)],
        );
        let result = self.api.storage().at_latest().await?.fetch(&query).await?;

        match result {
            Some(val) => {
                let decoded = val.to_value()?;
                let json_str = format!("{:?}", decoded);
                debug!(raw = json_str, "Node 原始数据");

                warn!("fetch_node 返回占位值 — SCALE 解码未实现");
                Ok(Some(NodeInfoCache {
                    node_id: hex::encode(node_id),
                    endpoint: String::new(), // TODO: 从 decoded 提取
                    node_public_key: [0u8; 32],
                    status: "Active".into(),
                    reputation: 5000,
                }))
            }
            None => Ok(None),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // 交易提交
    // ═══════════════════════════════════════════════════════════════

    /// 提交消息确认到链上
    pub async fn submit_confirmations(
        &self,
        confirmations: Vec<ConfirmationPayload>,
    ) -> anyhow::Result<()> {
        let encoded: Vec<Value> = confirmations.iter().map(|c| {
            Value::unnamed_composite(vec![
                Value::from_bytes(&c.bot_id_hash),
                Value::from_bytes(&c.msg_hash),
                Value::u128(c.sequence as u128),
                Value::u128(c.timestamp as u128),
            ])
        }).collect();

        let tx = subxt::dynamic::tx(
            "BotConsensus", "submit_confirmations",
            vec![Value::unnamed_composite(encoded)],
        );

        let _events = self.api.tx()
            .sign_and_submit_then_watch_default(&tx, &self.signer)
            .await?
            .wait_for_finalized_success()
            .await?;

        info!(count = confirmations.len(), "确认已上链");
        Ok(())
    }

    /// 提交动作日志到链上
    pub async fn submit_action_log(
        &self,
        community_id_hash: [u8; 32],
        action_type_idx: u8,
        target_user_hash: [u8; 32],
        executor_node_hash: [u8; 32],
        consensus_count: u8,
        sequence: u64,
        msg_hash: [u8; 32],
    ) -> anyhow::Result<()> {
        let action_type_value = Value::unnamed_variant(
            action_type_name(action_type_idx), vec![],
        );

        let tx = subxt::dynamic::tx(
            "BotGroupMgmt", "log_action",
            vec![
                Value::from_bytes(&community_id_hash),
                action_type_value,
                Value::from_bytes(&target_user_hash),
                Value::from_bytes(&executor_node_hash),
                Value::u128(consensus_count as u128),
                Value::u128(sequence as u128),
                Value::from_bytes(&msg_hash),
            ],
        );

        let _events = self.api.tx()
            .sign_and_submit_then_watch_default(&tx, &self.signer)
            .await?
            .wait_for_finalized_success()
            .await?;

        info!(sequence, "动作日志已上链");
        Ok(())
    }

    /// 提交 Equivocation 举报到链上
    pub async fn report_equivocation(
        &self,
        node_id_hash: [u8; 32],
        msg_hash_a: [u8; 32],
        msg_hash_b: [u8; 32],
        sequence: u64,
    ) -> anyhow::Result<()> {
        let tx = subxt::dynamic::tx(
            "BotConsensus", "report_equivocation",
            vec![
                Value::from_bytes(&node_id_hash),
                Value::from_bytes(&msg_hash_a),
                Value::from_bytes(&msg_hash_b),
                Value::u128(sequence as u128),
            ],
        );

        let _events = self.api.tx()
            .sign_and_submit_then_watch_default(&tx, &self.signer)
            .await?
            .wait_for_finalized_success()
            .await?;

        warn!(sequence, "Equivocation 举报已上链");
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // 区块订阅
    // ═══════════════════════════════════════════════════════════════

    /// 订阅 finalized 区块
    pub async fn subscribe_finalized_blocks(&self) -> anyhow::Result<
        impl futures_util::Stream<Item = Result<subxt::blocks::Block<SubstrateConfig, OnlineClient<SubstrateConfig>>, subxt::Error>>
    > {
        let stream = self.api.blocks().subscribe_finalized().await?;
        Ok(stream)
    }
}

/// 确认载荷
pub struct ConfirmationPayload {
    pub bot_id_hash: [u8; 32],
    pub msg_hash: [u8; 32],
    pub sequence: u64,
    pub timestamp: u64,
}

fn action_type_name(idx: u8) -> &'static str {
    match idx {
        0 => "Ban", 1 => "Mute", 2 => "Unmute",
        3 => "DeleteMessage", 4 => "PinMessage",
        5 => "ApproveJoin", 6 => "DeclineJoin",
        7 => "SendMessage", _ => "Unknown",
    }
}
