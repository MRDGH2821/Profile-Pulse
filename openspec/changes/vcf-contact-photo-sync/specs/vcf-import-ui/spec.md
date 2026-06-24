# VCF Import UI

**Status**: New capability  
**Source**: `src/gui/` (new module)

## ADDED Requirements

### Requirement: Main window has Import tab

The main window SHALL have a tabbed interface with an Import tab that allows users to import VCF files.

#### Scenario: User clicks Import tab

- **WHEN** user clicks "Import" tab in main window
- **THEN** the Import view is displayed with file picker options

#### Scenario: User selects VCF file

- **WHEN** user clicks "Choose File" button
- **THEN** native file dialog opens to select .vcf files
- **AND** selected file path is displayed

#### Scenario: User starts import

- **WHEN** user clicks "Import" button with valid file selected
- **THEN** import progress is shown
- **AND** contacts are parsed from VCF
- **AND** success message shows count of imported contacts

### Requirement: Import shows error on failure

Import SHALL show clear error messages when import fails.

#### Scenario: Invalid file format

- **WHEN** user selects a non-VCF file
- **THEN** error message shows "Invalid file format. Please select a .vcf file"

#### Scenario: Empty VCF file

- **WHEN** user imports a VCF file with no contacts
- **THEN** message shows "No contacts found in file"

#### Scenario: File read error

- **WHEN** file cannot be read (permissions, not found)
- **THEN** error message shows the specific issue

---

**Related**:
- VCF Import/Export: `../vcf-import-export/spec.md`