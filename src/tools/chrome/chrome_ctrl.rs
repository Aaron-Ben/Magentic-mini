use std::sync::Arc;
use std::path::Path;
use std::time::Duration;
use serde_json;
use tokio::fs;
use serde_json::Value;
use thirtyfour::{DesiredCapabilities,WebDriver, WindowHandle};
use thirtyfour::error:: {WebDriverError,WebDriverErrorInfo, WebDriverErrorValue, WebDriverResult};
use crate::tools::utils::animation_utils::AnimationUtils;
use crate::tools::utils::webpage_text_utils::{WebpageTextUtils, WebpageTextError};
use crate::tools::chrome::types::{InteractiveRegion, VisualViewport};
use std::collections::HashMap;

/// Chrome 浏览器控制器
pub struct Chrome {
    driver: Arc<WebDriver>,
    anim_utils: AnimationUtils,
}

impl Chrome {
    pub async fn new() -> Result<Self, WebDriverError> {
        let caps = DesiredCapabilities::chrome();
        let driver = WebDriver::new("http://localhost:9515", caps).await?;
        Ok(Self { 
            driver: Arc::new(driver),
            anim_utils: AnimationUtils::new(),
        })
    }

    pub async fn get_url(&self) -> Result<String,WebDriverError> {
        let url = self.driver.current_url().await?;
        Ok(url.to_string())
    }

    pub async fn get_title(&self) -> Result<String,WebDriverError> {
        self.driver.title().await.map_err(|e| e.into())
    }

    pub async fn wait_for_page_ready(&self) -> Result<(),WebDriverError> {
        self.driver.execute(
            r#"
            return new Promise((resolve) => {
                if (document.readyState === 'complete') {
                    resolve();
                } else {
                    window.addEventListener('load', resolve);
                }
            });
            "#,
            vec![]
        ).await?;

        tokio::time::sleep(Duration::from_micros(1000)).await;
        Ok(())
    }

    /// 标签页的管理
    async fn new_tab(&self, url: &str) -> WebDriverResult<WindowHandle> {
        let url = url.trim();
        self.driver.execute(&format!("window.open('{}', 'www.google.com');", url), vec![]).await?;
        
        let handles = self.driver.windows().await?;
        let handle = handles.last().ok_or_else(|| {
            WebDriverError::UnknownError(WebDriverErrorInfo {
                status: 500,
                error: "no tab find".to_string(),
                value: WebDriverErrorValue {
                    message: "Failed to get new tab handle".to_string(),
                    error: None,
                    stacktrace: None,
                    data: None,
                },
            })
        })?;
        Ok(handle.clone())
    }

    async fn switch_to_tab(&self, handle: &WindowHandle) -> WebDriverResult<()> {
        self.driver.switch_to_window(handle.clone()).await?;
        Ok(())
    }

    async fn close_tab(&self) -> WebDriverResult<()> {
        self.driver.close_window().await?;
        Ok(())
    }

    async fn go_back(&self) -> WebDriverResult<()> {
        self.driver.back().await?;
        Ok(())
    }

    async fn go_forward(&self) -> WebDriverResult<()> {
        self.driver.forward().await?;
        Ok(())
    }

    async fn refresh(&self) -> WebDriverResult<()> {
        self.driver.refresh().await?;
        Ok(())
    }

    /// 滚动管理
    async fn page_up(&self) -> WebDriverResult<()> {
        self.driver.execute("window.scrollBy({ top: -window.innerHeight / 2, behavior: 'smooth' });", vec![]).await?;
        Ok(())
    }

    async fn page_down(&self) -> WebDriverResult<()> {
        self.driver.execute("window.scrollBy({ top: window.innerHeight / 2, behavior: 'smooth' });", vec![]).await?;
        Ok(())
    }

