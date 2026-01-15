//! Label types and utilities for contact fields
//!
//! This module defines common label types for various contact fields
//! (emails, phones, addresses, dates, URLs) based on vCard/VCF standards
//! and common practices from contact management systems.

use serde::{Deserialize, Serialize};

/// Common email labels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmailLabel {
    /// Home email
    Home,
    /// Work email
    Work,
    /// Other email
    Other,
    /// Custom label
    Custom(String),
}

impl EmailLabel {
    /// Get common email label options for dropdowns
    pub fn common_options() -> Vec<&'static str> {
        vec!["Home", "Work", "Other"]
    }

    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "home" | "internet;home" => Self::Home,
            "work" | "internet;work" => Self::Work,
            "other" => Self::Other,
            _ => Self::Custom(s.to_string()),
        }
    }

    /// Convert to string for display
    pub fn as_str(&self) -> &str {
        match self {
            Self::Home => "Home",
            Self::Work => "Work",
            Self::Other => "Other",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Get the string value (owned)
    pub fn to_string_value(&self) -> String {
        self.as_str().to_string()
    }
}

impl Default for EmailLabel {
    fn default() -> Self {
        Self::Home
    }
}

/// Common phone number labels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhoneLabel {
    /// Home phone
    Home,
    /// Work phone
    Work,
    /// Mobile/cell phone
    Mobile,
    /// Main phone number
    Main,
    /// Home fax
    HomeFax,
    /// Work fax
    WorkFax,
    /// Pager
    Pager,
    /// Other phone
    Other,
    /// Custom label
    Custom(String),
}

impl PhoneLabel {
    /// Get common phone label options for dropdowns
    pub fn common_options() -> Vec<&'static str> {
        vec![
            "Mobile",
            "Home",
            "Work",
            "Main",
            "Home Fax",
            "Work Fax",
            "Pager",
            "Other",
        ]
    }

    /// Parse from string or TYPE parameters (case-insensitive)
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        match lower.as_str() {
            "home" => Self::Home,
            "work" => Self::Work,
            "cell" | "mobile" => Self::Mobile,
            "main" => Self::Main,
            "home;fax" | "home fax" | "homefax" => Self::HomeFax,
            "work;fax" | "work fax" | "workfax" => Self::WorkFax,
            "pager" => Self::Pager,
            "other" => Self::Other,
            "googlevoice" | "google voice" => Self::Custom("Google Voice".to_string()),
            _ => Self::Custom(s.to_string()),
        }
    }

    /// Convert to string for display
    pub fn as_str(&self) -> &str {
        match self {
            Self::Home => "Home",
            Self::Work => "Work",
            Self::Mobile => "Mobile",
            Self::Main => "Main",
            Self::HomeFax => "Home Fax",
            Self::WorkFax => "Work Fax",
            Self::Pager => "Pager",
            Self::Other => "Other",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Get the string value (owned)
    pub fn to_string_value(&self) -> String {
        self.as_str().to_string()
    }

    /// Convert to vCard TYPE parameter format
    pub fn to_vcard_type(&self) -> String {
        match self {
            Self::Home => "HOME".to_string(),
            Self::Work => "WORK".to_string(),
            Self::Mobile => "CELL".to_string(),
            Self::Main => "MAIN".to_string(),
            Self::HomeFax => "HOME;FAX".to_string(),
            Self::WorkFax => "WORK;FAX".to_string(),
            Self::Pager => "PAGER".to_string(),
            Self::Other => "OTHER".to_string(),
            Self::Custom(s) => s.to_uppercase(),
        }
    }
}

impl Default for PhoneLabel {
    fn default() -> Self {
        Self::Mobile
    }
}

/// Common address labels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressLabel {
    /// Home address
    Home,
    /// Work address
    Work,
    /// Other address
    Other,
    /// Custom label
    Custom(String),
}

