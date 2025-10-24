use std::collections::HashMap;
use anyhow::{Result,anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub chat_history: Vec<ChatMessage>,
    pub msg_type: MessageType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MessageType {
    Notify,
    Execute,
}

#[derive(Debug, Clone, Serialize, Deserialize,PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChatMessage {
    Text {
        role: MessageRole,
        source: String,
        content: String,
        #[serde(default)]
        metadata: HashMap<String, String>,
    },
    MultiModal {
        role: MessageRole,
        source: String,
        content: Vec<MultiModalContent>,
        #[serde(default)]
        metadata: HashMap<String, String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum MultiModalContent {
    Text(String),
    Image(Vec<u8>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemMessage {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UserContent {
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "multi_modal")]
    MultiModal(Vec<MultiModalContent>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserMessage {
    pub content: UserContent,
    pub source: String,
    #[serde(rename = "type")]
    pub message_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AssistantContent {
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "function_calls")]
    FunctionCalls(Vec<FunctionCall>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssistantMessage {
    pub content: AssistantContent,
    pub source: Option<String>,
    #[serde(rename = "type")]
    pub message_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolMessage {
    pub content: String,
    pub name: String,
    pub call_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "role", content = "content")]
pub enum LLMMessage {
    #[serde(rename = "system")]
    System(SystemMessage),
    #[serde(rename = "user")]
    User(UserMessage),
    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),
    #[serde(rename = "tool")]
    Tool(ToolMessage),
}


impl SystemMessage {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

impl UserMessage {
    pub fn new(content: UserContent, source: String) -> Self {
        Self {
            content,
            source,
            message_type: "UserMessage".to_string(),
        }
    }
}

impl AssistantMessage {
    pub fn new(content: AssistantContent, source: Option<String>) -> Self {
        Self {
            content,
            source,
            message_type: "AssistantMessage".to_string(),
        }
    }
}

impl ChatMessage {
    pub fn new_text(role: MessageRole, source: String, content: String) -> Self {
        ChatMessage::Text {
            role,
            source,
            content,
            metadata: HashMap::new(),
        }
    }
    
    pub fn new_multimodal(role: MessageRole, source: String, content: Vec<MultiModalContent>) -> Self {
        ChatMessage::MultiModal {
            role,
            source,
            content,
            metadata: HashMap::new(),
        }
    }
}



pub fn chat_message_to_llm_message(msg: &ChatMessage) -> Result<LLMMessage> {
    match msg {
        ChatMessage::Text { role, source, content, metadata } => {
            match role {
                MessageRole::System => {
                    Ok(LLMMessage::System(SystemMessage {
                        content: content.clone(),
                    }))
                }
                MessageRole::User => {
                    Ok(LLMMessage::User(UserMessage {
                        content: UserContent::String(content.clone()),
                        source: source.clone(),
                        message_type: "UserMessage".to_string(),
                    }))
                }
                MessageRole::Assistant => {
                    Ok(LLMMessage::Assistant(AssistantMessage {
                        content: AssistantContent::String(content.clone()),
                        source: Some(source.clone()),
                        message_type: "AssistantMessage".to_string(),
                    }))
                }
                MessageRole::Tool => {
                    let name = metadata
                        .get("tool_name")
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    let call_id = metadata
                        .get("tool_call_id")
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    Ok(LLMMessage::Tool(ToolMessage {
                        content: content.clone(),
                        name,
                        call_id,
                    }))
                }
            }
        }
        ChatMessage::MultiModal { role, source, content, .. } => {
            if *role != MessageRole::User {
                return Err(anyhow!("Only user can send multimodal messages"));
            }
            Ok(LLMMessage::User(UserMessage {
                content: UserContent::MultiModal(content.clone()),
                source: source.clone(),
                message_type: "UserMessage".to_string(),
            }))
        }
    }
}

pub fn chat_history_to_llm_messages(history: &[ChatMessage]) -> Result<Vec<LLMMessage>> {
    history.iter().map(chat_message_to_llm_message).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_user() {
        let chat_msg = ChatMessage::new_text(
            MessageRole::User,
            "user".to_string(),
            "Hello, world!".to_string(),
        );
        let llm_msg = chat_message_to_llm_message(&chat_msg).unwrap();
        match llm_msg {
            LLMMessage::User(UserMessage {
                content: UserContent::String(content),
                source,
                message_type,
            }) => {
                assert_eq!(content, "Hello, world!");
                assert_eq!(source, "user");
                assert_eq!(message_type, "UserMessage");
            }
            _ => panic!("Expected User message"),
        }
    }

    #[test]
    fn test_text_assistant() {
        let chat_msg = ChatMessage::new_text(
            MessageRole::Assistant,
            "planner".to_string(),
            "I will help you.".to_string(),
        );
        let llm_msg = chat_message_to_llm_message(&chat_msg).unwrap();
        match llm_msg {
            LLMMessage::Assistant(AssistantMessage {
                content: AssistantContent::String(content),
                source,
                message_type,
            }) => {
                assert_eq!(content, "I will help you.");
                assert_eq!(source, Some("planner".to_string()));
                assert_eq!(message_type, "AssistantMessage");
            }
            _ => panic!("Expected Assistant message"),
        }
    }

    #[test]
    fn test_text_system() {
        let chat_msg = ChatMessage::new_text(
            MessageRole::System,
            "system".to_string(),
            "You are a helpful AI.".to_string(),
        );
        let llm_msg = chat_message_to_llm_message(&chat_msg).unwrap();
        match llm_msg {
            LLMMessage::System(SystemMessage { content }) => {
                assert_eq!(content, "You are a helpful AI.");
            }
            _ => panic!("Expected System message"),
        }
    }

    #[test]
    fn test_text_tool() {
        let mut metadata = HashMap::new();
        metadata.insert("tool_name".to_string(), "github_checker".to_string());
        metadata.insert("tool_call_id".to_string(), "call_123".to_string());

        let chat_msg = ChatMessage::Text {
            role: MessageRole::Tool,
            source: "tool_executor".to_string(),
            content: "7000 stars".to_string(),
            metadata,
        };

        let llm_msg = chat_message_to_llm_message(&chat_msg).unwrap();
        match llm_msg {
            LLMMessage::Tool(ToolMessage { content, name, call_id }) => {
                assert_eq!(content, "7000 stars");
                assert_eq!(name, "github_checker");
                assert_eq!(call_id, "call_123");
            }
            _ => panic!("Expected Tool message"),
        }
    }

    #[test]
    fn test_multimodal_user() {
        let content = vec![
            MultiModalContent::Text("What is this?".to_string()),
            MultiModalContent::Image(vec![0x89, b'P', b'N', b'G']), // fake PNG header
        ];
        let chat_msg = ChatMessage::new_multimodal(
            MessageRole::User,
            "user".to_string(),
            content.clone(),
        );

        let llm_msg = chat_message_to_llm_message(&chat_msg).unwrap();
        match llm_msg {
            LLMMessage::User(UserMessage {
                content: UserContent::MultiModal(parts),
                source,
                message_type,
            }) => {
                assert_eq!(parts, content);
                assert_eq!(source, "user");
                assert_eq!(message_type, "UserMessage");
            }
            _ => panic!("Expected multimodal User message"),
        }
    }

    #[test]
    fn test_multimodal_non_user_should_fail() {
        let chat_msg = ChatMessage::new_multimodal(
            MessageRole::Assistant,
            "agent".to_string(),
            vec![MultiModalContent::Text("I see an image".to_string())],
        );

        let result = chat_message_to_llm_message(&chat_msg);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Only user can send multimodal messages"
        );
    }

    #[test]
    fn test_chat_history_to_llm_messages() {
        let history = vec![
            ChatMessage::new_text(MessageRole::System, "sys".to_string(), "System prompt".to_string()),
            ChatMessage::new_text(MessageRole::User, "user".to_string(), "Hi".to_string()),
            ChatMessage::new_text(MessageRole::Assistant, "ai".to_string(), "Hello!".to_string()),
        ];

        let llm_msgs = chat_history_to_llm_messages(&history).unwrap();
        assert_eq!(llm_msgs.len(), 3);
        matches!(llm_msgs[0], LLMMessage::System(_));
        matches!(llm_msgs[1], LLMMessage::User(_));
        matches!(llm_msgs[2], LLMMessage::Assistant(_));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original = ChatMessage::new_text(
            MessageRole::User,
            "test".to_string(),
            "Serialize me".to_string(),
        );
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }
}