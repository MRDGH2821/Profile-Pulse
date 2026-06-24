//! Social media integration module for Profile Pulse
//!
//! Contains implementations for fetching profile pictures from
//! various social media platforms like GitHub, Twitter/X, and LinkedIn.

use crate::core::contact::{Contact, SocialPlatform};
use crate::utils::FetchError;
use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Simple rate limiter using atomics
#[derive(Clone)]
pub struct SimpleRateLimiter {
    requests: Arc<AtomicU64>,
    last_reset: Arc<Mutex<Instant>>,
    max_per_second: u64,
}

impl SimpleRateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests: Arc::new(AtomicU64::new(0)),
            last_reset: Arc::new(Mutex::new(Instant::now())),
            max_per_second: (requests_per_minute.max(1) / 60) as u64,
        }
    }

    pub async fn until_ready(&self) {
        let mut last = self.last_reset.lock().await;
        let now = Instant::now();
        
        // Reset counter every second
        if now.duration_since(*last).as_secs() >= 1 {
            self.requests.store(0, Ordering::Relaxed);
            *last = now;
        }
        
        // Wait if we've hit the limit
        while self.requests.load(Ordering::Relaxed) >= self.max_per_second {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let now = Instant::now();
            if now.duration_since(*last).as_secs() >= 1 {
                self.requests.store(0, Ordering::Relaxed);
                *last = now;
            }
        }
        
        self.requests.fetch_add(1, Ordering::Relaxed);
    }
}

/// Result type for fetch operations
pub type FetchResult<T> = std::result::Result<T, FetchError>;

/// A fetched profile picture with metadata
#[derive(Debug, Clone)]
pub struct ProfilePhoto {
    /// The image data (JPEG or PNG)
    pub data: Vec<u8>,
    /// The URL the image was fetched from
    pub source_url: String,
    /// The platform (GitHub, Twitter, etc.)
    pub platform: String,
}

/// Trait for fetching social media profile data
#[async_trait]
pub trait ProfileFetcher: Send + Sync {
    /// Fetch profile picture by URL
    async fn fetch_profile_pic(&self, url: &str) -> FetchResult<ProfilePhoto>;

    /// Get the platform this fetcher handles
    fn platform(&self) -> SocialPlatform;
}

/// GitHub profile picture fetcher
pub struct GitHubFetcher {
    client: Arc<Client>,
    rate_limiter: SimpleRateLimiter,
}

impl GitHubFetcher {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Client::new()),
            rate_limiter: SimpleRateLimiter::new(60),
        }
    }

    fn extract_username(url: &str) -> Option<&str> {
        let path = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("github.com/");
        
        if path.contains('/') {
            Some(path.split('/').next()?.trim_end_matches('/'))
        } else if !path.is_empty() {
            Some(path)
        } else {
            None
        }
    }

    async fn fetch_pic_internal(&self, url: &str) -> FetchResult<ProfilePhoto> {
        let username = Self::extract_username(url)
            .ok_or_else(|| FetchError::InvalidUrl("Invalid GitHub URL".to_string()))?;

        info!("Fetching GitHub profile picture for: {}", username);

        self.rate_limiter.until_ready().await;

        let api_url = format!("https://api.github.com/users/{}", username);
        
        let response = self.client
            .get(&api_url)
            .header("User-Agent", "Profile-Pulse/0.1.0")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(FetchError::NotFound(format!("GitHub API returned: {}", status)));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| FetchError::Parse(e.to_string()))?;

        let avatar_url = json.get("avatar_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FetchError::NotFound("No avatar_url in response".to_string()))?;

        debug!("Fetching avatar from: {}", avatar_url);
        
        let image_response = self.client
            .get(avatar_url)
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        let data = image_response
            .bytes()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?
            .to_vec();

        Ok(ProfilePhoto {
            data,
            source_url: avatar_url.to_string(),
            platform: "GitHub".to_string(),
        })
    }
}

#[async_trait]
impl ProfileFetcher for GitHubFetcher {
    async fn fetch_profile_pic(&self, url: &str) -> FetchResult<ProfilePhoto> {
        self.fetch_pic_internal(url).await
    }

    fn platform(&self) -> SocialPlatform {
        SocialPlatform::GitHub
    }
}

/// Twitter/X profile picture fetcher
pub struct TwitterFetcher {
    client: Arc<Client>,
    rate_limiter: SimpleRateLimiter,
}

