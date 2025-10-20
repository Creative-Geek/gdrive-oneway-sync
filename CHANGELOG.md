# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-10-20

### Added

- Initial release of Google Drive One-Way Sync
- Real-time file synchronization from local folder to Google Drive
- Windows Service implementation for background operation
- File-based logging with automatic rotation (2MB per file, max 5 files)
- Service Account authentication for Google Drive API
- Automatic file watcher with debouncing (5 second delay)
- One-way upload functionality (local to cloud only)
- Comprehensive README with setup instructions
- Installation script (`install.bat`) for easy service deployment
- Configuration via `config.json` file
- Support for Google Drive folder ID targeting

### Features

- ✨ Real-time file monitoring and upload
- 🔄 Runs as Windows Service in background
- 📝 Detailed file-based logging
- 🔐 Secure Service Account authentication
- 💾 Optimized release builds (minimal binary size)
- 🎯 Simple one-way sync (no bidirectional complexity)

### Technical Details

- Built with Rust for performance and safety
- Uses `notify` for file system watching
- Integrates with Google Drive API v3
- Automatic error handling and recovery
- No panic unwraps - graceful error handling throughout

[Unreleased]: https://github.com/Creative-Geek/gdrive-oneway-sync/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Creative-Geek/gdrive-oneway-sync/releases/tag/v0.1.0
