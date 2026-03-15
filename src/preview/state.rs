use std::path::PathBuf;

use ratatui::style::Style;
use unicode_width::UnicodeWidthChar;

/// A position in content space (line index + display column).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentPos {
    pub line: usize,
    pub col: usize,
}

/// Tracks a mouse text selection as anchor (where the drag started) and cursor (current drag end).
#[derive(Debug, Clone)]
pub struct Selection {
    pub anchor: Option<ContentPos>,
    pub cursor: Option<ContentPos>,
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

impl Selection {
    pub fn new() -> Self {
        Self {
            anchor: None,
            cursor: None,
        }
    }

    pub fn is_active(&self) -> bool {
        match (self.anchor, self.cursor) {
            (Some(a), Some(c)) => a != c,
            _ => false,
        }
    }

    pub fn clear(&mut self) {
        self.anchor = None;
        self.cursor = None;
    }

    /// Returns (start, end) with start <= end in document order.
    pub fn normalized(&self) -> Option<(ContentPos, ContentPos)> {
        match (self.anchor, self.cursor) {
            (Some(a), Some(c)) => {
                if a.line < c.line || (a.line == c.line && a.col <= c.col) {
                    Some((a, c))
                } else {
                    Some((c, a))
                }
            }
            _ => None,
        }
    }
}

/// Classification of the preview content being displayed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewKind {
    Text,
    Rendered,
    Binary,
    Directory,
    Empty,
    Loading,
    Error(String),
    TooLarge,
    #[cfg(feature = "image-preview")]
    Image,
}

/// A single styled text segment within a line.
pub type StyledSpan = (String, Style);

/// Holds the state of the built-in preview panel.
pub struct PreviewState {
    /// Path currently being displayed.
    pub current_path: Option<PathBuf>,
    /// Pre-styled lines for rendering (syntax-highlighted text, hex dump, etc).
    pub content: Vec<Vec<StyledSpan>>,
    /// What kind of content we're showing.
    pub kind: PreviewKind,
    /// Vertical scroll position (line offset).
    pub scroll_offset: usize,
    /// Total number of content lines.
    pub total_lines: usize,
    /// Header info string (file size, type, etc).
    pub file_info: String,
    /// Current mouse text selection.
    pub selection: Selection,
    /// Cached mtime of the currently displayed file (to skip redundant reloads).
    pub cached_mtime: Option<std::time::SystemTime>,
    /// Whether to render Markdown files (user preference, not reset on clear).
    pub render_markdown: bool,
    /// Image rendering state (non-blocking via background thread).
    #[cfg(feature = "image-preview")]
    pub image_state: Option<ratatui_image::thread::ThreadProtocol>,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self::new()
    }
}

impl PreviewState {
    pub fn new() -> Self {
        Self {
            current_path: None,
            content: Vec::new(),
            kind: PreviewKind::Empty,
            scroll_offset: 0,
            total_lines: 0,
            file_info: String::new(),
            selection: Selection::new(),
            cached_mtime: None,
            render_markdown: true,
            #[cfg(feature = "image-preview")]
            image_state: None,
        }
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    pub fn scroll_down(&mut self, n: usize) {
        if self.total_lines > 0 {
            self.scroll_offset = (self.scroll_offset + n).min(self.total_lines.saturating_sub(1));
        }
    }

    pub fn clear(&mut self) {
        self.current_path = None;
        self.content.clear();
        self.kind = PreviewKind::Empty;
        self.scroll_offset = 0;
        self.total_lines = 0;
        self.file_info.clear();
        self.selection.clear();
        self.cached_mtime = None;
        #[cfg(feature = "image-preview")]
        {
            self.image_state = None;
        }
    }

    /// Apply a loaded preview result.
    pub fn apply(
        &mut self,
        path: PathBuf,
        kind: PreviewKind,
        content: Vec<Vec<StyledSpan>>,
        file_info: String,
    ) {
        self.cached_mtime = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok());
        self.total_lines = content.len();
        self.content = content;
        self.kind = kind;
        self.current_path = Some(path);
        self.file_info = file_info;
        self.scroll_offset = 0;
        self.selection.clear();
    }

