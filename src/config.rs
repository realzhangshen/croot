#![allow(dead_code)] // Config schema fields are deserialized from TOML; not all consumed yet

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use crate::syntax::theme::SyntaxConfig;

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
    #[serde(default)]
    pub syntax: SyntaxConfig,
}

/// Resolve a user color override, falling back to the default if unset.
fn resolve_color(user: Option<&String>, default: &str) -> Option<String> {
    user.cloned().or_else(|| Some(default.to_string()))
}

/// Declarative schema for the color palette.
///
/// A single invocation produces:
/// - `ColorConfig` (serde-deserialized, user overrides)
/// - `ColorDefaults` + `DEFAULT_COLORS` const (built-in defaults)
/// - `ColorConfig::resolved()` (fill `None` slots with defaults)
///
/// Adding a new color means editing exactly one line here, plus mirroring the
/// field in `render::colors` (which is validated at compile time by the
/// `DEFAULT_COLORS.xxx` access in that module).
macro_rules! define_color_schema {
    ( $( $name:ident = $default:expr ),* $(,)? ) => {
        #[derive(Debug, Clone, Default, Deserialize, Serialize)]
        pub struct ColorConfig {
            $(
                pub $name: Option<String>,
            )*
        }

        #[derive(Debug, Clone, Copy)]
        pub struct ColorDefaults {
            $(
                pub $name: &'static str,
            )*
        }

        pub const DEFAULT_COLORS: ColorDefaults = ColorDefaults {
            $(
                $name: $default,
            )*
        };

        impl ColorConfig {
            /// Return a copy with `None` fields filled in with built-in defaults.
            #[must_use]
            pub fn resolved(&self) -> Self {
                Self {
                    $(
                        $name: resolve_color(self.$name.as_ref(), DEFAULT_COLORS.$name),
                    )*
                }
            }
        }
    };
}

define_color_schema! {
    git_modified = "yellow",
    git_added = "green",
    git_deleted = "red",
    git_ignored = "dark_gray",
    git_conflicted = "light_red",
    git_staged_modified = "yellow",
    git_staged_added = "green",
    git_staged_deleted = "red",
    unfocused_header_bg = "dark_gray",
    unfocused_header_fg = "black",
    hex_values = "blue",
    hex_ascii = "gray",
    preview_dir_name = "blue",
    inline_code = "green",
    tree_line = "dark_gray",
    status_bar_bg = "black",
    status_bar_fg = "white",
    dir_color = "blue",
    default_fg = "reset",
    find_match = "cyan",
    popup_fg = "white",
    popup_bg = "black",
    popup_accent = "light_blue",
    popup_border_fg = "reset",
    popup_dim_fg = "reset",
    popup_input_bg = "white",
    popup_input_fg = "black",
    popup_selected_danger_bg = "red",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchOpenMode {
    /// Open search results in an external/GUI editor (background, no TUI suspend).
    External,
    /// Open search results in the terminal editor (suspend TUI).
    Editor,
}

impl Default for SearchOpenMode {
    fn default() -> Self {
        Self::External
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
    #[serde(default)]
    pub open_mode: SearchOpenMode,
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
            open_mode: SearchOpenMode::default(),
        }
    }
}

