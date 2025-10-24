use std::sync::Arc;
use std::path::Path;
use std::time::Duration;
use anyhow::{ Result, Context };
use serde_json;
use tokio::fs;
use serde_json::Value;
use thirtyfour::{DesiredCapabilities, WebDriver, WindowHandle};
use thirtyfour::prelude::*;
use tokio::time::sleep;
use std::collections::HashMap;


use crate::tools::utils::animation_utils::AnimationUtils;
use crate::tools::utils::webpage_text_utils::{WebpageTextUtils};
use crate::tools::chrome::types::{InteractiveRegion, VisualViewport, PageMetadata, TabInfo};

/// Chrome 浏览器控制器
#[derive(Debug)]
pub struct Chrome {
    pub driver: Arc<WebDriver>,
    anim_utils: AnimationUtils,
    animate_actions: bool,
    single_tab_mode: bool,
}

impl Chrome {
    pub async fn new() -> Result<Self> {
        let caps = DesiredCapabilities::chrome();
        let driver = WebDriver::new("http://localhost:9515", caps).await?;

        driver.get("https://www.google.com").await?;

        Ok(Self { 
            driver: Arc::new(driver),
            anim_utils: AnimationUtils::new(),
            animate_actions: true,
            single_tab_mode: true,
        })
    }

    pub async fn sleep(&self, duration: u64) -> Result<()> {
        self.wait_for_page_ready().await?;
        sleep(Duration::from_millis(duration)).await;
        Ok(())
    }

    // 导航到指定的URL(而且智能处理下载文件，将下载的文件保存到指定的文件夹，并显示确认的页面) 暂不进行实现下载逻辑
    pub async fn visit_page(&self, url: &str) -> Result<bool> {
        let _ =  self.wait_for_page_ready();
        self.driver.get(url).await?;
        Ok(true)
    }

    pub async fn get_url(&self) -> Result<String> {
        let url = self.driver.current_url().await?;
        Ok(url.to_string())
    }

    pub async fn get_title(&self) -> Result<String> {
        self.driver.title().await.map_err(|e| e.into())
    }

    pub async fn wait_for_page_ready(&self) -> Result<()> {
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

        Ok(())
    }

    /// 标签页的管理
    pub async fn new_tab(&self, url: &str) -> Result<WindowHandle> {
        let url = url.trim();
        self.driver
            .execute(&format!("window.open('{}', '_blank');", url), vec![])
            .await?;
        
        let handles = self.driver.windows().await?;
        let handle = handles.last().ok_or_else(|| {
            anyhow::anyhow!("Failed to get last window handle")
        })?;
        Ok(handle.clone())
    }

    // 获取标签页所有信息
    /* 
    返回一个包含所有标签页信息的列表，每个标签页信息包含：
    index: 标签页的位置索引
    title: 标签页的标题
    url: 标签页的URL
    is_active: 标签页是否当前可见
    is_controlled: 标签页是否被当前控制
     */
    pub async fn get_tabs_information(&self) -> Result<Vec<TabInfo>> {
        let handles = self.driver.windows().await?;
        let current_handle = self.driver.window().await?;
        let mut tabs_info = Vec::new();
        
        for (index, handle) in handles.iter().enumerate() {
            // 切换到当前标签页以获取信息
            self.driver.switch_to_window(handle.clone()).await?;
            
            let title = self.driver.title().await.unwrap_or_default();
            let url = self.driver.current_url().await?.to_string();
            
            // 检查是否是当前活跃的标签页
            let is_active = handle == &current_handle;
            
            // 检查是否是当前控制的标签页（这里假设当前标签页就是被控制的）
            let is_controlled = handle == &current_handle;
            
            let tab_info = TabInfo {
                index,
                title,
                url,
                is_active,
                is_controlled,
            };
            
            tabs_info.push(tab_info);
        }
        
        // 切换回原来的标签页
        self.driver.switch_to_window(current_handle).await?;
        
        Ok(tabs_info)
    }

