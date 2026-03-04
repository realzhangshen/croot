use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GitStatus {
    Clean,
    Ignored,
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
        }
    }

    pub fn is_dir(&self) -> bool {
        self.kind == NodeKind::Directory
    }
}
