use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct AgentId {
    pub agent_type: String,
    pub key: String,
}

impl AgentId {
    pub fn new(agent_type: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            agent_type: agent_type.into(),
            key: key.into(),
        }
    }
}

/// TopicId: 主题标识（type + source）
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TopicId {
    pub topic_type: String,
    pub source: String,
}

impl TopicId {
    pub fn new(topic_type: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            topic_type: topic_type.into(),
            source: source.into(),
        }
    }
}


#[derive(Debug, Clone)]
pub struct MessageContext {
    pub sender: Option<AgentId>,
    pub topic_id: Option<TopicId>,             
    pub is_rpc: bool,
    pub message_id: String,                     
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
pub struct StructuredMessage {
    pub content: serde_json::Value,
    pub source: String,
    pub format_string: Option<String>,
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
    // RequestPublish(GroupChatRequestPublish),    // 特定代理话题（分配任务）
    AgentResponse(GroupChatAgentResponse),      // 代理响应
    Message(GroupChatMessage),                  // 输出话题（给外部的观察者【teammanager】）
    Termination(GroupChatTermination),          // 输出的话题
    Error(GroupChatError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatStart {
    // pub messages: Option<Vec<BaseChatMessage>>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatAgentResponse {
    // pub agent_response: Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatMessage {
    // pub message: BaseChatMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatTermination {
    // pub message: StopMessage,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatError {
    pub error: String,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}
