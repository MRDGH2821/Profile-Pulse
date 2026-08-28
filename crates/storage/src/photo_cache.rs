use crate::error::StorageError;
use std::path::{Path, PathBuf};

pub fn photo_cache_dir(data_root: &Path) -> PathBuf {
    data_root.join("photo-cache")
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn store_photo(data_root: &Path, hash: &str, bytes: &[u8]) -> Result<(), StorageError> {
    let dir = photo_cache_dir(data_root);
    tokio::fs::create_dir_all(&dir).await?;
    tokio::fs::write(dir.join(hash), bytes).await?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn store_photo(_data_root: &Path, hash: &str, bytes: &[u8]) -> Result<(), StorageError> {
    crate::opfs::vfs::ensure_dir("photo-cache")
        .await
        .map_err(StorageError::Web)?;
    crate::opfs::vfs::write_bytes(&format!("photo-cache/{hash}"), bytes)
        .await
        .map_err(StorageError::Web)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_photo(data_root: &Path, hash: &str) -> Result<Vec<u8>, StorageError> {
    let path = photo_cache_dir(data_root).join(hash);
    if !path.exists() {
        return Err(StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("photo not found: {hash}"),
        )));
    }
    Ok(tokio::fs::read(path).await?)
}

#[cfg(target_arch = "wasm32")]
pub async fn load_photo(_data_root: &Path, hash: &str) -> Result<Vec<u8>, StorageError> {
    crate::opfs::vfs::read_bytes(&format!("photo-cache/{hash}"))
        .await
        .map_err(StorageError::Web)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_is_stable() {
        assert_eq!(
            sha256_hex(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}
