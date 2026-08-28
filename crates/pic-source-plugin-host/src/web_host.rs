use profile_pulse_core::PicSourcePluginId;
use profile_pulse_pic_source_plugin_api::{
    PicSourceHostApi, PicSourceHostContext, PicSourcePluginError,
};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// Browser host API: HTTP via `reqwest` (fetch) and in-memory plugin cache.
#[derive(Debug)]
pub struct PluginHostApi {
    client: Client,
    cache: Mutex<HashMap<String, Vec<u8>>>,
}

impl PluginHostApi {
    pub fn new(_cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            client: Client::builder()
                .user_agent("Profile-Pulse/0.1")
                .build()
                .expect("build reqwest client"),
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn cache_key(ctx: &PicSourceHostContext, key: &str) -> String {
        format!("{}:{key}", ctx.plugin_id.0)
    }
}

#[async_trait::async_trait]
impl PicSourceHostApi for PluginHostApi {
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
        let cache = self
            .cache
            .lock()
            .map_err(|_| PicSourcePluginError::Internal("cache lock poisoned".into()))?;
        Ok(cache.get(&Self::cache_key(ctx, key)).cloned())
    }

    async fn cache_put(
        &self,
        ctx: &PicSourceHostContext,
        key: &str,
        bytes: &[u8],
    ) -> Result<(), PicSourcePluginError> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| PicSourcePluginError::Internal("cache lock poisoned".into()))?;
        cache.insert(Self::cache_key(ctx, key), bytes.to_vec());
        Ok(())
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
