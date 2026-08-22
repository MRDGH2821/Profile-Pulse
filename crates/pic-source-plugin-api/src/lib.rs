//! Profile pic source plugin API — host callbacks and plugin trait.

pub mod error;
pub mod host;
pub mod plugin;
pub mod types;

pub use error::PicSourcePluginError;
pub use host::{PicSourceCapability, PicSourceHostApi, PicSourceHostContext};
pub use plugin::ProfilePicSourcePlugin;
pub use types::{
    ContactContext, PicSourcePluginMetadata, ProfilePicBytes, ProfilePicCandidate,
};

/// Stable ABI version for profile pic source plugins.
pub const PIC_SOURCE_PLUGIN_API_VERSION: u32 = 1;
