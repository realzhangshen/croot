use std::path::Path;

use ignore::WalkBuilder;

use super::node::{NodeKind, TreeNode};
use super::sorter::sort_nodes;

/// Read one level of a directory, respecting .gitignore rules.
/// Returns sorted children (directories first, then natural sort).
pub fn load_children(dir: &Path, depth: usize, show_hidden: bool, dirs_first: bool) -> Vec<TreeNode> {
    let mut nodes = Vec::new();

    let walker = WalkBuilder::new(dir)
        .max_depth(Some(1))
        .hidden(!show_hidden) // hidden(true) = skip dotfiles
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .sort_by_file_name(|a, b| a.cmp(b))
        .build();

    for entry in walker.flatten() {
        let path = entry.path().to_path_buf();

        // Skip the directory itself (depth 0 entry)
        if path == dir {
            continue;
        }

        let kind = if path.is_symlink() {
            NodeKind::Symlink
        } else if path.is_dir() {
            NodeKind::Directory
        } else {
            NodeKind::File
        };

        nodes.push(TreeNode::new(path, kind, depth));
    }

    sort_nodes(&mut nodes, dirs_first);
    nodes
}
