# Contact List View

**Status**: New capability  
**Source**: `src/gui/` (new module)

## ADDED Requirements

### Requirement: Display contacts from imported VCF

The contact list SHALL display all contacts parsed from the imported VCF file.

#### Scenario: Contacts displayed after import

- **WHEN** VCF file is imported successfully
- **THEN** a list of contacts is displayed
- **AND** each contact shows name (or "No Name" if missing)
- **AND** scrollable if many contacts

#### Scenario: Empty list shown when no contacts

- **WHEN** no contacts have been imported
- **THEN** placeholder message shows "No contacts. Import a VCF file to get started."

### Requirement: User can select a contact

The user SHALL be able to select a contact from the list to view details.

#### Scenario: Contact selected

- **WHEN** user clicks on a contact in the list
- **THEN** a contact detail view opens
- **AND** all fields for that contact are displayed

#### Scenario: Single click selection

- **WHEN** user clicks once on a contact row
- **THEN** that contact is highlighted
- **AND** double-click or button opens detail view

---

**Related**:
- Contact Fields: `../contact-fields/spec.md`