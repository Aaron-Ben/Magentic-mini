use crate::llm::LlmClient;
use crate::agents::plan_agent::{PlanAgent};
use crate::agents::plan_agent::config::OrchestratorConfig;
use anyhow::Result;

pub struct Orchestrator {
    pub plan_agent: PlanAgent,
}

impl Orchestrator {
    pub fn new(llm_client: LlmClient) -> Self {
        let config = OrchestratorConfig::default(); // 需要在 config.rs 中实现 Default
        let plan_agent = PlanAgent::new(llm_client, config);
        Self { plan_agent }
    }

    pub async fn orchestrator_step_planning(&mut self, user_input: &str) -> Result<()> {
        self.plan_agent.handle_user_input(user_input).await
    }
}