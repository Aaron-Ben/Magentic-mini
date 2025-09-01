use crate::llm::LlmClient;
use crate::types::*;
use crate::agents::PlanAgent;
use anyhow::Result;

pub struct Orchestrator {
    plan_agent: PlanAgent,
}

impl Orchestrator {
    pub fn new(llm_client: LlmClient) -> Self {
        let plan_agent = PlanAgent::new(llm_client);
        Self { plan_agent }
    }

    pub async fn orchestrator_step_planning(&self, user_input: &str) -> Result<Plan> {
        self.plan_agent.generate_plan_from_input(user_input).await
    }
}