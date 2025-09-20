use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::{collections::HashMap, fmt};

/// 普通计划步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub title: String,
    pub details: String,
    pub agent_name: String,
}

/// 哨兵计划步骤（监控型任务）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelPlanStep {
    pub title: String,
    pub details: String,
    pub agent_name: String,
    pub sleep_duration: u64,  // 秒
    pub condition: Condition, // 条件类型
}

/// 条件枚举：整数（迭代次数）或字符串（条件描述）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Condition {
    Iterations(u32),        // 固定迭代次数
    Expression(String),     // 条件表达式
}

/// 统一的步骤枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Step {
    Normal(PlanStep),
    Sentinel(SentinelPlanStep),
}

/// 主计划结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub task: Option<String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestUsage{
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub source: String,
    pub models_usage: Option<RequestUsage>,
    pub metadata: HashMap<String, String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageTypeItem {
    Chat(ChatMessage),      // 用户与AI的对话历史
    Event(AgentEvent),      // 代理事件，例如任务的开始，状态变更等
}

impl Plan {
    /// 获取步骤数量
    pub fn len(&self) -> usize {
        self.steps.len()
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
    
    /// 获取指定索引的步骤
    pub fn get(&self, index: usize) -> Option<&Step> {
        self.steps.get(index)
    }
    
    /// 创建新的计划
    pub fn new(task: Option<String>) -> Self {
        Self {
            task,
            steps: Vec::new(),
        }
    }
    
    /// 添加普通步骤
    pub fn add_step(&mut self, title: String, details: String, agent_name: String) {
        self.steps.push(Step::Normal(PlanStep {
            title,
            details,
            agent_name,
        }));
    }
    
    /// 添加哨兵步骤
    pub fn add_sentinel_step(
        &mut self, 
        title: String, 
        details: String, 
        agent_name: String,
        sleep_duration: u64,
        condition: Condition
    ) {
        self.steps.push(Step::Sentinel(SentinelPlanStep {
            title,
            details,
            agent_name,
            sleep_duration,
            condition,
        }));
    }
}

// ===== Trait 实现 =====

/// 索引访问（实现类似 Python 的 __getitem__）
impl std::ops::Index<usize> for Plan {
    type Output = Step;
    
    fn index(&self, index: usize) -> &Self::Output {
        &self.steps[index]
    }
}

/// 显示格式化（实现类似 Python 的 __str__）
impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        
        if let Some(ref task) = self.task {
            output.push_str(&format!("Task: {}\n", task));
        }
        
        for (i, step) in self.steps.iter().enumerate() {
            match step {
                Step::Normal(plan_step) => {
                    output.push_str(&format!(
                        "{}. {}: {}\n   {}\n",
                        i, plan_step.agent_name, plan_step.title, plan_step.details
                    ));
                }
                Step::Sentinel(sentinel_step) => {
                    output.push_str(&format!(
                        "{}. {}: {}\n   {}\n",
                        i, sentinel_step.agent_name, sentinel_step.title, sentinel_step.details
                    ));
                    
                    let condition_str = match &sentinel_step.condition {
                        Condition::Iterations(count) => format!("{} iterations", count),
                        Condition::Expression(expr) => expr.clone(),
                    };
                    
                    output.push_str(&format!(
                        "   [Sentinel: every {}s, condition: {}]\n",
                        sentinel_step.sleep_duration, condition_str
                    ));
                }
            }
        }
        
        write!(f, "{}", output)
    }
}

/// 从多种格式构造 Plan（类似 Python 的 from_list_of_dicts_or_str）
impl Plan {
    /// 从 JSON 字符串构造
    pub fn from_json(json_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let plan: Plan = serde_json::from_str(json_str)?;
        Ok(plan)
    }
    
