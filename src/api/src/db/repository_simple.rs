use anyhow::Result;
use chrono::Utc;
use sqlx::{PgPool, Row};

use super::models::{
    DbGame, DbGameAnswer, DbGameEntry, DbOptimalSolution, DbUser, NewGame, NewGameAnswer, NewGameEntry, NewOptimalSolution, NewUser,
};

#[derive(Clone)]
pub struct Repository {
    pool: PgPool,
}

impl Repository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // User operations
    pub async fn create_user(&self, new_user: NewUser) -> Result<DbUser> {
        let user = DbUser::new(new_user.cookie_token);

        sqlx::query(
            "INSERT INTO users (id, cookie_token, created_at, last_seen) VALUES ($1, $2, $3, $4)",
        )
        .bind(&user.id)
        .bind(&user.cookie_token)
        .bind(user.created_at)
        .bind(user.last_seen)
        .execute(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_cookie(&self, cookie_token: &str) -> Result<Option<DbUser>> {
        let row = sqlx::query(
            "SELECT id, cookie_token, created_at, last_seen FROM users WHERE cookie_token = $1",
        )
        .bind(cookie_token)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbUser {
                id: row.get("id"),
                cookie_token: row.get("cookie_token"),
                created_at: row.get("created_at"),
                last_seen: row.get("last_seen"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<Option<DbUser>> {
        let row =
            sqlx::query("SELECT id, cookie_token, created_at, last_seen FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;

        if let Some(row) = row {
            Ok(Some(DbUser {
                id: row.get("id"),
                cookie_token: row.get("cookie_token"),
                created_at: row.get("created_at"),
                last_seen: row.get("last_seen"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_user_last_seen(&self, user_id: &str) -> Result<()> {
        let now = Utc::now();
        sqlx::query("UPDATE users SET last_seen = $1 WHERE id = $2")
            .bind(now)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_game_by_date(&self, date: &str) -> Result<Option<DbGame>> {
        let row = sqlx::query("SELECT id, date, board_data, threshold_score, sequence_number, created_at FROM games WHERE date = $1")
            .bind(date)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(DbGame {
                id: row.get("id"),
                date: row.get("date"),
                board_data: row.get("board_data"),
                threshold_score: row.get("threshold_score"),
                sequence_number: row.get("sequence_number"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_game_by_id(&self, game_id: &str) -> Result<Option<DbGame>> {
        let row = sqlx::query("SELECT id, date, board_data, threshold_score, sequence_number, created_at FROM games WHERE id = $1")
            .bind(game_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(DbGame {
                id: row.get("id"),
                date: row.get("date"),
                board_data: row.get("board_data"),
                threshold_score: row.get("threshold_score"),
                sequence_number: row.get("sequence_number"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_game_by_sequence_number(
        &self,
        sequence_number: i32,
    ) -> Result<Option<DbGame>> {
        let row = sqlx::query("SELECT id, date, board_data, threshold_score, sequence_number, created_at FROM games WHERE sequence_number = $1")
            .bind(sequence_number)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(DbGame {
                id: row.get("id"),
                date: row.get("date"),
                board_data: row.get("board_data"),
                threshold_score: row.get("threshold_score"),
                sequence_number: row.get("sequence_number"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn game_exists_for_date(&self, date: &str) -> Result<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM games WHERE date = $1")
            .bind(date)
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    pub async fn get_next_sequence_number(&self) -> Result<i32> {
        let row = sqlx::query("SELECT MAX(sequence_number) as max_seq FROM games")
            .fetch_one(&self.pool)
            .await?;

        let max_sequence: Option<i32> = row.get("max_seq");
        Ok(max_sequence.unwrap_or(0) + 1)
    }

    // Game entry operations
    pub async fn create_or_update_game_entry(
        &self,
        new_entry: NewGameEntry,
    ) -> Result<DbGameEntry> {
        // Check if entry already exists
        let existing = self
            .get_game_entry(&new_entry.user_id, &new_entry.game_id)
            .await?;

        if let Some(existing_entry) = existing {
            // Update existing entry
            let now = Utc::now();
            sqlx::query("UPDATE game_entries SET answers_data = $1, total_score = $2, completed = $3, updated_at = $4 WHERE id = $5")
                .bind(&new_entry.answers_data)
                .bind(new_entry.total_score)
                .bind(new_entry.completed)
                .bind(now)
                .bind(&existing_entry.id)
                .execute(&self.pool)
                .await?;

            Ok(DbGameEntry {
                id: existing_entry.id,
                user_id: new_entry.user_id,
                game_id: new_entry.game_id,
                answers_data: new_entry.answers_data,
                total_score: new_entry.total_score,
                completed: new_entry.completed,
                created_at: existing_entry.created_at,
                updated_at: now,
            })
        } else {
            // Create new entry
            let entry = DbGameEntry::new(
                new_entry.user_id,
                new_entry.game_id,
                new_entry.answers_data,
                new_entry.total_score,
                new_entry.completed,
            );

            sqlx::query("INSERT INTO game_entries (id, user_id, game_id, answers_data, total_score, completed, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
                .bind(&entry.id)
                .bind(&entry.user_id)
                .bind(&entry.game_id)
                .bind(&entry.answers_data)
                .bind(entry.total_score)
                .bind(entry.completed)
                .bind(entry.created_at)
                .bind(entry.updated_at)
                .execute(&self.pool)
                .await?;

            Ok(entry)
        }
    }

    pub async fn get_game_entry(
        &self,
        user_id: &str,
        game_id: &str,
    ) -> Result<Option<DbGameEntry>> {
        let row = sqlx::query("SELECT id, user_id, game_id, answers_data, total_score, completed, created_at, updated_at FROM game_entries WHERE user_id = $1 AND game_id = $2")
            .bind(user_id)
            .bind(game_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(DbGameEntry {
                id: row.get("id"),
                user_id: row.get("user_id"),
                game_id: row.get("game_id"),
                answers_data: row.get("answers_data"),
                total_score: row.get("total_score"),
                completed: row.get("completed"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    // Create game and answers atomically
    pub async fn create_game_with_answers(
        &self,
        new_game: NewGame,
        mut game_answers: Vec<NewGameAnswer>,
        optimal_solution: Option<NewOptimalSolution>,
    ) -> Result<(DbGame, Vec<DbGameAnswer>)> {
        let mut tx = self.pool.begin().await?;

        // Create the game first
        let game = DbGame::new(
            new_game.date,
            new_game.board_data,
            new_game.threshold_score,
            new_game.sequence_number,
        );

        sqlx::query("INSERT INTO games (id, date, board_data, threshold_score, sequence_number, created_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(&game.id)
            .bind(&game.date)
            .bind(&game.board_data)
            .bind(game.threshold_score)
            .bind(game.sequence_number)
            .bind(game.created_at)
            .execute(&mut *tx)
            .await?;

        // Update all game_answers to use the actual game ID
        for answer in &mut game_answers {
            answer.game_id = game.id.clone();
        }

        // Create all the game answers
        let mut created_answers = Vec::new();
        for new_answer in game_answers {
            let answer = DbGameAnswer::new(new_answer.game_id, new_answer.word);

            sqlx::query(
                "INSERT INTO game_answers (id, game_id, word, created_at) VALUES ($1, $2, $3, $4)",
            )
            .bind(&answer.id)
            .bind(&answer.game_id)
            .bind(&answer.word)
            .bind(answer.created_at)
            .execute(&mut *tx)
            .await?;

            created_answers.push(answer);
        }

        // Create the optimal solution if provided
        if let Some(mut optimal_sol) = optimal_solution {
            optimal_sol.game_id = game.id.clone();
            let solution = DbOptimalSolution::new(
                optimal_sol.game_id,
                optimal_sol.words_and_scores,
                optimal_sol.total_score,
            );

            sqlx::query(
                "INSERT INTO optimal_solutions (id, game_id, words_and_scores, total_score, created_at) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(&solution.id)
            .bind(&solution.game_id)
            .bind(&solution.words_and_scores)
            .bind(solution.total_score)
            .bind(solution.created_at)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok((game, created_answers))
    }

    pub async fn get_game_words(&self, game_id: &str) -> Result<Vec<String>> {
        let rows = sqlx::query("SELECT DISTINCT word FROM game_answers WHERE game_id = $1")
            .bind(game_id)
            .fetch_all(&self.pool)
            .await?;

        let words = rows
            .into_iter()
            .map(|row| row.get::<String, _>("word"))
            .collect();

        Ok(words)
    }

    // Statistics operations
    pub async fn get_game_stats(
        &self,
        game_id: &str,
        user_score: i32,
    ) -> Result<(i32, i32, f64, i32, i32)> {
        // Get total players for this game
        let total_row = sqlx::query(
            "SELECT COUNT(*) as count FROM game_entries WHERE game_id = $1 AND completed = TRUE",
        )
        .bind(game_id)
        .fetch_one(&self.pool)
        .await?;
        let total_players: i64 = total_row.get("count");

        // Get number of players with score <= user_score (for ranking)
        let rank_row = sqlx::query("SELECT COUNT(*) as count FROM game_entries WHERE game_id = $1 AND completed = TRUE AND total_score <= $2")
            .bind(game_id)
            .bind(user_score)
            .fetch_one(&self.pool)
            .await?;
        let players_at_or_below: i64 = rank_row.get("count");

        // Calculate rank and percentile
        let user_rank = (total_players - players_at_or_below + 1) as i32;
        let percentile = if total_players > 0 {
            (players_at_or_below as f64 / total_players as f64) * 100.0
        } else {
            0.0
        };

        // Get average score
        let avg_row = sqlx::query("SELECT AVG(total_score::float) as avg_score FROM game_entries WHERE game_id = $1 AND completed = TRUE")
            .bind(game_id)
            .fetch_one(&self.pool)
            .await?;
        let avg_score: Option<f64> = avg_row.get("avg_score");

        // Get highest score
        let max_row = sqlx::query("SELECT MAX(total_score) as max_score FROM game_entries WHERE game_id = $1 AND completed = TRUE")
            .bind(game_id)
            .fetch_one(&self.pool)
            .await?;
        let highest_score: Option<i32> = max_row.get("max_score");

        Ok((
            total_players as i32,
            user_rank,
            percentile,
            avg_score.unwrap_or(0.0) as i32,
            highest_score.unwrap_or(0),
        ))
    }
}

#[cfg(all(test, feature = "database-tests"))]
mod tests {
    use sqlx::{Pool, Postgres};

    use super::*;

    fn create_test_board_data() -> String {
        r#"{"rows":[{"tiles":[{"letter":"t","points":1,"is_wildcard":false,"row":0,"col":0},{"letter":"e","points":1,"is_wildcard":false,"row":0,"col":1},{"letter":"s","points":1,"is_wildcard":false,"row":0,"col":2},{"letter":"t","points":1,"is_wildcard":false,"row":0,"col":3}]}]}"#.to_string()
    }

    // Commented out tests for PostgreSQL migration - tests depend on SQLite setup
    #[sqlx::test]
    async fn test_get_game_by_sequence_number_exists(pool: Pool<Postgres>) {
        let repo = Repository::new(pool);

        // Create a test game
        let new_game = NewGame {
            date: "2025-06-08".to_string(),
            board_data: create_test_board_data(),
            threshold_score: 40,
            sequence_number: 1,
        };

        let (created_game, _) = repo
            .create_game_with_answers(new_game, vec![], None)
            .await
            .unwrap();

        // Test getting the game by sequence number
        let retrieved_game = repo.get_game_by_sequence_number(1).await.unwrap();

        assert!(retrieved_game.is_some());
        let game = retrieved_game.unwrap();
        assert_eq!(game.id, created_game.id);
        assert_eq!(game.sequence_number, 1);
        assert_eq!(game.date, "2025-06-08");
        assert_eq!(game.threshold_score, 40);
    }

    #[sqlx::test]
    async fn test_get_game_by_sequence_number_not_exists(pool: Pool<Postgres>) {
        let repo = Repository::new(pool);

        // Test getting a non-existent sequence number
        let result = repo.get_game_by_sequence_number(999).await.unwrap();
        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn test_get_game_by_sequence_number_multiple_games(pool: Pool<Postgres>) {
        let repo = Repository::new(pool);

        // Create multiple test games with different sequence numbers
        let games = vec![
            NewGame {
                date: "2025-06-08".to_string(),
                board_data: create_test_board_data(),
                threshold_score: 40,
                sequence_number: 1,
            },
            NewGame {
                date: "2025-06-07".to_string(),
                board_data: create_test_board_data(),
                threshold_score: 35,
                sequence_number: 2,
            },
            NewGame {
                date: "2025-06-06".to_string(),
                board_data: create_test_board_data(),
                threshold_score: 45,
                sequence_number: 5,
            },
        ];

        let mut created_games = Vec::new();
        for game in games {
            created_games.push(repo.create_game_with_answers(game, vec![], None).await.unwrap());
        }

        // Test getting each game by sequence number
        let game1 = repo.get_game_by_sequence_number(1).await.unwrap().unwrap();
        assert_eq!(game1.sequence_number, 1);
        assert_eq!(game1.date, "2025-06-08");
        assert_eq!(game1.threshold_score, 40);

        let game2 = repo.get_game_by_sequence_number(2).await.unwrap().unwrap();
        assert_eq!(game2.sequence_number, 2);
        assert_eq!(game2.date, "2025-06-07");
        assert_eq!(game2.threshold_score, 35);

        let game5 = repo.get_game_by_sequence_number(5).await.unwrap().unwrap();
        assert_eq!(game5.sequence_number, 5);
        assert_eq!(game5.date, "2025-06-06");
        assert_eq!(game5.threshold_score, 45);

        // Test getting non-existent sequence numbers
        assert!(repo.get_game_by_sequence_number(3).await.unwrap().is_none());
        assert!(repo.get_game_by_sequence_number(4).await.unwrap().is_none());
        assert!(repo.get_game_by_sequence_number(6).await.unwrap().is_none());
    }

    #[sqlx::test]
    async fn test_get_next_sequence_number(pool: Pool<Postgres>) {
        let repo = Repository::new(pool);

        // Test getting next sequence number when no games exist
        let next_seq = repo.get_next_sequence_number().await.unwrap();
        assert_eq!(next_seq, 1);

        // Create a game
        let new_game = NewGame {
            date: "2025-06-08".to_string(),
            board_data: create_test_board_data(),
            threshold_score: 40,
            sequence_number: 1,
        };
        repo.create_game_with_answers(new_game, vec![], None)
            .await
            .unwrap();

        // Test getting next sequence number after creating one game
        let next_seq = repo.get_next_sequence_number().await.unwrap();
        assert_eq!(next_seq, 2);

        // Create another game with sequence number 5
        let new_game = NewGame {
            date: "2025-06-07".to_string(),
            board_data: create_test_board_data(),
            threshold_score: 35,
            sequence_number: 5,
        };
        repo.create_game_with_answers(new_game, vec![], None)
            .await
            .unwrap();

        // Test getting next sequence number - should be max + 1
        let next_seq = repo.get_next_sequence_number().await.unwrap();
        assert_eq!(next_seq, 6);
    }

    #[sqlx::test]
    async fn test_game_exists_for_date(pool: Pool<Postgres>) {
        let repo = Repository::new(pool);

        // Test when no game exists for date
        let exists = repo.game_exists_for_date("2025-06-08").await.unwrap();
        assert!(!exists);

        // Create a game
        let new_game = NewGame {
            date: "2025-06-08".to_string(),
            board_data: create_test_board_data(),
            threshold_score: 40,
            sequence_number: 1,
        };
        repo.create_game_with_answers(new_game, vec![], None)
            .await
            .unwrap();

        // Test when game exists for date
        let exists = repo.game_exists_for_date("2025-06-08").await.unwrap();
        assert!(exists);

        // Test when game still doesn't exist for different date
        let exists = repo.game_exists_for_date("2025-06-07").await.unwrap();
        assert!(!exists);
    }

    #[sqlx::test]
    async fn test_get_game_by_date(pool: Pool<Postgres>) {
        let repo = Repository::new(pool);

        // Test getting non-existent game by date
        let result = repo.get_game_by_date("2025-06-08").await.unwrap();
        assert!(result.is_none());

        // Create a game
        let new_game = NewGame {
            date: "2025-06-08".to_string(),
            board_data: create_test_board_data(),
            threshold_score: 40,
            sequence_number: 1,
        };
        let (created_game, _) = repo
            .create_game_with_answers(new_game, vec![], None)
            .await
            .unwrap();

        // Test getting existing game by date
        let retrieved_game = repo.get_game_by_date("2025-06-08").await.unwrap();
        assert!(retrieved_game.is_some());
        let game = retrieved_game.unwrap();
        assert_eq!(game.id, created_game.id);
        assert_eq!(game.date, "2025-06-08");
        assert_eq!(game.sequence_number, 1);
    }
}
