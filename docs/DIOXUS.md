# Dioxus 0.7 — Profile Pulse app guide

Official references:

- [Getting started](https://dioxuslabs.com/learn/0.7/getting_started/) — Rust, `dx` CLI, platform dependencies
- [Creating a new project](https://dioxuslabs.com/learn/0.7/tutorial/new_app/) — templates, `Cargo.toml` features, `Dioxus.toml`, assets

Profile Pulse uses the **Workspace** pattern from the tutorial: domain crates under `crates/` and the Dioxus shell in `crates/app`. You do not need `dx new` for this repo — the app crate already follows the same conventions a generated project would.

## Toolchain

### Rust

```bash
rustup toolchain install stable
rustup target add wasm32-unknown-unknown   # required for web / PWA
```

### Dioxus CLI (`dx`)

Preferred install paths from the [getting started guide](https://dioxuslabs.com/learn/0.7/getting_started/):

```bash
# Prebuilt binary (recommended)
curl -sSL https://dioxus.dev/install.sh | bash

# Or via cargo-binstall
cargo binstall dioxus-cli --force
```

**Nix / direnv:** `dioxus-cli` is already in the devshell (`nix/devshell.nix`). After `direnv allow`, `dx` is on `PATH`.

Verify the environment:

```bash
dx doctor
```

`dx doctor` reports missing toolchains (e.g. `wasm32-unknown-unknown`) and platform libraries (GTK/WebKit on Linux desktop).

### Linux desktop dependencies

Fedora (without Nix):

```bash
sudo dnf install gtk3-devel webkit2gtk4.1-devel librsvg2-devel openssl-devel libxdo-devel
```

Ubuntu and other distros: see the [Linux section](https://dioxuslabs.com/learn/0.7/getting_started/) in the official guide.

## Project layout (`crates/app`)

Matches the [tutorial structure](https://dioxuslabs.com/learn/0.7/tutorial/new_app/) with workspace-specific additions:

```text
crates/app/
├── Cargo.toml          # features: desktop (default), web
├── Dioxus.toml         # dx bundling / serve config
├── assets/             # static files (use asset!() macro)
│   └── styles.css
└── src/
    ├── main.rs         # fn main() { profile_pulse_app::launch(); }
    ├── lib.rs          # dioxus::launch(App), routes, views
    ├── routes.rs
    ├── state/          # desktop vs web AppState (target_arch cfg)
    └── views/
```

Domain logic stays in `crates/core`, `crates/storage`, `crates/sync`, etc. The app crate is UI + wiring only.

## Cargo features (official pattern)

`dx` enables the matching feature when you pass `--platform`:

| `dx` flag           | Cargo feature enabled | Dioxus feature   |
| ------------------- | --------------------- | ---------------- |
| `--platform desktop` | `desktop`             | `dioxus/desktop` |
| `--platform web`     | `web`                 | `dioxus/web`     |

`crates/app/Cargo.toml`:

```toml
[features]
default = ["desktop"]
desktop = ["dioxus/desktop", "dep:directories", "dep:rfd"]
web = ["dioxus/web"]
```

Equivalent manual builds:

```bash
# Desktop (native binary)
cargo run -p profile-pulse-app
cargo build -p profile-pulse-app --features desktop

# Web (WASM)
cargo build -p profile-pulse-app \
  --target wasm32-unknown-unknown \
  --no-default-features \
  --features web
```

## Running with `dx`

All `dx` commands run from **`crates/app`** (where `Dioxus.toml` lives):

```bash
cd crates/app

# Desktop — hot reload
dx serve --platform desktop

# Web — dev server (http://127.0.0.1:8080 by default)
dx serve --platform web
```

`dx serve` calls `cargo build` with the correct `--no-default-features --features <platform>` flags, same as a [new_app](https://dioxuslabs.com/learn/0.7/tutorial/new_app/) project.

## `Dioxus.toml`

Minimal config (see also `dx config init` in the CLI README):

```toml
[application]
name = "profile-pulse"
default_platform = "desktop"

[web.app]
title = "Profile Pulse"

[web.watcher]
watch_path = ["src", "assets"]

[web.resource.dev]
```

Styles are loaded in `lib.rs` via `asset!("/assets/styles.css")`. Assets can live anywhere in the tree; keeping them under `assets/` matches the official recommendation.

## Platform-specific app state

| Target  | Storage                         | Secrets / sync links      | File dialogs      |
| ------- | ------------------------------- | ------------------------- | ----------------- |
| Desktop | `FsVdirBackend` + SQLite index  | Filesystem + rusqlite     | `rfd`             |
| Web     | `OpfsVdirBackend` + JSON index  | Encrypted `localStorage`  | Deferred (Phase 6+) |

`src/state.rs` re-exports `desktop` or `web` modules via `#[cfg(target_arch = "wasm32")]`.

### Web vault passphrase

Before storing sync credentials in the browser:

```bash
export PROFILE_PULSE_VAULT_PASSPHRASE="choose-a-strong-passphrase"
```

## Troubleshooting

| Issue | Action |
| ----- | ------ |
| `dx: command not found` | `direnv allow`, or install via [getting started](https://dioxuslabs.com/learn/0.7/getting_started/) |
| Web build fails on tokio/mio | Build with `--no-default-features --features web`; network crates are cfg-gated |
| Desktop missing WebKit/GTK | Use Nix devshell or install Linux deps above |
| `dx doctor` warnings | Follow its suggestions; add `wasm32-unknown-unknown` for web |
