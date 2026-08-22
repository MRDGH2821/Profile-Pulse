use profile_pulse_core::{ContactId, ProfileId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("contact not found: {0}")]
    ContactNotFound(ContactId),
    #[error("profile not found: {0}")]
    ProfileNotFound(ProfileId),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("vcard error: {0}")]
    Vcard(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("toml error: {0}")]
    Toml(#[from] toml::ser::Error),
    #[error("toml parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("core error: {0}")]
    Core(#[from] profile_pulse_core::CoreError),
}
