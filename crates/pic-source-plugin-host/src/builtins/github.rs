use crate::builtins::github_username_from_url;
use crate::desktop_host::{DesktopHostApi, guess_content_type, host_context};
use async_trait::async_trait;
use profile_pulse_core::PicSourcePluginId;
use profile_pulse_pic_source_plugin_api::{
    ContactContext, PicSourceCapability, PicSourceHostApi, PicSourcePluginError,
    PicSourcePluginMetadata, ProfilePicBytes, ProfilePicCandidate, ProfilePicSourcePlugin,
};
use semver::Version;
use std::collections::HashSet;
use std::sync::Arc;

pub const PLUGIN_ID: &str = "profile-pulse.builtin.github-pic-source";

pub struct GithubPicSource {
    host: Arc<DesktopHostApi>,
}

impl GithubPicSource {
    pub fn new(host: Arc<DesktopHostApi>) -> Self {
        Self { host }
    }

    fn avatar_url(username: &str) -> String {
        format!("https://github.com/{username}.png?size=256")
    }

    fn collect_usernames(ctx: &ContactContext) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut usernames = Vec::new();
        for site in &ctx.websites {
            if let Some(user) = github_username_from_url(&site.url) {
                if seen.insert(user.clone()) {
                    usernames.push(user);
                }
            }
        }
        usernames
    }
}

#[async_trait]
impl ProfilePicSourcePlugin for GithubPicSource {
    fn metadata(&self) -> PicSourcePluginMetadata {
        PicSourcePluginMetadata {
            id: PicSourcePluginId(PLUGIN_ID.into()),
            name: "GitHub".into(),
            version: Version::new(1, 0, 0),
            min_host_version: Version::new(1, 0, 0),
            website_match: vec!["github.com".into()],
        }
    }

    fn capabilities(&self) -> Vec<PicSourceCapability> {
        vec![PicSourceCapability::Network]
    }

    async fn discover_sources(
        &self,
        ctx: &ContactContext,
    ) -> Result<Vec<ProfilePicCandidate>, PicSourcePluginError> {
        let mut candidates = Vec::new();
        for username in Self::collect_usernames(ctx) {
            let url = Self::avatar_url(&username);
            candidates.push(ProfilePicCandidate {
                source_key: username.clone(),
                label: format!("GitHub (@{username})"),
                preview_url: Some(url.clone()),
                fetch_token: format!("github:{username}"),
            });
        }
        Ok(candidates)
    }

    async fn fetch_pic(
        &self,
        candidate: &ProfilePicCandidate,
    ) -> Result<ProfilePicBytes, PicSourcePluginError> {
        let username = candidate
            .fetch_token
            .strip_prefix("github:")
            .ok_or_else(|| {
                PicSourcePluginError::InvalidCandidate("expected github:username".into())
            })?;
        let url = Self::avatar_url(username);
        let ctx = host_context(PLUGIN_ID);
        let bytes = self.host.http_get(&ctx, &url, &[]).await?;
        Ok(ProfilePicBytes {
            content_type: guess_content_type(&bytes),
            bytes,
        })
    }
}

/// Discover a GitHub avatar for a manually entered username (pic selector
/// convenience).
pub fn github_candidate_for_username(username: &str) -> Option<ProfilePicCandidate> {
    let user = username.trim().trim_start_matches('@');
    if user.is_empty() || user.contains('/') {
        return None;
    }
    let url = GithubPicSource::avatar_url(user);
    Some(ProfilePicCandidate {
        source_key: user.to_string(),
        label: format!("GitHub (@{user})"),
        preview_url: Some(url),
        fetch_token: format!("github:{user}"),
    })
}
