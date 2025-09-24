use anyhow::Result;
use dotenvy::dotenv;
use std::{env, time::Duration};
use tracing::info;

use pathfinder::db::{setup_database, SqliteRepository};
use pathfinder::game::GameEngine;
use pathfinder::memory_profiler::MemoryProfiler;
use pathfinder::security::SecurityConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting word game backend server");

    // Initialize memory profiler
    let mut memory_profiler = MemoryProfiler::new();
    memory_profiler.log_memory("startup");

    let sqlite_database_url =
        env::var("SQLITE_DATABASE_URL").unwrap_or_else(|_| "sqlite://pathfinder.db".to_string());

    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let http_port = env::var("HTTP_PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .expect("Invalid HTTP_PORT");

    // Setup database
    info!("Setting up database connection");
    let pool = setup_database(&sqlite_database_url).await?;
    // let _postgres_repository = PgRepository::new(pool.0);
    let sqlite_repository = SqliteRepository::new(pool);
    memory_profiler.log_memory("after_database_setup");

    // Setup game engine
    info!("Initializing game engine");
    let game_engine = GameEngine::new(std::path::PathBuf::from("wordlist"));
    memory_profiler.log_memory("after_game_engine_init");

    // Setup security configuration
    info!("Loading security configuration");
    let security_config = SecurityConfig::from_env();
    memory_profiler.log_memory("after_security_config");

    // Setup HTTP API
    info!("Creating API state");
    let api_state =
        pathfinder::http_api::ApiState::new(sqlite_repository.clone(), game_engine.clone());
    memory_profiler.log_memory("after_api_state");

    info!("Creating secure router");
    let http_router = pathfinder::http_api::create_secure_router(api_state, security_config);
    memory_profiler.log_memory("after_secure_router_creation");

    let http_addr = format!("{server_host}:{http_port}");

    info!("Starting HTTP API server on {}", http_addr);
    memory_profiler.log_memory("after_http_setup");

    // Start HTTP server
    let http_server = axum::serve(
        tokio::net::TcpListener::bind(&http_addr).await?,
        http_router,
    );

    memory_profiler.log_memory("after_full_startup");

    // Start 10-second memory monitoring
    let monitoring_duration = Duration::from_secs(3);
    let monitoring_interval = Duration::from_secs(1);

    info!("Starting 10-second memory monitoring");
    tokio::spawn(async move {
        memory_profiler
            .monitor_for_duration(monitoring_duration, monitoring_interval)
            .await;
        info!("Memory monitoring completed");
    });

    // Run HTTP server
    http_server
        .await
        .map_err(|e| anyhow::anyhow!("HTTP server error: {}", e))?;

    Ok(())
}