    async fn scroll_custom(&self, dir: &str, pixels: i32) -> WebDriverResult<()> {
        let scroll_amount = if dir == "up" { -pixels } else { pixels };
        self.driver.execute(&format!("window.scrollBy({{ top: {}, behavior: 'smooth' }});", scroll_amount), vec![]).await?;
        Ok(())
    }

    async fn scroll_element(&self, element_id: &str, dir: &str, pixels: i32) -> WebDriverResult<()> {
        let scroll_amount = if dir == "up" { -pixels } else { pixels };
        let script = format!(
            r#"
            (function() {{
                const elem = document.querySelector('[__elementId="{}"]');
                if (elem) {{
                    elem.scrollBy({{ top: {}, behavior: 'smooth' }});
                }} else {{
                    throw new Error('元素未找到');
                }}
            }})()
            "#,
            element_id,
            scroll_amount
        );
        self.driver.execute(&script, vec![]).await?;
        Ok(())
    }

    /// 鼠标管理
    async fn click_coords(&mut self, x: i32, y: i32, button: &str) -> WebDriverResult<()> {
        match button {
            "back" => {
                self.go_back().await?;
            }
            "forward" => {
                self.go_forward().await?;
            }
            "wheel" => {
                let (start_x, start_y) = self.anim_utils.last_cursor_position;
                self.anim_utils.gradual_cursor_animation(&self.driver, start_x, start_y, x as f64, y as f64, 10, 50)
                    .await?;
                self.driver.as_ref().execute(
                    &format!("window.scrollBy({{x: {}, y: {}}});", x, y),
                    vec![],
                ).await?;
                self.anim_utils.cleanup_animations(&self.driver).await?;
            }
            "left" | "right" => {
                let (start_x, start_y) = self.anim_utils.last_cursor_position;
                self.anim_utils.gradual_cursor_animation(&self.driver, start_x, start_y, x as f64, y as f64, 10, 50)
                    .await?;

                let action_chain = self.driver.as_ref().action_chain()
                    .move_to(x.into(), y.into());

                let action_chain = if button == "left" {
                    action_chain.click()
                } else {
                    action_chain.context_click()
                };

                action_chain.perform().await?;
                self.anim_utils.cleanup_animations(&self.driver).await?;
            }
            _ => {
                let error_info = WebDriverErrorInfo::new(format!("不支持的按钮类型: {}", button));
                return Err(WebDriverError::UnknownError(error_info).into());
            }
        }
        Ok(())
    }

    async fn double_coords(&mut self, x: i32, y: i32) -> WebDriverResult<()> {
        let (start_x, start_y) = self.anim_utils.last_cursor_position;
        self.anim_utils.gradual_cursor_animation(&self.driver, start_x, start_y, x as f64, y as f64, 10, 50)
            .await?;
        self.driver.as_ref().action_chain()
            .move_to(x.into(), y.into())
            .double_click()
            .perform().await?;
        self.anim_utils.cleanup_animations(&self.driver).await?;
        Ok(())
    }

    async fn hover_coords(&mut self, x: i32, y: i32) -> WebDriverResult<()> {
        let (start_x, start_y) = self.anim_utils.last_cursor_position;
        self.anim_utils.gradual_cursor_animation(&self.driver, start_x, start_y, x as f64, y as f64, 10, 50)
            .await?;
        self.driver.as_ref().action_chain()
            .move_to(x.into(), y.into())
            .perform().await?;
        self.anim_utils.cleanup_animations(&self.driver).await?;
        Ok(())
    }

