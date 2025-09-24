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

    let sqlite_database_url =
        env::var("SQLITE_DATABASE_URL").unwrap_or_else(|_| "sqlite://pathfinder.db".to_string());

    // Setup database
    info!("Setting up database connection");
    let _ = setup_database(&sqlite_database_url).await?;

    Ok(())
}
