# Building Profile Pulse

## Quick Start

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/yourusername/profile-pulse.git
cd profile-pulse

# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run
```

## Prerequisites

### Required

- **Rust 1.75.0 or later** - Install from [rustup.rs](https://rustup.rs)
- **SQLite 3.35.0 or later** - Usually pre-installed on most systems

### System Dependencies

#### Ubuntu/Debian

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  pkg-config \
  libsqlite3-dev \
  libssl-dev
```

#### Fedora

```bash
sudo dnf install -y \
  gcc \
  pkg-config \
  sqlite-devel \
  openssl-devel
```

#### macOS

```bash
# Using Homebrew
brew install sqlite3 pkg-config
```

#### Windows

- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
- SQLite is bundled with sqlx (no separate installation needed)

## Building

### Development Build

Fast compilation, includes debug symbols, no optimizations:

```bash
cargo build
```

The binary will be at `target/debug/profile-pulse` (or `profile-pulse.exe` on Windows).

### Release Build

Optimized for performance:

```bash
cargo build --release
```

The binary will be at `target/release/profile-pulse` (or `profile-pulse.exe` on Windows).

### Build with Specific Features

```bash
# Build with all features
cargo build --all-features

# Build with specific features
cargo build --features "feature-name"
```

## Running

### Development Mode

```bash
# Run directly
cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Run with environment file
cp .env.example .env
# Edit .env with your configuration
cargo run
```

### Release Mode

```bash
# Build and run optimized version
cargo run --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_contact_creation

# Run tests in a specific module
cargo test db::tests

# Run with multiple threads
cargo test -- --test-threads=4
```

## Development Tools

### Formatting

```bash
# Format all code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

### Linting

```bash
# Run Clippy
cargo clippy

# Clippy with all features
cargo clippy --all-features

# Clippy as error
cargo clippy -- -D warnings

# Auto-fix issues (when possible)
cargo clippy --fix
```

### Documentation

```bash
# Generate and open documentation
cargo doc --open

# Generate docs for all dependencies
cargo doc --document-private-items
```

### Watch Mode (Auto-rebuild)

Install cargo-watch:

```bash
cargo install cargo-watch
```

Then use:

```bash
# Auto-rebuild on changes
cargo watch -x run

# Auto-test on changes
cargo watch -x test

# Auto-check on changes
cargo watch -x check
```

## Database Setup

The application automatically creates and migrates the database on first run.

### Manual Migration

```bash
# The database is created automatically at:
# - Linux: ~/.local/share/profile-pulse/profile-pulse.db
# - macOS: ~/Library/Application Support/com.profile-pulse.Profile-Pulse/profile-pulse.db
# - Windows: %APPDATA%\profile-pulse\Profile Pulse\data\profile-pulse.db

# Or specify custom location with environment variable
export DATABASE_PATH=./my-database.db
cargo run
```

### Reset Database

```bash
# Delete the database file
rm ~/.local/share/profile-pulse/profile-pulse.db  # Linux
rm ~/Library/Application\ Support/com.profile-pulse.Profile-Pulse/profile-pulse.db  # macOS

# Run application to recreate
cargo run
```

## Troubleshooting

### Compilation Errors

**Error: `linker 'cc' not found`**

```bash
# Install C compiler
# Ubuntu/Debian: sudo apt install build-essential
# Fedora: sudo dnf install gcc
# macOS: xcode-select --install
```

**Error: `could not find native static library sqlite3`**

```bash
# Install SQLite development files
# Ubuntu/Debian: sudo apt install libsqlite3-dev
# Fedora: sudo dnf install sqlite-devel
# macOS: brew install sqlite3
```

**Error: `error[E0433]: failed to resolve`**

```bash
# Clean and rebuild
cargo clean
cargo build
```

### Runtime Errors

**Database locked**

- Close any other instances of the application
- Check for stale lock files
- Increase connection timeout in configuration

**Permission denied**

- Ensure write permissions for data directory
- Check disk space availability

**Failed to initialize database**

- Verify SQLite is installed
- Check database path is valid
- Ensure parent directories exist

## Environment Variables

Create a `.env` file in the project root (copy from `.env.example`):

```env
# Database location (optional)
DATABASE_PATH=./profile-pulse.db

# Logging level
RUST_LOG=profile_pulse=info,warn

# Debug mode (verbose logging)
DEBUG=1

# API keys (for future use)
GITHUB_TOKEN=your_token_here
```

## CI/CD

The project includes GitHub Actions workflows:

- **Tests**: Run on all platforms (Linux, macOS, Windows)
- **Formatting**: Check code formatting
- **Clippy**: Lint code
- **Security Audit**: Check for security vulnerabilities
- **Documentation**: Verify docs build correctly

## Performance Profiling

### Flamegraph (Linux only)

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph
```

### Benchmarking

```bash
# Install criterion
cargo install cargo-criterion

# Run benchmarks (when implemented)
cargo criterion
```

## Cross-Compilation

### Linux to Windows

```bash
# Install cross-compilation target
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

### Linux to macOS

```bash
# Requires osxcross toolchain
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

## Distribution

### Creating a Release Binary

```bash
# Build optimized release
cargo build --release

# Strip symbols (Linux/macOS)
strip target/release/profile-pulse

# The binary is now ready for distribution
# Location: target/release/profile-pulse
```

### Package Sizes

- **Debug build**: ~50-100 MB
- **Release build (unstripped)**: ~20-30 MB
- **Release build (stripped)**: ~5-10 MB

## Getting Help

- Check [DEVELOPMENT.md](docs/DEVELOPMENT.md) for detailed development guide
- Review [ARCHITECTURE.md](docs/ARCHITECTURE.md) for system design
- See [ROADMAP.md](docs/ROADMAP.md) for project status
- Open an issue on GitHub for build problems

## Next Steps

After successfully building:

1. Read [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for development workflow
2. Review [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) to understand the codebase
3. Check [docs/ROADMAP.md](docs/ROADMAP.md) for current project status
4. See [AGENTS.md](AGENTS.md) for AI assistant guidelines

---

**Last Updated**: January 13, 2025  
**Status**: Phase 1 - Foundation Complete
