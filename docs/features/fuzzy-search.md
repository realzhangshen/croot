# Search

croot now uses one unified workspace search instead of splitting search into local find, tree filter, filename search, and content search.

## Unified Workspace Search

Press `/` to open the search overlay. The legacy default keys `f`, `s`, and `S` also open the same overlay, so existing muscle memory still works.

For each query, croot runs both search backends:

- `fd` finds matching file names
- `rg` finds matching file contents

Results are shown together in one list:

- file-name matches appear as direct file rows
- content matches are grouped by file
- grouped text results can be collapsed or expanded in place

This makes the search flow closer to VS Code: type once, then decide whether you want the file itself or a specific text hit inside it.

## Navigation

- Type to update results live
- `Up` / `Down` move through the mixed result list
- `Enter` opens the selected file or match
- `Enter` on a grouped text row toggles collapse
- `Tab` jumps to the selected file in the tree without opening it
- `Esc` closes the overlay

When you open a text match, croot passes the matched line number through to the configured editor. When you jump with `Tab`, croot navigates the tree and scrolls the preview toward that line.

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
max_results = 500       # Maximum file rows per search source
open_mode = "external"  # "external" opens in a GUI/background editor;
                        # "editor" suspends croot for terminal editors
```

```toml
[keybindings]
search = "/"                 # Primary unified search entry
filter = "f"                 # Legacy alias -> unified search
global_search = "s"          # Legacy alias -> unified search
global_search_content = "S"  # Legacy alias -> unified search
```
