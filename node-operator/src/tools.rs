//! 工具定义 - 节点运维可用的操作
//!
//! 每个工具都是预定义的安全操作，LLM 只能调用这些工具

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use tracing::{info, warn};

use crate::llm_client::ToolDef;

/// 工具执行 trait
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult>;
    
    fn to_def(&self) -> ToolDef {
        ToolDef {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: self.parameters_schema(),
        }
    }
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub data: Option<serde_json::Value>,
}

impl ToolResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self { success: true, output: output.into(), data: None }
    }
    
    pub fn success_with_data(output: impl Into<String>, data: serde_json::Value) -> Self {
        Self { success: true, output: output.into(), data: Some(data) }
    }
    
    pub fn error(output: impl Into<String>) -> Self {
        Self { success: false, output: output.into(), data: None }
    }
}

/// 工具注册表
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// 创建仅包含本地工具的注册表
    pub fn new() -> Self {
        let mut registry = Self { tools: HashMap::new() };
        
        // 注册本地工具
        registry.register(Box::new(GetNodeStatus));
        registry.register(Box::new(DiagnoseNode));
        registry.register(Box::new(GetNodeLogs));
        registry.register(Box::new(GenerateChainSpec));
        registry.register(Box::new(CheckSystemResources));
        registry.register(Box::new(ListNodes));
        
        registry
    }

    /// 创建包含远程工具的注册表
    pub fn with_remote_tools() -> Self {
        use crate::remote_tools::*;
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let mut registry = Self::new();

        // 创建远程上下文
        if let Ok(ctx) = RemoteContext::new() {
            let ctx = Arc::new(Mutex::new(ctx));

            // 注册远程工具
            registry.register(Box::new(ListServers { context: Arc::clone(&ctx) }));
            registry.register(Box::new(SshExecute { context: Arc::clone(&ctx) }));
            registry.register(Box::new(DeployNode { context: Arc::clone(&ctx) }));
            registry.register(Box::new(RemoteStatus { context: Arc::clone(&ctx) }));
            registry.register(Box::new(StartNode { context: Arc::clone(&ctx) }));
            registry.register(Box::new(StopNode { context: Arc::clone(&ctx) }));
            registry.register(Box::new(GenerateAnsiblePlaybook));

            info!("Remote tools registered successfully");
        } else {
            warn!("Failed to initialize remote context, remote tools disabled");
        }

        // 注册云服务工具
        registry.register_cloud_tools();

        registry
    }

    /// 注册云服务器管理工具
    fn register_cloud_tools(&mut self) {
        use crate::cloud_tools::*;
        use std::sync::Arc;
        use tokio::sync::Mutex;

        if let Ok(ctx) = CloudContext::new() {
            let ctx = Arc::new(Mutex::new(ctx));

            self.register(Box::new(ListCloudRegions { context: Arc::clone(&ctx) }));
            self.register(Box::new(ListCloudPlans { context: Arc::clone(&ctx) }));
            self.register(Box::new(EstimateCost { context: Arc::clone(&ctx) }));
            self.register(Box::new(CreateCloudServer { context: Arc::clone(&ctx) }));
            self.register(Box::new(ListCloudServers { context: Arc::clone(&ctx) }));
            self.register(Box::new(DestroyCloudServer { context: Arc::clone(&ctx) }));
            self.register(Box::new(AutoDeployNode { context: Arc::clone(&ctx) }));
            
            // Lightning 支付工具
            self.register(Box::new(GetLightningBalance { context: Arc::clone(&ctx) }));
            self.register(Box::new(PayLightningInvoice { context: Arc::clone(&ctx) }));

            info!("Cloud tools registered successfully");
        } else {
            warn!("Cloud provider not configured, cloud tools disabled");
        }
    }
    
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }
    
    pub fn get_all_defs(&self) -> Vec<ToolDef> {
        self.tools.values().map(|t| t.to_def()).collect()
    }
    
    pub async fn execute(&self, name: &str, args: serde_json::Value) -> Result<ToolResult> {
        let tool = self.tools.get(name)
            .ok_or_else(|| anyhow!("Unknown tool: {}", name))?;
        
        info!("Executing tool: {} with args: {}", name, args);
        tool.execute(args).await
    }
}

