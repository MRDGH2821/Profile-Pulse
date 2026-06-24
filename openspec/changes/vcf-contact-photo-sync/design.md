# VCF Contact Photo Sync - Design

## Context

Profile Pulse is a desktop contact management app built with Rust and Iced GUI. Existing VCF parsing is in `src/vcf/mod.rs`. Contact data model supports `photo_blob` for storing profile pictures. The app currently lacks interactive GUI workflows for viewing and editing contacts.

## Goals / Non-Goals

**Goals:**
- Add main window with tabbed interface (Import, Contacts, Photo Fetcher)
- Implement VCF file picker via native dialog
- Display imported contacts in selectable list
- Create full contact editor for all VCF fields
- Fetch profile pictures from social URLs (GitHub, LinkedIn, Twitter, etc.)
- Allow saving selected photo to contact

**Non-Goals:**
- Real-time sync with cloud services (manual fetch only)
- Batch photo fetching for all contacts
- Photo cropping/editing within app
- Export functionality (out of scope for this change)

## Decisions

### Decision: Use Iced for GUI framework

**Rationale**: Project already uses `iced` framework. New GUI components should follow existing patterns.

**Alternative considered**: Tauri with React - rejected because existing codebase is pure Rust/Iced.

### Decision: Each contact opens in new window

**Rationale**: Contact detail view is a focused editing experience. Modal dialogs work well for contact editing in Iced.

**Alternative considered**: Tab within main window - rejected to keep main window simple with tabs only for major sections.

### Decision: Photo fetcher fetches per-URL, user selects

**Rationale**: User should choose which profile picture to save. Automatic selection without visual confirmation could pick wrong image.

### Decision: Store photo as blob in database

**Rationale**: Contact model already has `photo_blob: Option<Vec<u8>>` field. No schema changes needed.

## Risks / Trade-offs

[Risk: Social media rate limiting] → Mitigation: Add configurable delay between requests, show progress UI

[Risk: Photo fetch fails for some URLs] → Mitigation: Show error per-URL, allow retry, continue with successful ones

[Risk: Large VCF files] → Mitigation: Load contacts incrementally, show progress during import

[Risk: Photo formats] → Mitigation: Only accept JPEG/PNG, convert to JPEG before storing

## Migration Plan

1. New main window replaces existing (if any)
2. VCF import continues to work as before (backend unchanged)
3. Photo blob storage uses existing field - no migration needed

## Open Questions

- Should contact edits persist to database immediately or on "Save" button?
- How to handle contacts that already exist (merge vs replace)?
- Should photo fetcher support mass-fetch for multiple contacts?