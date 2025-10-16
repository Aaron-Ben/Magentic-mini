use anyhow::{anyhow, Result};
use async_openai::{
    types::{
        ChatCompletionRequestMessage, 
        ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent,
        CreateChatCompletionRequest,
        ImageUrl,
        ImageDetail,
    },
    config::OpenAIConfig,
    Client,
};
use base64::{Engine as _, engine::general_purpose};

use crate::{
    orchestrator::message::{
        FunctionCall, LLMMessage, UserContent, MultiModalContent, AssistantContent
    }, 
    tools::tool_metadata::ToolSchema
};

/// LLM 响应类型
#[derive(Debug, Clone)]
pub enum LLMResponse {
    Text(String),
    FunctionCalls(Vec<FunctionCall>),
    Error(String),
}

/// 调用 LLM API（支持 OpenAI 和阿里云 DashScope）
pub async fn call_llm(
    history: &[LLMMessage], 
    tools: &[ToolSchema]
) -> Result<Vec<LLMResponse>> {
    // 1. 初始化客户端（支持 OpenAI 和阿里云 DashScope）
    // 优先使用阿里云 DashScope，其次是 OpenAI
    let (api_key, base_url, model) = if let Ok(dashscope_key) = std::env::var("DASHSCOPE_API_KEY") {
        // 使用阿里云 DashScope
        let url = std::env::var("DASHSCOPE_BASE_URL")
            .unwrap_or_else(|_| "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string());
        let model_name = std::env::var("DASHSCOPE_MODEL")
            .unwrap_or_else(|_| "qwen-vl-max".to_string());
        (dashscope_key, url, model_name)
    } else {
        return Err(anyhow!("Neither DASHSCOPE_API_KEY nor OPENAI_API_KEY is set"));
    };
    
    let config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(base_url);
    
    let client = Client::with_config(config);
    
    // 2. 转换消息为 API 格式
    let mut api_messages: Vec<ChatCompletionRequestMessage> = Vec::new();
    
    for msg in history {
        match msg {
            LLMMessage::SystemMessage(sys_msg) => {
                api_messages.push(ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessage {
                        content: sys_msg.content.clone(),
                        name: None,
                    }
                ));
            }
            LLMMessage::UserMessage(user_msg) => {
                let mut content_parts = Vec::new();
                
                match &user_msg.content {
                    UserContent::String(text) => {
                        // 纯文本消息
                        content_parts.push(
                            async_openai::types::ChatCompletionRequestMessageContentPart::Text(
                                async_openai::types::ChatCompletionRequestMessageContentPartText {
                                    text: text.clone(),
                                }
                            )
                        );
                    }
                    UserContent::MultiModal(contents) => {
                        // 多模态消息
                        for content in contents {
                            match content {
                                MultiModalContent::String(text) => {
                                    content_parts.push(
                                        async_openai::types::ChatCompletionRequestMessageContentPart::Text(
                                            async_openai::types::ChatCompletionRequestMessageContentPartText {
                                                text: text.clone(),
                                            }
                                        )
                                    );
                                }
                                MultiModalContent::Image(img_bytes) => {
                                    // 将图片转换为 base64
                                    let base64_img = general_purpose::STANDARD.encode(img_bytes);
                                    let data_url = format!("data:image/png;base64,{}", base64_img);
                                    
                                    content_parts.push(
                                        async_openai::types::ChatCompletionRequestMessageContentPart::ImageUrl(
                                            async_openai::types::ChatCompletionRequestMessageContentPartImage {
                                                image_url: ImageUrl {
                                                    url: data_url,
                                                    detail: Some(ImageDetail::Auto),
                                                }
                                            }
                                        )
                                    );
                                }
                            }
                        }
                    }
                }
                
                api_messages.push(ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Array(content_parts),
                        name: None,
                    }
                ));
            }
            LLMMessage::AssistantMessage(asst_msg) => {
                let content_str = match &asst_msg.content {
                    AssistantContent::String(s) => Some(s.clone()),
                    AssistantContent::FunctionCalls(_) => None,
                };
                
                #[allow(deprecated)]
                api_messages.push(ChatCompletionRequestMessage::Assistant(
                    async_openai::types::ChatCompletionRequestAssistantMessage {
                        content: content_str,
                        name: None,
                        tool_calls: None,
                        function_call: None,
                    }
                ));
            }
            LLMMessage::FunctionExecutionResultMessage(_func_result) => {
                // TODO: 处理函数执行结果消息
                // 暂时跳过
            }
        }
    }
    
    // 3. 转换工具为 API 格式
    let api_tools: Vec<async_openai::types::ChatCompletionTool> = tools
        .iter()
        .map(|tool| {
            // 将 ParametersSchema 转换为 JSON Value
            let parameters_json = serde_json::to_value(&tool.parameters)
                .unwrap_or(serde_json::json!({}));
            
            async_openai::types::ChatCompletionTool {
                r#type: async_openai::types::ChatCompletionToolType::Function,
                function: async_openai::types::FunctionObject {
                    name: tool.name.clone(),
                    description: Some(tool.description.clone()),
                    parameters: Some(parameters_json),
                },
            }
        })
        .collect();
    
    // 4. 创建请求
    let request = CreateChatCompletionRequest {
        model,  // 使用动态选择的模型
        messages: api_messages,
        tools: Some(api_tools),
        temperature: Some(0.7),
        ..Default::default()
    };
    
    // 5. 调用 API
    let response = client.chat().create(request).await
        .map_err(|e| anyhow!("OpenAI API error: {}", e))?;
    
    // 6. 解析响应
    if let Some(choice) = response.choices.first() {
        // 检查是否有工具调用
        if let Some(tool_calls) = &choice.message.tool_calls {
            let function_calls: Vec<FunctionCall> = tool_calls
                .iter()
                .map(|tc| FunctionCall {
                    id: tc.id.clone(),
                    name: tc.function.name.clone(),
                    arguments: tc.function.arguments.clone(),
                })
                .collect();
            return Ok(vec![LLMResponse::FunctionCalls(function_calls)]);
        }
        
        // 否则返回文本响应
        if let Some(content) = &choice.message.content {
            return Ok(vec![LLMResponse::Text(content.clone())]);
        }
    }
    
    // 如果没有有效响应
    Err(anyhow!("No valid response from LLM"))
}
