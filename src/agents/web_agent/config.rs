use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAgentConfig {
    pub name: String,
    // model_client: ComponentModel | Dict[str, Any]
    // browser: ComponentModel | Dict[str, Any]
    pub model_context_token_limit: Option<usize>,
    pub downloads_folder: Option<String>,
    pub description: Option<String>,
    pub debug_dir: Option<String>,
    pub start_page: Option<String>,
    pub animate_actions: bool,
    pub to_save_screenshots: bool,
    pub max_actions_per_step: usize,
    pub to_resize_viewport: bool,
    // pub url_statuses: Option<HashMap<String, UrlStatus>>,
    pub url_block_list: Option<Vec<String>>,
    pub single_tab_mode: bool,
    pub json_model_output: bool,
    pub multiple_tools_per_call: bool,
    pub viewport_height: usize,
    pub viewport_width: usize,
    pub use_action_guard: bool,
}