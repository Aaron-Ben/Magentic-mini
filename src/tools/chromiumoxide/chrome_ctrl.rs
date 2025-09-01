use chromiumoxide::{Browser, BrowserConfig, Page, EvaluationResult, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use anyhow::{Result, anyhow};
use log::{info, error};
use std::fs;
use std::path::Path;

use super::types::{DOMRectangle, VisualViewport, InteractiveRegion};
use super::chrome_state::{save_browser_state, load_browser_state, BrowserState};
use crate::tools::utils::animation_utils::AnimationUtils;
use crate::tools::utils::webpage_text_utils::WebpageTextUtils;

static PAGE_SCRIPT: &str = include_str!("page_script.js");

pub struct ChromiumoxideController {
    browser: Option<Browser>,
    current_page: Option<Page>,
    animate_actions: bool,
    viewport_width: u32,
    viewport_height: u32,
    timeout_load: Duration,
    sleep_after_action: Duration,
    single_tab_mode: bool,
    // 添加工具实例
    animation_utils: AnimationUtils,
    webpage_text_utils: WebpageTextUtils,
}

impl ChromiumoxideController {
    pub fn new(
        animate_actions: bool,
        viewport_width: u32,
        viewport_height: u32,
        timeout_load: Duration,
        sleep_after_action: Duration,
        single_tab_mode: bool,
    ) -> Self {
        ChromiumoxideController {
            browser: None,
            current_page: None,
            animate_actions,
            viewport_width,
            viewport_height,
            timeout_load,
            sleep_after_action,
            single_tab_mode,
            // Initialize WebpageTextUtils
            webpage_text_utils: WebpageTextUtils::new(),
            // Create animation utils instance
            animation_utils: AnimationUtils::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        let config = BrowserConfig::builder()
            .window_size(self.viewport_width, self.viewport_height)
            .build()
            .map_err(|e| anyhow!("Failed to build browser config: {}", e))?;

        let (browser, mut handler) = Browser::launch(config).await?;
        self.browser = Some(browser);

        // 在新线程中处理事件
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                // 处理浏览器事件
                match event {
                    Ok(_) => {},
                    Err(e) => error!("Browser event error: {}", e),
                }
            }
        });

        info!("ChromiumoxideController initialized with animation and text utils");
        Ok(())
    }

    pub async fn new_page(&mut self, url: Option<&str>) -> Result<&mut Page> {
        let browser = self.browser.as_ref().ok_or_else(|| anyhow!("Browser not initialized"))?;
        let page = browser.new_page(url.unwrap_or("about:blank")).await?;
        
        // 注入页面脚本
        page.execute(PAGE_SCRIPT).await?;
        
        // 设置视口大小
        page.set_viewport_size(self.viewport_width, self.viewport_height).await?;
        
        self.current_page = Some(page);
        Ok(self.current_page.as_mut().unwrap())
    }

    pub async fn visit_page(&mut self, url: &str) -> Result<()> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;

        timeout(self.timeout_load, page.goto(url)).await
            .map_err(|_| anyhow!("Page load timeout"))?
            .map_err(|e| anyhow!("Page load error: {}", e))?;

        page.wait_for_load_state().await?;

        if !self.sleep_after_action.is_zero() {
            tokio::time::sleep(self.sleep_after_action).await;
        }

        Ok(())
    }

    // 截图
    pub async fn get_screenshot(&mut self, path: Option<&str>) -> Result<Vec<u8>> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        let screenshot = page.screenshot().await?;
        
        if let Some(file_path) = path {
            fs::write(file_path, &screenshot)?;
        }
        
        Ok(screenshot)
    }


    // 文本提取方法
    // 使用 page_script.js 的 getPageMarkdown 功能
    pub async fn get_page_markdown(&mut self, max_tokens: Option<usize>) -> Result<String> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        // 首先尝试使用 page_script.js 的简化版本
        let result: EvaluationResult<String> = page.evaluate("WebSurfer.getPageMarkdown()").await?;
        let markdown = result.into_value()?;
        
        // 如果指定了 token 限制，进行截断
        match max_tokens {
            Some(limit) => {
                if markdown.len() > limit * 4 {
                    Ok(markdown.chars().take(limit * 4).collect())
                } else {
                    Ok(markdown)
                }
            },
            None => Ok(markdown),
        }
    }

    // 使用 WebpageTextUtils 获取更完整的页面 Markdown（保留原有功能）
    pub async fn get_complete_page_markdown(&mut self, max_tokens: Option<usize>) -> Result<String> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        let max_tokens_i32 = max_tokens.map(|t| t as i32).unwrap_or(-1);
        self.webpage_text_utils.get_page_markdown(page, max_tokens_i32).await
    }

    // 使用 WebpageTextUtils 获取清理后的页面 Markdown
    pub async fn get_clean_page_markdown(&mut self, max_tokens: Option<usize>) -> Result<String> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        let max_tokens_i32 = max_tokens.map(|t| t as i32).unwrap_or(-1);
        self.webpage_text_utils.get_clean_page_markdown(page, max_tokens_i32).await
    }

    // 焦点元素方法
    pub async fn get_focused_rect_id(&mut self) -> Result<String> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        let result: EvaluationResult<String> = page.evaluate("WebSurfer.getFocusedElementId()").await?;
        Ok(result.into_value()?)
    }

    // 页面元数据方法
    pub async fn get_page_metadata(&mut self) -> Result<HashMap<String, serde_json::Value>> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        let result: EvaluationResult<HashMap<String, serde_json::Value>> = 
            page.evaluate("WebSurfer.getPageMetadata()").await?;
        
        Ok(result.into_value()?)
    }

    pub async fn go_back(&mut self) -> Result<bool> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        let result = timeout(self.timeout_load, page.go_back()).await;
        match result {
            Ok(Ok(_)) => Ok(true),
            Ok(Err(_)) | Err(_) => Ok(false),
        }
    }

    pub async fn go_forward(&mut self) -> Result<bool> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        let result = timeout(self.timeout_load, page.go_forward()).await;
        match result {
            Ok(Ok(_)) => Ok(true),
            Ok(Err(_)) | Err(_) => Ok(false),
        }
    }

    // 刷新
    pub async fn refresh_page(&mut self) -> Result<()> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        page.reload().await?;
        page.wait_for_load_state().await?;
        
        Ok(())
    }

    // 页面滚动
    pub async fn page_down(&mut self) -> Result<()> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        // 获取视口高度
        let viewport = self.get_visual_viewport().await?;
        let scroll_distance = viewport.height as i32;
        
        page.evaluate(&format!("window.scrollBy(0, {});", scroll_distance)).await?;
        
        if !self.sleep_after_action.is_zero() {
            tokio::time::sleep(self.sleep_after_action).await;
        }
        
        Ok(())
    }


    pub async fn click_coords(&mut self, x: f64, y: f64) -> Result<()> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        if self.animate_actions {
            self.animate_cursor_to(x, y).await?;
        }
        
        page.mouse().move_to(x, y).await?;
        page.mouse().down().await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        page.mouse().up().await?;
        
        if !self.sleep_after_action.is_zero() {
            tokio::time::sleep(self.sleep_after_action).await;
        }
        
        Ok(())
    }

    pub async fn fill_id(&mut self, element_id: &str, value: &str) -> Result<()> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        let selector = format!("[__elementId='{}']", element_id);
        
        // 等待元素出现
        timeout(self.timeout_load, page.wait_for_selector(&selector)).await
            .map_err(|_| anyhow!("Element wait timeout"))?
            .map_err(|e| anyhow!("Element not found: {}", e))?;
        
        // 聚焦元素
        page.focus_selector(&selector).await?;
        
        // 清空现有内容并填充新值
        page.keyboard().press("Control+a").await?;
        page.keyboard().press("Delete").await?;
        page.keyboard().r#type(value, None).await?;
        
        if !self.sleep_after_action.is_zero() {
            tokio::time::sleep(self.sleep_after_action).await;
        }
        
        Ok(())
    }

    // 悬停
    pub async fn hover_coords(&mut self, x: f64, y: f64) -> Result<()> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        page.mouse().move_to(x, y).await?;
        
        if !self.sleep_after_action.is_zero() {
            tokio::time::sleep(self.sleep_after_action).await;
        }
        
        Ok(())
    }

    // 键盘按键
    pub async fn keypress(&mut self, keys: &[String]) -> Result<()> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        for key in keys {
            page.keyboard().press(key).await?;
        }
        
        if !self.sleep_after_action.is_zero() {
            tokio::time::sleep(self.sleep_after_action).await;
        }
        
        Ok(())
    }

    // 直接输入
    pub async fn type_direct(&mut self, text: &str) -> Result<()> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        page.keyboard().r#type(text, None).await?;
        
        if !self.sleep_after_action.is_zero() {
            tokio::time::sleep(self.sleep_after_action).await;
        }
        
        Ok(())
    }

    // 双击
    pub async fn double_click_coords(&mut self, x: f64, y: f64) -> Result<()> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        page.mouse().move_to(x, y).await?;
        page.mouse().dblclick().await?;
        
        if !self.sleep_after_action.is_zero() {
            tokio::time::sleep(self.sleep_after_action).await;
        }
        
        Ok(())
    }

    pub async fn get_current_url_title(&mut self) -> Result<(String, String)> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        let url = page.url().await?;
        let title = page.title().await?;
        
        Ok((url, title))
    }

    // 睡眠
    pub async fn sleep(&mut self, duration: Duration) -> Result<()> {
        tokio::time::sleep(duration).await;
        Ok(())
    }

    // 使用 page_script.js 的 getVisibleText 功能
    pub async fn get_visible_text(&mut self) -> Result<String> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        let result: EvaluationResult<String> = page.evaluate("WebSurfer.getVisibleText()").await?;
        Ok(result.into_value()?)
    }

    // 使用 WebpageTextUtils 获取所有网页文本
    pub async fn get_all_webpage_text(&mut self, n_lines: usize) -> Result<String> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        self.webpage_text_utils.get_all_webpage_text(page, n_lines).await
    }

    // 检查是否为 PDF 页面
    pub async fn is_pdf_page(&mut self) -> Result<bool> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        self.webpage_text_utils.is_pdf_page(page).await
    }

    // 提取 PDF 内容
    pub async fn extract_pdf_content(&mut self) -> Result<String> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        self.webpage_text_utils.extract_pdf_content(page).await
    }

    // 使用统一的浏览器状态管理
    pub async fn save_browser_state(&mut self, simplified: bool) -> Result<BrowserState> {
        let browser = self.browser.as_ref().ok_or_else(|| anyhow!("Browser not initialized"))?;
        save_browser_state(browser, self.current_page.as_ref(), simplified).await
    }

    pub async fn load_browser_state(&mut self, state: BrowserState, load_only_active_tab: bool) -> Result<()> {
        let browser = self.browser.as_ref().ok_or_else(|| anyhow!("Browser not initialized"))?;
        load_browser_state(browser, state, load_only_active_tab).await
    }

    // 视口信息 - 使用正确的字段映射
    pub async fn get_visual_viewport(&mut self) -> Result<VisualViewport> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        let result: EvaluationResult<VisualViewport> = page.evaluate(r#"
            ({
                height: window.innerHeight,
                width: window.innerWidth,
                offsetLeft: window.pageXOffset,
                offsetTop: window.pageYOffset,
                pageLeft: window.pageXOffset,
                pageTop: window.pageYOffset,
                scale: window.devicePixelRatio,
                clientWidth: document.documentElement.clientWidth,
                clientHeight: document.documentElement.clientHeight,
                scrollWidth: document.documentElement.scrollWidth,
                scrollHeight: document.documentElement.scrollHeight,
            })
        "#).await?;
        
        Ok(result.into_value()?)
    }

    // 滚动方法
    pub async fn scroll_coords(&mut self, x: i32, y: i32) -> Result<()> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        page.evaluate(&format!("window.scrollTo({}, {});", x, y)).await?;
        
        if !self.sleep_after_action.is_zero() {
            tokio::time::sleep(self.sleep_after_action).await;
        }
        
        Ok(())
    }

    // 添加动画支持的光标移动方法
    pub async fn animate_cursor_to(&mut self, x: f64, y: f64) -> Result<()> {
        if let Some(page) = &self.current_page {
            let (start_x, start_y) = self.animation_utils.last_cursor_position;
            self.animation_utils.gradual_cursor_animation(
                page, 
                start_x, 
                start_y, 
                x, 
                y, 
                10, // steps
                50  // delay_ms
            ).await.map_err(|e| anyhow!("Animation failed: {}", e))?;
        }
        Ok(())
    }

    // 添加高亮元素的方法
    pub async fn highlight_element(&mut self, element_id: &str) -> Result<()> {
        if let Some(page) = &self.current_page {
            self.animation_utils.add_cursor_box(page, element_id).await
                .map_err(|e| anyhow!("Highlight failed: {}", e))?;
        }
        Ok(())
    }

    // 移除高亮
    pub async fn remove_highlight(&mut self, element_id: &str) -> Result<()> {
        if let Some(page) = &self.current_page {
            self.animation_utils.remove_cursor_box(page, element_id).await
                .map_err(|e| anyhow!("Remove highlight failed: {}", e))?;
        }
        Ok(())
    }

    // 修复 getInteractiveElements 调用
    pub async fn get_interactive_regions(&mut self) -> Result<Vec<InteractiveRegion>> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        // 使用 page_script.js 提供的 getInteractiveElements 方法
        let result: EvaluationResult<Vec<InteractiveRegion>> = 
            page.evaluate("WebSurfer.getInteractiveElements()").await?;
        
        Ok(result.into_value()?)
    }

    // 获取交互元素的完整信息（包含位置信息）
    pub async fn get_interactive_rects(&mut self) -> Result<HashMap<String, InteractiveRegion>> {
        let page = self.current_page.as_ref().ok_or_else(|| anyhow!("No current page"))?;
        
        let result: EvaluationResult<HashMap<String, InteractiveRegion>> = 
            page.evaluate("WebSurfer.getInteractiveRects()").await?;
        
        Ok(result.into_value()?)
    }

    pub async fn close(&mut self) -> Result<()> {
        // 清理动画效果
        if let Some(page) = &self.current_page {
            let _ = self.animation_utils.cleanup_animations(page).await;
        }
        
        if let Some(page) = self.current_page.take() {
            page.close().await?;
        }
        if let Some(browser) = self.browser.take() {
            browser.close().await?;
        }
        Ok(())
    }
}
