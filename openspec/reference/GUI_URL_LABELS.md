# GUI URL Labels Reference

How the GUI handles URL labels in contact forms.

---

## Overview

The GUI supports adding and editing labels for URLs directly in contact forms.

---

## Form Layout

Each URL entry has three elements in a row:

1. **URL Input Field**
   - Placeholder: "URL 1", "URL 2", etc.
   - Full URL entry (https://...)

2. **Label Input Field**
   - Placeholder: "Label (e.g., GitHub, Blog)"
   - Short descriptive label

3. **Remove Button**
   - Appears only when > 1 URL exists

---

## Recommended Labels

### Social Media

- GitHub
- LinkedIn
- Twitter or X
- Facebook
- Instagram
- Mastodon

### Other

- Blog
- Homepage
- Work
- Portfolio
- Profile

---

## VCF Import/Export

### Import

When importing VCF files, labels are extracted from itemN.X-ABLabel:

```
item1.URL:https://github.com/johndoe
item1.X-ABLabel:GitHub
```

↓

ContactUrl { url: "...", label: Some("GitHub") }

### Export

When exporting, labels are written as itemN.X-ABLabel:

```
item1.URL:https://github.com/johndoe
item1.X-ABLabel:GitHub
```

---

## Display in Detail View

URLs displayed with their labels:

```
URLs:
  GitHub: https://github.com/johndoe
  Blog: https://myblog.com
  LinkedIn: https://linkedin.com/in/johndoe
```

If no label, displayed without prefix:

```
  https://example.com
```

---

## User Workflow

### Adding a New Contact with URLs

1. Click "Add Contact"
2. Fill in name
3. In URLs section:
   - Enter URL: https://github.com/johndoe
   - Enter label: GitHub
4. Click "+ Add URL" for more URLs
5. Click "Save"

### Editing URL Labels

1. Open contact in edit mode
2. Navigate to URLs section
3. Modify the label field
4. Click "Save"

---

## Reference

Full guide: `.agents/GUI_URL_LABELS.md` (290 lines)

---

**Status**: Reference Only
