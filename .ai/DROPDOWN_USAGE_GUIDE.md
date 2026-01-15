# Label Dropdown Usage Guide

## Overview

This guide shows how to use the label dropdown feature in Profile Pulse to categorize your contact information.

## Quick Start

When adding or editing contact information, you'll see dropdown menus next to each field that let you choose how to label it. For example:
- An email can be labeled as "Home", "Work", "Other", or a custom label like "School"
- A phone can be labeled as "Mobile", "Home", "Work", "Main", or custom labels like "Google Voice"

## Visual Walkthrough

### Adding an Email Address

**Step 1: Default State**
```
Email Addresses                                    [+ Add Email]
┌────────────────────────────────────┐ ┌──────────┐
│ john@example.com                   │ │ Home ▼   │ [−]
└────────────────────────────────────┘ └──────────┘
```
- Email input field on the left (75% width)
- Label dropdown on the right (25% width)
- Default selection is "Home"

**Step 2: Changing Label**
```
Email Addresses                                    [+ Add Email]
┌────────────────────────────────────┐ ┌──────────┐
│ john@work.com                      │ │ Work ▼   │ [−]
└────────────────────────────────────┘ └──────────┘
```
- Click dropdown to see options: Home, Work, Other, Custom
- Select "Work" from the list

**Step 3: Using Custom Label**
```
Email Addresses                                    [+ Add Email]
┌────────────────────────────────────┐ ┌──────────┐
│ john@school.edu                    │ │ Custom ▼ │ [−]
└────────────────────────────────────┘ └──────────┘
┌────────────────────────────────────────────────────────┐
│ School                                                 │ ← Custom label input
└────────────────────────────────────────────────────────┘
```
- Select "Custom" from dropdown
- Text input field appears below
- Type custom label like "School" or "Personal Project"

### Adding a Phone Number

**Available Options**
```
Phone Numbers                                      [+ Add Phone]
┌────────────────────────────────────┐ ┌──────────┐
│ +1-555-0100                        │ │ Mobile ▼ │ [−]
└────────────────────────────────────┘ └──────────┘

Dropdown options:
  • Mobile    ← Most common, default
  • Home
  • Work
  • Main
  • Home Fax
  • Work Fax
  • Pager
  • Other
  • Custom
```

**Custom Phone Label Example**
```
Phone Numbers                                      [+ Add Phone]
┌────────────────────────────────────┐ ┌──────────┐
│ +1-800-GOOG-411                    │ │ Custom ▼ │ [−]
└────────────────────────────────────┘ └──────────┘
┌────────────────────────────────────────────────────────┐
│ Google Voice                                           │
└────────────────────────────────────────────────────────┘
```

### Adding an Address

**Layout**
```
Addresses                                          [+ Add Address]
┌──────────────────────────────────────────────────────────────┐
│ Address 1                                            [−]      │
│                                                                │
│ ┌──────────┐                                                  │
│ │ Home   ▼ │  ← Label dropdown                               │
│ └──────────┘                                                  │
│                                                                │
│ ┌────────────────────────────────────────────────────────┐   │
│ │ 123 Main St                                            │   │
│ └────────────────────────────────────────────────────────┘   │
│                                                                │
│ ┌─────────────────────┐ ┌──────┐ ┌──────────┐               │
│ │ Springfield         │ │ IL   │ │ 62701    │               │
│ └─────────────────────┘ └──────┘ └──────────┘               │
│                                                                │
│ ┌────────────────────────────────────────────────────────┐   │
│ │ USA                                                    │   │
│ └────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

**Custom Address Label**
```
┌──────────┐
│ Custom ▼ │
└──────────┘
┌────────────────────────────────────────────────────────┐
│ Vacation Home                                          │ ← Custom label
└────────────────────────────────────────────────────────┘
```

### Adding a Date

**Birthday Example**
```
Significant Dates
(Birthdays, anniversaries, etc.)
┌──────────────┐ ┌──────────────────────┐
│ Birthday   ▼ │ │ 1990-05-15           │ [−]
└──────────────┘ └──────────────────────┘
```

**Custom Date Example**
```
┌──────────────┐ ┌──────────────────────┐
│ Custom     ▼ │ │ 2015-06-20           │ [−]
└──────────────┘ └──────────────────────┘
┌────────────────────────────────────────────────────────┐
│ First Day at Current Job                              │
└────────────────────────────────────────────────────────┘
```

### Adding a URL

**Social Media Example**
```
URLs (websites, profile pictures)                  [+ Add URL]
Note: First URL will be used as profile picture source

┌────────────────────────────────────┐ ┌──────────┐
│ https://github.com/johndoe         │ │ GitHub ▼ │ [−]
└────────────────────────────────────┘ └──────────┘

