use std::path::{Path, PathBuf};

use crate::error::SyncError;

#[derive(Debug, Clone)]
pub struct SecretStore {
    root: PathBuf,
}

impl SecretStore {
    pub fn new(data_root: impl AsRef<Path>) -> Self {
        Self {
            root: data_root.as_ref().join("secrets"),
        }
    }

    fn path_for(&self, key: &str) -> PathBuf {
        let safe: String = key
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        self.root.join(format!("{safe}.secret"))
    }

    pub fn put(&self, key: &str, value: &str) -> Result<(), SyncError> {
        std::fs::create_dir_all(&self.root)
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        std::fs::write(self.path_for(key), value)
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(self.path_for(key)) {
                let mut perms = meta.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(self.path_for(key), perms);
            }
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, SyncError> {
        let path = self.path_for(key);
        if !path.exists() {
            return Ok(None);
        }
        let value = std::fs::read_to_string(path).map_err(|e| SyncError::Storage(e.to_string()))?;
        Ok(Some(value))
    }

    pub fn delete(&self, key: &str) -> Result<(), SyncError> {
        let path = self.path_for(key);
        if path.exists() {
            std::fs::remove_file(path).map_err(|e| SyncError::Storage(e.to_string()))?;
        }
        Ok(())
    }
}
