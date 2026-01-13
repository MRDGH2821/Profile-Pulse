//! Contact model and core data structures
//!
//! This module defines the primary Contact type and related structures for
//! managing contact information in Profile Pulse.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a contact with personal information and social media profiles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contact {
    /// Unique identifier for the contact
    pub id: Uuid,
    /// Full name of the contact
    pub name: String,
    /// Email address (optional)
    pub email: Option<String>,
    /// Phone number (optional)
    pub phone: Option<String>,
    /// Organization/company name (optional)
    pub organization: Option<String>,
    /// Job title (optional)
    pub title: Option<String>,
    /// URL to profile picture (optional)
    pub photo_url: Option<String>,
    /// Cached profile picture data (optional)
    pub photo_blob: Option<Vec<u8>>,
    /// List of social media profiles
    pub social_profiles: Vec<SocialProfile>,
    /// Additional custom fields from VCF or user-defined
    pub custom_fields: HashMap<String, String>,
    /// When the contact was created
    pub created_at: DateTime<Utc>,
    /// When the contact was last updated
    pub updated_at: DateTime<Utc>,
}

/// Represents a social media profile associated with a contact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocialProfile {
    /// Unique identifier for the profile
    pub id: Uuid,
    /// The social media platform
    pub platform: SocialPlatform,
    /// Username on the platform
    pub username: String,
    /// Full URL to the profile
    pub url: String,
    /// URL to the profile picture (optional)
    pub profile_pic_url: Option<String>,
    /// Whether this is a verified/confirmed profile
    pub verified: bool,
    /// Confidence score for discovered profiles (0.0-1.0)
    pub confidence_score: Option<f32>,
    /// When this profile was discovered (if auto-discovered)
    pub discovered_at: Option<DateTime<Utc>>,
    /// When the profile entry was created
    pub created_at: DateTime<Utc>,
    /// When the profile entry was last updated
    pub updated_at: DateTime<Utc>,
}

/// Supported social media platforms
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum SocialPlatform {
    /// LinkedIn professional network
    LinkedIn,
    /// Twitter/X social network
    Twitter,
    /// Facebook social network
    Facebook,
    /// Instagram photo sharing
    Instagram,
    /// GitHub code hosting
    GitHub,
    /// Mastodon federated social network
    Mastodon,
    /// Custom/other platform
    Other,
}

impl SocialPlatform {
    /// Returns the canonical name of the platform
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LinkedIn => "linkedin",
            Self::Twitter => "twitter",
            Self::Facebook => "facebook",
            Self::Instagram => "instagram",
            Self::GitHub => "github",
            Self::Mastodon => "mastodon",
            Self::Other => "other",
        }
    }

    /// Parse platform from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "linkedin" => Some(Self::LinkedIn),
            "twitter" | "x" => Some(Self::Twitter),
            "facebook" => Some(Self::Facebook),
            "instagram" => Some(Self::Instagram),
            "github" => Some(Self::GitHub),
            "mastodon" => Some(Self::Mastodon),
            _ => Some(Self::Other),
        }
    }

    /// Get the base URL for the platform
    pub fn base_url(&self) -> Option<&'static str> {
        match self {
            Self::LinkedIn => Some("https://www.linkedin.com/in/"),
            Self::Twitter => Some("https://twitter.com/"),
            Self::Facebook => Some("https://www.facebook.com/"),
            Self::Instagram => Some("https://www.instagram.com/"),
            Self::GitHub => Some("https://github.com/"),
            Self::Mastodon => None, // Instance-specific
            Self::Other => None,
        }
    }
}

impl std::fmt::Display for SocialPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Builder for creating new contacts
#[derive(Debug, Default)]
pub struct ContactBuilder {
    name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    organization: Option<String>,
    title: Option<String>,
    photo_url: Option<String>,
    social_profiles: Vec<SocialProfile>,
    custom_fields: HashMap<String, String>,
}

impl ContactBuilder {
    /// Create a new ContactBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the contact name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the email address
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Set the phone number
    pub fn phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    /// Set the organization
    pub fn organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }

    /// Set the job title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the photo URL
    pub fn photo_url(mut self, url: impl Into<String>) -> Self {
        self.photo_url = Some(url.into());
        self
    }

    /// Add a social profile
    pub fn social_profile(mut self, profile: SocialProfile) -> Self {
        self.social_profiles.push(profile);
        self
    }

    /// Add a custom field
    pub fn custom_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_fields.insert(key.into(), value.into());
        self
    }

    /// Build the Contact
    pub fn build(self) -> Result<Contact, ContactBuilderError> {
        let name = self.name.ok_or(ContactBuilderError::MissingName)?;

        if name.trim().is_empty() {
            return Err(ContactBuilderError::EmptyName);
        }

        let now = Utc::now();

        Ok(Contact {
            id: Uuid::new_v4(),
            name,
            email: self.email,
            phone: self.phone,
            organization: self.organization,
            title: self.title,
            photo_url: self.photo_url,
            photo_blob: None,
            social_profiles: self.social_profiles,
            custom_fields: self.custom_fields,
            created_at: now,
            updated_at: now,
        })
    }
}

/// Errors that can occur when building a Contact
#[derive(Debug, thiserror::Error)]
pub enum ContactBuilderError {
    #[error("Contact name is required")]
    MissingName,
    #[error("Contact name cannot be empty")]
    EmptyName,
}

impl Contact {
    /// Create a new contact with just a name
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            email: None,
            phone: None,
            organization: None,
            title: None,
            photo_url: None,
            photo_blob: None,
            social_profiles: Vec::new(),
            custom_fields: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a ContactBuilder
    pub fn builder() -> ContactBuilder {
        ContactBuilder::new()
    }

