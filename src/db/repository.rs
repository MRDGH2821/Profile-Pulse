//! Contact repository for database operations
//!
//! Provides CRUD operations and queries for contacts and related data.

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;

use super::models::{
    ContactAddressRow, ContactDateRow, ContactEmailRow, ContactPhoneRow, ContactRow,
    ContactUrlRow, CustomFieldRow, SocialProfileRow,
};
use crate::core::contact::{
    Contact, ContactAddress, ContactDate, ContactEmail, ContactPhone, ContactUrl, SocialProfile,
};

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
            INSERT INTO contacts (id, name, name_prefix, first_name, middle_name, last_name, name_suffix, nickname, notes, email, phone, organization, title, department, photo_url, photo_blob, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&contact_row.id)
        .bind(&contact_row.name)
        .bind(&contact_row.name_prefix)
        .bind(&contact_row.first_name)
        .bind(&contact_row.middle_name)
        .bind(&contact_row.last_name)
        .bind(&contact_row.name_suffix)
        .bind(&contact_row.nickname)
        .bind(&contact_row.notes)
        .bind(&contact_row.email)
        .bind(&contact_row.phone)
        .bind(&contact_row.organization)
        .bind(&contact_row.title)
        .bind(&contact_row.department)
        .bind(&contact_row.photo_url)
        .bind(&contact_row.photo_blob)
        .bind(contact_row.created_at)
        .bind(contact_row.updated_at)
        .execute(&mut *tx)
        .await
        .context("Failed to insert contact")?;

        // Insert contact emails
        for email in &contact.emails {
            self.insert_contact_email(&mut tx, email, &contact.id)
                .await?;
        }

        // Insert contact phones
        for phone in &contact.phones {
            self.insert_contact_phone(&mut tx, phone, &contact.id)
                .await?;
        }

        // Insert contact addresses
        for address in &contact.addresses {
            self.insert_contact_address(&mut tx, address, &contact.id)
                .await?;
        }

        // Insert contact dates
        for date in &contact.dates {
            self.insert_contact_date(&mut tx, date, &contact.id)
                .await?;
        }

        // Insert contact URLs
        for url in &contact.urls {
            self.insert_contact_url(&mut tx, url, &contact.id)
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

        // Fetch structured fields
        let emails = self.get_contact_emails(&id).await?;
        let phones = self.get_contact_phones(&id).await?;
        let addresses = self.get_contact_addresses(&id).await?;
        let dates = self.get_contact_dates(&id).await?;
        let urls = self.get_contact_urls(&id).await?;

        // Fetch custom fields
        let custom_fields = self.get_custom_fields(&id).await?;

        let mut contact = contact_row.to_contact(urls, custom_fields);
        contact.emails = emails;
        contact.phones = phones;
        contact.addresses = addresses;
        contact.dates = dates;

        Ok(Some(contact))
    }

    /// Update an existing contact
    pub async fn update(&self, contact: &Contact) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Update contact
        let contact_row = ContactRow::from_contact(contact);
        sqlx::query(
            r#"
            UPDATE contacts 
            SET name = ?, name_prefix = ?, first_name = ?, middle_name = ?, last_name = ?, 
                name_suffix = ?, nickname = ?, notes = ?, email = ?, phone = ?, 
                organization = ?, title = ?, department = ?, photo_url = ?, photo_blob = ?, 
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&contact_row.name)
        .bind(&contact_row.name_prefix)
        .bind(&contact_row.first_name)
        .bind(&contact_row.middle_name)
        .bind(&contact_row.last_name)
        .bind(&contact_row.name_suffix)
        .bind(&contact_row.nickname)
        .bind(&contact_row.notes)
        .bind(&contact_row.email)
        .bind(&contact_row.phone)
        .bind(&contact_row.organization)
        .bind(&contact_row.title)
        .bind(&contact_row.department)
        .bind(&contact_row.photo_url)
        .bind(&contact_row.photo_blob)
        .bind(contact_row.updated_at)
        .bind(&contact_row.id)
        .execute(&mut *tx)
        .await
        .context("Failed to update contact")?;

        // Delete and recreate contact emails (simpler than diff)
        sqlx::query("DELETE FROM contact_emails WHERE contact_id = ?")
            .bind(&contact_row.id)
            .execute(&mut *tx)
            .await?;

        for email in &contact.emails {
            self.insert_contact_email(&mut tx, email, &contact.id)
                .await?;
        }

        // Delete and recreate contact phones
        sqlx::query("DELETE FROM contact_phones WHERE contact_id = ?")
            .bind(&contact_row.id)
            .execute(&mut *tx)
            .await?;

        for phone in &contact.phones {
            self.insert_contact_phone(&mut tx, phone, &contact.id)
                .await?;
        }

        // Delete and recreate contact addresses
        sqlx::query("DELETE FROM contact_addresses WHERE contact_id = ?")
            .bind(&contact_row.id)
            .execute(&mut *tx)
            .await?;

        for address in &contact.addresses {
            self.insert_contact_address(&mut tx, address, &contact.id)
                .await?;
        }

        // Delete and recreate contact dates
        sqlx::query("DELETE FROM contact_dates WHERE contact_id = ?")
            .bind(&contact_row.id)
            .execute(&mut *tx)
            .await?;

        for date in &contact.dates {
            self.insert_contact_date(&mut tx, date, &contact.id)
                .await?;
        }

        // Delete and recreate contact URLs (simpler than diff)
        sqlx::query("DELETE FROM contact_urls WHERE contact_id = ?")
            .bind(&contact_row.id)
            .execute(&mut *tx)
            .await?;

        for url in &contact.urls {
            self.insert_contact_url(&mut tx, url, &contact.id)
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
            let emails = self.get_contact_emails(&id).await?;
            let phones = self.get_contact_phones(&id).await?;
            let addresses = self.get_contact_addresses(&id).await?;
            let dates = self.get_contact_dates(&id).await?;
            let urls = self.get_contact_urls(&id).await?;
            let custom_fields = self.get_custom_fields(&id).await?;
            
            let mut contact = row.to_contact(urls, custom_fields);
            contact.emails = emails;
            contact.phones = phones;
            contact.addresses = addresses;
            contact.dates = dates;
            contacts.push(contact);
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
            let emails = self.get_contact_emails(&id).await?;
            let phones = self.get_contact_phones(&id).await?;
            let addresses = self.get_contact_addresses(&id).await?;
            let dates = self.get_contact_dates(&id).await?;
            let urls = self.get_contact_urls(&id).await?;
            let custom_fields = self.get_custom_fields(&id).await?;
            
            let mut contact = row.to_contact(urls, custom_fields);
            contact.emails = emails;
            contact.phones = phones;
            contact.addresses = addresses;
            contact.dates = dates;
            contacts.push(contact);
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

    /// Helper: Insert a contact email within a transaction
    async fn insert_contact_email(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        email: &ContactEmail,
        contact_id: &Uuid,
    ) -> Result<()> {
        let row = ContactEmailRow::from_contact_email(email, contact_id);
        sqlx::query(
            r#"
            INSERT INTO contact_emails (id, contact_id, email, label, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&row.id)
        .bind(&row.contact_id)
        .bind(&row.email)
        .bind(&row.label)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Helper: Insert a contact phone within a transaction
    async fn insert_contact_phone(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        phone: &ContactPhone,
        contact_id: &Uuid,
    ) -> Result<()> {
        let row = ContactPhoneRow::from_contact_phone(phone, contact_id);
        sqlx::query(
            r#"
            INSERT INTO contact_phones (id, contact_id, phone, label, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&row.id)
        .bind(&row.contact_id)
        .bind(&row.phone)
        .bind(&row.label)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Helper: Insert a contact address within a transaction
    async fn insert_contact_address(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        address: &ContactAddress,
        contact_id: &Uuid,
    ) -> Result<()> {
        let row = ContactAddressRow::from_contact_address(address, contact_id);
        sqlx::query(
            r#"
            INSERT INTO contact_addresses (id, contact_id, street, city, state, postal_code, country, label, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&row.id)
        .bind(&row.contact_id)
        .bind(&row.street)
        .bind(&row.city)
        .bind(&row.state)
        .bind(&row.postal_code)
        .bind(&row.country)
        .bind(&row.label)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Helper: Insert a contact date within a transaction
    async fn insert_contact_date(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        date: &ContactDate,
        contact_id: &Uuid,
    ) -> Result<()> {
        let row = ContactDateRow::from_contact_date(date, contact_id);
        sqlx::query(
            r#"
            INSERT INTO contact_dates (id, contact_id, date, label, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&row.id)
        .bind(&row.contact_id)
        .bind(&row.date)
        .bind(&row.label)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Helper: Insert a contact URL within a transaction
    async fn insert_contact_url(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        url: &ContactUrl,
        contact_id: &Uuid,
    ) -> Result<()> {
        let row = ContactUrlRow::from_contact_url(url, contact_id);
        sqlx::query(
            r#"
            INSERT INTO contact_urls (id, contact_id, url, label, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&row.id)
        .bind(&row.contact_id)
        .bind(&row.url)
        .bind(&row.label)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Helper: Insert a social profile within a transaction (for profile_cache)
    async fn insert_social_profile(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        profile: &SocialProfile,
        contact_id: &Uuid,
    ) -> Result<()> {
        let row = SocialProfileRow::from_social_profile(profile, contact_id);
        sqlx::query(
            r#"
            INSERT INTO profile_cache (id, contact_id, platform, username, url, profile_pic_url, verified, confidence_score, discovered_at, created_at, updated_at)
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

    /// Helper: Get contact emails for a contact
    async fn get_contact_emails(&self, contact_id: &Uuid) -> Result<Vec<ContactEmail>> {
        let rows: Vec<ContactEmailRow> =
            sqlx::query_as("SELECT * FROM contact_emails WHERE contact_id = ?")
                .bind(contact_id.to_string())
                .fetch_all(&self.pool)
                .await?;

        Ok(rows.into_iter().map(|r| r.to_contact_email()).collect())
    }

    /// Helper: Get contact phones for a contact
    async fn get_contact_phones(&self, contact_id: &Uuid) -> Result<Vec<ContactPhone>> {
        let rows: Vec<ContactPhoneRow> =
            sqlx::query_as("SELECT * FROM contact_phones WHERE contact_id = ?")
                .bind(contact_id.to_string())
                .fetch_all(&self.pool)
                .await?;

        Ok(rows.into_iter().map(|r| r.to_contact_phone()).collect())
    }

    /// Helper: Get contact addresses for a contact
    async fn get_contact_addresses(&self, contact_id: &Uuid) -> Result<Vec<ContactAddress>> {
        let rows: Vec<ContactAddressRow> =
            sqlx::query_as("SELECT * FROM contact_addresses WHERE contact_id = ?")
                .bind(contact_id.to_string())
                .fetch_all(&self.pool)
                .await?;

        Ok(rows.into_iter().map(|r| r.to_contact_address()).collect())
    }

    /// Helper: Get contact dates for a contact
    async fn get_contact_dates(&self, contact_id: &Uuid) -> Result<Vec<ContactDate>> {
        let rows: Vec<ContactDateRow> =
            sqlx::query_as("SELECT * FROM contact_dates WHERE contact_id = ?")
                .bind(contact_id.to_string())
                .fetch_all(&self.pool)
                .await?;

        let mut dates = Vec::new();
        for row in rows {
            if let Ok(date) = row.to_contact_date() {
                dates.push(date);
            }
        }
        Ok(dates)
    }

    /// Helper: Get contact URLs for a contact
    async fn get_contact_urls(&self, contact_id: &Uuid) -> Result<Vec<ContactUrl>> {
        let rows: Vec<ContactUrlRow> =
            sqlx::query_as("SELECT * FROM contact_urls WHERE contact_id = ?")
                .bind(contact_id.to_string())
                .fetch_all(&self.pool)
                .await?;

        Ok(rows.into_iter().map(|r| r.to_contact_url()).collect())
    }

    /// Helper: Get social profiles for a contact (for profile_cache)
    async fn get_social_profiles(&self, contact_id: &Uuid) -> Result<Vec<SocialProfile>> {
        let rows: Vec<SocialProfileRow> =
            sqlx::query_as("SELECT * FROM profile_cache WHERE contact_id = ?")
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
    use chrono::{Datelike, NaiveDate};

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        // Run migrations
        sqlx::query(include_str!("migrations/20250113_001_initial_schema.sql"))
            .execute(&pool)
            .await
            .expect("Failed to run migration 1");

        sqlx::query(include_str!("migrations/20250114_002_add_urls_table.sql"))
            .execute(&pool)
            .await
            .expect("Failed to run migration 2");

        sqlx::query(include_str!("migrations/20250115_001_add_structured_fields.sql"))
            .execute(&pool)
            .await
            .expect("Failed to run migration 3");

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
    async fn test_contact_with_urls() {
        let pool = setup_test_db().await;
        let repo = ContactRepository::new(pool);

        // Create contact with URLs
        let url1 = ContactUrl::new("https://github.com/testuser", Some("GitHub".to_string()));
        let url2 = ContactUrl::new("https://myblog.com", Some("Blog".to_string()));

        let contact = Contact::builder()
            .name("Test User")
            .url(url1)
            .url(url2)
            .build()
            .unwrap();

        repo.create(&contact).await.unwrap();

        let retrieved = repo.read(contact.id).await.unwrap().unwrap();
        assert_eq!(retrieved.urls.len(), 2);
        
        let github_urls = retrieved.find_urls_by_label("GitHub");
        assert_eq!(github_urls.len(), 1);
        assert_eq!(github_urls[0].url, "https://github.com/testuser");
        
        let blog_urls = retrieved.find_urls_by_label("Blog");
        assert_eq!(blog_urls.len(), 1);
        assert_eq!(blog_urls[0].url, "https://myblog.com");
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

    #[tokio::test]
    async fn test_contact_with_structured_fields() {
        let pool = setup_test_db().await;
        let repo = ContactRepository::new(pool);

        // Create contact with all structured fields
        let email1 = ContactEmail::new("home@example.com", "Home");
        let email2 = ContactEmail::new("work@example.com", "Work");
        let phone1 = ContactPhone::new("+1234567890", "Mobile");
        let phone2 = ContactPhone::new("+9876543210", "Home");
        let address = ContactAddress::builder()
            .street("123 Main St")
            .city("Springfield")
            .state("IL")
            .postal_code("62701")
            .label("Home")
            .build();
        let date = ContactDate::new(
            NaiveDate::from_ymd_opt(1990, 5, 15).unwrap(),
            "Birthday",
        );

        let contact = Contact::builder()
            .name("Test User")
            .email_entry(email1)
            .email_entry(email2)
            .phone_entry(phone1)
            .phone_entry(phone2)
            .address(address)
            .date(date)
            .build()
            .unwrap();

        repo.create(&contact).await.unwrap();

        let retrieved = repo.read(contact.id).await.unwrap().unwrap();
        
        // Verify emails
        assert_eq!(retrieved.emails.len(), 2);
        assert_eq!(retrieved.emails[0].email, "home@example.com");
        assert_eq!(retrieved.emails[0].label, "Home");
        assert_eq!(retrieved.emails[1].email, "work@example.com");
        assert_eq!(retrieved.emails[1].label, "Work");
        
        // Verify phones
        assert_eq!(retrieved.phones.len(), 2);
        assert_eq!(retrieved.phones[0].phone, "+1234567890");
        assert_eq!(retrieved.phones[0].label, "Mobile");
        
        // Verify addresses
        assert_eq!(retrieved.addresses.len(), 1);
        assert_eq!(retrieved.addresses[0].street, Some("123 Main St".to_string()));
        assert_eq!(retrieved.addresses[0].city, Some("Springfield".to_string()));
        assert_eq!(retrieved.addresses[0].label, "Home");
        
        // Verify dates
        assert_eq!(retrieved.dates.len(), 1);
        assert_eq!(retrieved.dates[0].date.year(), 1990);
        assert_eq!(retrieved.dates[0].label, "Birthday");
    }

    #[tokio::test]
    async fn test_update_structured_fields() {
        let pool = setup_test_db().await;
        let repo = ContactRepository::new(pool);

        // Create contact with one email
        let mut contact = Contact::builder()
            .name("Test User")
            .email_entry(ContactEmail::new("old@example.com", "Home"))
            .build()
            .unwrap();

        repo.create(&contact).await.unwrap();

        // Update with different emails
        contact.emails.clear();
        contact.emails.push(ContactEmail::new("new1@example.com", "Home"));
        contact.emails.push(ContactEmail::new("new2@example.com", "Work"));

        repo.update(&contact).await.unwrap();

        let retrieved = repo.read(contact.id).await.unwrap().unwrap();
        assert_eq!(retrieved.emails.len(), 2);
        assert_eq!(retrieved.emails[0].email, "new1@example.com");
        assert_eq!(retrieved.emails[1].email, "new2@example.com");
    }
}
