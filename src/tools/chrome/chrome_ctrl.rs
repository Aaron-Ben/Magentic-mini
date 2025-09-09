use headless_chrome::{Browser, LaunchOptions, Tab};
use std::sync::Arc;
use tracing::{info, warn};
use anyhow::{Result, Context};
use std::time::Duration;
use tokio::time::sleep;

/// Chrome 浏览器控制器
pub struct ChromiumoxideController {
    browser: Browser,
    tab: Arc<Tab>,
}

impl ChromiumoxideController {
    /// 创建新的浏览器控制器实例
    pub fn new(headless: bool) -> Result<Self> {
        info!("正在启动 Chrome 浏览器...");
        
        let launch_options = LaunchOptions::default_builder()
            .headless(headless)
            .window_size(Some((1920, 1080)))
            .build()
            .context("构建浏览器启动选项失败")?;

        let browser = Browser::new(launch_options)
            .context("启动浏览器失败")?;

        let tab = browser.new_tab()
            .context("创建新标签页失败")?;

        info!("Chrome 浏览器启动成功");
        
        Ok(Self {
            browser,
            tab,
        })
    }

    /// 导航到指定 URL
    pub fn navigate_to(&self, url: &str) -> Result<()> {
        info!("正在导航到: {}", url);
        
        self.tab.navigate_to(url)
            .with_context(|| format!("导航到 {} 失败", url))?;
            
        // 等待页面加载完成
        self.tab.wait_until_navigated()
            .context("等待页面导航完成失败")?;
            
        info!("成功导航到: {}", url);
        Ok(())
    }

    /// 后退
    pub fn go_back(&self) -> Result<()> {
        info!("执行后退操作");
        
        // 使用 JavaScript 执行后退操作
        self.tab.evaluate("window.history.back()", false)
            .context("执行后退操作失败")?;
            
        // 等待页面加载完成
        self.tab.wait_until_navigated()
            .context("等待后退页面加载完成失败")?;
            
        info!("后退操作完成");
        Ok(())
    }

    /// 前进
    pub fn go_forward(&self) -> Result<()> {
        info!("执行前进操作");
        
        // 使用 JavaScript 执行前进操作
        self.tab.evaluate("window.history.forward()", false)
            .context("执行前进操作失败")?;
            
        // 等待页面加载完成
        self.tab.wait_until_navigated()
            .context("等待前进页面加载完成失败")?;
            
        info!("前进操作完成");
        Ok(())
    }

    /// 刷新页面
    pub fn refresh(&self) -> Result<()> {
        info!("执行刷新操作");
        
        self.tab.reload(false, None)
            .context("刷新页面失败")?;
            
        // 等待页面加载完成
        self.tab.wait_until_navigated()
            .context("等待刷新页面加载完成失败")?;
            
        info!("刷新操作完成");
        Ok(())
    }

    /// 获取当前页面的 URL
    pub fn get_current_url(&self) -> Result<String> {
        let url = self.tab.get_url();
        Ok(url)
    }

    /// 获取当前页面的标题
    pub fn get_page_title(&self) -> Result<String> {
        let result = self.tab.evaluate("document.title", false)
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

    /// 关闭浏览器
    pub fn close(self) -> Result<()> {
        info!("正在关闭浏览器...");
        
        if let Err(e) = self.tab.close(false) {
            warn!("关闭标签页时出现错误: {:?}", e);
        }
        
        drop(self.browser);
        info!("浏览器已关闭");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_browser_navigation() -> Result<()> {
        // 初始化日志
        let _ = tracing_subscriber::fmt::try_init();
        
        println!("\n=== 开始浏览器导航测试 ===");
        
        // 创建浏览器控制器（非无头模式，便于观察）
        let controller = ChromiumoxideController::new(false)
            .context("创建浏览器控制器失败")?;
            
        // 测试1: 导航到 bilibili.com
        println!("\n步骤1: 导航到 bilibili.com");
        controller.navigate_to("https://www.bilibili.com")?;
        
        let url = controller.get_current_url()?;
        let title = controller.get_page_title()?;
        println!("当前URL: {}", url);
        println!("页面标题: {}", title);
        
        // 等待 5 秒
        controller.wait(5).await;
        
        // 测试2: 导航到另一个页面
        println!("\n步骤2: 导航到百度");
        controller.navigate_to("https://www.baidu.com")?;
        
        let url = controller.get_current_url()?;
        let title = controller.get_page_title()?;
        println!("当前URL: {}", url);
        println!("页面标题: {}", title);
        
        // 等待 5 秒
        controller.wait(5).await;
        
        // 测试3: 后退到 bilibili
        println!("\n步骤3: 后退到上一页");
        controller.go_back()?;
        
        let url = controller.get_current_url()?;
        let title = controller.get_page_title()?;
        println!("当前URL: {}", url);
        println!("页面标题: {}", title);
        
        // 等待 5 秒
        controller.wait(5).await;
        
        // 测试4: 前进到百度
        println!("\n步骤4: 前进到下一页");
        controller.go_forward()?;
        
        let url = controller.get_current_url()?;
        let title = controller.get_page_title()?;
        println!("当前URL: {}", url);
        println!("页面标题: {}", title);
        
        // 等待 5 秒
        controller.wait(5).await;
        
        // 测试5: 刷新页面
        println!("\n步骤5: 刷新当前页面");
        controller.refresh()?;
        
        let url = controller.get_current_url()?;
        let title = controller.get_page_title()?;
        println!("当前URL: {}", url);
        println!("页面标题: {}", title);
        
        // 等待 5 秒
        controller.wait(5).await;
        
        // 测试6: 再次导航到 bilibili 进行最终验证
        println!("\n步骤6: 最终验证 - 再次导航到 bilibili.com");
        controller.navigate_to("https://www.bilibili.com")?;
        
        let url = controller.get_current_url()?;
        let title = controller.get_page_title()?;
        println!("当前URL: {}", url);
        println!("页面标题: {}", title);
        
        // 等待 5 秒观察结果
        controller.wait(5).await;
        
        // 关闭浏览器
        println!("\n关闭浏览器");
        controller.close()?;
        
        println!("\n=== 浏览器导航测试完成 ===");
        Ok(())
    }

    #[tokio::test]
    async fn test_headless_browser() -> Result<()> {
        // 测试无头模式浏览器
        let _ = tracing_subscriber::fmt::try_init();
        
        println!("\n=== 无头模式浏览器测试 ===");
        
        let controller = ChromiumoxideController::new(true)
            .context("创建无头浏览器失败")?;
            
        // 快速测试导航功能
        controller.navigate_to("https://www.bilibili.com")?;
        let url = controller.get_current_url()?;
        let title = controller.get_page_title()?;
        
        println!("无头模式 - URL: {}", url);
        println!("无头模式 - 标题: {}", title);
        
        assert!(url.contains("bilibili.com"));
        assert!(!title.is_empty());
        
        controller.close()?;
        
        println!("\n=== 无头模式测试完成 ===");
        Ok(())
    }
}