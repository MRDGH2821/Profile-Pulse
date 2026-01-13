//! Social media integration module for Profile Pulse
//!
//! Contains traits and implementations for fetching profile data from
//! various social media platforms.

// TODO: Implement ProfileFetcher trait
// TODO: Implement GitHub fetcher
// TODO: Implement LinkedIn fetcher (web scraping)
// TODO: Implement Twitter/X fetcher
// TODO: Implement rate limiting
// TODO: Implement caching

use crate::core::contact::SocialPlatform;
use crate::utils::FetchError;
use async_trait::async_trait;

/// Result type for fetch operations
pub type FetchResult<T> = std::result::Result<T, FetchError>;

/// Trait for fetching social media profile data
#[async_trait]
pub trait ProfileFetcher: Send + Sync {
    /// Fetch profile picture by username
    async fn fetch_profile_pic(&self, username: &str) -> FetchResult<Vec<u8>>;

    /// Search for profiles matching contact information
    async fn search_profile(
        &self,
        name: &str,
        email: Option<&str>,
    ) -> FetchResult<Vec<ProfileMatch>>;

    /// Get the platform this fetcher handles
    fn platform(&self) -> SocialPlatform;

    /// Check if rate limit allows request
    async fn can_fetch(&self) -> bool;
}

/// A potential profile match from search
#[derive(Debug, Clone)]
pub struct ProfileMatch {
    pub username: String,
    pub url: String,
    pub name: String,
    pub profile_pic_url: Option<String>,
    pub confidence: f32,
}

/// Rate limit status for a platform
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub remaining: u32,
    pub reset_at: chrono::DateTime<chrono::Utc>,
}
