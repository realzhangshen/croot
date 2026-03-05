use std::collections::HashSet;
use std::path::Path;

use ignore::WalkBuilder;

use crate::config::TreeConfig;

use super::node::{NodeKind, TreeNode};
use super::sorter::sort_nodes;

/// Read one level of a directory, respecting .gitignore rules and exclude list.
/// Returns sorted children (directories first, then natural sort).
pub fn load_children_with_meta(dir: &Path, depth: usize, config: &TreeConfig) -> Vec<TreeNode> {
    let show_hidden = config.show_hidden;
    let dirs_first = config.dirs_first;
    let exclude = &config.exclude;
    let show_ignored = config.show_ignored;
    let show_size = config.show_size;
    let show_modified = config.show_modified;
    let mut nodes = Vec::new();
    let exclude_set: HashSet<&str> = exclude.iter().map(std::string::String::as_str).collect();
    let need_meta = show_size || show_modified;

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

        let mut node = TreeNode::new(path.clone(), kind, depth);

        if need_meta {
            if let Ok(meta) = path.metadata() {
                if show_size && kind != NodeKind::Directory {
                    node.size = Some(meta.len());
                }
                if show_modified {
                    node.modified = meta.modified().ok();
                }
            }
        }

        nodes.push(node);
    }

    sort_nodes(&mut nodes, dirs_first);
    nodes
}
