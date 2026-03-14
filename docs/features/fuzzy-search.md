# Fuzzy Search

croot offers multiple search modes for quickly finding files.

## Local Search (`/`)

Press `/` to start a fuzzy search within the current tree. Type a partial filename and matching entries highlight in real time. The match count displays in the status bar.

- `Tab` / `Down` — jump to next match
- `Shift+Tab` / `Up` — jump to previous match
- `Enter` — confirm and stay on match
- `Esc` — cancel search

## Filter Mode (`f`)

Press `f` to enter filter mode. Unlike search, filter mode **hides** non-matching entries, showing only files that match your query. This is useful for large directories where you want to focus on specific file types or names.

Press `Esc` to exit filter mode and restore the full tree.

## Global Filename Search (`s`)

Press `s` to search filenames across the entire project using `fd`. This opens a picker with fuzzy matching results.

- Type to filter results
- `Up` / `Down` to navigate
- `Enter` to jump to the selected file
- `Esc` to cancel

Requires [fd](https://github.com/sharkdp/fd) to be installed.

## Global Content Search (`S`)

Press `S` to search file contents using `rg` (ripgrep). This finds text inside files across your project.

Requires [ripgrep](https://github.com/BurntSushi/ripgrep) to be installed.

## Configuration

```toml
[search]
fd_command = "fd"       # Path to fd binary
rg_command = "rg"       # Path to rg binary
max_results = 500       # Maximum number of search results
```

```toml
[keybindings]
search = "/"            # Local fuzzy search
filter = "f"            # Filter mode
global_search = "s"     # fd filename search
global_search_content = "S"  # rg content search
```
