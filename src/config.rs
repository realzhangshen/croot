#![allow(dead_code)] // Config schema fields are deserialized from TOML; not all consumed yet

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
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
    pub keybindings: KeybindingsConfig,
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
    pub goto_top: Option<String>,
    pub goto_bottom: Option<String>,
    pub select: Option<String>,
    pub clear_select: Option<String>,
    pub delete_selected: Option<String>,
    pub branch_picker: Option<String>,
    pub enter: Option<String>,
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
    pub fn to_toml_string(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }

    /// Return a hand-written default config template with comments.
    pub fn default_toml_with_comments() -> String {
        r#"# croot configuration
# Full reference: croot config (shows all resolved values)

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

# [keybindings]
# Keyboard shortcuts are disabled by default.
# Uncomment any line below to enable that shortcut.
# Supports: single chars ("q", "j"), named keys ("Enter", "Esc", "Space"),
#           modifiers ("Ctrl+c", "Shift+a", "Alt+x")
# quit = "q"
# cursor_up = "k"
# cursor_down = "j"
# cursor_left = "h"
# cursor_right = "l"
# toggle = "o"
# refresh = "r"
# new_file = "a"
# new_dir = "A"
# rename = "R"
# delete = "D"
# toggle_preview = "p"
# toggle_render = "m"
# open_in_editor = "e"
# open_externally = "x"
# collapse_all = "W"
# search = "/"
# goto_top = "g"
# goto_bottom = "G"
# select = "Space"
# clear_select = "Esc"
# delete_selected = "X"
# branch_picker = "b"
# enter = "Enter"
"#
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

/// Read a dotted key (e.g. `tree.show_hidden`) from the config file.
pub fn get_value(key: &str) -> Result<String, String> {
    let path = config_path();
    let content = std::fs::read_to_string(&path)
        .map_err(|_| format!("Config file not found: {}", path.display()))?;
    let table: toml::Value =
        toml::from_str(&content).map_err(|e| format!("Failed to parse config: {e}"))?;

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
