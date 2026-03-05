use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::StatefulWidget,
};
use unicode_width::UnicodeWidthChar;

use crate::config::PreviewConfig;
use crate::preview::state::{PreviewKind, PreviewState};
use crate::render::colors;

pub struct PreviewView<'a> {
    pub config: &'a PreviewConfig,
    pub focused: bool,
}

impl StatefulWidget for PreviewView<'_> {
    type State = PreviewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut PreviewState) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Header takes 1 line, content fills the rest
        let header_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        let content_area = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: area.height.saturating_sub(1),
        };

        self.render_header(header_area, buf, state);

        match &state.kind {
            PreviewKind::Empty => {
                render_centered_message(content_area, buf, "No file selected", Color::DarkGray);
            }
            PreviewKind::Loading => {
                render_centered_message(content_area, buf, "Loading...", Color::DarkGray);
            }
            PreviewKind::Error(msg) => {
                render_centered_message(content_area, buf, msg, Color::Red);
            }
            PreviewKind::TooLarge => {
                self.render_content(content_area, buf, state);
            }
            PreviewKind::Text
            | PreviewKind::Rendered
            | PreviewKind::Binary
            | PreviewKind::Directory => {
                self.render_content(content_area, buf, state);
            }
        }
    }
}

impl PreviewView<'_> {
    fn render_header(&self, area: Rect, buf: &mut Buffer, state: &PreviewState) {
        let bg = if self.focused {
            colors::STATUS_BAR_BG
        } else {
            Color::Rgb(0x3C, 0x3C, 0x3C)
        };
        let fg = if self.focused {
            colors::STATUS_BAR_FG
        } else {
            Color::Rgb(0xBB, 0xBB, 0xBB)
        };

        // Fill header background
        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_style(Style::default().bg(bg));
        }

        let filename = state
            .current_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map_or_else(
                || "Preview".to_string(),
                |n| n.to_string_lossy().into_owned(),
            );

        let mut spans = vec![
            Span::styled(" ", Style::default().bg(bg)),
            Span::styled(
                &filename,
                Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
            ),
        ];

        if state.kind == PreviewKind::Rendered {
            spans.push(Span::styled(
                " [MD]",
                Style::default().fg(Color::Cyan).bg(bg).add_modifier(Modifier::BOLD),
            ));
        }

        if !state.file_info.is_empty() {
            spans.push(Span::styled(
                format!("  {}", state.file_info),
                Style::default().fg(fg).bg(bg),
            ));
        }

        // Scroll indicator on the right
        if state.total_lines > 0 {
            let indicator = format!(" {}/{} ", state.scroll_offset + 1, state.total_lines);
            let indicator_width = indicator.len() as u16;
            let left_content = Line::from(spans);
            let left_width = left_content.width() as u16;

            buf.set_line(area.x, area.y, &left_content, area.width);

            if left_width + indicator_width < area.width {
                let indicator_x = area.x + area.width - indicator_width;
                buf.set_string(
                    indicator_x,
                    area.y,
                    &indicator,
                    Style::default().fg(fg).bg(bg),
                );
            }
        } else {
            let line = Line::from(spans);
            buf.set_line(area.x, area.y, &line, area.width);
        }
    }

    fn render_content(&self, area: Rect, buf: &mut Buffer, state: &PreviewState) {
        let height = area.height as usize;
        let gutter_width = if self.config.show_line_numbers && state.kind == PreviewKind::Text {
            let digits = if state.total_lines == 0 {
                1
            } else {
                (state.total_lines as f64).log10().floor() as u16 + 1
            };
            digits + 1
        } else {
            0
        };

        // Pre-compute normalized selection range
        let sel_range = state.selection.normalized();
        let highlight_style = Style::default().bg(colors::SELECTED_BG).fg(Color::White);

        for row in 0..height {
            let line_idx = state.scroll_offset + row;
            let y = area.y + row as u16;

            if line_idx >= state.content.len() {
                break;
            }

            let mut x = area.x;

            // Line number gutter
            if gutter_width > 0 {
                let line_num = format!(
                    "{:>width$} ",
                    line_idx + 1,
                    width = (gutter_width - 1) as usize
                );
                let gutter_style = Style::default().fg(Color::DarkGray);
                buf.set_string(x, y, &line_num, gutter_style);
                x += gutter_width;
            }

            let content_width = area.width.saturating_sub(gutter_width);

            // Determine if this line intersects the selection
            let line_sel = sel_range.and_then(|(start, end)| {
                if line_idx < start.line || line_idx > end.line {
                    return None;
                }
                let col_start = if line_idx == start.line { start.col } else { 0 };
                let col_end = if line_idx == end.line {
                    end.col
                } else {
                    usize::MAX
                };
                Some((col_start, col_end))
            });

            if let Some((sel_col_start, sel_col_end)) = line_sel {
                // Character-by-character rendering for lines with selection
                let mut col: usize = 0;
                for (text, style) in &state.content[line_idx] {
                    if col >= content_width as usize {
                        break;
                    }
                    for ch in text.chars() {
                        if col >= content_width as usize {
                            break;
                        }
                        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
                        if w == 0 {
                            continue;
                        }
                        let s = if col >= sel_col_start && col < sel_col_end {
                            highlight_style
                        } else {
                            *style
                        };
                        let mut char_buf = [0u8; 4];
                        let char_str = ch.encode_utf8(&mut char_buf);
                        buf.set_string(x + col as u16, y, char_str, s);
                        col += w;
                    }
                }
            } else {
                // Fast path: no selection on this line
                let mut col: u16 = 0;
                for (text, style) in &state.content[line_idx] {
                    if col >= content_width {
                        break;
                    }
                    let remaining = (content_width - col) as usize;
                    let mut char_end = 0;
                    let mut width_used = 0;
                    for ch in text.chars() {
                        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
                        if width_used + w > remaining {
                            break;
                        }
                        width_used += w;
                        char_end += ch.len_utf8();
                    }
                    let display = &text[..char_end];
                    buf.set_string(x + col, y, display, *style);
                    col += width_used as u16;
                }
            }
        }
    }
}

fn render_centered_message(area: Rect, buf: &mut Buffer, msg: &str, fg: Color) {
    if area.height == 0 {
        return;
    }
    let y = area.y + area.height / 2;
    let x = area.x + area.width.saturating_sub(msg.len() as u16) / 2;
    buf.set_string(x, y, msg, Style::default().fg(fg));
}
