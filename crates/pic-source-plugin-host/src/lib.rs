//! Profile pic source plugin host — built-in runtime and registry.

pub mod builtins;
pub mod desktop_host;
pub mod error;
pub mod registry;

pub use desktop_host::DesktopHostApi;
pub use error::HostError;
pub use registry::PicSourcePluginRegistry;

pub use builtins::github::github_candidate_for_username;
pub use builtins::gitlab::gitlab_candidate_for_username;
