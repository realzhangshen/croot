# Development

Contributing to croot.

## Prerequisites

- [Rust](https://rustup.rs/) 1.88+
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
make ci              # Run all CI checks (fmt, check, clippy, test)
make fix             # Auto-format code
make install-hooks   # Set up git hooks
```

## Git Hooks

- **Pre-commit**: runs `cargo fmt --check` (sub-second)
- **Pre-push**: mirrors CI with all four checks (fmt, check, clippy, test)

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

This runs all CI checks, updates the changelog, creates a git tag, and pushes it. The tag triggers GitHub Actions to build binaries and generate the demo GIF.
