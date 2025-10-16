use serde::{Deserialize, Serialize};
use serde_json::Value;

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

impl Plan {
    pub fn from_list_of_dicts_or_str(plan_input: impl Into<Value>) -> Option<Self> {
        let mut value = plan_input.into();

        if let Value::String(s) = &value {
            value = serde_json::from_str(s).ok()?;
        }

        if value.is_null() || (value.is_array() && value.as_array()?.is_empty()) {
            return None;
        }

        let (task, steps_value) = match &value {
            Value::Object(map) => {
                let task = map.get("task").and_then(|v| v.as_str()).map(|s| s.to_string());
                let steps_value = map.get("steps").cloned().unwrap_or(Value::Array(vec![]));
                (task, steps_value)
            }
            Value::Array(arr) => {
                (None, Value::Array(arr.clone()))
            }
            _ => {
                return None;
            }
        };

        let steps_array = steps_value.as_array()?;
        let mut steps = Vec::with_capacity(steps_array.len());

        for step_value in steps_array {
            let step_map = step_value.as_object()?;

            let title = step_map.get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled Step")
                .to_string();
            
            let details = step_map.get("details")
                .and_then(|v| v.as_str())
                .unwrap_or("No details provided.")
                .to_string();
            
            let agent_name = step_map.get("agent_name")
                .and_then(|v| v.as_str())
                .unwrap_or("agent")
                .to_string();

            steps.push(PlanStep { title, details, agent_name });
        }
        if !steps.is_empty() {
            Some(Plan { task, steps })
        } else {
            None
        }
    }
}
