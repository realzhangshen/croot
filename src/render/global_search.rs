use ratatui::{buffer::Buffer, layout::Rect, style::Modifier, widgets::Widget};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::colors;
use super::popup::draw_border;
use crate::search::{exact_match_positions, regex_match_positions, GlobalSearchType, SearchState};

const MIN_DIALOG_WIDTH: u16 = 40;
const MIN_DIALOG_HEIGHT: u16 = 10;
const DIALOG_MARGIN: u16 = 4;
const HEADER_ROWS: u16 = 3;
const FOOTER_ROWS: u16 = 3;

#[derive(Debug, Clone, Copy)]
struct GlobalSearchLayout {
    dialog: Rect,
    input_y: u16,
    separator_y: u16,
    results_y: u16,
    results_height: usize,
    footer_status_y: u16,
    footer_actions_y: u16,
    content_width: usize,
    footer_width: usize,
}

fn global_search_layout(area: Rect) -> GlobalSearchLayout {
    let dialog = global_search_rect(area);
    GlobalSearchLayout {
        input_y: dialog.y + 1,
        separator_y: dialog.y + 2,
        results_y: dialog.y + 3,
        results_height: dialog.height.saturating_sub(HEADER_ROWS + FOOTER_ROWS) as usize,
        footer_status_y: dialog.y + dialog.height.saturating_sub(3),
        footer_actions_y: dialog.y + dialog.height.saturating_sub(2),
        content_width: dialog.width.saturating_sub(3) as usize,
        footer_width: dialog.width.saturating_sub(4) as usize,
        dialog,
    }
}

/// Compute the centered overlay rect for the global search dialog.
/// Shared between render and mouse-hit-test to avoid layout drift.
pub fn global_search_rect(area: Rect) -> Rect {
    let width = (area.width * 3 / 5)
        .max(MIN_DIALOG_WIDTH)
        .min(area.width.saturating_sub(DIALOG_MARGIN));
    let height = (area.height * 3 / 5)
        .max(MIN_DIALOG_HEIGHT)
        .min(area.height.saturating_sub(DIALOG_MARGIN));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

pub(crate) fn global_search_results_height(area: Rect) -> usize {
    global_search_layout(area).results_height
}

/// Overlay widget for global file/content search (fd/rg).
pub struct GlobalSearchOverlay<'a> {
    pub state: &'a SearchState,
}

fn highlight_positions(query: &str, target: &str) -> Vec<usize> {
    if query.is_empty() {
        return Vec::new();
    }
    if let Some(positions) = exact_match_positions(query, target) {
        return positions;
    }

    let smart_case_insensitive = !query.chars().any(char::is_uppercase);
    let regex = regex::RegexBuilder::new(query)
        .case_insensitive(smart_case_insensitive)
        .build()
        .ok();
    regex
        .and_then(|re| regex_match_positions(&re, target))
        .unwrap_or_default()
}

fn highlight_style(base: ratatui::style::Style, selected: bool) -> ratatui::style::Style {
    if selected {
        base.add_modifier(Modifier::UNDERLINED)
    } else {
        base.fg(colors::find_match())
            .add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
    }
}

#[allow(clippy::too_many_arguments)]
fn render_highlighted_text(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    text: &str,
    max_width: usize,
    base: ratatui::style::Style,
    highlight: ratatui::style::Style,
    positions: &[usize],
) {
    if max_width == 0 {
        return;
    }

    let mut used = 0usize;
    let mut cursor_x = x;
    let mut truncated = false;
    let mut chars = text.char_indices().peekable();

    while let Some((byte_idx, ch)) = chars.next() {
        let width = UnicodeWidthChar::width(ch).unwrap_or(0);
        let has_more = chars.peek().is_some();
        if used + width > max_width || (has_more && used + width == max_width) {
            truncated = true;
            break;
        }

        let style = if positions.contains(&byte_idx) {
            highlight
        } else {
            base
        };
        buf.set_string(cursor_x, y, ch.to_string(), style);
        cursor_x = cursor_x.saturating_add(width as u16);
        used += width;
    }

    if truncated {
        buf.set_string(cursor_x, y, "…", base);
    }
}

fn footer_actions(max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let candidates = [
        "[Enter] open/toggle  [Tab] reveal path  [←/→] page  [Esc] close",
        "[Enter] open  [Tab] reveal path  [←/→] page  [Esc] close",
        "[Enter] open  [Tab] path  [←/→] page  [Esc] close",
        "Enter open  Tab path  Left/Right page  Esc close",
        "Enter open  Left/Right page  Esc close",
        "Enter open  Page  Esc close",
        "Enter  Page  Esc",
    ];

    for candidate in candidates {
        if UnicodeWidthStr::width(candidate) <= max_width {
            return candidate.to_string();
        }
    }

    super::text_util::truncate_with_ellipsis("Enter open  Page  Esc close", max_width)
}

