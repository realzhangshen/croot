# Git Integration

croot deeply integrates with git to show the status of every file in your repository.

## Status Indicators

When you open croot in a git repository, each file shows its git status:

| Color | Status |
|-------|--------|
| Green | Staged / added |
| Yellow | Modified (unstaged) |
| Red | Deleted |
| Light red | Conflicted |
| Gray | Ignored |

## Status Propagation

Git status propagates upward through the directory tree. If any file inside `src/` is modified, the `src/` directory itself shows as modified. This lets you spot changes at a glance without expanding every directory.

## Staged vs Unstaged

croot distinguishes between staged and unstaged changes:

- **Staged modified** — yellow (file has been `git add`-ed with changes)
- **Staged added** — green (new file that's been staged)
- **Staged deleted** — red (deleted file that's been staged)

## Automatic Updates

Git status refreshes automatically when:
- Files change on disk (via the filesystem watcher)
- You return to croot after editing in another pane

The refresh is debounced to avoid excessive git operations.

## Branch Picker

Press `b` to open the branch picker. It shows all local branches and lets you switch between them with fuzzy search.

## Color Customization

Override git status colors in your config:

```toml
[colors]
git_modified = "yellow"
git_added = "green"
git_deleted = "red"
git_ignored = "dark_gray"
git_conflicted = "light_red"
git_staged_modified = "yellow"
git_staged_added = "green"
git_staged_deleted = "red"
```
