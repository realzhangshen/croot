use super::types::MatchMode;

/// Fuzzy match: all characters of the query appear in order in the target.
pub fn fuzzy_match(query: &str, target: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let query_lower = query.to_ascii_lowercase();
    let target_lower = target.to_ascii_lowercase();
    let mut query_chars = query_lower.chars();
    let mut current = query_chars.next();

    for ch in target_lower.chars() {
        if let Some(q) = current {
            if ch == q {
                current = query_chars.next();
            }
        } else {
            return true;
        }
    }
    current.is_none()
}

/// Regex match using a pre-compiled regex.
pub fn regex_match(re: &regex::Regex, target: &str) -> bool {
    re.is_match(target)
}

/// Exact substring match (case-insensitive).
pub fn exact_match(query: &str, target: &str) -> bool {
    target.to_lowercase().contains(&query.to_lowercase())
}

/// Dispatch matching based on mode.
pub fn do_match(
    match_mode: MatchMode,
    query: &str,
    re: Option<&regex::Regex>,
    target: &str,
) -> bool {
    match match_mode {
        MatchMode::Fuzzy => fuzzy_match(query, target),
        MatchMode::Regex => re.is_some_and(|r| regex_match(r, target)),
        MatchMode::Exact => exact_match(query, target),
    }
}

/// Fuzzy match returning byte positions of each matched character.
pub fn fuzzy_match_positions(query: &str, target: &str) -> Option<Vec<usize>> {
    if query.is_empty() {
        return Some(vec![]);
    }
    let mut positions = Vec::new();
    let mut query_chars = query.chars();
    let mut current = query_chars.next();

    for (byte_idx, ch) in target.char_indices() {
        if let Some(q) = current {
            if ch.eq_ignore_ascii_case(&q) {
                positions.push(byte_idx);
                current = query_chars.next();
            }
        } else {
            break;
        }
    }
    if current.is_none() {
        Some(positions)
    } else {
        None
    }
}

/// Exact substring match returning byte positions (char boundaries) of the matched range.
///
/// Performs the search on lowercased strings, then maps the matched character
/// range back to byte offsets in the **original** `target` string. This avoids
/// panics when `to_lowercase()` changes the byte length of characters (e.g.
/// `İ` (2 bytes) → `i̇` (3 bytes)).
pub fn exact_match_positions(query: &str, target: &str) -> Option<Vec<usize>> {
    let target_lower = target.to_lowercase();
    let query_lower = query.to_lowercase();
    let match_start = target_lower.find(&query_lower)?;

    // Find which original chars correspond to the matched range in target_lower.
    // Walk both strings char-by-char to build the byte-offset mapping.
    let mut lower_byte = 0usize;
    let mut orig_positions = Vec::new();
    for (orig_byte, orig_char) in target.char_indices() {
        let lower_char_len: usize = orig_char.to_lowercase().map(char::len_utf8).sum();
        let lower_end = lower_byte + lower_char_len;
        // This original char contributes to [lower_byte..lower_end) in target_lower.
        // If any part overlaps with the match range, include the original byte offset.
        let match_end = match_start + query_lower.len();
        if lower_end > match_start && lower_byte < match_end {
            orig_positions.push(orig_byte);
        }
        lower_byte = lower_end;
        if lower_byte >= match_start + query_lower.len() && !orig_positions.is_empty() {
            break;
        }
    }

    if orig_positions.is_empty() {
        None
    } else {
        Some(orig_positions)
    }
}

/// Regex match returning byte positions (char boundaries) of the first match span.
pub fn regex_match_positions(re: &regex::Regex, target: &str) -> Option<Vec<usize>> {
    let m = re.find(target)?;
    // Collect only char-boundary byte offsets so highlighting works with multibyte chars
    Some(
        target[m.start()..m.end()]
            .char_indices()
            .map(|(i, _)| m.start() + i)
            .collect(),
    )
}

