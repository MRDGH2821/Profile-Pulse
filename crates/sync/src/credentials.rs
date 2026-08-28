use profile_pulse_core::ProfileId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushResult {
    pub target_kind: String,
    pub remote_id: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CardDavCredentials {
    pub username: String,
    pub password: String,
}

pub fn carddav_secret_key(profile_id: ProfileId) -> String {
    format!("carddav:{}", profile_id.0)
}
