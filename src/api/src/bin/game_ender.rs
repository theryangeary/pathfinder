use anyhow::Result;
use chrono::{Duration, NaiveDate, Utc};
use pathfinder::db::{models::DbGameEntry, repository_simple::Repository};
use serde_json::Value;
use sqlx::PgPool;
use std::env;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create database connection
    let pool = PgPool::connect(&database_url).await?;
    let repo = Repository::new(pool);

    // Find the most recent UTC date that has ended in all timezones (including Baker Island/Howland Island at UTC-12)
    let target_date = get_most_recent_completed_date();
    let date_str = target_date.format("%Y-%m-%d").to_string();

    info!("Processing games for date: {}", date_str);

    // Get all incomplete games for this date
    let incomplete_games = repo.get_incomplete_games_for_date(&date_str).await?;

    if incomplete_games.is_empty() {
        info!("No incomplete games found for date {}", date_str);
        return Ok(());
    }

    info!(
        "Found {} incomplete games for {}",
        incomplete_games.len(),
        date_str
    );

    // Process each game
    for game in incomplete_games {
        info!("Processing game {} ({})", game.id, game.date);

        // Get incomplete game entries for this game
        let incomplete_entries = repo.get_incomplete_game_entries_for_game(&game.id).await?;

        if incomplete_entries.is_empty() {
            info!("  No incomplete entries found for game {}", game.id);
            continue;
        }

        info!("  Found {} incomplete entries", incomplete_entries.len());

        // Check each entry to see if it has valid and complete answers
        let mut completed_entries = 0;
        for entry in incomplete_entries {
            if has_valid_complete_answers(&entry)? {
                info!("    Marking entry {} as completed", entry.id);
                repo.mark_game_entry_completed(&entry.id).await?;
                completed_entries += 1;
            }
        }

        if completed_entries > 0 {
            info!("  Marked {} entries as completed", completed_entries);

            // Mark the game itself as completed
            info!("  Marking game {} as completed", game.id);
            repo.mark_game_completed(&game.id).await?;
        } else {
            warn!("  No entries were marked as completed for game {}", game.id);
        }
    }

    info!("Completed processing games for date {}", date_str);
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

/// Checks if a game entry has valid and complete answers
fn has_valid_complete_answers(entry: &DbGameEntry) -> Result<bool> {
    // Parse the answers_data JSON
    let answers: Value = serde_json::from_str(&entry.answers_data)?;

    // Check if answers is an array and has exactly 5 elements (complete game)
    if let Some(answers_array) = answers.as_array() {
        if answers_array.len() != 5 {
            return Ok(false);
        }

        // Check that all 5 answers are valid (not null/empty)
        for answer in answers_array {
            if answer.is_null() {
                return Ok(false);
            }

            // If it's a string, check it's not empty
            if let Some(answer_str) = answer.as_str() {
                if answer_str.trim().is_empty() {
                    return Ok(false);
                }
            }
            // If it's an object, check it has required fields
            else if let Some(answer_obj) = answer.as_object() {
                // Check if it has a "word" field that's not empty
                if let Some(word) = answer_obj.get("word") {
                    if let Some(word_str) = word.as_str() {
                        if word_str.trim().is_empty() {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        // All 5 answers are valid and complete
        Ok(true)
    } else {
        // answers_data is not an array
        Ok(false)
    }
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

    #[test]
    fn test_has_valid_complete_answers_valid() {
        let entry = DbGameEntry {
            id: "test".to_string(),
            user_id: "user".to_string(),
            game_id: "game".to_string(),
            answers_data: r#"["word1", "word2", "word3", "word4", "word5"]"#.to_string(),
            total_score: 100,
            completed: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(has_valid_complete_answers(&entry).unwrap());
    }

    #[test]
    fn test_has_valid_complete_answers_incomplete() {
        let entry = DbGameEntry {
            id: "test".to_string(),
            user_id: "user".to_string(),
            game_id: "game".to_string(),
            answers_data: r#"["word1", "word2", null, "word4", "word5"]"#.to_string(),
            total_score: 80,
            completed: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(!has_valid_complete_answers(&entry).unwrap());
    }

    #[test]
    fn test_has_valid_complete_answers_too_few() {
        let entry = DbGameEntry {
            id: "test".to_string(),
            user_id: "user".to_string(),
            game_id: "game".to_string(),
            answers_data: r#"["word1", "word2", "word3"]"#.to_string(),
            total_score: 60,
            completed: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(!has_valid_complete_answers(&entry).unwrap());
    }

    #[test]
    fn test_has_valid_complete_answers_object_format() {
        let entry = DbGameEntry {
            id: "test".to_string(),
            user_id: "user".to_string(),
            game_id: "game".to_string(),
            answers_data: r#"[{"word": "word1"}, {"word": "word2"}, {"word": "word3"}, {"word": "word4"}, {"word": "word5"}]"#.to_string(),
            total_score: 100,
            completed: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(has_valid_complete_answers(&entry).unwrap());
    }

    #[test]
    fn test_has_valid_complete_answers_object_format_empty_word() {
        let entry = DbGameEntry {
            id: "test".to_string(),
            user_id: "user".to_string(),
            game_id: "game".to_string(),
            answers_data: r#"[{"word": "word1"}, {"word": ""}, {"word": "word3"}, {"word": "word4"}, {"word": "word5"}]"#.to_string(),
            total_score: 80,
            completed: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(!has_valid_complete_answers(&entry).unwrap());
    }
}
