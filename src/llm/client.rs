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
    
    pub async fn create_completion(&self, messages: Vec<LlmMessage>, response_format: Option<String>) -> Result<LlmResponse> {
        let mut request_body = json!({
            "model": "qwen-turbo",
            "input": {
                "messages": messages.iter().map(|msg| {
                    json!({
                        "role": msg.role,
                        "content": msg.content
                    })
                }).collect::<Vec<_>>()
            },
            "parameters": {
                "temperature": 0.7,
                "max_tokens": 2000
            }
        });

        // 如果指定了 response_format，添加到请求中
        if let Some(format) = response_format {
            if format == "json_object" {
                request_body["parameters"]["result_format"] = json!("message");
            }
        }

        let response = self.client
            .post(&format!("{}/services/aigc/text-generation/generation", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: Value = response.json().await?;
        
        // 根据 DashScope API 的实际响应格式解析
        let content = response_json["output"]["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format: {:?}", response_json))?;

        // 构造符合 LlmResponse 格式的响应
        let llm_response = LlmResponse {
            choices: vec![Choice {
                message: LlmMessage {
                    role: "assistant".to_string(),
                    content: content.to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
        };

        Ok(llm_response)
    }
}