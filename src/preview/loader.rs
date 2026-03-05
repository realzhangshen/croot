use std::fmt::Write;
use std::fs;
use std::path::Path;

use ratatui::style::{Color, Modifier, Style};

use crate::render::colors;

use super::highlight;
use super::render_md;
use super::state::{PreviewKind, StyledSpan};

/// Result of loading a file for preview.
pub struct LoadedPreview {
    pub kind: PreviewKind,
    pub content: Vec<Vec<StyledSpan>>,
    pub file_info: String,
}

/// Load a file for preview display.
///
/// Classifies the file type, reads content, and produces pre-styled lines.
/// `max_file_size_kb`: skip text preview for files larger than this (in KB).
/// `syntax_highlight`: whether to apply syntax highlighting.
pub fn load_preview(
    path: &Path,
    max_file_size_kb: u64,
    syntax_highlight: bool,
    render_markdown: bool,
    preview_width: usize,
) -> LoadedPreview {
    // Directories
    if path.is_dir() {
        return load_directory_preview(path);
    }

    // File metadata
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            return LoadedPreview {
                kind: PreviewKind::Error(format!("Cannot read: {e}")),
                content: Vec::new(),
                file_info: String::new(),
            };
        }
    };

    let size = metadata.len();
    let file_info = format_file_info(path, size);

    // Size check
    let max_bytes = max_file_size_kb * 1024;
    if size > max_bytes {
        return LoadedPreview {
            kind: PreviewKind::TooLarge,
            content: vec![vec![(
                format!("File too large for preview ({}).", format_size(size)),
                Style::default().fg(Color::DarkGray),
            )]],
            file_info,
        };
    }

    // Read first 8KB to detect content type
    let probe = match read_prefix(path, 8192) {
        Ok(data) => data,
        Err(e) => {
            return LoadedPreview {
                kind: PreviewKind::Error(format!("Read error: {e}")),
                content: Vec::new(),
                file_info,
            };
        }
    };

    if content_inspector::inspect(&probe).is_binary() {
        return load_binary_preview(path, &file_info);
    }

    // Text file — read full content
    load_text_preview(path, &file_info, syntax_highlight, render_markdown, preview_width)
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| matches!(ext, "md" | "mdx" | "markdown"))
}

fn load_text_preview(
    path: &Path,
    file_info: &str,
    syntax_highlight: bool,
    render_markdown: bool,
    preview_width: usize,
) -> LoadedPreview {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return LoadedPreview {
                kind: PreviewKind::Error(format!("Read error: {e}")),
                content: Vec::new(),
                file_info: file_info.to_string(),
            };
        }
    };

    // Markdown rendering path
    if render_markdown && is_markdown_file(path) {
        let lines = render_md::render_markdown(&content, preview_width);
        return LoadedPreview {
            kind: PreviewKind::Rendered,
            content: lines,
            file_info: file_info.to_string(),
        };
    }

    let max_lines = 10_000; // Cap for rendering performance
    let lines = if syntax_highlight {
        highlight::highlight_file(path, &content, max_lines)
    } else {
        highlight::plain_lines(&content, max_lines)
    };

    LoadedPreview {
        kind: PreviewKind::Text,
        content: lines,
        file_info: file_info.to_string(),
    }
}

fn load_binary_preview(path: &Path, file_info: &str) -> LoadedPreview {
    let data = read_prefix(path, 512).unwrap_or_default();

    let lines = generate_hex_dump(&data);

    LoadedPreview {
        kind: PreviewKind::Binary,
        content: lines,
        file_info: file_info.to_string(),
    }
}

fn load_directory_preview(path: &Path) -> LoadedPreview {
    let entries = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(e) => {
            return LoadedPreview {
                kind: PreviewKind::Error(format!("Cannot read directory: {e}")),
                content: Vec::new(),
                file_info: String::new(),
            };
        }
    };

    let mut files: Vec<String> = Vec::new();
    let mut dirs: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type().is_ok_and(|ft| ft.is_dir()) {
            dirs.push(name);
        } else {
            files.push(name);
        }
    }

    dirs.sort_unstable();
    files.sort_unstable();

    let dim = Style::default().fg(Color::DarkGray);
    let dir_style = Style::default()
        .fg(colors::PREVIEW_DIR_NAME)
        .add_modifier(Modifier::BOLD);
    let file_style = Style::default();

    let mut lines: Vec<Vec<StyledSpan>> = Vec::new();

    // Summary header
    lines.push(vec![(
        format!("{} dirs, {} files", dirs.len(), files.len()),
        dim,
    )]);
    lines.push(vec![(String::new(), Style::default())]);

    // Directories first
    for name in &dirs {
        lines.push(vec![
            (" ".to_string(), Style::default()),
            (format!("{name}/"), dir_style),
        ]);
    }

    // Then files
    for name in &files {
        lines.push(vec![
            ("  ".to_string(), Style::default()),
            (name.clone(), file_style),
        ]);
    }

    let dir_name = path.file_name().map_or_else(
        || path.to_string_lossy().into_owned(),
        |n| n.to_string_lossy().into_owned(),
    );

    LoadedPreview {
        kind: PreviewKind::Directory,
        content: lines,
        file_info: format!("{dir_name}/"),
    }
}

/// Generate xxd-style hex dump lines.
///
/// Format: `00000000  48 65 6c 6c 6f 20 57 6f  72 6c 64 21 0a ...  |Hello World!.|`
pub fn generate_hex_dump(data: &[u8]) -> Vec<Vec<StyledSpan>> {
    let offset_style = Style::default().fg(Color::DarkGray);
    let hex_style = Style::default().fg(colors::HEX_VALUES);
    let ascii_style = Style::default().fg(colors::HEX_ASCII);
    let separator_style = Style::default().fg(Color::DarkGray);

    let mut lines = Vec::new();
    let bytes_per_line = 16;

    for (chunk_idx, chunk) in data.chunks(bytes_per_line).enumerate() {
        let offset = chunk_idx * bytes_per_line;
        let mut spans: Vec<StyledSpan> = Vec::new();

        // Offset
        spans.push((format!("{offset:08x}  "), offset_style));

        // Hex bytes
        let mut hex = String::new();
        for (i, byte) in chunk.iter().enumerate() {
            let _ = write!(hex, "{byte:02x} ");
            if i == 7 {
                hex.push(' ');
            }
        }
        // Pad if short line
        let expected_len = bytes_per_line * 3 + 1; // "xx " * 16 + one extra space at midpoint
        while hex.len() < expected_len {
            hex.push(' ');
        }
        spans.push((hex, hex_style));

        // ASCII representation
        spans.push((" |".to_string(), separator_style));
        let ascii: String = chunk
            .iter()
            .map(|&b| {
                if (0x20..=0x7E).contains(&b) {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        spans.push((ascii, ascii_style));
        spans.push(("|".to_string(), separator_style));

        lines.push(spans);
    }

    lines
}

fn read_prefix(path: &Path, max_bytes: usize) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    let mut file = fs::File::open(path)?;
    let mut buf = vec![0u8; max_bytes];
    let n = file.read(&mut buf)?;
    buf.truncate(n);
    Ok(buf)
}

fn format_file_info(path: &Path, size: u64) -> String {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let size_str = format_size(size);
    if ext.is_empty() {
        size_str
    } else {
        format!("{size_str}  .{ext}")
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
