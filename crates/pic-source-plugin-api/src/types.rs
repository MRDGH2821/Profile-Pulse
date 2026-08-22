use profile_pulse_core::{PicSourcePluginId, WebsiteLink};
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicSourcePluginMetadata {
    pub id: PicSourcePluginId,
    pub name: String,
    pub version: Version,
    pub min_host_version: Version,
    pub website_match: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactContext {
    pub emails: Vec<String>,
    pub websites: Vec<WebsiteLink>,
    pub existing_photo_hash: Option<String>,
}

impl ContactContext {
    pub fn from_contact(contact: &profile_pulse_core::Contact) -> Self {
        Self {
            emails: contact.emails.iter().map(|e| e.address.clone()).collect(),
            websites: contact.websites.clone(),
            existing_photo_hash: contact.photo_content_hash.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProfilePicCandidate {
    pub source_key: String,
    pub label: String,
    pub preview_url: Option<String>,
    pub fetch_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilePicBytes {
    pub content_type: String,
    pub bytes: Vec<u8>,
}
