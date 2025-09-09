use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetrieveRelevantPlans {
    Never,
    Hint,
    Reuse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub cooperative_planning: bool,
    pub autonomous_execution: bool,
    pub allow_follow_up_input: bool,
    pub plan: Option<super::types::Plan>,
    pub max_turns: Option<usize>,
    pub allow_for_replans: bool,
    pub max_json_retries: usize,
    pub saved_facts: Option<String>,
    pub allowed_websites: Option<Vec<String>>,
    pub do_bing_search: bool,
    pub final_answer_prompt: Option<String>,
    pub model_context_token_limit: Option<usize>,
    pub retrieve_relevant_plans: RetrieveRelevantPlans,
    pub memory_controller_key: Option<String>,
    pub max_replans: Option<usize>,
    pub no_overwrite_of_task: bool,
    pub sentinel_tasks: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            cooperative_planning: true,
            autonomous_execution: false,
            allow_follow_up_input: true,
            plan: None,
            max_turns: Some(20),
            allow_for_replans: true,
            max_json_retries: 3,
            saved_facts: None,
            allowed_websites: None,
            do_bing_search: false,
            final_answer_prompt: None,
            model_context_token_limit: None,
            retrieve_relevant_plans: RetrieveRelevantPlans::Never,
            memory_controller_key: None,
            max_replans: Some(3),
            no_overwrite_of_task: false,
            sentinel_tasks: false,
        }
    }
}