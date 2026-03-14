# Getting Started

Get up and running with croot in under 5 minutes.

## Install

The fastest way to install on macOS:

```bash
brew install realzhangshen/croot/croot
```

See the [Installation guide](./installation) for other platforms and methods.

## Launch

```bash
croot            # Browse current directory
croot ~/projects # Browse a specific directory
```

croot opens a full-screen TUI file explorer in your terminal.

## Basic Navigation

| Key | Action |
|-----|--------|
| `Up` / `Down` | Move cursor |
| `Left` / `Right` | Collapse / expand directory |
| `Enter` | Toggle directory or open file |
| `Home` / `End` | Jump to top / bottom |
| `/` | Fuzzy search by filename |
| `Esc` | Cancel search or close popup |
| `Ctrl+C` | Quit |

## Open a Preview

Press `p` to toggle the split-pane preview. It shows syntax-highlighted source code, rendered Markdown, or hex dumps for binary files.

Navigate the tree while the preview updates automatically.

## File Operations

| Key | Action |
|-----|--------|
| `a` | Create new file |
| `A` | Create new directory |
| `R` | Rename |
| `D` | Delete |
| `v` | Toggle multi-select |
| `e` | Open in editor |

## Git Status

If you're in a git repository, croot shows file statuses:
- **Green** — staged or added
- **Yellow** — modified
- **Red** — deleted
- **Gray** — ignored

Status propagates to parent directories so you can see at a glance where changes live.

## Configuration

croot works out of the box with zero configuration. To customize behavior:

```bash
croot config        # Show current config values
croot config --edit # Open config in your editor
croot config --init # Create default config file
```

Config file location: `~/.config/croot/config.toml`

See the [Configuration guide](./configuration) for all options.

## Next Steps

- [All keybindings](./keybindings)
- [Git integration details](../features/git-integration)
- [File preview features](../features/file-preview)
- [Pair with cmux](../advanced/cmux-workflow)
