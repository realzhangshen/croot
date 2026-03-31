use ratatui::{buffer::Buffer, layout::Rect, style::Modifier, widgets::Widget};
use unicode_width::UnicodeWidthStr;

use super::colors;
use super::input_dialog::draw_border;
use crate::render::search_bar::{GlobalSearchType, SearchState};

/// Compute the centered overlay rect for the global search dialog.
/// Shared between render and mouse-hit-test to avoid layout drift.
pub fn global_search_rect(area: Rect) -> Rect {
    let width = (area.width * 3 / 5)
        .max(40)
        .min(area.width.saturating_sub(4));
    let height = (area.height * 3 / 5)
        .max(10)
        .min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

/// Overlay widget for global file/content search (fd/rg).
pub struct GlobalSearchOverlay<'a> {
    pub state: &'a SearchState,
}

impl Widget for GlobalSearchOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Guard: skip rendering if terminal is too small for the dialog
        if area.width < 10 || area.height < 6 {
            return;
        }

        let dialog = global_search_rect(area);

        let base = colors::popup_base();
        let border_style = colors::popup_border();

        // Fill background
        colors::clear_region(buf, dialog, base);

        // Draw border
        draw_border(buf, dialog, border_style);

        // Title
        let title = match self.state.global_search_type {
            GlobalSearchType::FileName => " Search Files ",
            GlobalSearchType::Content => " Search Contents ",
        };
        let title_x = dialog.x
            + (dialog
                .width
                .saturating_sub(UnicodeWidthStr::width(title) as u16))
                / 2;
        buf.set_string(
            title_x,
            dialog.y,
            title,
            colors::popup_border().add_modifier(Modifier::BOLD),
        );

        // Pre-fill input row with sunken input style
        let input_y = dialog.y + 1;
        let input_style = colors::popup_input();
        for col in (dialog.x + 1)..(dialog.x + dialog.width.saturating_sub(1)) {
            if let Some(cell) = buf.cell_mut((col, input_y)) {
                cell.reset();
                cell.set_style(input_style);
            }
        }

        // Input line
        let prompt = " > ";
        buf.set_string(dialog.x + 1, input_y, prompt, colors::popup_prompt());

        let input_x = dialog.x + 1 + prompt.len() as u16;
        // Available width: dialog.width - left_border(1) - prompt(3) - right_border(1)
        let input_width = dialog.width.saturating_sub(1 + prompt.len() as u16 + 1) as usize;
        let query_display =
            super::status_bar::truncate_start_to_display_width(&self.state.query, input_width);
        buf.set_string(input_x, input_y, &query_display, input_style);

        // Cursor (block cursor: swap fg/bg)
        let query_display_width = UnicodeWidthStr::width(self.state.query.as_str());
        let cursor_pos = if query_display_width > input_width {
            input_width
        } else {
            self.state.cursor_display_column()
        };
        if let Some(cell) = buf.cell_mut((input_x + cursor_pos as u16, input_y)) {
            cell.set_style(colors::popup_cursor());
            if cell.symbol() == " " || cell.symbol().is_empty() {
                cell.set_symbol(" ");
            }
        }

        // Separator
        let sep_y = dialog.y + 2;
        for col in (dialog.x + 1)..(dialog.x + dialog.width.saturating_sub(1)) {
            if let Some(cell) = buf.cell_mut((col, sep_y)) {
                cell.set_symbol("─");
                cell.set_style(border_style);
            }
        }

        // Results area
        let results_y = dialog.y + 3;
        let results_height = dialog.height.saturating_sub(5) as usize; // -3 top, -2 bottom
        let content_width = (dialog.width.saturating_sub(3)) as usize;

        if self.state.global_loading {
            buf.set_string(
                dialog.x + 2,
                results_y,
                "Searching...",
                colors::popup_warning(),
            );
        } else if let Some(ref err) = self.state.global_error {
            let display = truncate_str(err, content_width);
            buf.set_string(dialog.x + 2, results_y, display, colors::popup_error());
        } else if self.state.global_search_type == GlobalSearchType::Content {
            // ── Grouped content search results ──
            self.render_grouped_results(
                buf,
                dialog,
                results_y,
                results_height,
                content_width,
                base,
            );
        } else if self.state.global_results.is_empty() {
            if !self.state.query.is_empty() {
                buf.set_string(dialog.x + 2, results_y, "No results", colors::popup_dim());
            }
        } else {
            // ── Flat filename search results ──
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

                if is_selected {
                    for col in (dialog.x + 1)..(dialog.x + dialog.width.saturating_sub(1)) {
                        if let Some(cell) = buf.cell_mut((col, row_y)) {
                            cell.set_style(style);
                        }
                    }
                }

                let display = truncate_str(&result.display, content_width);
                buf.set_string(dialog.x + 2, row_y, &display, style);
            }
        }

        // Help line at bottom
        let help_y = dialog.y + dialog.height.saturating_sub(2);
        let help = if self.state.global_search_type == GlobalSearchType::Content {
            "[Enter] toggle/go to  [Esc] cancel"
        } else {
            "[Enter] go to  [Esc] cancel"
        };
        buf.set_string(dialog.x + 2, help_y, help, colors::popup_dim());

        // Result count
        let (has_results, count_str) = if self.state.global_search_type == GlobalSearchType::Content
        {
            let total_files = self.state.grouped_results.len();
            let total_matches: usize = self
                .state
                .grouped_results
                .iter()
                .map(|g| g.matches.len())
                .sum();
            if total_files > 0 {
                (
                    true,
                    format!(" {} files, {} matches ", total_files, total_matches),
                )
            } else {
                (false, String::new())
            }
        } else if !self.state.global_results.is_empty() {
            (
                true,
                format!(
                    " {}/{} ",
                    self.state.global_selected + 1,
                    self.state.global_results.len()
                ),
            )
        } else {
            (false, String::new())
        };
        if has_results {
            let count_x = dialog
                .x
                .saturating_add(dialog.width.saturating_sub(count_str.width() as u16 + 2));
            if count_x > dialog.x + 2 {
                buf.set_string(count_x, help_y, &count_str, colors::popup_success());
            }
        }
    }
}

