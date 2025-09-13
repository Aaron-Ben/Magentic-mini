use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::sync::Arc;
use tracing::{info, warn};
use anyhow::{anyhow, Context, Result};
use std::time::Duration;
use tokio::time::sleep;
use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::agents::plan_agent::config;
use crate::tools::chrome::types::{TabInfo,InteractiveRegion, VisualViewport, PageMetadata};
use crate::tools::utils::webpage_text_utils::WebpageTextUtils;

lazy_static! {
    pub static ref CUA_KEY_TO_CHROMIUM_KEY: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("/", "Divide");
        map.insert("\\", "Backslash");
        map.insert("alt", "Alt");
        map.insert("arrowdown", "ArrowDown");
        map.insert("arrowleft", "ArrowLeft");
        map.insert("arrowright", "ArrowRight");
        map.insert("arrowup", "ArrowUp");
        map.insert("backspace", "Backspace");
        map.insert("capslock", "CapsLock");
        map.insert("cmd", "Meta");
        map.insert("ctrl", "Control");
        map.insert("delete", "Delete");
        map.insert("end", "End");
        map.insert("enter", "Enter");
        map.insert("esc", "Escape");
        map.insert("home", "Home");
        map.insert("insert", "Insert");
        map.insert("option", "Alt");
        map.insert("pagedown", "PageDown");
        map.insert("pageup", "PageUp");
        map.insert("shift", "Shift");
        map.insert("space", " ");
        map.insert("super", "Meta");
        map.insert("tab", "Tab");
        map.insert("win", "Meta");
        map
    };
}

/// Chrome 浏览器控制器
pub struct Chrome {
    browser: Browser,
    /// 当前活跃标签页，当浏览器被最小化时，活跃的标签页为 None
    current_tab: Option<Arc<Tab>>,
    tabs: HashMap<usize, TabInfo>,
    /// 当前活跃标签页的ID
    current_tab_id: Option<usize>,
    // 可能用到的配置字段
    animate_actions: bool,
    downloads_folder: Option<String>,
    viewport_width: u32,
    viewport_height: u32,
    to_resize_viewport: bool,
    timeout_load: f64,
    sleep_after_action: f64,
    single_tab_mode: bool,
    // 页面脚本和工具
    page_script: String,
    text_utils: WebpageTextUtils,
}

#[derive(Debug)]
pub struct ChromeConfig {
    pub downloads_folder: Option<String>,
    pub animate_actions: bool,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub to_resize_viewport: bool,
    pub timeout_load: f64,
    pub sleep_after_action: f64,
    pub single_tab_mode: bool,
}

impl Default for ChromeConfig {
    fn default() -> Self {
        Self {
            downloads_folder: None,
            animate_actions: false,
            viewport_width: 1440,
            viewport_height: 900,
            to_resize_viewport: true,
            timeout_load: 30.0,
            sleep_after_action: 0.1,
            single_tab_mode: false,
        }
    }
    
}

impl Chrome {
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
        let tab_id = 1; 
        let tab_info = TabInfo {
            tab: tab.clone(),
            index: 0,
            title: "新标签页".to_string(),
            url: String::from("https://www.google.com"),
            is_active: true,
            is_controlled: false,
        };

        let mut tabs = HashMap::new();
        tabs.insert(tab_id, tab_info);

        info!("Chrome 浏览器启动成功");
        
