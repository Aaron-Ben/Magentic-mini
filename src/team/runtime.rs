use async_channel;
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;
use dyn_clone::DynClone;
use anyhow::{anyhow, Result};
use tracing::{info, warn, error};

use crate::types::message::{AgentId, GroupChatEvent, MessageContext, TopicId};
// runtime 的作用是依据topic_id，将消息分发给对应的代理

#[async_trait]
pub trait Agent: Send + Sync + DynClone {
    fn id(&self) -> &AgentId;
    async fn on_message(&self, message: BaseChatMessage, ctx: MessageContext) -> Result<BaseChatMessage>;
}

dyn_clone::clone_trait_object!(Agent);


pub trait Subscription: Send + Sync {
    fn is_match(&self, topic_id: &TopicId) -> bool;
    fn map_to_agent(&self, topic_id: &TopicId) -> AgentId;
}

/// TypeSubscription: 基于类型的订阅
#[derive(Debug, Clone)]
pub struct TypeSubscription {
    pub topic_type: String,
    pub agent_type: String,
}

// 匹配规则
impl Subscription for TypeSubscription {
    fn is_match(&self, topic_id: &TopicId) -> bool {
        topic_id.topic_type == self.topic_type
    }
    
    fn map_to_agent(&self, topic_id: &TopicId) -> AgentId {
        AgentId::new(self.agent_type.clone(), topic_id.source.clone())
    }
}

/// SubscriptionManager: 订阅管理器
#[derive(Clone)]
pub struct SubscriptionManager {
    subscriptions: Arc<RwLock<Vec<Arc<dyn Subscription>>>>,
    // 缓存：TopicId -> AgentId 列表
    cache: Arc<RwLock<HashMap<TopicId, Vec<AgentId>>>>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(Vec::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn add_subscription(&self, subscription: Arc<dyn Subscription>) {
        let mut subs = self.subscriptions.write().await;
        subs.push(subscription);
        // 清空缓存（订阅变化时）
        self.cache.write().await.clear();
    }
    
    // 订阅查询
    pub async fn get_subscribed_recipients(&self, topic_id: &TopicId) -> Vec<AgentId> {
        // 1. 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(recipients) = cache.get(topic_id) {
                info!("Cache hit for topic: {:?}", topic_id);
                return recipients.clone();
            }
        }
        
        // 2. 计算订阅者
        let mut recipients = Vec::new();
        let subs = self.subscriptions.read().await;
        for subscription in subs.iter() {
            if subscription.is_match(topic_id) {
                recipients.push(subscription.map_to_agent(topic_id));
            }
        }
        
        // 3. 缓存结果
        {
            let mut cache = self.cache.write().await;
            cache.insert(topic_id.clone(), recipients.clone());
            info!("Cached {} recipients for topic: {:?}", recipients.len(), topic_id);
        }
        
        recipients
    }
}

// ============ 消息信封 ============

#[derive(Debug)]
pub enum MessageEnvelope {
    Publish {
        message: GroupChatEvent,
        topic_id: TopicId,
        sender: Option<AgentId>,
        message_id: String,
    },
    Send {
        message: GroupChatEvent,
        recipient: AgentId,
        sender: Option<AgentId>,
        message_id: String,
        response_tx: tokio::sync::oneshot::Sender<BaseChatMessage>,
    },
}

// ============ AgentRuntime ============

pub struct AgentRuntime {
    subscription_manager: SubscriptionManager,
    message_sender: async_channel::Sender<MessageEnvelope>,
    message_receiver: async_channel::Receiver<MessageEnvelope>,
    agents: Arc<RwLock<HashMap<AgentId, Arc<dyn Agent>>>>,
    output_sender: async_channel::Sender<BaseChatMessage>,
}

