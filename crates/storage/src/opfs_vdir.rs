use crate::error::StorageError;
use crate::opfs::vfs;
use crate::traits::StorageBackend;
use async_trait::async_trait;
use profile_pulse_core::{Contact, ContactId, Profile, ProfileId, contact_from_vcard_bytes};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SlugManifest {
    slugs: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ContactManifest {
    contact_ids: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct OpfsVdirBackend;

impl OpfsVdirBackend {
    pub fn new() -> Self {
        Self
    }

    fn profile_toml_path(slug: &str) -> String {
        format!("profiles/{slug}/profile.toml")
    }

    fn contacts_manifest_path(slug: &str) -> String {
        format!("profiles/{slug}/contacts/manifest.json")
    }

    fn contact_vcf_path(slug: &str, id: ContactId) -> String {
        format!("profiles/{slug}/contacts/{}.vcf", id.0)
    }

    async fn read_slug_manifest(&self) -> Result<SlugManifest, StorageError> {
        match vfs::read_string("profiles/manifest.json").await {
            Ok(text) => {
                serde_json::from_str(&text).map_err(|err| StorageError::Web(err.to_string()))
            }
            Err(err) if err.contains("not found") => Ok(SlugManifest::default()),
            Err(err) => Err(StorageError::Web(err)),
        }
    }

    async fn write_slug_manifest(&self, manifest: &SlugManifest) -> Result<(), StorageError> {
        vfs::ensure_dir("profiles")
            .await
            .map_err(StorageError::Web)?;
        let text = serde_json::to_string_pretty(manifest)
            .map_err(|err| StorageError::Web(err.to_string()))?;
        vfs::write_string("profiles/manifest.json", &text)
            .await
            .map_err(StorageError::Web)
    }

    async fn read_contact_manifest(&self, slug: &str) -> Result<ContactManifest, StorageError> {
        match vfs::read_string(&Self::contacts_manifest_path(slug)).await {
            Ok(text) => {
                serde_json::from_str(&text).map_err(|err| StorageError::Web(err.to_string()))
            }
            Err(err) if err.contains("not found") => Ok(ContactManifest::default()),
            Err(err) => Err(StorageError::Web(err)),
        }
    }

    async fn write_contact_manifest(
        &self,
        slug: &str,
        manifest: &ContactManifest,
    ) -> Result<(), StorageError> {
        vfs::ensure_dir(&format!("profiles/{slug}/contacts"))
            .await
            .map_err(StorageError::Web)?;
        let text = serde_json::to_string_pretty(manifest)
            .map_err(|err| StorageError::Web(err.to_string()))?;
        vfs::write_string(&Self::contacts_manifest_path(slug), &text)
            .await
            .map_err(StorageError::Web)
    }

    async fn profile_slug_for_id(&self, id: ProfileId) -> Result<String, StorageError> {
        for slug in self.read_slug_manifest().await?.slugs {
            let path = Self::profile_toml_path(&slug);
            let text = vfs::read_string(&path).await.map_err(StorageError::Web)?;
            let profile: Profile = toml::from_str(&text)?;
            if profile.id == id {
                return Ok(slug);
            }
        }
        Err(StorageError::ProfileNotFound(id))
    }

    async fn track_slug(&self, slug: &str) -> Result<(), StorageError> {
        let mut manifest = self.read_slug_manifest().await?;
        if !manifest.slugs.iter().any(|entry| entry == slug) {
            manifest.slugs.push(slug.to_string());
            manifest.slugs.sort();
            self.write_slug_manifest(&manifest).await?;
        }
        Ok(())
    }

    async fn track_contact(&self, slug: &str, id: ContactId) -> Result<(), StorageError> {
        let id_text = id.0.to_string();
        let mut manifest = self.read_contact_manifest(slug).await?;
        if !manifest.contact_ids.iter().any(|entry| entry == &id_text) {
            manifest.contact_ids.push(id_text);
            manifest.contact_ids.sort();
            self.write_contact_manifest(slug, &manifest).await?;
        }
        Ok(())
    }

    async fn untrack_contact(&self, slug: &str, id: ContactId) -> Result<(), StorageError> {
        let id_text = id.0.to_string();
        let mut manifest = self.read_contact_manifest(slug).await?;
        manifest.contact_ids.retain(|entry| entry != &id_text);
        self.write_contact_manifest(slug, &manifest).await
    }
}

#[async_trait]
impl StorageBackend for OpfsVdirBackend {
    async fn list_profiles(&self) -> Result<Vec<Profile>, StorageError> {
        let mut profiles = Vec::new();
        for slug in self.read_slug_manifest().await?.slugs {
            let path = Self::profile_toml_path(&slug);
            if vfs::exists(&path).await.map_err(StorageError::Web)? {
                let text = vfs::read_string(&path).await.map_err(StorageError::Web)?;
                profiles.push(toml::from_str(&text)?);
            }
        }
        Ok(profiles)
    }

    async fn load_profile(&self, id: ProfileId) -> Result<Profile, StorageError> {
        let slug = self.profile_slug_for_id(id).await?;
        let text = vfs::read_string(&Self::profile_toml_path(&slug))
            .await
            .map_err(StorageError::Web)?;
        Ok(toml::from_str(&text)?)
    }

    async fn save_profile(&self, profile: &Profile) -> Result<(), StorageError> {
        vfs::ensure_dir(&format!("profiles/{}/contacts", profile.slug))
            .await
            .map_err(StorageError::Web)?;
        vfs::ensure_dir(&format!("profiles/{}/backups", profile.slug))
            .await
            .map_err(StorageError::Web)?;
        let text = toml::to_string_pretty(profile)?;
        vfs::write_string(&Self::profile_toml_path(&profile.slug), &text)
            .await
            .map_err(StorageError::Web)?;
        self.track_slug(&profile.slug).await
    }

    async fn list_contact_ids(
        &self,
        profile_id: ProfileId,
    ) -> Result<Vec<ContactId>, StorageError> {
        let slug = self.profile_slug_for_id(profile_id).await?;
        let manifest = self.read_contact_manifest(&slug).await?;
        manifest
            .contact_ids
            .iter()
            .map(|id| {
                uuid::Uuid::parse_str(id)
                    .map(ContactId)
                    .map_err(|err| StorageError::Web(err.to_string()))
            })
            .collect()
    }

    async fn load_contact(
        &self,
        profile_id: ProfileId,
        id: ContactId,
    ) -> Result<Contact, StorageError> {
        let slug = self.profile_slug_for_id(profile_id).await?;
        let path = Self::contact_vcf_path(&slug, id);
        if !vfs::exists(&path).await.map_err(StorageError::Web)? {
            return Err(StorageError::ContactNotFound(id));
        }
        let bytes = vfs::read_bytes(&path).await.map_err(StorageError::Web)?;
        contact_from_vcard_bytes(profile_id, id, &bytes).map_err(StorageError::Core)
    }

    async fn save_contact(
        &self,
        contact: &Contact,
        vcard_bytes: &[u8],
    ) -> Result<(), StorageError> {
        let profile = self.load_profile(contact.profile_id).await?;
        vfs::ensure_dir(&format!("profiles/{}/contacts", profile.slug))
            .await
            .map_err(StorageError::Web)?;
        vfs::write_bytes(
            &Self::contact_vcf_path(&profile.slug, contact.id),
            vcard_bytes,
        )
        .await
        .map_err(StorageError::Web)?;
        self.track_contact(&profile.slug, contact.id).await
    }

    async fn delete_contact(
        &self,
        profile_id: ProfileId,
        id: ContactId,
    ) -> Result<(), StorageError> {
        let slug = self.profile_slug_for_id(profile_id).await?;
        let path = Self::contact_vcf_path(&slug, id);
        if !vfs::exists(&path).await.map_err(StorageError::Web)? {
            return Err(StorageError::ContactNotFound(id));
        }
        vfs::remove_file(&path).await.map_err(StorageError::Web)?;
        self.untrack_contact(&slug, id).await
    }

    async fn export_profile_vcf_aggregate(
        &self,
        profile_id: ProfileId,
    ) -> Result<Vec<u8>, StorageError> {
        let ids = self.list_contact_ids(profile_id).await?;
        let mut out = Vec::new();
        for (index, id) in ids.iter().enumerate() {
            let contact = self.load_contact(profile_id, *id).await?;
            let profile = self.load_profile(profile_id).await?;
            let path = Self::contact_vcf_path(&profile.slug, contact.id);
            let bytes = vfs::read_bytes(&path).await.map_err(StorageError::Web)?;
            if index > 0 {
                out.extend_from_slice(b"\r\n");
            }
            out.extend_from_slice(&bytes);
        }
        Ok(out)
    }

    async fn import_vcf_into_profile(
        &self,
        profile_id: ProfileId,
        vcf_bytes: &[u8],
    ) -> Result<Vec<ContactId>, StorageError> {
        use profile_pulse_core::import_contacts_from_vcf;

        let contacts = import_contacts_from_vcf(profile_id, vcf_bytes)?;
        let mut ids = Vec::with_capacity(contacts.len());
        for contact in contacts {
            let vcard_bytes = profile_pulse_core::contact_to_vcard_bytes(&contact)
                .map_err(|e| StorageError::Vcard(e.to_string()))?;
            self.save_contact(&contact, &vcard_bytes).await?;
            ids.push(contact.id);
        }
        Ok(ids)
    }
}
