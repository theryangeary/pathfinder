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
    let game_engine = GameEngine::new(std::path::PathBuf::from("wordlist"));

    let date_str = "2025-06-30".to_string();
    let seed = Seeder::from(date_str).make_seed();
    let mut rng = rand::rngs::StdRng::from_seed(seed);

    match game_engine.try_generate_valid_board(&mut rng, 40).await {
        Ok((board, _, (optimal_words, metadata))) => {
            info!("Generated board: \n{}\n\nOptimal Answers:", board,);
            for (i, (word, score)) in optimal_words
                .iter()
                .zip(metadata.individual_scores.iter())
                .enumerate()
            {
                info!("{}. {} (score: {})", i + 1, word.word, score);
            }
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
