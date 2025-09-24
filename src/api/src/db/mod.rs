pub mod conversions;
pub mod models;
pub mod repository;
pub mod repository_postgres;
pub mod repository_sqlite;
pub mod storage_types;

pub use models::OptimalAnswer;
pub use repository::Repository;
pub use repository_postgres::PgRepository;
pub use repository_sqlite::SqliteRepository;

use anyhow::Result;
use sqlx::postgres::PgPool;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};

pub async fn setup_database(
    postgres_database_url: &str,
    sqlite_database_url: &str,
) -> Result<(PgPool, SqlitePool)> {
    // Connect to PostgreSQL database
    let pg_pool = PgPool::connect_lazy(postgres_database_url)?;

    // // Run migrations
    // run_migrations_postgres(&pg_pool).await?;

    if !Sqlite::database_exists(sqlite_database_url)
        .await
        .unwrap_or(false)
    {
        Sqlite::create_database(sqlite_database_url).await?;
    }

    let sqlite_pool = SqlitePool::connect(sqlite_database_url).await?;

    run_migrations_sqlite(&sqlite_pool).await?;

    Ok((pg_pool, sqlite_pool))
}

#[allow(unused)]
async fn run_migrations_postgres(pool: &PgPool) -> Result<()> {
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
            include_str!("../../migrations/postgres/001_initial.sql"),
        ),
        (
            "002_add_sequence_number.sql",
            include_str!("../../migrations/postgres/002_add_sequence_number.sql"),
        ),
        (
            "003_add_game_answers.sql",
            include_str!("../../migrations/postgres/003_add_game_answers.sql"),
        ),
        (
            "004_remove_paths_from_game_answers.sql",
            include_str!("../../migrations/postgres/004_remove_paths_from_game_answers.sql"),
        ),
        (
            "005_add_optimal_solutions.sql",
            include_str!("../../migrations/postgres/005_add_optimal_solutions.sql"),
        ),
        (
            "006_add_game_completion_fields.sql",
            include_str!("../../migrations/postgres/006_add_game_completion_fields.sql"),
        ),
        (
            "007_change_completed_at_to_timestamp_tz.sql",
            include_str!("../../migrations/postgres/007_change_completed_at_to_timestamp_tz.sql"),
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

    tracing::info!("Postgres database migrations completed");
    Ok(())
}

async fn run_migrations_sqlite(pool: &SqlitePool) -> Result<()> {
    // Create migrations table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT UNIQUE NOT NULL,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    "#,
    )
    .execute(pool)
    .await?;

    // List of migrations in order
    let migrations = [(
        "001_migrate_from_postgres.sql",
        include_str!("../../migrations/sqlite/001_migrate_from_postgres.sql"),
    )];

    for (filename, migration_sql) in &migrations {
        // Check if this migration has already been applied
        let applied = sqlx::query("SELECT filename FROM migrations WHERE filename = ?")
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
        sqlx::query("INSERT INTO migrations (filename) VALUES (?)")
            .bind(filename)
            .execute(pool)
            .await?;

        tracing::info!("Migration {} completed successfully", filename);
    }

    tracing::info!("SQLite database migrations completed");
    Ok(())
}