impl AgentRuntime {
    pub fn new() -> (Self, async_channel::Receiver<BaseChatMessage>) {
        let (msg_sender, msg_receiver) = async_channel::unbounded();
        let (output_sender, output_receiver) = async_channel::unbounded();
        
        (
            Self {
                subscription_manager: SubscriptionManager::new(),
                message_sender: msg_sender,
                message_receiver: msg_receiver,
                agents: Arc::new(RwLock::new(HashMap::new())),
                output_sender,
            },
            output_receiver,
        )
    }
    
    pub async fn add_subscription(&self, subscription: Arc<dyn Subscription>) {
        self.subscription_manager.add_subscription(subscription).await;
    }
    
    pub async fn register_agent(&self, agent: Arc<dyn Agent>) {
        let agent_id = agent.id().clone();
        let mut agents = self.agents.write().await;
        info!("Registering agent: {:?}", agent_id);
        agents.insert(agent_id, agent);
    }
    
    /// 发布消息（广播） 入口
    pub async fn publish_message(
        &self,
        message: GroupChatEvent,
        topic_id: TopicId,
        sender: Option<AgentId>,
    ) -> Result<()> {
        let message_id = Uuid::new_v4().to_string();
        info!("Publishing message {} to topic: {:?}", message_id, topic_id);
        
        let envelope = MessageEnvelope::Publish {
            message,
            topic_id,
            sender,
            message_id,
        };
        
        self.message_sender.send(envelope).await?;
        Ok(())
    }
    
    /// 发送消息（RPC）
    pub async fn send_message(
        &self,
        message: GroupChatEvent,
        recipient: AgentId,
        sender: Option<AgentId>,
    ) -> Result<BaseChatMessage> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let message_id = Uuid::new_v4().to_string();
        
        info!("Sending RPC message {} to agent: {:?}", message_id, recipient);
        
        let envelope = MessageEnvelope::Send {
            message,
            recipient,
            sender,
            message_id,
            response_tx: tx,
        };
        
