# Search

croot now uses one unified workspace search instead of splitting search into local find, tree filter, filename search, and content search.

## Unified Workspace Search

Press `/` to open the search overlay. If you want compatibility aliases, you can still opt back into `f`, `s`, or `S` in config, but `/` is the only built-in entry by default.

For each query, croot runs both search backends:

- `fd` finds matching file and directory names
- `rg` finds matching file contents

Results are shown together in one list:

- path-name matches appear as direct rows
- content matches are grouped by file
- grouped text results can be collapsed or expanded in place
- matching characters in file names and matched lines are highlighted inline

This makes the search flow closer to VS Code: type once, then decide whether you want the file itself or a specific text hit inside it.

## Navigation

- Type to update results live
- `Up` / `Down` move through the mixed result list
- Mouse wheel moves the current selection through the result list without leaving the overlay
- `Right` pages forward through the current result list; `Left` pages back
- `Enter` selects the current path result in the tree
- `Enter` on a text match selects the file, opens preview, jumps to the matched line, and highlights the matched content
- `Enter` on a grouped text row toggles collapse
- `Tab` reveals the underlying path in the tree; on a text match it also keeps the preview line jump and highlight
- `Esc` closes the overlay

This keeps the interaction inside croot instead of immediately suspending into an editor.

## Requirements

Unified search depends on:

- [fd](https://github.com/sharkdp/fd) for file-name search
- [ripgrep](https://github.com/BurntSushi/ripgrep) for content search

If one backend is unavailable, croot still shows results from the other and surfaces the backend error in the overlay.

## Configuration

```toml
[search]
fd_command = "fd"       # Path to fd binary
rg_command = "rg"       # Path to rg binary
max_results = 500       # Maximum path rows per search source
```

```toml
[keybindings]
search = "/"                 # Primary unified search entry
filter = "f"                 # Optional legacy alias -> unified search
global_search = "s"          # Optional legacy alias -> unified search
global_search_content = "S"  # Optional legacy alias -> unified search
```
