//! Contact repository for database operations
//!
//! Provides CRUD operations and queries for contacts and related data.

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;

use super::models::{ContactRow, CustomFieldRow, SocialProfileRow};
use crate::core::contact::{Contact, SocialProfile};

/// Repository for managing contacts in the database
#[derive(Debug, Clone)]
pub struct ContactRepository {
    pool: SqlitePool,
}

impl ContactRepository {
    /// Create a new ContactRepository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new contact in the database
    pub async fn create(&self, contact: &Contact) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Insert contact
        let contact_row = ContactRow::from_contact(contact);
        sqlx::query(
            r#"
            INSERT INTO contacts (id, name, email, phone, organization, title, photo_url, photo_blob, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&contact_row.id)
        .bind(&contact_row.name)
        .bind(&contact_row.email)
        .bind(&contact_row.phone)
        .bind(&contact_row.organization)
        .bind(&contact_row.title)
        .bind(&contact_row.photo_url)
        .bind(&contact_row.photo_blob)
        .bind(contact_row.created_at)
        .bind(contact_row.updated_at)
        .execute(&mut *tx)
        .await
        .context("Failed to insert contact")?;

        // Insert social profiles
        for profile in &contact.social_profiles {
            self.insert_social_profile(&mut tx, profile, &contact.id)
                .await?;
        }

        // Insert custom fields
        for (key, value) in &contact.custom_fields {
            self.insert_custom_field(&mut tx, &contact.id, key, value)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Read a contact by ID
    pub async fn read(&self, id: Uuid) -> Result<Option<Contact>> {
        let id_str = id.to_string();

        // Fetch contact
        let contact_row: Option<ContactRow> = sqlx::query_as("SELECT * FROM contacts WHERE id = ?")
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await?;

        let Some(contact_row) = contact_row else {
            return Ok(None);
        };

        // Fetch social profiles
        let profiles = self.get_social_profiles(&id).await?;

        // Fetch custom fields
        let custom_fields = self.get_custom_fields(&id).await?;

        Ok(Some(contact_row.to_contact(profiles, custom_fields)))
    }

    /// Update an existing contact
    pub async fn update(&self, contact: &Contact) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Update contact
        let contact_row = ContactRow::from_contact(contact);
        sqlx::query(
            r#"
            UPDATE contacts 
            SET name = ?, email = ?, phone = ?, organization = ?, title = ?, 
                photo_url = ?, photo_blob = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&contact_row.name)
        .bind(&contact_row.email)
        .bind(&contact_row.phone)
        .bind(&contact_row.organization)
        .bind(&contact_row.title)
        .bind(&contact_row.photo_url)
        .bind(&contact_row.photo_blob)
        .bind(contact_row.updated_at)
        .bind(&contact_row.id)
        .execute(&mut *tx)
        .await
        .context("Failed to update contact")?;

        // Delete and recreate social profiles (simpler than diff)
        sqlx::query("DELETE FROM social_profiles WHERE contact_id = ?")
            .bind(&contact_row.id)
            .execute(&mut *tx)
            .await?;

        for profile in &contact.social_profiles {
            self.insert_social_profile(&mut tx, profile, &contact.id)
                .await?;
        }

        // Delete and recreate custom fields
        sqlx::query("DELETE FROM custom_fields WHERE contact_id = ?")
            .bind(&contact_row.id)
            .execute(&mut *tx)
            .await?;

        for (key, value) in &contact.custom_fields {
            self.insert_custom_field(&mut tx, &contact.id, key, value)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Delete a contact by ID
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM contacts WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// List all contacts with optional limit and offset
    pub async fn list(&self, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<Contact>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let contact_rows: Vec<ContactRow> =
            sqlx::query_as("SELECT * FROM contacts ORDER BY name ASC LIMIT ? OFFSET ?")
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;

        let mut contacts = Vec::new();
        for row in contact_rows {
            let id = Uuid::parse_str(&row.id)?;
            let profiles = self.get_social_profiles(&id).await?;
            let custom_fields = self.get_custom_fields(&id).await?;
            contacts.push(row.to_contact(profiles, custom_fields));
        }

        Ok(contacts)
    }

    /// Search contacts by name, email, or phone
    pub async fn search(&self, query: &str) -> Result<Vec<Contact>> {
        let search_pattern = format!("%{}%", query);

        let contact_rows: Vec<ContactRow> = sqlx::query_as(
            r#"
            SELECT * FROM contacts 
            WHERE name LIKE ? OR email LIKE ? OR phone LIKE ?
            ORDER BY name ASC
            "#,
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(&self.pool)
        .await?;

        let mut contacts = Vec::new();
        for row in contact_rows {
            let id = Uuid::parse_str(&row.id)?;
            let profiles = self.get_social_profiles(&id).await?;
            let custom_fields = self.get_custom_fields(&id).await?;
            contacts.push(row.to_contact(profiles, custom_fields));
        }

        Ok(contacts)
    }

    /// Count total contacts
    pub async fn count(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM contacts")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    /// Helper: Insert a social profile within a transaction
    async fn insert_social_profile(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        profile: &SocialProfile,
        contact_id: &Uuid,
    ) -> Result<()> {
        let row = SocialProfileRow::from_social_profile(profile, contact_id);
        sqlx::query(
            r#"
            INSERT INTO social_profiles (id, contact_id, platform, username, url, profile_pic_url, verified, confidence_score, discovered_at, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&row.id)
        .bind(&row.contact_id)
        .bind(&row.platform)
        .bind(&row.username)
        .bind(&row.url)
        .bind(&row.profile_pic_url)
        .bind(row.verified)
        .bind(row.confidence_score)
        .bind(row.discovered_at)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Helper: Insert a custom field within a transaction
    async fn insert_custom_field(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        contact_id: &Uuid,
        key: &str,
        value: &str,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            r#"
            INSERT INTO custom_fields (contact_id, key, value, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(contact_id.to_string())
        .bind(key)
        .bind(value)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Helper: Get social profiles for a contact
    async fn get_social_profiles(&self, contact_id: &Uuid) -> Result<Vec<SocialProfile>> {
        let rows: Vec<SocialProfileRow> =
            sqlx::query_as("SELECT * FROM social_profiles WHERE contact_id = ?")
                .bind(contact_id.to_string())
                .fetch_all(&self.pool)
                .await?;

        Ok(rows.into_iter().map(|r| r.to_social_profile()).collect())
    }

    /// Helper: Get custom fields for a contact
    async fn get_custom_fields(&self, contact_id: &Uuid) -> Result<HashMap<String, String>> {
        let rows: Vec<CustomFieldRow> =
            sqlx::query_as("SELECT * FROM custom_fields WHERE contact_id = ?")
                .bind(contact_id.to_string())
                .fetch_all(&self.pool)
                .await?;

        Ok(rows.into_iter().map(|r| (r.key, r.value)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::contact::SocialPlatform;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        // Run migrations
        sqlx::query(include_str!("migrations/20250113_001_initial_schema.sql"))
            .execute(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn test_create_and_read_contact() {
        let pool = setup_test_db().await;
        let repo = ContactRepository::new(pool);

        let contact = Contact::builder()
            .name("Test User")
            .email("test@example.com")
            .build()
            .unwrap();

        repo.create(&contact).await.unwrap();

        let retrieved = repo.read(contact.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "Test User");
        assert_eq!(retrieved.email, Some("test@example.com".to_string()));
    }

    #[tokio::test]
    async fn test_update_contact() {
        let pool = setup_test_db().await;
        let repo = ContactRepository::new(pool);

        let mut contact = Contact::new("Original Name");
        repo.create(&contact).await.unwrap();

        contact.name = "Updated Name".to_string();
        contact.email = Some("updated@example.com".to_string());
        repo.update(&contact).await.unwrap();

        let retrieved = repo.read(contact.id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated Name");
        assert_eq!(retrieved.email, Some("updated@example.com".to_string()));
    }

    #[tokio::test]
    async fn test_delete_contact() {
        let pool = setup_test_db().await;
        let repo = ContactRepository::new(pool);

        let contact = Contact::new("To Delete");
        repo.create(&contact).await.unwrap();

        let exists = repo.read(contact.id).await.unwrap();
        assert!(exists.is_some());

        repo.delete(contact.id).await.unwrap();

        let deleted = repo.read(contact.id).await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_list_contacts() {
        let pool = setup_test_db().await;
        let repo = ContactRepository::new(pool);

        repo.create(&Contact::new("Alice")).await.unwrap();
        repo.create(&Contact::new("Bob")).await.unwrap();
        repo.create(&Contact::new("Charlie")).await.unwrap();

        let contacts = repo.list(Some(10), Some(0)).await.unwrap();
        assert_eq!(contacts.len(), 3);
        assert_eq!(contacts[0].name, "Alice"); // Sorted alphabetically
    }

    #[tokio::test]
    async fn test_search_contacts() {
        let pool = setup_test_db().await;
        let repo = ContactRepository::new(pool);

        let mut contact1 = Contact::new("John Doe");
        contact1.email = Some("john@example.com".to_string());
        repo.create(&contact1).await.unwrap();

        let mut contact2 = Contact::new("Jane Smith");
        contact2.email = Some("jane@example.com".to_string());
        repo.create(&contact2).await.unwrap();

        let results = repo.search("john").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "John Doe");

        let results = repo.search("example.com").await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_contact_with_social_profiles() {
        let pool = setup_test_db().await;
        let repo = ContactRepository::new(pool);

        let profile = SocialProfile::new(
            SocialPlatform::GitHub,
            "testuser",
            "https://github.com/testuser",
        );

        let contact = Contact::builder()
            .name("Test User")
            .social_profile(profile)
            .build()
            .unwrap();

        repo.create(&contact).await.unwrap();

        let retrieved = repo.read(contact.id).await.unwrap().unwrap();
        assert_eq!(retrieved.social_profiles.len(), 1);
        assert_eq!(
            retrieved.social_profiles[0].platform,
            SocialPlatform::GitHub
        );
        assert_eq!(retrieved.social_profiles[0].username, "testuser");
    }

    #[tokio::test]
    async fn test_count_contacts() {
        let pool = setup_test_db().await;
        let repo = ContactRepository::new(pool);

        assert_eq!(repo.count().await.unwrap(), 0);

        repo.create(&Contact::new("User 1")).await.unwrap();
        repo.create(&Contact::new("User 2")).await.unwrap();

        assert_eq!(repo.count().await.unwrap(), 2);
    }
}
