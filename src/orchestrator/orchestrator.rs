use chrono::Local;
use futures::lock::Mutex;
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use serde_json::Value;
use crate::agents::Agent;
use crate::orchestrator::config::OrchestratorConfig;
use crate::orchestrator::message::{ChatMessage, LLMMessage, Message, MessageType, SystemMessage, TextMessage, UserContent, UserMessage};
use crate::orchestrator::types::{OrchestratorState, ProgressLedger};
use crate::orchestrator::plan::{Plan, PlanResponse};
use anyhow::{Ok, Result};
use std::collections::HashMap;
use std::sync::{Arc};


#[derive(Debug)]
pub struct Orchestrator {
    // 基础字段
    pub name: String,
    pub agents: HashMap<String, Arc<Mutex<Box<dyn Agent>>>>,
    pub chat_history: Vec<ChatMessage>,
    pub participant_descriptions: Vec<String>,
    pub participant_names: Vec<String>,
    pub termination_conditions: Vec<String>,
    pub max_turns: Option<i32>,

    // 特有字段
    pub message: ChatMessage,
    model_context: Vec<LLMMessage>,         // 可能有误，暂时先这样
    model_client: Arc<LLMClient>,
    config: OrchestratorConfig,

    // 内部状态字段
    state: OrchestratorState,
    agent_execution_names: Vec<String>,
    agent_execution_descriptions: Vec<String>,
    team_description: String,
    last_browser_metadata_hash: String,
}

pub trait TerminationConditionTrait: Send + Sync {}

type ValidateJsonFn = Arc<dyn Fn(&JsonValue) -> bool + Send + Sync>;

impl Orchestrator {

