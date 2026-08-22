# Profile Pulse Rewrite — Phase 0 Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create the Cargo workspace, domain crate, and desktop vdir + SQLite index storage so later phases can build UI and pic source plugins on stable contracts.

**Architecture:** Convert the repo to a workspace; implement `profile-pulse-core` (types + vCard mapping + backup orchestration stubs) and `profile-pulse-storage` (`FsVdirBackend` + `SqliteContactIndex`); relocate legacy Iced code to `legacy/iced-app` without deleting it.

**Tech Stack:** Rust 2024, Tokio, vobject, rusqlite, thiserror, tempfile (tests), existing Nix flake devshell.

## Global Constraints

- UI framework **Dioxus 0.7** — not used in Phase 0 (Phase 1).
- v1 platforms: **desktop + web PWA** — Phase 0 implements **desktop filesystem storage only**.
- Live format: **vdir** (one `.vcf` per contact); SQLite is **index only**.
- Pre-write backup: **always** before contact save — implement `BackupService::snapshot_profile_before_write` in Phase 0.
- Profile pic source naming: `ProfilePicSourcePlugin`, `.pp-pic-source-plugin`, `pic-source-plugins/` (no generic `plugin-api`).
- Rust edition **2024**; license **GPL-3.0-or-later** on new crates.
- Commits: Conventional Commits + `Co-authored-by: Composer via Cursor <cursoragent@cursor.com>`.
- Implementation spec: [2026-08-22-rewrite-implementation-spec.md](../specs/2026-08-22-rewrite-implementation-spec.md).

---

## File Structure

| Path                                 | Responsibility                          |
| ------------------------------------ | --------------------------------------- |
| `Cargo.toml`                         | Workspace root; `[workspace.members]`   |
| `crates/core/Cargo.toml`             | `profile-pulse-core` package            |
| `crates/core/src/lib.rs`             | Re-exports domain + services            |
| `crates/core/src/model.rs`           | `Contact`, `Profile`, IDs, sync enums   |
| `crates/core/src/vcard.rs`           | vCard ↔ `Contact` mapping via `vobject` |
| `crates/core/src/backup.rs`          | `BackupService` pre-write snapshots     |
| `crates/core/src/contact_service.rs` | `ContactService` orchestration          |
| `crates/core/src/error.rs`           | `CoreError`                             |
| `crates/storage/Cargo.toml`          | `profile-pulse-storage` package         |
| `crates/storage/src/lib.rs`          | Re-exports backends                     |
| `crates/storage/src/traits.rs`       | `StorageBackend`, `ContactIndex` traits |
| `crates/storage/src/fs_vdir.rs`      | Desktop vdir implementation             |
| `crates/storage/src/sqlite_index.rs` | SQLite search index                     |
| `crates/storage/src/error.rs`        | `StorageError`                          |
| `legacy/iced-app/`                   | Moved from `src/` (optional member)     |
| `docs/ROADMAP.md`                    | Phase checklist (already created)       |

---

### Task 0: Record implementation spec (prerequisite doc)

**Files:**

- Verify: `docs/superpowers/specs/2026-08-22-rewrite-implementation-spec.md`

**Interfaces:**

- Consumes: [architecture design](../specs/2026-08-22-rewrite-architecture-design.md)
- Produces: Canonical types/traits referenced by Tasks 1–5

- [ ] **Step 1: Confirm spec exists and matches architecture locked decisions**

Read the implementation spec sections: Global constraints, Domain model, `StorageBackend`, on-disk layout.

- [ ] **Step 2: Commit spec if not already on branch**

Skip if `git log -1 --oneline -- docs/superpowers/specs/2026-08-22-rewrite-implementation-spec.md` shows a commit.

---

### Task 1: Create workspace and relocate legacy Iced app

**Files:**

- Create: `Cargo.toml` (workspace root — replace single-package root)
- Move: `src/` → `legacy/iced-app/src/`
- Move: `Cargo.lock` (workspace will regenerate)
- Create: `legacy/iced-app/Cargo.toml` (from current root manifest, rename package to `profile-pulse-legacy`)
- Modify: `.github/workflows/ci.yml` — build workspace members or `cargo check -p profile-pulse-core -p profile-pulse-storage` until app exists

**Interfaces:**

- Consumes: Current root `Cargo.toml`
- Produces: Workspace root; legacy crate at `profile-pulse-legacy`

- [ ] **Step 1: Create legacy crate manifest**