    /// Update the contact's timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Add a social profile to the contact
    pub fn add_social_profile(&mut self, profile: SocialProfile) {
        self.social_profiles.push(profile);
        self.touch();
    }

    /// Find a social profile by platform
    pub fn find_profile(&self, platform: SocialPlatform) -> Option<&SocialProfile> {
        self.social_profiles.iter().find(|p| p.platform == platform)
    }

    /// Check if contact has a profile on the given platform
    pub fn has_platform(&self, platform: SocialPlatform) -> bool {
        self.find_profile(platform).is_some()
    }

    /// Get all platforms this contact has profiles on
    pub fn platforms(&self) -> Vec<SocialPlatform> {
        self.social_profiles.iter().map(|p| p.platform).collect()
    }
}

impl SocialProfile {
    /// Create a new social profile
    pub fn new(
        platform: SocialPlatform,
        username: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            platform,
            username: username.into(),
            url: url.into(),
            profile_pic_url: None,
            verified: false,
            confidence_score: None,
            discovered_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark this profile as verified
    pub fn verify(&mut self) {
        self.verified = true;
        self.updated_at = Utc::now();
    }

    /// Set the confidence score for a discovered profile
    pub fn set_confidence(&mut self, score: f32) {
        self.confidence_score = Some(score.clamp(0.0, 1.0));
        if self.discovered_at.is_none() {
            self.discovered_at = Some(Utc::now());
        }
        self.updated_at = Utc::now();
    }

    /// Check if this is a high-confidence match
    pub fn is_high_confidence(&self) -> bool {
        self.confidence_score.is_some_and(|s| s >= 0.8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_creation() {
        let contact = Contact::new("John Doe");
        assert_eq!(contact.name, "John Doe");
        assert!(contact.email.is_none());
        assert!(contact.social_profiles.is_empty());
    }

    #[test]
    fn test_contact_builder() {
        let contact = Contact::builder()
            .name("Jane Smith")
            .email("jane@example.com")
            .phone("+1234567890")
            .organization("Example Corp")
            .title("Software Engineer")
            .build()
            .unwrap();

        assert_eq!(contact.name, "Jane Smith");
        assert_eq!(contact.email, Some("jane@example.com".to_string()));
        assert_eq!(contact.phone, Some("+1234567890".to_string()));
        assert_eq!(contact.organization, Some("Example Corp".to_string()));
        assert_eq!(contact.title, Some("Software Engineer".to_string()));
    }

    #[test]
    fn test_contact_builder_missing_name() {
        let result = Contact::builder().email("test@example.com").build();

        assert!(result.is_err());
        assert!(matches!(result, Err(ContactBuilderError::MissingName)));
    }

    #[test]
    fn test_contact_builder_empty_name() {
        let result = Contact::builder().name("   ").build();

        assert!(result.is_err());
        assert!(matches!(result, Err(ContactBuilderError::EmptyName)));
    }

    #[test]
    fn test_social_profile_creation() {
        let profile = SocialProfile::new(
            SocialPlatform::GitHub,
            "octocat",
            "https://github.com/octocat",
        );

        assert_eq!(profile.platform, SocialPlatform::GitHub);
        assert_eq!(profile.username, "octocat");
        assert_eq!(profile.url, "https://github.com/octocat");
        assert!(!profile.verified);
    }

    #[test]
    fn test_social_profile_verify() {
        let mut profile = SocialProfile::new(
            SocialPlatform::GitHub,
            "octocat",
            "https://github.com/octocat",
        );

        assert!(!profile.verified);
        profile.verify();
        assert!(profile.verified);
    }

    #[test]
    fn test_social_profile_confidence() {
        let mut profile = SocialProfile::new(
            SocialPlatform::LinkedIn,
            "john-doe",
            "https://linkedin.com/in/john-doe",
        );

        profile.set_confidence(0.95);
        assert_eq!(profile.confidence_score, Some(0.95));
        assert!(profile.is_high_confidence());
        assert!(profile.discovered_at.is_some());
    }

    #[test]
    fn test_social_platform_from_str() {
        assert_eq!(
            SocialPlatform::from_str("github"),
            Some(SocialPlatform::GitHub)
        );
        assert_eq!(
            SocialPlatform::from_str("GitHub"),
            Some(SocialPlatform::GitHub)
        );
        assert_eq!(
            SocialPlatform::from_str("twitter"),
            Some(SocialPlatform::Twitter)
        );
        assert_eq!(SocialPlatform::from_str("x"), Some(SocialPlatform::Twitter));
    }

    #[test]
    fn test_contact_add_profile() {
        let mut contact = Contact::new("John Doe");
        let profile = SocialProfile::new(
            SocialPlatform::GitHub,
            "johndoe",
            "https://github.com/johndoe",
        );

        assert!(!contact.has_platform(SocialPlatform::GitHub));
        contact.add_social_profile(profile);
        assert!(contact.has_platform(SocialPlatform::GitHub));
    }

    #[test]
    fn test_contact_find_profile() {
        let mut contact = Contact::new("John Doe");
        let profile = SocialProfile::new(
            SocialPlatform::GitHub,
            "johndoe",
            "https://github.com/johndoe",
        );

        contact.add_social_profile(profile.clone());

        let found = contact.find_profile(SocialPlatform::GitHub);
        assert!(found.is_some());
        assert_eq!(found.unwrap().username, "johndoe");

        let not_found = contact.find_profile(SocialPlatform::LinkedIn);
        assert!(not_found.is_none());
    }
}
