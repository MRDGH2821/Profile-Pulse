//! Profile Pulse sync — first-party cloud adapters.
pub mod adapter;
pub mod carddav;
pub mod error;
pub mod google;
pub mod links;
pub mod secrets;
pub mod service;

pub use adapter::{PushResult, RemoteChange, SyncAdapter};
pub use carddav::{CardDavAdapter, CardDavCredentials};
pub use error::SyncError;
pub use google::{GoogleContactsAdapter, GoogleTokenBundle, authorize_google_pkce};
pub use links::SyncLinkStore;
pub use secrets::SecretStore;
pub use service::SyncService;
