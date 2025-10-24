use std::sync::Arc;
use tracing::{error, info};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct LocalBrowserConfig {
    pub headless: bool,
    pub browser_channel: Option<String>,
    pub enable_downloads: bool,
    pub persistent_context: bool,
    pub browser_data_dir: Option<String>,
}

impl Default for LocalBrowserConfig {
    fn default() -> Self {
        Self {
            headless: false,
            browser_channel: None,
            enable_downloads: false,
            persistent_context: false,
            browser_data_dir: None,
        }
    }
}

pub struct LocalChromiumBrowser {
    config: LocalBrowserConfig,
    browser: Option<Browser>,
    context: Option<Arc<Tab>>,
}

impl LocalChromiumBrowser {
    pub fn new(config: LocalBrowserConfig) -> Self {
        Self {
            config,
            browser: None,
            context: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting browser...");

        let launch_options = LaunchOptions::default_builder()
            .headless(self.config.headless)
            .build()
            .map_err(|e| anyhow::anyhow!("Browser config error: {:?}", e))?;

        let browser = Browser::new(launch_options)
            .map_err(|e| anyhow::anyhow!("Failed to launch browser: {}", e))?;

        let tab = browser.new_tab()
            .map_err(|e| anyhow::anyhow!("Failed to create new tab: {}", e))?;

        tab.navigate_to("https://www.bing.com")
            .map_err(|e| anyhow::anyhow!("Failed to navigate to blank page: {}", e))?;

        self.browser = Some(browser);
        self.context = Some(tab);

        info!("Browser started");
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        info!("Closing browser...");

        if let Some(context) = self.context.take() {
            if let Err(e) = context.close(false) {
                error!("Error closing context: {:?}", e);
            }
        }

        if let Some(browser) = self.browser.take() {
            drop(browser);
        }

        info!("Browser closed");
        Ok(())
    }

    pub fn browser_context(&self) -> Option<&Arc<Tab>> {
        self.context.as_ref()
    }
}