use serde::Deserialize;

use super::types::{ContentMatch, FileGroup, GlobalSearchResult};

#[derive(Debug, Deserialize)]
struct RgJsonMessage {
    #[serde(rename = "type")]
    kind: String,
    data: RgJsonData,
}

#[derive(Debug, Default, Deserialize)]
struct RgJsonData {
    path: Option<RgJsonText>,
    lines: Option<RgJsonText>,
    line_number: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RgJsonText {
    Text { text: String },
    Bytes { bytes: String },
}

impl RgJsonText {
    fn into_text(self) -> Option<String> {
        match self {
            Self::Text { text } => Some(text),
            Self::Bytes { bytes } => {
                let _ = bytes;
                None
            }
        }
    }
}

/// Parsed fields from a single ripgrep `--json` `match` record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRgMatch {
    /// Path as reported by ripgrep (relative to the search root).
    pub file: String,
    /// 1-based line number of the match, if present.
    pub line_number: Option<usize>,
    /// Text of the matching line (may contain trailing newline), if present.
    pub context: Option<String>,
}

pub fn parse_rg_json_match(line: &str) -> Result<Option<ParsedRgMatch>, serde_json::Error> {
    let message: RgJsonMessage = serde_json::from_str(line)?;
    if message.kind != "match" {
        return Ok(None);
    }

    let Some(file) = message.data.path.and_then(RgJsonText::into_text) else {
        return Ok(None);
    };

    Ok(Some(ParsedRgMatch {
        file,
        line_number: message.data.line_number,
        context: message.data.lines.and_then(RgJsonText::into_text),
    }))
}

/// Group flat search results by file path into `FileGroup`s.
/// Fast path: checks the last group (rg output is typically grouped by file).
/// Falls back to full scan for non-consecutive same-path entries.
pub fn group_search_results(results: Vec<GlobalSearchResult>) -> Vec<FileGroup> {
    let mut groups: Vec<FileGroup> = Vec::new();
    for result in results {
        let m = ContentMatch {
            line: result.line,
            context: result.context,
        };
        // Fast path: last group has same path
        if let Some(last) = groups.last_mut() {
            if last.path == result.path {
                last.matches.push(m);
                continue;
            }
        }
        // Slow path: scan for existing group with this path
        if let Some(existing) = groups.iter_mut().find(|g| g.path == result.path) {
            existing.matches.push(m);
        } else {
            groups.push(FileGroup {
                path: result.path,
                display: result.display,
                matches: vec![m],
                collapsed: false,
            });
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_rg_json_match_standard_format() {
        let line = r#"{"type":"match","data":{"path":{"text":"src/main.rs"},"lines":{"text":"fn main() {\n"},"line_number":42}}"#;
        let parsed = parse_rg_json_match(line).unwrap();
        assert_eq!(
            parsed,
            Some(ParsedRgMatch {
                file: "src/main.rs".to_string(),
                line_number: Some(42),
                context: Some("fn main() {\n".to_string()),
            })
        );
    }

    #[test]
    fn parse_rg_json_match_handles_colons_in_path_and_context() {
        let line = r#"{"type":"match","data":{"path":{"text":"foo:123:bar.rs"},"lines":{"text":"prefix:456:suffix\n"},"line_number":45}}"#;
        let parsed = parse_rg_json_match(line).unwrap();
        assert_eq!(
            parsed,
            Some(ParsedRgMatch {
                file: "foo:123:bar.rs".to_string(),
                line_number: Some(45),
                context: Some("prefix:456:suffix\n".to_string()),
            })
        );
    }

    #[test]
    fn parse_rg_json_match_ignores_non_match_messages() {
        let line = r#"{"type":"begin","data":{"path":{"text":"src/main.rs"}}}"#;
        assert_eq!(parse_rg_json_match(line).unwrap(), None);
    }

    #[test]
    fn group_results_empty() {
        let groups = group_search_results(vec![]);
        assert!(groups.is_empty());
    }

    #[test]
    fn group_results_single_file_multiple_matches() {
        let results = vec![
            GlobalSearchResult {
                path: PathBuf::from("/a.rs"),
                display: "a.rs".into(),
                line: Some(10),
                context: Some("line 10".into()),
            },
            GlobalSearchResult {
                path: PathBuf::from("/a.rs"),
                display: "a.rs".into(),
                line: Some(20),
                context: Some("line 20".into()),
            },
        ];
        let groups = group_search_results(results);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].display, "a.rs");
        assert_eq!(groups[0].matches.len(), 2);
        assert_eq!(groups[0].matches[0].line, Some(10));
        assert_eq!(groups[0].matches[1].line, Some(20));
        assert!(!groups[0].collapsed);
    }

    #[test]
    fn group_results_multiple_files_preserves_order() {
        let results = vec![
            GlobalSearchResult {
                path: PathBuf::from("/b.rs"),
                display: "b.rs".into(),
                line: Some(1),
                context: None,
            },
            GlobalSearchResult {
                path: PathBuf::from("/a.rs"),
                display: "a.rs".into(),
                line: Some(5),
                context: Some("ctx".into()),
            },
        ];
        let groups = group_search_results(results);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].display, "b.rs");
        assert_eq!(groups[1].display, "a.rs");
    }

    #[test]
    fn group_results_non_consecutive_same_path_merges() {
        let results = vec![
            GlobalSearchResult {
                path: PathBuf::from("/a.rs"),
                display: "a.rs".into(),
                line: Some(1),
                context: None,
            },
            GlobalSearchResult {
                path: PathBuf::from("/b.rs"),
                display: "b.rs".into(),
                line: Some(2),
                context: None,
            },
            GlobalSearchResult {
                path: PathBuf::from("/a.rs"),
                display: "a.rs".into(),
                line: Some(3),
                context: None,
            },
        ];
        let groups = group_search_results(results);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].matches.len(), 2);
        assert_eq!(groups[0].matches[0].line, Some(1));
        assert_eq!(groups[0].matches[1].line, Some(3));
        assert_eq!(groups[1].matches.len(), 1);
    }

    #[test]
    fn group_results_optional_fields() {
        let results = vec![GlobalSearchResult {
            path: PathBuf::from("/x.rs"),
            display: "x.rs".into(),
            line: None,
            context: None,
        }];
        let groups = group_search_results(results);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].matches[0].line, None);
        assert_eq!(groups[0].matches[0].context, None);
    }
}
