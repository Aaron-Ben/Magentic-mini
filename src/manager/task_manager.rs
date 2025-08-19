use std::{collections::HashMap, sync::Arc};

use tokio::sync::{RwLock, broadcast};

use crate::{
	browser::client::BrowserClient,
	types::task::{Task, TaskRequest, TaskState, TaskStep},
};

pub struct TaskManager {
	tasks: RwLock<HashMap<String, Task>>, // task_id -> Task
	webdriver_url: String,
	streams: RwLock<HashMap<String, broadcast::Sender<String>>>,
}

impl TaskManager {
	pub fn new(webdriver_url: String) -> Arc<Self> {
		Arc::new(Self {
			tasks: RwLock::new(HashMap::new()),
			webdriver_url,
			streams: RwLock::new(HashMap::new()),
		})
	}

	pub async fn create_and_spawn(self: &Arc<Self>, req: TaskRequest) -> Task {
		let task = Task::new(req.session_id, req.content);
		let task_id = task.task_id.clone();
		self.tasks.write().await.insert(task_id.clone(), task.clone());

		let this = Arc::clone(self);
		tokio::spawn(async move {
			this.run_task(task_id).await;
		});

		task
	}

	pub async fn get(&self, task_id: &str) -> Option<Task> {
		self.tasks.read().await.get(task_id).cloned()
	}

	pub async fn subscribe(&self, task_id: &str) -> Option<broadcast::Receiver<String>> {
		let map = self.streams.read().await;
		map.get(task_id).map(|tx| tx.subscribe())
	}

	async fn run_task(self: &Arc<Self>, task_id: String) {
		let (tx, _rx) = broadcast::channel::<String>(100);
		self.streams.write().await.insert(task_id.clone(), tx.clone());

		let browser = match BrowserClient::connect(&self.webdriver_url).await {
			Ok(b) => b,
			Err(err) => {
				self.update_fail(task_id.clone(), format!("connect error: {}", err)).await;
				let _ = tx.send(format!("{{\"type\":\"error\",\"message\":{}}}", serde_json::to_string(&format!("connect error: {}", err)).unwrap()));
				return;
			}
		};

		// Demo: 打开页面 -> 简单等待 -> 记录 URL -> 结束
		if let Err(e) = browser.goto("https://bilibili.com").await {
			self.update_fail(task_id.clone(), format!("goto error: {}", e)).await;
			let _ = tx.send(format!("{{\"type\":\"error\",\"message\":{}}}", serde_json::to_string(&format!("goto error: {}", e)).unwrap()));
			let _ = browser.close().await;
			return;
		}
		browser.wait_for(500).await;
		match browser.current_url().await {
			Ok(url) => {
				self.push_step(task_id.clone(), "open".into(), url.clone()).await;
				let _ = tx.send(format!("{{\"type\":\"step\",\"name\":\"open\",\"result\":{}}}", serde_json::to_string(&url).unwrap()));
			}
			Err(e) => {
				self.update_fail(task_id.clone(), format!("get url error: {}", e)).await;
				let _ = tx.send(format!("{{\"type\":\"error\",\"message\":{}}}", serde_json::to_string(&format!("get url error: {}", e)).unwrap()));
				let _ = browser.close().await;
				return;
			}
		}

		let _ = browser.close().await;
		self.update_complete(task_id.clone()).await;
		let _ = tx.send("{\"type\":\"state\",\"value\":\"Completed\"}".to_string());
	}

	async fn push_step(&self, task_id: String, name: String, result: String) {
		let mut guard = self.tasks.write().await;
		if let Some(task) = guard.get_mut(&task_id) {
			task.steps.push(TaskStep {
				name,
				result,
				timestamp: chrono::Utc::now(),
			});
			task.updated_at = chrono::Utc::now();
		}
	}

	async fn update_complete(&self, task_id: String) {
		let mut guard = self.tasks.write().await;
		if let Some(task) = guard.get_mut(&task_id) {
			task.state = TaskState::Completed;
			task.updated_at = chrono::Utc::now();
		}
	}

	async fn update_fail(&self, task_id: String, err: String) {
		let mut guard = self.tasks.write().await;
		if let Some(task) = guard.get_mut(&task_id) {
			task.state = TaskState::Failed;
			task.steps.push(TaskStep {
				name: "error".into(),
				result: err,
				timestamp: chrono::Utc::now(),
			});
			task.updated_at = chrono::Utc::now();
		}
	}
}

