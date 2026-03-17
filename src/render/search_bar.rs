use std::collections::HashMap;
use std::path::PathBuf;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use unicode_width::UnicodeWidthStr;

use super::colors;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Find,
    Filter,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchMode {
    Fuzzy,
    Regex,
    Exact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalSearchType {
    FileName,
    Content,
}

#[derive(Debug, Clone)]
pub struct GlobalSearchResult {
    pub path: PathBuf,
    pub display: String,
    pub line: Option<usize>,
    pub context: Option<String>,
}

/// State for the search/filter bar.
#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    /// Byte offset into `query`. Must always be on a UTF-8 char boundary.
    pub(crate) cursor_pos: usize,
    pub mode: SearchMode,
    /// Indices of nodes that match the query (sorted).
    pub match_indices: Vec<usize>,
    /// Index into `match_indices` for the current match position.
    pub current_match: usize,
    /// Indices of nodes visible in filter mode (matches + ancestors, sorted).
    pub visible_indices: Vec<usize>,
    /// Cursor position before search started.
    pub origin_cursor: usize,
    /// Scroll offset before search started.
    pub origin_scroll_offset: usize,
    /// Cached compiled regex for Regex match mode.
    pub compiled_regex: Option<regex::Regex>,
    /// Per-node byte positions of matched characters (Find mode).
    /// Keyed by node index; absent entry → full-name underline fallback.
    pub match_char_positions: HashMap<usize, Vec<usize>>,
    // Global search fields
    pub global_results: Vec<GlobalSearchResult>,
    pub global_selected: usize,
    pub global_scroll_offset: usize,
    pub global_loading: bool,
    pub global_error: Option<String>,
    pub global_search_type: GlobalSearchType,
    pub global_visible_height: usize,
    pub request_id: u64,
}

impl SearchState {
    pub fn new(mode: SearchMode) -> Self {
        Self {
            query: String::new(),
            cursor_pos: 0,
            mode,
            match_indices: Vec::new(),
            current_match: 0,
            visible_indices: Vec::new(),
            origin_cursor: 0,
            origin_scroll_offset: 0,
            compiled_regex: None,
            match_char_positions: HashMap::new(),
            global_results: Vec::new(),
            global_selected: 0,
            global_scroll_offset: 0,
            global_loading: false,
            global_error: None,
            global_search_type: GlobalSearchType::FileName,
            global_visible_height: 0,
            request_id: 0,
        }
    }

    pub fn match_count(&self) -> usize {
        self.match_indices.len()
    }

    pub fn insert_char(&mut self, ch: char) {
        self.query.insert(self.cursor_pos, ch);
        self.cursor_pos += ch.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            let prev = self.query[..self.cursor_pos]
                .chars()
                .last()
                .map_or(0, char::len_utf8);
            self.cursor_pos -= prev;
            self.query.remove(self.cursor_pos);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            let prev = self.query[..self.cursor_pos]
                .chars()
                .last()
                .map_or(0, char::len_utf8);
            self.cursor_pos -= prev;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor_pos < self.query.len() {
            let next = self.query[self.cursor_pos..]
                .chars()
                .next()
                .map_or(0, char::len_utf8);
            self.cursor_pos += next;
        }
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor_pos = 0;
        self.match_indices.clear();
        self.current_match = 0;
        self.visible_indices.clear();
        self.compiled_regex = None;
        self.match_char_positions.clear();
        self.global_results.clear();
        self.global_selected = 0;
        self.global_scroll_offset = 0;
        self.global_loading = false;
        self.global_error = None;
    }

    pub fn is_empty(&self) -> bool {
        self.query.is_empty()
    }

    /// Convert a byte offset within `query` to its display column width.
    /// This bridges the gap between byte-based `cursor_pos` (used for string
    /// mutation) and the screen column where the cursor should render.
    pub fn cursor_display_column(&self) -> usize {
        cursor_byte_to_column(&self.query, self.cursor_pos)
    }

