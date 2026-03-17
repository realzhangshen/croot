#![allow(dead_code)] // Config schema fields are deserialized from TOML; not all consumed yet

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneralConfig {
    #[serde(default = "default_true")]
    pub use_trash: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self { use_trash: true }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
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
    pub popup_input_fg: Option<String>,
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
    pub popup_input_fg: &'static str,
    pub popup_selected_danger_bg: &'static str,
}

pub const DEFAULT_COLORS: ColorDefaults = ColorDefaults {
    git_modified: "yellow",
    git_added: "green",
    git_deleted: "red",
    git_ignored: "dark_gray",
    git_conflicted: "light_red",
    git_staged_modified: "yellow",
    git_staged_added: "green",
    git_staged_deleted: "red",
    unfocused_header_bg: "dark_gray",
    unfocused_header_fg: "black",
    hex_values: "blue",
    hex_ascii: "gray",
    preview_dir_name: "blue",
    inline_code: "green",
    tree_line: "dark_gray",
    status_bar_bg: "black",
    status_bar_fg: "white",
    dir_color: "blue",
    default_fg: "reset",
    find_match: "cyan",
    popup_fg: "white",
    popup_bg: "black",
    popup_accent: "light_blue",
    popup_border_fg: "reset",
    popup_dim_fg: "reset",
    popup_input_bg: "white",
    popup_input_fg: "black",
    popup_selected_danger_bg: "red",
};

/// Resolve a user color override, falling back to the default if unset.
fn resolve_color(user: Option<&String>, default: &str) -> Option<String> {
    user.cloned().or_else(|| Some(default.to_string()))
}

impl ColorConfig {
    /// Return a copy with `None` fields filled in with built-in defaults.
    #[must_use]
    pub fn resolved(&self) -> Self {
        macro_rules! r {
            ($field:ident) => {
                resolve_color(self.$field.as_ref(), DEFAULT_COLORS.$field)
            };
        }
        Self {
            git_modified: r!(git_modified),
            git_added: r!(git_added),
            git_deleted: r!(git_deleted),
            git_ignored: r!(git_ignored),
            git_conflicted: r!(git_conflicted),
            git_staged_modified: r!(git_staged_modified),
            git_staged_added: r!(git_staged_added),
            git_staged_deleted: r!(git_staged_deleted),
            unfocused_header_bg: r!(unfocused_header_bg),
            unfocused_header_fg: r!(unfocused_header_fg),
            hex_values: r!(hex_values),
            hex_ascii: r!(hex_ascii),
            preview_dir_name: r!(preview_dir_name),
            inline_code: r!(inline_code),
            tree_line: r!(tree_line),
            status_bar_bg: r!(status_bar_bg),
            status_bar_fg: r!(status_bar_fg),
            dir_color: r!(dir_color),
            default_fg: r!(default_fg),
            find_match: r!(find_match),
            popup_fg: r!(popup_fg),
            popup_bg: r!(popup_bg),
            popup_accent: r!(popup_accent),
            popup_border_fg: r!(popup_border_fg),
            popup_dim_fg: r!(popup_dim_fg),
            popup_input_bg: r!(popup_input_bg),
            popup_input_fg: r!(popup_input_fg),
            popup_selected_danger_bg: r!(popup_selected_danger_bg),
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
    #[must_use]
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

    // Split on '+' but handle trailing '+' as the literal key (e.g. "Ctrl++" → Ctrl + '+')
    let parts: Vec<&str> = s.split('+').collect();
    let mut modifiers = KeyModifiers::empty();

    let key_part = if parts.len() == 1 {
        parts[0]
    } else {
        // If the last part is empty, the key is literally '+' (e.g. "Ctrl++" or just "+")
        let last = if parts[parts.len() - 1].is_empty() {
            "+"
        } else {
            parts[parts.len() - 1]
        };
        // Parse modifier parts (all but last, and skip trailing empty from the '+' key)
        let modifier_end = if parts[parts.len() - 1].is_empty() {
            parts.len().saturating_sub(2) // skip both the empty last and the empty before it
        } else {
            parts.len() - 1
        };
        for &part in &parts[..modifier_end] {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                "alt" => modifiers |= KeyModifiers::ALT,
                "super" | "cmd" | "command" => modifiers |= KeyModifiers::SUPER,
                "" => {} // skip empty parts from consecutive '+'
                _ => return None,
            }
        }
        last
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
        s if s.chars().count() == 1 => {
            let ch = s.chars().next().unwrap();
            // Use the original case from key_part, not lowered
            let original_ch = key_part.chars().next().unwrap();
            if original_ch.is_uppercase() {
                modifiers |= KeyModifiers::SHIFT;
            }
            KeyCode::Char(original_ch.to_lowercase().next().unwrap_or(ch))
        }
        s if s.starts_with('f') => {
            let num: u8 = s.get(1..)?.parse().ok()?;
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
    #[serde(default = "default_true")]
    pub image_preview: bool,
    #[serde(default = "default_true")]
    pub show_git_diff: bool,
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
            image_preview: true,
            show_git_diff: true,
        }
    }
}

impl Config {
    /// Load config from ~/.config/croot/config.toml, or return defaults.
    pub fn load() -> Self {
        let path = config_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("croot: warning: config parse error: {e}; using defaults");
                    Self::default()
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(e) => {
                eprintln!("croot: warning: cannot read config: {e}; using defaults");
                Self::default()
            }
        }
    }

