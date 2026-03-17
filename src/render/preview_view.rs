use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::StatefulWidget,
};
use unicode_width::UnicodeWidthChar;

use crate::config::PreviewConfig;
use crate::git::diff::LineDiffStatus;
use crate::preview::state::{PreviewKind, PreviewState};
use crate::render::colors;

pub struct PreviewView<'a> {
    pub config: &'a PreviewConfig,
    pub focused: bool,
}

/// Compute the total gutter width (diff column + line numbers) for the preview panel.
///
/// This function is shared between the renderer and `PreviewLayout` (mouse hit-testing)
/// so the two always agree on content start position.
pub fn compute_gutter_width(
    show_line_numbers: bool,
    show_git_diff: bool,
    kind: &PreviewKind,
    total_lines: usize,
    has_diff: bool,
) -> u16 {
    if *kind != PreviewKind::Text {
        return 0;
    }

    let diff_col: u16 = u16::from(show_git_diff && has_diff);

    let line_num_cols: u16 = if show_line_numbers {
        let digits = if total_lines == 0 {
            1
        } else {
            (total_lines as f64).log10().floor() as u16 + 1
        };
        digits + 1 // digits + 1 space separator
    } else {
        0
    };

    diff_col + line_num_cols
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
            #[cfg(feature = "image-preview")]
            PreviewKind::Image => {
                self.render_image(content_area, buf, state);
            }
        }
    }
}

