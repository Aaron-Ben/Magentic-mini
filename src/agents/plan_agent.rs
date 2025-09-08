use crate::llm::LlmClient;
use crate::types::*;
use super::plan_agent_utils::prompt::*;
use anyhow::{Result, anyhow};
use serde_json::Value;
use colored::*;

pub struct PlanAgent {
    llm_client: LlmClient,
    sentinel_tasks_enabled: bool,  // 新增字段
}

impl PlanAgent {
    pub fn new(llm_client: LlmClient) -> Self {
        Self { 
            llm_client,
            sentinel_tasks_enabled: false,  // 默认关闭
        }
    }

    pub fn with_sentinel_tasks(llm_client: LlmClient, sentinel_tasks_enabled: bool) -> Self {
        Self {
            llm_client,
            sentinel_tasks_enabled,
        }
    }

    /// 从用户输入生成计划
    pub async fn generate_plan_from_input(&self, user_input: &str) -> Result<Plan> {
        println!("Generating plan for: {}", user_input);
        
        // 创建用户消息
        let user_message = create_user_message(user_input.to_string(), "user".to_string());
        
        self.generate_plan_from_messages(vec![user_message]).await
    }

    /// 从消息生成计划
    pub async fn generate_plan_from_messages(&self, user_messages: Vec<ChatMessage>) -> Result<Plan> {
        // 1. 创建系统消息
        let system_message = create_system_message(self.sentinel_tasks_enabled);
        
        // 2. 创建计划指令消息（来自 orchestrator）
        let plan_instruction = create_plan_instruction_message(self.sentinel_tasks_enabled);
        
        // 3. 构建最终的消息列表：SystemMessage + UserMessage(用户输入) + UserMessage(计划指令)
        let mut final_messages = vec![system_message];
        final_messages.extend(user_messages);
        final_messages.push(plan_instruction);
        
        // 4. 转换为 LLM 格式
        let llm_messages: Vec<LlmMessage> = final_messages.into_iter().map(|msg| {
            let content = msg.get_text().unwrap_or_default();
            let role = match msg.role {
                Role::System => "system".to_string(),
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
                Role::Tool => "tool".to_string(),
            };
            LlmMessage { role, content }
        }).collect();
        
        // 5. 调用 LLM
        let response = self.llm_client.create_completion(llm_messages, Some("json_object".to_string())).await?;
        
        let plan_content = response.choices.into_iter().next()
            .and_then(|c| Some(c.message.content))
            .ok_or_else(|| anyhow!("LLM did not return plan content"))?;
        
        // 6. 解析和验证 JSON 响应
        self.get_json_response(&plan_content).await
    }

    /// 获取并验证 JSON 响应
    async fn get_json_response(&self, json_content: &str) -> Result<Plan> {
        // 1. 解析 JSON
        let json_response: Value = serde_json::from_str(json_content)
            .map_err(|e| anyhow!("Failed to parse JSON response: {}", e))?;
        
        // 2. 验证 JSON 结构
        if !validate_plan_json(&json_response, self.sentinel_tasks_enabled) {
            return Err(anyhow!("Invalid plan JSON structure"));
        }
        
        // 3. 检查 needs_plan 字段
        let needs_plan = json_response["needs_plan"].as_bool().unwrap_or(true);
        
        if !needs_plan {
            // 不需要计划，直接返回响应
            let response = json_response["response"].as_str().unwrap_or("无需计划。");
            println!("{}", response.bright_cyan());
            return Err(anyhow!("No plan needed: {}", response));
        }
        
        // 4. 将 JSON 中的 steps 转化为 Plan 对象
        let plan = Plan::from_json(&json_response)?;
        
        // 5. 显示计划给用户
        println!("{}", "\n=== 生成的计划 ===".bright_green().bold());
        println!("{}", plan.to_display_string());
        
        Ok(plan)
    }

    /// 重新规划
    pub async fn replan(&self, original_input: &str) -> Result<Plan> {
        println!("{}", "正在重新规划...".bright_cyan().bold());
        
        // 重新生成计划，可以加入一些优化逻辑
        let new_plan = self.generate_plan_from_input(original_input).await?;
        
        println!("{}", "重新规划完成！".bright_cyan());
        Ok(new_plan)
    }

    /// 增加步骤
    pub async fn add_steps(&self, _current_plan: &Plan, _additional_requirements: &str) -> Result<Plan> {
        println!("{}", "正在增加步骤...".bright_blue().bold());
        println!("{}", "功能未实现 - 需要实现基于LLM的步骤增加逻辑".yellow());
        
        // TODO: 实现步骤增加逻辑
        // 1. 分析当前计划
        // 2. 理解额外需求
        // 3. 生成新的步骤
        // 4. 合并到现有计划中
        
        Err(anyhow!("功能未实现"))
    }

    /// 修改计划
    pub async fn modify_plan(&self, _current_plan: &Plan, _modification_request: &str) -> Result<Plan> {
        println!("{}", "正在修改计划...".bright_magenta().bold());
        println!("{}", "功能未实现 - 需要实现基于LLM的计划修改逻辑".yellow());
        
        // TODO: 实现计划修改逻辑
        // 1. 分析修改请求
        // 2. 识别需要修改的步骤
        // 3. 生成修改后的步骤
        // 4. 更新计划
        
        Err(anyhow!("功能未实现"))
    }
}