    /// Parse config from a TOML string, returning defaults on error.
    /// On parse errors, writes a warning to `warn_sink` (if provided).
    pub fn parse_with_warning(content: &str, warn_sink: &mut Option<String>) -> Self {
        match toml::from_str(content) {
            Ok(cfg) => cfg,
            Err(e) => {
                let msg = format!("croot: warning: config parse error: {e}; using defaults");
                if let Some(ref mut sink) = warn_sink {
                    *sink = msg;
                } else {
                    eprintln!("{msg}");
                }
                Self::default()
            }
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

[general]
# use_trash = true   # Move to OS trash instead of permanent delete

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
# image_preview = true
# show_git_diff = true

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
# Built-in defaults are ANSI-only and already tuned for stronger popup/input contrast.
# Add entries here only when you want to override them.
# Run `croot config` to see the full resolved palette.
# popup_bg = "black"
# popup_fg = "white"
# popup_accent = "light_blue"
# popup_border_fg = "blue"
# popup_input_bg = "white"
# popup_input_fg = "black"
# popup_selected_danger_bg = "red"
# status_bar_bg = "black"
# dir_color = "blue"
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
        match current.get(part) {
            Some(v) if v.is_table() => { /* exists and is a table — fall through to navigate below */
            }
            Some(_) => {
                return Err(format!(
                    "Key '{part}' already exists as a non-table value; cannot create sub-key"
                ));
            }
            None => {
                current
                    .as_table_mut()
                    .ok_or_else(|| format!("Expected table at '{part}'"))?
                    .insert(
                        (*part).to_string(),
                        toml::Value::Table(toml::map::Map::new()),
                    );
            }
        }
        current = current
            .get_mut(part)
            .ok_or_else(|| format!("Internal error: key '{part}' missing after insert"))?;
    }

    let leaf = parts.last().expect("non-empty after early return");
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
    fn general_config_defaults_to_use_trash_true() {
        let config = Config::default();
        assert!(config.general.use_trash, "use_trash should default to true");
    }

    #[test]
    fn general_config_deserializes_without_section() {
        let content = r"
[tree]
show_hidden = false
";
        let cfg: Config = toml::from_str(content).unwrap();
        assert!(
            cfg.general.use_trash,
            "Missing [general] should default use_trash to true"
        );
    }

    #[test]
    fn general_config_deserializes_use_trash_false() {
        let content = r"
[general]
use_trash = false
";
        let cfg: Config = toml::from_str(content).unwrap();
        assert!(!cfg.general.use_trash, "use_trash should be false when set");
    }

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
        assert!(toml.contains("popup_bg = \"black\""));
        assert!(toml.contains("popup_input_bg = \"white\""));
        assert!(toml.contains("dir_color = \"blue\""));
    }

