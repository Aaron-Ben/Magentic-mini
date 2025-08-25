use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: Uuid,
    pub content: String,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    UserInput,
    PlanRequest,
    StepExecution,
    StepResult,
    Error,
}

impl AgentMessage {
    pub fn new(content: String, message_type: MessageType) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            message_type,
        }
    }
}