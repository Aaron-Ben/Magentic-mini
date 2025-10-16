use chrono::Local;
use serde_json::Value as JsonValue;
use serde_json::Value;
use tokio_util::sync::CancellationToken;
use crate::orchestrator::config::OrchestratorConfig;
use crate::orchestrator::message::{LLMMessage, TextMessage, UserMessage};
use crate::orchestrator::types::{OrchestratorState, ProgressLedger};

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
    model_context: Vec<LLMMessage>,         // 可能有误，暂时先这样
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

    async fn handle_agent_response(
        &self,

    ) -> Result<()> {
        Ok(())
    }

    async fn prepare_final_answer(
        &self,
        reason: String,
        final_answer: Option<String>,
        force_stop: bool,
    ) -> Result<()> {
        Ok(())
    }

    async fn orchestrator_step(
        &self,
    ) -> Result<()> { 
        
        if self.state.in_planning_mode {
            self.orchestrator_step_planning().await?;
        } else {
            self.orchestrator_step_execution(false).await?;
        }

        Ok(())
    }

    async fn orchestrator_step_planning(
        &mut self,
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
        first_step: bool,
    ) -> Result<()> { 
        // 第一次计划
        if first_step {
            
            let content = format!(
                r#"
                We are working to address the following user request:
                \\n\\n
                {task}
                \\n\\n
                To answer this request we have assembled the following team:
                \\n\\n
                {team}
                \\n\\n
                Here is the plan to follow as best as possible:
                \\n\\n
                {plan}
                "#,
                task = self.state.task.clone(),
                team = self.team_description.clone(),
                plan = self.state.plan_str.clone(),
            );

            let ledger_message = TextMessage::new(content, self.name.clone());

            self.state.message_history.push(ledger_message);
        }

        if self.state.current_step_idx >= self.state.plan.len() || self.state.n_rounds > self.config.max_turns {
            self.prepare_final_answer("Max rounds reached".to_string(), None, false).await?;
            return Ok(());
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
            UserMessage::new(
                progress_ledger_prompt, self.name.clone()
            ),
        ));

        let json_str = self.get_json_response(context, self).await?;

        let progress_ledger: ProgressLedger = serde_json::from_str(json_str).expect("Failed to parse JSON");

        if !first_step {
            let need_to_replan = progress_ledger.need_to_replan.answer;
            let replan_reason = progress_ledger.need_to_replan.reason;

            if need_to_replan {
                if self.state.n_replans < self.config.max_replans {
                    self.state.n_replans += 1;
                    self.replan(replan_reason).await?;
                    return Ok(());
                } else {
                    let reason = format!("We need to replan but max replan attempts reached: {replan_reason} ");
                    self.prepare_final_answer(reason.to_string(), None,None).await?;
                    return Ok(());
                }
            }

            if progress_ledger.is_current_step_complete.answer {
                self.state.current_step_idx += 1;
            }
        }

        if self.state.current_step_idx >= self.state.plan {
            self.prepare_final_answer("Plan completed".to_string(), None, false).await?;
            return Ok(());
        }

        let new_instruction = self.get_agent_instruction(instruction, agent_name);

        message_to_send = TextMessage::new(new_instruction, self.name, {"internal": true});
        self.state.message_history.push(message_to_send);

        let next_speaker = progress_ledger.instruction_or_question.agent_name;
        for name in self.agent_execution_names {
            if name == next_speaker {
                self.request_next_speaker(next_speaker).await?;
                break;
            }
        }
    }

    async fn get_json_response(
        &mut self,
        messages: Vec<LLMMessage>,
    ) -> Result<Vec<LLMMessage>> {

        self.model_context.clear();

        for message in messages {
            self.model_context.push(message);
        }

        // llm_call 这里调用使用model_client，这里应该是LLM使用我们的prompt，给出xxx，暂时结构位置
        let response = Vec<messages>;
        Ok(response)
    }

    async fn request_next_speaker(
        &mut self,
        next_speaker: &str,
    ) -> Result<()> { 
        Ok(())
    }


    // 对话历史转为LLMMessage
    fn thread_to_context(&self, message:Option<Vec<BaseChatMessage>>) -> Vec<LLMMessage> {

        let chat_messages = message.unwrap_or(&self.state.message_history);

        let mut context_messages:Vec<LLMMessage> = Vec::new();
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

        Ok(())
    }

    fn get_progress_ledger_prompt(
        &self,
        task: String,
        plan: String,
        step_index: usize, 
        team: String,
        names: Vec<String>,
    ) -> Result<String> {
        let plan_steps = self
            .state
            .plan
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Plan must be initialized"))?;

        if step_index >= plan_steps.len() {
            return Err(anyhow::anyhow!(
                "step_index {} is out of bounds (plan has {} steps)",
                step_index,
                plan_steps.len()
            ));
        }

        let step = &plan_steps[step_index];
        let names_str = names.join(", ");
        let additional_instructions = String::new();

        let prompt = format!(
            r#"
        We are at step index {step_index} in the plan which is 
        Title: {step_title}
        Details: {step_details}
        agent_name: {agent_name}
        And we have assembled the following team:
        {team}
        The browser the web_surfer accesses is also controlled by the user.

        To make progress on the request, please answer the following questions, including necessary reasoning:

            - is_current_step_complete: Is the current step complete? (True if complete, or False if the current step is not yet complete)
            - need_to_replan: Do we need to create a new plan? (True if user has sent new instructions and the current plan can't address it. True if the current plan cannot address the user request because we are stuck in a loop, facing significant barriers, or the current approach is not working. False if we can continue with the current plan. Most of the time we don't need a new plan.)
            - instruction_or_question: Provide complete instructions to accomplish the current step with all context needed about the task and the plan. Provide a very detailed reasoning chain for how to complete the step. If the next agent is the user, pose it directly as a question. Otherwise pose it as something you will do.
            - agent_name: Decide which team member should complete the current step from the list of team members: {names}. 
            - progress_summary: Summarize all the information that has been gathered so far that would help in the completion of the plan including ones not present in the collected information. This should include any facts, educated guesses, or other information that has been gathered so far. Maintain any information gathered in the previous steps.

        Important: it is important to obey the user request and any messages they have sent previously.

        {additional_instructions}

        Please output an answer in pure JSON format according to the following schema. The JSON object must be parsable as-is. DO NOT OUTPUT ANYTHING OTHER THAN JSON, AND DO NOT DEVIATE FROM THIS SCHEMA:

            {{
                "is_current_step_complete": {{
                    "reason": string,
                    "answer": boolean
                }},
                "need_to_replan": {{
                    "reason": string,
                    "answer": boolean
                }},
                "instruction_or_question": {{
                    "answer": string,
                    "agent_name": string (the name of the agent that should complete the step from {names})
                }},
                "progress_summary": "a summary of the progress made so far"

            }}
            "#,
            task = task,
            plan = plan,
            step_index = step_index,
            step_title = step.title,
            step_details = step.details,
            agent_name = step.agent_name,
            team = team,
            names = names_str,
            additional_instructions = additional_instructions,
        );

        Ok(prompt)
    }

    pub fn validate_plan_json(json_response: &Value) -> bool {
        let obj = match json_response.as_object() {
            Some(obj) => obj,
            None => return false,
        };

        let keys = ["task", "steps", "needs_plan", "response", "plan_summary"];
        for &key in &keys {
            if !obj.contains_key(key) {
                return false;
            }
        }

        let steps = match obj.get("steps").and_then(|v| v.as_array()) {
            Some(s) => s,
            None => return false, // "steps" is not an array
        };

        for step in steps {
            let step_obj = match step.as_object() {
                Some(obj) => obj,
                None => return false,
            };

            let keys = ["title", "details", "agent_name"];
            for &key in &keys {
                if !step_obj.contains_key(key) {
                    return false;
                }
            }
        }
        true
    }

    pub fn get_agent_instruction(&self, instruction: String, agent_name: String) -> Result<String> {
        let prompt = format!(
            r#"    Step {step_index}: {step_title}
            \\n\\n
            {step_details}
            \\n\\n
            Instruction for {agent_name}: {instruction}
            "#,
            step_index = self.state.current_step_idx + 1,
            step_title = self.state.plan[self.state.current_step_idx].title,
            step_details = self.state.plan[self.state.current_step_idx].details,
            agent_name = agent_name,
            instruction = instruction,
            );

            Ok(prompt)
        }

}
