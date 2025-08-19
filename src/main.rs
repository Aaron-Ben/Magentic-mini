mod api;
mod browser;
mod manager;
mod types;

use std::net::SocketAddr;

use api::{handlers::AppState, routes::build_router};
use axum::{Router};
use manager::task_manager::TaskManager;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
	// 日志
	let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
	fmt().with_env_filter(filter).init();

	// 依赖
	let webdriver_url = std::env::var("WEBDRIVER_URL").unwrap_or_else(|_| "http://localhost:4444".to_string());
	let task_manager = TaskManager::new(webdriver_url);
	let state = AppState { task_manager };

	// 路由
	let app: Router = build_router().with_state(state);

	let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
	tracing::info!("listening on http://{}", addr);
	axum::Server::bind(&addr)
		.serve(app.into_make_service())
		.await
		.unwrap();
}