/// Search highlight positions used by the unified workspace search UI.
///
/// Prefers a literal substring highlight first, then falls back to ripgrep-like
/// regex semantics with smart-case matching.
pub fn search_match_positions(query: &str, target: &str) -> Option<Vec<usize>> {
    if query.is_empty() {
        return Some(vec![]);
    }
    if let Some(positions) = exact_match_positions(query, target) {
        return Some(positions);
    }

    let smart_case_insensitive = !query.chars().any(char::is_uppercase);
    let regex = regex::RegexBuilder::new(query)
        .case_insensitive(smart_case_insensitive)
        .build()
        .ok()?;
    regex_match_positions(&regex, target)
}

/// Dispatch position-returning match based on mode.
pub fn do_match_positions(
    match_mode: MatchMode,
    query: &str,
    re: Option<&regex::Regex>,
    target: &str,
) -> Option<Vec<usize>> {
    match match_mode {
        MatchMode::Fuzzy => fuzzy_match_positions(query, target),
        MatchMode::Regex => re.and_then(|r| regex_match_positions(r, target)),
        MatchMode::Exact => exact_match_positions(query, target),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_empty_matches_anything() {
        assert!(fuzzy_match("", "anything"));
    }

    #[test]
    fn fuzzy_exact_match() {
        assert!(fuzzy_match("app", "app.rs"));
    }

    #[test]
    fn fuzzy_subsequence() {
        assert!(fuzzy_match("ars", "app.rs"));
    }

    #[test]
    fn fuzzy_case_insensitive() {
        assert!(fuzzy_match("APP", "app.rs"));
    }

    #[test]
    fn fuzzy_no_match() {
        assert!(!fuzzy_match("xyz", "app.rs"));
    }

    #[test]
    fn fuzzy_partial_no_match() {
        assert!(!fuzzy_match("apz", "app.rs"));
    }

    #[test]
    fn exact_match_substring() {
        assert!(exact_match("handler", "input_handler.rs"));
        assert!(exact_match("Handler", "input_handler.rs"));
        assert!(!exact_match("xyz", "input_handler.rs"));
    }

    #[test]
    fn regex_match_pattern() {
        let re = regex::Regex::new("^handler").unwrap();
        assert!(regex_match(&re, "handler.rs"));
        assert!(!regex_match(&re, "input_handler.rs"));
    }

    #[test]
    fn do_match_dispatches_correctly() {
        let re = regex::Regex::new("^app").unwrap();
        assert!(do_match(MatchMode::Fuzzy, "ars", None, "app.rs"));
        assert!(do_match(MatchMode::Regex, "^app", Some(&re), "app.rs"));
        assert!(!do_match(MatchMode::Regex, "^app", None, "app.rs"));
        assert!(do_match(MatchMode::Exact, "app", None, "app.rs"));
    }

    #[test]
    fn fuzzy_match_positions_subsequence() {
        let pos = fuzzy_match_positions("ars", "app.rs");
        assert_eq!(pos, Some(vec![0, 4, 5]));
    }

    #[test]
    fn fuzzy_match_positions_case_insensitive() {
        let pos = fuzzy_match_positions("ARS", "app.rs");
        assert_eq!(pos, Some(vec![0, 4, 5]));
    }

    #[test]
    fn fuzzy_match_positions_no_match() {
        assert_eq!(fuzzy_match_positions("xyz", "app.rs"), None);
    }

    #[test]
    fn fuzzy_match_positions_empty_query() {
        assert_eq!(fuzzy_match_positions("", "anything"), Some(vec![]));
    }

    #[test]
    fn exact_match_positions_substring() {
        let pos = exact_match_positions("handler", "input_handler.rs");
        assert_eq!(pos, Some(vec![6, 7, 8, 9, 10, 11, 12]));
    }

    #[test]
    fn exact_match_positions_case_insensitive() {
        let pos = exact_match_positions("Handler", "input_handler.rs");
        assert_eq!(pos, Some(vec![6, 7, 8, 9, 10, 11, 12]));
    }

    #[test]
    fn exact_match_positions_no_match() {
        assert_eq!(exact_match_positions("xyz", "input_handler.rs"), None);
    }

    #[test]
    fn regex_match_positions_anchored() {
        let re = regex::Regex::new("^app").unwrap();
        let pos = regex_match_positions(&re, "app.rs");
        assert_eq!(pos, Some(vec![0, 1, 2]));
    }

    #[test]
    fn regex_match_positions_no_match() {
        let re = regex::Regex::new("^handler").unwrap();
        assert_eq!(regex_match_positions(&re, "input_handler.rs"), None);
    }

    #[test]
    fn search_match_positions_prefers_exact_substring() {
        let pos = search_match_positions("main", "src/main.rs");
        assert_eq!(pos, Some(vec![4, 5, 6, 7]));
    }

    #[test]
    fn search_match_positions_falls_back_to_regex() {
        let pos = search_match_positions("^main", "main.rs");
        assert_eq!(pos, Some(vec![0, 1, 2, 3]));
    }

    #[test]
    fn do_match_positions_dispatches() {
        let re = regex::Regex::new("^app").unwrap();
        assert!(do_match_positions(MatchMode::Fuzzy, "ars", None, "app.rs").is_some());
        assert!(do_match_positions(MatchMode::Regex, "^app", Some(&re), "app.rs").is_some());
        assert!(do_match_positions(MatchMode::Regex, "^app", None, "app.rs").is_none());
        assert!(do_match_positions(MatchMode::Exact, "app", None, "app.rs").is_some());
    }

    #[test]
    fn exact_match_positions_multibyte_returns_char_boundaries() {
        let pos = exact_match_positions("fé", "café.rs");
        assert!(pos.is_some());
        let positions = pos.unwrap();
        assert_eq!(positions, vec![2, 3]);
        for &p in &positions {
            assert!(
                "café.rs".is_char_boundary(p),
                "position {p} is not a char boundary"
            );
        }
    }

    #[test]
    fn exact_match_unicode_case_folding() {
        assert!(exact_match("É", "café.rs"));
    }

    #[test]
    fn regex_match_positions_multibyte_returns_char_boundaries() {
        let re = regex::Regex::new("fé").unwrap();
        let pos = regex_match_positions(&re, "café.rs");
        assert!(pos.is_some());
        let positions = pos.unwrap();
        assert_eq!(positions, vec![2, 3]);
    }

    #[test]
    fn exact_match_positions_case_folding_byte_length_change() {
        let target = "İstanbul.txt";
        let query = "i\u{0307}stanbul";
        let pos = exact_match_positions(query, target);
        assert!(pos.is_some(), "should match case-insensitively");
        let positions = pos.unwrap();
        for &p in &positions {
            assert!(
                target.is_char_boundary(p),
                "position {p} is not a char boundary in {target:?}"
            );
        }
        assert_eq!(positions[0], 0);
        assert_eq!(positions[1], 2);
    }

    #[test]
    fn exact_match_positions_mixed_byte_length_case_fold() {
        let target = "AİB";
        let query = "i\u{0307}";
        let pos = exact_match_positions(query, target);
        assert!(pos.is_some());
        let positions = pos.unwrap();
        for &p in &positions {
            assert!(
                target.is_char_boundary(p),
                "position {p} is not a char boundary in {target:?}"
            );
        }
        assert_eq!(positions, vec![1]);
    }

    #[test]
    fn exact_match_positions_eszett() {
        let target = "straße.txt";
        let pos = exact_match_positions("straße", target);
        assert!(pos.is_some());
        let positions = pos.unwrap();
        for &p in &positions {
            assert!(
                target.is_char_boundary(p),
                "position {p} is not a char boundary in {target:?}"
            );
        }
    }
}
