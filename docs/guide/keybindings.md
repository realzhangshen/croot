# Keybindings

croot provides two layers of keybindings: built-in defaults that work out of the box, and opt-in bindings you can enable in your config.

## Navigation

| Key | Action |
|-----|--------|
| `Up` / `k` | Move cursor up |
| `Down` / `j` | Move cursor down |
| `Left` / `h` | Collapse directory |
| `Right` / `l` | Expand directory |
| `Home` / `g` | Jump to top |
| `End` / `G` | Jump to bottom |
| `Enter` | Toggle directory or open file |
| `PageUp` | Scroll page up |
| `PageDown` | Scroll page down |

## Search & Filter

| Key | Action |
|-----|--------|
| `/` | Fuzzy search — jump to match |
| `f` | Filter — show only matching files |
| `s` | Global search — fd filename search |
| `S` | Global content search — rg content search |
| `Tab` | Next search match |
| `Shift+Tab` | Previous search match |
| `Esc` | Cancel search/filter |

## Preview

| Key | Action |
|-----|--------|
| `p` | Toggle preview pane |
| `m` | Toggle Markdown rendering (raw/rendered) |

## File Operations

| Key | Action |
|-----|--------|
| `a` | Create new file |
| `A` | Create new directory |
| `R` | Rename file/directory |
| `D` | Delete file/directory |
| `v` | Toggle multi-select |
| `e` | Open in editor |
| `x` | Open externally (Finder, xdg-open) |

## Other

| Key | Action |
|-----|--------|
| `q` | Quit |
| `r` | Refresh tree |
| `o` | Toggle expand/collapse |
| `W` | Collapse all directories |
| `b` | Branch picker (git repos) |
| `Ctrl+C` | Force quit |
| Right-click | Context menu |

## Customizing Keybindings

All keybindings are configurable in `~/.config/croot/config.toml`:

```toml
[keybindings]
# Override defaults
cursor_up = "Up"
cursor_down = "Down"
search = "/"
filter = "f"

# Enable opt-in bindings
quit = "q"
toggle_preview = "p"
new_file = "a"
delete = "D"

# Disable a binding by setting it to empty string
# filter = ""
```

### Key Format

- Single characters: `q`, `a`, `/`
- Special keys: `Enter`, `Esc`, `Tab`, `Space`, `Backspace`, `Delete`
- Arrow keys: `Up`, `Down`, `Left`, `Right`
- Navigation: `Home`, `End`, `PageUp`, `PageDown`
- Function keys: `F1` through `F12`
- Modifiers: `Ctrl+c`, `Shift+a`, `Alt+x`, `Super+k`
- Uppercase letters automatically include Shift: `S` = `Shift+s`
