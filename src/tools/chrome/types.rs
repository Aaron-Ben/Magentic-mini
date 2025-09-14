use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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
    pub aria_name: String,
    pub v_scrollable: bool,
    pub rects: Vec<DOMRectangle>,
}

#[derive(Clone)]
pub struct TabInfo {
    // 适用于多线程的环境中，Tab 对象需要在多个组件间共享
    pub tab: Arc<Tab>,
    // 对于headless_chrome来说，并不直接提供一个有序的列表来访问标签页。相反，通常使用标签页的 ID 来进行操作
    // 这里仿写一个index
    pub index: usize,
    pub title: String,
    pub url: String,
    pub is_active: bool,
    pub is_controlled: bool,
}

impl TabInfo{
    pub fn new(tab: Arc<Tab>, index: usize, title: String, url: String, is_active: bool) -> Self {
        Self {
            tab,
            index,
            title,
            url,
            is_active,
            is_controlled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageMetadata {
    #[serde(rename = "jsonID")]
    pub json_id: Option<serde_json::Value>,
    pub microdata: Option<serde_json::Value>,
    pub meta_tags: Option<HashMap<String, String>>,
}