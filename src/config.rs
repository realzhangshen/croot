#![allow(dead_code)] // Config schema fields are deserialized from TOML; not all consumed yet

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub tree: TreeConfig,
    #[serde(default)]
    pub preview: PreviewConfig,
    #[serde(default)]
    pub editor: EditorConfig,
    #[serde(default)]
    pub open: OpenConfig,
    #[serde(default)]
    pub mouse: MouseConfig,
    #[serde(default)]
    pub keybindings: KeybindingsConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub colors: ColorConfig,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ColorConfig {
    pub git_modified: Option<String>,
    pub git_added: Option<String>,
    pub git_deleted: Option<String>,
    pub git_ignored: Option<String>,
    pub git_conflicted: Option<String>,
    pub git_staged_modified: Option<String>,
    pub git_staged_added: Option<String>,
    pub git_staged_deleted: Option<String>,
    pub unfocused_header_bg: Option<String>,
    pub unfocused_header_fg: Option<String>,
    pub hex_values: Option<String>,
    pub hex_ascii: Option<String>,
    pub preview_dir_name: Option<String>,
    pub inline_code: Option<String>,
    pub tree_line: Option<String>,
    pub status_bar_bg: Option<String>,
    pub status_bar_fg: Option<String>,
    pub dir_color: Option<String>,
    pub default_fg: Option<String>,
    pub find_match: Option<String>,
    pub popup_fg: Option<String>,
    pub popup_bg: Option<String>,
    pub popup_accent: Option<String>,
    pub popup_border_fg: Option<String>,
    pub popup_dim_fg: Option<String>,
    pub popup_input_bg: Option<String>,
    pub popup_selected_danger_bg: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ColorDefaults {
    pub git_modified: &'static str,
    pub git_added: &'static str,
    pub git_deleted: &'static str,
    pub git_ignored: &'static str,
    pub git_conflicted: &'static str,
    pub git_staged_modified: &'static str,
    pub git_staged_added: &'static str,
    pub git_staged_deleted: &'static str,
    pub unfocused_header_bg: &'static str,
    pub unfocused_header_fg: &'static str,
    pub hex_values: &'static str,
    pub hex_ascii: &'static str,
    pub preview_dir_name: &'static str,
    pub inline_code: &'static str,
    pub tree_line: &'static str,
    pub status_bar_bg: &'static str,
    pub status_bar_fg: &'static str,
    pub dir_color: &'static str,
    pub default_fg: &'static str,
    pub find_match: &'static str,
    pub popup_fg: &'static str,
    pub popup_bg: &'static str,
    pub popup_accent: &'static str,
    pub popup_border_fg: &'static str,
    pub popup_dim_fg: &'static str,
    pub popup_input_bg: &'static str,
    pub popup_selected_danger_bg: &'static str,
}

pub const DEFAULT_COLORS: ColorDefaults = ColorDefaults {
    git_modified: "yellow",
    git_added: "green",
    git_deleted: "red",
    git_ignored: "dark_gray",
    git_conflicted: "red",
    git_staged_modified: "yellow",
    git_staged_added: "green",
    git_staged_deleted: "red",
    unfocused_header_bg: "dark_gray",
    unfocused_header_fg: "gray",
    hex_values: "light_blue",
    hex_ascii: "gray",
    preview_dir_name: "light_yellow",
    inline_code: "yellow",
    tree_line: "dark_gray",
    status_bar_bg: "dark_gray",
    status_bar_fg: "white",
    dir_color: "yellow",
    default_fg: "reset",
    find_match: "cyan",
    popup_fg: "indexed:15",
    popup_bg: "indexed:240",
    popup_accent: "indexed:12",
    popup_border_fg: "indexed:252",
    popup_dim_fg: "indexed:253",
    popup_input_bg: "indexed:236",
    popup_selected_danger_bg: "red",
};

impl ColorConfig {
    /// Return a copy with `None` fields filled in with built-in defaults.
    pub fn resolved(&self) -> Self {
        Self {
            git_modified: self
                .git_modified
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.git_modified.to_string())),
            git_added: self
                .git_added
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.git_added.to_string())),
            git_deleted: self
                .git_deleted
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.git_deleted.to_string())),
            git_ignored: self
                .git_ignored
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.git_ignored.to_string())),
            git_conflicted: self
                .git_conflicted
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.git_conflicted.to_string())),
            git_staged_modified: self
                .git_staged_modified
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.git_staged_modified.to_string())),
            git_staged_added: self
                .git_staged_added
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.git_staged_added.to_string())),
            git_staged_deleted: self
                .git_staged_deleted
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.git_staged_deleted.to_string())),
            unfocused_header_bg: self
                .unfocused_header_bg
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.unfocused_header_bg.to_string())),
            unfocused_header_fg: self
                .unfocused_header_fg
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.unfocused_header_fg.to_string())),
            hex_values: self
                .hex_values
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.hex_values.to_string())),
            hex_ascii: self
                .hex_ascii
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.hex_ascii.to_string())),
            preview_dir_name: self
                .preview_dir_name
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.preview_dir_name.to_string())),
            inline_code: self
                .inline_code
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.inline_code.to_string())),
            tree_line: self
                .tree_line
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.tree_line.to_string())),
            status_bar_bg: self
                .status_bar_bg
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.status_bar_bg.to_string())),
            status_bar_fg: self
                .status_bar_fg
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.status_bar_fg.to_string())),
            dir_color: self
                .dir_color
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.dir_color.to_string())),
            default_fg: self
                .default_fg
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.default_fg.to_string())),
            find_match: self
                .find_match
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.find_match.to_string())),
            popup_fg: self
                .popup_fg
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.popup_fg.to_string())),
            popup_bg: self
                .popup_bg
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.popup_bg.to_string())),
            popup_accent: self
                .popup_accent
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.popup_accent.to_string())),
            popup_border_fg: self
                .popup_border_fg
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.popup_border_fg.to_string())),
            popup_dim_fg: self
                .popup_dim_fg
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.popup_dim_fg.to_string())),
            popup_input_bg: self
                .popup_input_bg
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.popup_input_bg.to_string())),
            popup_selected_danger_bg: self
                .popup_selected_danger_bg
                .clone()
                .or_else(|| Some(DEFAULT_COLORS.popup_selected_danger_bg.to_string())),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MouseConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchConfig {
    #[serde(default = "default_fd_command")]
    pub fd_command: String,
    #[serde(default = "default_rg_command")]
    pub rg_command: String,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
}

