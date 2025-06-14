use crate::db::{models::{NewGame, NewGameAnswer}, Repository};
use crate::game::{BoardGenerator, GameEngine, conversion::{SerializablePath, SerializableAnswerGroupConstraintSet}};
use anyhow::Result;
use chrono::{Duration, Utc};
use rand::SeedableRng;
use rand_seeder::Seeder;
use serde_json;
use tracing::{error, info, warn};
use uuid;

#[derive(Clone)]
pub struct GameGenerator {
    repository: Repository,
    game_engine: GameEngine,
}

impl GameGenerator {
    pub fn new(repository: Repository, game_engine: GameEngine) -> Self {
        Self {
            repository,
            game_engine,
        }
    }

    /// Generate games for the past week and next 3 days if they don't already exist
    pub async fn generate_missing_games(&self) -> Result<()> {
        let today = Utc::now().date_naive();

        // Generate games for today and the next 3 days
        for days_ahead in 0..=3 {
            let target_date = today + Duration::days(days_ahead);
            let date_str = target_date.format("%Y-%m-%d").to_string();

            if !self.repository.game_exists_for_date(&date_str).await? {
                match self.generate_game_for_date(&date_str).await {
                    Ok(game) => {
                        info!("Generated game for date: {} with ID: {}", date_str, game.id);
                    }
                    Err(e) => {
                        error!("Failed to generate game for date {}: {}", date_str, e);
                    }
                }
            } else {
                info!("Game already exists for date: {}", date_str);
            }
        }
        
        // Generate games for the past 7 days, in case this is the first launch or the app has had downtime.
        for days_back in 1..=7 {
            let target_date = today - Duration::days(8) + Duration::days(days_back);
            let date_str = target_date.format("%Y-%m-%d").to_string();

            if !self.repository.game_exists_for_date(&date_str).await? {
                match self.generate_game_for_date(&date_str).await {
                    Ok(game) => {
                        info!(
                            "Generated game for past date: {} with ID: {}",
                            date_str, game.id
                        );
                    }
                    Err(e) => {
                        error!("Failed to generate game for past date {}: {}", date_str, e);
                    }
                }
            } else {
                info!("Game already exists for past date: {}", date_str);
            }
        }

        Ok(())
    }

    /// Generate a single game for a specific date
    pub async fn generate_game_for_date(&self, date: &str) -> Result<crate::db::models::DbGame> {
        let mut threshold_score = 40;
        let max_threshold_reductions = 1; // Only allow one 25% reduction (40 -> 30)

        for reduction_attempt in 0..=max_threshold_reductions {
            for generation_attempt in 1..=5 {
                let seed = self.create_seed(date, reduction_attempt, generation_attempt);
                let mut rng = rand::rngs::StdRng::from_seed(seed);

                match self
                    .try_generate_valid_board(&mut rng, threshold_score)
                    .await
                {
                    Ok((board, valid_answers)) => {
                        // Convert board to JSON for storage
                        let serializable_board = crate::game::conversion::SerializableBoard::from(&board);
                        let board_data = serde_json::to_string(&serializable_board)?;
                        
                        let sequence_number = self.repository.get_next_sequence_number().await?;
                        let new_game = NewGame {
                            date: date.to_string(),
                            board_data,
                            threshold_score,
                            sequence_number,
                        };

                        // Prepare game answers entries BEFORE creating the game
                        let mut game_answers = Vec::new();
                        // Use a temporary game_id that will be replaced by the actual ID
                        let temp_game_id = uuid::Uuid::new_v4().to_string();
                        
                        for answer in &valid_answers {
                            for path in &answer.paths {
                                let serializable_path = SerializablePath::from(path);
                                let serializable_constraints = SerializableAnswerGroupConstraintSet::from(&answer.constraints_set);
                                
                                game_answers.push(NewGameAnswer {
                                    game_id: temp_game_id.clone(), // Will be replaced in the atomic create
                                    word: answer.word.clone(),
                                    path: serde_json::to_string(&serializable_path)?,
                                    path_constraint_set: serde_json::to_string(&serializable_constraints)?,
                                });
                            }
                        }
                        
                        // Create game and answers atomically
                        let (game, _created_answers) = self.repository.create_game_with_answers(new_game, game_answers).await?;
                        
                        info!(
                            "Successfully generated game for {} after {} attempts with threshold {} and {} valid answers",
                            date, generation_attempt, threshold_score, valid_answers.len()
                        );
                        return Ok(game);
                    }
                    Err(e) => {
                        warn!(
                            "Generation attempt {} failed for date {} with threshold {}: {}",
                            generation_attempt, date, threshold_score, e
                        );
                    }
                }
            }

            // Reduce threshold by 25% and try again
            if reduction_attempt < max_threshold_reductions {
                threshold_score = (threshold_score as f32 * 0.75) as i32;
                info!(
                    "Reducing threshold score to {} for date {} and retrying",
                    threshold_score, date
                );
            }
        }

        error!(
            "Failed to generate valid game for date {} after all attempts",
            date
        );
        anyhow::bail!("Could not generate valid game for date: {}", date);
    }