Dropdown options:
  • HomePage
  • Work
  • Blog
  • Profile
  • GitHub    ← Great for developer profiles
  • LinkedIn
  • Twitter
  • Facebook
  • Instagram
  • Mastodon
  • Other
  • Custom
```

**Custom URL Example**
```
┌────────────────────────────────────┐ ┌──────────┐
│ https://myproject.io                │ │ Custom ▼ │ [−]
└────────────────────────────────────┘ └──────────┘
┌────────────────────────────────────────────────────────┐
│ Side Project                                           │
└────────────────────────────────────────────────────────┘
```

## Label Options by Field Type

### Email Labels
- **Home** - Personal email address
- **Work** - Professional/business email
- **Other** - Secondary or alternative email
- **Custom** - Anything else (School, Volunteer, etc.)

### Phone Labels
- **Mobile** - Cell phone (default, most common)
- **Home** - Landline at residence
- **Work** - Office phone number
- **Main** - Primary contact number
- **Home Fax** - Fax machine at home
- **Work Fax** - Fax machine at office
- **Pager** - Pager/beeper number
- **Other** - Secondary phone
- **Custom** - Google Voice, WhatsApp, etc.

### Address Labels
- **Home** - Residential address
- **Work** - Office/workplace address
- **Other** - Alternative address
- **Custom** - Vacation Home, PO Box, Storage Unit, etc.

### Date Labels
- **Birthday** - Date of birth
- **Anniversary** - Wedding anniversary or relationship milestone
- **Other** - Other significant date
- **Custom** - First Day at Job, Graduation, etc.

### URL Labels
- **HomePage** - Personal website or blog
- **Work** - Company website
- **Blog** - Personal or professional blog
- **Profile** - Generic profile page
- **GitHub** - GitHub profile
- **LinkedIn** - LinkedIn profile
- **Twitter** - Twitter/X profile
- **Facebook** - Facebook profile
- **Instagram** - Instagram profile
- **Mastodon** - Mastodon profile
- **Other** - Other website
- **Custom** - Portfolio, YouTube, TikTok, etc.

## Tips and Best Practices

### 1. Use Predefined Labels When Possible
Predefined labels (Home, Work, Mobile, etc.) are recognized by vCard standards and will work better when exporting to other contact management systems.

### 2. Custom Labels Are Preserved
Custom labels are fully preserved when:
- Saving to database
- Exporting to VCF files
- Importing from VCF files
- Syncing with other systems (if supported)

### 3. Keep Custom Labels Short
While there's no strict limit, shorter labels (1-3 words) display better in:
- Contact list views
- Export formats
- Mobile devices

Good: "Google Voice", "School", "Side Project"
Avoid: "The phone number I use when traveling internationally"

### 4. Be Consistent
If you use a custom label like "Google Voice" for one contact, use the same label for other contacts. This makes it easier to:
- Search and filter contacts
- Understand your contact organization
- Future label suggestion features will work better

### 5. Social Media Platforms
For social media URLs, use the predefined platform labels (GitHub, LinkedIn, etc.) rather than custom labels. This enables:
- Automatic profile fetching (future feature)
- Platform-specific icons
- Better integration with discovery features

## Editing Existing Contacts

### How Labels Appear When Editing

When you edit a contact that already has labeled fields:

**Predefined Label**
```
┌────────────────────────────────────┐ ┌──────────┐
│ john@work.com                      │ │ Work ▼   │ [−]
└────────────────────────────────────┘ └──────────┘
```
- Dropdown automatically shows "Work" selected

**Custom Label**
```
┌────────────────────────────────────┐ ┌──────────┐
│ john@school.edu                    │ │ Custom ▼ │ [−]
└────────────────────────────────────┘ └──────────┘
┌────────────────────────────────────────────────────────┐
│ School                                                 │
└────────────────────────────────────────────────────────┘
```
- Dropdown shows "Custom" selected
- Custom label text field appears with existing label

### Changing Labels

**From Predefined to Different Predefined**
1. Click dropdown
2. Select new option (e.g., Home → Work)
3. Label changes immediately

**From Predefined to Custom**
1. Click dropdown
2. Select "Custom"
3. Text field appears
4. Type custom label
5. Label updates as you type

**From Custom to Predefined**
1. Click dropdown
2. Select predefined option (e.g., Home)
3. Custom text field disappears
4. Label changes to predefined value

**Editing Custom Label Text**
1. Dropdown already shows "Custom"
2. Simply edit text in custom field
3. Label updates as you type

## Importing Contacts

When you import a VCF (vCard) file:

### Standard Labels Recognized
The system automatically recognizes standard vCard labels:
- TYPE=HOME → Home
- TYPE=WORK → Work
- TYPE=CELL → Mobile
- TYPE=PAGER → Pager
- etc.

### Custom Labels Preserved
Custom labels from other systems are preserved:
- Apple's `itemN.X-ABLabel` format
- Custom TYPE values
- Non-standard label formats

### What You'll See
After importing:
- Standard labels appear in dropdown as expected
- Custom labels show "Custom" selected with text field populated
- All labels are fully editable

## Exporting Contacts

When you export contacts to VCF format:

### Predefined Labels
Exported using standard vCard TYPE parameters:
```
EMAIL;TYPE=WORK:john@work.com
TEL;TYPE=CELL:+1-555-0100
ADR;TYPE=HOME:;;123 Main St;Springfield;IL;62701;USA
```

### Custom Labels
Exported using Apple's `itemN.X-ABLabel` format:
```
item1.EMAIL:john@school.edu
item1.X-ABLabel:School
item2.TEL:+1-800-GOOG-411
item2.X-ABLabel:Google Voice
```

This ensures maximum compatibility with:
- Apple Contacts (macOS, iOS)
- Google Contacts
- Microsoft Outlook
- Other vCard-compliant systems

## Keyboard Shortcuts

### Navigation
- **Tab** - Move to next field
- **Shift+Tab** - Move to previous field
- **Enter** - Open dropdown (when focused)
- **↑/↓ Arrow Keys** - Navigate dropdown options
- **Enter** - Select dropdown option
- **Esc** - Close dropdown without selecting

### Quick Entry
When dropdown is focused:
- Type first letter to jump to option (e.g., "W" → Work)
- Type multiple times to cycle through options starting with that letter

## Troubleshooting

### Dropdown Not Showing Options
- Click directly on the dropdown (arrow icon area)
- Make sure contact form is in edit mode
- Check that field hasn't been removed

### Custom Label Not Saving
- Make sure "Custom" is selected in dropdown
- Type label in the text field that appears below
- Verify label appears before clicking Save Contact

### Label Changed Unexpectedly
- Loading contacts with unrecognized labels automatically selects "Custom"
- This is normal behavior - your custom label is preserved in the text field

### Can't Enter Custom Label
- First select "Custom" from dropdown
- Text field will appear below dropdown
- Then type your custom label

## Examples: Common Use Cases

### Freelancer with Multiple Clients
```
Emails:
- personal@example.com        [Home]
- work@company.com            [Work]
- freelance@example.com       [Custom: Freelance]
- clientA@example.com         [Custom: Client A]

