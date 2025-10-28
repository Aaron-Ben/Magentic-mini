use std::env;
use crate::define_module_client;
use async_openai::{
    config::OpenAIConfig,
    Client,
};

define_module_client! {
    (struct LlmClient, "llm")
    client_type: Client<OpenAIConfig>,
    env: ["DASHSCOPE_BASE_URL", "DASHSCOPE_API_KEY"],
    setup: async {
        let base_url = env::var("DASHSCOPE_BASE_URL").expect("DASHSCOPE_BASE_URL is not set");
        let api_key = env::var("DASHSCOPE_API_KEY").expect("DASHSCOPE_API_KEY is not set");
        let llm_config = OpenAIConfig::new()
            .with_api_base(base_url)
            .with_api_key(api_key);

        Client::build(
            reqwest::Client::new(),
            llm_config,
            Default::default()
        )
    }
}