use crate::error::StorageError;
use crate::traits::StorageBackend;
use async_trait::async_trait;
use profile_pulse_core::{
    contact_from_vcard_bytes, Contact, ContactId, Profile, ProfileId,
};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone)]
pub struct FsVdirBackend {
    root: PathBuf,
}

impl FsVdirBackend {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn profiles_dir(&self) -> PathBuf {
        self.root.join("profiles")
    }

    async fn profile_dir_for_slug(&self, slug: &str) -> PathBuf {
        self.profiles_dir().join(slug)
    }

    async fn find_profile_dir(&self, id: ProfileId) -> Result<PathBuf, StorageError> {
        let profiles_dir = self.profiles_dir();
        if !profiles_dir.exists() {
            return Err(StorageError::ProfileNotFound(id));
        }

        let mut entries = fs::read_dir(&profiles_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let profile_path = path.join("profile.toml");
            if !profile_path.exists() {
                continue;
            }
            let text = fs::read_to_string(&profile_path).await?;
            let profile: Profile = toml::from_str(&text)?;
            if profile.id == id {
                return Ok(path);
            }
        }
        Err(StorageError::ProfileNotFound(id))
    }
}

#[async_trait]
impl StorageBackend for FsVdirBackend {
    async fn list_profiles(&self) -> Result<Vec<Profile>, StorageError> {
        let profiles_dir = self.profiles_dir();
        if !profiles_dir.exists() {
            return Ok(vec![]);
        }

        let mut profiles = Vec::new();
        let mut entries = fs::read_dir(&profiles_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let profile_path = entry.path().join("profile.toml");
            if profile_path.exists() {
                let text = fs::read_to_string(&profile_path).await?;
                profiles.push(toml::from_str(&text)?);
            }
        }
        Ok(profiles)
    }

    async fn load_profile(&self, id: ProfileId) -> Result<Profile, StorageError> {
        let dir = self.find_profile_dir(id).await?;
        let text = fs::read_to_string(dir.join("profile.toml")).await?;
        Ok(toml::from_str(&text)?)
    }

    async fn save_profile(&self, profile: &Profile) -> Result<(), StorageError> {
        let dir = self.profile_dir_for_slug(&profile.slug).await;
        fs::create_dir_all(dir.join("contacts")).await?;
        fs::create_dir_all(dir.join("backups")).await?;
        let text = toml::to_string_pretty(profile)?;
        fs::write(dir.join("profile.toml"), text).await?;
        Ok(())
    }

    async fn list_contact_ids(&self, profile_id: ProfileId) -> Result<Vec<ContactId>, StorageError> {
        let dir = self.find_profile_dir(profile_id).await?;
        let contacts_dir = dir.join("contacts");
        if !contacts_dir.exists() {
            return Ok(vec![]);
        }

        let mut ids = Vec::new();
        let mut entries = fs::read_dir(contacts_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("vcf") {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(uuid) = uuid::Uuid::parse_str(stem) {
                    ids.push(ContactId(uuid));
                }
            }
        }
        Ok(ids)
    }

    async fn load_contact(
        &self,
        profile_id: ProfileId,
        id: ContactId,
    ) -> Result<Contact, StorageError> {
        let dir = self.find_profile_dir(profile_id).await?;
        let path = dir.join("contacts").join(format!("{}.vcf", id.0));
        if !path.exists() {
            return Err(StorageError::ContactNotFound(id));
        }
        let bytes = fs::read(&path).await?;
        contact_from_vcard_bytes(profile_id, id, &bytes).map_err(StorageError::Core)
    }

    async fn save_contact(&self, contact: &Contact, vcard_bytes: &[u8]) -> Result<(), StorageError> {
        let profile = self.load_profile(contact.profile_id).await?;
        let dir = self.profile_dir_for_slug(&profile.slug).await;
        fs::create_dir_all(dir.join("contacts")).await?;
        let path = dir.join("contacts").join(format!("{}.vcf", contact.id.0));
        fs::write(path, vcard_bytes).await?;
        Ok(())
    }

    async fn delete_contact(
        &self,
        profile_id: ProfileId,
        id: ContactId,
    ) -> Result<(), StorageError> {
        let dir = self.find_profile_dir(profile_id).await?;
        let path = dir.join("contacts").join(format!("{}.vcf", id.0));
        if !path.exists() {
            return Err(StorageError::ContactNotFound(id));
        }
        fs::remove_file(path).await?;
        Ok(())
    }

    async fn export_profile_vcf_aggregate(
        &self,
        profile_id: ProfileId,
    ) -> Result<Vec<u8>, StorageError> {
        let dir = self.find_profile_dir(profile_id).await?;
        let contacts_dir = dir.join("contacts");
        let mut chunks = Vec::new();
        if contacts_dir.exists() {
            let mut entries = fs::read_dir(contacts_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("vcf") {
                    let bytes = fs::read(&path).await?;
                    chunks.push(bytes);
                }
            }
        }
        let mut out = Vec::new();
        for (i, chunk) in chunks.iter().enumerate() {
            if i > 0 {
                out.extend_from_slice(b"\r\n");
            }
            out.extend_from_slice(chunk);
        }
        Ok(out)
    }
}
