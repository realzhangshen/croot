use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthStr;

use super::colors;
use super::input_dialog::draw_border;
use super::status_bar::truncate_to_display_width;
use crate::git::branches::BranchInfo;
use crate::render::search_bar::fuzzy_match;

/// The kind of picker being shown (extensible for future use).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerKind {
    Branch,
}

/// A single item in the picker list.
#[derive(Debug, Clone)]
pub struct PickerItem {
    pub label: String,
    pub is_current: bool,
    pub is_separator: bool,
    pub is_remote: bool,
    /// The actionable value (e.g. branch name to switch to).
    pub data: String,
}

/// State for the picker overlay.
#[derive(Debug, Clone)]
pub struct PickerState {
    pub kind: PickerKind,
    pub all_items: Vec<PickerItem>,
    pub filtered_indices: Vec<usize>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub query: String,
    pub cursor_pos: usize,
    pub error_message: Option<String>,
}

impl PickerState {
    /// Create a new branch picker from a list of branches.
    pub fn new_branch(branches: &[BranchInfo]) -> Self {
        let has_remote = branches.iter().any(|b| b.is_remote);
        let mut items = Vec::new();

        for b in branches {
            // Insert separator before first remote branch
            if b.is_remote && !items.iter().any(|i: &PickerItem| i.is_separator) && has_remote {
                items.push(PickerItem {
                    label: "Remote".to_string(),
                    is_current: false,
                    is_separator: true,
                    is_remote: true,
                    data: String::new(),
                });
            }
            items.push(PickerItem {
                label: b.name.clone(),
                is_current: b.is_current,
                is_separator: false,
                is_remote: b.is_remote,
                data: b.name.clone(),
            });
        }

        let filtered_indices: Vec<usize> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| !item.is_separator)
            .map(|(i, _)| i)
            .collect();

        // Default-select the current branch so Enter without navigation is a no-op
        let selected = filtered_indices
            .iter()
            .position(|&i| items[i].is_current)
            .unwrap_or(0);

        Self {
            kind: PickerKind::Branch,
            all_items: items,
            filtered_indices,
            selected,
            scroll_offset: 0,
            query: String::new(),
            cursor_pos: 0,
            error_message: None,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.query.insert(self.cursor_pos, ch);
        self.cursor_pos += ch.len_utf8();
        self.update_filter();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            let prev = self.query[..self.cursor_pos]
                .chars()
                .last()
                .map_or(0, char::len_utf8);
            self.cursor_pos -= prev;
            self.query.remove(self.cursor_pos);
            self.update_filter();
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered_indices.is_empty() && self.selected < self.filtered_indices.len() - 1 {
            self.selected += 1;
        }
    }

    /// Returns the currently highlighted item, if any.
    pub fn selected_item(&self) -> Option<&PickerItem> {
        self.filtered_indices
            .get(self.selected)
            .and_then(|&idx| self.all_items.get(idx))
    }

    fn update_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered_indices = self
                .all_items
                .iter()
                .enumerate()
                .filter(|(_, item)| !item.is_separator)
                .map(|(i, _)| i)
                .collect();
        } else {
            self.filtered_indices = self
                .all_items
                .iter()
                .enumerate()
                .filter(|(_, item)| !item.is_separator && fuzzy_match(&self.query, &item.label))
                .map(|(i, _)| i)
                .collect();
        }
        self.selected = 0;
        self.scroll_offset = 0;
    }
}

/// Widget for rendering the picker overlay.
pub struct PickerWidget<'a> {
    pub state: &'a PickerState,
}

