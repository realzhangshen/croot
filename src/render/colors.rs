use ratatui::style::{Color, Modifier, Style};

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
pub const MULTI_SELECTED_BG: Color = Color::DarkGray;
pub const TREE_LINE: Color = Color::DarkGray;
pub const STATUS_BAR_BG: Color = Color::DarkGray;
pub const STATUS_BAR_FG: Color = Color::White;
pub const DIR_COLOR: Color = Color::Yellow;
pub const DEFAULT_FG: Color = Color::Reset;

// ── Adaptive style helpers (REVERSED-based, no hardcoded bg) ──────────

/// Tree-view hover row: subtle reverse + dim
pub fn hover_style() -> Style {
    Style::default().add_modifier(Modifier::REVERSED | Modifier::DIM)
}

/// Popup / menu base: reversed foreground ↔ background
pub fn popup_base() -> Style {
    Style::default().add_modifier(Modifier::REVERSED)
}

/// Popup selected item: bold (on top of the reversed base fill)
pub fn popup_selected() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

/// Popup dim text (separators, hints): reversed + dim
pub fn popup_dim() -> Style {
    Style::default().add_modifier(Modifier::REVERSED | Modifier::DIM)
}