    /// 从字典构造
    pub fn from_dict(dict: Map<String, serde_json::Value>) -> Result<Self, Box<dyn std::error::Error>> {
        let task = dict.get("task")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let steps = dict.get("steps")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|step| Self::parse_step(step).ok()).collect())
            .unwrap_or_default();
            
        Ok(Self { task, steps })
    }
    
    /// 从步骤列表构造
    pub fn from_steps_list(steps_data: Vec<serde_json::Value>) -> Result<Self, Box<dyn std::error::Error>> {
        let steps = steps_data.into_iter()
            .filter_map(|step| Self::parse_step(&step).ok())
            .collect();
            
        Ok(Self { task: None, steps })
    }
    
    /// 统一的构造方法（支持多种输入格式）
    pub fn from_various_formats(input: serde_json::Value) -> Result<Self, Box<dyn std::error::Error>> {
        match input {
            serde_json::Value::String(json_str) => {
                // 如果是字符串，尝试解析为 JSON
                Self::from_json(&json_str)
            }
            serde_json::Value::Object(dict) => {
                if dict.contains_key("steps") {
                    // 如果包含 steps 字段，当作完整 Plan
                    Self::from_dict(dict)
                } else {
                    // 当作单个步骤
                    let steps = vec![Self::parse_step(&serde_json::Value::Object(dict))?];
                    Ok(Self { task: None, steps })
                }
            }
            serde_json::Value::Array(array) => {
                Self::from_steps_list(array)
            }
            _ => Err("Unsupported input format".into())
        }
    }
    
    /// 解析单个步骤
    fn parse_step(step_value: &serde_json::Value) -> Result<Step, Box<dyn std::error::Error>> {
        let step_obj = step_value.as_object()
            .ok_or("Step must be an object")?;
            
        let title = step_obj.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Step")
            .to_string();
            
        let details = step_obj.get("details")
            .and_then(|v| v.as_str())
            .unwrap_or("No details provided")
            .to_string();
            
        let agent_name = step_obj.get("agent_name")
            .and_then(|v| v.as_str())
            .unwrap_or("agent")
            .to_string();
            
        // 检查是否为哨兵步骤
        if let (Some(sleep_duration), Some(condition)) = (
            step_obj.get("sleep_duration").and_then(|v| v.as_u64()),
            step_obj.get("condition")
        ) {
            let condition = if let Some(iterations) = condition.as_u64() {
                Condition::Iterations(iterations as u32)
            } else if let Some(expr) = condition.as_str() {
                Condition::Expression(expr.to_string())
            } else {
                return Err("Invalid condition format".into());
            };
            
            Ok(Step::Sentinel(SentinelPlanStep {
                title,
                details,
                agent_name,
                sleep_duration,
                condition,
            }))
        } else {
            Ok(Step::Normal(PlanStep {
                title,
                details,
                agent_name,
            }))
        }
    }
}

// ===== 迭代器支持 =====

impl IntoIterator for Plan {
    type Item = Step;
    type IntoIter = std::vec::IntoIter<Step>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.steps.into_iter()
    }
}

impl<'a> IntoIterator for &'a Plan {
    type Item = &'a Step;
    type IntoIter = std::slice::Iter<'a, Step>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.steps.iter()
    }
}

// ===== 实用方法 =====

impl Plan {
    /// 获取所有普通步骤
    pub fn normal_steps(&self) -> Vec<&PlanStep> {
        self.steps.iter().filter_map(|step| {
            if let Step::Normal(plan_step) = step {
                Some(plan_step)
            } else {
                None
            }
        }).collect()
    }
    
    /// 获取所有哨兵步骤
    pub fn sentinel_steps(&self) -> Vec<&SentinelPlanStep> {
        self.steps.iter().filter_map(|step| {
            if let Step::Sentinel(sentinel_step) = step {
                Some(sentinel_step)
            } else {
                None
            }
        }).collect()
    }
    
    /// 查找指定代理的步骤
    pub fn steps_for_agent(&self, agent_name: &str) -> Vec<&Step> {
        self.steps.iter().filter(|step| {
            match step {
                Step::Normal(plan_step) => plan_step.agent_name == agent_name,
                Step::Sentinel(sentinel_step) => sentinel_step.agent_name == agent_name,
            }
        }).collect()
    }
}
