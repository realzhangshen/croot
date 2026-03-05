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
