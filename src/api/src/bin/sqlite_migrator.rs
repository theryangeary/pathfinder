use anyhow::Result;
use chrono::{DateTime, Utc};
use pathfinder::db::{setup_database, PgRepository, SqliteRepository};
use sqlx::{PgPool, SqlitePool, Row};
use tracing::info;
use std::{collections::HashMap, env};

#[derive(Debug)]
struct MigrationStats {
    users: usize,
    games: usize,
    game_entries: usize,
    game_answers: usize,
    optimal_solutions: usize,
}

pub struct DataMigrator {
    pg_pool: PgPool,
    sqlite_pool: SqlitePool,
}

impl DataMigrator {
    pub fn new(pg_pool: PgPool, sqlite_pool: SqlitePool) -> Self {
        Self { pg_pool, sqlite_pool }
    }

    pub async fn migrate_all_data(&self) -> Result<MigrationStats> {
        println!("Starting data migration from PostgreSQL to SQLite...");

        // Enable foreign keys in SQLite
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&self.sqlite_pool)
            .await?;

        let mut stats = MigrationStats {
            users: 0,
            games: 0,
            game_entries: 0,
            game_answers: 0,
            optimal_solutions: 0,
        };

        // Migrate in dependency order
        stats.users = self.migrate_users().await?;
        stats.games = self.migrate_games().await?;
        stats.game_answers = self.migrate_game_answers().await?;
        stats.optimal_solutions = self.migrate_optimal_solutions().await?;
        stats.game_entries = self.migrate_game_entries().await?;

        println!("Migration completed successfully!");
        println!("Stats: {:#?}", stats);

