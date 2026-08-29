//! Profile pic source plugin host — built-in runtime, WASM loader, and registry.
pub mod builtins;
#[cfg(not(target_arch = "wasm32"))]
pub mod desktop_host;
pub mod error;
pub mod manifest;
#[cfg(not(target_arch = "wasm32"))]
pub mod package;
pub mod registry;
#[cfg(not(target_arch = "wasm32"))]
pub mod wasm_runtime;
#[cfg(target_arch = "wasm32")]
pub mod web_host;

pub use builtins::github::github_candidate_for_username;
pub use builtins::gitlab::gitlab_candidate_for_username;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop_host::{DesktopHostApi, guess_content_type, host_context};
pub use error::HostError;
pub use manifest::{
    PACKAGE_EXTENSION, PicSourcePluginManifest, PluginInstallState, capability_name,
};
#[cfg(not(target_arch = "wasm32"))]
pub use package::{install_package, preview_package};
pub use registry::{PicSourcePluginRegistry, PluginEntry, PluginRuntimeKind};
#[cfg(not(target_arch = "wasm32"))]
pub use wasm_runtime::WasmPicSourcePlugin;
#[cfg(target_arch = "wasm32")]
pub use web_host::{PluginHostApi as DesktopHostApi, guess_content_type, host_context};
