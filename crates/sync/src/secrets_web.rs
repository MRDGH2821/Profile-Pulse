use crate::error::SyncError;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::path::Path;

const STORAGE_PREFIX: &str = "profile-pulse-secret:";

fn vault_passphrase() -> Result<String, SyncError> {
    std::env::var("PROFILE_PULSE_VAULT_PASSPHRASE").map_err(|_| {
        SyncError::NotConfigured(
            "set PROFILE_PULSE_VAULT_PASSPHRASE before storing sync credentials".into(),
        )
    })
}

fn browser_storage() -> Result<web_sys::Storage, SyncError> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .ok_or_else(|| SyncError::Storage("localStorage unavailable".into()))
}

fn storage_key(key: &str) -> String {
    let safe: String = key
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    format!("{STORAGE_PREFIX}{safe}")
}

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], SyncError> {
    let params = Params::new(19 * 1024, 2, 1, Some(32))
        .map_err(|err| SyncError::Storage(err.to_string()))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut derived = [0u8; 32];
    argon
        .hash_password_into(passphrase.as_bytes(), salt, &mut derived)
        .map_err(|err| SyncError::Storage(err.to_string()))?;
    Ok(derived)
}

fn encrypt(value: &str) -> Result<String, SyncError> {
    let passphrase = vault_passphrase()?;
    let salt = uuid::Uuid::new_v4().as_bytes().to_vec();
    let key = derive_key(&passphrase, &salt)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|err| SyncError::Storage(err.to_string()))?;
    let nonce_bytes = uuid::Uuid::new_v4().as_bytes()[..12].to_vec();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, value.as_bytes())
        .map_err(|err| SyncError::Storage(err.to_string()))?;
    let payload = serde_json::json!({
        "salt": BASE64.encode(salt),
        "nonce": BASE64.encode(nonce_bytes),
        "ciphertext": BASE64.encode(ciphertext),
    });
    serde_json::to_string(&payload).map_err(|err| SyncError::Storage(err.to_string()))
}

fn decrypt(payload_text: &str) -> Result<String, SyncError> {
    let passphrase = vault_passphrase()?;
    let payload: serde_json::Value =
        serde_json::from_str(payload_text).map_err(|err| SyncError::Storage(err.to_string()))?;
    let salt = BASE64
        .decode(
            payload["salt"]
                .as_str()
                .ok_or_else(|| SyncError::Storage("missing salt".into()))?,
        )
        .map_err(|err| SyncError::Storage(err.to_string()))?;
    let nonce = BASE64
        .decode(
            payload["nonce"]
                .as_str()
                .ok_or_else(|| SyncError::Storage("missing nonce".into()))?,
        )
        .map_err(|err| SyncError::Storage(err.to_string()))?;
    let ciphertext = BASE64
        .decode(
            payload["ciphertext"]
                .as_str()
                .ok_or_else(|| SyncError::Storage("missing ciphertext".into()))?,
        )
        .map_err(|err| SyncError::Storage(err.to_string()))?;
    let key = derive_key(&passphrase, &salt)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|err| SyncError::Storage(err.to_string()))?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|err| SyncError::Storage(err.to_string()))?;
    String::from_utf8(plaintext).map_err(|err| SyncError::Storage(err.to_string()))
}
#[derive(Debug, Clone)]
pub struct SecretStore;

impl SecretStore {
    pub fn new(_data_root: impl AsRef<Path>) -> Self {
        Self
    }

    pub fn put(&self, key: &str, value: &str) -> Result<(), SyncError> {
        let storage = browser_storage()?;
        let encrypted = encrypt(value)?;
        storage
            .set_item(&storage_key(key), &encrypted)
            .map_err(|err| SyncError::Storage(format!("localStorage write failed: {err:?}")))
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, SyncError> {
        let storage = browser_storage()?;
        match storage
            .get_item(&storage_key(key))
            .map_err(|err| SyncError::Storage(format!("localStorage read failed: {err:?}")))?
        {
            Some(payload) => decrypt(&payload).map(Some),
            None => Ok(None),
        }
    }

    pub fn delete(&self, key: &str) -> Result<(), SyncError> {
        let storage = browser_storage()?;
        storage
            .remove_item(&storage_key(key))
            .map_err(|err| SyncError::Storage(format!("localStorage delete failed: {err:?}")))
    }
}
