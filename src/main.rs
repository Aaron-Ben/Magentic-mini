mod types;
mod agents;
mod llm;
mod cli;
mod routes;

use cli::CliInterface;
use anyhow::Result;
use tracing_subscriber;
use clap::{Parser, Subcommand};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

use routes::{AppState, create_routes};
use agents::Orchestrator;
use llm::LlmClient;

#[derive(Parser)]
#[command(name = "mini-magentic-ui")]
#[command(about = "智能任务规划和执行系统")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动 CLI 交互模式
    Cli,
    /// 启动 Web 服务器模式
    Web {
        /// 服务器端口
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
}

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
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Cli) | None => {
            // 启动 CLI
            let cli_interface = CliInterface::new()?;
            cli_interface.run().await?;
        }
        Some(Commands::Web { port }) => {
            // 启动 Web 服务器
            start_web_server(port).await?;
        }
    }
    
    Ok(())
}

async fn start_web_server(port: u16) -> Result<()> {
    println!("🚀 Starting Mini Magentic-UI Web Server on port {}", port);
    
    // 创建 LLM 客户端和 Orchestrator
    let llm_client = LlmClient::new()?;
    let orchestrator = Arc::new(Orchestrator::new(llm_client));
    
    // 创建应用状态
    let state = AppState {
        orchestrator,
        plans: Arc::new(Mutex::new(HashMap::new())),
    };
    
    // 创建路由
    let api_routes = create_routes();
    
    let app = Router::new()
        .merge(api_routes)
        .nest_service("/", ServeDir::new("frontend/dist"))
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    // 启动服务器
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("🌐 Server running at http://localhost:{}", port);
    println!("📱 Frontend available at http://localhost:{}", port);
    println!("🔗 API available at http://localhost:{}/api", port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}