    async fn drag_coords(&mut self, path: Vec<(i32, i32)>) -> WebDriverResult<()> {
        if path.is_empty() {
            return Ok(());
        }

        let window_size = self.driver.get_window_rect().await?;
        let mut adjusted_path = Vec::new();

        for &(mut x, mut y) in &path {
            if x < 0 {x = 0};
            if y < 0 {y = 0};
            if i64::from(x) > window_size.width {x = window_size.width as i32};
            if i64::from(y) > window_size.height {y = window_size.height as i32};
            adjusted_path.push((x, y));
        }

        let mut action_chain = self.driver.action_chain();

        // 第一步：移动到起点并按住
        let (start_x, start_y) = adjusted_path[0];
        action_chain = action_chain
            .move_to(start_x.into(), start_y.into())
            .click_and_hold();

        // 第二步：对后续每个点，使用相对位移 move_by
        let mut last_x = start_x;
        let mut last_y = start_y;

        for &(x, y) in &path[1..] {
            self.anim_utils.gradual_cursor_animation(&self.driver, last_x as f64, last_y as f64, x as f64, y as f64, 10, 50).await?;
            let dx = x - last_x;
            let dy = y - last_y;
            action_chain = action_chain.move_by_offset(dx.into(), dy.into());
            last_x = x;
            last_y = y;
        }

        // 第三步：释放鼠标
        action_chain.release().perform().await?;

        self.anim_utils.cleanup_animations(&self.driver).await?;
        Ok(())
    }

    /// 键盘管理



    /// 页面信息获取
    // 截图信息
    async fn get_screenshot(&self, path: Option<&str>) -> WebDriverResult<Vec<u8>> {
        let png_data = self.driver.screenshot_as_png().await?;
        if let Some(path_str) = path {
            let path = Path::new(path_str);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::write(path, &png_data).await?
        }
        Ok(png_data)
    }

    // 扫描页面并返回所有可交互元素的位置，大小和类型信息，这些元素会被注入一个唯一的__elementId,以便后续操作
    async fn get_interactive_rects(&self) -> Result<HashMap<String,InteractiveRegion>, WebDriverError> {

        let init_script = include_str!("page_script.js");
        self.driver
            .execute(init_script, Vec::new())
            .await?;

        // 执行 WebSurfer.getInteractiveRects()
        let json_value = self
            .driver
            .execute("return WebSurfer.getInteractiveRects();", Vec::new())
            .await?;

        let serde_value: serde_json::Value = json_value.json().clone();
        
        // 反序列化 JSON
        let result: HashMap<String, InteractiveRegion> = serde_json::from_value(serde_value.clone())
            .map_err(|e| {
                eprintln!("Failed to deserialize interactive rects: {}", e);
                eprintln!("Raw JSON: {}", serde_value);
                WebDriverError::UnknownError(WebDriverErrorInfo {
                    status: 500,
                    error: "unknown error".to_string(),
                    value: WebDriverErrorValue {
                        message: format!("Failed to parse interactive rects: {}", e),
                        error: None,
                        stacktrace: None,
                        data: None,
                    },
                })
            })?;

        Ok(result)
    }

    // 获取当前适口的尺寸，缩放比例和滚动位置
    async fn get_visual_viewport(&self) -> Result<VisualViewport,WebDriverError> {

        let init_script = include_str!("page_script.js");
        self.driver
            .execute(init_script, Vec::new())
            .await?;

        let result = self.driver
            .execute("return WebSurfer.getVisualViewport();", Vec::new())
            .await?;
        
        // Convert result to HashMap
        let viewport_data: HashMap<String, Value> = result
            .json()
            .as_object()
            .ok_or_else(||WebDriverError::ParseError("Failed to parse viewport data as object".to_string()))?
            .clone()
            .into_iter()
            .map(|(k,v)|(k,v.clone()))
            .collect();
        
        // Convert HashMap to VisualViewport
        VisualViewport::visualviewport_from_dict(&viewport_data)
            .map_err(|e| {
                WebDriverError::UnknownError(WebDriverErrorInfo {
                    status: 500,
                    error: "HashMap to VisualViewport Error".to_string(),
                    value: WebDriverErrorValue {
                        message: format!("HashMap to VisualViewport:Failed to parse interactive rects: {}", e),
                        error: None,
                        stacktrace: None,
                        data: None,
                    },
                })
            }
        )
        
    }
    
