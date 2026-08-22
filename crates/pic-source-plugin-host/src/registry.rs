use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use profile_pulse_core::PicSourcePluginId;
use profile_pulse_pic_source_plugin_api::{
    ContactContext, PicSourcePluginMetadata, ProfilePicBytes, ProfilePicCandidate,
    ProfilePicSourcePlugin,
};

use crate::builtins::all_builtins;
use crate::desktop_host::DesktopHostApi;
use crate::error::HostError;

struct RegisteredPlugin {
    plugin: Arc<dyn ProfilePicSourcePlugin>,
    enabled: bool,
}

/// Registry of built-in (and later WASM) profile pic source plugins.
pub struct PicSourcePluginRegistry {
    plugins: HashMap<String, RegisteredPlugin>,
}

impl PicSourcePluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Create a registry with all first-party built-in plugins registered.
    pub fn with_builtins(data_root: impl Into<PathBuf>) -> Arc<Self> {
        let data_root = data_root.into();
        let host = Arc::new(DesktopHostApi::new(
            data_root.join("plugin-host-cache"),
        ));
        let mut registry = Self::new();
        for plugin in all_builtins(host) {
            registry.register_builtin(plugin);
        }
        Arc::new(registry)
    }

    pub fn register_builtin(&mut self, plugin: Box<dyn ProfilePicSourcePlugin>) {
        let id = plugin.metadata().id.0.clone();
        self.plugins.insert(
            id,
            RegisteredPlugin {
                plugin: Arc::from(plugin),
                enabled: true,
            },
        );
    }

    pub fn list_metadata(&self) -> Vec<PicSourcePluginMetadata> {
        self.plugins
            .values()
            .filter(|entry| entry.enabled)
            .map(|entry| entry.plugin.metadata())
            .collect()
    }

    pub fn get(&self, plugin_id: &PicSourcePluginId) -> Result<Arc<dyn ProfilePicSourcePlugin>, HostError> {
        self.plugins
            .get(&plugin_id.0)
            .filter(|entry| entry.enabled)
            .map(|entry| Arc::clone(&entry.plugin))
            .ok_or_else(|| HostError::PluginNotFound(plugin_id.0.clone()))
    }

    pub async fn discover_all(
        &self,
        contact: &ContactContext,
    ) -> Result<Vec<(PicSourcePluginId, ProfilePicCandidate)>, HostError> {
        let mut out = Vec::new();
        for entry in self.plugins.values().filter(|e| e.enabled) {
            let plugin_id = entry.plugin.metadata().id.clone();
            match entry.plugin.discover_sources(contact).await {
                Ok(candidates) => {
                    for candidate in candidates {
                        out.push((plugin_id.clone(), candidate));
                    }
                }
                Err(err) => {
                    log::warn!("discover failed for {}: {err}", plugin_id.0);
                }
            }
        }
        Ok(out)
    }

    pub async fn fetch(
        &self,
        plugin_id: &PicSourcePluginId,
        candidate: &ProfilePicCandidate,
    ) -> Result<ProfilePicBytes, HostError> {
        let plugin = self.get(plugin_id)?;
        Ok(plugin.fetch_pic(candidate).await?)
    }

    pub async fn load_wasm_package(
        &mut self,
        _path: &Path,
        _host: Arc<dyn profile_pulse_pic_source_plugin_api::PicSourceHostApi>,
    ) -> Result<(), HostError> {
        Err(HostError::Http(
            "WASM pic source plugins are not supported until Phase 3".into(),
        ))
    }
}

impl Default for PicSourcePluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
