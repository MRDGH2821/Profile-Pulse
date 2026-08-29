use crate::error::StorageError;
use crate::opfs::vfs;
use crate::traits::ContactIndex;
use async_trait::async_trait;
use profile_pulse_core::{Contact, ContactId, ProfileId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexRow {
    display_name: String,
    search_text: String,
    updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct IndexData {
    profiles: HashMap<String, HashMap<String, IndexRow>>,
}

const INDEX_PATH: &str = "index/contacts.json";

#[derive(Debug, Clone)]
pub struct WebContactIndex;

impl WebContactIndex {
    pub fn new() -> Self {
        Self
    }

    async fn load(&self) -> Result<IndexData, StorageError> {
        match vfs::read_string(INDEX_PATH).await {
            Ok(text) => {
                serde_json::from_str(&text).map_err(|err| StorageError::Web(err.to_string()))
            }
            Err(err) if err.contains("not found") => Ok(IndexData::default()),
            Err(err) => Err(StorageError::Web(err)),
        }
    }

    async fn save(&self, data: &IndexData) -> Result<(), StorageError> {
        vfs::ensure_dir("index").await.map_err(StorageError::Web)?;
        let text =
            serde_json::to_string_pretty(data).map_err(|err| StorageError::Web(err.to_string()))?;
        vfs::write_string(INDEX_PATH, &text)
            .await
            .map_err(StorageError::Web)
    }
}

fn build_search_text(contact: &Contact) -> String {
    let mut parts = vec![contact.display_name.clone()];
    parts.extend(contact.emails.iter().map(|email| email.address.clone()));
    parts.extend(contact.phones.iter().map(|phone| phone.number.clone()));
    parts.extend(contact.websites.iter().map(|site| site.url.clone()));
    parts.join(" ").to_ascii_lowercase()
}

#[async_trait]
impl ContactIndex for WebContactIndex {
    async fn upsert_contact(&self, contact: &Contact) -> Result<(), StorageError> {
        let mut data = self.load().await?;
        let profile_key = contact.profile_id.0.to_string();
        let contact_key = contact.id.0.to_string();
        let rows = data.profiles.entry(profile_key).or_default();
        rows.insert(
            contact_key,
            IndexRow {
                display_name: contact.display_name.clone(),
                search_text: build_search_text(contact),
                updated_at: contact.updated_at.to_rfc3339(),
            },
        );
        self.save(&data).await
    }

    async fn remove_contact(
        &self,
        profile_id: ProfileId,
        id: ContactId,
    ) -> Result<(), StorageError> {
        let mut data = self.load().await?;
        if let Some(rows) = data.profiles.get_mut(&profile_id.0.to_string()) {
            rows.remove(&id.0.to_string());
        }
        self.save(&data).await
    }

    async fn search(
        &self,
        profile_id: ProfileId,
        query: &str,
        limit: u32,
    ) -> Result<Vec<ContactId>, StorageError> {
        let data = self.load().await?;
        let query = query.trim().to_ascii_lowercase();
        let Some(rows) = data.profiles.get(&profile_id.0.to_string()) else {
            return Ok(vec![]);
        };
        let mut matches: Vec<(String, String)> = rows
            .iter()
            .filter(|(_, row)| query.is_empty() || row.search_text.contains(&query))
            .map(|(id, row)| (id.clone(), row.updated_at.clone()))
            .collect();
        matches.sort_by(|left, right| right.1.cmp(&left.1));
        matches
            .into_iter()
            .take(limit as usize)
            .map(|(id, _)| {
                uuid::Uuid::parse_str(&id)
                    .map(ContactId)
                    .map_err(|err| StorageError::Web(err.to_string()))
            })
            .collect()
    }

    async fn clear_profile(&self, profile_id: ProfileId) -> Result<(), StorageError> {
        let mut data = self.load().await?;
        data.profiles.remove(&profile_id.0.to_string());
        self.save(&data).await
    }
}
