#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;

use croot::config::{ColorConfig, TreeConfig};
use git2::{Repository, RepositoryInitOptions, Signature};

static INIT_COLORS: Once = Once::new();

pub fn init_colors() {
    INIT_COLORS.call_once(|| {
        croot::render::colors::init(&ColorConfig::default());
    });
}

pub fn default_tree_config() -> TreeConfig {
    TreeConfig::default()
}

pub fn create_test_repo(dir: &Path) -> Repository {
    let mut options = RepositoryInitOptions::new();
    options.initial_head("main");

    let repo = Repository::init_opts(dir, &options).expect("initialize git repository");
    let signature = Signature::now("croot-tests", "croot@example.com").expect("create signature");

    let tree_id = {
        let mut index = repo.index().expect("open git index");
        let tree_id = index.write_tree().expect("write empty tree");
        index.write().expect("flush empty index");
        tree_id
    };

    let tree = repo.find_tree(tree_id).expect("find empty tree");
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    )
    .expect("create initial commit");
    drop(tree);

    repo
}

pub fn create_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent directories");
    }
    fs::write(&path, content).expect("write test file");
    path
}
