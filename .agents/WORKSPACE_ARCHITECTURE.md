# Workspace Architecture

## Overview

Profile Pulse uses a **workspace-based architecture** where each VCF file gets its own isolated environment with a dedicated database. This design makes VCF files the source of truth while using SQLite as a performance cache.

## Architecture Principles

### 1. VCF as Source of Truth

- **VCF files are portable** - Can be moved, synced, backed up independently
- **Human-readable format** - Standard vCard format, readable by any text editor
- **Database is a cache** - SQLite stores indexed data for fast queries
- **Regenerable** - Database can be rebuilt from VCF at any time

### 2. Workspace Isolation

- **Multiple address books** - Personal, work, family, etc.
- **Complete data separation** - No cross-workspace data leakage
- **Independent databases** - Each workspace has its own SQLite file
- **Atomic operations** - Delete workspace = delete entire folder

### 3. User-Centric Design

- **Workspace selector first** - Users choose which address book to work with
- **Visual workspace cards** - Show name, contact count, last accessed
- **Easy switching** - Return to selector and open different workspace
- **Simple backup** - Copy VCF file, that's it

## Directory Structure

```
~/.local/share/com.profile-pulse.Profile Pulse/
├── workspaces/
│   ├── workspaces.json                    # Workspace index
│   │
│   ├── 550e8400-e29b-41d4-a716-446655440000/
│   │   ├── contacts.vcf                   # Source of truth (Personal)
│   │   └── contacts.db                    # SQLite cache
│   │
│   ├── 6ba7b810-9dad-11d1-80b4-00c04fd430c8/
│   │   ├── contacts.vcf                   # Source of truth (Work)
│   │   └── contacts.db                    # SQLite cache
│   │
│   └── 7c9e6679-7425-40de-944b-e07fc1f90ae7/
│       ├── contacts.vcf                   # Source of truth (Family)
│       └── contacts.db                    # SQLite cache
│
└── cache/                                  # Shared cache (profile pictures, etc.)
```

### File Descriptions

- **`workspaces.json`** - Index of all workspaces with metadata (name, paths, timestamps)
- **`contacts.vcf`** - Standard vCard 3.0/4.0 file with all contact data
- **`contacts.db`** - SQLite database with indexed contact data for performance
- **UUID folders** - Each workspace in its own directory named by UUID

## Data Model

### Workspace

```rust
struct Workspace {
    id: Uuid,                           // Unique identifier
    name: String,                       // User-friendly name
    vcf_path: PathBuf,                  // Path to VCF file
    workspace_dir: PathBuf,             // Path to workspace folder
    db_path: PathBuf,                   // Path to SQLite DB
    created_at: DateTime<Utc>,          // Creation timestamp
    last_accessed: DateTime<Utc>,      // Last opened timestamp
    contact_count: usize,               // Cached count
}
```

### WorkspaceManager

```rust
impl WorkspaceManager {
    // Load all workspaces from index
    fn load_workspaces(&self) -> Result<Vec<Workspace>>;

    // Create new empty workspace
    fn create_empty_workspace(&self, name: String) -> Result<Workspace>;

    // Create workspace from existing VCF
    fn create_workspace(&self, name: String, vcf_path: PathBuf) -> Result<Workspace>;

    // Delete workspace (removes folder)
    fn delete_workspace(&self, workspace_id: Uuid) -> Result<()>;

    // Update workspace metadata
    fn update_workspace(&self, workspace: &Workspace) -> Result<()>;
}
```

## Application Flow

### Startup Sequence

```
┌─────────────────────────────────────┐
│   1. Launch Application             │
│      - No database initialization   │
│      - Create WorkspaceManager      │
└──────────────┬──────────────────────┘
               ▼
┌─────────────────────────────────────┐
│   2. Show Workspace Selector        │
│      - Load workspaces.json         │
│      - Display workspace cards      │
└──────────────┬──────────────────────┘
               ▼
┌─────────────────────────────────────┐
│   3. User Selects Workspace         │
│      - Initialize that workspace DB │
│      - Run migrations               │
│      - Create repository            │
└──────────────┬──────────────────────┘
               ▼
┌─────────────────────────────────────┐
│   4. Load Contacts                  │
│      - Query from workspace DB      │
│      - Display in main view         │
└─────────────────────────────────────┘
```

### Workspace Operations

