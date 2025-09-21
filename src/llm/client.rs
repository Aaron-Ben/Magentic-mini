use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::collections::HashMap;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::types::message::{FunctionCall, LLMMessage};

#[async_trait]
pub trait ChatCompletionClient: Send + Sync {
    async fn create(
        &self,
        messages: Vec<LLMMessage>,
        tools: Option<Vec<ToolSchema>>,
        json_output: Option<bool>,
        extra_create_args: Option<HashMap<String, Value>>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<CreateResult>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionExecutionResult {
    pub content: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinishReasons {
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "length")]
    Length,
    #[serde(rename = "function_calls")]
    FunctionCalls,
    #[serde(rename = "content_filter")]
    ContentFilter,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    Text(String),
    FunctionCalls(Vec<FunctionCall>),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResult {
    pub finish_reason: FinishReasons,
    pub content: Content,
    pub usage: RequestUsage,
    pub cached: bool,
    pub logprobs: Option<Vec<ChatCompletionTokenLogprob>>,
    pub thought: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionTokenLogprob {
    pub token: String,
    pub logprob: f32,
    pub top_logprobs: Option<Vec<TopLogprob>>,
    pub bytes: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopLogprob {
    pub token: String,
    pub logprob: f32,
    pub bytes: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model: "qwen-turbo".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

// 为dyn ChatCompletionClient手动实现Debug trait
impl std::fmt::Debug for dyn ChatCompletionClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatCompletionClient").finish()
    }
}

#[derive(Debug, Clone)]
pub struct LlmClient {
    client: Client,
    api_key: String,
    base_url: String,
    config: LlmConfig,
}

impl LlmClient {
    pub fn new() -> Result<Self> {
        Self::new_with_config(LlmConfig::default())
    }
    
    pub fn new_with_config(config: LlmConfig) -> Result<Self> {
        let api_key = env::var("DASHSCOPE_API_KEY")
            .context("DASHSCOPE_API_KEY environment variable not found")?;
        
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            client,
            api_key,
            base_url: "https://dashscope.aliyuncs.com/api/v1".to_string(),
            config,
        })
    }
    
    pub async fn create(
        &self,
        messages: Vec<LLMMessage>,
        tools: Option<Vec<ToolSchema>>,
        json_output: Option<bool>,
        extra_create_args: Option<HashMap<String, Value>>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<CreateResult> {
        // 检查取消令牌
        if let Some(token) = &cancellation_token {
            if token.is_cancelled() {
                return Err(anyhow::anyhow!("Request cancelled"));
            }
        }
        
        // 验证输入
        if messages.is_empty() {
            return Err(anyhow::anyhow!("Messages cannot be empty"));
        }
        
        let request_body = self.build_request_body(&messages, json_output, extra_create_args)?;
        
        let mut last_error = None;
        for attempt in 0..self.config.max_retries {
            // 再次检查取消令牌
            if let Some(token) = &cancellation_token {
                if token.is_cancelled() {
                    return Err(anyhow::anyhow!("Request cancelled"));
                }
            }
            
            match self.try_create(&request_body).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries - 1 {
                        tokio::time::sleep(Duration::from_millis(2u64.pow(attempt) * 100)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }
    
    fn build_request_body(
        &self, 
        messages: &[LLMMessage], 
        json_output: Option<bool>,
        extra_create_args: Option<HashMap<String, Value>>,
    ) -> Result<Value> {
        let mut parameters = json!({
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens
        });

        // 处理json_output
        if let Some(true) = json_output {
            parameters["result_format"] = json!("message");
            parameters["response_format"] = json!({"type": "json_object"});
        }

        // 合并extra_create_args到parameters
        if let Some(extra) = extra_create_args {
            for (key, value) in extra {
                parameters[key] = value;
            }
        }

        let request_body = json!({
            "model": self.config.model,
            "input": {
                "messages": messages.iter().map(|msg| {
                    match msg {
                        LLMMessage::SystemMessage ( content ) => json!({
                            "role": "system",
                            "content": content
                        }),
                        LLMMessage::UserMessage ( content ) => json!({
                            "role": "user", 
                            "content": content
                        }),
                        LLMMessage::AssistantMessage ( content ) => json!({
                            "role": "assistant",
                            "content": content
                        }),
                        LLMMessage::FunctionExecutionResultMessage ( content ) => {
                            // 处理函数执行结果
                            json!({
                                "role": "tool",
                                "content": content.content.iter().map(|result| {
                                    json!({
                                        "name": result.name,
                                        "content": result.content
                                    })
                                }).collect::<Vec<_>>()
                            })
                        }
                    }
                }).collect::<Vec<_>>()
            },
            "parameters": parameters
        });

        Ok(request_body)
    }
    
    async fn try_create(&self, request_body: &Value) -> Result<CreateResult> {
        let url = format!("{}/services/aigc/text-generation/generation", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request_body)
            .send()
            .await
            .context("Failed to send request to DashScope API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("API request failed with status {}: {}", status, error_text));
        }

        let response_json: Value = response.json().await
            .context("Failed to parse JSON response")?;
        
        self.parse_create_result(response_json)
    }
    
    fn parse_create_result(&self, response_json: Value) -> Result<CreateResult> {
        let output = response_json["output"].as_object()
            .context("Invalid response: missing output object")?;
        
        let choices = output["choices"].as_array()
            .context("Invalid response: missing choices array")?;
        
        if choices.is_empty() {
            return Err(anyhow::anyhow!("Invalid response: empty choices array"));
        }
        
        let choice = &choices[0];
        let message = choice["message"].as_object()
            .context("Invalid response: missing message object")?;
        
        let content = message["content"].as_str()
            .context("Invalid response: missing content")?;
            
        // 解析finish_reason (DashScope可能没有直接提供，这里使用默认值)
        let finish_reason = if content.len() >= self.config.max_tokens as usize {
            FinishReasons::Length
        } else {
            FinishReasons::Stop
        };
        
        // 估算token使用量（实际应该从API响应中获取）
        let usage = RequestUsage {
            prompt_tokens: 0, // 需要从API响应中获取
            completion_tokens: content.len() as i32 / 4, // 粗略估算
        };
        
        Ok(CreateResult {
            finish_reason,
            content: Content::Text(content.to_string()),
            usage,
            cached: false,
            logprobs: None,
            thought: None,
        })
    }
    
    // 获取当前配置
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }
    
    // 更新配置
    pub fn update_config(&mut self, config: LlmConfig) {
        self.config = config;
        // 如果需要，可以在这里重新创建client以应用新的超时设置
    }
}

// 工具相关类型（简化版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[async_trait]
impl ChatCompletionClient for LlmClient {
    async fn create(
        &self,
        messages: Vec<LLMMessage>,
        tools: Option<Vec<ToolSchema>>,
        json_output: Option<bool>,
        extra_create_args: Option<HashMap<String, Value>>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<CreateResult> {
        self.create(messages, tools, json_output, extra_create_args, cancellation_token).await
    }
}