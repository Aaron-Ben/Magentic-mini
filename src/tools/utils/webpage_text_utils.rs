use std::fmt::{Debug};
use std::io::Write;
use std::error::Error;
use std::time::Duration;
use anyhow::anyhow;
use std::fmt;
use pdf_extract::extract_text;
use std::sync::Arc;
use tiktoken_rs::{
    CoreBPE,
    tokenizer::{get_tokenizer, Tokenizer}, 
    // 导入库内置的编码方案创建函数（对应 Tokenizer 枚举的每个变体）
    cl100k_base, o200k_base, p50k_base, r50k_base, p50k_edit,
};
use pdf_extract::OutputError;
use reqwest::Client;
use tempfile::NamedTempFile;
use thirtyfour::prelude::*;
use serde_json::Value;
use html2md::parse_html;

#[derive(Debug)]
pub enum WebpageTextError {
    /// WebDriver 相关错误（如元素查找失败、JS执行错误等）
    WebDriver(WebDriverError),
    /// HTTP 请求错误（如下载PDF失败）
    Http(reqwest::Error),
    /// PDF 文本提取错误（如解析PDF失败）
    PdfExtract(pdf_extract::Error),
    /// IO 操作错误（如临时文件创建/写入失败）
    Io(std::io::Error),
    /// 分词器相关错误（如tiktoken初始化/编码失败）
    Tiktoken(anyhow::Error),
    /// JSON 解析错误（如JS返回值解析失败）
    SerdeJson(serde_json::Error),
    /// 存储文本提取错误
    ExtractText(OutputError),
    /// HTML 处理失败
    Html(String),
    /// 自定义业务逻辑错误（如JS返回非预期类型、内容无效等）
    Custom(String),
}

impl fmt::Display for WebpageTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebpageTextError::WebDriver(e) => write!(f, "WebDriver操作失败: {}", e),
            WebpageTextError::Http(e) => write!(f, "HTTP请求失败: {}", e),
            WebpageTextError::PdfExtract(e) => write!(f, "PDF文本提取失败: {}", e),
            WebpageTextError::Io(e) => write!(f, "IO操作失败: {}", e),
            WebpageTextError::Tiktoken(e) => write!(f, "分词器操作失败: {}", e),
            WebpageTextError::SerdeJson(e) => write!(f, "JSON解析失败: {}", e),
            WebpageTextError::ExtractText(e) => write!(f, "文本提取错误: {}",e),
            WebpageTextError::Html(e) => write!(f,"html提取错误: {}",e),
            WebpageTextError::Custom(msg) => write!(f, "业务逻辑错误: {}", msg),
        }
    }
}

impl Error for WebpageTextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            WebpageTextError::WebDriver(e) => Some(e),
            WebpageTextError::Http(e) => Some(e),
            WebpageTextError::PdfExtract(e) => Some(e),
            WebpageTextError::Io(e) => Some(e),
            WebpageTextError::Tiktoken(e) => Some(e.as_ref()),
            WebpageTextError::SerdeJson(e) => Some(e),
            WebpageTextError::ExtractText(e) => Some(e),
            WebpageTextError::Html(_) => None,
            WebpageTextError::Custom(_) => None, // 自定义错误无底层来源
        }
    }
}

impl From<WebDriverError> for WebpageTextError {
    fn from(e: WebDriverError) -> Self {
        WebpageTextError::WebDriver(e)
    }
}

// 从 reqwest::Error 转换为 WebpageTextError
impl From<reqwest::Error> for WebpageTextError {
    fn from(e: reqwest::Error) -> Self {
        WebpageTextError::Http(e)
    }
}

// 从 pdf_extract::Error 转换为 WebpageTextError
impl From<pdf_extract::Error> for WebpageTextError {
    fn from(e: pdf_extract::Error) -> Self {
        WebpageTextError::PdfExtract(e)
    }
}

// 从 std::io::Error 转换为 WebpageTextError
impl From<std::io::Error> for WebpageTextError {
    fn from(e: std::io::Error) -> Self {
        WebpageTextError::Io(e)
    }
}

