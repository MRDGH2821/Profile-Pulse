//! Contact model and core data structures
//!
//! This module defines the primary Contact type and related structures for
//! managing contact information in Profile Pulse.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::labels::{AddressLabel, DateLabel, EmailLabel, PhoneLabel};

/// Represents an email address with label
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactEmail {
    /// Unique identifier for the email
    pub id: Uuid,
    /// The email address
    pub email: String,
    /// Label for the email (e.g., "Home", "Work", "Other")
    pub label: String,
    /// When the email was created
    pub created_at: DateTime<Utc>,
    /// When the email was last updated
    pub updated_at: DateTime<Utc>,
}

/// Represents a phone number with label
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactPhone {
    /// Unique identifier for the phone
    pub id: Uuid,
    /// The phone number
    pub phone: String,
    /// Label for the phone (e.g., "Mobile", "Home", "Work")
    pub label: String,
    /// When the phone was created
    pub created_at: DateTime<Utc>,
    /// When the phone was last updated
    pub updated_at: DateTime<Utc>,
}

/// Represents a physical address
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactAddress {
    /// Unique identifier for the address
    pub id: Uuid,
    /// Street address (can be multi-line)
    pub street: Option<String>,
    /// City
    pub city: Option<String>,
    /// State/Province/Region
    pub state: Option<String>,
    /// Postal/ZIP code
    pub postal_code: Option<String>,
    /// Country
    pub country: Option<String>,
    /// Label for the address (e.g., "Home", "Work")
    pub label: String,
    /// When the address was created
    pub created_at: DateTime<Utc>,
    /// When the address was last updated
    pub updated_at: DateTime<Utc>,
}

/// Represents a significant date/event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactDate {
    /// Unique identifier for the date
    pub id: Uuid,
    /// The date
    pub date: NaiveDate,
    /// Label for the date (e.g., "Birthday", "Anniversary")
    pub label: String,
    /// When the date entry was created
    pub created_at: DateTime<Utc>,
    /// When the date entry was last updated
    pub updated_at: DateTime<Utc>,
}

/// Represents a URL associated with a contact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactUrl {
    /// Unique identifier for the URL
    pub id: Uuid,
    /// The URL itself
    pub url: String,
    /// Label for the URL (e.g., "GitHub", "LinkedIn", "Personal", "Blog")
    pub label: Option<String>,
    /// When the URL was created
    pub created_at: DateTime<Utc>,
    /// When the URL was last updated
    pub updated_at: DateTime<Utc>,
}

/// Represents a contact with personal information and social media profiles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contact {
    /// Unique identifier for the contact
    pub id: Uuid,
    /// Full name of the contact (computed from structured name fields if available)
    pub name: String,
    /// Name prefix (e.g., "Dr.", "Mr.", "Ms.")
    pub name_prefix: Option<String>,
    /// First name / given name
    pub first_name: Option<String>,
    /// Middle name
    pub middle_name: Option<String>,
    /// Last name / family name
    pub last_name: Option<String>,
    /// Name suffix (e.g., "Jr.", "Sr.", "III")
    pub name_suffix: Option<String>,
    /// Nickname
    pub nickname: Option<String>,
    /// Notes about the contact
    pub notes: Option<String>,
    /// Primary email address (optional, deprecated - use emails vec)
    pub email: Option<String>,
    /// Primary phone number (optional, deprecated - use phones vec)
    pub phone: Option<String>,
    /// List of email addresses with labels
    pub emails: Vec<ContactEmail>,
    /// List of phone numbers with labels
    pub phones: Vec<ContactPhone>,
    /// List of addresses with labels
    pub addresses: Vec<ContactAddress>,
    /// List of significant dates with labels
    pub dates: Vec<ContactDate>,
    /// Organization/company name (optional)
    pub organization: Option<String>,
    /// Job title (optional)
    pub title: Option<String>,
    /// Department (optional)
    pub department: Option<String>,
    /// URL to profile picture (optional)
    pub photo_url: Option<String>,
    /// Cached profile picture data (optional)
    pub photo_blob: Option<Vec<u8>>,
    /// List of URLs (including social media profiles)
    pub urls: Vec<ContactUrl>,
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

