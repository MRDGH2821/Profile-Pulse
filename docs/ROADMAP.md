# Development Roadmap: Profile Pulse

## Overview

This roadmap outlines the development phases for Profile Pulse, from initial setup through production release. Each phase builds incrementally on previous work, with regular testing and documentation updates.

**Project Timeline**: ~12 weeks (3 months)  
**Current Phase**: 🎯 Phase 2 - VCF Import/Export (Completed) → Phase 3 - Profile Discovery (Next)  
**Target Release**: Q2 2024

## Legend

- ✅ Completed
- 🚧 In Progress
- ⏭️ Planned
- ⏸️ Blocked
- ❌ Cancelled

---

## Phase 0: Planning & Setup (Week 0) ✅

**Goal**: Complete project planning and initial setup

**Duration**: 1 week

### Tasks

- [x] Define project scope and requirements
- [x] Choose technology stack
- [x] Create documentation structure
- [x] Write project plan (PLAN.md)
- [x] Write architecture documentation (ARCHITECTURE.md)
- [x] Write this roadmap (ROADMAP.md)
- [x] Write API integration guide (API_INTEGRATION.md)
- [x] Write development guide (DEVELOPMENT.md)
- [x] Set up Git repository structure
- [x] Configure pre-commit hooks
- [ ] Set up CI/CD pipeline (GitHub Actions)

**Deliverable**: Complete project documentation and repository setup

---

## Phase 1: Foundation (Weeks 1-2) ✅

**Goal**: Basic application structure with local storage and UI

**Duration**: 2 weeks

### Week 1: Project Setup & Database

#### Tasks

- [x] Initialize Cargo project
- [x] Configure Cargo.toml with dependencies
- [x] Set up project directory structure
- [x] Create SQLite schema
- [x] Write SQLx migrations
- [x] Implement database connection pool
- [x] Create Contact model
- [x] Create SocialProfile model
- [x] Implement ContactRepository (CRUD)
- [x] Write unit tests for repository

#### Acceptance Criteria

- [x] Project compiles without errors
- [x] Database migrations run successfully
- [x] Can create, read, update, delete contacts
- [x] All repository tests pass

### Week 2: Basic UI with Iced

#### Tasks

- [x] Set up Iced application structure
- [x] Implement main window and layout
- [x] Create contact list view
- [x] Create add/edit contact form
- [x] Implement basic navigation
- [x] Connect UI to repository layer
- [x] Add basic error handling
- [x] Implement contact search/filter
- [x] Write integration tests
- [x] **NEW**: Implement alphabetical pagination (A-Z filter)
- [x] **NEW**: Remove 100-contact limit (load all contacts)
- [x] **NEW**: Add comprehensive field support (Google Contacts compatible)
- [x] **NEW**: Implement multiple emails with add/remove buttons
- [x] **NEW**: Implement multiple phones with add/remove buttons
- [x] **NEW**: Implement multiple URLs with add/remove buttons
- [x] **NEW**: Add nickname, birthday, notes, department fields
- [x] **NEW**: Add social profile management in forms
- [x] **NEW**: Improve detail view to show all fields

#### Acceptance Criteria

- [x] Application launches and displays window
- [x] Can add new contacts via UI
- [x] Can view list of contacts
- [x] Can edit existing contacts
- [x] Can delete contacts with confirmation
- [x] Search filters contact list
- [x] No crashes on basic operations
- [x] **NEW**: Can filter contacts by first letter (A-Z)
- [x] **NEW**: All 300+ contacts display correctly with pagination
- [x] **NEW**: Can add/edit multiple emails, phones, and URLs
- [x] **NEW**: All Google Contacts fields are supported
- [x] **NEW**: URLs are preserved for profile picture fetching

**Phase 1 Deliverable**: Working desktop app with local contact management and comprehensive field support

---

## Phase 2: VCF Import/Export (Week 3) ✅

**Goal**: Import and export contacts in VCF format

**Duration**: 1 week

### Tasks

- [x] Research VCF/vCard format versions (2.1, 3.0, 4.0)
- [x] Integrate vcard crate or implement parser (custom parser implemented)
- [x] Implement VCF import functionality
- [x] Map VCF fields to Contact model
- [x] Extract social media URLs from VCF
- [x] Handle parsing errors gracefully
- [x] Implement VCF export functionality
- [x] Add UI for import/export (Import/Export buttons with file dialogs)
- [x] Test with real VCF files from various sources
- [x] Handle bulk import (multiple contacts)
- [x] Write comprehensive tests (6 VCF tests added)

