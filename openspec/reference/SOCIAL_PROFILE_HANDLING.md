# Social Profile Handling

How URLs are extracted and categorized during VCF import.

---

## Current Implementation

### Step 1: Extract URLs from VCF

Iterates through all URL properties in VCF.

### Step 2: Social Media Detection

Recognized platforms:

- linkedin.com/in/USERNAME
- twitter.com/USERNAME
- facebook.com/USERNAME
- instagram.com/USERNAME
- github.com/USERNAME

### Step 3: Username Extraction

Extracts username from URL path:

- github.com/johndoe → "johndoe"

---

## What Works

- ✅ Username extraction from full URLs
- ✅ Platform detection (LinkedIn, Twitter, etc.)
- ✅ SocialProfile data structure with rich metadata

---

## Problems Identified

| Issue                       | Impact                           |
| --------------------------- | -------------------------------- |
| Generic URLs lost           | blog.com, homepage.com discarded |
| URLs without usernames lost | github.com → discarded           |
| Custom labels ignored       | itemN.X-ABLabel not parsed       |

---

## Recommended Fix: Dual Storage

1. Store ALL URLs in custom_fields as url_0, url_1, etc.
2. ALSO extract social profiles to separate table
3. No data loss, enables profile fetching

---

## Test Case

**Input**:

```
item1.URL:https://blog.johndoe.com
item1.X-ABLabel:BLOG
item2.URL:https://github.com/johndoe
item2.X-ABLabel:GitHub
```

**Expected**:

- custom_fields: url_0 ("blog..."), url_0_label ("BLOG"), url_1 ("github..."), url_1_label ("GitHub")
- social_profiles: GitHub profile with username "johndoe"

---

## Reference

Full doc: `.agents/SOCIAL_PROFILE_HANDLING.md` (401 lines)

---

**Status**: Reference Only
