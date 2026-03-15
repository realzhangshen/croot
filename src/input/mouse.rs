use std::time::Instant;

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::handler::Action;

/// Tracks consecutive clicks on the same row to detect double-clicks.
pub struct ClickTracker {
    last_click_time: Option<Instant>,
    last_click_row: Option<u16>,
}

impl Default for ClickTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ClickTracker {
    pub fn new() -> Self {
        Self {
            last_click_time: None,
            last_click_row: None,
        }
    }

    /// Record a click on `row`. Returns `true` if this is a double-click
    /// (same row within 300ms).
    fn record_click(&mut self, row: u16) -> bool {
        let now = Instant::now();
        let is_double = match (self.last_click_time, self.last_click_row) {
            (Some(t), Some(r)) => r == row && now.duration_since(t).as_millis() < 300,
            _ => false,
        };
        if is_double {
            // Reset to prevent triple-click
            self.last_click_time = None;
            self.last_click_row = None;
        } else {
            self.last_click_time = Some(now);
            self.last_click_row = Some(row);
        }
        is_double
    }
}

/// Map a mouse event to an Action given the tree area's position.
/// `preview_x` is the x-coordinate where the preview pane starts (None if no preview visible).
pub fn handle_mouse(
    event: MouseEvent,
    tree_area_y: u16,
    tree_area_height: u16,
    preview_x: Option<u16>,
    click_tracker: &mut ClickTracker,
) -> Action {
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Check for separator hit (2-column zone around the separator)
            if let Some(px) = preview_x {
                let sep_x = px.saturating_sub(1);
                if event.column >= sep_x.saturating_sub(1) && event.column <= sep_x {
                    return Action::SeparatorDragStart;
                }
            }
            if preview_x.is_some_and(|px| event.column >= px) {
                return Action::SelectionStart(event.column, event.row);
            }
            let row = event.row;
            if row >= tree_area_y && row < tree_area_y + tree_area_height {
                let relative_row = row - tree_area_y;
                let mods = event.modifiers;
                if mods.contains(KeyModifiers::SUPER) || mods.contains(KeyModifiers::CONTROL) {
                    Action::ToggleSelectRow(relative_row)
                } else if mods.contains(KeyModifiers::SHIFT) {
                    Action::RangeSelectRow(relative_row)
                } else if click_tracker.record_click(relative_row) {
                    Action::DoubleClick(relative_row)
                } else {
                    Action::ClickRow(relative_row)
                }
            } else {
                Action::None
            }
        }
        MouseEventKind::Down(MouseButton::Right) => Action::RightClick(event.column, event.row),
        MouseEventKind::Drag(MouseButton::Left) => Action::DragUpdate(event.column, event.row),
        MouseEventKind::ScrollUp => {
            if preview_x.is_some_and(|px| event.column >= px) {
                Action::PreviewScrollUp(3)
            } else {
                Action::ScrollUp(3)
            }
        }
        MouseEventKind::ScrollDown => {
            if preview_x.is_some_and(|px| event.column >= px) {
                Action::PreviewScrollDown(3)
            } else {
                Action::ScrollDown(3)
            }
        }
        MouseEventKind::Moved => Action::Hover(event.column, event.row),
        _ => Action::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_left_click(col: u16, row: u16, modifiers: KeyModifiers) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: col,
            row,
            modifiers,
        }
    }

    #[test]
    fn cmd_click_returns_toggle_select() {
        let mut tracker = ClickTracker::new();
        let event = make_left_click(5, 2, KeyModifiers::SUPER);
        let action = handle_mouse(event, 0, 10, None, &mut tracker);
        assert_eq!(action, Action::ToggleSelectRow(2));
    }

    #[test]
    fn ctrl_click_returns_toggle_select() {
        let mut tracker = ClickTracker::new();
        let event = make_left_click(5, 3, KeyModifiers::CONTROL);
        let action = handle_mouse(event, 0, 10, None, &mut tracker);
        assert_eq!(action, Action::ToggleSelectRow(3));
    }

    #[test]
    fn shift_click_returns_range_select() {
        let mut tracker = ClickTracker::new();
        let event = make_left_click(5, 4, KeyModifiers::SHIFT);
        let action = handle_mouse(event, 0, 10, None, &mut tracker);
        assert_eq!(action, Action::RangeSelectRow(4));
    }

    #[test]
    fn plain_click_unchanged() {
        let mut tracker = ClickTracker::new();
        let event = make_left_click(5, 1, KeyModifiers::NONE);
        let action = handle_mouse(event, 0, 10, None, &mut tracker);
        assert_eq!(action, Action::ClickRow(1));
    }
}
