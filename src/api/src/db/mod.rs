pub mod models;
pub mod repository_simple;
pub mod storage_types;
pub mod conversions;

pub use repository_simple::Repository;

use sqlx::{sqlite::SqlitePool, migrate::MigrateDatabase, Sqlite};
use anyhow::Result;

pub async fn setup_database(database_url: &str) -> Result<SqlitePool> {
    // Create database if it doesn't exist
    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        Sqlite::create_database(database_url).await?;
        tracing::info!("Database created successfully");
    }

    let pool = SqlitePool::connect(database_url).await?;
    
    // Run migrations
    run_migrations(&pool).await?;
    
    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let migration_sql = include_str!("../../migrations/001_initial.sql");
    
    // Check if migrations have already been run
    let table_exists = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='users'")
        .fetch_optional(pool)
        .await?;
    
    if table_exists.is_some() {
        tracing::info!("Database migrations already completed");
        return Ok(());
    }
    
    // Split by semicolon and execute each statement
    for statement in migration_sql.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() {
            sqlx::query(statement).execute(pool).await?;
        }
    }
    
    tracing::info!("Database migrations completed");
    Ok(())
}