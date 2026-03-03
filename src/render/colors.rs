use ratatui::style::Color;

// VS Code exact true colors for git status
pub const GIT_MODIFIED: Color = Color::Rgb(0xE2, 0xC0, 0x6A);   // #E2C06A yellow
pub const GIT_ADDED: Color = Color::Rgb(0x73, 0xC9, 0x90);      // #73C990 green
pub const GIT_DELETED: Color = Color::Rgb(0xE8, 0x59, 0x50);    // #E85950 red
pub const GIT_IGNORED: Color = Color::Rgb(0x80, 0x80, 0x80);    // #808080 gray
pub const GIT_UNTRACKED: Color = Color::Rgb(0x73, 0xC9, 0x90);  // #73C990 green (same as added)
pub const GIT_CONFLICTED: Color = Color::Rgb(0xE8, 0x59, 0x50); // #E85950 red

// UI colors
pub const SELECTED_BG: Color = Color::Rgb(0x26, 0x4F, 0x78);    // #264F78 VS Code selection blue
pub const TREE_LINE: Color = Color::Rgb(0x58, 0x58, 0x58);       // #585858 dim gray for connectors
pub const STATUS_BAR_BG: Color = Color::Rgb(0x00, 0x7A, 0xCC);  // #007ACC VS Code blue
pub const STATUS_BAR_FG: Color = Color::Rgb(0xFF, 0xFF, 0xFF);
pub const DIR_COLOR: Color = Color::Rgb(0xDC, 0xDC, 0xAA);       // #DCDCAA warm yellow for dirs
pub const DEFAULT_FG: Color = Color::Rgb(0xCC, 0xCC, 0xCC);      // #CCCCCC light gray
