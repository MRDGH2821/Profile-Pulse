## ADDED Requirements

### Requirement: Fetch LinkedIn profile picture via scraping

The system SHALL attempt to fetch a LinkedIn user's profile picture by requesting their public profile page (`https://www.linkedin.com/in/{username}`) and extracting the Open Graph image (`og:image` meta tag) or the profile photo element from the HTML.

#### Scenario: Public profile with og:image

- **WHEN** the scraper requests a LinkedIn profile that has a public `og:image` meta tag
- **THEN** the fetcher downloads and returns the image bytes from that URL

#### Scenario: Profile page requires login

- **WHEN** LinkedIn returns a login wall or redirect instead of the profile page
- **THEN** the fetcher returns a `ProfileNotFound` error with a message indicating the profile is not publicly accessible

#### Scenario: LinkedIn blocks the request

- **WHEN** LinkedIn returns a 999 status code or CAPTCHA challenge
- **THEN** the fetcher returns a `RateLimitExceeded` error and pauses further LinkedIn requests for 1 hour

### Requirement: Conservative LinkedIn rate limiting

The system SHALL limit LinkedIn requests to a maximum of 80 requests per day per workspace, regardless of success or failure.

#### Scenario: Daily limit enforced

- **WHEN** 80 LinkedIn requests have been made in the current 24-hour window
- **THEN** all subsequent LinkedIn fetch requests are queued until the window resets

### Requirement: Extract username from LinkedIn URL

The system SHALL extract the LinkedIn username from a stored contact URL by matching the pattern `linkedin.com/in/{username}`.

#### Scenario: Standard LinkedIn URL

- **WHEN** the URL is `https://www.linkedin.com/in/johndoe`
- **THEN** the extracted username is `johndoe`

#### Scenario: URL with query parameters

- **WHEN** the URL is `https://linkedin.com/in/johndoe/?originalSubdomain=uk`
- **THEN** the extracted username is `johndoe` (query parameters ignored)

### Requirement: Graceful scraping failure handling

The system SHALL treat scraping failures as non-critical and continue processing other contacts in a batch. Failed scrapes SHALL be logged but not block the batch.

#### Scenario: One scraping failure in batch

- **WHEN** a batch fetch encounters a scraping failure for one contact
- **THEN** the failure is logged, the error is recorded for that contact, and the batch continues with remaining contacts
