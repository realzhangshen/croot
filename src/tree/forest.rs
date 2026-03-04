use std::path::PathBuf;

use super::loader::load_children_with_meta;
use super::node::TreeNode;

#[allow(clippy::struct_excessive_bools)]
pub struct FileTree {
    pub nodes: Vec<TreeNode>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub root: PathBuf,
    pub show_hidden: bool,
    pub dirs_first: bool,
    pub exclude: Vec<String>,
    pub show_ignored: bool,
    pub compact_folders: bool,
    pub show_size: bool,
    pub show_modified: bool,
    /// Node indices currently rendered on screen, set by the renderer.
    /// Used to map mouse click rows to actual node indices.
    pub rendered_indices: Vec<usize>,
}

impl FileTree {
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    pub fn new(
        root: PathBuf,
        show_hidden: bool,
        dirs_first: bool,
        exclude: Vec<String>,
        show_ignored: bool,
        compact_folders: bool,
        show_size: bool,
        show_modified: bool,
    ) -> Self {
        let children = load_children_with_meta(
            &root, 0, show_hidden, dirs_first, &exclude, show_ignored,
            show_size, show_modified,
        );
        Self {
            nodes: children,
            cursor: 0,
            scroll_offset: 0,
            root,
            show_hidden,
            dirs_first,
            exclude,
            show_ignored,
            compact_folders,
            show_size,
            show_modified,
            rendered_indices: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn selected(&self) -> Option<&TreeNode> {
        self.nodes.get(self.cursor)
    }

    /// Expand a directory node: load its children and insert them after it.
    pub fn expand(&mut self, index: usize) {
        if index >= self.nodes.len() {
            return;
        }
        let node = &self.nodes[index];
        if !node.is_dir() || node.is_expanded {
            return;
        }

        let depth = node.depth + 1;
        let path = node.path.clone();

        let children = load_children_with_meta(
            &path,
            depth,
            self.show_hidden,
            self.dirs_first,
            &self.exclude,
            self.show_ignored,
            self.show_size,
            self.show_modified,
        );

        self.nodes[index].is_expanded = true;
        self.nodes[index].children_loaded = true;

        // Insert children right after the expanded node
        let insert_pos = index + 1;
        self.nodes.splice(insert_pos..insert_pos, children);
    }

    /// Collapse a directory node: remove all descendant nodes.
    pub fn collapse(&mut self, index: usize) {
        if index >= self.nodes.len() {
            return;
        }
        let node = &self.nodes[index];
        if !node.is_dir() || !node.is_expanded {
            return;
        }

        let parent_depth = node.depth;

        // Find the range of children to remove: all subsequent nodes with depth > parent_depth
        let start = index + 1;
        let mut end = start;
        while end < self.nodes.len() && self.nodes[end].depth > parent_depth {
            end += 1;
        }

        self.nodes.drain(start..end);
        self.nodes[index].is_expanded = false;
        self.nodes[index].children_loaded = false;

        // Adjust cursor if it was in the removed range
        if self.cursor >= end {
            self.cursor -= end - start;
        } else if self.cursor > index {
            self.cursor = index;
        }
    }

    /// Toggle expand/collapse on the current node.
    pub fn toggle(&mut self, index: usize) {
        if index >= self.nodes.len() {
            return;
        }

        if !self.nodes[index].is_dir() {
            return;
        }

        if self.nodes[index].is_expanded {
            self.collapse(index);
        } else {
            self.expand(index);
        }
    }

    /// Move cursor up.
    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor down.
    pub fn cursor_down(&mut self) {
        if self.cursor + 1 < self.nodes.len() {
            self.cursor += 1;
        }
    }

    /// Collapse current dir or move to parent.
    pub fn cursor_left(&mut self) {
        if let Some(node) = self.nodes.get(self.cursor) {
            if node.is_dir() && node.is_expanded {
                self.collapse(self.cursor);
                return;
            }
            // Move to parent: find the nearest node above with depth - 1
            let target_depth = node.depth.saturating_sub(1);
            for i in (0..self.cursor).rev() {
                if self.nodes[i].depth == target_depth && self.nodes[i].is_dir() {
                    self.cursor = i;
                    return;
                }
            }
        }
    }

    /// Expand current dir or move to first child.
    pub fn cursor_right(&mut self) {
        let cursor = self.cursor;
        if cursor >= self.nodes.len() {
            return;
        }

        let is_dir = self.nodes[cursor].is_dir();
        let was_expanded = self.nodes[cursor].is_expanded;

        if is_dir {
            if !was_expanded {
                self.expand(cursor);
            }
            // Move to first child if there is one
            let depth = self.nodes[cursor].depth;
            if cursor + 1 < self.nodes.len() && self.nodes[cursor + 1].depth > depth {
                self.cursor = cursor + 1;
            }
        }
    }

    /// Ensure the cursor is visible within the given viewport height.
    /// Note: when `compact_folders` is on, the renderer handles scroll adjustment
    /// via `build_visible_indices`. This method is used as fallback and by tests.
    #[allow(dead_code)]
    pub fn adjust_scroll(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        }
        if self.cursor >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor - viewport_height + 1;
        }
    }

