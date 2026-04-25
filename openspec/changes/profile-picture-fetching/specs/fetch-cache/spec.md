## ADDED Requirements

### Requirement: In-memory LRU cache

The system SHALL maintain an in-memory LRU cache using Moka with a maximum size of 100MB. Profile picture data retrieved from the database SHALL be loaded into this cache for fast subsequent access.

#### Scenario: Cache hit on repeated access

- **WHEN** a profile picture is accessed that was recently fetched or loaded
- **THEN** the image data is returned from the in-memory cache without database or network access

#### Scenario: Cache eviction when full

- **WHEN** the in-memory cache exceeds 100MB
- **THEN** the least recently used entries are evicted to make room for new entries

#### Scenario: Cache populated on database load

- **WHEN** a profile picture is loaded from the database cache table
- **THEN** the image data is also stored in the in-memory cache for future fast access

### Requirement: Database-backed cache with TTL

The system SHALL use the existing `fetch_cache` table to persist cached HTTP responses and profile pictures. Each cache entry SHALL have a TTL (time-to-live) of 7 days after which it is considered expired.

#### Scenario: Cache hit within TTL

- **WHEN** a profile picture is requested that was fetched less than 7 days ago
- **THEN** the cached image is returned from the database without making a network request

#### Scenario: Cache miss (expired)

- **WHEN** a profile picture is requested that was fetched more than 7 days ago
- **THEN** the cache entry is considered expired, a fresh fetch is performed, and the cache is updated

#### Scenario: Cache miss (never fetched)

- **WHEN** a profile picture is requested for a contact that has never been fetched
- **THEN** a new fetch is performed and the result is stored in the cache

### Requirement: Deterministic cache keys

The system SHALL use deterministic cache keys derived from `SHA-256(platform:username)` to ensure consistent cache lookups across sessions.

#### Scenario: Same key for same platform+username

- **WHEN** the cache is queried for GitHub user "octocat"
- **THEN** the key is `SHA-256("github:octocat")` and returns the same result across sessions

#### Scenario: Different keys for different platforms

- **WHEN** the cache is queried for GitHub user "johndoe" and LinkedIn user "johndoe"
- **THEN** different cache keys are generated (different platform prefixes) and they return independent results

### Requirement: Cache invalidation on manual refresh

The system SHALL allow users to manually refresh a profile picture, which bypasses the cache and fetches a fresh copy from the platform.

#### Scenario: User triggers refresh

- **WHEN** the user clicks "Refresh" on a contact's profile picture
- **THEN** the existing cache entry is invalidated, a fresh fetch is performed, and the cache is updated with the new image
