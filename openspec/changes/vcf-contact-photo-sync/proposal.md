# VCF Contact Photo Sync

## Why

Profile Pulse currently supports VCF import. However, users cannot interactively view, select, and edit contacts from imported VCF files in the GUI. Additionally, there is no way to fetch profile pictures from the social media URLs stored in contacts. This update adds a complete contact viewing/editing workflow with photo fetching capabilities.

## What Changes

- **Main Window Restructure**: New main window with tabbed interface for Import, Contacts, and Photo Fetcher
- **VCF Import UI**: File picker and import workflow from main window
- **Contact List View**: Display all contacts from imported VCF in a selectable list
- **Contact Detail/Edit**: Full contact editor showing all VCF fields with edit button
- **Photo Fetcher Tab**: Tab to fetch profile pictures from social media URLs configured in contact
- **Photo Selection & Save**: Allow selecting a profile picture and saving it to contact

## Capabilities

### New Capabilities

- `vcf-import-ui`: New main window UI for VCF import workflow (file picker, import progress)
- `contact-list-view`: Display contacts from imported VCF file in a scrollable list
- `contact-detail-edit`: Full contact editor with all VCF fields, edit capability
- `photo-fetcher`: Fetch profile pictures from social URLs configured in contact
- `photo-save`: Save selected profile picture to contact

### Modified Capabilities

- `vcf-import-export`: Add interactive file picker workflow (requirement unchanged, UI layer added)

## Impact

- New GUI windows: Main window with tabs, Contact detail window
- New modules: GUI components for contact list, contact editor, photo fetcher
- Dependencies: `iced` for GUI, existing VCF parsing (`src/vcf/mod.rs`)
- Database: New `photo_blob` field already exists in Contact model

---

**Related Specs**:
- Contact fields: `contact-fields/spec.md`
- VCF Import/Export: `vcf-import-export/spec.md`