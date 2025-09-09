use crate::llm::LlmClient;
use super::types::*;
use super::config::OrchestratorConfig;
use super::messages::*;
use super::prompt::*;
use anyhow::{Result, anyhow};
use serde_json::Value;

#[derive(Debug)]
pub struct OrchestratorState {
    pub task: String,
    pub plan: Option<Plan>,
    pub current_step_idx: usize,
    pub message_history: Vec<ChatMessage>,
    pub n_rounds: usize,
    pub n_replans: usize,
    pub is_paused: bool,
}

impl Default for OrchestratorState {
    fn default() -> Self {
        Self {
            task: String::new(),
            plan: None,
            current_step_idx: 0,
            message_history: Vec::new(),
            n_rounds: 0,
            n_replans: 0,
            is_paused: false,
        }
    }
}

pub struct PlanAgent {
    llm_client: LlmClient,
    config: OrchestratorConfig,
    state: OrchestratorState,
    team_description: String,
}

impl PlanAgent {
    pub fn new(llm_client: LlmClient, config: OrchestratorConfig) -> Self {
        let mut agent = Self {
            llm_client,
            config,
            state: OrchestratorState::default(),
            team_description: String::new(),
        };
        
        // 设置默认的团队描述
        agent.set_default_team_description();
        agent
    }

    fn set_default_team_description(&mut self) {
        self.team_description = r#"
**web_surfer**: 一个网络浏览代理，可以搜索互联网、访问网站、查找餐厅、检查价格、进行在线购买、从网页收集信息、查找外卖平台和餐厅信息。对于买吃的请求，可以直接行动而无需多余的授权或澄清。

**coder_agent**: 一个编程代理，可以编写和执行代码、创建文件、处理数据和执行计算任务。
        "#.trim().to_string();
    }

    pub fn get_current_plan(&self) -> Option<&Plan> {
        self.state.plan.as_ref()
    }

    /// 获取可用的代理列表
    pub fn get_available_agents(&self) -> Vec<String> {
        vec![
            "web_surfer".to_string(),
            "coder_agent".to_string(),
        ]
    }

    /// 重置代理状态，准备处理新的对话
    pub fn reset_state(&mut self) {
        self.state = OrchestratorState::default();
    }

    /// 修改计划步骤
    pub fn modify_plan_step(
        &mut self,
        step_index: usize,
        new_title: Option<String>,
        new_details: Option<String>,
        new_agent_name: Option<String>,
    ) -> Result<()> {
        let plan = self.state.plan.as_mut()
            .ok_or_else(|| anyhow!("No plan available to modify"))?;
        
        if step_index >= plan.steps.len() {
            return Err(anyhow!("Step index {} out of bounds", step_index));
        }
        
        let step = &mut plan.steps[step_index];
        
        if let Some(title) = new_title {
            step.title = title;
        }
        if let Some(details) = new_details {
            step.details = details;
        }
        if let Some(agent_name) = new_agent_name {
            step.agent_name = agent_name;
        }
        
        Ok(())
    }

    /// 添加计划步骤
    pub fn add_plan_step(
        &mut self,
        title: String,
        details: String,
        agent_name: String,
        position: Option<usize>,
    ) -> Result<()> {
        let plan = self.state.plan.as_mut()
            .ok_or_else(|| anyhow!("No plan available to add step to"))?;
        
        let new_step = PlanStep {
            title,
            details,
            agent_name,
            step_type: None,
            condition: None,
            sleep_duration: None,
        };
        
        match position {
            Some(pos) => {
                if pos > plan.steps.len() {
                    return Err(anyhow!("Position {} out of bounds", pos));
                }
                plan.steps.insert(pos, new_step);
            }
            None => plan.steps.push(new_step),
        }
        
        Ok(())
    }

    /// 删除计划步骤
    pub fn remove_plan_step(&mut self, step_index: usize) -> Result<()> {
        let plan = self.state.plan.as_mut()
            .ok_or_else(|| anyhow!("No plan available to remove step from"))?;
        
        if step_index >= plan.steps.len() {
            return Err(anyhow!("Step index {} out of bounds", step_index));
        }
        
        if plan.steps.len() <= 1 {
            return Err(anyhow!("Cannot remove the last remaining step"));
        }
        
        plan.steps.remove(step_index);
        
        // 如果当前步骤索引超出了范围，调整它
        if self.state.current_step_idx >= plan.steps.len() {
            self.state.current_step_idx = plan.steps.len().saturating_sub(1);
        }
        
        Ok(())
    }

    pub async fn handle_user_input(&mut self, user_input: &str) -> Result<()> {
        // 创建用户消息
        let user_message = create_user_message(user_input.to_string(), "user".to_string());
        
        // 添加到历史记录
        self.state.message_history.push(user_message);
        
        // 开始编排过程
        self.orchestrate().await
    }

    /// 编排过程
    async fn orchestrate(&mut self) -> Result<()> {
        if self.state.plan.is_none() {
            // 规划阶段
            self.orchestrate_planning().await?;
        } else {
            // 执行阶段
            self.orchestrate_execution().await?;
        }
        Ok(())
    }

