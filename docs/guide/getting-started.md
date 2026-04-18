# Quickstart

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
| Left click | Toggle a directory or preview a file |
| `Home` / `End` | Jump to top / bottom |
| `/` | Search file names and file contents |
| `Esc` | Cancel search or close popup |
| `Ctrl+C` | Quit |

## Open a Preview

Click a file to open the split-pane preview. It shows syntax-highlighted source code, rendered Markdown, image previews when built with the optional image feature, or hex dumps for binary files.

Navigate the tree while the preview updates automatically.

## File Operations

Right-click a file, directory, or empty tree space to open the context menu. From there you can create files and directories, rename, delete, copy paths, reveal files, open externally, or open files in your editor.

Keyboard shortcuts for file operations are available as opt-in keybindings. See the [Keybindings](./keybindings) guide for a ready-to-copy config snippet.

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
croot config edit   # Open config in your editor
croot config init   # Create default config file
```

Config file location: `~/.config/croot/config.toml`

See the [Configuration guide](./configuration) for all options.

## Next Steps

- [All keybindings](./keybindings)
- [Git integration details](../features/git-integration)
- [File preview features](../features/file-preview)
- [Pair with cmux](../advanced/cmux-workflow)
