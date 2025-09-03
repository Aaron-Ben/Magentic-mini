use anyhow::{anyhow, Result};
use chromiumoxide::browser::{Browser};
use chromiumoxide::page::Page;
use chromiumoxide::cdp::browser_protocol::network::{Cookie, CookieParam, GetCookiesParams, SetCookiesParams, TimeSinceEpoch};
use log::{warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub cookies: Vec<Cookie>,
    pub origins: Vec<OriginState>,
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

async fn get_scroll_position(page: &Page) -> Result<(i64, i64), anyhow::Error> {
    let result = page
        .evaluate("() => ({ scrollX: window.scrollX, scrollY: window.scrollY })")
        .await
        .map_err(|e| anyhow!("Failed to evaluate scroll position: {}", e))?;
    let value: HashMap<String, f64> = result
        .into_value()
        .map_err(|e| anyhow!("Failed to parse scroll position: {}", e))?;
    Ok((
        value.get("scrollX").unwrap_or(&0.0).round() as i64,
        value.get("scrollY").unwrap_or(&0.0).round() as i64,
    ))
}

async fn get_storage_state(pages: &[Page]) -> Result<StorageState, anyhow::Error> {
    let mut cookies = Vec::new();
    let mut origins = Vec::new();

    // 获取 cookies（从第一个页面，假设共享上下文）
    if let Some(first_page) = pages.first() {
        cookies = first_page
            .execute(GetCookiesParams::default())
            .await
            .map(|res| res.cookies.clone())
            .map_err(|e| anyhow!("Failed to get cookies: {}", e))?;
    }

    // 获取每个页面的 local storage，按 origin 分组
    for page in pages {
        let page_url = page
            .url()
            .await
            .map_err(|e| anyhow!("Failed to get page URL: {}", e))?
            .unwrap_or_default();
        
        let origin = extract_origin(&page_url);
        
        let local_storage: HashMap<String, String> = page
            .evaluate(
                r#"
                () => {
                    const o = {};
                    for (let i = 0; i < localStorage.length; i++) {
                        const k = localStorage.key(i);
                        if (k) o[k] = localStorage.getItem(k) || "";
                    }
                    return o;
                }
                "#,
            )
            .await
            .map_err(|e| anyhow!("Failed to evaluate localStorage script: {}", e))?
            .into_value()
            .map_err(|e| anyhow!("Failed to get local storage: {}", e))?;

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

    Ok(StorageState { cookies, origins })
}

pub async fn save_browser_state(
    pages: &[Page],
    controlled_page: Option<&Page>,
    simplified: bool,
) -> Result<BrowserState, anyhow::Error> {
    let mut active_tab_index = 0;

    // 查找 controlled_page 的索引
    if let Some(ctrl_page) = controlled_page {
        if let Some(i) = pages.iter().position(|p| p.target_id() == ctrl_page.target_id()) {
            active_tab_index = i;
        }
    }

    // 收集所有标签页状态
    let mut tabs = Vec::with_capacity(pages.len());

    for (i, page) in pages.iter().enumerate() {
        let url = page
            .url()
            .await
            .map_err(|e| anyhow!("Failed to get page URL: {}", e))?
            .unwrap_or_else(|| "about:blank".to_string());

        let (scroll_x, scroll_y) = if simplified {
            (0, 0)
        } else {
            get_scroll_position(page).await?
        };

        tabs.push(Tab {
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
        get_storage_state(pages).await?
    };

    Ok(BrowserState {
        state,
        tabs,
        active_tab_index,
    })
}

pub async fn load_browser_state(
    browser: &Browser,
    state: BrowserState,
    load_only_active_tab: bool,
) -> Result<(), anyhow::Error> {
    // 1. 关闭所有 about:blank 页面
    let pages = browser.pages().await?;
    let mut pages_to_close = vec![];
    for page in pages {
        let page_url = page
            .url()
            .await
            .map_err(|e| anyhow!("Failed to get page URL: {}", e))?
            .unwrap_or_default();
        if page_url == "about:blank" {
            pages_to_close.push(page);
        }
    }
    for page in pages_to_close {
        let _ = page.close().await;
    }

    // 2. 恢复存储状态
    if !state.state.cookies.is_empty() {
        let cookie_params: Vec<CookieParam> = state.state.cookies
            .iter()
            .map(|cookie| CookieParam {
                name: cookie.name.clone(),
                value: cookie.value.clone(),
                url: None,
                domain: Some(cookie.domain.clone()),
                path: Some(cookie.path.clone()),
                secure: Some(cookie.secure),
                http_only: Some(cookie.http_only),
                same_site: cookie.same_site.clone(),
                expires: Some(TimeSinceEpoch::new(cookie.expires)),
                priority: Some(cookie.priority.clone()),
                same_party: Some(cookie.same_party),
                source_scheme: Some(cookie.source_scheme.clone()),
                source_port: Some(cookie.source_port),
                partition_key: cookie.partition_key.clone(),
            })
            .collect();

        browser
            .execute(SetCookiesParams {
                cookies: cookie_params,
            })
            .await
            .map_err(|e| anyhow!("Failed to set cookies: {}", e))?;
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

    let mut restored_pages = vec![];

    // 4. 创建新页面并跳转
    for tab in &tabs_to_restore {
        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| anyhow!("Failed to create new page: {}", e))?;
        
        if !tab.url.is_empty() && tab.url != "about:blank" {
            page.goto(&tab.url)
                .await
                .map_err(|e| anyhow!("Failed to navigate to {}: {}", tab.url, e))?;
            
            // 等待页面加载完成
            let _ = page.wait_for_navigation().await;
        }

        // 恢复 local storage
        let tab_origin = extract_origin(&tab.url);
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
                    let _ = page.evaluate(js.as_str()).await;
                }
            }
        }

        // 滚动到位置
        if tab.scroll_x != 0 || tab.scroll_y != 0 {
            let js = format!("() => window.scrollTo({}, {})", tab.scroll_x, tab.scroll_y);
            let _ = page.evaluate(js.as_str()).await;
        }

        restored_pages.push(page);
    }

    // 5. 激活正确的标签页
    if !restored_pages.is_empty() {
        let active_index = if load_only_active_tab {
            0
        } else {
            state.active_tab_index.min(restored_pages.len() - 1)
        };
        restored_pages[active_index]
            .bring_to_front()
            .await
            .map_err(|e| anyhow!("Failed to bring page to front: {}", e))?;

        sleep(Duration::from_millis(1000)).await;
    }

    Ok(())
}