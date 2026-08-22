use thiserror::Error;

#[derive(Debug, Error)]
pub enum HostError {
    #[error("plugin not found: {0}")]
    PluginNotFound(String),
    #[error("plugin error: {0}")]
    Plugin(#[from] profile_pulse_pic_source_plugin_api::PicSourcePluginError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("http error: {0}")]
    Http(String),
}
