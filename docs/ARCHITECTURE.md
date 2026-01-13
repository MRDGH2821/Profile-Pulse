# Architecture: Profile Pulse

## Table of Contents

- [System Overview](#system-overview)
- [Architecture Principles](#architecture-principles)
- [Component Architecture](#component-architecture)
- [Data Architecture](#data-architecture)
- [Module Design](#module-design)
- [Data Flow](#data-flow)
- [Security Architecture](#security-architecture)
- [Performance Considerations](#performance-considerations)
- [Design Patterns](#design-patterns)

## System Overview

Profile Pulse is a desktop application built using a layered architecture pattern with clear separation of concerns. The application follows the Elm architecture pattern (via Iced) for the UI layer and uses a repository pattern for data access.

```
┌─────────────────────────────────────────────────────────────┐
│                         UI Layer (Iced)                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │Contacts  │  │ Contact  │  │Discovery │  │Settings  │   │
│  │  View    │  │  Detail  │  │   View   │  │   View   │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Business Logic Layer                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Contact    │  │     VCF      │  │Social Media  │     │
│  │   Manager    │  │   Handler    │  │   Manager    │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Service Layer                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  Profile     │  │  Discovery   │  │    Cache     │     │
│  │  Fetchers    │  │   Service    │  │   Service    │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Data Access Layer                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Repository   │  │  SQLite DB   │  │  File I/O    │     │
│  │   Pattern    │  │   (SQLx)     │  │   (VCF)      │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

## Architecture Principles

### 1. Separation of Concerns

- UI logic separate from business logic
- Data access isolated in repository layer
- Each module has a single responsibility

### 2. Dependency Inversion

- High-level modules don't depend on low-level modules
- Both depend on abstractions (traits)
- Enables testing and modularity

### 3. Privacy by Design

- All data stored locally
- No telemetry or external tracking
- Encrypted sensitive data at rest
- User consent for all external requests

### 4. Fail-Safe Defaults

- Graceful degradation when services unavailable
- Conservative rate limits
- Opt-in for automatic features
- Clear error messages

### 5. Performance First

- Async operations for I/O
- Aggressive caching
- Lazy loading of images
- Efficient database queries

## Component Architecture

### UI Layer (`src/ui/`)

**Framework**: Iced (Elm Architecture)

**Components**:

```rust
// Main application state
pub struct App {
    state: AppState,
    contacts: Vec<Contact>,
    selected_contact: Option<Uuid>,
    current_view: View,
}

pub enum View {
    ContactList,
    ContactDetail(Uuid),
    Discovery,
    Settings,
}

pub enum Message {
    ContactsLoaded(Vec<Contact>),
    ContactSelected(Uuid),
    ContactUpdated(Contact),
    ProfilePicFetched(Uuid, Vec<u8>),
    ViewChanged(View),
    // ... more messages
}
```

**Key Characteristics**:

- Immutable state updates
- Message-based communication
- Predictable state management
- Type-safe UI construction

### Business Logic Layer (`src/core/`)

**Purpose**: Domain logic and business rules

**Modules**:

1. **Contact Manager** (`contact.rs`)
   - Contact validation
   - Contact merging
   - Duplicate detection
   - Contact search/filtering

2. **VCF Handler** (`vcf.rs`)
   - VCF parsing and generation
   - Field mapping
   - Version compatibility
   - Error recovery

3. **Social Media Manager** (`social_media.rs`)
   - Platform abstraction
   - URL validation
   - Profile data normalization

### Service Layer (`src/social/` & `src/discovery/`)

**Purpose**: External integrations and algorithms

**Key Services**:

1. **Profile Fetchers** - One per platform
2. **Discovery Service** - Profile search and matching
3. **Cache Service** - Request and image caching
4. **Rate Limiter** - Per-platform rate limiting

**ProfileFetcher Trait**:

```rust
#[async_trait]
pub trait ProfileFetcher: Send + Sync {
    async fn fetch_profile_pic(&self, username: &str)
        -> Result<Vec<u8>, FetchError>;

    async fn search_profile(&self, name: &str, email: Option<&str>)
        -> Result<Vec<ProfileMatch>, FetchError>;

    fn platform(&self) -> SocialPlatform;
    async fn can_fetch(&self) -> bool;
    fn rate_limit_status(&self) -> RateLimitStatus;
}
```

### Data Access Layer (`src/db/`)

**Pattern**: Repository Pattern

**Components**:

```rust
pub struct ContactRepository {
    pool: SqlitePool,
}

impl ContactRepository {
    pub async fn create(&self, contact: &Contact) -> Result<()>;
    pub async fn read(&self, id: Uuid) -> Result<Option<Contact>>;
    pub async fn update(&self, contact: &Contact) -> Result<()>;
    pub async fn delete(&self, id: Uuid) -> Result<()>;
    pub async fn list(&self, filter: ContactFilter) -> Result<Vec<Contact>>;
    pub async fn search(&self, query: &str) -> Result<Vec<Contact>>;
}

pub struct SocialProfileRepository {
    pool: SqlitePool,
}

pub struct CacheRepository {
    pool: SqlitePool,
}
```

## Data Architecture

### Database Schema

```sql
-- Core contacts table
CREATE TABLE contacts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    organization TEXT,
    title TEXT,
    photo_url TEXT,
    photo_blob BLOB,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_contacts_name ON contacts(name);
CREATE INDEX idx_contacts_email ON contacts(email);

-- Social media profiles
CREATE TABLE social_profiles (
    id TEXT PRIMARY KEY,
    contact_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    username TEXT NOT NULL,
    url TEXT NOT NULL,
    profile_pic_url TEXT,
    verified BOOLEAN DEFAULT FALSE,
    confidence_score REAL,
    discovered_at INTEGER,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE INDEX idx_social_profiles_contact ON social_profiles(contact_id);
CREATE INDEX idx_social_profiles_platform ON social_profiles(platform);

-- Custom VCF fields
CREATE TABLE custom_fields (
    contact_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (contact_id, key),
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

-- Fetch queue for batch operations
CREATE TABLE fetch_queue (
    id TEXT PRIMARY KEY,
    contact_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    username TEXT NOT NULL,
    status TEXT NOT NULL, -- pending, in_progress, success, failed, skipped
    priority INTEGER DEFAULT 0,
    retry_count INTEGER DEFAULT 0,
    last_attempt INTEGER,
    error_message TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE INDEX idx_fetch_queue_status ON fetch_queue(status);
CREATE INDEX idx_fetch_queue_priority ON fetch_queue(priority DESC, created_at ASC);

-- Cache table for HTTP responses
CREATE TABLE fetch_cache (
    key TEXT PRIMARY KEY,  -- SHA-256 hash of request
    data BLOB,
    content_type TEXT,
    cached_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    hit_count INTEGER DEFAULT 0
);

CREATE INDEX idx_fetch_cache_expires ON fetch_cache(expires_at);

-- Rate limit tracking
CREATE TABLE rate_limits (
    platform TEXT PRIMARY KEY,
    requests_made INTEGER DEFAULT 0,
    window_start INTEGER NOT NULL,
    last_request INTEGER
);

-- Application settings
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### Data Flow

#### 1. Import VCF Flow

```
User → Select VCF File
  ↓
VCF Handler → Parse File
  ↓
Contact Manager → Validate & Normalize
  ↓
Contact Repository → Check for Duplicates
  ↓
Contact Repository → Insert to Database
  ↓
UI Layer → Update Contact List
```

#### 2. Fetch Profile Picture Flow

```
User → Request Profile Pic Fetch
  ↓
Social Media Manager → Identify Platform
  ↓
Cache Service → Check Cache
  ↓ (cache miss)
Rate Limiter → Check Limit
  ↓ (allowed)
Profile Fetcher → HTTP Request
  ↓
Cache Service → Store Result
  ↓
Contact Repository → Update Contact
  ↓
UI Layer → Display Image
```

#### 3. Profile Discovery Flow

```
User → Initiate Discovery
  ↓
Discovery Service → Extract Search Terms
  ↓
Search Service → Query Search Engines/APIs
  ↓
Profile Matcher → Score Candidates
  ↓
UI Layer → Present Matches
  ↓
User → Confirm/Reject
  ↓
Contact Repository → Update if Confirmed
```

## Module Design

### Core Module: Contact Management

```rust
// src/core/contact.rs

pub struct ContactManager {
    repository: Arc<ContactRepository>,
}

impl ContactManager {
    pub async fn create_contact(&self, data: CreateContactData)
        -> Result<Contact> {
        // Validate
        self.validate_contact_data(&data)?;

        // Check for duplicates
        let duplicates = self.find_potential_duplicates(&data).await?;
        if !duplicates.is_empty() {
            return Err(Error::PotentialDuplicate(duplicates));
        }

        // Create
        let contact = Contact::new(data);
        self.repository.create(&contact).await?;

        Ok(contact)
    }

    pub async fn merge_contacts(&self, ids: Vec<Uuid>)
        -> Result<Contact> {
        // Load contacts
        let contacts = self.repository.read_many(&ids).await?;

        // Merge logic
        let merged = self.merge_logic(contacts)?;

        // Update database
        self.repository.create(&merged).await?;
        self.repository.delete_many(&ids).await?;

        Ok(merged)
    }

    fn find_potential_duplicates(&self, data: &CreateContactData)
        -> Result<Vec<Contact>> {
        // Fuzzy matching on name
        // Email matching
        // Phone matching
    }
}
```

### Service Module: Profile Fetching

```rust
// src/social/linkedin.rs

pub struct LinkedInFetcher {
    client: HttpClient,
    rate_limiter: Arc<RateLimiter<String>>,
    cache: Arc<CacheService>,
}

#[async_trait]
impl ProfileFetcher for LinkedInFetcher {
    async fn fetch_profile_pic(&self, username: &str)
        -> Result<Vec<u8>, FetchError> {

        // Check cache first
        let cache_key = format!("linkedin:pic:{}", username);
        if let Some(cached) = self.cache.get(&cache_key).await? {
            return Ok(cached);
        }

        // Check rate limit
        self.rate_limiter.until_key_ready("linkedin").await;

        // Build URL
        let url = format!("https://linkedin.com/in/{}", username);

        // Fetch with retry
        let html = self.client
            .get(&url)
            .retry_exponential(3)
            .send()
            .await?
            .text()
            .await?;

        // Parse profile picture URL
        let pic_url = self.extract_profile_pic_url(&html)?;

        // Download image
        let image_data = self.client
            .get(&pic_url)
            .send()
            .await?
            .bytes()
            .await?
            .to_vec();

        // Cache result
        self.cache.set(&cache_key, &image_data, Duration::days(7)).await?;

        Ok(image_data)
    }

    async fn search_profile(&self, name: &str, email: Option<&str>)
        -> Result<Vec<ProfileMatch>, FetchError> {
        // Implementation
    }

    fn platform(&self) -> SocialPlatform {
        SocialPlatform::LinkedIn
    }

    async fn can_fetch(&self) -> bool {
        self.rate_limiter.check_key("linkedin").is_ok()
    }

    fn rate_limit_status(&self) -> RateLimitStatus {
        // Implementation
    }
}
```

### Service Module: Profile Matching

```rust
// src/discovery/matcher.rs

pub struct ProfileMatcher {
    config: MatchConfig,
}

pub struct MatchConfig {
    pub name_weight: f32,       // 0.5
    pub email_weight: f32,      // 0.3
    pub location_weight: f32,   // 0.1
    pub company_weight: f32,    // 0.1
    pub min_confidence: f32,    // 0.6
}

impl ProfileMatcher {
    pub fn score_match(
        &self,
        contact: &Contact,
        candidate: &ProfileCandidate,
    ) -> MatchScore {
        let mut score = 0.0;
        let mut signals = Vec::new();

        // Name similarity using Jaro-Winkler
        let name_sim = self.name_similarity(&contact.name, &candidate.name);
        score += name_sim * self.config.name_weight;
        signals.push(Signal::Name(name_sim));

        // Email domain matching
        if let (Some(c_email), Some(p_email)) = (&contact.email, &candidate.email) {
            if self.email_domains_match(c_email, p_email) {
                score += self.config.email_weight;
                signals.push(Signal::EmailDomain);
            }
        }

        // Location matching
        if let (Some(c_loc), Some(p_loc)) = (&contact.location, &candidate.location) {
            if self.locations_match(c_loc, p_loc) {
                score += self.config.location_weight;
                signals.push(Signal::Location);
            }
        }

        // Company matching
        if let (Some(c_org), Some(p_org)) = (&contact.organization, &candidate.company) {
            if self.companies_match(c_org, p_org) {
                score += self.config.company_weight;
                signals.push(Signal::Company);
            }
        }

        MatchScore {
            confidence: score.min(1.0),
            signals,
            candidate: candidate.clone(),
        }
    }

    fn name_similarity(&self, name1: &str, name2: &str) -> f32 {
        // Normalize names
        let n1 = self.normalize_name(name1);
        let n2 = self.normalize_name(name2);

        // Jaro-Winkler distance
        strsim::jaro_winkler(&n1, &n2) as f32
    }

    fn normalize_name(&self, name: &str) -> String {
        // Remove titles (Dr., Mr., Mrs., etc.)
        // Remove middle names/initials
        // Lowercase
        // Remove accents
        name.to_lowercase()
            .trim()
            .to_string()
    }
}
```

## Security Architecture

### Data Security

**Encryption at Rest**:

- Profile pictures stored as encrypted BLOBs
- Sensitive fields (email, phone) encrypted
- Encryption key derived from system/user credentials
- Uses ChaCha20-Poly1305 AEAD

```rust
pub struct EncryptionService {
    key: Key,
}

impl EncryptionService {
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Implementation using chacha20poly1305
    }

    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        // Implementation
    }
}
```

**Data Validation**:

- Input sanitization for all user input
- SQL injection prevention (parameterized queries)
- Path traversal prevention
- XSS prevention (if web view used)

### Network Security

**HTTPS Only**:

- All external requests use HTTPS
- Certificate validation enabled
- TLS 1.2+ required

**Request Security**:

- User-Agent rotation to avoid fingerprinting
- No sensitive data in URLs
- Request timeout (30s default)
- Maximum response size limits

**Rate Limiting**:

- Per-platform rate limiters
- Token bucket algorithm
- Configurable limits
- Exponential backoff on failures

### Privacy Protections

**Local-Only Storage**:

- All data stored locally in SQLite
- No cloud sync by default
- No telemetry or analytics
- No crash reporting without consent

**User Consent**:

- Explicit consent for external requests
- Opt-in for profile discovery
- Clear explanation of data usage
- Easy data export/deletion

## Performance Considerations

### Async Operations

All I/O operations are asynchronous:

- Database queries (SQLx async)
- HTTP requests (Reqwest async)
- File I/O (Tokio async)
- Image processing (spawn_blocking)

### Caching Strategy

**Three-Level Cache**:

1. **In-Memory Cache** (Moka)
   - Hot data (recently viewed contacts)
   - LRU eviction
   - Size limit: 100MB

2. **Database Cache** (SQLite)
   - HTTP responses
   - Profile pictures
   - TTL-based expiration

3. **Disk Cache** (Optional)
   - Large assets
   - Compressed images

### Database Optimization

**Indexes**:

- Primary keys on all tables
- Foreign key indexes
- Search field indexes (name, email)
- Composite indexes for common queries

**Query Optimization**:

- Use EXPLAIN QUERY PLAN
- Avoid N+1 queries
- Batch inserts/updates
- Use transactions appropriately

**Connection Pooling**:

- SQLx connection pool
- Min connections: 1
- Max connections: 10
- Idle timeout: 10 minutes

### Image Optimization

**Compression**:

- Convert to WebP format
- Quality: 85%
- Progressive encoding
- Resize to 512x512 max

**Lazy Loading**:

- Load images on-demand
- Thumbnail generation
- Background loading
- Placeholder images

## Design Patterns

### 1. Repository Pattern

**Purpose**: Abstraction over data access

**Benefits**:

- Testability (mock repositories)
- Swappable storage backends
- Centralized query logic

### 2. Trait Objects

**Purpose**: Plugin architecture for fetchers

**Benefits**:

- Easy to add new platforms
- Uniform interface
- Runtime polymorphism

### 3. Builder Pattern

**Purpose**: Complex object construction

```rust
let contact = ContactBuilder::new()
    .name("John Doe")
    .email("john@example.com")
    .social_profile(linkedin)
    .build()?;
```

### 4. Strategy Pattern

**Purpose**: Interchangeable algorithms

**Used For**:

- Profile matching strategies
- Caching strategies
- Rate limiting strategies

### 5. Observer Pattern (via Messages)

**Purpose**: Event-driven updates

**In Iced**:

- UI updates via messages
- Decoupled components
- Predictable state flow

### 6. Factory Pattern

**Purpose**: Object creation

```rust
impl ProfileFetcherFactory {
    pub fn create(platform: SocialPlatform) -> Box<dyn ProfileFetcher> {
        match platform {
            SocialPlatform::LinkedIn => Box::new(LinkedInFetcher::new()),
            SocialPlatform::Twitter => Box::new(TwitterFetcher::new()),
            // ...
        }
    }
}
```

## Error Handling Strategy

### Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("VCF parse error: {0}")]
    VcfParse(String),

    #[error("Profile not found: {platform} - {username}")]
    ProfileNotFound {
        platform: String,
        username: String,
    },

    #[error("Rate limit exceeded for {platform}")]
    RateLimitExceeded {
        platform: String,
        retry_after: Duration,
    },

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

### Error Recovery

- Retry transient errors (network, rate limits)
- Graceful degradation (skip unavailable profiles)
- User-friendly error messages
- Detailed logging for debugging

---

**Last Updated**: 2024-01-XX  
**Status**: Planning Phase  
**Version**: 1.0
