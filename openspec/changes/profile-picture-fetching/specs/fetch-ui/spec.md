## ADDED Requirements

### Requirement: Fetch profile picture for single contact

The system SHALL provide a UI button on the contact detail view that triggers a profile picture fetch for the selected contact. The button SHALL be labeled "Fetch Profile Picture".

#### Scenario: Contact has labeled social media URL

- **WHEN** the user clicks "Fetch Profile Picture" on a contact that has a URL labeled "GitHub" with value "https://github.com/octocat"
- **THEN** the system extracts the username from the URL, fetches the profile picture, and displays it in the contact detail view

#### Scenario: Contact has no social media URLs

- **WHEN** the user clicks "Fetch Profile Picture" on a contact that has no URLs with social media labels
- **THEN** the system displays a message "No social media profiles found for this contact"

#### Scenario: Contact has multiple social media URLs

- **WHEN** the user clicks "Fetch Profile Picture" on a contact that has URLs labeled "GitHub" and "LinkedIn"
- **THEN** the system fetches profile pictures from both platforms and displays the highest-quality available image

### Requirement: Batch fetch for all contacts

The system SHALL provide a "Fetch All Profile Pictures" button in the main toolbar that triggers profile picture fetching for all contacts that have social media URLs but no cached profile picture.

#### Scenario: Batch fetch with progress

- **WHEN** the user clicks "Fetch All Profile Pictures"
- **THEN** the system shows a progress indicator displaying completed/total count (e.g., "Fetching 15/47") and updates it as each fetch completes

#### Scenario: Batch fetch completes

- **WHEN** all fetchable contacts have been processed
- **THEN** the progress indicator shows completion and a summary (e.g., "42 fetched, 3 failed, 2 skipped")

#### Scenario: Batch fetch cancellation

- **WHEN** the user clicks "Cancel" during a running batch fetch
- **THEN** the system stops processing after the current fetch completes and preserves any already-fetched images

### Requirement: Profile picture display in contact list

The system SHALL display fetched profile pictures as circular thumbnails (40x40 pixels) next to the contact name in the contact list view. Contacts without profile pictures SHALL display a placeholder icon.

#### Scenario: Contact with cached profile picture

- **WHEN** the contact list view renders a contact that has a cached profile picture
- **THEN** the profile picture is displayed as a circular thumbnail to the left of the contact name

#### Scenario: Contact without profile picture

- **WHEN** the contact list view renders a contact that has no cached profile picture
- **THEN** a placeholder icon (person silhouette) is displayed to the left of the contact name

### Requirement: Profile picture display in contact detail view

The system SHALL display the fetched profile picture in the contact detail view at a larger size (128x128 pixels), with the original image loaded lazily from the cache.

#### Scenario: Viewing contact with profile picture

- **WHEN** the user opens the detail view for a contact with a cached profile picture
- **THEN** the profile picture is displayed at 128x128 pixels alongside the contact's other information

#### Scenario: Manual refresh button

- **WHEN** the user clicks "Refresh" on a displayed profile picture
- **THEN** the system re-fetches the image from the source platform and updates the displayed picture

### Requirement: Fetch error display

The system SHALL display clear error messages when profile picture fetching fails, indicating the reason (network error, rate limit, profile not found) and the affected platform.

#### Scenario: Profile not found

- **WHEN** a fetch fails because the profile does not exist
- **THEN** an error message is displayed: "Profile not found on GitHub for username 'nonexistent'"

#### Scenario: Rate limit exceeded

- **WHEN** a fetch fails because the platform rate limit has been reached
- **THEN** a warning message is displayed: "Rate limit reached for GitHub. Try again in 45 minutes."

#### Scenario: Network error

- **WHEN** a fetch fails due to a network connectivity issue
- **THEN** an error message is displayed: "Network error: Could not connect to GitHub. Check your internet connection."
