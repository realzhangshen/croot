use ratatui::style::Color;

// VS Code exact true colors for git status (unstaged / working tree)
pub const GIT_MODIFIED: Color = Color::Rgb(0xE2, 0xC0, 0x6A); // #E2C06A yellow/orange
pub const GIT_ADDED: Color = Color::Rgb(0x73, 0xC9, 0x90); // #73C990 green
pub const GIT_DELETED: Color = Color::Rgb(0xE8, 0x59, 0x50); // #E85950 red
pub const GIT_IGNORED: Color = Color::Rgb(0x80, 0x80, 0x80); // #808080 gray
pub const GIT_CONFLICTED: Color = Color::Rgb(0xE8, 0x59, 0x50); // #E85950 red

// Staged (index) variants — slightly muted/darker shades
pub const GIT_STAGED_MODIFIED: Color = Color::Rgb(0xB8, 0x9A, 0x50); // #B89A50 darker yellow
pub const GIT_STAGED_ADDED: Color = Color::Rgb(0x5A, 0xA0, 0x72); // #5AA072 darker green
pub const GIT_STAGED_DELETED: Color = Color::Rgb(0xB8, 0x48, 0x40); // #B84840 darker red

// UI colors — ANSI / terminal-default so they adapt to any theme
pub const SELECTED_BG: Color = Color::DarkGray;
pub const TREE_LINE: Color = Color::DarkGray;
pub const STATUS_BAR_BG: Color = Color::DarkGray;
pub const STATUS_BAR_FG: Color = Color::Reset;
pub const DIR_COLOR: Color = Color::Yellow;
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
