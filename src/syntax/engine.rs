use std::path::Path;
use std::sync::OnceLock;

use syntect::easy::ScopeRegionIterator;
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};

use crate::preview::state::StyledSpan;

use super::scope_map::token_for_scope;
use super::semantic::SemanticToken;
use super::theme::active_theme;

fn syntax_set() -> &'static SyntaxSet {
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    SS.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// Map language tokens/extensions not present in syntect's default bundle to
/// a fallback token that IS present.  TypeScript/TSX → JavaScript is the main
/// practical case.
fn fallback_token(token: &str) -> Option<&'static str> {
    match token {
        "ts" | "tsx" | "typescript" | "TypeScript" | "TypeScriptReact" => Some("js"),
        "jsx" => Some("js"),
        "mjs" | "cjs" => Some("js"),
        _ => None,
    }
}

pub fn highlight_file(path: &Path, content: &str, max_lines: usize) -> Vec<Vec<StyledSpan>> {
    let ss = syntax_set();
    let syntax = find_syntax_for_path(ss, path);
    match syntax {
        Some(s) => highlight_with_syntax(ss, s, content, max_lines),
        None => plain_lines(content, max_lines),
    }
}

/// Look up syntax by file extension (without doing real file I/O).
/// Falls back through: extension lookup → fallback token → None.
fn find_syntax_for_path<'a>(ss: &'a SyntaxSet, path: &Path) -> Option<&'a SyntaxReference> {
    // Try by file extension first
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if let Some(syntax) = ss.find_syntax_by_extension(ext) {
            return Some(syntax);
        }
        // Extension not in bundle — try fallback mapping
        if let Some(fallback) = fallback_token(ext) {
            if let Some(syntax) = ss.find_syntax_by_token(fallback) {
                return Some(syntax);
            }
        }
    }
    // Try by full filename (e.g. "Makefile")
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if let Some(syntax) = ss.find_syntax_by_extension(name) {
            return Some(syntax);
        }
    }
    None
}

pub fn highlight_code(lang: &str, code: &str, max_lines: usize) -> Vec<Vec<StyledSpan>> {
    let ss = syntax_set();
    let syntax = ss
        .find_syntax_by_token(lang)
        .or_else(|| fallback_token(lang).and_then(|fb| ss.find_syntax_by_token(fb)));
    match syntax {
        Some(s) => highlight_with_syntax(ss, s, code, max_lines),
        None => plain_lines(code, max_lines),
    }
}

pub fn plain_lines(content: &str, max_lines: usize) -> Vec<Vec<StyledSpan>> {
    content
        .lines()
        .take(max_lines)
        .map(|line| vec![(line.to_string(), ratatui::style::Style::default())])
        .collect()
}

fn highlight_with_syntax(
    ss: &SyntaxSet,
    syntax: &SyntaxReference,
    source: &str,
    max_lines: usize,
) -> Vec<Vec<StyledSpan>> {
    let theme = active_theme();
    let default_style = theme.style_for(SemanticToken::Text);
    let mut parse_state = ParseState::new(syntax);
    let mut scope_stack = ScopeStack::new();
    let mut lines: Vec<Vec<StyledSpan>> = Vec::with_capacity(max_lines.min(128));

    for line in source.lines() {
        if lines.len() >= max_lines {
            break;
        }

        // PERF: allocates per line; could reuse a buffer or work with byte slices
        let line_with_nl = format!("{}\n", line);
        let ops = match parse_state.parse_line(&line_with_nl, ss) {
            Ok(ops) => ops,
            Err(_) => {
                lines.push(vec![(line.to_string(), default_style)]);
                continue;
            }
        };

        let mut current_line: Vec<StyledSpan> = Vec::new();

        for (token_text, op) in ScopeRegionIterator::new(&ops, &line_with_nl) {
            let _ = scope_stack.apply(op);

            if token_text.is_empty() {
                continue;
            }

            // Strip the trailing newline we added — we handle line breaks ourselves
            let text = if token_text.ends_with('\n') {
                &token_text[..token_text.len() - 1]
            } else {
                token_text
            };

            if text.is_empty() {
                continue;
            }

            // PERF: build_string() allocates per token; could cache Scope→SemanticToken
            let top_scope = scope_stack
                .as_slice()
                .last()
                .map(|s| s.build_string())
                .unwrap_or_default();

            let token = token_for_scope(&top_scope);
            let style = theme.style_for(token);
            current_line.push((text.to_string(), style));
        }

        lines.push(current_line);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_rust_with_ansi_styles() {
        let lines = highlight_code("rs", "fn main() { let x = 42; }", 100);
        assert!(!lines.is_empty());
        assert!(
            lines.iter().flatten().any(|(_, style)| style.fg.is_some()),
            "rust should produce styled spans"
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
    fn highlights_typescript_with_ansi_styles() {
        let lines = highlight_code("ts", "type User = string;", 100);
        assert!(!lines.is_empty());
        assert!(
            lines.iter().flatten().any(|(_, style)| style.fg.is_some()),
            "typescript should produce styled spans"
        );
    }

    #[test]
    fn highlights_python() {
        let lines = highlight_code("py", "def hello():\n    print('hi')\n", 100);
        assert!(!lines.is_empty());
        assert!(
            lines.iter().flatten().any(|(_, style)| style.fg.is_some()),
            "python should produce styled spans"
        );
    }

    #[test]
    fn highlights_go() {
        let lines = highlight_code("go", "func main() { fmt.Println(\"hi\") }", 100);
        assert!(!lines.is_empty());
        assert!(
            lines.iter().flatten().any(|(_, style)| style.fg.is_some()),
            "go should produce styled spans"
        );
    }

    #[test]
    fn highlights_c() {
        let lines = highlight_code("c", "#include <stdio.h>\nint main() { return 0; }", 100);
        assert!(!lines.is_empty());
        assert!(
            lines.iter().flatten().any(|(_, style)| style.fg.is_some()),
            "c should produce styled spans"
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

    #[test]
    fn highlight_file_detects_language_by_extension() {
        let path = std::path::PathBuf::from("example.py");
        let lines = highlight_file(&path, "x = 42", 100);
        assert!(
            lines.iter().flatten().any(|(_, style)| style.fg.is_some()),
            ".py file should be highlighted"
        );
    }

    #[test]
    fn highlight_file_detects_makefile_by_name() {
        let path = std::path::PathBuf::from("Makefile");
        let lines = highlight_file(&path, "all:\n\techo hello", 100);
        assert!(
            lines.iter().flatten().any(|(_, style)| style.fg.is_some()),
            "Makefile should be highlighted by filename"
        );
    }

    #[test]
    fn highlight_file_unknown_ext_is_plain() {
        let path = std::path::PathBuf::from("data.unknownext12345");
        let lines = highlight_file(&path, "hello world", 100);
        assert_eq!(
            lines,
            vec![vec![(
                "hello world".to_string(),
                ratatui::style::Style::default()
            )]]
        );
    }

    #[test]
    fn max_lines_is_respected() {
        let code = "a\nb\nc\nd\ne\nf\n";
        let lines = highlight_code("rs", code, 3);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn multiple_styles_in_rust_code() {
        let lines = highlight_code("rs", r#"let s = "hello\n";"#, 100);
        let styles: std::collections::HashSet<_> =
            lines.iter().flatten().map(|(_, style)| *style).collect();
        assert!(
            styles.len() >= 2,
            "Rust code should produce at least 2 distinct styles, got {}",
            styles.len()
        );
    }
}
