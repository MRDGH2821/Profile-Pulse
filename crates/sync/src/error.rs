use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("adapter not configured: {0}")]
    NotConfigured(String),
    #[error("authentication required for {0}")]
    AuthRequired(String),
    #[error("oauth error: {0}")]
    OAuth(String),
    #[error("http error: {0}")]
    Http(String),
    #[error("remote error: {0}")]
    Remote(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("core error: {0}")]
    Core(#[from] profile_pulse_core::CoreError),
}
