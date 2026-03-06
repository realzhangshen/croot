# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-03-07

### Added
- Hover highlight on tree nodes
- OSC 8 hyperlink support for file paths
- Context menu with right-click (including empty tree space)
- File operations (create, rename, delete) via context menu
- Search functionality
- Multi-select support

### Changed
- Expand syntax highlighting to 150+ languages via two-face
- Replace hardcoded background colors with REVERSED-based adaptive styles
- Simplify README to focus on core positioning

### Fixed
- Plain text preview using faint theme color instead of terminal foreground
- Context menu width inflated by UTF-8 separator byte length
- Hover highlight and context menu contrast on dark terminals
- Color contrast and OSC 8 hyperlink rendering artifacts
- OSC 8 hyperlinks by emitting after render instead of embedding in buffer cells

### Removed
- Dead cmux preview pane code
- cmux open/preview interaction entry points

## [0.3.0] - 2026-03-05

### Changed
- Replace hardcoded RGB colors with ANSI 16 palette for terminal theme adaptation

### Fixed
- Empty Nerd Font icons by using `\u{xxxx}` Unicode escapes
- Run `brew update` before upgrade in self-update to refresh tap

## [0.2.5] - 2026-03-05

### Added
- Draggable separator between tree and preview panes (ratio clamped 20%-80%)
- Markdown rendering preview with pulldown-cmark
- Preview re-renders on terminal resize to re-wrap content at new width

## [0.2.3] - 2026-03-05

### Changed
- Precompute tree connector guides in O(N) instead of O(D×N) per node
- Use HashSet for expanded-path lookup in refresh() for O(1) lookups
- Cache file/dir counts on FileTree to eliminate per-frame traversal
- Add mtime caching to skip redundant preview reloads on filesystem events
- Move GitStatus enum to dedicated git::types module
- Replace 8-parameter FileTree constructor with TreeConfig struct
- Split monolithic handle_action into focused sub-handlers
- Widgets accept config references instead of individual fields
- Extract layout types (FocusPane, PreviewLayout) to layout module
- Extract file watcher to dedicated watcher module
- Move apply_git_statuses to GitState::apply_to_nodes method

## [0.2.0] - 2026-03-04

### Added
- Built-in file preview panel with syntax highlighting (replaces external `bat`/`cat` dispatch)
- Mouse text selection and Command+C copy support in preview panel
- Compact folder display for single-child directory chains
- Info columns (file size, modification date) in tree view
- Staged file git status colors
- Ghostty terminal theme detection
- Git ignored file display with status visualization
- Project quality tooling: `rustfmt.toml`, `cargo-deny` config, Dependabot
- CI security audit job and MSRV (1.75) check job
- Linux x86_64 and aarch64 release targets

### Changed
- Refactored keyboard scroll routing to dispatch by focus at entry point
- Enabled clippy pedantic lints across the codebase
- Applied code quality fixes: `format!` captures, `f64::from()` casts, `map_or_else`, lifetime elision, `unwrap_or_default()`, module ordering

### Fixed
- Compact chain detection to correctly skip subtrees when checking for siblings
- All files appearing grey (DIM) in clean git repos
- rustfmt formatting in git status tests

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

[Unreleased]: https://github.com/realzhangshen/croot/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/realzhangshen/croot/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/realzhangshen/croot/compare/v0.2.5...v0.3.0
[0.2.5]: https://github.com/realzhangshen/croot/compare/v0.2.4...v0.2.5
[0.2.3]: https://github.com/realzhangshen/croot/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/realzhangshen/croot/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/realzhangshen/croot/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/realzhangshen/croot/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/realzhangshen/croot/releases/tag/v0.1.0
