use std::collections::HashMap;
use std::path::{Path, PathBuf};

use git2::{Repository, StatusOptions};

use crate::tree::node::TreeNode;

use super::propagator::propagate_to_dirs;
use super::types::GitStatus;

pub struct GitState {
    repo_root: PathBuf,
    file_statuses: HashMap<PathBuf, GitStatus>,
    dir_statuses: HashMap<PathBuf, GitStatus>,
    branch: Option<String>,
    /// Last error from loading git statuses (e.g. locked index).
    last_error: Option<String>,
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
            last_error: None,
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
            self.last_error = None;
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
            if dir == self.repo_root {
                break;
            }
            current = dir.parent();
        }
        false
    }

    /// Apply git statuses to a slice of tree nodes.
    pub fn apply_to_nodes(&self, nodes: &mut [TreeNode]) {
        for node in nodes {
            node.git_status = self.status_for(&node.path, node.is_dir());
        }
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    /// Returns the last error from loading git statuses, if any.
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
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
            Err(e) => {
                self.last_error = Some(format!("git status: {e}"));
                return;
            }
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
        return GitStatus::Conflicted;
    }

    if status.is_index_deleted() && status.is_wt_new() {
        // git rm --cached: removed from index but still on disk → untracked
        return GitStatus::Untracked;
    }

    // Check for unstaged (working tree) changes first — they take priority
    // because they represent the "current" state the user sees.
    if status.is_wt_deleted() {
        return GitStatus::Deleted;
    }
    if status.is_wt_modified() || status.is_wt_renamed() {
        return GitStatus::Modified;
    }
    if status.is_wt_new() {
        return GitStatus::Untracked;
    }

    // Pure staged changes (in index only, no working tree changes)
    if status.is_index_deleted() {
        return GitStatus::StagedDeleted;
    }
    if status.is_index_modified() || status.is_index_renamed() {
        return GitStatus::StagedModified;
    }
    if status.is_index_new() {
        return GitStatus::StagedAdded;
    }

    if status.is_ignored() {
        return GitStatus::Ignored;
    }

    GitStatus::Clean
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::node::{NodeKind, TreeNode};

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
    fn index_deleted_maps_to_staged_deleted() {
        assert_eq!(
            convert_status(git2::Status::INDEX_DELETED),
            GitStatus::StagedDeleted
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
    fn index_modified_maps_to_staged_modified() {
        assert_eq!(
            convert_status(git2::Status::INDEX_MODIFIED),
            GitStatus::StagedModified
        );
    }

    #[test]
    fn index_new_maps_to_staged_added() {
        assert_eq!(
            convert_status(git2::Status::INDEX_NEW),
            GitStatus::StagedAdded
        );
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
            last_error: None,
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

    // --- apply_to_nodes tests ---

    #[test]
    fn apply_to_nodes_sets_file_statuses() {
        let state = make_state(
            "/repo",
            vec![
                ("/repo/src/main.rs", GitStatus::Modified),
                ("/repo/src/lib.rs", GitStatus::StagedAdded),
            ],
            vec![],
        );
        let mut nodes = vec![
            TreeNode::new(PathBuf::from("/repo/src/main.rs"), NodeKind::File, 1),
            TreeNode::new(PathBuf::from("/repo/src/lib.rs"), NodeKind::File, 1),
            TreeNode::new(PathBuf::from("/repo/src/other.rs"), NodeKind::File, 1),
        ];
        state.apply_to_nodes(&mut nodes);
        assert_eq!(nodes[0].git_status, GitStatus::Modified);
        assert_eq!(nodes[1].git_status, GitStatus::StagedAdded);
        assert_eq!(nodes[2].git_status, GitStatus::Clean);
    }

    #[test]
    fn apply_to_nodes_sets_dir_statuses() {
        let state = make_state("/repo", vec![], vec![("/repo/src", GitStatus::Modified)]);
        let mut nodes = vec![TreeNode::new(
            PathBuf::from("/repo/src"),
            NodeKind::Directory,
            1,
        )];
        state.apply_to_nodes(&mut nodes);
        assert_eq!(nodes[0].git_status, GitStatus::Modified);
    }

    #[test]
    fn apply_to_nodes_propagates_ignored_to_children() {
        let state = make_state("/repo", vec![("/repo/vendor", GitStatus::Ignored)], vec![]);
        let mut nodes = vec![TreeNode::new(
            PathBuf::from("/repo/vendor/pkg/file.go"),
            NodeKind::File,
            3,
        )];
        state.apply_to_nodes(&mut nodes);
        assert_eq!(nodes[0].git_status, GitStatus::Ignored);
    }

    // --- status_for edge cases ---

    #[test]
    fn status_for_dir_falls_back_to_file_statuses() {
        // git2 reports some dirs (like ignored dirs) in file_statuses
        let state = make_state(
            "/repo",
            vec![("/repo/build", GitStatus::Ignored)],
            vec![], // no dir_statuses entry
        );
        assert_eq!(
            state.status_for(Path::new("/repo/build"), true),
            GitStatus::Ignored
        );
    }

    #[test]
    fn status_for_dir_prefers_dir_statuses_over_file() {
        let state = make_state(
            "/repo",
            vec![("/repo/src", GitStatus::Ignored)],
            vec![("/repo/src", GitStatus::Modified)],
        );
        // dir_statuses should take priority for directories
        assert_eq!(
            state.status_for(Path::new("/repo/src"), true),
            GitStatus::Modified
        );
    }

    // --- convert_status edge cases ---

    #[test]
    fn wt_renamed_maps_to_modified() {
        assert_eq!(
            convert_status(git2::Status::WT_RENAMED),
            GitStatus::Modified
        );
    }

    #[test]
    fn index_renamed_maps_to_staged_modified() {
        assert_eq!(
            convert_status(git2::Status::INDEX_RENAMED),
            GitStatus::StagedModified
        );
    }

    #[test]
    fn conflicted_takes_priority_over_other_flags() {
        // Even with other flags set, CONFLICTED should win
        let status = git2::Status::CONFLICTED | git2::Status::WT_MODIFIED;
        assert_eq!(convert_status(status), GitStatus::Conflicted);
    }

    #[test]
    fn wt_changes_take_priority_over_index() {
        // WT_MODIFIED + INDEX_NEW should show as Modified (unstaged wins)
        let status = git2::Status::WT_MODIFIED | git2::Status::INDEX_NEW;
        assert_eq!(convert_status(status), GitStatus::Modified);
    }
}
