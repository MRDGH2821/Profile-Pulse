# Quick Start Reference

Get Profile Pulse running in 5 minutes.

---

## Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

---

## Build and Run

```bash
# Clone repository
git clone https://github.com/yourusername/profile-pulse.git
cd profile-pulse

# First build (may take 5-10 minutes)
cargo build

# Run tests
cargo test

# Start application
cargo run
```

---

## Common Commands

| Command                 | Description      |
| ----------------------- | ---------------- |
| `cargo build`           | Build debug      |
| `cargo build --release` | Build optimized  |
| `cargo test`            | Run tests        |
| `cargo run`             | Run application  |
| `cargo fmt`             | Format code      |
| `cargo clippy`          | Run linter       |
| `cargo check`           | Check for errors |

---

## System Dependencies

### Ubuntu/Debian

```bash
sudo apt install -y build-essential pkg-config libsqlite3-dev libssl-dev
```

### macOS

```bash
brew install sqlite3 pkg-config
```

### Fedora

```bash
sudo dnf install -y gcc pkg-config sqlite-devel openssl-devel
```

---

## More Details

Full guide: `.agents/BUILDING.md` (386 lines)

---

**Status**: Reference Only
