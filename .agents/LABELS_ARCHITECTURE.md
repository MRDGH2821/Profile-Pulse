# Labels Architecture

This document describes the comprehensive label system for Profile Pulse contact fields.

## Overview

Profile Pulse implements a flexible label system for all contact fields (emails, phones, addresses, dates, URLs). The system provides:

- **Common label options** from vCard standards and popular contact managers
- **Custom text entry** for user-defined labels
- **Apple VCF compatibility** (handles `_$!<Label>!$_` format)
- **Type-safe enums** with string storage for flexibility

## Architecture

### Label Types

All label types follow the same pattern:

```rust
pub enum FieldLabel {
    CommonOption1,
    CommonOption2,
    Custom(String),
}
```

**Implemented Label Types**:

1. **EmailLabel** - `src/core/labels.rs`
   - Home, Work, Other, Custom(String)
2. **PhoneLabel** - `src/core/labels.rs`
   - Mobile, Home, Work, Main, HomeFax, WorkFax, Pager, Other, Custom(String)
3. **AddressLabel** - `src/core/labels.rs`
   - Home, Work, Other, Custom(String)
4. **DateLabel** - `src/core/labels.rs`
   - Birthday, Anniversary, Other, Custom(String)
5. **UrlLabel** - `src/core/labels.rs`
   - HomePage, Work, Blog, Profile, GitHub, LinkedIn, Twitter, Facebook, Instagram, Mastodon, Other, Custom(String)

### Common API

All label enums implement:

```rust
// Get dropdown options for UI
fn common_options() -> Vec<&'static str>

// Parse from string (case-insensitive)
fn from_str(s: &str) -> Self

// Convert to display string
fn as_str(&self) -> &str

// Get owned string value
fn to_string_value(&self) -> String

// Default trait
impl Default for Label
```

### Special Conversions

**PhoneLabel**:

- `to_vcard_type()` - Converts to vCard TYPE parameter format
  - Example: `Mobile` → `"CELL"`, `HomeFax` → `"HOME;FAX"`

**DateLabel & UrlLabel**:

- `to_apple_format()` - Converts to Apple's special label format
  - Example: `Anniversary` → `"_$!<Anniversary>!$_"`
  - Example: `HomePage` → `"_$!<HomePage>!$_"`

**UrlLabel**:

- `is_social_media()` - Returns true for social platform labels

## Data Model

### Structured Field Types

All structured fields in `src/core/contact.rs`:

```rust
pub struct ContactEmail {
    pub id: Uuid,
    pub email: String,
    pub label: String,  // Stored as string, not enum
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ContactPhone {
    pub id: Uuid,
    pub phone: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ContactAddress {
    pub id: Uuid,
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ContactDate {
    pub id: Uuid,
    pub date: NaiveDate,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ContactUrl {
    pub id: Uuid,
    pub url: String,
    pub label: Option<String>,  // Optional for backward compatibility
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Contact Model

The `Contact` struct includes collections of labeled fields:

```rust
pub struct Contact {
    // ... other fields
    pub emails: Vec<ContactEmail>,
    pub phones: Vec<ContactPhone>,
    pub addresses: Vec<ContactAddress>,
    pub dates: Vec<ContactDate>,
    pub urls: Vec<ContactUrl>,

    // Deprecated (for backward compatibility)
    pub email: Option<String>,
    pub phone: Option<String>,
}
```

### Helper Methods

```rust
impl Contact {
    // Add structured fields
    pub fn add_email(&mut self, email: ContactEmail);
    pub fn add_phone(&mut self, phone: ContactPhone);
    pub fn add_address(&mut self, address: ContactAddress);
    pub fn add_date(&mut self, date: ContactDate);

