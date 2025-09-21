use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

// === 基础类型定义 ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub id: String,
    pub name: String,
    pub arguments: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionExecutionResult {
    pub content: String,
    pub name: String,
    pub call_id: String,
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub code: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeResult {
    pub success: bool,
    pub output: String,
    pub code_blocks: Vec<CodeBlock>,
}

// === LLM消息类型===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMMessage {
    SystemMessage(SystemMessage),
    UserMessage(UserMessage),
    AssistantMessage(AssistantMessage),
    FunctionExecutionResultMessage(FunctionExecutionResultMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub content: String,
    _type: Option<String>,
}

impl SystemMessage {
    pub fn new(content: String) -> Self {
        SystemMessage {
            content,
            _type: Some("SystemMessage".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub content: UserContent,
    pub source: String,
    _type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserContent {
    Text(String),
    Multimodal(Vec<MultimodalItem>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultimodalItem {
    Text(String),
    Image(ImageData)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub data: String, // base64 encoded
    pub format: String,
}

impl UserMessage {
    pub fn new_text(content: String, source: String) -> Self {
        Self {
            content: UserContent::Text(content),
            source,
            _type: Some("UserMessage".to_string()),
        }
    }

    pub fn new_multi(content: Vec<MultimodalItem>, source: String) -> Self {
        Self {
            content: UserContent::Multimodal(content),
            source,
            _type: Some("UserMessage".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub content: AssistantContent,
    pub thought: Option<String>,
    pub source: String,
    _type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AssistantContent {
    Text(String),
    FunctionCalls(Vec<FunctionCall>),
}

impl AssistantMessage {
    pub fn new_text(content: String, source: String) -> Self {
        Self {
            content: AssistantContent::Text(content),
            thought: None,
            source,
            _type: Some("AssistantMessage".to_string()),
        }
    }

    pub fn new_function_calls(calls: Vec<FunctionCall>, source: String) -> Self {
        Self {
            content: AssistantContent::FunctionCalls(calls),
            thought: None,
            source,
            _type: Some("AssistantMessage".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionExecutionResultMessage {
    pub content: Vec<FunctionExecutionResult>,
    _type: Option<String>,
}

impl FunctionExecutionResultMessage {
    pub fn new(content: Vec<FunctionExecutionResult>) -> Self {
        Self {
            content,
            _type: Some("FunctionExecutionResultMessage".to_string()),
        }
    }
}

// === Agent消息基础trait ===

#[async_trait]
pub trait BaseMessage: Send + Sync {
    fn source(&self) -> &str;
    fn models_usage(&self) -> Option<&RequestUsage>;
    fn metadata(&self) -> &HashMap<String, String>;
    fn to_text(&self) -> String;
}

// === Agent聊天消息trait ===

#[async_trait]
pub trait ChatMessage: BaseMessage {
    /// 转换为纯文本，用于拼接prompt或日志
    fn to_model_text(&self) -> String;

    /// 转换为LLM可理解的消息
    fn to_model_message(&self) -> UserMessage;
}

// === Agent事件trait ===

#[async_trait]
pub trait AgentEvent: BaseMessage {
    // 事件通常只需要to_text，用于UI/日志显示
}

// === 具体聊天消息类型实现 ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMessage {
    pub content: String,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for TextMessage {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { self.content.clone() }
}

#[async_trait]
impl ChatMessage for TextMessage {
    fn to_model_text(&self) -> String { self.content.clone() }
    fn to_model_message(&self) -> UserMessage { 
        UserMessage::new_text(self.content.clone(), self.source.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalMessage {
    pub content: Vec<MultimodalItem>,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for MultiModalMessage {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { 
        self.content.iter()
            .map(|item| match item {
                MultimodalItem::Text(text) => text.clone(),
                MultimodalItem::Image(_) => "<image>".to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[async_trait]
impl ChatMessage for MultiModalMessage {
    fn to_model_text(&self) -> String {
        self.content.iter()
            .map(|item| match item {
                MultimodalItem::Text(text) => text.clone(),
                MultimodalItem::Image(_) => "[image]".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    fn to_model_message(&self) -> UserMessage {
        UserMessage::new_multi(self.content.clone(), self.source.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopMessage {
    pub content: String,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for StopMessage {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { self.content.clone() }
}

#[async_trait]
impl ChatMessage for StopMessage {
    fn to_model_text(&self) -> String { self.content.clone() }
    fn to_model_message(&self) -> UserMessage { 
        UserMessage::new_text(self.content.clone(), self.source.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffMessage {
    pub content: String,
    pub target: String,
    pub context: Vec<LLMMessage>,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for HandoffMessage {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { 
        format!("Handoff to {}: {}", self.target, self.content)
    }
}

#[async_trait]
impl ChatMessage for HandoffMessage {
    fn to_model_text(&self) -> String { self.content.clone() }
    fn to_model_message(&self) -> UserMessage { 
        UserMessage::new_text(self.content.clone(), self.source.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallSummaryMessage {
    pub content: String,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for ToolCallSummaryMessage {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { self.content.clone() }
}

#[async_trait]
impl ChatMessage for ToolCallSummaryMessage {
    fn to_model_text(&self) -> String { self.content.clone() }
    fn to_model_message(&self) -> UserMessage { 
        UserMessage::new_text(self.content.clone(), self.source.clone())
    }
}

// === 结构化消息（泛型）===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredMessage<T: Serialize + Clone> {
    pub content: T,
    pub format_string: Option<String>,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl<T: Serialize + Clone + Send + Sync> BaseMessage for StructuredMessage<T> {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { 
        if let Some(format) = &self.format_string {
            format.clone() // 实际实现中需要格式化
        } else {
            serde_json::to_string(&self.content).unwrap_or_default()
        }
    }
}

#[async_trait]
impl<T: Serialize + Clone + Send + Sync> ChatMessage for StructuredMessage<T> {
    fn to_model_text(&self) -> String { self.to_text() }
    fn to_model_message(&self) -> UserMessage { 
        UserMessage::new_text(self.to_text(), self.source.clone())
    }
}

// === Agent事件类型实现 ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequestEvent {
    pub content: Vec<FunctionCall>,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for ToolCallRequestEvent {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { 
        format!("Tool calls: {:?}", self.content)
    }
}

#[async_trait]
impl AgentEvent for ToolCallRequestEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGenerationEvent {
    pub retry_attempt: usize,
    pub content: String,
    pub code_blocks: Vec<CodeBlock>,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for CodeGenerationEvent {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { self.content.clone() }
}

#[async_trait]
impl AgentEvent for CodeGenerationEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionEvent {
    pub retry_attempt: usize,
    pub result: CodeResult,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for CodeExecutionEvent {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { self.result.output.clone() }
}

#[async_trait]
impl AgentEvent for CodeExecutionEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallExecutionEvent {
    pub content: Vec<FunctionExecutionResult>,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for ToolCallExecutionEvent {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { 
        format!("Tool results: {:?}", self.content)
    }
}

#[async_trait]
impl AgentEvent for ToolCallExecutionEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInputRequestedEvent {
    pub content: String,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for UserInputRequestedEvent {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { self.content.clone() }
}

#[async_trait]
impl AgentEvent for UserInputRequestedEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQueryEvent {
    pub content: String,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for MemoryQueryEvent {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { self.content.clone() }
}

#[async_trait]
impl AgentEvent for MemoryQueryEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelClientStreamingChunkEvent {
    pub content: String,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for ModelClientStreamingChunkEvent {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { self.content.clone() }
}

#[async_trait]
impl AgentEvent for ModelClientStreamingChunkEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtEvent {
    pub content: String,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for ThoughtEvent {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { self.content.clone() }
}

#[async_trait]
impl AgentEvent for ThoughtEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectSpeakerEvent {
    pub content: String,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for SelectSpeakerEvent {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { self.content.clone() }
}

#[async_trait]
impl AgentEvent for SelectSpeakerEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorEvent {
    pub content: String,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for SelectorEvent {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { self.content.clone() }
}

#[async_trait]
impl AgentEvent for SelectorEvent {}

// === 项目自定义类型 ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointEvent {
    pub state: String,
    pub content: String,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

impl CheckpointEvent {
    pub fn new(state: String, content: String, source: String) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("internal".to_string(), "yes".to_string());
        
        Self {
            state,
            content,
            source,
            models_usage: None,
            metadata,
        }
    }
}

#[async_trait]
impl BaseMessage for CheckpointEvent {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { "Checkpoint".to_string() }
}

#[async_trait]
impl AgentEvent for CheckpointEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMCallEventMessage {
    pub content: String,
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
impl BaseMessage for LLMCallEventMessage {
    fn source(&self) -> &str { &self.source }
    fn models_usage(&self) -> Option<&RequestUsage> { self.models_usage.as_ref() }
    fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    fn to_text(&self) -> String { self.content.clone() }
}

#[async_trait]
impl ChatMessage for LLMCallEventMessage {
    fn to_model_text(&self) -> String { self.content.clone() }
    fn to_model_message(&self) -> UserMessage { 
        UserMessage::new_text(self.content.clone(), self.source.clone())
    }
}

// === 统一的消息枚举类型 ===

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    // 聊天消息
    TextMessage(TextMessage),
    MultiModalMessage(MultiModalMessage),
    StopMessage(StopMessage),
    HandoffMessage(HandoffMessage),
    ToolCallSummaryMessage(ToolCallSummaryMessage),
    
    // 事件消息
    ToolCallRequestEvent(ToolCallRequestEvent),
    CodeGenerationEvent(CodeGenerationEvent),
    CodeExecutionEvent(CodeExecutionEvent),
    ToolCallExecutionEvent(ToolCallExecutionEvent),
    UserInputRequestedEvent(UserInputRequestedEvent),
    MemoryQueryEvent(MemoryQueryEvent),
    ModelClientStreamingChunkEvent(ModelClientStreamingChunkEvent),
    ThoughtEvent(ThoughtEvent),
    SelectSpeakerEvent(SelectSpeakerEvent),
    SelectorEvent(SelectorEvent),
    
    // 自定义消息
    CheckpointEvent(CheckpointEvent),
    LLMCallEventMessage(LLMCallEventMessage),
}

// 为Message实现统一的trait
#[async_trait]
impl BaseMessage for Message {
    fn source(&self) -> &str {
        match self {
            Message::TextMessage(msg) => msg.source(),
            Message::MultiModalMessage(msg) => msg.source(),
            Message::StopMessage(msg) => msg.source(),
            Message::HandoffMessage(msg) => msg.source(),
            Message::ToolCallSummaryMessage(msg) => msg.source(),
            Message::ToolCallRequestEvent(msg) => msg.source(),
            Message::CodeGenerationEvent(msg) => msg.source(),
            Message::CodeExecutionEvent(msg) => msg.source(),
            Message::ToolCallExecutionEvent(msg) => msg.source(),
            Message::UserInputRequestedEvent(msg) => msg.source(),
            Message::MemoryQueryEvent(msg) => msg.source(),
            Message::ModelClientStreamingChunkEvent(msg) => msg.source(),
            Message::ThoughtEvent(msg) => msg.source(),
            Message::SelectSpeakerEvent(msg) => msg.source(),
            Message::SelectorEvent(msg) => msg.source(),
            Message::CheckpointEvent(msg) => msg.source(),
            Message::LLMCallEventMessage(msg) => msg.source(),
        }
    }

    fn models_usage(&self) -> Option<&RequestUsage> {
        match self {
            Message::TextMessage(msg) => msg.models_usage(),
            Message::MultiModalMessage(msg) => msg.models_usage(),
            Message::StopMessage(msg) => msg.models_usage(),
            Message::HandoffMessage(msg) => msg.models_usage(),
            Message::ToolCallSummaryMessage(msg) => msg.models_usage(),
            Message::ToolCallRequestEvent(msg) => msg.models_usage(),
            Message::CodeGenerationEvent(msg) => msg.models_usage(),
            Message::CodeExecutionEvent(msg) => msg.models_usage(),
            Message::ToolCallExecutionEvent(msg) => msg.models_usage(),
            Message::UserInputRequestedEvent(msg) => msg.models_usage(),
            Message::MemoryQueryEvent(msg) => msg.models_usage(),
            Message::ModelClientStreamingChunkEvent(msg) => msg.models_usage(),
            Message::ThoughtEvent(msg) => msg.models_usage(),
            Message::SelectSpeakerEvent(msg) => msg.models_usage(),
            Message::SelectorEvent(msg) => msg.models_usage(),
            Message::CheckpointEvent(msg) => msg.models_usage(),
            Message::LLMCallEventMessage(msg) => msg.models_usage(),
        }
    }

    fn metadata(&self) -> &HashMap<String, String> {
        match self {
            Message::TextMessage(msg) => msg.metadata(),
            Message::MultiModalMessage(msg) => msg.metadata(),
            Message::StopMessage(msg) => msg.metadata(),
            Message::HandoffMessage(msg) => msg.metadata(),
            Message::ToolCallSummaryMessage(msg) => msg.metadata(),
            Message::ToolCallRequestEvent(msg) => msg.metadata(),
            Message::CodeGenerationEvent(msg) => msg.metadata(),
            Message::CodeExecutionEvent(msg) => msg.metadata(),
            Message::ToolCallExecutionEvent(msg) => msg.metadata(),
            Message::UserInputRequestedEvent(msg) => msg.metadata(),
            Message::MemoryQueryEvent(msg) => msg.metadata(),
            Message::ModelClientStreamingChunkEvent(msg) => msg.metadata(),
            Message::ThoughtEvent(msg) => msg.metadata(),
            Message::SelectSpeakerEvent(msg) => msg.metadata(),
            Message::SelectorEvent(msg) => msg.metadata(),
            Message::CheckpointEvent(msg) => msg.metadata(),
            Message::LLMCallEventMessage(msg) => msg.metadata(),
        }
    }

    fn to_text(&self) -> String {
        match self {
            Message::TextMessage(msg) => msg.to_text(),
            Message::MultiModalMessage(msg) => msg.to_text(),
            Message::StopMessage(msg) => msg.to_text(),
            Message::HandoffMessage(msg) => msg.to_text(),
            Message::ToolCallSummaryMessage(msg) => msg.to_text(),
            Message::ToolCallRequestEvent(msg) => msg.to_text(),
            Message::CodeGenerationEvent(msg) => msg.to_text(),
            Message::CodeExecutionEvent(msg) => msg.to_text(),
            Message::ToolCallExecutionEvent(msg) => msg.to_text(),
            Message::UserInputRequestedEvent(msg) => msg.to_text(),
            Message::MemoryQueryEvent(msg) => msg.to_text(),
            Message::ModelClientStreamingChunkEvent(msg) => msg.to_text(),
            Message::ThoughtEvent(msg) => msg.to_text(),
            Message::SelectSpeakerEvent(msg) => msg.to_text(),
            Message::SelectorEvent(msg) => msg.to_text(),
            Message::CheckpointEvent(msg) => msg.to_text(),
            Message::LLMCallEventMessage(msg) => msg.to_text(),
        }
    }
}