    /// Parse the query to determine match mode and effective query string.
    /// Compiles and caches regex when in Regex mode.
    pub fn effective_query(&mut self) -> (String, MatchMode) {
        if self.query.starts_with('/') && self.query.len() > 1 {
            let pattern = self.query[1..].to_string();
            let needs_recompile = self
                .compiled_regex
                .as_ref()
                .is_none_or(|r| r.as_str() != pattern);
            if needs_recompile {
                self.compiled_regex = regex::Regex::new(&pattern).ok();
            }
            (pattern, MatchMode::Regex)
        } else if self.query.starts_with('\'') && self.query.len() > 1 {
            self.compiled_regex = None;
            (self.query[1..].to_string(), MatchMode::Exact)
        } else {
            self.compiled_regex = None;
            (self.query.clone(), MatchMode::Fuzzy)
        }
    }
}

/// Convert a byte offset within a string to its display column width.
pub(crate) fn cursor_byte_to_column(s: &str, byte_pos: usize) -> usize {
    UnicodeWidthStr::width(&s[..byte_pos])
}

pub struct SearchBar<'a> {
    pub state: &'a SearchState,
    pub show_close_button: bool,
}

impl SearchBar<'_> {
    /// Returns the x coordinate where the close button `[×]` starts.
    /// Returns None if close button is not shown.
    pub fn close_button_x(area_x: u16, area_width: u16) -> u16 {
        area_x + area_width.saturating_sub(4)
    }
}

impl Widget for SearchBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg = colors::status_bar_bg();
        let style = colors::status_input();

        // Fill background
        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_style(style);
                cell.set_symbol(" ");
            }
        }

        // Mode-specific prompt
        let (prompt, prompt_color) = match self.state.mode {
            SearchMode::Find => (" / ", colors::find_match()),
            SearchMode::Filter => (" F ", colors::git_modified()),
            SearchMode::Global => (" S ", colors::popup_accent()),
        };

        buf.set_string(
            area.x,
            area.y,
            prompt,
            Style::default()
                .fg(prompt_color)
                .bg(bg)
                .add_modifier(Modifier::BOLD),
        );

        let input_x = area.x + prompt.len() as u16;
        // Reserve space for match info + optional close button on the right
        let close_width: u16 = if self.show_close_button { 4 } else { 0 }; // "[×] "
        let match_info_max: u16 = 8; // " 0/0 " or similar
        let right_reserve = close_width + match_info_max;
        let input_width = area
            .width
            .saturating_sub(prompt.len() as u16 + right_reserve) as usize;

        // Draw query text
        let display_text =
            super::status_bar::truncate_start_to_display_width(&self.state.query, input_width);
        buf.set_string(input_x, area.y, &display_text, colors::status_input());

        // Draw cursor
        let query_display_width = UnicodeWidthStr::width(self.state.query.as_str());
        let cursor_display_pos = if query_display_width > input_width {
            input_width
        } else {
            self.state.cursor_display_column()
        };
        if let Some(cell) = buf.cell_mut((input_x + cursor_display_pos as u16, area.y)) {
            cell.set_style(colors::status_cursor());
            if cell.symbol() == " " || cell.symbol().is_empty() {
                cell.set_symbol(" ");
            }
        }

        // Close button on the far right
        let close_btn = "[×]";
        let close_btn_width = UnicodeWidthStr::width(close_btn) as u16;
        let close_reserve = if self.show_close_button {
            close_btn_width + 1 // +1 for space
        } else {
            0
        };

        // Match info (mode-specific)
        let match_info = if self.state.query.is_empty() {
            String::new()
        } else {
            match self.state.mode {
                SearchMode::Find => {
                    if self.state.match_count() > 0 {
                        format!(
                            " {}/{} ",
                            self.state.current_match + 1,
                            self.state.match_count()
                        )
                    } else {
                        " 0/0 ".to_string()
                    }
                }
                SearchMode::Filter => {
                    format!(" {} matches ", self.state.match_count())
                }
                SearchMode::Global => String::new(),
            }
        };
        let match_info_width = UnicodeWidthStr::width(match_info.as_str()) as u16;
        let right_reserved = match_info_width + close_reserve;
        if !match_info.is_empty() && area.width > right_reserved {
            let info_x = area.x + area.width - right_reserved;
            let info_style = if self.state.match_count() > 0 {
                colors::status_success()
            } else {
                colors::status_error()
            };
            buf.set_string(info_x, area.y, &match_info, info_style);
        }

        // Draw close button (only if it fits)
        if self.show_close_button && area.width > close_btn_width + 1 {
            let close_x = area.x + area.width - close_btn_width - 1;
            buf.set_string(
                close_x,
                area.y,
                close_btn,
                colors::status_error().add_modifier(Modifier::BOLD),
            );
        }
    }
}

