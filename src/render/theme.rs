use ratatui::style::Color;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::colors;

/// Terminal theme colors derived from Ghostty config or sensible defaults.
pub struct Theme {
    pub selected_bg: Color,
    pub tree_line: Color,
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub dir_color: Color,
    pub default_fg: Color,
    // Git colors stay hardcoded — they need to be recognizable regardless of theme.
    pub git_modified: Color,
    pub git_added: Color,
    pub git_deleted: Color,
    pub git_ignored: Color,
    pub git_conflicted: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            selected_bg: colors::SELECTED_BG,
            tree_line: colors::TREE_LINE,
            status_bar_bg: colors::STATUS_BAR_BG,
            status_bar_fg: colors::STATUS_BAR_FG,
            dir_color: colors::DIR_COLOR,
            default_fg: colors::DEFAULT_FG,
            git_modified: colors::GIT_MODIFIED,
            git_added: colors::GIT_ADDED,
            git_deleted: colors::GIT_DELETED,
            git_ignored: colors::GIT_IGNORED,
            git_conflicted: colors::GIT_CONFLICTED,
        }
    }
}

impl Theme {
    /// Detect terminal theme. Returns Ghostty-derived colors if running inside
    /// Ghostty, otherwise falls back to VS Code dark defaults.
    pub fn detect() -> Self {
        if std::env::var("TERM_PROGRAM").as_deref() != Ok("ghostty") {
            return Self::default();
        }

        match Self::from_ghostty() {
            Some(theme) => theme,
            None => Self::default(),
        }
    }

    fn from_ghostty() -> Option<Self> {
        let config_dir = ghostty_config_dir()?;
        let config_path = config_dir.join("config");
        let config_text = std::fs::read_to_string(&config_path).ok()?;

        let mut merged = parse_kv(&config_text);

        // Resolve `theme` key → load theme file and merge (config overrides theme).
        if let Some(theme_name) = resolve_theme_name(merged.get("theme")) {
            if let Some(theme_text) = load_theme_file(&theme_name, &config_dir) {
                let theme_vals = parse_kv(&theme_text);
                // Theme file provides base; config values override.
                let config_overrides = merged.clone();
                merged = theme_vals;
                merged.extend(config_overrides);
            }
        }

        let bg = parse_color_value(merged.get("background"))?;
        let fg = parse_color_value(merged.get("foreground"))?;
        let sel_bg = parse_color_value(merged.get("selection-background"))
            .unwrap_or_else(|| blend(fg, bg, 0.3));

        // Parse palette for ANSI color 3 (yellow).
        let palette_3 = parse_palette_color(&merged, 3);

        let is_light = luminance(bg) > 0.5;

        let status_bar_bg = if is_light {
            shift_brightness(bg, -25)
        } else {
            shift_brightness(bg, 25)
        };

        let status_bar_fg = if contrast_ratio(fg, status_bar_bg) >= 3.0 {
            fg
        } else if is_light {
            Color::Rgb(0, 0, 0)
        } else {
            Color::Rgb(255, 255, 255)
        };

        let dir_color = if is_light {
            fg
        } else {
            palette_3.unwrap_or(colors::DIR_COLOR)
        };

        Some(Self {
            selected_bg: sel_bg,
            tree_line: blend(fg, bg, 0.6),
            status_bar_bg,
            status_bar_fg,
            dir_color,
            default_fg: fg,
            ..Self::default()
        })
    }
}

// ── Ghostty config parsing ──────────────────────────────────────────────────

fn ghostty_config_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg).join("ghostty");
        if p.is_dir() {
            return Some(p);
        }
    }
    let home = std::env::var("HOME").ok()?;
    let p = PathBuf::from(home).join(".config/ghostty");
    if p.is_dir() { Some(p) } else { None }
}

/// Parse key=value lines. Ignores comments (#) and blank lines.
fn parse_kv(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            // palette entries: `palette = 3=#abcdef` → store as `palette.3`
            if key == "palette" {
                if let Some((idx, color)) = value.split_once('=') {
                    map.insert(format!("palette.{}", idx.trim()), color.trim().to_string());
                }
            } else {
                map.insert(key.to_string(), value.to_string());
            }
        }
    }
    map
}

