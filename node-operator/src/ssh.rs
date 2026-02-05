//! SSH 连接管理器 - 安全的远程服务器操作
//!
//! 使用系统 ssh 命令执行远程操作，确保兼容性和稳定性

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::process::Command;
use tracing::{info, warn};

/// SSH 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub key_path: Option<String>,
}

impl ServerConfig {
    pub fn new(name: &str, host: &str, user: &str) -> Self {
        Self {
            name: name.to_string(),
            host: host.to_string(),
            port: 22,
            user: user.to_string(),
            key_path: None,
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_key(mut self, key_path: &str) -> Self {
        self.key_path = Some(key_path.to_string());
        self
    }
}

/// 命令执行结果
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub exit_code: u32,
    pub stdout: String,
    pub stderr: String,
}

impl CommandResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// SSH 连接管理器
pub struct SshManager {
    servers: HashMap<String, ServerConfig>,
    default_key_path: Option<String>,
}

impl SshManager {
    pub fn new() -> Self {
        let default_key = dirs::home_dir()
            .map(|h| h.join(".ssh/id_rsa").to_string_lossy().to_string());

        Self {
            servers: HashMap::new(),
            default_key_path: default_key,
        }
    }

    /// 从环境变量加载服务器配置
    pub fn from_env() -> Result<Self> {
        let mut manager = Self::new();

        // 格式: SSH_SERVERS="name1:host1:user1,name2:host2:user2"
        if let Ok(servers_env) = std::env::var("SSH_SERVERS") {
            for server_str in servers_env.split(',') {
                let parts: Vec<&str> = server_str.trim().split(':').collect();
                if parts.len() >= 3 {
                    let config = ServerConfig::new(parts[0], parts[1], parts[2]);
                    manager.add_server(config);
                }
            }
        }

        // SSH 密钥路径
        if let Ok(key_path) = std::env::var("SSH_KEY_PATH") {
            manager.default_key_path = Some(key_path);
        }

        Ok(manager)
    }

    /// 添加服务器配置
    pub fn add_server(&mut self, config: ServerConfig) {
        self.servers.insert(config.name.clone(), config);
    }

    /// 获取服务器列表
    pub fn list_servers(&self) -> Vec<&ServerConfig> {
        self.servers.values().collect()
    }

    /// 连接到服务器并执行命令
    pub async fn execute(&self, server_name: &str, command: &str) -> Result<CommandResult> {
        let config = self.servers.get(server_name)
            .ok_or_else(|| anyhow!("Server not found: {}", server_name))?;

        self.execute_on(config, command).await
    }

    /// 在指定配置的服务器上执行命令（使用系统 ssh）
    pub async fn execute_on(&self, config: &ServerConfig, command: &str) -> Result<CommandResult> {
        info!("Connecting to {}@{}:{}", config.user, config.host, config.port);

        // 构建 ssh 命令
        let mut ssh_cmd = Command::new("ssh");
        
        // 基本选项
        ssh_cmd.args([
            "-o", "StrictHostKeyChecking=accept-new",
            "-o", "BatchMode=yes",
            "-o", "ConnectTimeout=10",
        ]);

        // 端口
        if config.port != 22 {
            ssh_cmd.args(["-p", &config.port.to_string()]);
        }

        // SSH 密钥
        if let Some(key_path) = config.key_path.as_ref().or(self.default_key_path.as_ref()) {
            ssh_cmd.args(["-i", key_path]);
        }

        // 目标主机和命令
        let target = format!("{}@{}", config.user, config.host);
        ssh_cmd.arg(&target);
        ssh_cmd.arg(command);

        info!("Executing: ssh {} '{}'", target, command);

        // 执行命令
        let output = ssh_cmd.output()
            .map_err(|e| anyhow!("Failed to execute ssh command: {}", e))?;

        let exit_code = output.status.code().unwrap_or(-1) as u32;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            warn!("SSH command failed with exit code {}: {}", exit_code, stderr);
        }

        Ok(CommandResult {
            exit_code,
            stdout,
            stderr,
        })
    }

    /// 上传文件到远程服务器
    pub async fn upload_file(
        &self,
        server_name: &str,
        local_content: &[u8],
        remote_path: &str,
    ) -> Result<()> {
        // 使用 cat 命令写入文件（简化实现）
        // 生产环境应该使用 SFTP
        let content_base64 = base64_encode(local_content);
        let command = format!(
            "echo '{}' | base64 -d > {}",
            content_base64,
            remote_path
        );

        let result = self.execute(server_name, &command).await?;
        if !result.success() {
            return Err(anyhow!("Failed to upload file: {}", result.stderr));
        }

        Ok(())
    }
}

fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in data.chunks(3) {
        let mut n = (chunk[0] as u32) << 16;
        if chunk.len() > 1 {
            n |= (chunk[1] as u32) << 8;
        }
        if chunk.len() > 2 {
            n |= chunk[2] as u32;
        }
        
        result.push(ALPHABET[(n >> 18 & 0x3F) as usize] as char);
        result.push(ALPHABET[(n >> 12 & 0x3F) as usize] as char);
        
        if chunk.len() > 1 {
            result.push(ALPHABET[(n >> 6 & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        
        if chunk.len() > 2 {
            result.push(ALPHABET[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    
    result
}

/// 预定义的节点部署脚本
pub struct DeploymentScripts;

impl DeploymentScripts {
    /// 生成节点安装脚本
    pub fn install_node(chain: &str, node_type: &str) -> String {
        format!(r#"#!/bin/bash
set -e

echo "=== Installing Substrate Node ==="

# 安装依赖
if command -v apt-get &> /dev/null; then
    sudo apt-get update
    sudo apt-get install -y curl wget
elif command -v yum &> /dev/null; then
    sudo yum install -y curl wget
fi

# 下载节点二进制（示例）
CHAIN="{chain}"
NODE_TYPE="{node_type}"
INSTALL_DIR="/opt/substrate-node"

sudo mkdir -p $INSTALL_DIR
cd $INSTALL_DIR

# 这里应该替换为实际的下载地址
echo "Downloading node binary for $CHAIN..."
# wget -O node https://releases.example.com/$CHAIN/node
# chmod +x node

# 创建数据目录
sudo mkdir -p /var/lib/substrate-node

# 创建 systemd 服务
sudo tee /etc/systemd/system/substrate-node.service > /dev/null <<EOF
[Unit]
Description=Substrate Node
After=network.target

[Service]
Type=simple
User=root
ExecStart=$INSTALL_DIR/node --chain $CHAIN --{node_type}
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
echo "=== Installation complete ==="
"#, chain = chain, node_type = node_type)
    }

    /// 生成节点启动脚本
    pub fn start_node() -> String {
        r#"#!/bin/bash
sudo systemctl start substrate-node
sudo systemctl enable substrate-node
echo "Node started and enabled"
systemctl status substrate-node --no-pager
"#.to_string()
    }

    /// 生成节点停止脚本
    pub fn stop_node() -> String {
        r#"#!/bin/bash
sudo systemctl stop substrate-node
echo "Node stopped"
"#.to_string()
    }

    /// 生成节点状态检查脚本
    pub fn check_status() -> String {
        r#"#!/bin/bash
echo "=== System Status ==="
uptime
echo ""
echo "=== Disk Usage ==="
df -h /
echo ""
echo "=== Memory ==="
free -h
echo ""
echo "=== Node Service Status ==="
systemctl status substrate-node --no-pager 2>/dev/null || echo "Service not found"
echo ""
echo "=== Recent Logs ==="
journalctl -u substrate-node -n 20 --no-pager 2>/dev/null || echo "No logs found"
"#.to_string()
    }

    /// 生成节点升级脚本
    pub fn upgrade_node(download_url: &str) -> String {
        format!(r#"#!/bin/bash
set -e

echo "=== Upgrading Substrate Node ==="

# 停止服务
sudo systemctl stop substrate-node

# 备份旧版本
INSTALL_DIR="/opt/substrate-node"
sudo mv $INSTALL_DIR/node $INSTALL_DIR/node.bak || true

# 下载新版本
cd $INSTALL_DIR
sudo wget -O node "{download_url}"
sudo chmod +x node

# 启动服务
sudo systemctl start substrate-node

echo "=== Upgrade complete ==="
systemctl status substrate-node --no-pager
"#, download_url = download_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config() {
        let config = ServerConfig::new("test", "192.168.1.1", "root")
            .with_port(2222)
            .with_key("/path/to/key");

        assert_eq!(config.name, "test");
        assert_eq!(config.port, 2222);
    }

    #[test]
    fn test_deployment_scripts() {
        let script = DeploymentScripts::install_node("cosmos", "validator");
        assert!(script.contains("cosmos"));
        assert!(script.contains("validator"));
    }
}
