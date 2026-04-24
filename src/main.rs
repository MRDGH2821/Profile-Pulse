//! Profile Pulse - Desktop contact management with social media integration
//!
//! This application helps manage contacts stored in VCF files.

use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod core;
mod discovery;
mod social;
mod ui;
mod utils;
mod vcf;
mod workspace;

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
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

        Ok(Self {
            data_dir,
            cache_dir,
            debug: std::env::var("DEBUG").is_ok(),
        })
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists
        let _ = dotenvy::dotenv();

        Self::new()
    }
}

/// Initialize logging
fn init_logging(debug: bool) {
    let filter = if debug {
        tracing_subscriber::EnvFilter::new("profile_pulse=debug,info")
    } else {
        tracing_subscriber::EnvFilter::new("profile_pulse=info,warn")
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Main application entry point
fn main() -> Result<()> {
    // Load configuration
    let config = AppConfig::from_env().context("Failed to load application configuration")?;

    // Initialize logging
    init_logging(config.debug);

    info!("Starting Profile Pulse v{}", env!("CARGO_PKG_VERSION"));
    info!("Data directory: {}", config.data_dir.display());
    info!("Using VCF files directly (no database)");

    // Run the GUI application with workspace support
    info!("Launching GUI with workspace selector...");
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
