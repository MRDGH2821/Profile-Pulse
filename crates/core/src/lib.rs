//! Profile Pulse core — domain model and orchestration.
#[cfg(not(target_arch = "wasm32"))]
pub mod backup;
#[cfg(target_arch = "wasm32")]
mod backup_stub;
pub mod error;
pub mod model;
pub mod vcard;

#[cfg(not(target_arch = "wasm32"))]
pub use backup::{BackupRef, BackupService};
#[cfg(target_arch = "wasm32")]
pub use backup_stub::{BackupRef, BackupService};
pub use error::CoreError;
pub use model::*;
pub use vcard::{
    contact_from_vcard_bytes, contact_to_vcard_bytes, import_contacts_from_vcf, split_vcard_blocks,
};
