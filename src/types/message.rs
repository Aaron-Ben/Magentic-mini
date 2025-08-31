use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatMessage {
    TextMessage {
        content: String,
        source: String,
        timestamp: Option<String>,
    },
    MultiModalMessage {
        text_content: String,
        source: String,
        media_content: Option<String>,
        timestamp: Option<String>,
    },
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