    /// Return the visible slice of nodes for the current viewport.
    #[allow(dead_code)]
    pub fn visible_range(&self, viewport_height: usize) -> &[TreeNode] {
        let start = self.scroll_offset;
        let end = (start + viewport_height).min(self.nodes.len());
        &self.nodes[start..end]
    }

    /// Refresh expanded directories (re-read from filesystem).
    /// Preserves which directories were expanded by collecting their paths first.
    pub fn refresh(&mut self) {
        // Collect paths of expanded directories before rebuilding
        let expanded_paths: Vec<PathBuf> = self
            .nodes
            .iter()
            .filter(|n| n.is_dir() && n.is_expanded)
            .map(|n| n.path.clone())
            .collect();

        // Remember cursor path for restoration
        let cursor_path = self.nodes.get(self.cursor).map(|n| n.path.clone());

        // Re-read root from scratch
        self.nodes = load_children_with_meta(
            &self.root,
            0,
            self.show_hidden,
            self.dirs_first,
            &self.exclude,
            self.show_ignored,
            self.show_size,
            self.show_modified,
        );

        // Re-expand previously expanded dirs (forward scan, expanding shifts indices)
        let mut i = 0;
        while i < self.nodes.len() {
            if self.nodes[i].is_dir() && expanded_paths.contains(&self.nodes[i].path) {
                self.expand(i);
            }
            i += 1;
        }

        // Restore cursor position by path, or clamp to valid range
        if let Some(ref target) = cursor_path {
            self.cursor = self
                .nodes
                .iter()
                .position(|n| n.path == *target)
                .unwrap_or(0);
        }
        self.cursor = self.cursor.min(self.nodes.len().saturating_sub(1));
    }

    /// Check if a node at `index` is the last child of its parent.
    pub fn is_last_sibling(&self, index: usize) -> bool {
        let depth = self.nodes[index].depth;
        // Look at subsequent nodes: if the next node at the same depth or less doesn't exist
        // before a shallower node, this is the last sibling.
        for i in (index + 1)..self.nodes.len() {
            if self.nodes[i].depth <= depth {
                return self.nodes[i].depth < depth;
            }
        }
        true // last node at this depth
    }

    /// Count how many single-child directory nodes form a chain starting at `index`.
    /// Returns the number of intermediate dirs to skip (0 = no compaction).
    /// Only applies to expanded directory nodes that have exactly one child which is also
    /// an expanded directory.
    pub fn compact_chain_len(&self, index: usize) -> usize {
        if !self.compact_folders {
            return 0;
        }
        let node = &self.nodes[index];
        if !node.is_dir() || !node.is_expanded {
            return 0;
        }

        let mut count = 0;
        let mut cur = index;

        loop {
            let child_start = cur + 1;
            if child_start >= self.nodes.len() {
                break;
            }
            let child = &self.nodes[child_start];
            if child.depth != self.nodes[cur].depth + 1 {
                break;
            }
            // Check this dir has exactly one child (the next node after child_start
            // must either not exist or have depth <= child's parent's depth + 1)
            let second_child = child_start + 1;
            let has_single_child = if second_child >= self.nodes.len() {
                true
            } else {
                // If the second node is at same depth as child, there are multiple children
                self.nodes[second_child].depth <= self.nodes[cur].depth
                    || (child.is_dir()
                        && child.is_expanded
                        && self.nodes[second_child].depth > child.depth)
            };

            if !has_single_child || !child.is_dir() || !child.is_expanded {
                break;
            }

            count += 1;
            cur = child_start;
        }

        count
    }

    /// Build the compacted display name for a node at `index` that has `chain_len`
    /// intermediate directories merged into it.
    pub fn compact_display_name(&self, index: usize, chain_len: usize) -> String {
        let mut parts = vec![self.nodes[index].name.clone()];
        let mut cur = index;
        for _ in 0..chain_len {
            cur += 1;
            parts.push(self.nodes[cur].name.clone());
        }
        parts.join("/") + "/"
    }

