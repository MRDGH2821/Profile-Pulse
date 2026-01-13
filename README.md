# Profile Pulse 🔄

A desktop application for managing contacts with automatic social media profile picture synchronization and profile discovery.

[![Copier](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/copier-org/copier/refs/heads/master/img/badge/black-badge.json)](https://github.com/copier-org/copier)
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)
![Status](https://img.shields.io/badge/status-in%20development-yellow)

## 🎯 Overview

Profile Pulse is a cross-platform desktop application built in Rust that helps you manage your contacts while automatically keeping their profile pictures up-to-date from their social media accounts. Import your contacts from VCF files, and Profile Pulse will fetch their latest profile pictures from LinkedIn, Twitter, GitHub, Instagram, and other platforms.

## ✨ Features

### Current

- 🚧 **In Development** - Project in planning phase

### Planned

- 📇 **Contact Management** - Import, edit, and export contacts in VCF format
- 🖼️ **Auto Profile Pictures** - Automatically fetch profile pictures from social media
- 🔍 **Profile Discovery** - Find social media profiles based on contact information
- 💾 **Local Storage** - All data stored locally with SQLite
- 🔒 **Privacy First** - No cloud sync, your data stays on your device
- 🚀 **Performance** - Built in Rust for speed and efficiency
- 🌐 **Cross-Platform** - Windows, macOS, and Linux support

### Supported Platforms

- LinkedIn
- Twitter/X
- GitHub
- Facebook
- Instagram
- More coming...

## 🏗️ Technology Stack

- **Language**: Rust
- **GUI**: Iced (Pure Rust GUI framework)
- **Database**: SQLite with SQLx
- **HTTP Client**: Reqwest
- **Image Processing**: image crate
- **VCF Parsing**: vcard crate

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (install from [rustup.rs](https://rustup.rs))
- SQLite 3.x

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/profile-pulse.git
cd profile-pulse

# Build the project
cargo build --release

# Run the application
cargo run --release
```

### Development Build

```bash
# Build and run in development mode
cargo run

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## 📚 Documentation

- [Project Plan](docs/PLAN.md) - Comprehensive project planning and technology decisions
- [Architecture](docs/ARCHITECTURE.md) - Technical architecture and design patterns
- [Roadmap](docs/ROADMAP.md) - Development phases and milestones
- [API Integration](docs/API_INTEGRATION.md) - Social media API details and limitations
- [Development Guide](docs/DEVELOPMENT.md) - Setup, building, testing, and contributing

## 🗺️ Project Status

**Current Phase**: Planning & Architecture

See the [Roadmap](docs/ROADMAP.md) for detailed progress and upcoming features.

## 🤝 Contributing

Contributions are welcome! This project is in early development. Please see [DEVELOPMENT.md](docs/DEVELOPMENT.md) for guidelines.

## 📄 License

This project is licensed under the GNU General Public License v3.0 or later (GPLv3-or-later).

See [LICENSE](LICENSE) for the full license text.

This is free software: you are free to change and redistribute it under the terms of the GPL. There is NO WARRANTY, to the extent permitted by law.

## 🤖 AI Disclosure

Parts of this project's initial documentation were created with AI assistance. See [DISCLOSURE.md](DISCLOSURE.md) for full transparency about AI-generated content.

## 🔒 Privacy & Ethics

Profile Pulse is designed with privacy in mind:

- All data stored locally on your device
- No telemetry or tracking
- Respects social media platforms' terms of service
- Implements rate limiting to avoid abuse
- User confirmation required for auto-discovered profiles

## ⚠️ Disclaimer

This application fetches publicly available profile pictures. Always respect:

- Platform terms of service
- Rate limits and robots.txt
- User privacy and data protection laws (GDPR, CCPA, etc.)
- Copyright and intellectual property rights

## 🙏 Acknowledgments

Built with amazing open-source Rust crates. See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full list.

---

**Note**: Profile Pulse is under active development. Features and APIs may change.
