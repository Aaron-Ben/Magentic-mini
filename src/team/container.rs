use crate::{team::routed::RoutedAgent, types::message::{BaseChatMessage, GroupChatAgentResponse, GroupChatRequestPublish, GroupChatStart, MessageContext}};
use std::error::Error;
use std::sync::Arc;

pub struct  ChatAgentContainer {
    // routed_agent: RoutedAgent,

    parent_topic_type: String,
    output_topic_type: String,
    message_buffer: Vec<BaseChatMessage>,
}

impl ChatAgentContainer {
    pub fn new(
        // agent: Arc<dyn ChatAgent>,
        parent_topic_type: String, 
        output_topic_type: String
    ) -> Self {
        Self {
            // agent
            parent_topic_type,
            output_topic_type,
            message_buffer: Vec::new(),
        }
    }

    async fn handle_start(
        &mut self, 
        message: GroupChatStart, 
        _ctx: MessageContext
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(messages) = message.messages {
            for msg in messages {
                self.buffer_message(msg)?;
            }
        }
        Ok(())
    }
    

    pub async fn handle_agent_response(
        &mut self, 
        message: GroupChatAgentResponse, 
        _ctx: MessageContext
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.buffer_message(message.agent_response.chat_message)?;
        Ok(())
    }


    pub async fn handle_request(
        &mut self, 
        _message: GroupChatRequestPublish, 
        ctx: MessageContext
    ) -> Result<(), Box<dyn Error + Send + Sync>> {

        let mut response: Option<Response> = None;

        let mut stream = self
            .agent
            .on_messages_stream(&self.message_buffer, ctx.cancellation_token)
            .await?;

        while let Some(msg_result) = stream.next().await {
            let msg = msg_result?;

            
        }

        self.message_buffer.clear();

        self.publish_message(response, ctx).await?;

        Ok(())
    }


    fn buffer_message(&mut self, message: BaseChatMessage) -> Result<(), Box<dyn Error + Send + Sync>> {
        // 验证消息类型
        self.message_buffer.push(message);
        Ok(())
    }
}


pub fn create_chat_agent_container(
    parent_topic_type: String,
    output_topic_type: String,
    agent: Arc<dyn ChatAgent>,
    description: String,
) -> RoutedAgent<ChatAgentContainer> {
    let container = ChatAgentContainer::new(parent_topic_type, output_topic_type, agent);
    let mut routed = RoutedAgent::new(container, description);

    // 注册事件处理器（模拟 @event）
    routed.register_event(|c, m, ctx| async move { c.handle_start(m, ctx).await });
    routed.register_event(|c, m, ctx| async move { c.handle_agent_response(m, ctx).await });
    routed.register_event(|c, m, ctx| async move { c.handle_request(m, ctx).await });

    // 注册 RPC 处理器（模拟 @rpc）
    // routed.register_rpc(|c, m, ctx| async move { c.handle_reset(m, ctx).await });


    routed
}