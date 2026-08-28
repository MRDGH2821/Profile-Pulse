//! Profile Pulse storage — vdir backends and contact index.
pub mod contact_service;
pub mod error;
pub mod fs_vdir;
pub mod photo_cache;
pub mod profile_bundle;
#[cfg(not(target_arch = "wasm32"))]
pub mod sqlite_index;
pub mod traits;

#[cfg(target_arch = "wasm32")]
pub mod opfs;
#[cfg(target_arch = "wasm32")]
pub mod opfs_vdir;
#[cfg(target_arch = "wasm32")]
pub mod web_index;

pub use contact_service::ContactService;
pub use error::StorageError;
pub use fs_vdir::FsVdirBackend;
pub use photo_cache::{load_photo, photo_cache_dir, sha256_hex, store_photo};
#[cfg(not(target_arch = "wasm32"))]
pub use sqlite_index::SqliteContactIndex;
pub use traits::{ContactIndex, StorageBackend};

#[cfg(target_arch = "wasm32")]
pub use opfs_vdir::OpfsVdirBackend;
#[cfg(target_arch = "wasm32")]
pub use web_index::WebContactIndex;
