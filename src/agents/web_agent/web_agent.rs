use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::collections::HashSet;
use regex::Regex;
use chrono::Utc;
use anyhow::{anyhow, Result};
use serde_json::Value;
use serde_json::json;
use tldextract::{TldExtractor, TldOption};
use image::{imageops::FilterType};
use crate::agents::web_agent::prompt::WEB_SURFER_SYSTEM_MESSAGE;
use crate::agents::web_agent::set_of_mark::{PageState, add_set_of_mark};
use crate::agents::web_agent::tool_define::DefaultTools;
use crate::agents::web_agent::types::FunctionCall;
use crate::tools::chrome::chrome_ctrl::Chrome;
use crate::tools::chrome::types::InteractiveRegion;
use crate::tools::tool_metadata::ToolSchema;
use crate::tools::url_status_manager::{UrlStatus, UrlStatusManager};
use crate::types::message::{CancellationToken, LLMMessage, SystemMessage, TextMessage, UserMessage, MessageContent};

#[derive(Debug)]
pub enum LLMResponse {
    Text(String),
    FunctionCalls(Vec<FunctionCall>),
    Error(String),
}

#[async_trait::async_trait]
pub trait ActionGuard: Send + Sync + Debug{
    async fn get_approval(&self, request_msg: TextMessage) -> bool;
}

#[derive(Debug)]
pub struct WebAgent {
    chrome_ctrl: Option<Chrome>,
    chat_history: Option<Vec<LLMMessage>>,
    // chat_history: Option<Vec<LLMMessage>>,
    tools: Vec<ToolSchema>,

    url_status_manager: UrlStatusManager,
    last_rejected_url: Option<String>,
    action_guard: Option<Arc<dyn ActionGuard>>,     // 安全在多线程环境中使用，指出任意实现 ActionGuard trait 的类型
    name: String,
}

impl Default for WebAgent {

    fn default() -> Self {
        let default_tools = DefaultTools::new()
        .expect("Failed to load default tools");

        let tools = vec![
            default_tools.visit_url,
            default_tools.web_search,
            default_tools.history_back,
            default_tools.refresh_page,
            default_tools.page_up,
            default_tools.page_down,
            default_tools.scroll_down,
            default_tools.scroll_up,
            default_tools.click,
            default_tools.click_full,
            default_tools.input_text,
            default_tools.scroll_element_down,
            default_tools.scroll_element_up,
            default_tools.hover,
            default_tools.keypress,
            default_tools.answer_question,
            default_tools.summarize_page,
            default_tools.sleep,
            default_tools.stop_action,
            default_tools.select_option,
            default_tools.create_tab,
            default_tools.switch_tab,
            default_tools.close_tab,
            default_tools.upload_file,
        ];

        Self {
            chrome_ctrl: None,
            chat_history: Some(Vec::new()),
            tools,
            url_status_manager: UrlStatusManager::new(None, None),
            last_rejected_url: None,
            action_guard: None,
            name: "WebAgent".to_string(),
        }
    }

}

impl WebAgent {
    pub async fn new() -> Self {
        Self::default()
    }

    pub async fn initialize(&mut self) -> Result<()> {
        self.chrome_ctrl = Some(Chrome::new().await?);
        self.chat_history = Some(Vec::new());
        Ok(())
    }

    pub async fn chrome_mut(&mut self) -> Result<&mut Chrome> {
        self.chrome_ctrl.as_mut()
            .ok_or_else(|| anyhow!("Chrome context is not initialized. Call initialize() first."))
    }

    // web_agent的核心，接收用户或者，ent的消息，驱动浏览器进行一系列的操作，并将操作以流的形式（AsyncGenerator）逐步
    pub async fn _on_messages_steam(
        self,
    ) -> Result<()> {

        // 懒加载浏览器，确保浏览器已准备好


        // 如果是第一次加载浏览器，发送浏览器的地址？？？（可能是Docker中的）

        
        // 依据消息的类型，将消息添加到content中（多模态消息全部保留，文本消息只保留最后一条，为了避免历史消息进行影响）

        // 初始化一系列东西

        // 如果被暂停，直接返回提示的信息（The WebAgent is paused...）

        // 外部的取消令牌和内部的LLM进行联动，启动一个后台任务，agent被暂停时取消当前LLM操作

        // 从第0步到最大的步骤之间的执行（主循环）最重要的步骤！！！
            
            // 1）调用LLM，获取下一步要执行的动作，LLM返回的内容
                // response
                // 1）思考 or 总结 不执行工具
                // 2）Vec<FunctionCall> 工具调用列表（执行工具）
                // rect: 页面元素的坐标（用于进行标注）
                // tools: 工具列表
                // element_id_mapping: 映射页面元素ID
                // need_execute_tool: 是否需要执行工具

            // (final_usage：用于获取模型使用的token情况)    

            // 2）如果是不需要工具：（思考 or 总结） break
            
            // 3) 如果需要工具：解析工具名称名称和参数，对于answer_question的tools,需要添加一句prompt

                // 3.1) 审批机制 Action Guard， 三种机制，必须，无需，由 ActionGuard 

                // 3.2) 需要进行批准的话，动作的预览 是xxx动作，有一个预览的过程，高亮元素？【737-762】
                
                // 3.3) 获取用户的审批，如果拒绝，清除动画，中断当前的循环 【764-788】

                // 3.4) 执行工具，实际的动作 【789-808】

                // 3.5) 获取截图+页面描述 【809-848】 和3.6有一些“混在一起了”

                // 3.6）流式返回结果（动作结果+截图），同时记录到一系列的列表中。【821-862】

                // 3.7) 检查终止条件，f tool_call_name in non_action_tools 【863-865】
                    // 用户要求停止"stop_action", 
                    // 已读完内容，准备回答"answer_question" i: 
            
        // 异常处理：用户取消，返回友好的提示，其他错误，记录日志返回错误【868-895】
        
        // 清理工作：确保监控任务被取消，避免资源泄露

        // 生成最终的汇总：all_responses，获取最终的页面状态(截图和描述)，internal: yes 表面这是给其他的agent进行展示，不是直接给用户【897-950】

        Ok(())
    }
    

