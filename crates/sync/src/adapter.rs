use async_trait::async_trait;
use chrono::{DateTime, Utc};
use profile_pulse_core::Contact;

use crate::error::SyncError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteChange {
    pub remote_id: String,
    pub display_name: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushResult {
    pub target_kind: String,
    pub remote_id: String,
}

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
