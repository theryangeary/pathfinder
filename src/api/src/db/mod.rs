pub mod conversions;
pub mod models;
pub mod repository_simple;
pub mod storage_types;

pub use models::OptimalAnswer;
pub use repository_simple::Repository;

use anyhow::Result;
use sqlx::postgres::PgPool;

pub async fn setup_database(database_url: &str) -> Result<PgPool> {
    // Connect to PostgreSQL database
    let pool = PgPool::connect(database_url).await?;

    // Run migrations
    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &PgPool) -> Result<()> {
    // Create migrations table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS migrations (
            id SERIAL PRIMARY KEY,
            filename TEXT UNIQUE NOT NULL,
            applied_at TIMESTAMP DEFAULT NOW()
        )
    "#,
    )
    .execute(pool)
    .await?;

    // List of migrations in order
    let migrations = [
        (
            "001_initial.sql",
            include_str!("../../migrations/001_initial.sql"),
        ),
        (
            "002_add_sequence_number.sql",
            include_str!("../../migrations/002_add_sequence_number.sql"),
        ),
        (
            "003_add_game_answers.sql",
            include_str!("../../migrations/003_add_game_answers.sql"),
        ),
        (
            "004_remove_paths_from_game_answers.sql",
            include_str!("../../migrations/004_remove_paths_from_game_answers.sql"),
        ),
        (
            "005_add_optimal_solutions.sql",
            include_str!("../../migrations/005_add_optimal_solutions.sql"),
        ),
        (
            "006_add_game_completion_fields.sql",
            include_str!("../../migrations/006_add_game_completion_fields.sql"),
        ),
        (
            "007_change_completed_at_to_timestamp_tz.sql",
            include_str!("../../migrations/007_change_completed_at_to_timestamp_tz.sql"),
        ),
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
