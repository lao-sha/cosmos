//! 远程部署工具 - 通过 SSH 执行节点部署操作

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::approval::{ApprovalManager, AuditLog, PendingOperation, RiskLevel};
use crate::ssh::{DeploymentScripts, SshManager};
use crate::tools::{Tool, ToolResult};

/// 远程工具上下文
pub struct RemoteContext {
    pub ssh_manager: SshManager,
    pub approval_manager: ApprovalManager,
}

impl RemoteContext {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ssh_manager: SshManager::from_env()?,
            approval_manager: ApprovalManager::new(),
        })
    }
}

// ============================================================================
// 远程部署工具
// ============================================================================

/// 列出可用服务器
pub struct ListServers {
    pub context: Arc<Mutex<RemoteContext>>,
}

#[async_trait]
impl Tool for ListServers {
    fn name(&self) -> &'static str { "list_servers" }

    fn description(&self) -> &'static str {
        "列出所有配置的远程服务器"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        let ctx = self.context.lock().await;
        let servers = ctx.ssh_manager.list_servers();

        if servers.is_empty() {
            return Ok(ToolResult::success(
                "未配置任何服务器。请设置环境变量 SSH_SERVERS=\"name:host:user,...\""
            ));
        }

        let server_list: Vec<_> = servers.iter().map(|s| {
            serde_json::json!({
                "name": s.name,
                "host": s.host,
                "port": s.port,
                "user": s.user
            })
        }).collect();

        Ok(ToolResult::success_with_data(
            format!("共有 {} 个配置的服务器", server_list.len()),
            serde_json::json!({ "servers": server_list })
        ))
    }
}

/// SSH 远程执行命令
pub struct SshExecute {
    pub context: Arc<Mutex<RemoteContext>>,
}

#[async_trait]
impl Tool for SshExecute {
    fn name(&self) -> &'static str { "ssh_execute" }

    fn description(&self) -> &'static str {
        "在远程服务器上执行命令（需要审批）"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "server": {
                    "type": "string",
                    "description": "服务器名称（从 list_servers 获取）"
                },
                "command": {
                    "type": "string",
                    "description": "要执行的命令"
                }
            },
            "required": ["server", "command"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let server = args["server"].as_str()
            .ok_or_else(|| anyhow!("Missing server parameter"))?;
        let command = args["command"].as_str()
            .ok_or_else(|| anyhow!("Missing command parameter"))?;

        let ctx = self.context.lock().await;

        // 评估风险级别
        let risk_level = ctx.approval_manager.assess_risk(command);

        // 创建待审批操作
        let mut operation = PendingOperation::new(
            "ssh_execute",
            &format!("Execute command on {}", server),
            server,
            command,
            risk_level,
        );

        // 请求审批
        let approved = ctx.approval_manager.request_approval(&mut operation).await?;

        if !approved {
            return Ok(ToolResult::error("操作已被拒绝"));
        }

        // 执行命令
        info!("Executing approved command on {}: {}", server, command);

        match ctx.ssh_manager.execute(server, command).await {
            Ok(result) => {
                // 记录审计日志
                let audit = AuditLog::from_operation(
                    &operation,
                    Some(&result.stdout),
                    Some(result.exit_code),
                );
                audit.log();

                if result.success() {
                    Ok(ToolResult::success_with_data(
                        format!("命令执行成功\n{}", result.stdout),
                        serde_json::json!({
                            "exit_code": result.exit_code,
                            "stdout": result.stdout,
                            "stderr": result.stderr
                        })
                    ))
                } else {
                    Ok(ToolResult::error(format!(
                        "命令执行失败 (exit code: {})\nstdout: {}\nstderr: {}",
                        result.exit_code, result.stdout, result.stderr
                    )))
                }
            }
            Err(e) => {
                let audit = AuditLog::from_operation(&operation, Some(&e.to_string()), None);
                audit.log();
                Ok(ToolResult::error(format!("SSH 执行失败: {}", e)))
            }
        }
    }
}

/// 远程部署节点
pub struct DeployNode {
    pub context: Arc<Mutex<RemoteContext>>,
}

#[async_trait]
impl Tool for DeployNode {
    fn name(&self) -> &'static str { "deploy_node" }

