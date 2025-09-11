use headless_chrome::browser::tab;
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::sync::Arc;
use tracing::{info, warn};
use anyhow::{Result, Context};
use std::time::Duration;
use tokio::time::sleep;
use std::collections::HashMap;
use uuid::Uuid;
use std::fs;
use std::path::Path;
use crate::tools::chrome::types::{TabInfo, TabSummary, InteractiveRegion};

/// Chrome 浏览器控制器
pub struct Chrome {
    browser: Browser,
    /// 当前活跃标签页
    current_tab: Arc<Tab>,
    /// 所有标签页的映射表
    tabs: HashMap<String, TabInfo>,
    /// 当前活跃标签页的ID
    current_tab_id: String,
}

impl Chrome {
    /// 创建新的浏览器控制器实例
    pub fn new(headless: bool) -> Result<Self> {

        let launch_options = LaunchOptions::default_builder()
            .headless(headless)
            .window_size(Some((1920, 1080)))
            .build()
            .context("构建浏览器启动选项失败")?;

        let browser = Browser::new(launch_options)
            .context("启动浏览器失败")?;

        let tab = browser.new_tab()
            .context("创建新标签页失败")?;

        // 创建第一个标签页的信息
        let tab_id = Uuid::new_v4().to_string();
        let tab_info = TabInfo {
            tab: tab.clone(),
            id: tab_id.clone(),
            title: "新标签页".to_string(),
            url: "www.google.com".to_string(),
            is_active: true,
        };

        let mut tabs = HashMap::new();
        tabs.insert(tab_id.clone(), tab_info);

        info!("Chrome 浏览器启动成功");
        
        Ok(Self {
            browser,
            current_tab: tab,
            tabs,
            current_tab_id: tab_id,
        })
    }

    /// 导航到指定 URL
    pub fn navigate_to(&mut self, url: &str) -> Result<()> {
        
        self.current_tab.navigate_to(url)
            .with_context(|| format!("导航到 {} 失败", url))?;
            
        self.current_tab.wait_until_navigated()
            .context("等待页面导航完成失败")?;
            
        self.update_current_tab_info()?;
            
        info!("成功导航到: {}", url);
        Ok(())
    }

    /// 后退
    pub fn go_back(&mut self) -> Result<()> {
        
        // 后退操作
        self.current_tab.evaluate("window.history.back()", false)
            .context("执行后退操作失败")?;
            
        self.current_tab.wait_until_navigated()
            .context("等待后退页面加载完成失败")?;
            
        self.update_current_tab_info()?;
        Ok(())
    }

    /// 前进
    pub fn go_forward(&mut self) -> Result<()> {

        // 前进操作
        self.current_tab.evaluate("window.history.forward()", false)
            .context("执行前进操作失败")?;
            
        self.current_tab.wait_until_navigated()
            .context("等待前进页面加载完成失败")?;
            
        self.update_current_tab_info()?;
        Ok(())
    }

    /// 刷新页面
    pub fn refresh(&mut self) -> Result<()> {
        self.current_tab.reload(false, None)
            .context("刷新页面失败")?;
            
        self.current_tab.wait_until_navigated()
            .context("等待刷新页面加载完成失败")?;
            
        self.update_current_tab_info()?;
        Ok(())
    }

    /// 获取当前页面的 URL
    pub fn get_current_url(&self) -> Result<String> {
        let url = self.current_tab.get_url();
        Ok(url)
    }

