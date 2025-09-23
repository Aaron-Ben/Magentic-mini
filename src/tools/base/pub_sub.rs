use async_channel;
use async_trait::async_trait;
use std::{collections::HashMap, sync::{Arc, Mutex}};
use tokio::task::JoinHandle;
use uuid::Uuid;
use dyn_clone::DynClone;

use crate::types::message::{BaseChatMessage, CancellationToken, GroupChatEvent, Message, MessageContext};

#[async_trait]
pub trait Agent: Send + Sync + DynClone {
    async fn on_message(&self, message: BaseChatMessage, ctx: MessageContext);
}

dyn_clone::clone_trait_object!(Agent);

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone)]
pub struct Subscription {
    pub topic_type: String,     // 自己订阅其他代理的类型
    pub agent_type: String,     // 代理自身的类型
}

#[derive(Debug, Clone)]
pub struct SubscriptionManager {
    subscriptions: Vec<Subscription>,
    subscribed_recipients: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
            subscribed_recipients: Arc::new(Mutex::new(HashMap::new())),
        }
    } 

    pub async fn add_subscription(&mut self, subscription: Subscription) {
        self.subscriptions.push(subscription);
    }

    pub async fn get_subscribed_recipients(&self, topic: &String) -> Vec<String> {
        {
            let cache = self.subscribed_recipients.lock().unwrap();
            if let Some(recipients) = cache.get(topic) {
                return recipients.clone();
            }
        }   // 释放锁
        
        let recipients = self.calculate_recipients(topic);
        {
            let mut cache = self.subscribed_recipients.lock().unwrap();
            cache.insert(topic.clone(), recipients.clone());
        }
        recipients
    }

    fn calculate_recipients(&self, topic: &String) -> Vec<String> {
        let mut recipients = Vec::new();
        for subscription in &self.subscriptions {
            if subscription.topic_type == *topic {
                recipients.push(subscription.agent_type.clone());
            }
        }
        recipients
    }
}

pub struct AgentRuntime {
    subscription_manager: SubscriptionManager,
    message_sender: async_channel::Sender<MessageEnvelope>,
    message_receiver: async_channel::Receiver<MessageEnvelope>,
    agents: Arc<Mutex<HashMap<String,Box<dyn Agent>>>>,
}

