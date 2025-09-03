use anyhow::{anyhow, Result};
use headless_chrome::{Browser, Tab as HeadlessTab};
use headless_chrome::protocol::cdp::Network::CookieParam;
use log::{warn, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tab {
    pub url: String,
    pub index: usize,
    pub scroll_x: i64,
    pub scroll_y: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrowserState {
    pub state: StorageState,
    pub tabs: Vec<Tab>,
    pub active_tab_index: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageState {
    pub cookies: Vec<CookieData>,
    pub origins: Vec<OriginState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CookieData {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub expires: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OriginState {
    pub origin: String,
    pub local_storage: Vec<LocalStorageEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalStorageEntry {
    pub key: String,
    pub value: String,
}

fn extract_origin(url_str: &str) -> String {
    if let Ok(url) = Url::parse(url_str) {
        if let Some(host) = url.host_str() {
            format!("{}://{}", url.scheme(), host)
        } else {
            url_str.to_string()
        }
    } else {
        url_str.to_string()
    }
}

async fn get_scroll_position(tab: &Arc<HeadlessTab>) -> Result<(i64, i64), anyhow::Error> {
    let result = tab.evaluate("() => ({ scrollX: window.scrollX, scrollY: window.scrollY })", false)
        .map_err(|e| anyhow!("Failed to evaluate scroll position: {}", e))?;

    if let Some(value) = result.value {
        if let Ok(scroll_data) = serde_json::from_value::<HashMap<String, f64>>(value) {
            let scroll_x = scroll_data.get("scrollX").unwrap_or(&0.0).round() as i64;
            let scroll_y = scroll_data.get("scrollY").unwrap_or(&0.0).round() as i64;
            return Ok((scroll_x, scroll_y));
        }
    }
    
    Ok((0, 0))
}

async fn get_storage_state(tabs: &[Arc<HeadlessTab>]) -> Result<StorageState, anyhow::Error> {
    let mut cookies = Vec::new();
    let mut origins = Vec::new();

    // 获取 cookies（从第一个标签页）
    if let Some(first_tab) = tabs.first() {
        match first_tab.get_cookies() {
            Ok(tab_cookies) => {
                for cookie in tab_cookies {
                    cookies.push(CookieData {
                        name: cookie.name,
                        value: cookie.value,
                        domain: cookie.domain,
                        path: cookie.path,
                        secure: cookie.secure,
                        http_only: cookie.http_only,
                        expires: Some(cookie.expires),
                    });
                }
            }
            Err(e) => {
                error!("Failed to get cookies: {}", e);
            }
        }
    }

    // 获取每个标签页的 local storage，按 origin 分组
    for tab in tabs {
        let page_url = tab.get_url();
        let origin = extract_origin(&page_url);
        
        let local_storage_result = tab.evaluate(
            r#"
            () => {
                const o = {};
                try {
                    for (let i = 0; i < localStorage.length; i++) {
                        const k = localStorage.key(i);
                        if (k) o[k] = localStorage.getItem(k) || "";
                    }
                } catch (e) {
                    console.error('Failed to access localStorage:', e);
                }
                return o;
            }
            "#,
            false,
        );

        if let Ok(eval_result) = local_storage_result {
            if let Some(value) = eval_result.value {
                if let Ok(local_storage) = serde_json::from_value::<HashMap<String, String>>(value) {
                    let local_storage_entries = local_storage
                        .into_iter()
                        .map(|(key, value)| LocalStorageEntry { key, value })
                        .collect::<Vec<_>>();

                    if !local_storage_entries.is_empty() {
                        origins.push(OriginState {
                            origin,
                            local_storage: local_storage_entries,
                        });
                    }
                }
            }
        }
    }

    Ok(StorageState { cookies, origins })
}

/// Save the browser's storage state along with the URLs and scroll positions of all open tabs,
/// and identify the active tab.
pub async fn save_browser_state(
    tabs: &[Arc<HeadlessTab>],
    controlled_tab: Option<&Arc<HeadlessTab>>,
    simplified: bool,
) -> Result<BrowserState, anyhow::Error> {
    let mut active_tab_index = 0;

    // 查找 controlled_tab 的索引
    if let Some(ctrl_tab) = controlled_tab {
        for (i, tab) in tabs.iter().enumerate() {
            if Arc::ptr_eq(tab, ctrl_tab) {
                active_tab_index = i;
                break;
            }
        }
    }

    // 收集所有标签页状态
    let mut tab_states = Vec::with_capacity(tabs.len());

    for (i, tab) in tabs.iter().enumerate() {
        let url = tab.get_url();

        let (scroll_x, scroll_y) = if simplified {
            (0, 0)
        } else {
            get_scroll_position(tab).await.unwrap_or((0, 0))
        };

        tab_states.push(Tab {
            url,
            index: i,
            scroll_x,
            scroll_y,
        });
    }

    // 获取存储状态
    let state = if simplified {
        StorageState::default()
    } else {
        get_storage_state(tabs).await.unwrap_or_default()
    };

    Ok(BrowserState {
        state,
        tabs: tab_states,
        active_tab_index,
    })
}


pub async fn load_browser_state(
    browser: &Browser,
    state: BrowserState,
    load_only_active_tab: bool,
) -> Result<Vec<Arc<HeadlessTab>>, anyhow::Error> {
    // 1. 关闭所有 about:blank 页面
    let tabs = browser.get_tabs();
    let mut tabs_to_close = vec![];
    
    // 获取 tabs 的锁并迭代
    if let Ok(tabs_guard) = tabs.lock() {
        for tab in tabs_guard.iter() {
            let tab_url = tab.get_url();
            if tab_url == "about:blank" {
                tabs_to_close.push(tab.clone());
            }
        }
    }
    
    for tab in tabs_to_close {
        let _ = tab.close(false);
    }

    // 2. 恢复 cookies（在页面创建之前）
    if !state.state.cookies.is_empty() {
        // headless_chrome 需要在特定页面上设置 cookies
        // 我们先创建一个临时页面来设置 cookies
        if let Ok(temp_tab) = browser.new_tab() {
            let cookie_params: Vec<CookieParam> = state.state.cookies.iter().map(|cookie| {
                CookieParam {
                    name: cookie.name.clone(),
                    value: cookie.value.clone(),
                    url: None,
                    domain: Some(cookie.domain.clone()),
                    path: Some(cookie.path.clone()),
                    secure: Some(cookie.secure),
                    http_only: Some(cookie.http_only),
                    same_site: None,
                    expires: cookie.expires,
                    priority: None,
                    same_party: None,
                    source_scheme: None,
                    source_port: None,
                    partition_key: None,
                }
            }).collect();
            let _ = temp_tab.set_cookies(cookie_params);
            let _ = temp_tab.close(false);
        }
    }

    // 3. 确定要恢复的标签
    let tabs_to_restore = if load_only_active_tab {
        if state.active_tab_index < state.tabs.len() {
            vec![state.tabs[state.active_tab_index].clone()]
        } else {
            warn!("Invalid active tab index: {}, using first tab", state.active_tab_index);
            vec![state.tabs.first().ok_or_else(|| anyhow!("No tabs to restore"))?.clone()]
        }
    } else {
        state.tabs.clone()
    };

    let mut restored_tabs = vec![];

    // 4. 创建新标签页并跳转
    for tab_state in &tabs_to_restore {
        let tab = browser
            .new_tab()
            .map_err(|e| anyhow!("Failed to create new tab: {}", e))?;
        
        if !tab_state.url.is_empty() && tab_state.url != "about:blank" {
            tab.navigate_to(&tab_state.url)
                .map_err(|e| anyhow!("Failed to navigate to {}: {}", tab_state.url, e))?;
            
            // 等待页面加载完成
            let _ = tab.wait_until_navigated();
        }

        // 恢复 local storage
        let tab_origin = extract_origin(&tab_state.url);
        for origin_state in &state.state.origins {
            if origin_state.origin == tab_origin {
                for entry in &origin_state.local_storage {
                    let js = format!(
                        r#"
                        () => {{
                            try {{
                                localStorage.setItem('{}', '{}');
                            }} catch (e) {{
                                console.error('Failed to set localStorage:', e);
                            }}
                        }}
                        "#,
                        entry.key.replace("'", "\\'").replace("\\", "\\\\"),
                        entry.value.replace("'", "\\'").replace("\\", "\\\\")
                    );
                    let _ = tab.evaluate(&js, false);
                }
            }
        }

        // 滚动到位置
        if tab_state.scroll_x != 0 || tab_state.scroll_y != 0 {
            let js = format!("() => window.scrollTo({}, {})", tab_state.scroll_x, tab_state.scroll_y);
            let _ = tab.evaluate(&js, false);
        }

        restored_tabs.push(tab);
    }

    // 5. 激活正确的标签页
    if !restored_tabs.is_empty() {
        let active_index = if load_only_active_tab {
            0
        } else {
            state.active_tab_index.min(restored_tabs.len() - 1)
        };
        let _ = restored_tabs[active_index].activate();

        sleep(Duration::from_millis(5000)).await;
    }

    Ok(restored_tabs)
}