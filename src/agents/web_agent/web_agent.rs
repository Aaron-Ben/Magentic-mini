use std::collections::HashMap;

use thirtyfour::prelude::*;
use anyhow::{Result, anyhow};
use crate::agents::web_agent::prompt::WEB_SURFER_SYSTEM_MESSAGE;
use crate::agents::web_agent::set_of_mark::{PageState, _add_set_of_mark};
use crate::agents::web_agent::tool_define::DefaultTools;
use crate::agents::web_agent::types::FunctionCall;
use crate::tools::chrome::chrome_ctrl::Chrome;
use crate::tools::chrome::types::InteractiveRegion;
use crate::tools::tool_metadata::ToolSchema;
use crate::types::message::{CancellationToken, LLMMessage, SystemMessage};


pub enum LLMResponse {
    Text(String),
    FunctionCalls(Vec<FunctionCall>),
    Error(String),
}

#[derive(Debug)]
pub struct WebAgent {
    chrome_ctrl: Option<Chrome>,
    chat_history: Option<Vec<LLMMessage>>,
    tools: Vec<ToolSchema>,
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
            chat_history: None,
            tools,
        }
    }
}

impl WebAgent {
    pub async fn new() -> Self {
        Self::default()
    }

    pub async fn initialize(&mut self) -> Result<(), WebDriverError> {
        self.chrome_ctrl = Some(Chrome::new().await?);
        self.chat_history = Some(Vec::new());
        Ok(())
    }

    pub async fn chrome_mut(&mut self) -> Result<&mut Chrome, &'static str> {
        self.chrome_ctrl.as_mut()
            .ok_or("Chrome context is not initialized. Call initialize() first.")
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
    pub async fn _get_llm_response(
        &self,
        // cancellation_token: Option<CancellationToken>,
    ) -> Result<(
        LLMResponse,
        HashMap<String,InteractiveRegion>,
        Vec<ToolSchema>,
        HashMap<String,String>,
        bool,)>
    {

        // 1. 确保页面可用性
        self.chrome_ctrl.as_ref().unwrap()._wait_for_page_ready().await?;
        self.chrome_ctrl.as_ref().unwrap()._get_interactive_rects().await?;

        // 2. 准备聊天历史
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let history = self.build_history(&today).await?;

        // 3. 获取页面状态和元素
        let page_state = self.get_page_state_and_elements().await?;

        // // 4. 保存调试截图
        // self.save_debug_screenshot_if_needed(&screenshot).await?;

        // 5. 准备工具和上下文信息
        let tools = self.prepare_tools().await?;
        // let context_info = self.get_context_info(&rects, &reverse_element_id_mapping).await?;

        // // 6. 创建提示词
        // let text_prompt = self.create_prompt(&context_info, &tools).await?;

        // // 7. 添加多模态内容
        // let history = self.add_multimodal_content(history, &text_prompt, &screenshot).await?;

        // // 8. 应用 token 限制
        // let token_limited_history = self.apply_token_limit(history).await?;

        // // 9. 获取模型响应
        // let response = self.get_model_response(
        //     &token_limited_history, 
        //     &tools, 
        //     cancellation_token.as_ref()
        // ).await?;

        // // 10. 处理并返回响应
        // self.process_response(response, rects, tools, element_id_mapping).await

        Ok((LLMResponse::Text("".to_string()), HashMap::new(), self.tools.clone(), HashMap::new(), false))
    }

    async fn ensure_page_ready(&self) -> Result<()> {
        Ok(())
    }

    async fn build_history(&self,date_today: &str) -> Result<Vec<LLMMessage>> {
        
        let existing_history = self
            .chat_history
            .as_ref()
            .ok_or_else(|| anyhow!("Chat history not init"))?;
        
        let mut history = vec![
            LLMMessage::System(SystemMessage { 
                content: WEB_SURFER_SYSTEM_MESSAGE.replace("{date_today}", date_today)
            }),
        ];

        for msg in existing_history {
            // let filtered_msg = match msg {
            //     LLMMessage::User(Us)
            // }
            history.push(msg.clone());
        }
        
        Ok(history)
    }

    async fn get_page_state_and_elements(&self) -> Result<PageState> {
        let rects = self.chrome_ctrl.as_ref().unwrap()._get_interactive_rects().await?;
        let screenshot = self.chrome_ctrl.as_ref().unwrap()._get_screenshot(None).await?;
        let result = _add_set_of_mark(&screenshot, &rects, true)?;
        Ok(result)
    }

    async fn prepare_tools(&self) -> Result<Vec<ToolSchema>> {
        Ok(self.tools.clone())
    }

    async fn create_prompt(&self) -> Result<()> {
        Ok(())
    }

    async fn get_model_response(&self) -> Result<()> {
        Ok(())
    }

    async fn process_response(&self) -> Result<()> {
        Ok(())
    }


