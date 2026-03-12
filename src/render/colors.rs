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

// ── Popup overlay colors — fixed 256-color for guaranteed contrast ────
// Deliberately *not* theme-adaptive: popups need legible text regardless
// of whether the terminal palette is light or dark.
pub const POPUP_FG: Color = Color::Indexed(15); // #ffffff  bright white text
pub const POPUP_BG: Color = Color::Indexed(238); // #444444  dark-gray background
pub const POPUP_ACCENT: Color = Color::Indexed(12); // #5f87ff  selection highlight
pub const POPUP_BORDER_FG: Color = Color::Indexed(246); // #949494  visible border gray
pub const POPUP_DIM_FG: Color = Color::Indexed(249); // #b2b2b2  secondary text
pub const POPUP_INPUT_BG: Color = Color::Indexed(235); // #262626  sunken input field

// ── Style helpers ────────────────────────────────────────────────────

/// Tree-view hover row: subtle reverse + dim
pub fn hover_style() -> Style {
    Style::default().add_modifier(Modifier::REVERSED | Modifier::DIM)
}

/// Popup / menu base: bright white text on dark gray background.
pub fn popup_base() -> Style {
    Style::default().fg(POPUP_FG).bg(POPUP_BG)
}

/// Popup selected item: bright blue background with bright white text
pub fn popup_selected() -> Style {
    Style::reset()
        .bg(POPUP_ACCENT)
        .fg(POPUP_FG)
        .add_modifier(Modifier::BOLD)
}

/// Popup selected danger item (e.g. Delete): red background with bright white text
pub fn popup_selected_danger() -> Style {
    Style::reset()
        .bg(Color::Red)
        .fg(POPUP_FG)
        .add_modifier(Modifier::BOLD)
}

/// Popup dim text (hints, separators, [Cancel]): readable secondary text.
pub fn popup_dim() -> Style {
    Style::default().fg(POPUP_DIM_FG).bg(POPUP_BG)
}

/// Popup border: visible gray on popup background.
pub fn popup_border() -> Style {
    Style::default().fg(POPUP_BORDER_FG).bg(POPUP_BG)
}

/// Popup input field: bright white text on sunken dark background.
pub fn popup_input() -> Style {
    Style::default().fg(POPUP_FG).bg(POPUP_INPUT_BG)
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
                assert_eq!(cell.fg, POPUP_FG, "fg at ({col},{row})");
                assert_eq!(cell.bg, POPUP_BG, "bg at ({col},{row})");
                assert!(
                    !cell.modifier.contains(Modifier::REVERSED),
                    "should NOT have REVERSED at ({col},{row}): {:?}",
                    cell.modifier
                );
            }
        }
    }
}