// ============================================================================
// 工具实现
// ============================================================================

/// 获取节点状态
pub struct GetNodeStatus;

#[async_trait]
impl Tool for GetNodeStatus {
    fn name(&self) -> &'static str { "get_node_status" }
    
    fn description(&self) -> &'static str {
        "获取 Substrate 节点的运行状态，包括同步状态、对等节点数量、最新区块等"
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "rpc_endpoint": {
                    "type": "string",
                    "description": "节点 RPC 端点，默认 http://127.0.0.1:9944"
                }
            }
        })
    }
    
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let endpoint = args["rpc_endpoint"]
            .as_str()
            .unwrap_or("http://127.0.0.1:9944");
        
        // 调用 system_health RPC
        let client = reqwest::Client::new();
        let health_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "system_health",
            "params": []
        });
        
        match client.post(endpoint).json(&health_req).send().await {
            Ok(resp) => {
                let body: serde_json::Value = resp.json().await?;
                let health = &body["result"];
                
                // 获取同步状态
                let sync_req = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "system_syncState",
                    "params": []
                });
                
                let sync_resp = client.post(endpoint).json(&sync_req).send().await?;
                let sync_body: serde_json::Value = sync_resp.json().await?;
                let sync_state = &sync_body["result"];
                
                let status = serde_json::json!({
                    "peers": health["peers"],
                    "is_syncing": health["isSyncing"],
                    "should_have_peers": health["shouldHavePeers"],
                    "current_block": sync_state["currentBlock"],
                    "highest_block": sync_state["highestBlock"],
                });
                
                Ok(ToolResult::success_with_data(
                    format!("节点状态: {} 个对等节点, 当前区块 {}, 最高区块 {}, 同步中: {}",
                        status["peers"], status["current_block"], 
                        status["highest_block"], status["is_syncing"]),
                    status
                ))
            }
            Err(e) => {
                Ok(ToolResult::error(format!("无法连接到节点 {}: {}", endpoint, e)))
            }
        }
    }
}

/// 诊断节点问题
pub struct DiagnoseNode;

#[async_trait]
impl Tool for DiagnoseNode {
    fn name(&self) -> &'static str { "diagnose_node" }
    
    fn description(&self) -> &'static str {
        "诊断节点可能存在的问题，检查常见故障原因"
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "rpc_endpoint": {
                    "type": "string",
                    "description": "节点 RPC 端点"
                },
                "check_items": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "要检查的项目: connectivity, sync, peers, resources"
                }
            }
        })
    }
    
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let endpoint = args["rpc_endpoint"]
            .as_str()
            .unwrap_or("http://127.0.0.1:9944");
        
        let mut issues = Vec::new();
        let mut checks = Vec::new();
        
        // 检查连接性
        let client = reqwest::Client::new();
        let health_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "system_health",
            "params": []
        });
        
        match client.post(endpoint).json(&health_req).send().await {
            Ok(resp) => {
                checks.push("✓ RPC 端口可访问");
                
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    let health = &body["result"];
                    
                    // 检查对等节点
                    let peers = health["peers"].as_u64().unwrap_or(0);
                    if peers == 0 {
                        issues.push("⚠ 没有对等节点连接，可能是网络问题或 bootnodes 配置错误");
                    } else if peers < 3 {
                        issues.push("⚠ 对等节点数量较少，建议检查网络配置");
                    } else {
                        checks.push(format!("✓ 有 {} 个对等节点", peers).leak());
                    }
                    
                    // 检查同步状态
                    if health["isSyncing"].as_bool().unwrap_or(false) {
                        checks.push("✓ 节点正在同步中");
                    }
                }
            }
            Err(e) => {
                issues.push("✗ 无法连接到 RPC 端口");
                issues.push(Box::leak(format!("  错误: {}", e).into_boxed_str()));
            }
        }
        
        // 检查系统资源
        if let Ok(output) = Command::new("df").args(["-h", "/"]).output() {
            let df_output = String::from_utf8_lossy(&output.stdout);
            if df_output.contains("100%") || df_output.contains("99%") || df_output.contains("98%") {
                issues.push("⚠ 磁盘空间不足，可能影响节点运行");
            } else {
                checks.push("✓ 磁盘空间充足");
            }
        }
        
        let result = serde_json::json!({
            "checks_passed": checks,
            "issues_found": issues,
            "issue_count": issues.len()
        });
        
        let summary = if issues.is_empty() {
            "诊断完成，未发现问题".to_string()
        } else {
            format!("诊断完成，发现 {} 个问题:\n{}", issues.len(), issues.join("\n"))
        };
        
        Ok(ToolResult::success_with_data(summary, result))
    }
}

