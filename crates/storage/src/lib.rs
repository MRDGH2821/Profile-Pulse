//! Profile Pulse storage — vdir backends and contact index.

pub mod contact_service;
pub mod error;
pub mod fs_vdir;
pub mod photo_cache;
pub mod sqlite_index;
pub mod traits;

pub use contact_service::ContactService;
pub use photo_cache::{load_photo, photo_cache_dir, sha256_hex, store_photo};

pub use error::StorageError;
pub use fs_vdir::FsVdirBackend;
pub use sqlite_index::SqliteContactIndex;
pub use traits::{ContactIndex, StorageBackend};
