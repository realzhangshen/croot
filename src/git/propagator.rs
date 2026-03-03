use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::tree::node::GitStatus;

/// Propagate file statuses upward to parent directories.
/// Each directory gets the most severe status among its descendants.
///
/// Severity order: Conflicted > Deleted > Modified > Added > Untracked > Ignored > Clean
pub fn propagate_to_dirs(
    file_statuses: &HashMap<PathBuf, GitStatus>,
    repo_root: &Path,
) -> HashMap<PathBuf, GitStatus> {
    let mut dir_statuses: HashMap<PathBuf, GitStatus> = HashMap::new();

    for (file_path, &status) in file_statuses {
        if status == GitStatus::Clean {
            continue;
        }

        // Walk up from the file's parent to the repo root
        let mut current = file_path.parent();
        while let Some(dir) = current {
            // Stop when we've gone above the repo root
            if !dir.starts_with(repo_root) {
                break;
            }

            let entry = dir_statuses.entry(dir.to_path_buf()).or_insert(GitStatus::Clean);
            if status > *entry {
                *entry = status;
            }

            if dir == repo_root {
                break;
            }
            current = dir.parent();
        }
    }

    dir_statuses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_files_are_not_propagated() {
        let mut files = HashMap::new();
        files.insert(PathBuf::from("/repo/src/main.rs"), GitStatus::Clean);

        let dirs = propagate_to_dirs(&files, Path::new("/repo"));
        assert!(dirs.is_empty());
    }

    #[test]
    fn modified_file_propagates_to_all_ancestors() {
        let mut files = HashMap::new();
        files.insert(PathBuf::from("/repo/src/main.rs"), GitStatus::Modified);

        let dirs = propagate_to_dirs(&files, Path::new("/repo"));
        assert_eq!(dirs.get(Path::new("/repo/src")), Some(&GitStatus::Modified));
        assert_eq!(dirs.get(Path::new("/repo")), Some(&GitStatus::Modified));
    }

    #[test]
    fn most_severe_status_wins() {
        let mut files = HashMap::new();
        files.insert(PathBuf::from("/repo/src/a.rs"), GitStatus::Added);
        files.insert(PathBuf::from("/repo/src/b.rs"), GitStatus::Conflicted);

        let dirs = propagate_to_dirs(&files, Path::new("/repo"));
        // Conflicted > Added, so src/ should be Conflicted
        assert_eq!(dirs.get(Path::new("/repo/src")), Some(&GitStatus::Conflicted));
    }

    #[test]
    fn does_not_propagate_above_repo_root() {
        let mut files = HashMap::new();
        files.insert(PathBuf::from("/repo/file.rs"), GitStatus::Modified);

        let dirs = propagate_to_dirs(&files, Path::new("/repo"));
        assert_eq!(dirs.get(Path::new("/repo")), Some(&GitStatus::Modified));
        assert!(dirs.get(Path::new("/")).is_none());
    }
}
