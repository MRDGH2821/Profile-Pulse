use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProfileId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContactId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebsiteLink {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailAddress {
    pub label: String,
    pub address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhoneNumber {
    pub label: String,
    pub number: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contact {
    pub id: ContactId,
    pub profile_id: ProfileId,
    pub display_name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub emails: Vec<EmailAddress>,
    pub phones: Vec<PhoneNumber>,
    pub websites: Vec<WebsiteLink>,
    pub photo_content_hash: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSettings {
    pub scheduled_backup_enabled: bool,
    pub scheduled_backup_dir: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SyncTargetConfig {
    Google {
        enabled: bool,
    },
    Outlook {
        enabled: bool,
    },
    CardDav {
        enabled: bool,
        url: String,
    },
    AppleIcloud {
        enabled: bool,
    },
    OsContacts {
        enabled: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub id: ProfileId,
    pub name: String,
    pub slug: String,
    pub settings: ProfileSettings,
    pub sync_targets: Vec<SyncTargetConfig>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncDirection {
    Push,
    Pull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PullConflictResolution {
    KeepLocal,
    TakeRemote,
    Review,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contact_round_trips_serde() {
        let contact = Contact {
            id: ContactId(Uuid::new_v4()),
            profile_id: ProfileId(Uuid::new_v4()),
            display_name: "Ada Lovelace".into(),
            given_name: Some("Ada".into()),
            family_name: Some("Lovelace".into()),
            emails: vec![EmailAddress {
                label: "work".into(),
                address: "ada@example.com".into(),
            }],
            phones: vec![],
            websites: vec![WebsiteLink {
                label: "GitHub".into(),
                url: "https://github.com/octocat".into(),
            }],
            photo_content_hash: None,
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&contact).unwrap();
        let back: Contact = serde_json::from_str(&json).unwrap();
        assert_eq!(contact, back);
    }
}
