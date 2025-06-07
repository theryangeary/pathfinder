use crate::game::{GameEngine, BoardGenerator};
use crate::db::{Repository, models::NewGame};
use anyhow::Result;
use chrono::{DateTime, Utc, Duration, NaiveDate};
use rand::SeedableRng;
use rand_seeder::Seeder;
use tracing::{info, warn, error};
use serde_json;

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

    /// Generate games for the next 3 days if they don't already exist
    pub async fn generate_missing_games(&self) -> Result<()> {
        let today = Utc::now().date_naive();
        
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
                
                match self.try_generate_valid_board(&mut rng, threshold_score).await {
                    Ok(board_data) => {
                        let new_game = NewGame {
                            date: date.to_string(),
                            board_data,
                            threshold_score,
                        };
                        
                        let game = self.repository.create_game(new_game).await?;
                        info!(
                            "Successfully generated game for {} after {} attempts with threshold {}",
                            date, generation_attempt, threshold_score
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
    ) -> Result<String> {
        let board_generator = BoardGenerator::new();
        let board = board_generator.generate_board(rng);
        
        // Find all valid words and their scores
        let valid_answers = self.game_engine.find_all_valid_words(&board).await?;
        
        // Sort by score descending and take top 5
        let mut scores: Vec<i32> = valid_answers.iter().map(|answer| answer.score()).collect();
        scores.sort_by(|a, b| b.cmp(a));
        
        let top_5_sum: i32 = scores.iter().take(5).sum();
        
        if top_5_sum >= threshold_score {
            // Convert board to JSON for storage
            let serializable_board = crate::game::conversion::SerializableBoard::from(&board);
            let board_json = serde_json::to_string(&serializable_board)?;
            Ok(board_json)
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

