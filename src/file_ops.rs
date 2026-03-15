use std::path::{Component, Path, PathBuf};

use crate::render::input_dialog::DialogKind;

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
pub fn is_path_within_root(root: &Path, target: &Path) -> bool {
    let mut normalized = PathBuf::new();
    for comp in target.components() {
        match comp {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            _ => normalized.push(comp),
        }
    }
    normalized.starts_with(root)
}

/// Strict path-within-root check that canonicalizes the nearest existing
/// ancestor to defeat symlink-based path traversal.
pub fn is_path_within_root_strict(root: &Path, target: &Path) -> bool {
    // Fast lexical check first
    if !is_path_within_root(root, target) {
        return false;
    }
    // Canonicalize the nearest existing ancestor
    let mut check = target.to_path_buf();
    loop {
        match check.canonicalize() {
            Ok(canonical) => {
                let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
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
            let new_path = context_path.join(input);
            if !is_path_within_root_strict(root, &new_path) {
                return FileOpResult::Error("Path escapes workspace root".to_string());
            }
            if let Some(parent) = new_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return FileOpResult::Error(format!("Create dirs failed: {e}"));
                }
            }
            if let Err(e) = std::fs::File::create(&new_path) {
                return FileOpResult::Error(format!("Create file failed: {e}"));
            }
            FileOpResult::Ok
        }
        DialogKind::NewDir => {
            if input.is_empty() {
                return FileOpResult::Noop;
            }
            let new_path = context_path.join(input);
            if !is_path_within_root_strict(root, &new_path) {
                return FileOpResult::Error("Path escapes workspace root".to_string());
            }
            if let Err(e) = std::fs::create_dir_all(&new_path) {
                return FileOpResult::Error(format!("Create directory failed: {e}"));
            }
            FileOpResult::Ok
        }
        DialogKind::Rename => {
            if input.is_empty() || input == target_name {
                return FileOpResult::Noop;
            }
            if let Some(parent) = context_path.parent() {
                let new_path = parent.join(input);
                if !is_path_within_root_strict(root, &new_path) {
                    return FileOpResult::Error("Path escapes workspace root".to_string());
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

/// Get the directory context for a tree node (the node itself if dir, or its parent).
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

    // ── Path validation ─────────────────────────────────────────────────

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
        let tmp = std::env::temp_dir().join("croot_fileops_symlink_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("workspace")).unwrap();
        std::fs::create_dir_all(tmp.join("outside")).unwrap();

        let link_path = tmp.join("workspace/escape_link");
        symlink(tmp.join("outside"), &link_path).unwrap();

        let workspace = tmp.join("workspace");
        let target = link_path.join("evil.txt");

        assert!(is_path_within_root(&workspace, &target));
        assert!(!is_path_within_root_strict(&workspace, &target));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn strict_allows_normal_paths() {
        let tmp = std::env::temp_dir().join("croot_fileops_strict_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("workspace/subdir")).unwrap();

        let workspace = tmp.join("workspace");
        let target = workspace.join("subdir/new_file.txt");

        assert!(is_path_within_root_strict(&workspace, &target));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // ── execute_dialog ──────────────────────────────────────────────────

    #[test]
    fn new_file_creates_file() {
        let tmp = std::env::temp_dir().join("croot_fileops_newfile");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let result = execute_dialog(&DialogKind::NewFile, "test.txt", "", &tmp, &tmp, false);
        assert!(matches!(result, FileOpResult::Ok));
        assert!(tmp.join("test.txt").exists());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn new_file_empty_input_is_noop() {
        let tmp = std::env::temp_dir();
        let result = execute_dialog(&DialogKind::NewFile, "", "", &tmp, &tmp, false);
        assert!(matches!(result, FileOpResult::Noop));
    }

    #[test]
    fn new_file_rejects_path_escape() {
        let tmp = std::env::temp_dir().join("croot_fileops_escape");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let result = execute_dialog(
            &DialogKind::NewFile,
            "../../escape.txt",
            "",
            &tmp,
            &tmp,
            false,
        );
        assert!(matches!(result, FileOpResult::Error(_)));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn new_dir_creates_directory() {
        let tmp = std::env::temp_dir().join("croot_fileops_newdir");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let result = execute_dialog(&DialogKind::NewDir, "subdir", "", &tmp, &tmp, false);
        assert!(matches!(result, FileOpResult::Ok));
        assert!(tmp.join("subdir").is_dir());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn rename_same_name_is_noop() {
        let result = execute_dialog(
            &DialogKind::Rename,
            "same.txt",
            "same.txt",
            &std::env::temp_dir().join("same.txt"),
            &std::env::temp_dir(),
            false,
        );
        assert!(matches!(result, FileOpResult::Noop));
    }

    #[test]
    fn rename_nonexistent_returns_error() {
        let tmp = std::env::temp_dir().join("croot_fileops_rename_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let result = execute_dialog(
            &DialogKind::Rename,
            "new_name.txt",
            "old_name.txt",
            &tmp.join("old_name.txt"),
            &tmp,
            false,
        );
        assert!(matches!(result, FileOpResult::Error(_)));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn delete_nonexistent_returns_error() {
        let fake_path = std::env::temp_dir().join("croot_fileops_ghost_delete");
        let result = execute_dialog(
            &DialogKind::ConfirmDelete,
            "",
            "",
            &fake_path,
            &std::env::temp_dir(),
            false,
        );
        assert!(matches!(result, FileOpResult::Error(_)));
    }

    #[test]
    fn delete_file_succeeds() {
        let tmp = std::env::temp_dir().join("croot_fileops_delete_file");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let file = tmp.join("delete_me.txt");
        std::fs::write(&file, "content").unwrap();

        let result = execute_dialog(&DialogKind::ConfirmDelete, "", "", &file, &tmp, false);
        assert!(matches!(result, FileOpResult::Ok));
        assert!(!file.exists());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // ── dir_for_path ────────────────────────────────────────────────────

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

    // ── Property-based tests ────────────────────────────────────────────

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
