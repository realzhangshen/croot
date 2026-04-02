# Development

Contributing to croot.

## Prerequisites

- [Rust](https://rustup.rs/) 1.90+ (the repo pins `1.90.0` via `rust-toolchain.toml`)
- Git

## Setup

```bash
git clone https://github.com/realzhangshen/croot.git
cd croot
make install-hooks  # Set up pre-commit and pre-push hooks
```

## Development Workflow

croot follows **Test-Driven Development (TDD)**:

1. **Red** — write a failing test first
2. **Green** — write the minimum code to make it pass
3. **Refactor** — clean up while keeping tests green

Every new feature and bug fix starts with a test.

## Make Commands

```bash
make ci               # Run blocking CI checks (fmt, check, clippy, test)
make clippy-pedantic  # Advisory pedantic sweep with low-signal style lints filtered out
make clippy-fix       # Apply machine-fixable Clippy suggestions
make fix              # Auto-format code
make install-hooks    # Set up git hooks
```

`make ci` is the release gate. `make clippy-pedantic` is intentionally advisory
and filters out the noisiest style-only lints so new Clippy releases do not block
day-to-day work or releases.

## Git Hooks

- **Pre-commit**: runs `cargo fmt --check` (sub-second)
- **Pre-push**: mirrors the blocking CI checks (fmt, check, clippy, test)

Both are skippable with `--no-verify`.

## Testing

Tests are co-located with source code in `#[cfg(test)] mod tests` blocks:

```bash
cargo test              # Run all tests
cargo test config       # Run tests matching "config"
```

## Project Structure

```
src/
├── main.rs          # Entry point
├── app.rs           # Application state and event loop
├── config.rs        # Configuration system
├── tree/            # File tree data structures
├── input/           # Keyboard and mouse input handling
├── ui/              # TUI rendering (Ratatui)
├── git.rs           # Git status integration
├── preview.rs       # File preview
└── watcher.rs       # Filesystem watcher
```

## Release Process

```bash
make release VERSION=x.y.z
```

Before `make release`, update `Cargo.toml` and `CHANGELOG.md`, then run `make ci`.
`make release` validates the version/changelog entry, creates the git tag, and pushes it.
The tag triggers GitHub Actions to build binaries and generate the demo GIF.
