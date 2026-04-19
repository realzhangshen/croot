use std::collections::HashMap;
use std::path::PathBuf;

use unicode_width::UnicodeWidthStr;

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
    Unified,
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

/// A single match line within a file group (content search).
#[derive(Debug, Clone)]
pub struct ContentMatch {
    pub line: Option<usize>,
    pub context: Option<String>,
}

/// A file group containing all matches for a single file (content search).
#[derive(Debug, Clone)]
pub struct FileGroup {
    pub path: PathBuf,
    pub display: String,
    pub matches: Vec<ContentMatch>,
    pub collapsed: bool,
}

/// An item in the flattened visible list for grouped content search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupedItem {
    FileResult(usize),
    FileHeader(usize),
    MatchLine(usize, usize),
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
    pub global_results: Vec<GlobalSearchResult>,
    /// Grouped results for content search (VS Code-style).
    pub grouped_results: Vec<FileGroup>,
    pub global_selected: usize,
    pub global_scroll_offset: usize,
    pub file_loading: bool,
    pub content_loading: bool,
    pub file_error: Option<String>,
    pub content_error: Option<String>,
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
            grouped_results: Vec::new(),
            global_selected: 0,
            global_scroll_offset: 0,
            file_loading: false,
            content_loading: false,
            file_error: None,
            content_error: None,
            global_loading: false,
            global_error: None,
            global_search_type: GlobalSearchType::Unified,
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

    pub fn insert_str(&mut self, s: &str) {
        self.query.insert_str(self.cursor_pos, s);
        self.cursor_pos += s.len();
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
        self.grouped_results.clear();
        self.global_selected = 0;
        self.global_scroll_offset = 0;
        self.file_loading = false;
        self.content_loading = false;
        self.file_error = None;
        self.content_error = None;
        self.recompute_global_status();
    }

    pub fn is_empty(&self) -> bool {
        self.query.is_empty()
    }

    /// Byte offset within `query` to display column width, bridging
    /// byte-based `cursor_pos` and the screen column where the cursor renders.
    pub fn cursor_display_column(&self) -> usize {
        cursor_byte_to_column(&self.query, self.cursor_pos)
    }

    /// Determine match mode and effective query string.
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

    // ── Grouped content search helpers ─────────────────────────────────

    /// Total visible rows in grouped content search view.
    pub fn content_visible_item_count(&self) -> usize {
        self.grouped_results
            .iter()
            .map(|g| if g.collapsed { 1 } else { 1 + g.matches.len() })
            .sum()
    }

    pub fn content_match_count(&self) -> usize {
        self.grouped_results.iter().map(|g| g.matches.len()).sum()
    }

    pub fn has_any_results(&self) -> bool {
        !self.global_results.is_empty() || !self.grouped_results.is_empty()
    }

    pub fn recompute_global_status(&mut self) {
        self.global_loading = self.file_loading || self.content_loading;
        self.global_error = match (&self.file_error, &self.content_error) {
            (Some(file), Some(content)) => Some(format!("files: {file}; text: {content}")),
            (Some(file), None) => Some(format!("files: {file}")),
            (None, Some(content)) => Some(format!("text: {content}")),
            (None, None) => None,
        };
    }

    /// Total visible rows in the active global search view.
    pub fn visible_item_count(&self) -> usize {
        match self.global_search_type {
            GlobalSearchType::Unified => {
                self.global_results.len() + self.content_visible_item_count()
            }
            GlobalSearchType::FileName => self.global_results.len(),
            GlobalSearchType::Content => self.content_visible_item_count(),
        }
    }

    /// Map a flat visible-row index to a logical `GroupedItem`.
    pub fn resolve_item(&self, flat_idx: usize) -> Option<GroupedItem> {
        let file_count = match self.global_search_type {
            GlobalSearchType::Unified | GlobalSearchType::FileName => self.global_results.len(),
            GlobalSearchType::Content => 0,
        };
        if flat_idx < file_count {
            return Some(GroupedItem::FileResult(flat_idx));
        }

        let mut remaining = flat_idx.saturating_sub(file_count);
        for (gi, group) in self.grouped_results.iter().enumerate() {
            if remaining == 0 {
                return Some(GroupedItem::FileHeader(gi));
            }
            remaining -= 1; // consumed the header
            if !group.collapsed {
                if remaining < group.matches.len() {
                    return Some(GroupedItem::MatchLine(gi, remaining));
                }
                remaining -= group.matches.len();
            }
        }
        None
    }

    /// Flat row index of a group's header.
    pub fn flat_index_of_header(&self, group_idx: usize) -> usize {
        let mut idx = if self.global_search_type == GlobalSearchType::Unified {
            self.global_results.len()
        } else {
            0
        };
        for (gi, group) in self.grouped_results.iter().enumerate() {
            if gi == group_idx {
                return idx;
            }
            idx += 1; // header
            if !group.collapsed {
                idx += group.matches.len();
            }
        }
        idx
    }

    /// Clamp `global_selected` and `global_scroll_offset` to valid range.
    /// Call after any state change that affects visible row count.
    pub fn clamp_selection(&mut self) {
        let count = self.visible_item_count();
        if count == 0 {
            self.global_selected = 0;
            self.global_scroll_offset = 0;
            return;
        }
        if self.global_selected >= count {
            self.global_selected = count - 1;
        }
        if self.global_visible_height > 0
            && self.global_scroll_offset + self.global_visible_height > count
        {
            self.global_scroll_offset = count.saturating_sub(self.global_visible_height);
        }
        if self.global_selected < self.global_scroll_offset {
            self.global_scroll_offset = self.global_selected;
        }
    }

    pub fn move_global_selection_up(&mut self, amount: usize) {
        if self.visible_item_count() == 0 {
            self.global_selected = 0;
            self.global_scroll_offset = 0;
            return;
        }

        self.global_selected = self.global_selected.saturating_sub(amount);
        if self.global_selected < self.global_scroll_offset {
            self.global_scroll_offset = self.global_selected;
        }
    }

    pub fn move_global_selection_down(&mut self, amount: usize) {
        let count = self.visible_item_count();
        if count == 0 {
            self.global_selected = 0;
            self.global_scroll_offset = 0;
            return;
        }

        self.global_selected = self
            .global_selected
            .saturating_add(amount)
            .min(count.saturating_sub(1));
        let visible = self.global_visible_height;
        if visible > 0 && self.global_selected >= self.global_scroll_offset + visible {
            self.global_scroll_offset = self.global_selected.saturating_sub(visible - 1);
        }
    }

    pub fn page_global_selection_up(&mut self) {
        let count = self.visible_item_count();
        if count == 0 {
            self.global_selected = 0;
            self.global_scroll_offset = 0;
            return;
        }

        let step = self.global_visible_height.max(1);
        self.global_selected = self.global_selected.saturating_sub(step);
        if self.global_visible_height > 0 {
            self.global_scroll_offset = self.global_scroll_offset.saturating_sub(step);
        }
        self.clamp_selection();
    }

    pub fn page_global_selection_down(&mut self) {
        let count = self.visible_item_count();
        if count == 0 {
            self.global_selected = 0;
            self.global_scroll_offset = 0;
            return;
        }

        let step = self.global_visible_height.max(1);
        self.global_selected = self
            .global_selected
            .saturating_add(step)
            .min(count.saturating_sub(1));
        if self.global_visible_height > 0 {
            let max_scroll = count.saturating_sub(self.global_visible_height);
            self.global_scroll_offset = self
                .global_scroll_offset
                .saturating_add(step)
                .min(max_scroll);
        }
        self.clamp_selection();
    }
}