    /// For rendering tree connectors: determine which depths have a continuing vertical line.
    /// Returns a Vec of bools where index = depth, true = has more siblings below.
    pub fn connector_guides(&self, index: usize) -> Vec<bool> {
        let node = &self.nodes[index];
        let mut guides = vec![false; node.depth];

        // For each ancestor depth, check if there are more nodes at that depth after this index
        for (d, guide) in guides.iter_mut().enumerate() {
            for i in (index + 1)..self.nodes.len() {
                if self.nodes[i].depth < d {
                    break;
                }
                if self.nodes[i].depth == d {
                    *guide = true;
                    break;
                }
            }
        }

        guides
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::node::{NodeKind, TreeNode};

    /// Build a FileTree from a list of (name, kind, depth) tuples — no filesystem needed.
    fn tree_from(entries: &[(&str, NodeKind, usize)]) -> FileTree {
        let nodes = entries
            .iter()
            .map(|(name, kind, depth)| TreeNode::new(PathBuf::from(name), *kind, *depth))
            .collect();
        FileTree {
            nodes,
            cursor: 0,
            scroll_offset: 0,
            root: PathBuf::from("/tmp/test"),
            show_hidden: true,
            dirs_first: true,
            exclude: vec![],
            show_ignored: false,
            compact_folders: false,
            show_size: false,
            show_modified: false,
            rendered_indices: Vec::new(),
        }
    }

    fn names(tree: &FileTree) -> Vec<&str> {
        tree.nodes.iter().map(|n| n.name.as_str()).collect()
    }

    #[test]
    fn expand_out_of_bounds_is_noop() {
        let mut tree = tree_from(&[("a.txt", NodeKind::File, 0)]);
        tree.expand(99); // should not panic
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn collapse_out_of_bounds_is_noop() {
        let mut tree = tree_from(&[("a.txt", NodeKind::File, 0)]);
        tree.collapse(99); // should not panic
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn expand_file_is_noop() {
        let mut tree = tree_from(&[("a.txt", NodeKind::File, 0)]);
        tree.expand(0);
        assert_eq!(tree.len(), 1);
        assert!(!tree.nodes[0].is_expanded);
    }

    #[test]
    fn collapse_unexpanded_dir_is_noop() {
        let mut tree = tree_from(&[("src", NodeKind::Directory, 0)]);
        tree.collapse(0);
        assert!(!tree.nodes[0].is_expanded);
    }

    #[test]
    fn collapse_removes_children_and_adjusts_cursor() {
        let mut tree = tree_from(&[
            ("src", NodeKind::Directory, 0),
            ("main.rs", NodeKind::File, 1),
            ("lib.rs", NodeKind::File, 1),
            ("README", NodeKind::File, 0),
        ]);
        tree.nodes[0].is_expanded = true;
        tree.cursor = 3; // pointing at README

        tree.collapse(0);

        assert_eq!(names(&tree), vec!["src", "README"]);
        // cursor was at index 3 (README), which was beyond the removed range (1..3)
        // so it should shift down by 2
        assert_eq!(tree.cursor, 1);
    }

    #[test]
    fn collapse_moves_cursor_to_parent_if_inside_children() {
        let mut tree = tree_from(&[
            ("src", NodeKind::Directory, 0),
            ("main.rs", NodeKind::File, 1),
            ("lib.rs", NodeKind::File, 1),
        ]);
        tree.nodes[0].is_expanded = true;
        tree.cursor = 2; // pointing at lib.rs (inside collapsed range)

        tree.collapse(0);

        assert_eq!(names(&tree), vec!["src"]);
        assert_eq!(tree.cursor, 0); // snapped to parent
    }

    #[test]
    fn cursor_up_at_top_stays() {
        let mut tree = tree_from(&[("a", NodeKind::File, 0), ("b", NodeKind::File, 0)]);
        tree.cursor = 0;
        tree.cursor_up();
        assert_eq!(tree.cursor, 0);
    }

    #[test]
    fn cursor_down_at_bottom_stays() {
        let mut tree = tree_from(&[("a", NodeKind::File, 0), ("b", NodeKind::File, 0)]);
        tree.cursor = 1;
        tree.cursor_down();
        assert_eq!(tree.cursor, 1);
    }

    #[test]
    fn cursor_up_down_navigates() {
        let mut tree = tree_from(&[
            ("a", NodeKind::File, 0),
            ("b", NodeKind::File, 0),
            ("c", NodeKind::File, 0),
        ]);
        tree.cursor_down();
        assert_eq!(tree.cursor, 1);
        tree.cursor_down();
        assert_eq!(tree.cursor, 2);
        tree.cursor_up();
        assert_eq!(tree.cursor, 1);
    }

    #[test]
    fn toggle_on_empty_tree_is_noop() {
        let mut tree = tree_from(&[]);
        tree.toggle(0); // should not panic
    }

    #[test]
    fn toggle_on_file_is_noop() {
        let mut tree = tree_from(&[("a.txt", NodeKind::File, 0)]);
        tree.toggle(0);
        assert!(!tree.nodes[0].is_expanded);
    }

    #[test]
    fn is_last_sibling_single_node() {
        let tree = tree_from(&[("a.txt", NodeKind::File, 0)]);
        assert!(tree.is_last_sibling(0));
    }

    #[test]
    fn is_last_sibling_among_peers() {
        let tree = tree_from(&[
            ("a", NodeKind::File, 0),
            ("b", NodeKind::File, 0),
            ("c", NodeKind::File, 0),
        ]);
        assert!(!tree.is_last_sibling(0));
        assert!(!tree.is_last_sibling(1));
        assert!(tree.is_last_sibling(2));
    }

    #[test]
    fn adjust_scroll_keeps_cursor_visible() {
        let mut tree = tree_from(&[
            ("a", NodeKind::File, 0),
            ("b", NodeKind::File, 0),
            ("c", NodeKind::File, 0),
            ("d", NodeKind::File, 0),
            ("e", NodeKind::File, 0),
        ]);
        tree.cursor = 4;
        tree.adjust_scroll(3); // viewport of 3 lines
        assert!(tree.scroll_offset <= 2); // cursor 4 visible in window of 3
    }

    // ── Compact folders tests ───────────────────────────────────────────

    fn tree_from_compact(entries: &[(&str, NodeKind, usize, bool)]) -> FileTree {
        let nodes = entries
            .iter()
            .map(|(name, kind, depth, expanded)| {
                let mut node = TreeNode::new(PathBuf::from(name), *kind, *depth);
                node.is_expanded = *expanded;
                node.children_loaded = *expanded;
                node
            })
            .collect();
        FileTree {
            nodes,
            cursor: 0,
            scroll_offset: 0,
            root: PathBuf::from("/tmp/test"),
            show_hidden: true,
            dirs_first: true,
            exclude: vec![],
            show_ignored: false,
            compact_folders: true,
            show_size: false,
            show_modified: false,
            rendered_indices: Vec::new(),
        }
    }

    #[test]
    fn compact_chain_single_child_dirs() {
        // src/ (expanded) → utils/ (expanded) → helpers/ (expanded) → format.rs
        let tree = tree_from_compact(&[
            ("src", NodeKind::Directory, 0, true),
            ("utils", NodeKind::Directory, 1, true),
            ("helpers", NodeKind::Directory, 2, true),
            ("format.rs", NodeKind::File, 3, false),
        ]);
        // src has one child (utils), utils has one child (helpers) → chain of 2
        assert_eq!(tree.compact_chain_len(0), 2);
        assert_eq!(
            tree.compact_display_name(0, 2),
            "src/utils/helpers/"
        );
    }

    #[test]
    fn compact_chain_stops_at_multiple_children() {
        // src/ (expanded) → main.rs, lib.rs
        let tree = tree_from_compact(&[
            ("src", NodeKind::Directory, 0, true),
            ("main.rs", NodeKind::File, 1, false),
            ("lib.rs", NodeKind::File, 1, false),
        ]);
        assert_eq!(tree.compact_chain_len(0), 0);
    }

    #[test]
    fn compact_chain_stops_at_file_child() {
        // src/ (expanded) → main.rs
        let tree = tree_from_compact(&[
            ("src", NodeKind::Directory, 0, true),
            ("main.rs", NodeKind::File, 1, false),
        ]);
        assert_eq!(tree.compact_chain_len(0), 0);
    }

    #[test]
    fn compact_disabled_returns_zero() {
        let mut tree = tree_from_compact(&[
            ("src", NodeKind::Directory, 0, true),
            ("utils", NodeKind::Directory, 1, true),
            ("format.rs", NodeKind::File, 2, false),
        ]);
        tree.compact_folders = false;
        assert_eq!(tree.compact_chain_len(0), 0);
    }

    #[test]
    fn compact_chain_on_file_returns_zero() {
        let tree = tree_from_compact(&[("a.txt", NodeKind::File, 0, false)]);
        assert_eq!(tree.compact_chain_len(0), 0);
    }
}
