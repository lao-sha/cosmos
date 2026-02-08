//! 云服务器管理工具 - 自动购买和管理云服务器

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::approval::{ApprovalManager, AuditLog, PendingOperation, RiskLevel};
use crate::cloud_provider::{BudgetManager, CloudClient, CreateServerRequest};
use crate::lightning::LightningPaymentManager;
use crate::tools::{Tool, ToolResult};

/// 云服务工具上下文
pub struct CloudContext {
    pub client: CloudClient,
    pub budget: BudgetManager,
    pub approval: ApprovalManager,
    pub lightning: LightningPaymentManager,
}

impl CloudContext {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: CloudClient::from_env()?,
            budget: BudgetManager::from_env(),
            approval: ApprovalManager::new(),
            lightning: LightningPaymentManager::new(),
        })
    }
}

// ============================================================================
// 云服务器工具
// ============================================================================

/// 列出可用区域
pub struct ListCloudRegions {
    pub context: Arc<Mutex<CloudContext>>,
}

#[async_trait]
impl Tool for ListCloudRegions {
    fn name(&self) -> &'static str { "list_cloud_regions" }

    fn description(&self) -> &'static str {
        "列出云服务商可用的区域（数据中心位置）"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        let ctx = self.context.lock().await;
        
        match ctx.client.list_regions().await {
            Ok(regions) => {
                let region_list: Vec<_> = regions.iter().map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "name": r.name,
                        "country": r.country
                    })
                }).collect();

                Ok(ToolResult::success_with_data(
                    format!("{} 共有 {} 个可用区域", ctx.client.provider_name(), region_list.len()),
                    serde_json::json!({ "regions": region_list })
                ))
            }
            Err(e) => Ok(ToolResult::error(format!("获取区域失败: {}", e)))
        }
    }
}

/// 列出可用套餐和价格
pub struct ListCloudPlans {
    pub context: Arc<Mutex<CloudContext>>,
}

#[async_trait]
impl Tool for ListCloudPlans {
    fn name(&self) -> &'static str { "list_cloud_plans" }

    fn description(&self) -> &'static str {
        "列出云服务器可用的配置套餐和价格"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "min_vcpu": {
                    "type": "integer",
                    "description": "最小 CPU 核心数"
                },
                "min_ram_gb": {
                    "type": "integer",
                    "description": "最小内存 GB"
                },
                "max_price": {
                    "type": "number",
                    "description": "最高月价格（美元）"
                }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let ctx = self.context.lock().await;
        
        let min_vcpu = args["min_vcpu"].as_u64().unwrap_or(0) as u32;
        let min_ram_gb = args["min_ram_gb"].as_u64().unwrap_or(0) as u32;
        let max_price = args["max_price"].as_f64().unwrap_or(f64::MAX) as f32;

        match ctx.client.list_plans().await {
            Ok(plans) => {
                let filtered: Vec<_> = plans.iter()
                    .filter(|p| p.vcpu >= min_vcpu)
                    .filter(|p| p.ram_mb >= min_ram_gb * 1024)
                    .filter(|p| p.price_monthly <= max_price)
                    .take(10) // 限制返回数量
                    .map(|p| serde_json::json!({
                        "id": p.id,
                        "name": p.name,
                        "vcpu": p.vcpu,
                        "ram_gb": p.ram_mb / 1024,
                        "disk_gb": p.disk_gb,
                        "price_monthly": format!("${:.2}", p.price_monthly),
                        "price_hourly": format!("${:.4}", p.price_hourly)
                    }))
                    .collect();

                Ok(ToolResult::success_with_data(
                    format!("找到 {} 个符合条件的套餐", filtered.len()),
                    serde_json::json!({ "plans": filtered })
                ))
            }
            Err(e) => Ok(ToolResult::error(format!("获取套餐失败: {}", e)))
        }
    }
}

/// 估算成本
pub struct EstimateCost {
    pub context: Arc<Mutex<CloudContext>>,
}

#[async_trait]
impl Tool for EstimateCost {
    fn name(&self) -> &'static str { "estimate_cloud_cost" }