impl TwitterFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; Profile-Pulse/0.1.0)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client: Arc::new(client),
            rate_limiter: SimpleRateLimiter::new(60),
        }
    }

    fn extract_username(url: &str) -> Option<&str> {
        let url = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("twitter.com/")
            .trim_start_matches("x.com/")
            .trim_start_matches("www.twitter.com/")
            .trim_start_matches("www.x.com/");
        
        url.split('/').next()
            .map(|s| s.trim_end_matches('/'))
            .filter(|s| !s.is_empty() && !s.contains('?'))
    }

    async fn fetch_pic_internal(&self, url: &str) -> FetchResult<ProfilePhoto> {
        let username = Self::extract_username(url)
            .ok_or_else(|| FetchError::InvalidUrl("Invalid Twitter URL".to_string()))?;

        info!("Fetching Twitter profile picture for: @{}", username);

        self.rate_limiter.until_ready().await;

        let profile_url = format!("https://twitter.com/{}", username);
        
        let response = self.client
            .get(&profile_url)
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| FetchError::Parse(e.to_string()))?;

        // Parse and extract immediately, don't hold document across await
        let avatar_url = extract_twitter_avatar(&html);

        let avatar_url = avatar_url
            .ok_or_else(|| FetchError::NotFound("Could not find profile image".to_string()))?;

        debug!("Found Twitter avatar: {}", avatar_url);

        let image_response = self.client
            .get(&avatar_url)
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        let data = image_response
            .bytes()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?
            .to_vec();

        let source_url = avatar_url.clone();
        
        Ok(ProfilePhoto {
            data,
            source_url,
            platform: "Twitter".to_string(),
        })
    }
}

fn extract_twitter_avatar(html: &str) -> Option<String> {
    use scraper::{Html, Selector};
    
    let document = Html::parse_document(html);
    let meta_selector = Selector::parse("meta[property='og:image']").unwrap();
    
    if let Some(element) = document.select(&meta_selector).next() {
        if let Some(url) = element.value().attr("content") {
            return Some(url.to_string());
        }
    }

    // Try script tag parsing
    for script in document.select(&Selector::parse("script").unwrap()) {
        let text = script.text().collect::<String>();
        if text.contains("profile_image_url_https") {
            if let Some(start) = text.find("profile_image_url_https") {
                let rest = &text[start..];
                if let Some(quote_start) = rest.find('"') {
                    let rest = &rest[quote_start + 1..];
                    if let Some(quote_end) = rest.find('"') {
                        let url = &rest[..quote_end];
                        if url.contains("https") && !url.ends_with("_normal.png") {
                            return Some(url.to_string());
                        }
                    }
                }
            }
        }
    }
    
    None
}

#[async_trait]
impl ProfileFetcher for TwitterFetcher {
    async fn fetch_profile_pic(&self, url: &str) -> FetchResult<ProfilePhoto> {
        self.fetch_pic_internal(url).await
    }

    fn platform(&self) -> SocialPlatform {
        SocialPlatform::Twitter
    }
}

/// LinkedIn profile picture fetcher
pub struct LinkedInFetcher {
    client: Arc<Client>,
    rate_limiter: SimpleRateLimiter,
}

impl LinkedInFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; Profile-Pulse/0.1.0)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client: Arc::new(client),
            rate_limiter: SimpleRateLimiter::new(30),
        }
    }

    fn extract_username(url: &str) -> Option<&str> {
        let url = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("linkedin.com/in/")
            .trim_start_matches("www.linkedin.com/in/");
        
        Some(url.split('/').next()?)
            .map(|s| s.trim_end_matches('/'))
            .filter(|s| !s.is_empty())
    }

    async fn fetch_pic_internal(&self, url: &str) -> FetchResult<ProfilePhoto> {
        let _username = Self::extract_username(url)
            .ok_or_else(|| FetchError::InvalidUrl("Invalid LinkedIn URL".to_string()))?;

        warn!("LinkedIn profile fetching is limited without authentication");

        self.rate_limiter.until_ready().await;

        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| FetchError::Parse(e.to_string()))?;

        let avatar_url = extract_linkedin_avatar(&html);

        if avatar_url.is_none() {
            return Err(FetchError::NotFound(
                "Could not find LinkedIn profile image (may require login)".to_string()
            ));
        }

        let avatar_url = avatar_url.unwrap();
        debug!("Found LinkedIn avatar: {}", avatar_url);

        let image_response = self.client
            .get(&avatar_url)
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        let data = image_response
            .bytes()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?
            .to_vec();

        let source_url = avatar_url.clone();
        
        Ok(ProfilePhoto {
            data,
            source_url,
            platform: "LinkedIn".to_string(),
        })
    }
}

