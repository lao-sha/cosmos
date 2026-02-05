//! Node Operator Agent - LLM 驱动的 Substrate 节点运维助手
//!
//! # 概述
//!
//! 这是一个使用大语言模型（LLM）自动化管理 Substrate 节点的工具。
//! 它可以帮助运维人员：
//!
//! - 诊断节点问题
//! - 获取节点状态
//! - 生成配置建议
//! - 分析日志
//!
//! # 安全设计
//!
//! - LLM 只能调用预定义的工具，无法执行任意命令
//! - 危险操作（重启、升级等）只给出建议，不自动执行
//! - API KEY 应通过环境变量配置，生产环境建议使用 TEE 保护
//!
//! # 使用方法
//!
//! ```bash
//! # 设置环境变量
//! export ANTHROPIC_API_KEY="your-api-key"
//! export LLM_PROVIDER="claude"  # 或 "openai"
//!
//! # 运行 Agent
//! node-operator chat "检查节点状态"
//! ```

pub mod agent;
pub mod approval;
pub mod cloud_provider;
pub mod cloud_tools;
pub mod lightning;
pub mod llm_client;
pub mod remote_tools;
pub mod ssh;
pub mod tools;

pub use agent::NodeOperatorAgent;
pub use approval::{ApprovalManager, PendingOperation, RiskLevel};
pub use cloud_provider::{CloudClient, CloudProvider};
pub use lightning::{LightningPaymentManager, LnbitsClient};
pub use llm_client::{LlmClient, LlmProvider, Message};
pub use ssh::SshManager;
pub use tools::{Tool, ToolRegistry, ToolResult};
