use crate::error::CoreError;
use chrono::Utc;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct BackupRef {
    pub path: PathBuf,
    pub label: String,
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

    pub fn profile_dir(&self, profile_slug: &str) -> PathBuf {
        self.profiles_root.join(profile_slug)
    }

    pub async fn snapshot_profile_before_write(
        &self,
        profile_slug: &str,
    ) -> Result<BackupRef, CoreError> {
        let source = self.profile_dir(profile_slug);
        if !source.exists() {
            return Ok(BackupRef {
                path: source.join("backups").join("initial"),
                label: "initial".into(),
            });
        }

        let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ");
        let label = timestamp.to_string();
        let dest = source.join("backups").join(&label);
        copy_dir_excluding_backups(&source, &dest).await?;
        Ok(BackupRef { path: dest, label })
    }

    pub async fn list_profile_backups(
        &self,
        profile_slug: &str,
    ) -> Result<Vec<BackupRef>, CoreError> {
        let backups_dir = self.profile_dir(profile_slug).join("backups");
        if !backups_dir.exists() {
            return Ok(vec![]);
        }

        let mut refs = Vec::new();
        let mut entries = fs::read_dir(&backups_dir)
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
            if !file_type.is_dir() {
                continue;
            }
            let label = entry.file_name().to_string_lossy().into_owned();
            refs.push(BackupRef {
                path: entry.path(),
                label,
            });
        }
        refs.sort_by(|a, b| b.label.cmp(&a.label));
        Ok(refs)
    }

    pub async fn restore_profile_backup(
        &self,
        profile_slug: &str,
        backup_label: &str,
    ) -> Result<(), CoreError> {
        let profile_dir = self.profile_dir(profile_slug);
        let backup_dir = profile_dir.join("backups").join(backup_label);
        if !backup_dir.is_dir() {
            return Err(CoreError::Validation(format!(
                "backup not found: {backup_label}"
            )));
        }

        let _ = self.snapshot_profile_before_write(profile_slug).await?;

        let mut entries = fs::read_dir(&profile_dir)
            .await
            .map_err(|e| CoreError::Validation(e.to_string()))?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| CoreError::Validation(e.to_string()))?
        {
            let name = entry.file_name();
            if name == "backups" {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(&path)
                    .await
                    .map_err(|e| CoreError::Validation(e.to_string()))?;
            } else {
                fs::remove_file(&path)
                    .await
                    .map_err(|e| CoreError::Validation(e.to_string()))?;
            }
        }

        copy_dir_excluding_backups(&backup_dir, &profile_dir).await?;
        Ok(())
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
