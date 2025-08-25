use crate::llm::LlmClient;
use crate::types::*;
use anyhow::Result;
use serde_json::Value;

pub struct Orchestrator {
    llm_client: LlmClient,
}

impl Orchestrator {
    pub fn new(llm_client: LlmClient) -> Self {
        Self { llm_client }
    }

    pub async fn generate_plan(&self, user_input: &str) -> Result<Plan> {
        println!("🤖 Generating plan for: {}", user_input);
        
        let response = self.llm_client.generate_plan(user_input).await?;
        
        // 解析 JSON 响应
        let cleaned_response = self.extract_json_from_response(&response)?;
        let plan_data: Value = serde_json::from_str(&cleaned_response)?;
        
        let task = plan_data["task"]
            .as_str()
            .unwrap_or(user_input)
            .to_string();
        
        let mut steps = Vec::new();
        
        if let Some(steps_array) = plan_data["steps"].as_array() {
            for step_data in steps_array {
                let title = step_data["title"].as_str().unwrap_or("Untitled").to_string();
                let details = step_data["details"].as_str().unwrap_or("").to_string();
                let agent_type_str = step_data["agent_type"].as_str().unwrap_or("WebSurfer");
                
                let agent_type = match agent_type_str {
                    "Coder" => AgentType::Coder,
                    _ => AgentType::WebSurfer,
                };
                
                steps.push(PlanStep::new(title, details, agent_type));
            }
        }
        
        Ok(Plan::new(task, steps))
    }

    pub async fn execute_plan(&self, mut plan: Plan) -> Result<()> {
        println!("🚀 Executing plan: {}", plan.task);
        println!();

        for (i, step) in plan.steps.iter_mut().enumerate() {
            println!("📋 Step {}: {}", i + 1, step.title);
            println!("   Agent: {:?}", step.agent_type);
            println!("   Details: {}", step.details);
            
            step.status = StepStatus::InProgress;
            
            let agent_type_str = match step.agent_type {
                AgentType::WebSurfer => "WebSurfer",
                AgentType::Coder => "Coder",
            };
            
            match self.llm_client.execute_step(&step.details, agent_type_str).await {
                Ok(result) => {
                    println!("   ✅ Result: {}", result);
                    step.status = StepStatus::Completed;
                }
                Err(e) => {
                    println!("   ❌ Error: {}", e);
                    step.status = StepStatus::Failed;
                }
            }
            
            println!();
            
            // 添加延迟，让用户看到进度
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        
        println!("🎉 Plan execution completed!");
        Ok(())
    }

    fn extract_json_from_response(&self, response: &str) -> Result<String> {
        // 尝试提取 JSON 部分
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                if end >= start {
                    return Ok(response[start..=end].to_string());
                }
            }
        }
        
        // 如果没找到 JSON，返回原始响应
        Ok(response.to_string())
    }
}