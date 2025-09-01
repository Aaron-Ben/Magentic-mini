use crate::llm::LlmClient;
use crate::types::*;
use anyhow::{Result, anyhow};
use serde_json::Value;

pub struct PlanAgent {
    llm_client: LlmClient,
}

impl PlanAgent {
    pub fn new(llm_client: LlmClient) -> Self {
        Self { llm_client }
    }

    /// 从用户输入生成计划
    pub async fn generate_plan_from_input(&self, user_input: &str) -> Result<Plan> {
        println!("Generating plan for: {}", user_input);
        
        // 将用户输入转换为 ChatMessage 格式
        let chat_messages = vec![ChatMessage::TextMessage {
            content: user_input.to_string(),
            source: "user".to_string(),
            timestamp: None,
        }];
        
        self.generate_plan_from_messages(chat_messages).await
    }

    /// 从消息生成计划
    pub async fn generate_plan_from_messages(&self, chat_messages: Vec<ChatMessage>) -> Result<Plan> {
        // 1. 转换 chat_history 为 LLMMessage 格式
        let llm_messages: Vec<LlmMessage> = chat_messages.into_iter().map(|msg| {
            match msg {
                ChatMessage::TextMessage { content, source, .. } => LlmMessage {
                    role: if source == "user" { "user".to_string() } else { "assistant".to_string() },
                    content,
                },
                ChatMessage::MultiModalMessage { text_content, source, .. } => LlmMessage {
                    role: if source == "user" { "user".to_string() } else { "assistant".to_string() },
                    content: text_content, // 简化处理，只取文本内容
                },
            }
        }).collect();

        // 2. 构建 instruction_message
        let instruction_content = self.build_plan_instruction();

        let instruction_message = LlmMessage {
            role: "user".to_string(),
            content: instruction_content,
        };

        let mut final_messages = vec![instruction_message];
        final_messages.extend(llm_messages);

        // 3. LLM 调用
        let response = self.llm_client.create_completion(final_messages, Some("json_object".to_string())).await?;

        let plan_content = response.choices.into_iter().next()
            .and_then(|c| Some(c.message.content))
            .ok_or_else(|| anyhow!("LLM did not return plan content"))?;

        // 4. 解析结果
        self.parse_plan_response(&plan_content)
    }

    /// 构建计划生成的指令提示
    fn build_plan_instruction(&self) -> String {
        r###"
The above messages are a conversation between a user and an AI assistant.
The AI assistant helped the user with their task and arrived potentially at a "Final Answer" to accomplish their task.

We want to be able to learn a plan from the conversation that can be used to accomplish the task as efficiently as possible.
This plan should help us accomplish this task and tasks similar to it more efficiently in the future as we learned from the mistakes and successes of the AI assistant and the details of the conversation.

Guidelines:
- We want the most efficient and direct plan to accomplish the task. The less number of steps, the better. Some agents can perform multiple steps in one go.
- We don't need to repeat the exact sequence of the conversation, but rather we need to focus on how to get to the final answer most efficiently without directly giving the final answer.
- Include details about the actions performed, buttons clicked, urls visited if they are useful.
For instance, if the plan was trying to find the github stars of autogen and arrived at the link https://github.com/microsoft/autogen then mention that link.
Or if the web surfer clicked a specific button to create an issue, mention that button.

Here is an example of a plan that the AI assistant might follow:

Example:

User request: "On which social media platform does Autogen have the most followers?"

Step 1:
- title: "Find all social media platforms that Autogen is on"
- details: "1) do a search for autogen social media platforms using Bing, 2) find the official link for autogen where the social media platforms might be listed, 3) report back all the social media platforms that Autogen is on"
- agent_type: "WebSurfer"

Step 2:
- title: "Find the number of followers on Twitter"
- details: "Go to the official link for autogen on the web and find the number of followers on Twitter"
- agent_type: "WebSurfer"

Step 3:
- title: "Find the number of followers on LinkedIn"
- details: "Go to the official link for autogen on the web and find the number of followers on LinkedIn"
- agent_type: "WebSurfer"

Please provide the plan from the conversation above in JSON format with the following structure:
{
    "task": "task description",
    "steps": [
        {
            "title": "step title",
            "details": "step details",
            "agent_type": "WebSurfer" or "Coder"
        }
    ]
}

Again, DO NOT memorize the final answer in the plan.
        "###.to_string()
    }

    /// 解析LLM返回的计划内容
    fn parse_plan_response(&self, plan_content: &str) -> Result<Plan> {
        let plan_data: Value = serde_json::from_str(plan_content)?;
        
        let task = plan_data["task"]
            .as_str()
            .unwrap_or("Generated task")
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
}