impl Widget for PickerWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Need enough space for border + input + at least hint row
        if area.width < 8 || area.height < 5 {
            return;
        }

        let dialog_width = 44u16.min(area.width.saturating_sub(4));
        let max_visible_items = 10u16;
        let has_error = self.state.error_message.is_some();
        // 1 border top + 1 input + 1 blank + items + 1 hint + (1 error?) + 1 border bottom
        let item_count = self.state.filtered_indices.len() as u16;
        let visible_items = item_count.min(max_visible_items);
        // Also account for separator rows that appear in filtered view
        let separator_count = self.separator_count_in_view(visible_items);
        let content_rows = visible_items + separator_count;
        let dialog_height =
            (3 + content_rows + 1 + u16::from(has_error) + 1).min(area.height.saturating_sub(2));

        let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
        let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
        let dialog_rect = Rect::new(x, y, dialog_width, dialog_height);

        let base = colors::popup_base();
        let title_style = base.add_modifier(Modifier::BOLD);

        // Fill background
        for dy in 0..dialog_rect.height {
            for dx in 0..dialog_rect.width {
                if let Some(cell) = buf.cell_mut((dialog_rect.x + dx, dialog_rect.y + dy)) {
                    cell.set_style(base);
                    cell.set_symbol(" ");
                }
            }
        }

        draw_border(buf, dialog_rect, base);

        // Title
        let title = match self.state.kind {
            PickerKind::Branch => "Switch Branch",
        };
        let title_x =
            dialog_rect.x + (dialog_rect.width.saturating_sub(title.len() as u16 + 2)) / 2;
        buf.set_string(title_x, dialog_rect.y, format!(" {title} "), title_style);

        // Input field
        let input_y = dialog_rect.y + 1;
        let input_x = dialog_rect.x + 2;
        let input_width = dialog_rect.width.saturating_sub(4) as usize;

        // Clear input area of REVERSED
        for dx in 0..input_width {
            if let Some(cell) = buf.cell_mut((input_x + dx as u16, input_y)) {
                cell.reset();
            }
        }

        // Draw prompt
        buf.set_string(
            input_x,
            input_y,
            "> ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

        // Draw query
        let query_x = input_x + 2;
        let query_width = input_width.saturating_sub(2);
        let display_text = if self.state.query.len() > query_width {
            &self.state.query[self.state.query.len() - query_width..]
        } else {
            &self.state.query
        };
        buf.set_string(query_x, input_y, display_text, Style::default());

        // Draw cursor
        let cursor_display_pos = if self.state.query.len() > query_width {
            query_width
        } else {
            self.state.cursor_pos
        };
        if let Some(cell) = buf.cell_mut((query_x + cursor_display_pos as u16, input_y)) {
            cell.set_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED));
        }

        // Item list
        let list_y = dialog_rect.y + 3;
        let list_height = dialog_rect
            .height
            .saturating_sub(3 + 1 + u16::from(has_error) + 1);
        let inner_width = dialog_rect.width.saturating_sub(4) as usize;

        if list_height == 0 || inner_width == 0 {
            return;
        }

        // Build display rows: interleave separators and filtered items
        let display_rows = self.build_display_rows();
        let total_display = display_rows.len();

        // Adjust scroll to keep selected visible
        let selected_display_row = self.selected_display_row(&display_rows);
        let scroll = {
            let mut s = self.state.scroll_offset;
            if selected_display_row >= s + list_height as usize {
                s = selected_display_row.saturating_sub((list_height as usize).saturating_sub(1));
            }
            if selected_display_row < s {
                s = selected_display_row;
            }
            s
        };

        for row in 0..list_height as usize {
            let display_idx = scroll + row;
            if display_idx >= total_display {
                break;
            }
            let dy = list_y + row as u16;
            match &display_rows[display_idx] {
                DisplayRow::Separator(label) => {
                    let sep_text = format!("─ {label} ");
                    let sep_display_width = UnicodeWidthStr::width(sep_text.as_str());
                    let remaining = inner_width.saturating_sub(sep_display_width);
                    let full = format!("{sep_text}{}", "─".repeat(remaining));
                    let truncated = truncate_to_display_width(&full, inner_width);
                    buf.set_string(dialog_rect.x + 2, dy, &truncated, colors::popup_dim());
                }
                DisplayRow::Item {
                    filtered_idx,
                    item_idx,
                } => {
                    let item = &self.state.all_items[*item_idx];
                    let is_selected = *filtered_idx == self.state.selected;

                    let prefix = if item.is_current { "✓ " } else { "  " };
                    let label = format!("{prefix}{}", item.label);
                    let display = truncate_to_display_width(&label, inner_width);

                    let style = if is_selected {
                        colors::popup_selected()
                    } else {
                        base
                    };

                    // Fill the row with the style
                    for dx in 0..inner_width {
                        if let Some(cell) = buf.cell_mut((dialog_rect.x + 2 + dx as u16, dy)) {
                            cell.set_style(style);
                            cell.set_symbol(" ");
                        }
                    }
                    buf.set_string(dialog_rect.x + 2, dy, &display, style);
                }
            }
        }

        // Error message
        if let Some(ref err) = self.state.error_message {
            let err_y = dialog_rect.y + dialog_rect.height - 2;
            let err_style = Style::reset().fg(Color::Red).add_modifier(Modifier::BOLD);
            let truncated = truncate_to_display_width(err, inner_width);
            buf.set_string(dialog_rect.x + 2, err_y, &truncated, err_style);
        }

        // Hint
        let hint_y = dialog_rect.y + dialog_rect.height - 1;
        let hint = "[Enter] switch  [Esc] cancel";
        let hint_x = dialog_rect.x + (dialog_rect.width.saturating_sub(hint.len() as u16)) / 2;
        buf.set_string(hint_x, hint_y, hint, colors::popup_dim());
    }
}

/// Internal helper types for rendering.
enum DisplayRow {
    Separator(String),
    Item {
        filtered_idx: usize,
        item_idx: usize,
    },
}