impl AddressLabel {
    /// Get common address label options for dropdowns
    pub fn common_options() -> Vec<&'static str> {
        vec!["Home", "Work", "Other"]
    }

    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "home" => Self::Home,
            "work" => Self::Work,
            "other" => Self::Other,
            _ => Self::Custom(s.to_string()),
        }
    }

    /// Convert to string for display
    pub fn as_str(&self) -> &str {
        match self {
            Self::Home => "Home",
            Self::Work => "Work",
            Self::Other => "Other",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Get the string value (owned)
    pub fn to_string_value(&self) -> String {
        self.as_str().to_string()
    }
}

impl Default for AddressLabel {
    fn default() -> Self {
        Self::Home
    }
}

/// Common date/event labels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateLabel {
    /// Birthday
    Birthday,
    /// Anniversary
    Anniversary,
    /// Other significant date
    Other,
    /// Custom label
    Custom(String),
}

impl DateLabel {
    /// Get common date label options for dropdowns
    pub fn common_options() -> Vec<&'static str> {
        vec!["Birthday", "Anniversary", "Other"]
    }

    /// Parse from string (case-insensitive)
    /// Handles Apple's special format: _$!<Anniversary>!$_
    pub fn from_str(s: &str) -> Self {
        // Strip Apple's special label wrapper
        let cleaned = s
            .trim_start_matches("_$!<")
            .trim_end_matches(">!$_")
            .to_lowercase();

        match cleaned.as_str() {
            "birthday" | "bday" => Self::Birthday,
            "anniversary" => Self::Anniversary,
            "other" => Self::Other,
            _ => Self::Custom(s.to_string()),
        }
    }

    /// Convert to string for display
    pub fn as_str(&self) -> &str {
        match self {
            Self::Birthday => "Birthday",
            Self::Anniversary => "Anniversary",
            Self::Other => "Other",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Get the string value (owned)
    pub fn to_string_value(&self) -> String {
        self.as_str().to_string()
    }

    /// Convert to Apple's special label format for vCard export
    pub fn to_apple_format(&self) -> String {
        match self {
            Self::Anniversary => "_$!<Anniversary>!$_".to_string(),
            Self::Custom(s) => format!("_$!<{}>!$_", s),
            _ => self.to_string_value(),
        }
    }
}

impl Default for DateLabel {
    fn default() -> Self {
        Self::Birthday
    }
}

/// Common URL labels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UrlLabel {
    /// Homepage
    HomePage,
    /// Work website
    Work,
    /// Blog
    Blog,
    /// Profile page
    Profile,
    /// GitHub profile
    GitHub,
    /// LinkedIn profile
    LinkedIn,
    /// Twitter/X profile
    Twitter,
    /// Facebook profile
    Facebook,
    /// Instagram profile
    Instagram,
    /// Mastodon profile
    Mastodon,
    /// Other URL
    Other,
    /// Custom label
    Custom(String),
}

