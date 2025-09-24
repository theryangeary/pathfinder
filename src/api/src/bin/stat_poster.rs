use anyhow::Result;
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use std::env;
use tracing::{info, warn};

use pathfinder::db::{setup_database, Repository, SqliteRepository};
use pathfinder::social::{bluesky::BlueSkyPoster, Post};

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
    let postgres_database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/pathfinder".to_string());

    let sqlite_database_url =
        env::var("SQLITE_DATABASE_URL").unwrap_or_else(|_| "sqlite://pathfinder.db".to_string());

    // Setup database
    info!("Setting up database connection");
    let pool = setup_database(&postgres_database_url, &sqlite_database_url).await?;
    let sqlite_repository = SqliteRepository::new(pool.1);

    // Calculate previous day's date
    let previous_day = (Utc::now() - Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    info!("Fetching stats for date: {}", previous_day);

    // Get the previous day's game
    let game = sqlite_repository.get_game_by_date(&previous_day).await?;

    let report = match game {
        Some(game) => {
            info!("Found game for {}: {}", previous_day, game.id);

            // Get optimal solutions (professor's answers)
            let optimal_solutions = sqlite_repository.get_optimal_solutions(&game.id).await?;

            // Generate report
            let mut report = String::new();
            report.push_str(&format!("Game #{} {}\n", game.sequence_number, game.date));
            report.push('\n');
            report.push_str("Professor's Answers:\n");

            let mut total_score = 0;
            for solution in optimal_solutions {
                report.push_str(&format!("{}: {}\n", solution.word, solution.score));
                total_score += solution.score;
            }

            report.push('\n');
            report.push_str(&format!("Total: {total_score}\n"));

            Some(report)
        }
        None => {
            info!("No game found for date: {previous_day}\n");
            None
        }
    };

    if let Some(report) = report {
        print!("{report}");

        // Post to BlueSky if credentials are available
        if let (Ok(handle), Ok(password)) =
            (env::var("BLUESKY_HANDLE"), env::var("BLUESKY_PASSWORD"))
        {
            info!("Posting to BlueSky...");

            let mut poster = BlueSkyPoster::new(handle).with_password(password);

            match poster.authenticate().await {
                Ok(()) => match poster.post(report).await {
                    Ok(()) => info!("Successfully posted to BlueSky"),
                    Err(e) => warn!("Failed to post to BlueSky: {}", e),
                },
                Err(e) => warn!("Failed to authenticate with BlueSky: {}", e),
            }
        } else {
            info!("BlueSky credentials not found in environment, skipping post");
        }
    }
    Ok(())
}
