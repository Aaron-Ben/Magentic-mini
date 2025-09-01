use chromiumoxide::Page;
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;
use tiktoken_rs::cl100k_base;
use html2md;
use reqwest;
use anyhow::{Result, anyhow};
use pdf_extract;
use bytes::Bytes;

pub struct WebpageTextUtils {
    client: reqwest::Client,
}

impl WebpageTextUtils {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// 获取页面 innerText（前 N 行）
    pub async fn get_all_webpage_text(
        &self,
        page: &Page,
        n_lines: usize,
    ) -> Result<String> {
        let text: String = page
            .evaluate(
                r#"() => document.body.innerText || document.documentElement.innerText"#,
                json!([]),
            )
            .await?
            .try_into()
            .map_err(|_| anyhow!("Failed to parse text"))?;

        let lines: Vec<&str> = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .take(n_lines)
            .collect();

        Ok(lines.join("\n"))
    }

    /// 获取"可视区域"文本
    pub async fn get_visible_text(&self, page: &Page) -> Result<String> {
        let script = r#"
        (() => {
            function isElementVisible(el) {
                const rect = el.getBoundingClientRect();
                const style = window.getComputedStyle(el);
                return rect.width > 0 && 
                       rect.height > 0 && 
                       style.display !== 'none' && 
                       style.visibility !== 'hidden' &&
                       style.opacity !== '0';
            }
            
            const textElements = Array.from(document.querySelectorAll('*'))
                .filter(el => el.children.length === 0 && el.textContent.trim())
                .filter(isElementVisible);
                
            return textElements.map(el => el.textContent.trim()).join('\n');
        })()
        "#;

        let result: String = page
            .evaluate(script, json!([]))
            .await?
            .try_into()
            .map_err(|_| anyhow!("Expected string"))?;

        Ok(result)
    }

    /// 检查是否为 PDF 页面
    pub async fn is_pdf_page(&self, page: &Page) -> Result<bool> {
        let url = page.evaluate("() => window.location.href", json!([])).await?;
        let url: String = url.try_into().unwrap_or_default();

        if url.to_lowercase().ends_with(".pdf") {
            return Ok(true);
        }

        let is_pdf: bool = page
            .evaluate(
                r#"
                () => {
                    if (document.contentType === 'application/pdf') return true;
                    if (document.querySelector('embed[type="application/pdf"]') ||
                        document.querySelector('object[type="application/pdf"]')) return true;
                    if (window.PDFViewerApplication || document.querySelector('#viewer.pdfViewer')) return true;
                    return false;
                }
                "#,
                json!([]),
            )
            .await?
            .try_into()
            .unwrap_or(false);

        Ok(is_pdf)
    }

    /// 从 PDF 页面提取文本（优先浏览器内提取）
    pub async fn extract_pdf_content(&self, page: &Page) -> Result<String> {
        let url = page
            .evaluate("() => window.location.href", json!([]))
            .await?
            .try_into::<String>()
            .unwrap_or_default();

        // 1. 尝试浏览器内提取
        if let Ok(text) = self.extract_pdf_browser(page).await {
            if text.len() > 100 {
                return Ok(text);
            }
        }

        // 2. 下载 PDF 并解析
        println!("📄 Downloading PDF for text extraction...");
        match self.download_and_extract_pdf(&url).await {
            Ok(text) => {
                if text.is_empty() {
                    Ok("PDF 内容提取成功，但未找到文本内容。".to_string())
                } else {
                    Ok(text)
                }
            }
            Err(e) => {
                eprintln!("❌ PDF extraction failed: {}", e);
                Ok(format!("PDF 内容提取失败: {}", e))
            }
        }
    }

    async fn extract_pdf_browser(&self, page: &Page) -> Result<String> {
        let text: String = page
            .evaluate(
                r#"
                () => {
                    // PDF.js viewer
                    if (window.PDFViewerApplication) {
                        const divs = document.querySelectorAll('.textLayer div');
                        return Array.from(divs)
                            .map(d => d.textContent || '')
                            .join('\n');
                    }
                    // Fallback: visible text elements
                    const els = Array.from(document.querySelectorAll('p, span, div'))
                        .filter(el => {
                            const s = getComputedStyle(el);
                            return s.display !== 'none' && 
                                   s.visibility !== 'hidden' && 
                                   el.textContent?.trim();
                        });
                    return els.map(el => el.textContent || '').join('\n');
                }
                "#,
                json!([]),
            )
            .await?
            .try_into()
            .map_err(|_| anyhow!("Not a string"))?;

        Ok(text)
    }

