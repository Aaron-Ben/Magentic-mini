use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

// 基础消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMMessage {
    System { content: String },
    User { content: String },
    Assistant { content: String, thought: Option<String> },
    Tool { content: String, tool_call_id: String },
}

// 工具schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

// 模型客户端trait
#[async_trait]
pub trait ChatCompletionClient: Send + Sync {
    async fn count_tokens(&self, messages: &[LLMMessage], tools: &[ToolSchema]) -> Result<i32, Box<dyn std::error::Error>>;
    async fn remaining_tokens(&self, messages: &[LLMMessage], tools: &[ToolSchema]) -> Result<i32, Box<dyn std::error::Error>>;
}

// 配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLimitedChatCompletionContextConfig {
    pub token_limit: Option<i32>,
    pub tool_schema: Vec<ToolSchema>,
    pub initial_messages: Vec<LLMMessage>,
}

// 主要结构体
#[derive(Debug)]
pub struct TokenLimitedChatCompletionContext {
    model_client: Arc<dyn ChatCompletionClient>,
    token_limit: Option<i32>,
    tool_schema: Vec<ToolSchema>,
    messages: Vec<LLMMessage>,
    initial_messages: Vec<LLMMessage>,
}

impl TokenLimitedChatCompletionContext {
    /// 创建新的上下文管理器
    pub fn new(
        model_client: Arc<dyn ChatCompletionClient>,
        token_limit: Option<i32>,
        tool_schema: Vec<ToolSchema>,
        initial_messages: Vec<LLMMessage>,
    ) -> Self {
        let mut messages = Vec::new();
        messages.extend(initial_messages.clone());
        
        Self {
            model_client,
            token_limit,
            tool_schema,
            messages,
            initial_messages,
        }
    }

    /// 添加消息到上下文
    pub async fn add_message(&mut self, message: LLMMessage) -> Result<(), Box<dyn std::error::Error>> {
        self.messages.push(message);
        Ok(())
    }

    /// 获取限制token数量的消息列表
    pub async fn get_messages(&self) -> Result<Vec<LLMMessage>, Box<dyn std::error::Error>> {
        let mut messages = self.messages.clone();
        
        match self.token_limit {
            Some(limit) => {
                // 使用固定token限制
                while !messages.is_empty() {
                    let token_count = self.model_client.count_tokens(&messages, &self.tool_schema).await?;
                    if token_count <= limit {
                        break;
                    }
                    
                    // 从中间移除消息
                    let middle_index = messages.len() / 2;
                    messages.remove(middle_index);
                }
            }
            None => {
                // 使用模型客户端的remaining_tokens方法
                while !messages.is_empty() {
                    let remaining = self.model_client.remaining_tokens(&messages, &self.tool_schema).await?;
                    if remaining >= 0 {
                        break;
                    }
                    
                    // 从中间移除消息
                    let middle_index = messages.len() / 2;
                    messages.remove(middle_index);
                }
            }
        }
        
        // 处理特殊情况：第一个消息是工具调用结果
        if let Some(LLMMessage::Tool { .. }) = messages.first() {
            messages.remove(0);
        }
        
        Ok(messages)
    }

    /// 清空上下文
    pub async fn clear(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.messages.clear();
        Ok(())
    }

    /// 保存状态
    pub async fn save_state(&self) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error>> {
        let state = serde_json::to_value(&ChatCompletionContextState {
            messages: self.messages.clone(),
        })?;
        
        if let serde_json::Value::Object(map) = state {
            Ok(map.into_iter().collect())
        } else {
            Err("Failed to serialize state".into())
        }
    }

    /// 加载状态
    pub async fn load_state(&mut self, state: HashMap<String, serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
        let state_value = serde_json::Value::Object(state.into_iter().collect());
        let context_state: ChatCompletionContextState = serde_json::from_value(state_value)?;
        self.messages = context_state.messages;
        Ok(())
    }
}

// 状态结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatCompletionContextState {
    messages: Vec<LLMMessage>,
}