use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskRequest {
  pub session_id: String,
  pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
  pub task_id: String,
  pub session_id: String,
  pub content: String,
  pub state: TaskState,
  pub steps: Vec<TaskStep>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TaskState {
  Running,
  Completed,
  Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskStep {
  pub name: String,
  pub result: String,
  pub timestamp: DateTime<Utc>,
}

impl Task {
  pub fn new(session_id: String, content: String) -> Self {
    Self {
      task_id: format!("task_{}", Uuid::new_v4()),
      session_id,
      content,
      state: TaskState::Running,
      steps: Vec::new(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn create_task_defaults() {
		let t = Task::new("s1".into(), "do something".into());
		assert!(t.task_id.starts_with("task_"));
		assert_eq!(t.session_id, "s1");
		assert_eq!(matches!(t.state, TaskState::Running), true);
		assert!(t.created_at <= Utc::now());
		assert!(t.updated_at <= Utc::now());
	}
}