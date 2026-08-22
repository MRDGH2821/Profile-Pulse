use crate::error::HostError;
use profile_pulse_core::PicSourcePluginId;
use profile_pulse_pic_source_plugin_api::{
    PIC_SOURCE_PLUGIN_API_VERSION, PicSourceCapability, PicSourcePluginMetadata,
};
use semver::Version;
use serde::{Deserialize, Serialize};

pub const MANIFEST_FILE: &str = "manifest.toml";
pub const WASM_FILE: &str = "pic_source_plugin.wasm";
pub const INSTALL_FILE: &str = "install.toml";
pub const PACKAGE_EXTENSION: &str = "pp-pic-source-plugin";

#[derive(Debug, Clone, Deserialize)]
pub struct PicSourcePluginManifest {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub version: String,
    pub min_host_version: String,
    pub runtime: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub website_match: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginInstallState {
    pub enabled: bool,
    pub approved_capabilities: Vec<String>,
}

impl Default for PluginInstallState {
    fn default() -> Self {
        Self {
            enabled: true,
            approved_capabilities: vec![],
        }
    }
}

impl PicSourcePluginManifest {
    pub fn parse(bytes: &[u8]) -> Result<Self, HostError> {
        toml::from_str(std::str::from_utf8(bytes).map_err(|e| HostError::Manifest(e.to_string()))?)
            .map_err(|e| HostError::Manifest(e.to_string()))
    }

    pub fn validate(&self) -> Result<(), HostError> {
        if self.kind != "profile-pic-source" {
            return Err(HostError::Manifest(format!(
                "invalid kind `{}` (expected profile-pic-source)",
                self.kind
            )));
        }
        if self.runtime != "wasm" {
            return Err(HostError::Manifest(format!(
                "unsupported runtime `{}` (desktop Phase 3 supports wasm only)",
                self.runtime
            )));
        }
        if self.id.starts_with("profile-pulse.builtin.") {
            return Err(HostError::Manifest(
                "plugin id cannot use reserved profile-pulse.builtin.* prefix".into(),
            ));
        }
        Version::parse(&self.version)
            .map_err(|e| HostError::Manifest(format!("invalid version: {e}")))?;
        Version::parse(&self.min_host_version)
            .map_err(|e| HostError::Manifest(format!("invalid min_host_version: {e}")))?;
        let host_version = Version::new(u64::from(PIC_SOURCE_PLUGIN_API_VERSION), 0, 0);
        let min_host = Version::parse(&self.min_host_version)
            .map_err(|e| HostError::Manifest(e.to_string()))?;
        if host_version < min_host {
            return Err(HostError::Manifest(format!(
                "host API {host_version} is below plugin min_host_version {min_host}"
            )));
        }
        Ok(())
    }

    pub fn to_metadata(&self) -> Result<PicSourcePluginMetadata, HostError> {
        Ok(PicSourcePluginMetadata {
            id: PicSourcePluginId(self.id.clone()),
            name: self.name.clone(),
            version: Version::parse(&self.version)
                .map_err(|e| HostError::Manifest(e.to_string()))?,
            min_host_version: Version::parse(&self.min_host_version)
                .map_err(|e| HostError::Manifest(e.to_string()))?,
            website_match: self.website_match.clone(),
        })
    }

    pub fn requested_capabilities(&self) -> Vec<PicSourceCapability> {
        self.capabilities
            .iter()
            .filter_map(|cap| match cap.as_str() {
                "network" => Some(PicSourceCapability::Network),
                "read_secrets" => Some(PicSourceCapability::ReadSecrets),
                _ => None,
            })
            .collect()
    }
}

pub fn capability_name(cap: PicSourceCapability) -> &'static str {
    match cap {
        PicSourceCapability::Network => "network",
        PicSourceCapability::ReadSecrets => "read_secrets",
    }
}
