use std::collections::HashSet;
use std::path::Path;

use ignore::WalkBuilder;

use super::node::{NodeKind, TreeNode};
use super::sorter::sort_nodes;

/// Read one level of a directory, respecting .gitignore rules and exclude list.
/// Returns sorted children (directories first, then natural sort).
pub fn load_children(
    dir: &Path,
    depth: usize,
    show_hidden: bool,
    dirs_first: bool,
    exclude: &[String],
    show_ignored: bool,
) -> Vec<TreeNode> {
    let mut nodes = Vec::new();
    let exclude_set: HashSet<&str> = exclude.iter().map(std::string::String::as_str).collect();

    let walker = WalkBuilder::new(dir)
        .max_depth(Some(1))
        .hidden(!show_hidden) // hidden(true) = skip dotfiles
        .git_ignore(!show_ignored)
        .git_global(!show_ignored)
        .git_exclude(!show_ignored)
        .sort_by_file_name(std::cmp::Ord::cmp)
        .build();

    for entry in walker.flatten() {
        let path = entry.path().to_path_buf();

        // Skip the directory itself (depth 0 entry)
        if path == dir {
            continue;
        }

        // Skip entries matching the exclude list
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if exclude_set.contains(name) {
                continue;
            }
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
