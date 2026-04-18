# Configuration

croot uses a TOML config file at `~/.config/croot/config.toml` (or `$XDG_CONFIG_HOME/croot/config.toml`).

## Quick Start

```bash
croot config init    # Create default config file
croot config edit    # Open config in your editor
croot config         # Show all resolved values
```

## Config Sections

### `[tree]` — File Tree

```toml
[tree]
show_hidden = true        # Show hidden (dot) files
show_ignored = true       # Show git-ignored files
dirs_first = true         # Directories before files
compact_folders = true    # Collapse single-child dirs (a/b/c)
show_size = false         # Show file sizes
show_modified = false     # Show modification times
exclude = [".git", ".svn", ".hg", "CVS", ".DS_Store", "Thumbs.db"]
```

### `[preview]` — Preview Pane

```toml
[preview]
auto_preview = false      # Auto-open preview on start
preview_delay_ms = 150    # Debounce delay before updating preview
show_line_numbers = true  # Show line numbers in preview
max_file_size_kb = 1024   # Max file size to preview (KB)
syntax_highlight = true   # Legacy fallback for syntax.enabled
split_ratio = 0.5         # Tree/preview width ratio (0.0–1.0)
render_markdown = true    # Render Markdown or show raw
close_on_exit = true      # Close preview when quitting
image_preview = false     # True by default only when built with image-preview
show_git_diff = true      # Show git diff gutter in text previews
```

`preview.syntax_highlight` is kept as a legacy fallback, but new syntax highlighting settings live in `[syntax]`.

### `[syntax]` — ANSI Syntax Highlighting

Syntax highlighting uses syntect plus the extended two-face syntax bundle, so common languages such as Rust, Swift, TOML, Kotlin, Dockerfile, Nix, Dart, Zig, TypeScript, JSON, Markdown, and many more are recognized. Syntax token colors are ANSI-only: `reset`, ANSI 16 names, or `indexed:N`. Hex/RGB is intentionally rejected so code colors keep adapting to your terminal theme.

```toml
[syntax]
enabled = true

[syntax.tokens.text]
fg = "reset"

[syntax.tokens.keyword]
fg = "magenta"
bold = true

[syntax.tokens.type]
fg = "cyan"

[syntax.tokens.comment]
fg = "dark_gray"
italic = true
```

Available semantic tokens include:
- `text`
- `keyword`
- `type`
- `type_builtin`
- `string`
- `escape`
- `number`
- `constant`
- `comment`
- `function`
- `method`
- `constructor`
- `variable`
- `parameter`
- `property`
- `operator`
- `punctuation`
- `module`
- `tag`
- `attribute`
- `macro`
- `lifetime`

### `[editor]` — Editor Command

```toml
[editor]
command = "vim"        # Falls back to $VISUAL → $EDITOR → vi
external = "code -g"   # External/GUI editor for search results
                       # Uses file:line syntax (VS Code, Sublime, etc.)
                       # Falls back to [open] default if not set
```

### `[open]` — External Open Rules

```toml
[open]
default = "open"  # macOS: "open", Linux: "xdg-open"

[[open.rules]]
pattern = "*.pdf"
command = "zathura"

[[open.rules]]
pattern = "*.png"
command = "feh"
```

### `[mouse]`

```toml
[mouse]
enabled = true  # Set false to disable mouse capture
```

### `[keybindings]`

See the [Keybindings](./keybindings) page for full details.

```toml
[keybindings]
# Override built-in defaults
search = "/"                 # unified workspace search

# Enable opt-in shortcuts
filter = "f"                # legacy alias -> unified workspace search
global_search = "s"         # legacy alias -> unified workspace search
global_search_content = "S" # legacy alias -> unified workspace search
quit = "q"
toggle_preview = "p"
new_file = "a"
new_dir = "A"
rename = "R"
delete = "D"
open_in_editor = "e"
```

### `[search]` — External Search Tools

```toml
[search]
fd_command = "fd"      # Command for path-name search
rg_command = "rg"      # Command for content search
max_results = 500      # Maximum path rows per backend
                       # - fd caps direct path-name hits
                       # - rg caps unique files in grouped text hits
                       #   (each file displays up to 20 matches)
```

### `[colors]` — Color Customization

The built-in palette is ANSI-only and tuned for broad terminal compatibility. Override any color with ANSI names, indexed colors (0–255), or hex RGB.

```toml
[colors]
# Git status colors
git_modified = "yellow"
git_added = "green"
git_deleted = "red"
git_ignored = "dark_gray"
git_conflicted = "light_red"

# UI colors
dir_color = "blue"
tree_line = "dark_gray"
find_match = "cyan"

# Popup/dialog colors
popup_bg = "black"
popup_fg = "white"
popup_accent = "light_blue"
popup_border_fg = "reset"
popup_input_bg = "white"
popup_input_fg = "black"
```

Color formats:
- ANSI names: `red`, `green`, `blue`, `dark_gray`, `light_blue`, `reset`
- Indexed: `240` or `indexed:240`
- Hex RGB: `#ff0000`

## Reading and Setting Values

Read a single value:

```bash
croot config get tree.show_hidden
# true

croot config get preview.split_ratio
# 0.5
```

Set a value:

```bash
croot config set tree.show_hidden false
croot config set preview.split_ratio 0.3
```
