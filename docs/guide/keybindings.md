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
| `/` | Find by filename and jump between matches |
| `f` | Filter the tree to matching files |
| `s` | Global filename search using `fd` |
| `S` | Global content search using `rg` |
| `m` | Toggle Markdown rendered/raw preview |
| `Ctrl+C` / `Super+C` | Quit, or copy selected preview text |

`Esc`, `Enter`, `Tab`, `Shift+Tab`, and arrow keys are also used inside popups, dialogs, and search overlays. For example, local search uses `Tab` / `Down` for the next match, `Shift+Tab` / `Up` for the previous match, `Enter` to confirm, and `Esc` to cancel.

In global search, `Enter` opens the selected result in the configured editor. `Tab` jumps to the file in the tree without opening it.

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
