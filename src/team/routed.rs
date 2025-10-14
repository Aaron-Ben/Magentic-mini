/*作用是依据不同的message，将消息路由到不同的方法中，使用装饰器标记处理器方法（包装的代理），在初始化时自动发现这些方法，
通过router进行路由，使用FIFO 锁确保特定消息类型按照接收顺序进行处理*/

use std::pin::Pin;
use std::future::Future;
use std::error::Error;
use std::{any::{Any, TypeId}, collections::HashMap};
use crate::types::message::MessageContext;
use async_trait::async_trait;

#[async_trait]
pub trait BaseAgent: Send + Sync {
    async fn on_message_impl(
        &mut self,
        message: Box<dyn Any + Send>,
        ctx: MessageContext
    ) -> Result<Option<Box<dyn Any + Send>>, Box<dyn std::error::Error + Send + Sync>>;

    fn description(&self) -> &str;
}

// 消息处理器
pub struct MessageHandler<AgentT> {
    pub type_id: TypeId,
    pub handler: Box<dyn Fn(
        &mut AgentT,
        Box<dyn Any + Send>, 
        MessageContext
    ) -> Pin<Box<dyn Future<Output = Result<Option<Box<dyn Any + Send>>, Box<dyn Error + Send + Sync>>> + Send>> + Send + Sync>,
    pub is_rpc: bool,
    pub match_fn: Option<Box<dyn Fn(&dyn Any, &MessageContext) -> bool + Send + Sync>>,
}

// 4. 实现 RoutedAgent
pub struct RoutedAgent<AgentT> {
    agent: AgentT,
    description: String,
    handlers: HashMap<TypeId, Vec<MessageHandler<AgentT>>>,
}

impl<AgentT> RoutedAgent<AgentT>
where 
    AgentT: Send + Sync + 'static,
{
    pub fn new(agent: AgentT, description: String) -> Self {
        Self {
            agent,
            description,
            handlers: HashMap::new(),
        }
    }
    
    // 注册事件处理器（对应 @event）
    pub fn register_event<T, F, Fut>(&mut self, handler: F)
    where
        T: Any + Send + 'static,
        F: Fn(&mut AgentT, T, MessageContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send + 'static,
    {
        let wrapped = Box::new(move |agent: &mut AgentT, msg: Box<dyn Any + Send>, ctx: MessageContext| {
            let msg = *msg.downcast::<T>().unwrap();
            let fut = handler(agent,msg, ctx);
            Box::pin(async move {
                fut.await?;
                Ok(None)
            }) as Pin<Box<dyn Future<Output = Result<Option<Box<dyn Any + Send>>, Box<dyn Error + Send + Sync>>> + Send>>
        });
        
        self.handlers
            .entry(TypeId::of::<T>())
            .or_default()
            .push(MessageHandler{
                type_id: TypeId::of::<T>(),
                handler: wrapped,
                is_rpc: false,
                match_fn: None,
            });
    }
    
    pub fn register_rpc<T, R, F, Fut>(&mut self, handler: F)
    where
        T: Any + Send + 'static,
        R: Any + Send + 'static,
        F: Fn(&mut AgentT, T, MessageContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, Box<dyn Error + Send + Sync>>> + Send + 'static,
    {
        let wrapped = Box::new(move |agent: &mut AgentT, msg: Box<dyn Any + Send>, ctx: MessageContext| {
            let msg = *msg.downcast::<T>().unwrap();
            let fut = handler(agent, msg, ctx);
            Box::pin(async move {
                let result = fut.await?;
                Ok(Some(Box::new(result) as Box<dyn Any + Send>))
            }) as Pin<Box<dyn Future<Output = Result<Option<Box<dyn Any + Send>>, Box<dyn Error + Send + Sync>>> + Send>>
        });
        
        self.handlers
            .entry(TypeId::of::<T>())
            .or_default()
            .push(MessageHandler {
                type_id: TypeId::of::<T>(),
                handler: wrapped,
                is_rpc: true,
                match_fn: None,
            });
    }

    async fn route_message(
        &mut self,
        message: Box<dyn Any + Send>,
        ctx: MessageContext,
    ) -> Result<Option<Box<dyn Any + Send>>, Box<dyn Error + Send + Sync>> {
        let type_id = (&*message).type_id();    // 获取消息类型
        
        if let Some(handlers) = self.handlers.get(&type_id) {   // 获取消息类型对应的处理器
            for entry in handlers {
                // 检查 RPC 类型匹配
                if entry.is_rpc != ctx.is_rpc {
                    continue;
                }
                
                // 检查自定义匹配函数
                if let Some(match_fn) = &entry.match_fn {
                    if !match_fn(&*message, &ctx) {
                    }
                }
                
                // 调用处理器
                return (entry.handler)(&mut self.agent, message, ctx).await;
            }
        }
        
        Self::on_unhandled_message(message, ctx).await
    }

    async fn on_unhandled_message(
        _message: Box<dyn Any + Send>,
        _ctx: MessageContext,
    ) -> Result<Option<Box<dyn Any + Send>>, Box<dyn Error + Send + Sync>> {
        println!("Unhandled message");
        Ok(None)
    }
}

#[async_trait]
impl<AgentT> BaseAgent for RoutedAgent<AgentT>
where
    AgentT: Send + Sync + 'static,
{
    async fn on_message_impl(
        &mut self,
        message: Box<dyn Any + Send>,
        ctx: MessageContext,
    ) -> Result<Option<Box<dyn Any + Send>>, Box<dyn Error + Send + Sync>> {
        self.route_message(message, ctx).await
    }

    fn description(&self) -> &str {
        &self.description
    }
}
