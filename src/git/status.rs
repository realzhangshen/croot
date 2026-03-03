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
        if is_dir {
            self.dir_statuses
                .get(path)
                .copied()
                .unwrap_or(GitStatus::Clean)
        } else {
            self.file_statuses
                .get(path)
                .copied()
                .unwrap_or(GitStatus::Clean)
        }
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    fn read_branch(repo: &Repository) -> Option<String> {
        let head = repo.head().ok()?;
        if head.is_branch() {
            head.shorthand().map(|s| s.to_string())
        } else {
            // Detached HEAD — show short hash
            head.target()
                .map(|oid| format!("{:.7}", oid))
        }
    }

    fn load_statuses(&mut self, repo: &Repository) {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false);

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
    } else if status.is_wt_deleted() || status.is_index_deleted() {
        GitStatus::Deleted
    } else if status.is_wt_modified() || status.is_index_modified() || status.is_wt_renamed() || status.is_index_renamed() {
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
