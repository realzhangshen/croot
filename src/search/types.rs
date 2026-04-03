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
    // Global search fields
    pub global_results: Vec<GlobalSearchResult>,
    /// Grouped results for content search (VS Code-style).
    pub grouped_results: Vec<FileGroup>,
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
            grouped_results: Vec::new(),
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

    // ── Grouped content search helpers ─────────────────────────────────

    /// Total visible rows in grouped content search view.
    pub fn visible_item_count(&self) -> usize {
        self.grouped_results
            .iter()
            .map(|g| if g.collapsed { 1 } else { 1 + g.matches.len() })
            .sum()
    }

    /// Map a flat visible-row index to a logical `GroupedItem`.
    pub fn resolve_item(&self, flat_idx: usize) -> Option<GroupedItem> {
        let mut remaining = flat_idx;
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
        let mut idx = 0;
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
}

/// Convert a byte offset within a string to its display column width.
pub(crate) fn cursor_byte_to_column(s: &str, byte_pos: usize) -> usize {
    UnicodeWidthStr::width(&s[..byte_pos])
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
