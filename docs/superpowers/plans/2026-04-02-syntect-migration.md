# Syntect Migration: Replace tree-sitter with syntect

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace tree-sitter with syntect as the syntax highlighting parser, mapping TextMate scopes to existing SemanticToken ANSI colors, expanding language support from 6 to 200+.

**Architecture:** Use syntect's `ParseState` + `ScopeRegionIterator` to tokenize source code into scope stacks. Map the top-level scope prefix (e.g. `keyword.*`) to existing `SemanticToken` enum values. Keep `SemanticToken`, `SyntaxTheme`, and ANSI color mapping unchanged. The public API (`highlight_file`, `highlight_code`, `plain_lines`) stays identical.

**Tech Stack:** `syntect` 5.3 with `default-fancy` features (pure Rust, no C deps), replacing 5 tree-sitter crates.

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `Cargo.toml` | Modify | Remove tree-sitter deps, add syntect |
| `src/syntax/engine.rs` | Rewrite | New syntect-based highlighting engine |
| `src/syntax/scope_map.rs` | Create | TextMate scope prefix → SemanticToken mapping (replaces `capture_map.rs`) |
| `src/syntax/capture_map.rs` | Delete | No longer needed |
| `src/syntax/languages.rs` | Delete | syntect handles language detection internally |
| `src/syntax/mod.rs` | Modify | Update module declarations |
| `src/syntax/semantic.rs` | Keep | No changes |
| `src/syntax/theme.rs` | Keep | No changes |
| `src/preview/highlight.rs` | Keep | No changes (public API unchanged) |

---

### Task 1: Add syntect dependency and remove tree-sitter

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Update Cargo.toml**

Replace the tree-sitter dependencies with syntect. In `Cargo.toml`, remove these lines:

```
tree-sitter = "0.26.8"
tree-sitter-highlight = "0.26.8"
tree-sitter-language = "0.1.7"
tree-sitter-rust = "0.24.2"
tree-sitter-typescript = "0.23.2"
tree-sitter-javascript = "0.25.0"
tree-sitter-json = "0.24.8"
tree-sitter-md = "0.5.3"
```

And add:

```
syntect = { version = "5.3", default-features = false, features = ["parsing", "default-syntaxes", "regex-fancy", "dump-load"] }
```

- [ ] **Step 2: Verify it compiles (expect errors)**

Run: `cargo check 2>&1 | head -5`

Expected: Compilation errors in `src/syntax/engine.rs`, `src/syntax/capture_map.rs`, `src/syntax/languages.rs` because they import tree-sitter. This is correct -- we'll replace them next.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "build: replace tree-sitter deps with syntect"
```

---

### Task 2: Create scope_map.rs (TextMate scope → SemanticToken)

**Files:**
- Create: `src/syntax/scope_map.rs`
- Test: `src/syntax/scope_map.rs` (inline `#[cfg(test)] mod tests`)

- [ ] **Step 1: Write the failing tests**

Create `src/syntax/scope_map.rs` with the test module first:

