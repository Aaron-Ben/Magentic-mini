use anyhow::{anyhow, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use chromiumoxide::cdp::browser_protocol::page::Viewport;
use chromiumoxide::cdp::browser_protocol::network::Cookie;
use chromiumoxide::cdp::browser_protocol::storage::GetCookiesParams;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tab {
    pub url: String,
    pub index: usize,
    pub scroll_x: i64,
    pub scroll_y: i64,
}

// 定义 BrowserState 结构体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrowserState {
    pub state: StorageState,
    pub tabs: Vec<Tab>,
    pub active_tab_index: usize,
}

// StorageState：简化版本，包含 cookies 和 local_storage
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageState {
    pub cookies: Vec<Cookie>,
    pub local_storage: HashMap<String, Value>,
}

// 保存浏览器状态
pub async fn save_browser_state(
    browser: &Browser,
    controlled_page: Option<&Page>,
    simplified: bool,
) -> Result<BrowserState> {
    let pages = browser.pages().await?;

    // 确定活动标签索引
    let mut active_tab_index = 0;
    if let Some(controlled) = controlled_page {
        for (i, page) in pages.iter().enumerate() {
            if page.page_id() == controlled.page_id() {
                active_tab_index = i;
                break;
            }
        }
    }

    // 如果简化模式，跳过存储状态
    let state = if simplified {
        StorageState::default()
    } else {
        // 获取存储状态（以第一个页面为例）
        if pages.is_empty() {
            return Err(anyhow!("No pages available to save state"));
        }
        let first_page = &pages[0];
        
        // 获取 cookies
        let cookies_result = first_page.execute(GetCookiesParams::default()).await;
        let cookies = match cookies_result {
            Ok(result) => result.cookies,
            Err(e) => {
                error!("Failed to get cookies: {}", e);
                Vec::new()
            }
        };

        // 获取 localStorage
        let local_storage: HashMap<String, Value> = match first_page
            .evaluate("JSON.stringify(localStorage)")
            .await
        {
            Ok(res) => {
                let mut map = HashMap::new();
                if let Ok(value) = res.into_value() {
                    map.insert(first_page.url().to_string(), value);
                }
                map
            }
            Err(e) => {
                error!("Failed to get localStorage: {}", e);
                HashMap::new()
            }
        };

        StorageState {
            cookies,
            local_storage,
        }
    };

    // 保存标签页
    let mut tabs = Vec::new();
    for (i, page) in pages.iter().enumerate() {
        let (scroll_x, scroll_y) = if simplified {
            (0, 0)
        } else {
            // 通过 JS 获取滚动位置
            match page
                .evaluate("({ scrollX: window.scrollX, scrollY: window.scrollY })")
                .await
            {
                Ok(res) => {
                    if let Ok(scroll_data) = res.into_value::<HashMap<String, i64>>() {
                        (
                            *scroll_data.get("scrollX").unwrap_or(&0),
                            *scroll_data.get("scrollY").unwrap_or(&0),
                        )
                    } else {
                        (0, 0)
                    }
                }
                Err(e) => {
                    error!("Failed to get scroll position: {}", e);
                    (0, 0)
                }
            }
        };

        tabs.push(Tab {
            url: page.url().to_string(),
            index: i,
            scroll_x,
            scroll_y,
        });
    }

    Ok(BrowserState {
        state,
        tabs,
        active_tab_index,
    })
}

// 加载浏览器状态
pub async fn load_browser_state(
    browser: &Browser,
    state: BrowserState,
    load_only_active_tab: bool,
) -> Result<()> {
    // 获取当前页面
    let pages = browser.pages().await?;

    // 关闭空白页 (about:blank)
    for page in pages.iter() {
        if page.url() == "about:blank" {
            let _ = page.close().await;
        }
    }

    // 确定要恢复的标签页
    let tabs_to_restore = if load_only_active_tab {
        if state.active_tab_index < state.tabs.len() {
            vec![state.tabs[state.active_tab_index].clone()]
        } else {
            return Err(anyhow!("Invalid active tab index"));
        }
    } else {
        state.tabs.clone()
    };

    // 创建并恢复标签页
    let mut restored_pages: Vec<Page> = Vec::new();
    for tab in tabs_to_restore {
        let page = browser.new_page(&tab.url).await?;
        page.wait_for_navigation().await?;

        // 设置滚动位置
        let _ = page.evaluate(format!(
            "window.scrollTo({}, {})",
            tab.scroll_x, tab.scroll_y
        ))
        .await;

        restored_pages.push(page);
    }

    // 恢复存储状态（如果非简化）
    if !state.state.cookies.is_empty() || !state.state.local_storage.is_empty() {
        if let Some(first_page) = restored_pages.first() {
            // 设置 localStorage
            for (origin, ls) in state.state.local_storage {
                if origin == first_page.url() {
                    let _ = first_page
                        .evaluate(format!(
                            "Object.assign(localStorage, {})",
                            serde_json::to_string(&ls).unwrap_or_default()
                        ))
                        .await;
                }
            }
        }
    }

    // 激活活动标签
    if !restored_pages.is_empty() {
        let active_index = if load_only_active_tab { 0 } else { state.active_tab_index };
        if active_index < restored_pages.len() {
            let _ = restored_pages[active_index]
                .set_viewport(Some(Viewport::default()))
                .await;
        }
        // 等待 5 秒稳定
        sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}