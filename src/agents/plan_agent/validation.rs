use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum PlanAgentError {
    #[error("LLM request failed: {0}")]
    LlmError(String),
    
    #[error("JSON parsing failed: {0}")]
    JsonParseError(#[from] serde_json::Error),
    
    #[error("Validation failed: {0}")]
    ValidationError(String),
    
    #[error("Plan generation failed: {0}")]
    PlanGenerationError(String),
}

// 验证计划 JSON 响应
pub fn validate_plan_json(
    json_response: &Value,
    sentinel_tasks_enabled: bool,
) -> bool {
    if !json_response.is_object() {
        return false;
    }

    let obj = json_response.as_object().unwrap();
    
    // 检查必需的字段
    let required_keys = ["task", "steps", "needs_plan", "response", "plan_summary"];
    for key in required_keys.iter() {
        if !obj.contains_key(*key) {
            return false;
        }
    }

    // 验证 steps 数组
    if let Some(steps) = json_response["steps"].as_array() {
        for step in steps {
            if !validate_plan_step_json(step, sentinel_tasks_enabled) {
                return false;
            }
        }
    } else {
        return false;
    }
    
    true
}

fn validate_plan_step_json(step: &Value, sentinel_enabled: bool) -> bool {
    // 验证PlanStep的必需字段
    if !step.get("title").and_then(|v| v.as_str()).is_some() {
        return false;
    }
    if !step.get("details").and_then(|v| v.as_str()).is_some() {
        return false;
    }
    if !step.get("agent_name").and_then(|v| v.as_str()).is_some() {
        return false;
    }

    // 如果启用了sentinel任务，检查sentinel相关字段
    if sentinel_enabled && step.get("step_type").is_some() {
        let step_type = step["step_type"].as_str().unwrap_or("");
        if step_type == "SentinelPlanStep" {
            if !step.get("condition").is_some() {
                return false;
            }
            if !step.get("sleep_duration").is_some() {
                return false;
            }
        }
    }

    true
}