impl GlobalSearchOverlay<'_> {
    fn render_grouped_results(
        &self,
        buf: &mut Buffer,
        dialog: Rect,
        results_y: u16,
        results_height: usize,
        content_width: usize,
        base: ratatui::style::Style,
    ) {
        use ratatui::style::Modifier;

        if self.state.grouped_results.is_empty() {
            if !self.state.query.is_empty() {
                buf.set_string(dialog.x + 2, results_y, "No results", colors::popup_dim());
            }
            return;
        }

        let scroll = self.state.global_scroll_offset;
        let mut flat_idx: usize = 0;
        let mut rendered: usize = 0;

        for group in &self.state.grouped_results {
            if rendered >= results_height {
                break;
            }

            // File header row
            if flat_idx >= scroll {
                let row_y = results_y + rendered as u16;
                let is_selected = flat_idx == self.state.global_selected;
                let style = if is_selected {
                    colors::popup_selected()
                } else {
                    base
                };

                if is_selected {
                    for col in (dialog.x + 1)..(dialog.x + dialog.width.saturating_sub(1)) {
                        if let Some(cell) = buf.cell_mut((col, row_y)) {
                            cell.set_style(style);
                        }
                    }
                }

                let indicator = if group.collapsed { "▶ " } else { "▼ " };
                let match_label = if group.matches.len() == 1 {
                    format!(" (1 match)")
                } else {
                    format!(" ({} matches)", group.matches.len())
                };
                let header = format!("{}{}{}", indicator, group.display, match_label);
                let display = truncate_str(&header, content_width);
                buf.set_string(
                    dialog.x + 2,
                    row_y,
                    &display,
                    style.add_modifier(Modifier::BOLD),
                );

                rendered += 1;
            }
            flat_idx += 1;

            // Match lines (skip if collapsed)
            if !group.collapsed {
                for m in &group.matches {
                    if rendered >= results_height {
                        break;
                    }
                    if flat_idx >= scroll {
                        let row_y = results_y + rendered as u16;
                        let is_selected = flat_idx == self.state.global_selected;
                        let style = if is_selected {
                            colors::popup_selected()
                        } else {
                            base
                        };

                        if is_selected {
                            for col in (dialog.x + 1)..(dialog.x + dialog.width.saturating_sub(1)) {
                                if let Some(cell) = buf.cell_mut((col, row_y)) {
                                    cell.set_style(style);
                                }
                            }
                        }

                        let line_text = match (m.line, &m.context) {
                            (Some(ln), Some(ctx)) => format!("   {:>5}: {}", ln, ctx.trim()),
                            (Some(ln), None) => format!("   {:>5}:", ln),
                            (None, Some(ctx)) => format!("   {}", ctx.trim()),
                            (None, None) => "   ...".to_string(),
                        };
                        let display = truncate_str(&line_text, content_width);
                        buf.set_string(dialog.x + 2, row_y, &display, style);

                        rendered += 1;
                    }
                    flat_idx += 1;
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::search_bar::{SearchMode, SearchState};
    use ratatui::style::Modifier;

    #[test]
    fn tiny_terminal_no_panic() {
        let state = SearchState::new(SearchMode::Global);
        // Test various tiny terminal sizes that should not panic
        for (w, h) in [(5, 3), (8, 5), (9, 5), (3, 3), (1, 1), (10, 6)] {
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            let widget = GlobalSearchOverlay { state: &state };
            widget.render(area, &mut buf);
        }
    }

    #[test]
    fn narrow_terminal_no_panic() {
        let state = SearchState::new(SearchMode::Global);
        // Terminal wider than threshold but still narrow
        for w in 10..50 {
            let area = Rect::new(0, 0, w, 10);
            let mut buf = Buffer::empty(area);
            let widget = GlobalSearchOverlay { state: &state };
            widget.render(area, &mut buf);
        }
    }

    #[test]
    fn popup_body_uses_reversed() {
        let state = SearchState::new(SearchMode::Global);
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let widget = GlobalSearchOverlay { state: &state };
        widget.render(area, &mut buf);

        // The dialog is centered; pick a cell in the results area
        let width = (area.width * 3 / 5).max(40).min(area.width - 4);
        let height = (area.height * 3 / 5).max(10).min(area.height - 4);
        let dx = area.x + (area.width - width) / 2;
        let dy = area.y + (area.height - height) / 2;

        // Check a body cell (results area, row 4 from dialog top)
        let cell = buf.cell((dx + 3, dy + 4)).unwrap();
        assert!(
            cell.modifier.contains(Modifier::REVERSED),
            "popup body should have REVERSED, got {:?}",
            cell.modifier
        );
    }

    // ── Grouped content search rendering tests ─────────────────────────

    use crate::render::search_bar::{ContentMatch, FileGroup};
    use std::path::PathBuf;

    fn make_content_state(groups: Vec<FileGroup>) -> SearchState {
        let mut state = SearchState::new(SearchMode::Global);
        state.global_search_type = GlobalSearchType::Content;
        state.grouped_results = groups;
        state
    }

    fn sample_groups() -> Vec<FileGroup> {
        vec![
            FileGroup {
                path: PathBuf::from("src/app.rs"),
                display: "src/app.rs".into(),
                matches: vec![
                    ContentMatch {
                        line: Some(42),
                        context: Some("// TODO: refactor".into()),
                    },
                    ContentMatch {
                        line: Some(108),
                        context: Some("// TODO: error handling".into()),
                    },
                ],
                collapsed: false,
            },
            FileGroup {
                path: PathBuf::from("src/config.rs"),
                display: "src/config.rs".into(),
                matches: vec![ContentMatch {
                    line: Some(15),
                    context: Some("// TODO: validate".into()),
                }],
                collapsed: false,
            },
        ]
    }

    #[test]
    fn grouped_render_no_panic() {
        let state = make_content_state(sample_groups());
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let widget = GlobalSearchOverlay { state: &state };
        widget.render(area, &mut buf);
    }

    #[test]
    fn grouped_render_tiny_terminal_no_panic() {
        let state = make_content_state(sample_groups());
        for (w, h) in [(5, 3), (8, 5), (3, 3), (1, 1), (10, 6)] {
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            let widget = GlobalSearchOverlay { state: &state };
            widget.render(area, &mut buf);
        }
    }

    #[test]
    fn grouped_render_collapsed_no_panic() {
        let mut groups = sample_groups();
        groups[0].collapsed = true;
        let state = make_content_state(groups);
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let widget = GlobalSearchOverlay { state: &state };
        widget.render(area, &mut buf);
    }

    #[test]
    fn grouped_render_empty_no_results_message() {
        let mut state = make_content_state(vec![]);
        state.query = "test".into();
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let widget = GlobalSearchOverlay { state: &state };
        widget.render(area, &mut buf);

        // "No results" should appear somewhere in the buffer
        let text: String = (0..area.width)
            .filter_map(|x| {
                let dialog = global_search_rect(area);
                let y = dialog.y + 3;
                buf.cell((x, y)).map(|c| c.symbol().to_string())
            })
            .collect();
        assert!(
            text.contains("No results"),
            "Expected 'No results' in: {text}"
        );
    }

    #[test]
    fn grouped_render_optional_fields() {
        let groups = vec![FileGroup {
            path: PathBuf::from("x.rs"),
            display: "x.rs".into(),
            matches: vec![ContentMatch {
                line: None,
                context: None,
            }],
            collapsed: false,
        }];
        let state = make_content_state(groups);
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let widget = GlobalSearchOverlay { state: &state };
        widget.render(area, &mut buf);
        // Should not panic
    }
}
