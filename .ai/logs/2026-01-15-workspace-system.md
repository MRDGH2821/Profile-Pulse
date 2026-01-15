# AI Work Log - Workspace System Implementation

## 2026-01-15T04:41:33+00:00

### Request

User asked: "can you add a screen where every vcf file gets its own folder in the app's data location where database is also saved"

This addresses the concern about using a database by implementing a hybrid approach where:
- Each VCF file is the source of truth (portable, human-readable)
- Each VCF gets its own workspace folder with its own SQLite database (performance cache)
- Users can manage multiple address books independently
- Easy backup/sync (just backup VCF files)

### AI Model

**Model**: Claude Sonnet 4.5
**Provider**: Anthropic

### Work Performed

#### 1. **Workspace Module Created** (`src/workspace/mod.rs`)

**Workspace Structure**:
- Each workspace has a unique UUID
- Contains VCF file path (source of truth)
- Has its own directory with SQLite database
- Tracks metadata: name, creation time, last accessed, contact count

**WorkspaceManager Features**:
- Load/save workspace index (`workspaces.json`)
- Create empty workspaces (new address book)
- Create workspace from existing VCF file
- Delete workspaces (removes directory)
- Update workspace metadata
- Import/export VCF files

**Data Structure**:
```
~/.local/share/com.profile-pulse.Profile Pulse/
├── workspaces/
│   ├── workspaces.json                    # Index of all workspaces
│   ├── <uuid-1>/
│   │   ├── contacts.vcf                   # Source of truth
│   │   └── contacts.db                    # Performance cache
│   ├── <uuid-2>/
│   │   ├── contacts.vcf
│   │   └── contacts.db
│   └── <uuid-3>/
│       ├── contacts.vcf
│       └── contacts.db
```

#### 2. **UI Updates** (`src/ui/mod.rs`)

**New View**:
- `View::WorkspaceSelector` - First screen users see

**Updated State**:
- `workspace_manager: WorkspaceManager` - Manages all workspaces
- `current_workspace: Option<Workspace>` - Currently open workspace
- `workspaces: Vec<Workspace>` - List of available workspaces
- `repository: Option<ContactRepository>` - Now optional (only exists when workspace is open)
- `new_workspace_name: String` - Input field for creating new workspace

**New Messages**:
- `LoadWorkspaces` - Load all workspaces from index
- `WorkspacesLoaded(Result<Vec<Workspace>, String>)` - Workspaces loaded
- `SelectWorkspace(Uuid)` - User selects a workspace to open
- `WorkspaceSelected(Result<ContactRepository, String>)` - Workspace database initialized
- `CreateEmptyWorkspace` - Create new empty address book
- `DeleteWorkspace(Uuid)` - Delete a workspace
- `BackToWorkspaceSelector` - Return to workspace selector from main view
- `NewWorkspaceNameChanged(String)` - Update new workspace name input
- `CreateNewWorkspace` - Placeholder for VCF file picker
- `ImportVcfAsWorkspace` - Placeholder for VCF import

**Workspace Selector UI**:
- Title and subtitle explaining workspaces
- Create new workspace section with name input
- List of existing workspaces showing:
  - Workspace name
  - Contact count
  - VCF file path
  - Last accessed timestamp
  - Open button
  - Delete button
- Cards styled with rounded borders and light background

**Main View Updates**:
- Added "📁 Workspaces" button to header
- Title now shows current workspace name
- All database operations check if workspace is selected

#### 3. **Initialization Changes** (`src/main.rs`)

**Old Flow**:
1. Initialize global database
2. Run migrations
3. Create repository
4. Launch GUI

**New Flow**:
1. Launch GUI with workspace selector
2. User selects/creates workspace
3. Initialize that workspace's database
4. Load contacts from that workspace

**Benefits**:
- No upfront database initialization
- Faster startup
- Multiple address books support
- Each workspace isolated

#### 4. **Database Integration**

**Per-Workspace Databases**:
- Each workspace has `contacts.db` in its directory
- Same schema for all workspaces
- Migrations run when workspace first opened
- Completely isolated data

**Lazy Initialization**:
- Database only created when workspace selected
- `SelectWorkspace` message handles async DB initialization
- Error handling if DB initialization fails

### Files Changed

**Created** (1 file, ~351 lines):
- `src/workspace/mod.rs` (351 lines)
  - Workspace struct with metadata
  - WorkspaceManager with CRUD operations
  - 5 comprehensive unit tests

**Modified** (2 files, ~200 lines changed):
- `src/ui/mod.rs` (~180 lines added/changed)
  - Added workspace imports
  - Updated State struct with workspace fields
  - Added 10 new message variants
  - Added workspace message handlers
  - Created `view_workspace_selector()` function
  - Updated view matching to include workspace selector
  - Updated all repository access to check if workspace selected
  - Added workspace button to main view header
- `src/main.rs` (~20 lines changed)
  - Added workspace module import
  - Removed upfront database initialization
  - Changed to call `ui::run()` instead of `ui::run_with_repository()`

### Nature of Assistance

- **Architecture Design**: Designed workspace system to address database concerns
- **Data Structure**: Created workspace folder hierarchy
- **State Management**: Integrated workspace concept into Iced state machine
- **UI Design**: Created workspace selector screen with cards
- **Code Generation**: Implemented complete workspace module with tests
- **Error Handling**: Added proper Option checks for repository access
- **Async Integration**: Handled async database initialization per workspace

### Human Involvement

- Identified the core concern about using a database
- Proposed workspace-based solution
- Will test workspace functionality in running application:
  - Create new workspace
  - Switch between workspaces
  - Delete workspace
  - Verify data isolation
