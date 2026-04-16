use std::path::PathBuf;

use unicode_width::UnicodeWidthChar;

use crate::git::diff::{GitDiffHint, LineDiffStatus};
pub use crate::syntax::StyledSpan;

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
    /// Cached diff hint used when the current preview was generated. This
    /// is part of the cache key: if the same path+mtime is requested again
    /// but the derived hint has changed (e.g. git status went
    /// Clean → Modified after a background refresh), the cached preview
    /// must be invalidated because its diff gutter is now stale.
    pub cached_diff_hint: Option<GitDiffHint>,
    /// Per-line git diff status for gutter indicators (only for Text previews).
    pub line_diffs: Option<Vec<LineDiffStatus>>,
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
            cached_diff_hint: None,
            line_diffs: None,
            render_markdown: true,
            #[cfg(feature = "image-preview")]
            image_state: None,
        }
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    /// Scroll so that `rg_line` (1-based) is visible with a few lines of context above.
    /// Clamps to file length to avoid scrolling past EOF.
    pub fn scroll_to_line(&mut self, rg_line: usize) {
        let target_idx = rg_line.saturating_sub(1);
        let offset = target_idx.saturating_sub(3);
        self.scroll_offset = offset.min(self.total_lines.saturating_sub(1));
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
        self.cached_diff_hint = None;
        self.line_diffs = None;
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
        line_diffs: Option<Vec<LineDiffStatus>>,
        git_diff_hint: GitDiffHint,
    ) {
        self.cached_mtime = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok());
        self.cached_diff_hint = Some(git_diff_hint);
        self.total_lines = content.len();
        self.content = content;
        self.kind = kind;
        self.current_path = Some(path);
        self.file_info = file_info;
        self.line_diffs = line_diffs;
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
        // Image previews never show a diff gutter, so lock the hint to Skip.
        self.cached_diff_hint = Some(GitDiffHint::Skip);
        self.content.clear();
        self.total_lines = 0;
        self.kind = PreviewKind::Image;
        self.current_path = Some(path);
        self.file_info = file_info;
        self.line_diffs = None;
        self.scroll_offset = 0;
        self.selection.clear();
        self.image_state = Some(thread_proto);
    }

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
    use ratatui::style::Style;

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
        assert!(state.line_diffs.is_none());
    }

    #[cfg(feature = "image-preview")]
    #[test]
    fn clear_resets_image_state() {
        let mut state = PreviewState::new();
        // image_state starts as None, clear should keep it None
        state.clear();
        assert!(state.image_state.is_none());
    }

    // --- extract_line_range tests ---

    fn span(s: &str) -> StyledSpan {
        (s.to_string(), Style::default())
    }

    #[test]
    fn extract_line_range_full_span() {
        let spans = vec![span("hello")];
        assert_eq!(extract_line_range(&spans, 0, usize::MAX), "hello");
    }

    #[test]
    fn extract_line_range_substring() {
        let spans = vec![span("hello world")];
        assert_eq!(extract_line_range(&spans, 2, 7), "llo w");
    }

    #[test]
    fn extract_line_range_multi_span() {
        let spans = vec![span("ab"), span("cd"), span("ef")];
        assert_eq!(extract_line_range(&spans, 1, 5), "bcde");
    }

    #[test]
    fn extract_line_range_cjk_characters() {
        // CJK characters are 2 display columns wide
        let spans = vec![span("你好世界")]; // 8 display columns
                                            // col 0-1: 你, col 2-3: 好, col 4-5: 世, col 6-7: 界
        assert_eq!(extract_line_range(&spans, 0, 4), "你好");
        assert_eq!(extract_line_range(&spans, 2, 6), "好世");
    }

    #[test]
    fn extract_line_range_empty() {
        let spans: Vec<StyledSpan> = vec![];
        assert_eq!(extract_line_range(&spans, 0, 10), "");
    }

    // --- extract_selected_text tests ---

    fn state_with_content(lines: Vec<Vec<StyledSpan>>) -> PreviewState {
        let mut s = PreviewState::new();
        s.total_lines = lines.len();
        s.content = lines;
        s.kind = PreviewKind::Text;
        s
    }

    #[test]
    fn extract_selected_text_single_line() {
        let mut state = state_with_content(vec![vec![span("hello world")]]);
        state.selection.anchor = Some(ContentPos { line: 0, col: 2 });
        state.selection.cursor = Some(ContentPos { line: 0, col: 7 });
        assert_eq!(state.extract_selected_text().unwrap(), "llo w");
    }

    #[test]
    fn extract_selected_text_multi_line() {
        let mut state = state_with_content(vec![
            vec![span("line one")],
            vec![span("line two")],
            vec![span("line three")],
        ]);
        state.selection.anchor = Some(ContentPos { line: 0, col: 5 });
        state.selection.cursor = Some(ContentPos { line: 2, col: 4 });
        assert_eq!(
            state.extract_selected_text().unwrap(),
            "one\nline two\nline"
        );
    }

    #[test]
    fn extract_selected_text_reversed_selection() {
        let mut state = state_with_content(vec![vec![span("abcdef")]]);
        // Drag upward: cursor before anchor
        state.selection.anchor = Some(ContentPos { line: 0, col: 4 });
        state.selection.cursor = Some(ContentPos { line: 0, col: 1 });
        assert_eq!(state.extract_selected_text().unwrap(), "bcd");
    }

    #[test]
    fn extract_selected_text_same_position_returns_none() {
        let mut state = state_with_content(vec![vec![span("hello")]]);
        state.selection.anchor = Some(ContentPos { line: 0, col: 3 });
        state.selection.cursor = Some(ContentPos { line: 0, col: 3 });
        assert!(state.extract_selected_text().is_none());
    }

    #[test]
    fn extract_selected_text_no_selection_returns_none() {
        let state = state_with_content(vec![vec![span("hello")]]);
        assert!(state.extract_selected_text().is_none());
    }

    #[test]
    fn extract_selected_text_past_content_len() {
        let mut state = state_with_content(vec![vec![span("only line")]]);
        state.selection.anchor = Some(ContentPos { line: 0, col: 0 });
        state.selection.cursor = Some(ContentPos { line: 5, col: 0 });
        // Should not panic; extracts what's available
        let text = state.extract_selected_text().unwrap();
        assert!(text.contains("only line"));
    }

    // ── scroll_to_line tests ──────────────────────────────────────────

    #[test]
    fn scroll_to_line_first_line() {
        let mut state = PreviewState::new();
        state.total_lines = 100;
        state.scroll_to_line(1); // rg line 1 → index 0 → offset 0
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn scroll_to_line_middle() {
        let mut state = PreviewState::new();
        state.total_lines = 100;
        state.scroll_to_line(50); // index 49, minus 3 context = 46
        assert_eq!(state.scroll_offset, 46);
    }

    #[test]
    fn scroll_to_line_near_start() {
        let mut state = PreviewState::new();
        state.total_lines = 100;
        state.scroll_to_line(2); // index 1, saturating_sub(3) = 0
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn scroll_to_line_last_line() {
        let mut state = PreviewState::new();
        state.total_lines = 100;
        state.scroll_to_line(100); // index 99, minus 3 = 96, clamped to 99
        assert_eq!(state.scroll_offset, 96);
    }

    #[test]
    fn scroll_to_line_past_eof() {
        let mut state = PreviewState::new();
        state.total_lines = 100;
        state.scroll_to_line(200); // index 199, minus 3 = 196, clamped to 99
        assert_eq!(state.scroll_offset, 99);
    }

    #[test]
    fn scroll_to_line_empty_file() {
        let mut state = PreviewState::new();
        state.total_lines = 0;
        state.scroll_to_line(1); // should not panic, offset stays 0
        assert_eq!(state.scroll_offset, 0);
    }
}