    fn description(&self) -> &'static str {
        "在远程服务器上部署 Substrate 节点（需要审批）"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "server": {
                    "type": "string",
                    "description": "目标服务器名称"
                },
                "chain": {
                    "type": "string",
                    "description": "链名称，如 cosmos-mainnet, cosmos-testnet"
                },
                "node_type": {
                    "type": "string",
                    "enum": ["validator", "full", "archive"],
                    "description": "节点类型"
                }
            },
            "required": ["server", "chain", "node_type"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let server = args["server"].as_str()
            .ok_or_else(|| anyhow!("Missing server parameter"))?;
        let chain = args["chain"].as_str()
            .ok_or_else(|| anyhow!("Missing chain parameter"))?;
        let node_type = args["node_type"].as_str()
            .ok_or_else(|| anyhow!("Missing node_type parameter"))?;

        // 生成部署脚本
        let install_script = DeploymentScripts::install_node(chain, node_type);

        let ctx = self.context.lock().await;

        // 创建待审批操作（部署是高风险操作）
        let mut operation = PendingOperation::new(
            "deploy_node",
            &format!("Deploy {} {} node on {}", chain, node_type, server),
            server,
            &install_script,
            RiskLevel::High,
        );

        // 请求审批
        let approved = ctx.approval_manager.request_approval(&mut operation).await?;

        if !approved {
            return Ok(ToolResult::error("部署操作已被拒绝"));
        }

        // 执行部署
        info!("Deploying {} {} node on {}", chain, node_type, server);

        match ctx.ssh_manager.execute(server, &install_script).await {
            Ok(result) => {
                let audit = AuditLog::from_operation(
                    &operation,
                    Some(&result.stdout),
                    Some(result.exit_code),
                );
                audit.log();

                if result.success() {
                    Ok(ToolResult::success_with_data(
                        format!("节点部署成功！\n{}", result.stdout),
                        serde_json::json!({
                            "server": server,
                            "chain": chain,
                            "node_type": node_type,
                            "status": "installed"
                        })
                    ))
                } else {
                    Ok(ToolResult::error(format!(
                        "部署失败: {}\n{}",
                        result.stderr, result.stdout
                    )))
                }
            }
            Err(e) => Ok(ToolResult::error(format!("部署失败: {}", e)))
        }
    }
}

/// 远程检查服务器状态
pub struct RemoteStatus {
    pub context: Arc<Mutex<RemoteContext>>,
}

#[async_trait]
impl Tool for RemoteStatus {
    fn name(&self) -> &'static str { "remote_status" }

    fn description(&self) -> &'static str {
        "检查远程服务器和节点状态（安全的只读操作）"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "server": {
                    "type": "string",
                    "description": "服务器名称"
                }
            },
            "required": ["server"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let server = args["server"].as_str()
            .ok_or_else(|| anyhow!("Missing server parameter"))?;

        let ctx = self.context.lock().await;
        let status_script = DeploymentScripts::check_status();

        match ctx.ssh_manager.execute(server, &status_script).await {
            Ok(result) => {
                Ok(ToolResult::success_with_data(
                    format!("服务器 {} 状态:\n{}", server, result.stdout),
                    serde_json::json!({
                        "server": server,
                        "raw_output": result.stdout
                    })
                ))
            }
            Err(e) => Ok(ToolResult::error(format!("无法获取状态: {}", e)))
        }
    }
}

/// 启动远程节点
pub struct StartNode {
    pub context: Arc<Mutex<RemoteContext>>,
}

#[async_trait]
impl Tool for StartNode {
    fn name(&self) -> &'static str { "start_node" }

    fn description(&self) -> &'static str {
        "启动远程服务器上的节点服务（需要审批）"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "server": {
                    "type": "string",
                    "description": "服务器名称"
                }
            },
            "required": ["server"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let server = args["server"].as_str()
            .ok_or_else(|| anyhow!("Missing server parameter"))?;

        let ctx = self.context.lock().await;
        let start_script = DeploymentScripts::start_node();

        let mut operation = PendingOperation::new(
            "start_node",
            &format!("Start node service on {}", server),
            server,
            &start_script,
            RiskLevel::Medium,
        );

        let approved = ctx.approval_manager.request_approval(&mut operation).await?;
        if !approved {
            return Ok(ToolResult::error("操作已被拒绝"));
        }

        match ctx.ssh_manager.execute(server, &start_script).await {
            Ok(result) => {
                let audit = AuditLog::from_operation(&operation, Some(&result.stdout), Some(result.exit_code));
                audit.log();

                if result.success() {
                    Ok(ToolResult::success(format!("节点已启动\n{}", result.stdout)))
                } else {
                    Ok(ToolResult::error(format!("启动失败: {}", result.stderr)))
                }
            }
            Err(e) => Ok(ToolResult::error(format!("执行失败: {}", e)))
        }
    }
}

/// 停止远程节点
pub struct StopNode {
    pub context: Arc<Mutex<RemoteContext>>,
}