fn footer_status(state: &SearchState) -> Option<(String, ratatui::style::Style)> {
    let has_results = state.has_any_results();

    if let Some(err) = state.global_error.as_deref() {
        return Some((err.to_string(), colors::popup_warning()));
    }

    if state.global_loading && !has_results {
        return Some((
            "Searching workspace...".to_string(),
            colors::popup_warning(),
        ));
    }

    if !has_results {
        if state.query.is_empty() {
            return None;
        }
        return Some(("No results".to_string(), colors::popup_dim()));
    }

    let summary = match state.global_search_type {
        GlobalSearchType::Unified => format!(
            "{} paths, {} text files, {} matches",
            state.global_results.len(),
            state.grouped_results.len(),
            state.content_match_count()
        ),
        GlobalSearchType::FileName if !state.global_results.is_empty() => {
            format!(
                "{}/{}",
                state.global_selected + 1,
                state.global_results.len()
            )
        }
        GlobalSearchType::Content if !state.grouped_results.is_empty() => format!(
            "{} files, {} matches",
            state.grouped_results.len(),
            state.content_match_count()
        ),
        _ => String::new(),
    };

    if summary.is_empty() {
        None
    } else {
        Some((summary, colors::popup_success()))
    }
}

impl Widget for GlobalSearchOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height < 6 {
            return;
        }

        let layout = global_search_layout(area);
        let dialog = layout.dialog;

        let base = colors::popup_base();
        let border_style = colors::popup_border();

        colors::clear_region(buf, dialog, base);
        draw_border(buf, dialog, border_style);

        let title = match self.state.global_search_type {
            GlobalSearchType::Unified => " Search Workspace ",
            GlobalSearchType::FileName => " Search Paths ",
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

        let input_style = colors::popup_input();
        for col in (dialog.x + 1)..(dialog.x + dialog.width.saturating_sub(1)) {
            if let Some(cell) = buf.cell_mut((col, layout.input_y)) {
                cell.reset();
                cell.set_style(input_style);
            }
        }

        let prompt = " > ";
        buf.set_string(dialog.x + 1, layout.input_y, prompt, colors::popup_prompt());

        let input_x = dialog.x + 1 + prompt.len() as u16;
        // Available width: dialog.width - left_border(1) - prompt(3) - right_border(1)
        let input_width = dialog.width.saturating_sub(1 + prompt.len() as u16 + 1) as usize;
        let query_display =
            super::text_util::truncate_start_to_display_width(&self.state.query, input_width);
        buf.set_string(input_x, layout.input_y, &query_display, input_style);

        let query_display_width = UnicodeWidthStr::width(self.state.query.as_str());
        let cursor_pos = if query_display_width > input_width {
            input_width
        } else {
            self.state.cursor_display_column()
        };
        if let Some(cell) = buf.cell_mut((input_x + cursor_pos as u16, layout.input_y)) {
            cell.set_style(colors::popup_cursor());
            if cell.symbol() == " " || cell.symbol().is_empty() {
                cell.set_symbol(" ");
            }
        }

        for col in (dialog.x + 1)..(dialog.x + dialog.width.saturating_sub(1)) {
            if let Some(cell) = buf.cell_mut((col, layout.separator_y)) {
                cell.set_symbol("─");
                cell.set_style(border_style);
            }
        }

        let has_results = self.state.has_any_results();

        if has_results && layout.results_height > 0 {
            self.render_results(
                buf,
                dialog,
                layout.results_y,
                layout.results_height,
                layout.content_width,
                base,
            );
        }

        if let Some((status, style)) = footer_status(self.state) {
            let display = super::text_util::truncate_with_ellipsis(&status, layout.footer_width);
            buf.set_string(dialog.x + 2, layout.footer_status_y, display, style);
        }

        let actions = footer_actions(layout.footer_width);
        if !actions.is_empty() {
            buf.set_string(
                dialog.x + 2,
                layout.footer_actions_y,
                actions,
                colors::popup_dim(),
            );
        }
    }
}

