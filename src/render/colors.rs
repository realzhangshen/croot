use std::sync::OnceLock;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

use crate::config::{parse_color, ColorConfig, DEFAULT_COLORS};

/// Resolve a single color override, falling back to parsing the built-in
/// default string. The default must be a valid color spec (enforced by a
/// debug panic).
fn resolve_color_field(value: Option<&String>, default: &'static str) -> Color {
    value
        .map(String::as_str)
        .and_then(parse_color)
        .unwrap_or_else(|| parse_color(default).expect("default color should parse"))
}

/// Declarative palette: one line per field, generates the `ResolvedColors`
/// struct, its `from_config` constructor, and the public `fn name() -> Color`
/// accessors. The field list must stay in sync with the color schema in
/// [`crate::config`] — a mismatch surfaces as a missing-field compile error
/// on `DEFAULT_COLORS.xxx`, so drift is not silent.
macro_rules! define_color_palette {
    ( $( $name:ident ),* $(,)? ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct ResolvedColors {
            $(
                $name: Color,
            )*
        }

        impl Default for ResolvedColors {
            fn default() -> Self {
                Self::from_config(&ColorConfig::default())
            }
        }

        impl ResolvedColors {
            fn from_config(config: &ColorConfig) -> Self {
                Self {
                    $(
                        $name: resolve_color_field(config.$name.as_ref(), DEFAULT_COLORS.$name),
                    )*
                }
            }
        }

        $(
            pub fn $name() -> Color {
                palette().$name
            }
        )*
    };
}

define_color_palette!(
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

/// Tree connectors (│, ├─, └─, preview separator): dim for visual hierarchy.
pub fn tree_connector() -> Style {
    Style::default().fg(tree_line()).add_modifier(Modifier::DIM)
}

/// Tree-view hover row: subtle reverse + dim.
pub fn hover_style() -> Style {
    Style::default().add_modifier(Modifier::REVERSED | Modifier::DIM)
}

/// Preview search target row: rely on terminal reverse-video defaults for
/// contrast instead of hard-coded fg/bg pairs.
pub fn preview_search_target_row() -> Style {
    Style::default().add_modifier(Modifier::REVERSED)
}

/// Preview search target gutter: same reverse-video row fill, but dimmer so
/// line numbers stay secondary.
pub fn preview_search_target_gutter() -> Style {
    preview_search_target_row().add_modifier(Modifier::DIM)
}

/// Apply preview search target row styling to syntax-highlighted content while
/// preserving non-color modifiers such as bold or italic.
pub fn preview_search_target_text(base: Style) -> Style {
    Style::default()
        .add_modifier(Modifier::REVERSED | base.add_modifier)
        .remove_modifier(base.sub_modifier)
}

/// Preview search target match: stronger emphasis within the reversed row.
pub fn preview_search_target_match() -> Style {
    Style::default().add_modifier(Modifier::REVERSED | Modifier::UNDERLINED | Modifier::BOLD)
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
