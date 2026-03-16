/// Stress tests for large directory trees.
/// Verifies that tree loading, sorting, and refresh remain correct and performant
/// under load (1K+ files, deeply nested dirs, rapid operations).
use std::time::Instant;

use croot::config::TreeConfig;
use croot::tree::forest::FileTree;

/// Create a flat directory with `n` files.
fn create_flat_tree(dir: &std::path::Path, n: usize) {
    for i in 0..n {
        std::fs::write(dir.join(format!("file_{i:05}.txt")), format!("content {i}")).unwrap();
    }
}

/// Create a deeply nested directory structure: depth levels, each with `breadth` entries.
fn create_deep_tree(dir: &std::path::Path, depth: usize, breadth: usize) {
    if depth == 0 {
        for i in 0..breadth {
            std::fs::write(dir.join(format!("leaf_{i}.txt")), "leaf").unwrap();
        }
        return;
    }
    for i in 0..breadth {
        let sub = dir.join(format!("d{depth}_{i}"));
        std::fs::create_dir(&sub).unwrap();
        create_deep_tree(&sub, depth - 1, breadth);
    }
}

#[test]
fn stress_flat_1000_files() {
    let tmp = tempfile::tempdir().unwrap();
    create_flat_tree(tmp.path(), 1000);

    let start = Instant::now();
    let tree = FileTree::new(tmp.path().to_path_buf(), TreeConfig::default());
    let elapsed = start.elapsed();

    assert_eq!(tree.len(), 1000);
    assert!(
        elapsed.as_millis() < 5000,
        "Loading 1000 files took {elapsed:?}, expected < 5s"
    );
}

#[test]
fn stress_flat_2000_files_sorted_correctly() {
    let tmp = tempfile::tempdir().unwrap();
    create_flat_tree(tmp.path(), 2000);

    let tree = FileTree::new(tmp.path().to_path_buf(), TreeConfig::default());
    assert_eq!(tree.len(), 2000);

    // Verify natural sort order
    for pair in tree.nodes.windows(2) {
        let a = &pair[0].name;
        let b = &pair[1].name;
        assert!(
            a <= b,
            "Sort order violated: {a:?} should come before {b:?}"
        );
    }
}

#[test]
fn stress_deep_nested_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    // 4 levels deep, 3 entries each = 3^4 = 81 leaf files + directories
    create_deep_tree(tmp.path(), 4, 3);

    let start = Instant::now();
    let tree = FileTree::new(tmp.path().to_path_buf(), TreeConfig::default());
    let elapsed = start.elapsed();

    // Should load the top level only (not recursively expanded)
    assert!(!tree.is_empty(), "Tree should have nodes");
    assert!(
        elapsed.as_millis() < 2000,
        "Loading deep tree took {elapsed:?}, expected < 2s"
    );
}

#[test]
fn stress_refresh_preserves_count() {
    let tmp = tempfile::tempdir().unwrap();
    create_flat_tree(tmp.path(), 500);

    let mut tree = FileTree::new(tmp.path().to_path_buf(), TreeConfig::default());
    let initial_count = tree.len();
    assert_eq!(initial_count, 500);

    // Refresh should produce the same count
    tree.refresh();
    assert_eq!(tree.len(), initial_count);

    // Add more files and refresh
    for i in 500..600 {
        std::fs::write(
            tmp.path().join(format!("file_{i:05}.txt")),
            format!("new {i}"),
        )
        .unwrap();
    }
    tree.refresh();
    assert_eq!(tree.len(), 600);
}

#[test]
fn stress_mixed_dirs_and_files() {
    let tmp = tempfile::tempdir().unwrap();
    // Create 200 directories with 5 files each = 1000 files + 200 dirs
    for i in 0..200 {
        let dir = tmp.path().join(format!("dir_{i:04}"));
        std::fs::create_dir(&dir).unwrap();
        for j in 0..5 {
            std::fs::write(dir.join(format!("f_{j}.txt")), "content").unwrap();
        }
    }
    // Plus 100 top-level files
    for i in 0..100 {
        std::fs::write(tmp.path().join(format!("top_{i:04}.txt")), "top content").unwrap();
    }

    let tree = FileTree::new(tmp.path().to_path_buf(), TreeConfig::default());

    // Top level should have 200 dirs + 100 files = 300
    assert_eq!(tree.len(), 300);
    // dirs_first: first 200 should be dirs
    assert_eq!(tree.dir_count, 200);
    assert_eq!(tree.file_count, 100);
}

#[test]
fn stress_rapid_refresh_cycle() {
    let tmp = tempfile::tempdir().unwrap();
    create_flat_tree(tmp.path(), 100);

    let mut tree = FileTree::new(tmp.path().to_path_buf(), TreeConfig::default());

    let start = Instant::now();
    for _ in 0..50 {
        tree.refresh();
    }
    let elapsed = start.elapsed();

    assert_eq!(tree.len(), 100);
    assert!(
        elapsed.as_millis() < 5000,
        "50 refreshes took {elapsed:?}, expected < 5s"
    );
}