    pub async fn new(
        name: String,
        message: ChatMessage,
        participant_descriptions: Vec<String>,
        participant_names: Vec<String>,
        model_client: Arc<LLMClient>,
        config: OrchestratorConfig,
        termination_condition: Option<Box<dyn TerminationConditionTrait>>,
        max_turns: Option<i32>,
    ) -> Result<Self> {

        let user_agent_topic = "user_proxy".to_string();
        let web_agent_topic = "web_agent".to_string();

        // 初始化基础字段
        let mut orchestrator = Self {
            name,
            participant_descriptions,
            participant_names,
            termination_conditions: Vec::new(),
            max_turns,
            message: message,
            model_context: Vec::new(),
            model_client: model_client,
            config: config,
            
            // 临时值，会在setup_internals中正确初始化
            state: OrchestratorState::default(),
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
                .position(|name| name == "user_proxy") 
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

    async fn prepare_final_answer(
        &mut self,
        reason: String,
        final_answer: Option<String>,
    ) -> Result<()> {
        if final_answer.is_none() {
            let mut context = self.thread_to_context(None)?;
            context.push(LLMMessage::UserMessage(
                UserMessage::new(
                    UserContent::String(reason),
                    self.name.clone(),
                ),
            ));

            let final_answer_prompt = format!(
                r#"
                We are working on the following task:
                {task}
                The above messages contain the steps that took place to complete the task.
                Based on the information gathered, provide a final response to the user in response to the task.
                Make sure the user can easily verify your answer, include links if there are any. 
                Please refer to steps of the plan that was used to complete the task. Use the steps as a way to help the user verify your answer.
                Make sure to also say whether the answer was found using online search or from your own knowledge.
                There is no need to be verbose, but make sure it contains enough information for the user.
                "#,
                task = self.state.task.clone(),
            );

            let progress_summary = format!("Progress summary: {}", self.state.information_collected);

            let content = format!("{}\n\n{}", progress_summary, final_answer_prompt);
            context.push(LLMMessage::UserMessage(
                UserMessage::new( 
                    UserContent::String(content),
                    self.name.clone(),
                ),
            ));

            self.model_context.clear();
            for message in context {
                self.model_context.push(message.clone());
            }

            // 调用LLM
            let response = "";
            let _final_answer = Some(response);
        }

        let content = format!("Final answer: {}", final_answer.unwrap());
        let message = ChatMessage::Text(TextMessage::new(
            content,
            self.name.clone(),
        ));

        self.state.message_history.push(message.clone());
        self.notify_all(message).await?;

        // 结束
        // self.state.is_terminated = true;
        Ok(())
    }

    pub async fn notify_all(&self, content: ChatMessage) -> Result<()> {
        let notify_msg = Message {
            from: "orchestrator".to_string(),
            to: "all".to_string(),
            chat_history: vec![content.clone()],
            msg_type: MessageType::Notify,
        };

        for(_name, agent) in &self.agents {
            let mut agent = agent.lock().await;
            let _ = agent.on_message_stream(notify_msg.clone()).await?;
        }
        Ok(())
    }

    pub async fn select_next_speaker(&self, agent_name: String, content: ChatMessage) -> Result<()> {
        let execute_msg = Message {
            from: "Orchestrator".to_string(),
            to: agent_name.to_string(),
            chat_history: vec![content.clone()],
            msg_type: MessageType::Execute,
        };

        let agent = self.agents.get(&agent_name)
            .ok_or_else(|| anyhow::anyhow!("Agent {} not found", agent_name))?;
        
        let mut agent = agent.lock().await;
        agent.on_message_stream(execute_msg).await?;
        Ok(())
    }

    async fn handle_agent_response(&mut self, _agent_name: &str, response: ChatMessage) -> Result<()> {
        self.state.message_history.push(response.clone());
        self.orchestrator_step().await?;
        Ok(())
    }

    async fn orchestrator_step(
        &mut self,
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
        let mut plan_response: PlanResponse = PlanResponse::default();

        // first planning
        if self.state.task.is_empty() && self.state.plan_str.is_empty() {
            self.state.task = "Task:".to_string();

            let mut context = self.thread_to_context(None)?;
            context.push(LLMMessage::UserMessage(
                UserMessage::new(
                    UserContent::String(self.get_task_ledger_plan_prompt(self.team_description)?),
                    self.name.clone(),
                ),
            ));

            plan_response = self.get_json_response(context, self.validate_plan_json);

            self.state.plan = Plan::from_list_of_dicts_or_str(plan_response.steps);
            self.state.plan_str = serde_json::to_string(&self.state.plan.as_ref().unwrap())?;

            self.state.message_history.push(
                ChatMessage::Text(
                    TextMessage::new(
                        plan_response.response,
                         self.name.clone()
                        )
                    )
            );
        } else {
            if true {
                self.orchestrator_step_execution(true).await?;
                return Ok(());
            } else {
                let user_plan = "";
                if !user_plan.is_empty() {
                    self.state.plan = Plan::from_list_of_dicts_or_str(user_plan);
                    self.state.plan_str = user_plan.to_string();
                }

                let mut context = self.thread_to_context(None)?;

                context.push(LLMMessage::UserMessage(
                    UserMessage::new(
                        UserContent::String(user_plan.to_string()), 
                        self.name.clone()
                    )
                ));

                plan_response = self.get_json_response(context, self.validate_plan_json);

            }
        }

        if plan_response.needs_plan {
            return Ok(());
        } else {
            self.select_next_speaker(
                &self.name,
                
                self.state.message_history.clone()
            ).await?;
            return Ok(());
        }
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

            let ledger_message = ChatMessage::Text(TextMessage::new(content, self.name.clone()));

            self.state.message_history.push(ledger_message.clone());
        }

        let length = if let Some(plan) = &self.state.plan {
            plan.steps.len()
        } else {
            0
        };

        let max_turns = self.config.max_turns.unwrap_or(100) as usize;
        if self.state.current_step_idx >= length || self.state.n_rounds > max_turns {
            self.prepare_final_answer("Max rounds reached".to_string(), None).await?;
            return Ok(());
        }

        self.state.n_rounds += 1;
        let mut context = self.thread_to_context(None)?;

        
        let progress_ledger_prompt = self.get_progress_ledger_prompt(
            self.state.task,
            self.state.plan_str.clone(),
            self.state.current_step_idx,
            self.team_description,
            self.agent_execution_names,
        )?;

        context.push(LLMMessage::UserMessage(
            UserMessage::new(
                UserContent::String(progress_ledger_prompt), self.name.clone()
            ),
        ));

        let json_str = self.get_json_response(context, self).await?;

        let progress_ledger: ProgressLedger = self.get_json_response(context, self.validate_progress_ledger_json).await?;

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
                    self.prepare_final_answer(reason, None).await?;
                    return Ok(());
                }
            }

