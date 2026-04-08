# croot

A terminal file explorer that brings the VS Code sidebar experience to your command line — built with Rust and [Ratatui](https://ratatui.rs).

**[Documentation](https://realzhangshen.github.io/croot/)** | **[Getting Started](https://realzhangshen.github.io/croot/guide/getting-started)**

## Features

- **Git status integration** — see modified, staged, and untracked files at a glance
- **Git diff gutter** — see added/modified/removed lines in the preview panel
- **Real-time filesystem watching** — tree auto-refreshes on file changes
- **ANSI-native syntax-highlighted preview** — preview Rust, JavaScript, TypeScript, JSON, and Markdown with colors that follow your terminal theme
- **Bracketed paste protection** — prevents accidental actions from pasted text

## Pair with cmux

croot works great alongside [cmux](https://github.com/manaflow-ai/cmux) for a full vibe coding setup in the terminal — file tree on one side, editor and shell on the other. Use the "Open in cmux Tab" context menu action to open files in a new cmux tab without suspending croot.

## Installation

### Homebrew (macOS)

```bash
brew install realzhangshen/croot/croot
```

### From source

```bash
git clone https://github.com/realzhangshen/croot.git
cd croot
cargo build --release
# Binary is at target/release/croot
```

### Optional: image preview

Image preview (PNG/JPG/… rendered inline in the preview pane) is an
opt-in feature because it pulls in `ratatui-image` and the `image`
crate. Enable it at install time:

```bash
cargo install croot --features image-preview
# or when building from source:
cargo build --release --features image-preview
```

## Usage

```bash
croot            # Browse current directory
croot ~/projects # Browse a specific directory
```

## Configuration

Config file: `~/.config/croot/config.toml` (or `$XDG_CONFIG_HOME/croot/config.toml`)

The built-in default palette is ANSI-only and tuned for a light, higher-contrast popup/input experience inspired by Cursor Light in Ghostty.
Add a `[colors]` section only if you want to override those defaults for your own terminal theme.
Syntax highlighting also uses ANSI/indexed colors via `[syntax.tokens.*]`, so code colors follow the terminal theme instead of a fixed RGB code theme.

## Development

```bash
make install-hooks   # Set up pre-commit and pre-push hooks
make ci              # Run all CI checks locally (fmt, check, clippy, test)
make fix             # Auto-format code
```

The pre-commit hook runs `cargo fmt --check` (sub-second). The pre-push hook mirrors CI with all four checks. Both are skippable with `--no-verify`.

## License

[MIT](LICENSE)