    // 提取页面的元数据，如<title>,<meta>
    /* 在javascipt getPageMetadata 返回的字典格式
    {
        "jsonId":[...]      JSON-LD 结构和数据
        "meta_tags":{...}   HTML meta 标签
        "microdata":[...]   HTML5 微数据
    }

    JSON-LD
    ["{\"@context\": \"https://schema.org\", \"@type\": \"WebPage\", ...}", ...]
    HTML meta
    {
    "description": "页面描述",
    "keywords": "关键词1,关键词2",
    "og:title": "Open Graph标题",
    "og:description": "Open Graph描述",
    // ...
    }
    microdata - HTML5
    [{
    "itemType": "https://schema.org/Person",
    "name": "张三",
    "jobTitle": "工程师",
    // ... 其他属性
    }, ...]
    getPageMetadata 返回的确切结构
    {
        "jsonld": [         // 可选：JSON-LD数据数组
            '{"@type": "WebPage", "name": "标题"}',  // 字符串（原始JSON文本）
            '{"@type": "Person", "name": "作者"}'     // 字符串（原始JSON文本）
        ],
        "meta_tags": {      // 可选：HTML meta标签对象
            "title": "页面标题",
            "description": "页面描述", 
            "keywords": "关键词1,关键词2",
            "og:title": "Open Graph标题"
        },
        "microdata": [      // 可选：HTML5微数据对象数组
            {
                "itemType": "https://schema.org/WebPage",
                "name": "页面名",
                "author": {
                    "itemType": "https://schema.org/Person",
                    "name": "作者名"
                }
            }
        ]
    }
    最终的返回应该是metadata = {xxx}
     */
    async fn get_page_metadata(&self) -> Result<(),WebDriverError> {

        let init_script = include_str!("page_script.js");
        self.driver
            .execute(init_script, Vec::new())
            .await?;

        // 获取元数据
        self.driver
            .execute("return WebSurfer.getPageMetadata();", Vec::new())
            .await?;

        Ok(())
    }

    async fn get_all_webpage_text(&self,n_lines: Option<usize>) -> Result<String, WebDriverError> {
        
        let text_util = WebpageTextUtils::new(self.driver.clone());
        let page_text = text_util
            .get_all_webpage_text(n_lines)
            .await
            .map_err(Self::webpage_err_to_webdriver_err)?;

        Ok(page_text)
    }

    async fn get_visible_text(&self) -> Result<String, WebDriverError> {
        let init_script = include_str!("page_script.js");
        self.driver
            .execute(init_script, Vec::new())
            .await?;

        let result = self.driver
            .execute("return WebSurfer.getVisibleText();", Vec::new())
            .await?;
        
        let text = result.json().to_string();

        Ok(text)
    }

