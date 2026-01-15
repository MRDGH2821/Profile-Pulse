# VCF Parsing - Current Status

## Test File Analysis
**File**: `.ai/samples/test contact.vcf`

### Field Count in Test File:
- ✉️  **3 Emails** (HOME, WORK, Custom)
- 📱 **9 Phones** (HOME, WORK, CELL, MAIN, FAX×2, googleVoice, PAGER, unlabeled)
- 🌐 **6 URLs** (PROFILE, BLOG, HomePage, WORK, GitHub, Instagram)
- 🏠 **2 Addresses** (work, home) - SKIPPED FOR NOW
- 📅 **3 Dates** (BDAY, Anniversary, unlabeled X-ABDATE)
- 📝 **Other**: NICKNAME, NOTE, CATEGORIES, ORG, TITLE, 3 phonetic names, FILE-AS

---

## ✅ What We Currently Extract

### 1. **Name** (FN field)
```
FN:Prefix First name Middle name Surname Suffix
```
- ✅ Extract: "Prefix First name Middle name Surname Suffix"
- ❌ Missing: Structured name (prefix, middle, suffix separate)
- ❌ Missing: Phonetic names (X-PHONETIC-FIRST-NAME, etc.)
- ❌ Missing: FILE-AS for custom sorting

**Result**: Name stored correctly, but structure lost

---

### 2. **Email** (EMAIL fields) 
```
EMAIL;TYPE=INTERNET;TYPE=HOME:home@email.com
EMAIL;TYPE=INTERNET;TYPE=WORK:work@email.com
item1.EMAIL;TYPE=INTERNET:custom@email.com
```
- ✅ Extract: First email only → `home@email.com`
- ❌ Missing: Other 2 emails (work, custom)
- ❌ Missing: All TYPE labels (HOME, WORK, Custom)
- ❌ Missing: itemN.X-ABLabel pattern

**Result**: Only 1 of 3 emails preserved, no labels

---

### 3. **Phone** (TEL fields)
```
TEL;TYPE=HOME:+91 99999 99999
TEL;TYPE=WORK:+1 444-444-4444
... (7 more phones)
```
- ✅ Extract: First phone only → `+91 99999 99999`
- ❌ Missing: Other 8 phones
- ❌ Missing: All TYPE labels (HOME, WORK, CELL, etc.)
- ❌ Missing: Custom labels (googleVoice)

**Result**: Only 1 of 9 phones preserved, no labels

---

### 4. **Organization** (ORG field)
```
ORG:Company;Department
```
- ✅ Extract: "Company" (first part before semicolon)
- ❌ Missing: "Department" (second part)

**Result**: Company stored, department lost

---

### 5. **Title** (TITLE field)
```
TITLE:Job title
```
- ✅ Extract: "Job title"

**Result**: ✅ Fully working

---

### 6. **URLs** (URL fields)
```
item3.URL:https://profile.com
item3.X-ABLabel:PROFILE
item6.URL:https://github.com
item6.X-ABLabel:GitHub
item7.URL:https://instagram.com
item7.X-ABLabel:Instagram
```
- ✅ Extract: GitHub and Instagram URLs (parsed as social profiles)
- ❌ Missing: Other URLs (profile.com, blog.com, homepage.com, work.com)
- ❌ Missing: All URL labels (PROFILE, BLOG, HomePage, WORK)
- ❌ Missing: itemN.X-ABLabel associations

**Result**: 2 of 6 URLs preserved (as social profiles), others lost

---

### 7. **Social Profiles**
- ✅ Extract: GitHub profile (username from URL)
- ✅ Extract: Instagram profile (username from URL)
- ❌ Missing: LinkedIn, Twitter, Facebook (not in this test file)

**Result**: Partially working (URL detection works, but limited by URL extraction)

---

### 8. **Birthday** (BDAY field)
```
BDAY:19700101
```
- ❌ Missing: Not extracted at all
- ❌ Missing: Date format conversion (YYYYMMDD → YYYY-MM-DD)

**Result**: Birthday lost

---

### 9. **Significant Dates** (X-ABDATE fields)
```
item8.X-ABDATE:20000101
item8.X-ABLabel:_$!<Anniversary>!$_
X-ABDATE:20100101
```
- ❌ Missing: Not extracted
- ❌ Missing: itemN.X-ABLabel pattern for labels
- ❌ Missing: Date format conversion

**Result**: All significant dates lost

---

### 10. **Nickname** (NICKNAME field)
```
NICKNAME:Nickname
```
- ❌ Missing: Not explicitly extracted
- ⚠️  Might be in custom_fields (X- fields)

**Result**: Likely lost

---

### 11. **Notes** (NOTE field)
```
NOTE:Notes sample
```
- ✅ Extract: Stored in custom_fields as "NOTE"

**Result**: ✅ Working (in custom_fields)

---

### 12. **Categories** (CATEGORIES field)
```
CATEGORIES:myContacts
```
- ❌ Missing: Not extracted

**Result**: Lost

---

### 13. **Custom Fields** (X-* fields)
```
X-PHONETIC-FIRST-NAME:Phonetic first
X-PHONETIC-MIDDLE-NAME:Phonetic middle
X-PHONETIC-LAST-NAME:Phonetic last
X-FILE-AS:File as
```
- ✅ Extract: All X-* fields stored in custom_fields
- ⚠️  But no special handling (just raw storage)

**Result**: Preserved but not parsed

---

## 📊 Summary Statistics

### Data Preservation Rate:

