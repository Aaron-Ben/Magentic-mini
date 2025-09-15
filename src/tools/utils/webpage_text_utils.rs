use std::fmt::Debug;
use std::io::Write;
use pdf_extract::extract_text;
use tiktoken_rs::cl100k_base;
use std::sync::Arc;
use reqwest::Client;
use tempfile::NamedTempFile;
use thirtyfour::error::{WebDriverErrorInfo, WebDriverErrorValue};
use thirtyfour::prelude::*;
use serde_json::Value;
use html2md::parse_html;


#[derive(Debug,Clone)]
pub struct WebpageTextUtils {
    driver: Arc<WebDriver>,
}

impl WebpageTextUtils {
    pub fn new(driver: Arc<WebDriver>) -> Self {
        Self { driver }
    }

    pub async fn get_all_webpage_text(&self, n_lines: Option<usize>) -> String {
        let n_lines = match n_lines {
            Some(val) => val,
            None => 50,
        };
        match self.driver.find(By::Tag("body")).await {
            Ok(body_element) => {
                match body_element.text().await {
                    Ok(text_in_viewport) => {
                        // 按换行符分割文本
                        let lines: Vec<&str> = text_in_viewport.split('\n').collect();
                        
                        // 截取前n_lines行
                        let end_idx = std::cmp::min(n_lines,lines.len());
                        let limited_lines = &lines[0..end_idx];
                        
                        // 过滤空行
                        let non_empty_lines: Vec<&str> = limited_lines
                            .iter()
                            .filter(|&&line| !line.trim().is_empty())
                            .cloned()
                            .collect();
                        
                        non_empty_lines.join("\n")
                    }
                    Err(_) => String::new()
                }
            }
            Err(_) => String::new()
        }
    }

