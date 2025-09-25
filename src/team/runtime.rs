use async_channel;
use async_trait::async_trait;
use std::{collections::HashMap, sync::{Arc, Mutex}};
use tokio::task::JoinHandle;
use uuid::Uuid;
use dyn_clone::DynClone;
use anyhow::anyhow;

use crate::types::message::{BaseChatMessage, CancellationToken, GroupChatEvent, Message, MessageContext};

#[async_trait]
pub trait Agent: Send + Sync + DynClone {
    async fn on_message(&self, message: BaseChatMessage, ctx: MessageContext) -> BaseChatMessage;
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
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
        }
    } 

    pub async fn add_subscription(&mut self, subscription: Subscription) {
        self.subscriptions.push(subscription);
    }

    pub async fn get_subscribed_recipients(&self, topic: &str) -> Vec<String> {
        self.subscriptions
            .iter()
            .filter(|s|s.topic_type == topic)
            .map(|s|s.agent_type.clone())
            .collect()
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
// 用于send_message的响应通道
use tokio::sync::oneshot;

pub struct AgentRuntime {
    subscription_manager: SubscriptionManager,
    message_sender: async_channel::Sender<MessageEnvelope>,
    message_receiver: async_channel::Receiver<MessageEnvelope>,
    agents: Arc<Mutex<HashMap<String,Box<dyn Agent>>>>,
    output_sender: async_channel::Sender<BaseChatMessage>,
}

impl AgentRuntime {
    pub fn new() -> (Self, async_channel::Receiver<MessageEnvelope>, async_channel::Receiver<BaseChatMessage>) {
        let (msg_sender, msg_receiver) = async_channel::unbounded();
        let (output_sender, output_receiver) = async_channel::unbounded();
        (
            Self {
                subscription_manager: SubscriptionManager::new(),
                message_sender: msg_sender,
                message_receiver: msg_receiver.clone(),
                agents: Arc::new(Mutex::new(HashMap::new())),
                output_sender,
            },
            msg_receiver.clone(),
            output_receiver,
        )
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
            response_tx: None,
        };
        self.message_sender.send(envelope).await?;
        Ok(())
    }

    pub async fn send_message(
        &self,
        message: GroupChatEvent,
        topic_id: String,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<BaseChatMessage, Error> {
        let (tx, rx) = oneshot::channel();
        let envelope = MessageEnvelope {
            message,
            topic_id,
            cancellation_token,
            response_tx: Some(tx),
        };
        self.message_sender.send(envelope).await?;
        
        rx.await.map_err(|_| anyhow!("No response received".to_string()).into())

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
    
            // 提取消息内容
            let messages: Vec<BaseChatMessage> = match &envelope.message {
                GroupChatEvent::Start(start) => start.messages.clone().unwrap_or_default(),
                GroupChatEvent::AgentResponse(resp) => vec![resp.agent_response.chat_message.clone()],
                GroupChatEvent::Message(msg) => vec![msg.message.clone()],
                GroupChatEvent::Termination(term) => vec![BaseChatMessage::Stop(term.message.clone())],
                GroupChatEvent::RequestPublish(_) | GroupChatEvent::Error(_) => continue,
            };
    
            if messages.is_empty() {
                continue;
            }
    
            let topic_id = envelope.topic_id;
            let cancellation_token = envelope.cancellation_token.unwrap_or_default();
            let recipients = self
                .subscription_manager
                .get_subscribed_recipients(&topic_id)
                .await;
    
            println!("Recipients for topic {:?}: {:?}", topic_id, recipients);
    
            if let Some(response_tx) = envelope.response_tx {
                // ===== RPC 模式：只发给第一个订阅者 =====
                if let Some(agent_id) = recipients.first() {
                    let agent_id = agent_id.clone();
                    let message = messages[0].clone(); // 只处理第一条（RPC 通常单消息）
                    let agents = self.agents.clone();
                    let topic_id = topic_id.clone();
                    let cancellation_token = cancellation_token.clone();
    
                    tokio::spawn(async move {
                        let agent_opt = {
                            let agents_guard = agents.lock().unwrap();
                            agents_guard.get(&agent_id).cloned()
                        };
                        if let Some(agent) = agent_opt {
                            let context = MessageContext {
                                sender: None,
                                topic_id: Some(topic_id),
                                is_rpc: true,
                                cancellation_token,
                                message_id: Uuid::new_v4().to_string(),
                            };
                            let response = agent.on_message(message, context).await;
                            let _ = response_tx.send(response); // 忽略发送失败
                            println!("Agent {:?} responded to RPC", agent_id);
                        } else {
                            println!("RPC target agent {:?} not found", agent_id);
                        }
                    });
                } else {
                    // 无人订阅，关闭通道（可选：记录警告）
                    println!("No subscriber for RPC on topic: {}", topic_id);
                    drop(response_tx);
                }
            } else {
                // ===== 广播模式：发给所有订阅者 =====
                let mut handles = Vec::new();
                for message in messages {
                    for agent_id in &recipients {
                        let agent_id = agent_id.clone();
                        let message = message.clone();
                        let agents = self.agents.clone();
                        let output_sender = self.output_sender.clone();
                        let topic_id = topic_id.clone();
                        let cancellation_token = cancellation_token.clone();
    
                        let handle = tokio::spawn(async move {
                            let agent_opt = {
                                let agents_guard = agents.lock().unwrap();
                                agents_guard.get(&agent_id).cloned()
                            };
                            if let Some(agent) = agent_opt {
                                let context = MessageContext {
                                    sender: None,
                                    topic_id: Some(topic_id),
                                    is_rpc: false,
                                    cancellation_token,
                                    message_id: Uuid::new_v4().to_string(),
                                };
                                let response = agent.on_message(message, context).await;
                                let _ = output_sender.send(response).await;
                                println!("Agent {:?} processed broadcast message", agent_id);
                            } else {
                                println!("Broadcast target agent {:?} not found", agent_id);
                            }
                        });
                        handles.push(handle);
                    }
                }
    
                // 等待所有广播任务完成（可选，用于错误收集）
                for handle in handles {
                    if let Err(e) = handle.await {
                        eprintln!("Broadcast task panic: {:?}", e);
                    }
                }
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
            output_sender: self.output_sender.clone(),
        }
    }
}

#[derive(Debug)]
pub struct MessageEnvelope {
    pub message: GroupChatEvent,
    pub topic_id: String,
    pub cancellation_token: Option<CancellationToken>,
    pub response_tx: Option<oneshot::Sender<BaseChatMessage>>,
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
    async fn on_message(&self, message: BaseChatMessage, ctx: MessageContext) -> BaseChatMessage {
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

        message
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::message::{BaseChatMessage, CancellationToken, GroupChatEvent, GroupChatMessage, TextMessage};
    use std::collections::HashMap;
    use tokio::time::{timeout, Duration};

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_publish_message_broadcast() {
        let (mut runtime, _, output_receiver) = AgentRuntime::new();

        // 注册两个 agent
        let agent1 = WebAgent::new(runtime.clone(), "agent1".to_string());
        runtime.register_agent("agent1".to_string(), Box::new(agent1));

        let agent2 = WebAgent::new(runtime.clone(), "agent2".to_string());
        runtime.register_agent("agent2".to_string(), Box::new(agent2));

        // agent2 订阅 agent1 的 topic
        runtime
            .add_subscription(Subscription {
                topic_type: "agent1".to_string(),
                agent_type: "agent2".to_string(),
            })
            .await;

        // 启动 runtime
        let handle = runtime.start();

        // 构造广播消息
        let text_msg = BaseChatMessage::Text(TextMessage {
            content: "Hello from agent1!".to_string(),
            source: "agent1".to_string(),
            metadata: HashMap::new(),
        });

        let event = GroupChatEvent::Message(GroupChatMessage {
            message: text_msg.clone(),
        });

        // 发布广播
        runtime
            .publish_message(event, "agent1".to_string(), Some(CancellationToken::new()))
            .await
            .expect("Failed to publish message");

        // 从 output_receiver 接收响应（agent2 处理后输出）
        let received = timeout(Duration::from_secs(1), output_receiver.recv())
            .await
            .expect("Timeout waiting for broadcast response")
            .expect("Channel closed unexpectedly");

        assert_eq!(received.to_text(), "Hello from agent1!");

        // 关闭 runtime
        runtime.message_sender.close();
        handle.await.expect("Runtime panicked").expect("Runtime error");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_send_message_rpc() {
        let (mut runtime, _, _) = AgentRuntime::new();

        let agent1 = WebAgent::new(runtime.clone(), "agent1".to_string());
        runtime.register_agent("agent1".to_string(), Box::new(agent1));

        let agent2 = WebAgent::new(runtime.clone(), "agent2".to_string());
        runtime.register_agent("agent2".to_string(), Box::new(agent2));

        // agent2 订阅 agent1 的 topic
        runtime
            .add_subscription(Subscription {
                topic_type: "agent1".to_string(),
                agent_type: "agent2".to_string(),
            })
            .await;

        let handle = runtime.start();

        // 构造 RPC 消息
        let text_msg = BaseChatMessage::Text(TextMessage {
            content: "RPC call to agent2".to_string(),
            source: "agent1".to_string(),
            metadata: HashMap::new(),
        });

        let event = GroupChatEvent::Message(GroupChatMessage {
            message: text_msg,
        });

        // 发送 RPC 请求
        let response = runtime
            .send_message(event, "agent1".to_string(), Some(CancellationToken::new()))
            .await
            .expect("RPC failed");

        assert_eq!(response.to_text(), "RPC call to agent2");

        // 关闭
        runtime.message_sender.close();
        handle.await.expect("Runtime panicked").expect("Runtime error");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_rpc_with_no_subscriber() {
        let (runtime, _, _) = AgentRuntime::new();

        // 只注册 agent1，但不订阅任何 topic
        let agent1 = WebAgent::new(runtime.clone(), "agent1".to_string());
        runtime.register_agent("agent1".to_string(), Box::new(agent1));

        let handle = runtime.start();

        let text_msg = BaseChatMessage::Text(TextMessage {
            content: "To non-existent topic".to_string(),
            source: "agent1".to_string(),
            metadata: HashMap::new(),
        });

        let event = GroupChatEvent::Message(GroupChatMessage {
            message: text_msg,
        });

        // 发送 RPC 到无人订阅的 topic
        let result = runtime
            .send_message(event, "nonexistent".to_string(), Some(CancellationToken::new()))
            .await;

        // 应该超时（因为没人响应）
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "No response received");

        runtime.message_sender.close();
        handle.await.expect("Runtime panicked").expect("Runtime error");
    }
}