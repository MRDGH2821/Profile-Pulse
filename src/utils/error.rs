//! Error types and utilities for Profile Pulse
//!
//! Provides common error types used throughout the application.

use thiserror::Error;

/// Main application error type
#[derive(Error, Debug)]
pub enum AppError {
    /// Database operation failed
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// I/O operation failed
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// HTTP request failed
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization failed
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// VCF parsing failed
    #[error("VCF parse error: {0}")]
    VcfParse(String),

    /// Contact not found
    #[error("Contact not found: {0}")]
    ContactNotFound(String),

    /// Social profile not found
    #[error("Social profile not found: {platform} - {username}")]
    ProfileNotFound { platform: String, username: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded for {platform}. Retry after {retry_after:?}")]
    RateLimitExceeded {
        platform: String,
        retry_after: std::time::Duration,
    },

    /// Authentication required
    #[error("Authentication required for {platform}")]
    AuthenticationRequired { platform: String },

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Image processing error
    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    /// Generic error with context
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

/// Result type alias using AppError
pub type Result<T> = std::result::Result<T, AppError>;

impl AppError {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, AppError::Http(_) | AppError::RateLimitExceeded { .. })
    }

    /// Check if this is a rate limit error
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, AppError::RateLimitExceeded { .. })
    }

    /// Check if this is an authentication error
    pub fn is_auth_error(&self) -> bool {
        matches!(self, AppError::AuthenticationRequired { .. })
    }
}

/// Fetch-specific errors
#[derive(Error, Debug)]
pub enum FetchError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Profile not found: {platform} - {username}")]
    ProfileNotFound { platform: String, username: String },

    #[error("Rate limit exceeded for {platform}")]
    RateLimitExceeded {
        platform: String,
        retry_after: std::time::Duration,
    },

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Platform unavailable: {0}")]
    PlatformUnavailable(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Authentication required")]
    AuthRequired,
}

impl FetchError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            FetchError::Network(_) | FetchError::PlatformUnavailable(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_display() {
        let err = AppError::ContactNotFound("123".to_string());
        assert_eq!(err.to_string(), "Contact not found: 123");
    }

    #[test]
    fn test_is_retryable() {
        let err = AppError::RateLimitExceeded {
            platform: "github".to_string(),
            retry_after: std::time::Duration::from_secs(60),
        };
        assert!(err.is_retryable());

        let err = AppError::ContactNotFound("123".to_string());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_rate_limit() {
        let err = AppError::RateLimitExceeded {
            platform: "github".to_string(),
            retry_after: std::time::Duration::from_secs(60),
        };
        assert!(err.is_rate_limit());
    }

    #[test]
    fn test_fetch_error_retryable() {
        let err = FetchError::PlatformUnavailable("github".to_string());
        assert!(err.is_retryable());

        let err = FetchError::ParseError("invalid json".to_string());
        assert!(!err.is_retryable());
    }
}
