# Profile Pulse rewrite — brainstorming notes

**Branch / worktree:** `mrdgh2821/docs/rewrite-planning`  
**Human plan:** [profile-pulse-app.md](./profile-pulse-app.md)  
**Design spec:** [2026-08-22-rewrite-architecture-design.md](../superpowers/specs/2026-08-22-rewrite-architecture-design.md)  
**Updated:** 2026-08-22

## Locked decisions

| Topic                      | Choice                                                                                                                                      |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| UI framework               | Dioxus 0.7 (not Iced)                                                                                                                       |
| v1 platforms               | **Desktop + website (PWA)** — mobile dropped                                                                                                |
| Live contact book          | **A** — app-owned store; OS / Google / Apple / Outlook / CardDAV are adapters                                                               |
| Product focus (human plan) | Sync profile pics from social platforms into contacts; import/export/sync adapters                                                          |
| **Profile**                | Named local contact book. After create, user can sync contacts **to Google, Apple, Outlook, CardDAV, and OS all at once** (multi-target). |
| Sync semantics             | **C+** — push-only by default; separate explicit pull. **Background remote-change check**: if any linked target has edits, prompt user to pull. |
| Pull conflicts             | **C** — per-contact: Keep local / Take remote / Review                                                                                          |
| On-disk format             | **D** — **vdir** live (one `.vcf` per contact); maintain/export aggregated single VCF for backups                                               |
| Profile pic sources        | **Profile pic source plugin** architecture — built-ins: GitHub, GitLab, Gravatar; other platforms as optional/experimental pic source plugins |
| Pic source plugin naming   | See design spec: `ProfilePicSourcePlugin`, `.pp-pic-source-plugin`, `pic-source-plugins/`, manifest `kind = "profile-pic-source"` |
| User pic source format (v1) | **WASM `.pp-pic-source-plugin` only** — works on desktop and web                                                                               |
| Native user pic source plugins | Desktop only; **deferred to v1.1** (C ABI escape hatch)                                                                                     |
| Sync adapters              | **First-party only** — not profile pic source plugins (OAuth / contact writes)                                                                  |
| Web deployment             | Static **PWA**, local-first; no required backend for v1                                                                                     |
| Desktop storage            | Filesystem vdir + SQLite index                                                                                                              |
| Web storage                | OPFS vdir layout + IDB/SQLite index; encrypted secret vault                                                                                 |

## Open (next question)

| Topic                   | Options discussed                              |
| ----------------------- | ---------------------------------------------- |
| Auth for cloud targets  | OAuth PKCE / app passwords / OS Contacts APIs  |
| Remote-check UX details | Poll interval; per-target vs any-target prompt |
| SQLite role             | Index/search/pic cache only vs more            |
| Web secret storage      | Passphrase-encrypted IDB v1 vs WebAuthn later  |
| CORS / scraping on web  | Public-API-only pic source plugins vs desktop proxy       |

## Human plan summary

> **Note:** [profile-pulse-app.md](./profile-pulse-app.md) still says “desktop & mobile”. Platform decision above **supersedes** that for the rewrite.

- Primary job: fetch/scrape profile pics → apply to contacts
- Platforms: WhatsApp, Facebook, Instagram, Twitter, Discord, GitHub, GitLab, Gravatar, LinkedIn, Twitch
- Import: VCF, Google Contacts, Outlook, CardDAV
- Profiles: separate sources; per-profile settings + cache; shared pic cache OK
- UI: contact search; contact has tabs — details / editor / pic selector
- Websites: multiple links; convenience UI in pic selector
- Backups: VCF + profile import/export; scheduled; pre-write backup always
- Sync out: OS (desktop only), Google, Outlook, iCloud; per-contact Sync button

## Explicitly deferred

- Native desktop pic source plugin runtime (v1.1)
- Profile pic source plugin marketplace / signing
- Mobile apps
- Treating old OpenSpec as gospel (human plan is the rewrite source)