    // 网页内容转化为Markdown
    async fn get_page_markdown(&self,max_tokens:usize) -> Result<String, WebDriverError> {
        
        let markdown_utils = WebpageTextUtils::new(self.driver.clone());
        let markdown = markdown_utils
            .get_page_markdown(max_tokens.try_into().unwrap())
            .await
            .map_err(Self::webpage_err_to_webdriver_err)?;
        println!("Markdown content:\n{}",markdown);
        Ok(markdown)
    }
    // 生成一个包含页面标题，URL，滚动位置，可见文本和元数据的综合描述，用以向AI代理汇报当前的状态
    /*
    pub async fn describe_page(&self, get_screenshot: bool) -> (String, Option<Vec<u8>>, String) {
        let window_handle = self.driver.current_window_handle().await
            .map_err(|e| WebDriverError::Custom(format!("Failed to get current window: {}", e)))?;
    

        // 截图
        let screenshot = if get_screenshot {
            Some(self.get_screenshot(None).await)
        } else {
            None
        };
        

        // 获取页面标题和URL
        let page_title = self.get_title();
        let page_url = self.get_url();
        
        // 获取视口信息
        let viewport = self.get_visual_viewport().await;
        
        // 获取可见文本
        let visible_text = self.get_visible_text().await;
        
        // 获取页面元数据
        let page_metadata = self.get_page_metadata().await.unwrap_or_default();
        let metadata_json = serde_json::to_string_pretty(&page_metadata).unwrap_or_default();

        // 使用简单的字符串长度作为哈希
        let metadata_hash = format!("{:x}", metadata_json.len());

        // 计算滚动位置百分比
        let percent_visible = if viewport.unwrap().scroll_height > 0.0 {
            (viewport.unwrap().height * 100.0 / viewport.unwrap().scroll_height) as i32
        } else {
            100
        };
        
        let percent_scrolled = if viewport.unwrap().scroll_height > 0.0 {
            (viewport.unwrap().page_top * 100.0 / viewport.unwrap().scroll_height) as i32
        } else {
            0
        };
        
        // 确定位置描述
        let position_text = if percent_scrolled < 1 {
            String::from("at the top of the page")
        } else if percent_scrolled + percent_visible >= 99 {
            String::from("at the bottom of the page")
        } else {
            format!("{}% down from the top of the page", percent_scrolled)
        };

        // 构建描述消息
        let message_content = format!(
            "We are at the following webpage [{}]({}).\nThe viewport shows {}% of the webpage, and is positioned {}\nThe text in the viewport is:\n{}\nThe following metadata was extracted from the webpage:\n\n{}\n",
            page_title, page_url, percent_visible, position_text, visible_text, metadata_json
        );
        
        (message_content, screenshot, metadata_hash)
    }*/

    async fn quit(self) -> Result<(), WebDriverError> {
        <thirtyfour::WebDriver as Clone>::clone(&self.driver).quit().await?;
        Ok(())
    }