    /// 获取当前页面的标题
    pub fn get_page_title(&self) -> Result<String> {
        let result = self.current_tab.evaluate("document.title", false)
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

    /// 等待指定时间（用于测试）
    pub async fn wait(&self, seconds: u64) {
        info!("等待 {} 秒...", seconds);
        sleep(Duration::from_secs(seconds)).await;
    }

    /// 创建新的标签页
    pub fn new_tab(&mut self) -> Result<String> {
        let new_tab = self.browser.new_tab()
            .context("创建新标签页失败")?;
            
        let tab_id = Uuid::new_v4().to_string();
        let tab_info = TabInfo {
            tab: new_tab,
            id: tab_id.clone(),
            title: "新标签页".to_string(),
            url: "www.google.com".to_string(),
            is_active: false,
        };
        
        self.tabs.insert(tab_id.clone(), tab_info);
        Ok(tab_id)
    }
    
    /// 切换到指定标签页
    pub fn switch_to_tab(&mut self, tab_id: &str) -> Result<()> {
        info!("正在切换到标签页: {}", tab_id);
        
        let tab_info = self.tabs.get(tab_id)
            .ok_or_else(|| anyhow::anyhow!("标签页 {} 不存在", tab_id))?
            .clone();
            
        // 更新标签页的活跃状态
        if let Some(current_tab_info) = self.tabs.get_mut(&self.current_tab_id) {
            current_tab_info.is_active = false;
        }
        
        self.current_tab = tab_info.tab.clone();
        self.current_tab_id = tab_id.to_string();
        
        if let Some(new_current_tab_info) = self.tabs.get_mut(tab_id) {
            new_current_tab_info.is_active = true;
        }
        
        info!("成功切换到标签页: {}", tab_id);
        Ok(())
    }
    
    /// 关闭指定标签页
    pub fn close_tab(&mut self, tab_id: &str) -> Result<()> {
        
        if self.tabs.len() <= 1 {
            return Err(anyhow::anyhow!("无法关闭最后一个标签页"));
        }
        
        let tab_info = self.tabs.remove(tab_id)
            .ok_or_else(|| anyhow::anyhow!("标签页 {} 不存在", tab_id))?;
            
        // 关闭标签页
        if let Err(e) = tab_info.tab.close(false) {
            warn!("关闭标签页时出现错误: {:?}", e);
        }
        
        // 如果关闭的是当前活跃标签页，切换到第一个可用标签页
        if tab_id == self.current_tab_id {
            if let Some((first_tab_id, _)) = self.tabs.iter().next() {
                let first_tab_id = first_tab_id.clone();
                self.switch_to_tab(&first_tab_id)?;
            }
        }
        
        info!("标签页 {} 已关闭", tab_id);
        Ok(())
    }
    
    /// 获取所有标签页的摘要信息
    pub fn get_tabs_summary(&self) -> Result<Vec<TabSummary>> {
        let mut summaries = Vec::new();
        
        for tab_info in self.tabs.values() {
            let summary = TabSummary {
                id: tab_info.id.clone(),
                title: tab_info.title.clone(),
                url: tab_info.url.clone(),
                is_active: tab_info.is_active,
            };
            summaries.push(summary);
        }
        
        // 按活跃状态排序，活跃的在前
        summaries.sort_by(|a, b| b.is_active.cmp(&a.is_active));
        
        Ok(summaries)
    }
    
    /// 获取当前活跃标签页的ID
    pub fn get_current_tab_id(&self) -> &str {
        &self.current_tab_id
    }
    
    /// 获取标签页数量
    pub fn get_tab_count(&self) -> usize {
        self.tabs.len()
    }
    
    /// 获取所有标签页的详细信息（包括浏览器内部标签页）
    pub fn get_all_tabs(&self) -> Result<Vec<TabSummary>> {
        // 获取浏览器中的所有标签页
        let browser_tabs = self.browser.get_tabs()
            .lock()
            .map_err(|e| anyhow::anyhow!("获取浏览器标签页列表失败: {:?}", e))?;
            
        let mut all_tabs = Vec::new();
        
        for (index, tab) in browser_tabs.iter().enumerate() {
            let url = tab.get_url();
            let title_result = tab.evaluate("document.title", false);
            
            let title = match title_result {
                Ok(result) => {
                    result.value
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| "无标题".to_string())
                }
                Err(_) => "无法获取标题".to_string(),
            };
            
            // 检查是否为当前活跃标签页
            let is_active = self.tabs.values()
                .any(|tab_info| {
                    Arc::ptr_eq(&tab_info.tab, tab) && tab_info.is_active
                });
            
            all_tabs.push(TabSummary {
                id: format!("browser_tab_{}", index),
                title,
                url,
                is_active,
            });
        }
        
        Ok(all_tabs)
    }
    
    /// 更新当前标签页信息
    fn update_current_tab_info(&mut self) -> Result<()> {
        let url = self.get_current_url()?;
        let title = self.get_page_title()?;
        
        if let Some(tab_info) = self.tabs.get_mut(&self.current_tab_id) {
            tab_info.url = url;
            tab_info.title = title;
        }
        
        Ok(())
    }

    // 获取截图
    async fn get_screenshot(&self) -> Result<Vec<u8>> {
        let screenshot = self.current_tab.capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            false
        )?;
        Ok(screenshot)
    }

    async fn set_mark(&self) -> Result<()> {
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
        self.current_tab.evaluate(js_script, true)?;
        Ok(())
    }

    async fn clear_mark(&self) -> Result<()> {
        let js_script = r#"
            // 获取所有已添加的边框元素
            const redBorderElements = document.querySelectorAll('[style*="border: 2px solid red"]');
            
            // 移除边框
            redBorderElements.forEach(el => {
                el.style.border = '';
            });
        "#;
        self.current_tab.evaluate(js_script, true)?;
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