impl PickerWidget<'_> {
    /// Count separator rows that would appear in the visible window.
    fn separator_count_in_view(&self, _visible_items: u16) -> u16 {
        // Check if any filtered item is remote and any is local
        let has_local = self
            .state
            .filtered_indices
            .iter()
            .any(|&i| !self.state.all_items[i].is_remote);
        let has_remote = self
            .state
            .filtered_indices
            .iter()
            .any(|&i| self.state.all_items[i].is_remote);
        u16::from(has_local && has_remote)
    }

    /// Build display rows interleaving separators with filtered items.
    fn build_display_rows(&self) -> Vec<DisplayRow> {
        let mut rows = Vec::new();
        let mut seen_remote = false;

        for (filtered_idx, &item_idx) in self.state.filtered_indices.iter().enumerate() {
            let item = &self.state.all_items[item_idx];
            if item.is_remote && !seen_remote {
                // Check if there were any local items before this
                let has_local_before = rows.iter().any(|r| matches!(r, DisplayRow::Item { .. }));
                if has_local_before {
                    rows.push(DisplayRow::Separator("Remote".to_string()));
                }
                seen_remote = true;
            }
            rows.push(DisplayRow::Item {
                filtered_idx,
                item_idx,
            });
        }
        rows
    }

    /// Find the display row index for the currently selected filtered item.
    fn selected_display_row(&self, display_rows: &[DisplayRow]) -> usize {
        for (i, row) in display_rows.iter().enumerate() {
            if let DisplayRow::Item { filtered_idx, .. } = row {
                if *filtered_idx == self.state.selected {
                    return i;
                }
            }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_branches() -> Vec<BranchInfo> {
        vec![
            BranchInfo {
                name: "main".to_string(),
                is_remote: false,
                is_current: true,
            },
            BranchInfo {
                name: "feature/auth".to_string(),
                is_remote: false,
                is_current: false,
            },
            BranchInfo {
                name: "origin/main".to_string(),
                is_remote: true,
                is_current: false,
            },
        ]
    }

    #[test]
    fn new_branch_creates_items_with_separator() {
        let state = PickerState::new_branch(&make_branches());
        assert!(state.all_items.iter().any(|i| i.is_separator));
        assert_eq!(state.all_items.len(), 4); // 2 local + 1 separator + 1 remote
    }

    #[test]
    fn filter_narrows_results() {
        let mut state = PickerState::new_branch(&make_branches());
        state.insert_char('m');
        state.insert_char('a');
        // "ma" should match "main" and "origin/main"
        assert!(state.filtered_indices.len() >= 2);
    }

    #[test]
    fn selected_item_returns_correct_branch() {
        let state = PickerState::new_branch(&make_branches());
        let item = state.selected_item().unwrap();
        assert_eq!(item.label, "main");
        assert!(item.is_current);
    }

    #[test]
    fn move_down_and_up() {
        let mut state = PickerState::new_branch(&make_branches());
        assert_eq!(state.selected, 0);
        state.move_down();
        assert_eq!(state.selected, 1);
        state.move_up();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn selected_defaults_to_current_branch_not_first() {
        // Current branch is NOT alphabetically first
        let branches = vec![
            BranchInfo {
                name: "alpha".to_string(),
                is_remote: false,
                is_current: false,
            },
            BranchInfo {
                name: "beta".to_string(),
                is_remote: false,
                is_current: false,
            },
            BranchInfo {
                name: "main".to_string(),
                is_remote: false,
                is_current: true,
            },
        ];
        let state = PickerState::new_branch(&branches);
        let item = state.selected_item().unwrap();
        assert_eq!(item.label, "main");
        assert!(item.is_current);
        assert_eq!(state.selected, 2);
    }

    #[test]
    fn delete_char_updates_filter() {
        let mut state = PickerState::new_branch(&make_branches());
        state.insert_char('x');
        state.insert_char('y');
        state.insert_char('z');
        assert!(state.filtered_indices.is_empty());
        state.delete_char();
        state.delete_char();
        state.delete_char();
        // Back to empty query — all non-separator items shown
        assert_eq!(state.filtered_indices.len(), 3);
    }

    fn render_picker(state: &PickerState, width: u16, height: u16) -> Buffer {
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        let widget = PickerWidget { state };
        widget.render(area, &mut buf);
        buf
    }

    #[test]
    fn render_with_unicode_separator_no_panic() {
        // The separator uses "─" (3-byte UTF-8 char). Previously this panicked
        // because byte-length was used instead of display width for truncation.
        let state = PickerState::new_branch(&make_branches());
        // Width 44 triggered the original panic (byte index 40 inside "─")
        render_picker(&state, 48, 20);
    }

    #[test]
    fn render_with_current_branch_checkmark_no_panic() {
        // "✓" is multi-byte; ensure label truncation doesn't slice mid-char
        let branches = vec![BranchInfo {
            name: "a-very-long-branch-name-that-will-be-truncated".to_string(),
            is_remote: false,
            is_current: true,
        }];
        let state = PickerState::new_branch(&branches);
        render_picker(&state, 30, 15);
    }

    #[test]
    fn render_with_error_message_no_panic() {
        let mut state = PickerState::new_branch(&make_branches());
        state.error_message = Some("Something went wrong with émojis 🎉 and ñ".to_string());
        render_picker(&state, 40, 20);
    }

    #[test]
    fn render_tiny_terminal_no_panic() {
        // Extremely small terminal: list_height would be 0, dialog could be
        // smaller than border. Must not panic. Start from 1x1 since ratatui
        // Buffer itself panics with 0-dimension areas.
        let state = PickerState::new_branch(&make_branches());
        for w in 1..=10 {
            for h in 1..=10 {
                render_picker(&state, w, h);
            }
        }
    }
}
