use std::sync::OnceLock;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

use crate::config::{parse_color, ColorConfig, DEFAULT_COLORS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResolvedColors {
    git_modified: Color,
    git_added: Color,
    git_deleted: Color,
    git_ignored: Color,
    git_conflicted: Color,
    git_staged_modified: Color,
    git_staged_added: Color,
    git_staged_deleted: Color,
    unfocused_header_bg: Color,
    unfocused_header_fg: Color,
    hex_values: Color,
    hex_ascii: Color,
    preview_dir_name: Color,
    inline_code: Color,
    tree_line: Color,
    status_bar_bg: Color,
    status_bar_fg: Color,
    dir_color: Color,
    default_fg: Color,
    find_match: Color,
    popup_fg: Color,
    popup_bg: Color,
    popup_accent: Color,
    popup_border_fg: Color,
    popup_dim_fg: Color,
    popup_input_bg: Color,
    popup_input_fg: Color,
    popup_selected_danger_bg: Color,
}

impl Default for ResolvedColors {
    fn default() -> Self {
        Self::from_config(&ColorConfig::default())
    }
}

impl ResolvedColors {
    fn from_config(config: &ColorConfig) -> Self {
        fn resolve(value: Option<&String>, default: &'static str) -> Color {
            value
                .map(String::as_str)
                .and_then(parse_color)
                .unwrap_or_else(|| parse_color(default).expect("default color should parse"))
        }

        Self {
            git_modified: resolve(config.git_modified.as_ref(), DEFAULT_COLORS.git_modified),
            git_added: resolve(config.git_added.as_ref(), DEFAULT_COLORS.git_added),
            git_deleted: resolve(config.git_deleted.as_ref(), DEFAULT_COLORS.git_deleted),
            git_ignored: resolve(config.git_ignored.as_ref(), DEFAULT_COLORS.git_ignored),
            git_conflicted: resolve(
                config.git_conflicted.as_ref(),
                DEFAULT_COLORS.git_conflicted,
            ),
            git_staged_modified: resolve(
                config.git_staged_modified.as_ref(),
                DEFAULT_COLORS.git_staged_modified,
            ),
            git_staged_added: resolve(
                config.git_staged_added.as_ref(),
                DEFAULT_COLORS.git_staged_added,
            ),
            git_staged_deleted: resolve(
                config.git_staged_deleted.as_ref(),
                DEFAULT_COLORS.git_staged_deleted,
            ),
            unfocused_header_bg: resolve(
                config.unfocused_header_bg.as_ref(),
                DEFAULT_COLORS.unfocused_header_bg,
            ),
            unfocused_header_fg: resolve(
                config.unfocused_header_fg.as_ref(),
                DEFAULT_COLORS.unfocused_header_fg,
            ),
            hex_values: resolve(config.hex_values.as_ref(), DEFAULT_COLORS.hex_values),
            hex_ascii: resolve(config.hex_ascii.as_ref(), DEFAULT_COLORS.hex_ascii),
            preview_dir_name: resolve(
                config.preview_dir_name.as_ref(),
                DEFAULT_COLORS.preview_dir_name,
            ),
            inline_code: resolve(config.inline_code.as_ref(), DEFAULT_COLORS.inline_code),
            tree_line: resolve(config.tree_line.as_ref(), DEFAULT_COLORS.tree_line),
            status_bar_bg: resolve(config.status_bar_bg.as_ref(), DEFAULT_COLORS.status_bar_bg),
            status_bar_fg: resolve(config.status_bar_fg.as_ref(), DEFAULT_COLORS.status_bar_fg),
            dir_color: resolve(config.dir_color.as_ref(), DEFAULT_COLORS.dir_color),
            default_fg: resolve(config.default_fg.as_ref(), DEFAULT_COLORS.default_fg),
            find_match: resolve(config.find_match.as_ref(), DEFAULT_COLORS.find_match),
            popup_fg: resolve(config.popup_fg.as_ref(), DEFAULT_COLORS.popup_fg),
            popup_bg: resolve(config.popup_bg.as_ref(), DEFAULT_COLORS.popup_bg),
            popup_accent: resolve(config.popup_accent.as_ref(), DEFAULT_COLORS.popup_accent),
            popup_border_fg: resolve(
                config.popup_border_fg.as_ref(),
                DEFAULT_COLORS.popup_border_fg,
            ),
            popup_dim_fg: resolve(config.popup_dim_fg.as_ref(), DEFAULT_COLORS.popup_dim_fg),
            popup_input_bg: resolve(
                config.popup_input_bg.as_ref(),
                DEFAULT_COLORS.popup_input_bg,
            ),
            popup_input_fg: resolve(
                config.popup_input_fg.as_ref(),
                DEFAULT_COLORS.popup_input_fg,
            ),
            popup_selected_danger_bg: resolve(
                config.popup_selected_danger_bg.as_ref(),
                DEFAULT_COLORS.popup_selected_danger_bg,
            ),
        }
    }
}

/// Global color palette. Must be initialized exactly once via `init()` before
/// any `palette()` call. Subsequent `init()` calls are silently ignored by
/// `OnceLock` — this is fine for production (single init at startup) but means
/// tests that need custom colors should construct `ResolvedColors::from_config()`
/// directly rather than going through the global.
static COLORS: OnceLock<ResolvedColors> = OnceLock::new();

fn palette() -> &'static ResolvedColors {
    COLORS.get_or_init(ResolvedColors::default)
}