    pub async fn switch_tab(&self, index: usize) -> Result<()> {

        let _ = self.wait_for_page_ready().await?;
        let handles = self.driver.windows().await?;
        if index >= handles.len() {
            return Err(anyhow::anyhow!("Index out of bounds: 要切换到索引 {}, 但只有 {} 个标签页", index, handles.len()));
        }
        let handle = handles[index].clone();

        self.driver.switch_to_window(handle).await?;
        Ok(())
    }

    pub async fn close_tab_by_index(&self, index: usize) -> Result<()> {
        let handles = self.driver.windows().await?;
        if index >= handles.len() {
            return Err(anyhow::anyhow!("Index out of bounds: index={}, len={}", index, handles.len()));
        }
        let handle = handles[index].clone();
        self.driver.switch_to_window(handle).await?;
        self.driver.close_window().await?;
        
        // 关闭后，自动切换到第一个可用的标签页（避免焦点处于无效窗口）
        let remaining_handles = self.driver.windows().await?;
        
        if !remaining_handles.is_empty() {
            // 切换到第一个标签页
            self.driver.switch_to_window(remaining_handles[0].clone()).await?;
            println!("已自动切换到索引 0");
        }
        
        Ok(())
    }

    pub async fn go_back(&self) -> Result<()> {
        self.driver.back().await?;
        Ok(())
    }

    async fn go_forward(&self) -> Result<()> {
        self.driver.forward().await?;
        Ok(())
    }

    pub async fn refresh(&self) -> Result<()> {
        self.driver.refresh().await?;
        Ok(())
    }

    /// 滚动管理
    pub async fn page_up(&self) -> Result<()> {
        self.driver.execute("window.scrollBy({ top: -window.innerHeight / 2, behavior: 'smooth' });", vec![]).await?;
        Ok(())
    }

    pub async fn page_down(&self) -> Result<()> {
        self.driver.execute("window.scrollBy({ top: window.innerHeight / 2, behavior: 'smooth' });", vec![]).await?;
        Ok(())
    }

    // 鼠标滚动(需要优化动画)
    pub async fn scroll_mousewheel(&self, dir: &str, pixels: i32) -> Result<()> {
        self.wait_for_page_ready().await?;
        
        let scroll_amount = if dir == "up" { -pixels } else { pixels };
        self.driver.execute(&format!("window.scrollBy({{ top: {}, behavior: 'smooth' }});", scroll_amount), vec![]).await?;
        Ok(())
    }