// 从 tiktoken_rs::Error 转换为 WebpageTextError
// tiktoken_rs 中使用的是anyhow::Error
impl From<anyhow::Error> for WebpageTextError {
    fn from(e: anyhow::Error) -> Self {
        WebpageTextError::Tiktoken(e)
    }
}

// 从 serde_json::Error 转换为 WebpageTextError
impl From<serde_json::Error> for WebpageTextError {
    fn from(e: serde_json::Error) -> Self {
        WebpageTextError::SerdeJson(e)
    }
}

// 从 output::Error 转换为 WebpageTextError
impl From<OutputError> for WebpageTextError {
    fn from(e: OutputError) -> Self {
        WebpageTextError::ExtractText(e)
    }
}

impl From<String> for WebpageTextError {
    fn from(e: String) -> Self {
        WebpageTextError::Html(e)
    }
}

#[derive(Debug,Clone)]
pub struct WebpageTextUtils {
    driver: Arc<WebDriver>,
}

impl WebpageTextUtils {
    pub fn new(driver: Arc<WebDriver>) -> Self {
        Self { driver }
    }

    pub async fn get_all_webpage_text(&self, n_lines: Option<usize>) -> Result<String,WebpageTextError> {
        let n_lines = n_lines.unwrap_or(50);

        // 查找body元素：WebDriverError 自动转换为 WebpageTextError
        let body_element = self.driver.find(By::Tag("body")).await?;
        
        // 获取文本：错误自动转换
        let text_in_viewport = body_element.text().await?;

        // 按行处理文本
        let lines: Vec<&str> = text_in_viewport.split('\n').collect();
        let end_idx = std::cmp::min(n_lines, lines.len());
        let limited_lines = &lines[0..end_idx];

        // 过滤空行
        let non_empty_lines: Vec<&str> = limited_lines
            .iter()
            .filter(|&&line| !line.trim().is_empty())
            .cloned()
            .collect();

        Ok(non_empty_lines.join("\n"))
    }

    async fn is_pdf_page(&self) -> Result<bool,WebpageTextError> {
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

        let is_pdf = value.as_bool().ok_or_else(|| {
            WebpageTextError::Custom(format!(
                "判断PDF页面失败：JavaScript返回非布尔值，实际值为 {:?}", 
                value
            ))
        })?;
        Ok(is_pdf)
    }

    // 网页处理工具：网页（PDF界面）转化为Markdown
    pub async fn get_page_markdown(&self, max_tokens: i32) -> Result<String,WebpageTextError> {
        
        self.driver
            .set_implicit_wait_timeout(Duration::from_secs(10))
            .await?;
        
        if self.is_pdf_page().await? {
            return self.extract_pdf_content().await;
        }

        let html = self.get_clean_html().await?;

        let markdown = self.convert_html_markdown(&html);

        let final_markdown = if max_tokens > 0 {
            self.limit_token(&markdown, max_tokens as usize)
        } else {
            Ok(markdown)
        };

        final_markdown

    }

    async fn get_clean_html(&self) -> Result<String,WebpageTextError> {
        let script = r#"
            // 移除 script、style、注释和广告相关内容
            document.querySelectorAll('script, style, noscript, iframe, [class*="ad"], [id*="ad"]').forEach(el => el.remove());
            // 移除 HTML 注释
            let walker = document.createTreeWalker(document.body, NodeFilter.SHOW_COMMENT);
            while (walker.nextNode()) {
                walker.currentNode.remove();
            }
            // 返回清理后的 HTML
            return document.documentElement.outerHTML;
        "#;

        let result = self
            .driver
            .execute(script, vec![])
            .await
            .map_err(WebpageTextError::WebDriver)?;

        let html = result
            .json()
            .as_str()
            .ok_or_else(|| WebpageTextError::Html("无法解析清理后的 HTML".into()))?
            .to_string();

        Ok(html)
    }

