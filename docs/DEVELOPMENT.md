# Development Guide — Profile Pulse Rewrite

This guide covers the **Dioxus rewrite** (desktop + web PWA). For the frozen legacy Iced app, see `legacy/iced-app/`.

## Documentation map

| Doc                                                                                | Purpose                          |
| ---------------------------------------------------------------------------------- | -------------------------------- |
| [ROADMAP.md](ROADMAP.md)                                                           | Phased milestones and checklists |
| [Implementation spec](superpowers/specs/2026-08-22-rewrite-implementation-spec.md) | Types, traits, on-disk layout    |
| [Architecture design](superpowers/specs/2026-08-22-rewrite-architecture-design.md) | High-level decisions             |
| [Phase 0 plan](superpowers/plans/2026-08-22-rewrite-phase-0-foundation.md)         | First implementation tasks       |
| [Human plan](human-plans/profile-pulse-app.md)                                     | Product requirements             |
| [DIOXUS.md](DIOXUS.md)                                                             | Dioxus 0.7 app setup (`dx`, features, layout) |

## Prerequisites

- **Rust** 1.85+ (edition 2024)
- **Nix** + **direnv** (recommended) — `use flake` from repo root
- **Dioxus CLI** (`dx`) — required from Phase 1 onward ([official getting started](https://dioxuslabs.com/learn/0.7/getting_started/))

```bash
# In Nix/direnv shell (recommended): dx is already available
direnv allow

# Otherwise install dx (prebuilt binary — preferred over cargo install)
curl -sSL https://dioxus.dev/install.sh | bash

# Web builds also need the WASM target
rustup target add wasm32-unknown-unknown

# Verify toolchain + platform deps
dx doctor
```

See **[DIOXUS.md](DIOXUS.md)** for the full Profile Pulse ↔ Dioxus 0.7 mapping.

## Environment setup

```bash
cd /path/to/Profile-Pulse
direnv allow   # or: nix develop
```

Verify tooling:

```bash
rustc --version
cargo --version
nix flake check   # formatting + flake checks
```

## Repository layout (rewrite)

```text
crates/
  core/                      # profile-pulse-core — domain + services
  storage/                   # profile-pulse-storage — vdir + SQLite index
  pic-source-plugin-api/     # profile pic source plugin trait + types (Phase 2+)
  pic-source-plugin-host/    # built-in plugins + registry (Phase 2+)
  sync/                      # profile-pulse-sync — cloud adapters (Phase 5+)
  app/                       # profile-pulse-app — Dioxus UI (Phase 1+)
pic-source-plugins/          # WASM plugins (Phase 3+)
legacy/
  iced-app/       # frozen pre-rewrite app
```

Data directory (desktop): `~/.local/share/profile-pulse/`

## Common commands

### Phase 0 (core + storage)

```bash
# Run all workspace tests
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --all

# Check specific crates
cargo test -p profile-pulse-core
cargo test -p profile-pulse-storage
```

### Phase 1+ (Dioxus desktop)

Linux desktop builds need GTK 3, WebKitGTK 4.1, and libxdo. The Nix devshell includes these (`gtk3`, `webkitgtk_4_1`, `xdotool` for libxdo).

On Fedora without Nix:

```bash
sudo dnf install gtk3-devel webkit2gtk4.1-devel librsvg2-devel openssl-devel libxdo-devel
```

Run the app:

```bash
cargo run -p profile-pulse-app
# or with hot reload:
cd crates/app && dx serve --platform desktop
```

Profile bundles use extension `.pp-profile` (zip containing `profile.toml` + `aggregate.vcf`).
Scheduled backups run once on each app start when enabled for a profile.

### Phase 5+ (Cloud sync)

Set a Google OAuth desktop client ID (People API / Contacts scope):

```bash
export PROFILE_PULSE_GOOGLE_CLIENT_ID="your-client-id.apps.googleusercontent.com"
```

Link targets under **Sync** in the app header. Push is default — use **Sync contact** on a contact's Details tab. CardDAV uses server URL + username/app password stored locally under `{data_dir}/secrets/`.

### Phase 7 (Background sync)

While the desktop app is active, remote changes are polled every 15 minutes for the active profile. When changes are detected, a header banner links to **Sync** settings where you can **Pull remote changes**. Conflicts offer **Keep local**, **Take remote**, or **Review in editor** on the contact detail page.

### Phase 8 (Release readiness)

Rewrite crates (`core`, `storage`, `sync`, `app`, `pic-source-plugin-*`) should pass:

```bash
cargo clippy -p profile-pulse-app -p profile-pulse-sync -p profile-pulse-core \
  -p profile-pulse-storage -p profile-pulse-pic-source-plugin-host \
  -p profile-pulse-pic-source-plugin-api -- -D warnings
cargo test --workspace
```

The frozen `legacy/iced-app` crate may retain warnings.

### Phase 6 (Dioxus web PWA)

Follow **[DIOXUS.md](DIOXUS.md)** and the [official getting started guide](https://dioxuslabs.com/learn/0.7/getting_started/).

From `crates/app` (where `Dioxus.toml` lives):

```bash
cd crates/app
dx serve --platform web      # dev server with hot reload
```

Or compile WASM directly:

```bash
cargo build -p profile-pulse-app \
  --target wasm32-unknown-unknown \
  --no-default-features \
  --features web
```

Set a vault passphrase before storing sync credentials in the browser:

```bash
export PROFILE_PULSE_VAULT_PASSPHRASE="choose-a-strong-passphrase"
```

Data is stored in OPFS (`OpfsVdirBackend`) with sync secrets encrypted in `localStorage`. Cloud sync push/pull is not yet wired for web.

### Pre-commit

```bash
prek --all-files
# or after Phase 0 workspace exists:
prek run --all-files
```

### Phase 7+ (Background sync)

### Formatting (Nix)

```bash
nix fmt
nix flake check
```

## Branch naming

Use: `<first-name>/<type>/<work-name>` — e.g. `mrdgh2821/feat/phase-0-core`.

## Implementation workflow

1. Read the **phase plan** in `docs/superpowers/plans/`.
2. Implement tasks in order; each task ends with tests + commit.
3. Update checkboxes in [ROADMAP.md](ROADMAP.md) when a phase item completes.
4. Log AI-assisted work in `.agents/logs/YYYY-MM-DD.md`.
5. Include `Co-authored-by: Composer via Cursor <cursoragent@cursor.com>` on AI-assisted commits.

## Profile pic source plugin development (Phase 2+)

- Built-in plugins live in `crates/pic-source-plugin-host/src/builtins/`.
- User WASM packages use extension **`.pp-pic-source-plugin`** with `kind = "profile-pic-source"`.
- Sample plugin: `pic-source-plugins/sample-hello-pic-source/` — build with:

```bash
./scripts/build-sample-pic-source.sh
```

- Implement `ProfilePicSourcePlugin` from `profile-pulse-pic-source-plugin-api`.
- WASM plugins use JSON exports (`discover`, `fetch_pic`) and host imports (`env.http_get`).
- Plugins must not access the filesystem or network directly — use `PicSourceHostApi`.

## Testing conventions

- Unit tests: in-module `#[cfg(test)]`
- Integration tests: `crates/<crate>/tests/`
- Use `tempfile` for vdir backend tests
- Use `mockito` for HTTP in pic source plugin tests (Phase 2+)

## Troubleshooting

| Issue                                         | Action                                                 |
| --------------------------------------------- | ------------------------------------------------------ |
| Pre-commit cspell fail                        | Add terms to `.cspell.json` `words`                    |
| Cocogitto scope rejected                      | Use types without scope or valid scope from `cog.toml` |
| `edition 2024` errors                         | Update Rust: `rustup update stable`                    |
| Legacy Iced build breaks after workspace move | `cargo check -p profile-pulse-legacy`                  |

## AI transparency

All AI-assisted sessions require an entry in `.agents/logs/YYYY-MM-DD.md` per [AGENTS.md](../AGENTS.md).
