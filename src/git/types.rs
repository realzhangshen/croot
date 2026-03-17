/// Git status for a file or directory. Ordered by severity for propagation.
/// The derived `Ord` uses declaration order: later variants have higher priority.
/// Staged changes sort above Untracked; unstaged changes sort above staged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GitStatus {
    Clean,
    Ignored,
    Untracked,
    StagedAdded,
    StagedModified,
    StagedDeleted,
    Modified,
    Deleted,
    Conflicted,
}