Move existing package metadata to `legacy/iced-app/Cargo.toml` and set:

```toml
[package]
name = "profile-pulse-legacy"
version = "0.1.0"
edition = "2024"
publish = false

[[bin]]
name = "profile-pulse-legacy"
path = "src/main.rs"

# ... copy [dependencies] and [dev-dependencies] from root Cargo.toml unchanged ...
```

- [ ] **Step 2: Move sources**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
mkdir -p legacy/iced-app
git mv src legacy/iced-app/src
git mv Cargo.toml legacy/iced-app/Cargo.toml.bak  # temporary; merge into legacy/iced-app/Cargo.toml
```

Edit paths manually if `git mv` conflicts — end state: `legacy/iced-app/src/main.rs` exists.

- [ ] **Step 3: Write workspace root `Cargo.toml`**

```toml
[workspace]
resolver = "3"
members = [
  "crates/core",
  "crates/storage",
  "legacy/iced-app",
]

[workspace.package]
edition = "2024"
license = "GPL-3.0-or-later"
repository = "https://github.com/MRDGH2821/Profile-Pulse"

[workspace.dependencies]
anyhow = "1.0.100"
async-trait = "0.1.89"
chrono = { version = "0.4.42", features = ["serde"] }
directories = "6.0.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tokio = { version = "1.49", features = ["macros", "rt-multi-thread", "fs"] }
toml = "0.8"
uuid = { version = "1.19", features = ["serde", "v4"] }
vobject = "0.8"
rusqlite = { version = "0.32", features = ["bundled"] }
tempfile = "3.24"
```

- [ ] **Step 4: Verify legacy still checks**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
rtk cargo check -p profile-pulse-legacy
```

Expected: exit 0 (or fix path issues in legacy crate).

- [ ] **Step 5: Commit**

```bash
rtk git add Cargo.toml legacy/ crates/ .github/workflows/ci.yml
rtk git commit -m "$(cat <<'EOF'
refactor: convert repo to workspace and isolate legacy iced app

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

Update CI `cargo check` line to:

```yaml
run: cargo check --workspace --all-targets
```

---

### Task 2: Implement `profile-pulse-core` domain model

**Files:**

- Create: `crates/core/Cargo.toml`
- Create: `crates/core/src/lib.rs`
- Create: `crates/core/src/model.rs`
- Create: `crates/core/src/error.rs`
- Test: `crates/core/src/model.rs` (`#[cfg(test)]`)

**Interfaces:**

- Consumes: Types from [implementation spec](../specs/2026-08-22-rewrite-implementation-spec.md#domain-model-profile-pulse-core)
- Produces: `ProfileId`, `ContactId`, `Contact`, `Profile`, `WebsiteLink`, sync enums, `CoreError`

- [ ] **Step 1: Write failing model test**

Create `crates/core/Cargo.toml`:

```toml
[package]
name = "profile-pulse-core"
version = "0.1.0"
edition = "2024"
description = "Profile Pulse domain model and services"
license.workspace = true
repository.workspace = true

[dependencies]
chrono.workspace = true
serde.workspace = true
thiserror.workspace = true
uuid.workspace = true

[dev-dependencies]
```

Create `crates/core/src/model.rs` with test:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn contact_round_trips_serde() {
        let contact = Contact {
            id: ContactId(Uuid::new_v4()),
            profile_id: ProfileId(Uuid::new_v4()),
            display_name: "Ada Lovelace".into(),
            given_name: Some("Ada".into()),
            family_name: Some("Lovelace".into()),
            emails: vec![EmailAddress {
                label: "work".into(),
                address: "ada@example.com".into(),
            }],
            phones: vec![],
            websites: vec![WebsiteLink {
                label: "GitHub".into(),
                url: "https://github.com/octocat".into(),
            }],
            photo_content_hash: None,
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&contact).unwrap();
        let back: Contact = serde_json::from_str(&json).unwrap();
        assert_eq!(contact, back);
    }
}
```

- [ ] **Step 2: Run test — expect fail**

```bash
rtk cargo test -p profile-pulse-core contact_round_trips_serde
```

Expected: FAIL (module/types not defined).

- [ ] **Step 3: Implement `model.rs` and `error.rs`**

Implement all types from the implementation spec. `crates/core/src/lib.rs`:

```rust
pub mod error;
pub mod model;

