# Photo Fetcher

**Status**: New capability  
**Source**: `src/social/` (new module)

## ADDED Requirements

### Requirement: Fetch profile pictures from URLs

The system SHALL attempt to fetch profile pictures from social media URLs configured in a contact.

#### Scenario: Extract URLs from contact

- **WHEN** contact detail is opened for photo fetching
- **THEN** all URLs from the contact are extracted
- **AND** each URL is associated with its label (GitHub, LinkedIn, Twitter, etc.)

#### Scenario: Fetch from GitHub profile

- **WHEN** contact has a GitHub URL (github.com/username)
- **THEN** profile picture is fetched from GitHub API or profile page
- **AND** image is displayed as preview option

#### Scenario: Fetch from Twitter/X profile

- **WHEN** contact has Twitter/X URL (twitter.com/username)
- **THEN** profile picture is fetched from the profile page
- **AND** image is displayed as preview option

#### Scenario: Fetch from LinkedIn profile

- **WHEN** contact has LinkedIn URL (linkedin.com/in/username)
- **THEN** profile picture is fetched if available
- **AND** image is displayed as preview option

#### Scenario: Fetch from other URLs

- **WHEN** contact has other URL types
- **THEN** system attempts to find profile image
- **AND** displays result (may be success or failure)

### Requirement: Show fetch progress and errors

The system SHALL show progress while fetching and clear error messages on failure.

#### Scenario: Fetch in progress

- **WHEN** photos are being fetched
- **THEN** progress indicator shows which URL is being processed
- **AND** shows overall progress (e.g., "Fetching 2 of 5...")

#### Scenario: Fetch fails for URL

- **WHEN** fetch fails for a specific URL
- **THEN** error message is shown for that URL
- **AND** other fetches continue
- **AND** user can retry failed fetches

#### Scenario: No URLs configured

- **WHEN** contact has no URLs configured
- **THEN** message shows "No social media URLs configured in this contact"

---

**Related**:
- Contact Fields: `../contact-fields/spec.md`
- URL-based Social: `../url-based-social/spec.md`