        self.message_sender.send(envelope).await?;
        rx.await.map_err(|_| anyhow!("No response received"))
    }
    
    pub fn start(&self) -> JoinHandle<Result<()>> {
        let receiver = self.message_receiver.clone();
        let agents = self.agents.clone();
        let output_sender = self.output_sender.clone();
        let sub_mgr = self.subscription_manager.clone();
        
        tokio::spawn(async move {
            Self::process_messages(receiver, agents, output_sender, sub_mgr).await
        })
    }
    
    async fn process_messages(
        receiver: async_channel::Receiver<MessageEnvelope>,
        agents: Arc<RwLock<HashMap<AgentId, Arc<dyn Agent>>>>,
        output_sender: async_channel::Sender<BaseChatMessage>,
        sub_mgr: SubscriptionManager,
    ) -> Result<()> {
        while let Ok(envelope) = receiver.recv().await {
            match envelope {
                MessageEnvelope::Publish { message, topic_id, sender, message_id } => {
                    Self::handle_publish(
                        message,
                        topic_id,
                        sender,
                        message_id,
                        &agents,
                        &output_sender,
                        &sub_mgr,
                    ).await;
                }
                MessageEnvelope::Send { message, recipient, sender, message_id, response_tx } => {
                    Self::handle_send(
                        message,
                        recipient,
                        sender,
                        message_id,
                        response_tx,
                        &agents,
                    ).await;
                }
            }
        }
        info!("Message processing loop exited");
        Ok(())
    }
    
    /// 处理发布消息 分发消息
    async fn handle_publish(
        message: GroupChatEvent,
        topic_id: TopicId,
        sender: Option<AgentId>,
        message_id: String,
        agents: &Arc<RwLock<HashMap<AgentId, Arc<dyn Agent>>>>,
        output_sender: &async_channel::Sender<BaseChatMessage>,
        sub_mgr: &SubscriptionManager,
    ) {
        // 1. 查询订阅者（核心路由！）
        let recipients = sub_mgr.get_subscribed_recipients(&topic_id).await;
        info!("Found {} recipients for topic {:?}", recipients.len(), topic_id);
        
        // 2. 提取消息
        let messages = Self::extract_messages(&message);
        
        // 3. 并发处理所有订阅者
        let mut handles = Vec::new();
        
        for recipient_id in recipients {
            // ★ 跳过发送者（避免自己给自己发）
            if let Some(ref sender_id) = sender {
                if &recipient_id == sender_id {
                    info!("Skipping sender: {:?}", sender_id);
                    continue;
                }
            }
            
            for msg in &messages {
                let recipient_id = recipient_id.clone();
                let msg = msg.clone();
                let agents = agents.clone();
                let output_sender = output_sender.clone();
                let topic_id = topic_id.clone();
                let sender = sender.clone();
                let message_id = message_id.clone();
                
                let handle = tokio::spawn(async move {
                    let agent_opt = {
                        let agents_guard = agents.read().await;
                        agents_guard.get(&recipient_id).cloned()
                    };
                    
                    if let Some(agent) = agent_opt {
                        info!("Calling message handler for {:?}", recipient_id);
                        
                        let context = MessageContext {
                            sender,
                            topic_id: Some(topic_id),
                            is_rpc: false,
                            message_id,
                        };
                        
                        match agent.on_message(msg, context).await {
                            Ok(response) => {
                                if let Err(e) = output_sender.send(response).await {
                                    error!("Failed to send output: {:?}", e);
                                }
                            }
                            Err(e) => {
                                error!("Error processing message for {:?}: {:?}", recipient_id, e);
                            }
                        }
                    } else {
                        warn!("Agent not found: {:?}", recipient_id);
                    }
                });
                
                handles.push(handle);
            }
        }
        
        // 4. 等待所有任务完成
        for handle in handles {
            if let Err(e) = handle.await {
                error!("Task panic: {:?}", e);
            }
        }
    }
    
    /// 处理 RPC 消息
    async fn handle_send(
        message: GroupChatEvent,
        recipient: AgentId,
        sender: Option<AgentId>,
        message_id: String,
        response_tx: tokio::sync::oneshot::Sender<BaseChatMessage>,
        agents: &Arc<RwLock<HashMap<AgentId, Arc<dyn Agent>>>>,
    ) {
        let agent_opt = {
            let agents_guard = agents.read().await;
            agents_guard.get(&recipient).cloned()
        };
        
        if let Some(agent) = agent_opt {
            info!("Calling RPC handler for {:?}", recipient);
            
            let messages = Self::extract_messages(&message);
            if let Some(msg) = messages.first() {
                let context = MessageContext {
                    sender,
                    topic_id: None,
                    is_rpc: true,
                    message_id,
                };
                
                match agent.on_message(msg.clone(), context).await {
                    Ok(response) => {
                        let _ = response_tx.send(response);
                    }
                    Err(e) => {
                        error!("RPC error for {:?}: {:?}", recipient, e);
                    }
                }
            }
        } else {
            error!("RPC recipient not found: {:?}", recipient);
        }
    }
    
    fn extract_messages(event: &GroupChatEvent) -> Vec<BaseChatMessage> {
        match event {
            GroupChatEvent::Start(start) => start.messages.clone().unwrap_or_default(),
            GroupChatEvent::AgentResponse(resp) => vec![resp.agent_response.chat_message.clone()],
            GroupChatEvent::Message(msg) => vec![msg.message.clone()],
            GroupChatEvent::Termination(term) => vec![BaseChatMessage::Stop(term.message.clone())],
            _ => vec![],
        }
    }
}

impl Clone for AgentRuntime {
    fn clone(&self) -> Self {
        Self {
            subscription_manager: self.subscription_manager.clone(),
            message_sender: self.message_sender.clone(),
            message_receiver: self.message_receiver.clone(),
            agents: self.agents.clone(),
            output_sender: self.output_sender.clone(),
        }
    }
}
