# Dropdown Implementation Reference

How dropdowns are implemented in the Iced UI.

---

## Overview

Label dropdowns use Iced's PickList widget with common options + custom text entry.

---

## Email Label Dropdown

```rust
PickList::new(
    &EmailLabel::common_options(),
    selected_label,
    EmailLabelChanged,
)
```

### Options

- "Home" (default)
- "Work"
- "Other"
- "Custom" → shows text input

### Implementation

- PickList displays dropdown
- On "Custom" selection, text input appears
- User can type custom label

---

## Phone Label Dropdown

### Options

- Mobile (default)
- Home
- Work
- Main
- Home Fax
- Work Fax
- Pager
- Other
- Custom

### Mappings (TYPE parameter)

- Mobile → TYPE=CELL
- Home → TYPE=HOME
- Work → TYPE=WORK
- Home Fax → TYPE=HOME;FAX

---

## Address Label Dropdown

### Options

- Home (default)
- Work
- Other
- Custom

---

## Date Label Dropdown

### Options

- Birthday (default)
- Anniversary
- Other
- Custom

### Special Handling

- Anniversary uses Apple format: `_$!<Anniversary>!$_`

---

## URL Label Dropdown

### Options

- HomePage, Work, Blog, Profile
- GitHub, LinkedIn, Twitter, Facebook, Instagram, Mastodon
- Other
- Custom

### Social Detection

- GitHub, LinkedIn, Twitter, Facebook, Instagram, Mastodon → social media
- Others → generic URL

---

## UI Components

1. **PickList** - Dropdown selection
2. **TextInput** - Custom label entry
3. **Row** - Horizontal layout
4. **Button** - Add/Remove

---

## More Details

Full guide: `.agents/DROPDOWN_USAGE_GUIDE.md` (447 lines)

---

**Status**: Reference Only
