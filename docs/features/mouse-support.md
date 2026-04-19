# Mouse Support

croot has full mouse support that works alongside keyboard navigation.

## Mouse Actions

| Action | Effect |
|--------|--------|
| Click | Select a file/directory |
| Double-click | Expand/collapse directory or open file |
| Right-click | Open context menu |
| Scroll | Scroll the file tree, or move through global search results when the search overlay is open |
| Drag divider | Resize tree/preview split |

## Disabling Mouse

If mouse capture interferes with your terminal's native selection, disable it:

```toml
[mouse]
enabled = false
```

When disabled, you can use your terminal's native mouse features (text selection, right-click paste, etc.) while still using croot's keyboard navigation.
