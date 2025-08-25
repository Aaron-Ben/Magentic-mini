use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

pub struct LlmClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl LlmClient {
    pub fn new() -> Result<Self> {
        let api_key = env::var("DASHSCOPE_API_KEY")
            .map_err(|_| anyhow::anyhow!("DASHSCOPE_API_KEY not found"))?;
        
        Ok(Self {
            client: Client::new(),
            api_key,
            base_url: "https://dashscope.aliyuncs.com/api/v1".to_string(),
        })
    }

    pub async fn generate_plan(&self, user_input: &str) -> Result<String> {
        let prompt = format!(
            r#"你是一个任务规划助手，负责将用户请求拆解为可执行的步骤。

可用的代理类型：
- WebSurfer: 可以浏览网站、搜索信息、填写表单
- Coder: 可以编写和执行各种编程语言的代码

用户请求："{}"

请以JSON格式回复，包含以下结构：
{{
    "task": "任务描述",
    "steps": [
        {{
            "title": "步骤标题",
            "details": "详细描述",
            "agent_type": "WebSurfer" 或 "Coder"
        }}
    ]
}}

请保持步骤简单且可执行。"#,
            user_input
        );

        let response = self.client
            .post(&format!("{}/services/aigc/text-generation/generation", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": "qwen-turbo",
                "input": {
                    "messages": [
                        {
                            "role": "user",
                            "content": prompt
                        }
                    ]
                },
                "parameters": {
                    "temperature": 0.7,
                    "max_tokens": 1500
                }
            }))
            .send()
            .await?;

        let response_json: Value = response.json().await?;
        
        let content = response_json["output"]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

        Ok(content.to_string())
    }

    pub async fn execute_step(&self, step_details: &str, agent_type: &str) -> Result<String> {
        let prompt = format!(
            "你是一个{}代理。请执行这个步骤：{}\n\n请回复执行结果或你将采取的行动。",
            agent_type, step_details
        );

        let response = self.client
            .post(&format!("{}/services/aigc/text-generation/generation", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": "qwen-turbo",
                "input": {
                    "messages": [
                        {
                            "role": "user",
                            "content": prompt
                        }
                    ]
                },
                "parameters": {
                    "temperature": 0.7,
                    "max_tokens": 1000
                }
            }))
            .send()
            .await?;

        let response_json: Value = response.json().await?;
        
        let content = response_json["output"]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

        Ok(content.to_string())
    }
}