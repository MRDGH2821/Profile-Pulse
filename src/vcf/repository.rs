//! VCF-based repository for contact management
//!
//! This module provides a repository that reads and writes contacts directly
//! to VCF files, without any database caching layer.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::core::contact::Contact;
use crate::vcf::{export_contacts_to_vcf, import_contacts_from_vcf};

/// Repository for managing contacts stored in VCF files
#[derive(Debug, Clone)]
pub struct VcfRepository {
    /// Path to the VCF file
    vcf_path: PathBuf,
    /// In-memory cache of contacts (loaded on demand)
    contacts: HashMap<Uuid, Contact>,
    /// Whether the contacts have been loaded from disk
    loaded: bool,
}

impl VcfRepository {
    /// Create a new VcfRepository for the given VCF file path
    pub fn new(vcf_path: impl Into<PathBuf>) -> Self {
        Self {
            vcf_path: vcf_path.into(),
            contacts: HashMap::new(),
            loaded: false,
        }
    }

    /// Ensure contacts are loaded from disk
    fn ensure_loaded(&mut self) -> Result<()> {
        if self.loaded {
            return Ok(());
        }

        // If file doesn't exist, start with empty contacts
        if !self.vcf_path.exists() {
            self.contacts = HashMap::new();
            self.loaded = true;
            return Ok(());
        }

        // Load contacts from VCF file
        let vcf_content = fs::read_to_string(&self.vcf_path)
            .with_context(|| format!("Failed to read VCF file: {:?}", self.vcf_path))?;

        let contacts = import_contacts_from_vcf(&vcf_content)
            .context("Failed to parse VCF file")?;

        self.contacts = contacts.into_iter().map(|c| (c.id, c)).collect();
        self.loaded = true;

        Ok(())
    }

    /// Save all contacts to the VCF file
    fn save(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.vcf_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        // Convert contacts to sorted vector for consistent output
        let mut contacts: Vec<_> = self.contacts.values().cloned().collect();
        contacts.sort_by(|a, b| a.name.cmp(&b.name));

        // Export to VCF
        let vcf_content = export_contacts_to_vcf(&contacts)
            .context("Failed to export contacts to VCF")?;

        // Write to file
        fs::write(&self.vcf_path, vcf_content)
            .with_context(|| format!("Failed to write VCF file: {:?}", self.vcf_path))?;

        Ok(())
    }

    /// Create a new contact
    pub fn create(&mut self, contact: Contact) -> Result<()> {
        self.ensure_loaded()?;

        let id = contact.id;
        self.contacts.insert(id, contact);
        self.save()?;

        Ok(())
    }

    /// Read a contact by ID
    pub fn read(&mut self, id: Uuid) -> Result<Option<Contact>> {
        self.ensure_loaded()?;
        Ok(self.contacts.get(&id).cloned())
    }

    /// Update an existing contact
    pub fn update(&mut self, contact: Contact) -> Result<()> {
        self.ensure_loaded()?;

        let id = contact.id;
        if !self.contacts.contains_key(&id) {
            anyhow::bail!("Contact not found: {}", id);
        }

        self.contacts.insert(id, contact);
        self.save()?;

        Ok(())
    }

    /// Delete a contact by ID
    pub fn delete(&mut self, id: Uuid) -> Result<()> {
        self.ensure_loaded()?;

        if self.contacts.remove(&id).is_none() {
            anyhow::bail!("Contact not found: {}", id);
        }

        self.save()?;
        Ok(())
    }

    /// List all contacts with pagination
    pub fn list(&mut self, limit: usize, offset: usize) -> Result<Vec<Contact>> {
        self.ensure_loaded()?;

        let mut contacts: Vec<_> = self.contacts.values().cloned().collect();
        contacts.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(contacts.into_iter().skip(offset).take(limit).collect())
    }

