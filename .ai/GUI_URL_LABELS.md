# GUI URL Label Support

**Date**: 2026-01-15
**Feature**: URL Label Input in Contact Forms

---

## Overview

The GUI has been enhanced to support adding and editing labels for URLs. Users can now specify what each URL represents (e.g., "GitHub", "LinkedIn", "Blog", "Homepage") directly in the contact form.

---

## User Interface Changes

### Before (URLs only)
```
URLs Section:
[https://github.com/user                                    ] [−]
[https://myblog.com                                         ] [−]
```

### After (URLs + Labels)
```
URLs Section:
[https://github.com/user      ] [GitHub                    ] [−]
[https://myblog.com           ] [Blog                      ] [−]
[https://linkedin.com/in/user ] [LinkedIn                  ] [−]
```

---

## Form Layout

Each URL entry now consists of three elements in a row:

1. **URL Input Field** (wider, Fill)
   - Placeholder: "URL 1", "URL 2", etc.
   - Full URL entry (https://...)
   
2. **Label Input Field** (2x width, FillPortion(2))
   - Placeholder: "Label (e.g., GitHub, Blog)"
   - Short descriptive label for the URL
   
3. **Remove Button** (−)
   - Appears only when > 1 URL exists
   - Removes the current URL entry

### Add URL Button
- Located at the top of URLs section
- Adds a new empty URL+label pair

---

## Technical Implementation

### Data Structure

**ContactForm** now uses `Vec<UrlForm>` instead of `Vec<String>`:

```rust
pub struct UrlForm {
    pub url: String,
    pub label: String,
}

pub struct ContactForm {
    // ...
    pub urls: Vec<UrlForm>,  // Changed from Vec<String>
    // ...
}
```

### Message Handlers

Two separate message handlers for independent editing:

```rust
pub enum Message {
    // ...
    UrlChanged(usize, String),       // Edit URL at index
    UrlLabelChanged(usize, String),  // Edit label at index
    AddUrl,                          // Add new URL+label pair
    RemoveUrl(usize),                // Remove URL at index
    // ...
}
```

### Data Flow

**Form → Contact (Save)**:
```rust
ContactForm.urls: Vec<UrlForm>
    ↓ to_contact()
Contact.urls: Vec<ContactUrl>  // with label: Option<String>
    ↓ repository.create()
Database: contact_urls table (id, contact_id, url, label)
```

**Contact → Form (Load)**:
```rust
Database: contact_urls table
    ↓ repository.read()
Contact.urls: Vec<ContactUrl>
    ↓ from_contact()
ContactForm.urls: Vec<UrlForm>  // label extracted or empty string
```

---

## Label Guidelines

### Recommended Labels

For social media profiles:
- `GitHub` - GitHub profile
- `LinkedIn` - LinkedIn profile
- `Twitter` or `X` - Twitter/X profile
- `Facebook` - Facebook profile
- `Instagram` - Instagram profile
- `Mastodon` - Mastodon profile

For other URLs:
- `Blog` - Personal blog
- `Homepage` - Personal website
- `Work` - Work-related URL
- `Portfolio` - Portfolio website
- `Profile` - Generic profile page

### Label Behavior

- **Optional**: Labels can be left empty (stored as `None` in ContactUrl)
- **Case-Sensitive**: Labels are stored as entered ("GitHub" vs "github")
- **Free-form**: Users can enter any label text
- **Profile Fetching**: In Phase 3, labels will be used to identify which URLs to fetch profile data from

---

## VCF Import/Export

### Import
When importing VCF files, labels are extracted from `itemN.X-ABLabel`:

```
item1.URL:https://github.com/johndoe
item1.X-ABLabel:GitHub
```
↓
```rust
ContactUrl {
    url: "https://github.com/johndoe",
    label: Some("GitHub"),
}
```
↓ Displayed in form as:
```
[https://github.com/johndoe] [GitHub]
```

### Export
When exporting to VCF, labels are written as `itemN.X-ABLabel`:

```rust
ContactUrl {
    url: "https://github.com/johndoe",
    label: Some("GitHub"),
}
```
↓ Exported as:
```
item1.URL:https://github.com/johndoe
item1.X-ABLabel:GitHub
```

---

## User Workflow

### Adding a New Contact with URLs

1. Click "Add Contact" button
2. Fill in name and other fields
3. In URLs section:
   - Enter URL in first field: `https://github.com/johndoe`
   - Enter label in second field: `GitHub`
4. Click "+ Add URL" to add more URLs
5. Click "Save" to create contact

### Editing URL Labels

1. Open contact in edit mode
2. Navigate to URLs section
3. Modify the label field for any URL
4. Click "Save" to update

### Removing URLs

1. Open contact in edit mode
2. Click "−" button next to the URL to remove
3. Note: Cannot remove the last URL (at least one must remain)
4. Click "Save" to update

---

## Display in Contact Details

In the contact detail view, URLs are displayed with their labels:

```
URLs
  GitHub: https://github.com/johndoe
  Blog: https://myblog.com
  LinkedIn: https://linkedin.com/in/johndoe
```

If a URL has no label, it's displayed without a prefix:
```
  https://example.com
```

---

## Benefits

1. **User-Friendly**: Clear indication of what each URL represents
2. **VCF-Compliant**: Uses standard `itemN.X-ABLabel` format
3. **Flexible**: Supports any label text, not limited to predefined list
4. **Profile Fetching Ready**: Labels will be used in Phase 3 to identify which URLs to fetch profile data from
5. **No Data Loss**: All labels are preserved in import/export

---

## Future Enhancements (Phase 3)

When profile fetching is implemented, labels will be used to:

1. **Identify Social Platforms**: Match labels like "GitHub", "LinkedIn" to appropriate fetchers
2. **Skip Non-Social URLs**: URLs with labels like "Blog", "Homepage" won't be fetched
3. **Display Icons**: Show platform-specific icons based on labels
4. **Prioritize Fetching**: Fetch social profiles first, optional URLs later

---

## Testing Checklist

### Manual Testing (Pending)
- [ ] Add new contact with URLs and labels
- [ ] Edit existing URL labels
- [ ] Remove URLs (verify button disabled for last URL)
- [ ] Add multiple URLs with different labels
- [ ] Leave label empty (verify stored as None)
- [ ] Import VCF with labeled URLs
- [ ] Export contact and verify labels in VCF
- [ ] Verify labels display correctly in detail view

### Automated Testing (Complete)
- ✅ All 44 unit tests passing
- ✅ VCF import/export with labels tested
- ✅ ContactUrl creation with labels tested
- ✅ Database CRUD with URLs and labels tested

---

## Code References

- **UI Forms**: `src/ui/mod.rs` (lines ~60-80, ~140-165, ~350-370, ~580-620, ~1240-1265)
- **Data Models**: `src/core/contact.rs` (ContactUrl struct)
- **VCF Parser**: `src/vcf/mod.rs` (extract_urls function)
- **Database**: `src/db/models.rs` (ContactUrlRow)

---

**Status**: ✅ Implementation Complete, Manual Testing Pending
**Last Updated**: 2026-01-15