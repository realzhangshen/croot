use std::path::{Component, Path, PathBuf};

/// The kind of file operation dialog being shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogKind {
    NewFile,
    NewDir,
    Rename,
    ConfirmDelete,
}

impl DialogKind {
    /// Human-readable title for the dialog.
    pub fn title(&self) -> &'static str {
        match self {
            Self::NewFile => "New File",
            Self::NewDir => "New Directory",
            Self::Rename => "Rename",
            Self::ConfirmDelete => "Confirm Delete",
        }
    }
}

/// Result of executing a file dialog operation.
pub enum FileOpResult {
    /// Operation succeeded (tree should be refreshed).
    Ok,
    /// Operation failed with a user-visible error message.
    Error(String),
    /// Nothing to do (empty input, same name, etc.).
    Noop,
}

/// Check whether `target` (which may contain `..` or be absolute) resolves
/// to a path within `root`. Works purely on path components — no filesystem access.
///
/// **Note:** This is a lexical check only — it does not resolve symlinks or access
/// the filesystem. Callers performing security-sensitive operations (e.g. mutations)
/// should pass canonicalized paths or use `is_path_within_root_strict` instead.
pub fn is_path_within_root(root: &Path, target: &Path) -> bool {
    let normalize = |p: &Path| -> PathBuf {
        let mut out = PathBuf::new();
        for comp in p.components() {
            match comp {
                Component::ParentDir => {
                    out.pop();
                }
                Component::CurDir => {}
                _ => out.push(comp),
            }
        }
        out
    };
    normalize(target).starts_with(normalize(root))
}

/// Strict path-within-root check that canonicalizes the nearest existing
/// ancestor to defeat symlink-based path traversal.
pub fn is_path_within_root_strict(root: &Path, target: &Path) -> bool {
    if !is_path_within_root(root, target) {
        return false;
    }
    // Canonicalize the nearest existing ancestor to defeat symlink traversal
    let mut check = target.to_path_buf();
    loop {
        match check.canonicalize() {
            Ok(canonical) => {
                let Ok(canonical_root) = root.canonicalize() else {
                    return false;
                };
                return canonical.starts_with(&canonical_root);
            }
            Err(_) => {
                if !check.pop() {
                    return false;
                }
            }
        }
    }
}

