# croot

A lightweight terminal file tree sidebar, built with Rust and [Ratatui](https://ratatui.rs).

`croot` gives you a vim-style navigable file tree in your terminal, with git status integration, file preview via tmux, and real-time filesystem watching.

## Features

- Vim-style navigation (`hjkl`)
- Git status indicators (modified, staged, untracked, etc.)
- Auto file preview in a tmux split pane
- Real-time filesystem watching (auto-refresh on changes)
- Respects `.gitignore` rules
- Configurable via TOML

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

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `Down` | Move cursor down |
| `k` / `Up` | Move cursor up |
| `h` / `Left` | Collapse directory |
| `l` / `Right` | Expand directory |
| `Space` / `Tab` | Toggle expand/collapse |
| `Enter` | Open file (preview in tmux pane) |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `PageUp` / `PageDown` | Scroll by 10 lines |
| `r` | Refresh tree |
| `q` / `Ctrl+C` | Quit |

## Configuration

croot reads configuration from `~/.config/croot/config.toml` (or `$XDG_CONFIG_HOME/croot/config.toml`).

Example with all defaults:

```toml
[tree]
show_hidden = true
show_ignored = true
dirs_first = true
exclude = [".git", ".svn", ".hg", "CVS", ".DS_Store", "Thumbs.db"]

[preview]
auto_preview = false
preview_delay_ms = 150
close_on_exit = true

[cmux]
split_direction = "right"
split_ratio = 0.5
```

### Options

**`[tree]`** — File tree behavior
- `show_hidden` — Show hidden files (default: `true`)
- `show_ignored` — Show git-ignored files (default: `true`)
- `dirs_first` — Sort directories before files (default: `true`)
- `exclude` — Glob patterns to always exclude

**`[preview]`** — File preview (requires tmux)
- `auto_preview` — Automatically preview selected file (default: `false`)
- `preview_delay_ms` — Delay before previewing in ms (default: `150`)
- `close_on_exit` — Close preview pane on exit (default: `true`)

**`[cmux]`** — Tmux split settings
- `split_direction` — Split direction: `"right"` or `"below"` (default: `"right"`)
- `split_ratio` — Ratio of the split pane (default: `0.5`)

## License

[MIT](LICENSE)
