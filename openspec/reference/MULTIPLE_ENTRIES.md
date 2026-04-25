# Multiple Contact Entries Reference

How UI handles multiple entries of the same field type.

---

## Overview

Profile Pulse supports multiple entries per field type (emails, phones, addresses, dates, URLs) with labels.

---

## Email Handling

### Add Multiple Emails

- User clicks "Add Email" button
- New email form row appears with default label "Home"
- Each email has: address input + label dropdown + remove button
- Remove button only shows when > 1 email exists

### Label Options

- Home, Work, Other (preset)
- Custom (user-entered)

### Display in Detail View

```
Emails:
  ✉ home@example.com (Home)
  ✉ work@example.com (Work)
  ✉ other@example.com (Custom)
```

---

## Phone Handling

### Label Options

- Mobile (default), Home, Work, Main, Home Fax, Work Fax, Pager, Other

### Display

```
Phones:
  📱 +1 555-123-4567 (Mobile)
  📱 +1 555-987-6543 (Work)
  📱 +1 555-111-2222 (Home Fax)
```

---

## Address Handling

### Fields

- Street, City, State, Postal Code, Country
- Label: Home, Work, Other

### Display

```
Addresses:
  🏠 Home
     123 Main St
     Springfield, IL 62701
     USA
```

---

## Date Handling

### Label Options

- Birthday (default), Anniversary, Other

### Display

```
Dates:
  🎂 Birthday: 1990-05-15
  🎂 Anniversary: 2015-06-20
```

---

## URL Handling

### Label Options

- HomePage, Work, Blog, Profile
- GitHub, LinkedIn, Twitter, Facebook, Instagram, Mastodon

### Display

```
URLs:
  🌐 GitHub: https://github.com/user
  🌐 LinkedIn: https://linkedin.com/in/user
  🌐 Blog: https://myblog.com
```

---

## Implementation

- ContactForm uses `Vec<EmailForm>`, `Vec<PhoneForm>`, etc.
- UI adds/removes rows dynamically
- Database stores each as separate row

---

**Status**: Reference Only
