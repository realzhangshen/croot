use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use syntect::easy::ScopeRegionIterator;
use syntect::parsing::{ParseState, Scope, ScopeStack, SyntaxReference, SyntaxSet};

use crate::preview::state::StyledSpan;

/// Lines longer than this (in bytes) are parsed but not styled, to prevent
/// pathological highlighting on minified files. The parse state is still
/// maintained so subsequent lines remain correct.
const LONG_LINE_BYTES: usize = 8192;

use super::scope_map::token_for_scope;
use super::semantic::SemanticToken;
use super::theme::active_theme;

/// The active syntax set.
///
/// We use [`two_face::syntax::extra_newlines`] instead of syntect's bundled
/// `load_defaults_newlines` because the default bundle is missing many common
/// languages — Swift, TOML, Kotlin, Dockerfile, INI, Nix, Dart, Zig,
/// TypeScript, SCSS, GraphQL, Terraform, etc. The two-face crate (maintained
/// alongside `bat`) ships a combined `SyntaxSet` containing the syntect
/// defaults plus those extras, compatible with the same fancy-regex backend
/// we already use.
fn syntax_set() -> &'static SyntaxSet {
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    SS.get_or_init(two_face::syntax::extra_newlines)
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
    let mut line_buf = String::with_capacity(256);
    let mut scope_cache: HashMap<Scope, SemanticToken> = HashMap::new();

    for line in source.lines() {
        if lines.len() >= max_lines {
            break;
        }

        // Reuse buffer to avoid per-line allocation (syntect requires trailing \n)
        line_buf.clear();
        line_buf.push_str(line);
        line_buf.push('\n');

        let ops = match parse_state.parse_line(&line_buf, ss) {
            Ok(ops) => ops,
            Err(_) => {
                lines.push(vec![(line.to_string(), default_style)]);
                continue;
            }
        };

        // Long lines: still iterate ops to maintain scope_stack state,
        // but skip the expensive build_string()/styling per token.
        if line.len() > LONG_LINE_BYTES {
            for (_, op) in ScopeRegionIterator::new(&ops, &line_buf) {
                let _ = scope_stack.apply(op);
            }
            lines.push(vec![(line.to_string(), default_style)]);
            continue;
        }

        let mut current_line: Vec<StyledSpan> = Vec::new();

        for (token_text, op) in ScopeRegionIterator::new(&ops, &line_buf) {
            let _ = scope_stack.apply(op);

            if token_text.is_empty() {
                continue;
            }

            // Strip the trailing newline we added — we handle line breaks ourselves
            let text = if let Some(stripped) = token_text.strip_suffix('\n') {
                stripped
            } else {
                token_text
            };

            if text.is_empty() {
                continue;
            }

            // Cache Scope→SemanticToken to avoid repeated build_string()
            // (which locks a global mutex and allocates a String each call).
            let token = match scope_stack.as_slice().last() {
                Some(&scope) => *scope_cache.entry(scope).or_insert_with(|| {
                    let s = scope.build_string();
                    token_for_scope(&s)
                }),
                None => SemanticToken::Text,
            };

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

    /// Helper: assert that the given (lang token, sample code) gets at least
    /// one styled span — i.e. the syntax is recognised and produces colours.
    fn assert_highlighted(lang: &str, code: &str) {
        let lines = highlight_code(lang, code, 100);
        assert!(!lines.is_empty(), "{lang}: produced no lines");
        assert!(
            lines.iter().flatten().any(|(_, style)| style.fg.is_some()),
            "{lang}: expected styled spans, got plain text — syntax not recognised"
        );
    }

    #[test]
    fn highlights_swift() {
        assert_highlighted(
            "swift",
            "import Foundation\nfunc greet(name: String) -> String { return \"hi \\(name)\" }",
        );
    }

    #[test]
    fn highlights_toml() {
        assert_highlighted("toml", "[package]\nname = \"croot\"\nversion = \"0.5.5\"\n");
    }

    #[test]
    fn highlights_kotlin() {
        assert_highlighted("kt", "fun main() { println(\"hello\") }");
    }

    #[test]
    fn highlights_dockerfile() {
        let path = std::path::PathBuf::from("Dockerfile");
        let lines = highlight_file(&path, "FROM rust:1.90\nRUN cargo build\n", 100);
        assert!(
            lines.iter().flatten().any(|(_, style)| style.fg.is_some()),
            "Dockerfile should be highlighted by filename"
        );
    }

    #[test]
    fn highlights_ini() {
        assert_highlighted("ini", "[section]\nkey = value\n");
    }

    #[test]
    fn highlights_nix() {
        assert_highlighted("nix", "{ pkgs ? import <nixpkgs> {} }: pkgs.hello");
    }

    #[test]
    fn highlights_dart() {
        assert_highlighted("dart", "void main() { print('hi'); }");
    }

    #[test]
    fn highlights_zig() {
        assert_highlighted(
            "zig",
            "const std = @import(\"std\");\npub fn main() void {}",
        );
    }

    #[test]
    fn highlights_typescript_natively() {
        // With two-face, .ts has a real TypeScript syntax (not the JS fallback).
        let path = std::path::PathBuf::from("foo.ts");
        let lines = highlight_file(&path, "type User = { name: string };", 100);
        assert!(
            lines.iter().flatten().any(|(_, style)| style.fg.is_some()),
            ".ts file should be highlighted"
        );
    }

    #[test]
    fn highlight_file_detects_cargo_toml() {
        let path = std::path::PathBuf::from("Cargo.toml");
        let lines = highlight_file(
            &path,
            "[package]\nname = \"croot\"\nversion = \"0.5.5\"\n",
            100,
        );
        assert!(
            lines.iter().flatten().any(|(_, style)| style.fg.is_some()),
            "Cargo.toml should be highlighted"
        );
    }

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

    #[test]
    fn long_line_gets_default_style() {
        // A line exceeding LONG_LINE_BYTES should be rendered as a single span
        // with the theme's default text style, even if it contains code that
        // would normally produce multiple styled spans.
        let chunk = "let x = 42; ";
        let repeats = (LONG_LINE_BYTES / chunk.len()) + 1;
        let long_line = chunk.repeat(repeats);
        assert!(long_line.len() > LONG_LINE_BYTES);

        let code = format!("fn main() {{\n{long_line}\n}}");
        let lines = highlight_code("rs", &code, 100);
        assert_eq!(lines.len(), 3);

        // The long line (index 1) should be a single span with the default
        // text style — not multiple spans with keyword/number/operator colors.
        let long_spans = &lines[1];
        assert_eq!(
            long_spans.len(),
            1,
            "long line should be a single unstyled span"
        );
        let default_text_style = active_theme().style_for(SemanticToken::Text);
        assert_eq!(
            long_spans[0].1, default_text_style,
            "long line should have default text style"
        );
    }

    #[test]
    fn highlighting_correct_after_long_line() {
        // After a long line, parse state must still be valid so subsequent lines
        // are highlighted correctly (not broken by skipped styling).
        let chunk = "let x = 42; ";
        let repeats = (LONG_LINE_BYTES / chunk.len()) + 1;
        let long_line = chunk.repeat(repeats);

        let code = format!("fn main() {{\n{long_line}\nlet y = 42;\n}}");
        let lines = highlight_code("rs", &code, 100);
        assert_eq!(lines.len(), 4);

        // Line after the long line (index 2: "let y = 42;") should still be styled
        let after_spans = &lines[2];
        assert!(
            after_spans.iter().any(|(_, style)| style.fg.is_some()),
            "line after a long line should still be highlighted, got: {:?}",
            after_spans
        );
    }
}