- Will validate that VCF files remain the source of truth
- May suggest additional features (workspace import/export, backup, etc.)

### Testing Status

- ✅ **Compilation**: Successful (58 warnings for dead code - expected)
- ✅ **Workspace Tests**: 5/5 tests passing
  - `test_workspace_creation`
  - `test_workspace_manager_creation`
  - `test_create_empty_workspace`
  - `test_load_save_workspaces`
  - `test_delete_workspace`
- ✅ **Existing Tests**: All 70 tests still passing
- ⏳ **Manual GUI Testing**: Pending
  - Test workspace selector on app startup
  - Create new workspace
  - Open existing workspace
  - Switch between workspaces
  - Delete workspace
  - Verify database isolation

### Implementation Details

**Workspace Lifecycle**:

1. **App Startup**:
   ```
   Launch App → Show Workspace Selector → Load workspaces.json
   ```

2. **Create New Workspace**:
   ```
   Enter name → Click "Create Empty Workspace"
   → Create UUID folder → Create empty VCF → Create empty DB
   → Add to workspaces.json → Reload workspace list
   ```

3. **Open Workspace**:
   ```
   Click "Open" → Initialize DB for that workspace
   → Run migrations → Create repository → Load contacts → Show main view
   ```

4. **Return to Selector**:
   ```
   Click "📁 Workspaces" → Clear current workspace
   → Clear repository → Show workspace selector
   ```

5. **Delete Workspace**:
   ```
   Click "Delete" → Remove from workspaces.json
   → Delete workspace folder → Reload workspace list
   ```

**Data Isolation**:
- Each workspace has completely separate database
- No cross-workspace data leakage
- Deleting workspace removes all data
- VCF file can be backed up independently

**VCF as Source of Truth**:
- VCF file is the canonical data
- Database is just a performance cache
- Import/export always goes through VCF
- Can restore from VCF if database corrupted

### Advantages of This Approach

**Addresses Database Concerns**:
- ✅ VCF files are portable (not locked in database)
- ✅ Easy backup (just copy VCF files)
- ✅ Human-readable source of truth
- ✅ Database is clearly a cache, not primary storage

**Additional Benefits**:
- ✅ Multiple address books (personal, work, family, etc.)
- ✅ Data isolation between workspaces
- ✅ Fast performance (SQLite cache for each workspace)
- ✅ Flexible: Can work with VCF files directly
- ✅ Cloud sync friendly (sync VCF files, regenerate databases)

**Future Possibilities**:
- Workspace import/export (bundle VCF + metadata)
- Workspace templates
- Workspace-specific settings
- Workspace sharing (export VCF)
- Workspace cloud sync (sync VCF only)

### Known Issues

**Minor**:
- "Create from VCF file" not yet implemented (placeholder)
- "Import VCF as workspace" not yet implemented (placeholder)
- Contact count not updated after adding/removing contacts (cached value)
- No confirmation dialog before deleting workspace

**Future Enhancements Needed**:
- File picker for "Create from existing VCF"
- Workspace rename
- Workspace export (zip VCF + settings)
- Workspace backup (automatic VCF copies)
- Recently accessed workspaces (quick access)
- Workspace search/filter
- Workspace tags/categories

### User Experience Flow

**First Launch**:
```
1. App opens to Workspace Selector (empty)
2. User sees "Create New Workspace" section
3. User types "Personal Contacts"
4. User clicks "Create Empty Workspace"
5. Workspace appears in list below
6. User clicks "Open"
7. Database initializes
8. Main view opens (empty contact list)
9. User can import VCF or add contacts manually
```

**Subsequent Launches**:
```
1. App opens to Workspace Selector
2. User sees list of workspaces with metadata
3. User clicks "Open" on desired workspace
4. Last accessed timestamp updates
5. Main view opens with contacts loaded
```

**Switching Workspaces**:
```
1. User clicks "📁 Workspaces" button
2. Returns to Workspace Selector
3. Current workspace saved/closed
4. User selects different workspace
5. New workspace loaded
```

### Technical Decisions

**Why UUID for Folder Names?**
- Avoids conflicts from special characters in names
- Users can rename workspaces without moving folders
- Clean, consistent folder structure

**Why JSON Index File?**
- Fast to parse
- Human-readable for debugging
- Easy to backup with VCF files
- Can add metadata without touching databases

**Why SQLite Per Workspace?**
- Complete data isolation
- No complex multi-tenancy logic
- Can delete workspace by removing folder
- Each workspace independently upgradeable

**Why Keep VCF Separate from Index?**
- VCF is the source of truth
- Index is just metadata
- Can rebuild index from scanning folders
- VCF can be synced independently

### Migration Path

**For Existing Users** (if any existed):
The old `profile-pulse.db` would be ignored. Users would need to:
1. Export contacts to VCF
2. Create new workspace
3. Import VCF into workspace

Or we could add migration code to auto-create a "Default" workspace from the old database.

### Documentation Needed

- User guide for workspace concept
- How to backup workspaces (copy VCF files)
- How to share workspaces (export VCF)
- How to sync workspaces across devices
- Best practices for organizing workspaces

### Conclusion

This implementation transforms Profile Pulse from a single-database application to a **workspace-based contact manager** where:
- **VCF files are the source of truth** (addresses portability concerns)
- **SQLite provides performance** (fast search, efficient queries)
- **Users can manage multiple address books** (personal, work, etc.)
- **Each workspace is fully isolated** (no data leakage)

The database is now clearly positioned as a **performance cache** rather than the primary storage, addressing the user's concern while maintaining the benefits of database-backed queries and indexing.

**Ready for manual testing!**