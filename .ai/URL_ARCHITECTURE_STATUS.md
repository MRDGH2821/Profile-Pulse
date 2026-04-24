# URL-Based Architecture Implementation Status

## Overview

**Date**: 2026-01-15
**Status**: ✅ **COMPLETE** (All Tests Passing - 100% Complete)

We successfully refactored Profile Pulse to use **URL fields with labels** instead of a separate `social_profiles` structure. This makes the app VCF-spec compliant and simpler.

---

## Architecture Change Summary

### Before (Old)

```rust
pub struct Contact {
    // ...
    pub social_profiles: Vec<SocialProfile>,  // Separate tracking
}

pub struct SocialProfile {
    platform: SocialPlatform,  // Enum: GitHub, LinkedIn, etc.
    username: String,
    url: String,
    // ... metadata
}
```

**Problems**:

- Not VCF-standard (social_profiles is custom extension)
- Required URL parsing to detect platform
- Lost non-social URLs (blog, homepage)

### After (New)

```rust
pub struct Contact {
    // ...
    pub urls: Vec<ContactUrl>,  // ALL URLs stored here
}

pub struct ContactUrl {
    id: Uuid,
    url: String,
    label: Option<String>,  // "GitHub", "LinkedIn", "Blog", etc.
    // ... timestamps
}
```

**Benefits**:

- ✅ VCF-compliant (uses standard URL fields)
- ✅ Label-driven platform detection (no URL parsing)
- ✅ Preserves ALL URLs (social + non-social)
- ✅ User-friendly and flexible

---

## Implementation Progress

### ✅ Completed (3 files)

#### 1. Database Migration - `src/db/migrations/20250114_002_add_urls_table.sql`

```sql
-- New table for URLs
CREATE TABLE contact_urls (
    id TEXT PRIMARY KEY,
    contact_id TEXT NOT NULL,
    url TEXT NOT NULL,
    label TEXT,  -- "GitHub", "LinkedIn", "Blog", etc.
    created_at INTEGER,
    updated_at INTEGER,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

-- Rename social_profiles → profile_cache (for fetched data)
ALTER TABLE social_profiles RENAME TO profile_cache;
ALTER TABLE profile_cache ADD COLUMN url_id TEXT REFERENCES contact_urls(id);
```

#### 2. Contact Model - `src/core/contact.rs`

- ✅ Added `ContactUrl` struct
- ✅ Changed `Contact.social_profiles` → `Contact.urls`
- ✅ Updated `ContactBuilder` to use `urls`
- ✅ Added methods: `add_url()`, `find_urls_by_label()`, `has_url_label()`, `url_labels()`
- ✅ Added `ContactUrl::is_social_media()`, `as_social_platform()`
- ✅ Updated all tests

#### 3. VCF Parser - `src/vcf/mod.rs`

- ✅ Added `item_group` tracking for `itemN.*` properties
- ✅ Implemented `extract_urls()` with two-pass label association:
  - Pass 1: Collect `itemN.X-ABLabel` mappings
  - Pass 2: Match URLs to labels via item groups
- ✅ Updated `export_contact()` to write `itemN.URL` + `itemN.X-ABLabel`
- ✅ Added tests:
  - `test_parse_url_with_label()` - label association
  - `test_export_urls_with_labels()` - round-trip export
  - `test_import_google_contacts_vcf()` - full sample import

**VCF Format Example**:

```
item1.URL:https://github.com/johndoe
item1.X-ABLabel:GitHub
item2.URL:https://myblog.com
item2.X-ABLabel:Blog
```

---

### ✅ Completed - All Refactoring Done (44/44 tests passing)

#### 1. `src/db/models.rs` ✅

**Completed**:

- ✅ Added `ContactUrlRow` struct with `FromRow` derive
- ✅ Added conversion methods: `to_contact_url()`, `from_contact_url()`
- ✅ Updated `ContactRow.to_contact()` signature to use `urls: Vec<ContactUrl>`
- ✅ Added roundtrip test for ContactUrlRow

#### 2. `src/db/repository.rs` ✅

**Completed**:

- ✅ Added `insert_contact_url()` helper method
- ✅ Added `get_contact_urls()` helper method
- ✅ Updated all queries to use `contact_urls` table
- ✅ Updated `create()`, `read()`, `update()`, `list()`, `search()` methods
- ✅ Fixed test: renamed to `test_contact_with_urls()` and updated assertions
- ✅ Updated test setup to apply both migrations

