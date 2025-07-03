use anyhow::Result;
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use plotters::prelude::*;
use std::collections::HashMap;
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
    dbg!(&database_url);

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

    match game {
        Some(game) => {
            info!("Found game for {}: {}", previous_day, game.id);

            // Get score distribution
            let score_distribution = repository.get_score_distribution(&game.id).await?;
            info!("Found {} score entries", score_distribution.len());

            // Get optimal solutions
            let optimal_solutions = repository.get_optimal_solutions(&game.id).await?;

            // Generate graph
            let image_path = generate_score_distribution_graph(&score_distribution, &previous_day)?;

            // Output results
            println!("Image saved to: {image_path}");
            println!("Optimal answers for {previous_day}:");
            for solution in optimal_solutions {
                println!("  {} ({})", solution.word, solution.score);
            }
        }
        None => {
            println!("No game found for date: {previous_day}");
        }
    }

    Ok(())
}

fn generate_score_distribution_graph(scores: &[i32], date: &str) -> Result<String> {
    let image_path = format!("/tmp/score_distribution_{date}.png");

    // Create histogram data
    let mut histogram: HashMap<i32, i32> = HashMap::new();
    for &score in scores {
        *histogram.entry(score).or_insert(0) += 1;
    }

    if histogram.is_empty() {
        info!("No scores to plot");
        anyhow::bail!("no scores to plot");
    }

    // Convert to sorted vector for plotting
    let mut data: Vec<(i32, i32)> = histogram.into_iter().collect();
    data.sort_by_key(|(score, _)| *score);

    let min_score = data.first().map(|(s, _)| *s).unwrap_or(0);
    let max_score = data.last().map(|(s, _)| *s).unwrap_or(0);
    let max_count = data.iter().map(|(_, c)| *c).max().unwrap_or(0);

    // Create the plot
    let path_for_backend = image_path.clone();
    let root = BitMapBackend::new(&path_for_backend, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Score Distribution for {date}"), ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(min_score..max_score, 0..max_count)?;

    chart
        .configure_mesh()
        .x_desc("Score")
        .y_desc("Number of Players")
        .draw()?;

    chart
        .draw_series(data.iter().map(|(score, count)| {
            Rectangle::new([(*score, 0), (*score, *count)], BLUE.filled())
        }))?;

    root.present()?;
    info!("Score distribution graph saved to: {}", image_path);

    Ok(image_path)
}
