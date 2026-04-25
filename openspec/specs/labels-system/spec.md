# Labels System

**Status**: Implemented  
**Since**: Phase 1b (2026-01-15)  
**Source**: `src/core/labels.rs`

## Overview

Profile Pulse uses a label system for categorizing contact fields (emails, phones, addresses, dates, URLs). Labels support both common predefined options and custom user-defined values. The system handles VCF format conversions including Apple's special label format.

## ADDED Requirements

### Requirement: Email labels with common options

Emails SHALL support predefined labels (Home, Work, Other) and custom labels.

#### Scenario: Common email labels displayed

- **WHEN** user adds an email to a contact
- **THEN** dropdown shows: "Home", "Work", "Other" as options
- **AND** "Custom" shows a text input for user-defined label

#### Scenario: Email label from VCF TYPE parameter

- **WHEN** VCF contains `EMAIL;TYPE=INTERNET;TYPE=HOME:home@example.com`
- **THEN** label is parsed as "Home"
- **WHEN** VCF contains `EMAIL;TYPE=INTERNET;TYPE=WORK:work@example.com`
- **THEN** label is parsed as "Work"

#### Scenario: Case-insensitive parsing

- **WHEN** VCF contains `EMAIL;TYPE=INTERNET;TYPE=home:home@example.com`
- **THEN** label is still parsed as "Home" (case-insensitive)

### Requirement: Phone labels with common options

Phones SHALL support predefined labels (Mobile, Home, Work, Main, HomeFax, WorkFax, Pager, Other) and custom labels.

#### Scenario: Common phone labels displayed

- **WHEN** user adds a phone to a contact
- **THEN** dropdown shows: "Mobile", "Home", "Work", "Main", "Home Fax", "Work Fax", "Pager", "Other" as options

#### Scenario: Phone label from VCF TYPE

- **WHEN** VCF contains `TEL;TYPE=CELL:+1234567890`
- **THEN** label is parsed as "Mobile"
- **WHEN** VCF contains `TEL;TYPE=HOME;TYPE=FAX:+1234567890`
- **THEN** label is parsed as "Home Fax"
- **WHEN** VCF contains `TEL;TYPE=googlevoice:+1234567890`
- **THEN** label is parsed as custom "Google Voice"

#### Scenario: Phone label conversion back to VCF

- **WHEN** label is "Mobile"
- **THEN** VCF TYPE is "CELL"
- **WHEN** label is "Home Fax"
- **THEN** VCF TYPE is "HOME;FAX"
- **WHEN** label is custom "Google Voice"
- **THEN** uses itemN.X-ABLabel format

### Requirement: Address labels with common options

Addresses SHALL support predefined labels (Home, Work, Other) and custom labels.

#### Scenario: Address label from VCF

- **WHEN** VCF contains `ADR;TYPE=HOME:;;123 Main St;City;State;12345;Country`
- **THEN** label is parsed as "Home"
- **WHEN** VCF contains `ADR;TYPE=WORK:;;456 Business Ave;City;State;12345;Country`
- **THEN** label is parsed as "Work"

### Requirement: Date labels with Apple format support

Dates SHALL support predefined labels (Birthday, Anniversary, Other) with special handling for Apple's format.

#### Scenario: Date label parsing

- **WHEN** VCF contains `BDAY:19900515`
- **THEN** it's parsed as "Birthday"
- **WHEN** VCF contains `item4000.X-ABDATE:20100620` with `item4000.X-ABLabel:_$!<Anniversary>!$_`
- **THEN** it's parsed as "Anniversary"

#### Scenario: Apple format conversion

- **WHEN** label is "Anniversary" and exporting to Apple-format VCF
- **THEN** uses `item4000.X-ABLabel:_$!<Anniversary>!$_`
- **WHEN** label is custom "Graduation"
- **THEN** uses `item4000.X-ABLabel:_$!<Graduation>!$_`

### Requirement: URL labels for social media

URLs SHALL support labels that identify platforms (GitHub, LinkedIn, Twitter, etc.) and general categories.

#### Scenario: URL label identifies platform

- **WHEN** a URL has label="GitHub"
- **THEN** profile fetching knows to use GitHub API
- **WHEN** a URL has label="LinkedIn"
- **THEN** profile fetching knows to use LinkedIn scraping

#### Scenario: Common URL labels

- **WHEN** dropdown shows URL label options
- **THEN** includes: "HomePage", "Work", "Blog", "Profile", "GitHub", "LinkedIn", "Twitter", "Facebook", "Instagram", "Mastodon", "Other"

#### Scenario: Social media detection

- **WHEN** URL label is "GitHub", "LinkedIn", "Twitter", "Facebook", "Instagram", or "Mastodon"
- **THEN** it is detected as a social media platform label

### Requirement: Label to VCF TYPE conversion

Labels SHALL be converted to appropriate VCF TYPE parameters on export.

#### Scenario: Email label to TYPE

- **WHEN** label is "Home"
- **THEN** VCF TYPE is "INTERNET;TYPE=HOME"
- **WHEN** label is "Work"
- **THEN** VCF TYPE is "INTERNET;TYPE=WORK"
- **WHEN** label is custom "Personal"
- **THEN** uses itemN.X-ABLabel format

#### Scenario: Custom label uses itemN format

- **WHEN** email has custom label "Personal"
- **THEN** exported as:
  ```
  item1000.EMAIL;TYPE=INTERNET:email@example.com
  item1000.X-ABLabel:Personal
  ```
- **AND** item numbers are in ranges: 1000s (email), 2000s (phone), 3000s (address), 4000s (date), 5000s (URL)

---

## Label Enums

### EmailLabel

```rust
pub enum EmailLabel {
    Home,
    Work,
    Other,
    Custom(String),  // User-defined
}
```

### PhoneLabel

```rust
pub enum PhoneLabel {
    Home,
    Work,
    Mobile,   // TYPE=CELL
    Main,
    HomeFax,  // TYPE=HOME;FAX
    WorkFax,  // TYPE=WORK;FAX
    Pager,
    Other,
    Custom(String),
}
```

### AddressLabel

```rust
pub enum AddressLabel {
    Home,
    Work,
    Other,
    Custom(String),
}
```

### DateLabel

```rust
pub enum DateLabel {
    Birthday,
    Anniversary,
    Other,
    Custom(String),
}
```

### UrlLabel

```rust
pub enum UrlLabel {
    HomePage,
    Work,
    Blog,
    Profile,
    GitHub,
    LinkedIn,
    Twitter,
    Facebook,
    Instagram,
    Mastodon,
    Other,
    Custom(String),
}
```

### SocialPlatform (for profile fetching)

```rust
pub enum SocialPlatform {
    LinkedIn,
    Twitter,
    Facebook,
    Instagram,
    GitHub,
    Mastodon,
    Other,
}
```

---

## Implementation Notes

- Implemented in: `src/core/labels.rs` (461 lines)
- Contact integration: `src/core/contact.rs` (ContactEmail, ContactPhone, etc.)
- VCF integration: `src/vcf/mod.rs` (extract*\*, export*\* functions)
- Source: Work logs in `.agents/logs/2026-01-15.md`

---

**Related**:

- Contact fields spec: `contact-fields/spec.md`
- URL-based social spec: `url-based-social/spec.md`
- VCF spec: `vcf-import-export/spec.md`
