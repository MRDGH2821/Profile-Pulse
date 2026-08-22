use thiserror::Error;

#[derive(Debug, Error)]
pub enum PicSourcePluginError {
    #[error("network error: {0}")]
    Network(String),
    #[error("not found")]
    NotFound,
    #[error("invalid candidate: {0}")]
    InvalidCandidate(String),
    #[error("capability denied: {0}")]
    CapabilityDenied(String),
    #[error("internal error: {0}")]
    Internal(String),
}
