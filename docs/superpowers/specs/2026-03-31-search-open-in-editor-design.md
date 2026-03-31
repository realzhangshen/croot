# Search Results: Open in Editor

## Context

When content search finds a match, pressing Enter navigates to the file in the tree and scrolls the preview. But the most common next action is opening the file in an editor at that line. This adds a round-trip: search → navigate → open. The feature makes Enter open the editor directly, cutting out the middle step.

The same change applies to filename search for consistency.

## Design

### Keybinding Changes (GlobalSearch mode)

| Key | Before | After |
|-----|--------|-------|
| Enter (on MatchLine) | Navigate to file in tree | Open in editor at line |
| Enter (on FileHeader) | Toggle collapse/expand | Toggle collapse/expand (unchanged) |
| Enter (filename search) | Navigate to file in tree | Open in editor |
| Tab | Move selection down | Navigate to file in tree (old Enter behavior) |
| Down | Move selection down | Move selection down (unchanged) |
| BackTab (Shift+Tab) | Move selection up | Move selection up (unchanged) |
| Esc | Cancel search | Cancel search (unchanged) |

### Data Model

**`PostAction` enum** (`src/app.rs:54`): Add `Option<usize>` line number to all editor variants:

```rust
pub enum PostAction {
    None,
    OpenEditor(PathBuf, Option<usize>),
    OpenEditorSuspend(PathBuf, Option<usize>),
    OpenEditorCmux(PathBuf, Option<usize>),
}
```

**`Action` enum** (`src/input/handler.rs`): Add new variant:

```rust
GlobalSearchGoto,  // Navigate to file in tree (Tab key)
```

### Editor Line Number

Pass `+LINE` before the file path argument. This is the POSIX-standard goto-line syntax supported by vim, nvim, nano, helix, kakoune, emacs, and VS Code.

Example: `vim +42 src/app.rs`

### Files to Modify

1. **`src/app.rs`**
   - `PostAction` enum: add `Option<usize>` to all editor variants
   - `handle_action` match arms for `PostAction::OpenEditor*`: extract line, pass to editor functions
   - `handle_content_search_confirm` (`MatchLine` branch): return `PostAction::OpenEditor(path, line)` instead of navigating to tree
   - New `handle_content_search_goto`: old MatchLine logic (navigate to tree + scroll preview)
   - `handle_action` for `Action::GlobalSearchConfirm` (filename search branch): return `PostAction::OpenEditor(path, None)`
   - `handle_action` for new `Action::GlobalSearchGoto`: call goto logic for both search types
   - `open_editor_suspend`: accept `Option<usize>`, insert `+line` arg when `Some`
   - All existing `PostAction::OpenEditor(path)` call sites: change to `PostAction::OpenEditor(path, None)`

2. **`src/input/handler.rs`**
   - `Action` enum: add `GlobalSearchGoto`
   - `handle_key_global_search`: Tab → `Action::GlobalSearchGoto`, remove Tab from Down mapping

3. **`src/render/global_search.rs`**
   - Footer hint: `"[Enter] open  [Tab] go to  [Esc] cancel"` for content search
   - Footer hint: `"[Enter] open  [Tab] go to  [Esc] cancel"` for filename search

4. **`src/cmux/bridge.rs`**
   - `open_in_editor`: accept `Option<usize>`, pass to `build_editor_command`
   - `build_editor_command`: accept `Option<usize>`, insert `+line` before path when `Some`

5. **`src/app.rs` (mouse handler)**
   - `handle_global_search_mouse`: left-click on match line should trigger editor open (matches new Enter behavior)

### Verification

1. `cargo test` — all existing tests pass after PostAction signature changes
2. New tests:
   - `handle_key_global_search` returns `GlobalSearchGoto` for Tab
   - `handle_key_global_search` returns `GlobalSearchDown` for Down (no longer Tab)
   - Content search confirm on MatchLine returns `PostAction::OpenEditor` with line number
   - Content search goto on MatchLine navigates to file in tree (old behavior)
   - Filename search confirm returns `PostAction::OpenEditor`
   - `open_editor_suspend` includes `+line` in command args when line is Some
   - `build_editor_command` includes `+line` when provided
3. Manual: run `croot`, use `S` to search content, press Enter on a match → editor opens at that line
