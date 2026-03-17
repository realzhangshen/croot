use super::node::TreeNode;

/// Sort nodes: optionally directories first, then natural (case-insensitive) sort by name.
pub fn sort_nodes(nodes: &mut [TreeNode], dirs_first: bool) {
    nodes.sort_by(|a, b| {
        if dirs_first {
            let dir_ord = b.is_dir().cmp(&a.is_dir());
            if dir_ord != std::cmp::Ordering::Equal {
                return dir_ord;
            }
        }
        // Case-insensitive natural sort
        natural_cmp(&a.name, &b.name)
    });
}

/// Simple natural sort: splits strings into text and numeric segments,
/// comparing numbers by value and text case-insensitively.
fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        match (a_chars.peek(), b_chars.peek()) {
            // Both exhausted — tiebreak by raw bytes for deterministic ordering
            (None, None) => return a.cmp(b),
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(&ac), Some(&bc)) => {
                if ac.is_ascii_digit() && bc.is_ascii_digit() {
                    let a_num = consume_number(&mut a_chars);
                    let b_num = consume_number(&mut b_chars);
                    let ord = a_num.cmp(&b_num);
                    if ord != std::cmp::Ordering::Equal {
                        return ord;
                    }
                } else {
                    // Compare full lowercase expansions (handles multi-codepoint
                    // lowercasing like ß → ss or İ → i\u{307}).
                    let ord = ac.to_lowercase().cmp(bc.to_lowercase());
                    if ord != std::cmp::Ordering::Equal {
                        return ord;
                    }
                    a_chars.next();
                    b_chars.next();
                }
            }
        }
    }
}