pub fn init(config: &ColorConfig) {
    let _ = COLORS.set(ResolvedColors::from_config(config));
}

#[cfg(test)]
pub fn init_default_for_tests() {
    let _ = palette();
}

macro_rules! color_getters {
    ($($name:ident),+ $(,)?) => {
        $(
            pub fn $name() -> Color {
                palette().$name
            }
        )+
    };
}

color_getters!(
    git_modified,
    git_added,
    git_deleted,
    git_ignored,
    git_conflicted,
    git_staged_modified,
    git_staged_added,
    git_staged_deleted,
    unfocused_header_bg,
    unfocused_header_fg,
    hex_values,
    hex_ascii,
    preview_dir_name,
    inline_code,
    tree_line,
    status_bar_bg,
    status_bar_fg,
    dir_color,
    default_fg,
    find_match,
    popup_fg,
    popup_bg,
    popup_accent,
    popup_border_fg,
    popup_dim_fg,
    popup_input_bg,
    popup_input_fg,
    popup_selected_danger_bg,
);

/// Tree connectors (│, ├─, └─, preview separator): dim for visual hierarchy.
pub fn tree_connector() -> Style {
    Style::default().fg(tree_line()).add_modifier(Modifier::DIM)
}

/// Tree-view hover row: subtle reverse + dim.
pub fn hover_style() -> Style {
    Style::default().add_modifier(Modifier::REVERSED | Modifier::DIM)
}

/// Popup / menu base: REVERSED uses the terminal's default fg/bg pair,
/// which themes carefully tune for optimal contrast.
pub fn popup_base() -> Style {
    Style::default().add_modifier(Modifier::REVERSED)
}