/// Declarative schema for the keybinding map.
///
/// Generates:
/// - `KeybindingsConfig` struct (serde-deserialized, user overrides)
/// - `default_key_for` lookup function (name → built-in default)
/// - `KeybindingsConfig::resolved()` (fill defaulted slots, preserve opt-ins)
///
/// The schema distinguishes two kinds of fields:
///
/// - `defaults { name = "key", ... }`: shortcuts with a built-in default key.
///   A user may override the key or disable it with `""`.
/// - `opt_in { name, ... }`: shortcuts that are only active when the user
///   opts in (e.g. `q = "quit"`). Defaults stay `None`.
macro_rules! define_keybinding_schema {
    (
        defaults { $( $def_name:ident = $def_key:expr ),* $(,)? }
        opt_in { $( $opt_name:ident ),* $(,)? }
    ) => {
        #[derive(Debug, Clone, Default, Deserialize, Serialize)]
        pub struct KeybindingsConfig {
            $( pub $def_name: Option<String>, )*
            $( pub $opt_name: Option<String>, )*
        }

        /// Built-in default key for a keybinding field. Returns `None` for
        /// opt-in-only fields.
        #[must_use]
        pub fn default_key_for(field: &str) -> Option<&'static str> {
            match field {
                $( stringify!($def_name) => Some($def_key), )*
                _ => None,
            }
        }

        impl KeybindingsConfig {
            /// Return a copy with defaulted fields filled in with built-in
            /// defaults. User-set fields (including `""` to disable) are kept
            /// as-is; opt-in fields stay as the user left them.
            #[must_use]
            pub fn resolved(&self) -> Self {
                fn fill(field: Option<&String>, default: &'static str) -> Option<String> {
                    match field {
                        Some(s) => Some(s.clone()),
                        None => Some(default.to_string()),
                    }
                }
                Self {
                    $( $def_name: fill(self.$def_name.as_ref(), $def_key), )*
                    $( $opt_name: self.$opt_name.clone(), )*
                }
            }
        }
    };
}

define_keybinding_schema! {
    defaults {
        cursor_up = "Up",
        cursor_down = "Down",
        cursor_left = "Left",
        cursor_right = "Right",
        goto_top = "Home",
        goto_bottom = "End",
        search = "/",
        filter = "f",
        global_search = "s",
        global_search_content = "S",
        toggle_render = "m",
    }
    opt_in {
        quit,
        toggle,
        refresh,
        new_file,
        new_dir,
        rename,
        delete,
        toggle_preview,
        open_in_editor,
        open_externally,
        collapse_all,
        branch_picker,
        enter,
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
    #[serde(default)]
    pub external: Option<String>,
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
    #[serde(default = "default_image_preview")]
    pub image_preview: bool,
    #[serde(default = "default_true")]
    pub show_git_diff: bool,
}

/// Default value for `image_preview` follows the `image-preview` Cargo feature:
/// if the feature is compiled in, the default is `true`; otherwise the flag is
/// a no-op anyway, so defaulting to `false` makes `croot config` output match
/// reality instead of misleadingly claiming the feature is enabled.
fn default_image_preview() -> bool {
    cfg!(feature = "image-preview")
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
            image_preview: default_image_preview(),
            show_git_diff: true,
        }
    }
}

impl Config {
    pub fn syntax_enabled(&self) -> bool {
        self.syntax.enabled.unwrap_or(self.preview.syntax_highlight)
    }

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

    /// Return the hand-written default config template with comments.
    ///
    /// The source of truth lives in `docs/default_config.toml` so readers
    /// (and CI) can inspect it without opening Rust source. This function
    /// just re-exports it through `include_str!`.
    pub fn default_toml_with_comments() -> String {
        DEFAULT_CONFIG_TEMPLATE.to_string()
    }
}

/// Embedded copy of `docs/default_config.toml` — the canonical template
/// emitted by `croot config init` and friends.
const DEFAULT_CONFIG_TEMPLATE: &str = include_str!("../docs/default_config.toml");

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

