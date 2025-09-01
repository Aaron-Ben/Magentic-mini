use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DOMRectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VisualViewport {
    pub height: f64,
    pub width: f64,
    #[serde(rename = "offsetLeft")]
    pub offset_left: f64,
    #[serde(rename = "offsetTop")]
    pub offset_top: f64,
    #[serde(rename = "pageLeft")]
    pub page_left: f64,
    #[serde(rename = "pageTop")]
    pub page_top: f64,
    pub scale: f64,
    #[serde(rename = "clientWidth")]
    pub client_width: f64,
    #[serde(rename = "clientHeight")]
    pub client_height: f64,
    #[serde(rename = "scrollWidth")]
    pub scroll_width: f64,
    #[serde(rename = "scrollHeight")]
    pub scroll_height: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InteractiveRegion {
    pub tag_name: String,
    pub role: String,
    #[serde(rename = "aria-name")]
    pub aria_name: String,
    #[serde(rename = "v-scrollable")]
    pub v_scrollable: bool,
    pub rects: Vec<DOMRectangle>,
}

// 辅助函数，用于从 JSON 数据创建类型
impl DOMRectangle {
    pub fn from_json(data: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(data.clone())
    }
}

impl VisualViewport {
    pub fn from_json(data: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(data.clone())
    }
}

impl InteractiveRegion {
    pub fn from_json(data: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(data.clone())
    }
}