use ratatui::style::Color;

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
pub const SELECTED_BG: Color = Color::DarkGray;
pub const HOVER_BG: Color = Color::Indexed(240);
pub const TREE_LINE: Color = Color::DarkGray;
pub const STATUS_BAR_BG: Color = Color::DarkGray;
pub const STATUS_BAR_FG: Color = Color::White;
pub const DIR_COLOR: Color = Color::Yellow;
pub const MENU_BORDER: Color = Color::Gray;
pub const MENU_SELECTED_BG: Color = Color::Indexed(244);
pub const DEFAULT_FG: Color = Color::Reset;

/// Whether the terminal is currently using a light colour scheme.
/// Uses macOS `defaults` to check `AppleInterfaceStyle`; defaults to dark.
pub fn is_light() -> bool {
    !std::process::Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "Dark")
        .unwrap_or(true)
}
