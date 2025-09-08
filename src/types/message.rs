use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,  // 模型返回
    Tool,       // function/tool 返回
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: Vec<ContentPart>,
    pub name: Option<String>,       // 发送者标识
    pub timestamp: Option<DateTime<Utc>>,
}

impl ChatMessage {
    pub fn from_text(
        role: Role,
        text: impl Into<String>,
        source: Option<impl Into<String>>,
        timestamp: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            role,
            content: vec![ContentPart::Text { text: text.into() }],
            name: source.map(|s| s.into()),
            timestamp,
        }
    }

    // 获取纯文本内容（如果只有文本块）
    pub fn get_text(&self) -> Option<String> {
        if self.content.len() == 1 {
            if let ContentPart::Text { text } = &self.content[0] {
                return Some(text.clone());
            }
        }
        None
    }

    // 是否包含多模态内容（未来判断用）
    pub fn is_multimodal(&self) -> bool {
        self.content.iter().any(|part| !matches!(part, ContentPart::Text { .. }))
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub message: LlmMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub choices: Vec<Choice>,
}

impl From<LlmMessage> for ChatMessage {
    fn from(msg: LlmMessage) -> Self {
        // 尝试解析 role
        let role = match msg.role.as_str() {
            "system" => Role::System,
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "tool" => Role::Tool,
            _ => Role::User, // 默认 fallback
        };

        ChatMessage {
            role,
            content: vec![ContentPart::Text { text: msg.content }],
            name: None,
            timestamp: None,
        }
    }
}