# Database Schema

**Status**: Implemented  
**Since**: Phase 1 (2026-01)  
**Source**: `src/db/migrations/*.sql`

## Overview

Profile Pulse uses SQLite with schema migrations. The database lives in each workspace (as `contacts.db`) and is treated as a regenerable performance cache backed by VCF files.

## ADDED Requirements

### Requirement: Database per workspace

Each workspace SHALL have its own SQLite database file located in the workspace folder.

#### Scenario: Workspace has database

- **WHEN** workspace "Personal" is created
- **THEN** workspace folder contains `contacts.db`
- **AND** SQLite with complete schema

#### Scenario: Database regeneratable from VCF

- **WHEN** `contacts.db` is corrupted or deleted
- **THEN** application can rebuild database by re-importing `contacts.vcf`

### Requirement: Contacts table with core fields

The contacts table SHALL store primary contact information.

```sql
CREATE TABLE contacts (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    email TEXT,           -- Legacy, use contact_emails
    phone TEXT,           -- Legacy, use contact_phones
    organization TEXT,
    title TEXT,
    photo_url TEXT,
    photo_blob BLOB,      -- Cached profile picture
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### Requirement: Contact Emails table (structured fields)

Each contact SHALL support multiple emails with labels via contact_emails table.

```sql
CREATE TABLE contact_emails (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    email TEXT NOT NULL,
    label TEXT NOT NULL,   -- "Home", "Work", "Other", custom
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE INDEX idx_contact_emails_contact_id ON contact_emails(contact_id);
CREATE INDEX idx_contact_emails_label ON contact_emails(label);
```

### Requirement: Contact Phones table (structured fields)

Each contact SHALL support multiple phones with labels.

```sql
CREATE TABLE contact_phones (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    phone TEXT NOT NULL,
    label TEXT NOT NULL,   -- "Mobile", "Home", "Work", etc.
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
```

### Requirement: Contact Addresses table (structured fields)

Each contact SHALL support multiple addresses with labels.

```sql
CREATE TABLE contact_addresses (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    street TEXT,
    city TEXT,
    state TEXT,
    postal_code TEXT,
    country TEXT,
    label TEXT NOT NULL,   -- "Home", "Work", "Other"
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
```

### Requirement: Contact Dates table (structured fields)

Each contact SHALL support multiple significant dates with labels.

```sql
CREATE TABLE contact_dates (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    date TEXT NOT NULL,     -- ISO 8601 date (YYYY-MM-DD)
    label TEXT NOT NULL,  -- "Birthday", "Anniversary", "Other"
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
```

### Requirement: Contact URLs table

Each contact SHALL support multiple URLs with labels.

```sql
CREATE TABLE contact_urls (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    url TEXT NOT NULL,
    label TEXT,          -- "GitHub", "LinkedIn", "Blog", etc.
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
```

### Requirement: Custom fields table

The system SHALL support arbitrary VCF extension fields.

```sql
CREATE TABLE custom_fields (
    contact_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (contact_id, key),
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
```

### Requirement: Profile cache table

Fetched profile pictures SHALL be cached for performance.

```sql
-- Renamed from social_profiles during migration 002
CREATE TABLE profile_cache (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    username TEXT NOT NULL,
    url TEXT NOT NULL,
    profile_pic_url TEXT,
    verified BOOLEAN DEFAULT 0,
    confidence_score REAL,
    discovered_at INTEGER,
    url_id TEXT REFERENCES contact_urls(id),  -- Link to source URL
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
```

### Requirement: Fetch queue table

The system SHALL support queued batch profile fetching operations.

```sql
CREATE TABLE fetch_queue (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    username TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending', 'in_progress', 'success', 'failed', 'skipped')),
    priority INTEGER DEFAULT 0,
    retry_count INTEGER DEFAULT 0,
    last_attempt INTEGER,
    error_message TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
```

### Requirement: Fetch cache table (HTTP responses)

HTTP responses and data SHALL be cached with TTL expiration.

```sql
CREATE TABLE fetch_cache (
    key TEXT PRIMARY KEY NOT NULL,
    data BLOB,
    content_type TEXT,
    cached_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    hit_count INTEGER DEFAULT 0,
    last_accessed INTEGER NOT NULL
);
```

### Requirement: Rate limits table

The system SHALL track rate limit usage per platform.

```sql
CREATE TABLE rate_limits (
    platform TEXT PRIMARY KEY NOT NULL,
    requests_made INTEGER DEFAULT 0,
    window_start INTEGER NOT NULL,
    last_request INTEGER,
    daily_quota INTEGER NOT NULL,
    hourly_quota INTEGER NOT NULL
);
```

### Requirement: Settings table

Application configuration SHALL be stored in the database.

```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
```

---

## Migration History

| Migration                          | Version | Description                                                                                                           |
| ---------------------------------- | ------- | --------------------------------------------------------------------------------------------------------------------- |
| 20250113_001_initial               | 1       | Core tables (contacts, social_profiles→profile_cache, custom_fields, fetch_queue, fetch_cache, rate_limits, settings) |
| 20250114_002_add_urls              | 2       | contact_urls table, renamed social_profiles→profile_cache                                                             |
| 20250115_001_add_structured_fields | 3       | contact_emails, contact_phones, contact_addresses, contact_dates with data migration                                  |
| 20250115_002_add_name_fields       | 4       | (name components, notes via custom_fields)                                                                            |

---

## Implementation Notes

- Migration runner: `src/db/mod.rs` (run_migrations function)
- Models: `src/db/models.rs` (row types)
- Repository: `src/db/repository.rs` (CRUD operations)
- Tests: Unit tests for each migration
- Source: `.agents/STATUS.md`, work logs

---

**Related**:

- Workspace spec: `workspace-system/spec.md`
- Contact fields spec: `contact-fields/spec.md`
- Labels spec: `labels-system/spec.md`