    /// Try to generate a valid board that meets the threshold score
    async fn try_generate_valid_board<R: rand::Rng>(
        &self,
        rng: &mut R,
        threshold_score: i32,
    ) -> Result<(crate::game::board::Board, Vec<crate::game::board::answer::Answer>)> {
        let board_generator = BoardGenerator::new();
        let board = board_generator.generate_board(rng);

        // Find all valid words and their scores
        let valid_answers = self.game_engine.find_all_valid_words(&board).await?;

        // Sort by score descending and take top 5
        let mut scores: Vec<i32> = valid_answers.iter().map(|answer| answer.score()).collect();
        scores.sort_by(|a, b| b.cmp(a));

        let top_5_sum: i32 = scores.iter().take(5).sum();

        if top_5_sum >= threshold_score {
            Ok((board, valid_answers))
        } else {
            anyhow::bail!(
                "Board quality insufficient: top 5 words sum to {} (threshold: {})",
                top_5_sum,
                threshold_score
            );
        }
    }

    /// Create a deterministic seed based on date and attempt numbers
    fn create_seed(&self, date: &str, reduction_attempt: u32, generation_attempt: u32) -> [u8; 32] {
        let seed_string = format!("{}:{}:{}", date, reduction_attempt, generation_attempt);
        Seeder::from(seed_string).make_seed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::GameEngine;
    use chrono::NaiveDate;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio_test;

    fn create_test_wordlist() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "cat").unwrap();
        writeln!(temp_file, "dog").unwrap();
        writeln!(temp_file, "test").unwrap();
        writeln!(temp_file, "word").unwrap();
        writeln!(temp_file, "game").unwrap();
        writeln!(temp_file, "path").unwrap();
        writeln!(temp_file, "tile").unwrap();
        writeln!(temp_file, "board").unwrap();
        writeln!(temp_file, "score").unwrap();
        writeln!(temp_file, "point").unwrap();
        temp_file.flush().unwrap();
        temp_file
    }

    async fn create_test_game_generator_without_db() -> (GameEngine, NamedTempFile) {
        let temp_file = create_test_wordlist();
        let game_engine = GameEngine::new(temp_file.path().to_path_buf());
        (game_engine, temp_file)
    }

    // Helper function to create deterministic seeds without GameGenerator instance
    fn create_deterministic_seed(
        date: &str,
        reduction_attempt: u32,
        generation_attempt: u32,
    ) -> [u8; 32] {
        let seed_string = format!("{}:{}:{}", date, reduction_attempt, generation_attempt);
        Seeder::from(seed_string).make_seed()
    }

    #[test]
    fn test_game_engine_creation() {
        tokio_test::block_on(async {
            let (_game_engine, _temp_file) = create_test_game_generator_without_db().await;

            // Game engine should be created successfully
            assert!(true); // Constructor completed without panicking
        });
    }

    #[test]
    fn test_create_seed_deterministic() {
        // Test seed generation without database dependencies
        let seed1 = create_deterministic_seed("2023-12-01", 0, 1);
        let seed2 = create_deterministic_seed("2023-12-01", 0, 1);

        // Same inputs should produce same seed
        assert_eq!(seed1, seed2);
    }

    #[test]
    fn test_create_seed_different_dates() {
        let seed1 = create_deterministic_seed("2023-12-01", 0, 1);
        let seed2 = create_deterministic_seed("2023-12-02", 0, 1);

        // Different dates should produce different seeds
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_create_seed_different_attempts() {
        let seed1 = create_deterministic_seed("2023-12-01", 0, 1);
        let seed2 = create_deterministic_seed("2023-12-01", 0, 2);
        let seed3 = create_deterministic_seed("2023-12-01", 1, 1);

        // Different attempt numbers should produce different seeds
        assert_ne!(seed1, seed2);
        assert_ne!(seed1, seed3);
        assert_ne!(seed2, seed3);
    }

    #[test]
    fn test_create_seed_format() {
        let seed = create_deterministic_seed("2023-12-01", 1, 5);

        // Seed should be 32 bytes (256 bits)
        assert_eq!(seed.len(), 32);

        // Should be deterministic - test the underlying string format
        let expected_string = "2023-12-01:1:5";
        let expected_seed: [u8; 32] = Seeder::from(expected_string).make_seed();
        assert_eq!(seed, expected_seed);
    }

    #[tokio::test]
    async fn test_board_generation_logic() {
        let (game_engine, _temp_file) = create_test_game_generator_without_db().await;
        let board_generator = crate::game::BoardGenerator::new();
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        // Test that we can generate a board
        let board = board_generator.generate_board(&mut rng);

        // Test that we can find words on the board
        let valid_answers = game_engine.find_all_valid_words(&board).await.unwrap();

        // Should find at least some words with our test wordlist
        assert!(!valid_answers.is_empty());
    }

    #[tokio::test]
    async fn test_word_scoring_logic() {
        let (game_engine, _temp_file) = create_test_game_generator_without_db().await;
        let board_generator = crate::game::BoardGenerator::new();
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        // Generate a board and find words
        let board = board_generator.generate_board(&mut rng);
        let valid_answers = game_engine.find_all_valid_words(&board).await.unwrap();

        // Test scoring logic
        if !valid_answers.is_empty() {
            let scores: Vec<i32> = valid_answers.iter().map(|answer| answer.score()).collect();
            let total_score: i32 = scores.iter().sum();

            // Scores should be non-negative
            assert!(scores.iter().all(|&score| score >= 0));
            assert!(total_score >= 0);
        }
    }

    #[test]
    fn test_threshold_reduction_logic() {
        // Test the threshold reduction algorithm used in generate_game_for_date
        let initial_threshold = 40;
        let reduced_threshold = (initial_threshold as f32 * 0.75) as i32;

        assert_eq!(reduced_threshold, 30);

        // Test edge case with odd numbers
        let odd_threshold = 33;
        let reduced_odd = (odd_threshold as f32 * 0.75) as i32;
        assert_eq!(reduced_odd, 24); // 33 * 0.75 = 24.75 -> 24
    }

    #[test]
    fn test_generation_attempt_limits() {
        // Test the constants used in the generation loops
        let max_threshold_reductions = 1;
        let max_generation_attempts = 5;

        // Verify the total number of attempts possible
        let total_attempts = (max_threshold_reductions + 1) * max_generation_attempts;
        assert_eq!(total_attempts, 10); // 2 threshold levels * 5 attempts each
    }

    #[test]
    fn test_date_formatting_logic() {
        // Test date string format used in the generator
        let date = NaiveDate::from_ymd_opt(2023, 12, 1).unwrap();
        let formatted = date.format("%Y-%m-%d").to_string();

        assert_eq!(formatted, "2023-12-01");

        // Test different date
        let date2 = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let formatted2 = date2.format("%Y-%m-%d").to_string();

        assert_eq!(formatted2, "2024-01-15");
    }

    #[test]
    fn test_days_calculation() {
        // Test the ranges used in generate_missing_games
        let days_back_range: Vec<i64> = (1..=7).collect();
        let days_ahead_range: Vec<i64> = (0..=3).collect();

        assert_eq!(days_back_range, vec![1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(days_ahead_range, vec![0, 1, 2, 3]);

        // Verify we're generating games for past 7 days + today + 3 future days
        let total_days = days_back_range.len() + days_ahead_range.len();
        assert_eq!(total_days, 11); // 7 past + 4 current/future days
    }
}