fn default_fd_command() -> String {
    "fd".into()
}
fn default_rg_command() -> String {
    "rg".into()
}
fn default_max_results() -> usize {
    500
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            fd_command: default_fd_command(),
            rg_command: default_rg_command(),
            max_results: default_max_results(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct KeybindingsConfig {
    pub quit: Option<String>,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
    pub cursor_left: Option<String>,
    pub cursor_right: Option<String>,
    pub toggle: Option<String>,
    pub refresh: Option<String>,
    pub new_file: Option<String>,
    pub new_dir: Option<String>,
    pub rename: Option<String>,
    pub delete: Option<String>,
    pub toggle_preview: Option<String>,
    pub toggle_render: Option<String>,
    pub open_in_editor: Option<String>,
    pub open_externally: Option<String>,
    pub collapse_all: Option<String>,
    pub search: Option<String>,
    pub filter: Option<String>,
    pub global_search: Option<String>,
    pub global_search_content: Option<String>,
    pub goto_top: Option<String>,
    pub goto_bottom: Option<String>,
    pub branch_picker: Option<String>,
    pub enter: Option<String>,
}

/// Built-in default key for a keybinding field. Returns `None` for opt-in-only fields.
pub fn default_key_for(field: &str) -> Option<&'static str> {
    match field {
        "cursor_up" => Some("Up"),
        "cursor_down" => Some("Down"),
        "cursor_left" => Some("Left"),
        "cursor_right" => Some("Right"),
        "goto_top" => Some("Home"),
        "goto_bottom" => Some("End"),
        "search" => Some("/"),
        "filter" => Some("f"),
        "global_search" => Some("s"),
        "global_search_content" => Some("S"),
        "toggle_render" => Some("m"),
        _ => None,
    }
}

impl KeybindingsConfig {
    /// Return a copy with `None` fields filled in with built-in defaults.
    /// Fields the user explicitly set (including `""` to disable) are kept as-is.
    pub fn resolved(&self) -> Self {
        fn resolve(field: Option<&String>, name: &str) -> Option<String> {
            match field {
                Some(s) => Some(s.clone()),
                None => default_key_for(name).map(String::from),
            }
        }

        Self {
            cursor_up: resolve(self.cursor_up.as_ref(), "cursor_up"),
            cursor_down: resolve(self.cursor_down.as_ref(), "cursor_down"),
            cursor_left: resolve(self.cursor_left.as_ref(), "cursor_left"),
            cursor_right: resolve(self.cursor_right.as_ref(), "cursor_right"),
            goto_top: resolve(self.goto_top.as_ref(), "goto_top"),
            goto_bottom: resolve(self.goto_bottom.as_ref(), "goto_bottom"),
            search: resolve(self.search.as_ref(), "search"),
            filter: resolve(self.filter.as_ref(), "filter"),
            global_search: resolve(self.global_search.as_ref(), "global_search"),
            global_search_content: resolve(
                self.global_search_content.as_ref(),
                "global_search_content",
            ),
            toggle_render: resolve(self.toggle_render.as_ref(), "toggle_render"),
            // Opt-in fields: no defaults, keep as-is
            quit: self.quit.clone(),
            toggle: self.toggle.clone(),
            refresh: self.refresh.clone(),
            new_file: self.new_file.clone(),
            new_dir: self.new_dir.clone(),
            rename: self.rename.clone(),
            delete: self.delete.clone(),
            toggle_preview: self.toggle_preview.clone(),
            open_in_editor: self.open_in_editor.clone(),
            open_externally: self.open_externally.clone(),
            collapse_all: self.collapse_all.clone(),
            branch_picker: self.branch_picker.clone(),
            enter: self.enter.clone(),
        }
    }
}

/// Parse a key binding string like `"q"`, `"Enter"`, `"Ctrl+c"`, `"Shift+a"`.
pub fn parse_key(s: &str) -> Option<(KeyCode, KeyModifiers)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let parts: Vec<&str> = s.split('+').collect();
    let mut modifiers = KeyModifiers::empty();

    let key_part = if parts.len() == 1 {
        parts[0]
    } else {
        for &part in &parts[..parts.len() - 1] {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                "alt" => modifiers |= KeyModifiers::ALT,
                "super" | "cmd" | "command" => modifiers |= KeyModifiers::SUPER,
                _ => return None,
            }
        }
        parts[parts.len() - 1]
    };

    let code = match key_part.to_lowercase().as_str() {
        "enter" | "return" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "space" | " " => KeyCode::Char(' '),
        "backspace" | "bs" => KeyCode::Backspace,
        "delete" | "del" => KeyCode::Delete,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" | "pgup" => KeyCode::PageUp,
        "pagedown" | "pgdn" => KeyCode::PageDown,
        "insert" | "ins" => KeyCode::Insert,
        s if s.len() == 1 => {
            let ch = s.chars().next().unwrap();
            // Use the original case from key_part, not lowered
            let original_ch = key_part.chars().next().unwrap();
            if original_ch.is_uppercase() {
                modifiers |= KeyModifiers::SHIFT;
            }
            KeyCode::Char(original_ch.to_lowercase().next().unwrap_or(ch))
        }
        s if s.starts_with('f') => {
            let num: u8 = s[1..].parse().ok()?;
            KeyCode::F(num)
        }
        _ => return None,
    };

    Some((code, modifiers))
}

