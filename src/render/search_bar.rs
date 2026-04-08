use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use unicode_width::UnicodeWidthStr;

use super::colors;

// Re-export search types and matchers so existing `use crate::render::search_bar::*` still works.
pub use crate::search::matcher::{
    do_match, do_match_positions, exact_match, exact_match_positions, fuzzy_match,
    fuzzy_match_positions, regex_match, regex_match_positions,
};
pub(crate) use crate::search::types::cursor_byte_to_column;
pub use crate::search::types::{
    ContentMatch, FileGroup, GlobalSearchResult, GlobalSearchType, GroupedItem, MatchMode,
    SearchMode, SearchState,
};

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

        let prompt_width = UnicodeWidthStr::width(prompt) as u16;
        let input_x = area.x + prompt_width;
        // Reserve space for match info + optional close button on the right
        let close_width: u16 = if self.show_close_button { 4 } else { 0 }; // "[×] "
        let match_info_max: u16 = 8; // " 0/0 " or similar
        let right_reserve = close_width + match_info_max;
        let input_width = area.width.saturating_sub(prompt_width + right_reserve) as usize;

        // Draw query text
        let display_text =
            super::text_util::truncate_start_to_display_width(&self.state.query, input_width);
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
