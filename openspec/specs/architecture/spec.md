# Architecture

**Status**: Implemented  
**Since**: Phase 0 (documented) / Phase 1 (implemented)  
**Source**: `docs/ARCHITECTURE.md`

## Overview

Profile Pulse is a desktop contact management application built using a layered architecture pattern with clear separation of concerns. It follows the Elm architecture (via Iced) for the UI layer and uses the Repository pattern for data access.

## ADDED Requirements

### Requirement: Layered architecture

The application SHALL use a layered architecture with clear separation.

```
┌─────────────────────────────────────────────────┐
│ UI Layer (Iced)                                │
│   - Contacts View, Contact Detail, Discovery   │
└─────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│ Business Logic Layer (src/core, src/vcf)         │
│   - Contact validation, VCF handling            │
└─────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│ Data Access Layer (src/db, src/workspace)         │
│   - Repository Pattern, SQLite, File I/O        │
└─────────────────────────────────────────────────┘
```

### Requirement: Elm-style UI state management

The UI SHALL use Iced's Elm architecture pattern (Model-View-Message-Update).

#### Scenario: Iced message handling

- **WHEN** user performs an action (e.g., clicks button)
- **THEN** a Message is sent to the Update function
- **AND** Update returns a new Model + Commands
- **AND** View re-renders based on new Model

### Requirement: Repository pattern for data access

Data access SHALL be abstracted through repository interfaces.

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
```

### Requirement: Builder pattern for complex objects

Contact creation SHALL use a builder pattern.

```rust
let contact = ContactBuilder::new()
    .name("John Doe")
    .email("john@example.com")
    .build()?;
```

### Requirement: Trait objects for platform fetchers

Social media platform fetchers SHALL use a trait for pluggable implementations.

```rust
#[async_trait]
pub trait ProfileFetcher: Send + Sync {
    async fn fetch_profile_pic(&self, username: &str) -> Result<Vec<u8>, FetchError>;
    fn platform(&self) -> SocialPlatform;
    async fn can_fetch(&self) -> bool;
}
```

### Requirement: Async operations for I/O

All database and network operations SHALL be async to prevent UI blocking.

#### Scenario: Async repository

- **WHEN** repository CRUD operation is called
- **THEN** it returns a Future that resolves to Result
- **AND** UI uses Command::perform to execute async

### Requirement: Error handling with thiserror

Application errors SHALL use thiserror for type-safe error types.

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("VCF parse error: {0}")]
    VcfParse(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

---

## Design Patterns Summary

| Pattern             | Usage                                   |
| ------------------- | --------------------------------------- |
| Repository          | Data access abstraction                 |
| Builder             | Complex object construction             |
| Trait Objects       | Pluggable platform fetchers             |
| Observer (Messages) | Iced UI updates                         |
| Factory             | ProfileFetcher factory                  |
| Strategy            | Caching strategies, matching algorithms |

---

## Module Structure

```
src/
├── main.rs           # Entry point
├── core/             # Domain layer
│   ├── contact.rs    # Contact, ContactUrl, SocialProfile
│   └── labels.rs    # Label enums
├── db/               # Data access
│   ├── mod.rs       # Connection, migrations
│   ├── models.rs    # Row types
│   └── repository.rs # CRUD
├── vcf/             # VCF layer
│   ├── mod.rs       # Parser
│   └── repository.rs # File I/O
├── workspace/        # Workspace management
├── ui/               # Iced UI
├── social/           # Profile fetchers (placeholder)
├── discovery/        # Profile discovery (placeholder)
└── utils/           # Utilities
```

---

## Implementation Notes

- Architecture documented in: `docs/ARCHITECTURE.md` (829 lines)
- Most patterns implemented in source code
- UI: Iced with message-based updates
- Data: Repository pattern with SQLx async
- Source: `docs/ARCHITECTURE.md`, `docs/PLAN.md`

---

**Related**:

- Project context spec: `project-context/spec.md`
- Database schema spec: `database-schema/spec.md`
- VCF spec: `vcf-import-export/spec.md`