/// Parse a color string from config.
///
/// Supports ANSI names (`red`, `dark_gray`, `light-blue`, `reset`),
/// indexed colors (`indexed:240` or `240`), and RGB hex (`#ff0000`).
pub fn parse_color(s: &str) -> Option<Color> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(index) = trimmed.parse::<u8>() {
        return Some(Color::Indexed(index));
    }

    let lower = trimmed.to_ascii_lowercase();
    if let Some(index) = lower.strip_prefix("indexed:") {
        return index.trim().parse::<u8>().ok().map(Color::Indexed);
    }

    if let Some(hex) = trimmed.strip_prefix('#') {
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        return Some(Color::Rgb(r, g, b));
    }

    let normalized = lower.replace(['_', '-'], "");
    match normalized.as_str() {
        "reset" => Some(Color::Reset),
        "black" => Some(Color::Black),
        "darkgray" => Some(Color::DarkGray),
        "red" => Some(Color::Red),
        "lightred" => Some(Color::LightRed),
        "green" => Some(Color::Green),
        "lightgreen" => Some(Color::LightGreen),
        "yellow" => Some(Color::Yellow),
        "lightyellow" => Some(Color::LightYellow),
        "blue" => Some(Color::Blue),
        "lightblue" => Some(Color::LightBlue),
        "magenta" => Some(Color::Magenta),
        "lightmagenta" => Some(Color::LightMagenta),
        "cyan" => Some(Color::Cyan),
        "lightcyan" => Some(Color::LightCyan),
        "gray" | "grey" => Some(Color::Gray),
        "white" => Some(Color::White),
        _ => None,
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EditorConfig {
    #[serde(default)]
    pub command: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenConfig {
    #[serde(default = "default_open_command")]
    pub default: String,
    #[serde(default)]
    pub rules: Vec<OpenRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenRule {
    pub pattern: String,
    pub command: String,
}

fn default_open_command() -> String {
    if cfg!(target_os = "macos") {
        "open".into()
    } else {
        "xdg-open".into()
    }
}

impl Default for OpenConfig {
    fn default() -> Self {
        Self {
            default: default_open_command(),
            rules: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct TreeConfig {
    #[serde(default = "default_true")]
    pub show_hidden: bool,
    #[serde(default = "default_true")]
    pub show_ignored: bool,
    #[serde(default = "default_true")]
    pub dirs_first: bool,
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,
    #[serde(default = "default_true")]
    pub compact_folders: bool,
    #[serde(default)]
    pub show_size: bool,
    #[serde(default)]
    pub show_modified: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct PreviewConfig {
    #[serde(default)]
    pub auto_preview: bool,
    #[serde(default = "default_preview_delay")]
    pub preview_delay_ms: u64,
    #[serde(default = "default_true")]
    pub close_on_exit: bool,
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,
    #[serde(default = "default_max_file_size_kb")]
    pub max_file_size_kb: u64,
    #[serde(default = "default_true")]
    pub syntax_highlight: bool,
    #[serde(default = "default_split_ratio")]
    pub split_ratio: f32,
    #[serde(default = "default_true")]
    pub render_markdown: bool,
}

fn default_true() -> bool {
    true
}
fn default_preview_delay() -> u64 {
    150
}
fn default_split_ratio() -> f32 {
    0.5
}
fn default_max_file_size_kb() -> u64 {
    1024
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self {
            show_hidden: true,
            show_ignored: true,
            dirs_first: true,
            exclude: default_exclude(),
            compact_folders: true,
            show_size: false,
            show_modified: false,
        }
    }
}

fn default_exclude() -> Vec<String> {
    [".git", ".svn", ".hg", "CVS", ".DS_Store", "Thumbs.db"]
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            auto_preview: false,
            preview_delay_ms: 150,
            close_on_exit: true,
            show_line_numbers: true,
            max_file_size_kb: 1024,
            syntax_highlight: true,
            split_ratio: 0.5,
            render_markdown: true,
        }
    }
}

impl Config {
    /// Load config from ~/.config/croot/config.toml, or return defaults.
    pub fn load() -> Self {
        let path = config_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Serialize the resolved config to a TOML string.
    /// Keybinding defaults are filled in so the output shows effective bindings.
    pub fn to_toml_string(&self) -> String {
        let mut resolved = self.clone();
        resolved.keybindings = resolved.keybindings.resolved();
        resolved.colors = resolved.colors.resolved();
        toml::to_string_pretty(&resolved).unwrap_or_default()
    }

    /// Return a hand-written default config template with comments.
    pub fn default_toml_with_comments() -> String {
        r##"# croot configuration
# Full reference: croot config (shows all resolved values)

# ── Layer 1: Zero-config (works out of box) ──────────────
# Mouse enabled, basic keyboard shortcuts work.
# Arrow keys, /, f, s, S, m, Home/End, Esc, Ctrl+C all work.

[tree]
show_hidden = true
dirs_first = true
# show_ignored = true
# compact_folders = true
# show_size = false
# show_modified = false
# exclude = [".git", ".svn", ".hg", "CVS", ".DS_Store", "Thumbs.db"]

[preview]
auto_preview = false
# preview_delay_ms = 150
# show_line_numbers = true
# max_file_size_kb = 1024
# syntax_highlight = true
# split_ratio = 0.5
# render_markdown = true

[editor]
# command = "vim"    # Falls back to $VISUAL, $EDITOR, vi

[open]
# default = "open"   # macOS: "open", Linux: "xdg-open"
# [[open.rules]]
# pattern = "*.pdf"
# command = "zathura"

# ── Layer 2: Simple toggles ──────────────────────────────

[mouse]
# enabled = true          # Set false to disable mouse capture

[keybindings]
# Built-in defaults (override or disable with ""):
# cursor_up = "Up"        # default
# cursor_down = "Down"    # default
# cursor_left = "Left"    # default
# cursor_right = "Right"  # default
# goto_top = "Home"       # default
# goto_bottom = "End"     # default
# search = "/"            # default — find/jump to match
# filter = "f"            # default — filter tree to matches
# global_search = "s"     # default — fd file name search
# global_search_content = "S"  # default — rg content search
# toggle_render = "m"     # default — toggle markdown rendered/raw
#
# Opt-in (no default, uncomment to enable):
# quit = "q"
# toggle = "o"
# refresh = "r"
# new_file = "a"
# new_dir = "A"
# rename = "R"
# delete = "D"
# toggle_preview = "p"
# open_in_editor = "e"
# open_externally = "x"
# collapse_all = "W"
# branch_picker = "b"
# enter = "Enter"

[colors]
# Format: ANSI name ("red"), indexed ("indexed:240" or "240"), or hex ("#ff0000")
# Run `croot config` to see the full resolved palette.
# popup_bg = "indexed:240"
# popup_fg = "indexed:15"
# popup_accent = "indexed:12"
# popup_input_bg = "indexed:236"
# popup_selected_danger_bg = "red"
# dir_color = "yellow"
# default_fg = "reset"
# find_match = "cyan"
"##
        .to_string()
    }
}

pub fn config_path() -> PathBuf {
    dirs_fallback().join("croot").join("config.toml")
}

fn dirs_fallback() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg);
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config");
    }
    PathBuf::from(".config")
}

/// Resolve the editor command: config → $VISUAL → $EDITOR → "vi".
pub fn resolve_editor(config: &Config) -> String {
    if let Some(ref cmd) = config.editor.command {
        if !cmd.is_empty() {
            return cmd.clone();
        }
    }
    if let Ok(v) = std::env::var("VISUAL") {
        if !v.is_empty() {
            return v;
        }
    }
    if let Ok(v) = std::env::var("EDITOR") {
        if !v.is_empty() {
            return v;
        }
    }
    "vi".to_string()
}

/// Read a dotted key (e.g. `tree.show_hidden`) from the resolved config.
/// Always returns effective values, including built-in defaults for keybindings.
pub fn get_value(key: &str) -> Result<String, String> {
    let mut config = Config::load();
    config.keybindings = config.keybindings.resolved();
    config.colors = config.colors.resolved();
    let serialized =
        toml::to_string(&config).map_err(|e| format!("Failed to serialize config: {e}"))?;
    let table: toml::Value =
        toml::from_str(&serialized).map_err(|e| format!("Failed to parse config: {e}"))?;

    let val = navigate(&table, key)?;
    Ok(format_value(val))
}

/// Set a dotted key (e.g. `preview.split_ratio`) to a value in the config file.
/// Creates the file from defaults if it doesn't exist.
pub fn set_value(key: &str, value: &str) -> Result<(), String> {
    let path = config_path();

    let content = if let Ok(c) = std::fs::read_to_string(&path) {
        c
    } else {
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }
        String::new()
    };

    let mut table: toml::Value = if content.is_empty() {
        toml::Value::Table(toml::map::Map::new())
    } else {
        toml::from_str(&content).map_err(|e| format!("Failed to parse config: {e}"))?
    };

    let parsed_value = parse_value_str(value);

    // Check if target is an array — not supported via set
    if let Ok(existing) = navigate(&table, key) {
        if existing.is_array() {
            return Err(format!(
                "Cannot set array value '{key}' via CLI. Use `croot config edit` instead."
            ));
        }
    }

    insert_at_key(&mut table, key, parsed_value)?;

    let output =
        toml::to_string_pretty(&table).map_err(|e| format!("Failed to serialize config: {e}"))?;
    std::fs::write(&path, output).map_err(|e| format!("Failed to write config: {e}"))?;

    Ok(())
}

/// Navigate a TOML value tree by dotted key path.
fn navigate<'a>(val: &'a toml::Value, key: &str) -> Result<&'a toml::Value, String> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = val;
    for part in &parts {
        match current.get(part) {
            Some(v) => current = v,
            None => return Err(format!("Key '{key}' not found")),
        }
    }
    Ok(current)
}

