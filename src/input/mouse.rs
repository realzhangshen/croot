use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use super::handler::Action;

/// Map a mouse event to an Action given the tree area's position.
/// `preview_x` is the x-coordinate where the preview pane starts (None if no preview visible).
pub fn handle_mouse(
    event: MouseEvent,
    tree_area_y: u16,
    tree_area_height: u16,
    preview_x: Option<u16>,
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
                Action::ClickRow(relative_row)
            } else {
                Action::None
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            Action::DragUpdate(event.column, event.row)
        }
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
        _ => Action::None,
    }
}
