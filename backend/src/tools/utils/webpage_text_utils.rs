use std::fmt::{Debug};
use std::io::Write;
use anyhow::{anyhow, Context, Result};
use pdf_extract::extract_text;
use std::sync::Arc;
use tiktoken_rs::{
    CoreBPE,
    tokenizer::{get_tokenizer, Tokenizer}, 
    // 导入库内置的编码方案创建函数（对应 Tokenizer 枚举的每个变体）
    cl100k_base, o200k_base, p50k_base, r50k_base, p50k_edit,
};
use reqwest::Client;
use tempfile::NamedTempFile;
use thirtyfour::prelude::*;
use serde_json::Value;
use tokio::time::Duration;
use crate::tools::utils::markitdown_bridge::convert_html_to_markdown_with_markitdown;

#[derive(Debug,Clone)]
pub struct WebpageTextUtils {
    driver: Arc<WebDriver>,
}

impl WebpageTextUtils {
    pub fn new(driver: Arc<WebDriver>) -> Self {
        Self { driver }
    }

    pub async fn get_all_webpage_text(&self, n_lines: Option<usize>) -> Result<String> {
        let n_lines = n_lines.unwrap_or(50);

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

    async fn is_pdf_page(&self) -> Result<bool> {
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

        let is_pdf = value.as_bool()
        .ok_or_else(|| anyhow!("JavaScript returned non-boolean value for is_pdf_page"))?;
        Ok(is_pdf)
    }

    // 网页处理工具：网页（PDF界面）转化为Markdown
    pub async fn get_page_markdown(&self, max_tokens: i32) -> Result<String> {
        self.driver
            .set_implicit_wait_timeout(Duration::from_secs(10))
            .await?;

        if self.is_pdf_page().await? {
            return self.extract_pdf_content().await;
        }

        let html = self.get_clean_html().await?;

        let markdown = convert_html_to_markdown_with_markitdown(&html)
        .await
        .map_err(|e| anyhow!("markitdown 转换失败: {}", e))?;

        if max_tokens > 0 {
            self.limit_token(&markdown, max_tokens as usize)
        } else {
            Ok(markdown)
        }
    }

    async fn get_clean_html(&self) -> Result<String> {
        let script = r#"
            // 创建文档副本，避免修改原始DOM
            const cleanDoc = document.cloneNode(true);
            
            // 只移除脚本和广告，保留样式
            const elementsToRemove = [
                'script', 'noscript', 'iframe',
                '[class*="ad"]', '[id*="ad"]', '[class*="advertisement"]',
                '[class*="banner"]', '[class*="popup"]', '[class*="modal"]',
                '[class*="cookie"]', '[class*="consent"]'
            ];
            
            elementsToRemove.forEach(selector => {
                cleanDoc.querySelectorAll(selector).forEach(el => el.remove());
            });
            
            // 移除空的div和span
            cleanDoc.querySelectorAll('div, span').forEach(el => {
                if (el.textContent.trim() === '' && el.children.length === 0) {
                    el.remove();
                }
            });
            
            // 移除HTML注释
            const walker = cleanDoc.createTreeWalker(
                cleanDoc.body, 
                NodeFilter.SHOW_COMMENT, 
                null, 
                false
            );
            const commentsToRemove = [];
            let node;
            while (node = walker.nextNode()) {
                commentsToRemove.push(node);
            }
            commentsToRemove.forEach(comment => comment.remove());
            
            // 返回清理后的HTML
            return cleanDoc.documentElement.outerHTML;
        "#;
    
        let result = self
            .driver
            .execute(script, vec![])
            .await
            .context("Failed to execute script to get clean HTML")?;
    
        let html = result
            .json()
            .as_str()
            .ok_or_else(|| anyhow!("Failed to get HTML from script result"))?
            .to_string();
    
        Ok(html)
    }
    
    // Tokenizen 枚举 --> CoreBPE 实例
    fn tokenizer_to_core_bpe(tokenizer: Tokenizer) -> Result<CoreBPE> {
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
    fn limit_token(&self, content: &str, max_tokens: usize) -> Result<String>{
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
            .map_err(|e| anyhow!("Token解码失败：{}", e))?;

        Ok(limited_content)
    }

    // 从pdf 提取文本（高级实现，更好的错误处理）
    async fn extract_pdf_content(&self) -> Result<String> {
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
            return Err(anyhow!("PDF文本提取失败：提取结果为空字符串（可能是加密PDF或无效格式）"));
        }
        
        Ok(text_content)

    }

    // 从 pdf 提取文本（底层封装）
    async fn extract_pdf_browser(&self) -> Result<String> {
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