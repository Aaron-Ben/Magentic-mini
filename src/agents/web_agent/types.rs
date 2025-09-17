use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::tools::chrome::types::{InteractiveRegion};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct FunctionCall<T> {
    pub id: String,             // 要调用函数的名称
    pub arguments: T,      // 传递给函数的参数，可能包含多种类型，序列化为JSON字符串
    pub name: Option<String>,   // 函数调用的唯一标识符
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ParametersSchema {
    pub types: String,
    pub properties: HashMap<String, Value>,     // 定义每个具体参数的模式, 参数名称, 参数的详细定义（类型、描述、默认值等）
    pub required: Option<Vec<String>>,
    pub additional_properties: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ToolSchema {
    pub parameters: Option<ParametersSchema>,
    pub name: String,
    pub description: Option<String>,
    pub strict: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestUsage {   
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

#[derive(Clone, Debug)]
pub enum LLMOutput<T> {
    Text(String),
    FunctionCalls(Vec<FunctionCall<T>>)
}

#[derive(Debug)]
pub struct LLMResponse<T> {
    pub output: LLMOutput<T>,
    pub interactive: HashMap<String, InteractiveRegion>,
    pub tools: Vec<ToolSchema>,
    pub element_id: HashMap<String, String>,
    pub need_execute_tool: bool,
}