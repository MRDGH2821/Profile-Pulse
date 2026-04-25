## 1. HTTP Client & Rate Limiting Infrastructure

- [ ] 1.1 Add new crate dependencies to `Cargo.toml`: `governor` (rate limiting), `moka` (in-memory cache with `future` feature), `reqwest` with `rustls-tls` and `cookies` features, `scraper` (HTML parsing), `image` with `webp` feature, `sha2` (cache key hashing)
- [ ] 1.2 Create `src/social/http.rs` — shared `HttpClient` wrapper with User-Agent header, 30s timeout, and exponential backoff retry logic (3 attempts, 1s/2s/4s delays for 5xx only)
- [ ] 1.3 Create `src/social/rate_limiter.rs` — per-platform rate limiter using `governor`, with configurable quotas per platform (GitHub: 50/hr unauth / 4000/hr auth, LinkedIn: 80/day)
- [ ] 1.4 Create `src/social/fetch_queue.rs` — async fetch queue using `tokio::sync::mpsc` channel with a worker task, progress reporting channel, and cancellation support
- [ ] 1.5 Write unit tests for HTTP client retry behavior, rate limiter quota enforcement, and queue ordering

## 2. URL Username Extraction

- [ ] 2.1 Create `src/social/url_parser.rs` — functions to extract usernames from known platform URL patterns: `github.com/{username}`, `linkedin.com/in/{username}`, with trailing slash and query parameter handling
- [ ] 2.2 Write unit tests for URL parsing: standard URLs, trailing slashes, query params, non-profile URLs (e.g., `github.com/orgs/...`) that should be skipped

## 3. GitHub Profile Fetcher

- [ ] 3.1 Create `src/social/github.rs` — `GitHubFetcher` struct implementing `ProfileFetcher` trait: `fetch_profile_pic` calls `GET /users/{username}`, extracts `avatar_url`, downloads image bytes
- [ ] 3.2 Implement optional authentication support — read GitHub token from app config, send as `Authorization: token {token}` header, adjust rate limit based on token presence
- [ ] 3.3 Parse `X-RateLimit-Remaining` and `X-RateLimit-Reset` response headers to update internal rate limiter state
- [ ] 3.4 Handle error responses: 404 → `ProfileNotFound`, 403 with rate limit headers → `RateLimitExceeded`, network errors → `NetworkError`
- [ ] 3.5 Write unit tests with mocked HTTP responses (using `mockito`) for all GitHub API scenarios: success, 404, 403 rate limit, auth token header

## 4. LinkedIn Profile Fetcher (Scraping)

- [ ] 4.1 Create `src/social/linkedin.rs` — `LinkedInFetcher` struct implementing `ProfileFetcher` trait: fetches public profile page HTML, extracts `og:image` meta tag or profile photo element
- [ ] 4.2 Handle LinkedIn-specific error conditions: login wall (999 status), CAPTCHA challenge, redirect to auth — return appropriate `FetchError` variants
- [ ] 4.3 Write unit tests with sample HTML fixtures for successful extraction, login wall detection, and blocked request handling

## 5. Image Processing

- [ ] 5.1 Create `src/social/image.rs` — `process_image` function that decodes raw bytes, resizes to max 512x512 (maintain aspect ratio, no upscaling), encodes to WebP at 85% quality
- [ ] 5.2 Handle transparency (alpha channel) for PNG sources when encoding to WebP
- [ ] 5.3 Validate image data before processing — reject non-image data (HTML error pages, empty responses) with clear error message
- [ ] 5.4 Write unit tests: large image downscaled, small image not upscaled, non-square proportional resize, invalid data rejection, format conversion JPEG/PNG/GIF → WebP

## 6. Two-Level Cache

