use crate::error::SyncError;
use chrono::{DateTime, Utc};
use profile_pulse_core::{ContactId, ProfileId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SyncLink {
    pub contact_id: ContactId,
    pub target_kind: String,
    pub remote_id: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LinkRecord {
    remote_id: String,
    updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LinkData {
    links: HashMap<String, HashMap<String, HashMap<String, LinkRecord>>>,
}

const STORAGE_KEY: &str = "profile-pulse-sync-links";

fn browser_storage() -> Result<web_sys::Storage, SyncError> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .ok_or_else(|| SyncError::Storage("localStorage unavailable".into()))
}
#[derive(Debug, Clone)]
pub struct SyncLinkStore;

impl SyncLinkStore {
    pub fn new(_path: impl AsRef<Path>) -> Result<Self, SyncError> {
        Ok(Self)
    }

    fn load(&self) -> Result<LinkData, SyncError> {
        let storage = browser_storage()?;
        match storage
            .get_item(STORAGE_KEY)
            .map_err(|err| SyncError::Storage(format!("localStorage read failed: {err:?}")))?
        {
            Some(text) => {
                serde_json::from_str(&text).map_err(|err| SyncError::Storage(err.to_string()))
            }
            None => Ok(LinkData::default()),
        }
    }

    fn save(&self, data: &LinkData) -> Result<(), SyncError> {
        let storage = browser_storage()?;
        let text =
            serde_json::to_string(data).map_err(|err| SyncError::Storage(err.to_string()))?;
        storage
            .set_item(STORAGE_KEY, &text)
            .map_err(|err| SyncError::Storage(format!("localStorage write failed: {err:?}")))
    }

    pub fn get_remote_id(
        &self,
        profile_id: ProfileId,
        contact_id: ContactId,
        target_kind: &str,
    ) -> Result<Option<String>, SyncError> {
        Ok(self
            .get_link(profile_id, contact_id, target_kind)?
            .map(|link| link.remote_id))
    }

    pub fn get_link(
        &self,
        profile_id: ProfileId,
        contact_id: ContactId,
        target_kind: &str,
    ) -> Result<Option<SyncLink>, SyncError> {
        let data = self.load()?;
        Ok(data
            .links
            .get(&profile_id.0.to_string())
            .and_then(|contacts| contacts.get(&contact_id.0.to_string()))
            .and_then(|targets| targets.get(target_kind))
            .map(|record| SyncLink {
                contact_id,
                target_kind: target_kind.to_string(),
                remote_id: record.remote_id.clone(),
                updated_at: DateTime::parse_from_rfc3339(&record.updated_at)
                    .map_err(|e| SyncError::Storage(e.to_string()))?
                    .with_timezone(&Utc),
            }))
    }

    pub fn find_contact_by_remote_id(
        &self,
        profile_id: ProfileId,
        target_kind: &str,
        remote_id: &str,
    ) -> Result<Option<ContactId>, SyncError> {
        let data = self.load()?;
        let Some(contacts) = data.links.get(&profile_id.0.to_string()) else {
            return Ok(None);
        };
        for (contact_id, targets) in contacts {
            if let Some(record) = targets.get(target_kind) {
                if record.remote_id == remote_id {
                    let uuid = uuid::Uuid::parse_str(contact_id)
                        .map_err(|e| SyncError::Storage(e.to_string()))?;
                    return Ok(Some(ContactId(uuid)));
                }
            }
        }
        Ok(None)
    }

    pub fn list_links_for_profile(
        &self,
        profile_id: ProfileId,
    ) -> Result<Vec<SyncLink>, SyncError> {
        let data = self.load()?;
        let mut links = Vec::new();
        if let Some(contacts) = data.links.get(&profile_id.0.to_string()) {
            for (contact_id, targets) in contacts {
                let uuid = uuid::Uuid::parse_str(contact_id)
                    .map_err(|e| SyncError::Storage(e.to_string()))?;
                for (target_kind, record) in targets {
                    links.push(SyncLink {
                        contact_id: ContactId(uuid),
                        target_kind: target_kind.clone(),
                        remote_id: record.remote_id.clone(),
                        updated_at: DateTime::parse_from_rfc3339(&record.updated_at)
                            .map_err(|e| SyncError::Storage(e.to_string()))?
                            .with_timezone(&Utc),
                    });
                }
            }
        }
        Ok(links)
    }

    pub fn upsert_link(
        &self,
        profile_id: ProfileId,
        contact_id: ContactId,
        target_kind: &str,
        remote_id: &str,
        updated_at: DateTime<Utc>,
    ) -> Result<(), SyncError> {
        let mut data = self.load()?;
        data.links
            .entry(profile_id.0.to_string())
            .or_default()
            .entry(contact_id.0.to_string())
            .or_default()
            .insert(
                target_kind.to_string(),
                LinkRecord {
                    remote_id: remote_id.to_string(),
                    updated_at: updated_at.to_rfc3339(),
                },
            );
        self.save(&data)
    }
}
