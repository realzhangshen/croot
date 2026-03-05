use crate::preview::state::ContentPos;

/// Which pane has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Tree,
    Preview,
}

/// Cached layout coordinates of the preview content area (set during draw).
#[derive(Debug, Clone, Copy)]
pub struct PreviewLayout {
    /// Screen x where content text starts (after gutter).
    pub x: u16,
    /// Screen y where content starts (after header).
    pub y: u16,
    /// Height of the content area (excluding header).
    pub height: u16,
}

/// Map screen coordinates to content-space coordinates using the preview layout.
pub fn screen_to_content(
    layout: PreviewLayout,
    scroll_offset: usize,
    screen_col: u16,
    screen_row: u16,
) -> Option<ContentPos> {
    if screen_row < layout.y || screen_row >= layout.y + layout.height || screen_col < layout.x {
        return None;
    }

    let row_in_content = (screen_row - layout.y) as usize;
    let line = scroll_offset + row_in_content;
    let col = (screen_col - layout.x) as usize;

    Some(ContentPos { line, col })
}
