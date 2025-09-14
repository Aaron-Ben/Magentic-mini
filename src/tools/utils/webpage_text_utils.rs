use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use html2md;
use log::{error, info};
use pdf_extract;
use reqwest;
use tempfile::NamedTempFile;

pub struct WebpageTextUtils {
    page_script: String,
}

impl WebpageTextUtils {
    pub fn new() -> Self {
        let script_path = Path::new(file!())
            .parent()
            .unwrap()
            .join("..")
            .join("chromiumoxide")
            .join("page_script.js");
        let page_script = fs::read_to_string(script_path).unwrap_or_else(|_| {
            error!("Failed to read page_script.js from chromiumoxide directory, using empty script");
            String::new()
        });

        Self { page_script }
    }

    pub async fn get_all_webpage_text(&self, tab: &Arc<Tab>, n_lines: usize) -> String {
        let result = tab.evaluate("document.body.innerText", false);
        let text = match result {
            Ok(eval_result) => {
                if let Some(value) = eval_result.value {
                    serde_json::from_value::<String>(value).unwrap_or_default()
                } else {
                    String::new()
                }
            }
            Err(_) => String::new(),
        };

        text.lines()
            .take(n_lines)
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub async fn get_visible_text(&self, tab: &Arc<Tab>) -> String {
        
        if let Err(e) = tab.evaluate(&self.page_script, false) {
            eprintln!("Failed to evaluate page_script.js: {:?}", e);
        }

        let result = tab.evaluate("WebSurfer.getVisibleText();", true);

        match result {
            Ok(eval_result) => eval_result
                .value
                .and_then(|value| serde_json::from_value::<String>(value).ok())
                .unwrap_or_default(),
            Err(e) => {
                eprintln!("Failed to evaluate WebSurfer.getVisibleText(): {:?}", e);
                String::new()
            }
        }
    }

    pub async fn get_page_markdown(&self, tab: &Arc<Tab>, max_tokens: i32) -> String {
        let is_pdf = self.is_pdf_page(tab).await;

        let content = if is_pdf {
            self.extract_pdf_content(tab).await
        } else {
            // 注入 page_script.js 并获取HTML
            let _ = tab.evaluate(&self.page_script, false);
            
            let result = tab.evaluate("document.documentElement.outerHTML", false);
            let html = match result {
                Ok(eval_result) => {
                    if let Some(value) = eval_result.value {
                        serde_json::from_value::<String>(value).unwrap_or_default()
                    } else {
                        String::new()
                    }
                }
                Err(_) => String::new(),
            };
            html2md::parse_html(&html).to_string()
        };

        if max_tokens == -1 {
            return content;
        }

        // 针对 qwen-turbo 模型优化：使用字符数估算
        // 中文字符约 1 字符 = 1.2-1.5 tokens，这里使用保守估算
        let estimated_chars = (max_tokens as f32 * 0.7) as usize;
        
        if content.chars().count() <= estimated_chars {
            return content;
        }

        let truncated: String = content.chars().take(estimated_chars).collect();
        
        if let Some(last_period) = truncated.rfind('。') {
            truncated[..=last_period].to_string()
        } else if let Some(last_newline) = truncated.rfind('\n') {
            truncated[..=last_newline].to_string()
        } else {
            truncated
        }
    }

    /// 获取清理后的页面 Markdown，移除冗余内容
    pub async fn get_clean_page_markdown(&self, tab: &Arc<Tab>, max_tokens: i32) -> String {
        let content = self.get_page_markdown(tab, max_tokens).await;
        
        content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
            .replace("\n\n\n", "\n\n") // 减少多余的空行
    }

    async fn is_pdf_page(&self, tab: &Arc<Tab>) -> bool {
        let url = tab.get_url();
        
        if url.to_lowercase().ends_with(".pdf") {
            return true;
        }

        let js = r#"
            () => {
                if (document.contentType === 'application/pdf') return true;
                if (document.querySelector('embed[type="application/pdf"]') || 
                    document.querySelector('object[type="application/pdf"]')) return true;
                if (window.PDFViewerApplication || document.querySelector('#viewer.pdfViewer')) return true;
                return false;
            }
        "#;

        let result = tab.evaluate(js, false);
        match result {
            Ok(eval_result) => {
                if let Some(value) = eval_result.value {
                    serde_json::from_value::<bool>(value).unwrap_or_default()
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    async fn extract_pdf_content(&self, tab: &Arc<Tab>) -> String {
        let url = tab.get_url();
        
        if url.is_empty() {
            return "Error: page has no URL".to_string();
        }

        // 尝试从浏览器提取
        let browser_text = self.extract_pdf_browser(tab).await;
        if browser_text.len() > 100 {
            return browser_text;
        }

        info!("Using pdf_extract for better PDF extraction...");

        let client = reqwest::Client::new();
        let response = match client.get(&url).send().await {
            Ok(res) => res,
            Err(e) => return format!("Error downloading PDF: {}", e),
        };

        let pdf_data = match response.bytes().await {
            Ok(data) => data.to_vec(),
            Err(e) => return format!("Error reading PDF bytes: {}", e),
        };

        let mut temp_file = match NamedTempFile::new() {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to create temp file: {}", e);
                return "Error: failed to create temp file".to_string();
            }
        };

        if let Err(e) = temp_file.write_all(&pdf_data) {
            error!("Failed to write PDF to temp file: {}", e);
            return "Error: failed to write to temp file".to_string();
        }

        let result = pdf_extract::extract_text(temp_file.path());
        let _ = temp_file.close();

        match result {
            Ok(text) => text,
            Err(e) => {
                error!("Error extracting PDF text: {}", e);
                format!("Error extracting PDF text: {}", e)
            }
        }
    }

    async fn extract_pdf_browser(&self, tab: &Arc<Tab>) -> String {
        let js = r#"
            () => {
                if (window.PDFViewerApplication) {
                    const divs = document.querySelectorAll('.textLayer div');
                    return Array.from(divs)
                        .map(d => d.textContent || '')
                        .filter(t => t.trim())
                        .join('\\n');
                }
                const els = Array.from(document.querySelectorAll('p, span, div'))
                    .filter(el => {
                        const style = window.getComputedStyle(el);
                        return style.display !== 'none' && 
                               style.visibility !== 'hidden' && 
                               el.textContent.trim();
                    });
                return els.map(el => el.textContent || '').join('\\n');
            }
        "#;

        let result = tab.evaluate(js, false);
        match result {
            Ok(eval_result) => {
                if let Some(value) = eval_result.value {
                    serde_json::from_value::<String>(value).unwrap_or_default()
                } else {
                    String::new()
                }
            }
            Err(_) => String::new(),
        }
    }
}