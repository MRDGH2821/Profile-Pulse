# URL-Based Social Profiles

**Status**: Implemented  
**Since**: Phase 1b (2026-01-15)  
**Source**: `.agents/URL_ARCHITECTURE_STATUS.md`

## Overview

Profile Pulse stores ALL URLs (social media, blogs, homepages) as URL fields with labels. Platform identification is done via labels (e.g., "GitHub") rather than URL parsing. This makes the system VCF-compliant and simpler.

## ADDED Requirements

### Requirement: Store all URLs with labels

All URLs associated with a contact SHALL be stored as ContactUrl entries with an optional label.

#### Scenario: Social media stored with label

- **WHEN** user adds a GitHub profile URL to a contact
- **THEN** it's stored as ContactUrl with url="https://github.com/username", label="GitHub"

#### Scenario: Non-social URL stored with label

- **WHEN** user adds a personal blog URL to a contact
- **THEN** it's stored as ContactUrl with url="https://myblog.com", label="Blog"

#### Scenario: Generic URL

- **WHEN** user adds a URL without specifying a label
- **THEN** it's stored with label=None (or "Other" as default)

### Requirement: Label-driven platform identification

Platform detection for profile fetching SHALL be based on URL labels, not URL parsing.

#### Scenario: GitHub label identifies platform

- **WHEN** a URL has label="GitHub"
- **THEN** profile fetching uses GitHub API
- **WHEN** a URL has label="LinkedIn"
- **THEN** profile fetching uses LinkedIn scraping
- **WHEN** a URL has label="Twitter"
- **THEN** profile fetching uses Twitter API

#### Scenario: Platform detection skips non-social URLs

- **WHEN** URL has label="Blog" or "HomePage" or None
- **THEN** it's skipped during profile picture fetching

### Requirement: VCF export preserves URL labels

VCF export SHALL write URLs with labels using ITEM/ITEM.X-ABLabel format for custom labels.

#### Scenario: Export GitHub URL

- **WHEN** contact has a URL with label="GitHub"
- **THEN** it's exported as:
  ```
  URL:https://github.com/username
  X-ABLabel:GitHub
  ```

#### Scenario: Export custom label

- **WHEN** contact has URL with label="My Site"
- **THEN** it's exported as:
  ```
  item5000.URL:https://mysite.com
  item5000.X-ABLabel:My Site
  ```

### Requirement: ContactUrl data structure

ContactUrl SHALL contain id, url, label, and timestamps.

```rust
pub struct ContactUrl {
    pub id: Uuid,
    pub url: String,
    pub label: Option<String>,  // "GitHub", "LinkedIn", "Blog", etc.
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Requirement: URL methods

Contact and ContactUrl SHALL provide helper methods for URL management.

#### Scenario: Add URL to contact

- **WHEN** user adds a URL to a contact
- **THEN** ContactUrl is appended to contact.urls

#### Scenario: Find URLs by label

- **WHEN** code calls contact.find_urls_by_label("GitHub")
- **THEN** returns all ContactUrl entries with label matching "GitHub"

#### Scenario: Check if contact has platform

- **WHEN** code calls contact.has_url_label("LinkedIn")
- **THEN** returns true if any URL has that label

### Requirement: Social media detection

URLs with social media platform labels SHALL be detected as social media.

#### Scenario: GitHub is social media

- **WHEN** ContactUrl has label="GitHub"
- **THEN** is_social_media() returns true
- **AND** as_social_platform() returns SocialPlatform::GitHub

#### Scenario: Blog is not social media

- **WHEN** ContactUrl has label="Blog"
- **THEN** is_social_media() returns false
- **AND** as_social_platform() returns None

### Requirement: profile_cache table for fetched images

Fetched profile pictures SHALL be stored in the profile_cache table, linked to the source ContactUrl.

#### Scene: Fetched image linked to URL

- **WHEN** a profile picture is fetched for a URL
- **THEN** it's stored in profile_cache with url_id reference
- **AND** can be retrieved via the ContactUrl id

---

## Migration Summary

The system was refactored from a separate `social_profiles` table to using `contact_urls`:

**Before**:

```
Contact.social_profiles: Vec<SocialProfile>  (separate table)
SocialProfile had: platform, username, url, profile_pic_url, ...
```

**After**:

```
Contact.urls: Vec<ContactUrl>
ContactUrl has: url, label (platform derived from label)
```

**Benefits**:

- VCF-compliant (uses standard URL fields)
- Label-driven platform detection (no URL parsing)
- Preserves ALL URLs (social + non-social)

---

## Implementation Notes

- Implemented in: `src/core/contact.rs`, `src/vcf/mod.rs`, `src/db/repository.rs`
- Migration: `20250114_002_add_urls_table.sql`
- Tests: 44/44 passing after refactoring
- Source: `.agents/URL_ARCHITECTURE_STATUS.md`

---

**Related**:

- Labels spec: `labels-system/spec.md`
- Contact fields spec: `contact-fields/spec.md`
- Database spec: `database-schema/spec.md`
- Phase 3 change: `profile-picture-fetching/`
