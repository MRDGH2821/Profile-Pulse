use crate::adapter::SyncAdapter;
use crate::carddav::CardDavAdapter;
use crate::credentials::{CardDavCredentials, PushResult, carddav_secret_key};
use crate::error::SyncError;
use crate::google::{GoogleContactsAdapter, authorize_google_pkce};
use crate::links::SyncLinkStore;
use crate::secrets::SecretStore;
use chrono::Utc;
use profile_pulse_core::{
    Contact, ContactId, Profile, ProfileId, SyncTargetConfig, contact_to_vcard_bytes,
};
use std::path::Path;
use std::sync::Arc;

pub struct SyncService {
    secrets: SecretStore,
    links: SyncLinkStore,
    google_client_id: String,
}

impl SyncService {
    pub fn new(data_root: impl AsRef<Path>) -> Result<Self, SyncError> {
        let data_root = data_root.as_ref().to_path_buf();
        Ok(Self {
            secrets: SecretStore::new(&data_root),
            links: SyncLinkStore::new(data_root.join("sync_links.sqlite"))?,
            google_client_id: std::env::var("PROFILE_PULSE_GOOGLE_CLIENT_ID").unwrap_or_default(),
        })
    }

    pub fn secrets(&self) -> &SecretStore {
        &self.secrets
    }

    pub fn google_client_id(&self) -> &str {
        &self.google_client_id
    }

    pub async fn link_google(&self, profile_id: ProfileId) -> Result<(), SyncError> {
        authorize_google_pkce(&self.google_client_id, &self.secrets, profile_id).await?;
        Ok(())
    }

    pub fn save_carddav_credentials(
        &self,
        profile_id: ProfileId,
        credentials: &CardDavCredentials,
    ) -> Result<(), SyncError> {
        let raw =
            serde_json::to_string(credentials).map_err(|e| SyncError::Storage(e.to_string()))?;
        self.secrets.put(&carddav_secret_key(profile_id), &raw)
    }

    fn adapter_for(
        &self,
        profile_id: ProfileId,
        target: &SyncTargetConfig,
    ) -> Option<Arc<dyn SyncAdapter>> {
        match target {
            SyncTargetConfig::Google { enabled: true } => {
                Some(Arc::new(GoogleContactsAdapter::new(
                    self.secrets.clone(),
                    profile_id,
                    self.google_client_id.clone(),
                )))
            }
            SyncTargetConfig::CardDav { enabled: true, url } if !url.trim().is_empty() => {
                Some(Arc::new(CardDavAdapter::new(
                    self.secrets.clone(),
                    profile_id,
                    url.clone(),
                )))
            }
            _ => None,
        }
    }

    pub fn enabled_targets(profile: &Profile) -> Vec<(String, SyncTargetConfig)> {
        profile
            .sync_targets
            .iter()
            .filter(|target| target.is_enabled())
            .map(|target| (target.kind_label().to_string(), target.clone()))
            .collect()
    }

    /// Push-only by default — used by the per-contact Sync button.
    pub async fn push_contact(
        &self,
        profile: &Profile,
        contact: &Contact,
    ) -> Result<Vec<PushResult>, SyncError> {
        let vcard = contact_to_vcard_bytes(contact)?;
        let mut results = Vec::new();
        for (_label, target) in Self::enabled_targets(profile) {
            let Some(adapter) = self.adapter_for(profile.id, &target) else {
                continue;
            };
            let kind = adapter.target_kind();
            let existing = self.links.get_remote_id(profile.id, contact.id, kind)?;
            let remote_id = adapter
                .push_contact(contact, &vcard, existing.as_deref())
                .await?;
            self.links
                .upsert_link(profile.id, contact.id, kind, &remote_id, Utc::now())?;
            results.push(PushResult {
                target_kind: kind.to_string(),
                remote_id,
            });
        }
        if results.is_empty() {
            return Err(SyncError::NotConfigured(
                "enable Google or CardDAV sync targets for this profile".into(),
            ));
        }
        Ok(results)
    }

    pub async fn pull_contact(
        &self,
        profile: &Profile,
        target_kind: &str,
        contact_id: ContactId,
        remote_id: &str,
    ) -> Result<Contact, SyncError> {
        let target = profile
            .sync_targets
            .iter()
            .find(|t| t.kind_label() == target_kind && t.is_enabled())
            .cloned()
            .ok_or_else(|| SyncError::NotConfigured(format!("{target_kind} not enabled")))?;
        let adapter = self
            .adapter_for(profile.id, &target)
            .ok_or_else(|| SyncError::NotConfigured(format!("{target_kind} not configured")))?;
        let (mut contact, _vcard) = adapter.pull_contact(remote_id).await?;
        contact.id = contact_id;
        contact.profile_id = profile.id;
        self.links
            .upsert_link(profile.id, contact_id, target_kind, remote_id, Utc::now())?;
        Ok(contact)
    }
}