/// Popup selected item: explicit blue/white for clear visual distinction.
pub fn popup_selected() -> Style {
    Style::reset()
        .bg(Color::Blue)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

/// Popup selected danger item (e.g. Delete).
pub fn popup_selected_danger() -> Style {
    Style::reset()
        .bg(popup_selected_danger_bg())
        .fg(popup_fg())
        .add_modifier(Modifier::BOLD)
}

/// Popup dim text (hints, separators, [Cancel]).
pub fn popup_dim() -> Style {
    Style::default()
        .fg(popup_dim_fg())
        .add_modifier(Modifier::REVERSED | Modifier::DIM)
}

/// Popup border: REVERSED to match `popup_base` for consistent appearance.
pub fn popup_border() -> Style {
    Style::default()
        .fg(popup_border_fg())
        .add_modifier(Modifier::REVERSED)
}

/// Popup input field.
pub fn popup_input() -> Style {
    Style::default().fg(popup_input_fg()).bg(popup_input_bg())
}

/// Popup input prompt (e.g. `> `).
pub fn popup_prompt() -> Style {
    Style::default()
        .fg(popup_accent())
        .bg(popup_input_bg())
        .add_modifier(Modifier::BOLD)
}

/// Popup input cursor using the input field's own fg/bg.
pub fn popup_cursor() -> Style {
    Style::default().fg(popup_input_bg()).bg(popup_input_fg())
}

/// Popup success text (counts, confirmed states).
pub fn popup_success() -> Style {
    Style::default().fg(git_added()).bg(popup_bg())
}

/// Popup warning text (loading, pending states).
pub fn popup_warning() -> Style {
    Style::default().fg(git_modified()).bg(popup_bg())
}

/// Popup error text.
pub fn popup_error() -> Style {
    Style::default()
        .fg(git_deleted())
        .bg(popup_bg())
        .add_modifier(Modifier::BOLD)
}

/// Status/input bar base text.
pub fn status_input() -> Style {
    Style::default().fg(status_bar_fg()).bg(status_bar_bg())
}

/// Status/input bar cursor.
pub fn status_cursor() -> Style {
    Style::default().fg(status_bar_bg()).bg(status_bar_fg())
}

/// Status/input bar success text.
pub fn status_success() -> Style {
    Style::default().fg(git_added()).bg(status_bar_bg())
}

/// Status/input bar error text.
pub fn status_error() -> Style {
    Style::default().fg(git_deleted()).bg(status_bar_bg())
}

/// Clear a rectangular region and apply a fresh style.
/// Resets each cell first to prevent color bleed from underlying content.
pub fn clear_region(buf: &mut Buffer, rect: Rect, style: Style) {
    for row in rect.y..rect.y + rect.height {
        for col in rect.x..rect.x + rect.width {
            if let Some(cell) = buf.cell_mut((col, row)) {
                cell.reset();
                cell.set_style(style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_colors_use_existing_defaults() {
        let colors = ResolvedColors::from_config(&ColorConfig::default());

        assert_eq!(colors.popup_fg, Color::White);
        assert_eq!(colors.popup_bg, Color::Black);
        assert_eq!(colors.popup_input_fg, Color::Black);
        assert_eq!(colors.dir_color, Color::Blue);
        assert_eq!(colors.default_fg, Color::Reset);
        assert_eq!(colors.popup_selected_danger_bg, Color::Red);
    }

    #[test]
    fn resolved_colors_apply_valid_overrides_and_ignore_invalid_ones() {
        let config = ColorConfig {
            popup_bg: Some("#101010".to_string()),
            popup_fg: Some("indexed:254".to_string()),
            dir_color: Some("light-blue".to_string()),
            popup_dim_fg: Some("nope".to_string()),
            popup_input_fg: Some("black".to_string()),
            ..ColorConfig::default()
        };
        let colors = ResolvedColors::from_config(&config);

        assert_eq!(colors.popup_bg, Color::Rgb(16, 16, 16));
        assert_eq!(colors.popup_fg, Color::Indexed(254));
        assert_eq!(colors.popup_input_fg, Color::Black);
        assert_eq!(colors.dir_color, Color::LightBlue);
        // "nope" is invalid → falls back to default ("reset")
        assert_eq!(colors.popup_dim_fg, Color::Reset);
    }

    #[test]
    fn clear_region_wipes_preexisting_state() {
        init_default_for_tests();

        let area = Rect::new(0, 0, 4, 3);
        let mut buf = Buffer::empty(area);

        for row in 0..area.height {
            for col in 0..area.width {
                if let Some(cell) = buf.cell_mut((col, row)) {
                    cell.set_symbol("X");
                    cell.set_style(Style::default().fg(Color::Red).bg(Color::Green));
                }
            }
        }

        clear_region(&mut buf, area, popup_base());

        for row in 0..area.height {
            for col in 0..area.width {
                let cell = buf.cell((col, row)).unwrap();
                assert_eq!(cell.symbol(), " ", "symbol at ({col},{row})");
                // After reset + REVERSED, fg/bg are default (Reset)
                assert_eq!(cell.fg, Color::Reset, "fg at ({col},{row})");
                assert_eq!(cell.bg, Color::Reset, "bg at ({col},{row})");
                assert!(
                    cell.modifier.contains(Modifier::REVERSED),
                    "should have REVERSED at ({col},{row}): {:?}",
                    cell.modifier
                );
            }
        }
    }
}