Phones:
- +1-555-0100                 [Mobile]
- +1-555-0200                 [Custom: Business Line]
```

### Student
```
Emails:
- personal@gmail.com          [Home]
- student@university.edu      [Custom: School]
- club@university.edu         [Custom: Club President]

Phones:
- +1-555-0100                 [Mobile]
- +1-555-0200                 [Home]

Dates:
- 1995-03-15                  [Birthday]
- 2023-09-01                  [Custom: Started University]
```

### Developer/Tech Professional
```
URLs:
- https://github.com/username       [GitHub]
- https://linkedin.com/in/user      [LinkedIn]
- https://example.com               [HomePage]
- https://blog.example.com          [Blog]
- https://twitter.com/username      [Twitter]
- https://stackoverflow.com/u/123   [Custom: Stack Overflow]

Emails:
- personal@example.com              [Home]
- work@company.com                  [Work]
- opensource@example.com            [Custom: Open Source]
```

### Family Contact with Multiple Addresses
```
Addresses:
- 123 Main St, Springfield, IL     [Home]
- 456 Oak Ave, Chicago, IL         [Custom: Summer House]
- PO Box 789, Springfield, IL      [Custom: Mail Only]
```

## Future Features

These features are planned for future releases:

### Label Suggestions
- System will learn your custom labels
- Suggest frequently-used labels when entering new contacts
- Quick-select from your most common custom labels

### Label Icons
- Visual icons for predefined labels
- Platform logos for social media URLs
- Custom icon selection for custom labels

### Bulk Label Editing
- Change labels across multiple contacts at once
- Find and replace labels
- Normalize label variations

### Label Analytics
- See most-used labels
- Identify duplicate or similar custom labels
- Suggestions for label consolidation

---

## Getting Help

If you have questions or issues with label dropdowns:
1. Check this guide for examples
2. Review the main documentation in `docs/`
3. Submit an issue on GitHub
4. Contact support

**Remember**: All labels (predefined and custom) are preserved when saving, exporting, and importing contacts. You can't lose data by experimenting with labels!