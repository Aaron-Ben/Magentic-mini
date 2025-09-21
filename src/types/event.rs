use serde::{Deserialize, Serialize};
use crate::types::message::{Message,StopMessage};
use async_trait::async_trait;

// === Event Trait - 用于控制和协调 ===

#[async_trait]
pub trait Event: Send + Sync {
    // 事件通常不需要复杂的转换方法，主要用于传递控制信息
    fn event_type(&self) -> &str;
}

// === 异常处理类型 ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableException {
    pub error_type: String,
    pub error_message: String,
    pub traceback: Option<String>,
}

impl SerializableException {
    pub fn from_exception(error_type: String, error_message: String, traceback: Option<String>) -> Self {
        Self {
            error_type,
            error_message,
            traceback,
        }
    }
}

impl std::fmt::Display for SerializableException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(traceback) = &self.traceback {
            write!(f, "{}: {}\nTraceback:\n{}", self.error_type, self.error_message, traceback)
        } else {
            write!(f, "{}: {}", self.error_type, self.error_message)
        }
    }
}

// === Group Chat 事件类型 ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatStart {
    pub messages: Option<Vec<Message>>,
}

impl GroupChatStart {
    pub fn new(messages: Option<Vec<Message>>) -> Self {
        Self { messages }
    }
}

#[async_trait]
impl Event for GroupChatStart {
    fn event_type(&self) -> &str {
        "GroupChatStart"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatAgentResponse {
    pub agent_response: Response, // 需要定义Response类型
}

#[async_trait]
impl Event for GroupChatAgentResponse {
    fn event_type(&self) -> &str {
        "GroupChatAgentResponse"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatRequestPublish {
    // 空的请求体，实际内容通过其他方式传递
}

#[async_trait]
impl Event for GroupChatRequestPublish {
    fn event_type(&self) -> &str {
        "GroupChatRequestPublish"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatMessage {
    pub message: Message, // 包含实际的消息
}

#[async_trait]
impl Event for GroupChatMessage {
    fn event_type(&self) -> &str {
        "GroupChatMessage"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatTermination {
    pub message: StopMessage,
    pub error: Option<SerializableException>,
}

impl GroupChatTermination {
    pub fn new(message: StopMessage) -> Self {
        Self { message, error: None }
    }

    pub fn with_error(message: StopMessage, error: SerializableException) -> Self {
        Self { message, error: Some(error) }
    }
}

#[async_trait]
impl Event for GroupChatTermination {
    fn event_type(&self) -> &str {
        "GroupChatTermination"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatReset {
    // 重置请求，通常为空
}

#[async_trait]
impl Event for GroupChatReset {
    fn event_type(&self) -> &str {
        "GroupChatReset"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatPause {
    // 暂停请求，通常为空
}

#[async_trait]
impl Event for GroupChatPause {
    fn event_type(&self) -> &str {
        "GroupChatPause"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatResume {
    // 恢复请求，通常为空
}

#[async_trait]
impl Event for GroupChatResume {
    fn event_type(&self) -> &str {
        "GroupChatResume"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatError {
    pub error: SerializableException,
}

impl GroupChatError {
    pub fn new(error: SerializableException) -> Self {
        Self { error }
    }
}

#[async_trait]
impl Event for GroupChatError {
    fn event_type(&self) -> &str {
        "GroupChatError"
    }
}

// === 统一的Event枚举类型 ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatEvent {
    GroupChatStart(GroupChatStart),
    GroupChatAgentResponse(GroupChatAgentResponse),
    GroupChatRequestPublish(GroupChatRequestPublish),
    GroupChatMessage(GroupChatMessage),
    GroupChatTermination(GroupChatTermination),
    GroupChatReset(GroupChatReset),
    GroupChatPause(GroupChatPause),
    GroupChatResume(GroupChatResume),
    GroupChatError(GroupChatError),
}

// 为ChatEvent实现统一的Event trait
#[async_trait]
impl Event for ChatEvent {
    fn event_type(&self) -> &str {
        match self {
            ChatEvent::GroupChatStart(_) => "GroupChatStart",
            ChatEvent::GroupChatAgentResponse(_) => "GroupChatAgentResponse",
            ChatEvent::GroupChatRequestPublish(_) => "GroupChatRequestPublish",
            ChatEvent::GroupChatMessage(_) => "GroupChatMessage",
            ChatEvent::GroupChatTermination(_) => "GroupChatTermination",
            ChatEvent::GroupChatReset(_) => "GroupChatReset",
            ChatEvent::GroupChatPause(_) => "GroupChatPause",
            ChatEvent::GroupChatResume(_) => "GroupChatResume",
            ChatEvent::GroupChatError(_) => "GroupChatError",
        }
    }
}

// === 需要补充的Response类型（简化版）===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub inner: Option<Message>,
    pub termination: bool,
}

impl Response {
    pub fn new(message: Message) -> Self {
        Self {
            inner: Some(message),
            termination: false,
        }
    }

    pub fn termination(message: Message) -> Self {
        Self {
            inner: Some(message),
            termination: true,
        }
    }
}

// === 完整的类型层次结构 ===

/// 所有类型的大统一枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Communication {
    Message(Message),     // agent间的通信消息
    Event(ChatEvent),     // 控制和协调事件
}