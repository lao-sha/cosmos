//! Node Operator CLI - 节点运维命令行工具

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use tracing_subscriber::EnvFilter;

use node_operator::{NodeOperatorAgent, Message};

#[derive(Parser)]
#[command(name = "node-operator")]
#[command(about = "LLM 驱动的 Substrate 节点运维助手", long_about = None)]
struct Cli {
    /// 启用远程部署功能
    #[arg(short, long, global = true)]
    remote: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 单次对话
    Chat {
        /// 用户输入
        message: String,
    },
    
    /// 交互式对话模式
    Interactive,
    
    /// 列出可用工具
    Tools,
    
    /// 快速诊断节点
    Diagnose {
        /// 节点 RPC 端点
        #[arg(short, long, default_value = "http://127.0.0.1:9944")]
        endpoint: String,
    },
    
    /// 获取节点状态
    Status {
        /// 节点 RPC 端点
        #[arg(short, long, default_value = "http://127.0.0.1:9944")]
        endpoint: String,
    },

    /// 远程部署节点
    Deploy {
        /// 目标服务器名称
        #[arg(short, long)]
        server: String,

        /// 链名称
        #[arg(short, long, default_value = "cosmos")]
        chain: String,

        /// 节点类型
        #[arg(short, long, default_value = "full")]
        node_type: String,
    },

    /// 列出远程服务器
    Servers,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("node_operator=info".parse()?))
        .init();
    
    // 加载 .env 文件
    dotenvy::dotenv().ok();
    
    let cli = Cli::parse();
    
    let use_remote = cli.remote;

    match cli.command {
        Commands::Chat { message } => {
            run_chat(&message, use_remote).await?;
        }
        Commands::Interactive => {
            run_interactive(use_remote).await?;
        }
        Commands::Tools => {
            list_tools(use_remote);
        }
        Commands::Diagnose { endpoint } => {
            run_diagnose(&endpoint, use_remote).await?;
        }
        Commands::Status { endpoint } => {
            run_status(&endpoint, use_remote).await?;
        }
        Commands::Deploy { server, chain, node_type } => {
            run_deploy(&server, &chain, &node_type).await?;
        }
        Commands::Servers => {
            list_servers();
        }
    }
    
    Ok(())
}

async fn run_chat(message: &str, use_remote: bool) -> Result<()> {
    println!("🤖 正在分析...\n");
    
    let agent = if use_remote {
        NodeOperatorAgent::with_remote()?
    } else {
        NodeOperatorAgent::new()?
    };
    let response = agent.chat(message).await?;
    
    println!("{}", response);
    Ok(())
}

async fn run_interactive(use_remote: bool) -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    if use_remote {
        println!("║   Node Operator Agent - 交互式模式 (远程已启用)            ║");
    } else {
        println!("║   Node Operator Agent - 交互式模式                        ║");
    }
    println!("║   输入 'quit' 或 'exit' 退出                                ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    let agent = if use_remote {
        NodeOperatorAgent::with_remote()?
    } else {
        NodeOperatorAgent::new()?
    };
    let mut history: Vec<Message> = Vec::new();
    
    loop {
        print!("👤 You: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == "quit" || input == "exit" {
            println!("👋 再见！");
            break;
        }
        
        if input == "clear" {
            history.clear();
            println!("🧹 对话历史已清除\n");
            continue;
        }
        
        println!("\n🤖 Agent: 正在思考...");
        
        match agent.interactive_chat(&mut history, input).await {
            Ok(response) => {
                // 清除 "正在思考..." 行
                print!("\x1B[1A\x1B[2K");
                println!("🤖 Agent: {}\n", response);
            }
            Err(e) => {
                print!("\x1B[1A\x1B[2K");
                println!("❌ 错误: {}\n", e);
            }
        }
    }
    
    Ok(())
}

fn list_tools(use_remote: bool) {
    use node_operator::ToolRegistry;
    
    let registry = if use_remote {
        ToolRegistry::with_remote_tools()
    } else {
        ToolRegistry::new()
    };
    let tools = registry.get_all_defs();
    
    println!("可用工具列表{}：\n", if use_remote { " (含远程工具)" } else { "" });
    for tool in tools {
        println!("📦 {}", tool.name);
        println!("   {}", tool.description);
        println!();
    }
}

async fn run_diagnose(endpoint: &str, use_remote: bool) -> Result<()> {
    println!("🔍 正在诊断节点 {}...\n", endpoint);
    
    let agent = if use_remote {
        NodeOperatorAgent::with_remote()?
    } else {
        NodeOperatorAgent::new()?
    };
    let prompt = format!(
        "请对节点 {} 进行全面诊断，检查连接性、同步状态、对等节点和系统资源。",
        endpoint
    );
    
    let response = agent.chat(&prompt).await?;
    println!("{}", response);
    
    Ok(())
}

async fn run_status(endpoint: &str, use_remote: bool) -> Result<()> {
    println!("📊 正在获取节点状态 {}...\n", endpoint);
    
    let agent = if use_remote {
        NodeOperatorAgent::with_remote()?
    } else {
        NodeOperatorAgent::new()?
    };
    let prompt = format!("获取节点 {} 的状态", endpoint);
    
    let response = agent.chat(&prompt).await?;
    println!("{}", response);
    
    Ok(())
}

async fn run_deploy(server: &str, chain: &str, node_type: &str) -> Result<()> {
    println!("🚀 准备部署节点到 {}...\n", server);
    println!("   链: {}", chain);
    println!("   类型: {}", node_type);
    println!();

    let agent = NodeOperatorAgent::with_remote()?;
    let prompt = format!(
        "请在服务器 {} 上部署一个 {} 链的 {} 节点",
        server, chain, node_type
    );

    let response = agent.chat(&prompt).await?;
    println!("{}", response);

    Ok(())
}

fn list_servers() {
    use node_operator::SshManager;

    println!("配置的远程服务器：\n");

    match SshManager::from_env() {
        Ok(manager) => {
            let servers = manager.list_servers();
            if servers.is_empty() {
                println!("未配置任何服务器。");
                println!("\n请设置环境变量:");
                println!("  SSH_SERVERS=\"name1:host1:user1,name2:host2:user2\"");
                println!("  SSH_KEY_PATH=\"/path/to/private/key\"");
            } else {
                for server in servers {
                    println!("🖥️  {}", server.name);
                    println!("   主机: {}:{}", server.host, server.port);
                    println!("   用户: {}", server.user);
                    println!();
                }
            }
        }
        Err(e) => {
            println!("❌ 无法加载服务器配置: {}", e);
        }
    }
}
