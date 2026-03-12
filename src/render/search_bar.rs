use std::collections::HashMap;
use std::path::PathBuf;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

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
    pub cursor_pos: usize,
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
        let bg = colors::SEARCH_BAR_BG;
        let style = Style::default().fg(Color::Reset).bg(bg);

        // Fill background
        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_style(style);
                cell.set_symbol(" ");
            }
        }

        // Mode-specific prompt
        let (prompt, prompt_color) = match self.state.mode {
            SearchMode::Find => (" / ", Color::Cyan),
            SearchMode::Filter => (" F ", Color::Yellow),
            SearchMode::Global => (" S ", Color::Magenta),
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
        let input_width = area.width.saturating_sub(prompt.len() as u16 + 12) as usize;

        // Draw query text
        let display_text = if self.state.query.len() > input_width {
            &self.state.query[self.state.query.len() - input_width..]
        } else {
            &self.state.query
        };
        buf.set_string(
            input_x,
            area.y,
            display_text,
            Style::default().fg(Color::Indexed(15)).bg(bg),
        );

        // Draw cursor
        let cursor_display_pos = if self.state.query.len() > input_width {
            input_width
        } else {
            self.state.cursor_pos
        };
        if let Some(cell) = buf.cell_mut((input_x + cursor_display_pos as u16, area.y)) {
            cell.set_style(Style::default().fg(Color::Black).bg(Color::Indexed(15)));
            if cell.symbol() == " " || cell.symbol().is_empty() {
                cell.set_symbol(" ");
            }
        }

        // Close button on the far right
        let close_btn = "[×]";
        let close_reserve = if self.show_close_button {
            close_btn.len() as u16 + 1 // +1 for space
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
        let right_reserved = match_info.len() as u16 + close_reserve;
        if !match_info.is_empty() && area.width > right_reserved {
            let info_x = area.x + area.width - right_reserved;
            let info_style = if self.state.match_count() > 0 {
                Style::default().fg(Color::Green).bg(bg)
            } else {
                Style::default().fg(Color::Red).bg(bg)
            };
            buf.set_string(info_x, area.y, &match_info, info_style);
        }

        // Draw close button (only if it fits)
        if self.show_close_button && area.width > close_btn.len() as u16 + 1 {
            let close_x = area.x + area.width - close_btn.len() as u16 - 1;
            buf.set_string(
                close_x,
                area.y,
                close_btn,
                Style::default()
                    .fg(Color::Red)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
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
    target
        .to_ascii_lowercase()
        .contains(&query.to_ascii_lowercase())
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

/// Exact substring match returning byte positions of the matched range.
pub fn exact_match_positions(query: &str, target: &str) -> Option<Vec<usize>> {
    let target_lower = target.to_ascii_lowercase();
    let query_lower = query.to_ascii_lowercase();
    let start = target_lower.find(&query_lower)?;
    let end = start + query.len();
    Some((start..end).collect())
}

/// Regex match returning byte positions of the first match span.
pub fn regex_match_positions(re: &regex::Regex, target: &str) -> Option<Vec<usize>> {
    let m = re.find(target)?;
    Some((m.start()..m.end()).collect())
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
}
