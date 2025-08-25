use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: Uuid,
    pub task: String,
    pub steps: Vec<PlanStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: Uuid,
    pub title: String,
    pub details: String,
    pub agent_type: AgentType,
    pub status: StepStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    WebSurfer,
    Coder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl Plan {
    pub fn new(task: String, steps: Vec<PlanStep>) -> Self {
        Self {
            id: Uuid::new_v4(),
            task,
            steps,
        }
    }
}

impl PlanStep {
    pub fn new(title: String, details: String, agent_type: AgentType) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            details,
            agent_type,
            status: StepStatus::Pending,
        }
    }
}