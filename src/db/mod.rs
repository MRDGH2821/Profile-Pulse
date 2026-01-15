//! Database module for Profile Pulse
//!
//! Handles SQLite database connections, migrations, and provides
//! repositories for data access.

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;
use tracing::info;

pub mod models;
pub mod repository;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to the SQLite database file
    pub path: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Enable WAL mode for better concurrency
    pub enable_wal: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: "profile-pulse.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: 30,
            enable_wal: true,
        }
    }
}

/// Initialize database connection pool
pub async fn init_pool(config: &DatabaseConfig) -> Result<SqlitePool> {
    info!("Initializing database connection pool");

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(&config.path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).context("Failed to create database directory")?;
        }
    }

    // Configure SQLite connection options
    let mut options = SqliteConnectOptions::from_str(&format!("sqlite:{}", config.path))?
        .create_if_missing(true)
        .journal_mode(if config.enable_wal {
            SqliteJournalMode::Wal
        } else {
            SqliteJournalMode::Delete
        })
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(30));

    // Enable foreign keys
    options = options.foreign_keys(true);

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.connect_timeout))
        .connect_with(options)
        .await
        .context("Failed to create database connection pool")?;

    info!("Database connection pool initialized");

    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    info!("Running database migrations");

    // Read and execute the initial migration
    let migration_sql_1 = include_str!("migrations/20250113_001_initial_schema.sql");

    sqlx::query(migration_sql_1)
        .execute(pool)
        .await
        .context("Failed to run migration 1")?;

    // Read and execute the URLs table migration
    let migration_sql_2 = include_str!("migrations/20250114_002_add_urls_table.sql");

    sqlx::query(migration_sql_2)
        .execute(pool)
        .await
        .context("Failed to run migration 2")?;

    // Read and execute the structured fields migration
    let migration_sql_3 = include_str!("migrations/20250115_001_add_structured_fields.sql");

    sqlx::query(migration_sql_3)
        .execute(pool)
        .await
        .context("Failed to run migration 3")?;

    info!("Database migrations completed");
    Ok(())
}

/// Check database health
pub async fn health_check(pool: &SqlitePool) -> Result<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .context("Database health check failed")?;
    Ok(())
}

/// Get database statistics
pub async fn get_stats(pool: &SqlitePool) -> Result<DatabaseStats> {
    let contact_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM contacts")
        .fetch_one(pool)
        .await?;

    let profile_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM contact_urls")
        .fetch_one(pool)
        .await?;

    let cache_size: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM fetch_cache")
        .fetch_one(pool)
        .await?;

    Ok(DatabaseStats {
        contact_count,
        profile_count,
        cache_size,
    })
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub contact_count: i64,
    pub profile_count: i64,
    pub cache_size: i64,
}

/// Vacuum the database to reclaim space
pub async fn vacuum(pool: &SqlitePool) -> Result<()> {
    info!("Running database VACUUM");
    sqlx::query("VACUUM")
        .execute(pool)
        .await
        .context("Failed to vacuum database")?;
    info!("Database VACUUM completed");
    Ok(())
}

/// Clean expired cache entries
pub async fn clean_expired_cache(pool: &SqlitePool) -> Result<u64> {
    let now = chrono::Utc::now().timestamp();

    let result = sqlx::query("DELETE FROM fetch_cache WHERE expires_at < ?")
        .bind(now)
        .execute(pool)
        .await?;

    let deleted = result.rows_affected();
    if deleted > 0 {
        info!("Cleaned {} expired cache entries", deleted);
    }

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_pool() {
        let config = DatabaseConfig {
            path: ":memory:".to_string(),
            ..Default::default()
        };

        let pool = init_pool(&config).await.unwrap();
        assert!(health_check(&pool).await.is_ok());
    }

    #[tokio::test]
    async fn test_run_migrations() {
        let config = DatabaseConfig {
            path: ":memory:".to_string(),
            ..Default::default()
        };

        let pool = init_pool(&config).await.unwrap();
        run_migrations(&pool).await.unwrap();

        // Verify tables exist
        let result: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='contacts'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(result, 1);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let config = DatabaseConfig {
            path: ":memory:".to_string(),
            ..Default::default()
        };

        let pool = init_pool(&config).await.unwrap();
        run_migrations(&pool).await.unwrap();

        let stats = get_stats(&pool).await.unwrap();
        assert_eq!(stats.contact_count, 0);
        assert_eq!(stats.profile_count, 0);
    }
}