/// 获取节点日志
pub struct GetNodeLogs;

#[async_trait]
impl Tool for GetNodeLogs {
    fn name(&self) -> &'static str { "get_node_logs" }
    
    fn description(&self) -> &'static str {
        "获取节点最近的日志，用于分析问题"
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "service_name": {
                    "type": "string",
                    "description": "systemd 服务名称，默认 substrate-node"
                },
                "lines": {
                    "type": "integer",
                    "description": "获取最近多少行日志，默认 50"
                },
                "filter": {
                    "type": "string",
                    "description": "日志过滤关键词，如 error, warn"
                }
            }
        })
    }
    
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let service = args["service_name"].as_str().unwrap_or("substrate-node");
        let lines = args["lines"].as_u64().unwrap_or(50);
        let filter = args["filter"].as_str();
        
        // 尝试使用 journalctl
        let mut cmd = Command::new("journalctl");
        cmd.args(["-u", service, "-n", &lines.to_string(), "--no-pager"]);
        
        match cmd.output() {
            Ok(output) => {
                let logs = String::from_utf8_lossy(&output.stdout);
                
                let filtered_logs = if let Some(keyword) = filter {
                    logs.lines()
                        .filter(|line| line.to_lowercase().contains(&keyword.to_lowercase()))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    logs.to_string()
                };
                
                if filtered_logs.is_empty() {
                    Ok(ToolResult::success("未找到匹配的日志记录"))
                } else {
                    Ok(ToolResult::success_with_data(
                        format!("获取到 {} 服务的日志", service),
                        serde_json::json!({ "logs": filtered_logs })
                    ))
                }
            }
            Err(e) => {
                warn!("Failed to get logs via journalctl: {}", e);
                Ok(ToolResult::error(format!("无法获取日志: {}。请确保服务名称正确且有访问权限", e)))
            }
        }
    }
}

/// 生成 Chain Spec
pub struct GenerateChainSpec;

#[async_trait]
impl Tool for GenerateChainSpec {
    fn name(&self) -> &'static str { "generate_chain_spec" }
    