        Ok(Self {
            browser,
            current_tab: Some(tab),
            tabs,
            current_tab_id: Some(tab_id),
            animate_actions: false,
            downloads_folder: None,
            viewport_width: 1440,
            viewport_height: 1440,
            to_resize_viewport: true,
            timeout_load: 30.0,
            sleep_after_action: 0.1,
            single_tab_mode: false,
            page_script: String::new(),
            text_utils: WebpageTextUtils::new(),
        })
    }

    pub fn with_config(
        mut self,
        config: ChromeConfig,
    ) -> Result<Self> {
        // 验证配置参数
        assert!(config.viewport_width > 0);
        assert!(config.viewport_height > 0);
        assert!(config.timeout_load > 0.0);

        // 更新配置
        self.animate_actions = config.animate_actions;
        self.downloads_folder = config.downloads_folder;
        self.viewport_width = config.viewport_width;
        self.viewport_height = config.viewport_height;
        self.to_resize_viewport = config.to_resize_viewport;
        self.timeout_load = config.timeout_load;
        self.sleep_after_action = config.sleep_after_action;
        self.single_tab_mode = config.single_tab_mode;

        // 加载页面脚本
        let script_path = std::path::Path::new(file!())
            .parent()
            .unwrap()
            .join("page_script.js");
        self.page_script = std::fs::read_to_string(script_path)
            .unwrap_or_default();

        Ok(self)
    }


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

    pub fn go_back(&mut self) -> Result<()> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;
        
        tab.evaluate("window.history.back()", false)
            .context("执行后退操作失败")?;
            
        tab.wait_until_navigated()
            .context("等待后退页面加载完成失败")?;
            
        Ok(())
    }

    pub fn go_forward(&mut self) -> Result<()> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;

        tab.evaluate("window.history.forward()", false)
            .context("执行前进操作失败")?;
            
        tab.wait_until_navigated()
            .context("等待前进页面加载完成失败")?;
            
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;
            
        tab.reload(false, None)
            .context("刷新页面失败")?;
            
        tab.wait_until_navigated()
            .context("等待刷新页面加载完成失败")?;
            
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

    /// 等待指定时间（用于测试）
    pub async fn wait(&self, seconds: u64) {
        info!("等待 {} 秒...", seconds);
        sleep(Duration::from_secs(seconds)).await;
    }

    pub fn create_tab_id(&self) -> usize {
        self.tabs.len() + 1
    }

    pub fn reindex_tabs(&mut self) {
        for (index, (_, tab_info)) in self.tabs.iter_mut().enumerate() {
            tab_info.index = index;
        }
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
    
    /// 切换到指定标签页
    pub fn switch_tab(&mut self, tab_id: usize) -> Result<()> {
        let tabs_info = self.tabs.get(&tab_id)
            .ok_or_else(|| anyhow!("标签页 {} 不存在", tab_id))?
            .clone();
            
        // 更新标签页的活跃状态
        if let Some(current_id) = self.current_tab_id {
            if let Some(current_tab_info) = self.tabs.get_mut(&current_id) {
                current_tab_info.is_active = false;
            }
        }

        // 设置新的活跃标签页
        self.current_tab = Some(tabs_info.tab.clone());
        self.current_tab_id = Some(tab_id);

        if let Some(new_tab_info) = self.tabs.get_mut(&tab_id) {
            new_tab_info.is_active = true;
        }

        tabs_info.tab
            .activate()
            .context("激活标签页失败")?;

        info!("成功切换到标签页: {}", tab_id);
        Ok(())
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

    // 向输入框、文本区域或下拉框填充文本。支持先删除现有文本和在输入后按回车键
    pub fn fill_text (&mut self, _element_id: &str, _text: &str) -> Result<()> {
        Ok(())
    }

    // 选择下拉菜单选项
    pub fn select_option(&mut self, _element_id: &str, _option_text: &str) -> Result<()> {
        Ok(())
    }

    // 向输入框上传本地文件
    pub fn upload_file(&mut self, _element_id: &str, _file_path: &str) -> Result<()> {
        Ok(())
    }

    /// 页面滚动
    pub fn page_up() -> Result<()> {
        Ok(())
    }

    pub fn page_down() -> Result<()> {
        Ok(())
    }

    // 鼠标操作
    pub async fn click_coords(&mut self, x: i32, y: i32, button: &str) -> Result<Option<Arc<Tab>>> {
        
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;

        // 在点击位置添加一个临时的视觉指示器
        let highlight_js = format!(
            r#"
            (function() {{
                const indicator = document.createElement('div');
                indicator.style.cssText = `
                    position: fixed;
                    z-index: 10000;
                    pointer-events: none;
                    width: 20px;
                    height: 20px;
                    background: rgba(255, 0, 0, 0.3);
                    border: 2px solid red;
                    border-radius: 50%;
                    transform: translate(-50%, -50%);
                    left: {x}px;
                    top: {y}px;
                `;
                document.body.appendChild(indicator);
                setTimeout(() => indicator.remove(), 300);
            }})()
            "#
        );
        tab.evaluate(&highlight_js, true)?;

        tab.wait_until_navigated().context("等待页面导航完成失败")?;

        // 特殊按钮
        match button {
            "back" => {
                self.go_back()?;
                Ok(None)
            }
            "forward" => {
                self.go_forward()?;
                Ok(None)
            }
            "wheel" => {
                tab.evaluate(&format!("window.scrollBy({}, {});", x, y), true)?;
                Ok(None)
            }
            "left" | "right" => {

                // 创建鼠标事件序列
                let button_num = if button == "left" { 0 } else { 2 };
                let js = format!(
                    r#"
                    (function() {{
                        const target = document.elementFromPoint({}, {});
                        if (!target) throw new Error('No element at coordinates');
                        ['mousedown', 'mouseup', 'click'].forEach(type => {{
                            const event = new MouseEvent(type, {{
                                view: window,
                                bubbles: true,
                                cancelable: true,
                                clientX: {},
                                clientY: {},
                                button: {},
                                buttons: 1
                            }});
                            target.dispatchEvent(event);
                        }});
                        {}  // 右键额外触发contextmenu事件
                    }})();
                    "#,
                    x, y, x, y, button_num,
                    if button == "right" {
                        "target.dispatchEvent(new Event('contextmenu', { bubbles: true, cancelable: true }));"
                    } else {
                        ""
                    }
                );
                tab.evaluate(&js, true)
                    .map_err(|e| anyhow!("{}键点击失败: {}", button, e))?;
                Ok(None)
            }
            _ => Err(anyhow!("不支持的按钮类型: {}", button)),
        }

    }

    pub async fn double_coords(&mut self, x: i32, y: i32) -> Result<()> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;

        // 添加双击视觉反馈
        let highlight_js = format!(
            r#"
            (function() {{
                // 创建两个同心圆的动画效果
                const outer = document.createElement('div');
                const inner = document.createElement('div');
                
                outer.style.cssText = `
                    position: fixed;
                    z-index: 10000;
                    pointer-events: none;
                    width: 30px;
                    height: 30px;
                    border: 2px solid rgba(255, 0, 0, 0.6);
                    border-radius: 50%;
                    transform: translate(-50%, -50%);
                    left: {x}px;
                    top: {y}px;
                `;
                
                inner.style.cssText = `
                    position: fixed;
                    z-index: 10001;
                    pointer-events: none;
                    width: 16px;
                    height: 16px;
                    background: rgba(255, 0, 0, 0.3);
                    border-radius: 50%;
                    transform: translate(-50%, -50%);
                    left: {x}px;
                    top: {y}px;
                `;
                
                document.body.appendChild(outer);
                document.body.appendChild(inner);
                
                // 在两次点击的间隔显示
                setTimeout(() => inner.remove(), 200);
                setTimeout(() => outer.remove(), 400);
            }})()
            "#
        );
        tab.evaluate(&highlight_js, true)?;

        tab.wait_until_navigated().context("等待页面导航完成失败")?;

        // 执行双击事件序列
        let js = format!(
            r#"
            (function() {{
                const target = document.elementFromPoint({}, {});
                if (!target) throw new Error('No element at coordinates');
                
                // 第一次点击
                ['mousedown', 'mouseup', 'click'].forEach(type => {{
                    const event = new MouseEvent(type, {{
                        view: window,
                        bubbles: true,
                        cancelable: true,
                        clientX: {},
                        clientY: {},
                        detail: 1,
                        button: 0,
                        buttons: 1
                    }});
                    target.dispatchEvent(event);
                }});

                // 第二次点击
                ['mousedown', 'mouseup', 'click', 'dblclick'].forEach(type => {{
                    const event = new MouseEvent(type, {{
                        view: window,
                        bubbles: true,
                        cancelable: true,
                        clientX: {},
                        clientY: {},
                        detail: 2,
                        button: 0,
                        buttons: 1
                    }});
                    target.dispatchEvent(event);
                }});
            }})();
            "#,
            x, y, x, y, x, y
        );

        tab.evaluate(&js, true)
            .map_err(|e| anyhow!("双击操作失败: {}", e))?;

        
        Ok(())
    }

    pub async fn hover_coords(&mut self, x: i32, y: i32) -> Result<()> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;

        // 在悬停位置添加指示器
        let highlight_js = format!(
            r#"
            (function() {{
                const indicator = document.createElement('div');
                indicator.style.cssText = `
                    position: fixed;
                    z-index: 10000;
                    pointer-events: none;
                    width: 16px;
                    height: 16px;
                    border: 2px solid rgba(255, 165, 0, 0.8);
                    border-radius: 50%;
                    transform: translate(-50%, -50%);
                    left: {x}px;
                    top: {y}px;
                `;
                document.body.appendChild(indicator);
                setTimeout(() => indicator.remove(), 1000);
            }})()
            "#
        );
        tab.evaluate(&highlight_js, true)?;

        tab.wait_until_navigated().context("等待页面导航完成失败")?;

        // 执行悬停事件序列
        let js = format!(
            r#"
            (function() {{
                const target = document.elementFromPoint({}, {});
                if (!target) throw new Error('No element at coordinates');
                
                ['mouseover', 'mouseenter', 'mousemove'].forEach(type => {{
                    const event = new MouseEvent(type, {{
                        view: window,
                        bubbles: true,
                        cancelable: true,
                        clientX: {},
                        clientY: {},
                        button: 0,
                        buttons: 0
                    }});
                    target.dispatchEvent(event);
                }});
            }})();
            "#,
            x, y, x, y
        );

        tab.evaluate(&js, true)
            .map_err(|e| anyhow!("悬停操作失败: {}", e))?;

        Ok(())
    }

    pub async fn drag_coords(&mut self, path: Vec<(i32, i32)>) -> Result<()> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;

        tab.wait_until_navigated().context("等待页面导航完成失败")?;
        if path.is_empty() {
            return Ok(());
        }

        // 显示拖拽路径
        if let Some(&(start_x, start_y)) = path.first() {
            let path_points = path.iter()
                .map(|(x, y)| format!("{},{}", x, y))
                .collect::<Vec<String>>()
                .join(" ");

            let js = format!(
                r#"
                (function() {{
                    // 创建SVG元素来显示路径
                    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
                    svg.style.cssText = 'position: fixed; left: 0; top: 0; width: 100%; height: 100%; pointer-events: none; z-index: 10000;';
                    
                    // 创建路径
                    const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
                    path.setAttribute('d', 'M {} L {}'.replace(/,/g, ' '));
                    path.style.cssText = 'stroke: rgba(255,0,0,0.5); stroke-width: 2px; fill: none;';
                    
                    svg.appendChild(path);
                    document.body.appendChild(svg);
                    
                    // 添加起点指示器
                    const start = document.createElement('div');
                    start.style.cssText = `
                        position: fixed;
                        z-index: 10001;
                        width: 10px;
                        height: 10px;
                        background: red;
                        border-radius: 50%;
                        transform: translate(-50%, -50%);
                        left: {start_x}px;
                        top: {start_y}px;
                    `;
                    document.body.appendChild(start);
                    
                    setTimeout(() => {{
                        svg.remove();
                        start.remove();
                    }}, 500);
                }})()
                "#,
                format!("{},{}", start_x, start_y),
                path_points
            );
            tab.evaluate(&js, true)?;
        }

        // 按下鼠标
        tab.evaluate(&format!(
            r#"
            var event = new MouseEvent('mousedown', {{
                bubbles: true,
                cancelable: true,
                clientX: {},
                clientY: {},
                button: 0
            }});
            document.elementFromPoint({}, {}).dispatchEvent(event);
            "#,
            path[0].0, path[0].1, path[0].0, path[0].1
        ), true).map_err(|e| anyhow!("鼠标按下失败: {}", e))?;

        // 沿路径移动
        for &(x, y) in path.iter().skip(1) {

            tab.evaluate(&format!(
                r#"
                var event = new MouseEvent('mousemove', {{
                    bubbles: true,
                    cancelable: true,
                    clientX: {},
                    clientY: {},
                    buttons: 1
                }});
                document.elementFromPoint({}, {}).dispatchEvent(event);
                "#,
                x, y, x, y
            ), true).map_err(|e| anyhow!("拖拽移动失败: {}", e))?;
        }
        
        // 释放鼠标
        if let Some(&(last_x, last_y)) = path.last() {
            tab.evaluate(&format!(
                r#"
                var event = new MouseEvent('mouseup', {{
                    bubbles: true,
                    cancelable: true,
                    clientX: {},
                    clientY: {},
                    button: 0
                }});
                document.elementFromPoint({}, {}).dispatchEvent(event);
                "#,
                last_x, last_y, last_x, last_y
            ), true).map_err(|e| anyhow!("鼠标释放失败: {}", e))?;
        }
        Ok(())
    }

    pub async fn scroll_coords(&mut self, x: i32, y: i32, scroll_x: i32, scroll_y: i32) -> Result<()> {
        let tab = self.current_tab.as_ref()
            .ok_or_else(|| anyhow!("没有活跃的标签页"))?;

        // 添加滚动视觉反馈
        let scroll_indicator_js = format!(
            r#"
            (function() {{
                // 创建滚动指示器
                const indicator = document.createElement('div');
                indicator.style.cssText = `
                    position: fixed;
                    z-index: 10000;
                    pointer-events: none;
                    width: 30px;
                    height: 30px;
                    left: {x}px;
                    top: {y}px;
                    transform: translate(-50%, -50%);
                `;

                // 根据滚动方向设置箭头样式
                if (Math.abs({scroll_y}) > Math.abs({scroll_x})) {{
                    // 垂直滚动
                    const direction = {scroll_y} > 0 ? '↓' : '↑';
                    indicator.innerHTML = `<div style="
                        font-size: 24px;
                        color: red;
                        text-align: center;
                        line-height: 30px;
                    ">${{direction}}</div>`;
                }} else {{
                    // 水平滚动
                    const direction = {scroll_x} > 0 ? '→' : '←';
                    indicator.innerHTML = `<div style="
                        font-size: 24px;
                        color: red;
                        text-align: center;
                        line-height: 30px;
                    ">${{direction}}</div>`;
                }}

                document.body.appendChild(indicator);

                // 创建滚动轨迹
                const track = document.createElement('div');
                track.style.cssText = `
                    position: fixed;
                    z-index: 9999;
                    pointer-events: none;
                    background: rgba(255, 0, 0, 0.2);
                    border: 1px solid rgba(255, 0, 0, 0.4);
                    left: ${{Math.min({x}, {x} + {scroll_x})}}px;
                    top: ${{Math.min({y}, {y} + {scroll_y})}}px;
                    width: ${{Math.abs({scroll_x}) || 4}}px;
                    height: ${{Math.abs({scroll_y}) || 4}}px;
                `;
                
                document.body.appendChild(track);

                // 移除指示器和轨迹
                setTimeout(() => {{
                    indicator.remove();
                    track.remove();
                }}, 500);
            }})()
            "#
        );
        tab.evaluate(&scroll_indicator_js, true)?;

        tab.wait_until_navigated().context("等待页面导航完成失败")?;

        // 移动到指定位置
        tab.evaluate(&format!(
            r#"
            var event = new MouseEvent('mousemove', {{
                bubbles: true,
                cancelable: true,
                clientX: {},
                clientY: {}
            }});
            document.elementFromPoint({}, {}).dispatchEvent(event);
            window.scrollBy({}, {});
            "#,
            x, y, x, y, scroll_x, scroll_y
        ), true).map_err(|e| anyhow!("滚动失败: {}", e))?;
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