pub use error::CoreError;
pub use model::*;
```

- [ ] **Step 4: Run test — expect pass**

```bash
rtk cargo test -p profile-pulse-core
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
rtk git add crates/core
rtk git commit -m "$(cat <<'EOF'
feat(core): add domain model types for rewrite

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 3: Implement vCard mapping

**Files:**

- Create: `crates/core/src/vcard.rs`
- Modify: `crates/core/Cargo.toml` — add `vobject`, `serde_json` dev-dep for fixtures
- Modify: `crates/core/src/lib.rs` — `pub mod vcard;`

**Interfaces:**

- Consumes: `Contact`, `ContactId`, `ProfileId` from Task 2
- Produces: `contact_to_vcard_bytes(contact: &Contact) -> Result<Vec<u8>, CoreError>`, `contact_from_vcard_bytes(profile_id: ProfileId, id: ContactId, bytes: &[u8]) -> Result<Contact, CoreError>`

- [ ] **Step 1: Write failing vCard round-trip test**

Use `.agents/samples/test contact.vcf` or minimal inline vCard:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn vcard_round_trip_preserves_display_name_and_email() {
        let profile_id = ProfileId(Uuid::new_v4());
        let contact_id = ContactId(Uuid::new_v4());
        let original = Contact {
            id: contact_id,
            profile_id,
            display_name: "Test User".into(),
            given_name: None,
            family_name: None,
            emails: vec![EmailAddress {
                label: "home".into(),
                address: "test@example.com".into(),
            }],
            phones: vec![],
            websites: vec![],
            photo_content_hash: None,
            updated_at: chrono::Utc::now(),
        };
        let bytes = contact_to_vcard_bytes(&original).unwrap();
        let parsed = contact_from_vcard_bytes(profile_id, contact_id, &bytes).unwrap();
        assert_eq!(parsed.display_name, "Test User");
        assert_eq!(parsed.emails[0].address, "test@example.com");
    }
}
```

- [ ] **Step 2: Run test — expect fail**

```bash
rtk cargo test -p profile-pulse-core vcard_round_trip
```

- [ ] **Step 3: Implement mapping with `vobject`**

Map `FN`, `EMAIL`, `TEL`, `URL`, `PHOTO` (hash via `X-PROFILE-PULSE-PHOTO-HASH` when no inline photo).

- [ ] **Step 4: Run tests — expect pass**

```bash
rtk cargo test -p profile-pulse-core
```

- [ ] **Step 5: Commit**

```bash
rtk git add crates/core
rtk git commit -m "$(cat <<'EOF'
feat(core): add vcard mapping for contacts

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 4: Implement desktop vdir storage backend

**Files:**

- Create: `crates/storage/Cargo.toml`
- Create: `crates/storage/src/lib.rs`, `traits.rs`, `error.rs`, `fs_vdir.rs`
- Test: `crates/storage/tests/fs_vdir_integration.rs`

**Interfaces:**

- Consumes: `StorageBackend` trait from spec; `Contact`, `Profile` from `profile-pulse-core`
- Produces: `FsVdirBackend::new(root: PathBuf)`, implements `StorageBackend`

- [ ] **Step 1: Write failing integration test**

```rust
// crates/storage/tests/fs_vdir_integration.rs
use profile_pulse_core::{Contact, ContactId, EmailAddress, Profile, ProfileId, ProfileSettings};
use profile_pulse_storage::{FsVdirBackend, StorageBackend};
use tempfile::tempdir;
use uuid::Uuid;

#[tokio::test]
async fn save_and_load_contact_round_trip() {
    let dir = tempdir().unwrap();
    let backend = FsVdirBackend::new(dir.path().to_path_buf());
    let profile_id = ProfileId(Uuid::new_v4());
    let contact_id = ContactId(Uuid::new_v4());

    backend
        .save_profile(&Profile {
            id: profile_id,
            name: "Test".into(),
            slug: "test".into(),
            settings: ProfileSettings {
                scheduled_backup_enabled: false,
                scheduled_backup_dir: None,
            },
            sync_targets: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    let contact = Contact {
        id: contact_id,
        profile_id,
        display_name: "Jane".into(),
        given_name: None,
        family_name: None,
        emails: vec![EmailAddress {
            label: "home".into(),
            address: "jane@example.com".into(),
        }],
        phones: vec![],
        websites: vec![],
        photo_content_hash: None,
        updated_at: chrono::Utc::now(),
    };
    let vcard = profile_pulse_core::vcard::contact_to_vcard_bytes(&contact).unwrap();
    backend.save_contact(&contact, &vcard).await.unwrap();

    let loaded = backend.load_contact(profile_id, contact_id).await.unwrap();
    assert_eq!(loaded.display_name, "Jane");
    assert!(dir.path().join("profiles/test/contacts").exists());
}
```

