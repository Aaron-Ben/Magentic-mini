use chrono::Utc;
use std::collections::HashMap;
use super::types::*;

/// 用户消息
pub fn create_user_message(content: String, source: String) -> ChatMessage {
    ChatMessage::Text(TextMessage {
        base: BaseMessage {
            source: Some(source),
            metadata: HashMap::new(),
            timestamp: Some(Utc::now()),
        },
        content,
    })
}

/// 系统消息
pub fn create_system_message(content: String) -> ChatMessage {
    ChatMessage::Text(TextMessage {
        base: BaseMessage {
            source: Some("system".to_string()),
            metadata: HashMap::new(),
            timestamp: Some(Utc::now()),
        },
        content,
    })
}

/// 编排器消息
pub fn create_orchestrator_message(content: String) -> ChatMessage {
    let mut metadata = HashMap::new();
    metadata.insert("internal".to_string(), "yes".to_string());
    
    ChatMessage::Text(TextMessage {
        base: BaseMessage {
            source: Some("orchestrator".to_string()),
            metadata,
            timestamp: Some(Utc::now()),
        },
        content,
    })
}

/// 创建代理指令消息
pub fn create_agent_instruction_message(
    step_index: usize,
    step_title: String,
    step_details: String,
    agent_name: String,
    instruction: String,
) -> ChatMessage {
    let mut metadata = HashMap::new();
    metadata.insert("internal".to_string(), "yes".to_string());
    metadata.insert("step_index".to_string(), step_index.to_string());
    
    let content = format!(
        "Step {}: {}\n\n{}\n\nInstruction for {}: {}",
        step_index, step_title, step_details, agent_name, instruction
    );
    
    ChatMessage::Text(TextMessage {
        base: BaseMessage {
            source: Some(agent_name),
            metadata,
            timestamp: Some(Utc::now()),
        },
        content,
    })
}