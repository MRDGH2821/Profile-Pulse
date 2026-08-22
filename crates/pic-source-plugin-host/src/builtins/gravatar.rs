use std::sync::Arc;

use async_trait::async_trait;
use profile_pulse_core::PicSourcePluginId;
use profile_pulse_pic_source_plugin_api::{
    ContactContext, PicSourceCapability, PicSourceHostApi, PicSourcePluginError,
    PicSourcePluginMetadata, ProfilePicBytes, ProfilePicCandidate, ProfilePicSourcePlugin,
};
use semver::Version;

use crate::desktop_host::{guess_content_type, host_context, DesktopHostApi};

pub const PLUGIN_ID: &str = "profile-pulse.builtin.gravatar-pic-source";

pub struct GravatarPicSource {
    host: Arc<DesktopHostApi>,
}

impl GravatarPicSource {
    pub fn new(host: Arc<DesktopHostApi>) -> Self {
        Self { host }
    }

    fn gravatar_url(email: &str) -> String {
        let normalized = email.trim().to_ascii_lowercase();
        let hash = format!("{:x}", md5::compute(normalized.as_bytes()));
        format!("https://www.gravatar.com/avatar/{hash}?d=404&s=256")
    }
}

#[async_trait]
impl ProfilePicSourcePlugin for GravatarPicSource {
    fn metadata(&self) -> PicSourcePluginMetadata {
        PicSourcePluginMetadata {
            id: PicSourcePluginId(PLUGIN_ID.into()),
            name: "Gravatar".into(),
            version: Version::new(1, 0, 0),
            min_host_version: Version::new(1, 0, 0),
            website_match: vec![],
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
        for email in &ctx.emails {
            let address = email.trim();
            if address.is_empty() || !address.contains('@') {
                continue;
            }
            let url = Self::gravatar_url(address);
            candidates.push(ProfilePicCandidate {
                source_key: address.to_ascii_lowercase(),
                label: format!("Gravatar ({address})"),
                preview_url: Some(url.clone()),
                fetch_token: url,
            });
        }
        Ok(candidates)
    }

    async fn fetch_pic(
        &self,
        candidate: &ProfilePicCandidate,
    ) -> Result<ProfilePicBytes, PicSourcePluginError> {
        let ctx = host_context(PLUGIN_ID);
        let bytes = self
            .host
            .http_get(&ctx, &candidate.fetch_token, &[])
            .await?;
        Ok(ProfilePicBytes {
            content_type: guess_content_type(&bytes),
            bytes,
        })
    }
}