fn extract_linkedin_avatar(html: &str) -> Option<String> {
    use scraper::{Html, Selector};
    
    let document = Html::parse_document(html);
    let meta_selector = Selector::parse("meta[property='og:image']").unwrap();
    
    if let Some(element) = document.select(&meta_selector).next() {
        return element.value().attr("content").map(|s| s.to_string());
    }
    
    None
}

#[async_trait]
impl ProfileFetcher for LinkedInFetcher {
    async fn fetch_profile_pic(&self, url: &str) -> FetchResult<ProfilePhoto> {
        self.fetch_pic_internal(url).await
    }

    fn platform(&self) -> SocialPlatform {
        SocialPlatform::LinkedIn
    }
}

/// Generic URL image fetcher
pub struct GenericFetcher {
    client: Arc<Client>,
    rate_limiter: SimpleRateLimiter,
}

impl GenericFetcher {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Client::new()),
            rate_limiter: SimpleRateLimiter::new(30),
        }
    }

    async fn fetch_pic_internal(&self, url: &str) -> FetchResult<ProfilePhoto> {
        info!("Attempting to fetch profile picture from: {}", url);

        self.rate_limiter.until_ready().await;

        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| FetchError::Parse(e.to_string()))?;

        let avatar_url = extract_generic_avatar(&html);

        if avatar_url.is_none() {
            return Err(FetchError::NotFound("No profile image found".to_string()));
        }

        let avatar_url = avatar_url.unwrap();
        let final_url = if avatar_url.starts_with("http") {
            avatar_url
        } else {
            let base = url.split('/').collect::<Vec<_>>().join("/");
            format!("{}/{}", base.trim_end_matches('/'), avatar_url)
        };

        debug!("Found profile image: {}", final_url);

        let image_response = self.client
            .get(&final_url)
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        let data = image_response
            .bytes()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?
            .to_vec();

        Ok(ProfilePhoto {
            data,
            source_url: final_url,
            platform: "Website".to_string(),
        })
    }
}

fn extract_generic_avatar(html: &str) -> Option<String> {
    use scraper::{Html, Selector};
    
    let document = Html::parse_document(html);
    let selectors = [
        "meta[property='og:image']",
        "meta[name='twitter:image']", 
        "img[class*='profile']",
        "img[class*='avatar']",
        "img[class*='photo']",
    ];

    for selector in selectors.iter() {
        if let Ok(sel) = Selector::parse(selector) {
            if let Some(element) = document.select(&sel).next() {
                if let Some(url) = element.value().attr("content") {
                    return Some(url.to_string());
                }
                if let Some(url) = element.value().attr("src") {
                    return Some(url.to_string());
                }
            }
        }
    }
    
    None
}

#[async_trait]
impl ProfileFetcher for GenericFetcher {
    async fn fetch_profile_pic(&self, url: &str) -> FetchResult<ProfilePhoto> {
        self.fetch_pic_internal(url).await
    }

    fn platform(&self) -> SocialPlatform {
        SocialPlatform::Other
    }
}

/// Google profile picture fetcher (for Google+ legacy profiles or Google Contacts)
pub struct GoogleFetcher {
    client: Arc<Client>,
    rate_limiter: SimpleRateLimiter,
}

impl GoogleFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; Profile-Pulse/0.1.0)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client: Arc::new(client),
            rate_limiter: SimpleRateLimiter::new(30),
        }
    }

    fn extract_profile_id(url: &str) -> Option<&str> {
        // Google+ legacy: plus.google.com/ profile
        let url = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("plus.google.com/")
            .trim_start_matches("www.plus.google.com/");
        
        url.split('/').next()
            .map(|s| s.trim_end_matches('/'))
            .filter(|s| !s.is_empty() && *s != "pages" && *s != "profile")
    }

    async fn fetch_pic_internal(&self, url: &str) -> FetchResult<ProfilePhoto> {
        let _profile_id = Self::extract_profile_id(url)
            .ok_or_else(|| FetchError::InvalidUrl("Invalid Google+ URL".to_string()))?;

        warn!("Google+ profiles are deprecated; may require authentication");

        self.rate_limiter.until_ready().await;

        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| FetchError::Parse(e.to_string()))?;

        let avatar_url = extract_generic_avatar(&html);

        if avatar_url.is_none() {
            return Err(FetchError::NotFound(
                "Could not find Google profile image (may require login)".to_string()
            ));
        }

        let avatar_url = avatar_url.unwrap();
        debug!("Found Google avatar: {}", avatar_url);

        let image_response = self.client
            .get(&avatar_url)
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        let data = image_response
            .bytes()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?
            .to_vec();

        Ok(ProfilePhoto {
            data,
            source_url: avatar_url.to_string(),
            platform: "Google".to_string(),
        })
    }
}

