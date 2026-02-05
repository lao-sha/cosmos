//! Agent 核心逻辑 - 协调 LLM 与工具执行

use anyhow::Result;
use tracing::{info, warn};

use crate::llm_client::{create_llm_client, LlmClient, Message};
use crate::tools::{ToolRegistry, ToolResult};

/// 节点运维 Agent
pub struct NodeOperatorAgent {
    llm: Box<dyn LlmClient>,
    tools: ToolRegistry,
    system_prompt: String,
}

impl NodeOperatorAgent {
    /// 创建仅包含本地工具的 Agent
    pub fn new() -> Result<Self> {
        Self::with_tools(ToolRegistry::new())
    }

    /// 创建包含远程部署工具的 Agent
    pub fn with_remote() -> Result<Self> {
        Self::with_tools(ToolRegistry::with_remote_tools())
    }

    fn with_tools(tools: ToolRegistry) -> Result<Self> {
        let llm = create_llm_client()?;
        
        let system_prompt = r#"你是一个 Substrate 区块链节点运维助手。你的职责是帮助用户管理和诊断节点。

可用工具分为两类:

【本地工具】
- get_node_status: 获取节点运行状态
- diagnose_node: 诊断节点问题
- get_node_logs: 获取节点日志
- generate_chain_spec: 生成节点配置建议
- check_system_resources: 检查系统资源
- list_nodes: 列出管理的节点

【远程部署工具】（如果可用）
- list_servers: 列出配置的远程服务器
- remote_status: 检查远程服务器状态
- ssh_execute: 在远程服务器执行命令（需审批）
- deploy_node: 部署节点到远程服务器（需审批）
- start_node: 启动远程节点服务（需审批）
- stop_node: 停止远程节点服务（需审批）
- generate_ansible_playbook: 生成 Ansible 批量部署脚本

工作原则:
1. 优先使用工具获取实际数据，不要猜测
2. 诊断问题时，先检查基础连接性，再深入分析
3. 给出的建议要具体、可操作
4. 远程执行命令前，系统会请求用户审批
5. 使用中文回复用户

当用户描述问题时，主动调用相关工具进行诊断。"#.to_string();
        
        info!("NodeOperatorAgent initialized with {} tools", tools.get_all_defs().len());
        
        Ok(Self { llm, tools, system_prompt })
    }
    
    /// 处理用户请求
    pub async fn chat(&self, user_input: &str) -> Result<String> {
        let messages = vec![
            Message::system(&self.system_prompt),
            Message::user(user_input),
        ];
        
        let tool_defs = self.tools.get_all_defs();
        
        // 第一轮：LLM 理解意图并决定是否调用工具
        let response = self.llm.chat(messages.clone(), Some(tool_defs.clone())).await?;
        
        if response.tool_calls.is_empty() {
            // 无需工具调用，直接返回
            return Ok(response.content);
        }
        
        // 执行工具调用
        let mut tool_results = Vec::new();
        for call in &response.tool_calls {
            info!("Executing tool: {}", call.name);
            match self.tools.execute(&call.name, call.arguments.clone()).await {
                Ok(result) => {
                    tool_results.push((call.name.clone(), result));
                }
                Err(e) => {
                    warn!("Tool execution failed: {}", e);
                    tool_results.push((
                        call.name.clone(),
                        ToolResult::error(format!("工具执行失败: {}", e))
                    ));
                }
            }
        }
        
        // 构造包含工具结果的消息
        let tool_results_text = tool_results.iter()
            .map(|(name, result)| {
                format!("工具 {} 执行结果:\n{}\n数据: {}",
                    name,
                    result.output,
                    result.data.as_ref().map(|d| d.to_string()).unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        
        // 第二轮：LLM 根据工具结果生成最终回复
        let final_messages = vec![
            Message::system(&self.system_prompt),
            Message::user(user_input),
            Message::assistant(&response.content),
            Message::user(format!("工具执行结果:\n{}\n\n请根据以上结果回答用户的问题。", tool_results_text)),
        ];
        
        let final_response = self.llm.chat(final_messages, None).await?;
        
        Ok(final_response.content)
    }
    
    /// 交互式对话（多轮）
    pub async fn interactive_chat(&self, history: &mut Vec<Message>, user_input: &str) -> Result<String> {
        history.push(Message::user(user_input));
        
        let mut messages = vec![Message::system(&self.system_prompt)];
        messages.extend(history.clone());
        
        let tool_defs = self.tools.get_all_defs();
        let response = self.llm.chat(messages, Some(tool_defs)).await?;
        
        // 处理工具调用（同上逻辑）
        let final_content = if response.tool_calls.is_empty() {
            response.content.clone()
        } else {
            let mut tool_results = Vec::new();
            for call in &response.tool_calls {
                match self.tools.execute(&call.name, call.arguments.clone()).await {
                    Ok(result) => tool_results.push((call.name.clone(), result)),
                    Err(e) => tool_results.push((
                        call.name.clone(),
                        ToolResult::error(format!("执行失败: {}", e))
                    )),
                }
            }
            
            let tool_results_text = tool_results.iter()
                .map(|(name, result)| format!("[{}]: {}", name, result.output))
                .collect::<Vec<_>>()
                .join("\n");
            
            // 让 LLM 总结
            let summary_messages = vec![
                Message::system(&self.system_prompt),
                Message::user(user_input),
                Message::user(format!("工具结果:\n{}\n\n请总结回复。", tool_results_text)),
            ];
            
            self.llm.chat(summary_messages, None).await?.content
        };
        
        history.push(Message::assistant(&final_content));
        Ok(final_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // 需要 API KEY
    async fn test_agent_basic() {
        let agent = NodeOperatorAgent::new().unwrap();
        let response = agent.chat("列出所有节点").await.unwrap();
        println!("Response: {}", response);
    }
}
