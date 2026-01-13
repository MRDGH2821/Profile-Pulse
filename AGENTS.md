# AGENTS Instructions

This file provides guidance for AI coding assistants working with this project.

## Project Context

- **Project Name**: Profile Pulse
- **Project Type**: Desktop contact management application with social media integration
- **Language**: Rust
- **GUI Framework**: Iced (pure Rust)
- **Database**: SQLite with SQLx
- **License**: GPLv3-or-later
- **Status**: Planning phase (no code implementation yet)

## Project Documentation

Before making changes, review the comprehensive documentation in the `docs/` directory:

- **[docs/PLAN.md](docs/PLAN.md)** - Technology stack, architecture decisions, and implementation strategy
- **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System architecture, data models, and design patterns
- **[docs/ROADMAP.md](docs/ROADMAP.md)** - Development phases, tasks, and milestones
- **[docs/API_INTEGRATION.md](docs/API_INTEGRATION.md)** - Social media platform integration guides
- **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** - Development setup, testing, and contribution guidelines

## General Guidelines

### Communication

- Explain what you're doing and why before making changes
- Ask for clarification when requirements are ambiguous
- Provide context for decisions, especially when multiple approaches exist
- Reference relevant documentation when making architectural decisions

### Code Quality

- Follow existing code style and conventions in the project
- Run linters and formatters before committing changes
- Ensure all changes pass pre-commit hooks
- Write tests for new functionality
- Follow Rust best practices and idioms

### File Operations

- Always check if a file exists before attempting to modify it
- Use appropriate tools to search for files rather than guessing paths
- Preserve file formatting and structure unless explicitly asked to change it

## Rust-Specific Guidelines

### Code Style

- Follow `rustfmt` formatting (run `cargo fmt`)
- Address all `clippy` warnings (run `cargo clippy`)
- Use meaningful variable and function names (snake_case)
- Type names in PascalCase, constants in SCREAMING_SNAKE_CASE
- Document public APIs with doc comments (`///`)

### Architecture Principles