    #[test]
    fn default_template_mentions_colors_section() {
        let template = Config::default_toml_with_comments();

        assert!(template.contains("[colors]"));
        assert!(template.contains("popup_bg"));
    }

    #[test]
    fn parse_with_warning_valid_toml_returns_config() {
        let content = r"
[tree]
show_hidden = false
";
        let mut warning = None;
        let cfg = Config::parse_with_warning(content, &mut warning);
        assert!(warning.is_none());
        assert!(!cfg.tree.show_hidden);
    }

    #[test]
    fn parse_with_warning_invalid_toml_returns_defaults_and_warns() {
        let content = "this is [not valid{ toml!!!";
        let mut warning = Some(String::new());
        let cfg = Config::parse_with_warning(content, &mut warning);
        // Should return defaults
        assert!(cfg.tree.show_hidden); // default is true
                                       // Should have written a warning
        let msg = warning.unwrap();
        assert!(
            msg.contains("config parse error"),
            "Expected warning message, got: {msg}"
        );
    }

    #[test]
    fn parse_with_warning_wrong_types_returns_defaults() {
        // Valid TOML but wrong types for our schema
        let content = r#"
[tree]
show_hidden = "not a bool"
"#;
        let mut warning = Some(String::new());
        let cfg = Config::parse_with_warning(content, &mut warning);
        assert!(cfg.tree.show_hidden); // default
        assert!(warning.unwrap().contains("config parse error"));
    }

    #[test]
    fn insert_at_key_rejects_overwrite_non_table() {
        let mut root = toml::Value::Table(toml::map::Map::new());
        insert_at_key(&mut root, "theme", toml::Value::String("dark".to_string())).unwrap();
        // Now try to set "theme.bg" — should fail because "theme" is a string
        let result = insert_at_key(
            &mut root,
            "theme.bg",
            toml::Value::String("#000".to_string()),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("non-table"));
    }

    #[test]
    fn insert_at_key_creates_missing_intermediate() {
        let mut root = toml::Value::Table(toml::map::Map::new());
        insert_at_key(&mut root, "a.b.c", toml::Value::String("val".to_string())).unwrap();
        let val = navigate(&root, "a.b.c").unwrap();
        assert_eq!(val.as_str(), Some("val"));
    }

    #[test]
    fn insert_at_key_preserves_existing_table() {
        let mut root = toml::Value::Table(toml::map::Map::new());
        insert_at_key(&mut root, "a.x", toml::Value::Integer(1)).unwrap();
        insert_at_key(&mut root, "a.y", toml::Value::Integer(2)).unwrap();
        // Both keys should exist
        assert_eq!(navigate(&root, "a.x").unwrap().as_integer(), Some(1));
        assert_eq!(navigate(&root, "a.y").unwrap().as_integer(), Some(2));
    }

    #[test]
    fn show_git_diff_defaults_to_true() {
        let config = Config::default();
        assert!(config.preview.show_git_diff);
    }

    #[test]
    fn show_git_diff_deserializes_false() {
        let content = r"
[preview]
show_git_diff = false
";
        let cfg: Config = toml::from_str(content).unwrap();
        assert!(!cfg.preview.show_git_diff);
    }

    #[test]
    fn show_git_diff_defaults_when_missing() {
        let content = r"
[preview]
auto_preview = true
";
        let cfg: Config = toml::from_str(content).unwrap();
        assert!(cfg.preview.show_git_diff);
    }

    #[test]
    fn parse_key_plus_literal() {
        // "+" alone should parse as the '+' character
        let result = parse_key("+");
        assert_eq!(result, Some((KeyCode::Char('+'), KeyModifiers::empty())));
    }

    #[test]
    fn parse_key_ctrl_plus() {
        // "Ctrl++" should parse as Ctrl + '+'
        let result = parse_key("Ctrl++");
        assert_eq!(result, Some((KeyCode::Char('+'), KeyModifiers::CONTROL)));
    }

    #[test]
    fn parse_key_multibyte_f_prefix_no_panic() {
        // "fé" should not panic (previously sliced at byte boundary mid-char)
        let result = parse_key("fé");
        assert!(result.is_none());
    }
}
