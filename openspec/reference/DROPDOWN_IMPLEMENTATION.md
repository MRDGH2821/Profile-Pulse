# Label Dropdown Implementation Reference

Label dropdown implementation for contact fields.

---

## Features

### Dropdown Widgets for All Label Types

| Field Type | Options                                                                                                |
| ---------- | ------------------------------------------------------------------------------------------------------ |
| Email      | Home, Work, Other, Custom                                                                              |
| Phone      | Mobile, Home, Work, Main, Home Fax, Work Fax, Pager, Other, Custom                                     |
| Address    | Home, Work, Other, Custom                                                                              |
| Date       | Birthday, Anniversary, Other, Custom                                                                   |
| URL        | HomePage, Work, Blog, Profile, GitHub, LinkedIn, Twitter, Facebook, Instagram, Mastodon, Other, Custom |

---

## Custom Label Support

When "Custom" is selected:

- Text input field appears below dropdown
- User can type any custom label
- Label saved as-is to database
- Preserved in import/export cycles

---

## Smart Label Recognition

Case-insensitive matching:

- "WORK" → Work option
- "cell" → Mobile option

Unknown labels:

- Automatically select "Custom"
- Populate text field with existing label

---

## Form Layout

### Email/Phone/URL Fields

```
[Input Field (75%)     ] [Label Dropdown (25%)] [Remove]
[Custom Label Input                              ] (if Custom selected)
```

### Date Fields

```
[Label Dropdown (33%)] [Date Input (67%)     ] [Remove]
[Custom Label Input                              ] (if Custom selected)
```

---

## Database Storage

Labels stored as plain strings per field:

- contact_emails.label
- contact_phones.label
- contact_addresses.label
- contact_dates.label
- contact_urls.label

---

## VCF Import/Export

**Import**: TYPE parameters → dropdown options  
**Export**: Predefined labels → TYPE, Custom → itemN.X-ABLabel  
**Apple Format**: _$!<Label>!$_ handled correctly

---

## Reference

Full guide: `.agents/DROPDOWN_IMPLEMENTATION.md` (251 lines)

---

**Status**: Reference Only
