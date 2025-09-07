mod types;
mod agents;
mod llm;
mod cli;

use cli::CliInterface;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    if std::env::var("DASHSCOPE_API_KEY").is_err() {
        eprintln!("Error: DASHSCOPE_API_KEY environment variable is required");
        eprintln!("Please set it with: export DASHSCOPE_API_KEY=your_api_key");
        eprintln!("Get your API key from: https://dashscope.console.aliyun.com/");
        std::process::exit(1);
    }
    
    let mut cli_interface = CliInterface::new()?;
    cli_interface.run().await?;
    
    Ok(())
}