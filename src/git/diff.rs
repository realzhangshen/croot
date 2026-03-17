use std::path::Path;

use similar::{ChangeTag, TextDiff};

/// Per-line diff status for the preview gutter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineDiffStatus {
    Unchanged,
    Added,
    Modified,
    DeletedAbove,
}

/// Compute per-line diff status by comparing the HEAD version of a file
/// against `current_content`.
///
/// Returns `None` if: no git repo, no HEAD commit, binary/non-UTF-8 HEAD content,
/// or any git error. Returns `Some(vec![Added; n])` for untracked files.
///
/// Takes `current_content` as a parameter to avoid re-reading the file
/// (the preview loader already has it).
pub fn compute_line_diff(
    repo_path: &Path,
    file_path: &Path,
    current_content: &str,
) -> Option<Vec<LineDiffStatus>> {
    let repo = git2::Repository::discover(repo_path).ok()?;
    let workdir = repo.workdir()?.canonicalize().ok()?;
    let canonical_file = file_path.canonicalize().ok()?;
    let relative = canonical_file.strip_prefix(&workdir).ok()?;

    let Some(head_content) = get_head_content(&repo, relative) else {
        // File not in HEAD (untracked/new) → all lines Added
        let line_count = current_content.lines().count().max(1);
        return Some(vec![LineDiffStatus::Added; line_count]);
    };

    Some(diff_lines(&head_content, current_content))
}

/// Retrieve the UTF-8 content of a file from HEAD.
fn get_head_content(repo: &git2::Repository, relative: &Path) -> Option<String> {
    let head = repo.head().ok()?;
    let tree = head.peel_to_tree().ok()?;
    // Convert path to forward slashes for git
    let git_path = relative.to_str()?;
    let entry = tree.get_path(Path::new(git_path)).ok()?;
    let blob = repo.find_blob(entry.id()).ok()?;
    // Only handle UTF-8 text
    std::str::from_utf8(blob.content()).ok().map(String::from)
}

