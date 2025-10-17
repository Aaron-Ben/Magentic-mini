use async_trait::async_trait;
use crate::orchestrator::message::{ChatMessage, Message};

#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;

    async fn on_message_stream(&mut self, message: Message) -> Result<ChatMessage>;
}