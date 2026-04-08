# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.5] - 2026-04-08

### Added
- Streaming `fd`/`rg` search jobs with proper cancellation for long-running searches
- Richer syntax token palette covering escapes, constants, constructors, macros, and lifetimes
- Async integration coverage for search cancellation, background refresh, and cache consistency

### Changed
- Move syntax highlighting back to syntect for broader language coverage while keeping configurable ANSI token colors
- Move full tree refresh to a background thread with generation tracking
- Cache displayable tree indices and guides for faster large-tree rendering
- Split app and search internals into focused modules
- Bump Rust dependencies, GitHub Actions, and web tooling

### Fixed
- Prevent stale preview and background refresh results from overwriting newer state
- Prevent new file, new directory, and rename operations from clobbering existing paths
- Add missing semantic scope mappings and repair search imports after module extraction

## [0.5.4] - 2026-04-02

### Added
- VS Code-style grouped content search results for ripgrep matches
- Configurable `search.open_mode` to open search results in an external editor or the terminal editor
- `[syntax]` config section with per-token ANSI color customization (`[syntax.tokens.*]`)

### Changed
- Press `Enter` in global search to open the selected result in the editor; `Tab` now navigates to the file in the tree
- Content search now uses `rg --json` for more robust parsing and grouped per-file matches
- Replace syntect/two-face with tree-sitter for ANSI-native syntax highlighting — code colors now follow the terminal theme instead of using fixed RGB palettes
- Supported languages are currently Rust, JavaScript, TypeScript/TSX, JSON, and Markdown; legacy `preview.syntax_highlight` toggle still works as a fallback
- Remove `syntect`, `two-face`, `bincode`, and `onig` dependencies

### Fixed
- Shift+letter keybinding normalization no longer drops the `SHIFT` modifier
- Preview results are discarded when stale, preventing text preview desync after selection changes

## [0.5.3] - 2026-03-18

### Added
- Git diff gutter indicators in preview panel
- Bracketed paste support to prevent accidental actions from pasted text
- cmux tab-based editor opening (avoids suspending croot)

### Changed
- Split "Open in Editor" context menu into two actions: "Open in Editor" and "Open in cmux Tab"
- Bump CI actions (checkout v6, setup-go v6, upload-pages-artifact v4)
- Bump MSRV to 1.90

### Fixed
- cmux double-quoting bug and error visibility on editor fallback
- Click-to-preview broken when preview panel is hidden
- Display-width bugs, state sync, Unicode sorting, and robustness (16 issues)
- CI: allow NCSA license, remove stale advisory

## [0.5.2] - 2026-03-16

### Added
- Image preview support using ratatui-image (behind `image-preview` feature flag)
- Configurable trash/delete behavior (move to trash vs permanent delete)
- Integration test suite
- Terminal size caching for reduced syscall overhead

### Changed
- Branch switch is now non-blocking (uses spawn_blocking + channel)
- Context menu: "Copy Path" renamed to "Copy Relative Path" with split entries

### Fixed
- Raw mode leak when `App::new` fails — terminal now always restored
- Unicode panic on case-folding byte length changes (e.g., İ→i̇)
- TOCTOU: bounded file reads via `file.take()` to prevent unbounded allocation
- Search highlighting returning byte indices instead of char boundaries
- Path traversal via symlink escape in ConfirmDelete
- O(N²) per-frame `compact_chain_len` computation — now cached
- Cursor snap past last visible node causing potential OOB panic
- Binary preview silently swallowing read errors
- Filter ancestor walk picking wrong parent in sibling trees
- Separator hit detection off-by-one in mouse handler
- Byte-width vs display-width in Markdown tables and centered messages
- Blocking `thread::sleep` in editor suspend replaced with status bar error

## [0.5.1] - 2026-03-14

### Added
- Shell completions subcommand via clap_complete
- Comprehensive tests for render_md.rs (17 tests) and tree/loader.rs (7 tests)