| Field Type | In File | Extracted | Rate |
|------------|---------|-----------|------|
| Name | 1 | 1 | ✅ 100% |
| Emails | 3 | 1 | ❌ 33% |
| Phones | 9 | 1 | ❌ 11% |
| Organization | 1 (+dept) | 1 | ⚠️  50% |
| Title | 1 | 1 | ✅ 100% |
| URLs | 6 | 2* | ❌ 33% |
| Social Profiles | 2 | 2 | ✅ 100% |
| Birthday | 1 | 0 | ❌ 0% |
| Dates | 2 | 0 | ❌ 0% |
| Nickname | 1 | 0 | ❌ 0% |
| Notes | 1 | 1 | ✅ 100% |
| Addresses | 2 | 0 | ⏭️ Skipped |
| Categories | 1 | 0 | ❌ 0% |

*Only URLs recognized as social profiles

### Overall Data Loss:
- **Captured**: ~40-50% of data
- **Lost**: ~50-60% of data
- **Major losses**: Multiple emails/phones, labels, dates, nickname

---

## 🚨 Critical Issues

### 1. **itemN.X-ABLabel Pattern NOT Handled**
```vcf
item1.EMAIL;TYPE=INTERNET:custom@email.com
item1.X-ABLabel:Custom
```
- We don't associate item1.EMAIL with item1.X-ABLabel
- Lose all custom labels

### 2. **Only First Value Extracted**
- `extract_email()` returns first email only
- `extract_phone()` returns first phone only
- Other values completely lost

### 3. **Date Format Not Converted**
- VCF uses: `YYYYMMDD` (e.g., `19700101`)
- We need: `YYYY-MM-DD` (e.g., `1970-01-01`)

### 4. **No BDAY Handling**
- Birthday field not extracted at all

### 5. **No X-ABDATE Handling**
- Significant dates not extracted

### 6. **No NICKNAME Handling**
- Nickname not explicitly extracted

### 7. **Department Lost from ORG**
- Only first part of `ORG:Company;Department` extracted

---

## 🎯 What Our UI Supports vs What We Import

### UI Supports:
- ✅ Multiple emails (Vec<String>)
- ✅ Multiple phones (Vec<String>)
- ✅ Multiple URLs (Vec<String>)
- ✅ Multiple addresses (Vec<Address>)
- ✅ Multiple significant dates (Vec<SignificantDate>)
- ✅ Nickname field
- ✅ Birthday field
- ✅ Department field
- ✅ Notes field
- ✅ Custom fields (Vec<CustomFieldPair>)

### VCF Parser Extracts:
- ❌ Only 1 email
- ❌ Only 1 phone
- ❌ Only social profile URLs (not all URLs)
- ❌ No addresses (skipped)
- ❌ No dates
- ❌ No nickname
- ❌ No birthday
- ❌ No department (from ORG)
- ✅ Notes (in custom_fields)
- ⚠️  X-* fields (raw, not parsed)

**Gap**: UI can display everything, but importer doesn't populate it!

---

## 🔧 What Needs Fixing

### Priority 1 (High Impact):
1. **Extract ALL emails** (not just first)
2. **Extract ALL phones** (not just first)
3. **Extract ALL URLs** (not just social profiles)
4. **Parse BDAY field** with date conversion
5. **Parse X-ABDATE fields** with date conversion
6. **Parse NICKNAME field**
7. **Extract department from ORG field**

### Priority 2 (Labels & Types):
8. **Handle itemN.X-ABLabel pattern**
9. **Store TYPE parameters** (HOME, WORK, CELL)
10. **Associate labels with fields**

### Priority 3 (Nice to Have):
11. Parse structured name (N field) for prefix/suffix
12. Extract phonetic names
13. Extract FILE-AS
14. Extract CATEGORIES

---

## 📝 Recommended Next Steps

### Option A: Quick Wins (2-3 hours)
- Fix email extraction (all emails)
- Fix phone extraction (all phones)  
- Fix URL extraction (all URLs)
- Add BDAY parsing
- Add NICKNAME parsing
- Add department from ORG

### Option B: Comprehensive Fix (5-6 hours)
- All of Option A
- Add X-ABDATE parsing
- Implement itemN.X-ABLabel handling
- Store field types (HOME/WORK/CELL)
- Date format conversion (YYYYMMDD → YYYY-MM-DD)

### Option C: Phase 3 First
- Continue with profile picture fetching
- Fix VCF parsing later when needed

---

## 🧪 Testing Plan

After fixes:
1. Import test contact.vcf
2. Verify in UI:
   - All 3 emails visible
   - All 9 phones visible
   - All 6 URLs visible
   - Birthday displayed
   - Nickname displayed
   - Department displayed
   - Notes displayed
3. Export and re-import (round-trip test)
4. Compare with original

---

## Current Code Location

**Parser**: `src/vcf/mod.rs`
- Lines 1-400: Import logic
- Key functions:
  - `extract_email()` - Line ~208 (returns first only)
  - `extract_phone()` - Line ~218 (returns first only)
  - `extract_social_profiles()` - Line ~250 (partial URL handling)
  - `extract_custom_fields()` - Line ~369 (handles X-* fields)

**Missing Functions**:
- `extract_all_emails()` - Needed
- `extract_all_phones()` - Needed
- `extract_all_urls()` - Needed
- `extract_birthday()` - Needed
- `extract_significant_dates()` - Needed
- `extract_nickname()` - Needed
- `parse_item_labels()` - Needed

