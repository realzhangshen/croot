# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Project quality tooling: `rustfmt.toml`, `cargo-deny` config, Dependabot
- CI security audit job and MSRV (1.75) check job
- Linux x86_64 and aarch64 release targets

### Changed
- Enabled clippy pedantic lints across the codebase
- Applied code quality fixes: `format!` captures, `f64::from()` casts, `map_or_else`, lifetime elision, `unwrap_or_default()`, module ordering

## [0.1.0] - 2025-01-18

### Added
- TUI file tree sidebar with real-time filesystem watching
- Git status integration with colored markers (modified, staged, untracked, ignored)
- Nerd Font icons for 100+ file types and directories
- Terminal theme detection (dark/light) for iTerm2, Terminal.app, Ghostty
- tmux/screen multiplexer bridge for seamless pane communication
- File preview dispatch (bat, cat fallback) with syntax highlighting
- Configurable via `~/.config/croot/config.toml`
- Vim-style keyboard navigation (j/k, g/G, Enter to toggle/open)
- Directory-first sorting with dotfile support
- macOS (ARM + x86_64) release binaries via GitHub Actions

### Fixed
- File tree not refreshing on filesystem changes
- Git ignored file display: removed redundant status marker, fixed directory lookup
- macOS x86_64 CI build using correct runner (macos-14)

[Unreleased]: https://github.com/realzhangshen/croot/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/realzhangshen/croot/releases/tag/v0.1.0