```rust
use super::semantic::SemanticToken;

/// Maps a TextMate scope name to a SemanticToken using longest-prefix matching.
pub fn token_for_scope(scope: &str) -> SemanticToken {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_map_correctly() {
        assert_eq!(token_for_scope("keyword.control.rust"), SemanticToken::Keyword);
        assert_eq!(token_for_scope("keyword.operator"), SemanticToken::Operator);
        assert_eq!(token_for_scope("keyword.other.fn.rust"), SemanticToken::Keyword);
        assert_eq!(token_for_scope("keyword"), SemanticToken::Keyword);
    }

    #[test]
    fn storage_maps_to_keyword() {
        assert_eq!(token_for_scope("storage.type"), SemanticToken::Keyword);
        assert_eq!(token_for_scope("storage.modifier"), SemanticToken::Keyword);
    }

    #[test]
    fn strings_map_correctly() {
        assert_eq!(token_for_scope("string.quoted.double"), SemanticToken::String);
        assert_eq!(token_for_scope("string.quoted.single"), SemanticToken::String);
        assert_eq!(token_for_scope("string.regexp"), SemanticToken::String);
    }

    #[test]
    fn constants_map_correctly() {
        assert_eq!(token_for_scope("constant.numeric.integer"), SemanticToken::Number);
        assert_eq!(token_for_scope("constant.numeric.float"), SemanticToken::Number);
        assert_eq!(token_for_scope("constant.numeric"), SemanticToken::Number);
        assert_eq!(token_for_scope("constant.character.escape"), SemanticToken::Escape);
        assert_eq!(token_for_scope("constant.language"), SemanticToken::Constant);
        assert_eq!(token_for_scope("constant.other"), SemanticToken::Constant);
    }

    #[test]
    fn entity_names_map_correctly() {
        assert_eq!(token_for_scope("entity.name.function"), SemanticToken::Function);
        assert_eq!(token_for_scope("entity.name.function.rust"), SemanticToken::Function);
        assert_eq!(token_for_scope("entity.name.type"), SemanticToken::Type);
        assert_eq!(token_for_scope("entity.name.type.class"), SemanticToken::Type);
        assert_eq!(token_for_scope("entity.name.tag"), SemanticToken::Tag);
        assert_eq!(token_for_scope("entity.name.tag.html"), SemanticToken::Tag);
        assert_eq!(token_for_scope("entity.name.section"), SemanticToken::Tag);
        assert_eq!(token_for_scope("entity.other.attribute-name"), SemanticToken::Attribute);
    }

    #[test]
    fn comments_map_correctly() {
        assert_eq!(token_for_scope("comment.line"), SemanticToken::Comment);
        assert_eq!(token_for_scope("comment.block"), SemanticToken::Comment);
        assert_eq!(token_for_scope("comment.block.documentation"), SemanticToken::Comment);
    }

    #[test]
    fn variables_map_correctly() {
        assert_eq!(token_for_scope("variable.other"), SemanticToken::Variable);
        assert_eq!(token_for_scope("variable.parameter"), SemanticToken::Parameter);
        assert_eq!(token_for_scope("variable.language"), SemanticToken::Variable);
        assert_eq!(token_for_scope("variable.function"), SemanticToken::Function);
    }

    #[test]
    fn support_maps_correctly() {
        assert_eq!(token_for_scope("support.function"), SemanticToken::Function);
        assert_eq!(token_for_scope("support.type"), SemanticToken::TypeBuiltin);
        assert_eq!(token_for_scope("support.class"), SemanticToken::TypeBuiltin);
        assert_eq!(token_for_scope("support.constant"), SemanticToken::Constant);
        assert_eq!(token_for_scope("support.module"), SemanticToken::Module);
    }

    #[test]
    fn punctuation_maps_correctly() {
        assert_eq!(token_for_scope("punctuation.definition.string"), SemanticToken::String);
        assert_eq!(token_for_scope("punctuation.separator"), SemanticToken::Punctuation);
        assert_eq!(token_for_scope("punctuation.section"), SemanticToken::Punctuation);
        assert_eq!(token_for_scope("punctuation.terminator"), SemanticToken::Punctuation);
        assert_eq!(token_for_scope("punctuation.accessor"), SemanticToken::Punctuation);
        assert_eq!(token_for_scope("punctuation.definition.comment"), SemanticToken::Comment);
        assert_eq!(token_for_scope("punctuation.definition.tag"), SemanticToken::Tag);
    }

    #[test]
    fn markup_maps_correctly() {
        assert_eq!(token_for_scope("markup.heading"), SemanticToken::Tag);
        assert_eq!(token_for_scope("markup.bold"), SemanticToken::Tag);
        assert_eq!(token_for_scope("markup.italic"), SemanticToken::Tag);
        assert_eq!(token_for_scope("markup.raw"), SemanticToken::String);
        assert_eq!(token_for_scope("markup.raw.inline"), SemanticToken::String);
    }

    #[test]
    fn meta_scopes_fall_through_to_text() {
        assert_eq!(token_for_scope("meta.function"), SemanticToken::Text);
        assert_eq!(token_for_scope("meta.block"), SemanticToken::Text);
    }

    #[test]
    fn unknown_scope_returns_text() {
        assert_eq!(token_for_scope("totally.unknown.scope"), SemanticToken::Text);
        assert_eq!(token_for_scope(""), SemanticToken::Text);
    }

    #[test]
    fn keyword_operator_maps_to_operator_not_keyword() {
        // More specific prefix should win
        assert_eq!(token_for_scope("keyword.operator.assignment"), SemanticToken::Operator);
        assert_eq!(token_for_scope("keyword.operator"), SemanticToken::Operator);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p croot scope_map -- 2>&1 | tail -3`

Expected: FAIL -- `todo!()` panics.

- [ ] **Step 3: Implement token_for_scope**

Replace the `todo!()` in `token_for_scope` with:

