# Project Plan: Profile Pulse

## Table of Contents

- [Project Overview](#project-overview)
- [Technology Stack](#technology-stack)
- [GUI Framework Selection](#gui-framework-selection)
- [Project Structure](#project-structure)
- [Key Rust Crates](#key-rust-crates)
- [Data Models](#data-models)
- [Implementation Strategy](#implementation-strategy)
- [Technical Challenges](#technical-challenges)
- [Decision Log](#decision-log)

## Project Overview

**Project Name**: Profile Pulse

**Purpose**: A desktop application for managing contacts with automatic social media profile picture synchronization and profile discovery.

**Target Users**: Individuals and professionals who want to maintain up-to-date contact information with current profile pictures from social media platforms.

**Core Value Proposition**:

- Automatically sync profile pictures from multiple social media platforms
- Discover social media profiles based on contact information
- Import/export standard VCF (vCard) files
- Privacy-focused with local-only storage
- Cross-platform desktop application

## Technology Stack

### Language: Rust

**Rationale**:

- **Performance**: Native performance for desktop applications
- **Memory Safety**: Rust's ownership system prevents common bugs
- **Cross-platform**: Excellent cross-platform support
- **Small Binaries**: Compiled binaries are small and fast
- **Modern Ecosystem**: Growing ecosystem of high-quality crates
- **No Runtime**: No runtime dependencies (unlike Python/Node.js)
- **Long-term Stability**: No hurry, can leverage Rust's learning curve

### Core Technologies

| Component        | Technology                | Rationale                                   |
| ---------------- | ------------------------- | ------------------------------------------- |
| GUI Framework    | Iced                      | Pure Rust, Elm architecture, cross-platform |
| Database         | SQLite + SQLx             | Lightweight, serverless, type-safe queries  |
| HTTP Client      | Reqwest                   | Industry standard, async support            |
| Web Scraping     | Scraper + Headless Chrome | Handle both static and dynamic content      |
| Image Processing | image crate               | Pure Rust, comprehensive format support     |
| VCF Parsing      | vcard crate               | Standard VCF/vCard format support           |
| Async Runtime    | Tokio                     | Most mature async runtime for Rust          |
| Error Handling   | thiserror + anyhow        | Ergonomic error handling                    |
| Logging          | tracing                   | Structured logging                          |
| Serialization    | Serde                     | De-facto standard for serialization         |

## GUI Framework Selection

### Options Considered

#### 1. Iced ⭐ (Selected)

**Pros**:

- Pure Rust implementation
- Elm-inspired architecture (predictable state management)
- Cross-platform (Windows, macOS, Linux, Web)
- Modern, responsive design capabilities
- Type-safe UI construction
- Good for complex state management
- Active development

**Cons**:

- Smaller ecosystem compared to web-based solutions
- Styling requires more code than CSS
- Fewer pre-built components
- Learning curve for Elm architecture

**Decision**: Selected for pure Rust experience and type safety

#### 2. Tauri

**Pros**:

- Beautiful modern UI with web technologies
- Very small bundle size (~3-5MB)
- Leverage existing web skills
- Rich ecosystem of UI components

**Cons**:

- Requires web technology knowledge (HTML/CSS/JS)
- Not "pure Rust" (uses webview)
- Slight overhead from webview layer

**Decision**: Excellent alternative if UI development becomes too challenging with Iced

#### 3. Egui

**Pros**:

- Immediate mode GUI (very fast development)
- Pure Rust
- Excellent for data-heavy applications
- Hot reloading support

**Cons**:

- Less native-looking
- Different paradigm (immediate mode)
- Limited styling options

**Decision**: Good for prototyping but may not provide desired UX

## Project Structure

```
profile-pulse/
├── Cargo.toml                  # Project manifest
├── Cargo.lock                  # Dependency lock file
├── .cargo/
│   └── config.toml            # Cargo configuration
├── src/
│   ├── main.rs                # Entry point
│   ├── app.rs                 # Main application state and logic
│   │
│   ├── ui/                    # User interface layer
│   │   ├── mod.rs
│   │   ├── contacts_view.rs   # Contact list view
│   │   ├── contact_detail.rs  # Contact detail/edit view
│   │   ├── settings.rs        # Settings/preferences view
│   │   ├── discovery.rs       # Profile discovery view
│   │   └── components/        # Reusable UI components
│   │       ├── mod.rs
│   │       ├── contact_card.rs
│   │       ├── image_viewer.rs
│   │       ├── profile_badge.rs
│   │       └── search_bar.rs
│   │
│   ├── core/                  # Business logic layer
│   │   ├── mod.rs
│   │   ├── contact.rs         # Contact model and operations
│   │   ├── vcf.rs             # VCF import/export logic
│   │   └── social_media.rs    # Social media profile types
│   │
│   ├── db/                    # Data access layer
│   │   ├── mod.rs
│   │   ├── schema.rs          # Database schema definitions
│   │   ├── models.rs          # Database models
│   │   ├── repository.rs      # Data access repository
│   │   └── migrations/        # SQLx migrations
│   │       └── 001_initial.sql
│   │
│   ├── social/                # Social media integration layer
│   │   ├── mod.rs
│   │   ├── traits.rs          # ProfileFetcher trait
│   │   ├── linkedin.rs        # LinkedIn integration
│   │   ├── twitter.rs         # Twitter/X integration
│   │   ├── facebook.rs        # Facebook integration
│   │   ├── instagram.rs       # Instagram integration
│   │   ├── github.rs          # GitHub integration
│   │   └── cache.rs           # Caching layer for fetched data
│   │
│   ├── discovery/             # Profile discovery layer
│   │   ├── mod.rs
│   │   ├── search.rs          # Search engine integration
│   │   ├── matcher.rs         # Profile matching algorithms
│   │   └── scorer.rs          # Confidence scoring
│   │
│   └── utils/                 # Utility modules
│       ├── mod.rs
│       ├── http.rs            # HTTP client setup and middleware
│       ├── image.rs           # Image processing utilities
│       ├── error.rs           # Error types and conversions
│       └── config.rs          # Configuration management
│
├── tests/                     # Integration tests
│   ├── integration/
│   │   ├── vcf_tests.rs
│   │   ├── social_tests.rs
│   │   └── db_tests.rs
│   └── fixtures/
│       ├── sample_contacts.vcf
│       └── mock_profiles.json
│
├── assets/                    # Static assets
│   ├── icons/
│   │   ├── app.ico
│   │   └── app.icns
│   ├── fonts/
│   └── images/
│
├── docs/                      # Documentation
│   ├── PLAN.md               # This file
│   ├── ARCHITECTURE.md
│   ├── ROADMAP.md
│   ├── API_INTEGRATION.md
│   └── DEVELOPMENT.md
│
└── scripts/                   # Build and utility scripts
    ├── setup.sh
    └── package.sh
```

## Key Rust Crates

### Production Dependencies

```toml
[dependencies]
# GUI Framework
iced = { version = "0.12", features = ["tokio", "image", "debug"] }

# Async Runtime
tokio = { version = "1.35", features = ["full"] }

# Database
sqlx = { version = "0.7", features = [
    "sqlite",
    "runtime-tokio-rustls",
    "migrate",
    "chrono",
    "uuid"
] }

# VCF/vCard Support
vcard = "0.3"

# HTTP Client
reqwest = { version = "0.11", features = [
    "json",
    "cookies",
    "rustls-tls"
] }
reqwest-middleware = "0.2"
reqwest-retry = "0.3"

# Web Scraping
scraper = "0.18"
# For JavaScript-heavy sites (optional)
headless_chrome = { version = "1.0", optional = true }

# Image Processing
image = { version = "0.24", features = ["jpeg", "png", "webp"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error Handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"

# Configuration
config = "0.13"
directories = "5.0"  # Platform-specific directories

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1.6", features = ["v4", "serde"] }

# Fuzzy Matching (for profile discovery)
strsim = "0.10"
fuzzy-matcher = "0.3.7"

# Rate Limiting
governor = "0.6"

# Caching
moka = { version = "0.12", features = ["future"] }

# Async Traits
async-trait = "0.1"

[dev-dependencies]
# Testing
mockito = "1.2"
tempfile = "3.8"
tokio-test = "0.4"
insta = "1.34"  # Snapshot testing

# Benchmarking
criterion = "0.5"
```

### Crate Justifications

| Crate       | Purpose           | Why This One                               |
| ----------- | ----------------- | ------------------------------------------ |
| `iced`      | GUI framework     | Pure Rust, type-safe, Elm architecture     |
| `tokio`     | Async runtime     | Most mature and widely used                |
| `sqlx`      | Database          | Compile-time query checking, async support |
| `reqwest`   | HTTP client       | Industry standard, well-maintained         |
| `scraper`   | HTML parsing      | Fast, ergonomic CSS selector API           |
| `image`     | Image processing  | Comprehensive format support               |
| `thiserror` | Error definitions | Derive macros for clean error types        |
| `anyhow`    | Error handling    | Context and error chaining                 |
| `tracing`   | Logging           | Structured logging, async-aware            |
| `governor`  | Rate limiting     | Token bucket algorithm, async support      |
| `moka`      | Caching           | High-performance, concurrent cache         |

## Data Models

### Core Contact Model

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub organization: Option<String>,
    pub title: Option<String>,
    pub photo_url: Option<String>,
    pub photo_blob: Option<Vec<u8>>,  // Cached image data
    pub social_profiles: Vec<SocialProfile>,
    pub custom_fields: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialProfile {
    pub id: Uuid,
    pub platform: SocialPlatform,
    pub username: String,
    pub url: String,
    pub profile_pic_url: Option<String>,
    pub verified: bool,
    pub confidence_score: Option<f32>,  // 0.0-1.0 for discovered profiles
    pub discovered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SocialPlatform {
    LinkedIn,
    Twitter,
    Facebook,
    Instagram,
    GitHub,
    Custom(String),
}
```

### ProfileFetcher Trait

```rust
use async_trait::async_trait;

#[async_trait]
pub trait ProfileFetcher: Send + Sync {
    /// Fetch profile picture by username
    async fn fetch_profile_pic(&self, username: &str) -> Result<Vec<u8>, FetchError>;

    /// Search for profiles matching contact information
    async fn search_profile(
        &self,
        name: &str,
        email: Option<&str>,
    ) -> Result<Vec<ProfileMatch>, FetchError>;

    /// Get the platform this fetcher handles
    fn platform(&self) -> SocialPlatform;

    /// Check if rate limit allows request
    async fn can_fetch(&self) -> bool;

    /// Get rate limit status
    fn rate_limit_status(&self) -> RateLimitStatus;
}

#[derive(Debug)]
pub struct ProfileMatch {
    pub username: String,
    pub url: String,
    pub name: String,
    pub profile_pic_url: Option<String>,
    pub confidence: f32,  // 0.0-1.0
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct RateLimitStatus {
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
}
```

## Implementation Strategy

### Phase-Based Approach

The project will be developed in phases, with each phase building on the previous:

1. **Foundation** (2 weeks)
   - Basic application structure
   - GUI framework setup
   - Database schema and migrations
   - Contact CRUD operations

2. **VCF Support** (1 week)
   - Import/export VCF files
   - Parse social media URLs from VCF fields
   - Handle various VCF versions

3. **Profile Picture Fetching** (3 weeks)
   - Implement ProfileFetcher trait
   - Platform-specific implementations
   - Rate limiting and caching
   - Queue system for batch operations

4. **Profile Discovery** (3 weeks)
   - Search engine integration
   - Profile matching algorithms
   - Confidence scoring
   - User confirmation workflow

5. **Polish & Distribution** (3 weeks)
   - UI/UX improvements
   - Error handling and logging
   - Cross-platform testing
   - Packaging and distribution

### Development Principles

1. **Test-Driven Development**: Write tests before implementation
2. **Incremental Development**: Small, working increments
3. **Documentation**: Document as you go
4. **Code Review**: Self-review before committing
5. **Performance**: Profile and optimize hot paths
6. **User Privacy**: Privacy-first design decisions

## Technical Challenges

### Challenge 1: Social Media API Access

**Problem**: Many social networks restrict API access or require authentication.

**Solutions**:

- Use official APIs where available (Twitter API, LinkedIn API, GitHub API)
- Implement web scraping as fallback (respect robots.txt)
- Handle rate limits gracefully with exponential backoff
- Consider paid API services (Clearbit, FullContact) for enhanced features
- Implement robust error handling for blocked requests

**Risks**:

- APIs may change without notice
- Rate limits may be too restrictive
- Terms of Service compliance
- Legal concerns around scraping

**Mitigations**:

- Modular design allows swapping implementations
- Aggressive caching to minimize requests
- User-configurable rate limits
- Clear disclaimers about ToS compliance

### Challenge 2: Profile Matching Accuracy

**Problem**: Matching contacts to social media profiles without false positives.

**Solution Strategy**:

- Multi-signal matching:
  - Name similarity (Jaro-Winkler distance)
  - Email domain matching
  - Location matching
  - Company/organization matching
  - Photo similarity (optional, advanced)
- Confidence scoring (0.0-1.0)
- Always require user confirmation for matches below 0.9
- Learn from user confirmations/rejections

**Implementation**:

```rust
pub struct ProfileMatcher {
    name_weight: f32,      // 0.5
    email_weight: f32,     // 0.3
    location_weight: f32,  // 0.1
    company_weight: f32,   // 0.1
}
```

### Challenge 3: Rate Limiting

**Problem**: Getting blocked by social media sites due to too many requests.

**Solutions**:

- Implement per-platform rate limiters (token bucket algorithm)
- Exponential backoff on failures
- Respect Retry-After headers
- Distribute requests over time
- Cache aggressively (24-hour minimum)
- Queue system for batch operations
- User-configurable delays

**Rate Limit Targets**:

- LinkedIn: ~100 requests/day (unofficial, conservative)
- Twitter: Varies by API tier
- GitHub: 60/hour unauthenticated, 5000/hour authenticated
- Instagram: Very restrictive, may require login
- Facebook: Graph API with app registration

### Challenge 4: Image Storage and Caching

**Problem**: Efficiently store and manage profile pictures.

**Solutions**:

- Store images as BLOBs in SQLite
- Implement LRU cache for in-memory images
- Compress images (WebP format)
- Set cache expiration (7-30 days)
- Allow manual refresh
- Lazy loading in UI

**Storage Estimates**:

- 10,000 contacts @ 50KB each = ~500MB
- With compression: ~200MB
- Acceptable for local storage

### Challenge 5: Cross-Platform Compatibility

**Problem**: Ensuring consistent behavior across Windows, macOS, and Linux.

**Solutions**:

- Use platform-agnostic crates
- Test on all platforms regularly
- Use `directories` crate for platform-specific paths
- Handle file path separators correctly
- Platform-specific packaging (MSI, DMG, AppImage)

**Testing Strategy**:

- GitHub Actions CI for all platforms
- Manual testing on physical hardware
- Beta testing program

## Decision Log

### Decision 1: Pure Rust GUI (Iced) vs Web-based (Tauri)

**Date**: 2024-01-XX

**Decision**: Use Iced (pure Rust GUI)

**Rationale**:

- User preference for pure Rust implementation
- Learning opportunity for Rust GUI development
- No external dependencies (webview)
- Type safety throughout the stack
- Can switch to Tauri later if needed

**Trade-offs**:

- Longer development time for UI
- Less polished UI initially
- Fewer pre-built components

### Decision 2: SQLite vs Embedded Key-Value Store

**Date**: 2024-01-XX

**Decision**: Use SQLite with SQLx

**Rationale**:

- Relational data model fits contacts well
- Excellent query capabilities
- ACID compliance
- Mature and battle-tested
- Compile-time query checking with SQLx
- Easy to export data

**Alternatives Considered**:

- sled: Fast but less mature
- RocksDB: Overkill for this use case

### Decision 3: Async Runtime (Tokio vs async-std)

**Date**: 2024-01-XX

**Decision**: Use Tokio

**Rationale**:

- Most mature async runtime
- Best ecosystem support
- Iced supports Tokio
- Most crates support Tokio
- Excellent documentation

### Decision 4: VCF Library vs Custom Parser

**Date**: 2024-01-XX

**Decision**: Use existing `vcard` crate, with option to extend

**Rationale**:

- VCF format is complex with many versions
- Don't reinvent the wheel
- Can extend if needed
- Focus on core features

### Decision 5: Web Scraping Approach

**Date**: 2024-01-XX

**Decision**: Use `scraper` for static content, optionally add headless browser for dynamic content

**Rationale**:

- Most sites can be scraped with static HTML parsing
- Headless browsers are heavy and slow
- Start simple, add complexity only if needed
- Keep `headless_chrome` as optional dependency

## Next Steps

1. ✅ Document project plan (this file)
2. ⏭️ Set up initial Cargo project structure
3. ⏭️ Implement database schema and migrations
4. ⏭️ Create basic GUI with Iced
5. ⏭️ Implement contact CRUD operations
6. ⏭️ Follow roadmap for remaining features

---

**Last Updated**: 2024-01-XX  
**Status**: Planning Phase  
**Next Review**: After MVP completion
