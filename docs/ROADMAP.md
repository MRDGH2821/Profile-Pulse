# Profile Pulse Rewrite — Roadmap

**Status:** Phase 5 in progress on `mrdgh2821/feat/phase-5-cloud-sync`  
**Architecture:** [2026-08-22-rewrite-architecture-design.md](superpowers/specs/2026-08-22-rewrite-architecture-design.md)  
**Implementation spec:** [2026-08-22-rewrite-implementation-spec.md](superpowers/specs/2026-08-22-rewrite-implementation-spec.md)  
**Human plan:** [profile-pulse-app.md](human-plans/profile-pulse-app.md)

## Vision

Cross-platform **desktop + web PWA** app that keeps device/cloud contacts updated with profile pictures from social and web platforms. App-owned **vdir** contact store; Google / Outlook / CardDAV / OS are **sync adapters**. Profile pictures come from **built-in** and user **profile pic source plugins** (WASM).

## Phase overview

| Phase | Name | Outcome | Plan |
| --- | --- | --- | --- |
| 0 | Foundation | Workspace, core types, desktop vdir + SQLite index | [phase-0](superpowers/plans/2026-08-22-rewrite-phase-0-foundation.md) |
| 1 | Desktop UI shell | Dioxus desktop: profiles, contact search/list, details (read-only) | TBD |
| 2 | Built-in pic sources | Pic source plugin API/host + Gravatar, GitHub, GitLab | Done |
| 3 | WASM pic sources | WASM loader, sample `.pp-pic-source-plugin`, manager UI | Done |
| 4 | Backup & import/export | Pre-write backup UI, aggregate VCF, profile bundle | Done |
| 5 | Cloud sync | Google + CardDAV push/pull, per-contact sync button | In progress |
| 6 | OS sync (desktop) | Linux OS contacts adapter | TBD |
| 7 | Web PWA | Dioxus web, OPFS storage backend | TBD |
| 8 | Background sync | Remote-change polling, pull prompts, conflict UI | TBD |

## Phase 0 — Foundation

- [x] Cargo workspace with `profile-pulse-core`, `profile-pulse-storage`
- [x] Legacy Iced app moved to `legacy/iced-app` (no new features)
- [x] Domain model: `Contact`, `Profile`, IDs, sync enums
- [x] vCard read/write via vCard 3.0 mapping (core `vcard` module)
- [x] Desktop vdir: `profiles/<slug>/contacts/<uuid>.vcf`
- [x] SQLite contact search index
- [x] Pre-write profile snapshot backup
- [x] `ContactService::update_contact` orchestration
- [ ] Workspace tests + clippy clean (legacy crate may retain warnings)

## Phase 1 — Desktop UI shell

- [x] `profile-pulse-app` crate with Dioxus 0.7 desktop feature
- [x] Profile picker and create-profile flow
- [x] Contact search bar wired to `ContactIndex::search`
- [x] Contact list view
- [x] Contact details tab (read-only)
- [x] Placeholder tabs for editor and pic selector

## Phase 2 — Built-in profile pic sources

- [x] `profile-pulse-pic-source-plugin-api` crate
- [x] `profile-pulse-pic-source-plugin-host` with `BuiltinRuntime`
- [x] Built-in: `profile-pulse.builtin.gravatar-pic-source`
- [x] Built-in: `profile-pulse.builtin.github-pic-source`
- [x] Built-in: `profile-pulse.builtin.gitlab-pic-source`
- [x] Pic selector tab: discover + preview + apply via `ContactService::apply_profile_pic`
- [x] Website link convenience actions in pic selector

## Phase 3 — WASM profile pic sources

- [x] `.pp-pic-source-plugin` manifest parser (`kind = "profile-pic-source"`)
- [x] Desktop `WasmRuntime` (wasmtime)
- [x] Sample plugin `sample-hello-pic-source`
- [x] Settings → Profile pic sources: list, enable/disable, install from file
- [x] Capability approval on install

## Phase 4 — Backup & import/export

- [x] Contact editor tab (full CRUD)
- [x] VCF file import
- [x] Aggregate VCF export
- [x] Profile import/export bundle
- [x] Scheduled backup settings (desktop)
- [x] Backup browser UI

## Phase 5 — Cloud sync

- [x] `profile-pulse-sync` crate
- [x] OAuth PKCE for Google Contacts
- [x] CardDAV adapter (app password / token)
- [x] Push-only default; explicit pull action
- [x] Per-contact Sync button
- [x] Multi-target link on profile create

## Phase 6 — OS sync (desktop)

- [ ] Linux OS contacts adapter spike
- [ ] Push contact to OS address book
- [ ] Document Windows/macOS follow-ups

## Phase 7 — Web PWA

- [ ] Dioxus web target build
- [ ] `OpfsVdirBackend` + web contact index
- [ ] Passphrase-encrypted secret vault
- [ ] Static deploy docs (GitHub Pages or similar)
- [ ] WASM pic source plugins in browser

## Phase 8 — Background sync & conflicts

- [ ] 15-minute remote-change poll while app active
- [ ] Pull prompt listing changed targets
- [ ] Per-contact conflict resolution: Keep local / Take remote / Review

## Explicitly out of scope (v1)

- Mobile native apps
- Native desktop user pic source plugins (v1.1)
- Profile pic source marketplace / signing
- User-installable sync plugins
- Server-side backend for web

## Legacy code

The pre-rewrite **Iced** application lives in `legacy/iced-app/`. It is frozen — all new work uses the workspace crates under `crates/`.

## Related docs

- [DEVELOPMENT.md](DEVELOPMENT.md) — setup and commands
- [Brainstorming notes](human-plans/brainstorming-notes.md) — decision log
