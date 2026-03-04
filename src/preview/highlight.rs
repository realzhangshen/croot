use std::path::Path;
use std::sync::OnceLock;

use ratatui::style::{Color, Modifier, Style};
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

use super::state::StyledSpan;

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_DARK: OnceLock<Theme> = OnceLock::new();
static THEME_LIGHT: OnceLock<Theme> = OnceLock::new();

fn syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme_dark() -> &'static Theme {
    THEME_DARK.get_or_init(|| {
        let ts = ThemeSet::load_defaults();
        ts.themes["base16-ocean.dark"].clone()
    })
}

fn theme_light() -> &'static Theme {
    THEME_LIGHT.get_or_init(|| {
        let ts = ThemeSet::load_defaults();
        ts.themes["base16-ocean.light"].clone()
    })
}

/// Convert syntect foreground color to ratatui Style.
fn syntect_style_to_ratatui(style: syntect::highlighting::Style) -> Style {
    let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
    let mut s = Style::default().fg(fg);
    if style
        .font_style
        .contains(syntect::highlighting::FontStyle::BOLD)
    {
        s = s.add_modifier(Modifier::BOLD);
    }
    if style
        .font_style
        .contains(syntect::highlighting::FontStyle::ITALIC)
    {
        s = s.add_modifier(Modifier::ITALIC);
    }
    s
}

/// Highlight file content with syntax coloring.
///
/// Returns pre-styled lines ready for rendering.
/// `max_lines` caps how many lines we process (performance guard).
pub fn highlight_file(
    path: &Path,
    content: &str,
    max_lines: usize,
    is_light: bool,
) -> Vec<Vec<StyledSpan>> {
    let ss = syntax_set();
    let theme = if is_light { theme_light() } else { theme_dark() };

    // Find syntax by extension, then by first line
    let syntax = ss
        .find_syntax_for_file(path)
        .ok()
        .flatten()
        .or_else(|| {
            content
                .lines()
                .next()
                .and_then(|first| ss.find_syntax_by_first_line(first))
        })
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut highlighter = syntect::easy::HighlightLines::new(syntax, theme);
    let mut result = Vec::with_capacity(max_lines.min(content.lines().count()));

    for (i, line) in content.lines().enumerate() {
        if i >= max_lines {
            break;
        }

        match highlighter.highlight_line(line, ss) {
            Ok(ranges) => {
                let spans: Vec<StyledSpan> = ranges
                    .into_iter()
                    .map(|(style, text)| (text.to_string(), syntect_style_to_ratatui(style)))
                    .collect();
                result.push(spans);
            }
            Err(_) => {
                // Fallback: plain text for this line
                result.push(vec![(line.to_string(), Style::default())]);
            }
        }
    }

    result
}

/// Render plain text without syntax highlighting.
pub fn plain_lines(content: &str, max_lines: usize) -> Vec<Vec<StyledSpan>> {
    content
        .lines()
        .take(max_lines)
        .map(|line| vec![(line.to_string(), Style::default())])
        .collect()
}
