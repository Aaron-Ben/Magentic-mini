use std::env;

use crate::common::define_module_client;

use async_openai:: {config:: OpenAIClient,Client};

define_module_client! {
    (struct LlmClient, "llm")
    client_type: Client<OpenAIConfig>,
    env: ["OPENAI_BASE_URL", "OPENAI_API_KEY"],
    setup: async {
        let base_url = env::var("OPENAI_BASE_URL").expect("OPENAI_BASE_URL is not set");
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is not set");
        let openai_config = OpenAIConfig::new()
            .with_api_base(base_url)
            .with_api_key(api_key);

        Client::build(
            reqwest::Client::new(),
            openai_config,
            Default::default()
        )
    }
}