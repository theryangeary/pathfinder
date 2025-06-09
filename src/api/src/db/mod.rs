pub mod models;
pub mod repository_simple;
pub mod storage_types;
pub mod conversions;

pub use repository_simple::Repository;

use sqlx::{postgres::PgPool, migrate::MigrateDatabase, Postgres};
use anyhow::Result;

pub async fn setup_database(database_url: &str) -> Result<PgPool> {
    // Note: PostgreSQL database should be created externally
    // We don't auto-create PostgreSQL databases like we did with SQLite
    
    let pool = PgPool::connect(database_url).await?;
    
    // Run migrations
    run_migrations(&pool).await?;
    
    Ok(pool)
}

async fn run_migrations(pool: &PgPool) -> Result<()> {
    // Create migrations table to track applied migrations
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY,
            filename TEXT UNIQUE NOT NULL,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    "#).execute(pool).await?;
    
    // List of migrations in order
    let migrations = [
        ("001_initial.sql", include_str!("../../migrations/001_initial.sql")),
        ("002_add_sequence_number.sql", include_str!("../../migrations/002_add_sequence_number.sql")),
    ];
    
    for (filename, migration_sql) in &migrations {
        // Check if this migration has already been applied
        let applied = sqlx::query("SELECT filename FROM migrations WHERE filename = $1")
            .bind(filename)
            .fetch_optional(pool)
            .await?;
        
        if applied.is_some() {
            tracing::info!("Migration {} already applied, skipping", filename);
            continue;
        }
        
        tracing::info!("Running migration: {}", filename);
        
        // Split by semicolon and execute each statement
        for statement in migration_sql.split(';') {
            let statement = statement.trim();
            if !statement.is_empty() {
                // Use execute and ignore "table already exists" errors for CREATE TABLE statements
                if statement.to_uppercase().starts_with("CREATE TABLE") {
                    let _ = sqlx::query(statement).execute(pool).await;
                } else {
                    sqlx::query(statement).execute(pool).await?;
                }
            }
        }
        
        // Record that this migration was applied
        sqlx::query("INSERT INTO migrations (filename) VALUES ($1)")
            .bind(filename)
            .execute(pool)
            .await?;
        
        tracing::info!("Migration {} completed successfully", filename);
    }
    
    tracing::info!("All database migrations completed");
    Ok(())
}