    /* 观察当前浏览器的状态，构造提示词，调用LLM，返回下一步要执行的动作（思考），以及上下文信息*/
    pub async fn get_llm_response(
        &self,
    ) -> Result<(
        LLMResponse,
        HashMap<String,InteractiveRegion>,
        Vec<ToolSchema>,
        HashMap<String,String>,
        bool,)>
    {

        // 1. 确保页面可用性
        self.chrome_ctrl.as_ref().unwrap().wait_for_page_ready().await?;

        // 2. 准备聊天历史
        let date_today = Utc::now().format("%Y-%m-%d").to_string();
        let mut history = self.chat_history.as_ref().unwrap().clone();

        let system_content = WEB_SURFER_SYSTEM_MESSAGE.replace("{date_today}", &date_today);
        history.push(LLMMessage::System(SystemMessage {
            content: system_content,
        }));

        let screenshot = self.chrome_ctrl.as_ref().unwrap().get_screenshot(None).await?;

        // 3. 获取页面状态和元素
        let (page_state, original_rects) = self.get_page_state_and_elements().await?;


        let reverse_element_id_mapping: HashMap<String, String> = page_state
            .element_id_mapping
            .iter()
            .map(|(k, v)| (v.clone(), k.clone()))
            .collect();

        let rects: HashMap<String, InteractiveRegion> = original_rects
            .into_iter()
            .map(|(k,v)|{
                let new_key = reverse_element_id_mapping
                    .get(&k)
                    .cloned()
                    .unwrap_or(k);
                (new_key, v)
            })
            .collect();

        let (num_tabs, tab_info) = self.get_tabs_info().await?;
        let tabs_info_str = format!("There are {} tabs open. The tabs are as follows:\n{}", num_tabs, tab_info);
        // 4. 准备工具和上下文信息
        let mut tools = Vec::new();

        let default_tools = DefaultTools::new().unwrap();
        let base_tools = vec![
            &default_tools.stop_action,
            &default_tools.visit_url,
            &default_tools.web_search,
            &default_tools.click,
            &default_tools.input_text,
            &default_tools.answer_question,
            &default_tools.sleep,
            &default_tools.hover,
            &default_tools.history_back,
            &default_tools.keypress,
            &default_tools.refresh_page,
            &default_tools.scroll_down,
            &default_tools.scroll_up,
            &default_tools.page_up,
            &default_tools.page_down,
            &default_tools.create_tab,
        ];

        for tool in base_tools {
            tools.push(tool.clone());
        }

        if num_tabs > 1 {
            tools.push(default_tools.switch_tab.clone());
            tools.push(default_tools.close_tab.clone());
        }

        if page_state.element_id_mapping.iter().any(|(_, rect)| rect == "option") {
           tools.push(default_tools.select_option.clone());
        }

        // 获取当前聚焦的元素
        let focused = self.chrome_ctrl.as_ref().unwrap().get_focused_rect_id().await?;
        // 进行反转，自定义的-->实际的
        let focused = reverse_element_id_mapping.get(&focused).cloned().unwrap_or(focused);

        let focused_hint = if !focused.is_empty() {
            let name = self.target_name(&focused, &rects);
            let name_part = if let Some(n) = name {
                format!("(and name '{}')", n)
            } else {
                String::new()
            };
            // 获取元素的 role，如果找不到则默认为 "control"
            let role = rects
                .get(&focused)
                .map(|region| region.role.as_str())
                .unwrap_or("control");

            format!(
                "\nThe {} with ID {} {}currently has the input focus.\n\n",
                role, focused, name_part
            )
        } else {
            String::new()
        };

        let visible_targets = format!(
            "{}\n\n",
            self.format_target_list(&page_state.visible_rects, &rects).join("\n")
        );

        // 当前视口外的元素
        let mut other_targets:Vec<String> = Vec::new();
        other_targets.extend(self.format_target_list(&page_state.rects_above, &rects));
        other_targets.extend(self.format_target_list(&page_state.rects_below, &rects));

        let other_targets_str = if !other_targets.is_empty() {
            let other_target_names: Vec<String> = other_targets
                .iter()
                .filter_map(|target| {
                    serde_json::from_str::<serde_json::Value>(target).ok()
                })
                .filter_map(|target_dict| {
                    let name = target_dict
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let role = target_dict
                        .get("role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if !name.is_empty() {
                        Some(name)
                    } else if !role.is_empty() {
                        Some(format!("{} control", role))
                    } else {
                        None
                    }
                })
                .take(30)
                .collect();

            format!(
                "Some additional valid interaction targets (not shown, you need to scroll to interact with them) include:\n{}\n\n",
                other_target_names.join(", ")
            )
        } else {
            String::new()
        };

        let webpage_text = self.chrome_ctrl.as_ref().unwrap().get_visible_text().await?;
        let url = self.chrome_ctrl.as_ref().unwrap().get_url().await?;
        
        let last_outside_message = "".to_string();
        let consider_screenshot = "Consider the following screenshot of a web browser,".to_string();
        let text_prompt = format!(
            r#" The last request received was: {}
        Note that attached images may be relevant to the request.
        {}
        The webpage has the following text:
        {}
        Attached is a screenshot of the current page:
        {} which is open to the page '{}'. In this screenshot, interactive elements are outlined in bounding boxes in red. Each bounding box has a numeric ID label in red. Additional information about each visible label is listed below:
        {}{}{}"#,
            last_outside_message,
            tabs_info_str,
            webpage_text,
            consider_screenshot,
            url,
            visible_targets,
            other_targets_str,
            focused_hint,
        ).trim().to_string();

        // 5. 处理两张截图 + token 限制
        let img = image::load_from_memory(&screenshot)?;
        let resize_screenshot = img.resize(1024, 1024, FilterType::Triangle);
        let resize_som_screenshot = page_state.som_screenshot.resize(1024, 1024, FilterType::Triangle);
        
        // 将图片转换为字节数组（PNG 格式）
        let mut som_bytes = Vec::new();
        resize_som_screenshot.write_to(
            &mut std::io::Cursor::new(&mut som_bytes),
            image::ImageFormat::Png
        )?;
        
        let mut screenshot_bytes = Vec::new();
        resize_screenshot.write_to(
            &mut std::io::Cursor::new(&mut screenshot_bytes),
            image::ImageFormat::Png
        )?;
        
        
        // 6.2 添加用户消息（文本提示 + 两张图片）
        history.push(LLMMessage::User(UserMessage {
            content: vec![
                MessageContent::Text(text_prompt),
                // MessageContent::Image(som_bytes),       // SOM 标注截图
                // MessageContent::Image(screenshot_bytes), // 原始截图
            ],
        }));

        // println!("history: {:?}", history);

        // 7. 获取模型响应
        let llm_response = self.call_llm(&history, &tools).await?;
        
        // 8. 解析响应，判断是否需要执行工具
        let (content, need_execute_tool) = match llm_response {
            // 如果是文本响应，不需要执行工具
            LLMResponse::Text(text) => {
                (LLMResponse::Text(text), false)
            }
            // 如果是函数调用，需要执行工具
            LLMResponse::FunctionCalls(calls) => {
                (LLMResponse::FunctionCalls(calls), true)
            }
            // 如果是错误
            LLMResponse::Error(err) => {
                return Err(anyhow!("LLM Error: {}", err));
            }
        };

        Ok((content, rects, tools, page_state.element_id_mapping, need_execute_tool))
    }


