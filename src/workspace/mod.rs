//! Workspace management for multiple VCF files
//!
//! Each workspace represents a VCF file (single source of truth).
//! This allows users to manage multiple address books independently.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// A workspace represents a VCF file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Unique identifier for this workspace
    pub id: Uuid,
    /// Display name for this workspace
    pub name: String,
    /// Path to the VCF file (source of truth)
    pub vcf_path: PathBuf,
    /// Path to the workspace directory
    pub workspace_dir: PathBuf,
    /// When this workspace was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When this workspace was last accessed
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    /// Number of contacts in this workspace (cached)
    pub contact_count: usize,
}

impl Workspace {
    /// Create a new workspace
    pub fn new(name: String, vcf_path: PathBuf, workspace_dir: PathBuf) -> Self {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        Self {
            id,
            name,
            vcf_path,
            workspace_dir,
            created_at: now,
            last_accessed: now,
            contact_count: 0,
        }
    }

    /// Update last accessed timestamp
    pub fn touch(&mut self) {
        self.last_accessed = chrono::Utc::now();
    }

    /// Check if the VCF file exists
    pub fn vcf_exists(&self) -> bool {
        self.vcf_path.exists()
    }

    /// Check if the workspace directory exists
    pub fn workspace_exists(&self) -> bool {
        self.workspace_dir.exists()
    }
}

/// Manages all workspaces
#[derive(Debug, Clone)]
pub struct WorkspaceManager {
    /// Root directory for all workspaces
    root_dir: PathBuf,
    /// Path to the workspaces index file
    index_path: PathBuf,
}

impl WorkspaceManager {
    /// Create a new workspace manager
    pub fn new(root_dir: PathBuf) -> Result<Self> {
        // Create root directory if it doesn't exist
        std::fs::create_dir_all(&root_dir)
            .context("Failed to create workspaces directory")?;

        let index_path = root_dir.join("workspaces.json");

        Ok(Self {
            root_dir,
            index_path,
        })
    }

    /// Load all workspaces from the index file
    pub fn load_workspaces(&self) -> Result<Vec<Workspace>> {
        if !self.index_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&self.index_path)
            .context("Failed to read workspaces index")?;

        let workspaces: Vec<Workspace> = serde_json::from_str(&content)
            .context("Failed to parse workspaces index")?;

