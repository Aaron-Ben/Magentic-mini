use serde::{Serialize, Deserialize};
use crate::types::plan::Plan;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub cooperative_planning: bool,
    pub autonomous_execution: bool,
    pub allow_follow_up_input: bool,
    pub plan: Option<Plan>,
    pub max_turns: Option<usize>,
    pub allow_for_replans: bool,
    pub max_json_retries: usize,
    pub saved_facts: Option<String>,
    pub allowed_websites: Option<Vec<String>>,
    pub do_bing_search: bool,
    pub final_answer_prompt: Option<String>,
    pub model_context_token_limit: Option<usize>,
    pub retrieve_relevant_plans: Option<String>,
}