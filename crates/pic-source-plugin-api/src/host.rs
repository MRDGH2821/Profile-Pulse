use profile_pulse_core::PicSourcePluginId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PicSourceCapability {
    Network,
    ReadSecrets,
}

#[derive(Debug, Clone)]
pub struct PicSourceHostContext {
    pub plugin_id: PicSourcePluginId,
}

#[async_trait::async_trait]
pub trait PicSourceHostApi: Send + Sync {
    async fn http_get(
        &self,
        ctx: &PicSourceHostContext,
        url: &str,
        headers: &[(&str, &str)],
    ) -> Result<Vec<u8>, super::PicSourcePluginError>;
    async fn get_secret(
        &self,
        ctx: &PicSourceHostContext,
        key: &str,
    ) -> Result<Option<String>, super::PicSourcePluginError>;
    async fn cache_get(
        &self,
        ctx: &PicSourceHostContext,
        key: &str,
    ) -> Result<Option<Vec<u8>>, super::PicSourcePluginError>;
    async fn cache_put(
        &self,
        ctx: &PicSourceHostContext,
        key: &str,
        bytes: &[u8],
    ) -> Result<(), super::PicSourcePluginError>;
    fn log(&self, ctx: &PicSourceHostContext, level: log::Level, message: &str);
}
