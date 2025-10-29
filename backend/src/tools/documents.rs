use std::{collections::HashMap};

use anyhow::Result;

use crate::{clients::py_client::PyClient};

#[derive(Debug, Clone)]
pub struct Document {
    pub content: String,
    pub metadata: HashMap<String, String>,
}

pub struct DocumentProcessor {
    py_client: PyClient,
}

impl DocumentProcessor {
    pub fn new(py_client: PyClient) -> Self {
        Self { py_client }
    }

    pub async fn process_document(
        &self,
        file_path: &str
    ) -> Result<()> {
        let pages = self.py_client.load_pdf(file_path).await?;
        println!("pages: {:?}", pages);

        let mut documents = Vec::new();
        for (i, content) in pages.iter().enumerate() {
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), file_path.to_string());
            metadata.insert("page".to_string(), (i + 1).to_string());
            documents.push(Document {
                content: content.clone(),
                metadata,
            });
        }

        // 分割文档
        let chucks = self.split_document(&documents).await?;

        // 添加到向量数据库中

        Ok(())
    }

    #[allow(dead_code)]
    async fn split_document(&self, documents: &[Document]) -> Result<Vec<Document>> {
        let merged_doc = self.merge_documents(documents);
        let split_points = self.get_split_points(merged_doc.content.as_str(), &["\n\n"]).await?;
        let _text_chuck = self.split_by_points(merged_doc.content.as_str(), &split_points).await?;
        unimplemented!();
    }

    fn merge_documents(&self, documents: &[Document]) -> Document {
        let content = documents
            .iter()
            .map(|d| d.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        let mut metadata = HashMap::new();
        for doc in documents {
            metadata.extend(doc.metadata.clone());
        }
        Document {
            content,
            metadata,
        }
    }

    async fn get_split_points(&self, text: &str, split_points: &[&str]) -> Result<Vec<String>> {
        
        unimplemented!();
    }

    async fn split_by_points(&self, text: &str, split_points: &[&str]) -> Result<Vec<String>> {
        let mut chunks = Vec::new();
        let mut current_pos = 0;

        for &point in split_points {
            if let Some(pos) = text[current_pos..].find(point) {
                let abs_pos = current_pos + pos;

                if abs_pos > current_pos {
                    let chunk = text[current_pos..abs_pos].trim();
                    if !chunk.is_empty() {
                        chunks.push(chunk.to_string());
                    }
                }

                current_pos = abs_pos;
            }
        }
        if current_pos < text.len() {
            let chuck = text[current_pos..].trim();
            if !chuck.is_empty() {
                chunks.push(chuck.to_string());
            }
        }

        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_py_client_load_pdf() {       
        let py_client = PyClient::new("http://localhost:8001");
        let test_pdf_path = "/Users/xuenai/Downloads/djcftlqw.pdf";
        
        let pages = py_client.load_pdf(test_pdf_path).await.unwrap(); // 如果出错，测试会 panic 并标记为 failed
        println!("✅ 成功加载 PDF，共 {} 页", pages.len());
        for (i, page) in pages.iter().enumerate() {
            println!("第 {} 页内容长度: {} 字符", i + 1, page.len());
        }
    }
}
