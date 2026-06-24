# VCF Contact Photo Sync - Tasks

## 1. Project Setup

- [x] 1.1 Add required dependencies (reqwest for HTTP, image for processing)
- [x] 1.2 Create GUI module structure (src/gui/, src/social/)
- [x] 1.3 Add logging setup for photo fetcher debugging

## 2. Main Window with Tabs

- [x] 2.1 Create main window with tabbed interface (Import, Contacts, Photo Fetcher)
- [x] 2.2 Implement Import tab view with file picker
- [x] 2.3 Add native file dialog integration
- [x] 2.4 Wire up VCF import to main window

## 3. Contact List View

- [x] 3.1 Create contact list component
- [x] 3.2 Display contact names from imported VCF
- [x] 3.3 Make list scrollable for many contacts
- [x] 3.4 Add selection handler (single click highlights)
- [x] 3.5 Add contact detail open action

## 4. Contact Detail Edit

- [x] 4.1 Create contact detail window/view
- [x] 4.2 Display all name fields (prefix, first, middle, last, suffix)
- [x] 4.3 Display organization, title, department, nickname, notes
- [x] 4.4 Display all emails with labels
- [x] 4.5 Display all phones with labels
- [x] 4.6 Display all addresses with labels
- [x] 4.7 Display all dates with labels
- [x] 4.8 Display all URLs with labels
- [x] 4.9 Display existing photo if present
- [x] 4.10 Add Edit button to enable edit mode
- [x] 4.11 Add Save/Cancel buttons in edit mode
- [x] 4.12 Implement add/remove for repeatable fields

## 5. Photo Fetcher

- [x] 5.1 Create photo fetcher module
- [x] 5.2 Extract URLs from contact
- [x] 5.3 Add GitHub profile photo fetcher
- [x] 5.4 Add Twitter/X profile photo fetcher
- [x] 5.5 Add LinkedIn profile photo fetcher
- [x] 5.6 Implement generic URL image fetcher
- [x] 5.7 Add progress display during fetch
- [x] 5.8 Handle fetch errors per URL
- [x] 5.9 Add retry functionality

## 6. Photo Save

- [ ] 6.1 Display fetched photos as thumbnails
- [ ] 6.2 Add photo selection UI
- [ ] 6.3 Add larger preview on click
- [x] 6.4 Implement save to contact photo_blob
- [x] 6.5 Validate image format (accept JPEG/PNG only)
- [x] 6.6 Handle photo replacement

## 7. Integration & Testing

- [x] 7.1 Wire Photo Fetcher tab to contact detail
- [ ] 7.2 Test full import-fetch-save workflow
- [x] 7.3 Test error handling (no URLs, fetch failures)
- [ ] 7.4 Test contact edit persistence
- [ ] 7.5 Run existing VCF tests to ensure no regression