### VCF Sources to Test

- [x] Google Contacts (primary target - comprehensive field support added)
- [ ] Apple Contacts (to be tested)
- [ ] Outlook (to be tested)
- [ ] Android Contacts (to be tested)
- [x] Custom VCF files

### Acceptance Criteria

- [x] Can import VCF files with 90%+ field accuracy
- [x] Social media URLs extracted correctly
- [x] Can export contacts to valid VCF 4.0
- [x] Handles malformed VCF gracefully
- [x] Supports batch import
- [x] Success/error messages shown to user

**Phase 2 Deliverable**: Import existing contacts from VCF files and export to VCF

---

## Phase 3: Profile Picture Fetching (Weeks 4-6) ⏭️

**Goal**: Automatically fetch profile pictures from social media

**Duration**: 3 weeks

### Week 4: Infrastructure

#### Tasks

- [ ] Design ProfileFetcher trait
- [ ] Implement HTTP client with retry logic
- [ ] Implement rate limiter (token bucket)
- [ ] Create cache service (in-memory + DB)
- [ ] Implement fetch queue system
- [ ] Create error types for fetching
- [ ] Add logging and telemetry
- [ ] Write tests for infrastructure

#### Acceptance Criteria

- [ ] Rate limiter prevents excessive requests
- [ ] Cache reduces duplicate requests
- [ ] Retry logic handles transient failures
- [ ] Queue processes fetches in order

### Week 5: Platform Implementations

#### Tasks

- [ ] Implement GitHubFetcher
  - [ ] Profile picture API
  - [ ] Search by name/email
  - [ ] Handle rate limits (60/hour unauth)
- [ ] Implement LinkedInFetcher
  - [ ] Web scraping approach
  - [ ] Profile picture extraction
  - [ ] Conservative rate limits
- [ ] Implement TwitterFetcher
  - [ ] API integration (if available)
  - [ ] Fallback to scraping
  - [ ] Handle authentication
- [ ] Write platform-specific tests
- [ ] Add mock responses for testing

#### Acceptance Criteria

- [ ] GitHub fetcher works reliably
- [ ] LinkedIn fetcher extracts pictures
- [ ] Twitter fetcher handles both API and scraping
- [ ] All fetchers respect rate limits
- [ ] Tests cover happy path and errors

### Week 6: Additional Platforms & UI

#### Tasks

- [ ] Research Instagram API/scraping approach
- [ ] Implement InstagramFetcher (if feasible)
- [ ] Research Facebook Graph API
- [ ] Implement FacebookFetcher (if feasible)
- [ ] Add fetch UI controls
- [ ] Implement progress indicators
- [ ] Add batch fetch functionality
- [ ] Implement image processing (resize, compress)
- [ ] Add profile picture display in UI
- [ ] Handle fetch failures in UI
- [ ] Write end-to-end tests

#### Acceptance Criteria

- [ ] Can fetch from at least 3 platforms
- [ ] UI shows fetch progress
- [ ] Images display correctly
- [ ] Errors shown to user
- [ ] Can batch fetch for multiple contacts
- [ ] Images cached and compressed

**Phase 3 Deliverable**: Automatic profile picture fetching from multiple platforms

---

## Phase 4: Profile Discovery (Weeks 7-9) ⏭️

**Goal**: Discover social media profiles automatically

**Duration**: 3 weeks

### Week 7: Search Infrastructure

#### Tasks

- [ ] Research search APIs (Google Custom Search, Bing, etc.)
- [ ] Implement search service abstraction
- [ ] Integrate Google Custom Search API
- [ ] Implement query builder
- [ ] Parse search results
- [ ] Extract candidate profiles
- [ ] Add search caching
- [ ] Write tests for search service

#### Acceptance Criteria

- [ ] Can search for profiles by name
- [ ] Extracts candidate URLs
- [ ] Handles API errors
- [ ] Caches search results
- [ ] Tests cover various queries

### Week 8: Profile Matching

#### Tasks

