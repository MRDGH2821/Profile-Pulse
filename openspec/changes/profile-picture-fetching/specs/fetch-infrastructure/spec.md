## ADDED Requirements

### Requirement: HTTP client with retry logic

The system SHALL provide a shared HTTP client configured with a descriptive User-Agent string, connection timeouts (30s), and automatic retry with exponential backoff (up to 3 attempts) for transient failures (5xx status codes, connection errors).

#### Scenario: Successful request on first attempt

- **WHEN** the HTTP client sends a GET request to a valid URL
- **THEN** the response body is returned immediately without retry

#### Scenario: Retry on server error

- **WHEN** the HTTP client receives a 5xx response
- **THEN** the client retries up to 3 times with exponential backoff (1s, 2s, 4s) and returns the successful response or the final error

#### Scenario: No retry on client error

- **WHEN** the HTTP client receives a 4xx response (e.g., 404 Not Found)
- **THEN** the error is returned immediately without retry

### Requirement: Per-platform rate limiting

The system SHALL enforce configurable rate limits per social media platform using a token bucket algorithm. Each platform SHALL have independent rate limits that do not affect other platforms.

#### Scenario: Request within rate limit

- **WHEN** a fetch request is made for a platform that has remaining quota
- **THEN** the request proceeds immediately

#### Scenario: Rate limit exceeded

- **WHEN** a fetch request is made for a platform that has exhausted its quota
- **THEN** the request is delayed until the quota replenishes, or queued for later processing

#### Scenario: GitHub rate limit defaults

- **WHEN** no GitHub authentication token is configured
- **THEN** the rate limit is set to 50 requests per hour

#### Scenario: GitHub authenticated rate limit

- **WHEN** a GitHub personal access token is configured
- **THEN** the rate limit is set to 4000 requests per hour

### Requirement: Async fetch queue

The system SHALL provide an async queue that accepts fetch requests and processes them in order, respecting per-platform rate limits. The queue SHALL report progress (completed/total) through a channel.

#### Scenario: Submit batch fetch request

- **WHEN** the user requests profile pictures for multiple contacts
- **THEN** the requests are queued and processed sequentially per platform, with progress updates emitted after each completion

#### Scenario: Cancel in-progress batch

- **WHEN** the user cancels a running batch fetch
- **THEN** the queue stops processing after the current request completes and discards remaining pending requests

### Requirement: ProfileFetcher trait implementation contract

All platform fetchers SHALL implement the `ProfileFetcher` trait defined in `src/social/traits.rs`, providing `fetch_profile_pic`, `search_profile`, `platform`, `can_fetch`, and `rate_limit_status` methods.

#### Scenario: Fetcher reports correct platform

- **WHEN** `GitHubFetcher::platform()` is called
- **THEN** it returns `SocialPlatform::GitHub`

#### Scenario: Fetcher reports rate limit status

- **WHEN** `fetcher.rate_limit_status()` is called
- **THEN** it returns the current remaining quota and reset time for that platform
