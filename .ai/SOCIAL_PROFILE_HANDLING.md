# Social Profile Handling - Current Implementation

## 🔍 How It Currently Works

### Step 1: VCF Import Process
```rust
// In parse_vcard()
let social_profiles = extract_social_profiles(&vcard);
for profile in social_profiles {
    builder = builder.social_profile(profile);
}
```

### Step 2: URL Extraction
```rust
fn extract_social_profiles(vcard: &VCard) -> Vec<SocialProfile> {
    for prop in &vcard.properties {
        if prop.name == "URL" || prop.name == "X-SOCIALPROFILE" {
            if let Some(profile) = parse_social_url(&prop.value) {
                profiles.push(profile);
            }
        }
    }
}
```

**Key point**: Iterates through ALL URL fields, but only keeps recognized social media URLs.

### Step 3: Social Media Detection
```rust
fn parse_social_url(url: &str) -> Option<SocialProfile> {
    // Returns Some(profile) ONLY if URL matches:
    // - linkedin.com/in/USERNAME
    // - twitter.com/USERNAME or x.com/USERNAME
    // - facebook.com/USERNAME
    // - instagram.com/USERNAME
    // - github.com/USERNAME
    // - DOMAIN/@USERNAME (Mastodon)
    
    // Returns None for everything else
}
```

### Step 4: Username Extraction
```rust
fn extract_path_segment(url: &str, after: &str) -> Option<String> {
    // Extracts username from path
    // Example: "github.com/johndoe" → "johndoe"
}
```

---

## 📊 Test File Analysis

### URLs in test contact.vcf:

| URL | Label | Result |
|-----|-------|--------|
| `https://profile.com` | PROFILE | ❌ Lost (no username path) |
| `https://blog.com` | BLOG | ❌ Lost (no username path) |
| `https://homepage.com` | HomePage | ❌ Lost (no username path) |
| `https://work.com` | WORK | ❌ Lost (no username path) |
| `https://github.com` | GitHub | ❌ Lost (no username!) |
| `https://instagram.com` | Instagram | ❌ Lost (no username!) |

### Why They're Lost:

1. **Generic URLs** (profile.com, blog.com, etc.)
   - Not recognized as social media platforms
   - `parse_social_url()` returns `None`
   - Discarded completely

2. **Social domains without usernames** (github.com, instagram.com)
   - Recognized as platforms
   - But no username in path
   - `extract_path_segment()` returns `None`
   - Also discarded

---

## 🚨 Critical Problems

### Problem 1: Only Social Media URLs Are Kept
**Current behavior**:
- ✅ Keeps: `https://github.com/johndoe`
- ❌ Loses: `https://blog.com`

**What we need**:
- ✅ Keep ALL URLs
- ✅ Tag social media URLs as profiles
- ✅ Store generic URLs separately

### Problem 2: Social URLs Without Usernames Lost
**Current behavior**:
- ✅ Keeps: `https://instagram.com/johndoe`
- ❌ Loses: `https://instagram.com`

**What we need**:
- ✅ Keep URL even without username
- ⚠️  Maybe mark as "unverified" or "incomplete"

### Problem 3: Custom Labels Ignored
**Current behavior**:
```vcf
item6.URL:https://github.com
item6.X-ABLabel:GitHub
```
- Label "GitHub" is read but not associated
- `itemN.X-ABLabel` pattern not handled

**What we need**:
- Parse `itemN.` prefix
- Associate `X-ABLabel` with corresponding field
- Store label with URL/profile

### Problem 4: Social Profiles vs Generic URLs
**Data structure mismatch**:

**We have**:
```rust
Contact {
    social_profiles: Vec<SocialProfile>,  // Only for recognized platforms
    // Where do generic URLs go?
}
```

**We need**:
```rust
Contact {
    social_profiles: Vec<SocialProfile>,  // LinkedIn, GitHub, etc.
    urls: Vec<String>,                    // OR store ALL in custom_fields
}
```

**Current workaround**:
- URLs stored in `custom_fields` as `url_0`, `url_1`, etc. (from UI)
- But VCF import doesn't populate these!

---

## 🎯 What Actually Happens on Import

### Example: Import test contact.vcf

**Input**:
```vcf
item3.URL:https://profile.com
item6.URL:https://github.com
item7.URL:https://instagram.com
```

**Processing**:
1. `extract_social_profiles()` is called
2. All 3 URLs processed by `parse_social_url()`
3. Results:
   - `profile.com` → `None` (not a recognized platform)
   - `github.com` → `None` (no username path)
   - `instagram.com` → `None` (no username path)
4. All URLs discarded
5. **Result**: Contact has 0 social profiles, 0 URLs

**Expected**:
- Contact should have 3 URLs stored somewhere
- GitHub and Instagram should be tagged as potential social profiles

---

## 🔄 Current Data Flow

```
VCF File
  ↓
parse_vcard()
  ↓
extract_social_profiles() → filters for social media URLs only
  ↓
parse_social_url() → returns None for non-social or incomplete URLs
  ↓
ContactBuilder.social_profile() → adds to social_profiles Vec
  ↓
Contact { social_profiles: Vec<SocialProfile> }
  ↓
Database (social_profiles table)
```

