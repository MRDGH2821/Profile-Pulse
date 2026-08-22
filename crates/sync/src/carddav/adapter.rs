use async_trait::async_trait;
use chrono::{DateTime, Utc};
use profile_pulse_core::{contact_from_vcard_bytes, Contact, ContactId};
use reqwest::Client;

use crate::adapter::{RemoteChange, SyncAdapter};
use crate::error::SyncError;
use crate::secrets::SecretStore;

pub fn carddav_secret_key(profile_id: profile_pulse_core::ProfileId) -> String {
    format!("carddav:{}", profile_id.0)
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CardDavCredentials {
    pub username: String,
    pub password: String,
}

pub struct CardDavAdapter {
    client: Client,
    secrets: SecretStore,
    profile_id: profile_pulse_core::ProfileId,
    base_url: String,
}

impl CardDavAdapter {
    pub fn new(
        secrets: SecretStore,
        profile_id: profile_pulse_core::ProfileId,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            secrets,
            profile_id,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    fn credentials(&self) -> Result<CardDavCredentials, SyncError> {
        let raw = self
            .secrets
            .get(&carddav_secret_key(self.profile_id))?
            .ok_or_else(|| SyncError::AuthRequired("CardDAV".into()))?;
        serde_json::from_str(&raw).map_err(|e| SyncError::Storage(e.to_string()))
    }

    fn vcard_url(&self, remote_id: &str) -> String {
        if remote_id.starts_with("http://") || remote_id.starts_with("https://") {
            remote_id.to_string()
        } else {
            format!("{}/{}.vcf", self.base_url, remote_id)
        }
    }
}

#[async_trait]
impl SyncAdapter for CardDavAdapter {
    fn target_kind(&self) -> &'static str {
        "carddav"
    }

    async fn push_contact(
        &self,
        contact: &Contact,
        vcard_bytes: &[u8],
        existing_remote_id: Option<&str>,
    ) -> Result<String, SyncError> {
        let credentials = self.credentials()?;
        let remote_id = existing_remote_id
            .map(str::to_string)
            .unwrap_or_else(|| contact.id.0.to_string());
        let url = self.vcard_url(&remote_id);

        let response = self
            .client
            .put(&url)
            .basic_auth(&credentials.username, Some(&credentials.password))
            .header("Content-Type", "text/vcard; charset=utf-8")
            .body(vcard_bytes.to_vec())
            .send()
            .await
            .map_err(|e| SyncError::Http(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(SyncError::Remote(format!("CardDAV {status}: {text}")));
        }
        Ok(remote_id)
    }

    async fn pull_contact(&self, remote_id: &str) -> Result<(Contact, Vec<u8>), SyncError> {
        let credentials = self.credentials()?;
        let url = self.vcard_url(remote_id);
        let response = self
            .client
            .get(&url)
            .basic_auth(&credentials.username, Some(&credentials.password))
            .send()
            .await
            .map_err(|e| SyncError::Http(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(SyncError::Remote(format!("CardDAV {status}: {text}")));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| SyncError::Http(e.to_string()))?
            .to_vec();
        let contact_id = ContactId(uuid::Uuid::new_v4());
        let contact =
            contact_from_vcard_bytes(self.profile_id, contact_id, &bytes)?;
        Ok((contact, bytes))
    }

    async fn check_remote_changes(
        &self,
        _since: DateTime<Utc>,
    ) -> Result<Vec<RemoteChange>, SyncError> {
        Ok(vec![])
    }
}
