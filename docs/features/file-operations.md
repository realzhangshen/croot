# File Operations

croot lets you manage files and directories without leaving the terminal.

## Create

| Key | Action |
|-----|--------|
| `a` | Create a new file |
| `A` | Create a new directory |

A dialog prompts for the name. The new entry appears in the tree immediately.

## Rename

Press `R` to rename the selected file or directory. The dialog pre-fills with the current name.

## Delete

Press `D` to delete the selected file or directory. A confirmation dialog appears before deletion.

## Multi-Select

Press `v` to toggle selection on the current item. Selected items are highlighted. You can then perform bulk operations (delete) on all selected items.

Navigate with `j`/`k` or arrow keys while selecting multiple items.

## Open in Editor

Press `e` to open the selected file in your configured editor.

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

Press `x` to open the selected file with the system default application (`open` on macOS, `xdg-open` on Linux).

Configure custom open rules per file pattern:

```toml
[open]
default = "open"

[[open.rules]]
pattern = "*.pdf"
command = "zathura"
```
