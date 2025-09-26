use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum GetValueError {
    #[error("键 '{0}' 未找到")]
    KeyNotFound(String),
    #[error("键 '{0}' 的类型不匹配，期望 {1}，实际 {2}")]
    TypeMismatch(String, String, String),
    #[error("键 '{0}' 包含无效数字")]
    InValidNumber(String),
}

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("解析JSON解析失败: {0}")]
    JsonLdParseError(String),
    #[error("类型不匹配：{0}")]
    TypeMismatch(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rect {
    pub bottom: f64,
    pub height: f64,
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub width: f64,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct VisualViewport {
    pub height: f64,
    pub width: f64,
    pub offset_left: f64,
    pub offset_top: f64,
    pub page_left: f64,
    pub page_top: f64,
    pub scale: f64,
    pub client_width: f64,
    pub client_height: f64,
    pub scroll_width: f64,
    pub scroll_height: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InteractiveRegion {
    pub tag_name: String,
    pub role: String,
    #[serde(rename = "aria-name")]
    pub aria_name: String,
    #[serde(rename = "v-scrollable")]
    pub v_scrollable: bool,
    pub rects: Vec<Rect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub index: usize,
    pub title: String,
    pub url: String,
    pub is_active: bool,
    pub is_controlled: bool,
}

impl TabInfo{
    pub fn new(index: usize, title: String, url: String, is_active: bool) -> Self {
        Self {
            index,
            title,
            url,
            is_active,
            is_controlled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaTags {
    pub charset: Option<String>,
    #[serde(rename = "httpEquiv")]
    pub http_equiv: Option<HashMap<String, String>>,
    pub name: Option<HashMap<String, String>>,
    pub property: Option<HashMap<String, String>>,
    pub other: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageMetadata {

    pub domain: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,

    // JSON-LD 数据数组，每个元素是已解析的 JSON 对象（非字符串）
    #[serde(rename = "jsonld")]
    pub json_ld: Option<Vec<Value>>,
    // HTML <meta> 标签键值对，如 {"description": "...", "og:title": "..."}
    pub meta_tags: Option<MetaTags>,
    // HTML5 Microdata 项数组，每个元素是一个对象（可能嵌套）
    pub microdata: Option<Vec<Value>>,              // 这样能够处理简单的字符串值，嵌套对象，数组，以及混合类型
}


/// 从JSON值中获取数字字段
fn get_number(value: &HashMap<String, Value>, key: &str) -> Result<f64, GetValueError> {
    let val = value.get(key)
        .ok_or_else(|| GetValueError::KeyNotFound(key.to_string()))?;

    match val {
        Value::Number(num) => {
            num.as_f64()
                .ok_or_else(||GetValueError::InValidNumber(key.to_string()))
        }
        Value::String(s) => {
            s.parse::<f64>()
                .map_err(|_| GetValueError::TypeMismatch(
                    key.to_string(),
                    "数字".to_string(),
                    format!("字符串 '{}'",s),
                ))
        }
        _ => Err(GetValueError::TypeMismatch(
            key.to_string(),
            "数字".to_string(),
            format!("{:?}",val),
        ))
    }
}

impl VisualViewport {
    /// Json格式转化为VisualViewport实例
    pub fn visualviewport_from_dict(viewport: &HashMap<String, Value>) -> Result<VisualViewport, GetValueError> {
        Ok(Self {
            height: get_number(viewport, "height")?,
            width: get_number(viewport, "width")?,
            offset_left: get_number(viewport, "offsetLeft")?,
            offset_top: get_number(viewport, "offsetTop")?,
            page_left: get_number(viewport, "pageLeft")?,
            page_top: get_number(viewport, "pageTop")?,
            scale: get_number(viewport, "scale")?,
            client_width: get_number(viewport, "clientWidth")?,
            client_height: get_number(viewport, "clientHeight")?,
            scroll_width: get_number(viewport, "scrollWidth")?,
            scroll_height: get_number(viewport, "scrollHeight")?,
        })
    }
}

// 一些PageMetadata的辅助方法(可能用不上)
impl PageMetadata {
    pub fn has_metadata(&self) -> bool {
        self.meta_tags.is_some() ||
        self.json_ld.is_some() ||
        self.microdata.is_some()
    }

    /// 获取所有可用的元数据类型
    pub fn available_data_types(&self) -> Vec<&str> {
        let mut types = Vec::new();
        
        if self.meta_tags.is_some() {
            types.push("meta_tags");
        }
        if self.json_ld.is_some() {
            types.push("jsonld");
        }
        if self.microdata.is_some() {
            types.push("microdata");
        }
        
        types
    }

    /// 智能获取页面描述（按优先级）
    pub fn get_description(&self) -> Option<String> {
        // 1. 尝试从meta_tags获取
        if let Some(meta_tags) = &self.meta_tags {
            if let Some(name_tags) = &meta_tags.name {
                if let Some(desc) = name_tags.get("description") {
                    return Some(desc.clone());
                }
            }
        }
        
        // 2. 尝试从JSON-LD获取
        if let Some(jsonld) = &self.json_ld {
            for item in jsonld {
                if let Value::Object(obj) = item {
                    if let Some(Value::String(desc)) = obj.get("description") {
                        return Some(desc.clone());
                    }
                }
            }
        }
         // 3. 尝试从微数据获取
        if let Some(microdata) = &self.microdata {
            for item in microdata {
                if let Value::Object(obj) = item {
                    if let Some(Value::String(desc)) = obj.get("description") {
                        return Some(desc.clone());
                    }
                }
            }
        }
        
        None
    }
}