    /// 调用 LLM 获取响应
    async fn call_llm(
        &self, 
        history: &[LLMMessage], 
        tools: &[ToolSchema]
    ) -> Result<LLMResponse> {
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
        
        // 1. 初始化客户端（支持 OpenAI 和阿里云 DashScope）
        // 优先使用阿里云 DashScope，其次是 OpenAI
        let (api_key, base_url, model) = if let Ok(dashscope_key) = std::env::var("DASHSCOPE_API_KEY") {
            // 使用阿里云 DashScope
            let url = std::env::var("DASHSCOPE_BASE_URL")
                .unwrap_or_else(|_| "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string());
            let model_name = std::env::var("DASHSCOPE_MODEL")
                .unwrap_or_else(|_| "qwen-vl-max".to_string());
            (dashscope_key, url, model_name)
        } else if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
            // 使用 OpenAI
            let url = std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
            let model_name = std::env::var("OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4o".to_string());
            (openai_key, url, model_name)
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
                LLMMessage::System(sys_msg) => {
                    api_messages.push(ChatCompletionRequestMessage::System(
                        ChatCompletionRequestSystemMessage {
                            content: sys_msg.content.clone(),
                            name: None,
                        }
                    ));
                }
                LLMMessage::User(user_msg) => {
                    let mut content_parts = Vec::new();
                    
                    for content in &user_msg.content {
                        match content {
                            MessageContent::Text(text) => {
                                content_parts.push(
                                    async_openai::types::ChatCompletionRequestMessageContentPart::Text(
                                        async_openai::types::ChatCompletionRequestMessageContentPartText {
                                            text: text.clone(),
                                        }
                                    )
                                );
                            }
                            MessageContent::Image(img_bytes) => {
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
                    
                    api_messages.push(ChatCompletionRequestMessage::User(
                        ChatCompletionRequestUserMessage {
                            content: ChatCompletionRequestUserMessageContent::Array(content_parts),
                            name: None,
                        }
                    ));
                }
                LLMMessage::Assistant(asst_msg) => {
                    #[allow(deprecated)]
                    api_messages.push(ChatCompletionRequestMessage::Assistant(
                        async_openai::types::ChatCompletionRequestAssistantMessage {
                            content: Some(asst_msg.content.clone()),
                            name: None,
                            tool_calls: None,
                            function_call: None,
                        }
                    ));
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
                return Ok(LLMResponse::FunctionCalls(function_calls));
            }
            
            // 否则返回文本响应
            if let Some(content) = &choice.message.content {
                return Ok(LLMResponse::Text(content.clone()));
            }
        }
        
        // 如果没有有效响应
        Err(anyhow!("No valid response from LLM"))
    }


    async fn get_page_state_and_elements(&self) -> Result<(PageState, HashMap<String, InteractiveRegion>)> {
        let rects = self.chrome_ctrl.as_ref().unwrap().get_interactive_rects().await?;
        let screenshot = self.chrome_ctrl.as_ref().unwrap().get_screenshot(None).await?;
        let page_state = add_set_of_mark(&screenshot, &rects, true)?;
        Ok((page_state, rects))
    }

    pub async fn check_url_and_generate_msg(&mut self, url: String) -> Result<(String,bool)> {
        // 特殊处理 chrome-error界面
        if url == "chrome-error://chromewebdata/" {
            if let Some(last_rejected) = self.last_rejected_url.take() {
                let msg = format!(
                    "I am not allowed to visit the website {} because it is not in the list of websites I can access and the use has declined to approve it.",
                    last_rejected
                );
                return Ok((msg, false));
            }
        }
        // 检查是否被blocked
        if self.url_status_manager.is_url_blocked(&url) {
            let msg = format!(
                "I am not allowed to visit the website {} because it has been blocked.",
                url
            );
            return Ok((msg, false));
        }
        // 检查是否允许
        if !self.url_status_manager.is_url_allowed(&url) {
            if !self.url_status_manager.is_url_rejected(&url) {
                // 提取域名（fqdn）
                let domain = {
                    // 使用临时 extractor（或可缓存到 WebAgent）
                    let extractor = TldExtractor::new(TldOption::default());
                    let extracted = extractor.extract(&url).unwrap_or_else(|_| {
                        tldextract::TldResult { domain: None, subdomain: None, suffix: None }
                    });
                    match (&extracted.domain, &extracted.suffix) {
                        (Some(domain), Some(suffix)) => format!("{}.{}", domain, suffix),
                        (Some(domain), None) => domain.clone(),
                        _ => String::new(),
                    }
                };
                let domain = if domain.is_empty() { url.clone() } else { domain };

                let approved = if let Some(guard) = &self.action_guard {
                    let request_msg = TextMessage {
                        source: self.name.clone(),
                        content: format!(
                            "The website {} is not allowed. Would you like to allow the domain {} for this session?",
                            url, domain
                        ),
                        metadata: HashMap::new(),
                    };
                    guard.get_approval(request_msg).await
                } else {
                    false
                };

                if approved {
                    self.url_status_manager.set_url_status(&domain, UrlStatus::Allowed);
                    return Ok(("".to_string(), true));
                } else {
                    self.url_status_manager.set_url_status(&domain, UrlStatus::Rejected);
                }
            }

            // 记录最后被拒绝的 URL
            self.last_rejected_url = Some(url.clone());
            let msg = format!(
                "I am not allowed to visit the website {} because it is not in the list of websites I can access and the user has declined to allow it.",
                url
            );
            return Ok((msg, false));
        }

        Ok(("".to_string(),true)) 
    }

    pub async fn get_tabs_info(&self) -> Result<(usize,String)> {
        let tabs_info = self.chrome_ctrl.as_ref().unwrap().get_tabs_information().await?;
        let num_tabs = tabs_info.len();

        let tabs_info_str = tabs_info
            .iter()
            .map(|tab|{
                let mut parts = vec![
                    format!("Tab {}: {} ({})", tab.index, tab.title, tab.url),
                ];

                if tab.is_active {
                    parts.push(" [CURRENTLY SHOWN]".to_string());
                }

                if tab.is_controlled {
                    parts.push(" [CONTROLLED]".to_string());
                }
                parts.join(" ")
            })
            .collect::<Vec<String>>()
            .join("\n");
        Ok((num_tabs, tabs_info_str))
    }

    pub fn format_target_list(
        &self,
        ids: &[String],
        rects: &HashMap<String, InteractiveRegion>,
    ) -> Vec<String> {
        let unique_ids: HashSet<_> = ids.iter().collect();
        let mut targets: Vec<(i32, String)> = Vec::new();
        
        let newline_regex = Regex::new(r"[\n\r]+").unwrap();
        
        for id in unique_ids {
            if let Some(rect) = rects.get(id) {
                // 获取 role
                let mut aria_role = rect.role.trim().to_string();
                if aria_role.is_empty() {
                    aria_role = rect.tag_name.trim().to_string();
                }
                
                // 获取 name
                let aria_name = rect
                    .aria_name
                    .as_ref()
                    .map(|name| {
                        let cleaned = newline_regex.replace_all(name, " ");
                        cleaned.trim().to_string()
                    })
                    .unwrap_or_default();
                
                // 确定可用的 actions
                let mut actions = vec!["click", "hover"];

                if rect.role == "textbox" 
                    || rect.role == "searchbox" 
                    || rect.role == "combobox"
                    || rect.tag_name == "input"
                    || rect.tag_name == "textarea"
                    || rect.tag_name == "search"
                {
                    actions.push("input_text");
                }
            
                if rect.role == "option" {
                    actions = vec!["select_option"];
                }
                
                if aria_role == "input, type=file" {
                    actions = vec!["upload_file"];
                }
                
                // 限制 name 最多 100 字符
                let aria_name_truncated: String = aria_name
                    .chars()
                    .take(100)
                    .collect();
                
                // 使用 serde_json 安全地构建 JSON
                let target_json = json!({
                    "id": id.parse::<i32>().unwrap_or(0),
                    "name": aria_name_truncated,
                    "role": aria_role,
                    "tools": actions
                });
                
                let id_num = id.parse::<i32>().unwrap_or(0);
                targets.push((id_num, target_json.to_string()));
            }
        }
        targets.sort_by_key(|(id, _)| *id);
        targets.into_iter().map(|(_, target)| target).collect()
    }

    pub async fn executor_tool(
        &mut self,
        messages: Vec<FunctionCall>,                    // 提取工具的名称
        rects: HashMap<String, InteractiveRegion>,      // 主要传递给需要与页面元素交互的工具
        tools: Vec<ToolSchema>,                         // 工具列表
        element_id_mapping: HashMap<String, String>,    // 为页面元素提供ID映射
        cancellation_token: Option<CancellationToken>,  // 支持异步操作的取消功能
    ) -> Result<String> {
        // 确保浏览器上下文已准备好，保证仅有一个FunctionCall（为了一次执行一个动作）
        self.chrome_ctrl
            .as_ref()
            .ok_or_else(|| anyhow!("Chrome controller not initialized"))?
            .wait_for_page_ready()
            .await?;

        if messages.len() != 1 {
            return Err(anyhow::anyhow!("Expected exactly one function call"));
        }

        // 从 function call 中获取参数(工具的名称[name] 和 参数[arguments])
        let function_call = &messages[0];
        let name = &function_call.name;
        let args: serde_json::Value = serde_json::from_str(&function_call.arguments)
            .map_err(|e| anyhow::anyhow!("Failed to parse function arguments: {}", e))?;

        println!("Executing tool: {}({})", name, serde_json::to_string(&args)?);

        // 验证工具是否存在
        let available_tools: Vec<String> = tools.iter()
            .map(|tool| tool.name.clone())
            .collect();
        
        if !available_tools.contains(name) {
            let tool_names = available_tools.join(", ");
            return Err(anyhow::anyhow!(
                "Unknown tool '{}'. Please choose from:\n\n{}", 
                name, 
                tool_names
            ));
        }

        // 根据工具名称执行对应的工具函数
        let action_description = match name.as_str() {
            "click" => self.execute_tool_click(args, &rects, &element_id_mapping).await?,
            "input_text" => self.execute_tool_input_text(args, &rects, &element_id_mapping).await?,
            "hover" => self.execute_tool_hover(args, &rects, &element_id_mapping).await?,
            "select_option" => self.execute_tool_select_option(args, &rects, &element_id_mapping).await?,
            "upload_file" => self.execute_tool_upload_file(args, &rects, &element_id_mapping).await?,
            "click_full" => self.execute_tool_click_full(args, &rects, &element_id_mapping).await?,
            "answer_question" => self.execute_tool_answer_question(args, cancellation_token).await?,
            // "summarize_page" => self.execute_tool_summarize_page(args, cancellation_token).await?,
            "visit_url" => self.execute_tool_visit_url(args).await?,
            _ => {
                return Err(anyhow::anyhow!("Tool '{}' is not implemented yet", name));
            }
        };

        // TODO: 处理下载相关逻辑
        // if let Some(last_download) = &self.last_download {
        //     if let Some(download_folder) = &self.downloads_folder {
        //         action_description.push_str(&format!(
        //             "\n\nSuccessfully downloaded '{}' to local path: {}",
        //             last_download.suggested_filename,
        //             download_folder
        //         ));
        //     }
        // }

        Ok(action_description)
    }

    // 终止Agent执行，并返回最终的答案
    pub async fn execute_tool_stop_action(&mut self, args: String) -> Result<String> {
        Ok(args)
    }

    async fn execute_tool_visit_url(&mut self, args: Value) -> Result<String> {

        self.chrome_ctrl
            .as_ref()
            .ok_or_else(|| anyhow!("Chrome controller not initialized"))?
            .wait_for_page_ready()
            .await?;
        let url = args
            .get("url")
            .and_then(|v|v.as_str())
            .ok_or_else(|| anyhow::anyhow!("URL is required"))?;

        let (ret, approved) = self.check_url_and_generate_msg(url.to_string()).await?;
        if !approved {
            return Ok(ret);
        }

        let action_description = format!("I type '{}' into the browser address bar.", url);

        let reset_prior_metadata = 
            if url.starts_with("https://") 
                || url.starts_with("http://") 
                || url.starts_with("file://") 
                || url.starts_with("about:") 
            {
                self.chrome_ctrl.as_ref().unwrap().visit_page(url).await?
            } else if url.contains(" ") {
                let (ret, approved) = self.check_url_and_generate_msg("bing.com".to_string()).await?;
                if !approved {
                    return Ok(ret);
                }
                let search_url = format!("https://www.bing.com/search?q={}", url);
                self.chrome_ctrl.as_ref().unwrap().visit_page(&search_url).await?
            } else {
                let full_url = format!("https://{}", url);
                self.chrome_ctrl.as_ref().unwrap().visit_page(&full_url).await?
            };

        // 4. 更新状态
        if reset_prior_metadata {
            // self.prior_metadata_hash = None;
        }

        Ok(action_description)
    }

    async fn execute_tool_history_back(&self) -> Result<String> {
        self.chrome_ctrl.as_ref().unwrap().wait_for_page_ready().await?;
        match self.chrome_ctrl.as_ref().unwrap().go_back().await {
            Ok(()) => {
                return Ok("I clicked the browser back button.".to_string())
            }
            Err(_) => {
                return Ok("No previous page in the browser history or couldn't navigate back.".to_string())
            }
        }
    }

    async fn execute_tool_refresh_page(&self) -> Result<String> {
        self.chrome_ctrl.as_ref().unwrap().wait_for_page_ready().await?;
        self.chrome_ctrl.as_ref().unwrap().refresh().await?;
        Ok("I refreshed the current page.".to_string())
    }

    async fn execute_tool_web_search(&mut self, args: serde_json::Value) -> Result<String> {

        self.chrome_ctrl.as_ref().unwrap().wait_for_page_ready().await?;
        let (ret, approved) = self.check_url_and_generate_msg("bing.com".to_string()).await?;

        if !approved {
            return Ok(ret);
        }

        let query = args.get("query").and_then(|v|v.as_str()).ok_or_else(|| anyhow::anyhow!("Query is required"))?;
        let search_url = format!("https://www.bing.com/search?q={}&FORM=QBLH", query);
        self.chrome_ctrl.as_ref().unwrap().visit_page(&search_url).await?;

        let reset_prior_metadata = self
            .chrome_ctrl
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Chrome controller not initialized"))?
            .visit_page(&search_url)
            .await?;

        if reset_prior_metadata {
            // self.prior_metadata_hash = None;
        }

        Ok(format!("I typed '{}' into the browser search bar.", query))
    }

    async fn execute_tool_page_up(&self) -> Result<String> {
        self.chrome_ctrl.as_ref().unwrap().wait_for_page_ready().await?;
        self.chrome_ctrl.as_ref().unwrap().page_up().await?;
        Ok("I scrolled up one page in the browser".to_string())
    }

    async fn execute_tool_page_down(&self) -> Result<String> {
        self.chrome_ctrl.as_ref().unwrap().wait_for_page_ready().await?;
        self.chrome_ctrl.as_ref().unwrap().page_down().await?;
        Ok("I scrolled down one page in the browser".to_string())
    }

    async fn execute_tool_scroll_down(&self, args: serde_json::Value) -> Result<String> {
        self.chrome_ctrl.as_ref().unwrap().wait_for_page_ready().await?;
        let pixels = args.get("pixels").and_then(|v|v.as_i64()).unwrap_or(400) as i32;
        self.chrome_ctrl.as_ref().unwrap().scroll_mousewheel("down", pixels).await?;
        Ok(format!("I scrolled down {} pixels in the browser.", pixels))
    }

    async fn execute_tool_scroll_up(&self, args: serde_json::Value) -> Result<String> {
        self.chrome_ctrl.as_ref().unwrap().wait_for_page_ready().await?;
        let pixels = args.get("pixels").and_then(|v|v.as_i64()).unwrap_or(400) as i32;
        self.chrome_ctrl.as_ref().unwrap().scroll_mousewheel("up", pixels).await?;
        Ok(format!("I scrolled up {} pixels in the browser.", pixels))
    }

    async fn execute_tool_click(
        &self,
        args: serde_json::Value,
        rects: &HashMap<String, InteractiveRegion>,
        element_id_mapping: &HashMap<String, String>,
    ) -> Result<String> {
        let target_id = args.get("target_id").and_then(|v|v.as_str()).unwrap_or("1");

        let target_name = self.target_name(target_id, &rects);
        let mapped_id = element_id_mapping
            .get(target_id)
            .ok_or_else(|| anyhow!("Target ID '{}' not found in mapping", target_id))?;

        let action_description = if let Some(name) = target_name {
            format!("I clicked '{}'.", name)
        } else {
            "I clicked the control.".to_string()
        };

        // let new_page_info = self
        //     .chrome_ctrl
        //     .as_ref()
        //     .ok_or_else(|| anyhow!("Chrome controller not initialized"))?
        //     .click_id(mapped_id, 0.0, "left")
        //     .await?;

        // if let Some(page_info) = new_page_info {
            // self.prior_metadata_hash = None; // 重置元数据

            // let (ret, approved) = self
            //     .check_url_and_generate_msg(page_info.url.clone())
            //     .await?;
            // if !approved {
            //     return Ok(ret);
            // }
        // }
        
        Ok(action_description)
    }

    async fn execute_tool_click_full(
        &self,
        args: serde_json::Value,
        rects: &HashMap<String, InteractiveRegion>,
        element_id_mapping: &HashMap<String, String>,
    ) -> Result<String> {
        let target_id = args
        .get("target_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'target_id' is required"))?;

        let target_name = self.target_name(target_id, &rects);
        let mapped_id = element_id_mapping
            .get(target_id)
            .ok_or_else(|| anyhow!("Target ID '{}' not found in mapping", target_id))?;

            let hold = args
            .get("hold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let button = args
            .get("button")
            .and_then(|v| v.as_str())
            .unwrap_or("left");

        let action_description = if let Some(name) = target_name {
            format!(
                "I clicked '{}' with button '{}' and hold {} seconds.",
                name, button, hold
            )
        } else {
            format!(
                "I clicked the control with button '{}' and hold {} seconds.",
                button, hold
            )
        };

        // let new_page_info = self
        //     .chrome_ctrl
        //     .as_ref()
        //     .ok_or_else(|| anyhow!("Chrome controller not initialized"))?
        //     .click_id(mapped_id, hold, button)
        //     .await?;

        // if let Some(page_info) = new_page_info {
        //     // self.prior_metadata_hash = None;

        //     let (ret, approved) = self
        //         .check_url_and_generate_msg(page_info.url.clone())
        //         .await?;
        //     if !approved {
        //         return Ok(ret);
        //     }
        // }

        Ok(action_description)
    }

    async fn execute_tool_input_text(
        &mut self,
        args: serde_json::Value,
        rects: &HashMap<String, InteractiveRegion>,
        element_id_mapping: &HashMap<String, String>,
    ) -> Result<String> {
        let input_field_id = args
            .get("input_field_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("'input_field_id' is required"))?;
        let text_value = args
            .get("text_value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("'text_value' is required"))?;
        let delete_existing_text = args
            .get("delete_existing_text")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let press_enter = args
            .get("press_enter")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let input_field_name = self.target_name(input_field_id, &rects);
        let mapped_id = element_id_mapping
            .get(input_field_id)
            .ok_or_else(|| anyhow!("Input field ID '{}' not found in mapping", input_field_id))?;

        let action_description = if let Some(name) = input_field_name {
            format!("I typed '{}' into '{}'.", text_value, name)
        } else {
            format!("I typed '{}'.", text_value)
        };

        self.chrome_ctrl
            .as_mut()
            .ok_or_else(|| anyhow!("Chrome controller not initialized"))?
            .fill_id(mapped_id, text_value, press_enter, delete_existing_text)
            .await?;
        Ok(action_description)
    }

    async fn execute_tool_answer_question(
        &mut self,
        args: serde_json::Value,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<String> {
        let question = args.get("question").and_then(|v|v.as_str()).ok_or_else(|| anyhow!("'question' is required"))?;
        return self.summarize_page(Some(question), cancellation_token).await;
    }

    async fn execute_tool_summarize_page(
        &mut self,
        question: Option<&str>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<String> { 
        return self.summarize_page(question, cancellation_token).await;
    }

    async fn execute_tool_hover(
        &mut self,
        args: serde_json::Value,
        rects: &HashMap<String, InteractiveRegion>,
        element_id_mapping: &HashMap<String, String>,
    ) -> Result<String> {
        let target_id = args
            .get("target_id")
            .and_then(|v|v.as_str())
            .ok_or_else(|| anyhow!("'target_id' is required"))?;
        
        let target_name = self.target_name(target_id, &rects);
        let mapping_id = element_id_mapping
            .get(target_id)
            .map(|id| id.as_str())
            .unwrap_or(target_id);

        let action_descirption = if let Some(name) = target_name {
            format!("I hovered over '{}'.", name)
        } else {
            format!("I hovered over the control.")
        };

        let _ = self.chrome_ctrl.as_mut().unwrap().hover_id(mapping_id).await?;

        Ok(action_descirption)
    }


    async fn execute_tool_sleep(&mut self, args: serde_json::Value) -> Result<String> {
        let duration = args.get("duration").and_then(|v|v.as_i64()).unwrap_or(1000) as u64;
        self.chrome_ctrl.as_mut().unwrap().sleep(duration).await?;
        Ok(format!("I waited {} seconds.", duration))
    }

    async fn execute_tool_select_option(
        &self,
        _args: serde_json::Value,
        _rects: &HashMap<String, InteractiveRegion>,
        _element_id_mapping: &HashMap<String, String>,
    ) -> Result<String> {
        // TODO: 实现选择选项功能
        Ok("Select option action executed".to_string())
    }

    async fn execute_tool_create_tab(&mut self, args: serde_json::Value) -> Result<String> {
        let url = args.get("url").and_then(|v|v.as_str()).unwrap_or("https://www.bing.com");
        let (ret,approved) = self.check_url_and_generate_msg(url.to_string()).await?;
        if !approved {
            return Ok(ret);
        }

        let action_description = format!("I created a new tab and navigated to '{}'.", url);
        let _ = self.chrome_ctrl.as_mut().unwrap().new_tab(url).await?;

        Ok(action_description)
    }

    async fn execute_tool_switch_tab(&mut self, args: serde_json::Value) -> Result<String> {
        let tab_index = args.get("tab_index").and_then(|v|v.as_i64()).unwrap_or(0);

        if tab_index < 0 {
            return Err(anyhow!("tab_index must be non-negative"));
        }

        let chrome_ctrl = self.chrome_ctrl.as_mut().unwrap();

        let handles = chrome_ctrl.driver.window().await?;

        chrome_ctrl.switch_tab(&handles).await?;
    
        let action_description = format!("I switched to tab {}.", tab_index);
        Ok(action_description)
    }

    async fn execute_tool_close_tab(&mut self, args: serde_json::Value) -> Result<String> {
        let tab_index = args.get("tab_index").and_then(|v|v.as_i64()).unwrap_or(0);

        self.chrome_ctrl.as_mut().unwrap().close_tab().await?;
    
        let action_description = format!("I closed tab {}.", tab_index);
        Ok(action_description)
    }

    async fn execute_tool_upload_file(
        &self,
        _args: serde_json::Value,
        _rects: &HashMap<String, InteractiveRegion>,
        _element_id_mapping: &HashMap<String, String>,
    ) -> Result<String> {
        // TODO: 实现文件上传功能
        Ok("Upload file action executed".to_string())
    }

    async fn execute_tool_keypress(&self, _args: serde_json::Value) -> Result<String> {
        // TODO: 实现按键功能
        Ok("Press action executed".to_string())
    }

    fn target_name(&self, target: &str, rects: &HashMap<String, InteractiveRegion>) -> Option<String> {
        rects
            .get(target)
            .and_then(|region| region.aria_name.as_ref())
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
    }

    pub async fn summarize_page(
        &mut self, 
        _question: Option<&str>, 
        _cancellation_token: Option<CancellationToken>
    ) -> Result<String> {
    /* 
        let page_markdown = self
            .chrome_ctrl
            .as_mut()
            .ok_or_else(|| anyhow!("Chrome controller not initialized"))?
            .get_page_markdown(1000)
            .await?;

        let title = self
            .chrome_ctrl
            .as_mut()
            .ok_or_else(|| anyhow!("Chrome controller not initialized"))?
            .get_title()
            .await?;

            let screenshot_bytes = self
            .chrome_ctrl
            .as_mut()
            .ok_or_else(|| anyhow!("Chrome controller not initialized"))?
            .get_screenshot(None)
            .await?;

        let img = image::load_from_memory_with_format(&screenshot_bytes, ImageFormat::Png)?;
        let scaled = img.resize_exact(MLM_WIDTH, MLM_HEIGHT, image::imageops::FilterType::Triangle);
        let ag_image = AGImage::from_dynamic_image(&scaled);

        // 构建提示
        let prompt = (&title, question);

        // Token 计算
        let bpe = get_bpe_from_model("gpt-4o")
            .map_err(|e| anyhow!("Tokenization error: {}", e))?;

        let prompt_tokens = bpe.encode_with_special_tokens(&prompt).len();
        let max_content_tokens = MAX_MODEL_TOKENS
            .saturating_sub(SCREENSHOT_TOKENS)
            .saturating_sub(prompt_tokens)
            .saturating_sub(1000); // 缓冲

        let content = if max_content_tokens == 0 {
            prompt.clone()
        } else {
            let content_tokens = bpe.encode_with_special_tokens(&page_markdown);
            if content_tokens.len() > max_content_tokens {
                let truncated_tokens = &content_tokens[..max_content_tokens];
                let truncated_text = bpe.decode(truncated_tokens);
                format!("Page content (truncated):\n{}\n\n{}", truncated_text, prompt)
            } else {
                format!("Page content:\n{}\n\n{}", page_markdown, prompt)
            }
        };

        let mut messages = vec![
            LLMMessage::System(SystemMessage {
                content: WEB_SURFER_QA_SYSTEM_MESSAGE.to_string(),
            }),
            LLMMessage::User(UserMessage {
                content: vec![content.into(), ag_image.into()],
                source: self.name.clone(),
            }),
        ];


        let response = self
            .model_client
            .create(&messages, cancellation_token)
            .await
            .map_err(|e| anyhow!("LLM error: {}", e))?;

        self.model_usage.push(response.usage.clone());

        Ok(response.content)
        */
        Ok("".to_string())
    }

    fn web_surfer_qa_prompt(title: &str, question: Option<&str>) -> String {
        let base_prompt = format!(
            "We are visiting the webpage '{}'. Its full-text content are pasted below, along with a screenshot of the page's current viewport.",
            title
        );
    
        if let Some(q) = question {
            format!("{} Please answer the following question completely: '{}':\n\n", base_prompt, q)
        } else {
            format!("{} Please summarize the webpage into one or two paragraphs:\n\n", base_prompt)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试基本的 LLM 响应
    
    /// 测试 Google 搜索 "grok"
    /// 运行方式：cargo test test_google_search_grok -- --ignored --nocapture
    #[tokio::test]
    #[ignore] // 需要浏览器和 API key，使用 cargo test -- --ignored 运行
    async fn test_google_search_grok() -> Result<()> {

        dotenv::dotenv().ok();

        // 1. 创建并初始化 WebAgent
        let mut agent = WebAgent::new().await;
        agent.initialize().await?;
        
        println!("✅ WebAgent 初始化成功");
        
        // 2. 访问 Google 首页（在单独的作用域中完成，避免借用冲突）
        {
            println!("\n📍 正在访问 Google...");
            let chrome = agent.chrome_mut().await?;
            chrome.visit_page("https://www.google.com").await?;
            chrome.sleep(2000).await?;
            println!("✅ 已访问 Google");
        } // chrome 的借用在这里结束
        
        // 3. 模拟用户输入：在 Google 搜索 grok
        if let Some(history) = agent.chat_history.as_mut() {
            history.push(LLMMessage::User(UserMessage {
                content: vec![MessageContent::Text("在谷歌搜索grok".to_string())],
            }));
        }
        
        println!("\n🤖 正在调用 LLM 获取响应...");
        
        // 4. 调用 get_llm_response 获取 LLM 的决策
        let (response, rects, tools, element_id_mapping, need_execute_tool) = 
            agent.get_llm_response().await?;

        // 5. 打印结果
        println!("\n{}", "=".repeat(60));
        println!("📊 LLM 响应结果: {:?}", response);
        println!("{}", "=".repeat(60));
        
        match &response {
            LLMResponse::Text(text) => {
                println!("\n💬 文本响应：\n{}", text);
            }
            LLMResponse::FunctionCalls(calls) => {
                println!("\n🔧 工具调用（共 {} 个）：", calls.len());
                for (i, call) in calls.iter().enumerate() {
                    println!("\n  [{}] 工具名称: {}", i + 1, call.name);
                    println!("      工具ID: {}", call.id);
                    println!("      参数: {}", call.arguments);
                }
            }
            LLMResponse::Error(err) => {
                println!("\n❌ 错误: {}", err);
            }
        }
        
        println!("\n📍 页面交互元素数量: {}", rects.len());
        println!("🔧 可用工具数量: {}", tools.len());
        println!("🗺️  元素ID映射数量: {}", element_id_mapping.len());
        println!("⚙️  需要执行工具: {}", need_execute_tool);
        
        // 6. 如果需要执行工具，展示第一个工具的详细信息
        if need_execute_tool {
            if let LLMResponse::FunctionCalls(calls) = &response {
                if let Some(first_call) = calls.first() {
                    println!("\n{}", "=".repeat(60));
                    println!("🎯 第一个工具调用详情");
                    println!("{}", "=".repeat(60));
                    println!("工具: {}", first_call.name);
                    
                    // 尝试解析参数
                    if let Ok(args) = serde_json::from_str::<serde_json::Value>(&first_call.arguments) {
                        println!("参数（格式化）:");
                        println!("{}", serde_json::to_string_pretty(&args).unwrap_or(first_call.arguments.clone()));
                    }
                }
            }
        }

        println!("\n{}", "=".repeat(60));
        
        // 7. 等待一下再关闭浏览器，方便查看
        {
            let chrome = agent.chrome_mut().await?;
            chrome.sleep(3000).await?;
        }
        
        Ok(())
    }
}
