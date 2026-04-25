## ADDED Requirements

### Requirement: Fetch GitHub profile picture by username

The system SHALL fetch a GitHub user's profile picture by calling `GET https://api.github.com/users/{username}` and downloading the image from the `avatar_url` field in the JSON response.

#### Scenario: Valid username returns profile picture

- **WHEN** the fetcher requests a profile picture for an existing GitHub username
- **THEN** the fetcher returns the raw image bytes from the avatar URL

#### Scenario: Non-existent username returns error

- **WHEN** the fetcher requests a profile picture for a username that does not exist on GitHub
- **THEN** the fetcher returns a `ProfileNotFound` error with the platform name and username

#### Scenario: GitHub API rate limit exceeded

- **WHEN** the fetcher receives a 403 response with rate limit headers from GitHub
- **THEN** the fetcher returns a `RateLimitExceeded` error with the retry-after duration parsed from response headers

### Requirement: Extract username from GitHub URL

The system SHALL extract the GitHub username from a stored contact URL by matching the pattern `github.com/{username}` (with or without protocol prefix and trailing slashes).

#### Scenario: Standard GitHub profile URL

- **WHEN** the URL is `https://github.com/octocat`
- **THEN** the extracted username is `octocat`

#### Scenario: URL with trailing slash

- **WHEN** the URL is `https://github.com/octocat/`
- **THEN** the extracted username is `octocat`

#### Scenario: Non-profile GitHub URL ignored

- **WHEN** the URL is `https://github.com/orgs/some-org`
- **THEN** no username is extracted and the fetch is skipped

### Requirement: Optional GitHub authentication

The system SHALL support an optional GitHub personal access token (configured via settings) to increase the rate limit from 60/hr to 5000/hr. When configured, the token SHALL be sent as a `Authorization: token {token}` header.

#### Scenario: No token configured

- **WHEN** no GitHub token is set in configuration
- **THEN** requests are sent without authentication headers and use the 50 req/hr rate limit

#### Scenario: Token configured

- **WHEN** a valid GitHub personal access token is set in configuration
- **THEN** requests include the `Authorization` header and use the 4000 req/hr rate limit

### Requirement: Respect GitHub response headers

The system SHALL read `X-RateLimit-Remaining` and `X-RateLimit-Reset` headers from GitHub API responses and update the internal rate limiter state accordingly.

#### Scenario: Rate limit remaining header updates state

- **WHEN** a GitHub API response includes `X-RateLimit-Remaining: 45`
- **THEN** the internal rate limiter is updated to reflect 45 remaining requests
