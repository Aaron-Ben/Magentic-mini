use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InteractiveRegion {
    pub tag_name: String,
    pub role: String,
    pub aria_name: String,
    pub v_scrollable: bool,
    pub rects: Vec<DOMRectangle>,
}
