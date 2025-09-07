use anyhow::Result;
use axum::async_trait;

use super::models::{
    DbGame, DbGameAnswer, DbGameEntry, DbUser, NewGame, NewGameAnswer, NewGameEntry,
    NewOptimalSolution, NewUser, OptimalAnswer,
};

#[async_trait]
pub trait Repository {
    // User operations
    async fn create_user(&self, new_user: NewUser) -> Result<DbUser>;

    async fn get_user_by_cookie(&self, cookie_token: &str) -> Result<Option<DbUser>>;

    async fn get_user_by_id(&self, user_id: &str) -> Result<Option<DbUser>>;

    async fn update_user_last_seen(&self, user_id: &str) -> Result<()>;

    async fn get_game_by_date(&self, date: &str) -> Result<Option<DbGame>>;

    async fn get_game_by_id(&self, game_id: &str) -> Result<Option<DbGame>>;

    async fn get_game_by_sequence_number(&self, sequence_number: i32) -> Result<Option<DbGame>>;

    async fn game_exists_for_date(&self, date: &str) -> Result<bool>;

    async fn get_next_sequence_number(&self) -> Result<i32>;

    // Game entry operations
    async fn create_or_update_game_entry(&self, new_entry: NewGameEntry) -> Result<DbGameEntry>;

    async fn get_game_entry(&self, user_id: &str, game_id: &str) -> Result<Option<DbGameEntry>>;

    // Create game and answers atomically
    async fn create_game_with_answers(
        &self,
        new_game: NewGame,
        mut game_answers: Vec<NewGameAnswer>,
        optimal_solution: Option<NewOptimalSolution>,
    ) -> Result<(DbGame, Vec<DbGameAnswer>)>;

    async fn get_game_words(&self, game_id: &str) -> Result<Vec<String>>;

    // Get score distribution for a specific game
    async fn get_score_distribution(&self, game_id: &str) -> Result<Vec<i32>>;

    // Get optimal solutions for a specific game
    async fn get_optimal_solutions(&self, game_id: &str) -> Result<Vec<OptimalAnswer>>;

    // Completion tracking operations
    async fn mark_game_completed(&self, game_id: &str) -> Result<()>;

    async fn get_incomplete_games_for_date(&self, date: &str) -> Result<Vec<DbGame>>;

    async fn get_incomplete_game_entries_for_game(&self, game_id: &str)
        -> Result<Vec<DbGameEntry>>;

    async fn mark_game_entry_completed(&self, entry_id: &str) -> Result<()>;

    // Statistics operations
    async fn get_game_stats(
        &self,
        game_id: &str,
        user_score: i32,
    ) -> Result<(i32, i32, f64, i32, i32)>;
}