    fn description(&self) -> &'static str {
        "估算创建服务器的成本，并检查预算是否充足"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "plan_id": {
                    "type": "string",
                    "description": "套餐 ID"
                },
                "count": {
                    "type": "integer",
                    "description": "服务器数量，默认 1"
                }
            },
            "required": ["plan_id"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let ctx = self.context.lock().await;
        
        let plan_id = args["plan_id"].as_str()
            .ok_or_else(|| anyhow!("Missing plan_id"))?;
        let count = args["count"].as_u64().unwrap_or(1) as u32;

        match ctx.client.list_plans().await {
            Ok(plans) => {
                if let Some(plan) = plans.iter().find(|p| p.id == plan_id) {
                    let monthly_cost = plan.price_monthly * count as f32;
                    let hourly_cost = plan.price_hourly * count as f32;
                    let can_afford = ctx.budget.can_afford(monthly_cost);
                    let remaining = ctx.budget.remaining();

                    Ok(ToolResult::success_with_data(
                        format!(
                            "预估成本: ${:.2}/月 (${:.4}/小时)\n预算剩余: ${:.2}/月\n{}",
                            monthly_cost,
                            hourly_cost,
                            remaining,
                            if can_afford { "✅ 预算充足" } else { "❌ 超出预算" }
                        ),
                        serde_json::json!({
                            "plan": plan.name,
                            "count": count,
                            "monthly_cost": monthly_cost,
                            "hourly_cost": hourly_cost,
                            "budget_remaining": remaining,
                            "can_afford": can_afford
                        })
                    ))
                } else {
                    Ok(ToolResult::error(format!("未找到套餐: {}", plan_id)))
                }
            }
            Err(e) => Ok(ToolResult::error(format!("获取套餐失败: {}", e)))
        }
    }
}

/// 创建云服务器
pub struct CreateCloudServer {
    pub context: Arc<Mutex<CloudContext>>,
}

#[async_trait]
impl Tool for CreateCloudServer {
    fn name(&self) -> &'static str { "create_cloud_server" }

    fn description(&self) -> &'static str {
        "购买并创建云服务器（需要审批，会产生费用）"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "服务器名称"
                },
                "region": {
                    "type": "string",
                    "description": "区域 ID（从 list_cloud_regions 获取）"
                },
                "plan": {
                    "type": "string",
                    "description": "套餐 ID（从 list_cloud_plans 获取）"
                },
                "os": {
                    "type": "string",
                    "description": "操作系统 ID，默认 Ubuntu 22.04"
                }
            },
            "required": ["name", "region", "plan"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let mut ctx = self.context.lock().await;
        
        let name = args["name"].as_str()
            .ok_or_else(|| anyhow!("Missing name"))?;
        let region = args["region"].as_str()
            .ok_or_else(|| anyhow!("Missing region"))?;
        let plan = args["plan"].as_str()
            .ok_or_else(|| anyhow!("Missing plan"))?;
        let os = args["os"].as_str().unwrap_or("ubuntu-22-04-x64");

        // 获取套餐价格
        let plans = ctx.client.list_plans().await?;
        let plan_info = plans.iter().find(|p| p.id == plan)
            .ok_or_else(|| anyhow!("Invalid plan: {}", plan))?;

        // 检查预算
        if !ctx.budget.can_afford(plan_info.price_monthly) {
            return Ok(ToolResult::error(format!(
                "预算不足！\n套餐价格: ${:.2}/月\n预算剩余: ${:.2}/月",
                plan_info.price_monthly,
                ctx.budget.remaining()
            )));
        }

        // 创建审批请求
        let command = format!(
            "创建服务器:\n  名称: {}\n  区域: {}\n  套餐: {} ({})\n  费用: ${:.2}/月",
            name, region, plan, plan_info.name, plan_info.price_monthly
        );

        let mut operation = PendingOperation::new(
            "create_cloud_server",
            &format!("在 {} 创建云服务器 {}", ctx.client.provider_name(), name),
            ctx.client.provider_name(),
            &command,
            RiskLevel::Critical, // 涉及资金，极高风险
        );

        // 请求审批
        let approved = ctx.approval.request_approval(&mut operation).await?;
        if !approved {
            return Ok(ToolResult::error("创建服务器操作已被拒绝"));
        }

        // 执行创建
        info!("Creating cloud server: {} on {}", name, ctx.client.provider_name());

        let req = CreateServerRequest {
            name: name.to_string(),
            region: region.to_string(),
            plan: plan.to_string(),
            os: os.to_string(),
            ssh_keys: vec![], // TODO: 支持 SSH 密钥
            label: Some("node-operator".to_string()),
        };

        match ctx.client.create_server(&req).await {
            Ok(server) => {
                // 更新预算
                ctx.budget.current_spend += plan_info.price_monthly;

                let audit = AuditLog::from_operation(&operation, Some("Server created"), Some(0));
                audit.log();

                Ok(ToolResult::success_with_data(
                    format!(
                        "✅ 服务器创建成功！\nID: {}\nIP: {}\n状态: {}\n\n⚠️ 服务器正在初始化，IP 地址可能需要几分钟才能分配。",
                        server.id, server.ip_address, server.status
                    ),
                    serde_json::json!({
                        "id": server.id,
                        "name": server.name,
                        "ip_address": server.ip_address,
                        "status": server.status,
                        "monthly_cost": plan_info.price_monthly
                    })
                ))
            }
            Err(e) => {
                let audit = AuditLog::from_operation(&operation, Some(&e.to_string()), Some(1));
                audit.log();
                Ok(ToolResult::error(format!("创建服务器失败: {}", e)))
            }
        }
    }
}

