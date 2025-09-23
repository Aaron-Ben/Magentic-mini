use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestUsage {
    prompt_tokens: i32,
    completion_tokens: i32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContext {
    pub sender: Option<String>,
    pub topic_id: Option<String>,              // 消息的主题
    pub is_rpc: bool,
    pub cancellation_token: CancellationToken,  
    pub message_id: String,                     
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancellationToken {
    pub id: Uuid,
    pub cancelled: bool,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            cancelled: false,
        }
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

pub trait Message: Send + Sync {
    fn to_text(&self) -> String;

    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error>
    where Self: Serialize
    {
        serde_json::to_value(self)
    }

    fn message_type(&self) -> &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMMessage {
    System(SystemMessage),          // 系统消息
    User(UserMessage),              // 用户输入消息
    Assistant(AssistantMessage),    // 助手（ai）回复消息

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub content: Vec<MessageContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Image(Vec<u8>),  // 图片数据
}

/// 聊天消息基类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BaseChatMessage {
    Text(TextMessage),
    MultiModal(MultiModalMessage),
    Structured(StructuredMessage),  // 结构化消息
    Stop(StopMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMessage {
    pub content: String,
    pub source: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalMessage {
    pub content: Vec<MessageContent>,
    pub source: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredMessage {
    pub content: serde_json::Value,
    pub source: String,
    pub format_string: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopMessage {
    pub content: String,
    pub source: String,
}

#[async_trait]
impl Message for StopMessage {
    fn to_text(&self) -> String {
        self.content.clone()
    }

    fn message_type(&self) -> &'static str {
        "StopMessage"
    }
    
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BaseAgentEvent {
    SelectSpeaker(SelectSpeakerEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectSpeakerEvent {
    pub content: Vec<String>,  // 选择的发言者列表
    pub source: String,
}

// 团队控制消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupChatEvent {
    Start(GroupChatStart),                      // 群聊的话题（初始化）
    RequestPublish(GroupChatRequestPublish),    // 特定代理话题（分配任务）
    AgentResponse(GroupChatAgentResponse),      // 代理响应
    Message(GroupChatMessage),                  // 输出话题（给外部的观察者【teammanager】）
    Termination(GroupChatTermination),          // 输出的话题
    Error(GroupChatError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatStart {
    pub messages: Option<Vec<BaseChatMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatRequestPublish {
    // 空消息，主要起信号作用
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatAgentResponse {
    pub agent_response: Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub chat_message: BaseChatMessage,
    pub inner_messages: Option<Vec<BaseChatMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatMessage {
    pub message: BaseChatMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatTermination {
    pub message: StopMessage,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatError {
    pub error: String,
}

/// 统一的 Message 枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    LLM(LLMMessage),
    Chat(BaseChatMessage),
    AgentEvent(BaseAgentEvent),
    GroupEvent(GroupChatEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

impl Message for LLMMessage {
    fn to_text(&self) -> String {
        match self {
            LLMMessage::System(msg) => format!("System: {}", msg.content),
            LLMMessage::User(msg) => format!("User: {}", msg.content.iter()
                .map(|c| match c {
                    MessageContent::Text(t) => t.clone(),
                    MessageContent::Image(_) => "[Image]".to_string(),
                })
                .collect::<Vec<_>>()
                .join(" ")),
            LLMMessage::Assistant(msg) => format!("Assistant: {}", msg.content),
        }
    }
    
    fn message_type(&self) -> &'static str {
        match self {
            LLMMessage::System(_) => "SystemMessage",
            LLMMessage::User(_) => "UserMessage",
            LLMMessage::Assistant(_) => "AssistantMessage",
        }
    }
}

impl Message for BaseChatMessage {
    fn to_text(&self) -> String {
        match self {
            BaseChatMessage::Text(msg) => msg.content.clone(),
            BaseChatMessage::MultiModal(msg) => msg.content.iter()
                .map(|c| match c {
                    MessageContent::Text(t) => t.clone(),
                    MessageContent::Image(_) => "[Image]".to_string(),
                })
                .collect::<Vec<_>>()
                .join(" "),
            BaseChatMessage::Structured(msg) => msg.content.to_string(),
            BaseChatMessage::Stop(msg) => msg.content.clone(),
        }
    }
    
    fn message_type(&self) -> &'static str {
        match self {
            BaseChatMessage::Text(_) => "TextMessage",
            BaseChatMessage::MultiModal(_) => "MultiModalMessage",
            BaseChatMessage::Structured(_) => "StructuredMessage",
            BaseChatMessage::Stop(_) => "StopMessage",
        }
    }
}


impl Message for BaseAgentEvent {
    fn to_text(&self) -> String {
        match self {
            BaseAgentEvent::SelectSpeaker(event) => 
                format!("Selected speakers: {}", event.content.join(", ")),
        }
    }
    
    fn message_type(&self) -> &'static str {
        match self {
            BaseAgentEvent::SelectSpeaker(_) => "SelectSpeakerEvent",
        }
    }
}


impl Message for GroupChatEvent {
    fn to_text(&self) -> String {
        match self {
            GroupChatEvent::Start(_) => "Group chat started".to_string(),
            GroupChatEvent::RequestPublish(_) => "Request to publish".to_string(),
            GroupChatEvent::AgentResponse(resp) => resp.agent_response.chat_message.to_text(),
            GroupChatEvent::Message(msg) => msg.message.to_text(),
            GroupChatEvent::Termination(term) => term.message.to_text(),
            GroupChatEvent::Error(err) => format!("Error: {}", err.error),
        }
    }
    fn message_type(&self) -> &'static str {
        match self {
            GroupChatEvent::Start(_) => "GroupChatStart",
            GroupChatEvent::RequestPublish(_) => "GroupChatRequestPublish",
            GroupChatEvent::AgentResponse(_) => "GroupChatAgentResponse",
            GroupChatEvent::Message(_) => "GroupChatMessage",
            GroupChatEvent::Termination(_) => "GroupChatTermination",
            GroupChatEvent::Error(_) => "GroupChatError",
        }
    }
}


#[cfg(test)] 
mod test {
    use super::*;
    #[tokio::test]
    async fn test_message() {
        let text_msg = BaseChatMessage::Text(TextMessage {
            content: "你好，帮我搜索天气".to_string(),
            source: "user".to_string(),
            metadata: HashMap::new(),
        });
        
        // 创建团队控制消息
        let request_msg = GroupChatEvent::RequestPublish(GroupChatRequestPublish {});
        
        // 创建消息上下文
        let context = MessageContext {
            sender: Some(String::from("orchestrator")),
            topic_id: Some(String::from("web_surfer")),
            is_rpc: false,
            cancellation_token: CancellationToken::new(),
            message_id: Uuid::new_v4().to_string(),
        };
        
        println!("Text message: {}", text_msg.to_text());
        println!("Request message type: {}", request_msg.message_type());
        println!("Context sender: {:?}", context.sender);
    }
}