**Missing flow**:
```
VCF File (all URLs)
  ↓
extract_all_urls() ← NEW FUNCTION NEEDED
  ↓
Store in Contact.custom_fields as url_0, url_1, etc.
  ↓
Database (in contact custom_fields JSON)
```

---

## ✅ What Works Well

1. **Username extraction** from full URLs
   - `github.com/johndoe` → username: "johndoe" ✅
   - Handles query params, fragments, trailing slashes ✅

2. **Platform detection** for recognized domains
   - LinkedIn, Twitter/X, Facebook, Instagram, GitHub, Mastodon ✅
   - Case-insensitive matching ✅

3. **SocialProfile data structure**
   - Rich metadata: verified, confidence_score, discovered_at ✅
   - UUID for each profile ✅
   - Timestamps ✅

4. **Database storage**
   - Social profiles stored in separate table ✅
   - Proper foreign key relationships ✅

---

## ❌ What's Broken

1. **Generic URLs completely lost** (blog, homepage, work site)
2. **Incomplete social URLs lost** (github.com without /username)
3. **Custom labels ignored** (PROFILE, BLOG, HomePage)
4. **itemN.X-ABLabel pattern not parsed**
5. **No fallback storage** for unrecognized URLs
6. **Only looks at URL fields** (ignores custom URL fields in VCF)

---

## 🛠️ How to Fix

### Fix 1: Extract ALL URLs (Not Just Social)
```rust
fn extract_all_urls(vcard: &VCard) -> Vec<(String, Option<String>)> {
    // Returns Vec of (url, optional_label)
    let mut urls = Vec::new();
    
    // Parse itemN.X-ABLabel associations
    let labels = parse_item_labels(vcard);
    
    for prop in &vcard.properties {
        if prop.name == "URL" || prop.name.ends_with(".URL") {
            let label = extract_label_for_item(prop.name, &labels);
            urls.push((prop.value.clone(), label));
        }
    }
    
    urls
}
```

### Fix 2: Store URLs in custom_fields
```rust
// In parse_vcard()
let urls = extract_all_urls(&vcard);
for (i, (url, label)) in urls.iter().enumerate() {
    builder = builder.custom_field(format!("url_{}", i), url.clone());
    if let Some(lbl) = label {
        builder = builder.custom_field(format!("url_{}_label", i), lbl.clone());
    }
}
```

### Fix 3: Still Extract Social Profiles
```rust
// Also attempt to parse as social profile
for (url, _label) in &urls {
    if let Some(profile) = parse_social_url(url) {
        builder = builder.social_profile(profile);
    }
}
```

**Result**: 
- ALL URLs preserved in custom_fields
- Social media URLs ALSO in social_profiles
- Labels associated with URLs
- No data loss!

---

## 📝 Recommended Implementation

### Option A: Dual Storage (Recommended)
- Store ALL URLs in `custom_fields` (url_0, url_1, etc.)
- ALSO extract social profiles to `social_profiles` table
- Social URLs exist in both places
- No data loss, enables profile fetching

### Option B: URLs Only
- Store ALL URLs in `custom_fields`
- Dynamically detect social platforms in UI
- No separate social_profiles table usage
- Simpler but loses rich metadata

### Option C: Social Profiles Only
- Keep current behavior
- Accept loss of generic URLs
- Only social media profiles matter
- Simplest but most data loss

**Vote**: Option A - Best of both worlds

---

## 🧪 Test Case

### Input VCF:
```vcf
item1.URL:https://blog.johndoe.com
item1.X-ABLabel:BLOG
item2.URL:https://github.com/johndoe
item2.X-ABLabel:GitHub
URL;TYPE=WORK:https://company.com
```

### Expected Output:
```rust
Contact {
    custom_fields: {
        "url_0": "https://blog.johndoe.com",
        "url_0_label": "BLOG",
        "url_1": "https://github.com/johndoe",
        "url_1_label": "GitHub",
        "url_2": "https://company.com",
        "url_2_label": "WORK",
    },
    social_profiles: [
        SocialProfile {
            platform: GitHub,
            username: "johndoe",
            url: "https://github.com/johndoe",
        }
    ]
}
```

### Current Output:
```rust
Contact {
    custom_fields: {},
    social_profiles: [
        SocialProfile {
            platform: GitHub,
            username: "johndoe",
            url: "https://github.com/johndoe",
        }
    ]
}
```

**Data loss**: Blog URL and company URL lost!

---

## 📊 Summary

| Aspect | Current | Needed |
|--------|---------|--------|
| Social URLs with username | ✅ Extracted | ✅ Keep |
| Social URLs without username | ❌ Lost | ⚠️  Preserve |
| Generic URLs | ❌ Lost | ✅ Preserve |
| URL labels | ❌ Ignored | ✅ Preserve |
| itemN.X-ABLabel | ❌ Not parsed | ✅ Parse |
| Storage location | social_profiles only | custom_fields + social_profiles |

**Verdict**: Need to implement dual-storage approach to preserve all data while maintaining social profile functionality.