#[async_trait]
impl Tool for StopNode {
    fn name(&self) -> &'static str { "stop_node" }

    fn description(&self) -> &'static str {
        "停止远程服务器上的节点服务（需要审批）"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "server": {
                    "type": "string",
                    "description": "服务器名称"
                }
            },
            "required": ["server"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let server = args["server"].as_str()
            .ok_or_else(|| anyhow!("Missing server parameter"))?;

        let ctx = self.context.lock().await;
        let stop_script = DeploymentScripts::stop_node();

        let mut operation = PendingOperation::new(
            "stop_node",
            &format!("Stop node service on {}", server),
            server,
            &stop_script,
            RiskLevel::High,
        );

        let approved = ctx.approval_manager.request_approval(&mut operation).await?;
        if !approved {
            return Ok(ToolResult::error("操作已被拒绝"));
        }

        match ctx.ssh_manager.execute(server, &stop_script).await {
            Ok(result) => {
                let audit = AuditLog::from_operation(&operation, Some(&result.stdout), Some(result.exit_code));
                audit.log();

                if result.success() {
                    Ok(ToolResult::success(format!("节点已停止\n{}", result.stdout)))
                } else {
                    Ok(ToolResult::error(format!("停止失败: {}", result.stderr)))
                }
            }
            Err(e) => Ok(ToolResult::error(format!("执行失败: {}", e)))
        }
    }
}

/// 生成 Ansible Playbook
pub struct GenerateAnsiblePlaybook;

#[async_trait]
impl Tool for GenerateAnsiblePlaybook {
    fn name(&self) -> &'static str { "generate_ansible_playbook" }

    fn description(&self) -> &'static str {
        "生成用于批量部署节点的 Ansible Playbook"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "chain": {
                    "type": "string",
                    "description": "链名称"
                },
                "node_type": {
                    "type": "string",
                    "enum": ["validator", "full", "archive"],
                    "description": "节点类型"
                },
                "servers": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "目标服务器列表"
                }
            },
            "required": ["chain", "node_type"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let chain = args["chain"].as_str().unwrap_or("cosmos");
        let node_type = args["node_type"].as_str().unwrap_or("full");

        let playbook = generate_ansible_playbook(chain, node_type);
        let inventory = generate_ansible_inventory();

        Ok(ToolResult::success_with_data(
            "已生成 Ansible Playbook，请保存以下文件",
            serde_json::json!({
                "playbook.yml": playbook,
                "inventory.ini": inventory,
                "usage": "ansible-playbook -i inventory.ini playbook.yml"
            })
        ))
    }
}

fn generate_ansible_playbook(chain: &str, node_type: &str) -> String {
    format!(r#"---
# Ansible Playbook: Deploy Substrate Node
# Generated by Node Operator Agent

- name: Deploy {chain} {node_type} node
  hosts: substrate_nodes
  become: yes
  vars:
    chain_name: "{chain}"
    node_type: "{node_type}"
    install_dir: "/opt/substrate-node"
    data_dir: "/var/lib/substrate-node"

  tasks:
    - name: Install dependencies
      apt:
        name:
          - curl
          - wget
          - jq
        state: present
        update_cache: yes
      when: ansible_os_family == "Debian"

    - name: Create installation directory
      file:
        path: "{{{{ install_dir }}}}"
        state: directory
        mode: '0755'

    - name: Create data directory
      file:
        path: "{{{{ data_dir }}}}"
        state: directory
        mode: '0755'

    - name: Download node binary
      get_url:
        url: "https://releases.example.com/{{{{ chain_name }}}}/node"
        dest: "{{{{ install_dir }}}}/node"
        mode: '0755'
      # 注意：需要替换为实际的下载地址

    - name: Create systemd service
      template:
        src: substrate-node.service.j2
        dest: /etc/systemd/system/substrate-node.service
      notify: Restart substrate-node

    - name: Enable and start service
      systemd:
        name: substrate-node
        enabled: yes
        state: started
        daemon_reload: yes

  handlers:
    - name: Restart substrate-node
      systemd:
        name: substrate-node
        state: restarted
"#, chain = chain, node_type = node_type)
}

fn generate_ansible_inventory() -> String {
    r#"# Ansible Inventory
# 编辑此文件添加你的服务器

[substrate_nodes]
# node1 ansible_host=192.168.1.10 ansible_user=root
# node2 ansible_host=192.168.1.11 ansible_user=root
# node3 ansible_host=192.168.1.12 ansible_user=root

[substrate_nodes:vars]
ansible_python_interpreter=/usr/bin/python3
"#.to_string()
}
