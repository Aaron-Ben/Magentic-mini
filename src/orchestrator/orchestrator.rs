use chrono::Local;
use serde_json::Value as JsonValue;
use tokio_util::sync::CancellationToken;
use crate::orchestrator::config::OrchestratorConfig;
use crate::types::event::GroupChatStart;
use crate::orchestrator::types::{OrchestratorState,MessageContext};
use crate::types::message::{ChatMessage, LLMMessage, TextMessage, UserMessage};
use crate::llm::client::{ChatCompletionClient};
use anyhow::{bail, Ok, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_channel::Sender;



#[derive(Debug)]
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
    message: Box<dyn ChatMessage>,
    output_message_queue: Arc<Mutex<Sender<MessageType>>>,
    model_client: Arc<dyn ChatCompletionClient>,
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
        message: Box<dyn ChatMessage>,
        participant_topic_types: Vec<String>,
        participant_descriptions: Vec<String>,
        participant_names: Vec<String>,
        output_message_queue: Arc<Mutex<async_channel::Sender<MessageType>>>,
        model_client: Arc<dyn ChatCompletionClient>,
        config: OrchestratorConfig,
        termination_condition: Option<Box<dyn TerminationConditionTrait>>,
        max_turns: Option<i32>,
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
    ) -> Result<()> {
        // 检查对话是否已经终止

        // 确保消息不为空

        // 发送消息给所有代理
        

        // 将消息添加到历史记录
        for msg in message.messages {
            self.state.message_history.push(msg);
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
    ) -> Result<()> {
        Ok(())
    }

    async fn orchestrator_step(
        &mut self,
        ctx: MessageContext,
    ) -> Result<()> { 
        
        if self.state.is_paused {
            self.request_next_speaker(&self.user_agent_topic, ctx).await?;
            return Ok(());
        }

        if self.state.in_planning_mode {
            self.orchestrator_step_planning(ctx.cancellation_token).await?;
        } else {
            self.orchestrator_step_execution(ctx.cancellation_token,false).await?;
        }

        Ok(())
    }

    async fn orchestrator_step_planning(
        &mut self,
        cancellation_token: CancellationToken,
    ) -> Result<()> { 

        // Planning stage

        // first planning

        if self.state.task.is_empty() && self.state.plan_str.is_empty() {
            self.state.task = "Task:".to_string();
        } else {

        }

        Ok(())
    }


    async fn orchestrator_step_execution(
        &mut self,
        cancellation_token: CancellationToken,
        first_step: bool,
    ) -> Result<()> { 
        if first_step {
            // TODO
        }

        self.state.n_rounds += 1;
        let context = self.thread_to_context(None);

        
        let progress_ledger_prompt = self.get_progress_ledger_prompt(
            self.state.task.clone(),
            self.state.plan_str.clone(),
            self.state.current_step_idx,
            self.team_description.clone(),
            self.agent_execution_names.clone(),
        );

        context.push(LLMMessage::UserMessage(
            UserMessage::new_text(progress_ledger_prompt, self.name.clone())
        ));

        let plan_response = self.get_json_response().await?;

        if self.state.is_paused {
            unimplemented!()
        }

        self.state.plan_str = String::new();

        // self.state.message_history.push

        

        

        Ok(())
    }

    async fn get_json_response(
        &mut self,
    ) -> Result<()> {

        Ok(())
    }

    async fn request_next_speaker(
        &mut self,
        next_speaker: &str,
        ctx: MessageContext,
    ) -> Result<()> { 
        Ok(())
    }


    // 对话历史转为LLMMessage
    fn thread_to_context(&self, message:Option<String>) -> Vec<LLMMessage> {

        let chat_messages = message.unwrap_or(&self.state.message_history);

        let mut context_messages = Vec::new();
        let date_today = Local::now().format("%Y-%m-%d").to_string();


        if self.state.in_planning_mode {
            let planning_prompt = format!("This is a planning step. The task is: {}", self.state.task);
            context_messages.push(LLMMessage::SystemMessage(
                planning_prompt,
            ));
        } else {
            let execution_prompt = format!("This is a execution step. The task is: {}", self.state.task);
            context_messages.push(LLMMessage::SystemMessage(
                execution_prompt,
            ));
        }

        // 步骤 3: 使用辅助函数转换对话历史
        // let is_multimodal = self.model_client.model_info.vision;
        // let converted_history = convert_agent_messages_to_llm_messages(
        //     chat_messages,
        //     &self.name,
        //     is_multimodal,
        // );

        // // 将转换后的历史记录追加到上下文中
        // context_messages.extend(converted_history);

        // // 步骤 4: 返回最终构建完成的上下文
        // context_messages

        Vec::new()

    }

    async fn replan(&self,reason:String,cancellation_token: CancellationToken) -> Result<()> {
        self.state.in_planning_mode = true;

        let context = self.thread_to_context(None);

        // Store completed steps



        Ok(())
    }

    async fn publish_group_chat_message(
        self,
        content: String,
        cancellation_token: CancellationToken,
        internal: bool,
        metadata: Option<HashMap<String,String>>
    ) -> Result<()> {
        let message = TextMessage::new{
            content : content,
            source: self.name.clone(),
            models_usage: None,
            metadata: metadata.or_else(
                {
                    let mut map = std::collections::HashMap::new();
                    map.insert("internal".to_string(), internal.to_string());
                    map
                }
            ),
        };

        // publish_message

        self.output_message_queue.send(message).await?;

        // publish_message

        Ok(())
    }

    fn get_progress_ledger_prompt(
        &self,
        task: String,
        plan: String,
        step_index: i32, 
        team: String,
        names: Vec<String>,
    ) -> Result<String> {

        let additional_instructions = String::new();

        let step_type = "PlanStep".to_string();

        Ok(String::new())
    }

}

