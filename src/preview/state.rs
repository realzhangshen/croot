use std::path::PathBuf;

use ratatui::style::Style;
use unicode_width::UnicodeWidthChar;

/// Which pane has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Tree,
    Preview,
}

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
    Binary,
    Directory,
    Empty,
    Loading,
    Error(String),
    TooLarge,
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
    }

    /// Apply a loaded preview result.
    pub fn apply(&mut self, path: PathBuf, kind: PreviewKind, content: Vec<Vec<StyledSpan>>, file_info: String) {
        self.total_lines = content.len();
        self.content = content;
        self.kind = kind;
        self.current_path = Some(path);
        self.file_info = file_info;
        self.scroll_offset = 0;
        self.selection.clear();
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
