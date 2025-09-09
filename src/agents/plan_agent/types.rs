use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use serde_json::Value;

// ContentPart用于多模态消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: String },
}

// 消息类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    MultiModal,
    Stop,
}

// LLM消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

// LLM响应中的选择项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub message: LlmMessage,
    pub finish_reason: Option<String>,
}

// LLM响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub choices: Vec<Choice>,
    pub usage: Option<Value>,
}

// 计划步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub title: String,
    pub details: String,
    pub agent_name: String,
    pub step_type: Option<String>,
    pub condition: Option<Value>, // 可以是数字或字符串
    pub sleep_duration: Option<i64>,
}

// 计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub response: String,
    pub task: String,
    pub plan_summary: String,
    pub needs_plan: bool,
    pub steps: Vec<PlanStep>,
}

impl Plan {
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseMessage {
    pub source: Option<String>,
    pub metadata: HashMap<String, String>,
    pub timestamp: Option<DateTime<Utc>>,
}

// 对应Python的TextMessage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMessage {
    #[serde(flatten)]
    pub base: BaseMessage,
    pub content: String,
}

// MultiModalMessage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalMessage {
    #[serde(flatten)]
    pub base: BaseMessage,
    pub content: Vec<ContentPart>,
}

// StopMessage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopMessage {
    #[serde(flatten)]
    pub base: BaseMessage,
    pub content: String,
}

// 统一的消息枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatMessage {
    Text(TextMessage),
    MultiModal(MultiModalMessage),
    Stop(StopMessage),
}

impl ChatMessage {
    pub fn source(&self) -> Option<&str> {
        match self {
            ChatMessage::Text(msg) => msg.base.source.as_deref(),
            ChatMessage::MultiModal(msg) => msg.base.source.as_deref(),
            ChatMessage::Stop(msg) => msg.base.source.as_deref(),
        }
    }
    
    pub fn metadata(&self) -> &HashMap<String, String> {
        match self {
            ChatMessage::Text(msg) => &msg.base.metadata,
            ChatMessage::MultiModal(msg) => &msg.base.metadata,
            ChatMessage::Stop(msg) => &msg.base.metadata,
        }
    }
    
    pub fn content(&self) -> String {
        match self {
            ChatMessage::Text(msg) => msg.content.clone(),
            ChatMessage::MultiModal(msg) => {
                // 简化处理，返回第一个文本内容
                msg.content.iter().find_map(|part| {
                    if let ContentPart::Text { text } = part {
                        Some(text.clone())
                    } else {
                        None
                    }
                }).unwrap_or_default()
            },
            ChatMessage::Stop(msg) => msg.content.clone(),
        }
    }
}