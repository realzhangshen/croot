use std::collections::HashMap;
use std::path::{Path, PathBuf};

use git2::{Repository, StatusOptions};

use crate::tree::node::GitStatus;

use super::propagator::propagate_to_dirs;

pub struct GitState {
    repo_root: PathBuf,
    file_statuses: HashMap<PathBuf, GitStatus>,
    dir_statuses: HashMap<PathBuf, GitStatus>,
    branch: Option<String>,
}

impl GitState {
    /// Attempt to discover a git repo from the given path and load statuses.
    pub fn load(path: &Path) -> Option<Self> {
        let repo = Repository::discover(path).ok()?;
        let repo_root = repo.workdir()?.to_path_buf();

        let mut state = GitState {
            repo_root: repo_root.clone(),
            file_statuses: HashMap::new(),
            dir_statuses: HashMap::new(),
            branch: None,
        };

        state.branch = Self::read_branch(&repo);
        state.load_statuses(&repo);
        state.dir_statuses = propagate_to_dirs(&state.file_statuses, &repo_root);

        Some(state)
    }

    /// Re-read all statuses from the repository.
    pub fn refresh(&mut self) {
        if let Ok(repo) = Repository::open(&self.repo_root) {
            self.branch = Self::read_branch(&repo);
            self.file_statuses.clear();
            self.dir_statuses.clear();
            self.load_statuses(&repo);
            self.dir_statuses = propagate_to_dirs(&self.file_statuses, &self.repo_root);
        }
    }

    /// Get the git status for a file or directory.
    pub fn status_for(&self, path: &Path, is_dir: bool) -> GitStatus {
        let direct = if is_dir {
            self.dir_statuses
                .get(path)
                .or_else(|| self.file_statuses.get(path))
                .copied()
        } else {
            self.file_statuses.get(path).copied()
        };
        if let Some(status) = direct {
            return status;
        }
        if self.is_inside_ignored(path) {
            return GitStatus::Ignored;
        }
        GitStatus::Clean
    }

