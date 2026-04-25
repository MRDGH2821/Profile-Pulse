# VCF Import/Export

**Status**: Implemented  
**Since**: Phase 2 (2026-01-15)  
**Source**: `src/vcf/mod.rs`, `.agents/VCF_CURRENT_STATUS.md`

## Overview

Profile Pulse supports importing and exporting contacts in vCard (VCF) format. The parser handles multiple entries per field, labels via TYPE parameters and itemN.X-ABLabel pattern, and preserves all structured contact fields.

## ADDED Requirements

### Requirement: Import vCard 3.0/4.0 format

The system SHALL parse VCF files in both vCard 3.0 and 4.0 format.

#### Scenario: Import vCard 3.0

- **WHEN** VCF file has `BEGIN:VCARD` and `VERSION:3.0`
- **THEN** all fields are parsed correctly

#### Scenario: Import vCard 4.0

- **WHEN** VCF file has `BEGIN:VCARD` and `VERSION:4.0`
- **THEN** all fields are parsed correctly

### Requirement: Extract all emails (not just first)

VCF import SHALL extract ALL email entries, not just the first one.

#### Scenario: Multiple emails extracted

- **WHEN** VCF contains:
  ```
  EMAIL;TYPE=INTERNET;TYPE=HOME:home@email.com
  EMAIL;TYPE=INTERNET;TYPE=WORK:work@email.com
  item1.EMAIL;TYPE=INTERNET:personal@email.com
  item1.X-ABLabel:Personal
  ```
- **THEN** three ContactEmail entries are created: "home@email.com" (Home), "work@email.com" (Work), "personal@email.com" (Personal)

### Requirement: Extract all phone numbers

VCF import SHALL extract ALL phone entries, not just the first one.

#### Scenario: Multiple phones extracted

- **WHEN** VCF contains 9 phone entries (as in test contact.vcf)
- **THEN** all 9 are stored as ContactPhone entries with correct labels

### Requirement: Parse structured name components

VCF import SHALL extract all N property components (prefix, first, middle, last, suffix).

#### Scenario: Structured name from N field

- **WHEN** VCF contains `N:Smith;Jane;Marie;Dr.;Jr.`
- **THEN** contact has: name_prefix="Dr.", first_name="Jane", middle_name="Marie", last_name="Smith", name_suffix="Jr."

### Requirement: Extract all URLs with labels

VCF import SHALL extract ALL URL entries with their labels.

#### Scenario: URLs with labels

- **WHEN** VCF contains:
  ```
  URL:https://github.com/user
  item1.URL:https://linkedin.com/in/user
  item1.X-ABLabel:LinkedIn
  ```
- **THEN** two ContactUrl entries are created with correct labels

### Requirement: Parse dates with labels

VCF import SHALL extract BDAY and X-ABDATE fields with labels.

#### Scenario: Birthday from BDAY

- **WHEN** VCF contains `BDAY:19900515`
- **THEN** ContactDate is created with label="Birthday", date=1990-05-15

#### Scenario: Anniversary from itemN.X-ABDATE

- **WHEN** VCF contains:
  ```
  item4000.X-ABDATE:20100620
  item4000.X-ABLabel:_$!<Anniversary>!$_
  ```
- **THEN** ContactDate is created with label="Anniversary", date=2010-06-20

### Requirement: Extract organization and department

VCF import SHALL extract both company (first ORG component) and department (second ORG component).

#### Scenario: ORG with department

- **WHEN** VCF contains `ORG:TechCorp Inc;Research & Development`
- **THEN** organization="TechCorp Inc", department="Research & Development"

### Requirement: Extract nickname and notes

VCF import SHALL extract NICKNAME and NOTE fields.

#### Scenario: Nickname extracted

- **WHEN** VCF contains `NICKNAME:Johnny`
- **THEN** contact.nickname = "Johnny"

#### Scenario: Notes with newlines

- **WHEN** VCF contains:
  ```
  NOTE:Line 1\nLine 2\nLine 3
  ```
- **THEN** notes are unescaped (newlines restored)

### Requirement: Custom labels via itemN.X-ABLabel

VCF import SHALL handle custom labels using the itemN.X-ABLabel pattern.

