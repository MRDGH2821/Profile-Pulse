# Profile Pulse rewrite — brainstorming notes

**Branch / worktree:** `mrdgh2821/docs/rewrite-planning`  
**Human plan:** [profile-pulse-app.md](./profile-pulse-app.md)  
**Updated:** 2026-08-22

## Locked decisions

| Topic                      | Choice                                                                             |
| -------------------------- | ---------------------------------------------------------------------------------- |
| UI framework               | Dioxus 0.7 (not Iced)                                                              |
| v1 platforms               | Desktop + mobile; web later                                                        |
| Live contact book          | **A** — app-owned store; OS / Google / Outlook / iCloud are adapters               |
| Product focus (human plan) | Sync profile pics from social platforms into contacts; import/export/sync adapters |

## Open (next question)

| Topic                    | Options discussed                                                                            |
| ------------------------ | -------------------------------------------------------------------------------------------- |
| What is a **Profile**?   | A named book / B source binding / **C book + optional sync targets** (recommended) / D other |
| App-owned on-disk format | Single `contacts.vcf` vs vdir vs other                                                       |
| System Contacts in v1    | None vs import/export vs live sync                                                           |
| Platform fetchers for v1 | Many listed (WhatsApp…Twitch); scrape vs API; ToS risk                                       |

## Human plan summary

- Cross-platform desktop & mobile
- Primary job: scrape/fetch profile pics → apply to contacts
- Platforms: WhatsApp, Facebook, Instagram, Twitter, Discord, GitHub, GitLab, Gravatar, LinkedIn, Twitch
- Import: VCF, Google Contacts, Outlook, CardDAV
- Profiles: separate sources; per-profile settings + cache; shared pic cache OK
- UI: contact search; contact has tabs — details / editor / pic selector
- Websites: multiple links; convenience UI in pic selector
- Backups: VCF + profile import/export; scheduled; pre-write backup always
- Sync out: OS, Google, Outlook, iCloud; per-contact Sync button

## Explicitly deferred

- Full design spec / implementation plan until brainstorming finishes
- Treating old OpenSpec as gospel (human plan is the rewrite source)
