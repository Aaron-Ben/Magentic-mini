use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
	#[error("Browser error: {0}")]
	Browser(String),

	#[error("Task not found: {0}")]
	TaskNotFound(String),

	#[error("Internal error: {0}")]
	Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