    fn description(&self) -> &'static str {
        "生成节点配置建议或 chain spec 模板"
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "node_type": {
                    "type": "string",
                    "enum": ["validator", "full", "archive", "light"],
                    "description": "节点类型"
                },
                "network": {
                    "type": "string",
                    "description": "网络名称，如 mainnet, testnet"
                }
            },
            "required": ["node_type"]
        })
    }
    
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let node_type = args["node_type"].as_str().unwrap_or("full");
        let network = args["network"].as_str().unwrap_or("testnet");
        
        let config = match node_type {
            "validator" => serde_json::json!({
                "node_type": "validator",
                "recommended_config": {
                    "pruning": "archive",
                    "rpc_external": false,
                    "ws_external": false,
                    "validator": true,
                    "telemetry_url": format!("wss://telemetry.{}.io/submit/ 0", network),
                    "min_peers": 25,
                    "max_peers": 100
                },
                "hardware_requirements": {
                    "cpu": "8+ cores",
                    "ram": "32GB+",
                    "storage": "1TB+ NVMe SSD",
                    "network": "1Gbps"
                },
                "security_notes": [
                    "禁用外部 RPC/WS 访问",
                    "使用防火墙限制入站连接",
                    "定期备份 keystore",
                    "使用 session keys 而非直接使用 stash key"
                ]
            }),
            "archive" => serde_json::json!({
                "node_type": "archive",
                "recommended_config": {
                    "pruning": "archive",
                    "rpc_external": true,
                    "ws_external": true,
                    "rpc_cors": "all",
                    "rpc_methods": "safe"
                },
                "hardware_requirements": {
                    "cpu": "4+ cores",
                    "ram": "16GB+",
                    "storage": "2TB+ SSD (grows over time)",
                    "network": "500Mbps"
                }
            }),
            _ => serde_json::json!({
                "node_type": node_type,
                "recommended_config": {
                    "pruning": 256,
                    "rpc_external": false,
                    "ws_external": false,
                    "state_cache_size": 67108864
                },
                "hardware_requirements": {
                    "cpu": "2+ cores",
                    "ram": "8GB+",
                    "storage": "100GB+ SSD",
                    "network": "100Mbps"
                }
            })
        };
        
        Ok(ToolResult::success_with_data(
            format!("已生成 {} 类型节点的配置建议", node_type),
            config
        ))
    }
}

/// 检查系统资源
pub struct CheckSystemResources;

#[async_trait]
impl Tool for CheckSystemResources {
    fn name(&self) -> &'static str { "check_system_resources" }
    
    fn description(&self) -> &'static str {
        "检查系统 CPU、内存、磁盘使用情况"
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }
    
    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        let mut resources = serde_json::Map::new();
        
        // CPU 信息
        if let Ok(output) = Command::new("nproc").output() {
            let cores = String::from_utf8_lossy(&output.stdout).trim().to_string();
            resources.insert("cpu_cores".to_string(), serde_json::json!(cores));
        }
        
        // 内存信息
        if let Ok(output) = Command::new("free").args(["-h"]).output() {
            let mem_info = String::from_utf8_lossy(&output.stdout);
            resources.insert("memory_info".to_string(), serde_json::json!(mem_info.to_string()));
        }
        
        // 磁盘信息
        if let Ok(output) = Command::new("df").args(["-h"]).output() {
            let disk_info = String::from_utf8_lossy(&output.stdout);
            resources.insert("disk_info".to_string(), serde_json::json!(disk_info.to_string()));
        }
        
        // 负载信息
        if let Ok(output) = Command::new("uptime").output() {
            let uptime = String::from_utf8_lossy(&output.stdout).trim().to_string();
            resources.insert("uptime".to_string(), serde_json::json!(uptime));
        }
        
        Ok(ToolResult::success_with_data(
            "系统资源检查完成",
            serde_json::Value::Object(resources)
        ))
    }
}

/// 列出管理的节点
pub struct ListNodes;

#[async_trait]
impl Tool for ListNodes {
    fn name(&self) -> &'static str { "list_nodes" }
    
    fn description(&self) -> &'static str {
        "列出当前配置管理的所有节点"
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }
    
    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        // MVP: 从环境变量或配置文件读取节点列表
        let nodes_env = std::env::var("MANAGED_NODES")
            .unwrap_or_else(|_| "http://127.0.0.1:9944".to_string());
        
        let nodes: Vec<&str> = nodes_env.split(',').collect();
        
        let node_list: Vec<_> = nodes.iter().enumerate().map(|(i, endpoint)| {
            serde_json::json!({
                "id": i + 1,
                "endpoint": endpoint.trim(),
                "status": "unknown"
            })
        }).collect();
        
        Ok(ToolResult::success_with_data(
            format!("共有 {} 个节点", node_list.len()),
            serde_json::json!({ "nodes": node_list })
        ))
    }
}