        Ok(workspaces)
    }

    /// Save workspaces to the index file
    pub fn save_workspaces(&self, workspaces: &[Workspace]) -> Result<()> {
        let content = serde_json::to_string_pretty(workspaces)
            .context("Failed to serialize workspaces")?;

        std::fs::write(&self.index_path, content)
            .context("Failed to write workspaces index")?;

        Ok(())
    }

    /// Create a new workspace from a VCF file
    pub fn create_workspace(
        &self,
        name: String,
        vcf_path: PathBuf,
    ) -> Result<Workspace> {
        // Validate VCF file exists
        if !vcf_path.exists() {
            anyhow::bail!("VCF file does not exist: {}", vcf_path.display());
        }

        // Create workspace directory
        let workspace_id = Uuid::new_v4();
        let workspace_dir = self.root_dir.join(workspace_id.to_string());
        std::fs::create_dir_all(&workspace_dir)
            .context("Failed to create workspace directory")?;

        // Copy VCF file to workspace directory as backup
        let vcf_copy = workspace_dir.join("contacts.vcf");
        std::fs::copy(&vcf_path, &vcf_copy)
            .context("Failed to copy VCF file to workspace")?;

        // Create workspace
        let workspace = Workspace::new(name, vcf_path, workspace_dir);

        // Load existing workspaces and add new one
        let mut workspaces = self.load_workspaces()?;
        workspaces.push(workspace.clone());
        self.save_workspaces(&workspaces)?;

        Ok(workspace)
    }

    /// Create a new empty workspace
    pub fn create_empty_workspace(&self, name: String) -> Result<Workspace> {
        // Create workspace directory
        let workspace_id = Uuid::new_v4();
        let workspace_dir = self.root_dir.join(workspace_id.to_string());
        std::fs::create_dir_all(&workspace_dir)
            .context("Failed to create workspace directory")?;

        // Create empty VCF file
        let vcf_path = workspace_dir.join("contacts.vcf");
        std::fs::write(&vcf_path, "BEGIN:VCARD\nVERSION:3.0\nEND:VCARD\n")
            .context("Failed to create empty VCF file")?;

        // Create workspace
        let workspace = Workspace::new(name, vcf_path, workspace_dir);

        // Load existing workspaces and add new one
        let mut workspaces = self.load_workspaces()?;
        workspaces.push(workspace.clone());
        self.save_workspaces(&workspaces)?;

        Ok(workspace)
    }

    /// Delete a workspace
    pub fn delete_workspace(&self, workspace_id: Uuid) -> Result<()> {
        let mut workspaces = self.load_workspaces()?;

        // Find and remove workspace
        let workspace = workspaces
            .iter()
            .find(|w| w.id == workspace_id)
            .context("Workspace not found")?
            .clone();

        workspaces.retain(|w| w.id != workspace_id);
        self.save_workspaces(&workspaces)?;

        // Delete workspace directory
        if workspace.workspace_dir.exists() {
            std::fs::remove_dir_all(&workspace.workspace_dir)
                .context("Failed to delete workspace directory")?;
        }

        Ok(())
    }

    /// Update workspace metadata
    pub fn update_workspace(&self, workspace: &Workspace) -> Result<()> {
        let mut workspaces = self.load_workspaces()?;

        // Find and update workspace
        let idx = workspaces
            .iter()
            .position(|w| w.id == workspace.id)
            .context("Workspace not found")?;

        workspaces[idx] = workspace.clone();
        self.save_workspaces(&workspaces)?;

        Ok(())
    }

    /// Get workspace by ID
    pub fn get_workspace(&self, workspace_id: Uuid) -> Result<Option<Workspace>> {
        let workspaces = self.load_workspaces()?;
        Ok(workspaces.into_iter().find(|w| w.id == workspace_id))
    }

    /// Import VCF file to workspace (update existing contacts)
    pub fn import_vcf_to_workspace(&self, workspace_id: Uuid, vcf_path: &Path) -> Result<()> {
        let mut workspaces = self.load_workspaces()?;

        // Find workspace
        let workspace = workspaces
            .iter_mut()
            .find(|w| w.id == workspace_id)
            .context("Workspace not found")?;

        // Copy VCF file
        std::fs::copy(vcf_path, &workspace.vcf_path)
            .context("Failed to copy VCF file")?;

        // Also update the backup copy
        let vcf_backup = workspace.workspace_dir.join("contacts.vcf");
        std::fs::copy(vcf_path, &vcf_backup)
            .context("Failed to update VCF backup")?;

        workspace.touch();
        self.save_workspaces(&workspaces)?;

        Ok(())
    }

    /// Export workspace to VCF file
    pub fn export_workspace_vcf(&self, workspace_id: Uuid, dest_path: &Path) -> Result<()> {
        let workspace = self
            .get_workspace(workspace_id)?
            .context("Workspace not found")?;

        // Copy VCF file to destination
        std::fs::copy(&workspace.vcf_path, dest_path)
            .context("Failed to export VCF file")?;

        Ok(())
    }

    /// Get the default workspace (create if none exists)
    pub fn get_or_create_default_workspace(&self) -> Result<Workspace> {
        let workspaces = self.load_workspaces()?;

        if let Some(workspace) = workspaces.first() {
            return Ok(workspace.clone());
        }

        // Create default workspace
        self.create_empty_workspace("My Contacts".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workspace_creation() {
        let name = "Test Workspace".to_string();
        let vcf_path = PathBuf::from("/tmp/test.vcf");
        let workspace_dir = PathBuf::from("/tmp/workspace");

        let workspace = Workspace::new(name.clone(), vcf_path.clone(), workspace_dir.clone());

        assert_eq!(workspace.name, name);
        assert_eq!(workspace.vcf_path, vcf_path);
        assert_eq!(workspace.workspace_dir, workspace_dir);
        assert!(workspace.vcf_path.to_string_lossy().contains("test.vcf"));
    }

    #[test]
    fn test_workspace_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = WorkspaceManager::new(temp_dir.path().to_path_buf());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_create_empty_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let manager = WorkspaceManager::new(temp_dir.path().to_path_buf()).unwrap();

        let workspace = manager.create_empty_workspace("Test".to_string());
        assert!(workspace.is_ok());

        let workspace = workspace.unwrap();
        assert_eq!(workspace.name, "Test");
        assert!(workspace.vcf_path.exists());
        assert!(workspace.workspace_dir.exists());
    }

    #[test]
    fn test_load_save_workspaces() {
        let temp_dir = TempDir::new().unwrap();
        let manager = WorkspaceManager::new(temp_dir.path().to_path_buf()).unwrap();

        // Create workspace
        let workspace = manager.create_empty_workspace("Test".to_string()).unwrap();

        // Load workspaces
        let workspaces = manager.load_workspaces().unwrap();
        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].id, workspace.id);
        assert_eq!(workspaces[0].name, "Test");
    }

    #[test]
    fn test_delete_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let manager = WorkspaceManager::new(temp_dir.path().to_path_buf()).unwrap();

        // Create workspace
        let workspace = manager.create_empty_workspace("Test".to_string()).unwrap();
        let workspace_id = workspace.id;

        // Verify it exists
        let workspaces = manager.load_workspaces().unwrap();
        assert_eq!(workspaces.len(), 1);

        // Delete workspace
        let result = manager.delete_workspace(workspace_id);
        assert!(result.is_ok());

        // Verify it's gone
        let workspaces = manager.load_workspaces().unwrap();
        assert_eq!(workspaces.len(), 0);
    }
}