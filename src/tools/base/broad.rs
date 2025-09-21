use std::{sync::Arc};
use async_trait::async_trait;

#[derive(Debug,Clone,PartialEq, Eq, Hash)]
pub struct AgentId {
    pub agent_type: String,
    pub key: String,
}

impl AgentId {
    pub fn new (agent_type: String, key: String) -> Self {
        Self {
            agent_type,
            key,
        }
    }

    pub fn from_str(agent_id: &str) -> Result<Self, String> {
        let parts: Vec<&str> = agent_id.split(":").collect();
        if parts.len() != 2 {
            return Err(format!("Invalid agent ID: {}", agent_id));
        }
        Ok(Self {
            agent_type: parts[0].to_string(),
            key: parts[1].to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicId {
    pub topic_type: String,
    pub source: String,
}

impl TopicId {
    pub fn new(topic_type: &str, source: &str) -> Self {
        Self {
            topic_type: topic_type.to_string(),
            source: source.to_string(),
        }
    }

    pub fn from_str(topic_id: &str) -> Result<Self> {
        let parts: Vec<_> = topic_id.split('/').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid topic id format: {}", topic_id));
        }
        Ok(Self {
            topic_type: parts[0].to_string(),
            source: parts[1].to_string(),
        })
    }
}

impl std::fmt::Display for TopicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.topic_type, self.source)
    }
}

#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false))
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
pub trait AgentRuntime: Send + Sync {
    async fn publish_message(
        &self, 
        message: Box<dyn std::any::Any + Send + Sync>,
        topic_id: TopicId,
        sender: Option<AgentId>,
        cancellation_token: CancellationToken,
        message_id: Option<String>,
    ) -> Result<(),Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
pub trait BaseAgent: Send + Sync {
    fn id(&self) -> &AgentId;
    fn runtime(&self) -> &Arc<dyn AgentRuntime>;

    async fn publish_message(
        &self,
        message: Box<dyn std::any::Any + Send + Sync>,
        topic_id: TopicId,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(token) = &cancellation_token {
            if token.is_cancelled() {
                return Err("Operation cancelled".into());
            }
        }
        
        self.runtime()
            .publish_message(
                message,
                topic_id,
                Some(self.id().clone()),
                cancellation_token.unwrap_or_default(),
                None,
            )
            .await
    }
}