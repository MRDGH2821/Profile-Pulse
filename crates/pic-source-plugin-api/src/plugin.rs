use async_trait::async_trait;

use crate::{
    ContactContext, PicSourceCapability, PicSourcePluginError, PicSourcePluginMetadata,
    ProfilePicBytes, ProfilePicCandidate,
};

#[async_trait]
pub trait ProfilePicSourcePlugin: Send + Sync {
    fn metadata(&self) -> PicSourcePluginMetadata;
    fn capabilities(&self) -> Vec<PicSourceCapability>;

    async fn discover_sources(
        &self,
        ctx: &ContactContext,
    ) -> Result<Vec<ProfilePicCandidate>, PicSourcePluginError>;

    async fn fetch_pic(
        &self,
        candidate: &ProfilePicCandidate,
    ) -> Result<ProfilePicBytes, PicSourcePluginError>;
}
