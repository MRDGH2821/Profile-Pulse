# Contact Detail Edit

**Status**: New capability  
**Source**: `src/gui/` (new module)

## ADDED Requirements

### Requirement: Contact detail shows all fields

The contact detail view SHALL display all VCF fields for the selected contact.

#### Scenario: All name fields displayed

- **WHEN** contact is opened
- **THEN** name prefix, first name, middle name, last name, suffix are shown
- **AND** organization, title, department are shown
- **AND** nickname, notes are shown

#### Scenario: Multiple emails displayed

- **WHEN** contact has multiple emails
- **THEN** each email is shown with its label (Home, Work, etc.)

#### Scenario: Multiple phones displayed

- **WHEN** contact has multiple phone numbers
- **THEN** each phone is shown with its label

#### Scenario: Multiple addresses displayed

- **WHEN** contact has multiple addresses
- **THEN** each address is shown with street, city, state, postal code, country, and label

#### Scenario: Dates displayed

- **WHEN** contact has dates (birthday, anniversary)
- **THEN** each date is shown with its label

#### Scenario: URLs displayed

- **WHEN** contact has URLs (social profiles, websites)
- **THEN** each URL is shown with its label (GitHub, LinkedIn, etc.)

#### Scenario: Photo displayed if exists

- **WHEN** contact has a photo_blob
- **THEN** the photo is displayed in the detail view

### Requirement: User can edit contact fields

The user SHALL be able to edit contact fields when edit mode is active.

#### Scenario: Edit button activates edit mode

- **WHEN** user clicks "Edit" button
- **THEN** all editable fields become input fields
- **AND** "Save" and "Cancel" buttons appear

#### Scenario: User saves changes

- **WHEN** user clicks "Save" after editing
- **THEN** changes are persisted to the contact
- **AND** view returns to display mode

#### Scenario: User cancels edit

- **WHEN** user clicks "Cancel" during edit
- **THEN** all changes are discarded
- **AND** view returns to display mode

### Requirement: Add/remove repeatable fields

The user SHALL be able to add or remove repeatable fields (emails, phones, addresses, URLs).

#### Scenario: Add new email

- **WHEN** user clicks "Add Email" button
- **THEN** a new email field appears with empty value
- **AND** label dropdown is available

#### Scenario: Remove email

- **WHEN** user clicks remove button on an email
- **THEN** that email entry is removed

---

**Related**:
- Contact Fields: `../contact-fields/spec.md`
- VCF Import/Export: `../vcf-import-export/spec.md`