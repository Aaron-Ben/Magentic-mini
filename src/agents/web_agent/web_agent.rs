use std::collections::HashMap;
use std::sync::Arc;
use thirtyfour::prelude::*;
use anyhow::{Result};
use tokio_util::sync::CancellationToken;
use crate::agents::web_agent::types::{FunctionCall,ToolSchema,RequestUsage};
use crate::tools::chrome::types::InteractiveRegion;
use crate::agents::web_agent::types::LLMResponse;

#[derive(Debug)]
pub struct WebAgent {
    driver: Option<Arc<WebDriver>>,
    model_usage: RequestUsage,
}

impl WebAgent {
    pub async fn new() -> Self {
        Self {
            driver:None,
            model_usage: RequestUsage::Default,
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

    /* 观察当前浏览器的状态，构造提示词，调用LLM，返回下一步要执行的动作（思考），以及上下文信息*/
    pub async fn get_llm_response(
        self,
        cancellation_token: Option<CancellationToken>
    ) -> Result<LLMResponse<T>> {

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

    // web_agent的核心，接收用户或者，ent的消息，驱动浏览器进行一系列的操作，并将操作以流的形式（AsyncGenerator）逐步
    pub async fn on_messages_steam(
        self,
        message: Vec<xxx>,
        cancellation_token: Option<CancellationToken>,
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
    
    pub async fn executor_tool(
        &self,
        messages:Vec<FunctionCall<T>>,              // 提取工具的名称
        rect: HashMap<String,InteractiveRegion>,    // 主要传递给需要与页面元素交互的工具
        tools: Vec<ToolSchema>,                     // 工具列表
        element_id_mapping: HashMap<String,String>, // 为页面元素提供ID映射
        cancellation_token: Option<CancellationToken>,  // 支持异步操作的取消功能
    ) -> Result<()> {
        // 确保浏览器上下文和页面已准备好，保证仅有一个FunctionCal（为了一次执行一个动作）
        let drive = self.driver()
            .map_err(|e| format!("Content error: {}",e));

        if messages.len() != 1 {
            return Err("Expected exactly one function call".into());
        }

        // 从 function call 中获取参数(工具的名称[name] 和 参数[arguments])

        let name = messages[0].name.clone();
        let args: serde_json::Value = serde_json::from_str(&messages[0].arguments)?;

        log::debug!(
            "Executing tool: {} with args: {:?}",
            name, args
        );

        let tools_func_name = format!("execute_tool_{}",name);

        // 将工具调用使用文本表示（例如：click{target_id:"123"}）添加到inner_messages列表中，这些消息最终会作为“内部思考”过程的一部分返回给用户或其他代理

        
        // 查找Webagent中是否存在具体的工具方法，列出所有的方法供LLM选择

        // match FunctionCall

        // 创建一个字典，用以向执行函数传递参数


        // 创建异步认为执行工具函数（注意：每一次仅有一个工具被调用），再启动一个任务，用于监听工具的执行，因为有一个功能（可以取消任务执行）

        // let tool_task = async move {
        //     let tool_result = tool_function(tool_params).await;
        //     tool_result
        // };

        // success: 工具执行返回的描述字符串（有下载任务，也加入下载信息）
        // fail: 清理残留的动画效果，返回一个说明操作被取消的字符串
        

        // 将描述的字符串返回给on_messages_steam(这个函数把结果返回给LLM，用于指导操作)
        /*
        
        let function_call = &message[0];
        let name = &function_call.name;
        let args: Value = from_str(&function_call.arguments)?;
    
        
        // 验证工具是否存在
        let available_tools: Vec<String> = tools.iter()
            .map(|tool| tool.name.clone())
            .collect();
        
        if !available_tools.contains(name) {
            let tool_names = available_tools.join(", ");
            return Err(format!("Unknown tool '{}'. Please choose from:\n\n{}", name, tool_names).into());
        }
        
        // 创建工具执行任务
        let tool_result = self.execute_specific_tool(
            name,
            args,
            rects,
            element_id_mapping,
            cancellation_token,
        ).await?;
     // 处理下载相关逻辑
        let mut result = tool_result;
        // if let Some(last_download) = &self.last_download {
        //     if let Some(download_folder) = &self.downloads_folder {
        //         result.push_str(&format!(
        //             "\n\nSuccessfully downloaded '{}' to local path: {}",
        //             last_download.suggested_filename,
        //             download_folder
        //         ));
        //     }
        // }
        
        Ok(result) */

        Ok(())
    }

    pub async fn execute_tool_press() -> String {
        "".to_string()
    }



}