    /// Search contacts by name, email, or phone
    pub fn search(&mut self, query: &str) -> Result<Vec<Contact>> {
        self.ensure_loaded()?;

        let query_lower = query.to_lowercase();
        let mut contacts: Vec<_> = self
            .contacts
            .values()
            .filter(|c| {
                c.name.to_lowercase().contains(&query_lower)
                    || c.primary_email()
                        .map(|e| e.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || c.primary_phone()
                        .map(|p| p.contains(&query_lower))
                        .unwrap_or(false)
            })
            .cloned()
            .collect();

        contacts.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(contacts)
    }

    /// Count total number of contacts
    pub fn count(&mut self) -> Result<usize> {
        self.ensure_loaded()?;
        Ok(self.contacts.len())
    }

    /// List all contacts (no pagination)
    pub fn list_all(&mut self) -> Result<Vec<Contact>> {
        self.ensure_loaded()?;

        let mut contacts: Vec<_> = self.contacts.values().cloned().collect();
        contacts.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(contacts)
    }

    /// Get the VCF file path
    pub fn vcf_path(&self) -> &Path {
        &self.vcf_path
    }

    /// Reload contacts from disk (discarding in-memory changes)
    pub fn reload(&mut self) -> Result<()> {
        self.loaded = false;
        self.contacts.clear();
        self.ensure_loaded()
    }

    /// Check if the VCF file exists
    pub fn exists(&self) -> bool {
        self.vcf_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::contact::ContactBuilder;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_read() {
        let dir = tempdir().unwrap();
        let vcf_path = dir.path().join("contacts.vcf");
        let mut repo = VcfRepository::new(&vcf_path);

        let contact = ContactBuilder::new()
            .name("John Doe")
            .first_name("John")
            .last_name("Doe")
            .email("john@example.com")
            .build()
            .unwrap();

        let id = contact.id;

        // Create contact
        repo.create(contact.clone()).unwrap();

        // Read it back
        let retrieved = repo.read(id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "John Doe");

        // Verify VCF file was created
        assert!(vcf_path.exists());
    }

    #[test]
    fn test_update() {
        let dir = tempdir().unwrap();
        let vcf_path = dir.path().join("contacts.vcf");
        let mut repo = VcfRepository::new(&vcf_path);

        let mut contact = ContactBuilder::new()
            .name("Jane Doe")
            .first_name("Jane")
            .last_name("Doe")
            .build()
            .unwrap();

        let id = contact.id;
        repo.create(contact.clone()).unwrap();

        // Update the contact
        contact.first_name = Some("Janet".to_string());
        repo.update(contact).unwrap();

        // Read it back
        let retrieved = repo.read(id).unwrap().unwrap();
        assert_eq!(retrieved.first_name, Some("Janet".to_string()));
    }

    #[test]
    fn test_delete() {
        let dir = tempdir().unwrap();
        let vcf_path = dir.path().join("contacts.vcf");
        let mut repo = VcfRepository::new(&vcf_path);

        let contact = ContactBuilder::new()
            .name("Delete Me")
            .first_name("Delete")
            .last_name("Me")
            .build()
            .unwrap();

        let id = contact.id;
        repo.create(contact).unwrap();

        // Delete the contact
        repo.delete(id).unwrap();

        // Verify it's gone
        let retrieved = repo.read(id).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_list() {
        let dir = tempdir().unwrap();
        let vcf_path = dir.path().join("contacts.vcf");
        let mut repo = VcfRepository::new(&vcf_path);

        // Create multiple contacts
        for i in 1..=5 {
            let contact = ContactBuilder::new()
                .name(format!("Person {}", i))
                .first_name(format!("Person{}", i))
                .build()
                .unwrap();
            repo.create(contact).unwrap();
        }

        // List with pagination
        let page1 = repo.list(2, 0).unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = repo.list(2, 2).unwrap();
        assert_eq!(page2.len(), 2);

        // Count
        assert_eq!(repo.count().unwrap(), 5);
    }

    #[test]
    fn test_search() {
        let dir = tempdir().unwrap();
        let vcf_path = dir.path().join("contacts.vcf");
        let mut repo = VcfRepository::new(&vcf_path);

        let contact1 = ContactBuilder::new()
            .name("Alice Smith")
            .first_name("Alice")
            .last_name("Smith")
            .email("alice@example.com")
            .build()
            .unwrap();

        let contact2 = ContactBuilder::new()
            .name("Bob Jones")
            .first_name("Bob")
            .last_name("Jones")
            .email("bob@example.com")
            .build()
            .unwrap();

        repo.create(contact1).unwrap();
        repo.create(contact2).unwrap();

        // Search by name
        let results = repo.search("Alice").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Alice Smith");

        // Search by email
        let results = repo.search("bob@").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Bob Jones");
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();
        let vcf_path = dir.path().join("contacts.vcf");

        // Create and save a contact
        {
            let mut repo = VcfRepository::new(&vcf_path);
            let contact = ContactBuilder::new()
                .name("Test User")
                .first_name("Test")
                .last_name("User")
                .build()
                .unwrap();
            repo.create(contact).unwrap();
        }

        // Create a new repository instance and verify data persisted
        {
            let mut repo = VcfRepository::new(&vcf_path);
            let contacts = repo.list_all().unwrap();
            assert_eq!(contacts.len(), 1);
            assert_eq!(contacts[0].name, "Test User");
        }
    }
}