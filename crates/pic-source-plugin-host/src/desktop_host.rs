use profile_pulse_core::PicSourcePluginId;
use profile_pulse_pic_source_plugin_api::{
    PicSourceHostApi, PicSourceHostContext, PicSourcePluginError,
};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::path::PathBuf;
use tokio::fs;

/// Desktop host API: HTTP via reqwest, plugin-scoped filesystem cache.
#[derive(Debug)]
pub struct DesktopHostApi {
    cache_dir: PathBuf,
    client: Client,
}

impl DesktopHostApi {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            client: Client::builder()
                .user_agent("Profile-Pulse/0.1")
                .build()
                .expect("build reqwest client"),
        }
    }

    fn plugin_cache_dir(&self, ctx: &PicSourceHostContext) -> PathBuf {
        self.cache_dir
            .join(sanitize_path_component(&ctx.plugin_id.0))
    }

    fn cache_path(&self, ctx: &PicSourceHostContext, key: &str) -> PathBuf {
        self.plugin_cache_dir(ctx)
            .join(sanitize_path_component(key))
    }
}

fn sanitize_path_component(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[async_trait::async_trait]
impl PicSourceHostApi for DesktopHostApi {
    async fn http_get(
        &self,
        _ctx: &PicSourceHostContext,
        url: &str,
        headers: &[(&str, &str)],
    ) -> Result<Vec<u8>, PicSourcePluginError> {
        let mut header_map = HeaderMap::new();
        for (name, value) in headers {
            let name = HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| PicSourcePluginError::Internal(format!("invalid header name: {e}")))?;
            let value = HeaderValue::from_str(value).map_err(|e| {
                PicSourcePluginError::Internal(format!("invalid header value: {e}"))
            })?;
            header_map.insert(name, value);
        }
        let response = self
            .client
            .get(url)
            .headers(header_map)
            .send()
            .await
            .map_err(|e| PicSourcePluginError::Network(e.to_string()))?;
        if response.status().as_u16() == 404 {
            return Err(PicSourcePluginError::NotFound);
        }
        if !response.status().is_success() {
            return Err(PicSourcePluginError::Network(format!(
                "HTTP {}",
                response.status()
            )));
        }
        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| PicSourcePluginError::Network(e.to_string()))
    }

    async fn get_secret(
        &self,
        _ctx: &PicSourceHostContext,
        _key: &str,
    ) -> Result<Option<String>, PicSourcePluginError> {
        Ok(None)
    }

    async fn cache_get(
        &self,
        ctx: &PicSourceHostContext,
        key: &str,
    ) -> Result<Option<Vec<u8>>, PicSourcePluginError> {
        let path = self.cache_path(ctx, key);
        if !path.exists() {
            return Ok(None);
        }
        fs::read(&path)
            .await
            .map(Some)
            .map_err(|e| PicSourcePluginError::Internal(e.to_string()))
    }

    async fn cache_put(
        &self,
        ctx: &PicSourceHostContext,
        key: &str,
        bytes: &[u8],
    ) -> Result<(), PicSourcePluginError> {
        let dir = self.plugin_cache_dir(ctx);
        fs::create_dir_all(&dir)
            .await
            .map_err(|e| PicSourcePluginError::Internal(e.to_string()))?;
        fs::write(self.cache_path(ctx, key), bytes)
            .await
            .map_err(|e| PicSourcePluginError::Internal(e.to_string()))
    }

    fn log(&self, ctx: &PicSourceHostContext, level: log::Level, message: &str) {
        log::log!(target: "pic_source_plugin", level, "[{}] {message}", ctx.plugin_id.0);
    }
}

pub fn host_context(plugin_id: &str) -> PicSourceHostContext {
    PicSourceHostContext {
        plugin_id: PicSourcePluginId(plugin_id.to_string()),
    }
}

pub fn guess_content_type(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg".into()
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png".into()
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        "image/gif".into()
    } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp".into()
    } else {
        "application/octet-stream".into()
    }
}
