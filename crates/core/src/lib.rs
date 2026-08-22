//! Profile Pulse core — domain model and orchestration.

pub mod backup;
pub mod error;
pub mod model;
pub mod vcard;

pub use backup::{BackupRef, BackupService};
pub use error::CoreError;
pub use model::*;
pub use vcard::{contact_from_vcard_bytes, contact_to_vcard_bytes};