/// 列出已创建的服务器
pub struct ListCloudServers {
    pub context: Arc<Mutex<CloudContext>>,
}

#[async_trait]
impl Tool for ListCloudServers {
    fn name(&self) -> &'static str { "list_cloud_servers" }

    fn description(&self) -> &'static str {
        "列出所有已创建的云服务器"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        let ctx = self.context.lock().await;
        
        match ctx.client.list_servers().await {
            Ok(servers) => {
                if servers.is_empty() {
                    return Ok(ToolResult::success("没有找到服务器"));
                }

                let server_list: Vec<_> = servers.iter().map(|s| {
                    serde_json::json!({
                        "id": s.id,
                        "name": s.name,
                        "ip": s.ip_address,
                        "region": s.region,
                        "status": s.status,
                        "plan": s.plan
                    })
                }).collect();

                Ok(ToolResult::success_with_data(
                    format!("共有 {} 个服务器", server_list.len()),
                    serde_json::json!({ "servers": server_list })
                ))
            }
            Err(e) => Ok(ToolResult::error(format!("获取服务器列表失败: {}", e)))
        }
    }
}

/// 销毁服务器
pub struct DestroyCloudServer {
    pub context: Arc<Mutex<CloudContext>>,
}

#[async_trait]
impl Tool for DestroyCloudServer {
    fn name(&self) -> &'static str { "destroy_cloud_server" }

    fn description(&self) -> &'static str {
        "销毁云服务器（需要审批，数据将永久丢失）"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "server_id": {
                    "type": "string",
                    "description": "服务器 ID"
                }
            },
            "required": ["server_id"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let ctx = self.context.lock().await;
        
        let server_id = args["server_id"].as_str()
            .ok_or_else(|| anyhow!("Missing server_id"))?;

        // 创建审批请求
        let mut operation = PendingOperation::new(
            "destroy_cloud_server",
            &format!("销毁服务器 {}", server_id),
            ctx.client.provider_name(),
            &format!("DELETE /instances/{}", server_id),
            RiskLevel::Critical,
        );

        let approved = ctx.approval.request_approval(&mut operation).await?;
        if !approved {
            return Ok(ToolResult::error("销毁操作已被拒绝"));
        }

        match ctx.client.destroy_server(server_id).await {
            Ok(()) => {
                let audit = AuditLog::from_operation(&operation, Some("Server destroyed"), Some(0));
                audit.log();
                Ok(ToolResult::success(format!("✅ 服务器 {} 已销毁", server_id)))
            }
            Err(e) => Ok(ToolResult::error(format!("销毁失败: {}", e)))
        }
    }
}

/// 自动部署完整流程
pub struct AutoDeployNode {
    pub context: Arc<Mutex<CloudContext>>,
}

#[async_trait]
impl Tool for AutoDeployNode {
    fn name(&self) -> &'static str { "auto_deploy_node" }

    fn description(&self) -> &'static str {
        "自动购买服务器并部署节点的完整流程（需要审批）。会自动选择合适的配置。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "node_name": {
                    "type": "string",
                    "description": "节点名称"
                },
                "node_type": {
                    "type": "string",
                    "enum": ["validator", "full", "archive"],
                    "description": "节点类型"
                },
                "region": {
                    "type": "string",
                    "description": "区域 ID"
                },
                "chain": {
                    "type": "string",
                    "description": "链名称，默认 nexus"
                }
            },
            "required": ["node_name", "node_type", "region"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let node_name = args["node_name"].as_str()
            .ok_or_else(|| anyhow!("Missing node_name"))?;
        let node_type = args["node_type"].as_str()
            .ok_or_else(|| anyhow!("Missing node_type"))?;
        let region = args["region"].as_str()
            .ok_or_else(|| anyhow!("Missing region"))?;
        let chain = args["chain"].as_str().unwrap_or("nexus");

        // 根据节点类型推荐配置
        let (desc, min_vcpu, min_ram) = crate::cloud_provider::recommend_plan_for_node(node_type);

        let ctx = self.context.lock().await;

        // 查找合适的套餐
        let plans = ctx.client.list_plans().await?;
        let suitable_plan = plans.iter()
            .filter(|p| p.vcpu >= min_vcpu && p.ram_mb >= min_ram)
            .min_by(|a, b| a.price_monthly.partial_cmp(&b.price_monthly).unwrap());

        let plan = match suitable_plan {
            Some(p) => p,
            None => return Ok(ToolResult::error(format!(
                "没有找到满足 {} 要求的套餐（需要 {}+ vCPU, {}+ MB RAM）",
                desc, min_vcpu, min_ram
            ))),
        };

        // 返回部署计划，让用户确认
        Ok(ToolResult::success_with_data(
            format!(
                "📋 自动部署计划\n\n\
                节点类型: {} ({})\n\
                链: {}\n\
                区域: {}\n\
                推荐套餐: {} ({})\n\
                费用: ${:.2}/月\n\n\
                请使用 create_cloud_server 工具执行创建，或调整参数后重试。",
                node_type, desc, chain, region, plan.id, plan.name, plan.price_monthly
            ),
            serde_json::json!({
                "node_name": node_name,
                "node_type": node_type,
                "chain": chain,
                "region": region,
                "recommended_plan": {
                    "id": plan.id,
                    "name": plan.name,
                    "vcpu": plan.vcpu,
                    "ram_mb": plan.ram_mb,
                    "price_monthly": plan.price_monthly
                },
                "next_step": "create_cloud_server"
            })
        ))
    }
}