/// Convert a byte offset within a string to its display column width.
pub(crate) fn cursor_byte_to_column(s: &str, byte_pos: usize) -> usize {
    UnicodeWidthStr::width(&s[..byte_pos])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn search_state_new_defaults() {
        let state = SearchState::new(SearchMode::Find);
        assert_eq!(state.mode, SearchMode::Find);
        assert!(state.query.is_empty());
        assert_eq!(state.cursor_pos, 0);
        assert_eq!(state.match_count(), 0);
    }

    #[test]
    fn search_mode_variants() {
        assert_ne!(SearchMode::Find, SearchMode::Filter);
        assert_ne!(SearchMode::Filter, SearchMode::Global);
    }

    #[test]
    fn match_mode_variants() {
        assert_ne!(MatchMode::Fuzzy, MatchMode::Regex);
        assert_ne!(MatchMode::Regex, MatchMode::Exact);
    }

    #[test]
    fn unified_visible_items_include_file_results_and_content_rows() {
        let mut state = SearchState::new(SearchMode::Global);
        state.global_search_type = GlobalSearchType::Unified;
        state.global_results = vec![
            GlobalSearchResult {
                path: PathBuf::from("src/main.rs"),
                display: "src/main.rs".into(),
                line: None,
                context: None,
            },
            GlobalSearchResult {
                path: PathBuf::from("src/lib.rs"),
                display: "src/lib.rs".into(),
                line: None,
                context: None,
            },
        ];
        state.grouped_results = vec![FileGroup {
            path: PathBuf::from("src/app.rs"),
            display: "src/app.rs".into(),
            matches: vec![
                ContentMatch {
                    line: Some(12),
                    context: Some("fn start_search()".into()),
                },
                ContentMatch {
                    line: Some(48),
                    context: Some("render_search_overlay();".into()),
                },
            ],
            collapsed: false,
        }];

        assert_eq!(state.visible_item_count(), 5);
        assert_eq!(state.resolve_item(0), Some(GroupedItem::FileResult(0)));
        assert_eq!(state.resolve_item(1), Some(GroupedItem::FileResult(1)));
        assert_eq!(state.resolve_item(2), Some(GroupedItem::FileHeader(0)));
        assert_eq!(state.resolve_item(3), Some(GroupedItem::MatchLine(0, 0)));
        assert_eq!(state.resolve_item(4), Some(GroupedItem::MatchLine(0, 1)));
    }

    #[test]
    fn move_global_selection_down_keeps_selection_visible() {
        let mut state = SearchState::new(SearchMode::Global);
        state.global_visible_height = 5;
        state.global_results = (0..20)
            .map(|i| GlobalSearchResult {
                path: PathBuf::from(format!("file{i}.rs")),
                display: format!("file{i}.rs"),
                line: None,
                context: None,
            })
            .collect();

        state.move_global_selection_down(6);

        assert_eq!(state.global_selected, 6);
        assert_eq!(state.global_scroll_offset, 2);
    }

    #[test]
    fn page_global_selection_down_advances_by_a_full_page() {
        let mut state = SearchState::new(SearchMode::Global);
        state.global_visible_height = 4;
        state.global_results = (0..12)
            .map(|i| GlobalSearchResult {
                path: PathBuf::from(format!("file{i}.rs")),
                display: format!("file{i}.rs"),
                line: None,
                context: None,
            })
            .collect();

        state.page_global_selection_down();

        assert_eq!(state.global_selected, 4);
        assert_eq!(state.global_scroll_offset, 4);
    }

    #[test]
    fn page_global_selection_up_moves_to_previous_page() {
        let mut state = SearchState::new(SearchMode::Global);
        state.global_visible_height = 4;
        state.global_results = (0..12)
            .map(|i| GlobalSearchResult {
                path: PathBuf::from(format!("file{i}.rs")),
                display: format!("file{i}.rs"),
                line: None,
                context: None,
            })
            .collect();
        state.global_selected = 8;
        state.global_scroll_offset = 8;

        state.page_global_selection_up();

        assert_eq!(state.global_selected, 4);
        assert_eq!(state.global_scroll_offset, 4);
    }
}
