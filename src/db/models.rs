//! Database models for SQLite persistence
//!
//! These models represent the database structure and provide conversion
//! to/from domain models.

use chrono::{DateTime, TimeZone, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::core::contact::{Contact, ContactAddress, ContactDate, ContactEmail, ContactPhone, ContactUrl, SocialPlatform, SocialProfile};
use chrono::NaiveDate;
use std::collections::HashMap;

/// Database representation of a Contact
#[derive(Debug, Clone, FromRow)]
pub struct ContactRow {
    pub id: String,
    pub name: String,
    pub name_prefix: Option<String>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub name_suffix: Option<String>,
    pub nickname: Option<String>,
    pub notes: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub organization: Option<String>,
    pub title: Option<String>,
    pub department: Option<String>,
    pub photo_url: Option<String>,
    pub photo_blob: Option<Vec<u8>>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ContactRow {
    /// Convert to domain Contact model
    pub fn to_contact(
        self,
        urls: Vec<ContactUrl>,
        custom_fields: HashMap<String, String>,
    ) -> Contact {
        Contact {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::new_v4()),
            name: self.name,
            name_prefix: self.name_prefix,
            first_name: self.first_name,
            middle_name: self.middle_name,
            last_name: self.last_name,
            name_suffix: self.name_suffix,
            nickname: self.nickname,
            notes: self.notes,
            email: self.email,
            phone: self.phone,
            emails: Vec::new(), // TODO: Load from database
            phones: Vec::new(), // TODO: Load from database
            addresses: Vec::new(), // TODO: Load from database
            dates: Vec::new(), // TODO: Load from database
            organization: self.organization,
            title: self.title,
            department: self.department,
            photo_url: self.photo_url,
            photo_blob: self.photo_blob,
            urls,
            social_profiles: Vec::new(), // TODO: Load from database
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
            name_prefix: contact.name_prefix.clone(),
            first_name: contact.first_name.clone(),
            middle_name: contact.middle_name.clone(),
            last_name: contact.last_name.clone(),
            name_suffix: contact.name_suffix.clone(),
            nickname: contact.nickname.clone(),
            notes: contact.notes.clone(),
            email: contact.email.clone(),
            phone: contact.phone.clone(),
            organization: contact.organization.clone(),
            title: contact.title.clone(),
            department: contact.department.clone(),
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

/// Database representation of a ContactEmail
#[derive(Debug, Clone, FromRow)]
pub struct ContactEmailRow {
    pub id: String,
    pub contact_id: String,
    pub email: String,
    pub label: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ContactEmailRow {
    /// Convert to domain ContactEmail model
    pub fn to_contact_email(self) -> ContactEmail {
        ContactEmail {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::new_v4()),
            email: self.email,
            label: self.label,
            created_at: Utc.timestamp_opt(self.created_at, 0).unwrap(),
            updated_at: Utc.timestamp_opt(self.updated_at, 0).unwrap(),
        }
    }

    /// Create from domain ContactEmail model
    pub fn from_contact_email(contact_email: &ContactEmail, contact_id: &Uuid) -> Self {
        Self {
            id: contact_email.id.to_string(),
            contact_id: contact_id.to_string(),
            email: contact_email.email.clone(),
            label: contact_email.label.clone(),
            created_at: contact_email.created_at.timestamp(),
            updated_at: contact_email.updated_at.timestamp(),
        }
    }
}

/// Database representation of a ContactPhone
#[derive(Debug, Clone, FromRow)]
pub struct ContactPhoneRow {
    pub id: String,
    pub contact_id: String,
    pub phone: String,
    pub label: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ContactPhoneRow {
    /// Convert to domain ContactPhone model
    pub fn to_contact_phone(self) -> ContactPhone {
        ContactPhone {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::new_v4()),
            phone: self.phone,
            label: self.label,
            created_at: Utc.timestamp_opt(self.created_at, 0).unwrap(),
            updated_at: Utc.timestamp_opt(self.updated_at, 0).unwrap(),
        }
    }

    /// Create from domain ContactPhone model
    pub fn from_contact_phone(contact_phone: &ContactPhone, contact_id: &Uuid) -> Self {
        Self {
            id: contact_phone.id.to_string(),
            contact_id: contact_id.to_string(),
            phone: contact_phone.phone.clone(),
            label: contact_phone.label.clone(),
            created_at: contact_phone.created_at.timestamp(),
            updated_at: contact_phone.updated_at.timestamp(),
        }
    }
}

/// Database representation of a ContactAddress
#[derive(Debug, Clone, FromRow)]
pub struct ContactAddressRow {
    pub id: String,
    pub contact_id: String,
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub label: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ContactAddressRow {
    /// Convert to domain ContactAddress model
    pub fn to_contact_address(self) -> ContactAddress {
        ContactAddress {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::new_v4()),
            street: self.street,
            city: self.city,
            state: self.state,
            postal_code: self.postal_code,
            country: self.country,
            label: self.label,
            created_at: Utc.timestamp_opt(self.created_at, 0).unwrap(),
            updated_at: Utc.timestamp_opt(self.updated_at, 0).unwrap(),
        }
    }

    /// Create from domain ContactAddress model
    pub fn from_contact_address(contact_address: &ContactAddress, contact_id: &Uuid) -> Self {
        Self {
            id: contact_address.id.to_string(),
            contact_id: contact_id.to_string(),
            street: contact_address.street.clone(),
            city: contact_address.city.clone(),
            state: contact_address.state.clone(),
            postal_code: contact_address.postal_code.clone(),
            country: contact_address.country.clone(),
            label: contact_address.label.clone(),
            created_at: contact_address.created_at.timestamp(),
            updated_at: contact_address.updated_at.timestamp(),
        }
    }
}

/// Database representation of a ContactDate
#[derive(Debug, Clone, FromRow)]
pub struct ContactDateRow {
    pub id: String,
    pub contact_id: String,
    pub date: String, // ISO 8601 format (YYYY-MM-DD)
    pub label: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ContactDateRow {
    /// Convert to domain ContactDate model
    pub fn to_contact_date(self) -> Result<ContactDate, String> {
        let date = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
            .map_err(|e| format!("Failed to parse date: {}", e))?;
        
        Ok(ContactDate {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::new_v4()),
            date,
            label: self.label,
            created_at: Utc.timestamp_opt(self.created_at, 0).unwrap(),
            updated_at: Utc.timestamp_opt(self.updated_at, 0).unwrap(),
        })
    }

    /// Create from domain ContactDate model
    pub fn from_contact_date(contact_date: &ContactDate, contact_id: &Uuid) -> Self {
        Self {
            id: contact_date.id.to_string(),
            contact_id: contact_id.to_string(),
            date: contact_date.date.format("%Y-%m-%d").to_string(),
            label: contact_date.label.clone(),
            created_at: contact_date.created_at.timestamp(),
            updated_at: contact_date.updated_at.timestamp(),
        }
    }
}

/// Database representation of a ContactUrl
#[derive(Debug, Clone, FromRow)]
pub struct ContactUrlRow {
    pub id: String,
    pub contact_id: String,
    pub url: String,
    pub label: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ContactUrlRow {
    /// Convert to domain ContactUrl model
    pub fn to_contact_url(self) -> ContactUrl {
        ContactUrl {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::new_v4()),
            url: self.url,
            label: self.label,
            created_at: Utc.timestamp_opt(self.created_at, 0).unwrap(),
            updated_at: Utc.timestamp_opt(self.updated_at, 0).unwrap(),
        }
    }

    /// Create from domain ContactUrl model
    pub fn from_contact_url(contact_url: &ContactUrl, contact_id: &Uuid) -> Self {
        Self {
            id: contact_url.id.to_string(),
            contact_id: contact_id.to_string(),
            url: contact_url.url.clone(),
            label: contact_url.label.clone(),
            created_at: contact_url.created_at.timestamp(),
            updated_at: contact_url.updated_at.timestamp(),
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
    fn test_contact_email_row_roundtrip() {
        let contact_email = ContactEmail::new("test@example.com", "Home");
        let contact_id = Uuid::new_v4();
        let row = ContactEmailRow::from_contact_email(&contact_email, &contact_id);
        let converted = row.to_contact_email();

        assert_eq!(contact_email.id, converted.id);
        assert_eq!(contact_email.email, converted.email);
        assert_eq!(contact_email.label, converted.label);
    }

    #[test]
    fn test_contact_phone_row_roundtrip() {
        let contact_phone = ContactPhone::new("+1234567890", "Mobile");
        let contact_id = Uuid::new_v4();
        let row = ContactPhoneRow::from_contact_phone(&contact_phone, &contact_id);
        let converted = row.to_contact_phone();

        assert_eq!(contact_phone.id, converted.id);
        assert_eq!(contact_phone.phone, converted.phone);
        assert_eq!(contact_phone.label, converted.label);
    }

    #[test]
    fn test_contact_address_row_roundtrip() {
        let contact_address = ContactAddress::builder()
            .street("123 Main St")
            .city("Springfield")
            .label("Home")
            .build();
        let contact_id = Uuid::new_v4();
        let row = ContactAddressRow::from_contact_address(&contact_address, &contact_id);
        let converted = row.to_contact_address();

        assert_eq!(contact_address.id, converted.id);
        assert_eq!(contact_address.street, converted.street);
        assert_eq!(contact_address.city, converted.city);
        assert_eq!(contact_address.label, converted.label);
    }

    #[test]
    fn test_contact_date_row_roundtrip() {
        let date = NaiveDate::from_ymd_opt(1990, 5, 15).unwrap();
        let contact_date = ContactDate::new(date, "Birthday");
        let contact_id = Uuid::new_v4();
        let row = ContactDateRow::from_contact_date(&contact_date, &contact_id);
        let converted = row.to_contact_date().unwrap();

        assert_eq!(contact_date.id, converted.id);
        assert_eq!(contact_date.date, converted.date);
        assert_eq!(contact_date.label, converted.label);
    }

    #[test]
    fn test_contact_url_row_roundtrip() {
        let contact_url = ContactUrl::new("https://github.com/test", Some("GitHub".to_string()));
        let contact_id = Uuid::new_v4();
        let row = ContactUrlRow::from_contact_url(&contact_url, &contact_id);
        let converted = row.to_contact_url();

        assert_eq!(contact_url.id, converted.id);
        assert_eq!(contact_url.url, converted.url);
        assert_eq!(contact_url.label, converted.label);
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
