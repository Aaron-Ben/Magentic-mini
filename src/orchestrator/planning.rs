use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::types::message::{LLMMessage};
use crate::types::plan::{Plan, PlanStep};
use crate::llm::client::{LlmClient};

/// 代表一个可用的 Agent
#[derive(Debug, Clone)]
pub struct Agent {
    pub name: String,
    pub description: String,
}

/// 计划生成器 (Planner)
pub struct Planner {
    agents: Vec<Agent>,
    llm_client: LlmClient,
}

impl Planner {
    /// 创建一个新的 Planner 实例
    pub fn new(agents: Vec<Agent>) -> Result<Self> {
        let llm_client = LlmClient::new()?;
        Ok(Self { agents, llm_client })
    }

    /// 根据用户任务生成计划
    pub async fn generate_plan(&self, user_task: &str) -> Result<Plan> {
        // 步骤 a: 构建 Prompts
        let team_description = self.get_team_description();
        let system_prompt = self.build_system_prompt(&team_description);
        let user_prompt = self.build_user_prompt(&team_description, user_task);

        println!("--- System Prompt (sent to LLM) ---\n{}\n", system_prompt);
        println!("--- User Prompt (sent to LLM) ---\n{}\n", user_prompt);
        
        // 步骤 b: 调用 LLM
        let messages = vec![
            LLMMessage::System(crate::types::message::SystemMessage {
                content: system_prompt,
            }),
            LLMMessage::User(crate::types::message::UserMessage {
                content: vec![crate::types::message::MessageContent::Text(user_prompt)],
            }),
        ];

        let llm_response = self.llm_client
            .create(messages, Some(true), None, None)
            .await?;
        
        // 步骤 c: 解析 LLM 的响应
        let content_text = match llm_response.content {
            crate::llm::client::Content::Text(text) => {
                println!("--- LLM JSON Response ---\n{}\n", text);
                text
            },
        };

        let parsed_response: LlmPlanResponse = serde_json::from_str(&content_text)?;

        // 步骤 d: 将解析后的响应转换为最终的 Plan 对象
        let plan = Plan {
            task: Some(parsed_response.task),
            steps: parsed_response.steps,
        };

        Ok(plan)
    }

    /// 构建团队描述字符串
    fn get_team_description(&self) -> String {
        self.agents
            .iter()
            .map(|agent| format!("{}: {}", agent.name, agent.description))
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// 构建系统提示
    fn build_system_prompt(&self, team_description: &str) -> String {
        format!(
            r#"
You are a helpful AI assistant named Magentic-UI. Your goal is to help the user with their request.
You are a planner, and your task is to devise a plan to address the user's request.

You have access to the following team members that can help you:
{}

Your plan should be a sequence of steps. You must output a JSON object, and nothing else.
"#,
            team_description
        )
    }

    /// 构建请求计划的用户提示
    fn build_user_prompt(&self, team_description: &str, user_task: &str) -> String {
        format!(
            r#"
Please create a plan for the task: "{}"

Your response must be a single JSON object that adheres to the following schema. Do not add any text before or after the JSON object.

Team available:
{}

JSON Schema:
{{
    "response": "a complete response to the user request if no plan is needed.",
    "task": "a complete description of the task requested by the user",
    "plan_summary": "a complete summary of the plan if a plan is needed, otherwise an empty string",
    "needs_plan": true,
    "steps": [
        {{
            "title": "title of step 1",
            "details": "details of step 1",
            "agent_name": "the name of the agent that should complete the step"
        }},
        {{
            "title": "title of step 2",
            "details": "details of step 2",
            "agent_name": "the name of the agent that should complete the step"
        }}
    ]
}}
"#,
            user_task, team_description
        )
    }
}

/// 这个结构体用来匹配 LLM 返回的完整 JSON 对象
#[derive(Serialize, Deserialize, Debug)]
struct LlmPlanResponse {
    task: String,
    plan_summary: String,
    needs_plan: bool,
    response: String,
    steps: Vec<PlanStep>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_planner_generation() {
        // 初始化我们的 Agents
        let agents = vec![
            Agent {
                name: "web_surfer".to_string(),
                description: "An agent that can browse the web to find information.".to_string(),
            },
            Agent {
                name: "coder_agent".to_string(),
                description: "An agent that can write and execute code in a sandboxed environment.".to_string(),
            },
        ];

        // 创建 Planner
        let planner = Planner::new(agents).expect("Failed to create planner");

        // 定义用户的输入任务
        let user_task = "Execute the starter code for the autogen repo";
        println!("--- User Task ---\n{}\n", user_task);

        // 生成计划
        match planner.generate_plan(user_task).await {
            Ok(plan) => {
                println!("--- Plan Generation Successful! ---");
                println!("{:#?}", plan); 
            }
            Err(e) => {
                eprintln!("Error generating plan: {}", e);
            }
        }
    }
}