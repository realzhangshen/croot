use super::node::TreeNode;

/// Sort nodes: directories first, then natural (case-insensitive) sort by name.
pub fn sort_nodes(nodes: &mut [TreeNode]) {
    nodes.sort_by(|a, b| {
        // Directories come first
        let dir_ord = b.is_dir().cmp(&a.is_dir());
        if dir_ord != std::cmp::Ordering::Equal {
            return dir_ord;
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
            (None, None) => return std::cmp::Ordering::Equal,
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
                    let a_lower = ac.to_ascii_lowercase();
                    let b_lower = bc.to_ascii_lowercase();
                    let ord = a_lower.cmp(&b_lower);
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