fn consume_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> u64 {
    let mut n: u64 = 0;
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            n = n.saturating_mul(10).saturating_add(c as u64 - '0' as u64);
            chars.next();
        } else {
            break;
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::node::{NodeKind, TreeNode};
    use std::path::PathBuf;

    fn file_node(name: &str) -> TreeNode {
        TreeNode::new(PathBuf::from(name), NodeKind::File, 0)
    }

    fn dir_node(name: &str) -> TreeNode {
        TreeNode::new(PathBuf::from(name), NodeKind::Directory, 0)
    }

    fn names(nodes: &[TreeNode]) -> Vec<&str> {
        nodes.iter().map(|n| n.name.as_str()).collect()
    }

    #[test]
    fn natural_sort_numeric_segments() {
        assert_eq!(natural_cmp("file1", "file2"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("file2", "file10"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("file10", "file2"), std::cmp::Ordering::Greater);
        assert_eq!(natural_cmp("file10", "file10"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn natural_sort_case_insensitive() {
        // Case-only differences: uppercase sorts before lowercase (byte-level tiebreak)
        assert_eq!(natural_cmp("ABC", "abc"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("abc", "ABC"), std::cmp::Ordering::Greater);
        assert_eq!(natural_cmp("apple", "Banana"), std::cmp::Ordering::Less);
    }

    #[test]
    fn natural_sort_unicode_case_insensitive() {
        // Accented characters should sort case-insensitively
        assert_eq!(natural_cmp("ñ_file", "Ñ_file"), std::cmp::Ordering::Greater);
        assert_eq!(natural_cmp("über", "Über"), std::cmp::Ordering::Greater);
        // ü and Ü should compare equal after lowercasing (unlike to_ascii_lowercase)
        assert_eq!(natural_cmp("ü", "ü"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn natural_sort_eszett_multi_codepoint_lowercase() {
        // ß lowercases to "ss" (2 codepoints) — with the fix the full
        // expansion is compared, not just the first codepoint.
        // Previously ß.to_lowercase().next() gave 's', making ß == s.
        // Now ß.to_lowercase() gives ['s','s'] which is > ['s'], so ß > s.
        assert_eq!(natural_cmp("ß", "s"), std::cmp::Ordering::Greater);
        // And ß should not equal a single 's' (it did before the fix)
        assert_ne!(natural_cmp("ß_file", "s_file"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn natural_sort_mixed() {
        assert_eq!(natural_cmp("a1b", "a1c"), std::cmp::Ordering::Less);
        assert_eq!(natural_cmp("a2b", "a10b"), std::cmp::Ordering::Less);
    }

    #[test]
    fn sort_nodes_dirs_first() {
        let mut nodes = vec![
            file_node("zebra.txt"),
            dir_node("alpha"),
            file_node("apple.txt"),
            dir_node("beta"),
        ];
        sort_nodes(&mut nodes, true);
        assert_eq!(
            names(&nodes),
            vec!["alpha", "beta", "apple.txt", "zebra.txt"]
        );
    }

    #[test]
    fn sort_nodes_no_dirs_first() {
        let mut nodes = vec![
            file_node("zebra.txt"),
            dir_node("delta"),
            file_node("apple.txt"),
            dir_node("beta"),
        ];
        sort_nodes(&mut nodes, false);
        assert_eq!(
            names(&nodes),
            vec!["apple.txt", "beta", "delta", "zebra.txt"]
        );
    }

    #[test]
    fn sort_nodes_natural_order() {
        let mut nodes = vec![
            file_node("file10.txt"),
            file_node("file2.txt"),
            file_node("file1.txt"),
        ];
        sort_nodes(&mut nodes, true);
        assert_eq!(names(&nodes), vec!["file1.txt", "file2.txt", "file10.txt"]);
    }

    // ── Property-based tests ────────────────────────────────────────────

    mod proptests {
        use super::*;
        use proptest::prelude::*;
        use std::cmp::Ordering;

        proptest! {
            #[test]
            fn natural_cmp_reflexive(ref s in "[a-zA-Z0-9_.]{1,30}") {
                prop_assert_eq!(natural_cmp(s, s), Ordering::Equal);
            }

            #[test]
            fn natural_cmp_antisymmetric(
                ref a in "[a-zA-Z0-9_.]{1,20}",
                ref b in "[a-zA-Z0-9_.]{1,20}",
            ) {
                let ab = natural_cmp(a, b);
                let ba = natural_cmp(b, a);
                match ab {
                    Ordering::Less => prop_assert_eq!(ba, Ordering::Greater),
                    Ordering::Greater => prop_assert_eq!(ba, Ordering::Less),
                    Ordering::Equal => prop_assert_eq!(ba, Ordering::Equal),
                }
            }

            #[test]
            fn natural_cmp_transitive(
                ref a in "[a-zA-Z0-9]{1,10}",
                ref b in "[a-zA-Z0-9]{1,10}",
                ref c in "[a-zA-Z0-9]{1,10}",
            ) {
                let ab = natural_cmp(a, b);
                let bc = natural_cmp(b, c);
                let ac = natural_cmp(a, c);
                // If a <= b and b <= c, then a <= c
                if ab != Ordering::Greater && bc != Ordering::Greater {
                    prop_assert_ne!(ac, Ordering::Greater);
                }
                // If a >= b and b >= c, then a >= c
                if ab != Ordering::Less && bc != Ordering::Less {
                    prop_assert_ne!(ac, Ordering::Less);
                }
            }

            #[test]
            fn sort_nodes_is_idempotent(
                names in prop::collection::vec("[a-zA-Z0-9_.]{1,15}", 0..50),
            ) {
                let mut nodes: Vec<TreeNode> = names
                    .iter()
                    .map(|n| file_node(n))
                    .collect();
                sort_nodes(&mut nodes, true);
                let first_sort: Vec<String> = nodes.iter().map(|n| n.name.clone()).collect();
                sort_nodes(&mut nodes, true);
                let second_sort: Vec<String> = nodes.iter().map(|n| n.name.clone()).collect();
                prop_assert_eq!(first_sort, second_sort);
            }
        }
    }
}
