use axum::{routing::{get, post}, Router};

use crate::api::handlers::{create_task, get_task, task_stream, AppState};

pub fn build_router() -> Router<AppState> {
	Router::new()
		.route("/tasks", post(create_task))
		.route("/tasks/:task_id", get(get_task))
		.route("/tasks/:task_id/stream", get(task_stream))
		.route("/health", get(|| async { "ok" }))
}


#[cfg(test)]
mod tests {
	use super::*;
	use axum::http::{Request, StatusCode};
	use axum::body::Body;
	use tower::ServiceExt; // for `oneshot`
	use crate::api::handlers::AppState;
	use crate::manager::task_manager::TaskManager;

	#[tokio::test]
	async fn health_ok() {
		let state = AppState { task_manager: TaskManager::new("http://localhost:4444".into()) };
		let app = build_router().with_state(state);
		let response = app
			.oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
			.await
			.unwrap();
		assert_eq!(response.status(), StatusCode::OK);
	}
}
