# Profile Pulse App

Dioxus 0.7 application shell (desktop + web). Follows the [official Desktop template](https://github.com/DioxusLabs/dioxus-template/tree/main/Desktop) inside the Cargo workspace.

## Layout

```text
Cargo.toml
Dioxus.toml          # dx serve / bundle config
.cargo/config.toml   # workspace target-dir
assets/              # manganis assets (asset! macro)
icons/               # bundle icons (desktop installers)
public/              # static files copied to dist (manifest, etc.)
src/
  main.rs            # entrypoint
  lib.rs             # dioxus::launch(App)
```

## Run

All `dx` commands run from **this directory**:

```bash
dx serve --platform desktop
dx serve --platform web
dx build --platform desktop
dx bundle --package-types deb
```

## Toolchain

- `dx` — from Nix devshell (`dioxus-cli`) or [install script](https://dioxus.dev/install.sh)
- `rustup target add wasm32-unknown-unknown` — required for web
- `wasm-bindgen-cli` version must match `Cargo.lock` (run `dx build --platform web`; if mismatched, `cargo install wasm-bindgen-cli --version <lockfile-version> --locked`)

See [docs/DIOXUS.md](../../docs/DIOXUS.md) for full setup.
