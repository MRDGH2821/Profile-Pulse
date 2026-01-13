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

- Document all AI-assisted changes in the `.ai/logs` folder as markdown files
- Use the naming format: `YYYY-MM-DD.md` (e.g., `2024-12-15.md`)
- Each documentation file should include:
  - The prompt or request that initiated the work
  - Description of what was done
  - Which AI model was used (e.g., Claude Sonnet 4.5, GPT-4, etc.)
- If more prompts are provided on the same day, append them to the existing log file with timestamps
- Use the `date` command to generate timestamps (e.g., `date --iso-8601=seconds` or `date '+%Y-%m-%d %H:%M:%S'`)
- Place any other relevant documents (prompts, examples, references) in the `.ai` folder
- This provides transparency and helps track AI contributions to the project

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

## AI Usage and Disclosure

**IMPORTANT**: This project maintains transparency about AI assistance.

### When Using AI Tools

1. **Document in DISCLOSURE.md**: Update [DISCLOSURE.md](DISCLOSURE.md) with:
   - Date of AI usage
   - AI model and provider
   - What was generated (files, code, documentation)
   - Nature of human involvement and review

2. **Template for Updates**:

   ```markdown
   ### [Date] - [Brief Description]

   **AI Model**: [Model name and version]
   **Generated Content**:

   - [File or feature description]

   **Human Involvement**:

   - [What decisions were made]
   - [How output was reviewed/modified]
   ```

3. **Commit Message**: Include AI disclosure in commit

   ```
   feat(social): implement GitHub profile fetcher

   AI-assisted implementation reviewed and tested.
   See DISCLOSURE.md for details.
   ```

### AI Assistance Guidelines

- AI can help with: boilerplate code, documentation, test cases, refactoring suggestions
- Human must: review all code, test thoroughly, make final decisions, approve changes
- Always: validate AI suggestions against project architecture and Rust best practices
- Document: all significant AI contributions in DISCLOSURE.md

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
5. Document AI usage in [DISCLOSURE.md](DISCLOSURE.md)

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

- Keep [DISCLOSURE.md](DISCLOSURE.md) current with AI usage
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
