# Development Reference

This file contains development setup information for Profile Pulse.

**Note**: This is a reference document, not a specification.

---

## Prerequisites

**Required**:

- Rust 1.75.0 or higher
- SQLite 3.35.0 or higher
- Git
- C compiler (for native dependencies)

**Optional**:

- rust-analyzer
- cargo-watch
- cargo-nextest
- cargo-edit

---

## Quick Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/profile-pulse.git
cd profile-pulse

# Build the project
cargo build

# Run the application
cargo run

# Run tests
cargo test
```

---

## Environment Variables

Create a `.env` file:

```env
DATABASE_URL=sqlite:./profile-pulse.db
RUST_LOG=profile_pulse=debug,info
GITHUB_TOKEN=your_github_personal_access_token
```

---

## System Dependencies

### macOS

```bash
brew install sqlite3 pkg-config
```

### Ubuntu/Debian

```bash
sudo apt install -y build-essential pkg-config libsqlite3-dev libssl-dev
```

### Fedora

```bash
sudo dnf install -y gcc pkg-config sqlite-devel openssl-devel
```

---

## Project Structure

```
profile-pulse/
├── src/
│   ├── main.rs
│   ├── core/        # Domain models
│   ├── db/         # Database layer
│   ├── vcf/        # VCF import/export
│   ├── workspace/   # Workspace management
│   ├── ui/         # Iced GUI
│   ├── social/     # Profile fetchers
│   ├── discovery/  # Profile discovery
│   └── utils/      # Utilities
├── tests/           # Integration tests
├── Cargo.toml      # Project manifest
└── .env.example   # Config template
```

---

## Building and Testing

```bash
# Format code
cargo fmt

# Check for errors
cargo check

# Run linter
cargo clippy

# Run tests
cargo test

# Build for release
cargo build --release
```

---

## Database Development

```bash
# Run migrations
cargo sqlx migrate run

# Create new migration
cargo sqlx migrate create add_new_table
```

---

## More Details

Full documentation: `docs/DEVELOPMENT.md` (794 lines)

---

**Status**: Reference Only  
**Last Updated**: 2026-01-15
