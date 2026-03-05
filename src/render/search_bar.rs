use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use super::colors;

/// State for the search/filter bar.
#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub cursor_pos: usize,
    pub match_count: usize,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            cursor_pos: 0,
            match_count: 0,
        }
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
        self.match_count = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.query.is_empty()
    }
}

pub struct SearchBar<'a> {
    pub state: &'a SearchState,
}

impl Widget for SearchBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg = colors::STATUS_BAR_BG;
        let style = Style::default().fg(Color::Reset).bg(bg);

        // Fill background
        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_style(style);
                cell.set_symbol(" ");
            }
        }

        // Search icon and prompt
        let prompt = " / ";
        buf.set_string(
            area.x,
            area.y,
            prompt,
            Style::default()
                .fg(Color::Cyan)
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
            Style::default().fg(Color::White).bg(bg),
        );

        // Draw cursor
        let cursor_display_pos = if self.state.query.len() > input_width {
            input_width
        } else {
            self.state.cursor_pos
        };
        if let Some(cell) = buf.cell_mut((input_x + cursor_display_pos as u16, area.y)) {
            cell.set_style(Style::default().fg(Color::Black).bg(Color::White));
            if cell.symbol() == " " || cell.symbol().is_empty() {
                cell.set_symbol(" ");
            }
        }

        // Match count on the right
        let match_info = if self.state.query.is_empty() {
            String::new()
        } else {
            format!(" {} matches ", self.state.match_count)
        };
        if !match_info.is_empty() {
            let info_x = area.x + area.width - match_info.len() as u16;
            let info_style = if self.state.match_count > 0 {
                Style::default().fg(Color::Green).bg(bg)
            } else {
                Style::default().fg(Color::Red).bg(bg)
            };
            buf.set_string(info_x, area.y, &match_info, info_style);
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
}