    pub fn webpage_err_to_webdriver_err(webpage_err: WebpageTextError) -> WebDriverError {
        match webpage_err {
            WebpageTextError::WebDriver(inner_err) => inner_err,
            WebpageTextError::Http(inner_err) => WebDriverError::UnknownError(
                WebDriverErrorInfo::new(format!("HTTP请求错误（原WebpageTextError）: {}", inner_err))
            ),
            WebpageTextError::PdfExtract(inner_err) => WebDriverError::UnknownError(
                WebDriverErrorInfo::new(format!("PDF提取错误（原WebpageTextError）: {}", inner_err))
            ),
            WebpageTextError::Io(inner_err) => WebDriverError::UnknownError(
                WebDriverErrorInfo::new(format!("IO操作错误（原WebpageTextError）: {}", inner_err))
            ),
            WebpageTextError::Tiktoken(inner_err) => WebDriverError::UnknownError(
                WebDriverErrorInfo::new(format!("Tokenizer错误（原WebpageTextError）: {}", inner_err))   
            ),
            WebpageTextError::SerdeJson(inner_err) => WebDriverError::UnknownError(
                WebDriverErrorInfo::new(format!("JSON解析错误（原WebpageTextError）: {}", inner_err))
            ),
            WebpageTextError::ExtractText(inner_err) => WebDriverError::UnknownError(
                WebDriverErrorInfo::new(format!("文本提取错误（原WebpageTextError）: {}", inner_err))
            ),
            WebpageTextError::Custom(inner_msg) => WebDriverError::UnknownError(
                WebDriverErrorInfo::new(format!("业务逻辑错误（原WebpageTextError）: {}", inner_msg))
            ),
            WebpageTextError::Html(inner_err) => WebDriverError::UnknownError(
                WebDriverErrorInfo::new(format!("HTML提取错误（原HtmlError）: {}", inner_err))
            ),
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use thirtyfour::error::WebDriverResult;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_chrome() -> WebDriverResult<()> {
        let chrome = Chrome::new().await?;
        let tab = chrome.new_tab("https://www.google.com").await?;
        chrome.switch_to_tab(&tab).await?;
        let cur_url = chrome.get_url().await?;
        println!("当前Url:{}",cur_url);
        sleep(Duration::from_secs(3)).await;
        chrome.get_page_markdown(3000).await?;
        sleep(Duration::from_secs(5)).await;
        // 关闭浏览器
        chrome.quit().await?;
        Ok(())
    }
}


/* 
impl Chrome {
    /// 页面导航与管理
    // 导航到指定的URL，而且智能处理下载文件，将下载的文件保存到指定的文件夹，并显示确认的页面
    pub fn visit_page() -> Result<()> { 
        Ok(())
    }

    // 导航到指定 URL
    pub fn navigate_to(&mut self, url: &str) -> Result<()> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;
            
        tab.navigate_to(url)
            .with_context(|| format!("导航到 {} 失败", url))?;
            
        tab.wait_until_navigated()
            .context("等待页面导航完成失败")?;
            
        info!("成功导航到: {}", url);
        Ok(())
    }

    /// 获取所有标签页的信息
    /* 
    返回一个包含所有标签页信息的列表，每个标签页信息包含：
    index: 标签页的位置索引
    title: 标签页的标题
    url: 标签页的URL
    is_active: 标签页是否当前可见
    is_controlled: 标签页是否被当前控制
     */
    pub async fn get_tabs_information(&self) -> Result<Vec<TabInfo>> {
        let mut tabs_info = Vec::new();
        
        // 遍历所有标签页
        for (tab_id, tab_info) in &self.tabs {
            // 获取标签页的可见状态
            let is_visible = tab_info.tab.evaluate("document.visibilityState", false)
                .ok()
                .and_then(|r| r.value)
                .and_then(|v| v.as_str().map(|s| s == "visible"))
                .unwrap_or(false);
                
            let title = tab_info.tab.get_title().unwrap_or_default();
            let url = tab_info.tab.get_url();
            
            // 更新标签页信息
            let info = TabInfo {
                tab: tab_info.tab.clone(),
                index: tab_info.index,
                title,
                url,
                is_active: is_visible,
                is_controlled: Some(*tab_id) == self.current_tab_id,
            };
            
            tabs_info.push(info);
        }
        
        // 按索引排序
        tabs_info.sort_by_key(|info| info.index);
        
        Ok(tabs_info)
    }

    /// 元素交互
    // 点击具有特定 __elementId 属性的元素。它能处理右键点击、按住点击、在单标签模式下阻止新窗口打开，以及检测点击后触发的下载或新页面
    pub async fn click_id(&mut self, _element_id: &str) -> Result<()> {
        Ok(())
    }
    

    // 寻找目标元素（很重要的方法，涉及多个步骤，未测试）
    /* 其中element_id 代表页面上交互元素的唯一标识符，它是由 page_script.js 中扫描所有的交互元素
       为交互元素分配唯一ID，然后 page_script.js 中的getInteractiveRects 返回所有交互元素及其
       element_id, add_set_of_mark() 创建连续的数字ID映射到原始的elememt_id,工具参数使用ID
       工具参数接收数字ID，通过element_id_mapping 转化为原始的element_id,hover_id 使用element_id
       定位并操作元素
       -JavaScript扫描 → page_script.js 识别交互元素并分配 __elementId
       -元素信息收集 → playwright_controller.py 获取元素矩形和属性信息
       -ID映射创建 → _set_of_mark.py 创建用户友好的数字ID映射
       -工具调用 → _web_surfer.py 接收数字ID并转换为原始ID
       -元素定位 → playwright_controller.py 使用 __elementId 定位元素
       -动画执行 → animation_utils.py 执行光标动画和元素高亮
       -鼠标悬停 → playwright_controller.py 执行实际的鼠标悬停操作
       */
    pub async fn hover_id(&mut self, element_id: &str) -> Result<()> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;
        
        tab.wait_until_navigated().context("等待页面导航完成失败")?;

        // 1. 查找元素
        let _element = tab
            .wait_for_element(&format!(r#"[__elementId="{}"]"#, element_id))
            .with_context(|| format!("未找到元素 ID 为 {} 的元素", element_id))?;

        // 2. 添加黄色边框
        let js_add_border = format!(
            "document.getElementById('{}').style.border = '2px solid yellow';",
            element_id
        );
        tab.evaluate(&js_add_border, false)
            .context("添加黄色边框失败")?;

        // 3. 确保元素在视图内
        let js_scroll_into_view = format!(
            "document.getElementById('{}').scrollIntoView({{ behavior: 'smooth', block: 'center', inline: 'center' }});",
            element_id
        );
        tab.evaluate(&js_scroll_into_view, false)
            .context("滚动元素到视图内失败")?;

        // 4. 进行滚动
        sleep(Duration::from_millis(800)).await;

        // 5. 获取元素中心坐标
        let js_get_center = format!(
            "const el = document.getElementById('{}');
            if (!el) throw new Error('Element not found');
            const rect = el.getBoundingClientRect();
            [rect.left + rect.width / 2, rect.top + rect.height / 2];", element_id
        );

        let coords: Vec<f64> = tab
            .evaluate(&js_get_center, false)?
            .value
            .and_then(|v| v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_f64())
                    .collect()
                }))
                .unwrap_or_default();

