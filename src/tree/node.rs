use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    File,
    Directory,
    Symlink,
}

/// Git status for a file or directory. Ordered by severity for propagation.
/// Unstaged changes take priority over staged (shown more prominently).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)] // All variants are part of the public API
pub enum GitStatus {
    Clean,
    Ignored,
    StagedAdded,
    StagedModified,
    StagedDeleted,
    Untracked,
    Added,
    Modified,
    Deleted,
    Conflicted,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub kind: NodeKind,
    pub depth: usize,
    pub is_expanded: bool,
    pub children_loaded: bool,
    pub git_status: GitStatus,
    /// File size in bytes (only populated when `show_size` is enabled).
    pub size: Option<u64>,
    /// Last modification time (only populated when `show_modified` is enabled).
    pub modified: Option<SystemTime>,
}

impl TreeNode {
    pub fn new(path: PathBuf, kind: NodeKind, depth: usize) -> Self {
        let name = path.file_name().map_or_else(
            || path.to_string_lossy().into_owned(),
            |n| n.to_string_lossy().into_owned(),
        );

        Self {
            path,
            name,
            kind,
            depth,
            is_expanded: false,
            children_loaded: false,
            git_status: GitStatus::Clean,
            size: None,
            modified: None,
        }
    }

    pub fn is_dir(&self) -> bool {
        self.kind == NodeKind::Directory
    }
}
