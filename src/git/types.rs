/// Git status for a file or directory. Ordered by severity for propagation.
/// Unstaged changes take priority over staged (shown more prominently).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GitStatus {
    Clean,
    Ignored,
    StagedAdded,
    StagedModified,
    StagedDeleted,
    Untracked,
    Modified,
    Deleted,
    Conflicted,
}