impl AgentRuntime {
    pub fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self {
            subscription_manager: SubscriptionManager::new(),
            message_sender: sender,
            message_receiver: receiver,
            agents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_subscription(&mut self, subscription: Subscription) {
        self.subscription_manager.add_subscription(subscription).await;
    }

    pub fn register_agent(&self, agent_id: String, agent: Box<dyn Agent>) {
        let mut agents = self.agents.lock().unwrap();
        agents.insert(agent_id, agent);
    }

    pub async fn publish_message(
        &self,
        message: GroupChatEvent,
        topic_id: String,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let envelope = MessageEnvelope {
            message,
            topic_id,
            cancellation_token,
        };
        self.message_sender.send(envelope).await?;
        Ok(())
    }

    pub fn start(&self) -> JoinHandle<Result<(), Error>> {
        let this = self.clone();
        tokio::spawn(async move {
            this.process_messages().await
        })
    }

    async fn process_messages(&self) -> Result<(), Error> {
        while let Ok(envelope) = self.message_receiver.recv().await {
            println!("Received envelope: {:?}", envelope);
            let messages: Vec<BaseChatMessage> = match &envelope.message {
                GroupChatEvent::Start(start) => start.messages.clone().unwrap_or_default(),
                GroupChatEvent::AgentResponse(resp) => vec![resp.agent_response.chat_message.clone()],
                GroupChatEvent::Message(msg) => vec![msg.message.clone()],
                GroupChatEvent::Termination(term) => vec![BaseChatMessage::Stop(term.message.clone())],
                GroupChatEvent::RequestPublish(_) | GroupChatEvent::Error(_) => continue,
            };

            let recipients = self
                .subscription_manager
                .get_subscribed_recipients(&envelope.topic_id)
                .await;
            println!("Recipients for topic {:?}: {:?}", envelope.topic_id, recipients);

            let mut handles = Vec::new();
            for message in messages {
                for agent_id in recipients.iter() {
                    println!("Dispatching to agent: {:?}", agent_id);
                    let agent_id = agent_id.clone();
                    let message = message.clone();
                    let envelope = envelope.clone();
                    let agents = self.agents.clone();

                    let handle = tokio::spawn(async move {
                        // 先获取 agent 的 Arc，然后再解锁
                        let agent_opt = {
                            let agents_guard = agents.lock().unwrap();
                            agents_guard.get(&agent_id).cloned()
                        };
                        if let Some(agent) = agent_opt {
                            let context = MessageContext {
                                sender: None, // 忽略发送者
                                topic_id: Some(envelope.topic_id.clone()),
                                is_rpc: false,
                                cancellation_token: envelope.cancellation_token.clone().unwrap(),
                                message_id: Uuid::new_v4().to_string(),
                            };
                            agent.on_message(message, context).await;
                            println!("Agent {:?} finished processing", agent_id);
                        } else {
                            println!("Agent {:?} not found", agent_id);
                        }
                    });
                    handles.push(handle);
                }
            }
            for handle in handles {
                handle.await.map_err(|e| {
                    eprintln!("分发任务 panic: {:?}", e);
                    "分发任务 panic" as &str
                })?;
            }
        }
        println!("Message processing loop exited");
        Ok(())
    }
}

impl Clone for AgentRuntime {
    fn clone(&self) -> Self {
        Self {
            subscription_manager: self.subscription_manager.clone(),
            message_sender: self.message_sender.clone(),
            message_receiver: self.message_receiver.clone(),
            agents: self.agents.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageEnvelope {
    pub message: GroupChatEvent,
    pub topic_id: String,
    pub cancellation_token: Option<CancellationToken>,
}

#[derive(Clone)]
pub struct WebAgent {
    runtime: AgentRuntime,
    id: String,
}


impl WebAgent {
    pub fn new(runtime: AgentRuntime, id: String) -> Self {
        Self { runtime, id }
    }
}

#[async_trait]
impl Agent for WebAgent {
    async fn on_message(&self, message: BaseChatMessage, ctx: MessageContext) {
        println!("Agent {:?} received message: {:?}", self.id, message);
        let msg_text = message.to_text();
        println!("msg_text: {:?}", msg_text);
        let topic = ctx.topic_id.as_ref().map(|t| t.as_str()).unwrap_or("unknown");

        if self.id == "agent2" && msg_text == "Hello, world" {
            println!("[{}] Agent2 成功接收消息: {:?}", topic, msg_text);
            println!("测试完毕");
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        println!("消息处理完成");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::message::{BaseChatMessage, CancellationToken, GroupChatEvent, GroupChatMessage, Message, TextMessage};
    use std::collections::HashMap;
    use tokio::time::Duration;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_two_agents_message_exchange() {
        let mut runtime = AgentRuntime::new();

        let agent1_id = String::from("agent1");
        let agent1 = WebAgent::new(runtime.clone(), agent1_id.clone());
        runtime.register_agent(agent1_id.clone(), Box::new(agent1));

        let agent2_id = String::from("agent2");
        let agent2 = WebAgent::new(runtime.clone(), agent2_id.clone());
        runtime.register_agent(agent2_id.clone(), Box::new(agent2));

        runtime
            .add_subscription(Subscription {
                topic_type: String::from("agent1"),
                agent_type: String::from("agent2"),
            })
            .await;


        println!("Agents subscribed successfully");

        let text_message = BaseChatMessage::Text(TextMessage {
            content: "Hello, world".to_string(),
            source: "agent1".to_string(),
            metadata: HashMap::new(),
        });

        let group_chat_event = GroupChatEvent::Message(GroupChatMessage {
            message: text_message.clone(),
        });

        let topic_id = String::from("agent1");
        let cancellation_token = CancellationToken::new();

        let runtime_handle = runtime.start();

        runtime
            .publish_message(group_chat_event.clone(), topic_id.clone(), Some(cancellation_token))
            .await
            .expect("Failed to publish message");

        println!("send message type is {:?}", group_chat_event.message_type());
        println!("Message published from agent1");

        tokio::time::sleep(Duration::from_millis(500)).await;

        runtime.message_sender.close();
        runtime_handle
            .await
            .expect("Runtime failed")
            .expect("Runtime error");

        println!("Test completed");
    }
}