/// Insert a value at a dotted key path, creating intermediate tables as needed.
fn insert_at_key(root: &mut toml::Value, key: &str, value: toml::Value) -> Result<(), String> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() {
        return Err("Empty key".to_string());
    }

    let mut current = root;
    // Navigate/create intermediate tables
    for part in &parts[..parts.len() - 1] {
        if !current.get(part).is_some_and(toml::Value::is_table) {
            current
                .as_table_mut()
                .ok_or_else(|| format!("Expected table at '{part}'"))?
                .insert(
                    (*part).to_string(),
                    toml::Value::Table(toml::map::Map::new()),
                );
        }
        current = current.get_mut(part).unwrap();
    }

    let leaf = parts.last().unwrap();
    current
        .as_table_mut()
        .ok_or_else(|| format!("Expected table for key '{key}'"))?
        .insert((*leaf).to_string(), value);

    Ok(())
}

/// Parse a CLI string into an appropriate TOML value type.
fn parse_value_str(s: &str) -> toml::Value {
    match s {
        "true" => toml::Value::Boolean(true),
        "false" => toml::Value::Boolean(false),
        _ => {
            // Try integer
            if let Ok(n) = s.parse::<i64>() {
                return toml::Value::Integer(n);
            }
            // Try float
            if let Ok(f) = s.parse::<f64>() {
                return toml::Value::Float(f);
            }
            toml::Value::String(s.to_string())
        }
    }
}

