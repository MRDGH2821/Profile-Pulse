use std::path::{Path, PathBuf};

use crate::error::StorageError;

pub fn photo_cache_dir(data_root: &Path) -> PathBuf {
    data_root.join("photo-cache")
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

pub async fn store_photo(data_root: &Path, hash: &str, bytes: &[u8]) -> Result<(), StorageError> {
    let dir = photo_cache_dir(data_root);
    tokio::fs::create_dir_all(&dir).await?;
    tokio::fs::write(dir.join(hash), bytes).await?;
    Ok(())
}

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
