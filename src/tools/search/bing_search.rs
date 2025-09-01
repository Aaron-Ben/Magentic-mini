use anyhow::{Result, anyhow};
use log::{info, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use url::Url;
use urlencoding::encode;
use tiktoken_rs::cl100k_base;

use crate::tools::chromiumoxide::ChromiumoxideController;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BingSearchResults {
    pub search_results: String,
    pub links: Vec<LinkInfo>,
    pub page_contents: HashMap<String, String>,
    pub combined_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInfo {
    pub display_text: String,
    pub url: String,
}

/// Extract markdown content from a given URL
/// 
/// # Arguments
/// * `url` - The URL to extract content from
/// 
/// # Returns
/// A tuple containing the URL and the markdown content extracted from the page
pub async fn extract_page_markdown(url: &str) -> Result<(String, String)> {
    let mut controller = ChromiumoxideController::new(
        false, // animate_actions
        1280,  // viewport_width
        720,   // viewport_height
        Duration::from_secs(30), // timeout_load
        Duration::from_millis(500), // sleep_after_action
        true,  // single_tab_mode
    );

    match controller.initialize().await {
        Ok(_) => {
            match controller.new_page(None).await {
                Ok(_) => {
                    match controller.visit_page(url).await {
                        Ok(_) => {
                            // 等待页面加载
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            
                            match controller.get_page_markdown(None).await {
                                Ok(markdown) => {
                                    let _ = controller.close().await;
                                    Ok((url.to_string(), markdown))
                                }
                                Err(e) => {
                                    error!("Error extracting markdown content: {}", e);
                                    let _ = controller.close().await;
                                    Ok((url.to_string(), "Error extracting content".to_string()))
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error visiting page {}: {}", url, e);
                            let _ = controller.close().await;
                            Ok((url.to_string(), "Error extracting content".to_string()))
                        }
                    }
                }
                Err(e) => {
                    error!("Error creating new page: {}", e);
                    let _ = controller.close().await;
                    Ok((url.to_string(), "Error extracting content".to_string()))
                }
            }
        }
        Err(e) => {
            error!("Error initializing browser: {}", e);
            Ok((url.to_string(), "Error extracting content".to_string()))
        }
    }
}

/// Check if a URL is valid
fn is_valid_url(url: &str) -> bool {
    match Url::parse(url) {
        Ok(parsed_url) => {
            matches!(parsed_url.scheme(), "http" | "https") && parsed_url.host().is_some()
        }
        Err(_) => false,
    }
}

/// Extract links from markdown text
/// 
/// # Arguments
/// * `markdown_text` - The markdown text to extract links from
/// 
/// # Returns
/// Vector of LinkInfo containing display_text and url
fn extract_links(markdown_text: &str) -> Vec<LinkInfo> {
    let mut links = Vec::new();
    
    for line in markdown_text.lines() {
        // Match markdown link format: [display_text](url)
        if line.matches('[').count() == 1 
            && line.matches(']').count() == 1 
            && line.matches('(').count() == 1 
            && line.matches(')').count() == 1 {
            
            if let Some(display_start) = line.find('[') {
                if let Some(display_end) = line.find(']') {
                    if let Some(url_start) = line.find('(') {
                        if let Some(url_end) = line.find(')') {
                            if display_start < display_end && url_start < url_end && display_end < url_start {
                                let display_text = &line[display_start + 1..display_end];
                                let url = &line[url_start + 1..url_end];
                                
                                if is_valid_url(url) {
                                    links.push(LinkInfo {
                                        display_text: display_text.to_string(),
                                        url: url.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    links
}

/// Get the Bing search results for a given query
/// 
/// WARNING: This function spawns multiple browser instances which may consume a lot of resources and can cause risks.
/// 
/// # Arguments
/// * `query` - The search query to use
/// * `max_pages` - Maximum number of pages to extract (default: 3)
/// * `timeout_seconds` - Maximum time in seconds to wait for search results (default: 10)
/// * `max_tokens_per_page` - Maximum number of tokens to extract from each page (default: 10000)
/// 
/// # Returns
/// BingSearchResults containing search results markdown, links, and extracted content
pub async fn get_bing_search_results(
    query: &str,
    max_pages: Option<usize>,
    timeout_seconds: Option<u64>,
    max_tokens_per_page: Option<usize>,
) -> Result<BingSearchResults> {
    let max_pages = max_pages.unwrap_or(3);
    let timeout_seconds = timeout_seconds.unwrap_or(10);
    let max_tokens_per_page = max_tokens_per_page.unwrap_or(10000);
    
    let mut search_results = String::new();
    let mut links = Vec::new();
    let mut page_contents = HashMap::new();
    let mut combined_content = String::new();

    // Encode the search query
    let encoded_query = encode(query);
    let search_url = format!("https://www.bing.com/search?q={}&FORM=QBLH", encoded_query);

    // Extract search results page with timeout
    let search_result = timeout(
        Duration::from_secs(timeout_seconds),
        extract_page_markdown(&search_url)
    ).await;

    match search_result {
        Ok(Ok((_, markdown))) => {
            search_results = markdown.clone();
            
            // Extract links from search results
            links = extract_links(&markdown);
            
            // Get first few URLs for content extraction
            let first_few_urls: Vec<String> = links
                .iter()
                .take(max_pages)
                .map(|link| link.url.clone())
                .collect();

            // Extract content from pages in parallel
            let tasks: Vec<_> = first_few_urls
                .iter()
                .map(|url| extract_page_markdown(url))
                .collect();

            let extracted_contents = futures::future::join_all(tasks).await;

            // Process extracted contents
            for (i, result) in extracted_contents.into_iter().enumerate() {
                if let Ok((url, content)) = result {
                    if content != "Error extracting content" {
                        page_contents.insert(url, content);
                    }
                }
            }

            // Combine all extracted page contents
            if !page_contents.is_empty() {
                combined_content = format!("Search Results for {}\n\n", query);
                
                for (url, content) in &page_contents {
                    let token_limited_content = if max_tokens_per_page == 0 {
                        content.clone()
                    } else {
                        // Use tiktoken for token limiting
                        match cl100k_base() {
                            Ok(encoder) => {
                                let tokens = encoder.encode(&content, None);
                                let limited_tokens = if tokens.len() > max_tokens_per_page {
                                    &tokens[..max_tokens_per_page]
                                } else {
                                    &tokens
                                };
                                encoder.decode(limited_tokens).unwrap_or_else(|_| content.clone())
                            }
                            Err(_) => {
                                // Fallback to character-based limiting
                                if content.len() > max_tokens_per_page * 4 {
                                    content.chars().take(max_tokens_per_page * 4).collect()
                                } else {
                                    content.clone()
                                }
                            }
                        }
                    };
                    
                    combined_content.push_str(&format!("Page: {}\n{}\n\n", url, token_limited_content));
                }
            }
        }
        Ok(Err(e)) => {
            error!("Error extracting search results: {}", e);
            return Ok(BingSearchResults {
                search_results: String::new(),
                links: Vec::new(),
                page_contents: HashMap::new(),
                combined_content: String::new(),
            });
        }
        Err(_) => {
            info!("Timeout occurred while getting search results");
            // Return partial results if any
            if !page_contents.is_empty() {
                combined_content = format!("Search Results for {} (Partial results due to timeout)\n\n", query);
                for (url, content) in &page_contents {
                    let token_limited_content = if max_tokens_per_page == 0 {
                        content.clone()
                    } else {
                        match cl100k_base() {
                            Ok(encoder) => {
                                let tokens = encoder.encode(&content, None);
                                let limited_tokens = if tokens.len() > max_tokens_per_page {
                                    &tokens[..max_tokens_per_page]
                                } else {
                                    &tokens
                                };
                                encoder.decode(limited_tokens).unwrap_or_else(|_| content.clone())
                            }
                            Err(_) => {
                                if content.len() > max_tokens_per_page * 4 {
                                    content.chars().take(max_tokens_per_page * 4).collect()
                                } else {
                                    content.clone()
                                }
                            }
                        }
                    };
                    combined_content.push_str(&format!("Page: {}\n{}\n\n", url, token_limited_content));
                }
            } else {
                return Ok(BingSearchResults {
                    search_results: String::new(),
                    links: Vec::new(),
                    page_contents: HashMap::new(),
                    combined_content: String::new(),
                });
            }
        }
    }

    Ok(BingSearchResults {
        search_results,
        links,
        page_contents,
        combined_content,
    })
}