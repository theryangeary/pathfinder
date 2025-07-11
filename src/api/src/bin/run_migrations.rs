use anyhow::Result;
use dotenvy::dotenv;
use std::env;
use tracing::info;

use pathfinder::db::setup_database;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting stat-poster for previous day's puzzle");

    // Get configuration from environment
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/pathfinder".to_string());

    // Setup database
    info!("Setting up database connection");
    let _ = setup_database(&database_url).await?;

    Ok(())
}
