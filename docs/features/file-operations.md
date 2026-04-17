# File Operations

croot lets you manage files and directories without leaving the terminal.

## Create

Right-click a directory, or the empty space below the tree, and choose **New File** or **New Directory**.

A dialog prompts for the name. The new entry appears in the tree immediately.

## Rename

Right-click a file or directory and choose **Rename**. The dialog pre-fills with the current name.

## Delete

Right-click a file or directory and choose **Delete**. A confirmation dialog appears before deletion.

## Open in Editor

Right-click a file and choose **Open in Editor**. When croot detects a cmux session, file menus also include **Open in cmux Tab** so the editor can open without suspending croot.

The editor is resolved in this order:
1. `editor.command` in config
2. `$VISUAL` environment variable
3. `$EDITOR` environment variable
4. `vi` (fallback)

```toml
[editor]
command = "nvim"
```

## Open Externally

Right-click a file and choose **Open Externally** to open it with the system default application (`open` on macOS, `xdg-open` on Linux).

Configure custom open rules per file pattern:

```toml
[open]
default = "open"

[[open.rules]]
pattern = "*.pdf"
command = "zathura"
```

## Optional Keyboard Shortcuts

File operation shortcuts are opt-in. Add any of these to `[keybindings]`:

```toml
[keybindings]
new_file = "a"
new_dir = "A"
rename = "R"
delete = "D"
open_in_editor = "e"
open_externally = "x"
```