#### Create New Workspace

```
User Input: "Personal Contacts"
     ↓
Generate UUID: 550e8400-...
     ↓
Create Folder: workspaces/550e8400-.../
     ↓
Create VCF: contacts.vcf (empty)
     ↓
Create DB: contacts.db (with schema)
     ↓
Add to workspaces.json
     ↓
Reload Workspace List
```

#### Open Workspace

```
User Clicks "Open"
     ↓
Get Workspace by ID
     ↓
Initialize SQLite Pool
     ↓
Run Migrations
     ↓
Create ContactRepository
     ↓
Load Contacts from DB
     ↓
Switch to Main View
```

#### Switch Workspace

```
User Clicks "📁 Workspaces"
     ↓
Save Current Workspace State
     ↓
Close Repository/Connection
     ↓
Clear Current Workspace
     ↓
Return to Workspace Selector
     ↓
User Selects Different Workspace
     ↓
Open New Workspace (see above)
```

#### Delete Workspace

```
User Clicks "Delete"
     ↓
Remove from workspaces.json
     ↓
Delete Workspace Folder
     ↓
Reload Workspace List
```

## UI Components

### Workspace Selector View

```
┌─────────────────────────────────────────────────────┐
│  Select or Create a Workspace                       │
│  Each workspace manages its own VCF file and database│
│                                                      │
│  Create New Workspace                               │
│  ┌────────────────────────┐  ┌──────────────────┐  │
│  │ Workspace name         │  │ Create Empty     │  │
│  └────────────────────────┘  └──────────────────┘  │
│                                                      │
│  Existing Workspaces                                │
│  ┌─────────────────────────────────────────────┐   │
│  │  Personal Contacts          12 contacts     │   │
│  │  VCF: .../contacts.vcf                      │   │
│  │  Last accessed: 2026-01-15 04:30            │   │
│  │  [Open]  [Delete]                           │   │
│  └─────────────────────────────────────────────┘   │
│                                                      │
│  ┌─────────────────────────────────────────────┐   │
│  │  Work Contacts              45 contacts     │   │
│  │  VCF: .../contacts.vcf                      │   │
│  │  Last accessed: 2026-01-14 10:15            │   │
│  │  [Open]  [Delete]                           │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

### Main View (With Workspace)

```
┌─────────────────────────────────────────────────────┐
│  Profile Pulse - Personal Contacts                  │
│  [+ Add] [📥 Import] [📤 Export] [📁 Workspaces]    │
│                                                      │
│  [Search contacts...]                               │
│  [All] [A] [B] [C] ... [Z]                         │
│                                                      │
│  Contact List...                                    │
└─────────────────────────────────────────────────────┘
```

## Advantages

### For Users

1. **Multiple Address Books** - Separate personal, work, family contacts
2. **Easy Backup** - Copy VCF file to backup contacts
3. **Portable Data** - VCF works with any contact app
4. **Visual Organization** - See all address books at a glance
5. **Fast Performance** - Database provides instant search

### For Developers

1. **Clear Separation** - VCF = storage, SQLite = index
2. **Simple Architecture** - Each workspace independent
3. **Easy Testing** - Create temporary workspace folders
4. **No Multi-Tenancy** - No complex tenant isolation logic
5. **Atomic Cleanup** - Delete folder = delete everything

### For Data Management

1. **Cloud Sync Friendly** - Sync VCF files, regenerate databases
2. **Version Control Ready** - VCF files can be git-tracked
3. **Migration Friendly** - Import/export via standard VCF
4. **Disaster Recovery** - Rebuild DB from VCF if corrupted
5. **Audit Trail** - VCF changes visible in text diff

## Database Schema (Per Workspace)

Each workspace database has the same schema:

```sql
-- Core contacts table
CREATE TABLE contacts (...);

-- Structured fields (with labels)
CREATE TABLE contact_emails (...);
CREATE TABLE contact_phones (...);
CREATE TABLE contact_addresses (...);
CREATE TABLE contact_dates (...);
CREATE TABLE contact_urls (...);

