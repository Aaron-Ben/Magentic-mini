use anyhow::Result;
use mini_magentic_backend::clients::PostgresClient;
use mini_magentic_backend::common::ModuleClient;
use std::path::Path;
#[tokio::main]
async fn main() -> Result<()> {
    dotenv::from_path(Path::new("backend/.env")).ok();
    println!("DATABASE_URL = {:?}", std::env::var("DATABASE_URL"));
    let _postgres = PostgresClient::setup_connection().await;
    println!("postgres 创建成功");
    Ok(())
}
