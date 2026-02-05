//! LLM Client - 支持 Claude、OpenAI 和 DeepSeek API
//!
//! 环境变量配置:
//! - ANTHROPIC_API_KEY: Claude API 密钥
//! - OPENAI_API_KEY: OpenAI API 密钥
//! - DEEPSEEK_API_KEY: DeepSeek API 密钥
//! - LLM_PROVIDER: "claude" 或 "openai" 或 "deepseek"

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// LLM 提供商
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProvider {
    Claude,
    OpenAI,
    DeepSeek,
}

impl LlmProvider {
    pub fn from_env() -> Result<Self> {
        let provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "claude".to_string());
        match provider.to_lowercase().as_str() {
            "claude" | "anthropic" => Ok(Self::Claude),
            "openai" | "gpt" => Ok(Self::OpenAI),
            "deepseek" => Ok(Self::DeepSeek),
            _ => Err(anyhow!("Unknown LLM provider: {}", provider)),
        }
    }
}

/// LLM 客户端 trait
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn chat(&self, messages: Vec<Message>, tools: Option<Vec<ToolDef>>) -> Result<LlmResponse>;
}

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".to_string(), content: content.into() }
    }
    
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".to_string(), content: content.into() }
    }
    
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".to_string(), content: content.into() }
    }
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// LLM 响应
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Claude 客户端
pub struct ClaudeClient {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

impl ClaudeClient {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| anyhow!("ANTHROPIC_API_KEY not set"))?;
        
        Ok(Self {
            api_key,
            client: reqwest::Client::new(),
            model: "claude-3-5-sonnet-20241022".to_string(),
        })
    }
}

#[async_trait]
impl LlmClient for ClaudeClient {
    async fn chat(&self, messages: Vec<Message>, tools: Option<Vec<ToolDef>>) -> Result<LlmResponse> {
        let system_msg = messages.iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone())
            .unwrap_or_default();
        
        let user_messages: Vec<_> = messages.iter()
            .filter(|m| m.role != "system")
            .map(|m| serde_json::json!({
                "role": m.role,
                "content": m.content
            }))
            .collect();
        
        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "system": system_msg,
            "messages": user_messages
        });
        
        if let Some(tools) = tools {
            let claude_tools: Vec<_> = tools.iter().map(|t| serde_json::json!({
                "name": t.name,
                "description": t.description,
                "input_schema": t.parameters
            })).collect();
            body["tools"] = serde_json::json!(claude_tools);
        }
        
        debug!("Sending request to Claude API");
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;
        
        let status = response.status();
        let text = response.text().await?;
        
        if !status.is_success() {
            return Err(anyhow!("Claude API error ({}): {}", status, text));
        }
        
        let resp: serde_json::Value = serde_json::from_str(&text)?;
        
        let mut content = String::new();
        let mut tool_calls = Vec::new();
        
        if let Some(contents) = resp["content"].as_array() {
            for item in contents {
                match item["type"].as_str() {
                    Some("text") => {
                        content = item["text"].as_str().unwrap_or("").to_string();
                    }
                    Some("tool_use") => {
                        tool_calls.push(ToolCall {
                            name: item["name"].as_str().unwrap_or("").to_string(),
                            arguments: item["input"].clone(),
                        });
                    }
                    _ => {}
                }
            }
        }
        
        info!("Claude response received, {} tool calls", tool_calls.len());
        
        Ok(LlmResponse { content, tool_calls })
    }
}

/// OpenAI 客户端
pub struct OpenAIClient {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

impl OpenAIClient {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow!("OPENAI_API_KEY not set"))?;
        
        Ok(Self {
            api_key,
            client: reqwest::Client::new(),
            model: "gpt-4-turbo-preview".to_string(),
        })
    }
}

