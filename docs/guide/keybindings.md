# Keybindings

croot has a small default keyboard layer and a larger set of opt-in shortcuts. Defaults work immediately; opt-in actions only become active after you add them to `~/.config/croot/config.toml`.

## Built-in Defaults

| Key | Action |
|-----|--------|
| `Up` | Move cursor up |
| `Down` | Move cursor down |
| `Left` | Collapse directory |
| `Right` | Expand directory |
| `Home` | Jump to top |
| `End` | Jump to bottom |
| `/` | Open unified workspace search |
| `f` | Open unified workspace search |
| `s` | Open unified workspace search |
| `S` | Open unified workspace search |
| `m` | Toggle Markdown rendered/raw preview |
| `Ctrl+C` / `Super+C` | Quit, or copy selected preview text |

The unified search overlay queries both file names and file contents for the same input. File-name matches appear inline with grouped content matches, so you can decide whether to open the file directly or jump to a specific hit line.

Inside the overlay, `Up` / `Down` move through results, `Enter` opens the selected file or match, `Enter` on a grouped text result toggles collapse, `Tab` jumps to the file in the tree, and `Esc` cancels.

## Opt-in Shortcuts

These actions have no default key until you configure them:

| Config key | Example | Action |
|------------|---------|--------|
| `quit` | `q` | Quit |
| `toggle` | `o` | Toggle directory expand/collapse |
| `refresh` | `r` | Refresh tree |
| `new_file` | `a` | Create new file |
| `new_dir` | `A` | Create new directory |
| `rename` | `R` | Rename selected file or directory |
| `delete` | `D` | Delete selected file or directory |
| `toggle_preview` | `p` | Toggle preview pane |
| `open_in_editor` | `e` | Open selected file in editor |
| `open_externally` | `x` | Open selected file with the system default app |
| `collapse_all` | `W` | Collapse all directories |
| `branch_picker` | `b` | Open the git branch picker |
| `enter` | `Enter` | Toggle directory or open selected file |

## Customizing Keybindings

```toml
[keybindings]
# Override built-in defaults. Overrides replace the original key.
cursor_up = "Up"
cursor_down = "Down"
search = "/"
filter = "f"

# Legacy aliases that now open the same unified search overlay.
global_search = "s"
global_search_content = "S"

# Disable a built-in binding by setting it to an empty string.
# filter = ""

# Enable opt-in bindings.
quit = "q"
toggle_preview = "p"
new_file = "a"
new_dir = "A"
rename = "R"
delete = "D"
open_in_editor = "e"
open_externally = "x"
enter = "Enter"
```

## Key Format

- Single characters: `q`, `a`, `/`
- Special keys: `Enter`, `Esc`, `Tab`, `Space`, `Backspace`, `Delete`
- Arrow keys: `Up`, `Down`, `Left`, `Right`
- Navigation names: `Home`, `End`, `PageUp`, `PageDown`
- Function keys: `F1` through `F12`
- Modifiers: `Ctrl+c`, `Shift+a`, `Alt+x`, `Super+k`
- Uppercase letters automatically include Shift: `S` = `Shift+s`
