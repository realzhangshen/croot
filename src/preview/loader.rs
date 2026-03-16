use std::fmt::Write;
use std::fs;
use std::path::Path;

use ratatui::style::{Color, Modifier, Style};

use crate::render::colors;

use super::highlight;
use super::render_md;
#[cfg(feature = "image-preview")]
use super::state::is_image_extension;
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
    image_preview: bool,
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

    // Image detection — before binary probe since images are binary
    #[cfg(feature = "image-preview")]
    if image_preview && is_image_file(path) {
        return LoadedPreview {
            kind: PreviewKind::Image,
            content: Vec::new(),
            file_info,
        };
    }
    #[cfg(not(feature = "image-preview"))]
    let _ = image_preview;

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
    load_text_preview(
        path,
        &file_info,
        syntax_highlight,
        render_markdown,
        preview_width,
    )
}

#[cfg(feature = "image-preview")]
fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|ext| is_image_extension(&ext))
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|ext| matches!(ext.as_str(), "md" | "mdx" | "markdown"))
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
    match read_prefix(path, 512) {
        Ok(data) => LoadedPreview {
            kind: PreviewKind::Binary,
            content: generate_hex_dump(&data),
            file_info: file_info.to_string(),
        },
        Err(e) => LoadedPreview {
            kind: PreviewKind::Error(format!("Read error: {e}")),
            content: Vec::new(),
            file_info: file_info.to_string(),
        },
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
        .fg(colors::preview_dir_name())
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
    let hex_style = Style::default().fg(colors::hex_values());
    let ascii_style = Style::default().fg(colors::hex_ascii());
    let separator_style = Style::default().fg(Color::DarkGray);

    let mut lines = Vec::new();
    let bytes_per_line = 16;

    for (chunk_idx, chunk) in data.chunks(bytes_per_line).enumerate() {
        let offset = chunk_idx * bytes_per_line;
        let mut spans: Vec<StyledSpan> = Vec::new();

        // Offset
        spans.push((format!("{offset:08x}  "), offset_style));

        // Hex bytes — two groups of 8 separated by an extra space
        let mut hex = String::new();
        for (i, byte) in chunk.iter().enumerate() {
            let _ = write!(hex, "{byte:02x} ");
            if i == 7 {
                hex.push(' ');
            }
        }
        // Pad short last line: ensure midpoint separator exists, then pad to full width
        if chunk.len() <= 8 {
            // Midpoint separator was never added; insert it at the right position
            let midpoint_pos = 8 * 3; // "xx " * 8
            while hex.len() < midpoint_pos {
                hex.push(' ');
            }
            hex.push(' '); // midpoint separator
        }
        let expected_len = bytes_per_line * 3 + 1; // "xx " * 16 + one midpoint space
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
    let file = fs::File::open(path)?;
    let mut buf = Vec::with_capacity(max_bytes);
    file.take(max_bytes as u64).read_to_end(&mut buf)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[cfg(feature = "image-preview")]
    #[test]
    fn is_image_file_matches_common_extensions() {
        assert!(is_image_file(Path::new("photo.png")));
        assert!(is_image_file(Path::new("photo.jpg")));
        assert!(is_image_file(Path::new("photo.jpeg")));
        assert!(is_image_file(Path::new("photo.gif")));
        assert!(is_image_file(Path::new("photo.webp")));
        assert!(is_image_file(Path::new("photo.bmp")));
        assert!(is_image_file(Path::new("photo.ico")));
        assert!(is_image_file(Path::new("photo.tiff")));
        assert!(is_image_file(Path::new("photo.tif")));
    }

    #[cfg(feature = "image-preview")]
    #[test]
    fn is_image_file_case_insensitive() {
        assert!(is_image_file(Path::new("photo.PNG")));
        assert!(is_image_file(Path::new("photo.JPG")));
    }

    #[cfg(feature = "image-preview")]
    #[test]
    fn is_image_file_rejects_non_image() {
        assert!(!is_image_file(Path::new("code.rs")));
        assert!(!is_image_file(Path::new("readme.md")));
        assert!(!is_image_file(Path::new("no_extension")));
    }

    #[cfg(feature = "image-preview")]
    #[test]
    fn load_preview_returns_image_kind_for_png() {
        let dir = tempfile::tempdir().unwrap();
        let png_path = dir.path().join("test.png");
        // Write minimal valid PNG header
        std::fs::write(&png_path, b"\x89PNG\r\n\x1a\n").unwrap();

        let result = load_preview(&png_path, 1024, true, true, 80, true);
        assert_eq!(result.kind, PreviewKind::Image);
        assert!(result.content.is_empty());
    }

    #[cfg(feature = "image-preview")]
    #[test]
    fn load_preview_returns_binary_when_image_preview_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let png_path = dir.path().join("test.png");
        std::fs::write(&png_path, b"\x89PNG\r\n\x1a\n").unwrap();

        let result = load_preview(&png_path, 1024, true, true, 80, false);
        assert_eq!(result.kind, PreviewKind::Binary);
    }

    #[test]
    fn is_markdown_file_case_insensitive() {
        assert!(is_markdown_file(Path::new("README.MD")));
        assert!(is_markdown_file(Path::new("notes.Md")));
        assert!(is_markdown_file(Path::new("doc.markdown")));
        assert!(is_markdown_file(Path::new("doc.MDX")));
        assert!(!is_markdown_file(Path::new("code.rs")));
    }

    #[test]
    fn hex_dump_short_last_line_has_aligned_ascii() {
        // 20 bytes = 1 full line (16) + 1 partial line (4)
        let data: Vec<u8> = (0..20).collect();
        let lines = generate_hex_dump(&data);
        assert_eq!(lines.len(), 2);
        // Both lines' hex spans (index 1) should have the same length for alignment
        let hex_full = &lines[0][1].0;
        let hex_short = &lines[1][1].0;
        assert_eq!(
            hex_full.len(),
            hex_short.len(),
            "hex columns should be same width: full={hex_full:?}, short={hex_short:?}"
        );
    }
}
