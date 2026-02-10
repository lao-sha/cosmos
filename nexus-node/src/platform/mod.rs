pub mod discord;
pub mod telegram;

use crate::types::{SignedMessage, ActionType};
use crate::chain_cache::ChainCache;
use crate::rule_engine::RuleContext;

/// Node 侧平台适配器 Trait
///
/// 每个平台实现此 trait，从 SignedMessage 中提取平台特定信息，
/// 构建 RuleContext（供规则引擎）和 determine_action（供 Leader）。
pub trait NodePlatformAdapter: Send + Sync {
    /// 平台名称
    fn platform_name(&self) -> &str;

    /// 从 SignedMessage 提取 RuleContext（供规则引擎使用）
    fn build_context(
        &self,
        message: &SignedMessage,
        chain_cache: Option<&ChainCache>,
    ) -> Option<RuleContext>;

    /// 从 SignedMessage 判断动作类型（供 Leader 使用）
    ///
    /// 返回: (动作类型, chat_id, 动作参数)
    fn determine_action(
        &self,
        message: &SignedMessage,
    ) -> (ActionType, i64, serde_json::Value);
}

/// 根据 platform 字段获取对应的适配器
pub fn get_adapter(platform: &str) -> Option<Box<dyn NodePlatformAdapter>> {
    match platform {
        "telegram" => Some(Box::new(telegram::TelegramNodeAdapter)),
        "discord" => Some(Box::new(discord::DiscordNodeAdapter)),
        _ => None,
    }
}
