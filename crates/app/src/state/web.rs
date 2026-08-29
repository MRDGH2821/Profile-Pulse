use dioxus::prelude::*;
use profile_pulse_core::{Profile, ProfileId, ProfileSettings, SyncTargetConfig};
use profile_pulse_pic_source_plugin_host::PicSourcePluginRegistry;
use profile_pulse_storage::{ContactService, OpfsVdirBackend, StorageBackend, WebContactIndex};
use profile_pulse_sync::SyncService;
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

pub type AppBackend = OpfsVdirBackend;
pub type AppIndex = WebContactIndex;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<AppBackend>,
    pub index: Arc<AppIndex>,
    pub contact_service: Arc<ContactService<AppBackend, AppIndex>>,
    pub plugin_registry: Arc<RwLock<PicSourcePluginRegistry>>,
    pub sync_service: Arc<SyncService>,
    data_root: PathBuf,
}

impl AppState {
    pub fn initialize() -> Self {
        let data_root = PathBuf::from("opfs://profile-pulse");
        let storage = Arc::new(OpfsVdirBackend::new());
        let index = Arc::new(WebContactIndex::new());
        let contact_service = Arc::new(ContactService::new(
            storage.clone(),
            index.clone(),
            data_root.clone(),
        ));
        let plugin_registry = PicSourcePluginRegistry::with_builtins(&data_root);
        let sync_service = Arc::new(SyncService::new(&data_root).expect("initialize sync service"));
        Self {
            storage,
            index,
            contact_service,
            plugin_registry,
            sync_service,
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

    pub async fn create_profile(
        &self,
        name: &str,
        sync_targets: Vec<SyncTargetConfig>,
    ) -> Result<Profile, String> {
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
                scheduled_backup_last_run: None,
                last_remote_sync_check: None,
            },
            sync_targets,
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