    // Tokenizen 枚举 --> CoreBPE 实例
    fn tokenizer_to_core_bpe(tokenizer: Tokenizer) -> Result<CoreBPE,anyhow::Error> {
        match tokenizer {
            Tokenizer::O200kBase => o200k_base(),    // 对应 O200kBase 编码方案
            Tokenizer::Cl100kBase => cl100k_base(),  // 对应 Cl100kBase 编码方案（GPT-4/3.5 用）
            Tokenizer::P50kBase => p50k_base(),      // 对应 P50kBase 编码方案（text-davinci-003 用）
            Tokenizer::R50kBase => r50k_base(),      // 对应 R50kBase 编码方案（旧文本模型用）
            Tokenizer::P50kEdit => p50k_edit(),      // 对应 P50kEdit 编码方案（编辑模型用）
            _ => Err(anyhow!("当前场景不支持该分词模型")),
        }
    }

    // 限制tokn数量
    fn limit_token(&self, content: &str, max_tokens: usize) -> Result<String, WebpageTextError>{
        if content.is_empty() {
            return Ok(String::new())
        }
        // 根据模型确定编码方案
        let model = "gpt-4-0314";
        let tokenizer_type = get_tokenizer(model).unwrap();

        // Tokenizer 枚举转为真正的 CoreBPE 实例
        let bpe = Self::tokenizer_to_core_bpe(tokenizer_type)?;

        let tokens = bpe.encode_with_special_tokens(content);
        let limited_tokens = if tokens.len() > max_tokens {
            tokens.into_iter().take(max_tokens).collect::<Vec<usize>>()
        } else {
            tokens
        };

        // 步骤5：解码 Token 为文本（使用 CoreBPE 源码中的 decode 方法，自动验证 UTF-8）
        let limited_content = bpe
            .decode(limited_tokens)
            .map_err(|e| WebpageTextError::Tiktoken(anyhow!("Token解码失败：{}", e)))?;

        Ok(limited_content)
    }

    // 提取HTML转化为Markdown
    fn convert_html_markdown(&self,html: &str) -> String {
        
        let mut markdown = parse_html(html);
    
        markdown = self.clean_markdown(&markdown);

        markdown
    }

    fn clean_markdown(&self, markdown: &str) -> String {

        let re = regex::Regex::new(r"\n\s*\n\s*\n+").unwrap();

        let cleaned = re.replace_all(markdown, "\n\n").trim().to_string();
        
        // 过滤完全空的行，但保留代码块相关内容
        cleaned
            .lines()
            .filter(|line| !line.trim().is_empty() || line.contains("```"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    // 从pdf 提取文本（高级实现，更好的错误处理）
    async fn extract_pdf_content(&self) -> Result<String,WebpageTextError> {
        let url = self.driver.current_url().await?;
        

        let browser_text = self.extract_pdf_browser().await?;
        if !browser_text.is_empty() && browser_text.len() > 100 {
            return Ok(browser_text)
        }

        // 下载PDF文件
        let client = Client::new();
        let response  = client.get(url.to_string())
            .send()
            .await?;

        let pdf_data = response.bytes()
            .await?;

        let mut temp_file = NamedTempFile::new()?;
        
        // 写入PDF数据到临时文件
        temp_file.write_all(&pdf_data)?;
        
        // 使用pdf_extract库提取文本
        let text_content = extract_text(temp_file.path())?;

        // 检查提取结果是否有效
        if text_content.is_empty() {
            return Err(WebpageTextError::Custom(
                "PDF文本提取失败：提取结果为空字符串（可能是加密PDF或无效格式）".to_string()
            ));
        }
        
        Ok(text_content)

    }

    // 从 pdf 提取文本（底层封装）
    async fn extract_pdf_browser(&self) -> Result<String,WebpageTextError> {
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
            return textElements.map(el => el.textContent).join('\n');
        "#;

        // 获取 ScriptRet
        let script_ret = self.driver
            .execute(script, Vec::new())
            .await?;

        // 从 ScriptRet 中提取字符串值
        match script_ret.json() {
            Value::String(text) => Ok(text.trim().to_string()),
            Value::Null => Ok(String::new()),
            _ => {
                eprintln!("警告：PDF提取返回了非字符串值: {:?}", script_ret.json());
                Ok(String::new())
            }
        }
    }

}