    pub async fn _executor_tool(
        &self,
        messages: Vec<FunctionCall>,                    // 提取工具的名称
        rects: HashMap<String, InteractiveRegion>,      // 主要传递给需要与页面元素交互的工具
        tools: Vec<ToolSchema>,                         // 工具列表
        element_id_mapping: HashMap<String, String>,    // 为页面元素提供ID映射
        cancellation_token: Option<CancellationToken>,  // 支持异步操作的取消功能
    ) -> Result<String> {
        // 确保浏览器上下文已准备好，保证仅有一个FunctionCall（为了一次执行一个动作）
        let _ = self.chrome_ctrl.as_ref().unwrap()._wait_for_page_ready();

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
            "click" => self._execute_tool_click(args, &rects, &element_id_mapping).await?,
            "input_text" => self._execute_tool_input_text(args, &rects, &element_id_mapping).await?,
            "hover" => self._execute_tool_hover(args, &rects, &element_id_mapping).await?,
            "select_option" => self._execute_tool_select_option(args, &rects, &element_id_mapping).await?,
            "upload_file" => self._execute_tool_upload_file(args, &rects, &element_id_mapping).await?,
            "click_full" => self._execute_tool_click_full(args, &rects, &element_id_mapping).await?,
            "answer_question" => self._execute_tool_answer_question(args, cancellation_token).await?,
            "summarize_page" => self._execute_tool_summarize_page(args, cancellation_token).await?,
            "visit_url" => self._execute_tool_visit_url(args).await?,
            "press" => self._execute_tool_press(args).await?,
            "scroll" => self._execute_tool_scroll(args).await?,
            "screenshot" => self._execute_tool_screenshot().await?,
            "get_page_content" => self._execute_tool_get_page_content().await?,
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

    // 具体的工具执行函数
    async fn _execute_tool_click(
        &self,
        _args: serde_json::Value,
        _rects: &HashMap<String, InteractiveRegion>,
        _element_id_mapping: &HashMap<String, String>,
    ) -> Result<String> {
        // TODO: 实现点击功能
        Ok("Click action executed".to_string())
    }

    async fn _execute_tool_input_text(
        &self,
        _args: serde_json::Value,
        _rects: &HashMap<String, InteractiveRegion>,
        _element_id_mapping: &HashMap<String, String>,
    ) -> Result<String> {
        // TODO: 实现文本输入功能
        Ok("Input text action executed".to_string())
    }

    async fn _execute_tool_hover(
        &self,
        _args: serde_json::Value,
        _rects: &HashMap<String, InteractiveRegion>,
        _element_id_mapping: &HashMap<String, String>,
    ) -> Result<String> {
        // TODO: 实现悬停功能
        Ok("Hover action executed".to_string())
    }

    async fn _execute_tool_select_option(
        &self,
        _args: serde_json::Value,
        _rects: &HashMap<String, InteractiveRegion>,
        _element_id_mapping: &HashMap<String, String>,
    ) -> Result<String> {
        // TODO: 实现选择选项功能
        Ok("Select option action executed".to_string())
    }

    async fn _execute_tool_upload_file(
        &self,
        _args: serde_json::Value,
        _rects: &HashMap<String, InteractiveRegion>,
        _element_id_mapping: &HashMap<String, String>,
    ) -> Result<String> {
        // TODO: 实现文件上传功能
        Ok("Upload file action executed".to_string())
    }

    async fn _execute_tool_click_full(
        &self,
        _args: serde_json::Value,
        _rects: &HashMap<String, InteractiveRegion>,
        _element_id_mapping: &HashMap<String, String>,
    ) -> Result<String> {
        // TODO: 实现完整点击功能
        Ok("Click full action executed".to_string())
    }

    async fn _execute_tool_answer_question(
        &self,
        _args: serde_json::Value,
        _cancellation_token: Option<CancellationToken>,
    ) -> Result<String> {
        // TODO: 实现回答问题功能
        Ok("Answer question action executed".to_string())
    }

    async fn _execute_tool_summarize_page(
        &self,
        _args: serde_json::Value,
        _cancellation_token: Option<CancellationToken>,
    ) -> Result<String> {
        // TODO: 实现页面总结功能
        Ok("Summarize page action executed".to_string())
    }

    async fn _execute_tool_visit_url(&self, _args: serde_json::Value) -> Result<String> {
        // TODO: 实现访问URL功能
        Ok("Visit URL action executed".to_string())
    }

    async fn _execute_tool_press(&self, _args: serde_json::Value) -> Result<String> {
        // TODO: 实现按键功能
        Ok("Press action executed".to_string())
    }

    async fn _execute_tool_scroll(&self, _args: serde_json::Value) -> Result<String> {
        // TODO: 实现滚动功能
        Ok("Scroll action executed".to_string())
    }

    async fn _execute_tool_screenshot(&self) -> Result<String> {
        // TODO: 实现截图功能
        Ok("Screenshot action executed".to_string())
    }

    async fn _execute_tool_get_page_content(&self) -> Result<String> {
        // TODO: 实现获取页面内容功能
        Ok("Get page content action executed".to_string())
    }



}