/// Fuzzy match: all characters of the query appear in order in the target.
pub fn fuzzy_match(query: &str, target: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let query_lower = query.to_ascii_lowercase();
    let target_lower = target.to_ascii_lowercase();
    let mut query_chars = query_lower.chars();
    let mut current = query_chars.next();

    for ch in target_lower.chars() {
        if let Some(q) = current {
            if ch == q {
                current = query_chars.next();
            }
        } else {
            return true;
        }
    }
    current.is_none()
}

/// Regex match using a pre-compiled regex.
pub fn regex_match(re: &regex::Regex, target: &str) -> bool {
    re.is_match(target)
}

/// Exact substring match (case-insensitive).
pub fn exact_match(query: &str, target: &str) -> bool {
    target.to_lowercase().contains(&query.to_lowercase())
}

/// Dispatch matching based on mode.
pub fn do_match(
    match_mode: MatchMode,
    query: &str,
    re: Option<&regex::Regex>,
    target: &str,
) -> bool {
    match match_mode {
        MatchMode::Fuzzy => fuzzy_match(query, target),
        MatchMode::Regex => re.is_some_and(|r| regex_match(r, target)),
        MatchMode::Exact => exact_match(query, target),
    }
}

/// Fuzzy match returning byte positions of each matched character.
pub fn fuzzy_match_positions(query: &str, target: &str) -> Option<Vec<usize>> {
    if query.is_empty() {
        return Some(vec![]);
    }
    let mut positions = Vec::new();
    let mut query_chars = query.chars();
    let mut current = query_chars.next();

    for (byte_idx, ch) in target.char_indices() {
        if let Some(q) = current {
            if ch.eq_ignore_ascii_case(&q) {
                positions.push(byte_idx);
                current = query_chars.next();
            }
        } else {
            break;
        }
    }
    if current.is_none() {
        Some(positions)
    } else {
        None
    }
}

/// Exact substring match returning byte positions (char boundaries) of the matched range.
///
/// Performs the search on lowercased strings, then maps the matched character
/// range back to byte offsets in the **original** `target` string. This avoids
/// panics when `to_lowercase()` changes the byte length of characters (e.g.
/// `İ` (2 bytes) → `i̇` (3 bytes)).
pub fn exact_match_positions(query: &str, target: &str) -> Option<Vec<usize>> {
    let target_lower = target.to_lowercase();
    let query_lower = query.to_lowercase();
    let match_start = target_lower.find(&query_lower)?;

    // Find which original chars correspond to the matched range in target_lower.
    // Walk both strings char-by-char to build the byte-offset mapping.
    let mut lower_byte = 0usize;
    let mut orig_positions = Vec::new();
    for (orig_byte, orig_char) in target.char_indices() {
        let lower_char_len: usize = orig_char.to_lowercase().map(char::len_utf8).sum();
        let lower_end = lower_byte + lower_char_len;
        // This original char contributes to [lower_byte..lower_end) in target_lower.
        // If any part overlaps with the match range, include the original byte offset.
        let match_end = match_start + query_lower.len();
        if lower_end > match_start && lower_byte < match_end {
            orig_positions.push(orig_byte);
        }
        lower_byte = lower_end;
        if lower_byte >= match_start + query_lower.len() && !orig_positions.is_empty() {
            break;
        }
    }

    if orig_positions.is_empty() {
        None
    } else {
        Some(orig_positions)
    }
}

