# Implementation Spec: Profile Pulse Rewrite (v1)

Date: 2026-08-22  
Status: **Ready for implementation**  
Architecture: [2026-08-22-rewrite-architecture-design.md](./2026-08-22-rewrite-architecture-design.md)  
Human plan: [profile-pulse-app.md](../../human-plans/profile-pulse-app.md)  
Phase 0 plan: [2026-08-22-rewrite-phase-0-foundation.md](../plans/2026-08-22-rewrite-phase-0-foundation.md)

## Purpose

This document turns the architecture design into **implementable contracts**: domain types, traits, on-disk layouts, crate boundaries, phased delivery, and **default answers** for open design questions so Phase 0 can start without re-litigating brainstorming.

The legacy Iced app under `src/` is **superseded**. The rewrite lives in a Cargo **workspace** under `crates/`. Do not extend the legacy crate for new features.

## Global constraints (verbatim)

| Constraint | Value |
| --- | --- |
| UI framework | **Dioxus 0.7** |
| v1 platforms | **Desktop (native) + website (PWA)** — no mobile |
| Live contact book | App-owned store; sync targets are adapters |
| On-disk live format | **vdir** — one `.vcf` per contact |
| Backup export | Aggregated single `.vcf` + profile metadata |
| Pre-write backup | **Always** before contact/profile mutation |
| Sync default | Push-only; pull is explicit |
| Pull conflicts | Per-contact: Keep local / Take remote / Review |
| Profile pic extensibility | **Profile pic source plugins** only in v1 |
| User pic source package | `.pp-pic-source-plugin` (WASM) |
| Native user pic sources | Desktop only — **v1.1** |
| Sync adapters | First-party only — not pic source plugins |
| Web deployment | Static PWA, local-first, no required backend |
| License | GPLv3-or-later |
| Rust edition | **2024** (match existing repo) |

## Resolved defaults (v1 start)

These close open questions from the architecture design so implementation can proceed. Change via ADR + spec update if needed.

| Topic | v1 default |
| --- | --- |
| Cloud auth | **OAuth 2.0 PKCE** for Google and Outlook; **app password** or token-per-server for CardDAV |
| Remote-change check | Poll every **15 minutes** while app is active; **one prompt** listing which linked targets changed |
| SQLite role | **Index only** — search, sort keys, pic metadata, sync bookkeeping; **vdir `.vcf` files are source of truth** |
| Web secrets | **Passphrase-encrypted IDB** vault (Argon2id + AES-GCM); WebAuthn wrap deferred |
| Web scraping pic sources | **Public HTTP/API only** on web; document CORS limits; desktop may add host fetch proxy later |
| First OS adapter | **Linux** (author platform: Fedora) via **Evolution Data Server / flatpak portal** or `contacts` DB — spike in Phase 6 |
| First cloud sync adapter | **Google Contacts** |
| Second cloud sync adapter | **CardDAV** (generic; covers many providers) |
| Shared pic cache | Global under app data: `cache/avatars/<sha256>` |

## Workspace layout

```text
Cargo.toml                          # [workspace] members
crates/
  core/                             # profile-pulse-core
  storage/                          # profile-pulse-storage
  pic-source-plugin-api/            # profile-pulse-pic-source-plugin-api
  pic-source-plugin-host/           # profile-pulse-pic-source-plugin-host
  sync/                             # profile-pulse-sync (Phase 5+)
  app/                              # profile-pulse-app (Dioxus; Phase 1+)
legacy/
  iced-app/                         # moved from src/ — build optional, no new features
pic-source-plugins/
  builtin-gravatar/
  builtin-github/
  builtin-gitlab/
  sample-hello-pic-source/          # WASM sample (Phase 3)
docs/
  ROADMAP.md
  DEVELOPMENT.md
```

Workspace package names use kebab-case; Rust crate names use underscores (`profile_pulse_core`).

## On-disk layout (desktop)

