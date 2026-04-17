# File Preview

croot's split-pane preview lets you read files without leaving the file tree.

## Toggle Preview

Click a file to open the preview pane. Click the same selected file again to hide the preview, or right-click a directory/workspace area and choose **Toggle Preview**. For keyboard-first use, enable `toggle_preview = "p"` in `[keybindings]`.

## Syntax Highlighting

The preview uses ANSI-native syntax highlighting, so code colors follow your terminal's theme instead of a fixed RGB palette. The built-in highlighter uses the syntect and two-face syntax bundle, covering Rust, Swift, TOML including `Cargo.toml`, Kotlin, Dockerfile, INI, Nix, Dart, Zig, TypeScript/TSX, SCSS, GraphQL, Terraform, JSON, Markdown, and many more.

## Markdown Rendering

Markdown files are rendered with formatting by default. Press `m` to toggle between rendered and raw Markdown view.

## Binary Files

Binary files are displayed as hex dumps with both hex values and ASCII representation.

## Image Preview

Image preview is optional. Build or install with the `image-preview` Cargo feature to render supported image files inline:

```bash
cargo install croot --features image-preview
```

## Configuration

```toml
[preview]
auto_preview = false      # Open preview automatically on start
preview_delay_ms = 150    # Debounce delay before preview updates
show_line_numbers = true  # Show line numbers
max_file_size_kb = 1024   # Skip files larger than this (KB)
syntax_highlight = true   # Legacy fallback for syntax.enabled
split_ratio = 0.5         # Width ratio (0.0 = all tree, 1.0 = all preview)
render_markdown = true    # Render Markdown by default
image_preview = false     # True by default only when built with image-preview
show_git_diff = true      # Show git diff gutter

[syntax]
enabled = true

[syntax.tokens.keyword]
fg = "magenta"
bold = true

[syntax.tokens.type]
fg = "cyan"
```

`preview.syntax_highlight` is still accepted as a legacy fallback, but new syntax color customization lives under `[syntax]`.

## Resizing

Drag the divider between the tree and preview to resize. The `split_ratio` config sets the default.

## Directory Preview

When a directory is selected, the preview shows a listing of its contents with file counts.