impl UrlLabel {
    /// Get common URL label options for dropdowns
    pub fn common_options() -> Vec<&'static str> {
        vec![
            "HomePage",
            "Work",
            "Blog",
            "Profile",
            "GitHub",
            "LinkedIn",
            "Twitter",
            "Facebook",
            "Instagram",
            "Mastodon",
            "Other",
        ]
    }

    /// Parse from string (case-insensitive)
    /// Handles Apple's special format: _$!<HomePage>!$_
    pub fn from_str(s: &str) -> Self {
        // Strip Apple's special label wrapper
        let cleaned = s
            .trim_start_matches("_$!<")
            .trim_end_matches(">!$_")
            .to_lowercase();

        match cleaned.as_str() {
            "homepage" | "home page" => Self::HomePage,
            "work" => Self::Work,
            "blog" => Self::Blog,
            "profile" => Self::Profile,
            "github" => Self::GitHub,
            "linkedin" => Self::LinkedIn,
            "twitter" | "x" => Self::Twitter,
            "facebook" => Self::Facebook,
            "instagram" => Self::Instagram,
            "mastodon" => Self::Mastodon,
            "other" => Self::Other,
            _ => Self::Custom(s.to_string()),
        }
    }

    /// Convert to string for display
    pub fn as_str(&self) -> &str {
        match self {
            Self::HomePage => "HomePage",
            Self::Work => "Work",
            Self::Blog => "Blog",
            Self::Profile => "Profile",
            Self::GitHub => "GitHub",
            Self::LinkedIn => "LinkedIn",
            Self::Twitter => "Twitter",
            Self::Facebook => "Facebook",
            Self::Instagram => "Instagram",
            Self::Mastodon => "Mastodon",
            Self::Other => "Other",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Get the string value (owned)
    pub fn to_string_value(&self) -> String {
        self.as_str().to_string()
    }

    /// Check if this is a social media platform
    pub fn is_social_media(&self) -> bool {
        matches!(
            self,
            Self::GitHub
                | Self::LinkedIn
                | Self::Twitter
                | Self::Facebook
                | Self::Instagram
                | Self::Mastodon
        )
    }

    /// Convert to Apple's special label format for vCard export (if needed)
    pub fn to_apple_format(&self) -> String {
        match self {
            Self::HomePage => "_$!<HomePage>!$_".to_string(),
            _ => self.to_string_value(),
        }
    }
}

impl Default for UrlLabel {
    fn default() -> Self {
        Self::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_label_parsing() {
        assert_eq!(EmailLabel::from_str("home"), EmailLabel::Home);
        assert_eq!(EmailLabel::from_str("WORK"), EmailLabel::Work);
        assert_eq!(
            EmailLabel::from_str("Custom"),
            EmailLabel::Custom("Custom".to_string())
        );
    }

    #[test]
    fn test_phone_label_parsing() {
        assert_eq!(PhoneLabel::from_str("mobile"), PhoneLabel::Mobile);
        assert_eq!(PhoneLabel::from_str("CELL"), PhoneLabel::Mobile);
        assert_eq!(PhoneLabel::from_str("home;fax"), PhoneLabel::HomeFax);
        assert_eq!(
            PhoneLabel::from_str("googleVoice"),
            PhoneLabel::Custom("Google Voice".to_string())
        );
    }

    #[test]
    fn test_phone_label_vcard_type() {
        assert_eq!(PhoneLabel::Mobile.to_vcard_type(), "CELL");
        assert_eq!(PhoneLabel::HomeFax.to_vcard_type(), "HOME;FAX");
        assert_eq!(
            PhoneLabel::Custom("Google Voice".to_string()).to_vcard_type(),
            "GOOGLE VOICE"
        );
    }

    #[test]
    fn test_date_label_apple_format() {
        assert_eq!(DateLabel::from_str("_$!<Anniversary>!$_"), DateLabel::Anniversary);
        assert_eq!(DateLabel::Anniversary.to_apple_format(), "_$!<Anniversary>!$_");
    }

    #[test]
    fn test_url_label_apple_format() {
        assert_eq!(UrlLabel::from_str("_$!<HomePage>!$_"), UrlLabel::HomePage);
        assert_eq!(UrlLabel::HomePage.to_apple_format(), "_$!<HomePage>!$_");
    }

    #[test]
    fn test_url_label_social_media() {
        assert!(UrlLabel::GitHub.is_social_media());
        assert!(UrlLabel::LinkedIn.is_social_media());
        assert!(!UrlLabel::Blog.is_social_media());
        assert!(!UrlLabel::HomePage.is_social_media());
    }

    #[test]
    fn test_label_common_options() {
        assert!(EmailLabel::common_options().contains(&"Home"));
        assert!(PhoneLabel::common_options().contains(&"Mobile"));
        assert!(AddressLabel::common_options().contains(&"Work"));
        assert!(DateLabel::common_options().contains(&"Birthday"));
        assert!(UrlLabel::common_options().contains(&"GitHub"));
    }
}