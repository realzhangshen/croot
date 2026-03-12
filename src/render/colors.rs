use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

// Git status colors — ANSI 16 so they adapt to any terminal theme
pub const GIT_MODIFIED: Color = Color::Yellow;
pub const GIT_ADDED: Color = Color::Green;
pub const GIT_DELETED: Color = Color::Red;
pub const GIT_IGNORED: Color = Color::DarkGray;
pub const GIT_CONFLICTED: Color = Color::Red;

// Staged variants — same ANSI color, distinguished via DIM modifier in tree_view
pub const GIT_STAGED_MODIFIED: Color = Color::Yellow;
pub const GIT_STAGED_ADDED: Color = Color::Green;
pub const GIT_STAGED_DELETED: Color = Color::Red;

// Preview / UI accent colors
pub const UNFOCUSED_HEADER_BG: Color = Color::DarkGray;
pub const UNFOCUSED_HEADER_FG: Color = Color::Gray;
pub const HEX_VALUES: Color = Color::LightBlue;
pub const HEX_ASCII: Color = Color::Gray;
pub const PREVIEW_DIR_NAME: Color = Color::LightYellow;
pub const INLINE_CODE: Color = Color::Yellow;

// UI colors — ANSI / terminal-default so they adapt to any theme
// Cursor row uses Modifier::REVERSED (no explicit bg) for maximum contrast
pub const TREE_LINE: Color = Color::DarkGray;
pub const STATUS_BAR_BG: Color = Color::DarkGray;
pub const STATUS_BAR_FG: Color = Color::White;
pub const DIR_COLOR: Color = Color::Yellow;
pub const DEFAULT_FG: Color = Color::Reset;
pub const FIND_MATCH: Color = Color::Cyan;

// ── Adaptive style helpers (REVERSED-based, no hardcoded bg) ──────────

/// Tree-view hover row: subtle reverse + dim
pub fn hover_style() -> Style {
    Style::default().add_modifier(Modifier::REVERSED | Modifier::DIM)
}

/// Popup / menu base: explicit White/Black + REVERSED → black text on white bg.
/// Explicit colors force the backend to always emit SetColors, preventing
/// color bleed when the terminal retains stale state from prior frames.
pub fn popup_base() -> Style {
    Style::default()
        .fg(Color::White)
        .bg(Color::Black)
        .add_modifier(Modifier::REVERSED)
}

/// Popup selected item: blue background with bold white text
pub fn popup_selected() -> Style {
    Style::reset()
        .bg(Color::Blue)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

/// Popup selected danger item (e.g. Delete): red background with bold white text
pub fn popup_selected_danger() -> Style {
    Style::reset()
        .bg(Color::Red)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

/// Popup dim text (separators, hints): explicit White/Black + REVERSED + DIM.
/// Same explicit-color rationale as `popup_base()`.
pub fn popup_dim() -> Style {
    Style::default()
        .fg(Color::White)
        .bg(Color::Black)
        .add_modifier(Modifier::REVERSED | Modifier::DIM)
}

/// Clear a rectangular region and apply a fresh style.
/// Resets each cell first to prevent color bleed from underlying content.
pub fn clear_region(buf: &mut Buffer, rect: Rect, style: Style) {
    for row in rect.y..rect.y + rect.height {
        for col in rect.x..rect.x + rect.width {
            if let Some(cell) = buf.cell_mut((col, row)) {
                cell.reset();
                cell.set_style(style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_region_wipes_preexisting_state() {
        let area = Rect::new(0, 0, 4, 3);
        let mut buf = Buffer::empty(area);

        // Pre-fill with colored content
        for row in 0..area.height {
            for col in 0..area.width {
                if let Some(cell) = buf.cell_mut((col, row)) {
                    cell.set_symbol("X");
                    cell.set_style(Style::default().fg(Color::Red).bg(Color::Green));
                }
            }
        }

        // Apply clear_region with popup_base()
        clear_region(&mut buf, area, popup_base());

        // Every cell should be fully reset + popup_base applied
        for row in 0..area.height {
            for col in 0..area.width {
                let cell = buf.cell((col, row)).unwrap();
                assert_eq!(cell.symbol(), " ", "symbol at ({col},{row})");
                assert_eq!(cell.fg, Color::White, "fg at ({col},{row})");
                assert_eq!(cell.bg, Color::Black, "bg at ({col},{row})");
                assert!(
                    cell.modifier.contains(Modifier::REVERSED),
                    "modifier at ({col},{row}): {:?}",
                    cell.modifier
                );
            }
        }
    }
}