// ============================================================================
// Lightning 支付工具
// ============================================================================

/// 查询 Lightning 钱包余额
pub struct GetLightningBalance {
    pub context: Arc<Mutex<CloudContext>>,
}

#[async_trait]
impl Tool for GetLightningBalance {
    fn name(&self) -> &'static str { "get_lightning_balance" }

    fn description(&self) -> &'static str {
        "查询 LNbits 钱包余额（用于支付 LNVPS 等 Lightning 服务）"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        let ctx = self.context.lock().await;
        
        if !ctx.lightning.is_configured() {
            return Ok(ToolResult::error(
                "LNbits 未配置。请设置 LNBITS_URL 和 LNBITS_ADMIN_KEY 环境变量。"
            ));
        }

        match ctx.lightning.get_balance().await {
            Ok(balance) => {
                Ok(ToolResult::success_with_data(
                    format!(
                        "⚡ Lightning 钱包余额: {} sats (~${:.2} USD)",
                        balance.balance_sats,
                        balance.balance_sats as f64 * 0.0003 // 约 $30k/BTC
                    ),
                    serde_json::json!({
                        "balance_sats": balance.balance_sats,
                        "balance_msat": balance.balance_msat,
                        "auto_pay_enabled": ctx.lightning.is_auto_pay_enabled()
                    })
                ))
            }
            Err(e) => Ok(ToolResult::error(format!("获取余额失败: {}", e)))
        }
    }
}

/// 支付 Lightning Invoice
pub struct PayLightningInvoice {
    pub context: Arc<Mutex<CloudContext>>,
}

#[async_trait]
impl Tool for PayLightningInvoice {
    fn name(&self) -> &'static str { "pay_lightning_invoice" }

    fn description(&self) -> &'static str {
        "支付 Lightning Network Invoice（用于支付 LNVPS 等服务，需要审批）"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "invoice": {
                    "type": "string",
                    "description": "Lightning Invoice (bolt11 格式，以 lnbc 开头)"
                },
                "description": {
                    "type": "string",
                    "description": "支付说明（用于审计）"
                }
            },
            "required": ["invoice"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let ctx = self.context.lock().await;
        
        if !ctx.lightning.is_configured() {
            return Ok(ToolResult::error(
                "LNbits 未配置。请设置 LNBITS_URL 和 LNBITS_ADMIN_KEY 环境变量。"
            ));
        }

        let invoice = args["invoice"].as_str()
            .ok_or_else(|| anyhow!("Missing invoice"))?;
        let description = args["description"].as_str()
            .unwrap_or("Lightning payment");

        // 创建审批请求
        let mut operation = PendingOperation::new(
            "pay_lightning_invoice",
            &format!("支付 Lightning Invoice: {}", description),
            "LNbits",
            &format!("Invoice: {}...", &invoice[..std::cmp::min(50, invoice.len())]),
            RiskLevel::High, // 涉及资金
        );

        let approved = ctx.approval.request_approval(&mut operation).await?;
        if !approved {
            return Ok(ToolResult::error("支付操作已被拒绝"));
        }

        match ctx.lightning.pay(invoice).await {
            Ok(result) => {
                let audit = AuditLog::from_operation(&operation, Some("Payment successful"), Some(0));
                audit.log();

                Ok(ToolResult::success_with_data(
                    format!("✅ 支付成功！\nPayment Hash: {}", result.payment_hash),
                    serde_json::json!({
                        "payment_hash": result.payment_hash,
                        "checking_id": result.checking_id,
                        "preimage": result.preimage
                    })
                ))
            }
            Err(e) => {
                let audit = AuditLog::from_operation(&operation, Some(&e.to_string()), Some(1));
                audit.log();
                Ok(ToolResult::error(format!("支付失败: {}", e)))
            }
        }
    }
}
