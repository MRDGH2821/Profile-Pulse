use crate::error::StorageError;
use crate::profile_bundle::{export_profile_bundle, import_profile_bundle};
use crate::traits::{ContactIndex, StorageBackend};
use chrono::Utc;
use profile_pulse_core::{
    BackupRef, BackupService, Contact, ContactId, Profile, ProfileId, contact_to_vcard_bytes,
};
use profile_pulse_pic_source_plugin_api::ProfilePicBytes;
use std::path::{Path, PathBuf};
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use tokio::fs;

pub struct ContactService<B, I> {
    storage: Arc<B>,
    index: Arc<I>,
    backup: BackupService,
    data_root: PathBuf,
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
            data_root,
        }
    }

    pub fn data_root(&self) -> &Path {
        &self.data_root
    }

    async fn profile_slug(&self, profile_id: ProfileId) -> Result<String, StorageError> {
        Ok(self.storage.load_profile(profile_id).await?.slug)
    }

    pub async fn create_contact(
        &self,
        profile_id: ProfileId,
        display_name: String,
    ) -> Result<Contact, StorageError> {
        let name = display_name.trim();
        if name.is_empty() {
            return Err(StorageError::Vcard("display name is required".into()));
        }
        let contact = Contact {
            id: ContactId(uuid::Uuid::new_v4()),
            profile_id,
            display_name: name.to_string(),
            given_name: None,
            family_name: None,
            emails: vec![],
            phones: vec![],
            websites: vec![],
            photo_content_hash: None,
            updated_at: Utc::now(),
        };
        self.update_contact(contact.clone()).await?;
        Ok(contact)
    }

    pub async fn update_contact(&self, contact: Contact) -> Result<(), StorageError> {
        let profile = self.storage.load_profile(contact.profile_id).await?;
        let _backup = self
            .backup
            .snapshot_profile_before_write(&profile.slug)
            .await
            .map_err(|e| StorageError::Vcard(e.to_string()))?;
        let vcard_bytes =
            contact_to_vcard_bytes(&contact).map_err(|e| StorageError::Vcard(e.to_string()))?;
        self.storage.save_contact(&contact, &vcard_bytes).await?;
        self.index.upsert_contact(&contact).await?;
        Ok(())
    }

    pub async fn delete_contact(
        &self,
        profile_id: ProfileId,
        contact_id: ContactId,
    ) -> Result<(), StorageError> {
        let slug = self.profile_slug(profile_id).await?;
        let _backup = self
            .backup
            .snapshot_profile_before_write(&slug)
            .await
            .map_err(|e| StorageError::Vcard(e.to_string()))?;
        self.storage.delete_contact(profile_id, contact_id).await?;
        self.index.remove_contact(profile_id, contact_id).await?;
        Ok(())
    }

    pub async fn import_vcf(
        &self,
        profile_id: ProfileId,
        vcf_bytes: &[u8],
    ) -> Result<Vec<ContactId>, StorageError> {
        let slug = self.profile_slug(profile_id).await?;
        let _backup = self
            .backup
            .snapshot_profile_before_write(&slug)
            .await
            .map_err(|e| StorageError::Vcard(e.to_string()))?;
        let ids = self
            .storage
            .import_vcf_into_profile(profile_id, vcf_bytes)
            .await?;
        for id in &ids {
            let contact = self.storage.load_contact(profile_id, *id).await?;
            self.index.upsert_contact(&contact).await?;
        }
        Ok(ids)
    }

    pub async fn export_vcf_aggregate(
        &self,
        profile_id: ProfileId,
    ) -> Result<Vec<u8>, StorageError> {
        self.storage.export_profile_vcf_aggregate(profile_id).await
    }

    pub async fn export_profile_bundle(
        &self,
        profile_id: ProfileId,
    ) -> Result<Vec<u8>, StorageError> {
        export_profile_bundle(self.storage.as_ref(), profile_id).await
    }

    pub async fn import_profile_bundle(&self, bytes: &[u8]) -> Result<Profile, StorageError> {
        let profile = import_profile_bundle(self.storage.as_ref(), bytes).await?;
        self.reindex_profile(profile.id).await?;
        Ok(profile)
    }

    pub async fn list_backups(
        &self,
        profile_id: ProfileId,
    ) -> Result<Vec<BackupRef>, StorageError> {
        let slug = self.profile_slug(profile_id).await?;
        self.backup
            .list_profile_backups(&slug)
            .await
            .map_err(|e| StorageError::Vcard(e.to_string()))
    }

    pub async fn restore_backup(
        &self,
        profile_id: ProfileId,
        backup_label: &str,
    ) -> Result<(), StorageError> {
        let slug = self.profile_slug(profile_id).await?;
        self.backup
            .restore_profile_backup(&slug, backup_label)
            .await
            .map_err(|e| StorageError::Vcard(e.to_string()))?;
        self.reindex_profile(profile_id).await
    }

    pub async fn reindex_profile(&self, profile_id: ProfileId) -> Result<(), StorageError> {
        self.index.clear_profile(profile_id).await?;
        let ids = self.storage.list_contact_ids(profile_id).await?;
        for id in ids {
            let contact = self.storage.load_contact(profile_id, id).await?;
            self.index.upsert_contact(&contact).await?;
        }
        Ok(())
    }

    pub async fn update_profile_settings(
        &self,
        mut profile: Profile,
    ) -> Result<Profile, StorageError> {
        profile.updated_at = Utc::now();
        self.storage.save_profile(&profile).await?;
        Ok(profile)
    }

    pub async fn run_scheduled_backups(&self) -> Result<u32, StorageError> {
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(0);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let profiles = self.storage.list_profiles().await?;
            let mut count = 0u32;
            for mut profile in profiles {
                if !profile.settings.scheduled_backup_enabled {
                    continue;
                }
                let Some(dir) = profile.settings.scheduled_backup_dir.clone() else {
                    continue;
                };
                if dir.trim().is_empty() {
                    continue;
                }
                let bundle = self.export_profile_bundle(profile.id).await?;
                fs::create_dir_all(&dir).await?;
                let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ");
                let filename = format!("{}-profile-pulse-{}.pp-profile", profile.slug, timestamp);
                let path = PathBuf::from(&dir).join(filename);
                fs::write(&path, &bundle).await?;
                profile.settings.scheduled_backup_last_run = Some(Utc::now());
                profile.updated_at = Utc::now();
                self.storage.save_profile(&profile).await?;
                count += 1;
            }
            Ok(count)
        }
    }

    pub async fn apply_profile_pic(
        &self,
        profile_id: ProfileId,
        contact_id: ContactId,
        pic: ProfilePicBytes,
    ) -> Result<Contact, StorageError> {
        let mut contact = self.storage.load_contact(profile_id, contact_id).await?;
        let hash = crate::photo_cache::sha256_hex(&pic.bytes);
        crate::photo_cache::store_photo(&self.data_root, &hash, &pic.bytes).await?;
        contact.photo_content_hash = Some(hash);
        contact.updated_at = Utc::now();
        self.update_contact(contact.clone()).await?;
        Ok(contact)
    }
}
