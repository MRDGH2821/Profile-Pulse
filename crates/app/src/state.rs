use dioxus::prelude::*;
use profile_pulse_core::{Profile, ProfileId, ProfileSettings};
use profile_pulse_pic_source_plugin_host::PicSourcePluginRegistry;
use profile_pulse_storage::{
    ContactService, FsVdirBackend, SqliteContactIndex, StorageBackend,
};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Clone, Copy)]
pub struct ActiveProfile(pub Signal<Option<ProfileId>>);

impl ActiveProfile {
    pub fn set(&mut self, profile_id: ProfileId) {
        self.0.set(Some(profile_id));
    }

    pub fn id(&self) -> Option<ProfileId> {
        self.0()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<FsVdirBackend>,
    pub index: Arc<SqliteContactIndex>,
    pub contact_service: Arc<ContactService<FsVdirBackend, SqliteContactIndex>>,
    pub plugin_registry: Arc<RwLock<PicSourcePluginRegistry>>,
    data_root: PathBuf,
}

impl AppState {
    pub fn initialize() -> Self {
        let data_root = data_directory();
        std::fs::create_dir_all(&data_root).ok();
        let storage = Arc::new(FsVdirBackend::new(data_root.clone()));
        let index = Arc::new(
            SqliteContactIndex::new(data_root.join("index.sqlite"))
                .expect("initialize sqlite contact index"),
        );
        let contact_service = Arc::new(ContactService::new(
            storage.clone(),
            index.clone(),
            data_root.clone(),
        ));
        let plugin_registry = PicSourcePluginRegistry::with_builtins(&data_root);

        Self {
            storage,
            index,
            contact_service,
            plugin_registry,
            data_root,
        }
    }

    pub fn data_root(&self) -> &PathBuf {
        &self.data_root
    }

    pub async fn list_profiles(&self) -> Result<Vec<Profile>, String> {
        self.storage
            .list_profiles()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn create_profile(&self, name: &str) -> Result<Profile, String> {
        let slug = slugify(name);
        if slug.is_empty() {
            return Err("Profile name must contain at least one letter or number".into());
        }

        let now = chrono::Utc::now();
        let profile = Profile {
            id: ProfileId(uuid::Uuid::new_v4()),
            name: name.to_string(),
            slug,
            settings: ProfileSettings {
                scheduled_backup_enabled: false,
                scheduled_backup_dir: None,
            },
            sync_targets: vec![],
            created_at: now,
            updated_at: now,
        };

        self.storage
            .save_profile(&profile)
            .await
            .map_err(|e| e.to_string())?;
        Ok(profile)
    }
}

pub fn data_directory() -> PathBuf {
    directories::ProjectDirs::from("org", "profile-pulse", "Profile Pulse")
        .map(|dirs| dirs.data_local_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".profile-pulse"))
}

pub fn slugify(name: &str) -> String {
    let slug: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    slug.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_normalizes_names() {
        assert_eq!(slugify("Personal Contacts"), "personal-contacts");
    }
}