        if coords.len() != 2 {
            return Err(anyhow!("无法获取元素中心坐标"));
        }

        let center_x = coords[0] as i32;
        let center_y = coords[1] as i32;

        // 6. 在悬停前移除黄色边框
        let js_remove_border = format!(
            "document.getElementById('{}').style.border = '';",
            element_id
        );
        tab.evaluate(&js_remove_border, false)
            .context("移除黄色边框失败")?;

        // 8.执行悬停
        self.hover_coords(center_x, center_y)
            .await
            .context("悬停到元素中心失败")?;

        Ok(())
    }

    // 向输入框、文本区域或下拉框填充文本。支持先删除现有文本和在输入后按回车键
    pub fn fill_text (&mut self, _element_id: &str, _text: &str) -> Result<()> {
        Ok(())
    }

    // 选择下拉菜单选项
    pub fn select_option(&mut self, _element_id: &str, _option_text: &str) -> Result<()> {
        Ok(())
    }

    // 向输入框上传本地文件
    pub fn upload_file(&mut self, element_id: &str, file_path: &str) -> Result<()> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;
        tab.wait_until_navigated().context("等待页面导航完成失败")?;

        let srcipt = format!(r#"
        
        "#, element_id, file_path);
        self.page_script
            .replace("{{ELEMENT_ID}}", element_id)
            .replace("{{FILE_PATH}}", file_path);
        Ok(())
    }

    /// 键盘操作
 
    async fn set_mark(&self) -> Result<()> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;
            
        let js_script = r#"
            // 查询可交互元素
            const interactiveElements = document.querySelectorAll('a, button, input, select, textarea, [contenteditable], [onclick], [onchange]');
            
            // 元素添加红色边框
            interactiveElements.forEach(el => {
                el.style.border = '2px solid red';
                el.style.boxSizing = 'border-box';
            });
            
            interactiveElements.length;
        "#;
        tab.evaluate(js_script, true)?;
        Ok(())
    }

    async fn clear_mark(&self) -> Result<()> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;
            
        let js_script = r#"
            // 获取所有已添加的边框元素
            const redBorderElements = document.querySelectorAll('[style*="border: 2px solid red"]');
            
            // 移除边框
            redBorderElements.forEach(el => {
                el.style.border = '';
            });
        "#;
        tab.evaluate(js_script, true)?;
        Ok(())
    }

    /// 关闭浏览器
    pub fn close(mut self) -> Result<()> {
        
        // 关闭所有标签页
        for (tab_id, tab_info) in self.tabs.drain() {
            if let Err(e) = tab_info.tab.close(false) {
                warn!("关闭标签页 {} 时出现错误: {:?}", tab_id, e);
            }
        }
        
        drop(self.browser);
        Ok(())
    }
}
*/