#[async_trait]
impl ProfileFetcher for GoogleFetcher {
    async fn fetch_profile_pic(&self, url: &str) -> FetchResult<ProfilePhoto> {
        self.fetch_pic_internal(url).await
    }

    fn platform(&self) -> SocialPlatform {
        SocialPlatform::Other // Google isn't in SocialPlatform enum
    }
}

/// Facebook profile picture fetcher
pub struct FacebookFetcher {
    client: Arc<Client>,
    rate_limiter: SimpleRateLimiter,
}

impl FacebookFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; Profile-Pulse/0.1.0)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client: Arc::new(client),
            rate_limiter: SimpleRateLimiter::new(30),
        }
    }

    fn extract_username(url: &str) -> Option<&str> {
        let url = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("facebook.com/")
            .trim_start_matches("www.facebook.com/")
            .trim_start_matches("fb.com/");
        
        url.split('/').next()
            .map(|s| s.trim_end_matches('/'))
            .filter(|s| !s.is_empty() && *s != "profile.php" && !s.contains('?'))
    }

    async fn fetch_pic_internal(&self, url: &str) -> FetchResult<ProfilePhoto> {
        let username = Self::extract_username(url)
            .ok_or_else(|| FetchError::InvalidUrl("Invalid Facebook URL".to_string()))?;

        warn!("Facebook profile fetching may require authentication");

        self.rate_limiter.until_ready().await;

        // Try to get the page
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| FetchError::Parse(e.to_string()))?;

        let avatar_url = extract_facebook_avatar(&html, username);

        if avatar_url.is_none() {
            return Err(FetchError::NotFound(
                "Could not find Facebook profile image (may require login)".to_string()
            ));
        }

        let avatar_url = avatar_url.unwrap();
        debug!("Found Facebook avatar: {}", avatar_url);

        let image_response = self.client
            .get(&avatar_url)
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        let data = image_response
            .bytes()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?
            .to_vec();

        Ok(ProfilePhoto {
            data,
            source_url: avatar_url.to_string(),
            platform: "Facebook".to_string(),
        })
    }
}

fn extract_facebook_avatar(html: &str, _username: &str) -> Option<String> {
    use scraper::{Html, Selector};
    
    let document = Html::parse_document(html);
    
    // Try meta og:image first
    let meta_selector = Selector::parse("meta[property='og:image']").unwrap();
    if let Some(element) = document.select(&meta_selector).next() {
        if let Some(url) = element.value().attr("content") {
            return Some(url.to_string());
        }
    }

    // Try to find profile image in the page
    let img_selectors = [
        "img[alt*='Profile Picture']",
        "image[class*='profile']",
        "img[class*='fb']",
    ];
    
    for selector in img_selectors.iter() {
        if let Ok(sel) = Selector::parse(selector) {
            if let Some(element) = document.select(&sel).next() {
                if let Some(url) = element.value().attr("src") {
                    return Some(url.to_string());
                }
            }
        }
    }
    
    None
}

#[async_trait]
impl ProfileFetcher for FacebookFetcher {
    async fn fetch_profile_pic(&self, url: &str) -> FetchResult<ProfilePhoto> {
        self.fetch_pic_internal(url).await
    }

    fn platform(&self) -> SocialPlatform {
        SocialPlatform::Facebook
    }
}

/// Instagram profile picture fetcher
pub struct InstagramFetcher {
    client: Arc<Client>,
    rate_limiter: SimpleRateLimiter,
}

