mod db;
mod game;
mod game_generator;
mod http_api;
mod security;

use anyhow::Result;
use dotenvy::dotenv;
use std::env;
use tokio_cron_scheduler::{JobScheduler, Job};
use tracing::{info, error};

use db::{setup_database, Repository};
use game::GameEngine;
use game_generator::GameGenerator;
use security::SecurityConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting word game backend server");

    // Get configuration from environment
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/pathfinder".to_string());
    let server_host = env::var("SERVER_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let http_port = env::var("HTTP_PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .expect("Invalid HTTP_PORT");

    // Setup database
    info!("Setting up database connection");
    let pool = setup_database(&database_url).await?;
    let repository = Repository::new(pool);

    // Setup game engine
    info!("Initializing game engine");
    let game_engine = GameEngine::new("wordlist").await?;

    // Setup security configuration
    info!("Loading security configuration");
    let security_config = SecurityConfig::from_env();

    // Setup HTTP API
    let api_state = http_api::ApiState::new(repository.clone(), game_engine.clone());
    let http_router = http_api::create_secure_router(api_state, security_config);

    let http_addr = format!("{}:{}", server_host, http_port);
    
    info!("Starting HTTP API server on {}", http_addr);
    
    // Start HTTP server
    let http_server = axum::serve(
        tokio::net::TcpListener::bind(&http_addr).await?,
        http_router
    );

    // Setup game generator and run background tasks
    let game_generator = GameGenerator::new(repository.clone(), game_engine.clone());
    
    // Spawn background task for initial game generation
    let initial_game_generator = game_generator.clone();
    tokio::spawn(async move {
        info!("Generating missing games in background");
        if let Err(e) = initial_game_generator.generate_missing_games().await {
            error!("Failed to generate missing games in background: {}", e);
        } else {
            info!("Background game generation completed successfully");
        }
    });

    // Setup cron scheduler for daily game generation
    info!("Setting up cron scheduler");
    let sched = JobScheduler::new().await?;
    
    // Clone dependencies for the cron job
    let cron_game_generator = game_generator.clone();
    
    // Schedule job to run at midnight UTC every day
    sched.add(
        Job::new_async("0 0 0 * * *", move |_uuid, _l| {
            let game_generator = cron_game_generator.clone();
            Box::pin(async move {
                info!("Running scheduled game generation");
                if let Err(e) = game_generator.generate_missing_games().await {
                    error!("Scheduled game generation failed: {}", e);
                } else {
                    info!("Scheduled game generation completed successfully");
                }
            })
        })?
    ).await?;
    
    sched.start().await?;
    info!("Cron scheduler started");

    // Run HTTP server
    http_server.await.map_err(|e| anyhow::anyhow!("HTTP server error: {}", e))?;

    Ok(())
}