/// Handle `theme = dark:Foo,light:Bar` or `theme = MyTheme`.
fn resolve_theme_name(raw: Option<&String>) -> Option<String> {
    let raw = raw?.trim().to_string();
    if raw.is_empty() {
        return None;
    }
    // Conditional format: `dark:Name,light:Name`
    if raw.contains(':') {
        for part in raw.split(',') {
            let part = part.trim();
            if let Some(name) = part.strip_prefix("dark:") {
                return Some(name.trim().to_string());
            }
        }
        // If no dark variant, take first available.
        if let Some((_, name)) = raw.split(',').next()?.split_once(':') {
            return Some(name.trim().to_string());
        }
    }
    Some(raw)
}

/// Load a theme file from Ghostty's theme directories.
fn load_theme_file(name: &str, config_dir: &Path) -> Option<String> {
    // 1. User themes: ~/.config/ghostty/themes/<name>
    let user_theme = config_dir.join("themes").join(name);
    if let Ok(text) = std::fs::read_to_string(&user_theme) {
        return Some(text);
    }
    // 2. Bundled themes: $GHOSTTY_RESOURCES_DIR/themes/<name>
    if let Ok(res_dir) = std::env::var("GHOSTTY_RESOURCES_DIR") {
        let bundled = PathBuf::from(res_dir).join("themes").join(name);
        if let Ok(text) = std::fs::read_to_string(&bundled) {
            return Some(text);
        }
    }
    None
}

// ── Color parsing & math ────────────────────────────────────────────────────

/// Parse hex color string: `#RGB`, `#RRGGBB`, or bare `RRGGBB`.
fn parse_hex(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.strip_prefix('#').unwrap_or(s);
    match s.len() {
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()?;
            let g = u8::from_str_radix(&s[1..2], 16).ok()?;
            let b = u8::from_str_radix(&s[2..3], 16).ok()?;
            Some((r * 17, g * 17, b * 17))
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some((r, g, b))
        }
        _ => None,
    }
}

fn parse_color_value(val: Option<&String>) -> Option<Color> {
    let (r, g, b) = parse_hex(val?)?;
    Some(Color::Rgb(r, g, b))
}

fn parse_palette_color(map: &HashMap<String, String>, index: u8) -> Option<Color> {
    let key = format!("palette.{}", index);
    parse_color_value(map.get(&key))
}

