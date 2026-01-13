# API Integration Guide: Profile Pulse

## Table of Contents

- [Overview](#overview)
- [General Principles](#general-principles)
- [Platform Integrations](#platform-integrations)
  - [GitHub](#github)
  - [LinkedIn](#linkedin)
  - [Twitter/X](#twitterx)
  - [Facebook](#facebook)
  - [Instagram](#instagram)
- [Rate Limiting Strategy](#rate-limiting-strategy)
- [Error Handling](#error-handling)
- [Testing](#testing)
- [Legal & Compliance](#legal--compliance)

---

## Overview

This document provides detailed information about integrating with various social media platforms to fetch profile pictures and discover profiles. Each platform has unique characteristics, rate limits, and technical approaches.

### Integration Status

| Platform  | Method         | Auth Required | Rate Limit                     | Difficulty | Status             |
| --------- | -------------- | ------------- | ------------------------------ | ---------- | ------------------ |
| GitHub    | REST API       | Optional      | 60/hr (unauth), 5000/hr (auth) | Easy       | ✅ Recommended     |
| LinkedIn  | Web Scraping   | No            | ~100/day (conservative)        | Medium     | ⚠️ Fragile         |
| Twitter/X | API v2         | Yes           | Varies by tier                 | Medium     | ⚠️ Paid required   |
| Facebook  | Graph API      | Yes           | Varies                         | Hard       | ⚠️ App review      |
| Instagram | Unofficial API | Maybe         | Very restrictive               | Hard       | ❌ Not recommended |

---

## General Principles

### 1. Respect Rate Limits

Always implement conservative rate limiting:

- Start with lower limits than documented
- Implement exponential backoff
- Cache aggressively
- Respect `Retry-After` headers

### 2. Graceful Degradation

- Platform unavailable → Skip and continue
- Rate limit hit → Queue for later
- Authentication failed → Show clear error
- Network error → Retry with backoff

### 3. Privacy & Ethics

- Only access public profiles
- Respect robots.txt
- Follow platform Terms of Service
- Provide clear user consent
- Allow users to opt-out per platform

### 4. User-Agent Policy

Always use a descriptive User-Agent:

```rust
const USER_AGENT: &str = "ProfilePulse/1.0 (Contact Management App; +https://github.com/yourusername/profile-pulse)";
```

---

## Platform Integrations

### GitHub

**Status**: ✅ Recommended (Most reliable)

#### Why GitHub First?

- Excellent API documentation
- Generous rate limits with authentication
- No authentication required for basic use
- Stable API with versioning
- Clear terms of service

#### API Endpoints

**Get User Profile**:

```http
GET https://api.github.com/users/{username}
```

**Response**:

```json
{
  "login": "octocat",
  "id": 1,
  "avatar_url": "https://avatars.githubusercontent.com/u/1?v=4",
  "name": "The Octocat",
  "email": "octocat@github.com",
  "bio": "...",
  "location": "San Francisco",
  "company": "@github"
}
```

#### Rate Limits

**Unauthenticated**: 60 requests/hour per IP
**Authenticated**: 5,000 requests/hour

Check rate limit status:

```http
GET https://api.github.com/rate_limit
```

#### Implementation

```rust
pub struct GitHubFetcher {
    client: reqwest::Client,
    token: Option<String>,
    rate_limiter: Arc<RateLimiter>,
}

impl GitHubFetcher {
    pub fn new(token: Option<String>) -> Self {
        let rate_limit = if token.is_some() { 5000 } else { 60 };

        Self {
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
            token,
            rate_limiter: Arc::new(RateLimiter::new(rate_limit, Duration::hours(1))),
        }
    }

    pub async fn fetch_profile(&self, username: &str) -> Result<GitHubProfile> {
        self.rate_limiter.acquire().await?;

        let url = format!("https://api.github.com/users/{}", username);
        let mut request = self.client.get(&url);

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request
            .send()
            .await?
            .error_for_status()?;

        // Check remaining rate limit
        if let Some(remaining) = response.headers().get("x-ratelimit-remaining") {
            tracing::info!("GitHub rate limit remaining: {}", remaining.to_str()?);
        }

        let profile: GitHubProfile = response.json().await?;
        Ok(profile)
    }
}
```

#### Best Practices

1. **Use Personal Access Token** for higher rate limits
2. **Cache responses** for 24 hours minimum
3. **Check rate limit headers** in each response
4. **Handle 404** gracefully (user not found)
5. **Handle 403** as rate limit exceeded

#### Search by Email

GitHub doesn't directly support email search, but you can:

1. Extract username from email if it's `username@users.noreply.github.com`
2. Use Google search: `site:github.com "email@example.com"`

---

### LinkedIn

**Status**: ⚠️ Fragile (No official public API)

#### Challenges

- No public API for profile data
- Official API requires partnership
- Must use web scraping
- HTML structure changes frequently
- Rate limiting is aggressive
- May require login for some profiles

#### Approach: Web Scraping

**Profile URL Format**:

```
https://www.linkedin.com/in/{username}/
```

#### Implementation Strategy

```rust
pub struct LinkedInFetcher {
    client: reqwest::Client,
    rate_limiter: Arc<RateLimiter>,
    cache: Arc<CacheService>,
}

impl LinkedInFetcher {
    pub async fn fetch_profile_pic(&self, username: &str) -> Result<Vec<u8>> {
        // Rate limit: Very conservative (10 requests/hour)
        self.rate_limiter.acquire().await?;

        let url = format!("https://www.linkedin.com/in/{}/", username);

        // Random delay to appear more human-like
        tokio::time::sleep(Duration::from_secs(rand::thread_rng().gen_range(2..5))).await;

        let html = self.client
            .get(&url)
            .header("User-Agent", BROWSER_USER_AGENT)
            .header("Accept-Language", "en-US,en;q=0.9")
            .send()
            .await?
            .text()
            .await?;

        // Parse HTML
        let document = scraper::Html::parse_document(&html);

        // Selector for profile picture (may need updates)
        let selector = scraper::Selector::parse("img.pv-top-card-profile-picture__image").unwrap();

        if let Some(img) = document.select(&selector).next() {
            if let Some(src) = img.value().attr("src") {
                return self.download_image(src).await;
            }
        }

        Err(FetchError::ProfilePictureNotFound)
    }
}
```

#### Selectors (As of 2024)

**Note**: These change frequently!

```rust
// Profile picture
"img.pv-top-card-profile-picture__image"
"img[data-delayed-url*='profile-displayphoto-shrink']"

// Name
"h1.text-heading-xlarge"

// Headline
"div.text-body-medium"
```

#### Rate Limiting

**Recommended Limits**:

- 10 requests per hour
- 100 requests per day
- Random delays between requests (2-5 seconds)
- Retry after 24 hours if blocked

#### Detection Avoidance

```rust
const BROWSER_USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
];

fn get_random_user_agent() -> &'static str {
    BROWSER_USER_AGENTS.choose(&mut rand::thread_rng()).unwrap()
}
```

#### Fallback: LinkedIn Public Profile Badge

LinkedIn offers a public profile badge that includes profile picture:

```
https://www.linkedin.com/in/{username}/overlay/photo/
```

This may be more stable but quality is limited.

#### Best Practices

1. **Cache aggressively** (7 days minimum)
2. **Implement circuit breaker** - stop if too many failures
3. **Rotate user agents** if doing multiple requests
4. **Add random delays** between requests
5. **Monitor for blocks** and back off immediately
6. **Consider this optional** - don't fail if LinkedIn unavailable

---

### Twitter/X

**Status**: ⚠️ API Access Restricted (Paid tiers required)

#### API Changes

As of 2023, Twitter significantly restricted API access:

- Free tier: Severely limited
- Basic tier: $100/month
- Pro tier: $5,000/month

#### API v2 Endpoints

**Get User by Username**:

```http
GET https://api.twitter.com/2/users/by/username/{username}
```

**Response**:

```json
{
  "data": {
    "id": "783214",
    "name": "Twitter",
    "username": "Twitter",
    "profile_image_url": "https://pbs.twimg.com/profile_images/..."
  }
}
```

#### Authentication

OAuth 2.0 Bearer Token required:

```rust
impl TwitterFetcher {
    pub fn new(bearer_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            bearer_token,
            rate_limiter: Arc::new(RateLimiter::new(300, Duration::minutes(15))),
        }
    }

    pub async fn fetch_profile(&self, username: &str) -> Result<TwitterProfile> {
        self.rate_limiter.acquire().await?;

        let url = format!("https://api.twitter.com/2/users/by/username/{}", username);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.bearer_token))
            .query(&[("user.fields", "profile_image_url,description,location")])
            .send()
            .await?
            .json::<TwitterResponse>()
            .await?;

        Ok(response.data)
    }
}
```

#### Rate Limits (Basic Tier)

- 300 requests per 15-minute window
- Monthly tweet cap: 10,000

#### Fallback: Web Scraping

If API unavailable, scraping is possible but:

- Requires handling JavaScript rendering (headless browser)
- Very fragile
- Risk of IP blocks
- Not recommended for production

```rust
// Using headless_chrome (optional dependency)
#[cfg(feature = "headless")]
pub async fn scrape_twitter_profile(username: &str) -> Result<String> {
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;

    tab.navigate_to(&format!("https://twitter.com/{}", username))?;
    tab.wait_for_element("img[alt*='profile']")?;

    let img = tab.find_element("img[alt*='profile']")?;
    let src = img.get_attribute_value("src")?.unwrap();

    Ok(src)
}
```

#### Recommendation

- **If budget allows**: Use API Basic tier
- **If no budget**: Make Twitter optional, focus on other platforms
- **For future**: Monitor API pricing changes

---

### Facebook

**Status**: ⚠️ Requires App Review

#### Graph API

Facebook provides the Graph API, but:

- Requires app registration
- Requires app review for most permissions
- Public profile access is limited
- Profile pictures require user token

#### Public Profile Picture

If you have a Facebook user ID or username:

```http
GET https://graph.facebook.com/{user-id}/picture?type=large
```

This redirects to the profile picture URL.

#### Implementation

```rust
pub struct FacebookFetcher {
    client: reqwest::Client,
    app_access_token: String,
}

impl FacebookFetcher {
    pub async fn get_profile_pic_url(&self, user_id: &str) -> Result<String> {
        let url = format!(
            "https://graph.facebook.com/{}/picture?type=large&redirect=false&access_token={}",
            user_id,
            self.app_access_token
        );

        let response: FacebookPictureResponse = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.data.url)
    }
}
```

#### Getting App Access Token

```http
GET https://graph.facebook.com/oauth/access_token
  ?client_id={app-id}
  &client_secret={app-secret}
  &grant_type=client_credentials
```

#### Challenges

1. **User ID Discovery**: Finding Facebook user ID from name/email is difficult
2. **Privacy Settings**: Many profiles are private
3. **App Review**: Required for advanced features
4. **Rate Limits**: Varies by app tier

#### Recommendation

- **Phase 1**: Skip Facebook integration
- **Phase 2**: Add if user provides Facebook user ID/username
- **Future**: Consider Graph API if app review approved

---

### Instagram

**Status**: ❌ Not Recommended

#### Why Not Instagram?

1. **No Official Public API**: Instagram deprecated public API
2. **Requires Facebook Login**: All API access through Facebook
3. **Very Restrictive**: Profile data access severely limited
4. **Frequent Blocks**: Aggressive anti-scraping measures
5. **Legal Risks**: Terms of Service explicitly prohibit scraping

#### If You Must...

**Unofficial Approaches** (use at your own risk):

1. **Instagram Basic Display API**:
   - Requires user authorization
   - Only works for user's own profile
   - Not useful for contact management

2. **Web Scraping**:
   - Extremely fragile
   - Requires JavaScript rendering
   - IP blocks are common
   - May face legal action

3. **Third-party Services**:
   - Services like Picodash, RapidAPI
   - Paid and may violate Instagram ToS
   - No guarantee of continued service

#### Recommendation

**Skip Instagram** unless:

- User manually provides Instagram username
- You find a compliant API service
- Instagram releases a public API

---

## Rate Limiting Strategy

### Implementation Architecture

```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

pub struct RateLimitManager {
    limiters: HashMap<SocialPlatform, Arc<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>>,
}

impl RateLimitManager {
    pub fn new() -> Self {
        let mut limiters = HashMap::new();

        // GitHub: 60/hour unauthenticated
        limiters.insert(
            SocialPlatform::GitHub,
            Arc::new(RateLimiter::keyed(Quota::per_hour(NonZeroU32::new(60).unwrap())))
        );

        // LinkedIn: 10/hour (conservative)
        limiters.insert(
            SocialPlatform::LinkedIn,
            Arc::new(RateLimiter::keyed(Quota::per_hour(NonZeroU32::new(10).unwrap())))
        );

        // Twitter: 300/15min
        limiters.insert(
            SocialPlatform::Twitter,
            Arc::new(RateLimiter::keyed(Quota::per_minute(NonZeroU32::new(20).unwrap())))
        );

        Self { limiters }
    }

    pub async fn acquire(&self, platform: SocialPlatform) -> Result<()> {
        if let Some(limiter) = self.limiters.get(&platform) {
            limiter.until_key_ready(&platform.to_string()).await;
            Ok(())
        } else {
            Err(Error::PlatformNotSupported(platform))
        }
    }
}
```

### Exponential Backoff

```rust
pub async fn fetch_with_retry<F, T>(
    fetch_fn: F,
    max_retries: u32,
) -> Result<T>
where
    F: Fn() -> BoxFuture<'static, Result<T>>,
{
    let mut retry_count = 0;

    loop {
        match fetch_fn().await {
            Ok(result) => return Ok(result),
            Err(e) if e.is_retryable() && retry_count < max_retries => {
                let delay = Duration::from_secs(2_u64.pow(retry_count));
                tracing::warn!("Fetch failed, retrying in {:?}: {}", delay, e);
                tokio::time::sleep(delay).await;
                retry_count += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## Error Handling

### Error Types

```rust
#[derive(Error, Debug)]
pub enum FetchError {
    #[error("Rate limit exceeded for {platform}. Retry after {retry_after:?}")]
    RateLimitExceeded {
        platform: String,
        retry_after: Duration,
    },

    #[error("Profile not found: {platform} - {username}")]
    ProfileNotFound {
        platform: String,
        username: String,
    },

    #[error("Authentication required for {platform}")]
    AuthenticationRequired {
        platform: String,
    },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Platform temporarily unavailable: {platform}")]
    PlatformUnavailable {
        platform: String,
    },

    #[error("Invalid response from {platform}: {reason}")]
    InvalidResponse {
        platform: String,
        reason: String,
    },
}

impl FetchError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            FetchError::Network(_) | FetchError::PlatformUnavailable { .. }
        )
    }

    pub fn is_rate_limit(&self) -> bool {
        matches!(self, FetchError::RateLimitExceeded { .. })
    }
}
```

---

## Testing

### Mock Responses

```rust
#[cfg(test)]
mod tests {
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_github_fetch_success() {
        let _m = mock("GET", "/users/octocat")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "login": "octocat",
                "avatar_url": "https://example.com/avatar.jpg"
            }"#)
            .create();

        let fetcher = GitHubFetcher::new_with_base_url(server_url(), None);
        let profile = fetcher.fetch_profile("octocat").await.unwrap();

        assert_eq!(profile.login, "octocat");
    }

    #[tokio::test]
    async fn test_rate_limit_handling() {
        let _m = mock("GET", "/users/octocat")
            .with_status(403)
            .with_header("x-ratelimit-remaining", "0")
            .with_header("x-ratelimit-reset", "1234567890")
            .create();

        let fetcher = GitHubFetcher::new_with_base_url(server_url(), None);
        let result = fetcher.fetch_profile("octocat").await;

        assert!(matches!(result, Err(FetchError::RateLimitExceeded { .. })));
    }
}
```

### Integration Testing

Create test accounts on each platform for safe testing:

- GitHub: Create test account
- LinkedIn: Use your own profile only
- Twitter: Test account if API available

---

## Legal & Compliance

### Terms of Service Compliance

**Before implementing any integration, review**:

- Platform Terms of Service
- Developer Agreement
- API Terms
- robots.txt
- Data Use Policy

### Key Legal Considerations

1. **GitHub**: ✅ Explicitly allows API use for applications
2. **LinkedIn**: ⚠️ ToS prohibits scraping; user agreements unclear
3. **Twitter**: ⚠️ Requires compliance with Developer Agreement
4. **Facebook**: ⚠️ Strict platform policies
5. **Instagram**: ❌ Explicitly prohibits unauthorized access

### User Consent

Always:

- Disclose what data is accessed
- Explain how data is used
- Provide opt-out mechanism
- Respect privacy settings
- Allow per-platform enabling/disabling

### GDPR Compliance

If operating in EU:

- Provide data access (export)
- Provide data deletion
- Maintain data processing records
- Get explicit consent
- Allow data portability

### Disclaimer Template

```
Profile Pulse accesses publicly available profile information from social
media platforms. By using this feature, you acknowledge that:

1. You have the right to access this information
2. This is for personal, non-commercial use
3. You will respect platform Terms of Service
4. Profile Pulse is not affiliated with these platforms
5. Service availability may vary based on platform policies

Always respect individuals' privacy and platform guidelines.
```

---

## Best Practices Summary

1. **Start with GitHub** - Most reliable and generous API
2. **Make everything optional** - App should work without any platform
3. **Cache aggressively** - Reduce API calls
4. **Fail gracefully** - Continue if one platform fails
5. **User transparency** - Show what's happening
6. **Respect privacy** - Only public profiles, with consent
7. **Monitor costs** - Track API usage if using paid tiers
8. **Keep updated** - APIs change, monitor platform updates
9. **Legal review** - Have a lawyer review if commercial
10. **Community feedback** - Listen to user concerns

---

**Document Version**: 1.0  
**Last Updated**: 2024-01-XX  
**Next Review**: Quarterly or when platform APIs change  
**Maintained By**: Platform Integration Team
