use anyhow::Result;
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use std::env;
use tracing::info;

use pathfinder::db::{setup_database, Repository};

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
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/pathfinder".to_string());

    // Setup database
    info!("Setting up database connection");
    let pool = setup_database(&database_url).await?;
    let repository = Repository::new(pool);

    // Calculate previous day's date
    let previous_day = (Utc::now() - Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    info!("Fetching stats for date: {}", previous_day);

    // Get the previous day's game
    let game = repository.get_game_by_date(&previous_day).await?;

    let report = match game {
        Some(game) => {
            info!("Found game for {}: {}", previous_day, game.id);

            // Get optimal solutions (professor's answers)
            let optimal_solutions = repository.get_optimal_solutions(&game.id).await?;

            // Generate report
            let mut report = String::new();
            report.push_str(&format!("Game #{} {}\n", game.sequence_number, game.date));
            report.push_str("\n");
            report.push_str("Professor's Answers:\n");
            
            let mut total_score = 0;
            for solution in optimal_solutions {
                report.push_str(&format!("{}: {}\n", solution.word, solution.score));
                total_score += solution.score;
            }
            
            report.push_str("\n");
            report.push_str(&format!("Total: {}\n", total_score));
            
            Some(report)
        }
        None => {
            info!("No game found for date: {}\n", previous_day);
            None
        }
    };

    if let Some(report) = report {
        print!("{}", report);
    }
    Ok(())
}

