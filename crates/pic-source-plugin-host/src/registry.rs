use crate::builtins::all_builtins;
use crate::desktop_host::DesktopHostApi;
use crate::error::HostError;
use crate::manifest::MANIFEST_FILE;
use crate::package::{
    approved_capabilities_from_install, install_package, plugin_install_dir, plugins_root,
    read_install_state, read_manifest_from_dir, write_install_state,
};
use crate::wasm_runtime::WasmPicSourcePlugin;
use profile_pulse_core::PicSourcePluginId;
use profile_pulse_pic_source_plugin_api::{
    ContactContext, PicSourceCapability, PicSourceHostApi, PicSourcePluginMetadata,
    ProfilePicBytes, ProfilePicCandidate, ProfilePicSourcePlugin,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginRuntimeKind {
    Builtin,
    Wasm,
}

#[derive(Debug, Clone)]
pub struct PluginEntry {
    pub metadata: PicSourcePluginMetadata,
    pub enabled: bool,
    pub runtime: PluginRuntimeKind,
    pub requested_capabilities: Vec<PicSourceCapability>,
    pub approved_capabilities: Vec<PicSourceCapability>,
}

struct RegisteredPlugin {
    plugin: Arc<dyn ProfilePicSourcePlugin>,
    enabled: bool,
    runtime: PluginRuntimeKind,
    requested_capabilities: Vec<PicSourceCapability>,
    approved_capabilities: Vec<PicSourceCapability>,
}

/// Registry and manager for built-in and WASM profile pic source plugins.
pub struct PicSourcePluginRegistry {
    plugins: HashMap<String, RegisteredPlugin>,
    data_root: PathBuf,
    host: Arc<DesktopHostApi>,
}

impl PicSourcePluginRegistry {
    pub fn new(data_root: PathBuf, host: Arc<DesktopHostApi>) -> Self {
        Self {
            plugins: HashMap::new(),
            data_root,
            host,
        }
    }

    /// Create a registry with built-ins and installed WASM plugins loaded.
    pub fn with_builtins(data_root: impl Into<PathBuf>) -> Arc<RwLock<Self>> {
        let data_root = data_root.into();
        let host = Arc::new(DesktopHostApi::new(data_root.join("plugin-host-cache")));
        let mut registry = Self::new(data_root.clone(), host.clone());
        for plugin in all_builtins(host) {
            registry.register_builtin(plugin);
        }
        if let Err(err) = registry.load_installed_wasm_plugins() {
            log::warn!("failed to load installed wasm plugins: {err}");
        }
        Arc::new(RwLock::new(registry))
    }

    pub fn register_builtin(&mut self, plugin: Box<dyn ProfilePicSourcePlugin>) {
        let metadata = plugin.metadata();
        let id = metadata.id.0.clone();
        self.plugins.insert(
            id,
            RegisteredPlugin {
                plugin: Arc::from(plugin),
                enabled: true,
                runtime: PluginRuntimeKind::Builtin,
                requested_capabilities: vec![],
                approved_capabilities: vec![],
            },
        );
    }

    pub fn list_entries(&self) -> Vec<PluginEntry> {
        self.plugins
            .values()
            .map(|entry| PluginEntry {
                metadata: entry.plugin.metadata(),
                enabled: entry.enabled,
                runtime: entry.runtime,
                requested_capabilities: entry.requested_capabilities.clone(),
                approved_capabilities: entry.approved_capabilities.clone(),
            })
            .collect()
    }

    pub fn list_metadata(&self) -> Vec<PicSourcePluginMetadata> {
        self.plugins
            .values()
            .filter(|entry| entry.enabled)
            .map(|entry| entry.plugin.metadata())
            .collect()
    }

    pub fn set_enabled(&mut self, plugin_id: &str, enabled: bool) -> Result<(), HostError> {
        let entry = self
            .plugins
            .get_mut(plugin_id)
            .ok_or_else(|| HostError::PluginNotFound(plugin_id.to_string()))?;
        if entry.runtime == PluginRuntimeKind::Builtin {
            entry.enabled = enabled;
            return Ok(());
        }
        entry.enabled = enabled;
        let install_dir = plugin_install_dir(&self.data_root, plugin_id);
        let mut state = read_install_state(&install_dir)?;
        state.enabled = enabled;
        write_install_state(&install_dir, &state)?;
        Ok(())
    }

    pub fn get(
        &self,
        plugin_id: &PicSourcePluginId,
    ) -> Result<Arc<dyn ProfilePicSourcePlugin>, HostError> {
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

    pub async fn install_package(
        &mut self,
        source: &Path,
        approved: &[PicSourceCapability],
    ) -> Result<PicSourcePluginId, HostError> {
        let install_dir = install_package(&self.data_root, source, approved).await?;
        self.register_wasm_install(&install_dir)
    }

    pub fn load_installed_wasm_plugins(&mut self) -> Result<(), HostError> {
        let root = plugins_root(&self.data_root);
        if !root.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(&root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let path = entry.path();
                if path.join(MANIFEST_FILE).exists() {
                    if let Err(err) = self.register_wasm_install(&path) {
                        log::warn!("skip wasm plugin at {}: {err}", path.display());
                    }
                }
            }
        }
        Ok(())
    }

    fn register_wasm_install(
        &mut self,
        install_dir: &Path,
    ) -> Result<PicSourcePluginId, HostError> {
        let manifest = read_manifest_from_dir(install_dir)?;
        let install_state = read_install_state(install_dir)?;
        let approved = approved_capabilities_from_install(&manifest, &install_state);
        let plugin = WasmPicSourcePlugin::from_install_dir(
            install_dir,
            Arc::clone(&self.host),
            approved.clone(),
        )?;
        let metadata = plugin.metadata();
        let id = metadata.id.0.clone();
        let requested = manifest.requested_capabilities();
        self.plugins.insert(
            id.clone(),
            RegisteredPlugin {
                plugin: Arc::new(plugin),
                enabled: install_state.enabled,
                runtime: PluginRuntimeKind::Wasm,
                requested_capabilities: requested,
                approved_capabilities: approved,
            },
        );
        Ok(PicSourcePluginId(id))
    }

    pub async fn load_wasm_package(
        &mut self,
        path: &Path,
        _host: Arc<dyn PicSourceHostApi>,
    ) -> Result<(), HostError> {
        self.install_package(path, &[]).await?;
        Ok(())
    }
}

impl Default for PicSourcePluginRegistry {
    fn default() -> Self {
        Self::new(
            PathBuf::from(".profile-pulse"),
            Arc::new(DesktopHostApi::new(".profile-pulse/plugin-host-cache")),
        )
    }
}