#### Scenario: Custom email label

- **WHEN** VCF contains:
  ```
  item1000.EMAIL;TYPE=INTERNET:custom@example.com
  item1000.X-ABLabel:Personal
  ```
- **THEN** ContactEmail has label="Personal"

#### Scenario: Custom phone label

- **WHEN** VCF contains:
  ```
  item2000.TEL:+1234567890
  item2000.X-ABLabel:Google Voice
  ```
- **THEN** ContactPhone has label="Google Voice"

### Requirement: Export to VCF format

The system SHALL export contacts to valid VCF format, preserving labels.

#### Scenario: Round-trip preservation

- **WHEN** contact is imported from VCF and re-exported
- **THEN** all fields are preserved in the exported VCF
- **AND** labels are converted back to VCF TYPE parameters or itemN.X-ABLabel format

#### Scenario: Export structured name to N field

- **WHEN** contact has name_prefix, first_name, middle_name, last_name, name_suffix
- **THEN** N field is: `N:Last;First;Middle;Prefix;Suffix`

#### Scenario: Export custom labels use itemN format

- **WHEN** contact has a custom label (not standard)
- **THEN** it's exported with itemN.X-ABLabel format

### Requirement: VCF value unescaping

VCF import SHALL handle escaped characters in values (particularly in NOTE fields).

#### Scenario: Escaped semicolons in NOTE

- **WHEN** VCF contains `NOTE:Some text\; more text`
- **THEN** the backslash-escaped semicolon is unescaped to ";"

---

## Implementation Details

### Import Functions (src/vcf/mod.rs)

| Function                    | Purpose                                  |
| --------------------------- | ---------------------------------------- |
| `parse_vcard()`             | Main entry - parses a single vCard entry |
| `extract_all_emails()`      | Extract all EMAIL fields with labels     |
| `extract_all_phones()`      | Extract all TEL fields with labels       |
| `extract_all_addresses()`   | Extract all ADR fields with labels       |
| `extract_all_dates()`       | Extract BDAY and X-ABDATE fields         |
| `extract_all_urls()`        | Extract URL fields with labels           |
| `extract_structured_name()` | Parse N property components              |
| `extract_nickname()`        | Parse NICKNAME field                     |
| `extract_notes()`           | Parse NOTE with unescaping               |
| `extract_department()`      | Parse ORG second component               |

### Export Functions

| Function              | Purpose                                      |
| --------------------- | -------------------------------------------- |
| `export_contact()`    | Main entry - exports single contact to VCF   |
| `export_field()`      | Export one field with proper TYPE parameters |
| `export_item_field()` | Export with itemN.X-ABLabel format           |

### Item Number Ranges

- 1000-1999: Email custom labels
- 2000-2999: Phone custom labels
- 3000-3999: Address custom labels
- 4000-4999: Date custom labels
- 5000-5999: URL custom labels

---

## Test Coverage

From `.agents/logs/2026-01-15.md`:

- ✅ `test_import_google_contacts_vcf()` - All 6 URLs with labels extracted
- ✅ `test_parse_url_with_label()` - itemN.X-ABLabel association works
- ✅ `test_export_urls_with_labels()` - Round-trip preserves labels
- ✅ `test_export_structured_emails()` - Email export with labels
- ✅ `test_export_structured_phones()` - Phone export with labels
- ✅ `test_export_structured_addresses()` - Address export with labels
- ✅ `test_export_structured_dates()` - Date export with labels
- ✅ `test_import_export_roundtrip()` - Full round-trip
- ✅ `test_structured_name_fields()` - N field component extraction
- ✅ `test_department_without_organization()` - X-DEPARTMENT fallback

---

## Implementation Notes

- Implemented in: `src/vcf/mod.rs` (main parser)
- Repository: `src/vcf/repository.rs` (file operations)
- Tests: 70/70 passing (100%)
- Source: `.agents/VCF_CURRENT_STATUS.md`, `.agents/logs/2026-01-15.md`

---

**Related**:

- Contact fields spec: `contact-fields/spec.md`
- Labels spec: `labels-system/spec.md`
- Database spec: `database-schema/spec.md`
