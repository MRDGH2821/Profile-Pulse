## ADDED Requirements

### Requirement: Resize fetched images

The system SHALL resize all fetched profile pictures to a maximum dimension of 512x512 pixels while maintaining aspect ratio. Images smaller than 512x512 SHALL NOT be upscaled.

#### Scenario: Large image downscaled

- **WHEN** a fetched image is 1024x1024 pixels
- **THEN** the image is resized to 512x512 pixels maintaining aspect ratio

#### Scenario: Small image not upscaled

- **WHEN** a fetched image is 128x128 pixels
- **THEN** the image is stored at 128x128 pixels without upscaling

#### Scenario: Non-square image resized proportionally

- **WHEN** a fetched image is 800x600 pixels
- **THEN** the image is resized to 512x384 pixels (max dimension 512, aspect ratio preserved)

### Requirement: Compress images to WebP format

The system SHALL convert all fetched images to WebP format at 85% quality before storage, regardless of the original format (JPEG, PNG, GIF, etc.).

#### Scenario: JPEG converted to WebP

- **WHEN** a fetched image is in JPEG format
- **THEN** the image is encoded as WebP at 85% quality and the resulting bytes are stored

#### Scenario: PNG with transparency converted to WebP

- **WHEN** a fetched image is a PNG with alpha transparency
- **THEN** the image is encoded as WebP with alpha channel preserved at 85% quality

#### Scenario: Original format discarded after processing

- **WHEN** image processing completes successfully
- **THEN** only the WebP-encoded bytes are stored; the original raw bytes are not persisted

### Requirement: Validate image data before processing

The system SHALL validate that fetched data is a valid image before attempting to process it. Invalid or corrupt images SHALL be rejected with a clear error.

#### Scenario: Valid image data

- **WHEN** the fetched data is a valid JPEG or PNG image
- **THEN** the image is decoded and processed successfully

#### Scenario: Invalid image data

- **WHEN** the fetched data is not a valid image (e.g., HTML error page, empty response)
- **THEN** the system returns an error indicating the fetched data is not a valid image

### Requirement: Store processed image in profile_cache

The system SHALL store the processed image bytes in the `profile_cache` table, linked to the source `contact_urls` entry via `url_id`. The stored record SHALL include the content type, fetch timestamp, and source URL.

#### Scenario: First fetch for a URL

- **WHEN** a profile picture is fetched for a URL that has no cached entry
- **THEN** a new row is inserted into `profile_cache` with the image BLOB, `url_id`, content type "image/webp", and current timestamp

#### Scenario: Re-fetch updates existing entry

- **WHEN** a profile picture is re-fetched for a URL that already has a cached entry
- **THEN** the existing row in `profile_cache` is updated with the new image BLOB and refreshed timestamp
