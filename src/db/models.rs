//! Database models for SQLite persistence
//!
//! These models represent the database structure and provide conversion
//! to/from domain models.

use chrono::{DateTime, TimeZone, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::core::contact::{Contact, SocialPlatform, SocialProfile};
use std::collections::HashMap;

/// Database representation of a Contact
#[derive(Debug, Clone, FromRow)]
pub struct ContactRow {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub organization: Option<String>,
    pub title: Option<String>,
    pub photo_url: Option<String>,
    pub photo_blob: Option<Vec<u8>>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ContactRow {
    /// Convert to domain Contact model
    pub fn to_contact(
        self,
        profiles: Vec<SocialProfile>,
        custom_fields: HashMap<String, String>,
    ) -> Contact {
        Contact {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::new_v4()),
            name: self.name,
            email: self.email,
            phone: self.phone,
            organization: self.organization,
            title: self.title,
            photo_url: self.photo_url,
            photo_blob: self.photo_blob,
            social_profiles: profiles,
            custom_fields,
            created_at: Utc.timestamp_opt(self.created_at, 0).unwrap(),
            updated_at: Utc.timestamp_opt(self.updated_at, 0).unwrap(),
        }
    }

    /// Create from domain Contact model
    pub fn from_contact(contact: &Contact) -> Self {
        Self {
            id: contact.id.to_string(),
            name: contact.name.clone(),
            email: contact.email.clone(),
            phone: contact.phone.clone(),
            organization: contact.organization.clone(),
            title: contact.title.clone(),
            photo_url: contact.photo_url.clone(),
            photo_blob: contact.photo_blob.clone(),
            created_at: contact.created_at.timestamp(),
            updated_at: contact.updated_at.timestamp(),
        }
    }
}

/// Database representation of a SocialProfile
#[derive(Debug, Clone, FromRow)]
pub struct SocialProfileRow {
    pub id: String,
    pub contact_id: String,
    pub platform: String,
    pub username: String,
    pub url: String,
    pub profile_pic_url: Option<String>,
    pub verified: bool,
    pub confidence_score: Option<f64>,
    pub discovered_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl SocialProfileRow {
    /// Convert to domain SocialProfile model
    pub fn to_social_profile(self) -> SocialProfile {
        SocialProfile {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::new_v4()),
            platform: SocialPlatform::from_str(&self.platform).unwrap_or(SocialPlatform::Other),
            username: self.username,
            url: self.url,
            profile_pic_url: self.profile_pic_url,
            verified: self.verified,
            confidence_score: self.confidence_score.map(|s| s as f32),
            discovered_at: self
                .discovered_at
                .map(|ts| Utc.timestamp_opt(ts, 0).unwrap()),
            created_at: Utc.timestamp_opt(self.created_at, 0).unwrap(),
            updated_at: Utc.timestamp_opt(self.updated_at, 0).unwrap(),
        }
    }

    /// Create from domain SocialProfile model
    pub fn from_social_profile(profile: &SocialProfile, contact_id: &Uuid) -> Self {
        Self {
            id: profile.id.to_string(),
            contact_id: contact_id.to_string(),
            platform: profile.platform.to_string(),
            username: profile.username.clone(),
            url: profile.url.clone(),
            profile_pic_url: profile.profile_pic_url.clone(),
            verified: profile.verified,
            confidence_score: profile.confidence_score.map(|s| s as f64),
            discovered_at: profile.discovered_at.map(|dt| dt.timestamp()),
            created_at: profile.created_at.timestamp(),
            updated_at: profile.updated_at.timestamp(),
        }
    }
}

/// Database representation of a custom field
#[derive(Debug, Clone, FromRow)]
pub struct CustomFieldRow {
    pub contact_id: String,
    pub key: String,
    pub value: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Database representation of a fetch queue entry
#[derive(Debug, Clone, FromRow)]
pub struct FetchQueueRow {
    pub id: String,
    pub contact_id: String,
    pub platform: String,
    pub username: String,
    pub status: String,
    pub priority: i32,
    pub retry_count: i32,
    pub last_attempt: Option<i64>,
    pub error_message: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Database representation of a cache entry
#[derive(Debug, Clone, FromRow)]
pub struct FetchCacheRow {
    pub key: String,
    pub data: Option<Vec<u8>>,
    pub content_type: Option<String>,
    pub cached_at: i64,
    pub expires_at: i64,
    pub hit_count: i32,
    pub last_accessed: i64,
}

/// Database representation of rate limit tracking
#[derive(Debug, Clone, FromRow)]
pub struct RateLimitRow {
    pub platform: String,
    pub requests_made: i32,
    pub window_start: i64,
    pub last_request: Option<i64>,
    pub daily_quota: i32,
    pub hourly_quota: i32,
}

/// Database representation of a setting
#[derive(Debug, Clone, FromRow)]
pub struct SettingRow {
    pub key: String,
    pub value: String,
    pub updated_at: i64,
}

/// Helper to convert DateTime to timestamp
pub fn datetime_to_timestamp(dt: DateTime<Utc>) -> i64 {
    dt.timestamp()
}

/// Helper to convert timestamp to DateTime
pub fn timestamp_to_datetime(ts: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(ts, 0).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_row_roundtrip() {
        let contact = Contact::new("Test User");
        let row = ContactRow::from_contact(&contact);
        let converted = row.to_contact(vec![], HashMap::new());

        assert_eq!(contact.id, converted.id);
        assert_eq!(contact.name, converted.name);
    }

    #[test]
    fn test_social_profile_row_roundtrip() {
        let profile = SocialProfile::new(
            SocialPlatform::GitHub,
            "testuser",
            "https://github.com/testuser",
        );
        let contact_id = Uuid::new_v4();
        let row = SocialProfileRow::from_social_profile(&profile, &contact_id);
        let converted = row.to_social_profile();

        assert_eq!(profile.id, converted.id);
        assert_eq!(profile.username, converted.username);
        assert_eq!(profile.platform, converted.platform);
    }

    #[test]
    fn test_datetime_conversion() {
        let now = Utc::now();
        let ts = datetime_to_timestamp(now);
        let converted = timestamp_to_datetime(ts);

        // Compare timestamps (seconds precision)
        assert_eq!(now.timestamp(), converted.timestamp());
    }
}
