use std::time::Duration;

use fantoccini::{Client, ClientBuilder, Locator};

use crate::types::error::{AppError, AppResult};

pub struct BrowserClient {
	inner: Client,
}

impl BrowserClient {
	pub async fn connect(webdriver_url: &str) -> AppResult<Self> {
		let client = ClientBuilder::native()
			.connect(webdriver_url)
			.await
			.map_err(|e| AppError::Browser(e.to_string()))?;
		Ok(Self { inner: client })
	}

	pub async fn goto(&self, url: &str) -> AppResult<()> {
		self.inner
			.goto(url)
			.await
			.map_err(|e| AppError::Browser(e.to_string()))
	}

	pub async fn click_css(&self, selector: &str) -> AppResult<()> {
		self.inner
			.find(Locator::Css(selector))
			.await
			.map_err(|e| AppError::Browser(e.to_string()))?
			.click()
			.await
			.map_err(|e| AppError::Browser(e.to_string()))
	}

	pub async fn type_css(&self, selector: &str, text: &str) -> AppResult<()> {
		let elem = self
			.inner
			.find(Locator::Css(selector))
			.await
			.map_err(|e| AppError::Browser(e.to_string()))?;
		elem.send_keys(text)
			.await
			.map_err(|e| AppError::Browser(e.to_string()))
	}

	pub async fn wait_for(&self, millis: u64) {
		tokio::time::sleep(Duration::from_millis(millis)).await;
	}

	pub async fn current_url(&self) -> AppResult<String> {
		let url = self
			.inner
			.current_url()
			.await
			.map_err(|e| AppError::Browser(e.to_string()))?;
		Ok(url.as_ref().to_string())
	}

	pub async fn close(self) -> AppResult<()> {
		self.inner.close().await.map_err(|e| AppError::Browser(e.to_string()))
	}
}
