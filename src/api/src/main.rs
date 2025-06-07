mod db;
mod game;
mod game_generator;
mod service;
mod serialization;
mod http_api;

use anyhow::Result;
use dotenvy::dotenv;
use std::env;
use tokio_cron_scheduler::{JobScheduler, Job};
use tonic::transport::Server;
use tower_http::cors::CorsLayer;
use tracing::{info, error};

use db::{setup_database, Repository};
use game::GameEngine;
use game_generator::GameGenerator;
use service::{WordGameServiceImpl, wordgame::word_game_service_server::WordGameServiceServer};

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
        .unwrap_or_else(|_| "sqlite://game.db".to_string());
    let server_host = env::var("SERVER_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let grpc_port = env::var("GRPC_PORT")
        .unwrap_or_else(|_| "50051".to_string())
        .parse::<u16>()
        .expect("Invalid GRPC_PORT");
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

    // Setup game generator
    let game_generator = GameGenerator::new(repository.clone(), game_engine.clone());

    // Generate missing games on startup
    info!("Generating missing games on startup");
    if let Err(e) = game_generator.generate_missing_games().await {
        error!("Failed to generate missing games on startup: {}", e);
    }

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

    // Setup HTTP API
    let api_state = http_api::ApiState::new(repository.clone(), game_engine.clone());
    let http_router = http_api::create_router(api_state);
    
    // Setup gRPC service
    let grpc_service = WordGameServiceImpl::new(repository, game_engine);
    let grpc_service = WordGameServiceServer::new(grpc_service);

    let grpc_addr = format!("{}:{}", server_host, grpc_port).parse::<std::net::SocketAddr>()?;
    let http_addr = format!("{}:{}", server_host, http_port);
    
    info!("Starting HTTP API server on {}", http_addr);
    info!("Starting gRPC server on {}", grpc_addr);
    
    // Run both servers concurrently
    let http_server = axum::serve(
        tokio::net::TcpListener::bind(&http_addr).await?,
        http_router
    );
    
    let grpc_server = Server::builder()
        .add_service(grpc_service)
        .serve(grpc_addr);

    tokio::try_join!(
        async { http_server.await.map_err(|e| anyhow::anyhow!("HTTP server error: {}", e)) },
        async { grpc_server.await.map_err(|e| anyhow::anyhow!("gRPC server error: {}", e)) }
    )?;

    Ok(())
}