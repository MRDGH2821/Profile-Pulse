# Build Reference

Comprehensive build guide for Profile Pulse.

---

## Quick Start

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/yourusername/profile-pulse.git
cd profile-pulse

# Build
cargo build

# Run
cargo run

# Test
cargo test
```

---

## Prerequisites

### Required

- **Rust 1.75.0+** - Install from rustup.rs
- **SQLite 3.35.0+** - Usually pre-installed

### System Dependencies

#### Ubuntu/Debian

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libsqlite3-dev libssl-dev
```

#### macOS

```bash
brew install sqlite3 pkg-config
```

#### Fedora

```bash
sudo dnf install -y gcc pkg-config sqlite-devel openssl-devel
```

#### Windows

- Install Visual Studio Build Tools
- SQLite bundled with sqlx

---

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check

# Lint
cargo clippy

# Format
cargo fmt
```

---

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

---

## Database

```bash
# Run migrations
cargo sqlx migrate run

# Create migration
cargo sqlx migrate create add_new_table
```

---

## Dependencies

Key crates:

```toml
iced = "0.12"
tokio = "1.35"
sqlx = "0.7"
reqwest = "0.11"
thiserror = "1.0"
tracing = "0.1"
```

---

## More Details

Full guide: `.agents/BUILDING.md` (386 lines)

---

**Status**: Reference Only
