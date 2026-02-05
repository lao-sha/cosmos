//! 安全审批机制 - 危险操作需人工确认
//!
//! 所有远程执行操作都需要通过审批流程

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use uuid::Uuid;

/// 操作风险级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 只读操作，无风险
    Safe,
    /// 低风险，可能影响性能
    Low,
    /// 中风险，可能导致服务中断
    Medium,
    /// 高风险，可能导致数据丢失
    High,
    /// 极高风险，需要多重确认
    Critical,
}

impl RiskLevel {
    pub fn requires_approval(&self) -> bool {
        matches!(self, RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical)
    }

    pub fn description(&self) -> &'static str {
        match self {
            RiskLevel::Safe => "安全操作",
            RiskLevel::Low => "低风险",
            RiskLevel::Medium => "中风险 - 可能导致服务中断",
            RiskLevel::High => "高风险 - 可能导致数据丢失",
            RiskLevel::Critical => "极高风险 - 需要多重确认",
        }
    }
}

/// 待审批的操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOperation {
    pub id: String,
    pub operation_type: String,
    pub description: String,
    pub target_server: String,
    pub command: String,
    pub risk_level: RiskLevel,
    pub created_at: DateTime<Utc>,
    pub approved: Option<bool>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<String>,
}

impl PendingOperation {
    pub fn new(
        operation_type: &str,
        description: &str,
        target_server: &str,
        command: &str,
        risk_level: RiskLevel,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string()[..8].to_string(),
            operation_type: operation_type.to_string(),
            description: description.to_string(),
            target_server: target_server.to_string(),
            command: command.to_string(),
            risk_level,
            created_at: Utc::now(),
            approved: None,
            approved_at: None,
            approved_by: None,
        }
    }
}

/// 审批管理器
pub struct ApprovalManager {
    auto_approve_safe: bool,
}

impl ApprovalManager {
    pub fn new() -> Self {
        Self {
            auto_approve_safe: true,
        }
    }

    /// 请求审批
    pub async fn request_approval(&self, operation: &mut PendingOperation) -> Result<bool> {
        // 安全操作自动通过
        if !operation.risk_level.requires_approval() && self.auto_approve_safe {
            operation.approved = Some(true);
            operation.approved_at = Some(Utc::now());
            operation.approved_by = Some("auto".to_string());
            return Ok(true);
        }

        // 需要人工审批
        self.prompt_user_approval(operation)
    }

    /// 命令行交互式审批
    fn prompt_user_approval(&self, operation: &mut PendingOperation) -> Result<bool> {
        println!();
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                    ⚠️  操作审批请求                            ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║ 操作ID:   {:<50} ║", operation.id);
        println!("║ 类型:     {:<50} ║", operation.operation_type);
        println!("║ 目标:     {:<50} ║", operation.target_server);
        println!("║ 风险级别: {:<50} ║", operation.risk_level.description());
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║ 描述: {}", operation.description);
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║ 将执行的命令:");
        
        // 显示命令（截断过长的命令）
        for line in operation.command.lines().take(10) {
            let display_line = if line.len() > 60 {
                format!("{}...", &line[..57])
            } else {
                line.to_string()
            };
            println!("║   {}", display_line);
        }
        
        if operation.command.lines().count() > 10 {
            println!("║   ... (更多内容已省略)");
        }
        
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        // 高风险操作需要输入确认码
        if operation.risk_level == RiskLevel::Critical {
            let confirm_code = &operation.id[..4];
            print!("⚠️  极高风险操作！请输入确认码 [{}] 以继续，或输入 'n' 取消: ", confirm_code);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input == confirm_code {
                operation.approved = Some(true);
                operation.approved_at = Some(Utc::now());
                operation.approved_by = Some("user".to_string());
                println!("✅ 操作已批准");
                return Ok(true);
            } else {
                operation.approved = Some(false);
                println!("❌ 操作已拒绝");
                return Ok(false);
            }
        }

        // 普通风险操作只需 y/n
        print!("是否批准此操作? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input == "y" || input == "yes" {
            operation.approved = Some(true);
            operation.approved_at = Some(Utc::now());
            operation.approved_by = Some("user".to_string());
            println!("✅ 操作已批准");
            Ok(true)
        } else {
            operation.approved = Some(false);
            println!("❌ 操作已拒绝");
            Ok(false)
        }
    }

    /// 评估命令的风险级别
    pub fn assess_risk(&self, command: &str) -> RiskLevel {
        let command_lower = command.to_lowercase();

        // 极高风险操作
        if command_lower.contains("rm -rf")
            || command_lower.contains("mkfs")
            || command_lower.contains("dd if=")
            || command_lower.contains(":(){ :|:& };:")
        {
            return RiskLevel::Critical;
        }

        // 高风险操作
        if command_lower.contains("rm ")
            || command_lower.contains("reboot")
            || command_lower.contains("shutdown")
            || command_lower.contains("systemctl stop")
            || command_lower.contains("kill -9")
        {
            return RiskLevel::High;
        }

        // 中风险操作
        if command_lower.contains("systemctl restart")
            || command_lower.contains("apt-get install")
            || command_lower.contains("yum install")
            || command_lower.contains("wget")
            || command_lower.contains("curl") && command_lower.contains("|")
        {
            return RiskLevel::Medium;
        }

        // 低风险操作
        if command_lower.contains("systemctl status")
            || command_lower.contains("journalctl")
            || command_lower.contains("cat ")
            || command_lower.contains("ls ")
            || command_lower.contains("df ")
            || command_lower.contains("free ")
            || command_lower.contains("uptime")
            || command_lower.contains("ps ")
        {
            return RiskLevel::Safe;
        }

        // 默认中风险
        RiskLevel::Low
    }
}

/// 审计日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub timestamp: DateTime<Utc>,
    pub operation_id: String,
    pub operation_type: String,
    pub target_server: String,
    pub command: String,
    pub approved: bool,
    pub approved_by: String,
    pub result: Option<String>,
    pub exit_code: Option<u32>,
}

impl AuditLog {
    pub fn from_operation(operation: &PendingOperation, result: Option<&str>, exit_code: Option<u32>) -> Self {
        Self {
            timestamp: Utc::now(),
            operation_id: operation.id.clone(),
            operation_type: operation.operation_type.clone(),
            target_server: operation.target_server.clone(),
            command: operation.command.clone(),
            approved: operation.approved.unwrap_or(false),
            approved_by: operation.approved_by.clone().unwrap_or_default(),
            result: result.map(|s| s.to_string()),
            exit_code,
        }
    }

    /// 输出日志
    pub fn log(&self) {
        println!(
            "[AUDIT] {} | {} | {} | {} | approved={} by={} | exit={}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.operation_id,
            self.operation_type,
            self.target_server,
            self.approved,
            self.approved_by,
            self.exit_code.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string())
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_assessment() {
        let manager = ApprovalManager::new();

        assert_eq!(manager.assess_risk("ls -la"), RiskLevel::Safe);
        assert_eq!(manager.assess_risk("systemctl status node"), RiskLevel::Safe);
        assert_eq!(manager.assess_risk("systemctl restart node"), RiskLevel::Medium);
        assert_eq!(manager.assess_risk("rm /tmp/file"), RiskLevel::High);
        assert_eq!(manager.assess_risk("rm -rf /"), RiskLevel::Critical);
    }

    #[test]
    fn test_pending_operation() {
        let op = PendingOperation::new(
            "deploy",
            "Deploy node to server",
            "server1",
            "echo hello",
            RiskLevel::Low,
        );

        assert_eq!(op.id.len(), 8);
        assert!(!op.risk_level.requires_approval());
    }
}
