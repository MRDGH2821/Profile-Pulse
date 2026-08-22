//! Profile pic source plugin host — built-in runtime, WASM loader, and registry.

pub mod builtins;
pub mod desktop_host;
pub mod error;
pub mod manifest;
pub mod package;
pub mod registry;
pub mod wasm_runtime;

pub use desktop_host::DesktopHostApi;
pub use error::HostError;
pub use manifest::{capability_name, PicSourcePluginManifest, PluginInstallState, PACKAGE_EXTENSION};
pub use package::{install_package, preview_package};
pub use registry::{PicSourcePluginRegistry, PluginEntry, PluginRuntimeKind};
pub use wasm_runtime::WasmPicSourcePlugin;

pub use builtins::github::github_candidate_for_username;
pub use builtins::gitlab::gitlab_candidate_for_username;
