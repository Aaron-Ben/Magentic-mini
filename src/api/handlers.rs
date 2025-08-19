use std::sync::Arc;

use axum::{extract::{Path, State}, Json};
use axum::http::StatusCode;
use axum::response::Result as AxumResult;

use crate::{
	manager::task_manager::TaskManager,
	types::task::{Task, TaskRequest},
};

#[derive(Clone)]
pub struct AppState {
	pub task_manager: Arc<TaskManager>,
}

pub async fn create_task(
	State(state): State<AppState>,
	Json(payload): Json<TaskRequest>,
) -> Json<Task> {
	let task = state.task_manager.create_and_spawn(payload).await;
	Json(task)
}

pub async fn get_task(
	State(state): State<AppState>,
	Path(task_id): Path<String>,
) -> AxumResult<Json<Task>> {
	match state.task_manager.get(&task_id).await {
		Some(task) => Ok(Json(task)),
		None => Err(StatusCode::NOT_FOUND.into()),
	}
}