/// WCAG relative luminance.
fn luminance(color: Color) -> f64 {
    let (r, g, b) = match color {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => return 0.0,
    };
    let to_linear = |c: u8| -> f64 {
        let s = c as f64 / 255.0;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * to_linear(r) + 0.7152 * to_linear(g) + 0.0722 * to_linear(b)
}

/// WCAG contrast ratio between two colors.
fn contrast_ratio(a: Color, b: Color) -> f64 {
    let la = luminance(a);
    let lb = luminance(b);
    let (lighter, darker) = if la > lb { (la, lb) } else { (lb, la) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Blend `from` toward `to` by factor t (0.0 = all `from`, 1.0 = all `to`).
fn blend(from: Color, to: Color, t: f64) -> Color {
    let (r1, g1, b1) = match from {
        Color::Rgb(r, g, b) => (r as f64, g as f64, b as f64),
        _ => return from,
    };
    let (r2, g2, b2) = match to {
        Color::Rgb(r, g, b) => (r as f64, g as f64, b as f64),
        _ => return from,
    };
    Color::Rgb(
        (r1 + (r2 - r1) * t).round() as u8,
        (g1 + (g2 - g1) * t).round() as u8,
        (b1 + (b2 - b1) * t).round() as u8,
    )
}

/// Shift brightness of an RGB color by `delta` (-255..255).
fn shift_brightness(color: Color, delta: i16) -> Color {
    let (r, g, b) = match color {
        Color::Rgb(r, g, b) => (r as i16, g as i16, b as i16),
        _ => return color,
    };
    Color::Rgb(
        (r + delta).clamp(0, 255) as u8,
        (g + delta).clamp(0, 255) as u8,
        (b + delta).clamp(0, 255) as u8,
    )
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_rrggbb() {
        assert_eq!(parse_hex("#ff8000"), Some((255, 128, 0)));
        assert_eq!(parse_hex("ff8000"), Some((255, 128, 0)));
    }

    #[test]
    fn parse_hex_rgb_shorthand() {
        assert_eq!(parse_hex("#f80"), Some((255, 136, 0)));
    }

    #[test]
    fn parse_hex_invalid() {
        assert_eq!(parse_hex(""), None);
        assert_eq!(parse_hex("#gggggg"), None);
        assert_eq!(parse_hex("#12345"), None);
    }

    #[test]
    fn luminance_black_is_zero() {
        let l = luminance(Color::Rgb(0, 0, 0));
        assert!((l - 0.0).abs() < 0.001);
    }

    #[test]
    fn luminance_white_is_one() {
        let l = luminance(Color::Rgb(255, 255, 255));
        assert!((l - 1.0).abs() < 0.001);
    }

    #[test]
    fn light_background_detected() {
        // cursor-light background #f7f7f4
        let l = luminance(Color::Rgb(0xf7, 0xf7, 0xf4));
        assert!(l > 0.5, "luminance {} should be > 0.5 for light bg", l);
    }

    #[test]
    fn dark_background_detected() {
        // typical dark terminal bg
        let l = luminance(Color::Rgb(0x1e, 0x1e, 0x2e));
        assert!(l < 0.5, "luminance {} should be < 0.5 for dark bg", l);
    }

    #[test]
    fn blend_midpoint() {
        let c = blend(
            Color::Rgb(0, 0, 0),
            Color::Rgb(200, 200, 200),
            0.5,
        );
        assert_eq!(c, Color::Rgb(100, 100, 100));
    }

    #[test]
    fn shift_brightness_clamps() {
        let c = shift_brightness(Color::Rgb(250, 10, 128), 30);
        assert_eq!(c, Color::Rgb(255, 40, 158));

        let c = shift_brightness(Color::Rgb(250, 10, 128), -30);
        assert_eq!(c, Color::Rgb(220, 0, 98));
    }

    #[test]
    fn contrast_ratio_black_white() {
        let cr = contrast_ratio(Color::Rgb(0, 0, 0), Color::Rgb(255, 255, 255));
        assert!((cr - 21.0).abs() < 0.1, "contrast ratio {} should be ~21", cr);
    }

    #[test]
    fn parse_kv_basic() {
        let text = "background = #1e1e2e\nforeground = #cdd6f4\n# comment\n\n";
        let map = parse_kv(text);
        assert_eq!(map.get("background").unwrap(), "#1e1e2e");
        assert_eq!(map.get("foreground").unwrap(), "#cdd6f4");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn parse_kv_palette() {
        let text = "palette = 3=#e5c890\npalette = 5=#f4b8e4\n";
        let map = parse_kv(text);
        assert_eq!(map.get("palette.3").unwrap(), "#e5c890");
        assert_eq!(map.get("palette.5").unwrap(), "#f4b8e4");
    }

    #[test]
    fn resolve_theme_simple() {
        let raw = "catppuccin-mocha".to_string();
        assert_eq!(resolve_theme_name(Some(&raw)), Some("catppuccin-mocha".to_string()));
    }

    #[test]
    fn resolve_theme_conditional() {
        let raw = "light:cursor-light,dark:cursor-dark".to_string();
        assert_eq!(resolve_theme_name(Some(&raw)), Some("cursor-dark".to_string()));
    }

    #[test]
    fn resolve_theme_none() {
        assert_eq!(resolve_theme_name(None), None);
        let empty = "".to_string();
        assert_eq!(resolve_theme_name(Some(&empty)), None);
    }

    #[test]
    fn theme_default_matches_colors_module() {
        let t = Theme::default();
        assert_eq!(t.selected_bg, colors::SELECTED_BG);
        assert_eq!(t.status_bar_bg, colors::STATUS_BAR_BG);
        assert_eq!(t.default_fg, colors::DEFAULT_FG);
    }
}
