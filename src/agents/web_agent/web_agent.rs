use std::collections::HashMap;
use std::sync::Arc;
use thirtyfour::prelude::*;
use anyhow::Result;
use crate::agents::web_agent::types::{FunctionCall, ToolSchema};
use crate::tools::chrome::types::InteractiveRegion;
use crate::types::message::CancellationToken;

#[derive(Debug)]
pub struct WebAgent {
    driver: Option<Arc<WebDriver>>,
    // model_usage: RequestUsage,
}

impl WebAgent {
    pub async fn new() -> Self {
        Self {
            driver:None,
            // model_usage: RequestUsage::Default,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), WebDriverError> {
        let caps = DesiredCapabilities::chrome();
        let driver = WebDriver::new("http://localhost:9515", caps).await?;
        
        
        self.driver = Some(Arc::new(driver));
        Ok(())
    }

    pub fn driver(&self) -> Result<&Arc<WebDriver>, &'static str> {
        self.driver.as_ref()
            .ok_or("Browser context is not initialized. Call initialize() first.")
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
        self,
    ) -> Result<()> {

        // 检查页面存活，可交互，如果失败创建空的页面，避免崩溃

        // 构建对话历史（角色设定，日期，任务目标）

        // 保存用户消息的图像，但是移除agent的图像（节省token）【997-1012】

        // 获取页面可交互元素，生成标记截图（Set-of-Mark）便于使用编号引用元素，而不是CSS选择器【1014-1020】

        // 获取标签页信息（使其支持多标签页）【1039-1044】

        // 动态构建工具列表，根据当前的页面选择哪些工具 【1049-1073】

        // 构造焦点提示，告诉LLM当前那个元素获取了焦点 【1075-1092】

        // 构造可见（视口内的）/不可见元素（下滑，上滑才能看到的）列表， 【1093-1123】

        // 获取页面的纯文本内容

        // 构造最终的提示词，文本状态以及多模态，其中的som_screenshot(带编号的截图，用于定位元素)  screenshot（原始截图，理解页面） 【1127-1185】

        // 上下文进行压缩 【裁剪。。。】是否需要长短期记忆？

        // 调用LLM，工具调用 and json 调用（备用的）

        // 返回结果

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
        let _driver = self.driver()
            .map_err(|e| anyhow::anyhow!("Browser context is not initialized: {}", e))?;

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