- [ ] Design matching algorithm
- [ ] Implement name similarity (Jaro-Winkler)
- [ ] Implement email domain matching
- [ ] Implement location matching
- [ ] Implement company matching
- [ ] Create confidence scoring system
- [ ] Tune matching thresholds
- [ ] Test with real data
- [ ] Handle edge cases
- [ ] Write comprehensive tests

#### Test Cases

- [ ] Exact name match
- [ ] Name with middle initial
- [ ] Name variations (nicknames)
- [ ] Common names (requires additional signals)
- [ ] International names
- [ ] Company name matching

#### Acceptance Criteria

- [ ] Matching accuracy > 85%
- [ ] False positive rate < 5%
- [ ] Confidence scores correlate with accuracy
- [ ] Handles edge cases gracefully

### Week 9: Discovery UI & Workflow

#### Tasks

- [ ] Design discovery UI mockup
- [ ] Implement discovery view
- [ ] Show candidate profiles with confidence scores
- [ ] Add confirm/reject workflow
- [ ] Implement bulk discovery
- [ ] Add manual URL entry
- [ ] Show discovery progress
- [ ] Store discovery metadata
- [ ] Add discovery history
- [ ] Write UI integration tests

#### Acceptance Criteria

- [ ] UI clearly shows match confidence
- [ ] User can confirm/reject matches
- [ ] Can discover profiles for multiple contacts
- [ ] Progress indicated clearly
- [ ] Confirmed matches saved to contact

**Phase 4 Deliverable**: Automatic social media profile discovery with user confirmation

---

## Phase 5: Polish & Distribution (Weeks 10-12) ⏭️

**Goal**: Production-ready application with distributions

**Duration**: 3 weeks

### Week 10: Feature Completion

#### Tasks

- [ ] Implement settings/preferences
  - [ ] Rate limit configuration
  - [ ] Cache settings
  - [ ] Default import/export location
  - [ ] Privacy settings
- [ ] Add keyboard shortcuts
- [ ] Implement contact merge functionality
- [ ] Add duplicate detection
- [ ] Implement contact groups/tags
- [ ] Add export options (CSV, JSON)
- [ ] Improve error messages
- [ ] Add tooltips and help text
- [ ] Write user documentation

#### Acceptance Criteria

- [ ] Settings persist across sessions
- [ ] Keyboard shortcuts work
- [ ] Can merge duplicate contacts
- [ ] Export formats work correctly
- [ ] User documentation complete

### Week 11: Testing & Bug Fixes

#### Tasks

- [ ] Comprehensive integration testing
- [ ] Performance testing
  - [ ] Test with 10,000 contacts
  - [ ] Measure memory usage
  - [ ] Optimize slow operations
- [ ] Cross-platform testing
  - [ ] Windows 10/11
  - [ ] macOS 12+
  - [ ] Ubuntu 22.04 LTS
  - [ ] Fedora 39
- [ ] Fix discovered bugs
- [ ] Security audit
- [ ] Code review
- [ ] Update documentation

#### Acceptance Criteria

- [ ] No critical bugs
- [ ] Performance acceptable on all platforms
- [ ] Memory usage under 500MB with 10k contacts
- [ ] All platforms tested
- [ ] Security issues addressed

### Week 12: Packaging & Release

#### Tasks

- [ ] Set up GitHub releases
- [ ] Create release notes template
- [ ] Package for Windows (MSI installer)
  - [ ] Code signing certificate
  - [ ] Windows Defender exclusion
- [ ] Package for macOS (DMG/app bundle)
  - [ ] Code signing and notarization
  - [ ] Universal binary (Intel + ARM)
- [ ] Package for Linux
  - [ ] AppImage
  - [ ] Debian package (.deb)
  - [ ] RPM package
  - [ ] Flatpak (optional)
- [ ] Create installation instructions
- [ ] Write release announcement
- [ ] Set up update mechanism
- [ ] Create demo video
- [ ] v1.0.0 release

#### Deliverables

- [ ] Windows installer (MSI)
- [ ] macOS app bundle (DMG)
- [ ] Linux packages (AppImage, deb, rpm)
- [ ] Release notes
- [ ] Installation guide
- [ ] User guide
- [ ] Demo video

**Phase 5 Deliverable**: Production release v1.0.0 with installers for all platforms

---

