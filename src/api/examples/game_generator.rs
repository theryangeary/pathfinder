use anyhow::Result;
use dotenvy::dotenv;
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use pathfinder::db::{setup_database, Repository};
use pathfinder::game::GameEngine;
use pathfinder::game_generator::GameGenerator;

async fn run_game_generation() -> Result<()> {
    // Get configuration from environment
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/pathfinder".to_string());

    // // Setup database
    info!("Setting up database connection");
    let pool = setup_database(&database_url).await?;
    let repository = Repository::new(pool);

    // // Setup game engine
    // info!("Initializing game engine");
    let game_engine = GameEngine::new(std::path::PathBuf::from("wordlist"));

    // // Setup game generator
    let game_generator = GameGenerator::new(repository, game_engine);

    // // Generate missing games
    // info!("Generating missing games");
    // match game_generator.generate_missing_games().await {
    //     Ok(()) => {
    //         info!("Game generation completed successfully");
    //     }
    //     Err(e) => {
    //         error!("Game generation failed: {}", e);
    //         return Err(e);
    //     }
    // }

    let date_str = "2025-06-30".to_string();
    match game_generator.generate_game_for_date(&date_str).await {
        Ok(game) => {
            info!(
                "Generated game for past date: {} with ID: {}",
                date_str, game.id
            );
            dbg!(game);
        }
        Err(e) => {
            error!("Failed to generate game for past date {}: {}", date_str, e);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
        // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    run_game_generation().await?;
    Ok(())
}
