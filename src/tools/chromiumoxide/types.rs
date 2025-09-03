use std::collections::HashMap;
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

lazy_static::lazy_static! {
    pub static ref CUA_KEY_TO_CHROMIUM_KEY: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("/", "Divide");
        map.insert("\\", "Backslash");
        map.insert("alt", "Alt");
        map.insert("arrowdown", "ArrowDown");
        map.insert("arrowleft", "ArrowLeft");
        map.insert("arrowright", "ArrowRight");
        map.insert("arrowup", "ArrowUp");
        map.insert("backspace", "Backspace");
        map.insert("capslock", "CapsLock");
        map.insert("cmd", "Meta");
        map.insert("ctrl", "Control");
        map.insert("delete", "Delete");
        map.insert("end", "End");
        map.insert("enter", "Enter");
        map.insert("esc", "Escape");
        map.insert("home", "Home");
        map.insert("insert", "Insert");
        map.insert("option", "Alt");
        map.insert("pagedown", "PageDown");
        map.insert("pageup", "PageUp");
        map.insert("shift", "Shift");
        map.insert("space", " ");
        map.insert("super", "Meta");
        map.insert("tab", "Tab");
        map.insert("win", "Meta");
        map
    };
}
