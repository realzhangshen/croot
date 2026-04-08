//! Shared rendering helpers for overlay popups (dialogs, context menus,
//! pickers, overlays).
//!
//! Consolidates border drawing so every popup uses the same rounded corners
//! and avoids the hand-rolled loops that used to live in each renderer.

use ratatui::{buffer::Buffer, layout::Rect, style::Style};

/// Draw a rounded box border (`╭╮╰╯ ─ │`) around `rect` using `style`.
///
/// Cells outside the buffer are ignored. A rect with `width < 2` or
/// `height < 2` is a no-op.
pub(crate) fn draw_border(buf: &mut Buffer, rect: Rect, style: Style) {
    if rect.width < 2 || rect.height < 2 {
        return;
    }

    let right = rect.x + rect.width - 1;
    let bottom = rect.y + rect.height - 1;

    // Corners
    for (x, y, sym) in [
        (rect.x, rect.y, "╭"),
        (right, rect.y, "╮"),
        (rect.x, bottom, "╰"),
        (right, bottom, "╯"),
    ] {
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_symbol(sym);
            cell.set_style(style);
        }
    }

    // Horizontal edges
    for x in (rect.x + 1)..right {
        for y in [rect.y, bottom] {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_symbol("─");
                cell.set_style(style);
            }
        }
    }

    // Vertical edges
    for y in (rect.y + 1)..bottom {
        for x in [rect.x, right] {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_symbol("│");
                cell.set_style(style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    fn corners(buf: &Buffer, rect: Rect) -> (String, String, String, String) {
        let right = rect.x + rect.width - 1;
        let bottom = rect.y + rect.height - 1;
        (
            buf.cell((rect.x, rect.y)).unwrap().symbol().to_string(),
            buf.cell((right, rect.y)).unwrap().symbol().to_string(),
            buf.cell((rect.x, bottom)).unwrap().symbol().to_string(),
            buf.cell((right, bottom)).unwrap().symbol().to_string(),
        )
    }

    #[test]
    fn draw_border_sets_rounded_corners() {
        let area = Rect::new(0, 0, 10, 5);
        let rect = Rect::new(1, 1, 6, 3);
        let mut buf = Buffer::empty(area);
        draw_border(&mut buf, rect, Style::default());
        assert_eq!(
            corners(&buf, rect),
            ("╭".into(), "╮".into(), "╰".into(), "╯".into())
        );
    }

    #[test]
    fn draw_border_fills_edges() {
        let area = Rect::new(0, 0, 10, 5);
        let rect = Rect::new(1, 1, 5, 3);
        let mut buf = Buffer::empty(area);
        draw_border(&mut buf, rect, Style::default());
        // Top edge between corners
        assert_eq!(buf.cell((2, 1)).unwrap().symbol(), "─");
        // Bottom edge between corners
        assert_eq!(buf.cell((2, 3)).unwrap().symbol(), "─");
        // Left and right verticals
        assert_eq!(buf.cell((1, 2)).unwrap().symbol(), "│");
        assert_eq!(buf.cell((5, 2)).unwrap().symbol(), "│");
    }

    #[test]
    fn draw_border_tiny_rect_is_noop() {
        let area = Rect::new(0, 0, 5, 5);
        let mut buf = Buffer::empty(area);
        // Should not panic
        draw_border(&mut buf, Rect::new(0, 0, 1, 5), Style::default());
        draw_border(&mut buf, Rect::new(0, 0, 5, 1), Style::default());
        draw_border(&mut buf, Rect::new(0, 0, 0, 0), Style::default());
    }
}
