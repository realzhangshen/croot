use std::path::Path;

use tree_sitter_highlight::{HighlightEvent, Highlighter};

use crate::preview::state::StyledSpan;

use super::capture_map::token_for_highlight;
use super::languages::{find_by_path, find_by_token};
use super::theme::active_theme;

pub fn highlight_file(path: &Path, content: &str, max_lines: usize) -> Vec<Vec<StyledSpan>> {
    let Some(definition) = find_by_path(path) else {
        return plain_lines(content, max_lines);
    };
    highlight_with_config((definition.config)(), content, max_lines)
}

pub fn highlight_code(lang: &str, code: &str, max_lines: usize) -> Vec<Vec<StyledSpan>> {
    let Some(definition) = find_by_token(lang) else {
        return plain_lines(code, max_lines);
    };
    highlight_with_config((definition.config)(), code, max_lines)
}

pub fn plain_lines(content: &str, max_lines: usize) -> Vec<Vec<StyledSpan>> {
    content
        .lines()
        .take(max_lines)
        .map(|line| vec![(line.to_string(), ratatui::style::Style::default())])
        .collect()
}

fn highlight_with_config(
    config: &tree_sitter_highlight::HighlightConfiguration,
    source: &str,
    max_lines: usize,
) -> Vec<Vec<StyledSpan>> {
    let mut highlighter = Highlighter::new();
    let Ok(events) = highlighter.highlight(config, source.as_bytes(), None, |_| None) else {
        return plain_lines(source, max_lines);
    };

    let theme = active_theme();
    let mut lines: Vec<Vec<StyledSpan>> = Vec::with_capacity(max_lines.min(128));
    let mut current_line: Vec<StyledSpan> = Vec::new();
    let mut style_stack = vec![theme.style_for(super::semantic::SemanticToken::Text)];

    for event in events {
        let Ok(event) = event else {
            return plain_lines(source, max_lines);
        };

        match event {
            HighlightEvent::Source { start, end } => {
                if lines.len() >= max_lines {
                    break;
                }
                append_source_segment(
                    &source[start..end],
                    *style_stack
                        .last()
                        .expect("style stack always contains text"),
                    &mut lines,
                    &mut current_line,
                    max_lines,
                );
                if lines.len() >= max_lines {
                    break;
                }
            }
            HighlightEvent::HighlightStart(highlight) => {
                let token = token_for_highlight(highlight);
                style_stack.push(theme.style_for(token));
            }
            HighlightEvent::HighlightEnd => {
                if style_stack.len() > 1 {
                    style_stack.pop();
                }
            }
        }
    }

    if lines.len() < max_lines
        && (!current_line.is_empty() || source.ends_with('\n') && source == "\n")
    {
        lines.push(current_line);
    }

    lines
}

fn append_source_segment(
    segment: &str,
    style: ratatui::style::Style,
    lines: &mut Vec<Vec<StyledSpan>>,
    current_line: &mut Vec<StyledSpan>,
    max_lines: usize,
) {
    let mut start = 0;

    for (idx, ch) in segment.char_indices() {
        if ch != '\n' {
            continue;
        }

        if idx > start {
            current_line.push((segment[start..idx].to_string(), style));
        }

        if lines.len() < max_lines {
            lines.push(std::mem::take(current_line));
        }
        if lines.len() >= max_lines {
            return;
        }

        start = idx + ch.len_utf8();
    }

    if start < segment.len() {
        current_line.push((segment[start..].to_string(), style));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_typescript_with_ansi_styles() {
        let lines = highlight_code("ts", "type User = string;", 100);
        assert!(!lines.is_empty());
        assert!(
            lines.iter().flatten().any(|(_, style)| style.fg.is_some()),
            "typescript should produce styled spans"
        );
        assert!(
            lines
                .iter()
                .flatten()
                .all(|(_, style)| !matches!(style.fg, Some(ratatui::style::Color::Rgb(..)))),
            "syntax engine should only emit ANSI/indexed/reset colors"
        );
    }

    #[test]
    fn unknown_language_falls_back_to_plain_lines() {
        let lines = highlight_code("unknownlang", "hello", 100);
        assert_eq!(
            lines,
            vec![vec![(
                "hello".to_string(),
                ratatui::style::Style::default()
            )]]
        );
    }
}
