use crate::error::CoreError;
use chrono::Utc;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct BackupRef {
    pub path: PathBuf,
}

pub struct BackupService {
    profiles_root: PathBuf,
}

impl BackupService {
    pub fn new(profiles_root: impl Into<PathBuf>) -> Self {
        Self {
            profiles_root: profiles_root.into(),
        }
    }

    pub async fn snapshot_profile_before_write(
        &self,
        profile_slug: &str,
    ) -> Result<BackupRef, CoreError> {
        let source = self.profiles_root.join(profile_slug);
        if !source.exists() {
            return Ok(BackupRef {
                path: source.join("backups").join("initial"),
            });
        }

        let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ");
        let dest = source.join("backups").join(timestamp.to_string());
        copy_dir_excluding_backups(&source, &dest).await?;
        Ok(BackupRef { path: dest })
    }
}

async fn copy_dir_excluding_backups(src: &Path, dst: &Path) -> Result<(), CoreError> {
    fs::create_dir_all(dst)
        .await
        .map_err(|e| CoreError::Validation(e.to_string()))?;
    let mut entries = fs::read_dir(src)
        .await
        .map_err(|e| CoreError::Validation(e.to_string()))?;
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| CoreError::Validation(e.to_string()))?
    {
        let file_type = entry
            .file_type()
            .await
            .map_err(|e| CoreError::Validation(e.to_string()))?;
        let name = entry.file_name();
        if name == "backups" {
            continue;
        }
        let from = entry.path();
        let to = dst.join(name);
        if file_type.is_dir() {
            Box::pin(copy_dir_excluding_backups(&from, &to)).await?;
        } else {
            fs::copy(&from, &to)
                .await
                .map_err(|e| CoreError::Validation(e.to_string()))?;
        }
    }
    Ok(())
}
