use crate::error::CoreError;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BackupRef {
    pub path: PathBuf,
    pub label: String,
}

pub struct BackupService;

impl BackupService {
    pub fn new(_profiles_root: impl Into<PathBuf>) -> Self {
        Self
    }

    pub async fn snapshot_profile_before_write(
        &self,
        profile_slug: &str,
    ) -> Result<BackupRef, CoreError> {
        Ok(BackupRef {
            path: PathBuf::from(format!("opfs://profiles/{profile_slug}/backups/pending")),
            label: "pending".into(),
        })
    }

    pub async fn list_profile_backups(
        &self,
        _profile_slug: &str,
    ) -> Result<Vec<BackupRef>, CoreError> {
        Ok(vec![])
    }

    pub async fn restore_profile_backup(
        &self,
        _profile_slug: &str,
        _backup_label: &str,
    ) -> Result<(), CoreError> {
        Err(CoreError::Validation(
            "backup restore is not yet available in the web build".into(),
        ))
    }
}
