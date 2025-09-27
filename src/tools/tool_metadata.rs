use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

// 全局元数据注册表
lazy_static::lazy_static! {
    static ref TOOL_METADATA_REGISTRY: RwLock<HashMap<String, ToolMetadata>> = 
        RwLock::new(HashMap::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalLevel {
    #[serde(rename = "always")]
    Always,
    #[serde(rename = "maybe")]
    Maybe,
    #[serde(rename = "never")]
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    #[serde(rename = "requires_approval")] // 更准确的字段名
    pub approval: ApprovalLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: ParametersSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametersSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: serde_json::Value,
    pub required: Vec<String>,
}

use serde_json::Value;

/// 工具定义格式（类似 OpenAI Function Calling）
#[derive(Deserialize)]
struct ToolDef {
    #[serde(rename = "function")]
    function: FunctionDef,
    metadata: Option<ToolMetadata>,
}

#[derive(Deserialize)]
struct FunctionDef {
    name: String,
    description: String,
    parameters: ParametersDef,
}

#[derive(Deserialize)]
struct ParametersDef {
    #[serde(rename = "type")]
    schema_type: String,
    properties: Value,
    required: Vec<String>,
}

pub fn load_tool(tooldef_json: &str) -> Result<ToolSchema, Box<dyn std::error::Error>> {
    let tooldef: ToolDef = serde_json::from_str(tooldef_json)?;
    
    // 注册元数据
    if let Some(metadata) = tooldef.metadata {
        TOOL_METADATA_REGISTRY
            .write()
            .unwrap()
            .insert(tooldef.function.name.clone(), metadata);
    }

    Ok(ToolSchema {
        name: tooldef.function.name,
        description: tooldef.function.description,
        parameters: ParametersSchema {
            schema_type: tooldef.function.parameters.schema_type,
            properties: tooldef.function.parameters.properties,
            required: tooldef.function.parameters.required,
        },
    })
}

pub fn get_tool_metadata(tool_name: &str) -> Option<ToolMetadata> {
    TOOL_METADATA_REGISTRY
        .read()
        .unwrap()
        .get(tool_name)
        .cloned()
}

pub fn make_approval_prompt(
    guarded_examples: &[&str],
    unguarded_examples: &[&str],
    category: Option<&str>,
) -> String {
    let category = category.unwrap_or("actions that require approval");
    format!(
        "Is this action something that would require human approval before being done? \
         Example: {}; but {} are not {}.",
        guarded_examples.join(", "),
        unguarded_examples.join(", "),
        category
    )
}