fn is_single_name(input: &str) -> bool {
    let mut components = Path::new(input).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

fn names_differ_only_by_case(current: &str, requested: &str) -> bool {
    current != requested && current.to_lowercase() == requested.to_lowercase()
}

#[cfg(unix)]
fn paths_refer_to_same_existing_entry(left: &Path, right: &Path) -> std::io::Result<bool> {
    use std::os::unix::fs::MetadataExt;

    let left_meta = std::fs::symlink_metadata(left)?;
    let right_meta = std::fs::symlink_metadata(right)?;
    Ok(left_meta.dev() == right_meta.dev() && left_meta.ino() == right_meta.ino())
}

#[cfg(not(unix))]
fn paths_refer_to_same_existing_entry(left: &Path, right: &Path) -> std::io::Result<bool> {
    Ok(left.canonicalize()? == right.canonicalize()?)
}

/// Execute a confirmed dialog operation on the filesystem.
/// Returns a `FileOpResult` indicating success, error, or no-op.
pub fn execute_dialog(
    kind: &DialogKind,
    input: &str,
    target_name: &str,
    context_path: &Path,
    root: &Path,
    use_trash: bool,
) -> FileOpResult {
    match kind {
        DialogKind::NewFile => {
            if input.is_empty() {
                return FileOpResult::Noop;
            }
            if !is_single_name(input) {
                return FileOpResult::Error(
                    "Name must be a single file or directory name".to_string(),
                );
            }
            let new_path = context_path.join(input);
            if !is_path_within_root_strict(root, &new_path) {
                return FileOpResult::Error("Path escapes workspace root".to_string());
            }
            if let Err(e) = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&new_path)
            {
                return FileOpResult::Error(format!("Create file failed: {e}"));
            }
            FileOpResult::Ok
        }
        DialogKind::NewDir => {
            if input.is_empty() {
                return FileOpResult::Noop;
            }
            if !is_single_name(input) {
                return FileOpResult::Error(
                    "Name must be a single file or directory name".to_string(),
                );
            }
            let new_path = context_path.join(input);
            if !is_path_within_root_strict(root, &new_path) {
                return FileOpResult::Error("Path escapes workspace root".to_string());
            }
            if let Err(e) = std::fs::create_dir(&new_path) {
                return FileOpResult::Error(format!("Create directory failed: {e}"));
            }
            FileOpResult::Ok
        }
        DialogKind::Rename => {
            if input.is_empty() || input == target_name {
                return FileOpResult::Noop;
            }
            if !is_single_name(input) {
                return FileOpResult::Error(
                    "Name must be a single file or directory name".to_string(),
                );
            }
            if let Some(parent) = context_path.parent() {
                let new_path = parent.join(input);
                if !is_path_within_root_strict(root, &new_path) {
                    return FileOpResult::Error("Path escapes workspace root".to_string());
                }
                // TOCTOU note: there is a microsecond-scale race between this
                // existence check and the rename below — on Unix, `rename(2)`
                // silently overwrites an existing target, so a file that races
                // into existence here could still be clobbered. This is
                // intentionally accepted: croot is an interactive single-user
                // TUI, the rename is driven by a human keystroke, and a truly
                // atomic cross-platform "rename if not exists" would require
                // either `renameat2(RENAME_NOREPLACE)` (Linux-only, unsafe) or
                // a hard-link/unlink dance that does not work for directories.
                match new_path.try_exists() {
                    Ok(true) => {
                        let is_case_only_current_entry =
                            names_differ_only_by_case(target_name, input)
                                && match paths_refer_to_same_existing_entry(context_path, &new_path)
                                {
                                    Ok(same) => same,
                                    Err(e) => {
                                        return FileOpResult::Error(format!("Rename failed: {e}"));
                                    }
                                };
                        if !is_case_only_current_entry {
                            return FileOpResult::Error("Target already exists".to_string());
                        }
                    }
                    Err(e) => return FileOpResult::Error(format!("Rename failed: {e}")),
                    Ok(false) => {}
                }
                if let Err(e) = std::fs::rename(context_path, &new_path) {
                    return FileOpResult::Error(format!("Rename failed: {e}"));
                }
                FileOpResult::Ok
            } else {
                FileOpResult::Noop
            }
        }
        DialogKind::ConfirmDelete => {
            let path = context_path;
            if !is_path_within_root_strict(root, path) {
                return FileOpResult::Error("Path escapes workspace root".to_string());
            }
            if use_trash {
                if let Err(e) = trash::delete(path) {
                    return FileOpResult::Error(format!("Trash failed: {e}"));
                }
            } else if path.is_dir() {
                if let Err(e) = std::fs::remove_dir_all(path) {
                    return FileOpResult::Error(format!("Delete failed: {e}"));
                }
            } else if let Err(e) = std::fs::remove_file(path) {
                return FileOpResult::Error(format!("Delete failed: {e}"));
            }
            FileOpResult::Ok
        }
    }
}