## Post-Release Roadmap (Future)

### Version 1.1 (Q3 2024)

**Enhancements**:

- [ ] Dark mode support
- [ ] Contact notes and history
- [ ] Birthday reminders
- [ ] Custom fields UI
- [ ] Improved search (fuzzy matching)
- [ ] Contact import from social media
- [ ] Profile picture update notifications

### Version 1.2 (Q4 2024)

**New Features**:

- [ ] Contact synchronization (optional cloud sync)
- [ ] Mobile companion app
- [ ] Advanced duplicate detection
- [ ] Contact backup/restore
- [ ] Export to various CRM formats
- [ ] Plugin system for custom fetchers
- [ ] API for integrations

### Version 2.0 (2025)

**Major Features**:

- [ ] Team/shared contacts
- [ ] Contact relationship mapping
- [ ] AI-powered profile matching
- [ ] Automatic contact enrichment
- [ ] Analytics and insights
- [ ] Calendar integration
- [ ] Email integration

---

## Risk Management

### High Priority Risks

| Risk                                     | Probability | Impact | Mitigation                                    |
| ---------------------------------------- | ----------- | ------ | --------------------------------------------- |
| Social media APIs change/restrict access | High        | High   | Use scraping fallbacks, modular design        |
| Rate limiting too aggressive             | Medium      | Medium | Implement queue system, conservative defaults |
| Poor profile matching accuracy           | Medium      | High   | Extensive testing, tunable thresholds         |
| Cross-platform bugs                      | Medium      | Medium | CI/CD on all platforms, beta testing          |
| Legal/ToS concerns                       | Low         | High   | Clear disclaimers, respect ToS, legal review  |

### Medium Priority Risks

| Risk                                       | Probability | Impact | Mitigation                             |
| ------------------------------------------ | ----------- | ------ | -------------------------------------- |
| Performance issues with large datasets     | Medium      | Medium | Optimization, pagination, lazy loading |
| UI/UX not intuitive                        | Low         | Medium | User testing, iterative design         |
| Installation/packaging issues              | Low         | Medium | Test on clean systems, good docs       |
| Dependencies with security vulnerabilities | Low         | Medium | Regular audits, automated scanning     |

---

## Success Metrics

### Phase 1 Success

- [ ] Application runs on all target platforms
- [ ] Can manage 1,000 contacts without issues
- [ ] Zero critical bugs

### Phase 2 Success

- [ ] 95%+ VCF import success rate
- [ ] Supports VCF from 5+ sources

### Phase 3 Success

- [ ] Successfully fetches from 3+ platforms
- [ ] 90%+ fetch success rate
- [ ] Respects all rate limits

### Phase 4 Success

- [ ] 85%+ matching accuracy
- [ ] <5% false positive rate
- [ ] Users find 3+ new profiles on average

### Phase 5 Success

- [ ] Successful installations on all platforms
- [ ] No critical bugs in release
- [ ] Positive user feedback
- [ ] 100 downloads in first month

---

## Resource Requirements

### Development Tools

- Rust toolchain (rustup)
- SQLite tools
- Git
- IDE (VS Code, RustRover, etc.)
- API keys for testing:
  - Google Custom Search API
  - Twitter API (if available)
  - GitHub personal access token

### Testing Resources

- Windows 10/11 VM or hardware
- macOS system (Intel and/or ARM)
- Linux VMs (Ubuntu, Fedora)
- Test VCF files from various sources
- Test social media accounts

### Distribution

- Code signing certificates (Windows, macOS)
- GitHub account for releases
- Domain name (optional, for website)

---

## Review Schedule

- **Weekly**: Review progress against roadmap
- **End of Phase**: Complete phase retrospective
- **Monthly**: Update roadmap based on learnings
- **Pre-release**: Final roadmap review and v1.1 planning

---

**Document Version**: 1.0  
**Last Updated**: 2024-01-XX  
**Next Review**: After Phase 1 completion  
**Maintained By**: Project Lead

---

## Notes

- Timeline assumes solo developer working part-time (~20 hours/week)
- Adjust timeline if working full-time or with team
- Some platform integrations (Instagram, Facebook) may be dropped if technically infeasible
- Focus on quality over feature count - better to ship fewer features well
- User feedback after Phase 3 may shift priorities
