use std::{collections::HashMap, str};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use crate::types::plan::Plan;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub chat_history: Vec<ChatMessage>,
    pub msg_type: MessageType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ChatMessage {
    #[serde(rename = "TextMessage")]
    Text(TextMessage),

    #[serde(rename = "MultiModalMessage")]
    MultiModal(MultiModalMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MessageType {
    Notify,
    Execute,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HumanInputFormat {
    pub content: String,
    pub accepted: bool,
    pub plan: Option<Plan>,
}

impl HumanInputFormat {
    pub fn from_str(input_str: &str) -> Self {
        match serde_json::from_str::<Value>(input_str) {
            Ok(Value::Object(data)) => {
                Self {
                    content: data.get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),

                    accepted: data.get("accepted")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),

                    plan: data.get("plan")
                        .cloned()
                        .and_then(Plan::from_list_of_dicts_or_str),
                }
            }
            _ => Self { content: input_str.to_string(), accepted: false, plan: None }
        }
    }
}

impl fmt::Display for HumanInputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HumanInputFormat {{ content: '{}', accepted: {}, plan: {:?} }}", 
               self.content, self.accepted, self.plan)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BaseChatMessage {
    pub source: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BaseAgentEvent {
    pub source: String,
    pub metadata: HashMap<String, String>,
}

pub enum HistoryMessage {
    BaseAgentEvent(BaseAgentEvent),
    BaseChatMessage(BaseChatMessage),
}

#[derive(Debug,Serialize, Deserialize, Clone)]
pub struct BaseTextChatMessage {
    #[serde(flatten)]
    pub base: BaseChatMessage,
    pub content: String,
}

#[derive(Debug,Serialize, Deserialize, Clone)]
pub struct StopMessage {
    #[serde(flatten)]
    pub base: BaseTextChatMessage,
    #[serde(rename = "type")]
    pub message_type: String,
}

impl StopMessage {
    pub fn new(base: BaseTextChatMessage) -> Self {
        Self { base, message_type: "StopMessage".to_string() }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextMessage {
    #[serde(flatten)]
    pub base: BaseTextChatMessage,
    #[serde(rename = "type")]
    pub message_type: String,
}

impl TextMessage {
    pub fn new(content: String, source: String) -> Self {
        Self { 
            base: BaseTextChatMessage {
                base: BaseChatMessage {
                    source: source,
                    metadata: HashMap::new(),
                },
                content: content,
            },
            message_type: "TextMessage".to_string(),
        }
    }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MultiModalContent {
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "image")]
    Image(Vec<u8>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MultiModalMessage {
    #[serde(flatten)]
    pub base: BaseChatMessage,
    pub content: Vec<MultiModalContent>,
    #[serde(rename = "type")]
    pub message_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemMessage {
    pub content: String,
    #[serde(rename = "type")]
    pub message_type: String,
}

impl SystemMessage {
    pub fn new(content: String) -> Self {
        Self { content, message_type: "SystemMessage".to_string() }
    }
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

impl UserMessage {
    pub fn new (content: UserContent, source: String) -> Self {
        Self { content, source, message_type: "UserMessage".to_string() }
    }
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
    pub thought: Option<String>,
    pub source: Option<String>,
    #[serde(rename = "type")]
    pub message_type: String,
}

impl AssistantMessage {
    pub fn new(content: AssistantContent, thought: Option<String>, source: Option<String>) -> Self {
        Self { content, thought, source, message_type: "AssistantMessage".to_string() }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionExecutionResult {
    pub content: String,
    pub name: String,
    pub call_id : String,
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionExecutionResultMessage {
    pub content: Vec<FunctionExecutionResult>,
    #[serde(rename = "type")]
    pub message_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LLMMessage {
    SystemMessage(SystemMessage),
    UserMessage(UserMessage),
    AssistantMessage(AssistantMessage),
    FunctionExecutionResultMessage(FunctionExecutionResultMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCallRequestEvent {
    #[serde(flatten)]
    pub base: BaseAgentEvent,
    pub content: Vec<FunctionCall>,
    #[serde(rename = "type")]
    pub message_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCallExecutionEvent {
    #[serde(flatten)]
    pub base: BaseAgentEvent,
    pub content: Vec<FunctionExecutionResult>,
    #[serde(rename = "type")]
    pub message_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HandofMessage {
    #[serde(flatten)]
    pub base: BaseTextChatMessage,
    pub target: String,
    pub content: Vec<LLMMessage>,
    #[serde(rename = "type")]
    pub message_type: String,
}