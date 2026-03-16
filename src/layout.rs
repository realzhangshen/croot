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

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> PreviewLayout {
        PreviewLayout {
            x: 10,
            y: 5,
            height: 20,
        }
    }

    #[test]
    fn within_layout_returns_content_pos() {
        let pos = screen_to_content(layout(), 0, 15, 10).unwrap();
        assert_eq!(pos, ContentPos { line: 5, col: 5 });
    }

    #[test]
    fn scroll_offset_added_to_line() {
        let pos = screen_to_content(layout(), 100, 10, 5).unwrap();
        assert_eq!(pos, ContentPos { line: 100, col: 0 });
    }

    #[test]
    fn above_layout_returns_none() {
        assert!(screen_to_content(layout(), 0, 15, 4).is_none());
    }

    #[test]
    fn left_of_layout_returns_none() {
        assert!(screen_to_content(layout(), 0, 9, 10).is_none());
    }

    #[test]
    fn at_bottom_boundary_returns_none() {
        // y=5, height=20 → valid rows are 5..25, so row 25 is out
        assert!(screen_to_content(layout(), 0, 10, 25).is_none());
    }

    #[test]
    fn last_valid_row() {
        let pos = screen_to_content(layout(), 0, 10, 24).unwrap();
        assert_eq!(pos, ContentPos { line: 19, col: 0 });
    }

    #[test]
    fn top_left_corner() {
        let pos = screen_to_content(layout(), 0, 10, 5).unwrap();
        assert_eq!(pos, ContentPos { line: 0, col: 0 });
    }
}