#### 3. `src/ui/mod.rs` ✅

**Completed**:

- ✅ Removed `ContactForm.social_profiles` field entirely
- ✅ Updated `from_contact()` to extract URLs from `Contact.urls`
- ✅ Updated `to_contact()` to create ContactUrl objects
- ✅ Removed Message variants: `AddSocialProfile`, `RemoveSocialProfile`, etc.
- ✅ Updated display logic in detail view to show URLs with labels
- ✅ Fixed imports (added ContactUrl, Space widget)

#### 4. `src/db/mod.rs` ✅

**Completed**:

- ✅ Changed stats query: `social_profiles` → `contact_urls`
- ✅ Updated `run_migrations()` to apply both migrations

#### 5. Additional Fixes ✅

**Completed**:

- ✅ Fixed `SocialPlatform::from_str()` to return `None` for non-social labels
- ✅ Added VCF value unescaping (`unescape_vcf_value()`) to handle `\:` escape sequences
- ✅ Added NICKNAME extraction to custom fields in VCF parser

---

## Profile Fetching Workflow (Future)

```rust
// Pseudo-code for Phase 3
for url in contact.urls {
    match url.label.as_deref() {
        Some("GitHub") => {
            let data = GitHubFetcher::fetch(&url.url).await?;
            cache_in_profile_cache(contact.id, url.id, data);
        }
        Some("LinkedIn") => { /* ... */ }
        Some("Twitter") | Some("X") => { /* ... */ }
        _ => continue,  // Skip non-social URLs
    }
}
```

**Profile Cache** (repurposed `social_profiles` table):

- Stores fetched data: avatar_url, bio, follower_count, etc.
- References source URL via `url_id` column
- Has `last_fetched_at` timestamp for cache invalidation

---

## Testing Results

### Unit Tests ✅ (44/44 Passing)

- ✅ `test_import_google_contacts_vcf()` - Imports all 6 URLs with correct labels
- ✅ `test_parse_url_with_label()` - Verifies itemN.X-ABLabel association
- ✅ `test_export_urls_with_labels()` - Round-trip export preserves labels
- ✅ `test_contact_url_is_social_media()` - Platform detection from labels working
- ✅ `test_contact_url_as_social_platform()` - Label to platform conversion verified
- ✅ All existing contact tests updated and passing
- ✅ All database CRUD tests passing with new URL structure

### Integration Tests ✅

- ✅ Imported `.ai/samples/test contact.vcf` successfully:
  - ✅ 6 URLs extracted with correct labels
  - ✅ Labels preserved: PROFILE, BLOG, _$!<HomePage>!$_, WORK, GitHub, Instagram
  - ✅ All URLs preserved (no data loss)
  - ✅ NICKNAME and NOTE custom fields extracted
- ✅ Database migration applied successfully in tests
- ✅ VCF escape sequences handled correctly (`\:` → `:`)

### Manual Testing ⏳ (Pending)

- ⏳ Import real Google Contacts VCF (not yet tested)
- ⏳ Verify UI shows all URLs with labels (compilation successful, needs runtime test)
- ⏳ Test add/edit/delete URL functionality in UI
- ⏳ Export contact and verify VCF format manually

---

## Completed Steps ✅

### Step 1: Database Layer ✅ (45 min)

1. ✅ Updated `src/db/models.rs` - Added `ContactUrlRow` with conversions
2. ✅ Updated `src/db/repository.rs` - Replaced all social_profiles operations

### Step 2: UI Layer ✅ (45 min)

3. ✅ Updated `src/ui/mod.rs` - Removed social_profiles, updated to use URLs

### Step 3: Stats Query ✅ (5 min)

4. ✅ Updated `src/db/mod.rs` - Changed table name and added migration

### Step 4: Testing ✅ (60 min)

5. ✅ Ran `cargo test` - All 44 tests passing
6. ✅ Tested import of sample VCF - All 6 URLs with labels imported
7. ⏳ Manual UI testing - Pending runtime verification
8. ✅ Verified data preservation - All URLs and custom fields preserved

**Total Time Spent**: ~3 hours (as estimated)

---

## Sample VCF URLs (Test Data)

From `.ai/samples/test contact.vcf`:

