mod common;

use std::fs;

use croot::tree::forest::FileTree;
use tempfile::tempdir;

use common::{create_file, default_tree_config};

#[test]
fn expand_collapse_and_navigation_follow_real_filesystem() {
    let dir = tempdir().expect("create temp dir");
    fs::create_dir(dir.path().join("src")).expect("create src dir");
    create_file(dir.path(), "src/lib.rs", "pub fn lib() {}\n");
    create_file(dir.path(), "src/main.rs", "fn main() {}\n");
    create_file(dir.path(), "README.md", "# croot\n");

    let mut tree = FileTree::new(dir.path().to_path_buf(), default_tree_config());
    let collapsed_len = tree.len();
    let src_idx = tree
        .nodes
        .iter()
        .position(|node| node.name == "src")
        .expect("find src dir");
    let readme_idx = tree
        .nodes
        .iter()
        .position(|node| node.name == "README.md")
        .expect("find README");

    assert!(src_idx < readme_idx, "directories should sort before files");

    tree.cursor = src_idx;
    tree.cursor_right();
    assert!(tree.nodes[src_idx].is_expanded);
    assert!(tree.len() > collapsed_len);
    assert_eq!(tree.cursor, src_idx + 1);
    assert_eq!(tree.selected().expect("selected child").depth, 1);

    tree.cursor_left();
    assert_eq!(tree.cursor, src_idx);
    assert!(tree.nodes[src_idx].is_expanded);

    tree.cursor_left();
    assert!(!tree.nodes[src_idx].is_expanded);
    assert_eq!(tree.len(), collapsed_len);

    for _ in 0..20 {
        tree.cursor_down();
    }
    assert_eq!(tree.cursor, tree.len().saturating_sub(1));

    for _ in 0..20 {
        tree.cursor_up();
    }
    assert_eq!(tree.cursor, 0);
}

#[test]
fn refresh_preserves_expanded_state_and_discovers_new_children() {
    let dir = tempdir().expect("create temp dir");
    fs::create_dir(dir.path().join("src")).expect("create src dir");
    create_file(dir.path(), "src/lib.rs", "pub fn lib() {}\n");

    let mut tree = FileTree::new(dir.path().to_path_buf(), default_tree_config());
    let src_path = dir.path().join("src");
    let src_idx = tree
        .nodes
        .iter()
        .position(|node| node.path == src_path)
        .expect("find src dir");
    tree.expand(src_idx);
    assert!(tree.nodes[src_idx].is_expanded);

    create_file(dir.path(), "src/new.rs", "pub fn new_fn() {}\n");
    tree.refresh();

    let src_idx = tree
        .nodes
        .iter()
        .position(|node| node.path == src_path)
        .expect("find refreshed src dir");
    assert!(tree.nodes[src_idx].is_expanded);
    assert!(
        tree.nodes
            .iter()
            .any(|node| node.path == dir.path().join("src/new.rs") && node.depth == 1),
        "refresh should keep src expanded and surface new children"
    );
}

#[test]
fn tree_config_filters_hidden_entries_and_respects_ordering() {
    let dir = tempdir().expect("create temp dir");
    fs::create_dir(dir.path().join("adir")).expect("create visible dir");
    fs::create_dir(dir.path().join("skip-me")).expect("create excluded dir");
    create_file(dir.path(), ".hidden", "secret\n");
    create_file(dir.path(), "visible.txt", "shown\n");

    let mut config = default_tree_config();
    config.show_hidden = false;
    config.exclude = vec!["skip-me".to_string()];
    config.dirs_first = true;

    let tree = FileTree::new(dir.path().to_path_buf(), config);
    let names: Vec<&str> = tree.nodes.iter().map(|node| node.name.as_str()).collect();

    assert_eq!(names.first().copied(), Some("adir"));
    assert!(names.contains(&"visible.txt"));
    assert!(!names.contains(&".hidden"));
    assert!(!names.contains(&"skip-me"));
}

#[test]
fn compact_folders_reports_chain_length_from_loaded_tree() {
    let dir = tempdir().expect("create temp dir");
    fs::create_dir_all(dir.path().join("a/b/c")).expect("create nested dirs");
    create_file(dir.path(), "a/b/c/file.txt", "hello\n");

    let mut config = default_tree_config();
    config.compact_folders = true;

    let mut tree = FileTree::new(dir.path().to_path_buf(), config);
    let a_path = dir.path().join("a");
    let b_path = dir.path().join("a/b");
    let c_path = dir.path().join("a/b/c");

    let a_idx = tree
        .nodes
        .iter()
        .position(|node| node.path == a_path)
        .expect("find a");
    tree.expand(a_idx);

    let b_idx = tree
        .nodes
        .iter()
        .position(|node| node.path == b_path)
        .expect("find b");
    tree.expand(b_idx);

    let c_idx = tree
        .nodes
        .iter()
        .position(|node| node.path == c_path)
        .expect("find c");
    tree.expand(c_idx);

    let a_idx = tree
        .nodes
        .iter()
        .position(|node| node.path == dir.path().join("a"))
        .expect("find refreshed a");
    let b_idx = tree
        .nodes
        .iter()
        .position(|node| node.path == dir.path().join("a/b"))
        .expect("find refreshed b");
    let c_idx = tree
        .nodes
        .iter()
        .position(|node| node.path == dir.path().join("a/b/c"))
        .expect("find refreshed c");

    assert_eq!(tree.compact_chain_len(a_idx), 2);
    assert_eq!(tree.compact_display_name_for(a_idx), "a/b/c/");

    let displayable = tree.build_displayable_indices();
    assert!(displayable.contains(&a_idx));
    assert!(!displayable.contains(&b_idx));
    assert!(!displayable.contains(&c_idx));
}
