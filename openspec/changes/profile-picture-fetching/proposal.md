## Why

Profile Pulse currently stores contact URLs with labels (e.g., "GitHub", "LinkedIn") but does nothing with them beyond display. Users import contacts from VCF files with social media URLs yet have no way to automatically fetch profile pictures from those platforms. Phase 3 of the roadmap — profile picture fetching — is the natural next step to make stored URLs actionable and give contacts visual identity.

## What Changes

- Add an async HTTP client infrastructure with retry logic, rate limiting (token bucket), and request queuing
- Implement per-platform profile picture fetchers (GitHub API first, then LinkedIn/Twitter via scraping)
- Add an in-memory + database cache layer for fetched images to avoid redundant network requests
- Integrate image processing (resize, compress to WebP) before storage
- Add fetch UI controls: single-contact fetch, batch fetch for all contacts, progress indicators
- Store fetched profile pictures as BLOBs in the existing `profile_cache` table, linked to `contact_urls` via `url_id`
- Display profile pictures in the contact list view and contact detail view

## Capabilities

### New Capabilities

- `fetch-infrastructure`: HTTP client setup, retry logic with exponential backoff, per-platform rate limiting (governor token bucket), and an async fetch queue for batch operations
- `profile-fetcher-github`: GitHub-specific fetcher using the public API (`/users/{username}`) to retrieve avatar URLs and download profile pictures (60 req/hr unauthenticated, 5000/hr authenticated)
- `profile-fetcher-scraping`: LinkedIn and Twitter fetchers using HTML scraping as a fallback approach, with conservative rate limits and robust error handling
- `image-processing`: Resize fetched images to 512x512 max, compress to WebP format at 85% quality, and store as BLOBs in the database
- `fetch-cache`: Two-level cache (in-memory LRU via Moka + database-backed `fetch_cache` table) with TTL-based expiration to minimize redundant requests
- `fetch-ui`: UI controls for triggering profile picture fetches (single, batch, per-platform), progress bar, status indicators, and error display

### Modified Capabilities

## Impact

- **Code**: New `src/social/` modules (http.rs, rate_limiter.rs, fetch_queue.rs, github.rs, linkedin.rs, twitter.rs, cache.rs, image.rs), updates to `src/ui/mod.rs` for fetch controls and image display, updates to `src/db/` for cache storage and retrieval
- **Dependencies**: New crates — `governor` (rate limiting), `moka` (in-memory cache), `reqwest` with `rustls-tls` (already partially used), `image` (image processing), `scraper` (HTML parsing for scraping)
- **Database**: Leverage existing `fetch_cache` and `profile_cache` tables defined in migrations; may need minor schema adjustments for image storage
- **Performance**: Async batch operations with rate limiting ensure no UI blocking; aggressive caching keeps network usage minimal