    /// 下载 PDF 并提取文本内容
    async fn download_and_extract_pdf(&self, url: &str) -> Result<String> {
        // 下载 PDF 文件
        let response = self.client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Magentic-mini PDF Downloader)")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("无法下载 PDF: HTTP {}", response.status()));
        }

        let pdf_data = response.bytes().await?;
        
        // 检查是否为有效的 PDF 文件
        if !pdf_data.starts_with(b"%PDF") {
            return Err(anyhow!("下载的文件不是有效的 PDF 格式"));
        }

        // 使用 pdf-extract crate 提取文本
        match self.extract_text_from_pdf_bytes(&pdf_data) {
            Ok(text) => Ok(text),
            Err(e) => {
                // 如果 pdf-extract 失败，尝试 lopdf
                self.extract_text_with_lopdf(&pdf_data).await
                    .map_err(|_| anyhow!("PDF 文本提取失败: {}", e))
            }
        }
    }

    /// 使用 pdf-extract crate 提取 PDF 文本
    fn extract_text_from_pdf_bytes(&self, pdf_data: &Bytes) -> Result<String> {
        // 创建临时文件
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(pdf_data)?;
        let temp_path = temp_file.path();

        // 使用 pdf-extract 提取文本
        let text = pdf_extract::extract_text(temp_path)
            .map_err(|e| anyhow!("pdf-extract 提取失败: {}", e))?;

        Ok(text)
    }

    /// 使用 lopdf crate 提取 PDF 文本（备用方案）
    async fn extract_text_with_lopdf(&self, pdf_data: &Bytes) -> Result<String> {
        use lopdf::Document;
        use std::io::Cursor;

        let mut cursor = Cursor::new(pdf_data);
        let doc = Document::load_from(&mut cursor)
            .map_err(|e| anyhow!("lopdf 加载失败: {}", e))?;

        let mut text = String::new();

        // 遍历所有页面
        for page_id in doc.get_pages().keys() {
            if let Ok(page_text) = doc.extract_text(&[*page_id]) {
                text.push_str(&page_text);
                text.push('\n');
            }
        }

        Ok(text.trim().to_string())
    }

    /// 获取网页 Markdown（HTML → Markdown）
    pub async fn get_page_markdown(
        &self,
        page: &Page,
        max_tokens: i32,
    ) -> Result<String> {
        // 检查是否为 PDF
        if self.is_pdf_page(page).await? {
            return self.extract_pdf_content(page).await;
        }

        // 获取 HTML
        let html: String = page
            .evaluate("() => document.documentElement.outerHTML", json!([]))
            .await?
            .try_into()
            .map_err(|_| anyhow!("HTML not string"))?;

        // 使用 html2md 库转换为 Markdown
        let markdown = html2md::parse_html(&html);

        // Token 截断
        let limited_markdown = if max_tokens == -1 {
            markdown
        } else {
            let encoder = cl100k_base().map_err(|e| anyhow!("Tokenizer err: {}", e))?;
            let tokens = encoder.encode(&markdown, None);
            let truncated_tokens = &tokens[..std::cmp::min(tokens.len(), max_tokens as usize)];
            encoder.decode(truncated_tokens).map_err(|e| anyhow!(e.to_string()))?
        };

        Ok(limited_markdown)
    }

    /// 清理 HTML 内容并转换为更简洁的 Markdown
    pub async fn get_clean_page_markdown(
        &self,
        page: &Page,
        max_tokens: i32,
    ) -> Result<String> {
        // 检查是否为 PDF
        if self.is_pdf_page(page).await? {
            return self.extract_pdf_content(page).await;
        }

        // 获取主要内容区域的 HTML
        let content_html: String = page
            .evaluate(
                r#"
                () => {
                    // 尝试找到主要内容区域
                    const candidates = [
                        'main', 'article', '.content', '#content', 
                        '.main-content', '.article-content', '.post-content',
                        '[role="main"]', '.container', '.wrapper'
                    ];
                    
                    for (const selector of candidates) {
                        const element = document.querySelector(selector);
                        if (element && element.textContent.trim().length > 100) {
                            return element.innerHTML;
                        }
                    }
                    
                    // 如果没找到，使用 body
                    return document.body.innerHTML;
                }
                "#,
                json!([]),
            )
            .await?
            .try_into()
            .map_err(|_| anyhow!("HTML not string"))?;

        // 转换为 Markdown
        let markdown = html2md::parse_html(&content_html);

        // 清理多余的空行和格式
        let cleaned_markdown = markdown
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        // Token 截断
        let limited_markdown = if max_tokens == -1 {
            cleaned_markdown
        } else {
            let encoder = cl100k_base().map_err(|e| anyhow!("Tokenizer err: {}", e))?;
            let tokens = encoder.encode(&cleaned_markdown, None);
            let truncated_tokens = &tokens[..std::cmp::min(tokens.len(), max_tokens as usize)];
            encoder.decode(truncated_tokens).map_err(|e| anyhow!(e.to_string()))?
        };

        Ok(limited_markdown)
    }

    /// 下载文件到临时位置
    pub async fn download_file(&self, url: &str) -> Result<tempfile::NamedTempFile> {
        let response = self.client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Magentic-mini File Downloader)")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("无法下载文件: HTTP {}", response.status()));
        }

        let mut temp_file = NamedTempFile::new()?;
        let content = response.bytes().await?;
        temp_file.write_all(&content)?;

        Ok(temp_file)
    }

    /// 检测文件类型
    pub fn detect_file_type(&self, url: &str) -> String {
        let url_lower = url.to_lowercase();
        
        if url_lower.ends_with(".pdf") {
            "pdf".to_string()
        } else if url_lower.ends_with(".doc") || url_lower.ends_with(".docx") {
            "document".to_string()
        } else if url_lower.ends_with(".txt") {
            "text".to_string()
        } else if url_lower.contains("pdf") {
            "pdf".to_string()
        } else {
            "unknown".to_string()
        }
    }
}

impl Default for WebpageTextUtils {
    fn default() -> Self {
        Self::new()
    }
}