use anyhow::Result;
use dotenvy::dotenv;
use std::env;
use tracing::{error, info};

use word_game_backend::db::{setup_database, Repository};
use word_game_backend::game::GameEngine;
use word_game_backend::game_generator::GameGenerator;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting game generator");

    // Get configuration from environment
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/pathfinder".to_string());

    // Setup database
    info!("Setting up database connection");
    let pool = setup_database(&database_url).await?;
    let repository = Repository::new(pool);

    // Setup game engine
    info!("Initializing game engine");
    let game_engine = GameEngine::new(std::path::PathBuf::from("wordlist"));

    // Setup game generator
    let game_generator = GameGenerator::new(repository, game_engine);

    // Generate missing games
    info!("Generating missing games");
    match game_generator.generate_missing_games().await {
        Ok(()) => {
            info!("Game generation completed successfully");
        }
        Err(e) => {
            error!("Game generation failed: {}", e);
            std::process::exit(1);
        }
    }

    info!("Game generator finished successfully");
    Ok(())
}