```rust
use super::semantic::SemanticToken;

/// Ordered scope-prefix rules. Longer (more specific) prefixes come first
/// so the first match wins.
const SCOPE_RULES: &[(&str, SemanticToken)] = &[
    // ── keyword (specific before general) ────────────────
    ("keyword.operator", SemanticToken::Operator),
    ("keyword", SemanticToken::Keyword),
    // ── storage → Keyword ────────────────────────────────
    ("storage", SemanticToken::Keyword),
    // ── constant (specific before general) ───────────────
    ("constant.numeric", SemanticToken::Number),
    ("constant.character.escape", SemanticToken::Escape),
    ("constant.language", SemanticToken::Constant),
    ("constant", SemanticToken::Constant),
    // ── string ───────────────────────────────────────────
    ("string", SemanticToken::String),
    // ── comment ──────────────────────────────────────────
    ("comment", SemanticToken::Comment),
    // ── entity.name (specific before general) ────────────
    ("entity.name.function", SemanticToken::Function),
    ("entity.name.type", SemanticToken::Type),
    ("entity.name.tag", SemanticToken::Tag),
    ("entity.name.section", SemanticToken::Tag),
    ("entity.other.attribute-name", SemanticToken::Attribute),
    ("entity.other.inherited-class", SemanticToken::Type),
    ("entity.name", SemanticToken::Variable),
    // ── variable (specific before general) ───────────────
    ("variable.parameter", SemanticToken::Parameter),
    ("variable.function", SemanticToken::Function),
    ("variable", SemanticToken::Variable),
    // ── support (specific before general) ────────────────
    ("support.function", SemanticToken::Function),
    ("support.type", SemanticToken::TypeBuiltin),
    ("support.class", SemanticToken::TypeBuiltin),
    ("support.constant", SemanticToken::Constant),
    ("support.module", SemanticToken::Module),
    ("support", SemanticToken::Variable),
    // ── punctuation (specific before general) ────────────
    ("punctuation.definition.comment", SemanticToken::Comment),
    ("punctuation.definition.string", SemanticToken::String),
    ("punctuation.definition.tag", SemanticToken::Tag),
    ("punctuation", SemanticToken::Punctuation),
    // ── markup ───────────────────────────────────────────
    ("markup.raw", SemanticToken::String),
    ("markup", SemanticToken::Tag),
    // ── meta → transparent (Text) ────────────────────────
    ("meta", SemanticToken::Text),
    // ── source → transparent ─────────────────────────────
    ("source", SemanticToken::Text),
];

/// Maps a TextMate scope name to a SemanticToken using longest-prefix matching.
///
/// Scopes like `keyword.control.rust` are matched against the prefix table
/// from most-specific to least-specific. Unknown scopes return `Text`.
pub fn token_for_scope(scope: &str) -> SemanticToken {
    if scope.is_empty() {
        return SemanticToken::Text;
    }
    for &(prefix, token) in SCOPE_RULES {
        if scope == prefix || scope.starts_with(prefix) && scope.as_bytes().get(prefix.len()) == Some(&b'.') {
            return token;
        }
    }
    SemanticToken::Text
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p croot scope_map -- 2>&1 | tail -5`

Expected: All tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/syntax/scope_map.rs
git commit -m "feat(syntax): add scope_map for TextMate scope to SemanticToken mapping"
```

---

### Task 3: Rewrite engine.rs to use syntect

**Files:**
- Rewrite: `src/syntax/engine.rs`

- [ ] **Step 1: Write the failing tests**

Replace the entire contents of `src/syntax/engine.rs` with:

```rust
use std::path::Path;
use std::sync::OnceLock;

use syntect::easy::ScopeRegionIterator;
use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};

use crate::preview::state::StyledSpan;

use super::scope_map::token_for_scope;
use super::semantic::SemanticToken;
use super::theme::active_theme;

fn syntax_set() -> &'static SyntaxSet {
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    SS.get_or_init(SyntaxSet::load_defaults_newlines)
}

pub fn highlight_file(path: &Path, content: &str, max_lines: usize) -> Vec<Vec<StyledSpan>> {
    let ss = syntax_set();
    let syntax = match ss.find_syntax_for_file(path) {
        Ok(Some(s)) => s,
        _ => return plain_lines(content, max_lines),
    };
    highlight_with_syntax(ss, syntax, content, max_lines)
}