Base: `$XDG_DATA_HOME/profile-pulse/` (typically `~/.local/share/profile-pulse/`)

```text
profile-pulse/
  profiles/
    <profile-slug>/
      profile.toml              # name, id, settings, linked sync targets
      contacts/
        <contact-uuid>.vcf      # one vCard per contact (live truth)
      backups/
        <timestamp>/            # pre-write snapshots
  cache/
    avatars/
      <sha256>                  # raw image bytes, shared across profiles
  pic-source-plugins/
    <plugin-id>/                # unpacked .pp-pic-source-plugin
  index.sqlite                  # global search/sync index (all profiles)
```

Web OPFS mirrors the same **logical** tree under `/profile-pulse/`.

### `profile.toml` (minimal v1)

```toml
id = "550e8400-e29b-41d4-a716-446655440000"
name = "Personal"
slug = "personal"
created_at = "2026-08-22T06:00:00Z"
updated_at = "2026-08-22T06:00:00Z"

[settings]
scheduled_backup_enabled = false
scheduled_backup_dir = ""

[[sync_targets]]
kind = "google"
enabled = true

[[sync_targets]]
kind = "carddav"
enabled = false
url = "https://example.com/dav/"
```

## Domain model (`profile-pulse-core`)

### Identifiers

```rust
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProfileId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ContactId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PicSourcePluginId(pub String); // manifest `id`, e.g. profile-pulse.builtin.gravatar-pic-source
```

