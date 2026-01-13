# Quick Start Guide

Get Profile Pulse running in 5 minutes.

## Prerequisites

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

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

## Common Commands

### Development
```bash
# Build (debug mode)
cargo build

# Build (release mode - optimized)
cargo build --release

# Run application
cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Auto-rebuild on changes
cargo install cargo-watch
cargo watch -x run
```

### Testing
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_contact_creation

# Run tests in specific module
cargo test db::tests
```

### Code Quality
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Lint code
cargo clippy

# Fix auto-fixable lints
cargo clippy --fix
```

### Database
```bash
# Database is created automatically on first run
# Default location:
# - Linux: ~/.local/share/profile-pulse/profile-pulse.db
# - macOS: ~/Library/Application Support/com.profile-pulse.Profile-Pulse/profile-pulse.db
# - Windows: %APPDATA%\profile-pulse\Profile Pulse\data\profile-pulse.db

# Use custom database location
export DATABASE_PATH=./my-database.db
cargo run

# Reset database (delete and recreate)
rm ~/.local/share/profile-pulse/profile-pulse.db  # Linux
cargo run
```

### Configuration
```bash
# Copy example environment file
cp .env.example .env

# Edit with your settings
nano .env  # or vim, code, etc.

# Common environment variables
export RUST_LOG=debug
export DATABASE_PATH=./profile-pulse.db
export DEBUG=1
```

## Troubleshooting

### Build Errors

**Problem**: `linker 'cc' not found`
```bash
# Ubuntu/Debian
sudo apt install build-essential

# Fedora
sudo dnf install gcc

# macOS
xcode-select --install
```

**Problem**: `could not find native static library sqlite3`
```bash
# Ubuntu/Debian
sudo apt install libsqlite3-dev

# Fedora
sudo dnf install sqlite-devel

# macOS
brew install sqlite3
```

**Problem**: Dependencies fail to compile
```bash
# Clean and rebuild
cargo clean
cargo build
```

### Runtime Errors

**Problem**: Permission denied
- Check data directory permissions
- Ensure sufficient disk space

**Problem**: Database locked
- Close other instances of the application
- Remove stale lock files

## Development Workflow

1. **Make changes** to source code
2. **Format** code: `cargo fmt`
3. **Check** for errors: `cargo check`
4. **Run** tests: `cargo test`
5. **Lint** code: `cargo clippy`
6. **Build** release: `cargo build --release`

## Next Steps

- Read [BUILDING.md](BUILDING.md) for detailed build instructions
- Review [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for development guide
- Check [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) to understand the codebase
- See [STATUS.md](STATUS.md) for current project status
- Review [ROADMAP.md](docs/ROADMAP.md) for planned features

## Getting Help

- **Build issues**: See [BUILDING.md](BUILDING.md)
- **Development questions**: See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)
- **Bug reports**: Open an issue on GitHub
- **Feature requests**: Check [ROADMAP.md](docs/ROADMAP.md) first

---

**Ready to contribute?** See [AGENTS.md](AGENTS.md) for AI assistant guidelines and [DISCLOSURE.md](DISCLOSURE.md) for transparency about AI-generated code.