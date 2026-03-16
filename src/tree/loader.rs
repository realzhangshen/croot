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

        let kind = match entry.file_type() {
            Some(ft) if ft.is_symlink() => NodeKind::Symlink,
            Some(ft) if ft.is_dir() => NodeKind::Directory,
            _ => NodeKind::File,
        };

        let mut node = TreeNode::new(path.clone(), kind, depth);

        if need_meta {
            // Use symlink_metadata for symlinks to show the link's own size,
            // fall back to metadata for regular files/dirs
            let meta_result = if kind == NodeKind::Symlink {
                path.symlink_metadata()
            } else {
                path.metadata()
            };
            if let Ok(meta) = meta_result {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TreeConfig;
    use std::fs;

    fn default_config() -> TreeConfig {
        TreeConfig::default()
    }

    #[test]
    fn basic_directory_listing() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("file_a.txt"), "a").unwrap();
        fs::write(dir.path().join("file_b.txt"), "b").unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();

        let nodes = load_children_with_meta(dir.path(), 0, &default_config());
        let names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();

        assert!(names.contains(&"file_a.txt"));
        assert!(names.contains(&"file_b.txt"));
        assert!(names.contains(&"subdir"));
    }

    #[test]
    fn empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        let nodes = load_children_with_meta(dir.path(), 0, &default_config());
        assert!(nodes.is_empty());
    }

    #[test]
    fn dirs_first_ordering() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("aaa_file.txt"), "").unwrap();
        fs::create_dir(dir.path().join("zzz_dir")).unwrap();

        let config = TreeConfig {
            dirs_first: true,
            ..default_config()
        };
        let nodes = load_children_with_meta(dir.path(), 0, &config);

        // The directory should appear before the file
        let dir_pos = nodes.iter().position(|n| n.name == "zzz_dir").unwrap();
        let file_pos = nodes.iter().position(|n| n.name == "aaa_file.txt").unwrap();
        assert!(
            dir_pos < file_pos,
            "Directory should come before file when dirs_first=true"
        );
    }

    #[test]
    fn hidden_file_filtering() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".hidden"), "").unwrap();
        fs::write(dir.path().join("visible"), "").unwrap();

        // show_hidden=false should filter out dotfiles
        let config = TreeConfig {
            show_hidden: false,
            ..default_config()
        };
        let nodes = load_children_with_meta(dir.path(), 0, &config);
        let names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(
            !names.contains(&".hidden"),
            "Hidden file should be filtered"
        );
        assert!(names.contains(&"visible"));

        // show_hidden=true should include dotfiles
        let config_show = TreeConfig {
            show_hidden: true,
            ..default_config()
        };
        let nodes = load_children_with_meta(dir.path(), 0, &config_show);
        let names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&".hidden"), "Hidden file should be visible");
    }

    #[test]
    fn exclude_list_filters_entries() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join("keep.txt"), "").unwrap();

        let config = TreeConfig {
            exclude: vec![".git".to_string()],
            show_hidden: true,
            ..default_config()
        };
        let nodes = load_children_with_meta(dir.path(), 0, &config);
        let names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(!names.contains(&".git"), ".git should be excluded");
        assert!(names.contains(&"keep.txt"));
    }

    #[test]
    fn symlink_detected() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.txt");
        fs::write(&target, "data").unwrap();

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target, dir.path().join("link.txt")).unwrap();
            let nodes = load_children_with_meta(dir.path(), 0, &default_config());
            let link_node = nodes.iter().find(|n| n.name == "link.txt").unwrap();
            assert_eq!(link_node.kind, NodeKind::Symlink);
        }
    }

    #[test]
    fn depth_is_set_correctly() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("f.txt"), "").unwrap();

        let nodes = load_children_with_meta(dir.path(), 3, &default_config());
        assert!(!nodes.is_empty());
        for node in &nodes {
            assert_eq!(node.depth, 3);
        }
    }
}
