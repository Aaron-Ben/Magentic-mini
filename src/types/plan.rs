use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Plan {
    pub task: Option<String>,
    pub steps: Vec<PlanStep>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlanStep {
    pub title: String,
    pub details: String,
    pub agent_name: String,
}