- Refer to [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for:
  - Layered architecture (UI, Business Logic, Service, Data Access)
  - Design patterns (Repository, Trait Objects, Builder, Strategy)
  - Error handling strategy (thiserror + anyhow)
  - Security considerations

### Dependencies

- Check [docs/PLAN.md](docs/PLAN.md) for approved crates
- New dependencies should align with project architecture
- Justify dependency additions with specific use cases
- Consider alternatives and trade-offs

## Development Workflow

### Before Starting Work

1. Review the [ROADMAP.md](docs/ROADMAP.md) to understand current phase
2. Check [ARCHITECTURE.md](docs/ARCHITECTURE.md) for design patterns
3. Review [DEVELOPMENT.md](docs/DEVELOPMENT.md) for setup instructions
4. Understand the module structure and responsibilities

### AI-Assisted Work Documentation

**IMPORTANT**: This project maintains full transparency about AI assistance through work logs.

**All AI-assisted work must be documented** in `.ai/logs/YYYY-MM-DD.md` files:

- **Naming format**: `YYYY-MM-DD.md` (e.g., `2024-12-15.md`)
- **Multiple sessions per day**: Append to the existing log file with timestamps
- **Generate timestamps**: Use `date --iso-8601=seconds` or `date '+%Y-%m-%d %H:%M:%S'`

**Each log entry must include**:

1. **Timestamp** - When the work was performed
2. **Request/Prompt** - What initiated the work (user request or task description)
3. **AI Model** - Model name and version (e.g., Claude Sonnet 4.5, GPT-4, etc.)
4. **Provider** - AI provider (e.g., Anthropic, OpenAI)
5. **Work Performed** - Detailed description of what was done
6. **Files Changed** - List of files created/modified with line counts
7. **Nature of Assistance** - Type of help (code generation, documentation, refactoring, debugging, etc.)
8. **Human Involvement** - What decisions were made by humans, how output was reviewed/tested/modified, what was rejected or changed
9. **Testing Status** - Whether code was tested, compilation status, test results

**Example log entry format**:

```markdown
## 2024-12-15 14:30:22+00:00

### Request

User asked to implement GitHub profile fetcher with rate limiting

### AI Model

**Model**: Claude Sonnet 4.5
**Provider**: Anthropic

### Work Performed

- Implemented GitHubFetcher struct with async trait
- Added rate limiting using governor crate
- Created comprehensive error handling
- Added unit tests and integration tests

### Files Changed

- `src/social/github.rs` (created, ~250 lines)
- `tests/integration/github_tests.rs` (created, ~80 lines)
- `Cargo.toml` (modified, added governor dependency)

### Nature of Assistance

- Code generation for fetcher implementation
- Test case generation
- Error handling patterns

### Human Involvement

- Reviewed all generated code for correctness
- Modified rate limiting to be more conservative (5 req/min instead of 10)
- Added additional error cases not covered by AI
- Tested with real GitHub API
- Approved final implementation after modifications

### Testing Status

- ✅ Compiled successfully
- ✅ All 12 unit tests passing
- ✅ Integration tests passing with mock API
- ⏳ Manual testing with real API pending
```

**Additional Materials**: Place any other relevant documents (prompts, examples, references, generated docs) in the `.ai` folder

**Commit Message Format**: Reference the work log in commit messages:

```
feat(social): implement GitHub profile fetcher

AI-assisted implementation reviewed and tested.
See .ai/logs/2024-12-15.md for details.
```

## Dev Environment Tips

### Development Process

- Use `--help` or `help` subcommand to get help on a command. It can even reveal hints on how to proceed ahead or optimize the number of steps.
- Check tool documentation before asking the user for configuration details

```bash
# Format code
cargo fmt

# Check for errors
cargo check

# Run linter
cargo clippy

# Run tests
cargo test

# Run all pre-commit hooks
prek --all-files
```

### Testing

- Write unit tests in the same file as the code (`#[cfg(test)] mod tests`)
- Write integration tests in `tests/` directory
- Use `#[tokio::test]` for async tests
- Mock external APIs using `mockito`
- Test database operations with SQLx test features

## Pre-commit Hooks (prek)

### Installation

- Install with `uv tool install prek` and run checks via `prek --all-files`
- Enable the hooks with `prek install --install-hooks` so they run automatically on each commit

### Working with Hooks

- If a pre-commit hook fails, read the error message carefully - it often suggests the fix
- Run `prek --all-files` before committing to catch issues early
- Some hooks auto-fix issues (like formatters); others require manual intervention

## Linting and Formatting

### MegaLinter

- Configuration is in `.mega-linter.yml`
- Run locally with: `npx mega-linter-runner --flavor documentation`
- Check reports in `megalinter-reports/` directory
- Not all linters need to pass - some are informational

### CSpell (Spell Checking)

- Configuration is in `.cspell.json`
- Add project-specific words to the `words` array
- Technical terms to add: Rust crates, API names, social media platforms
- Both file content and commit messages are spell-checked

### Prettier

- Configuration is in `.prettierrc.json`
- Formats markdown, JSON, YAML files
- Auto-fixes on pre-commit

## Commit Messages

### Format

- Follow Conventional Commits format: `<type>(<scope>): <description>` as given here - https://www.conventionalcommits.org/en/v1.0.0/
- Valid types: `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, `test`
- Valid scopes: `ui`, `core`, `db`, `social`, `discovery`, `deps`, `zed`, `vscode`, `cspell`, `megalinter`, `precommit`
- For additional scopes, refer `conventional-pre-commit` hook in `.pre-commit-config.yaml`. It has additional scopes and is the source of truth.

### Examples

```txt
feat(ui): add contact list view with Iced
fix(db): resolve SQLite connection pool deadlock
docs(api): update LinkedIn integration guide
refactor(social): extract common fetcher logic to trait
test(core): add contact validation tests
chore(deps): update sqlx to 0.7.3
```

## AI Usage and Transparency

**IMPORTANT**: This project maintains full transparency about AI assistance.

### Documentation Requirements

All AI-assisted work must be documented as described in the "AI-Assisted Work Documentation" section above. Every AI session requires creating or updating the daily log file in `.ai/logs/YYYY-MM-DD.md`.

### AI Assistance Guidelines

**AI can help with**:

- Boilerplate code and scaffolding
- Documentation and comments
- Test cases and test data
- Refactoring suggestions
- Bug fixes and debugging
- Code review and optimization suggestions
- Research and best practices

**Human must**:

- Review all AI-generated code thoroughly
- Test all functionality comprehensively
- Make final decisions on architecture and approach
- Approve all changes before committing
- Understand the code (never commit code you don't understand)

**Always**:

- Validate AI suggestions against project architecture (see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md))
- Follow Rust best practices and idioms
- Ensure code passes all tests and linters
- Document the AI assistance in `.ai/logs/`
- Include human review notes in the log

**Never**:

- Commit AI-generated code without review
- Use AI-generated code you don't understand
- Skip testing because "AI wrote it"
- Forget to document AI usage
- Rely solely on AI for architectural decisions

## Troubleshooting

### Common Issues

**Pre-commit hooks failing on commit:**

- Run `prek --all-files` to see all issues at once
- Fix formatting issues first (Prettier, whitespace)
- Then address spell checking and linting

**Rust compilation errors:**

- Check [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for system dependencies
- Run `cargo check` for detailed error messages
- Ensure Rust toolchain is up to date: `rustup update`

**Database issues:**

- Review [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for schema
- Run migrations: `sqlx migrate run`
- Check connection string in `.env` file

**Spell check failures:**

- Add legitimate technical terms to `.cspell.json` `words` array
- Add Rust-specific terms: crate names, function names, etc.
- Use proper capitalization for proper nouns

### Getting Help

- Most tools support `--help` flag for detailed usage
- Check tool documentation before modifying configurations
- Review existing configuration files for examples
- Consult [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for troubleshooting guide

## Best Practices

### Before Making Changes

1. Review relevant documentation in `docs/`
2. Understand current phase from [ROADMAP.md](docs/ROADMAP.md)
3. Check if similar functionality already exists
4. Consider impact on architecture and future features

### When Adding Features

1. Follow the phased approach in [ROADMAP.md](docs/ROADMAP.md)
2. Implement according to [ARCHITECTURE.md](docs/ARCHITECTURE.md) design
3. Write tests alongside implementation
4. Update documentation as needed
5. Document AI usage in `.ai/logs/YYYY-MM-DD.md`

### When Integrating APIs

1. Consult [docs/API_INTEGRATION.md](docs/API_INTEGRATION.md) for platform specifics
2. Implement rate limiting per platform guidelines
3. Add caching to minimize requests
4. Handle errors gracefully with retry logic
5. Respect platform Terms of Service

### Testing Changes

- Run all pre-commit hooks: `prek --all-files`
- Verify tests pass: `cargo test`
- Check clippy: `cargo clippy`
- Format code: `cargo fmt`
- Ensure documentation is updated

### Documentation Updates

- Keep `.ai/logs/` current with all AI usage (required for every session)
- Update [ROADMAP.md](docs/ROADMAP.md) task checkboxes as work completes
- Modify [ARCHITECTURE.md](docs/ARCHITECTURE.md) if design changes
- Update [DEVELOPMENT.md](docs/DEVELOPMENT.md) if workflow changes

## Privacy and Ethics

This project handles personal data (contact information). Always:

- Follow privacy-by-design principles (see [ARCHITECTURE.md](docs/ARCHITECTURE.md))
- Store all data locally (no cloud sync without consent)
- Respect social media platform Terms of Service
- Implement rate limiting to avoid abuse
- Get user consent for external API calls
- Review [docs/API_INTEGRATION.md](docs/API_INTEGRATION.md) for legal considerations

## License Compliance

- Project is licensed under **GPLv3-or-later**
- All contributions must be compatible with GPL
- Include license headers in source files
- Document third-party dependencies and their licenses
- Review LICENSE file for full terms

---

**Remember**: This is a transparency-first project. Always document AI usage, follow the architecture, and maintain high code quality standards.
