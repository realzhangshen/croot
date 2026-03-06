# croot

A terminal file explorer that brings the VS Code sidebar experience to your command line — built with Rust and [Ratatui](https://ratatui.rs).

## Features

- **Git status integration** — see modified, staged, and untracked files at a glance
- **Real-time filesystem watching** — tree auto-refreshes on file changes
- **Syntax-highlighted preview** — preview files with full syntax highlighting (150+ languages)

## Pair with cmux

croot works great alongside [cmux](https://github.com/realzhangshen/cmux) for a full vibe coding setup in the terminal — file tree on one side, editor and shell on the other.

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

## Usage

```bash
croot            # Browse current directory
croot ~/projects # Browse a specific directory
```

## Configuration

Config file: `~/.config/croot/config.toml` (or `$XDG_CONFIG_HOME/croot/config.toml`)

## License

[MIT](LICENSE)
