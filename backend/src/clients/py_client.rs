use std::time::Duration;

use anyhow::{Result, anyhow};

pub struct PyClient {
    base_url: String,
    client: reqwest::Client,
}

impl PyClient {
    pub fn new(base_url: &str) -> Self {
        let base_url = base_url.replace("localhost", "127.0.0.1");
        // 禁用系统代理以避免在沙盒环境中访问系统配置失败
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build reqwest client");
        Self {
            base_url,
            client,
        }
    }

    pub async fn load_pdf(&self, file_path: &str) -> Result<Vec<String>> {
        let url = format!("{}/pdf/load", self.base_url);
        let response = self.client.post(&url)
            .json(&serde_json::json!({
                "file_path": file_path,
            }))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send POST request to {}: {}", url, e))?;

        if !response.status().is_success() {
            return Err(anyhow!("HTTP error: {}", response.status()));
        }

        let body = response.text().await
            .map_err(|e| anyhow!("Failed to read response body from {}: {}", url, e))?;

        if body.trim().is_empty() {
            return Err(anyhow!("Empty response body from {}", url));
        }

        let pages: Vec<String> = serde_json::from_str(&body)
            .map_err(|e| anyhow!("Failed to parse JSON response from {}: {}", url, e))?;
        Ok(pages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_pdf() {
        let client = PyClient::new("http://localhost:8000");
        let pages = client.load_pdf("test.pdf").await.unwrap();
        println!("Pages: {:?}", pages);
    }
}