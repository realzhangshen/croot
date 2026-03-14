# File Preview

croot's split-pane preview lets you read files without leaving the file tree.

## Toggle Preview

Press `p` to toggle the preview pane. The file under the cursor is previewed automatically.

## Syntax Highlighting

The preview supports syntax highlighting for 150+ languages including Rust, Python, JavaScript, TypeScript, Go, C/C++, Java, Ruby, and many more.

## Markdown Rendering

Markdown files are rendered with formatting by default. Press `m` to toggle between rendered and raw Markdown view.

## Binary Files

Binary files are displayed as hex dumps with both hex values and ASCII representation.

## Configuration

```toml
[preview]
auto_preview = false      # Open preview automatically on start
preview_delay_ms = 150    # Debounce delay before preview updates
show_line_numbers = true  # Show line numbers
max_file_size_kb = 1024   # Skip files larger than this (KB)
syntax_highlight = true   # Enable/disable syntax highlighting
split_ratio = 0.5         # Width ratio (0.0 = all tree, 1.0 = all preview)
render_markdown = true    # Render Markdown by default
```

## Resizing

Drag the divider between the tree and preview to resize. The `split_ratio` config sets the default.

## Directory Preview

When a directory is selected, the preview shows a listing of its contents with file counts.
