# Development Guide: Profile Pulse

## Table of Contents

- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Building the Project](#building-the-project)
- [Testing](#testing)
- [Code Style](#code-style)
- [Database Development](#database-development)
- [Debugging](#debugging)
- [Contributing](#contributing)
- [Troubleshooting](#troubleshooting)

---

## Getting Started

### Prerequisites

**Required**:

- Rust 1.75.0 or higher ([Install Rust](https://rustup.rs))
- SQLite 3.35.0 or higher
- Git
- C compiler (for native dependencies)

**Optional but Recommended**:

- [rust-analyzer](https://rust-analyzer.github.io/) for IDE support
- [cargo-watch](https://crates.io/crates/cargo-watch) for auto-rebuild
- [cargo-nextest](https://nextest-rs.github.io/nextest/) for faster testing
- [cargo-edit](https://crates.io/crates/cargo-edit) for dependency management

### Quick Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/profile-pulse.git
cd profile-pulse

# Install additional cargo tools (optional)
cargo install cargo-watch cargo-nextest cargo-edit

# Build the project
cargo build

# Run the application
cargo run

# Run tests
cargo test
```

---

## Development Environment

### Recommended IDEs

#### VS Code

**Extensions**:

- `rust-analyzer` - Language server
- `CodeLLDB` - Debugger
- `crates` - Dependency management
- `Better TOML` - TOML syntax highlighting
- `Error Lens` - Inline error display

**Settings** (`.vscode/settings.json`):

```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "rust-analyzer.lens.enable": true
}
```

#### RustRover / IntelliJ IDEA

- Full Rust support built-in
- Excellent debugging capabilities
- Database integration tools

#### Neovim/Vim

- `nvim-lspconfig` with rust-analyzer
- `rust.vim` or `rust-tools.nvim`
- `nvim-dap` for debugging

### Environment Variables

Create a `.env` file in the project root:

```env
# Database
DATABASE_URL=sqlite:./profile-pulse.db

# Logging
RUST_LOG=profile_pulse=debug,info

# API Keys (optional)
GITHUB_TOKEN=your_github_personal_access_token
TWITTER_BEARER_TOKEN=your_twitter_bearer_token
GOOGLE_SEARCH_API_KEY=your_google_api_key

# Development settings
CACHE_DIR=./cache
DATA_DIR=./data
```

### System Dependencies

#### macOS

```bash
# Using Homebrew
brew install sqlite3
brew install pkg-config
```

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

#### Windows

- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
- SQLite is bundled with `sqlx`

---

## Project Structure

```
profile-pulse/
├── .github/              # GitHub Actions workflows
├── .vscode/              # VS Code settings (gitignored)
├── docs/                 # Documentation
├── src/
│   ├── main.rs          # Application entry point
│   ├── app.rs           # Main app state and logic
│   ├── ui/              # User interface components
│   ├── core/            # Business logic
│   ├── db/              # Database layer
│   ├── social/          # Social media integrations
│   ├── discovery/       # Profile discovery
│   └── utils/           # Utilities and helpers
├── tests/               # Integration tests
├── assets/              # Static assets
├── Cargo.toml           # Project manifest
└── Cargo.lock           # Dependency lock file
```

### Key Files

- `Cargo.toml` - Project dependencies and metadata
- `.env` - Environment variables (not committed)
- `sqlx-data.json` - Cached SQLx query metadata
- `rust-toolchain.toml` - Rust toolchain version

---

## Building the Project

### Development Build

```bash
# Standard debug build
cargo build

# With all features
cargo build --all-features

# Release build (optimized)
cargo build --release
```

### Build Profiles

**Debug** (default):

- Fast compilation
- Includes debug symbols
- No optimizations
- ~10x slower runtime

**Release**:

- Slower compilation
- Optimized binary
- Strip debug symbols
- Production-ready

**Custom Profile** (in `Cargo.toml`):

```toml
[profile.dev-opt]
inherits = "dev"
opt-level = 1

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
```

### Feature Flags

```bash
# Build with specific features
cargo build --features "headless"

# Build without default features
cargo build --no-default-features

# List all features
cargo metadata --format-version=1 | jq '.packages[0].features'
```

---

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_contact_creation

# Run tests in specific module
cargo test db::tests

# Run with multiple threads
cargo test -- --test-threads=4
```

### Test Organization

```rust
// Unit tests (in same file as code)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_validation() {
        let contact = Contact::new("John Doe");
        assert!(contact.is_valid());
    }
}

// Integration tests (in tests/ directory)
#[tokio::test]
async fn test_full_import_flow() {
    // Test implementation
}
```

### Testing with Database

```bash
# Set test database
export DATABASE_URL=sqlite::memory:

# Run database tests
cargo test --features "test-db"
```

### Mocking External APIs

```rust
#[cfg(test)]
mod tests {
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_github_api() {
        let _m = mock("GET", "/users/octocat")
            .with_status(200)
            .with_body(r#"{"login": "octocat"}"#)
            .create();

        // Test with mock server
    }
}
```

### Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Open report
open coverage/index.html
```

---

## Code Style

### Formatting

```bash
# Format all code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check

# Format specific file
rustfmt src/main.rs
```

### Linting

```bash
# Run Clippy
cargo clippy

# Clippy with all features
cargo clippy --all-features

# Clippy as error
cargo clippy -- -D warnings

# Fix automatically (when possible)
cargo clippy --fix
```

### Style Guidelines

**Naming Conventions**:

- `snake_case` for functions, variables, modules
- `PascalCase` for types, traits, enums
- `SCREAMING_SNAKE_CASE` for constants
- Prefix unused variables with `_`

**Code Organization**:

```rust
// 1. Imports (grouped and sorted)
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::core::Contact;

// 2. Type definitions
pub struct ContactManager {
    repository: Arc<ContactRepository>,
}

// 3. Trait implementations
impl ContactManager {
    pub fn new(repository: Arc<ContactRepository>) -> Self {
        Self { repository }
    }
}

// 4. Tests
#[cfg(test)]
mod tests {
    use super::*;
}
```

**Documentation**:

````rust
/// Manages contact operations including CRUD and search.
///
/// # Examples
///
/// ```
/// let manager = ContactManager::new(repository);
/// let contact = manager.create_contact(data).await?;
/// ```
pub struct ContactManager {
    repository: Arc<ContactRepository>,
}
````

---

## Database Development

### Migrations

```bash
# Create new migration
sqlx migrate add create_contacts_table

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info
```

### Migration File Example

```sql
-- migrations/001_create_contacts_table.sql
CREATE TABLE contacts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_contacts_name ON contacts(name);
```

### SQLx Compile-Time Verification

```bash
# Prepare query metadata (offline mode)
cargo sqlx prepare

# This generates sqlx-data.json
# Commit this file for CI/CD
```

### Database Testing

```rust
#[sqlx::test]
async fn test_create_contact(pool: SqlitePool) -> sqlx::Result<()> {
    let repo = ContactRepository::new(pool);

    let contact = Contact {
        id: Uuid::new_v4(),
        name: "Test User".to_string(),
        email: Some("test@example.com".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    repo.create(&contact).await?;

    let retrieved = repo.read(contact.id).await?;
    assert!(retrieved.is_some());

    Ok(())
}
```

---

## Debugging

### Logging

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Specific module logging
RUST_LOG=profile_pulse::social=trace cargo run

# Multiple modules
RUST_LOG=profile_pulse::social=debug,profile_pulse::db=info cargo run
```

### Using tracing

```rust
use tracing::{debug, info, warn, error, trace};

pub async fn fetch_profile(&self, username: &str) -> Result<Profile> {
    info!("Fetching profile for {}", username);

    match self.api_fetch(username).await {
        Ok(profile) => {
            debug!("Successfully fetched profile: {:?}", profile);
            Ok(profile)
        }
        Err(e) => {
            error!("Failed to fetch profile: {}", e);
            Err(e)
        }
    }
}
```

### VS Code Debugging

**launch.json**:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Profile Pulse",
      "cargo": {
        "args": ["build", "--bin=profile-pulse"]
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug"
      }
    }
  ]
}
```

### Performance Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph (Linux only)
cargo flamegraph

# CPU profiling with perf (Linux)
perf record -g cargo run
perf report
```

---

## Contributing

### Workflow

1. **Fork the repository**
2. **Create a feature branch**
   ```bash
   git checkout -b feature/my-new-feature
   ```
3. **Make your changes**
4. **Run tests and linting**
   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```
5. **Commit with conventional commits**
   ```bash
   git commit -m "feat(social): add Twitter profile fetching"
   ```
6. **Push and create pull request**

### Commit Message Format

Following [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types**:

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements

**Scopes**:

- `ui`: User interface changes
- `core`: Business logic
- `db`: Database layer
- `social`: Social media integrations
- `discovery`: Profile discovery
- `deps`: Dependencies

**Examples**:

```
feat(social): add LinkedIn profile picture fetching
fix(db): resolve connection pool deadlock
docs(api): update GitHub API documentation
refactor(ui): simplify contact list rendering
test(core): add contact validation tests
```

### Code Review Checklist

Before submitting PR:

- [ ] All tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] No Clippy warnings (`cargo clippy`)
- [ ] Documentation updated
- [ ] Commit messages follow convention
- [ ] No merge conflicts
- [ ] Feature flag added if needed
- [ ] Migration added if schema changed

### Pull Request Template

```markdown
## Description

Brief description of changes

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist

- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings
```

---

## Troubleshooting

### Common Issues

#### Compilation Errors

**Issue**: `error: linker 'cc' not found`

```bash
# Install C compiler
# macOS: xcode-select --install
# Ubuntu: sudo apt install build-essential
# Windows: Install Visual Studio Build Tools
```

**Issue**: `error: could not find native static library sqlite3`

```bash
# Install SQLite development files
# Ubuntu: sudo apt install libsqlite3-dev
# macOS: brew install sqlite3
```

#### Runtime Issues

**Issue**: Database locked

```rust
// Increase connection pool timeout
SqlitePool::connect("sqlite:./profile-pulse.db?timeout=30")
```

**Issue**: Rate limit errors

```rust
// Check rate limiter configuration
// Verify cache is working
// Add delays between requests
```

**Issue**: Images not loading

```bash
# Check disk space
df -h

# Verify cache directory permissions
ls -la ./cache

# Check image format support
cargo build --features "image/webp"
```

### Debug Mode

```bash
# Build with maximum debug info
RUSTFLAGS="-C debuginfo=2" cargo build

# Run with backtrace
RUST_BACKTRACE=1 cargo run

# Full backtrace
RUST_BACKTRACE=full cargo run
```

### Getting Help

1. **Check existing issues**: [GitHub Issues](https://github.com/yourusername/profile-pulse/issues)
2. **Documentation**: Review [ARCHITECTURE.md](./ARCHITECTURE.md) and [PLAN.md](./PLAN.md)
3. **Ask questions**: Open a discussion on GitHub
4. **Community**: Join our Discord/Matrix channel (if available)

---

## Additional Resources

### Learning Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Iced Documentation](https://docs.rs/iced/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

### Tools

- [cargo-edit](https://crates.io/crates/cargo-edit) - Manage dependencies
- [cargo-watch](https://crates.io/crates/cargo-watch) - Auto-rebuild on changes
- [cargo-nextest](https://nextest-rs.github.io/) - Faster test runner
- [cargo-audit](https://crates.io/crates/cargo-audit) - Security auditing
- [cargo-outdated](https://crates.io/crates/cargo-outdated) - Check for updates

---

**Document Version**: 1.0  
**Last Updated**: 2024-01-XX  
**Maintained By**: Development Team

---

## Quick Reference

```bash
# Development workflow
cargo watch -x run                    # Auto-reload on changes
cargo test --workspace               # Test all packages
cargo clippy --fix --allow-dirty     # Auto-fix lints
cargo doc --open                     # Generate and open docs

# Database
sqlx migrate run                     # Apply migrations
sqlx migrate revert                  # Rollback migration
cargo sqlx prepare                   # Prepare offline mode

# Release
cargo build --release                # Optimized build
strip target/release/profile-pulse   # Strip symbols (Linux/macOS)
cargo bloat --release               # Analyze binary size
```

Happy coding! 🦀
