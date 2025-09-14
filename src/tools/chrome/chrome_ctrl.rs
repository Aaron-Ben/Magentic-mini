use std::error::Error;
use std::sync::Arc;
use thirtyfour::{DesiredCapabilities,WebDriver, WindowHandle};
use thirtyfour::error:: {WebDriverError,WebDriverErrorInfo};
use crate::tools::utils::animation_utils::AnimationUtils;

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

    /// 标签页的管理
    async fn new_tab(&self, url: &str) -> Result<WindowHandle, Box<dyn Error + Send + Sync>> {
        let url = url.trim();
        self.driver.execute(&format!("window.open('{}', 'www.google.com');", url), vec![]).await?;
        let handles = self.driver.windows().await?;
        Ok(handles.last().ok_or("无法获取新标签页句柄")?.clone())
    }

    async fn switch_to_tab(&self, handle: &WindowHandle) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.driver.switch_to_window(handle.clone()).await?;
        Ok(())
    }

    async fn close_tab(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.driver.close_window().await?;
        Ok(())
    }

    async fn go_back(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.driver.back().await?;
        Ok(())
    }

    async fn go_forward(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.driver.forward().await?;
        Ok(())
    }

    async fn refresh(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.driver.refresh().await?;
        Ok(())
    }

    /// 滚动管理
    async fn page_up(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.driver.execute("window.scrollBy({ top: -window.innerHeight / 2, behavior: 'smooth' });", vec![]).await?;
        Ok(())
    }

    async fn page_down(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.driver.execute("window.scrollBy({ top: window.innerHeight / 2, behavior: 'smooth' });", vec![]).await?;
        Ok(())
    }

    async fn scroll_custom(&self, dir: &str, pixels: i32) -> Result<(), Box<dyn Error + Send + Sync>> {
        let scroll_amount = if dir == "up" { -pixels } else { pixels };
        self.driver.execute(&format!("window.scrollBy({{ top: {}, behavior: 'smooth' }});", scroll_amount), vec![]).await?;
        Ok(())
    }

    async fn scroll_element(&self, element_id: &str, dir: &str, pixels: i32) -> Result<(), Box<dyn Error + Send + Sync>> {
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
    async fn click_coords(&mut self, x: i32, y: i32, button: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
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

    async fn double_coords(&mut self, x: i32, y: i32) -> Result<(), Box<dyn Error + Send + Sync>> {
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

    async fn hover_coords(&mut self, x: i32, y: i32) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (start_x, start_y) = self.anim_utils.last_cursor_position;
        self.anim_utils.gradual_cursor_animation(&self.driver, start_x, start_y, x as f64, y as f64, 10, 50)
            .await?;
        self.driver.as_ref().action_chain()
            .move_to(x.into(), y.into())
            .perform().await?;
        self.anim_utils.cleanup_animations(&self.driver).await?;
        Ok(())
    }

    async fn drag_coords(&mut self, path: Vec<(i32, i32)>) -> Result<(), Box<dyn Error + Send + Sync>> {
        if path.is_empty() {
            return Ok(());
        }

        let window_size = self.driver.get_window_rect().await?;
        let mut adjusted_path = Vec::new();

        for &(mut x, mut y) in &path {
            if (x < 0) {x = 0};
            if (y < 0) {y = 0};
            if (i64::from(x) > window_size.width) {x = window_size.width as i32};
            if (i64::from(y) > window_size.height) {y = window_size.height as i32};
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

    async fn quit(self) -> Result<(), WebDriverError> {
        <thirtyfour::WebDriver as Clone>::clone(&self.driver).quit().await?;
        Ok(())
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_chrome() -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut chrome = Chrome::new().await?;
        let tab1 = chrome.new_tab("https://www.bilibili.com").await?;
        sleep(Duration::from_secs(2)).await;
        let tab2 = chrome.new_tab("https://www.baidu.com").await?;
        sleep(Duration::from_secs(2)).await;
        chrome.switch_to_tab(&tab1).await?;
        sleep(Duration::from_secs(2)).await;
        chrome.page_down().await?;
        sleep(Duration::from_secs(2)).await;
        chrome.page_up().await?;
        sleep(Duration::from_secs(2)).await;
        chrome.scroll_custom("down", 1000).await?;
        sleep(Duration::from_secs(2)).await;
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



    /// 获取当前页面的 URL
    pub fn get_current_url(&self) -> Result<String> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;
            
        Ok(tab.get_url())
    }

    /// 获取当前页面的标题
    pub fn get_page_title(&self) -> Result<String> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;
            
        let result = tab.evaluate("document.title", false)
            .context("获取页面标题失败")?;
            
        let title = result.value
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "无标题".to_string());
            
        Ok(title)
    }

    /// 创建新的标签页
    pub fn new_tab(&mut self, url: &str) -> Result<TabInfo> {
        let new_tab = self.browser.new_tab()
            .context("创建新标签页失败")?;
            
        new_tab.wait_until_navigated()
            .context("等待新标签页加载完成失败")?;

        // 激活标签页，使其成为当前活动的标签页
        let navigate_result = new_tab.navigate_to(url);
        let url_result = match navigate_result {
            Ok(_) => {
                new_tab.wait_until_navigated().context("等待新标签页加载完成失败")?;
                new_tab.get_url()
            }
            Err(_) => String::new(),
        };

        let title = new_tab.get_title().unwrap_or_default();
        let index = self.tabs.len();
        let tab_id = self.create_tab_id();
        let tab_info = TabInfo::new(new_tab, index, title, url_result, true);

        // 更新当前活跃标签页状态
        if let Some(current_tab_id) = self.current_tab_id {
            if let Some(current_tab_info) = self.tabs.get_mut(&current_tab_id) {
                current_tab_info.is_active = false;
            }
        }

        // 插入新标签页
        self.tabs.insert(tab_id, tab_info.clone());
        self.current_tab_id = Some(tab_id);
        self.current_tab = Some(tab_info.tab.clone());

        info!("创建新标签页: ID={}, URL={}", tab_id, url);

        Ok(tab_info)
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
    
    
    /// 关闭指定标签页
    pub fn close_tab(&mut self, tab_id: usize) -> Result<()> {
        if self.tabs.len() <= 1 {
            return Err(anyhow!("无法关闭最后一个标签页"));
        }
        
        let tab_info = self.tabs.remove(&tab_id)
            .ok_or_else(|| anyhow!("标签页 {} 不存在", tab_id))?;
            
        // 关闭标签页
        if let Err(e) = tab_info.tab.close(false) {
            warn!("关闭标签页时出现错误: {:?}", e);
        }
        
        // 如果关闭的是当前活跃标签页，切换到第一个可用标签页
        if let Some((new_tab_id, _)) = self.tabs.iter().next() {
            let new_tab_id = *new_tab_id;
            self.switch_tab(new_tab_id)?;
        } else {
            self.current_tab_id = None;
            self.current_tab = None;
        }

        self.reindex_tabs();
        
        info!("标签页 {} 已关闭", tab_id);
        Ok(())
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

    /// 页面滚动


    // 滚动指定的元素，例如内部的滚动条
    pub fn scroll_element(&mut self, element_id: &str, dir: &str, pixels: Option<i32>) -> Result<()> {
        let pixels = pixels.unwrap_or(100); // 默认滚动100像素
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;

        tab.wait_until_navigated().context("等待页面导航完成失败")?;

        let js_script = format!(
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
            pixels * if dir == "up" { -1 } else { 1 }
        );
        tab.evaluate(&js_script, true)
            .context("执行元素滚动操作失败")?;
        std::thread::sleep(Duration::from_millis(500)); // 等待滚动动画完成

        Ok(())
    }

    // 键盘操作
    pub async fn keypress(&self, keys: Vec<&str>) -> Result<()> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;

        tab.wait_until_navigated()
            .context("等待页面导航完成失败")?;
            
        let mapped_keys: Vec<&str> = keys
            .into_iter()
            .map(|key| {
                CUA_KEY_TO_CHROMIUM_KEY
                .get(key.to_lowercase().as_str())
                .copied()
                .unwrap_or(key)
            })
            .collect();

        // 首先获取当前焦点元素
        let js = format!(
            r#"
            (function() {{
                // 如果已有焦点元素，使用它
                const activeElement = document.activeElement;
                if (activeElement && activeElement !== document.body) {{
                    return activeElement;
                }}
                
                // 否则尝试找到一个可输入的元素
                const inputElement = document.querySelector('input[type="text"], input[type="search"], textarea, [contenteditable="true"]');
                if (inputElement) {{
                    inputElement.focus();
                    return inputElement;
                }}
                
                return document.body;
            }})();
            "#
        );
        
        // 执行 JS 获取目标元素
        tab.evaluate(&js, false)
            .context("获取输入元素失败")?;
            
        for key in &mapped_keys {
            // 对于每个键都模拟完整的键盘事件序列
            let js = format!(
                r#"
                (function() {{
                    const target = document.activeElement;
                    const key = '{}';
                    
                    // keydown 事件
                    const keydownEvent = new KeyboardEvent('keydown', {{
                        key,
                        code: key.length === 1 ? 'Key' + key.toUpperCase() : key,
                        bubbles: true,
                        cancelable: true,
                    }});
                    target.dispatchEvent(keydownEvent);
                    
                    // 如果是单个字符，触发 input 事件
                    if (key.length === 1) {{
                        if (target.value !== undefined) {{
                            target.value += key;
                            const inputEvent = new InputEvent('input', {{
                                bubbles: true,
                                cancelable: true,
                                data: key,
                                inputType: 'insertText',
                            }});
                            target.dispatchEvent(inputEvent);
                        }}
                    }}
                    
                    // keyup 事件
                    const keyupEvent = new KeyboardEvent('keyup', {{
                        key,
                        code: key.length === 1 ? 'Key' + key.toUpperCase() : key,
                        bubbles: true,
                        cancelable: true,
                    }});
                    target.dispatchEvent(keyupEvent);
                }})();
                "#,
                key
            );
            
            tab.evaluate(&js, false)
                .context("键盘事件触发失败")?;
        }

        Ok(())
    }

    /// 获取页面的信息（非常重要的一系列方法）
    // 获取当前页面的截图(仅仅字节信息即可)
    async fn get_screenshot(&self) -> Result<Vec<u8>> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;
            
        let screenshot = tab.capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            false
        )?;
        Ok(screenshot)
    }

    // 扫描页面并返回所有可交互元素的位置，大小和类型信息，这些元素会被注入一个唯一的__elementId,以便后续操作
    async fn get_interactive_rects(&self) -> Result<InteractiveRegion> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;

        // 注入
        let script_path = std::path::Path::new(file!())
            .parent()
            .unwrap()
            .join("page_script.js");
        let page_script = std::fs::read_to_string(script_path)
            .unwrap_or_else(|_| String::new());
        
        // 执行
        if let Err(e) = tab.evaluate(&page_script, false) {
            warn!("Failed to inject page script: {:?}", e);
        }
        
        // 获取交互区域
        let result = tab.evaluate("WebSurfer.getInteractiveRects();", false)
            .context("Failed to get interactive rects")?;
        
        let interactive_rects = result.value
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        
        Ok(interactive_rects)
    }

    // 获取当前适口的尺寸，缩放比例和滚动位置
    async fn get_visual_viewport(&self) -> Result<VisualViewport> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;

        let script_path = std::path::Path::new(file!())
            .parent()
            .unwrap()
            .join("page_script.js");
        let page_script = std::fs::read_to_string(script_path)
            .unwrap_or_else(|_| String::new());

        if let Err(e) = tab.evaluate(&page_script, false) {
            warn!("Failed to inject page script: {:?}", e);
        }

        let result = tab.evaluate("WebSurfer.getVisualViewport();", false)
            .context("Failed to get visual viewport")?;

        let visual_viewport = result.value
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        Ok(visual_viewport)
    }

    // 获取页面的元数据（title, mata等标签）
    async fn get_page_metadata(&self) -> Result<PageMetadata> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;
        
        let script_path = std::path::Path::new(file!())
            .parent()
            .unwrap()
            .join("page_script.js");

        let page_script = std::fs::read_to_string(script_path).unwrap_or_else(|_| String::new());

        if let Err(e) = tab.evaluate(&page_script, false) {
            warn!("Failed to inject page script: {:?}", e);
        }

        // 获取元数据
        let result = tab.evaluate("WebSurfer.getPageMetadata();", false)
            .context("Failed to get page metadata")?;

        let metadata: PageMetadata = result.value
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        Ok(metadata)
    }

    // 获取整个页面的纯文本
    async fn get_page_text(&self) -> Result<String> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;
            
        let utils = WebpageTextUtils::new();
        let tab_arc = tab.clone();
        let result = utils.get_all_webpage_text(&tab_arc, 50).await;
        Ok(result)
    }

    // 获取当前视图的可见文本
    pub async fn get_visible_text(&self, tab: &Arc<Tab>) -> Result<String> {
        let utils = WebpageTextUtils::new();
        let result = utils.get_visible_text(tab).await;
        Ok(result)
    }

    // 转化为markdown
    pub async fn convert_to_markdown(&self) -> Result<String> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;
            
        let utils = WebpageTextUtils::new();
        let tab_arc = tab.clone();
        let result = utils.get_page_markdown(&tab_arc, 50).await;
        Ok(result)
    }

    // 生成一个包含页面标题，URL，滚动位置，可见文本和元数据的综合描述，用以向AI代理汇报当前的状态
    pub async fn describe_page(&self, get_screenshot: bool) -> (String, Option<Vec<u8>>, String) {
        let tab = if let Some(tab) = self.current_tab.as_ref() {
            let _ = tab.wait_until_navigated();
            tab
        } else {
            return (String::from("No active tab"), None, String::from(""));
        };

        // 截图
        let screenshot = if get_screenshot {
            tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, false)
                .ok()
        } else {
            None
        };
        

        // 获取页面标题和URL
        let page_title = self.get_page_title().unwrap_or_default();
        let page_url = self.get_current_url().unwrap_or_default();
        
        // 获取视口信息
        let viewport = self.get_visual_viewport().await.unwrap_or_default();
        
        // 获取可见文本
        let visible_text = self.get_visible_text(tab).await.unwrap_or_default();
        
        // 获取页面元数据
        let page_metadata = self.get_page_metadata().await.unwrap_or_default();
        let metadata_json = serde_json::to_string_pretty(&page_metadata).unwrap_or_default();

        // 使用简单的字符串长度作为哈希
        let metadata_hash = format!("{:x}", metadata_json.len());

        // 计算滚动位置百分比
        let percent_visible = if viewport.scroll_height > 0.0 {
            (viewport.height * 100.0 / viewport.scroll_height) as i32
        } else {
            100
        };
        
        let percent_scrolled = if viewport.scroll_height > 0.0 {
            (viewport.page_top * 100.0 / viewport.scroll_height) as i32
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
    }
 
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

