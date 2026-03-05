#![allow(dead_code)] // Config schema fields are deserialized from TOML; not all consumed yet

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub tree: TreeConfig,
    #[serde(default)]
    pub preview: PreviewConfig,
    #[serde(default)]
    pub cmux: CmuxConfig,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct CmuxConfig {
    #[serde(default = "default_split_direction")]
    pub split_direction: String,
    #[serde(default = "default_split_ratio")]
    pub split_ratio: f32,
}

fn default_true() -> bool {
    true
}
fn default_preview_delay() -> u64 {
    150
}
fn default_split_direction() -> String {
    "right".into()
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

impl Default for CmuxConfig {
    fn default() -> Self {
        Self {
            split_direction: "right".into(),
            split_ratio: 0.5,
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
}

fn config_path() -> PathBuf {
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