impl InstagramFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; Profile-Pulse/0.1.0)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client: Arc::new(client),
            rate_limiter: SimpleRateLimiter::new(30),
        }
    }

    fn extract_username(url: &str) -> Option<&str> {
        let url = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("instagram.com/")
            .trim_start_matches("www.instagram.com/");
        
        url.split('/').next()
            .map(|s| s.trim_end_matches('/'))
            .filter(|s| !s.is_empty())
    }

    async fn fetch_pic_internal(&self, url: &str) -> FetchResult<ProfilePhoto> {
        let username = Self::extract_username(url)
            .ok_or_else(|| FetchError::InvalidUrl("Invalid Instagram URL".to_string()))?;

        info!("Fetching Instagram profile picture for: @{}", username);

        // Try public API first
        let api_url = format!("https://instagram.com/{}?__a=1", username);
        
        if let Ok(response) = self.client
            .get(&api_url)
            .header("User-Agent", "Profile-Pulse/0.1.0")
            .send()
            .await
        {
            if response.status().is_success() {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    if let Some(user) = json.get("graphql").and_then(|g| g.get("user")) {
                        if let Some(profile_pic) = user.get("profile_pic_url").and_then(|p| p.as_str()) {
                            let image_response = self.client
                                .get(profile_pic)
                                .send()
                                .await
                                .map_err(|e| FetchError::Request(e.to_string()))?;

                            let data = image_response
                                .bytes()
                                .await
                                .map_err(|e| FetchError::Request(e.to_string()))?
                                .to_vec();

                            return Ok(ProfilePhoto {
                                data,
                                source_url: profile_pic.to_string(),
                                platform: "Instagram".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Fallback to scraping
        warn!("Instagram API unavailable, trying page scrape");
        self.rate_limiter.until_ready().await;

        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| FetchError::Parse(e.to_string()))?;

        let avatar_url = extract_instagram_avatar(&html);

        if avatar_url.is_none() {
            return Err(FetchError::NotFound(
                "Could not find Instagram profile image".to_string()
            ));
        }

        let avatar_url = avatar_url.unwrap();
        debug!("Found Instagram avatar: {}", avatar_url);

        let image_response = self.client
            .get(&avatar_url)
            .send()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        let data = image_response
            .bytes()
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?
            .to_vec();

        Ok(ProfilePhoto {
            data,
            source_url: avatar_url.to_string(),
            platform: "Instagram".to_string(),
        })
    }
}

fn extract_instagram_avatar(html: &str) -> Option<String> {
    use scraper::{Html, Selector};
    
    let document = Html::parse_document(html);
    
    // Look for profile picture in meta tags
    let meta_selector = Selector::parse("meta[property='og:image']").unwrap();
    if let Some(element) = document.select(&meta_selector).next() {
        if let Some(url) = element.value().attr("content") {
            return Some(url.to_string());
        }
    }

    // Try to find profile image
    let img_selector = Selector::parse("img[class*='profile']").unwrap();
    if let Some(element) = document.select(&img_selector).next() {
        if let Some(url) = element.value().attr("src") {
            return Some(url.to_string());
        }
    }
    
    None
}

#[async_trait]
impl ProfileFetcher for InstagramFetcher {
    async fn fetch_profile_pic(&self, url: &str) -> FetchResult<ProfilePhoto> {
        self.fetch_pic_internal(url).await
    }

    fn platform(&self) -> SocialPlatform {
        SocialPlatform::Instagram
    }
}

/// Fetch profile pictures from all URLs in a contact
pub async fn fetch_contact_photos(contact: &Contact) -> Vec<FetchResult<ProfilePhoto>> {
    let github = GitHubFetcher::new();
    let twitter = TwitterFetcher::new();
    let linkedin = LinkedInFetcher::new();
    let google = GoogleFetcher::new();
    let facebook = FacebookFetcher::new();
    let instagram = InstagramFetcher::new();
    let generic = GenericFetcher::new();

    let mut results = Vec::new();

    for contact_url in &contact.urls {
        let url = &contact_url.url;
        let label = contact_url.label.as_deref().unwrap_or("");

        let result = match label.to_lowercase().as_str() {
            "github" => (&github).fetch_profile_pic(url).await,
            "twitter" | "x" => (&twitter).fetch_profile_pic(url).await,
            "linkedin" => (&linkedin).fetch_profile_pic(url).await,
            "google" | "google+" => (&google).fetch_profile_pic(url).await,
            "facebook" | "fb" => (&facebook).fetch_profile_pic(url).await,
            "instagram" | "ig" => (&instagram).fetch_profile_pic(url).await,
            _ => (&generic).fetch_profile_pic(url).await,
        };

        results.push(result);
    }

    results
}