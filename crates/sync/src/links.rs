use crate::error::SyncError;
use chrono::{DateTime, Utc};
use profile_pulse_core::{ContactId, ProfileId};
use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SyncLinkStore {
    conn: Arc<Mutex<Connection>>,
}

impl SyncLinkStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, SyncError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| SyncError::Storage(e.to_string()))?;
        }
        let conn =
            Connection::open(path.as_ref()).map_err(|e| SyncError::Storage(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sync_links (
                profile_id TEXT NOT NULL,
                contact_id TEXT NOT NULL,
                target_kind TEXT NOT NULL,
                remote_id TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (profile_id, contact_id, target_kind)
            );",
        )
        .map_err(|e| SyncError::Storage(e.to_string()))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn get_remote_id(
        &self,
        profile_id: ProfileId,
        contact_id: ContactId,
        target_kind: &str,
    ) -> Result<Option<String>, SyncError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT remote_id FROM sync_links
                 WHERE profile_id = ?1 AND contact_id = ?2 AND target_kind = ?3",
            )
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query(params![
                profile_id.0.to_string(),
                contact_id.0.to_string(),
                target_kind
            ])
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| SyncError::Storage(e.to_string()))? {
            let remote_id: String = row.get(0).map_err(|e| SyncError::Storage(e.to_string()))?;
            return Ok(Some(remote_id));
        }
        Ok(None)
    }

    pub fn upsert_link(
        &self,
        profile_id: ProfileId,
        contact_id: ContactId,
        target_kind: &str,
        remote_id: &str,
        updated_at: DateTime<Utc>,
    ) -> Result<(), SyncError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT INTO sync_links (profile_id, contact_id, target_kind, remote_id, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(profile_id, contact_id, target_kind) DO UPDATE SET
               remote_id = excluded.remote_id,
               updated_at = excluded.updated_at",
            params![
                profile_id.0.to_string(),
                contact_id.0.to_string(),
                target_kind,
                remote_id,
                updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| SyncError::Storage(e.to_string()))?;
        Ok(())
    }
}