impl ContactEmail {
    /// Create a new email entry
    pub fn new(email: impl Into<String>, label: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email: email.into(),
            label: label.into(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create with EmailLabel enum
    pub fn with_label_enum(email: impl Into<String>, label: EmailLabel) -> Self {
        Self::new(email, label.to_string_value())
    }
}

impl ContactPhone {
    /// Create a new phone entry
    pub fn new(phone: impl Into<String>, label: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            phone: phone.into(),
            label: label.into(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create with PhoneLabel enum
    pub fn with_label_enum(phone: impl Into<String>, label: PhoneLabel) -> Self {
        Self::new(phone, label.to_string_value())
    }
}

impl ContactAddress {
    /// Create a new address entry
    pub fn new(label: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            street: None,
            city: None,
            state: None,
            postal_code: None,
            country: None,
            label: label.into(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create with AddressLabel enum
    pub fn with_label_enum(label: AddressLabel) -> Self {
        Self::new(label.to_string_value())
    }

    /// Create a builder for address
    pub fn builder() -> ContactAddressBuilder {
        ContactAddressBuilder::default()
    }

    /// Check if address is empty
    pub fn is_empty(&self) -> bool {
        self.street.is_none()
            && self.city.is_none()
            && self.state.is_none()
            && self.postal_code.is_none()
            && self.country.is_none()
    }

    /// Format address as a single line
    pub fn format_oneline(&self) -> String {
        let parts: Vec<&str> = [
            self.street.as_deref(),
            self.city.as_deref(),
            self.state.as_deref(),
            self.postal_code.as_deref(),
            self.country.as_deref(),
        ]
        .iter()
        .filter_map(|&x| x)
        .collect();
        parts.join(", ")
    }
}

/// Builder for ContactAddress
#[derive(Debug, Default)]
pub struct ContactAddressBuilder {
    street: Option<String>,
    city: Option<String>,
    state: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    label: Option<String>,
}

impl ContactAddressBuilder {
    pub fn street(mut self, street: impl Into<String>) -> Self {
        self.street = Some(street.into());
        self
    }

    pub fn city(mut self, city: impl Into<String>) -> Self {
        self.city = Some(city.into());
        self
    }

    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn postal_code(mut self, postal_code: impl Into<String>) -> Self {
        self.postal_code = Some(postal_code.into());
        self
    }

    pub fn country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn build(self) -> ContactAddress {
        let now = Utc::now();
        ContactAddress {
            id: Uuid::new_v4(),
            street: self.street,
            city: self.city,
            state: self.state,
            postal_code: self.postal_code,
            country: self.country,
            label: self.label.unwrap_or_else(|| "Home".to_string()),
            created_at: now,
            updated_at: now,
        }
    }
}

impl ContactDate {
    /// Create a new date entry
    pub fn new(date: NaiveDate, label: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            date,
            label: label.into(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create with DateLabel enum
    pub fn with_label_enum(date: NaiveDate, label: DateLabel) -> Self {
        Self::new(date, label.to_string_value())
    }

    /// Parse from YYYYMMDD string format
    pub fn from_yyyymmdd(yyyymmdd: &str, label: impl Into<String>) -> Result<Self, String> {
        // Parse YYYYMMDD format
        let date = if yyyymmdd.len() == 8 {
            let year = yyyymmdd[0..4].parse::<i32>().map_err(|_| "Invalid year".to_string())?;
            let month = yyyymmdd[4..6].parse::<u32>().map_err(|_| "Invalid month".to_string())?;
            let day = yyyymmdd[6..8].parse::<u32>().map_err(|_| "Invalid day".to_string())?;
            NaiveDate::from_ymd_opt(year, month, day)
                .ok_or_else(|| "Invalid date".to_string())?
        } else {
            NaiveDate::parse_from_str(yyyymmdd, "%Y%m%d")
                .map_err(|e| format!("Parse error: {}", e))?
        };
        
        Ok(Self::new(date, label))
    }

    /// Convert to YYYYMMDD string format
    pub fn to_yyyymmdd(&self) -> String {
        self.date.format("%Y%m%d").to_string()
    }
}

/// Builder for creating new contacts
#[derive(Debug, Default)]
pub struct ContactBuilder {
    name: String,
    name_prefix: Option<String>,
    first_name: Option<String>,
    middle_name: Option<String>,
    last_name: Option<String>,
    name_suffix: Option<String>,
    nickname: Option<String>,
    notes: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    organization: Option<String>,
    title: Option<String>,
    department: Option<String>,
    photo_url: Option<String>,
    emails: Vec<ContactEmail>,
    phones: Vec<ContactPhone>,
    addresses: Vec<ContactAddress>,
    dates: Vec<ContactDate>,
    urls: Vec<ContactUrl>,
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
        self.name = name.into();
        self
    }

    /// Set name prefix
    pub fn name_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.name_prefix = Some(prefix.into());
        self
    }

    /// Set first name
    pub fn first_name(mut self, first: impl Into<String>) -> Self {
        self.first_name = Some(first.into());
        self
    }

    /// Set middle name
    pub fn middle_name(mut self, middle: impl Into<String>) -> Self {
        self.middle_name = Some(middle.into());
        self
    }

    /// Set last name
    pub fn last_name(mut self, last: impl Into<String>) -> Self {
        self.last_name = Some(last.into());
        self
    }

    /// Set name suffix
    pub fn name_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.name_suffix = Some(suffix.into());
        self
    }

    /// Set nickname
    pub fn nickname(mut self, nickname: impl Into<String>) -> Self {
        self.nickname = Some(nickname.into());
        self
    }

    /// Set notes
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
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
    pub fn photo_url(mut self, photo_url: impl Into<String>) -> Self {
        self.photo_url = Some(photo_url.into());
        self
    }

    /// Set department
    pub fn department(mut self, department: impl Into<String>) -> Self {
        self.department = Some(department.into());
        self
    }

    /// Add an email
    pub fn email_entry(mut self, email: ContactEmail) -> Self {
        self.emails.push(email);
        self
    }

    /// Add a phone
    pub fn phone_entry(mut self, phone: ContactPhone) -> Self {
        self.phones.push(phone);
        self
    }

    /// Add an address
    pub fn address(mut self, address: ContactAddress) -> Self {
        self.addresses.push(address);
        self
    }

    /// Add a date
    pub fn date(mut self, date: ContactDate) -> Self {
        self.dates.push(date);
        self
    }

    /// Add a URL
    pub fn url(mut self, url: ContactUrl) -> Self {
        self.urls.push(url);
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
        if self.name.trim().is_empty() {
            return Err(ContactBuilderError::EmptyName);
        }

        let now = Utc::now();

        Ok(Contact {
            id: Uuid::new_v4(),
            name: self.name.clone(),
            name_prefix: self.name_prefix.clone(),
            first_name: self.first_name.clone(),
            middle_name: self.middle_name.clone(),
            last_name: self.last_name.clone(),
            name_suffix: self.name_suffix.clone(),
            nickname: self.nickname.clone(),
            notes: self.notes.clone(),
            email: self.email.clone(),
            phone: self.phone.clone(),
            organization: self.organization.clone(),
            title: self.title.clone(),
            department: self.department.clone(),
            photo_url: self.photo_url.clone(),
            photo_blob: None,
            emails: self.emails.clone(),
            phones: self.phones.clone(),
            addresses: self.addresses.clone(),
            dates: self.dates.clone(),
            urls: self.urls.clone(),
            social_profiles: self.social_profiles.clone(),
            custom_fields: self.custom_fields.clone(),
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
            name_prefix: None,
            first_name: None,
            middle_name: None,
            last_name: None,
            name_suffix: None,
            nickname: None,
            notes: None,
            email: None,
            phone: None,
            emails: Vec::new(),
            phones: Vec::new(),
            addresses: Vec::new(),
            dates: Vec::new(),
            organization: None,
            title: None,
            department: None,
            photo_url: None,
            photo_blob: None,
            urls: Vec::new(),
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

    /// Add an email to the contact
    pub fn add_email(&mut self, email: ContactEmail) {
        self.emails.push(email);
        self.touch();
    }

    /// Add a phone to the contact
    pub fn add_phone(&mut self, phone: ContactPhone) {
        self.phones.push(phone);
        self.touch();
    }

    /// Add an address to the contact
    pub fn add_address(&mut self, address: ContactAddress) {
        self.addresses.push(address);
        self.touch();
    }

    /// Add a date to the contact
    pub fn add_date(&mut self, date: ContactDate) {
        self.dates.push(date);
        self.touch();
    }

    /// Add a URL to the contact
    pub fn add_url(&mut self, url: ContactUrl) {
        self.urls.push(url);
        self.touch();
    }

    /// Find URLs by label
    pub fn find_urls_by_label(&self, label: &str) -> Vec<&ContactUrl> {
        self.urls
            .iter()
            .filter(|u| u.label.as_deref() == Some(label))
            .collect()
    }

    /// Check if contact has a URL with the given label
    pub fn has_url_label(&self, label: &str) -> bool {
        self.urls.iter().any(|u| u.label.as_deref() == Some(label))
    }

    /// Get all unique URL labels
    pub fn url_labels(&self) -> Vec<String> {
        self.urls
            .iter()
            .filter_map(|u| u.label.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Get the primary email (first email or deprecated email field)
    pub fn primary_email(&self) -> Option<&str> {
        self.emails
            .first()
            .map(|e| e.email.as_str())
            .or(self.email.as_deref())
    }

    /// Get the primary phone (first phone or deprecated phone field)
    pub fn primary_phone(&self) -> Option<&str> {
        self.phones
            .first()
            .map(|p| p.phone.as_str())
            .or(self.phone.as_deref())
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

impl ContactUrl {
    /// Create a new URL entry
    pub fn new(url: impl Into<String>, label: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            url: url.into(),
            label,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if this URL is likely a social media profile
    pub fn is_social_media(&self) -> bool {
        let url_lower = self.url.to_lowercase();
        url_lower.contains("linkedin.com")
            || url_lower.contains("twitter.com")
            || url_lower.contains("facebook.com")
            || url_lower.contains("instagram.com")
            || url_lower.contains("github.com")
            || url_lower.contains("mastodon")
    }

    /// Attempt to parse the social platform from the URL
    pub fn as_social_platform(&self) -> Option<SocialPlatform> {
        SocialPlatform::from_str(&self.url)
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