pub fn highlight_code(lang: &str, code: &str, max_lines: usize) -> Vec<Vec<StyledSpan>> {
    let ss = syntax_set();
    let syntax = match ss.find_syntax_by_token(lang) {
        Some(s) => s,
        None => return plain_lines(code, max_lines),
    };
    highlight_with_syntax(ss, syntax, code, max_lines)
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
    syntax: &syntect::parsing::SyntaxReference,
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
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test -p croot engine:: -- 2>&1 | tail -10`

Expected: All tests PASS. If any fail, fix the scope mapping.

- [ ] **Step 3: Commit**

```bash
git add src/syntax/engine.rs
git commit -m "feat(syntax): rewrite engine to use syntect parser"
```

---

### Task 4: Update mod.rs and delete old files

**Files:**
- Modify: `src/syntax/mod.rs`
- Delete: `src/syntax/capture_map.rs`
- Delete: `src/syntax/languages.rs`

- [ ] **Step 1: Update mod.rs**

Replace contents of `src/syntax/mod.rs` with:

```rust
pub mod engine;
pub mod scope_map;
pub mod semantic;
pub mod theme;
```

- [ ] **Step 2: Delete old files**

```bash
rm src/syntax/capture_map.rs src/syntax/languages.rs
```

- [ ] **Step 3: Run full test suite**

Run: `cargo test 2>&1 | tail -5`

Expected: All tests pass. There should be no remaining references to `capture_map` or `languages` outside the deleted files (verified earlier: only `engine.rs` imports them, and it's been rewritten).

- [ ] **Step 4: Commit**

```bash
git add src/syntax/mod.rs
git add -u src/syntax/capture_map.rs src/syntax/languages.rs
git commit -m "refactor(syntax): remove tree-sitter modules, register scope_map"
```

---

### Task 5: Update highlight.rs tests for expanded language support

**Files:**
- Modify: `src/preview/highlight.rs`

- [ ] **Step 1: Update tests to cover new languages**

In `src/preview/highlight.rs`, replace the `supported_extensions_are_highlighted` and `supported_tokens_are_highlighted` tests with expanded versions:

```rust
    #[test]
    fn supported_extensions_are_highlighted() {
        let cases = [
            ("main.rs", "fn main() { println!(\"hello\"); }"),
            ("app.tsx", "export const App = () => <div />;"),
            ("script.js", "function greet(name) { return name; }"),
            ("data.json", "{\"key\": true, \"count\": 3}"),
            ("main.py", "def hello():\n    print('hi')"),
            ("main.go", "func main() { fmt.Println(\"hi\") }"),
            ("main.c", "int main() { return 0; }"),
            ("main.cpp", "int main() { return 0; }"),
            ("Main.java", "class Main { public static void main(String[] a) {} }"),
            ("style.css", "body { color: red; }"),
            ("page.html", "<html><body>hello</body></html>"),
            ("config.yaml", "key: value\nlist:\n  - item"),
            ("config.toml", "[section]\nkey = \"value\""),
            ("script.sh", "#!/bin/bash\necho hello"),
            ("query.sql", "SELECT * FROM users WHERE id = 1;"),
        ];
        for (filename, content) in cases {
            let path = PathBuf::from(filename);
            let result = highlight_file(&path, content, 100);
            assert!(
                has_highlighting(&result),
                "{filename} should be syntax-highlighted, got plain text"
            );
        }
    }

    #[test]
    fn supported_tokens_are_highlighted() {
        let cases = [
            ("rs", "fn main() { println!(\"hello\"); }"),
            ("ts", "type User = string;"),
            ("tsx", "export const App = () => <div />;"),
            ("javascript", "const x = 1;"),
            ("json", "{\"ok\": true}"),
            ("md", "# Heading"),
            ("py", "def hello(): pass"),
            ("go", "func main() {}"),
            ("c", "int main() { return 0; }"),
            ("java", "class Main {}"),
            ("css", "body { color: red; }"),
            ("html", "<html></html>"),
            ("yaml", "key: value"),
            ("sql", "SELECT 1;"),
            ("sh", "echo hello"),
        ];
        for (token, code) in cases {
            let result = highlight_code(token, code, 100);
            assert!(
                has_highlighting(&result),
                "token '{token}' should be syntax-highlighted, got plain text"
            );
        }
    }
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p croot highlight:: -- 2>&1 | tail -10`

Expected: All tests pass. If a language token doesn't work (e.g. `"sh"` vs `"bash"`), adjust the token string to match what `SyntaxSet::find_syntax_by_token` expects.

- [ ] **Step 3: Commit**

```bash
git add src/preview/highlight.rs
git commit -m "test(syntax): expand highlight tests to cover 15 languages"
```

---

### Task 6: Run full test suite and verify binary size

- [ ] **Step 1: Run full test suite**

Run: `cargo test 2>&1 | tail -20`

Expected: All tests pass.

- [ ] **Step 2: Check binary size change**

```bash
cargo build --release 2>&1 | tail -3
ls -lh target/release/croot
```

Note the size. The syntect binary dump (~2MB) replaces 5 tree-sitter grammars. Size should be comparable or smaller.

- [ ] **Step 3: Smoke test with real files**

```bash
cargo run -- --help
```

Verify the binary works.

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat(syntax): complete syntect migration, 200+ languages supported"
```

---

## Notes

- `SyntaxSet::load_defaults_newlines()` is lazy-initialized via `OnceLock`, so first highlight pays a one-time cost (~5ms) to deserialize the syntax dump.
- `Scope::build_string()` locks a global mutex. In the hot loop this is acceptable because we only call it once per token. If profiling shows this as a bottleneck, we can cache scope→token lookups with a `HashMap<Scope, SemanticToken>`.
- syntect's `find_syntax_by_token` tries extension first, then case-insensitive name. This means `"rust"`, `"rs"`, `"Rust"` all work for Markdown fenced code blocks.
- The `default-fancy` feature uses pure-Rust regex (`fancy-regex`), avoiding C build dependencies from Oniguruma.