            if progress_ledger.is_current_step_complete.answer {
                self.state.current_step_idx += 1;
            }
        }

        let plan_length = if let Some(plan) = &self.state.plan {
            plan.steps.len()
        } else {
            0
        };

        if self.state.current_step_idx >= plan_length {
            self.prepare_final_answer("Plan completed".to_string(), None).await?;
            return Ok(());
        }

        let new_instruction = self.get_agent_instruction(
            progress_ledger.instruction_or_question.answer.clone(),
            progress_ledger.instruction_or_question.agent_name.clone()
        )?;

        let message_to_send = ChatMessage::Text(TextMessage::new(
            new_instruction, 
            self.name.clone()
        ));
        self.state.message_history.push(message_to_send);

        let next_speaker = progress_ledger.instruction_or_question.agent_name;
        for name in self.agent_execution_names {
            if name == next_speaker {
                self.select_next_speaker(next_speaker).await?;
                break;
            }
        }
        Ok(())
    }

    async fn get_json_response<T: DeserializeOwned>(
        &mut self,
        messages: Vec<LLMMessage>,
        validate_json: ValidateJsonFn,
    ) -> Result<T> {

        self.model_context.clear();

        for message in messages {
            self.model_context.push(message);
        }

        // llm_call 这里调用使用model_client，这里应该是LLM使用我们的prompt，给出xxx，暂时结构位置
        // let response = Vec<messages>;
        // Ok(response)
        Ok(T::default())
    }

    // 对话历史转为LLMMessage
    fn thread_to_context(&self, message:Option<Vec<ChatMessage>>) -> Result<Vec<LLMMessage>> {

        let chat_messages = message.unwrap_or(&self.state.message_history);

        let mut context_messages:Vec<LLMMessage> = Vec::new();
        let date_today = Local::now().format("%Y-%m-%d").to_string();

        if self.state.in_planning_mode {
            let planning_prompt = format!("This is a planning step. The task is: {}", self.state.task);
            context_messages.push(LLMMessage::SystemMessage(
                SystemMessage::new(planning_prompt),
            ));
        } else {
            let execution_prompt = format!("This is a execution step. The task is: {}", self.state.task);
            context_messages.push(LLMMessage::SystemMessage(
                SystemMessage::new(execution_prompt),
            ));
        }

        // 步骤 3: 使用辅助函数转换对话历史
        // let converted_history = convert_agent_messages_to_llm_messages(
        //     chat_messages,
        //     &self.name,
        //     is_multimodal,
        // );

        // 将转换后的历史记录追加到上下文中
        // context_messages.extend(converted_history);

        // 步骤 4: 返回最终构建完成的上下文
        // context_messages

        Ok(Vec::new())

    }

    async fn replan(&self,reason:String) -> Result<()> {
        self.state.in_planning_mode = true;

        let context = self.thread_to_context(None)?;

        let completed_steps = if let Some(ref plan) = self.state.plan {
            &plan.steps[..self.state.current_step_idx]
        } else {
            &[]
        };

        completed_steps
            .iter()
            .enumerate()
            .map(|(i,step)| -> String {
                format!(
                    "COMPLETED STEP {}: title=\"{}\", details=\"{}\", agent=\"{}\"",
                    i + 1,
                    step.title,
                    step.details,
                    step.agent_name
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let replan_prompt = self.get_task_ledger_replan_prompt(self.team_description.clone(), self.state.task.clone(), self.state.plan_str.clone())?;

        context.push(LLMMessage::UserMessage(
            UserMessage::new(
                UserContent::String(replan_prompt), 
                self.name.clone()
            )
        ));

        let plan_response: PlanResponse = self.get_json_response(context, self.validate_plan_json);

        let new_plan = if plan_response.steps.is_empty() {
            None
        } else {
            Some(Plan {
                task: Some(plan_response.task),
                steps: plan_response.steps,
            })
        };

        let combined_steps = if let Some(plan) = new_plan {
            [completed_steps, &plan.steps].concat().to_vec()
        } else {
            completed_steps
        };

        let new_plan_obj = Plan {
            task: self.state.task.clone(),
            steps: combined_steps,
        };

        let json_str = serde_json::to_string(&new_plan_obj)?;

        self.state.plan_str = json_str;
        self.state.plan = Some(new_plan_obj);

        let plan_summary = plan_response
            .get("plan_summary")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let updated_summary = format!("Replanning: {}",plan_summary);

        plan_response.plan_summary = updated_summary;

        let json_string = serde_json::to_string(&plan_response)?;

        self.notify_all(ChatMessage::Text(
            TextMessage::new(
                json_string, 
                self.name.clone())
            )).await?;
        
        // 操作交给用户
        self.select_next_speaker(&self.name, ChatMessage::Text(TextMessage::new(json_string, self.name.clone()))).await?;
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

        let step = if let Some(plan) = &self.state.plan {
            &plan.steps[step_index]
        } else {
            return Err(anyhow::anyhow!("Plan must be initialized"));
        };


        let names_str = names.join(", ");
        let additional_instructions = String::new();

        let prompt = format!(
            r#"
        Recall we are working on the following request:
        {task}
        This is our current plan:
        {plan}

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

        let steps = match obj.get("steps") {
            Some(Value::Array(s)) => s,
            _ => return false,
        };

        for step in steps {
            let step_obj = match step.as_object() {
                Some(obj) => obj,
                None => return false,
            };

            if !step_obj.contains_key("title")
                || !step_obj.contains_key("details")
                || !step_obj.contains_key("agent_name")
            {
                return false;
            }
        }
        true
    }

    pub fn get_agent_instruction(&self, instruction: String, agent_name: String) -> Result<String> {
        
        let steps = if let Some(plan) = &self.state.plan {
            &plan.steps
        } else {
            return Err(anyhow::anyhow!("Plan must be initialized"));
        };
        
        let prompt = format!(
            r#"    Step {step_index}: {step_title}
            \\n\\n
            {step_details}
            \\n\\n
            Instruction for {agent_name}: {instruction}
            "#,
            step_index = self.state.current_step_idx + 1,
            step_title = steps[self.state.current_step_idx].title,
            step_details = steps[self.state.current_step_idx].details,
            agent_name = agent_name,
            instruction = instruction,
        );

        Ok(prompt)
    }

    pub fn get_task_ledger_plan_prompt(&self, team: String) -> Result<String> {
        let base_prompt = format!(
            r#"
            You have access to the following team members that can help you address the request each with unique expertise:
            {team}
            Remember, there is no requirement to involve all team members -- a team member's particular expertise may not be needed for this task.
            When you answer without a plan and your answer includes factual information, make sure to say whether the answer was found using online search or from your own internal knowledge.
            Your plan should be a sequence of steps that will complete the task."#,
            team = team,
        );

        let step_types_section = r#"
            Each step should have a title, details and agent_name fields.

            The title should be a short one sentence description of the step.

            The details should be a detailed description of the step. The details should be concise and directly describe the action to be taken.
            The details should start with a brief recap of the title in one short sentence. We then follow it with a new line. We then add any additional details without repeating information from the title. We should be concise but mention all crucial details to allow the human to verify the step.
            The details should not be longer that 2 sentences.

            The agent_name should be the name of the agent that will execute the step. The agent_name should be one of the team members listed above.

            Output an answer in pure JSON format according to the following schema. The JSON object must be parsable as-is. DO NOT OUTPUT ANYTHING OTHER THAN JSON, AND DO NOT DEVIATE FROM THIS SCHEMA:

            The JSON object should have the following structure:

            {
                "response": "a complete response to the user request for Case 1.",
                "task": "a complete description of the task requested by the user",
                "plan_summary": "a complete summary of the plan if a plan is needed, otherwise an empty string",
                "needs_plan": boolean,
                "steps":
                [
                    {
                        "title": "title of step 1",
                        "details": "recap the title in one short sentence \n remaining details of step 1",
                        "agent_name": "the name of the agent that should complete the step"
                    },
                    {
                        "title": "title of step 2",
                        "details": "recap the title in one short sentence \n remaining details of step 2",
                        "agent_name": "the name of the agent that should complete the step"
                    },
                    ...
                ]
            }"#;

        Ok(format!("{}\n\n{}", base_prompt.trim(), step_types_section.trim()))
    }

    pub fn get_task_ledger_replan_prompt(&self, team: String,task: String, current_plan: String) -> Result<String> {
        let replan_intro = format!(r#"
            The task we are trying to complete is:
            {}
            The plan we have tried to complete is:
            {}
            We have not been able to make progress on our task.
            We need to find a new plan to tackle the task that addresses the failures in trying to complete the task previously."#,
            task, current_plan
        );

        let base_plan_prompt = self.get_task_ledger_plan_prompt(team)?;
        Ok(format!("{}\n\n{}", replan_intro, base_plan_prompt))
    }

}
