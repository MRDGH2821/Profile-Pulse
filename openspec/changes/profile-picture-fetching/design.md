## Context

Profile Pulse is a Rust desktop contact manager using Iced for GUI and SQLite for storage. Phases 0–2 are complete: the app has a workspace system, VCF import/export, structured contact fields (emails, phones, addresses, dates, URLs) with labels, and full database CRUD.

Contacts already store URLs with platform labels (e.g., `label="GitHub"`, `url="https://github.com/username"`). The `profile_cache` and `fetch_cache` tables exist in the database schema but are unused. The `ProfileFetcher` trait is defined in `src/social/traits.rs` but has no implementations.

The next roadmap phase is to make these stored URLs actionable by fetching profile pictures from social media platforms.

## Goals / Non-Goals

**Goals:**

- Implement the `ProfileFetcher` trait for GitHub (API-based, primary target) and at least LinkedIn (scraping-based, secondary)
- Provide a two-level cache (in-memory LRU + database) to avoid redundant network requests
- Add rate limiting per platform to respect API limits and avoid IP bans
- Process and store fetched images (resize to 512x512, compress) efficiently
- Expose fetch operations through the UI with progress feedback
- Support single-contact and batch fetch workflows
- Handle all error conditions gracefully (network failures, rate limits, missing profiles)

**Non-Goals:**

- Profile discovery / search (that is Phase 4 on the roadmap)
- Facebook and Instagram integration (too restrictive, per API_INTEGRATION.md assessment)
- Twitter/X API integration (requires paid API tier — can be added later)
- OAuth flows or user authentication with social platforms
- Real-time profile picture monitoring or change notifications
- Profile data enrichment beyond the profile picture (bio, location, etc.)

## Decisions

### 1. GitHub as the primary platform (API-based)

**Decision**: Implement GitHub fetcher first using the public REST API (`GET /users/{username}`).

**Rationale**: GitHub has the most reliable, well-documented public API with generous rate limits (60/hr unauthenticated, 5000/hr with a free personal access token). No scraping needed. The `avatar_url` field in the API response directly provides the profile picture URL.

**Alternatives considered**:

- LinkedIn first: No public API, requires fragile scraping — too risky as first implementation
- Twitter first: Requires paid API tier — blocks users without API keys
- All platforms simultaneously: Too much scope for first pass; GitHub validates the architecture end-to-end

### 2. Username extraction from stored URLs

**Decision**: Extract the username from stored contact URLs by matching known URL patterns per platform (e.g., `github.com/{username}`, `linkedin.com/in/{username}`).

**Rationale**: Contacts already store labeled URLs. The label identifies the platform, and the URL path contains the username. No additional user input needed.

**Alternatives considered**:

- Ask user to enter username separately: Redundant — the URL already contains it
- Parse URLs without labels: Fragile — would need to detect platform from domain

### 3. Rate limiting with governor (token bucket)

**Decision**: Use the `governor` crate for per-platform rate limiting with configurable token bucket parameters.

**Rationale**: `governor` implements the GCRA (Generic Cell Rate Algorithm) which is a variant of token bucket. It's async-compatible, well-tested, and already listed in `Cargo.toml` plan. Each platform gets its own rate limiter with conservative defaults.

**Configuration**:

- GitHub: 50 req/hr (unauth), 4000 req/hr (auth) — intentionally below actual limits for safety margin
- LinkedIn: 80 req/day — conservative estimate for scraping
- Global: 100 req/hr across all platforms as a safety net

### 4. Two-level cache architecture

**Decision**: In-memory LRU cache (Moka, 100MB max) + database-backed cache (`fetch_cache` table with TTL expiration).

**Rationale**: Hot data stays in memory for instant access. Database cache survives app restarts. TTL-based expiration ensures images refresh periodically. The existing `fetch_cache` table schema is already defined in migrations.

**Cache key strategy**: `SHA-256(platform:username)` — deterministic, collision-resistant, platform-scoped.

### 5. Image storage as BLOBs in profile_cache table

**Decision**: Store processed images as BLOBs in the existing `profile_cache` table, linked to `contact_urls` via `url_id`.

**Rationale**: Keeps images co-located with contact data in the workspace database. No external file management needed. The `profile_cache` table was designed for this purpose. Images are compressed to WebP at 85% quality, resized to max 512x512, so each image is ~20-50KB — manageable for thousands of contacts.

### 6. Async fetch queue for batch operations

**Decision**: Implement a `FetchQueue` using `tokio::sync::mpsc` channels with a worker task that processes fetch requests sequentially per platform.

**Rationale**: Batch operations (fetch for all contacts) should not block the UI. A channel-based queue allows the UI to submit requests and receive progress updates. Sequential processing per platform respects rate limits naturally. The queue can be paused/cancelled.

### 7. Iced message-based integration

**Decision**: Integrate fetch operations into the existing Iced Elm architecture via new `Message` variants and async subscription commands.

**Rationale**: Consistent with the existing codebase pattern. Iced's `Command::perform` handles async operations. Progress updates flow through messages. No architectural changes needed.

## Risks / Trade-offs

**[Scraping fragility]** → LinkedIn/Twitter scraping WILL break when sites change their HTML structure. Mitigation: GitHub is the primary platform (API-based, stable). Scraping is best-effort with clear error messages. Users can manually set profile pictures as fallback.

**[Rate limit accuracy]** → Conservative limits mean slower batch operations. Mitigation: Users can configure a GitHub personal access token for 80x higher limits. Cache aggressively so most operations are cache hits.

**[Image quality vs. size]** → WebP at 85% quality may show artifacts on some images. Mitigation: 85% is visually indistinguishable for profile pictures (typically small, simple images). Users can refresh to re-fetch if needed.

**[Database size growth]** → Storing images as BLOBs increases database size (~20-50KB per image). Mitigation: 1000 contacts = ~50MB total, well within acceptable range. Can add cleanup of old cached images if needed.

**[Platform ToS compliance]** → Scraping LinkedIn may violate their ToS. Mitigation: Use public profiles only, respect robots.txt, implement opt-in per platform, show clear disclaimers. GitHub API is ToS-compliant by design.