    // Get primary values (with fallback to deprecated fields)
    pub fn primary_email(&self) -> Option<&str>;
    pub fn primary_phone(&self) -> Option<&str>;
}
```

## Database Schema

### Tables (Migration: 20250115_001_add_structured_fields.sql)

**contact_emails**:

```sql
CREATE TABLE contact_emails (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    email TEXT NOT NULL,
    label TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
CREATE INDEX idx_contact_emails_contact_id ON contact_emails(contact_id);
CREATE INDEX idx_contact_emails_label ON contact_emails(label);
```

**contact_phones**:

```sql
CREATE TABLE contact_phones (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    phone TEXT NOT NULL,
    label TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
CREATE INDEX idx_contact_phones_contact_id ON contact_phones(contact_id);
CREATE INDEX idx_contact_phones_label ON contact_phones(label);
```

**contact_addresses**:

```sql
CREATE TABLE contact_addresses (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    street TEXT,
    city TEXT,
    state TEXT,
    postal_code TEXT,
    country TEXT,
    label TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
CREATE INDEX idx_contact_addresses_contact_id ON contact_addresses(contact_id);
CREATE INDEX idx_contact_addresses_label ON contact_addresses(label);
```

**contact_dates**:

```sql
CREATE TABLE contact_dates (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    date TEXT NOT NULL,  -- ISO 8601 format (YYYY-MM-DD)
    label TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
CREATE INDEX idx_contact_dates_contact_id ON contact_dates(contact_id);
CREATE INDEX idx_contact_dates_label ON contact_dates(label);
```

### Database Models (src/db/models.rs)

Row types for each structured field:

- `ContactEmailRow` - to/from conversions
- `ContactPhoneRow` - to/from conversions
- `ContactAddressRow` - to/from conversions
- `ContactDateRow` - to/from conversions (handles ISO 8601 date format)

## VCF Integration

### Import Behavior

**Email**: Parse `EMAIL;TYPE=HOME` and `itemN.X-ABLabel`

```
EMAIL;TYPE=INTERNET;TYPE=HOME:home@email.com  → Home
EMAIL;TYPE=INTERNET;TYPE=WORK:work@email.com  → Work
item1.EMAIL;TYPE=INTERNET:custom@email.com
item1.X-ABLabel:Custom                         → Custom
```

**Phone**: Parse `TEL;TYPE=CELL` and handle special cases

```
TEL;TYPE=CELL:+1234567890           → Mobile
TEL;TYPE=HOME:+9999999999           → Home
TEL;TYPE=HOME;TYPE=FAX:+111111111   → Home Fax
item2.TEL:+6905999
item2.X-ABLabel:googleVoice         → Google Voice
```

**Address**: Parse `ADR;TYPE=HOME`

```
ADR;TYPE=HOME:;;123 Main St;Springfield;IL;62701;USA  → Home
ADR;TYPE=WORK:;;456 Office Ave;Chicago;IL;60601;USA   → Work
```

**Date**: Parse `BDAY` and `X-ABDATE` with labels

```
BDAY:19900515                                  → Birthday
item8.X-ABDATE:20000101
item8.X-ABLabel:_$!<Anniversary>!$_           → Anniversary
```

**URL**: Parse `URL;TYPE=WORK` and `itemN.X-ABLabel` (already implemented)

```
URL;TYPE=WORK:https://work.com                 → Work
item6.URL:https://github.com
item6.X-ABLabel:GitHub                         → GitHub
item3.URL:https://profile.com
item3.X-ABLabel:PROFILE                        → Profile
```

### Export Behavior

Export structured fields back to vCard format:

**With TYPE parameters** (for standard labels):

```
EMAIL;TYPE=INTERNET;TYPE=HOME:home@email.com
TEL;TYPE=CELL:+1234567890
ADR;TYPE=HOME:;;123 Main St;Springfield;IL;62701;USA
```

**With itemN.X-ABLabel** (for custom/special labels):

```
item1.EMAIL;TYPE=INTERNET:custom@email.com
item1.X-ABLabel:Custom

item2.TEL:+6905999
item2.X-ABLabel:Google Voice

item3.URL:https://github.com
item3.X-ABLabel:GitHub
```

**Apple special format** (for Anniversary, HomePage):

```
item4.X-ABDATE:20000101
item4.X-ABLabel:_$!<Anniversary>!$_

item5.URL:https://homepage.com
item5.X-ABLabel:_$!<HomePage>!$_
```

## UI Implementation (TODO)

### Dropdown + Custom Text Entry

Each labeled field should have:

1. **Dropdown menu** with common options
2. **"Custom..." option** that reveals text input
3. **Current label** displayed or editable

Example mockup:

```
┌─────────────────────────────────────────────┐
│ Email 1:                                    │
│ ┌────────────────────────┐ ┌──────────────┐│
│ │ john@example.com       │ │ [Home ▼]     ││
│ └────────────────────────┘ └──────────────┘│
│                                             │
│ Email 2:                                    │
│ ┌────────────────────────┐ ┌──────────────┐│
│ │ work@company.com       │ │ [Work ▼]     ││
│ └────────────────────────┘ └──────────────┘│
│                                             │
│ Email 3:                                    │
│ ┌────────────────────────┐ ┌──────────────┐│
│ │ custom@email.com       │ │ [Custom...▼] ││
│ └────────────────────────┘ └──────────────┘│
│ ┌────────────────────────┐                 │
│ │ Personal               │ (custom label)  │
│ └────────────────────────┘                 │
└─────────────────────────────────────────────┘
```

### Implementation Notes

**Iced widget approach**:

```rust
// For each field
Row::new()
    .push(TextInput::new("Email", &email.email)
        .on_input(|s| Message::EmailChanged(index, s)))
    .push(PickList::new(
        EmailLabel::common_options(),
        Some(&email.label),
        |label| Message::EmailLabelChanged(index, label)
    ))
    .push(Button::new("−")
        .on_press(Message::RemoveEmail(index)))
```

**Custom label handling**:

- If user selects "Custom..." from dropdown, show additional TextInput
- Store the actual custom text in the label field
- On form load, check if label matches common options; if not, show as custom

## Label Matching Strategy

### Case Sensitivity

**Storage**: Labels are stored case-sensitively (preserves user input)
**Matching**: Label matching should be case-insensitive

Example:

```rust
// When fetching profiles by label
contact.urls.iter()
    .filter(|url| url.label.as_deref()
        .map(|l| l.eq_ignore_ascii_case("github"))
        .unwrap_or(false))
```

### Normalization (Optional)

Consider normalizing labels on save:

```rust
// Normalize common labels to standard casing
fn normalize_email_label(label: &str) -> String {
    EmailLabel::from_str(label).to_string_value()
}
```

This ensures:

- "home" → "Home"
- "WORK" → "Work"
- "custom" → "custom" (preserved as-is)

## Migration Strategy

### Existing Data

The migration script automatically migrates:

- Existing `contacts.email` → `contact_emails` with label "Home"
- Existing `contacts.phone` → `contact_phones` with label "Mobile"

### Backward Compatibility

The deprecated `email` and `phone` fields remain in the schema for now:

- `Contact.primary_email()` returns first email or deprecated field
- `Contact.primary_phone()` returns first phone or deprecated field
- VCF import populates both old and new fields temporarily

Future versions can:

1. Mark fields as deprecated in UI
2. Prompt users to migrate
3. Remove deprecated fields in major version bump

## Testing

### Test Coverage

**Unit Tests** (`src/core/labels.rs`):

- Label parsing (case-insensitive)
- Common options verification
- Apple format conversion
- VCF TYPE parameter conversion
- Social media detection

**Integration Tests** (TODO):

- VCF import with all label types
- VCF export preserving labels
- Round-trip import/export verification
- Database CRUD operations
- UI label selection

### Test Data

Use `.ai/samples/test contact.vcf` which contains:

- 3 emails (Home, Work, Custom)
- 9 phones (Home, Work, Cell, Main, Home Fax, Work Fax, Google Voice, Pager, unlabeled)
- 6 URLs (Profile, Blog, HomePage, Work, GitHub, Instagram)
- 2 dates (Birthday, Anniversary)
- 2 addresses (not in current sample, but standard)

## Future Enhancements

### Label Suggestions

Learn from user's custom labels:

```sql
SELECT DISTINCT label, COUNT(*) as usage
FROM contact_emails
WHERE label NOT IN ('Home', 'Work', 'Other')
GROUP BY label
ORDER BY usage DESC
LIMIT 5;
```

Show frequently-used custom labels as suggestions.

### Label Icons

Associate icons with common labels:

- Email: 🏠 Home, 💼 Work, 📧 Other
- Phone: 📱 Mobile, 🏠 Home, 💼 Work, 📠 Fax
- Address: 🏠 Home, 💼 Work, 📍 Other
- Date: 🎂 Birthday, 💍 Anniversary
- URL: 🐙 GitHub, 💼 LinkedIn, 🐦 Twitter

### Label Localization

Support translated labels in UI:

```rust
fn localize_label(label: &str, locale: &str) -> String {
    match (label, locale) {
        ("Home", "es") => "Casa",
        ("Work", "es") => "Trabajo",
        ("Mobile", "fr") => "Mobile",
        _ => label.to_string(),
    }
}
```

Store labels in English internally, translate for display.

## References

- **vCard 3.0 Spec**: RFC 2426
- **vCard 4.0 Spec**: RFC 6350
- **Apple AddressBook Format**: Proprietary (_$!<Label>!$_ format)
- **Google Contacts Export**: Standard TYPE parameters
- **Test Sample**: `.ai/samples/test contact.vcf`

---

**Status**: Core implementation complete, VCF parser and UI integration pending

**Last Updated**: 2026-01-15
