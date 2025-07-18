use anyhow::Result;
use chrono::{Duration, NaiveDate, Utc};
use pathfinder::db::conversions::AnswerStorage;
use pathfinder::db::repository_simple::Repository;
use pathfinder::game::{conversion::SerializableBoard, GameEngine};
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create database connection
    let pool = PgPool::connect(&database_url).await?;
    let repo = Repository::new(pool);

    // Initialize game engine for validation
    let game_engine = GameEngine::new(std::path::PathBuf::from("wordlist"));

    // Find the most recent UTC date that has ended in all timezones (including Baker Island/Howland Island at UTC-12)
    let target_date = get_most_recent_completed_date();
    let date_str = target_date.format("%Y-%m-%d").to_string();

    println!("Processing games for date: {date_str}");

    // Get all incomplete games for this date
    let incomplete_games = repo.get_incomplete_games_for_date(&date_str).await?;

    if incomplete_games.is_empty() {
        println!("No incomplete games found for date {date_str}");
        return Ok(());
    }

    println!(
        "Found {} incomplete games for {}",
        incomplete_games.len(),
        date_str
    );

    // Process each game
    for game in incomplete_games {
        println!("Processing game {} ({})", game.id, game.date);

        // Get incomplete game entries for this game
        let incomplete_entries = repo.get_incomplete_game_entries_for_game(&game.id).await?;

        if incomplete_entries.is_empty() {
            println!("  No incomplete entries found for game {}", game.id);
            continue;
        }

        println!("  Found {} incomplete entries", incomplete_entries.len());

        // Get the game data for validation
        let game_data = match repo.get_game_by_id(&game.id).await? {
            Some(game) => game,
            None => {
                println!("  Game data not found for game {}", game.id);
                continue;
            }
        };

        // Parse the board from game data
        let serializable_board: SerializableBoard =
            match serde_json::from_str(&game_data.board_data) {
                Ok(board) => board,
                Err(e) => {
                    println!("  Failed to parse board data for game {}: {}", game.id, e);
                    continue;
                }
            };
        let board: pathfinder::game::Board = serializable_board.into();

        // Check each entry to see if it has valid and complete answers
        let mut completed_entries = 0;
        for entry in incomplete_entries {
            // Parse answers from the entry
            let answers = match AnswerStorage::deserialize_to_api_answers(&entry.answers_data) {
                Ok(answers) => answers,
                Err(e) => {
                    println!("    Entry {} has invalid answer data: {}", entry.id, e);
                    continue;
                }
            };

            // Check if entry has exactly 5 answers
            if answers.len() != 5 {
                println!(
                    "    Entry {} has {} answers (expected 5), skipping",
                    entry.id,
                    answers.len()
                );
                continue;
            }

            // Validate answers using GameEngine
            if let Err(validation_error) = game_engine.validate_api_answer_group(&board, answers) {
                println!(
                    "    Entry {} has invalid answers: {}",
                    entry.id, validation_error
                );
                continue;
            }

            println!("    Marking entry {} as completed", entry.id);
            repo.mark_game_entry_completed(&entry.id).await?;
            completed_entries += 1;
        }

        println!("  Marked {completed_entries} entries as completed");

        // Mark the game itself as completed
        println!("  Marking game {} as completed", game.id);
        repo.mark_game_completed(&game.id).await?;
    }

    println!("Completed processing games for date {date_str}");
    Ok(())
}

/// Returns the most recent UTC date that has ended even in the latest timezone (Baker Island/Howland Island: UTC-12)
fn get_most_recent_completed_date() -> NaiveDate {
    let now = Utc::now();

    // Baker Island/Howland Island are at UTC-12, the furthest timezone behind UTC
    // We need to subtract 12 hours from UTC to get the local time there.
    let latest_timezone_time = now - Duration::hours(12);

    // Get the date in the latest timezone
    let latest_timezone_date = latest_timezone_time.date_naive();

    // Since we want the most recent date that has completely ended everywhere,
    // we need to go back one more day from the latest timezone date
    latest_timezone_date - Duration::days(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_get_most_recent_completed_date() {
        // Mock the current time for testing
        let test_date = get_most_recent_completed_date();
        let now = Utc::now();

        // The returned date should be at least 1 day ago
        assert!(test_date < now.date_naive());

        // The returned date should be within reasonable bounds (not too far in the past)
        let days_diff = (now.date_naive() - test_date).num_days();
        assert!((1..=2).contains(&days_diff));
    }
}
