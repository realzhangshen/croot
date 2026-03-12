use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthStr;

use super::colors;
use super::input_dialog::draw_border;
use crate::render::search_bar::{GlobalSearchType, SearchState};

/// Overlay widget for global file/content search (fd/rg).
pub struct GlobalSearchOverlay<'a> {
    pub state: &'a SearchState,
}

impl Widget for GlobalSearchOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Dialog dimensions: centered, ~60% width, ~60% height
        let width = (area.width * 3 / 5)
            .max(40)
            .min(area.width.saturating_sub(4));
        let height = (area.height * 3 / 5)
            .max(10)
            .min(area.height.saturating_sub(4));
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let dialog = Rect::new(x, y, width, height);

        let base = colors::popup_base();
        let border_style = colors::popup_dim();

        // Fill background
        colors::clear_region(buf, dialog, base);

        // Draw border
        draw_border(buf, dialog, border_style);

        // Title
        let title = match self.state.global_search_type {
            GlobalSearchType::FileName => " Search Files ",
            GlobalSearchType::Content => " Search Contents ",
        };
        let title_x = dialog.x + (dialog.width.saturating_sub(title.len() as u16)) / 2;
        buf.set_string(
            title_x,
            dialog.y,
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        );

        // Input line
        let input_y = dialog.y + 1;
        let prompt = " > ";
        buf.set_string(
            dialog.x + 1,
            input_y,
            prompt,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::REVERSED),
        );

        let input_x = dialog.x + 1 + prompt.len() as u16;
        let input_width = (dialog.width - 3) as usize;
        let query_display = if self.state.query.len() > input_width {
            &self.state.query[self.state.query.len() - input_width..]
        } else {
            &self.state.query
        };
        buf.set_string(
            input_x,
            input_y,
            query_display,
            Style::default()
                .fg(Color::Indexed(15))
                .add_modifier(Modifier::REVERSED),
        );

        // Cursor
        let cursor_pos = if self.state.query.len() > input_width {
            input_width
        } else {
            self.state.cursor_pos
        };
        if let Some(cell) = buf.cell_mut((input_x + cursor_pos as u16, input_y)) {
            cell.set_style(Style::default().fg(Color::Black).bg(Color::Indexed(15)));
            if cell.symbol() == " " || cell.symbol().is_empty() {
                cell.set_symbol(" ");
            }
        }

        // Separator
        let sep_y = dialog.y + 2;
        for col in (dialog.x + 1)..(dialog.x + dialog.width - 1) {
            if let Some(cell) = buf.cell_mut((col, sep_y)) {
                cell.set_symbol("─");
                cell.set_style(border_style);
            }
        }

        // Results area
        let results_y = dialog.y + 3;
        let results_height = dialog.height.saturating_sub(5) as usize; // -3 top, -2 bottom
        let content_width = (dialog.width - 3) as usize;

        if self.state.global_loading {
            buf.set_string(
                dialog.x + 2,
                results_y,
                "Searching...",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::REVERSED),
            );
        } else if let Some(ref err) = self.state.global_error {
            let display = truncate_str(err, content_width);
            buf.set_string(
                dialog.x + 2,
                results_y,
                display,
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::REVERSED),
            );
        } else if self.state.global_results.is_empty() {
            if !self.state.query.is_empty() {
                buf.set_string(
                    dialog.x + 2,
                    results_y,
                    "No results",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::REVERSED),
                );
            }
        } else {
            let start = self.state.global_scroll_offset;
            let end = (start + results_height).min(self.state.global_results.len());

            for (i, result) in self.state.global_results[start..end].iter().enumerate() {
                let row_y = results_y + i as u16;
                let is_selected = start + i == self.state.global_selected;

                let style = if is_selected {
                    colors::popup_selected()
                } else {
                    base
                };

                // Fill row background for selected item
                if is_selected {
                    for col in (dialog.x + 1)..(dialog.x + dialog.width - 1) {
                        if let Some(cell) = buf.cell_mut((col, row_y)) {
                            cell.set_style(style);
                        }
                    }
                }

                // Display line
                let display = match self.state.global_search_type {
                    GlobalSearchType::FileName => truncate_str(&result.display, content_width),
                    GlobalSearchType::Content => {
                        if let (Some(line), Some(ref ctx)) = (result.line, &result.context) {
                            let text = format!("{}:{} {}", result.display, line, ctx.trim());
                            truncate_str(&text, content_width)
                        } else {
                            truncate_str(&result.display, content_width)
                        }
                    }
                };

                buf.set_string(dialog.x + 2, row_y, &display, style);
            }
        }

        // Help line at bottom
        let help_y = dialog.y + dialog.height - 2;
        let help = "[Enter] go to  [Esc] cancel";
        buf.set_string(
            dialog.x + 2,
            help_y,
            help,
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::REVERSED),
        );

        // Result count
        if !self.state.global_results.is_empty() {
            let count = format!(
                " {}/{} ",
                self.state.global_selected + 1,
                self.state.global_results.len()
            );
            let count_x = dialog.x + dialog.width - count.width() as u16 - 2;
            if count_x > dialog.x + 2 {
                buf.set_string(
                    count_x,
                    help_y,
                    &count,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::REVERSED),
                );
            }
        }
    }
}

fn truncate_str(s: &str, max_width: usize) -> String {
    if s.width() <= max_width {
        s.to_string()
    } else {
        let mut result = String::new();
        let mut width = 0;
        for ch in s.chars() {
            let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if width + cw > max_width.saturating_sub(1) {
                result.push('…');
                break;
            }
            result.push(ch);
            width += cw;
        }
        result
    }
}
