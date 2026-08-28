# Profile Pulse App

Dioxus 0.7 application shell (desktop + web). Matches the [bare-bones template](https://dioxuslabs.com/learn/0.7/tutorial/new_app/) layout inside the Cargo workspace.

## Layout

```text
Cargo.toml
Dioxus.toml
assets/
src/
  main.rs    # entrypoint
  lib.rs     # dioxus::launch(App)
```

## Run

```bash
# from this directory
dx serve --platform desktop
dx serve --platform web
```

See [docs/DIOXUS.md](../../docs/DIOXUS.md) for toolchain setup and workspace conventions.
