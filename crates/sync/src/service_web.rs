use crate::conflict::{
    PullApplyResult, PullConflict, PullPrepareResult, RemoteChange, TargetRemoteChanges,
};
use crate::credentials::{CardDavCredentials, PushResult, carddav_secret_key};
use crate::error::SyncError;
use crate::links_web::SyncLinkStore;
use crate::secrets_web::SecretStore;
use profile_pulse_core::{
    Contact, ContactId, Profile, ProfileId, PullConflictResolution, SyncTargetConfig,
};
use std::path::Path;

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

    pub async fn link_google(&self, _profile_id: ProfileId) -> Result<(), SyncError> {
        Err(SyncError::NotConfigured(
            "Google OAuth linking is not yet available in the web build".into(),
        ))
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

    pub fn enabled_targets(profile: &Profile) -> Vec<(String, SyncTargetConfig)> {
        profile
            .sync_targets
            .iter()
            .filter(|target| target.is_enabled())
            .map(|target| (target.kind_label().to_string(), target.clone()))
            .collect()
    }

    pub async fn push_contact(
        &self,
        _profile: &Profile,
        _contact: &Contact,
    ) -> Result<Vec<PushResult>, SyncError> {
        Err(SyncError::NotConfigured(
            "cloud sync push is not yet available in the web build".into(),
        ))
    }

    pub async fn pull_contact(
        &self,
        _profile: &Profile,
        _target_kind: &str,
        _contact_id: ContactId,
        _remote_id: &str,
    ) -> Result<Contact, SyncError> {
        Err(SyncError::NotConfigured(
            "cloud sync pull is not yet available in the web build".into(),
        ))
    }

    pub async fn poll_remote_changes(
        &self,
        _profile: &Profile,
    ) -> Result<Vec<TargetRemoteChanges>, SyncError> {
        Ok(vec![])
    }

    pub async fn prepare_pull_item(
        &self,
        _profile: &Profile,
        _target_kind: &str,
        _change: &RemoteChange,
        _contact_id: ContactId,
        _local: Option<&Contact>,
    ) -> Result<PullPrepareResult, SyncError> {
        Err(SyncError::NotConfigured(
            "cloud sync pull is not yet available in the web build".into(),
        ))
    }

    pub async fn pull_with_resolution(
        &self,
        _profile: &Profile,
        _conflict: &PullConflict,
        _resolution: PullConflictResolution,
    ) -> Result<(Contact, PullApplyResult), SyncError> {
        Err(SyncError::NotConfigured(
            "cloud sync pull is not yet available in the web build".into(),
        ))
    }

    pub async fn apply_pull_item(
        &self,
        _profile: &Profile,
        _target_kind: &str,
        _remote_id: &str,
        contact: Contact,
    ) -> Result<Contact, SyncError> {
        Ok(contact)
    }

    pub fn links(&self) -> &SyncLinkStore {
        &self.links
    }
}
