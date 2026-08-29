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

Matches the [Desktop template](https://github.com/DioxusLabs/dioxus-template/tree/main/Desktop) with workspace-specific additions:

```text
crates/app/
├── .cargo/config.toml  # target-dir → workspace target/
├── Cargo.toml          # features: desktop (default), web
├── Dioxus.toml         # dx serve / bundle / watcher config
├── assets/             # manganis assets (asset! macro)
│   └── styles.css
├── icons/              # bundle icons for desktop installers
│   └── icon.png
├── public/             # static files copied to dist (PWA manifest, etc.)
│   └── manifest.json
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

| `dx` flag            | Cargo feature enabled | Dioxus feature   |
| -------------------- | --------------------- | ---------------- |
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

Full config generated from the official template + `dx config init` (see `crates/app/Dioxus.toml`):

```toml
[application]
name = "profile-pulse"
default_platform = "desktop"
out_dir = "dist"
asset_dir = "assets"
public_dir = "public"

[web.app]
title = "Profile Pulse"

[web.watcher]
reload_html = true
index_on_404 = true
watch_path = ["src", "assets", "public"]

[web.resource.dev]
style = ["/assets/styles.css"]

[bundle]
identifier = "com.github.mrdgh2821.profile-pulse"
icon = ["icons/icon.png"]
```

Styles are also loaded in `lib.rs` via `asset!("/assets/styles.css")` for desktop. The `[web.resource.dev]` entry ensures the dev server injects CSS for web hot reload.

### Web toolchain notes

```bash
rustup target add wasm32-unknown-unknown
# wasm-bindgen-cli must match Cargo.lock — check with:
grep 'name = "wasm-bindgen"$' -A1 ../../Cargo.lock
cargo install wasm-bindgen-cli --version 0.2.127 --locked
```

The Nix devshell includes `wasm-bindgen-cli` (nixpkgs may lag the lockfile; install the matching version if `dx build --platform web` reports a mismatch).

## Platform-specific app state

| Target  | Storage                        | Secrets / sync links     | File dialogs        |
| ------- | ------------------------------ | ------------------------ | ------------------- |
| Desktop | `FsVdirBackend` + SQLite index | Filesystem + rusqlite    | `rfd`               |
| Web     | `OpfsVdirBackend` + JSON index | Encrypted `localStorage` | Deferred (Phase 6+) |

`src/state.rs` re-exports `desktop` or `web` modules via `#[cfg(target_arch = "wasm32")]`.

### Web vault passphrase

Before storing sync credentials in the browser:

```bash
export PROFILE_PULSE_VAULT_PASSPHRASE="choose-a-strong-passphrase"
```

## Troubleshooting

| Issue                                                          | Action                                                                                                                                                        |
| -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `dx: command not found`                                        | `direnv allow`, or install via [getting started](https://dioxuslabs.com/learn/0.7/getting_started/)                                                           |
| Web build fails on tokio/mio                                   | Build with `--no-default-features --features web`; network crates are cfg-gated                                                                               |
| Desktop missing WebKit/GTK                                     | Use Nix devshell or install Linux deps above                                                                                                                  |
| `dx doctor` warnings                                           | Follow its suggestions; add `wasm32-unknown-unknown` for web                                                                                                  |
| GTK `colorreload-gtk-module` / `window-decorations-gtk-module` | KDE writes `gtk-modules=…` in `~/.config/gtk-3.0/settings.ini`. Nix devshell sets `GTK_MODULES=""` to override. Run `direnv reload`, then restart `dx serve`. |
| `GStreamer element appsink not found`                          | Use the Nix devshell (includes GStreamer + `GST_PLUGIN_SYSTEM_PATH_1_0`). Harmless for this app if you are not playing media.                                 |
