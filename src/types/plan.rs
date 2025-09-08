use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::Value;
use anyhow::{Result, anyhow};

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
    pub agent_name: String,  // 改为 agent_name 而不是 agent_type
    pub status: StepStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    WebSurfer,
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

    /// 从 JSON 对象创建 Plan
    pub fn from_json(plan_json: &Value) -> Result<Self> {
        let task = plan_json["task"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing task field"))?
            .to_string();
        
        let mut steps = Vec::new();
        
        if let Some(steps_array) = plan_json["steps"].as_array() {
            for step_json in steps_array {
                steps.push(PlanStep::from_json(step_json)?);
            }
        }
        
        Ok(Self::new(task, steps))
    }

    /// 将 Plan 转换为 JSON 字符串显示给用户
    pub fn to_display_string(&self) -> String {
        serde_json::to_string_pretty(self)
            .unwrap_or_else(|_| format!("Failed to serialize plan: {}", self.task))
    }
}

impl PlanStep {
    pub fn new(title: String, details: String, agent_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            details,
            agent_name,
            status: StepStatus::Pending,
        }
    }

    /// 从 JSON 对象创建 PlanStep
    pub fn from_json(step_json: &Value) -> Result<Self> {
        let title = step_json["title"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing title field"))?
            .to_string();
        
        let details = step_json["details"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing details field"))?
            .to_string();
        
        let agent_name = step_json["agent_name"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing agent_name field"))?
            .to_string();
        
        Ok(Self::new(title, details, agent_name))
    }
}