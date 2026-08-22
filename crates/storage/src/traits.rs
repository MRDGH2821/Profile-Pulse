use crate::error::StorageError;
use async_trait::async_trait;
use profile_pulse_core::{Contact, ContactId, Profile, ProfileId};

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn list_profiles(&self) -> Result<Vec<Profile>, StorageError>;
    async fn load_profile(&self, id: ProfileId) -> Result<Profile, StorageError>;
    async fn save_profile(&self, profile: &Profile) -> Result<(), StorageError>;
    async fn list_contact_ids(&self, profile_id: ProfileId)
    -> Result<Vec<ContactId>, StorageError>;
    async fn load_contact(
        &self,
        profile_id: ProfileId,
        id: ContactId,
    ) -> Result<Contact, StorageError>;
    async fn save_contact(&self, contact: &Contact, vcard_bytes: &[u8])
    -> Result<(), StorageError>;
    async fn delete_contact(
        &self,
        profile_id: ProfileId,
        id: ContactId,
    ) -> Result<(), StorageError>;
    async fn export_profile_vcf_aggregate(
        &self,
        profile_id: ProfileId,
    ) -> Result<Vec<u8>, StorageError>;
    async fn import_vcf_into_profile(
        &self,
        profile_id: ProfileId,
        vcf_bytes: &[u8],
    ) -> Result<Vec<ContactId>, StorageError>;
}

#[async_trait]
pub trait ContactIndex: Send + Sync {
    async fn upsert_contact(&self, contact: &Contact) -> Result<(), StorageError>;
    async fn remove_contact(
        &self,
        profile_id: ProfileId,
        id: ContactId,
    ) -> Result<(), StorageError>;
    async fn search(
        &self,
        profile_id: ProfileId,
        query: &str,
        limit: u32,
    ) -> Result<Vec<ContactId>, StorageError>;
    async fn clear_profile(&self, profile_id: ProfileId) -> Result<(), StorageError>;
}