/// Diff two strings line-by-line and produce per-line status for `new_content`.
///
/// Algorithm:
/// - `Equal` ops → `Unchanged`
/// - `Delete` followed by `Insert` in the same hunk → `Modified` (on the inserted lines)
/// - `Insert` alone → `Added`
/// - `Delete` alone → mark the next line as `DeletedAbove`
/// - EOF deletion with no following line → mark the last result line as `DeletedAbove`
fn diff_lines(old: &str, new: &str) -> Vec<LineDiffStatus> {
    let diff = TextDiff::from_lines(old, new);
    let changes: Vec<_> = diff.iter_all_changes().collect();

    let mut result: Vec<LineDiffStatus> = Vec::new();
    // Track pending deletes that haven't been paired with inserts
    let mut pending_deletes: usize = 0;

    for change in &changes {
        match change.tag() {
            ChangeTag::Equal => {
                if pending_deletes > 0 {
                    // Deletes followed by equal → mark this line as DeletedAbove
                    result.push(LineDiffStatus::DeletedAbove);
                    pending_deletes = 0;
                } else {
                    result.push(LineDiffStatus::Unchanged);
                }
            }
            ChangeTag::Delete => {
                pending_deletes += 1;
            }
            ChangeTag::Insert => {
                if pending_deletes > 0 {
                    result.push(LineDiffStatus::Modified);
                    pending_deletes = pending_deletes.saturating_sub(1);
                } else {
                    result.push(LineDiffStatus::Added);
                }
            }
        }
    }

    // EOF edge case: if there are pending deletes at the end with no following line,
    // attach the marker to the last content line — but only if it is not already
    // Modified (a replace hunk where N old lines became M < N new lines).
    if pending_deletes > 0 {
        if let Some(last) = result.last_mut() {
            if *last != LineDiffStatus::Modified {
                *last = LineDiffStatus::DeletedAbove;
            }
            // Modified lines already indicate a change; the extra deletions
            // are part of the same hunk and don't need a separate marker.
        } else {
            // File is now empty but had deletions — single marker line
            result.push(LineDiffStatus::DeletedAbove);
        }
    }

    // Ensure at least one entry for non-empty content
    if result.is_empty() && !new.is_empty() {
        result.push(LineDiffStatus::Unchanged);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- diff_lines unit tests ---

    #[test]
    fn identical_content_all_unchanged() {
        let content = "line1\nline2\nline3\n";
        let result = diff_lines(content, content);
        assert_eq!(
            result,
            vec![
                LineDiffStatus::Unchanged,
                LineDiffStatus::Unchanged,
                LineDiffStatus::Unchanged,
            ]
        );
    }

    #[test]
    fn all_new_lines_added() {
        let result = diff_lines("", "new1\nnew2\n");
        assert_eq!(result, vec![LineDiffStatus::Added, LineDiffStatus::Added]);
    }

    #[test]
    fn deleted_lines_only() {
        // Old has 3 lines, new has 1 — the remaining line should show DeletedAbove
        let result = diff_lines("a\nb\nc\n", "a\n");
        assert_eq!(result, vec![LineDiffStatus::DeletedAbove]);
    }

    #[test]
    fn modified_lines() {
        // Replace line 2
        let result = diff_lines("a\nb\nc\n", "a\nB\nc\n");
        assert_eq!(
            result,
            vec![
                LineDiffStatus::Unchanged,
                LineDiffStatus::Modified,
                LineDiffStatus::Unchanged,
            ]
        );
    }

    #[test]
    fn mixed_add_and_modify() {
        let result = diff_lines("a\nb\n", "a\nB\nc\n");
        assert_eq!(
            result,
            vec![
                LineDiffStatus::Unchanged,
                LineDiffStatus::Modified,
                LineDiffStatus::Added,
            ]
        );
    }

    #[test]
    fn replace_more_with_fewer_at_eof_preserves_modified() {
        // Old: a\nb\nc\n → New: a\nX\n (replace 2 lines with 1 at EOF)
        // X should be Modified, not overwritten to DeletedAbove
        let result = diff_lines("a\nb\nc\n", "a\nX\n");
        assert_eq!(
            result,
            vec![LineDiffStatus::Unchanged, LineDiffStatus::Modified]
        );
    }

    #[test]
    fn eof_deletion_marks_last_line() {
        // Old: a\nb\nc\n  →  New: a\nb\n  (deleted c at end)
        let result = diff_lines("a\nb\nc\n", "a\nb\n");
        // "a" is unchanged, "b" should be marked DeletedAbove since c was deleted after it
        assert_eq!(
            result,
            vec![LineDiffStatus::Unchanged, LineDiffStatus::DeletedAbove]
        );
    }

    #[test]
    fn empty_new_file() {
        let result = diff_lines("a\nb\n", "");
        // All lines deleted, result should have single DeletedAbove marker
        assert_eq!(result, vec![LineDiffStatus::DeletedAbove]);
    }

    #[test]
    fn empty_both() {
        let result = diff_lines("", "");
        assert!(result.is_empty());
    }

    #[test]
    fn insert_in_middle() {
        let result = diff_lines("a\nc\n", "a\nb\nc\n");
        assert_eq!(
            result,
            vec![
                LineDiffStatus::Unchanged,
                LineDiffStatus::Added,
                LineDiffStatus::Unchanged,
            ]
        );
    }

    #[test]
    fn delete_in_middle_marks_next_line() {
        // Delete b from "a\nb\nc\n" → "a\nc\n"
        let result = diff_lines("a\nb\nc\n", "a\nc\n");
        assert_eq!(
            result,
            vec![LineDiffStatus::Unchanged, LineDiffStatus::DeletedAbove]
        );
    }

    // --- Integration test with git2 + tempdir ---

    #[test]
    fn compute_line_diff_with_real_repo() {
        let dir = tempfile::tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();

        // Create initial commit with a file
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "line1\nline2\nline3\n").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = git2::Signature::now("test", "test@test.com").unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .unwrap();

        // Modify the file
        let new_content = "line1\nmodified\nline3\nnew_line\n";
        std::fs::write(&file_path, new_content).unwrap();

        let result = compute_line_diff(dir.path(), &file_path, new_content);
        let statuses = result.unwrap();
        assert_eq!(statuses.len(), 4);
        assert_eq!(statuses[0], LineDiffStatus::Unchanged);
        assert_eq!(statuses[1], LineDiffStatus::Modified);
        assert_eq!(statuses[2], LineDiffStatus::Unchanged);
        assert_eq!(statuses[3], LineDiffStatus::Added);
    }

    #[test]
    fn compute_line_diff_untracked_file() {
        let dir = tempfile::tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();

        // Create initial commit (empty)
        let mut index = repo.index().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = git2::Signature::now("test", "test@test.com").unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .unwrap();

        // Create an untracked file
        let file_path = dir.path().join("new.txt");
        let content = "a\nb\nc\n";
        std::fs::write(&file_path, content).unwrap();

        let result = compute_line_diff(dir.path(), &file_path, content);
        let statuses = result.unwrap();
        assert_eq!(statuses.len(), 3);
        assert!(statuses.iter().all(|s| *s == LineDiffStatus::Added));
    }

    #[test]
    fn compute_line_diff_no_repo_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        std::fs::write(&file_path, "content").unwrap();

        assert!(compute_line_diff(dir.path(), &file_path, "content").is_none());
    }

    #[test]
    fn compute_line_diff_clean_file_all_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();

        let file_path = dir.path().join("test.txt");
        let content = "hello\nworld\n";
        std::fs::write(&file_path, content).unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = git2::Signature::now("test", "test@test.com").unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .unwrap();

        let result = compute_line_diff(dir.path(), &file_path, content);
        let statuses = result.unwrap();
        assert_eq!(statuses.len(), 2);
        assert!(statuses.iter().all(|s| *s == LineDiffStatus::Unchanged));
    }
}