impl GlobalSearchOverlay<'_> {
    fn render_results(
        &self,
        buf: &mut Buffer,
        dialog: Rect,
        results_y: u16,
        results_height: usize,
        content_width: usize,
        base: ratatui::style::Style,
    ) {
        use ratatui::style::Modifier;

        let start = self
            .state
            .global_scroll_offset
            .min(self.state.visible_item_count());
        let end = (start + results_height).min(self.state.visible_item_count());

        for (row, flat_idx) in (start..end).enumerate() {
            let row_y = results_y + row as u16;
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

            let Some(item) = self.state.resolve_item(flat_idx) else {
                continue;
            };
            match item {
                crate::search::GroupedItem::FileResult(idx) => {
                    let Some(result) = self.state.global_results.get(idx) else {
                        continue;
                    };
                    let label = if self.state.global_search_type == GlobalSearchType::Unified {
                        "[path] "
                    } else {
                        ""
                    };
                    let start_x = dialog.x + 2;
                    buf.set_string(start_x, row_y, label, style);
                    let consumed = UnicodeWidthStr::width(label);
                    let max_width = content_width.saturating_sub(consumed);
                    let positions = highlight_positions(&self.state.query, &result.display);
                    render_highlighted_text(
                        buf,
                        start_x + consumed as u16,
                        row_y,
                        &result.display,
                        max_width,
                        style,
                        highlight_style(style, is_selected),
                        &positions,
                    );
                }
                crate::search::GroupedItem::FileHeader(g) => {
                    let Some(group) = self.state.grouped_results.get(g) else {
                        continue;
                    };
                    let indicator = if group.collapsed { "▶ " } else { "▼ " };
                    let match_label = if group.matches.len() == 1 {
                        " (1 match)".to_string()
                    } else {
                        format!(" ({} matches)", group.matches.len())
                    };
                    let prefix = if self.state.global_search_type == GlobalSearchType::Unified {
                        "[text] "
                    } else {
                        ""
                    };
                    let start_x = dialog.x + 2;
                    let header_prefix = format!("{prefix}{indicator}");
                    buf.set_string(
                        start_x,
                        row_y,
                        &header_prefix,
                        style.add_modifier(Modifier::BOLD),
                    );
                    let prefix_width = UnicodeWidthStr::width(header_prefix.as_str());
                    let label_width = UnicodeWidthStr::width(match_label.as_str());
                    let available = content_width
                        .saturating_sub(prefix_width)
                        .saturating_sub(label_width);
                    let positions = highlight_positions(&self.state.query, &group.display);
                    render_highlighted_text(
                        buf,
                        start_x + prefix_width as u16,
                        row_y,
                        &group.display,
                        available,
                        style.add_modifier(Modifier::BOLD),
                        highlight_style(style.add_modifier(Modifier::BOLD), is_selected),
                        &positions,
                    );
                    let display_width =
                        UnicodeWidthStr::width(group.display.as_str()).min(available);
                    let label_x = start_x + (prefix_width + display_width) as u16;
                    let remaining = content_width.saturating_sub(prefix_width + display_width);
                    if remaining > 0 {
                        let label =
                            super::text_util::truncate_with_ellipsis(&match_label, remaining);
                        buf.set_string(label_x, row_y, &label, style.add_modifier(Modifier::BOLD));
                    }
                }
                crate::search::GroupedItem::MatchLine(g, m) => {
                    let Some(group) = self.state.grouped_results.get(g) else {
                        continue;
                    };
                    let Some(matched) = group.matches.get(m) else {
                        continue;
                    };
                    let (prefix, content) = match (matched.line, &matched.context) {
                        (Some(ln), Some(ctx)) => {
                            (format!("        {:>5}: ", ln), ctx.trim().to_string())
                        }
                        (Some(ln), None) => (format!("        {:>5}: ", ln), String::new()),
                        (None, Some(ctx)) => ("        ".to_string(), ctx.trim().to_string()),
                        (None, None) => ("        ".to_string(), "...".to_string()),
                    };
                    let start_x = dialog.x + 2;
                    buf.set_string(start_x, row_y, &prefix, style);
                    let prefix_width = UnicodeWidthStr::width(prefix.as_str());
                    let positions = highlight_positions(&self.state.query, &content);
                    render_highlighted_text(
                        buf,
                        start_x + prefix_width as u16,
                        row_y,
                        &content,
                        content_width.saturating_sub(prefix_width),
                        style,
                        highlight_style(style, is_selected),
                        &positions,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{SearchMode, SearchState};
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

    use crate::search::{ContentMatch, FileGroup};
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

        let layout = global_search_layout(area);
        let text: String = (layout.dialog.x + 1..layout.dialog.x + layout.dialog.width - 1)
            .filter_map(|x| {
                buf.cell((x, layout.footer_status_y))
                    .map(|c| c.symbol().to_string())
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

    #[test]
    fn unified_render_shows_path_and_text_labels() {
        let mut state = make_content_state(sample_groups());
        state.global_search_type = GlobalSearchType::Unified;
        state.global_results = vec![crate::search::GlobalSearchResult {
            path: PathBuf::from("src/main.rs"),
            display: "src/main.rs".into(),
            line: None,
            context: None,
        }];

        let area = Rect::new(0, 0, 90, 28);
        let mut buf = Buffer::empty(area);
        let widget = GlobalSearchOverlay { state: &state };
        widget.render(area, &mut buf);

        let mut all_text = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                if let Some(cell) = buf.cell((x, y)) {
                    all_text.push_str(cell.symbol());
                }
            }
        }

        assert!(all_text.contains("[path]"));
        assert!(all_text.contains("[text]"));
    }

    #[test]
    fn unified_render_highlights_file_match_characters() {
        let mut state = SearchState::new(SearchMode::Global);
        state.global_search_type = GlobalSearchType::Unified;
        state.query = "main".into();
        state.global_results = vec![crate::search::GlobalSearchResult {
            path: PathBuf::from("src/main.rs"),
            display: "src/main.rs".into(),
            line: None,
            context: None,
        }];
        state.grouped_results = vec![FileGroup {
            path: PathBuf::from("src/app.rs"),
            display: "src/app.rs".into(),
            matches: vec![ContentMatch {
                line: Some(12),
                context: Some("fn bootstrap()".into()),
            }],
            collapsed: false,
        }];
        state.global_selected = 1;

        let area = Rect::new(0, 0, 90, 28);
        let mut buf = Buffer::empty(area);
        let widget = GlobalSearchOverlay { state: &state };
        widget.render(area, &mut buf);

        let dialog = global_search_rect(area);
        let row_y = dialog.y + 3;
        let match_x = dialog.x + 2 + "[path] src/".len() as u16;
        let cell = buf.cell((match_x, row_y)).unwrap();

        assert_eq!(cell.fg, colors::find_match());
        assert!(cell.modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn footer_actions_do_not_render_past_dialog_border_on_narrow_layouts() {
        let state = SearchState::new(SearchMode::Global);
        let area = Rect::new(0, 0, 50, 20);
        let mut buf = Buffer::empty(area);
        let widget = GlobalSearchOverlay { state: &state };
        widget.render(area, &mut buf);

        let dialog = global_search_rect(area);
        let footer_y = dialog.y + dialog.height.saturating_sub(2);

        for x in dialog.x + dialog.width..area.width {
            let cell = buf.cell((x, footer_y)).unwrap();
            assert_eq!(
                cell.symbol(),
                " ",
                "footer action text overflowed outside dialog at x={x}: {:?}",
                cell.symbol()
            );
        }
    }

    #[test]
    fn footer_reserves_a_dedicated_status_row_for_results() {
        let mut state = SearchState::new(SearchMode::Global);
        state.global_search_type = GlobalSearchType::FileName;
        state.global_results = vec![
            crate::search::GlobalSearchResult {
                path: PathBuf::from("src/main.rs"),
                display: "src/main.rs".into(),
                line: None,
                context: None,
            },
            crate::search::GlobalSearchResult {
                path: PathBuf::from("src/lib.rs"),
                display: "src/lib.rs".into(),
                line: None,
                context: None,
            },
            crate::search::GlobalSearchResult {
                path: PathBuf::from("src/app.rs"),
                display: "src/app.rs".into(),
                line: None,
                context: None,
            },
            crate::search::GlobalSearchResult {
                path: PathBuf::from("src/config.rs"),
                display: "src/config.rs".into(),
                line: None,
                context: None,
            },
        ];

        let area = Rect::new(0, 0, 80, 18);
        let mut buf = Buffer::empty(area);
        let widget = GlobalSearchOverlay { state: &state };
        widget.render(area, &mut buf);

        let dialog = global_search_rect(area);
        let status_y = dialog.y + dialog.height.saturating_sub(3);
        let status_text: String = (dialog.x + 1..dialog.x + dialog.width.saturating_sub(1))
            .filter_map(|x| {
                buf.cell((x, status_y))
                    .map(|cell| cell.symbol().to_string())
            })
            .collect();

        assert!(
            status_text.contains("1/4"),
            "expected footer status row to show selection summary, got {status_text:?}"
        );
        assert!(
            !status_text.contains("src/config.rs"),
            "expected footer status row to stay reserved for footer content, got {status_text:?}"
        );
    }
}