    /// Check if a path is nested inside an ignored directory.
    fn is_inside_ignored(&self, path: &Path) -> bool {
        let mut current = path.parent();
        while let Some(dir) = current {
            if !dir.starts_with(&self.repo_root) {
                break;
            }
            if self.file_statuses.get(dir) == Some(&GitStatus::Ignored) {
                return true;
            }
            if self.dir_statuses.get(dir) == Some(&GitStatus::Ignored) {
                return true;
            }
            if dir == self.repo_root {
                break;
            }
            current = dir.parent();
        }
        false
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    fn read_branch(repo: &Repository) -> Option<String> {
        let head = repo.head().ok()?;
        if head.is_branch() {
            head.shorthand().map(std::string::ToString::to_string)
        } else {
            // Detached HEAD — show short hash
            head.target().map(|oid| format!("{oid:.7}"))
        }
    }

    fn load_statuses(&mut self, repo: &Repository) {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(true);

        let statuses = match repo.statuses(Some(&mut opts)) {
            Ok(s) => s,
            Err(_) => return,
        };

        for entry in statuses.iter() {
            let Some(path_str) = entry.path() else {
                continue;
            };
            let abs_path = self.repo_root.join(path_str);
            let status = convert_status(entry.status());
            self.file_statuses.insert(abs_path, status);
        }
    }
}

fn convert_status(status: git2::Status) -> GitStatus {
    if status.is_conflicted() {
        GitStatus::Conflicted
    } else if status.is_index_deleted() && status.is_wt_new() {
        // git rm --cached: removed from index but still on disk → untracked
        GitStatus::Untracked
    } else if status.is_wt_deleted() || status.is_index_deleted() {
        GitStatus::Deleted
    } else if status.is_wt_modified()
        || status.is_index_modified()
        || status.is_wt_renamed()
        || status.is_index_renamed()
    {
        GitStatus::Modified
    } else if status.is_index_new() {
        GitStatus::Added
    } else if status.is_wt_new() {
        GitStatus::Untracked
    } else if status.is_ignored() {
        GitStatus::Ignored
    } else {
        GitStatus::Clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- convert_status tests ---

    #[test]
    fn conflicted_maps_to_conflicted() {
        assert_eq!(
            convert_status(git2::Status::CONFLICTED),
            GitStatus::Conflicted
        );
    }

    #[test]
    fn index_deleted_plus_wt_new_maps_to_untracked() {
        // Simulates `git rm --cached`: file removed from index but still on disk
        let status = git2::Status::INDEX_DELETED | git2::Status::WT_NEW;
        assert_eq!(convert_status(status), GitStatus::Untracked);
    }

    #[test]
    fn wt_deleted_maps_to_deleted() {
        assert_eq!(convert_status(git2::Status::WT_DELETED), GitStatus::Deleted);
    }

    #[test]
    fn index_deleted_maps_to_deleted() {
        assert_eq!(
            convert_status(git2::Status::INDEX_DELETED),
            GitStatus::Deleted
        );
    }

    #[test]
    fn wt_modified_maps_to_modified() {
        assert_eq!(
            convert_status(git2::Status::WT_MODIFIED),
            GitStatus::Modified
        );
    }

    #[test]
    fn index_modified_maps_to_modified() {
        assert_eq!(
            convert_status(git2::Status::INDEX_MODIFIED),
            GitStatus::Modified
        );
    }

    #[test]
    fn index_new_maps_to_added() {
        assert_eq!(convert_status(git2::Status::INDEX_NEW), GitStatus::Added);
    }

    #[test]
    fn wt_new_maps_to_untracked() {
        assert_eq!(convert_status(git2::Status::WT_NEW), GitStatus::Untracked);
    }

    #[test]
    fn ignored_maps_to_ignored() {
        assert_eq!(convert_status(git2::Status::IGNORED), GitStatus::Ignored);
    }

    #[test]
    fn empty_status_maps_to_clean() {
        assert_eq!(convert_status(git2::Status::CURRENT), GitStatus::Clean);
    }

    // --- status_for / is_inside_ignored tests ---

    fn make_state(
        repo_root: &str,
        files: Vec<(&str, GitStatus)>,
        dirs: Vec<(&str, GitStatus)>,
    ) -> GitState {
        GitState {
            repo_root: PathBuf::from(repo_root),
            file_statuses: files
                .into_iter()
                .map(|(p, s)| (PathBuf::from(p), s))
                .collect(),
            dir_statuses: dirs
                .into_iter()
                .map(|(p, s)| (PathBuf::from(p), s))
                .collect(),
            branch: None,
        }
    }

    #[test]
    fn status_for_returns_direct_file_status() {
        let state = make_state(
            "/repo",
            vec![("/repo/src/main.rs", GitStatus::Modified)],
            vec![],
        );
        assert_eq!(
            state.status_for(Path::new("/repo/src/main.rs"), false),
            GitStatus::Modified
        );
    }

    #[test]
    fn status_for_returns_direct_dir_status() {
        let state = make_state("/repo", vec![], vec![("/repo/src", GitStatus::Modified)]);
        assert_eq!(
            state.status_for(Path::new("/repo/src"), true),
            GitStatus::Modified
        );
    }

    #[test]
    fn status_for_file_inside_ignored_dir() {
        // git2 reports node_modules/ as ignored in file_statuses
        let state = make_state(
            "/repo",
            vec![("/repo/node_modules", GitStatus::Ignored)],
            vec![],
        );
        assert_eq!(
            state.status_for(Path::new("/repo/node_modules/express/index.js"), false),
            GitStatus::Ignored
        );
    }

    #[test]
    fn status_for_file_inside_ignored_dir_via_dir_statuses() {
        let state = make_state("/repo", vec![], vec![("/repo/target", GitStatus::Ignored)]);
        assert_eq!(
            state.status_for(Path::new("/repo/target/debug/build/foo.o"), false),
            GitStatus::Ignored
        );
    }

    #[test]
    fn status_for_ignored_dir_found_in_file_statuses() {
        let state = make_state("/repo", vec![("/repo/target", GitStatus::Ignored)], vec![]);
        assert_eq!(
            state.status_for(Path::new("/repo/target"), true),
            GitStatus::Ignored
        );
    }

    #[test]
    fn status_for_does_not_walk_above_repo_root() {
        // Ignored dir exists above repo root — should not match
        let state = make_state("/repo", vec![("/ignored_dir", GitStatus::Ignored)], vec![]);
        assert_eq!(
            state.status_for(Path::new("/ignored_dir/file.txt"), false),
            GitStatus::Clean
        );
    }

    #[test]
    fn status_for_unknown_file_returns_clean() {
        let state = make_state("/repo", vec![], vec![]);
        assert_eq!(
            state.status_for(Path::new("/repo/unknown.txt"), false),
            GitStatus::Clean
        );
    }
}
