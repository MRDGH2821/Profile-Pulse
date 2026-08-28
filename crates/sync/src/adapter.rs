use crate::conflict::RemoteChange;
use crate::error::SyncError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use profile_pulse_core::Contact;

#[async_trait]
pub trait SyncAdapter: Send + Sync {
    fn target_kind(&self) -> &'static str;
    async fn push_contact(
        &self,
        contact: &Contact,
        vcard_bytes: &[u8],
        existing_remote_id: Option<&str>,
    ) -> Result<String, SyncError>;
    async fn pull_contact(&self, remote_id: &str) -> Result<(Contact, Vec<u8>), SyncError>;
    async fn check_remote_changes(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<RemoteChange>, SyncError>;
}