/// Regex match returning byte positions (char boundaries) of the first match span.
pub fn regex_match_positions(re: &regex::Regex, target: &str) -> Option<Vec<usize>> {
    let m = re.find(target)?;
    // Collect only char-boundary byte offsets so highlighting works with multibyte chars
    Some(
        target[m.start()..m.end()]
            .char_indices()
            .map(|(i, _)| m.start() + i)
            .collect(),
    )
}

/// Dispatch position-returning match based on mode.
pub fn do_match_positions(
    match_mode: MatchMode,
    query: &str,
    re: Option<&regex::Regex>,
    target: &str,
) -> Option<Vec<usize>> {
    match match_mode {
        MatchMode::Fuzzy => fuzzy_match_positions(query, target),
        MatchMode::Regex => re.and_then(|r| regex_match_positions(r, target)),
        MatchMode::Exact => exact_match_positions(query, target),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_empty_matches_anything() {
        assert!(fuzzy_match("", "anything"));
    }

    #[test]
    fn fuzzy_exact_match() {
        assert!(fuzzy_match("app", "app.rs"));
    }

    #[test]
    fn fuzzy_subsequence() {
        assert!(fuzzy_match("ars", "app.rs"));
    }

    #[test]
    fn fuzzy_case_insensitive() {
        assert!(fuzzy_match("APP", "app.rs"));
    }

    #[test]
    fn fuzzy_no_match() {
        assert!(!fuzzy_match("xyz", "app.rs"));
    }

    #[test]
    fn fuzzy_partial_no_match() {
        assert!(!fuzzy_match("apz", "app.rs"));
    }

    #[test]
    fn exact_match_substring() {
        assert!(exact_match("handler", "input_handler.rs"));
        assert!(exact_match("Handler", "input_handler.rs"));
        assert!(!exact_match("xyz", "input_handler.rs"));
    }

    #[test]
    fn regex_match_pattern() {
        let re = regex::Regex::new("^handler").unwrap();
        assert!(regex_match(&re, "handler.rs"));
        assert!(!regex_match(&re, "input_handler.rs"));
    }

    #[test]
    fn effective_query_fuzzy() {
        let mut state = SearchState::new(SearchMode::Find);
        state.query = "handler".to_string();
        let (q, mode) = state.effective_query();
        assert_eq!(q, "handler");
        assert_eq!(mode, MatchMode::Fuzzy);
    }

    #[test]
    fn effective_query_regex() {
        let mut state = SearchState::new(SearchMode::Find);
        state.query = "/^handler".to_string();
        let (q, mode) = state.effective_query();
        assert_eq!(q, "^handler");
        assert_eq!(mode, MatchMode::Regex);
        assert!(state.compiled_regex.is_some());
    }

    #[test]
    fn effective_query_exact() {
        let mut state = SearchState::new(SearchMode::Find);
        state.query = "'handler.rs".to_string();
        let (q, mode) = state.effective_query();
        assert_eq!(q, "handler.rs");
        assert_eq!(mode, MatchMode::Exact);
    }

    #[test]
    fn effective_query_invalid_regex_no_panic() {
        let mut state = SearchState::new(SearchMode::Find);
        state.query = "/[".to_string();
        let (_, mode) = state.effective_query();
        assert_eq!(mode, MatchMode::Regex);
        assert!(state.compiled_regex.is_none());
    }

    #[test]
    fn do_match_dispatches_correctly() {
        let re = regex::Regex::new("^app").unwrap();
        assert!(do_match(MatchMode::Fuzzy, "ars", None, "app.rs"));
        assert!(do_match(MatchMode::Regex, "^app", Some(&re), "app.rs"));
        assert!(!do_match(MatchMode::Regex, "^app", None, "app.rs"));
        assert!(do_match(MatchMode::Exact, "app", None, "app.rs"));
    }

    #[test]
    fn match_count_reflects_indices() {
        let mut state = SearchState::new(SearchMode::Find);
        assert_eq!(state.match_count(), 0);
        state.match_indices = vec![1, 3, 5];
        assert_eq!(state.match_count(), 3);
    }

    // ── Position-returning match tests ──────────────────────────────────

    #[test]
    fn fuzzy_match_positions_subsequence() {
        let pos = fuzzy_match_positions("ars", "app.rs");
        assert_eq!(pos, Some(vec![0, 4, 5]));
    }

    #[test]
    fn fuzzy_match_positions_case_insensitive() {
        let pos = fuzzy_match_positions("ARS", "app.rs");
        assert_eq!(pos, Some(vec![0, 4, 5]));
    }

    #[test]
    fn fuzzy_match_positions_no_match() {
        assert_eq!(fuzzy_match_positions("xyz", "app.rs"), None);
    }

    #[test]
    fn fuzzy_match_positions_empty_query() {
        assert_eq!(fuzzy_match_positions("", "anything"), Some(vec![]));
    }

    #[test]
    fn exact_match_positions_substring() {
        let pos = exact_match_positions("handler", "input_handler.rs");
        assert_eq!(pos, Some(vec![6, 7, 8, 9, 10, 11, 12]));
    }

    #[test]
    fn exact_match_positions_case_insensitive() {
        let pos = exact_match_positions("Handler", "input_handler.rs");
        assert_eq!(pos, Some(vec![6, 7, 8, 9, 10, 11, 12]));
    }

    #[test]
    fn exact_match_positions_no_match() {
        assert_eq!(exact_match_positions("xyz", "input_handler.rs"), None);
    }

    #[test]
    fn regex_match_positions_anchored() {
        let re = regex::Regex::new("^app").unwrap();
        let pos = regex_match_positions(&re, "app.rs");
        assert_eq!(pos, Some(vec![0, 1, 2]));
    }

    #[test]
    fn regex_match_positions_no_match() {
        let re = regex::Regex::new("^handler").unwrap();
        assert_eq!(regex_match_positions(&re, "input_handler.rs"), None);
    }

    #[test]
    fn do_match_positions_dispatches() {
        let re = regex::Regex::new("^app").unwrap();
        assert!(do_match_positions(MatchMode::Fuzzy, "ars", None, "app.rs").is_some());
        assert!(do_match_positions(MatchMode::Regex, "^app", Some(&re), "app.rs").is_some());
        assert!(do_match_positions(MatchMode::Regex, "^app", None, "app.rs").is_none());
        assert!(do_match_positions(MatchMode::Exact, "app", None, "app.rs").is_some());
    }

    // ── Bug 2: cursor byte-to-column ──────────────────────────────────

    #[test]
    fn cursor_byte_to_column_ascii() {
        assert_eq!(cursor_byte_to_column("hello", 5), 5);
        assert_eq!(cursor_byte_to_column("hello", 0), 0);
    }

    #[test]
    fn cursor_byte_to_column_multibyte() {
        // "café" — é is 2 bytes (0xC3 0xA9) but 1 display column
        let s = "café";
        assert_eq!(s.len(), 5); // 3 ASCII + 2-byte é
                                // After inserting all chars, cursor_pos == 5 (bytes), display == 4 columns
        let mut state = SearchState::new(SearchMode::Find);
        for ch in s.chars() {
            state.insert_char(ch);
        }
        assert_eq!(state.cursor_pos, 5);
        assert_eq!(state.cursor_display_column(), 4);
    }

    #[test]
    fn exact_match_positions_multibyte_returns_char_boundaries() {
        // "café.rs" — 'é' is 2 bytes; match "fé" should return char-boundary positions
        let pos = exact_match_positions("fé", "café.rs");
        assert!(pos.is_some());
        let positions = pos.unwrap();
        // 'f' starts at byte 2, 'é' starts at byte 3 (2 bytes), so positions = [2, 3]
        assert_eq!(positions, vec![2, 3]);
        // Verify all positions are valid char boundaries
        for &p in &positions {
            assert!(
                "café.rs".is_char_boundary(p),
                "position {p} is not a char boundary"
            );
        }
    }

    #[test]
    fn exact_match_unicode_case_folding() {
        // to_lowercase handles non-ASCII: 'É' should match 'é'
        assert!(exact_match("É", "café.rs"));
    }

    #[test]
    fn regex_match_positions_multibyte_returns_char_boundaries() {
        let re = regex::Regex::new("fé").unwrap();
        let pos = regex_match_positions(&re, "café.rs");
        assert!(pos.is_some());
        let positions = pos.unwrap();
        assert_eq!(positions, vec![2, 3]);
    }

    #[test]
    fn cursor_byte_to_column_cjk() {
        // CJK chars are 3 bytes each, 2 display columns each
        let s = "你好";
        let mut state = SearchState::new(SearchMode::Find);
        for ch in s.chars() {
            state.insert_char(ch);
        }
        assert_eq!(state.cursor_pos, 6); // 2 × 3 bytes
        assert_eq!(state.cursor_display_column(), 4); // 2 × 2 columns
    }

    #[test]
    fn exact_match_positions_case_folding_byte_length_change() {
        // İ (U+0130, 2 bytes) lowercases to i̇ (i + U+0307, 3 bytes).
        // Positions must be in the ORIGINAL string, not the lowercased one.
        // Use the lowercased query form that actually matches.
        let target = "İstanbul.txt";
        let query = "i\u{0307}stanbul"; // "i̇stanbul" — the actual lowercase of İstanbul
        let pos = exact_match_positions(query, target);
        assert!(pos.is_some(), "should match case-insensitively");
        let positions = pos.unwrap();
        // Verify all positions are valid char boundaries in the original string
        for &p in &positions {
            assert!(
                target.is_char_boundary(p),
                "position {p} is not a char boundary in {target:?}"
            );
        }
        // İ is at byte 0 (2 bytes), s at byte 2, t at byte 3, etc.
        assert_eq!(positions[0], 0); // İ
        assert_eq!(positions[1], 2); // s
    }

    #[test]
    fn exact_match_positions_mixed_byte_length_case_fold() {
        // Test with a target where lowercasing changes byte lengths mid-string.
        // "AİB" → lowercase "ai̇b". Query "i̇" should match İ in the original.
        let target = "AİB";
        let query = "i\u{0307}"; // lowercase of İ
        let pos = exact_match_positions(query, target);
        assert!(pos.is_some());
        let positions = pos.unwrap();
        for &p in &positions {
            assert!(
                target.is_char_boundary(p),
                "position {p} is not a char boundary in {target:?}"
            );
        }
        // A is 1 byte, İ starts at byte 1 (2 bytes)
        assert_eq!(positions, vec![1]);
    }

    #[test]
    fn exact_match_positions_eszett() {
        // ß (U+00DF, 2 bytes) lowercases to ß (same), no byte change
        let target = "straße.txt";
        let pos = exact_match_positions("straße", target);
        assert!(pos.is_some());
        let positions = pos.unwrap();
        for &p in &positions {
            assert!(
                target.is_char_boundary(p),
                "position {p} is not a char boundary in {target:?}"
            );
        }
    }
}