-- Cache for fetched profile data
CREATE TABLE profile_cache (...);
```

**Key Points**:

- Schema identical across all workspaces
- Migrations run independently per workspace
- No shared data between workspace databases
- Each workspace can be at different schema version (though we'll keep them in sync)

## Synchronization Strategy

### VCF as Synchronization Unit

```
Device A                    Cloud Storage                Device B
┌──────────┐               ┌──────────┐                ┌──────────┐
│ contacts │               │ contacts │                │ contacts │
│   .vcf   │ ──Upload────► │   .vcf   │ ──Download──► │   .vcf   │
│          │               │          │                │          │
│ (+ DB)   │               │          │                │ (+ DB)   │
└──────────┘               └──────────┘                └──────────┘
     ↓                                                       ↓
  Regenerate                                            Regenerate
  Database                                              Database
```

**Benefits**:

- Only sync VCF files (small, standard format)
- Databases regenerated on each device
- No database sync conflicts
- Standard cloud storage (Dropbox, iCloud, etc.)

## Performance Considerations

### Why SQLite Per Workspace?

**Advantages**:

- Fast indexed queries for search/filter
- Efficient for hundreds/thousands of contacts
- Better than parsing VCF on every operation
- Enables advanced features (caching, analytics)

**Trade-offs**:

- Additional storage (VCF + DB)
- Need to keep VCF and DB in sync
- Startup overhead (initialize DB)

**Mitigation**:

- Lazy initialization (only when workspace opened)
- DB is considered regenerable cache
- VCF remains source of truth
- Can rebuild DB from VCF if needed

### Memory Footprint

- **Small workspaces (<100 contacts)**: Negligible overhead
- **Medium workspaces (100-1000 contacts)**: ~1-5 MB per workspace
- **Large workspaces (>1000 contacts)**: ~5-20 MB per workspace

Only active workspace database is loaded in memory.

## Future Enhancements

### Planned Features

1. **Workspace Import/Export**
   - Bundle VCF + metadata into single file
   - Share workspaces between devices

2. **Workspace Templates**
   - Pre-configured workspaces for common use cases
   - "Personal", "Work", "Family" templates

3. **Workspace Search**
   - Search across all workspaces
   - Quick switcher (Cmd+K style)

4. **Workspace Sync**
   - Built-in sync service
   - Automatic VCF backup to cloud

5. **Workspace Merge**
   - Combine multiple workspaces
   - Duplicate detection

### Possible Features

- Workspace tags/categories
- Workspace color coding
- Workspace-specific settings (privacy levels, etc.)
- Workspace analytics (most contacted, etc.)
- Workspace sharing (read-only access)
- Workspace history (snapshots, undo)

## Security Considerations

### Data Isolation

- Each workspace completely isolated
- No SQL injection across workspaces
- Deleting workspace removes all data

### Privacy

- All data stored locally
- No telemetry from workspaces
- User controls VCF file location
- Can encrypt VCF files at rest (OS-level)

### Backup

- Users should backup VCF files regularly
- Consider automatic VCF snapshots
- Database can be rebuilt from VCF

## Migration from Old Architecture

For users who had the previous single-database setup:

```rust
fn migrate_old_database_to_workspace() -> Result<()> {
    // 1. Export all contacts from old DB to VCF
    let contacts = old_db.list_all();
    let vcf_content = export_to_vcf(contacts);

    // 2. Create "Default" workspace
    let workspace = workspace_manager.create_empty_workspace("Default");

    // 3. Import VCF into new workspace
    import_vcf_to_workspace(&workspace, vcf_content);

    // 4. Archive old database (don't delete yet)
    rename("profile-pulse.db", "profile-pulse.db.backup");

    Ok(())
}
```

## Testing Strategy

### Unit Tests

- Workspace creation/deletion
- Workspace index serialization
- Path manipulation

### Integration Tests

- Full workspace lifecycle
- Database initialization
- VCF import/export within workspace

### Manual Tests

- Create multiple workspaces
- Switch between workspaces
- Verify data isolation
- Delete workspace
- Backup/restore VCF

## Conclusion

The workspace architecture transforms Profile Pulse into a **professional contact management system** that:

✅ **Respects user data ownership** (VCF files are portable)
✅ **Provides excellent performance** (SQLite indexes and queries)
✅ **Supports multiple address books** (personal, work, etc.)
✅ **Ensures data isolation** (no cross-workspace leakage)
✅ **Enables easy backup/sync** (VCF files are standard format)

This architecture addresses the concern about "why use a database" by making it clear that **VCF is the source of truth** and **SQLite is just a performance cache**. Users get the best of both worlds: portability and performance.
