use std::path::Path;

use crate::syntax::engine;

use super::state::StyledSpan;

/// Highlight file content with syntax coloring.
///
/// Returns pre-styled lines ready for rendering.
/// `max_lines` caps how many lines we process (performance guard).
pub fn highlight_file(path: &Path, content: &str, max_lines: usize) -> Vec<Vec<StyledSpan>> {
    engine::highlight_file(path, content, max_lines)
}

/// Highlight a code snippet by language name (for use in Markdown fenced code blocks).
///
/// Falls back to plain text if the language is not recognized.
pub fn highlight_code(lang: &str, code: &str, max_lines: usize) -> Vec<Vec<StyledSpan>> {
    engine::highlight_code(lang, code, max_lines)
}

/// Render plain text without syntax highlighting.
pub fn plain_lines(content: &str, max_lines: usize) -> Vec<Vec<StyledSpan>> {
    engine::plain_lines(content, max_lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;
    use std::path::PathBuf;

    /// Helper: returns true if the result contains any non-default styling.
    fn has_highlighting(lines: &[Vec<StyledSpan>]) -> bool {
        lines
            .iter()
            .flatten()
            .any(|(_, style)| *style != Style::default())
    }

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
            (
                "Main.java",
                "class Main { public static void main(String[] a) {} }",
            ),
            ("style.css", "body { color: red; }"),
            ("page.html", "<html><body>hello</body></html>"),
            ("config.yaml", "key: value\nlist:\n  - item"),
            ("config.xml", "<config><key>value</key></config>"),
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

    #[test]
    fn unknown_extension_falls_back_to_plain_text() {
        let path = PathBuf::from("data.unknownext12345");
        let result = highlight_file(&path, "hello world", 100);
        assert!(
            !has_highlighting(&result),
            "unknown extension should produce plain text"
        );
    }

    #[test]
    fn unknown_token_falls_back_to_plain_text() {
        let result = highlight_code("unknownlang12345", "hello world", 100);
        assert_eq!(
            result,
            vec![vec![("hello world".to_string(), Style::default())]]
        );
    }
}