impl PreviewView<'_> {
    fn render_header(&self, area: Rect, buf: &mut Buffer, state: &PreviewState) {
        let bg = if self.focused {
            colors::status_bar_bg()
        } else {
            colors::unfocused_header_bg()
        };
        let fg = if self.focused {
            colors::status_bar_fg()
        } else {
            colors::unfocused_header_fg()
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
                Style::default()
                    .fg(Color::Cyan)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        #[cfg(feature = "image-preview")]
        if state.kind == PreviewKind::Image {
            spans.push(Span::styled(
                " [IMG]",
                Style::default()
                    .fg(Color::Magenta)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
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

    #[cfg(feature = "image-preview")]
    #[allow(clippy::unused_self)]
    fn render_image(&self, area: Rect, buf: &mut Buffer, state: &mut PreviewState) {
        use ratatui_image::{thread::ThreadProtocol, StatefulImage};

        if let Some(ref mut thread_proto) = state.image_state {
            let image_widget = StatefulImage::<ThreadProtocol>::new();
            image_widget.render(area, buf, thread_proto);
        } else {
            render_centered_message(area, buf, "Image preview not available", Color::DarkGray);
        }
    }

    fn render_content(&self, area: Rect, buf: &mut Buffer, state: &PreviewState) {
        let height = area.height as usize;
        let has_diff = state.line_diffs.is_some();
        let gutter_width = compute_gutter_width(
            self.config.show_line_numbers,
            self.config.show_git_diff,
            &state.kind,
            state.total_lines,
            has_diff,
        );
        let diff_col_width: u16 =
            u16::from(self.config.show_git_diff && has_diff && state.kind == PreviewKind::Text);
        let line_num_width = gutter_width.saturating_sub(diff_col_width);

        // Pre-compute normalized selection range
        let sel_range = state.selection.normalized();
        let highlight_style = Style::default().add_modifier(Modifier::REVERSED);

        for row in 0..height {
            let line_idx = state.scroll_offset + row;
            let y = area.y + row as u16;

            if line_idx >= state.content.len() {
                break;
            }

            let mut x = area.x;

            // Git diff indicator column
            if diff_col_width > 0 {
                let diff_status = state
                    .line_diffs
                    .as_ref()
                    .and_then(|diffs| diffs.get(line_idx))
                    .copied()
                    .unwrap_or(LineDiffStatus::Unchanged);
                let (symbol, color) = match diff_status {
                    LineDiffStatus::Added => ("\u{258e}", colors::git_added()), // ▎
                    LineDiffStatus::Modified => ("\u{258e}", colors::git_modified()), // ▎ (blue via palette)
                    LineDiffStatus::DeletedAbove => ("\u{2594}", colors::git_deleted()), // ▔
                    LineDiffStatus::Unchanged => (" ", Color::Reset),
                };
                buf.set_string(x, y, symbol, Style::default().fg(color));
                x += diff_col_width;
            }

            // Line number gutter
            if line_num_width > 0 {
                let line_num = format!(
                    "{:>width$} ",
                    line_idx + 1,
                    width = (line_num_width - 1) as usize
                );
                let gutter_style = Style::default().fg(Color::DarkGray);
                buf.set_string(x, y, &line_num, gutter_style);
                x += line_num_width;
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
                        let s = if col >= sel_col_start && col < sel_col_end {
                            highlight_style
                        } else {
                            *style
                        };
                        if w == 0 {
                            // Zero-width combining char: append to previous cell
                            if col > 0 {
                                let prev_x = x + (col as u16).saturating_sub(1);
                                if let Some(cell) = buf.cell_mut((prev_x, y)) {
                                    let mut sym = cell.symbol().to_string();
                                    sym.push(ch);
                                    cell.set_symbol(&sym);
                                }
                            }
                            continue;
                        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gutter_width_text_with_line_numbers_and_diff() {
        let w = compute_gutter_width(true, true, &PreviewKind::Text, 100, true);
        // 1 diff col + 3 digits + 1 space = 5
        assert_eq!(w, 5);
    }

    #[test]
    fn gutter_width_text_with_line_numbers_no_diff() {
        let w = compute_gutter_width(true, false, &PreviewKind::Text, 100, false);
        // 0 diff + 3 digits + 1 space = 4
        assert_eq!(w, 4);
    }

    #[test]
    fn gutter_width_text_no_line_numbers_with_diff() {
        let w = compute_gutter_width(false, true, &PreviewKind::Text, 100, true);
        // 1 diff col + 0 = 1
        assert_eq!(w, 1);
    }

    #[test]
    fn gutter_width_text_nothing() {
        let w = compute_gutter_width(false, false, &PreviewKind::Text, 100, false);
        assert_eq!(w, 0);
    }

    #[test]
    fn gutter_width_non_text_always_zero() {
        assert_eq!(
            compute_gutter_width(true, true, &PreviewKind::Binary, 100, true),
            0
        );
        assert_eq!(
            compute_gutter_width(true, true, &PreviewKind::Directory, 100, true),
            0
        );
        assert_eq!(
            compute_gutter_width(true, true, &PreviewKind::Rendered, 100, true),
            0
        );
    }

    #[test]
    fn gutter_width_diff_enabled_but_no_diff_data() {
        // show_git_diff is true but has_diff is false (no diff data available)
        let w = compute_gutter_width(true, true, &PreviewKind::Text, 100, false);
        // No diff col, just line numbers: 3+1 = 4
        assert_eq!(w, 4);
    }

    #[test]
    fn gutter_width_scales_with_line_count() {
        // 9 lines → 1 digit + 1 space = 2
        assert_eq!(
            compute_gutter_width(true, false, &PreviewKind::Text, 9, false),
            2
        );
        // 10 lines → 2 digits + 1 space = 3
        assert_eq!(
            compute_gutter_width(true, false, &PreviewKind::Text, 10, false),
            3
        );
        // 1000 lines → 4 digits + 1 space = 5
        assert_eq!(
            compute_gutter_width(true, false, &PreviewKind::Text, 1000, false),
            5
        );
    }
}

fn render_centered_message(area: Rect, buf: &mut Buffer, msg: &str, fg: Color) {
    if area.height == 0 {
        return;
    }
    let y = area.y + area.height / 2;
    let msg_width = unicode_width::UnicodeWidthStr::width(msg) as u16;
    let x = area.x + area.width.saturating_sub(msg_width) / 2;
    buf.set_string(x, y, msg, Style::default().fg(fg));
}
