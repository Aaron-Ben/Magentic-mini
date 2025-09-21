use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use tokio_util::sync::CancellationToken;
use crate::types::event::GroupChatStart;
use crate::orchestrator::types::{OrchestratorState,MessageContext};
use crate::tools::base::token_limited::{LLMMessage, TokenLimitedChatCompletionContext};
use crate::types::StopMessage;
use anyhow::{bail, Ok, Result};
use std::sync::{Arc, Mutex};
use async_channel::Sender;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub cooperative_planning: bool,
    pub autonomous_execution: bool,
    pub allow_follow_up_input: bool,
    pub plan: Option<Plan>,
    pub max_turns: Option<usize>,
    pub allow_for_replans: bool,
    pub max_json_retries: usize,
    pub saved_facts: Option<String>,
    pub allowed_websites: Option<Vec<String>>,
    pub do_bing_search: bool,
    pub final_answer_prompt: Option<String>,
    pub model_context_token_limit: Option<usize>,
    pub retrieve_relevant_plans: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Orchestrator<MessageType> {
    // 基础字段
    pub name: String,
    pub group_topic_type: String,           // 群聊的主题类型
    pub output_topic_type: String,          // 输出的主题类型
    pub participant_topic_types: Vec<String>,
    pub participant_descriptions: Vec<String>,
    pub participant_names: Vec<String>,
    pub termination_conditions: Vec<String>,
    pub max_turns: Option<i32>,
    pub termination_condition: Option<Box<dyn TerminationConditionTrait>>,

    // 特有字段
    message: MessageTypeItem,
    output_message_queue: Arc<Mutex<Sender<MessageType>>>,
    model_client: Arc<dyn ChatCompletionClient>,
    model_context: TokenLimitedChatCompletionContext,       // 对话的上下文管理器
    config: OrchestratorConfig,
    user_agent_topic: String,
    web_agent_topic: String,

    // 内部状态字段
    state: OrchestratorState,
    agent_execution_names: Vec<String>,
    agent_execution_descriptions: Vec<String>,
    team_description: String,
    last_browser_metadata_hash: String,
}

pub trait TerminationConditionTrait: Send + Sync {}

type ValidateJsonFn = Arc<dyn Fn(&JsonValue) -> bool + Send + Sync>;

impl <MessageType> Orchestrator<MessageType> {

    pub async fn new(
        name: String,
        group_topic_type: String,
        output_topic_type: String,
        message: MessageTypeItem,
        participant_topic_types: Vec<String>,
        participant_descriptions: Vec<String>,
        participant_names: Vec<String>,
        output_message_queue: Arc<Mutex<async_channel::Sender<MessageType>>>,
        model_client: Arc<dyn ChatCompletionClient>,
        config: OrchestratorConfig,
        termination_condition: Option<Box<dyn TerminationConditionTrait>>,
        max_turns: Option<usize>,
    ) -> Result<Self> {

        let user_agent_topic = "user_proxy".to_string();
        let web_agent_topic = "web_agent".to_string();

        if !participant_names.contains(&user_agent_topic) {
            if !(config.autonomous_execution && config.allow_follow_up_input) {
                bail!("User agent topic {} not in participant names {:?}", 
                      user_agent_topic, participant_names);
            }
        }

        // 初始化基础字段
        let mut orchestrator = Self {
            name,
            group_topic_type,
            output_topic_type,
            participant_topic_types,
            participant_descriptions,
            participant_names,
            termination_conditions: Vec::new(),
            max_turns,
            termination_condition,
            message: message,
            output_message_queue: output_message_queue,
            model_client: model_client,
            model_context: TokenLimitedChatCompletionContext::new(
                model_client.clone(),
                config.model_context_token_limit,
            ),
            config: config,
            user_agent_topic: user_agent_topic,
            web_agent_topic: web_agent_topic,
            
            // 临时值，会在setup_internals中正确初始化
            state: OrchestratorState::default(),
            agent_execution_names: Vec::new(),
            agent_execution_descriptions: Vec::new(),
            team_description: String::new(),
            last_browser_metadata_hash: String::new(),
        };

        orchestrator.set_internal_variables()?;

        Ok(orchestrator)
    }

