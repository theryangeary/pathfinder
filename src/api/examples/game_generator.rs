use anyhow::Result;
use dotenvy::dotenv;
use rand::SeedableRng;
use rand_seeder::Seeder;
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use pathfinder::db::{setup_database, Repository};
use pathfinder::game::GameEngine;
use pathfinder::game_generator::GameGenerator;

async fn run_game_generation() -> Result<()> {

    // // Setup game engine
    // info!("Initializing game engine");
    let game_engine = GameEngine::new(std::path::PathBuf::from("wordlist"));

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
    let seed = Seeder::from(date_str).make_seed();
    let mut rng = rand::rngs::StdRng::from_seed(seed);

    match game_engine.try_generate_valid_board(&mut rng, 40).await {
        Ok((board, answers)) => {
            info!(
                "Generated board: \n{}",
                board,
            );
        }
        Err(e) => {
            error!("Failed to generate game: {}", e);
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
