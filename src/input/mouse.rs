use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use super::handler::Action;

/// Map a mouse event to an Action given the tree area's position.
pub fn handle_mouse(event: MouseEvent, tree_area_y: u16, tree_area_height: u16) -> Action {
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let row = event.row;
            if row >= tree_area_y && row < tree_area_y + tree_area_height {
                let relative_row = row - tree_area_y;
                Action::ClickRow(relative_row)
            } else {
                Action::None
            }
        }
        MouseEventKind::ScrollUp => Action::ScrollUp(3),
        MouseEventKind::ScrollDown => Action::ScrollDown(3),
        _ => Action::None,
    }
}
