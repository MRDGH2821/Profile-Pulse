use crate::error::SyncError;
use chrono::{DateTime, Utc};
use profile_pulse_core::{ContactId, ProfileId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

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
            Some(text) => serde_json::from_str(&text).map_err(|err| SyncError::Storage(err.to_string())),
            None => Ok(LinkData::default()),
        }
    }

    fn save(&self, data: &LinkData) -> Result<(), SyncError> {
        let storage = browser_storage()?;
        let text = serde_json::to_string(data).map_err(|err| SyncError::Storage(err.to_string()))?;
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
        let data = self.load()?;
        Ok(data
            .links
            .get(&profile_id.0.to_string())
            .and_then(|contacts| contacts.get(&contact_id.0.to_string()))
            .and_then(|targets| targets.get(target_kind))
            .map(|record| record.remote_id.clone()))
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
