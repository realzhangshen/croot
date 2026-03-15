mod common;

use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use croot::git::branches::list_branches;
use croot::git::status::GitState;
use croot::git::types::GitStatus;
use croot::tree::forest::FileTree;
use git2::{Repository, Signature};
use tempfile::tempdir;

use common::{create_file, create_test_repo, default_tree_config};

fn stage_path(repo: &Repository, relative_path: &str) {
    let mut index = repo.index().expect("open git index");
    index
        .add_path(Path::new(relative_path))
        .expect("stage path in git index");
    index.write().expect("flush git index");
}

fn commit_index(repo: &Repository, message: &str) {
    let tree_id = {
        let mut index = repo.index().expect("open git index");
        index.write().expect("flush git index");
        index.write_tree().expect("write git tree")
    };

    let tree = repo.find_tree(tree_id).expect("find committed tree");
    let signature = Signature::now("croot-tests", "croot@example.com").expect("create signature");

    match repo
        .head()
        .ok()
        .and_then(|head| head.target())
        .and_then(|oid| repo.find_commit(oid).ok())
    {
        Some(parent) => {
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[&parent],
            )
            .expect("commit staged changes");
        }
        None => {
            repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])
                .expect("create root commit");
        }
    }
}

fn advance_worktree_clock() {
    thread::sleep(Duration::from_millis(1_100));
}

#[test]
fn git_state_loads_branch_and_reports_untracked_files() {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path().canonicalize().expect("canonicalize temp dir");
    let repo = create_test_repo(&root);

    create_file(&root, "tracked.txt", "tracked\n");
    stage_path(&repo, "tracked.txt");
    commit_index(&repo, "Track file");
    advance_worktree_clock();
    create_file(&root, "untracked.txt", "new\n");

    let git = GitState::load(&root).expect("load git state");

    assert_eq!(git.branch(), Some("main"));
    assert_eq!(
        git.status_for(&root.join("tracked.txt"), false),
        GitStatus::Clean
    );
    assert_eq!(
        git.status_for(&root.join("untracked.txt"), false),
        GitStatus::Untracked
    );

    let branches = list_branches(git.repo_root());
    assert!(branches
        .iter()
        .any(|branch| branch.name == "main" && branch.is_current && !branch.is_remote));
}

#[test]
fn git_state_applies_file_and_directory_statuses_to_tree_nodes() {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path().canonicalize().expect("canonicalize temp dir");
    let repo = create_test_repo(&root);

    fs::create_dir(root.join("src")).expect("create src dir");
    create_file(&root, "src/changed.rs", "fn changed() {}\n");
    create_file(&root, "src/tracked.rs", "fn tracked() {}\n");
    stage_path(&repo, "src/changed.rs");
    stage_path(&repo, "src/tracked.rs");
    commit_index(&repo, "Add tracked src files");
    advance_worktree_clock();

    create_file(
        &root,
        "src/changed.rs",
        "fn changed() {\n    println!(\"changed\");\n}\n",
    );
    create_file(&root, "src/staged.rs", "fn staged() {}\n");
    stage_path(&repo, "src/staged.rs");

    let git = GitState::load(&root).expect("load git state");

    assert_eq!(
        git.status_for(&root.join("src/changed.rs"), false),
        GitStatus::Modified
    );
    assert_eq!(
        git.status_for(&root.join("src/staged.rs"), false),
        GitStatus::StagedAdded
    );
    assert_eq!(git.status_for(&root.join("src"), true), GitStatus::Modified);

    let mut tree = FileTree::new(root.clone(), default_tree_config());
    let src_idx = tree
        .nodes
        .iter()
        .position(|node| node.path == root.join("src"))
        .expect("find src dir");
    tree.expand(src_idx);

    git.apply_to_nodes(&mut tree.nodes);

    let src_node = tree
        .nodes
        .iter()
        .find(|node| node.path == root.join("src"))
        .expect("find src node");
    let changed_node = tree
        .nodes
        .iter()
        .find(|node| node.path == root.join("src/changed.rs"))
        .expect("find changed node");
    let staged_node = tree
        .nodes
        .iter()
        .find(|node| node.path == root.join("src/staged.rs"))
        .expect("find staged node");

    assert_eq!(src_node.git_status, GitStatus::Modified);
    assert_eq!(changed_node.git_status, GitStatus::Modified);
    assert_eq!(staged_node.git_status, GitStatus::StagedAdded);
}

#[test]
fn git_state_refresh_reflects_new_changes() {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path().canonicalize().expect("canonicalize temp dir");
    let repo = create_test_repo(&root);

    create_file(&root, "tracked.txt", "tracked\n");
    stage_path(&repo, "tracked.txt");
    commit_index(&repo, "Track file");
    advance_worktree_clock();

    let mut git = GitState::load(&root).expect("load git state");
    assert_eq!(
        git.status_for(&root.join("tracked.txt"), false),
        GitStatus::Clean
    );

    create_file(&root, "tracked.txt", "tracked and modified\n");
    create_file(&root, "fresh.txt", "brand new\n");
    git.refresh();

    assert_eq!(
        git.status_for(&root.join("tracked.txt"), false),
        GitStatus::Modified
    );
    assert_eq!(
        git.status_for(&root.join("fresh.txt"), false),
        GitStatus::Untracked
    );
}