    async fn is_pdf_page(&self) -> Result<bool,WebDriverError> {
        let url = self.driver.current_url().await?;
        if url.to_string().to_lowercase().ends_with(".pdf") {
            return Ok(true);
        }

        let result = self.driver
            .execute(r#"
                // 检查内容类型
                if (document.contentType === 'application/pdf') return true;
                
                // 检查 PDF 嵌入元素
                if (document.querySelector('embed[type="application/pdf"]') || 
                    document.querySelector('object[type="application/pdf"]')) return true;
                
                // 检查 PDF.js 查看器
                if (window.PDFViewerApplication || document.querySelector('#viewer.pdfViewer')) return true;
                
                return false;
            "#, Vec::new()).await?;

        let value: Value = result.json().clone();

        let is_pdf = match value.as_bool() {
            Some(b) => b,
            None => {
                eprintln!("JavaScript 返回值不是布尔值，而是: {:?}", value);
                return Err(WebDriverError::JavascriptError(
                    WebDriverErrorInfo::new(
                        "JavaScript 返回值无法转换为布尔值".to_string(),
                    )
                ));
            }
        };
        Ok(is_pdf)
    }

    // 网页处理工具：网页（PDF界面）转化为Markdown
    pub async fn get_page_markdown(&self, max_tokens: i32) -> Result<String,WebDriverError> {
        let is_pdf = self.is_pdf_page().await?;

        let raw_content = if is_pdf {
            // PDF
            self.extract_pdf_content().await?
        } else {
            // 普通网页：HTML --> MarkDown
            self.extract_html_markdown().await?
        };

        if max_tokens == -1 {
            Ok(raw_content)
        }else {
            Ok(self.limit_token(&raw_content, max_tokens as usize).await?)
        }

    }

    // 限制tokn数量
    async fn limit_token(&self, content: &str, max_tokens: usize) -> Result<String, WebDriverError>{
        if content.is_empty() {
            return Ok(String::new())
        }

        let bpe = cl100k_base()
            .map_err(|e| WebDriverError)?;

        let tokens = bpe.encode_with_special_tokens(content);

        if tokens.len() <= max_tokens {
            Ok(content.to_string())
        } else {
            let limited_tokens = &tokens[..max_tokens];
            let limited_content = bpe.decode(limited_tokens)
                .map_err(|e| WebDriverError)?;

            Ok(limited_content)
        }


        // let tokenizer = BertTokenizer::from_pretrained(
        //     "bert-base-chinese",
        //     Some("https://huggingface.co/bert-base-chinese/resolve/main/vocab.txt"),
        // ).map_err(|e| WebDriverError::UnknownError(format!("初始化分词器失败: {}", e)))?;

        // // 分词（保留特殊标记）
        // let encoding = tokenizer.encode(
        //     Some(content),
        //     None,
        //     max_tokens,
        //     Tokenizer::TruncationStrategy::LongestFirst,
        //     0,
        //     None,
        //     false,
        //     false,
        // ).map_err(|e| WebDriverError::UnknownError(format!("分词失败: {}", e)))?;

        // // 解码截断后的token
        // let limited_content = tokenizer.decode(
        //     &encoding.get_ids()[..encoding.get_ids().len().min(max_tokens)],
        //     true,
        // ).map_err(|e| WebDriverError::UnknownError(format!("解码失败: {}", e)))?;

    }

    // 提取HTML转化为Markdown
    async fn extract_html_markdown(&self) -> Result<String,WebDriverError> {
        let html = self.driver
            .execute("return document.documentElement.outerHTML;", Vec::new())
            .await?;

        let html_str = match html.json() {
            serde_json::Value::String(s) => s,
            _ => {
                let err_info = WebDriverErrorInfo {
                    status: 500,
                    error: "JavaScript execution did not return a string".to_string(),
                    value: WebDriverErrorValue {
                        message: "extract_html_markdown-WebDriverErrorValue".to_string(),
                        error:None,
                        stacktrace:None,
                        data:None
                    }
                };
                return Err(WebDriverError::InvalidArgument(err_info));
            }
        };

        self.driver.current_url().await?;

        let markdown = parse_html(&html_str);

        Ok(markdown)
    
    }

    // 从pdf 提取文本（高级实现，更好的错误处理）
    async fn extract_pdf_content(&self) -> Result<String,WebDriverError> {
        let url = self.driver.current_url().await?;
        

        let browser_text = self.extract_pdf_browser().await?;
        if !browser_text.is_empty() && browser_text.len() > 100 {
            return Ok(browser_text)
        }

        // 下载PDF文件
        let client = Client::new();
        let response  = client.get(url)
            .send()
            .await
            .map_err(|e| {
                WebDriverError::UnknownError(WebDriverErrorInfo::new(format!("下载失败：{}",e)))
            })?;

        let pdf_data = response.bytes()
            .await
            .map_err(|e| {
                WebDriverError::UnknownError(WebDriverErrorInfo::new(format!("错误读取数据：{}",e)))
            })?;

        let mut temp_file = NamedTempFile::new_in("pdf_extract_")
            .map_err(|e| {
                WebDriverError::UnknownError(WebDriverErrorInfo::new(format!("Failed to create temp file: {}", e)))
            })?;
        
        // 写入PDF数据到临时文件
        temp_file.write_all(&pdf_data)
            .map_err(|e| {
                WebDriverError::UnknownError(WebDriverErrorInfo::new(format!("Failed to write to temp file: {}", e)))
            })?;
        
        // 获取临时文件路径
        let temp_path = temp_file.path().to_path_buf();
        
        // 保持临时文件不被删除
        let _temp_file = temp_file.into_temp_path();
        
        // 使用pdf_extract库提取文本
        let text_content = extract_text(temp_path)
            .map_err(|e| {
                WebDriverError::UnknownError(WebDriverErrorInfo::new(format!("Failed to extract text from PDF: {}", e)))
            })?;
        
        // 临时文件会在这里自动清理
        
        Ok(text_content)

    }

    // 从 pdf 提取文本（底层封装）
    async fn extract_pdf_browser(&self) -> Result<String,WebDriverError> {
        let script = r#"
            // For PDF.js viewer
            if (window.PDFViewerApplication) {
                const textContent = document.querySelectorAll('.textLayer div');
                if (textContent.length > 0) {
                    return Array.from(textContent).map(div => div.textContent).join('\\n');
                }
            }
            
            // For embedded PDFs, try to get text from visible elements
            const textElements = Array.from(document.querySelectorAll('p, span, div'))
                .filter(el => {
                    const style = window.getComputedStyle(el);
                    return style.display !== 'none' && 
                           style.visibility !== 'hidden' && 
                           el.textContent.trim() !== '';
                });
        "#;

        // 获取 ScriptRet
        let script_ret: ScriptRet = self.driver
            .execute(script, Vec::new())
            .await?;

        // 从 ScriptRet 中提取字符串值
        match script_ret.json() {
            Value::String(text) => Ok(text.clone()),
            Value::Null => Ok(String::new()),
            _ => {
                eprintln!("警告：PDF提取返回了非字符串值: {:?}", script_ret.json());
                Ok(String::new())
            }
        }
    }

}