/// Format a TOML value for display.
fn format_value(val: &toml::Value) -> String {
    match val {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(n) => n.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value).collect();
            format!("[{}]", items.join(", "))
        }
        toml::Value::Table(_) | toml::Value::Datetime(_) => val.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_color_supports_ansi_names() {
        assert_eq!(parse_color("red"), Some(Color::Red));
        assert_eq!(parse_color("Dark_Gray"), Some(Color::DarkGray));
        assert_eq!(parse_color("light-blue"), Some(Color::LightBlue));
        assert_eq!(parse_color("reset"), Some(Color::Reset));
    }

    #[test]
    fn parse_color_supports_indexed_forms() {
        assert_eq!(parse_color("240"), Some(Color::Indexed(240)));
        assert_eq!(parse_color("indexed:15"), Some(Color::Indexed(15)));
        assert_eq!(parse_color("INDEXED:252"), Some(Color::Indexed(252)));
    }

    #[test]
    fn parse_color_supports_hex_rgb() {
        assert_eq!(parse_color("#ff0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("#00FF88"), Some(Color::Rgb(0, 255, 136)));
    }

    #[test]
    fn parse_color_rejects_invalid_values() {
        assert_eq!(parse_color("indexed:999"), None);
        assert_eq!(parse_color("#12"), None);
        assert_eq!(parse_color("not-a-color"), None);
    }

    #[test]
    fn color_config_resolved_fills_missing_defaults() {
        let resolved = ColorConfig::default().resolved();

        assert_eq!(resolved.popup_bg.as_deref(), Some(DEFAULT_COLORS.popup_bg));
        assert_eq!(resolved.popup_fg.as_deref(), Some(DEFAULT_COLORS.popup_fg));
        assert_eq!(
            resolved.dir_color.as_deref(),
            Some(DEFAULT_COLORS.dir_color)
        );
    }

    #[test]
    fn color_config_resolved_preserves_user_values() {
        let config = ColorConfig {
            popup_bg: Some("#101010".to_string()),
            dir_color: Some("cyan".to_string()),
            ..ColorConfig::default()
        };
        let resolved = config.resolved();

        assert_eq!(resolved.popup_bg.as_deref(), Some("#101010"));
        assert_eq!(resolved.dir_color.as_deref(), Some("cyan"));
        assert_eq!(resolved.popup_fg.as_deref(), Some(DEFAULT_COLORS.popup_fg));
    }

    #[test]
    fn resolved_toml_includes_color_defaults() {
        let toml = Config::default().to_toml_string();

        assert!(toml.contains("[colors]"));
        assert!(toml.contains("popup_bg = \"indexed:240\""));
        assert!(toml.contains("dir_color = \"yellow\""));
    }

    #[test]
    fn default_template_mentions_colors_section() {
        let template = Config::default_toml_with_comments();

        assert!(template.contains("[colors]"));
        assert!(template.contains("popup_bg"));
    }
}
