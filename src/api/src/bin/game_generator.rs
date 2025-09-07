use anyhow::Result;
use dotenvy::dotenv;
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use pathfinder::db::{setup_database, PgRepository, SqliteRepository};
use pathfinder::game::GameEngine;
use pathfinder::game_generator::GameGenerator;

async fn run_game_generation() -> Result<()> {
    // Get configuration from environment
    let postgres_database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/pathfinder".to_string());

    let sqlite_database_url =
        env::var("SQLITE_DATABASE_URL").unwrap_or_else(|_| "sqlite://pathfinder.db".to_string());

    // Setup database
    info!("Setting up database connection");
    let pool = setup_database(&postgres_database_url, &sqlite_database_url).await?;
    let postgres_repository = PgRepository::new(pool.0);
    let _sqlite_repository = SqliteRepository::new(pool.1);

    // Setup game engine
    info!("Initializing game engine");
    let game_engine = GameEngine::new(std::path::PathBuf::from("wordlist"));

    // Setup game generator
    let game_generator = GameGenerator::new(postgres_repository, game_engine);

    // Generate missing games
    info!("Generating missing games");
    match game_generator.generate_missing_games().await {
        Ok(()) => {
            info!("Game generation completed successfully");
        }
        Err(e) => {
            error!("Game generation failed: {}", e);
            return Err(e);
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

    // Check for --cron flag
    let args: Vec<String> = env::args().collect();
    let is_cron_mode = args.contains(&"--cron".to_string());

    if is_cron_mode {
        info!("Starting game generator in cron mode");

        // Run once on startup
        info!("Running initial game generation on startup");
        if let Err(e) = run_game_generation().await {
            error!("Initial game generation failed: {}", e);
        }

        // Create scheduler
        let sched = JobScheduler::new().await?;

        // Add job that runs at 1:30 AM UTC daily
        sched
            .add(Job::new_async("0 30 1 * * *", |_uuid, _l| {
                Box::pin(async {
                    info!("Running scheduled game generation");
                    if let Err(e) = run_game_generation().await {
                        error!("Scheduled game generation failed: {}", e);
                    }
                })
            })?)
            .await?;

        // Start the scheduler
        sched.start().await?;

        info!("Game generator cron scheduler started - running daily at 1:30 AM UTC");

        // Keep the process running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await; // Sleep for 1 hour
        }
    } else {
        info!("Starting game generator (single run)");
        run_game_generation().await?;
        info!("Game generator finished successfully");
    }

    Ok(())
}
