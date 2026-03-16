use std::time::Instant;

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

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
                if event.column >= sep_x && event.column <= sep_x + 1 {
                    return Action::SeparatorDragStart;
                }
            }
            if preview_x.is_some_and(|px| event.column >= px) {
                return Action::SelectionStart(event.column, event.row);
            }
            let row = event.row;
            if row >= tree_area_y && row < tree_area_y + tree_area_height {
                let relative_row = row - tree_area_y;
                if click_tracker.record_click(relative_row) {
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
        MouseEventKind::Up(MouseButton::Left) => Action::DragEnd,
        _ => Action::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_left_click(col: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::NONE,
        }
    }

    #[test]
    fn plain_click_returns_click_row() {
        let mut tracker = ClickTracker::new();
        let event = make_left_click(5, 1);
        let action = handle_mouse(event, 0, 10, None, &mut tracker);
        assert_eq!(action, Action::ClickRow(1));
    }

    #[test]
    fn double_click_detected() {
        let mut tracker = ClickTracker::new();
        // First click
        let event = make_left_click(5, 2);
        let action = handle_mouse(event, 0, 10, None, &mut tracker);
        assert_eq!(action, Action::ClickRow(2));
        // Second click on same row immediately → double-click
        let event2 = make_left_click(5, 2);
        let action2 = handle_mouse(event2, 0, 10, None, &mut tracker);
        assert_eq!(action2, Action::DoubleClick(2));
    }

    #[test]
    fn double_click_resets_after_detection() {
        let mut tracker = ClickTracker::new();
        // First click + double click
        handle_mouse(make_left_click(5, 2), 0, 10, None, &mut tracker);
        handle_mouse(make_left_click(5, 2), 0, 10, None, &mut tracker);
        // Third click should be a single click again (not triple)
        let action = handle_mouse(make_left_click(5, 2), 0, 10, None, &mut tracker);
        assert_eq!(action, Action::ClickRow(2));
    }

    #[test]
    fn click_different_row_not_double_click() {
        let mut tracker = ClickTracker::new();
        handle_mouse(make_left_click(5, 2), 0, 10, None, &mut tracker);
        let action = handle_mouse(make_left_click(5, 3), 0, 10, None, &mut tracker);
        assert_eq!(action, Action::ClickRow(3));
    }

    #[test]
    fn click_outside_tree_area_returns_none() {
        let mut tracker = ClickTracker::new();
        // Tree starts at y=2, height=5 → valid rows 2..7
        let event = make_left_click(5, 0); // above tree area
        let action = handle_mouse(event, 2, 5, None, &mut tracker);
        assert_eq!(action, Action::None);
        // Below tree area
        let event2 = make_left_click(5, 8);
        let action2 = handle_mouse(event2, 2, 5, None, &mut tracker);
        assert_eq!(action2, Action::None);
    }

    #[test]
    fn click_adjusts_for_tree_area_offset() {
        let mut tracker = ClickTracker::new();
        // Tree starts at y=3
        let event = make_left_click(5, 5);
        let action = handle_mouse(event, 3, 10, None, &mut tracker);
        assert_eq!(action, Action::ClickRow(2)); // 5 - 3 = 2
    }

    #[test]
    fn right_click_returns_right_click_action() {
        let mut tracker = ClickTracker::new();
        let event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Right),
            column: 10,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        let action = handle_mouse(event, 0, 10, None, &mut tracker);
        assert_eq!(action, Action::RightClick(10, 5));
    }

    #[test]
    fn scroll_up_in_tree_area() {
        let mut tracker = ClickTracker::new();
        let event = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 5,
            row: 3,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        let action = handle_mouse(event, 0, 10, None, &mut tracker);
        assert_eq!(action, Action::ScrollUp(3));
    }

    #[test]
    fn scroll_down_in_preview_area() {
        let mut tracker = ClickTracker::new();
        let event = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 50, // in preview area
            row: 3,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        let action = handle_mouse(event, 0, 10, Some(40), &mut tracker);
        assert_eq!(action, Action::PreviewScrollDown(3));
    }

    #[test]
    fn scroll_up_in_preview_area() {
        let mut tracker = ClickTracker::new();
        let event = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 50,
            row: 3,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        let action = handle_mouse(event, 0, 10, Some(40), &mut tracker);
        assert_eq!(action, Action::PreviewScrollUp(3));
    }

    #[test]
    fn drag_returns_drag_update() {
        let mut tracker = ClickTracker::new();
        let event = MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 20,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        let action = handle_mouse(event, 0, 10, None, &mut tracker);
        assert_eq!(action, Action::DragUpdate(20, 5));
    }

    #[test]
    fn mouse_move_returns_hover() {
        let mut tracker = ClickTracker::new();
        let event = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 10,
            row: 3,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        let action = handle_mouse(event, 0, 10, None, &mut tracker);
        assert_eq!(action, Action::Hover(10, 3));
    }

    #[test]
    fn click_on_separator_starts_drag() {
        let mut tracker = ClickTracker::new();
        // Preview starts at col 40, separator at 39
        let event = make_left_click(39, 5);
        let action = handle_mouse(event, 0, 10, Some(40), &mut tracker);
        assert_eq!(action, Action::SeparatorDragStart);
    }

    #[test]
    fn click_in_preview_starts_selection() {
        let mut tracker = ClickTracker::new();
        // Click in the preview area (col >= preview_x)
        let event = make_left_click(45, 5);
        let action = handle_mouse(event, 0, 10, Some(40), &mut tracker);
        assert_eq!(action, Action::SelectionStart(45, 5));
    }

    #[test]
    fn mouse_up_returns_drag_end() {
        let mut tracker = ClickTracker::new();
        let event = MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 20,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        let action = handle_mouse(event, 0, 10, None, &mut tracker);
        assert_eq!(action, Action::DragEnd);
    }
}