/// Resolve the external editor command from config, or `None` (caller falls back to OS open).
pub fn resolve_external_editor(config: &Config) -> Option<String> {
    config
        .editor
        .external
        .as_ref()
        .filter(|s| !s.is_empty())
        .cloned()
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
    fn image_preview_default_tracks_cargo_feature() {
        // The default for preview.image_preview should match whether the
        // image-preview Cargo feature is enabled. This keeps `croot config`
        // output honest instead of claiming image preview is on in a build
        // that can't actually render images.
        let config = Config::default();
        assert_eq!(
            config.preview.image_preview,
            cfg!(feature = "image-preview")
        );

        // Parsing an empty [preview] section should produce the same value:
        // this exercises the serde default, not just Default::default().
        let content = r"
[preview]
";
        let cfg: Config = toml::from_str(content).unwrap();
        assert_eq!(cfg.preview.image_preview, cfg!(feature = "image-preview"));
    }

    #[test]
    fn image_preview_explicit_config_overrides_default() {
        // User override still wins regardless of build feature.
        let content = r"
[preview]
image_preview = true
";
        let cfg: Config = toml::from_str(content).unwrap();
        assert!(cfg.preview.image_preview);

        let content = r"
[preview]
image_preview = false
";
        let cfg: Config = toml::from_str(content).unwrap();
        assert!(!cfg.preview.image_preview);
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
    fn default_template_mentions_syntax_section() {
        let template = Config::default_toml_with_comments();

        assert!(template.contains("[syntax]"));
        assert!(template.contains("[syntax.tokens.keyword]"));
    }

    #[test]
    fn default_template_parses_as_config() {
        // Guard against drift: the embedded docs/default_config.toml must
        // always be a valid Config. If someone adds a new commented-out
        // example that happens to have a syntax error, this test catches it.
        let template = Config::default_toml_with_comments();
        toml::from_str::<Config>(&template)
            .expect("embedded default_config.toml should deserialize into Config");
    }

    #[test]
    fn syntax_enabled_uses_new_section_when_set() {
        let content = r#"
[preview]
syntax_highlight = false

[syntax]
enabled = true
"#;
        let cfg: Config = toml::from_str(content).unwrap();
        assert!(cfg.syntax_enabled());
    }

    #[test]
    fn syntax_enabled_falls_back_to_legacy_preview_toggle() {
        let content = r#"
[preview]
syntax_highlight = false
"#;
        let cfg: Config = toml::from_str(content).unwrap();
        assert!(!cfg.syntax_enabled());
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

    // ── SearchOpenMode / EditorConfig.external tests ─────────────────

    #[test]
    fn search_open_mode_defaults_to_external() {
        let config = Config::default();
        assert_eq!(config.search.open_mode, SearchOpenMode::External);
    }

    #[test]
    fn search_open_mode_parses_editor() {
        let content = r#"
[search]
open_mode = "editor"
"#;
        let cfg: Config = toml::from_str(content).unwrap();
        assert_eq!(cfg.search.open_mode, SearchOpenMode::Editor);
    }

    #[test]
    fn search_open_mode_parses_external() {
        let content = r#"
[search]
open_mode = "external"
"#;
        let cfg: Config = toml::from_str(content).unwrap();
        assert_eq!(cfg.search.open_mode, SearchOpenMode::External);
    }

    #[test]
    fn search_open_mode_defaults_when_missing_from_toml() {
        let content = r"
[search]
max_results = 100
";
        let cfg: Config = toml::from_str(content).unwrap();
        assert_eq!(cfg.search.open_mode, SearchOpenMode::External);
    }

    #[test]
    fn editor_external_defaults_to_none() {
        let config = Config::default();
        assert!(config.editor.external.is_none());
    }

    #[test]
    fn editor_external_parses_from_toml() {
        let content = r#"
[editor]
external = "code -g"
"#;
        let cfg: Config = toml::from_str(content).unwrap();
        assert_eq!(cfg.editor.external.as_deref(), Some("code -g"));
    }

    #[test]
    fn resolve_external_editor_returns_none_when_unset() {
        let config = Config::default();
        assert!(resolve_external_editor(&config).is_none());
    }

    #[test]
    fn resolve_external_editor_returns_configured_value() {
        let mut config = Config::default();
        config.editor.external = Some("code -g".to_string());
        assert_eq!(resolve_external_editor(&config).as_deref(), Some("code -g"));
    }

    #[test]
    fn resolve_external_editor_ignores_empty_string() {
        let mut config = Config::default();
        config.editor.external = Some(String::new());
        assert!(resolve_external_editor(&config).is_none());
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
