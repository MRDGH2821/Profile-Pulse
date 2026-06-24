# Photo Save

**Status**: New capability  
**Source**: `src/gui/` (new module)

## ADDED Requirements

### Requirement: User can select a photo to save

The user SHALL be able to select one of the fetched profile pictures to save to the contact.

#### Scenario: Select photo from fetched list

- **WHEN** photos have been fetched from URLs
- **THEN** each photo is displayed as a selectable thumbnail
- **AND** user can click to select one

#### Scenario: Save selected photo

- **WHEN** user clicks "Save Photo" button
- **THEN** the selected photo is saved to the contact's photo_blob field
- **AND** success message is shown

#### Scenario: Replace existing photo

- **WHEN** contact already has a photo and user saves a new one
- **THEN** the existing photo is replaced
- **AND** old photo is discarded

### Requirement: Preview photo before saving

The user SHALL be able to preview the full-size photo before deciding to save.

#### Scenario: Preview on selected photo

- **WHEN** user clicks on a photo thumbnail
- **THEN** larger preview is displayed
- **AND** user can confirm or select different photo

#### Scenario: Cancel without saving

- **WHEN** user decides not to save any photo
- **THEN** user can return to contact without changes
- **AND** original photo (if any) is preserved

### Requirement: Supported image formats

The system SHALL only accept common image formats for saving.

#### Scenario: Valid image format

- **WHEN** fetched image is JPEG or PNG
- **THEN** it can be saved to contact

#### Scenario: Invalid format rejected

- **WHEN** fetched image is not JPEG or PNG
- **THEN** error message shows "Unsupported image format"

---

**Related**:
- Contact Fields: `../contact-fields/spec.md`