        Ok(stats)
    }

    async fn migrate_users(&self) -> Result<usize> {
        println!("Migrating users...");

        let rows = sqlx::query("SELECT id, cookie_token, created_at, last_seen FROM users ORDER BY created_at")
            .fetch_all(&self.pg_pool)
            .await?;

        let mut count = 0;
        for row in rows {
            let id: String = row.get("id");
            let cookie_token: String = row.get("cookie_token");
            let created_at: DateTime<Utc> = row.get("created_at");
            let last_seen: DateTime<Utc> = row.get("last_seen");

            sqlx::query("INSERT INTO users (id, cookie_token, created_at, last_seen) VALUES (?1, ?2, ?3, ?4)")
                .bind(&id)
                .bind(&cookie_token)
                .bind(created_at.to_rfc3339())
                .bind(last_seen.to_rfc3339())
                .execute(&self.sqlite_pool)
                .await?;

            count += 1;
        }

        println!("Migrated {} users", count);
        Ok(count)
    }

    async fn migrate_games(&self) -> Result<usize> {
        println!("Migrating games...");

        let rows = sqlx::query("SELECT id, date, board_data, threshold_score, sequence_number, completed, completed_at, created_at FROM games ORDER BY sequence_number")
            .fetch_all(&self.pg_pool)
            .await?;

        let mut count = 0;
        for row in rows {
            let id: String = row.get("id");
            let date: String = row.get("date");
            let board_data: String = row.get("board_data");
            let threshold_score: i32 = row.get("threshold_score");
            let sequence_number: i32 = row.get("sequence_number");
            let completed: bool = row.get("completed");
            let completed_at: Option<DateTime<Utc>> = row.get("completed_at");
            let created_at: DateTime<Utc> = row.get("created_at");

            sqlx::query("INSERT INTO games (id, date, board_data, threshold_score, sequence_number, completed, completed_at, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")
                .bind(&id)
                .bind(&date)
                .bind(&board_data)
                .bind(threshold_score)
                .bind(sequence_number)
                .bind(if completed { 1 } else { 0 })
                .bind(completed_at.map(|dt| dt.to_rfc3339()))
                .bind(created_at.to_rfc3339())
                .execute(&self.sqlite_pool)
                .await?;

            count += 1;
        }

        println!("Migrated {} games", count);
        Ok(count)
    }

    async fn migrate_game_answers(&self) -> Result<usize> {
        println!("Migrating game answers...");

        let rows = sqlx::query("SELECT id, game_id, word, created_at FROM game_answers ORDER BY game_id, word")
            .fetch_all(&self.pg_pool)
            .await?;

        let mut count = 0;
        for row in rows {
            let id: String = row.get("id");
            let game_id: String = row.get("game_id");
            let word: String = row.get("word");
            let created_at: DateTime<Utc> = row.get("created_at");

            sqlx::query("INSERT INTO game_answers (id, game_id, word, created_at) VALUES (?1, ?2, ?3, ?4)")
                .bind(&id)
                .bind(&game_id)
                .bind(&word)
                .bind(created_at.to_rfc3339())
                .execute(&self.sqlite_pool)
                .await?;

            count += 1;
        }

        println!("Migrated {} game answers", count);
        Ok(count)
    }

    async fn migrate_optimal_solutions(&self) -> Result<usize> {
        println!("Migrating optimal solutions...");

        let rows = sqlx::query("SELECT id, game_id, words_and_scores, total_score, created_at FROM optimal_solutions ORDER BY game_id")
            .fetch_all(&self.pg_pool)
            .await?;

        let mut count = 0;
        for row in rows {
            let id: String = row.get("id");
            let game_id: String = row.get("game_id");
            let words_and_scores: String = row.get("words_and_scores");
            let total_score: i32 = row.get("total_score");
            let created_at: DateTime<Utc> = row.get("created_at");

            sqlx::query("INSERT INTO optimal_solutions (id, game_id, words_and_scores, total_score, created_at) VALUES (?1, ?2, ?3, ?4, ?5)")
                .bind(&id)
                .bind(&game_id)
                .bind(&words_and_scores)
                .bind(total_score)
                .bind(created_at.to_rfc3339())
                .execute(&self.sqlite_pool)
                .await?;

            count += 1;
        }

        println!("Migrated {} optimal solutions", count);
        Ok(count)
    }

    async fn migrate_game_entries(&self) -> Result<usize> {
        println!("Migrating game entries...");

        let rows = sqlx::query("SELECT id, user_id, game_id, answers_data, total_score, completed, created_at, updated_at FROM game_entries ORDER BY created_at")
            .fetch_all(&self.pg_pool)
            .await?;

        let mut count = 0;
        for row in rows {
            let id: String = row.get("id");
            let user_id: String = row.get("user_id");
            let game_id: String = row.get("game_id");
            let answers_data: String = row.get("answers_data");
            let total_score: i32 = row.get("total_score");
            let completed: bool = row.get("completed");
            let created_at: DateTime<Utc> = row.get("created_at");
            let updated_at: DateTime<Utc> = row.get("updated_at");

            sqlx::query("INSERT INTO game_entries (id, user_id, game_id, answers_data, total_score, completed, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")
                .bind(&id)
                .bind(&user_id)
                .bind(&game_id)
                .bind(&answers_data)
                .bind(total_score)
                .bind(if completed { 1 } else { 0 })
                .bind(created_at.to_rfc3339())
                .bind(updated_at.to_rfc3339())
                .execute(&self.sqlite_pool)
                .await?;

            count += 1;
        }

        println!("Migrated {} game entries", count);
        Ok(count)
    }

    pub async fn verify_migration(&self) -> Result<()> {
        println!("Verifying migration...");

        // Compare row counts
        let tables = vec!["users", "games", "game_entries", "game_answers", "optimal_solutions"];

        for table in tables {
            let pg_count: i64 = sqlx::query(&format!("SELECT COUNT(*) as count FROM {}", table))
                .fetch_one(&self.pg_pool)
                .await?
                .get("count");

            let sqlite_count: i32 = sqlx::query(&format!("SELECT COUNT(*) as count FROM {}", table))
                .fetch_one(&self.sqlite_pool)
                .await?
                .get("count");

            if pg_count != sqlite_count as i64 {
                anyhow::bail!("Row count mismatch for table {}: PostgreSQL={}, SQLite={}", table, pg_count, sqlite_count);
            }

            println!("✓ {}: {} rows", table, pg_count);
        }

        // Verify some sample data integrity
        self.verify_sample_data().await?;

        println!("✓ Migration verification completed successfully!");
        Ok(())
    }

    async fn verify_sample_data(&self) -> Result<()> {
        // Check a sample user
        if let Some(pg_row) = sqlx::query("SELECT id, cookie_token FROM users LIMIT 1")
            .fetch_optional(&self.pg_pool)
            .await?
        {
            let user_id: String = pg_row.get("id");
            let pg_token: String = pg_row.get("cookie_token");

            let sqlite_row = sqlx::query("SELECT cookie_token FROM users WHERE id = ?1")
                .bind(&user_id)
                .fetch_one(&self.sqlite_pool)
                .await?;
            let sqlite_token: String = sqlite_row.get("cookie_token");

            if pg_token != sqlite_token {
                anyhow::bail!("User data mismatch for user {}", user_id);
            }
        }

        // Check a sample game
        if let Some(pg_row) = sqlx::query("SELECT id, date, sequence_number FROM games LIMIT 1")
            .fetch_optional(&self.pg_pool)
            .await?
        {
            let game_id: String = pg_row.get("id");
            let pg_date: String = pg_row.get("date");
            let pg_seq: i32 = pg_row.get("sequence_number");

            let sqlite_row = sqlx::query("SELECT date, sequence_number FROM games WHERE id = ?1")
                .bind(&game_id)
                .fetch_one(&self.sqlite_pool)
                .await?;
            let sqlite_date: String = sqlite_row.get("date");
            let sqlite_seq: i32 = sqlite_row.get("sequence_number");

            if pg_date != sqlite_date || pg_seq != sqlite_seq {
                anyhow::bail!("Game data mismatch for game {}", game_id);
            }
        }

        Ok(())
    }
}

// Example usage function
pub async fn run_migration() -> Result<()> {
    let postgres_database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/pathfinder".to_string());

    let sqlite_database_url =
        env::var("SQLITE_DATABASE_URL").unwrap_or_else(|_| "sqlite://pathfinder.db".to_string());

    // Setup database
    info!("Setting up database connection");
    let pool = setup_database(&postgres_database_url, &sqlite_database_url).await?;

    let migrator = DataMigrator::new(pool.0, pool.1);
    
    // Run the migration
    let stats = migrator.migrate_all_data().await?;
    
    // Verify the migration
    migrator.verify_migration().await?;
    
    println!("Migration completed successfully with stats: {:#?}", stats);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    run_migration().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migration() {
        // This would be a full integration test
        // You'd need both databases set up for testing
    }
}
