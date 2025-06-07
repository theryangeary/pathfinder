use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{SqlitePool, Row};
use uuid::Uuid;

use super::models::{DbUser, DbGame, DbGameEntry, NewUser, NewGame, NewGameEntry};

#[derive(Clone)]
pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // User operations
    pub async fn create_user(&self, new_user: NewUser) -> Result<DbUser> {
        let user = DbUser::new(new_user.cookie_token);
        
        sqlx::query("INSERT INTO users (id, cookie_token, created_at, last_seen) VALUES (?, ?, ?, ?)")
            .bind(&user.id)
            .bind(&user.cookie_token)
            .bind(&user.created_at)
            .bind(&user.last_seen)
            .execute(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn get_user_by_cookie(&self, cookie_token: &str) -> Result<Option<DbUser>> {
        let row = sqlx::query("SELECT id, cookie_token, created_at, last_seen FROM users WHERE cookie_token = ?")
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

    pub async fn update_user_last_seen(&self, user_id: &str) -> Result<()> {
        let now = Utc::now();
        sqlx::query("UPDATE users SET last_seen = ? WHERE id = ?")
            .bind(now)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Game operations
    pub async fn create_game(&self, new_game: NewGame) -> Result<DbGame> {
        let game = DbGame::new(new_game.date, new_game.board_data, new_game.threshold_score);
        
        sqlx::query("INSERT INTO games (id, date, board_data, threshold_score, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind(&game.id)
            .bind(&game.date)
            .bind(&game.board_data)
            .bind(&game.threshold_score)
            .bind(&game.created_at)
            .execute(&self.pool)
            .await?;

        Ok(game)
    }

    pub async fn get_game_by_date(&self, date: &str) -> Result<Option<DbGame>> {
        let row = sqlx::query("SELECT id, date, board_data, threshold_score, created_at FROM games WHERE date = ?")
            .bind(date)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(DbGame {
                id: row.get("id"),
                date: row.get("date"),
                board_data: row.get("board_data"),
                threshold_score: row.get("threshold_score"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_game_by_id(&self, game_id: &str) -> Result<Option<DbGame>> {
        let row = sqlx::query("SELECT id, date, board_data, threshold_score, created_at FROM games WHERE id = ?")
            .bind(game_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(DbGame {
                id: row.get("id"),
                date: row.get("date"),
                board_data: row.get("board_data"),
                threshold_score: row.get("threshold_score"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_random_historical_game(&self) -> Result<Option<DbGame>> {
        let row = sqlx::query("SELECT id, date, board_data, threshold_score, created_at FROM games ORDER BY RANDOM() LIMIT 1")
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(DbGame {
                id: row.get("id"),
                date: row.get("date"),
                board_data: row.get("board_data"),
                threshold_score: row.get("threshold_score"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn game_exists_for_date(&self, date: &str) -> Result<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM games WHERE date = ?")
            .bind(date)
            .fetch_one(&self.pool)
            .await?;
        
        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    // Game entry operations
    pub async fn create_or_update_game_entry(&self, new_entry: NewGameEntry) -> Result<DbGameEntry> {
        // Check if entry already exists
        let existing = self.get_game_entry(&new_entry.user_id, &new_entry.game_id).await?;
        
        if let Some(existing_entry) = existing {
            // Update existing entry
            let now = Utc::now();
            sqlx::query("UPDATE game_entries SET answers_data = ?, total_score = ?, completed = ?, updated_at = ? WHERE id = ?")
                .bind(&new_entry.answers_data)
                .bind(&new_entry.total_score)
                .bind(&new_entry.completed)
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

            sqlx::query("INSERT INTO game_entries (id, user_id, game_id, answers_data, total_score, completed, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&entry.id)
                .bind(&entry.user_id)
                .bind(&entry.game_id)
                .bind(&entry.answers_data)
                .bind(&entry.total_score)
                .bind(&entry.completed)
                .bind(&entry.created_at)
                .bind(&entry.updated_at)
                .execute(&self.pool)
                .await?;

            Ok(entry)
        }
    }

    pub async fn get_game_entry(&self, user_id: &str, game_id: &str) -> Result<Option<DbGameEntry>> {
        let row = sqlx::query("SELECT id, user_id, game_id, answers_data, total_score, completed, created_at, updated_at FROM game_entries WHERE user_id = ? AND game_id = ?")
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

    // Statistics operations
    pub async fn get_game_stats(&self, game_id: &str, user_score: i32) -> Result<(i32, i32, f64, i32, i32)> {
        // Get total players for this game
        let total_row = sqlx::query("SELECT COUNT(*) as count FROM game_entries WHERE game_id = ? AND completed = TRUE")
            .bind(game_id)
            .fetch_one(&self.pool)
            .await?;
        let total_players: i64 = total_row.get("count");

        // Get number of players with score <= user_score (for ranking)
        let rank_row = sqlx::query("SELECT COUNT(*) as count FROM game_entries WHERE game_id = ? AND completed = TRUE AND total_score <= ?")
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
        let avg_row = sqlx::query("SELECT AVG(CAST(total_score AS REAL)) as avg_score FROM game_entries WHERE game_id = ? AND completed = TRUE")
            .bind(game_id)
            .fetch_one(&self.pool)
            .await?;
        let avg_score: Option<f64> = avg_row.get("avg_score");

        // Get highest score
        let max_row = sqlx::query("SELECT MAX(total_score) as max_score FROM game_entries WHERE game_id = ? AND completed = TRUE")
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