//! Profile Pulse - Desktop contact management with social media integration
//!
//! This application helps manage contacts and automatically syncs profile
//! pictures from various social media platforms.

use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod core;
mod db;
mod discovery;
mod social;
mod ui;
mod utils;

use db::{init_pool, run_migrations, DatabaseConfig};

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Application data directory
    pub data_dir: PathBuf,
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Enable debug mode
    pub debug: bool,
}

impl AppConfig {
    /// Create a new application configuration
    pub fn new() -> Result<Self> {
        // Get project directories
        let proj_dirs = ProjectDirs::from("com", "profile-pulse", "Profile Pulse")
            .context("Failed to determine project directories")?;

        let data_dir = proj_dirs.data_dir().to_path_buf();
        let cache_dir = proj_dirs.cache_dir().to_path_buf();

        // Create directories if they don't exist
        std::fs::create_dir_all(&data_dir).context("Failed to create data directory")?;
        std::fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        // Database path
        let db_path = data_dir.join("profile-pulse.db");

        Ok(Self {
            database: DatabaseConfig {
                path: db_path.to_string_lossy().to_string(),
                max_connections: 10,
                min_connections: 1,
                connect_timeout: 30,
                enable_wal: true,
            },
            data_dir,
            cache_dir,
            debug: std::env::var("DEBUG").is_ok(),
        })
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists
        let _ = dotenvy::dotenv();

        let mut config = Self::new()?;

        // Override with environment variables if present
        if let Ok(db_path) = std::env::var("DATABASE_PATH") {
            config.database.path = db_path;
        }

        if let Ok(max_conn) = std::env::var("DB_MAX_CONNECTIONS") {
            if let Ok(num) = max_conn.parse() {
                config.database.max_connections = num;
            }
        }

        Ok(config)
    }
}

/// Initialize logging
fn init_logging(debug: bool) {
    let filter = if debug {
        tracing_subscriber::EnvFilter::new("profile_pulse=debug,sqlx=info,info")
    } else {
        tracing_subscriber::EnvFilter::new("profile_pulse=info,warn")
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Initialize the application
async fn init_app(config: &AppConfig) -> Result<()> {
    info!("Initializing Profile Pulse");
    info!("Data directory: {}", config.data_dir.display());
    info!("Cache directory: {}", config.cache_dir.display());
    info!("Database path: {}", config.database.path);

    // Initialize database
    info!("Connecting to database...");
    let pool = init_pool(&config.database)
        .await
        .context("Failed to initialize database")?;

    // Run migrations
    info!("Running database migrations...");
    run_migrations(&pool)
        .await
        .context("Failed to run database migrations")?;

    // Verify database health
    db::health_check(&pool)
        .await
        .context("Database health check failed")?;

    // Get initial statistics
    let stats = db::get_stats(&pool).await?;
    info!(
        "Database ready - Contacts: {}, Profiles: {}, Cache entries: {}",
        stats.contact_count, stats.profile_count, stats.cache_size
    );

    Ok(())
}

/// Main application entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = AppConfig::from_env().context("Failed to load application configuration")?;

    // Initialize logging
    init_logging(config.debug);

    info!("Starting Profile Pulse v{}", env!("CARGO_PKG_VERSION"));

    // Initialize application
    if let Err(e) = init_app(&config).await {
        error!("Failed to initialize application: {}", e);
        error!("Application will now exit");
        return Err(e);
    }

    info!("Application initialized successfully");

    // Run the GUI application
    info!("Launching GUI...");
    match ui::run() {
        Ok(_) => {
            info!("Application closed normally");
            Ok(())
        }
        Err(e) => {
            error!("GUI error: {}", e);
            Err(anyhow::anyhow!("GUI error: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_creation() {
        let config = AppConfig::new();
        assert!(config.is_ok());
    }

    #[test]
    fn test_app_config_paths_exist() {
        let config = AppConfig::new().unwrap();
        assert!(config.data_dir.exists() || config.data_dir.parent().unwrap().exists());
    }
}
