# Workspace System

**Status**: Implemented  
**Since**: Phase 1 (2026-01)  
**Source**: `.agents/WORKSPACE_ARCHITECTURE.md`

## Overview

Profile Pulse uses a workspace-based architecture where each VCF file gets its own isolated environment with a dedicated SQLite database. VCF files are the source of truth while SQLite provides performance caching.

## ADDED Requirements

### Requirement: VCF as source of truth

Contact data SHALL be stored in VCF files as the primary storage format. The SQLite database is a performance cache that can be regenerated from VCF at any time.

#### Scenario: New workspace created

- **WHEN** user creates a new workspace named "Personal Contacts"
- **THEN** a folder is created with `contacts.vcf` (empty but valid VCF) and `contacts.db` (SQLite cache)
- **AND** the workspace is recorded in `workspaces.json`

#### Scenario: Database regeneratable from VCF

- **WHEN** the SQLite database is corrupted or deleted
- **THEN** the application can rebuild the database by parsing `contacts.vcf`

### Requirement: Workspace isolation

Each workspace SHALL have complete data isolation with its own VCF file and SQLite database. No cross-workspace data access is possible.

#### Scenario: Multiple workspaces coexist

- **WHEN** user has workspaces "Personal", "Work", and "Family"
- **THEN** each has its own UUID folder with separate `contacts.vcf` and `contacts.db`
- **AND** deleting one workspace does not affect others

#### Scenario: Workspaces listed in index

- **WHEN** the application starts
- **THEN** all workspaces are loaded from `workspaces.json`
- **AND** displayed as workspace cards in the selector view

### Requirement: Workspace data directory structure

The application data directory SHALL follow this structure:

```
~/.local/share/com.profile-pulse.Profile Pulse/
├── workspaces/
│   ├── workspaces.json              # Workspace index
│   │
│   ├── {uuid}/
│   │   ├── contacts.vcf         # Source of truth
│   │   └── contacts.db          # SQLite cache
│   └── ...
└── cache/                        # Shared cache
```

#### Scenario: Standard directory creation

- **WHEN** application initializes
- **THEN** the directory structure is created under the platform-specific data directory
- **AND** `workspaces.json` is created if it doesn't exist

### Requirement: Workspace CRUD operations

Users SHALL be able to create, open, and delete workspaces.

#### Scenario: Create empty workspace

- **WHEN** user enters a workspace name and clicks "Create Empty"
- **THEN** a new workspace folder is created with valid empty `contacts.vcf`
- **AND** SQLite database is initialized with migrations
- **AND** workspace appears in the list

#### Scenario: Create workspace from existing VCF

- **WHEN** user has an existing VCF file and selects "Import VCF"
- **THEN** the file is copied into a new workspace folder
- **AND** database is populated from the VCF content

#### Scenario: Delete workspace

- **WHEN** user clicks "Delete" on a workspace
- **THEN** the workspace is removed from `workspaces.json`
- **AND** the workspace folder is deleted (with confirmation)

### Requirement: Workspace metadata tracking

Each workspace SHALL track metadata for display in the workspace selector.

#### Scenario: Workspace card displays metadata

- **WHEN** the workspace selector shows workspace cards
- **THEN** each card displays: name, contact count, last accessed timestamp, VCF file path
- **AND** shows "Open" and "Delete" buttons

### Requirement: Workspace switching

Users SHALL be able to switch between workspaces at any time.

#### Scenario: Switch to different workspace

- **WHEN** user clicks "Workspaces" button in main view
- **THEN** the current workspace state is saved
- **AND** the workspace selector is displayed
- **AND** user can select a different workspace

---

## Implementation Notes

- Implemented in: `src/workspace/mod.rs`
- WorkspaceManager: handles CRUD operations on workspaces
- VCF file operations: via `src/vcf/repository.rs`
- Database per workspace: migrations run on workspace open
- Source: `.agents/WORKSPACE_ARCHITECTURE.md` (458 lines)

---

**Related**:

- VCF spec: `vcf-import-export/spec.md`
- Database spec: `database-schema/spec.md`