### Fixed
- UTF-8 panics: replace byte-slicing with unicode-aware truncation in search bar, global search, picker, and input dialog
- Cursor rendered at byte offset instead of display column in search/input components
- Path traversal in file dialogs (new/rename) via component-based normalization and symlink-aware validation
- Global search scroll-down not adjusting scroll_offset to keep selection visible
- Editor command parsing failing on quoted paths with spaces (now uses shell_words::split)
- Config `set` silently corrupting non-table intermediate values
- Stale search state after file operations (create/rename/delete)
- Global search confirm not updating preview panel
- Context menu ignoring user keybindings and only using hard-coded defaults
- Duplicate `execute_menu_action_sync` causing TogglePreview via mouse to skip preview load
- fd/rg config commands not parsed correctly when containing flags (e.g., "fd --hidden")
- Silent error swallowing in file operations — errors now surface in status bar
- Config parse errors silently falling back to defaults without warning
- Global search overlay crash on tiny terminals (<10x6)
- Dialog width arithmetic overflow on narrow terminals
- Duplicate keybinding conflicts now detected and warned

### Changed
- Improved self-update message for non-Homebrew users

## [0.5.0] - 2026-03-14

### Added
- Branch picker UI (`b` to switch branches) with mouse support
- Character-level find highlighting (Yazi-style)
- Find / Filter / Global Search modes (redesigned search)
- Clickable Confirm/Cancel buttons in dialogs
- User-configurable color system via `[colors]` config section
- Match highlighting on cursor/hover rows in Find mode

### Changed
- Redesign interaction model: mouse-first with sensible keyboard defaults
- Keybindings now opt-in via config (toolbar removed)
- Popup styling: REVERSED-based adaptive styles with BOLD for light theme contrast
- Redesign file tree visuals: ANSI 16 icon colors, DIM hierarchy, status bar icons

### Removed
- Multi-select feature (select, clear, delete-selected)
- Toolbar (keybindings are opt-in via config)

### Fixed
- Branch picker panic on multi-byte UTF-8 truncation
- GlobalSearch overlay closing on mouse movement
- Overlay color bleed with explicit colors and hyperlink guard
- Popup text contrast for light terminal themes
- Unused Color import warning in picker.rs

## [0.4.1] - 2026-03-11

### Added
- "Open in $EDITOR" feature for files
- "Open Externally" feature for files
- `croot config` subcommand for CLI-based configuration management
- GitHub Pages landing page with automated demo GIF generation
- Toggle preview off when clicking the already-selected file

### Fixed
- OSC 8 hyperlink text overflow corrupting terminal display

### Changed
- Dependency bumps (ratatui 0.30, git2 0.20.4, notify 8.2.0, toml 1.0.3)
- CI action bumps (checkout v6, upload-artifact v7, download-artifact v8)
- Pre-commit/pre-push hooks and Makefile for local CI checks

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

## [0.2.4] - 2026-03-05

### Changed
- Harden release workflow (validate tag format, check tap token, fix heredoc)

### Fixed
- cargo-deny config (allow BSL-1.0/CC0-1.0, ignore unmaintained advisories)
- CI: use cargo-deny-action and bump MSRV to 1.88

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

## [0.2.2] - 2026-03-05

### Fixed
- Self-update to use Homebrew instead of cargo

## [0.2.1] - 2026-03-04

_Release-only commit (CI/packaging fix). No user-facing changes._

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

[Unreleased]: https://github.com/realzhangshen/croot/compare/v0.5.5...HEAD
[0.5.5]: https://github.com/realzhangshen/croot/compare/v0.5.4...v0.5.5
[0.5.4]: https://github.com/realzhangshen/croot/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/realzhangshen/croot/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/realzhangshen/croot/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/realzhangshen/croot/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/realzhangshen/croot/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/realzhangshen/croot/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/realzhangshen/croot/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/realzhangshen/croot/compare/v0.2.5...v0.3.0
[0.2.5]: https://github.com/realzhangshen/croot/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/realzhangshen/croot/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/realzhangshen/croot/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/realzhangshen/croot/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/realzhangshen/croot/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/realzhangshen/croot/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/realzhangshen/croot/releases/tag/v0.1.0
