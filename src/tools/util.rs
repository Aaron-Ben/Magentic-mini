use crate::orchestrator::message::{AssistantMessage, BaseTextChatMessage, HandofMessage, HumanInputFormat, LLMMessage, MultiModalContent, StopMessage, TextMessage, ToolCallExecutionEvent, ToolCallRequestEvent, UserContent, UserMessage};

#[derive(Debug, Clone)]
pub enum AgentMessage {
    ToolCallRequest(ToolCallRequestEvent),
    ToolCallExecution(ToolCallExecutionEvent),
    Stop(StopMessage),
    Handoff(HandofMessage),
    Text(TextMessage),
    MultiModal(MultiModalContent),
    Other(BaseTextChatMessage), // for unknown but text-based
}

pub fn thread_to_context(
    messages: Vec<AgentMessage>,
    agent_name: &str,
) -> Vec<LLMMessage> {
    let mut context = Vec::new();

    for m in messages {
        match m {
            AgentMessage::ToolCallRequest(_) | AgentMessage::ToolCallExecution(_) => {
                // Ignore
                continue;
            }

            AgentMessage::Stop(stop_msg) => {
                context.push(LLMMessage::UserMessage(UserMessage {
                    content: UserContent::String(stop_msg.base.content.clone()),
                    source: stop_msg.base.base.source.clone(),
                    message_type: "UserMessage".to_string(),
                }));
            }

            AgentMessage::Handoff(handoff_msg) => {
                context.push(LLMMessage::UserMessage(UserMessage {
                    content: UserContent::String(handoff_msg.base.content.clone()),
                    source: handoff_msg.base.base.source.clone(),
                    message_type: "UserMessage".to_string(),
                }));
            }

            AgentMessage::Text(text_msg) => {
                let source = &text_msg.base.base.source;
                if source == agent_name {
                    context.push(LLMMessage::AssistantMessage(AssistantMessage {
                        content: AssistantMessage::String(text_msg.base.content.clone()),
                        thought: None,
                        source: Some(source.clone()),
                        message_type: "AssistantMessage".to_string(),
                    }));
                } else if source == "user_proxy" || source == "user" {
                    let human_input = HumanInputFormat::from_str(&text_msg.base.content);
                    let mut content = human_input.content;
                    if let Some(plan) = human_input.plan {
                        content.push_str("\n\nI created the following plan: ");
                        content.push_str(&serde_json::to_string(&plan).unwrap_or_default());
                    }
                    context.push(LLMMessage::UserMessage(UserMessage {
                        content: UserContent::String(content),
                        source: source.clone(),
                        message_type: "UserMessage".to_string(),
                    }));
                } else {
                    context.push(LLMMessage::UserMessage(UserMessage {
                        content: UserContent::String(text_msg.base.content.clone()),
                        source: source.clone(),
                        message_type: "UserMessage".to_string(),
                    }));
                }
            }

            AgentMessage::MultiModal(multi_msg) => {
                let source = &multi_msg.base.source;
                if source == "user_proxy" || source == "user" {
                    let mut content_list = Vec::new();
                    for item in &multi_msg.content {
                        match item {
                            MultiModalContent::String(s) => {
                                let human_input = HumanInputFormat::from_str(s);
                                let mut content_str = human_input.content;
                                if let Some(plan) = human_input.plan {
                                    content_str.push_str("\n\nI created the following plan: ");
                                    content_str.push_str(&serde_json::to_string(&plan).unwrap_or_default());
                                }
                                content_list.push(UserContentItem::String(content_str));
                            }
                            MultiModalContent::Image(bytes) => {
                                content_list.push(UserContentItem::Image(bytes.clone()));
                            }
                        }
                    }
                    context.push(LLMMessage::UserMessage(UserMessage {
                        content: UserContent::List(content_list),
                        source: source.clone(),
                        message_type: "UserMessage".to_string(),
                    }));
                } else {
                    let content_list: Vec<UserContentItem> = multi_msg
                        .content
                        .into_iter()
                        .map(|item| match item {
                            MultiModalContent::String(s) => UserContentItem::String(s),
                            MultiModalContent::Image(b) => UserContentItem::Image(b),
                        })
                        .collect();
                    context.push(LLMMessage::UserMessage(UserMessage {
                        content: UserContent::List(content_list),
                        source: source.clone(),
                        message_type: "UserMessage".to_string(),
                    }));
                }
            }

            AgentMessage::Other(other_msg) => {
                context.push(LLMMessage::UserMessage(UserMessage {
                    content: UserContent::String(other_msg.content.clone()),
                    source: other_msg.base.source.clone(),
                    message_type: "UserMessage".to_string(),
                }));
            }
        }
    }

    context
}