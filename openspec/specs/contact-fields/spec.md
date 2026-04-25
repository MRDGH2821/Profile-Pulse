# Contact Fields

**Status**: Implemented  
**Since**: Phase 1b (2026-01-15)  
**Source**: `src/core/contact.rs`

## Overview

Profile Pulse supports comprehensive contact fields with labels for all field types. The contact model uses explicit structured fields rather than flat key-value pairs.

## ADDED Requirements

### Requirement: Contact has structured name components

A contact SHALL have separate fields for each name component to support international name formats.

| Field        | Type           | Description                       |
| ------------ | -------------- | --------------------------------- |
| name         | String         | Computed full name (display name) |
| name_prefix  | Option<String> | e.g., "Dr.", "Mr.", "Ms."         |
| first_name   | Option<String> | Given name                        |
| middle_name  | Option<String> | Additional names                  |
| last_name    | Option<String> | Family name                       |
| name_suffix  | Option<String> | e.g., "Jr.", "Sr.", "III"         |
| nickname     | Option<String> | Nickname                          |
| organization | Option<String> | Company/organization              |
| title        | Option<String> | Job title                         |
| department   | Option<String> | Department                        |
| notes        | Option<String> | Free-form notes                   |

#### Scenario: Structured name from VCF

- **WHEN** VCF imports `N:Smith;Jane;Marie;Dr.;Jr.`
- **THEN** the contact has name_prefix="Dr.", first_name="Jane", middle_name="Marie", last_name="Smith", name_suffix="Jr."
- **AND** name is computed from these components

#### Scenario: Display name computed from components

- **WHEN** a contact has first_name="Jane" and last_name="Smith"
- **THEN** the display name shows "Jane Smith"

### Requirement: Multiple email addresses with labels

A contact SHALL support multiple email addresses, each with a label.

#### Scenario: Contact with multiple emails

- **WHEN** user adds emails to a contact
- **THEN** at least one email can have label "Home", another "Work", another custom label
- **AND** each is stored as separate ContactEmail

#### Scenario: Primary email accessible

- **WHEN** code accesses contact.email
- **THEN** it returns the first email in the emails list, or None if empty

### Requirement: Multiple phone numbers with labels

A contact SHALL support multiple phone numbers, each with a label.

#### Scenario: Contact with multiple phones

- **WHEN** user adds phone numbers to a contact
- **THEN** at least one can have label "Mobile", another "Home", another "Work"
- **AND** each is stored as separate ContactPhone

### Requirement: Multiple addresses with labels

A contact SHALL support multiple addresses, each with a label.

#### Scenario: Contact with multiple addresses

- **WHEN** user adds addresses to a contact
- **THEN** one can have label "Home", another "Work"
- **AND** each address has street, city, state, postal_code, country fields

### Requirement: Multiple significant dates with labels

A contact SHALL support multiple dates (birthday, anniversary, etc.) with labels.

#### Scenario: Contact with birthday and anniversary

- **WHEN** user adds dates to a contact
- **THEN** one can have label "Birthday", another "Anniversary"
- **AND** dates are stored as YYYYMMDD format in VCF

### Requirement: URLs and social media as URLs with labels

A contact SHALL store ALL URLs (social media, websites) as ContactUrl entries with labels.

#### Scenario: Social media stored as URL with label

- **WHEN** a contact has GitHub profile at github.com/johndoe
- **THEN** it's stored as ContactUrl with url="https://github.com/johndoe", label="GitHub"
- **AND** NOT as a separate SocialProfile (that is deprecated)

### Requirement: Custom fields for VCF extensions

A contact SHALL support arbitrary custom fields for VCF extension properties.

#### Scenario: Custom VCF field preserved

- **WHEN** VCF contains X-CUSTOMFIELD:SomeValue
- **THEN** it's stored in custom_fields HashMap
- **AND** exported back to VCF on export

### Requirement: Timestamps for audit trail

All contacts and their fields SHALL have created_at and updated_at timestamps.

#### Scenario: Field timestamps

- **WHEN** a contact is created
- **THEN** created_at is set to current time
- **AND** updated_at is set to current time
- **WHEN** a contact field is modified
- **THEN** updated_at is refreshed to current time

---

## Data Structures

### ContactEmail

```rust
pub struct ContactEmail {
    pub id: Uuid,
    pub email: String,
    pub label: String,  // "Home", "Work", "Other", or custom
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### ContactPhone

```rust
pub struct ContactPhone {
    pub id: Uuid,
    pub phone: String,
    pub label: String,  // "Mobile", "Home", "Work", etc.
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### ContactAddress

```rust
pub struct ContactAddress {
    pub id: Uuid,
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub label: String,  // "Home", "Work", "Other"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### ContactDate

```rust
pub struct ContactDate {
    pub id: Uuid,
    pub date: NaiveDate,
    pub label: String,  // "Birthday", "Anniversary", "Other"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### ContactUrl

```rust
pub struct ContactUrl {
    pub id: Uuid,
    pub url: String,
    pub label: Option<String>,  // "GitHub", "LinkedIn", "Blog", etc.
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Contact (summary)

```rust
pub struct Contact {
    pub id: Uuid,
    pub name: String,
    pub name_prefix: Option<String>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub name_suffix: Option<String>,
    pub nickname: Option<String>,
    pub notes: Option<String>,
    pub emails: Vec<ContactEmail>,
    pub phones: Vec<ContactPhone>,
    pub addresses: Vec<ContactAddress>,
    pub dates: Vec<ContactDate>,
    pub organization: Option<String>,
    pub title: Option<String>,
    pub department: Option<String>,
    pub photo_url: Option<String>,
    pub photo_blob: Option<Vec<u8>>,
    pub urls: Vec<ContactUrl>,
    pub social_profiles: Vec<SocialProfile>,  // deprecated
    pub custom_fields: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## Implementation Notes

- Implemented in: `src/core/contact.rs` (982 lines)
- Database integration: `src/db/models.rs`, `src/db/repository.rs`
- VCF support: `src/vcf/mod.rs`
- Labels: `src/core/labels.rs`
- Source: Work logs in `.agents/logs/2026-01-15.md`

---

**Related**:

- Labels spec: `labels-system/spec.md`
- URL spec: `url-based-social/spec.md`
- Database spec: `database-schema/spec.md`
- VCF spec: `vcf-import-export/spec.md`
