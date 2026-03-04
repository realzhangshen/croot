use std::path::PathBuf;

use ratatui::style::Style;

/// Which pane has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Tree,
    Preview,
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
    }

    /// Apply a loaded preview result.
    pub fn apply(&mut self, path: PathBuf, kind: PreviewKind, content: Vec<Vec<StyledSpan>>, file_info: String) {
        self.total_lines = content.len();
        self.content = content;
        self.kind = kind;
        self.current_path = Some(path);
        self.file_info = file_info;
        self.scroll_offset = 0;
    }
}