- [ ] **Step 2: Run test — expect fail**

```bash
rtk cargo test -p profile-pulse-storage save_and_load_contact_round_trip
```

- [ ] **Step 3: Implement `FsVdirBackend`**

Layout per spec: `profiles/<slug>/profile.toml`, `contacts/<uuid>.vcf`.

- [ ] **Step 4: Run tests — expect pass**

```bash
rtk cargo test -p profile-pulse-storage
```

- [ ] **Step 5: Commit**

```bash
rtk git add crates/storage
rtk git commit -m "$(cat <<'EOF'
feat(storage): add filesystem vdir backend

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 5: SQLite contact index + backup service

**Files:**

- Create: `crates/storage/src/sqlite_index.rs`
- Create: `crates/core/src/backup.rs`, `crates/core/src/contact_service.rs`
- Modify: `crates/core/Cargo.toml` — add `profile-pulse-storage` path dep for integration tests only if needed (prefer storage tests calling index)

**Interfaces:**

- Consumes: `ContactIndex` trait; `FsVdirBackend`
- Produces: `SqliteContactIndex::new(path)`, `BackupService::snapshot_profile_before_write`, `ContactService::update_contact`

- [ ] **Step 1: Write failing search test**

```rust
#[tokio::test]
async fn index_finds_contact_by_display_name() {
    let dir = tempdir().unwrap();
    let index = SqliteContactIndex::new(dir.path().join("index.sqlite")).unwrap();
    // upsert contact with display_name "Unique Name XYZ"
    let ids = index.search(profile_id, "Unique Name", 10).await.unwrap();
    assert_eq!(ids.len(), 1);
}
```

- [ ] **Step 2: Implement SQLite schema**

```sql
CREATE TABLE contacts (
  profile_id TEXT NOT NULL,
  contact_id TEXT NOT NULL,
  display_name TEXT NOT NULL,
  search_text TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (profile_id, contact_id)
);
CREATE INDEX idx_contacts_search ON contacts(profile_id, search_text);
```

- [ ] **Step 3: Implement `BackupService`**

Copy `profiles/<slug>/` to `profiles/<slug>/backups/<timestamp>/` before writes (excluding `backups/` subtree).

- [ ] **Step 4: Wire `ContactService::update_contact`**

Call backup → save vdir → upsert index.

- [ ] **Step 5: Run full workspace tests**

```bash
rtk cargo test --workspace
rtk cargo clippy --workspace -- -D warnings
```

- [ ] **Step 6: Commit**

```bash
rtk git add crates/
rtk git commit -m "$(cat <<'EOF'
feat(storage): add sqlite index and pre-write backup service

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 6: Verify and update roadmap

**Files:**

- Modify: `docs/ROADMAP.md` — mark Phase 0 items complete
- Modify: `docs/superpowers/specs/2026-08-22-rewrite-architecture-design.md` — status → Approved

**Interfaces:**

- Consumes: Tasks 1–5 green CI
- Produces: Phase 0 complete; Phase 1 plan can be written

- [ ] **Step 1: Run Nix check if available**

```bash
rtk nix flake check
```

- [ ] **Step 2: Update ROADMAP Phase 0 checkboxes**

- [ ] **Step 3: Commit docs**

```bash
rtk git add docs/
rtk git commit -m "$(cat <<'EOF'
docs: mark rewrite phase 0 foundation complete

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

## Spec Coverage Checklist

| Requirement                              | Task   |
| ---------------------------------------- | ------ |
| Workspace crate layout                   | Task 1 |
| Domain types (`Contact`, `Profile`, IDs) | Task 2 |
| vCard mapping                            | Task 3 |
| Desktop vdir layout                      | Task 4 |
| `StorageBackend` trait                   | Task 4 |
| SQLite index only (not source of truth)  | Task 5 |
| Pre-write backup                         | Task 5 |
| Legacy Iced isolated                     | Task 1 |
| Implementation spec as source of truth   | Task 0 |

## Next phase

After Phase 0, write **Phase 1 plan** (`docs/superpowers/plans/2026-08-22-rewrite-phase-1-desktop-ui.md`): Dioxus 0.7 desktop shell, contact list, read-only details using `ContactService`.