    /// Apply an image preview result.
    #[cfg(feature = "image-preview")]
    pub fn apply_image(
        &mut self,
        path: PathBuf,
        file_info: String,
        thread_proto: ratatui_image::thread::ThreadProtocol,
    ) {
        self.cached_mtime = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok());
        self.content.clear();
        self.total_lines = 0;
        self.kind = PreviewKind::Image;
        self.current_path = Some(path);
        self.file_info = file_info;
        self.scroll_offset = 0;
        self.selection.clear();
        self.image_state = Some(thread_proto);
    }

    /// Extract the selected text from the content spans.
    pub fn extract_selected_text(&self) -> Option<String> {
        let (start, end) = self.selection.normalized()?;
        if start == end {
            return None;
        }

        let mut result = String::new();

        for line_idx in start.line..=end.line {
            if line_idx >= self.content.len() {
                break;
            }

            let col_start = if line_idx == start.line { start.col } else { 0 };
            let col_end = if line_idx == end.line {
                end.col
            } else {
                usize::MAX
            };

            let line_text = extract_line_range(&self.content[line_idx], col_start, col_end);
            result.push_str(&line_text);

            if line_idx < end.line {
                result.push('\n');
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}

/// Check whether a file extension indicates an image format.
#[cfg_attr(not(feature = "image-preview"), allow(dead_code))]
pub fn is_image_extension(ext: &str) -> bool {
    matches!(
        ext,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "tiff" | "tif"
    )
}

/// Extract text from styled spans between display columns `col_start` and `col_end`.
fn extract_line_range(spans: &[StyledSpan], col_start: usize, col_end: usize) -> String {
    let mut result = String::new();
    let mut col: usize = 0;

    for (text, _style) in spans {
        for ch in text.chars() {
            let w = UnicodeWidthChar::width(ch).unwrap_or(0);
            if col >= col_end {
                return result;
            }
            if col + w > col_start && col < col_end {
                result.push(ch);
            }
            col += w;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_image_extension_recognizes_common_formats() {
        assert!(is_image_extension("png"));
        assert!(is_image_extension("jpg"));
        assert!(is_image_extension("jpeg"));
        assert!(is_image_extension("gif"));
        assert!(is_image_extension("webp"));
        assert!(is_image_extension("bmp"));
        assert!(is_image_extension("ico"));
        assert!(is_image_extension("tiff"));
        assert!(is_image_extension("tif"));
    }

    #[test]
    fn is_image_extension_rejects_non_image() {
        assert!(!is_image_extension("rs"));
        assert!(!is_image_extension("txt"));
        assert!(!is_image_extension("md"));
        assert!(!is_image_extension("toml"));
    }

    #[test]
    fn clear_resets_all_fields() {
        let mut state = PreviewState::new();
        state.kind = PreviewKind::Text;
        state.content = vec![vec![("hello".into(), Style::default())]];
        state.total_lines = 1;
        state.scroll_offset = 5;
        state.file_info = "test".into();
        state.current_path = Some(PathBuf::from("/tmp/test.rs"));

        state.clear();

        assert_eq!(state.kind, PreviewKind::Empty);
        assert!(state.content.is_empty());
        assert_eq!(state.total_lines, 0);
        assert_eq!(state.scroll_offset, 0);
        assert!(state.file_info.is_empty());
        assert!(state.current_path.is_none());
        assert!(state.cached_mtime.is_none());
    }

    #[cfg(feature = "image-preview")]
    #[test]
    fn clear_resets_image_state() {
        let mut state = PreviewState::new();
        // image_state starts as None, clear should keep it None
        state.clear();
        assert!(state.image_state.is_none());
    }
}
