use crate::error::StorageError;
use crate::traits::{ContactIndex, StorageBackend};
use profile_pulse_core::{contact_to_vcard_bytes, BackupService, Contact};
use std::path::PathBuf;
use std::sync::Arc;

pub struct ContactService<B, I> {
    storage: Arc<B>,
    index: Arc<I>,
    backup: BackupService,
}

impl<B, I> ContactService<B, I>
where
    B: StorageBackend,
    I: ContactIndex,
{
    pub fn new(storage: Arc<B>, index: Arc<I>, data_root: PathBuf) -> Self {
        let profiles_root = data_root.join("profiles");
        Self {
            storage,
            index,
            backup: BackupService::new(profiles_root),
        }
    }

    pub async fn update_contact(&self, contact: Contact) -> Result<(), StorageError> {
        let profile = self.storage.load_profile(contact.profile_id).await?;
        let _backup = self
            .backup
            .snapshot_profile_before_write(&profile.slug)
            .await
            .map_err(|e| StorageError::Vcard(e.to_string()))?;
        let vcard_bytes = contact_to_vcard_bytes(&contact).map_err(|e| StorageError::Vcard(e.to_string()))?;
        self.storage.save_contact(&contact, &vcard_bytes).await?;
        self.index.upsert_contact(&contact).await?;
        Ok(())
    }
}