    pub async fn scroll_element(&self, element_id: &str, dir: &str, pixels: i32) -> Result<()> {
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
    async fn click_coords(&mut self, x: i32, y: i32, button: &str) -> Result<()> {
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
                return Err(anyhow::anyhow!("不支持的按钮类型: {}", button));
            }
        }
        Ok(())
    }

    async fn double_coords(&mut self, x: i32, y: i32) -> Result<()> {
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

    async fn hover_coords(&mut self, x: i32, y: i32) -> Result<()> {
        let (start_x, start_y) = self.anim_utils.last_cursor_position;
        self.anim_utils.gradual_cursor_animation(&self.driver, start_x, start_y, x as f64, y as f64, 10, 50)
            .await?;
        self.driver.as_ref().action_chain()
            .move_to(x.into(), y.into())
            .perform().await?;
        self.anim_utils.cleanup_animations(&self.driver).await?;
        Ok(())
    }

    async fn drag_coords(&mut self, path: Vec<(i32, i32)>) -> Result<()> {
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


    /// 页面信息获取
    // 截图信息
    pub async fn get_screenshot(&self, path: Option<&str>) -> Result<Vec<u8>> {
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
    pub async fn get_interactive_rects(&self) -> Result<HashMap<String,InteractiveRegion>> {

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
            .context("Failed to deserialize interactive rects from JSON")?;

        println!("result: {:?}", result); 

        Ok(result)
    }

    pub async fn select_option(&self, _identifier: &str) -> Result<String> {
        // TODO
        Ok("Select option action executed".to_string())
    }

    // 获取当前适口的尺寸，缩放比例和滚动位置
    async fn get_visual_viewport(&self) -> Result<VisualViewport> {

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
            .ok_or_else(|| anyhow::anyhow!("WebSurfer.getVisualViewport() did not return a JSON object"))?
            .iter()
            .map(|(k,v)|(k.clone(),v.clone()))
            .collect();
        
        // Convert HashMap to VisualViewport
        VisualViewport::visualviewport_from_dict(&viewport_data)
            .context("Failed to convert HashMap to VisualViewport")
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
    async fn get_page_metadata_data(&self) -> Result<PageMetadata> {
        let init_script = include_str!("page_script.js");
        self.driver
            .execute(init_script, Vec::new())
            .await?;

        // 获取元数据
        let result = self.driver
            .execute("return WebSurfer.getPageMetadata();", Vec::new())
            .await?;
        
        // 获取当前页面信息
        let title = self.get_title().await.ok().unwrap_or_default();
        let url = self.get_url().await.ok().unwrap_or_default();
        
        // 解析元数据
        let metadata_json: serde_json::Value = result.json().clone();
        
        // 尝试解析为PageMetadata
        let mut page_metadata = PageMetadata {
            domain: url.parse::<url::Url>().ok().and_then(|u| Some(u.domain()?.to_string())),
            title: Some(title),
            url: Some(url),
            ..Default::default()
        };

        
        // 解析JSON-LD数据
        if let Some(jsonld) = metadata_json.get("jsonld") {
            if let Ok(jsonld_vec) = serde_json::from_value(jsonld.clone()) {
                page_metadata.json_ld = Some(jsonld_vec);
            }
        }
        
        // 解析meta标签
        if let Some(meta_tags) = metadata_json.get("meta_tags") {
            if let Ok(meta_tags_struct) = serde_json::from_value(meta_tags.clone()) {
                page_metadata.meta_tags = Some(meta_tags_struct);
            }
        }
        
        // 解析微数据
        if let Some(microdata) = metadata_json.get("microdata") {
            if let Ok(microdata_vec) = serde_json::from_value(microdata.clone()) {
                page_metadata.microdata = Some(microdata_vec);
            }
        }
        
        Ok(page_metadata)
    }

    async fn get_all_webpage_text(&self,n_lines: Option<usize>) -> Result<String> {
        
        let text_util = WebpageTextUtils::new(self.driver.clone());
        let page_text = text_util
            .get_all_webpage_text(n_lines)
            .await
            .context("Failed to get all webpage text")?;

        Ok(page_text)
    }

    pub async fn get_visible_text(&self) -> Result<String> {
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
    pub async fn get_page_markdown(&self,max_tokens:usize) -> Result<String> {
        
        let markdown_utils = WebpageTextUtils::new(self.driver.clone());
        let markdown = markdown_utils
            .get_page_markdown(max_tokens.try_into().unwrap())
            .await
            .context("Failed to get page markdown")?;
        println!("Markdown content:\n{}",markdown);
        Ok(markdown)
    }
    
    // 生成一个包含页面标题，URL，滚动位置，可见文本和元数据的综合描述，用以向AI代理汇报当前的状态
    pub async fn describe_page(
        &self,
        get_screenshot: bool,
    ) -> Result<(String, Option<Vec<u8>>, String)> {
        // 确保页面已加载完成
        self.wait_for_page_ready().await?;
        
        // 获取截图
        let screenshot = if get_screenshot {
            Some(self.get_screenshot(None).await?)
        } else {
            None
        };
        
        // 获取页面标题和URL
        let page_title = self.get_title().await?;
        let page_url = self.get_url().await?;
        
        // 获取视口信息
        let viewport = self.get_visual_viewport().await?;
        
        // 获取可见文本
        let viewport_text = self.get_visible_text().await?;
        
        // 计算百分比
        let percent_visible = if viewport.scroll_height > 0.0 {
            ((viewport.height * 100.0) / viewport.scroll_height) as i32
        } else {
            100
        };
        
        let percent_scrolled = if viewport.scroll_height > 0.0 {
            ((viewport.page_top * 100.0) / viewport.scroll_height) as i32
        } else {
            0
        };
        
        // 确定位置描述
        let position_text = if percent_scrolled < 1 {
            "at the top of the page".to_string()
        } else if percent_scrolled + percent_visible >= 99 {
            "at the bottom of the page".to_string()
        } else {
            format!("{}% down from the top of the page", percent_scrolled)
        };
        
        // 获取页面元数据
        let page_metadata = self.get_page_metadata_data().await?;
        let metadata_json = serde_json::to_string_pretty(&page_metadata)
            .unwrap_or_else(|_| "{}".to_string());
        
        // 生成元数据哈希
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        metadata_json.hash(&mut hasher);
        let metadata_hash = format!("{:x}", hasher.finish());
        
        // 构建描述消息
        let message_content = format!(
            "We are at the following webpage [{}]({}).\nThe viewport shows {}% of the webpage, and is positioned {}\nThe text in the viewport is:\n {}\n\nThe following metadata was extracted from the webpage:\n\n{}\n",
            page_title, page_url, percent_visible, position_text, viewport_text, metadata_json.trim()
        );
        
        Ok((message_content, screenshot, metadata_hash))
    }

    // 点击具有特定 __elementId 属性的元素。它能处理右键点击、按住点击（在单标签模式下阻止新窗口打开，以及检测点击后触发的下载或新页面） 括号内暂不进行实现
    pub async fn click_id(
        &mut self,
        identifier: &str,   // 特定元素的标号
        _hold: f64,         // 长按（暂未实现）
        button: &str,       // "left" | "right"
    ) -> Result<bool> {

        let _ = self.wait_for_page_ready().await?;

        // 首先检查元素是否存在，如果不存在则先扫描页面
        let element_exists = self.driver.execute(
            &format!(
                r#"
                const element = document.querySelector('[__elementId="{}"]');
                return element !== null;
                "#,
                identifier
            ),
            vec![]
        ).await?;

        let element_exists: bool = element_exists.json().as_bool().unwrap_or(false);
        
        if !element_exists {
            println!("元素 {} 不存在，开始扫描页面...", identifier);
            // 重新扫描页面以获取所有交互元素
            self.get_interactive_rects().await?;
            
            // 再次检查元素是否存在
            let element_exists_after_scan = self.driver.execute(
                &format!(
                    r#"
                    const element = document.querySelector('[__elementId="{}"]');
                    return element !== null;
                    "#,
                    identifier
                ),
                vec![]
            ).await?;
            
            let element_exists_after_scan: bool = element_exists_after_scan.json().as_bool().unwrap_or(false);
            
            if !element_exists_after_scan {
                return Err(anyhow::anyhow!("元素 '{}' 在页面中不存在", identifier));
            }
        }

        // 滚动到元素可见
        self.driver.execute(
            &format!(
                r#"
                const element = document.querySelector('[__elementId="{}"]');
                if (!element) {{
                    throw new Error('Element with ID "{}" not found');
                }}
                element.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                return true;
                "#, 
                identifier, identifier
            ),
            vec![]
        ).await?;

        // 等待让元素滚动完成
        self.sleep(300).await?;

        let rect = self
        .driver
        .execute(
            &format!(
                r#"
                const el = document.querySelector('[__elementId="{}"]');
                if (!el) throw new Error('Element not found');
                const rect = el.getBoundingClientRect();
                return {{ x: rect.left, y: rect.top, width: rect.width, height: rect.height }};
                "#,
                identifier
            ),
            vec![],
        )
        .await?;

        let rect_data: serde_json::Value = rect.json().clone();
        let x = rect_data["x"].as_f64().unwrap_or(0.0);
        let y = rect_data["y"].as_f64().unwrap_or(0.0);
        let width = rect_data["width"].as_f64().unwrap_or(0.0);
        let height = rect_data["height"].as_f64().unwrap_or(0.0);

        let center_x = x + width / 2.0;
        let center_y = y + height / 2.0;

        // 3. 记录原始窗口句柄（用于检测新标签页）
        let original_handles = self.driver.windows().await?;

        // 4. 执行带动画的鼠标移动
        if self.animate_actions {
            self.anim_utils
                .add_cursor_box(&self.driver, identifier)
                .await?;

            let (start_x, start_y) = self.anim_utils.last_cursor_position;
            self.anim_utils
                .gradual_cursor_animation(
                    &self.driver,
                    start_x,
                    start_y,
                    center_x,
                    center_y,
                    10,
                    50,
                )
                .await?;
            self.sleep(100).await?;
        }

        // 5. 执行点击操作
        match button {
            "left" | "right" => {
                let action_chain = self.driver.as_ref().action_chain()
                    .move_to(center_x as i64, center_y as i64);

                let action_chain = if button == "left" {
                    action_chain.click()
                } else {
                    action_chain.context_click()
                };

                action_chain.perform().await?;
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported mouse button"));
            }
        }

        // 6. 清理动画
        if self.animate_actions {
            self.anim_utils
                .remove_cursor_box(&self.driver, identifier)
                .await?;
        }

        // 7. 检测是否打开了新标签页/窗口
        self.sleep(300).await?;
        let current_handles = self.driver.windows().await?;

        let open_new_handle = current_handles
            .iter()
            .any(|h| !original_handles.contains(h));

        Ok(open_new_handle)
    }

    /// 将鼠标悬停在具有特定标识符的元素上
    /// 支持动画效果和普通悬停
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
    pub async fn hover_id(
        &mut self,
        identifier: &str,
    ) -> Result<()> {
        // 确保页面已加载完成
        let _ = self.wait_for_page_ready().await; 
        
        // 滚动到元素可见
        self.driver.execute(
            &format!("document.querySelector('[__elementId=\"{}\"]').scrollIntoView({{ behavior: 'smooth', block: 'center' }});", identifier),
            vec![]
        ).await?;
        
        // 等待一下让滚动完成
        self.sleep(300).await?;
        
        // 获取元素边界框
        let rect = self.driver.execute(
            &format!(
                r#"
                const el = document.querySelector('[__elementId="{}"]');
                if (!el) throw new Error('Element not found');
                const rect = el.getBoundingClientRect();
                return {{ x: rect.left, y: rect.top, width: rect.width, height: rect.height }};
                "#,
                identifier
            ),
            vec![]
        ).await?;
        
        let rect_data: serde_json::Value = rect.json().clone();
        let x = rect_data["x"].as_f64().unwrap_or(0.0);
        let y = rect_data["y"].as_f64().unwrap_or(0.0);
        let width = rect_data["width"].as_f64().unwrap_or(0.0);
        let height = rect_data["height"].as_f64().unwrap_or(0.0);
        
        let end_x = x + width / 2.0;
        let end_y = y + height / 2.0;
        
        // 执行悬停操作
        if self.animate_actions {
            // 添加光标动画
            self.anim_utils.add_cursor_box(&self.driver, identifier).await?;
            
            // 移动光标到元素中心
            let (start_x, start_y) = self.anim_utils.last_cursor_position;
            self.anim_utils.gradual_cursor_animation(
                &self.driver,
                start_x,
                start_y,
                end_x,
                end_y,
                10,
                50
            ).await?;
            
            self.sleep(100).await?;
            
            // 移动到元素中心
            self.driver.action_chain()
                .move_to(end_x as i64, end_y as i64)
                .perform().await?;
            
            // 清理动画效果
            self.anim_utils.remove_cursor_box(&self.driver, identifier).await?;
        } else {
            // 直接移动到元素中心
            self.driver.action_chain()
                .move_to(end_x as i64, end_y as i64)
                .perform().await?;
        }
        
        Ok(())
    }

    /// 向具有特定标识符的元素填充文本(键盘输入)
    /// 适用于文本输入框、文本区域和下拉框
    pub async fn fill_id(
        &mut self,
        identifier: &str,
        value: &str,
        press_enter: bool,
        delete_existing_text: bool,
    ) -> Result<()> {
        // 确保页面已加载完成
        let _ = self.wait_for_page_ready().await;
        
        // 滚动到元素可见
        self.driver.execute(
            &format!("document.querySelector('[__elementId=\"{}\"]').scrollIntoView({{ behavior: 'smooth', block: 'center' }});", identifier),
            vec![]
        ).await?;
        
        // 获取元素边界框
        let rect = self.driver.execute(
            &format!(
                r#"
                const el = document.querySelector('[__elementId="{}"]');
                if (!el) throw new Error('Element not found');
                const rect = el.getBoundingClientRect();
                return {{ x: rect.left, y: rect.top, width: rect.width, height: rect.height }};
                "#,
                identifier
            ),
            vec![]
        ).await?;
        
        let rect_data: serde_json::Value = rect.json().clone();
        let x = rect_data["x"].as_f64().unwrap_or(0.0);
        let y = rect_data["y"].as_f64().unwrap_or(0.0);
        let width = rect_data["width"].as_f64().unwrap_or(0.0);
        let height = rect_data["height"].as_f64().unwrap_or(0.0);
        
        let end_x = x + width / 2.0;
        let end_y = y + height / 2.0;
        
        // 单标签模式：移除target属性防止新标签页
        if self.single_tab_mode {
            self.driver.execute(
                &format!(
                    r#"
                    const el = document.querySelector('[__elementId="{}"]');
                    if (el) el.removeAttribute('target');
                    // 移除所有 <a> 标签的 target 属性
                    document.querySelectorAll('a[target=_blank]').forEach(a => a.removeAttribute('target'));
                    // 移除所有 <form> 标签的 target 属性
                    document.querySelectorAll('form[target=_blank]').forEach(frm => frm.removeAttribute('target'));
                    "#,
                    identifier
                ),
                vec![]
            ).await?;
        }
        
        // 执行填充操作
        if self.animate_actions {
            // 添加光标动画
            self.anim_utils.add_cursor_box(&self.driver, identifier).await?;
            
            // 移动光标到元素中心
            let (start_x, start_y) = self.anim_utils.last_cursor_position;
            self.anim_utils.gradual_cursor_animation(
                &self.driver,
                start_x,
                start_y,
                end_x,
                end_y,
                10,
                50
            ).await?;
            
            self.sleep(100).await?;
        }
        
        // 点击元素获得焦点
        self.driver.action_chain()
            .move_to(end_x as i64, end_y as i64)
            .click()
            .perform().await?;
        
        // 删除现有文本
        if delete_existing_text {
            self.driver.action_chain()
                .key_down(Key::Control)
                .send_keys("a")
                .key_up(Key::Control)
                .send_keys(Key::Backspace)
                .perform().await?;
        }
        
        // 输入文本
        if self.animate_actions {
            // 为短文本使用较慢的输入速度，长文本使用较快的速度
            let delay_ms = if value.len() < 100 { 20 + (30.0 * 0.5) as u64 } else { 5 };
            
            // 逐字符输入以模拟打字效果
            for ch in value.chars() {
                self.driver.action_chain()
                    .send_keys(&ch.to_string())
                    .perform().await?;
                self.sleep(delay_ms).await?;
            }
        } else {
            // 直接输入文本
            self.driver.action_chain()
                .send_keys(value)
                .perform().await?;
        }
        
        // 按回车键
        if press_enter {
            self.sleep(100).await?;
            self.driver.action_chain()
                .send_keys(Key::Enter)
                .perform().await?;
        }
        
        // 清理动画效果
        if self.animate_actions {
            self.anim_utils.remove_cursor_box(&self.driver, identifier).await?;
        }
        
        Ok(())
    }

    pub async fn get_focused_rect_id(&self) -> Result<String> {
        let _ = self.wait_for_page_ready().await;


        let script_exists = self.driver.execute(
            "return typeof WebSurfer !== 'undefined';",
            vec![]
        ).await?;
        
        if !script_exists.json().as_bool().unwrap_or(false) {
            // 如果脚本不存在，先注入
            let init_script = include_str!("page_script.js");
            self.driver.execute(init_script, Vec::new()).await?;
        }

        let focused = self.driver.execute(
            "return WebSurfer.getFocusedElementId();",
            vec![]
        ).await?;
        let focused_id = focused
            .json()
            .as_str()
            .unwrap_or("")
            .to_string();
        Ok(focused_id)
    }

    async fn quit(self) -> Result<()> {
        let _ = Arc::try_unwrap(self.driver)
            .map_err(|_| anyhow::anyhow!("Failed to unwrap driver"))?
            .quit()
            .await
            .context("Failed to quit WebDriver");
        Ok(())
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    // chromedriver --port=9515  
    #[tokio::test]
    async fn test_chrome() -> Result<()> {
        let chrome = Chrome::new().await?;

        let _ = chrome.new_tab("https://www.bilibili.com").await?;
        let _ = chrome.new_tab("https://www.baidu.com").await?;
        let _ = chrome.new_tab("https://www.qq.com").await?;
        let _ = chrome.new_tab("https://www.taobao.com").await?;
        let _ = chrome.new_tab("https://www.jd.com").await?;

        chrome.sleep(3000).await?;

        chrome.switch_tab(3).await?;
        chrome.sleep(2000).await?;
        chrome.close_tab_by_index(0).await?;
        chrome.sleep(2000).await?;
        chrome.switch_tab(3).await?;
        
        chrome.sleep(3000).await?;
        
        // 关闭浏览器
        chrome.quit().await?;
        Ok(())
    }


    #[tokio::test]
    async fn test_click_id() -> Result<()> {
        let mut chrome = Chrome::new().await?;
        let _ = chrome.new_tab("https://www.bilibili.com").await?;
        chrome.switch_tab(0).await?;
        chrome.sleep(2000).await?;

        println!("开始测试点击方法，尝试点击元素 29");
        chrome.click_id("29", 0.0, "right").await?;
        
        chrome.sleep(2000).await?;
        chrome.quit().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_fill_id() -> Result<()> {
        let mut chrome = Chrome::new().await?;
        
        let _ = chrome.new_tab("https://www.bilibili.com").await?;
        chrome.switch_tab(0).await?;
        chrome.sleep(2000).await?;
        
        // 获取交互元素信息
        let interactive_rects = chrome.get_interactive_rects().await?;
        println!("找到 {} 个交互元素", interactive_rects.len());
        
        println!("开始测试fill_id方法，输入: 小约翰可汗");
        chrome.fill_id(
            "19",
            "小约翰可汗",
            true,  // press_enter
            true   // delete_existing_text
        ).await?;
        
        println!("成功输入文本并按下回车");
        chrome.sleep(2000).await?;
        
        // 检查当前URL是否包含搜索内容
        let current_url = chrome.get_url().await?;
        println!("当前URL: {}", current_url);
        
        if current_url.contains("bilibili") {
            println!("测试成功：搜索功能正常工作");
        } else {
            println!("搜索可能没有正确执行");
        }
    
        
        chrome.sleep(2000).await?;
        chrome.quit().await?;
        Ok(())
    }
}