/// Directory context for a tree node: itself if dir, otherwise its parent.
pub fn dir_for_path(path: &Path, is_dir: bool, root: &Path) -> PathBuf {
    if is_dir {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(root).to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn rejects_absolute_path() {
        let root = Path::new("/home/user/project");
        let target = PathBuf::from("/etc/passwd");
        assert!(!is_path_within_root(root, &target));
    }

    #[test]
    fn rejects_dotdot_escape() {
        let root = Path::new("/home/user/project");
        let target = root.join("../../etc");
        assert!(!is_path_within_root(root, &target));
    }

    #[test]
    fn allows_normal_subpath() {
        let root = Path::new("/home/user/project");
        let target = root.join("subdir/file.txt");
        assert!(is_path_within_root(root, &target));
    }

    #[test]
    fn allows_dotdot_within_root() {
        let root = Path::new("/home/user/project");
        let target = root.join("a/../b");
        assert!(is_path_within_root(root, &target));
    }

    #[test]
    fn strict_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("workspace")).unwrap();
        std::fs::create_dir_all(tmp.path().join("outside")).unwrap();

        let link_path = tmp.path().join("workspace/escape_link");
        symlink(tmp.path().join("outside"), &link_path).unwrap();

        let workspace = tmp.path().join("workspace");
        let target = link_path.join("evil.txt");

        assert!(is_path_within_root(&workspace, &target));
        assert!(!is_path_within_root_strict(&workspace, &target));
    }

    #[test]
    fn strict_allows_normal_paths() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("workspace/subdir")).unwrap();

        let workspace = tmp.path().join("workspace");
        let target = workspace.join("subdir/new_file.txt");

        assert!(is_path_within_root_strict(&workspace, &target));
    }

    #[test]
    fn strict_returns_false_when_root_cannot_be_canonicalized() {
        let tmp = TempDir::new().unwrap();
        let workspace = tmp.path().join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();

        // Use a nonexistent root — canonicalize will fail
        let bad_root = Path::new("/nonexistent_croot_test_root_xyz");
        let target = workspace.join("file.txt");

        // Should return false because root can't be canonicalized
        assert!(!is_path_within_root_strict(bad_root, &target));
    }

    #[test]
    fn new_file_creates_file() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();

        let result = execute_dialog(&DialogKind::NewFile, "test.txt", "", dir, dir, false);
        assert!(matches!(result, FileOpResult::Ok));
        assert!(dir.join("test.txt").exists());
    }

    #[test]
    fn new_file_empty_input_is_noop() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        let result = execute_dialog(&DialogKind::NewFile, "", "", dir, dir, false);
        assert!(matches!(result, FileOpResult::Noop));
    }

    #[test]
    fn new_file_rejects_path_escape() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();

        let result = execute_dialog(
            &DialogKind::NewFile,
            "../../escape.txt",
            "",
            dir,
            dir,
            false,
        );
        assert!(matches!(result, FileOpResult::Error(_)));
    }

    #[test]
    fn new_file_rejects_nested_name() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();

        let result = execute_dialog(&DialogKind::NewFile, "nested/file.txt", "", dir, dir, false);

        assert!(matches!(result, FileOpResult::Error(_)));
        assert!(!dir.join("nested").exists());
    }

    #[test]
    fn new_file_does_not_truncate_existing_file() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        let file = dir.join("existing.txt");
        std::fs::write(&file, "keep me").unwrap();

        let result = execute_dialog(&DialogKind::NewFile, "existing.txt", "", dir, dir, false);

        assert!(matches!(result, FileOpResult::Error(_)));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "keep me");
    }

    #[test]
    fn new_dir_creates_directory() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();

        let result = execute_dialog(&DialogKind::NewDir, "subdir", "", dir, dir, false);
        assert!(matches!(result, FileOpResult::Ok));
        assert!(dir.join("subdir").is_dir());
    }

    #[test]
    fn new_dir_existing_path_returns_error() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        std::fs::create_dir(dir.join("subdir")).unwrap();

        let result = execute_dialog(&DialogKind::NewDir, "subdir", "", dir, dir, false);

        assert!(matches!(result, FileOpResult::Error(_)));
    }

    #[test]
    fn rename_same_name_is_noop() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        let result = execute_dialog(
            &DialogKind::Rename,
            "same.txt",
            "same.txt",
            &dir.join("same.txt"),
            dir,
            false,
        );
        assert!(matches!(result, FileOpResult::Noop));
    }

    #[test]
    fn rename_nonexistent_returns_error() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();

        let result = execute_dialog(
            &DialogKind::Rename,
            "new_name.txt",
            "old_name.txt",
            &dir.join("old_name.txt"),
            dir,
            false,
        );
        assert!(matches!(result, FileOpResult::Error(_)));
    }

    #[test]
    fn rename_rejects_nested_name() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        let file = dir.join("old.txt");
        std::fs::write(&file, "old").unwrap();

        let result = execute_dialog(
            &DialogKind::Rename,
            "nested/new.txt",
            "old.txt",
            &file,
            dir,
            false,
        );

        assert!(matches!(result, FileOpResult::Error(_)));
        assert!(file.exists());
        assert!(!dir.join("nested").exists());
    }

    #[test]
    fn rename_existing_target_does_not_overwrite() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        let source = dir.join("source.txt");
        let target = dir.join("target.txt");
        std::fs::write(&source, "source").unwrap();
        std::fs::write(&target, "target").unwrap();

        let result = execute_dialog(
            &DialogKind::Rename,
            "target.txt",
            "source.txt",
            &source,
            dir,
            false,
        );

        assert!(matches!(result, FileOpResult::Error(_)));
        assert_eq!(std::fs::read_to_string(&source).unwrap(), "source");
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "target");
    }

    #[test]
    fn rename_case_only_updates_directory_entry() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        let source = dir.join("MixedCase.txt");
        let target = dir.join("mixedcase.txt");
        std::fs::write(&source, "case-sensitive rename").unwrap();

        let result = execute_dialog(
            &DialogKind::Rename,
            "mixedcase.txt",
            "MixedCase.txt",
            &source,
            dir,
            false,
        );

        assert!(matches!(result, FileOpResult::Ok));
        assert_eq!(
            std::fs::read_to_string(&target).unwrap(),
            "case-sensitive rename"
        );
        let names: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["mixedcase.txt"]);
    }

    #[test]
    fn delete_nonexistent_returns_error() {
        let tmp = TempDir::new().unwrap();
        let fake_path = tmp.path().join("ghost_file");
        let result = execute_dialog(
            &DialogKind::ConfirmDelete,
            "",
            "",
            &fake_path,
            tmp.path(),
            false,
        );
        assert!(matches!(result, FileOpResult::Error(_)));
    }

    #[test]
    fn delete_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;
        let tmp = TempDir::new().unwrap();
        let workspace = tmp.path().join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();

        let outside = tmp.path().join("outside");
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("secret.txt"), "secret").unwrap();

        let link = workspace.join("escape_link");
        symlink(&outside, &link).unwrap();

        let target = link.join("secret.txt");
        let result = execute_dialog(
            &DialogKind::ConfirmDelete,
            "",
            "",
            &target,
            &workspace,
            false,
        );
        assert!(matches!(result, FileOpResult::Error(_)));
        // File outside workspace should still exist
        assert!(outside.join("secret.txt").exists());
    }

    #[test]
    fn delete_file_succeeds() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        let file = dir.join("delete_me.txt");
        std::fs::write(&file, "content").unwrap();

        let result = execute_dialog(&DialogKind::ConfirmDelete, "", "", &file, dir, false);
        assert!(matches!(result, FileOpResult::Ok));
        assert!(!file.exists());
    }

    #[test]
    fn dir_for_path_returns_dir_itself() {
        let root = Path::new("/root");
        let path = Path::new("/root/subdir");
        assert_eq!(
            dir_for_path(path, true, root),
            PathBuf::from("/root/subdir")
        );
    }

    #[test]
    fn dir_for_path_returns_parent_for_file() {
        let root = Path::new("/root");
        let path = Path::new("/root/subdir/file.txt");
        assert_eq!(
            dir_for_path(path, false, root),
            PathBuf::from("/root/subdir")
        );
    }

    #[test]
    fn dir_for_path_falls_back_to_root() {
        let root = Path::new("/root");
        let path = Path::new("/");
        // A root path has no parent, so should fall back
        assert_eq!(dir_for_path(path, false, root), PathBuf::from("/root"));
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn subpath_always_within_root(
                segment in "[a-zA-Z0-9_]{1,20}",
            ) {
                let root = Path::new("/home/user/project");
                let target = root.join(&segment);
                prop_assert!(is_path_within_root(root, &target));
            }

            #[test]
            fn absolute_path_outside_root_rejected(
                segment in "[a-zA-Z0-9_]{1,20}",
            ) {
                let root = Path::new("/home/user/project");
                let target = PathBuf::from(format!("/other/{segment}"));
                prop_assert!(!is_path_within_root(root, &target));
            }

            #[test]
            fn dotdot_escape_detected(
                n in 3usize..10,
                segment in "[a-zA-Z0-9_]{1,10}",
            ) {
                let root = Path::new("/home/user/project");
                let mut target = root.to_path_buf();
                for _ in 0..n {
                    target = target.join("..");
                }
                target = target.join(&segment);
                // n >= 3 means we escape /home/user/project completely
                prop_assert!(!is_path_within_root(root, &target));
            }

            #[test]
            fn nested_subpath_within_root(
                segments in prop::collection::vec("[a-zA-Z0-9_]{1,10}", 1..5),
            ) {
                let root = Path::new("/home/user/project");
                let mut target = root.to_path_buf();
                for seg in &segments {
                    target = target.join(seg);
                }
                prop_assert!(is_path_within_root(root, &target));
            }
        }
    }
}