- [ ] 6.1 Create `src/social/cache.rs` — `CacheService` struct wrapping a Moka in-memory cache (100MB max) and database-backed `fetch_cache` table queries
- [ ] 6.2 Implement `get()` method: check in-memory cache first, fall back to database, populate in-memory cache on DB hit
- [ ] 6.3 Implement `set()` method: store in both in-memory cache and database with TTL (7 days)
- [ ] 6.4 Implement `invalidate()` method: remove entry from both cache levels for manual refresh
- [ ] 6.5 Implement deterministic cache key: `SHA-256(platform:username)` using `sha2` crate
- [ ] 6.6 Write integration tests with in-memory SQLite: cache miss → fetch → cache hit, TTL expiration, invalidation, key determinism

## 7. Profile Cache Database Integration

- [ ] 7.1 Verify `profile_cache` table schema matches requirements (url_id, image BLOB, content_type, timestamps) — create migration if needed
- [ ] 7.2 Create `src/db/cache_repository.rs` — repository methods for storing/retrieving processed images: `save_profile_image`, `get_profile_image`, `delete_profile_image`
- [ ] 7.3 Write database tests: insert image, retrieve by url_id, update on re-fetch, cascade delete when contact URL is removed

## 8. Fetch Orchestrator

- [ ] 8.1 Create `src/social/orchestrator.rs` — `FetchOrchestrator` that ties together URL parsing, fetcher selection (by label), image processing, cache storage, and error handling into a single `fetch_for_contact()` method
- [ ] 8.2 Implement `fetch_all()` method: iterate contacts with social URLs, submit to fetch queue, collect results with progress tracking
- [ ] 8.3 Implement platform detection from URL labels: "GitHub" → GitHubFetcher, "LinkedIn" → LinkedInFetcher, other social labels → skip with warning
- [ ] 8.4 Write integration test: end-to-end fetch for a contact with a GitHub URL (mocked), verifying image is processed, cached, and stored in database

## 9. UI — Fetch Controls

- [ ] 9.1 Add "Fetch Profile Picture" button to the contact detail view in `src/ui/mod.rs` — triggers async fetch via `Command::perform`
- [ ] 9.2 Add "Fetch All Profile Pictures" button to the main toolbar — triggers batch fetch with progress tracking
- [ ] 9.3 Add progress indicator UI: display "Fetching X/Y" text during batch operations, with "Cancel" button
- [ ] 9.4 Add new `Message` variants for fetch lifecycle: `FetchProfilePic(Uuid)`, `FetchProfilePicComplete(Uuid, Result<Vec<u8>>)``, `FetchAllProfilePics`, `FetchProgress(usize, usize)`, `CancelFetch`, `RefreshProfilePic(Uuid)`
- [ ] 9.5 Handle fetch errors in UI: display error banners with platform-specific messages (not found, rate limit, network error)

## 10. UI — Profile Picture Display

- [ ] 10.1 Add profile picture thumbnail (40x40 circular) to contact list items, with placeholder icon for contacts without pictures
- [ ] 10.2 Add larger profile picture display (128x128) to contact detail view, loaded lazily from cache
- [ ] 10.3 Add "Refresh" button overlay on the detail view profile picture for manual re-fetch
- [ ] 10.4 Load profile pictures into `Handle` (Iced image handle) from database BLOBs on contact list load

## 11. Configuration & Settings

- [ ] 11.1 Add GitHub personal access token field to app configuration (`src/utils/config.rs`)
- [ ] 11.2 Create a basic settings panel in the UI for entering the GitHub token (optional, with explanation of rate limit benefits)
- [ ] 11.3 Persist GitHub token in the `settings` database table (encrypted at rest in future, plain text for MVP)

## 12. Testing & Polish

- [ ] 12.1 Write integration test: import VCF with GitHub URLs → trigger batch fetch → verify images cached and displayed
- [ ] 12.2 Test batch fetch cancellation: start batch → cancel mid-way → verify partial results preserved
- [ ] 12.3 Test cache TTL expiration: fetch image → manually expire cache entry → re-fetch → verify new image stored
- [ ] 12.4 Run `cargo clippy` and `cargo fmt` — fix all warnings
- [ ] 12.5 Run `cargo test` — ensure all existing tests still pass with no regressions