    // 设置内部变量
    fn set_internal_variables(&mut self) -> Result<()> {
        self.state = OrchestratorState::default();

        self.agent_execution_descriptions = self.participant_descriptions.clone();
        self.agent_execution_names = self.participant_names.clone();

        // 根据autonomous_execution配置过滤用户代理
        if self.config.autonomous_execution {
            if let Some(user_index) = self.agent_execution_names
                .iter()
                .position(|name| name == &self.user_agent_topic) 
            {
                self.agent_execution_names.remove(user_index);
                self.agent_execution_descriptions.remove(user_index);
            }
        }

        // 添加"无操作"代理
        self.agent_execution_names.push("no_action_agent".to_string());
        self.agent_execution_descriptions.push(
            "If for this step no action is needed, you can use this agent to perform no action".to_string()
        );

        // 团队描述
        self.team_description = self.agent_execution_names
            .iter()
            .zip(&self.agent_execution_descriptions)
            .map(|(name, description)| format!("{} - {}", name, description.trim()))
            .collect::<Vec<_>>()
            .join("\n");

        // 初始化浏览器元数据哈希
        self.last_browser_metadata_hash = String::new();

        Ok(())
    }


    // 入口,通过选择发言着让群聊的开始
    pub async fn handle_start(
        &mut self,
        message: GroupChatStart,
        ctx: MessageContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 检查对话是否已经终止
        if let Some(ref termination_condition) = self.termination_condition {
            if termination_condition.terminated().await {
                let early_stop_message = StopMessage {
                    content: "The group chat has already terminated.".to_string(),
                    source: self.name.clone(),
                };
                self.signal_termination(early_stop_message).await?;
                return Ok(());
            }
        }

        // 确保消息不为空
        let messages = message.messages.as_ref()
            .ok_or("Messages cannot be None")?;

        // 发送消息给所有代理
        self.publish_message(
            GroupChatStart {
                messages: messages.clone(),
            },
            DefaultTopicId {
                topic_type: self.group_topic_type.clone(),
            },
            ctx.cancellation_token,
        ).await?;

        // 将消息添加到历史记录
        for msg in messages {
            self.state.message_history.push(msg.clone());
        }

        self.orchestrator_step(ctx.cancellation_token).await?;

        Ok(())
    }

    async fn handle_agent_response(
        &self,

    ) -> Result<()> {
        Ok(())
    }

    async fn prepare_final_answer(
        &self,
        reason: String,
        cancellation_token: CancellationToken,
        final_answer: Option<String>,
        force_stop: bool,
    ) -> Result<()> {

    }

    async fn orchestrator_step(&mut self,cancellation_token: CancellationToken) -> Result<()> { 
        
        if self.state.is_paused {
            self.request_next_speaker(&self.user_agent_topic, cancellation_token);
            return Ok(());
        }

        if self.state.in_planning_mode {
            self.orchestrator_step_planning(cancellation_token)
        } else {
            self.orchestrator_step_execution(cancellation_token)
        }
    }

    async fn orchestrator_step_planning(
        &mut self,
        cancellation_token: CancellationToken,
    ) -> Result<()> { 
        // 
    }


    async fn orchestrator_step_execution(
        &mut self,
        cancellation_token: CancellationToken,
        first_step: bool,
    ) -> Result<()> { 
        if first_step {
            
        }
    }

    async fn get_json_response(
        &mut self,
        messages: Vec<LLMMessage>,
        validate_json: ValidateJsonFn,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        let mut retries = 0;
        let mut exception_message = String::new();

        while retries < self.config.max_json_retries {
            // 清空并重建上下文
            self.model_context.clear().await;

            for msg in &messages {
                self.model_context.add_message(msg.clone()).await;
            }

            if !exception_message.is_empty() {
                let feedback_msg = UserMessage::new(exception_message.clone(), self._name.clone());
                self._model_context.add_message(feedback_msg).await;
            }

            let token_limited_messages = self._model_context.get_messages().await;

            // 调用模型

        }


        Ok(())
    }

    async fn request_next_speaker(&mut self, next_speaker: &str, cancellation_token: CancellationToken) -> Result<()> { 
        Ok(())
    }


}