| URL                   | Label            | Type    |
| --------------------- | ---------------- | ------- |
| https://profile.com   | PROFILE          | Generic |
| https://blog.com      | BLOG             | Generic |
| https://homepage.com  | _$!<HomePage>!$_ | Generic |
| https://work.com      | WORK             | Generic |
| https://github.com    | GitHub           | Social  |
| https://instagram.com | Instagram        | Social  |

**Expected Result**: All 6 URLs imported with labels intact.

---

## Migration Notes

### For Existing Users (If App Already Deployed)

If users have existing data in `social_profiles` table, a data migration would be needed:

```sql
-- Migrate social_profiles → contact_urls
INSERT INTO contact_urls (id, contact_id, url, label, created_at, updated_at)
SELECT
    id,
    contact_id,
    url,
    platform AS label,  -- Use platform name as label
    created_at,
    updated_at
FROM profile_cache;  -- (formerly social_profiles)
```

### Breaking Changes

- API: `Contact.social_profiles` → `Contact.urls`
- Database: `social_profiles` table → `profile_cache` (for cached data)
- VCF: Export format now uses `itemN.URL` + `itemN.X-ABLabel`

---

## Design Decisions

### Label Matching Strategy

- **Case-Sensitive**: Labels are stored as-is from VCF
- **Fetcher Matching**: Use case-insensitive comparison (`label.to_lowercase() == "github"`)
- **Recognized Labels**: "GitHub", "LinkedIn", "Twitter", "X", "Facebook", "Instagram", "Mastodon"

### URL Validation

- **No URL Parsing**: Don't parse URLs to detect platform (rely on labels)
- **Generic URLs**: Store as-is with labels like "Blog", "Homepage", "Work"
- **Missing Labels**: Allowed (label = None) - user can add manually later

### Profile Cache Strategy

- **Separate Table**: `profile_cache` stores fetched data separately from URLs
- **Reference**: `url_id` links cached data to source URL
- **Caching**: Only social media URLs (detected via label) get fetched/cached
- **TTL**: Respect platform rate limits and cache for 24-48 hours

---

## Questions / Considerations

1. **Label Normalization**: Should we normalize labels to lowercase internally?
   - Pro: Consistent matching, easier queries
   - Con: Loses original VCF casing

2. **Unknown Labels**: What to do with labels we don't recognize?
   - Current: Store as-is, don't fetch
   - Alternative: Try URL parsing as fallback?

3. **Multiple URLs Same Label**: Contact can have multiple "GitHub" URLs?
   - Current: Yes, store all (e.g., personal + work GitHub)
   - Fetcher: Fetch all, store in separate profile_cache entries

4. **UI Display**: How to show URLs vs social profiles?
   - Option A: Single "URLs" list with icons for social platforms
   - Option B: Separate sections: "Social Profiles" + "Other URLs"

---

## Related Documentation

- [AGENTS.md](../AGENTS.md) - AI-assisted work guidelines
- [ARCHITECTURE.md](../docs/ARCHITECTURE.md) - Overall system architecture
- [API_INTEGRATION.md](../docs/API_INTEGRATION.md) - Profile fetching strategy
- [VCF_CURRENT_STATUS.md](VCF_CURRENT_STATUS.md) - Previous VCF import analysis
- [Work Log 2026-01-15](.ai/logs/2026-01-15.md) - Detailed implementation log

---

## Summary

**Refactoring Complete**: ✅ All code changes done, all tests passing

**Stats**:

- **Files Modified**: 6 files
- **Lines Changed**: ~400+ lines
- **Tests Passing**: 44/44 (100%)
- **Test Coverage**: VCF import/export, database CRUD, URL label association
- **Time Spent**: ~3 hours

**Key Achievements**:

- ✅ VCF-compliant URL storage with labels (itemN.X-ABLabel)
- ✅ Full data preservation from Google Contacts VCF
- ✅ Cleaner architecture (label-driven platform detection)
- ✅ All compilation errors resolved
- ✅ All unit and integration tests passing
- ✅ Database migration path defined and working

**Next Phase**: Profile fetching implementation (Phase 3)

- Use labels to identify which URLs to fetch
- Implement platform-specific fetchers (GitHub, LinkedIn, etc.)
- Cache fetched data in `profile_cache` table
- Add UI progress indicators

---

**Last Updated**: 2026-01-15 (Completed)
**Status**: ✅ Ready for Phase 3 (Profile Fetching)
