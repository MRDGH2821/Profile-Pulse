use crate::error::StorageError;
use crate::traits::ContactIndex;
use async_trait::async_trait;
use profile_pulse_core::{Contact, ContactId, ProfileId};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SqliteContactIndex {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteContactIndex {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path.as_ref())
            .map_err(|e| StorageError::Database(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS contacts (
                profile_id TEXT NOT NULL,
                contact_id TEXT NOT NULL,
                display_name TEXT NOT NULL,
                search_text TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (profile_id, contact_id)
            );
            CREATE INDEX IF NOT EXISTS idx_contacts_search ON contacts(profile_id, search_text);",
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn with_conn<F, T>(&self, f: F) -> Result<T, StorageError>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        f(&conn).map_err(|e| StorageError::Database(e.to_string()))
    }
}

fn build_search_text(contact: &Contact) -> String {
    let mut parts = vec![contact.display_name.clone()];
    parts.extend(contact.emails.iter().map(|e| e.address.clone()));
    parts.extend(contact.phones.iter().map(|p| p.number.clone()));
    parts.extend(contact.websites.iter().map(|w| w.url.clone()));
    parts.join(" ").to_ascii_lowercase()
}

#[async_trait]
impl ContactIndex for SqliteContactIndex {
    async fn upsert_contact(&self, contact: &Contact) -> Result<(), StorageError> {
        let contact = contact.clone();
        let this = self.clone();
        tokio::task::spawn_blocking(move || {
            this.with_conn(|conn| {
                conn.execute(
                    "INSERT INTO contacts (profile_id, contact_id, display_name, search_text, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(profile_id, contact_id) DO UPDATE SET
                       display_name = excluded.display_name,
                       search_text = excluded.search_text,
                       updated_at = excluded.updated_at",
                    params![
                        contact.profile_id.0.to_string(),
                        contact.id.0.to_string(),
                        contact.display_name,
                        build_search_text(&contact),
                        contact.updated_at.to_rfc3339(),
                    ],
                )?;
                Ok(())
            })
        })
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?
    }

    async fn remove_contact(
        &self,
        profile_id: ProfileId,
        id: ContactId,
    ) -> Result<(), StorageError> {
        let this = self.clone();
        tokio::task::spawn_blocking(move || {
            this.with_conn(|conn| {
                conn.execute(
                    "DELETE FROM contacts WHERE profile_id = ?1 AND contact_id = ?2",
                    params![profile_id.0.to_string(), id.0.to_string()],
                )?;
                Ok(())
            })
        })
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?
    }

    async fn search(
        &self,
        profile_id: ProfileId,
        query: &str,
        limit: u32,
    ) -> Result<Vec<ContactId>, StorageError> {
        let this = self.clone();
        let q = format!("%{}%", query.to_ascii_lowercase());
        tokio::task::spawn_blocking(move || {
            this.with_conn(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT contact_id FROM contacts
                     WHERE profile_id = ?1 AND search_text LIKE ?2
                     ORDER BY display_name ASC
                     LIMIT ?3",
                )?;
                let rows = stmt.query_map(
                    params![profile_id.0.to_string(), q, limit],
                    |row| row.get::<_, String>(0),
                )?;
                let mut ids = Vec::new();
                for row in rows {
                    let id_str = row?;
                    if let Ok(uuid) = uuid::Uuid::parse_str(&id_str) {
                        ids.push(ContactId(uuid));
                    }
                }
                Ok(ids)
            })
        })
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?
    }
}
