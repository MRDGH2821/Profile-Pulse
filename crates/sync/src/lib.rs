//! Profile Pulse sync — first-party cloud adapters.
#[cfg(not(target_arch = "wasm32"))]
pub mod adapter;
#[cfg(not(target_arch = "wasm32"))]
pub mod carddav;
pub mod conflict;
pub mod credentials;
pub mod error;
#[cfg(not(target_arch = "wasm32"))]
pub mod google;
#[cfg(not(target_arch = "wasm32"))]
pub mod links;
#[cfg(target_arch = "wasm32")]
pub mod links_web;
#[cfg(not(target_arch = "wasm32"))]
pub mod secrets;
#[cfg(target_arch = "wasm32")]
pub mod secrets_web;
#[cfg(not(target_arch = "wasm32"))]
mod service;
#[cfg(target_arch = "wasm32")]
mod service_web;

#[cfg(not(target_arch = "wasm32"))]
pub use adapter::SyncAdapter;
#[cfg(not(target_arch = "wasm32"))]
pub use carddav::CardDavAdapter;
pub use conflict::{
    PullApplyResult, PullConflict, PullPrepareResult, RemoteChange, TargetRemoteChanges,
    is_pull_conflict, resolve_pull_conflict,
};
pub use credentials::{CardDavCredentials, PushResult, carddav_secret_key};
pub use error::SyncError;
#[cfg(not(target_arch = "wasm32"))]
pub use google::{GoogleContactsAdapter, GoogleTokenBundle, authorize_google_pkce};
#[cfg(not(target_arch = "wasm32"))]
pub use links::SyncLink;
#[cfg(not(target_arch = "wasm32"))]
pub use links::SyncLinkStore;
#[cfg(target_arch = "wasm32")]
pub use links_web::{SyncLink, SyncLinkStore};
#[cfg(not(target_arch = "wasm32"))]
pub use secrets::SecretStore;
#[cfg(target_arch = "wasm32")]
pub use secrets_web::SecretStore;
#[cfg(not(target_arch = "wasm32"))]
pub use service::SyncService;
#[cfg(target_arch = "wasm32")]
pub use service_web::SyncService;
