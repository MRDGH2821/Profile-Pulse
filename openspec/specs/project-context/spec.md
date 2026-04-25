# Project Context: Profile Pulse

## Project Overview

**Name**: Profile Pulse  
**Type**: Desktop contact management application  
**Language**: Rust  
**GUI Framework**: Iced (pure Rust)  
**Database**: SQLite with SQLx  
**License**: GPLv3-or-later  
**Version**: 0.1.0  
**Status**: Foundation phase (Phases 0-2 complete, Phase 3 in progress)

## Technology Stack

| Component        | Technology         | Notes                         |
| ---------------- | ------------------ | ----------------------------- |
| GUI Framework    | Iced               | Pure Rust, Elm architecture   |
| Database         | SQLite + SQLx      | Async queries, migrations     |
| HTTP Client      | reqwest            | Async, with retry middleware  |
| Web Scraping     | scraper            | For LinkedIn/Twitter fallback |
| Image Processing | image crate        | WebP conversion               |
| Async Runtime    | Tokio              | For all async operations      |
| Error Handling   | thiserror + anyhow | Custom error types            |
| Logging          | tracing            | Structured logging            |
| Rate Limiting    | governor           | Token bucket algorithm        |
| Caching          | Moka               | In-memory LRU cache           |

## Completed Capabilities

### Phase 0: Planning & Setup ✅

- [x] Project scope and requirements defined
- [x] Technology stack selected (Rust + Iced + SQLite)
- [x] Architecture designed (layered pattern)
- [x] Development roadmap created (5 phases)
- [x] API integration strategy documented
- [x] CI/CD pipeline configured (GitHub Actions)
- [x] License (GPLv3-or-later)
- [x] AI disclosure framework

### Phase 1: Foundation ✅ (80%)

- [x] Cargo project initialized
- [x] Database layer (SQLite migrations, CRUD)
- [x] Contact model with builder pattern
- [x] Structured fields (emails, phones, addresses, dates, URLs)
- [x] Labels system for all field types
- [x] Workspace system (VCF as source of truth)
- [x] UI forms with structured field support

### Phase 2: VCF Import/Export ✅

- [x] VCF parser (vCard 3.0/4.0)
- [x] Social media URL extraction from VCF
- [x] Import/export UI
- [x] Comprehensive test coverage

### Phase 3: Profile Fetching (in progress)

- [x] ProfileFetcher trait defined
- ⏭️ GitHub fetcher implementation
- ⏭️ LinkedIn fetcher implementation
- ⏭️ Rate limiting
- ⏭️ Caching layer
- ⏭️ Image processing
- ⏭️ UI integration

### Phase 4: Discovery (pending)

- [x] Module structure created
- [ ] Search integration
- [ ] Matching algorithms
- [ ] Confidence scoring
- [ ] Discovery UI

## Key Architecture Decisions

| Decision                  | Rationale                            |
| ------------------------- | ------------------------------------ |
| Rust + Iced               | Pure Rust, type-safe, cross-platform |
| SQLite per workspace      | Performance + VCF portability        |
| VCF as source of truth    | User data ownership, portability     |
| URL-based social profiles | VCF-compliant, label-driven          |
| Labels for field types    | Apple/Google VCF compatibility       |
| Async everything          | Non-blocking UI                      |

## Current Source Structure

```
src/
├── main.rs            # Application entry
├── core/              # Domain models
│   ├── contact.rs     # Contact, SocialProfile, ContactUrl
│   └── labels.rs      # Label enums
├── db/               # Database layer
│   ├── mod.rs        # Connection pool, migrations
│   ├── models.rs     # Row types
│   └── repository.rs # CRUD operations
├── vcf/              # VCF import/export
│   ├── mod.rs        # Parser
│   └── repository.rs # VCF file operations
├── workspace/        # Workspace management
│   └── mod.rs        # WorkspaceManager
├── ui/               # Iced GUI
│   └── mod.rs        # ContactForm, views
├── social/           # Social media (placeholder)
│   └── mod.rs        # ProfileFetcher trait
├── discovery/        # Profile discovery (placeholder)
│   └── mod.rs        # Jaro-Winkler similarity
└── utils/           # Utilities
    └── error.rs     # AppError, FetchError
```

## Database Schema

Migrations applied (in order):

1. `20250113_001_initial_schema.sql` - Core tables
2. `20250114_002_add_urls_table.sql` - URL fields
3. `20250115_001_add_structured_fields.sql` - Emails, phones, addresses, dates
4. `20250115_002_add_name_fields_and_notes.sql` - Name components, notes

## Dependencies (key crates)

```toml
iced = "0.12"
tokio = "1.35"
sqlx = "0.7"
reqwest = "0.11"
scraper = "0.18"
image = "0.24"
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
governor = "0.6"
moka = "0.12"
async-trait = "0.1"
```

## Related Documentation

- Architecture: `docs/ARCHITECTURE.md`
- Roadmap: `docs/ROADMAP.md`
- API Integration: `docs/API_INTEGRATION.md`
- Development: `docs/DEVELOPMENT.md`
- Implementation history: `.agents/logs/2026-01-15.md`

---

**Last Updated**: 2026-01-15  
**Maintained By**: OpenSpec migration