#[async_trait]
impl LlmClient for OpenAIClient {
    async fn chat(&self, messages: Vec<Message>, tools: Option<Vec<ToolDef>>) -> Result<LlmResponse> {
        let openai_messages: Vec<_> = messages.iter().map(|m| serde_json::json!({
            "role": m.role,
            "content": m.content
        })).collect();
        
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": openai_messages
        });
        
        if let Some(tools) = tools {
            let openai_tools: Vec<_> = tools.iter().map(|t| serde_json::json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters
                }
            })).collect();
            body["tools"] = serde_json::json!(openai_tools);
        }
        
        debug!("Sending request to OpenAI API");
        
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
        
        let status = response.status();
        let text = response.text().await?;
        
        if !status.is_success() {
            return Err(anyhow!("OpenAI API error ({}): {}", status, text));
        }
        
        let resp: serde_json::Value = serde_json::from_str(&text)?;
        
        let choice = &resp["choices"][0]["message"];
        let content = choice["content"].as_str().unwrap_or("").to_string();
        
        let mut tool_calls = Vec::new();
        if let Some(calls) = choice["tool_calls"].as_array() {
            for call in calls {
                if let Some(func) = call["function"].as_object() {
                    tool_calls.push(ToolCall {
                        name: func["name"].as_str().unwrap_or("").to_string(),
                        arguments: serde_json::from_str(
                            func["arguments"].as_str().unwrap_or("{}")
                        ).unwrap_or_default(),
                    });
                }
            }
        }
        
        info!("OpenAI response received, {} tool calls", tool_calls.len());
        
        Ok(LlmResponse { content, tool_calls })
    }
}

/// DeepSeek 客户端
/// DeepSeek API 与 OpenAI 兼容
pub struct DeepSeekClient {
    api_key: String,
    client: reqwest::Client,
    model: String,
    base_url: String,
}

impl DeepSeekClient {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("DEEPSEEK_API_KEY")
            .map_err(|_| anyhow!("DEEPSEEK_API_KEY not set"))?;
        
        // 支持自定义 base_url，方便使用代理
        let base_url = std::env::var("DEEPSEEK_BASE_URL")
            .unwrap_or_else(|_| "https://api.deepseek.com".to_string());
        
        // 支持自定义模型
        let model = std::env::var("DEEPSEEK_MODEL")
            .unwrap_or_else(|_| "deepseek-chat".to_string());
        
        Ok(Self {
            api_key,
            client: reqwest::Client::new(),
            model,
            base_url,
        })
    }
}

#[async_trait]
impl LlmClient for DeepSeekClient {
    async fn chat(&self, messages: Vec<Message>, tools: Option<Vec<ToolDef>>) -> Result<LlmResponse> {
        let deepseek_messages: Vec<_> = messages.iter().map(|m| serde_json::json!({
            "role": m.role,
            "content": m.content
        })).collect();
        
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": deepseek_messages,
            "max_tokens": 4096
        });
        
        // DeepSeek 支持 OpenAI 格式的工具调用
        if let Some(tools) = tools {
            let deepseek_tools: Vec<_> = tools.iter().map(|t| serde_json::json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters
                }
            })).collect();
            body["tools"] = serde_json::json!(deepseek_tools);
        }
        
        debug!("Sending request to DeepSeek API: {}", self.base_url);
        
        let url = format!("{}/v1/chat/completions", self.base_url);
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
        
        let status = response.status();
        let text = response.text().await?;
        
        if !status.is_success() {
            return Err(anyhow!("DeepSeek API error ({}): {}", status, text));
        }
        
        let resp: serde_json::Value = serde_json::from_str(&text)?;
        
        let choice = &resp["choices"][0]["message"];
        let content = choice["content"].as_str().unwrap_or("").to_string();
        
        let mut tool_calls = Vec::new();
        if let Some(calls) = choice["tool_calls"].as_array() {
            for call in calls {
                if let Some(func) = call["function"].as_object() {
                    tool_calls.push(ToolCall {
                        name: func["name"].as_str().unwrap_or("").to_string(),
                        arguments: serde_json::from_str(
                            func["arguments"].as_str().unwrap_or("{}")
                        ).unwrap_or_default(),
                    });
                }
            }
        }
        
        info!("DeepSeek response received, {} tool calls", tool_calls.len());
        
        Ok(LlmResponse { content, tool_calls })
    }
}

/// 创建 LLM 客户端
pub fn create_llm_client() -> Result<Box<dyn LlmClient>> {
    let provider = LlmProvider::from_env()?;
    match provider {
        LlmProvider::Claude => Ok(Box::new(ClaudeClient::new()?)),
        LlmProvider::OpenAI => Ok(Box::new(OpenAIClient::new()?)),
        LlmProvider::DeepSeek => Ok(Box::new(DeepSeekClient::new()?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");
    }
}
