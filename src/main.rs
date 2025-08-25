mod types;
mod agents;
mod llm;
mod cli;

use cli::CliInterface;
use anyhow::Result;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载 .env 文件
    dotenv::dotenv().ok();
    
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 检查环境变量
    if std::env::var("DASHSCOPE_API_KEY").is_err() {
        eprintln!("❌ Error: DASHSCOPE_API_KEY environment variable is required");
        eprintln!("Please set it with: export DASHSCOPE_API_KEY=your_api_key");
        eprintln!("Get your API key from: https://dashscope.console.aliyun.com/");
        std::process::exit(1);
    }
    
    // 启动 CLI
    let cli = CliInterface::new()?;
    cli.run().await?;
    
    Ok(())
}