### Contact and profile

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WebsiteLink {
    pub label: String,  // e.g. "GitHub", "work"
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EmailAddress {
    pub label: String,
    pub address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PhoneNumber {
    pub label: String,
    pub number: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Contact {
    pub id: ContactId,
    pub profile_id: ProfileId,
    pub display_name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub emails: Vec<EmailAddress>,
    pub phones: Vec<PhoneNumber>,
    pub websites: Vec<WebsiteLink>,
    /// SHA-256 hex of cached PHOTO bytes, if set
    pub photo_content_hash: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub id: ProfileId,
    pub name: String,
    pub slug: String,
    pub settings: ProfileSettings,
    pub sync_targets: Vec<SyncTargetConfig>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProfileSettings {
    pub scheduled_backup_enabled: bool,
    pub scheduled_backup_dir: Option<String>,
}
```

### Sync configuration (first-party)

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SyncTargetConfig {
    Google { enabled: bool },
    Outlook { enabled: bool },
    CardDav { enabled: bool, url: String },
    AppleIcloud { enabled: bool },
    OsContacts { enabled: bool }, // desktop cfg only
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    Push,
    Pull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullConflictResolution {
    KeepLocal,
    TakeRemote,
    Review,
}
```

### vCard mapping rules

- **Read/write** via `vobject` crate (already in repo dependencies — reuse from workspace).
- **Multiple `URL` properties** map to `websites`; use `TYPE` param or custom `X-PROFILE-PULSE-LABEL` for labels when standard params insufficient.
- **PHOTO** in vdir file stores embedded base64 **or** references app cache hash via `X-PROFILE-PULSE-PHOTO-HASH` when bytes live in shared cache (implementation choice: prefer hash + sidecar cache for dedupe).
- Contact file name: `contacts/<contact-uuid>.vcf` where UUID matches `Contact.id`.

## Storage layer (`profile-pulse-storage`)

### Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("contact not found: {0}")]
    ContactNotFound(ContactId),
    #[error("profile not found: {0}")]
    ProfileNotFound(ProfileId),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("vcard parse error: {0}")]
    Vcard(String),
    #[error("database error: {0}")]
    Database(String),
}
```

### `StorageBackend` trait

```rust
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    async fn list_profiles(&self) -> Result<Vec<Profile>, StorageError>;
    async fn load_profile(&self, id: ProfileId) -> Result<Profile, StorageError>;
    async fn save_profile(&self, profile: &Profile) -> Result<(), StorageError>;

    async fn list_contact_ids(&self, profile_id: ProfileId) -> Result<Vec<ContactId>, StorageError>;
    async fn load_contact(&self, profile_id: ProfileId, id: ContactId) -> Result<Contact, StorageError>;
    async fn save_contact(&self, contact: &Contact, vcard_bytes: &[u8]) -> Result<(), StorageError>;
    async fn delete_contact(&self, profile_id: ProfileId, id: ContactId) -> Result<(), StorageError>;

    async fn export_profile_vcf_aggregate(&self, profile_id: ProfileId) -> Result<Vec<u8>, StorageError>;
    async fn import_vcf_into_profile(&self, profile_id: ProfileId, vcf_bytes: &[u8]) -> Result<Vec<ContactId>, StorageError>;
}
```

### `ContactIndex` trait (SQLite)

```rust
#[async_trait::async_trait]
pub trait ContactIndex: Send + Sync {
    async fn upsert_contact(&self, contact: &Contact) -> Result<(), StorageError>;
    async fn remove_contact(&self, profile_id: ProfileId, id: ContactId) -> Result<(), StorageError>;
    async fn search(&self, profile_id: ProfileId, query: &str, limit: u32) -> Result<Vec<ContactId>, StorageError>;
}
```

Implementations:

| Type | Platform | Phase |
| --- | --- | --- |
| `FsVdirBackend` | Desktop | 0 |
| `OpfsVdirBackend` | Web WASM | 7 |
| `SqliteContactIndex` | Desktop | 0 |
| `WebContactIndex` | Web (IDB or wasm-sqlite) | 7 |

### `BackupService`

Lives in `profile-pulse-core` (orchestration) calling storage:

```rust
pub struct BackupService<B: StorageBackend> { /* ... */ }

impl<B: StorageBackend> BackupService<B> {
    pub async fn snapshot_profile_before_write(&self, profile_id: ProfileId) -> Result<BackupRef, CoreError>;
    pub async fn export_profile_bundle(&self, profile_id: ProfileId) -> Result<Vec<u8>, CoreError>; // zip: profile.toml + aggregate.vcf
}
```

## Profile pic source plugin API (`profile-pulse-pic-source-plugin-api`)

Crate version: **`PIC_SOURCE_PLUGIN_API_VERSION = 1`**.

### Host callback surface (plugins call these)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PicSourceCapability {
    Network,
    ReadSecrets,
}

#[derive(Debug, Clone)]
pub struct PicSourceHostContext {
    pub plugin_id: PicSourcePluginId,
}

#[async_trait::async_trait]
pub trait PicSourceHostApi: Send + Sync {
    async fn http_get(&self, ctx: &PicSourceHostContext, url: &str, headers: &[(&str, &str)]) -> Result<Vec<u8>, PicSourcePluginError>;
    async fn get_secret(&self, ctx: &PicSourceHostContext, key: &str) -> Result<Option<String>, PicSourcePluginError>;
    async fn cache_get(&self, ctx: &PicSourceHostContext, key: &str) -> Result<Option<Vec<u8>>, PicSourcePluginError>;
    async fn cache_put(&self, ctx: &PicSourceHostContext, key: &str, bytes: &[u8]) -> Result<(), PicSourcePluginError>;
    fn log(&self, ctx: &PicSourceHostContext, level: log::Level, message: &str);
}
```

### Plugin trait (built-in Rust impl; WASM exports mirror this)

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PicSourcePluginMetadata {
    pub id: PicSourcePluginId,
    pub name: String,
    pub version: semver::Version,
    pub min_host_version: semver::Version,
    pub website_match: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ContactContext {
    pub emails: Vec<String>,
    pub websites: Vec<WebsiteLink>,
    pub existing_photo_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfilePicCandidate {
    pub source_key: String,       // stable within plugin
    pub label: String,            // UI label
    pub preview_url: Option<String>,
    pub fetch_token: String,      // opaque to host; passed to fetch_pic
}

#[derive(Debug, Clone)]
pub struct ProfilePicBytes {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[async_trait::async_trait]
pub trait ProfilePicSourcePlugin: Send + Sync {
    fn metadata(&self) -> PicSourcePluginMetadata;
    fn capabilities(&self) -> Vec<PicSourceCapability>;
    async fn discover_sources(&self, ctx: &ContactContext) -> Result<Vec<ProfilePicCandidate>, PicSourcePluginError>;
    async fn fetch_pic(&self, candidate: &ProfilePicCandidate) -> Result<ProfilePicBytes, PicSourcePluginError>;
}
```

### Manifest (`manifest.toml`)

Required fields:

```toml
kind = "profile-pic-source"
id = "community.example-pic-source"
name = "Example profile pic source"
version = "0.1.0"
min_host_version = "1.0.0"
runtime = "wasm"
capabilities = ["network"]
website_match = []
```

Host **rejects** install if `kind != "profile-pic-source"`.

### Built-in plugin IDs (reserved)

| ID | Platform |
| --- | --- |
| `profile-pulse.builtin.gravatar-pic-source` | Gravatar |
| `profile-pulse.builtin.github-pic-source` | GitHub |
| `profile-pulse.builtin.gitlab-pic-source` | GitLab |

## Pic source plugin host (`profile-pulse-pic-source-plugin-host`)

```rust
pub struct PicSourcePluginRegistry {
    /* builtins + loaded wasm */
}

impl PicSourcePluginRegistry {
    pub fn register_builtin(&mut self, plugin: Box<dyn ProfilePicSourcePlugin>);
    pub async fn load_wasm_package(&mut self, path: &Path, host: Arc<dyn PicSourceHostApi>) -> Result<(), HostError>;
    pub fn list_metadata(&self) -> Vec<PicSourcePluginMetadata>;
    pub async fn discover_all(&self, contact: &ContactContext) -> Result<Vec<(PicSourcePluginId, ProfilePicCandidate)>, HostError>;
    pub async fn fetch(&self, plugin_id: &PicSourcePluginId, candidate: &ProfilePicCandidate) -> Result<ProfilePicBytes, HostError>;
}
```

Phase 3 adds `WasmRuntime` using **wasmtime** on desktop and browser WASM on web.

## Sync layer (`profile-pulse-sync`) — Phase 5+

```rust
#[async_trait::async_trait]
pub trait SyncAdapter: Send + Sync {
    fn target_kind(&self) -> &'static str;
    async fn push_contact(&self, contact: &Contact, vcard_bytes: &[u8]) -> Result<(), SyncError>;
    async fn pull_contact(&self, remote_id: &str) -> Result<(Contact, Vec<u8>), SyncError>;
    async fn check_remote_changes(&self, since: chrono::DateTime<chrono::Utc>) -> Result<Vec<RemoteChange>, SyncError>;
}
```

OAuth token storage uses desktop keychain / web encrypted IDB — **not** available to pic source plugins except via scoped `get_secret` for their own keys only.

## UI (`profile-pulse-app`) — Phase 1+

Dioxus 0.7 with features:

- `desktop` — `dioxus/desktop`
- `web` — `dioxus/web`

### Routes (v1)

| Route | Description |
| --- | --- |
| `/` | Profile picker / default profile contacts |
| `/profiles/:id/contacts` | Contact search + list |
| `/profiles/:id/contacts/:cid` | Tabbed: details / editor / pic selector |
| `/settings/pic-sources` | Profile pic source plugin manager |
| `/settings/sync` | Sync target linking |
| `/settings/backups` | Backup schedule + export |

Pic selector tab calls `PicSourcePluginRegistry::discover_all`, renders candidates, on select writes PHOTO via `ContactService::apply_profile_pic`.

## Application services (`profile-pulse-core`)

```rust
pub struct ContactService<B: StorageBackend, I: ContactIndex> {
    storage: Arc<B>,
    index: Arc<I>,
    backup: BackupService<B>,
}

impl<B: StorageBackend, I: ContactIndex> ContactService<B, I> {
    pub async fn update_contact(&self, contact: Contact, vcard_bytes: Vec<u8>) -> Result<(), CoreError> {
        self.backup.snapshot_profile_before_write(contact.profile_id).await?;
        self.storage.save_contact(&contact, &vcard_bytes).await?;
        self.index.upsert_contact(&contact).await?;
        Ok(())
    }

    pub async fn apply_profile_pic(
        &self,
        profile_id: ProfileId,
        contact_id: ContactId,
        pic: ProfilePicBytes,
    ) -> Result<(), CoreError> { /* hash → cache → update vcard PHOTO → save */ }
}
```

## Implementation phases

| Phase | Name | Deliverable | Plan doc |
| --- | --- | --- | --- |
| **0** | Foundation | Workspace, core types, desktop vdir + SQLite index | [phase-0](../plans/2026-08-22-rewrite-phase-0-foundation.md) |
| **1** | Desktop UI shell | Dioxus desktop, contact list/search, read-only details | TBD at Phase 1 start |
| **2** | Built-in pic sources | API + host + Gravatar/GitHub/GitLab | TBD |
| **3** | WASM pic sources | Wasm loader, sample plugin, settings UI | TBD |
| **4** | Backup/export | Pre-write backup, aggregate VCF, profile import/export | TBD |
| **5** | Cloud sync | Google + CardDAV push/pull + conflict UI | TBD |
| **6** | OS sync (desktop) | Linux OS contacts adapter | TBD |
| **7** | Web PWA | Dioxus web + OPFS backend | TBD |
| **8** | Background sync | Remote-change polling + pull prompts | TBD |

See [docs/ROADMAP.md](../../ROADMAP.md) for milestone checklist.

## Testing strategy

| Layer | Approach |
| --- | --- |
| `core` | Unit tests for vCard mapping, backup paths, conflict enums |
| `storage` | `tempfile` integration tests: round-trip contact save/load, search index |
| `pic-source-plugin-api` | Mock `PicSourceHostApi` |
| `pic-source-plugin-host` | Built-in plugin tests against mock HTTP |
| `app` | Dioxus RSX snapshot tests where feasible; manual desktop smoke |

Run: `cargo test --workspace` from repo root (after workspace conversion).

## Legacy code handling

1. Phase 0 moves `src/` → `legacy/iced-app/` as optional workspace member `profile-pulse-legacy`.
2. Root default `cargo build` builds **new** `profile-pulse-app` once Phase 1 lands; until then builds `core` + `storage` only.
3. Do **not** delete legacy until Phase 2 proves vCard path works end-to-end.

## Dependencies (workspace-level guidance)

| Crate | Purpose | Phase |
| --- | --- | --- |
| `tokio` | Async runtime (desktop) | 0 |
| `thiserror`, `anyhow` | Errors | 0 |
| `serde`, `serde_json`, `toml` | Config/serialization | 0 |
| `uuid`, `chrono` | IDs/time | 0 |
| `vobject` | vCard | 0 |
| `rusqlite` + `tokio` blocking pool | SQLite index | 0 |
| `directories` | XDG paths | 0 |
| `dioxus` 0.7 | UI | 1 |
| `reqwest` | HTTP (host + plugins) | 2 |
| `wasmtime` | Desktop WASM plugins | 3 |
| `oauth2` | Cloud sync | 5 |

Pin versions in workspace `Cargo.toml` `[workspace.dependencies]`.

## Non-goals (unchanged from architecture)

- Mobile apps, native user pic source plugins (v1), plugin marketplace, server backend, user sync plugins.

## References

- [Architecture design](./2026-08-22-rewrite-architecture-design.md)
- [Brainstorming notes](../../human-plans/brainstorming-notes.md)
- [Development guide](../../DEVELOPMENT.md)
- [Roadmap](../../ROADMAP.md)
