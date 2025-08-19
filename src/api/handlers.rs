use std::sync::Arc;

use axum::{extract::{Path, State}, Json};
use axum::http::StatusCode;
use axum::response::Result as AxumResult;
use axum::response::sse::{Event, Sse};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use futures_core::Stream;

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

pub async fn task_stream(
	State(state): State<AppState>,
	Path(task_id): Path<String>,
) -> AxumResult<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>> {
	if let Some(rx) = state.task_manager.subscribe(&task_id).await {
		let stream = BroadcastStream::new(rx).map(|msg| {
			match msg {
				Ok(text) => Ok(Event::default().data(text)),
				Err(_) => Ok(Event::default().comment("lagging")),
			}
		});
		Ok(Sse::new(stream))
	} else {
		Err(StatusCode::NOT_FOUND.into())
	}
}