    /// 规划阶段
    async fn orchestrate_planning(&mut self) -> Result<()> {
        let last_message = self.state.message_history.last()
            .ok_or_else(|| anyhow!("No messages in history"))?;
        
        let user_content = last_message.content();
        
        // 设置任务
        self.state.task = format!("TASK: {}", user_content);
        
        // 创建系统消息
        let system_prompt = system_message_planning(self.config.sentinel_tasks)
            .replace("{team}", &self.team_description);
        let system_message = create_system_message(system_prompt);
        
        // 创建计划指令消息
        let plan_prompt = plan_prompt_json(self.config.sentinel_tasks)
            .replace("{team}", &self.team_description);
        let plan_instruction = create_orchestrator_message(plan_prompt);
        
        // 创建用户请求消息
        let user_request_message = create_user_message(
            user_content,
            "user".to_string()
        );
        
        // 构建消息列表
        let messages = vec![
            system_message,
            plan_instruction,
            user_request_message,
        ];
        
        // 调用LLM获取计划
        let plan_response = self.get_json_response(&messages).await?;
        
        // 验证并创建计划
        let plan_json_str = serde_json::to_string(&plan_response)
            .map_err(|e| anyhow!("Failed to serialize plan response: {}", e))?;
        let plan = Plan::from_json(&plan_json_str)?;
        self.state.plan = Some(plan);
            
        println!("Plan created successfully");
        Ok(())
    }

    /// 执行阶段
    async fn orchestrate_execution(&mut self) -> Result<()> {
        let plan = self.state.plan.as_ref()
            .ok_or_else(|| anyhow!("No plan available"))?;
        
        if self.state.current_step_idx >= plan.steps.len() {
            println!("Plan execution completed");
            return Ok(());
        }
        
        let current_step = &plan.steps[self.state.current_step_idx];
        
        // 创建代理指令消息
        let instruction_message = create_agent_instruction_message(
            self.state.current_step_idx + 1,
            current_step.title.clone(),
            current_step.details.clone(),
            current_step.agent_name.clone(),
            "Execute this step".to_string(), // 这里可以从progress ledger获取
        );
        
        // 添加到历史记录
        self.state.message_history.push(instruction_message);
        
        // 移动到下一步
        self.state.current_step_idx += 1;
        self.state.n_rounds += 1;
        
        Ok(())
    }

    /// 重新规划方法 - 使用现有的计划和任务信息生成新的计划
    pub async fn replan(&mut self) -> Result<()> {
        // 检查是否有当前任务和计划
        let current_plan = self.state.plan.as_ref()
            .ok_or_else(|| anyhow!("No current plan available for replanning"))?;
        
        if self.state.task.is_empty() {
            return Err(anyhow!("No current task available for replanning"));
        }
        
        // 增加重新规划计数
        self.state.n_replans += 1;
        
        // 创建重新规划的系统消息（包含中文输出指令）
        let replan_prompt = replan_prompt_json(self.config.sentinel_tasks)
            .replace("{team}", &self.team_description)
            .replace("{task}", &self.state.task)
            .replace("{plan}", &current_plan.to_json().unwrap_or_else(|_| "无法序列化当前计划".to_string()));
        
        let system_message = create_system_message(format!(
            "{}\n\n请用中文返回响应。请重新制定计划以解决之前计划执行中遇到的问题。", 
            replan_prompt
        ));
        
        // 构建消息列表
        let messages = vec![system_message];
        
        println!("=== 开始重新规划 ===");
        println!("重新规划次数: {}", self.state.n_replans);
        
        // 调用LLM获取新计划
        let plan_response = self.get_json_response(&messages).await?;
        
        // 验证并创建新计划
        let plan_json_str = serde_json::to_string(&plan_response)
            .map_err(|e| anyhow!("Failed to serialize replan response: {}", e))?;
        let new_plan = Plan::from_json(&plan_json_str)?;
        
        // 更新状态
        self.state.plan = Some(new_plan);
        self.state.current_step_idx = 0; // 重置步骤索引
        
        println!("=== 重新规划完成 ===");
        Ok(())
    }

    /// 获取JSON响应
    async fn get_json_response(&self, messages: &[ChatMessage]) -> Result<Value> {
        // 转换为LLM格式
        let llm_messages: Vec<LlmMessage> = messages.iter().map(|msg| {
            LlmMessage {
                role: match msg {
                    ChatMessage::Text(_) => "user".to_string(),
                    ChatMessage::MultiModal(_) => "user".to_string(),
                    ChatMessage::Stop(_) => "assistant".to_string(),
                },
                content: msg.content(),
            }
        }).collect();
        
        println!("\n=== 发送给LLM的消息 ===");
        for (i, msg) in llm_messages.iter().enumerate() {
            println!("消息 {}: [{}] {}", i + 1, msg.role, msg.content);
            println!("{}", "-".repeat(50));
        }
        
        // 调用LLM
        let response = self.llm_client.create_completion(
            llm_messages, 
            Some("json_object".to_string())
        ).await?;
        
        // 解析JSON
        let content = response.choices.first()
            .and_then(|c| Some(c.message.content.clone()))
            .ok_or_else(|| anyhow!("No content in LLM response"))?;
        
        println!("\n=== LLM返回的原始内容 ===");
        println!("{}", content);
        println!("{}", "=".repeat(50));
        
        let json_result = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse JSON: {}", e));
            
        if let Ok(ref json_value) = json_result {
            println!("\n=== 解析后的JSON结构 ===");
            println!("{}", serde_json::to_string_pretty(json_value).unwrap_or_else(|_| "Failed to pretty print".to_string()));
            println!("{}", "=".repeat(50));
        }
        
        json_result
    }
}