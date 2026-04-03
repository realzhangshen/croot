use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::config::TreeConfig;

use super::loader::load_children_with_meta;
use super::node::{NodeKind, TreeNode};

pub struct FileTree {
    pub nodes: Vec<TreeNode>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub root: PathBuf,
    pub config: TreeConfig,
    /// Node indices currently rendered on screen, set by the renderer.
    /// Used to map mouse click rows to actual node indices.
    pub rendered_indices: Vec<usize>,
    /// Cached count of visible file nodes.
    pub file_count: usize,
    /// Cached count of visible directory nodes.
    pub dir_count: usize,
    /// Cached compact chain lengths (`node_index` → `chain_len`), invalidated on mutation.
    pub(crate) chain_len_cache: HashMap<usize, usize>,
    /// Whether `chain_len_cache` is valid.
    pub(crate) chain_cache_valid: bool,
    /// Cached list of displayable node indices (skipping compact chain intermediates).
    pub(crate) displayable_cache: Vec<usize>,
    /// Whether `displayable_cache` is valid.
    pub(crate) displayable_cache_valid: bool,
    /// Cached connector guides for all nodes.
    pub(crate) guides_cache: Vec<Vec<bool>>,
    /// Whether `guides_cache` is valid.
    pub(crate) guides_cache_valid: bool,
}

impl FileTree {
    pub fn new(root: PathBuf, config: TreeConfig) -> Self {
        let children = load_children_with_meta(&root, 0, &config);
        let file_count = children.iter().filter(|n| n.kind == NodeKind::File).count();
        let dir_count = children
            .iter()
            .filter(|n| n.kind == NodeKind::Directory)
            .count();
        Self {
            nodes: children,
            cursor: 0,
            scroll_offset: 0,
            root,
            config,
            rendered_indices: Vec::new(),
            file_count,
            dir_count,
            chain_len_cache: HashMap::new(),
            chain_cache_valid: false,
            displayable_cache: Vec::new(),
            displayable_cache_valid: false,
            guides_cache: Vec::new(),
            guides_cache_valid: false,
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

    /// Mark the chain length cache as stale. Must be called after any node list mutation.
    fn invalidate_chain_cache(&mut self) {
        self.chain_cache_valid = false;
        self.displayable_cache_valid = false;
        self.guides_cache_valid = false;
    }

    /// Get the compact chain length for a node, using the cache if available.
    /// If the cache is stale, recomputes all chain lengths in a single O(N) pass.
    ///
    /// **Important:** Only valid for displayable-head indices (i.e., indices that
    /// appear in `rendered_indices`). Mid-chain indices are not cached and will
    /// return 0 with a debug assertion failure.
    pub fn cached_chain_len(&mut self, index: usize) -> usize {
        if !self.chain_cache_valid {
            self.chain_len_cache.clear();
            let mut i = 0;
            while i < self.nodes.len() {
                let chain = self.compact_chain_len(i);
                self.chain_len_cache.insert(i, chain);
                i += chain + 1;
            }
            self.chain_cache_valid = true;
        }
        let result = self.chain_len_cache.get(&index).copied().unwrap_or(0);
        debug_assert!(
            result > 0 || self.chain_len_cache.contains_key(&index),
            "cached_chain_len called with mid-chain index {index}; use displayable-head indices only"
        );
        result
    }

    /// Get displayable indices, using cache if valid. Rebuilds in O(N) if stale.
    pub fn cached_displayable_indices(&mut self) -> &[usize] {
        if !self.displayable_cache_valid {
            self.displayable_cache.clear();
            let mut i = 0;
            while i < self.nodes.len() {
                self.displayable_cache.push(i);
                let chain = self.cached_chain_len(i);
                i += chain + 1;
            }
            self.displayable_cache_valid = true;
        }
        &self.displayable_cache
    }

    /// Get connector guides, using cache if valid.
    pub fn cached_guides(&mut self) -> &[Vec<bool>] {
        if !self.guides_cache_valid {
            self.guides_cache = self.precompute_all_guides();
            self.guides_cache_valid = true;
        }
        &self.guides_cache
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

        let children = load_children_with_meta(&path, depth, &self.config);

        self.nodes[index].is_expanded = true;
        self.nodes[index].children_loaded = true;

        // Update cached counts
        for child in &children {
            match child.kind {
                NodeKind::File => self.file_count += 1,
                NodeKind::Directory => self.dir_count += 1,
                NodeKind::Symlink => self.file_count += 1,
            }
        }

        // Insert children right after the expanded node
        let insert_pos = index + 1;
        self.nodes.splice(insert_pos..insert_pos, children);
        self.invalidate_chain_cache();
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

        // Update cached counts before draining
        for node in &self.nodes[start..end] {
            match node.kind {
                NodeKind::File | NodeKind::Symlink => self.file_count -= 1,
                NodeKind::Directory => self.dir_count -= 1,
            }
        }

        let removed_count = end - start;
        self.nodes.drain(start..end);
        self.nodes[index].is_expanded = false;
        self.nodes[index].children_loaded = false;
        self.invalidate_chain_cache();

        // Adjust cursor if it was in the removed range
        if self.cursor >= end {
            self.cursor -= removed_count;
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
            if node.depth == 0 {
                return; // already at root level, no parent to navigate to
            }
            let target_depth = node.depth - 1;
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

    /// Recompute cached file/directory counts from the current nodes.
    fn recount(&mut self) {
        self.file_count = self
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::File || n.kind == NodeKind::Symlink)
            .count();
        self.dir_count = self
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Directory)
            .count();
    }

    /// Collapse all expanded directories back to the root level.
    pub fn collapse_all(&mut self) {
        let cursor_path = self.nodes.get(self.cursor).map(|n| n.path.clone());

        self.nodes = load_children_with_meta(&self.root, 0, &self.config);
        self.recount();
        self.invalidate_chain_cache();
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

    /// Refresh expanded directories (re-read from filesystem).
    /// Preserves which directories were expanded by collecting their paths first.
    pub fn refresh(&mut self) {
        // Collect paths of expanded directories before rebuilding (HashSet for O(1) lookup)
        let expanded_paths: HashSet<PathBuf> = self
            .nodes
            .iter()
            .filter(|n| n.is_dir() && n.is_expanded)
            .map(|n| n.path.clone())
            .collect();

        // Remember cursor path for restoration
        let cursor_path = self.nodes.get(self.cursor).map(|n| n.path.clone());

        // Re-read root from scratch
        self.nodes = load_children_with_meta(&self.root, 0, &self.config);

        // Re-expand previously expanded dirs (forward scan, expanding shifts indices)
        let mut i = 0;
        while i < self.nodes.len() {
            if self.nodes[i].is_dir() && expanded_paths.contains(&self.nodes[i].path) {
                self.expand(i);
            }
            i += 1;
        }

        self.recount();

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
        if index >= self.nodes.len() {
            return true;
        }
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
        if !self.config.compact_folders {
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
            // Check this dir has exactly one direct child by scanning past
            // the first child's entire subtree to see if a sibling exists.
            let target_depth = self.nodes[cur].depth + 1;
            let has_single_child = {
                let mut j = child_start + 1;
                while j < self.nodes.len() && self.nodes[j].depth > target_depth {
                    j += 1;
                }
                // Single child if we hit end-of-list or a node shallower than target
                j >= self.nodes.len() || self.nodes[j].depth < target_depth
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

    /// Build a list of node indices that are actually displayable on screen,
    /// skipping intermediate nodes in compact chains.
    pub fn build_displayable_indices(&mut self) -> Vec<usize> {
        self.cached_displayable_indices().to_vec()
    }

    /// Get the display name for a node, using the compact chain name if applicable.
    /// Uses the cache when available for O(1) lookups.
    pub fn compact_display_name_for(&mut self, idx: usize) -> String {
        let chain_len = self.cached_chain_len(idx);
        if chain_len > 0 {
            self.compact_display_name(idx, chain_len)
        } else {
            self.nodes[idx].name.clone()
        }
    }

    /// Navigate the tree to a target path, expanding parent directories as needed.
    /// Returns true if the path was found and cursor set.
    pub fn navigate_to_path(&mut self, target: &std::path::Path) -> bool {
        // First check if target is already visible in nodes
        if let Some(idx) = self.nodes.iter().position(|n| n.path == target) {
            self.cursor = idx;
            return true;
        }

        // Expand parent directories along the path
        let rel = match target.strip_prefix(&self.root) {
            Ok(r) => r,
            Err(_) => return false,
        };

        let mut current_path = self.root.clone();
        for component in rel.components() {
            current_path = current_path.join(component.as_os_str());

            if let Some(idx) = self.nodes.iter().position(|n| n.path == current_path) {
                if current_path == target {
                    self.cursor = idx;
                    return true;
                }
                if self.nodes[idx].is_dir() && !self.nodes[idx].is_expanded {
                    self.expand(idx);
                }
            } else {
                return false;
            }
        }

        false
    }

    /// Create a fresh `FileTree` from disk, re-expanding the given paths.
    /// Pure function suitable for calling on a background thread.
    pub fn snapshot_refresh(
        root: PathBuf,
        config: TreeConfig,
        expanded_paths: &HashSet<PathBuf>,
    ) -> Self {
        let mut tree = Self::new(root, config);
        let mut i = 0;
        while i < tree.nodes.len() {
            if tree.nodes[i].is_dir() && expanded_paths.contains(&tree.nodes[i].path) {
                tree.expand(i);
            }
            i += 1;
        }
        tree.recount();
        tree
    }

    /// Precompute connector guides for all nodes in O(N) total using a reverse scan.
    /// Returns a Vec where entry[i] is the guides Vec for node i.
    pub fn precompute_all_guides(&self) -> Vec<Vec<bool>> {
        let n = self.nodes.len();
        if n == 0 {
            return Vec::new();
        }

        let max_depth = self.nodes.iter().map(|n| n.depth).max().unwrap_or(0);

        // has_more[d] = true if there is a node at depth d somewhere after current position
        let mut has_more = vec![false; max_depth + 1];

        // Build guides in reverse, then reverse the whole thing
        let mut rev_results: Vec<Vec<bool>> = Vec::with_capacity(n);

        for i in (0..n).rev() {
            let depth = self.nodes[i].depth;
            let mut guides = vec![false; depth];
            for (d, guide) in guides.iter_mut().enumerate() {
                *guide = has_more[d];
            }
            rev_results.push(guides);

            // Mark this depth as having a node (for nodes above this one)
            has_more[depth] = true;
            // Any depth deeper than this node resets (no continuation above a shallower node)
            for item in &mut has_more[(depth + 1)..=max_depth] {
                *item = false;
            }
        }

        rev_results.reverse();
        rev_results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::node::TreeNode;

    /// Build a `FileTree` from a list of (name, kind, depth) tuples — no filesystem needed.
    fn tree_from(entries: &[(&str, NodeKind, usize)]) -> FileTree {
        let nodes: Vec<TreeNode> = entries
            .iter()
            .map(|(name, kind, depth)| TreeNode::new(PathBuf::from(name), *kind, *depth))
            .collect();
        let file_count = nodes
            .iter()
            .filter(|n| n.kind == NodeKind::File || n.kind == NodeKind::Symlink)
            .count();
        let dir_count = nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Directory)
            .count();
        let config = TreeConfig {
            compact_folders: false,
            ..TreeConfig::default()
        };
        FileTree {
            nodes,
            cursor: 0,
            scroll_offset: 0,
            root: PathBuf::from("/tmp/test"),
            config,
            rendered_indices: Vec::new(),
            file_count,
            dir_count,
            chain_len_cache: HashMap::new(),
            chain_cache_valid: false,
            displayable_cache: Vec::new(),
            displayable_cache_valid: false,
            guides_cache: Vec::new(),
            guides_cache_valid: false,
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
    fn is_last_sibling_out_of_bounds_returns_true() {
        let tree = tree_from(&[("a.txt", NodeKind::File, 0)]);
        assert!(tree.is_last_sibling(99)); // should not panic
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

    #[test]
    fn cursor_left_at_depth_0_stays_put() {
        let mut tree = tree_from(&[
            ("dir_a", NodeKind::Directory, 0),
            ("file_b", NodeKind::File, 0),
        ]);
        tree.cursor = 1; // on file_b at depth 0
        tree.cursor_left();
        // Should not jump to dir_a — there's no parent at depth 0
        assert_eq!(tree.cursor, 1);
    }

    // ── Compact folders tests ───────────────────────────────────────────

    fn tree_from_compact(entries: &[(&str, NodeKind, usize, bool)]) -> FileTree {
        let nodes: Vec<TreeNode> = entries
            .iter()
            .map(|(name, kind, depth, expanded)| {
                let mut node = TreeNode::new(PathBuf::from(name), *kind, *depth);
                node.is_expanded = *expanded;
                node.children_loaded = *expanded;
                node
            })
            .collect();
        let file_count = nodes
            .iter()
            .filter(|n| n.kind == NodeKind::File || n.kind == NodeKind::Symlink)
            .count();
        let dir_count = nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Directory)
            .count();
        FileTree {
            nodes,
            cursor: 0,
            scroll_offset: 0,
            root: PathBuf::from("/tmp/test"),
            config: TreeConfig::default(), // compact_folders defaults to true
            rendered_indices: Vec::new(),
            file_count,
            dir_count,
            chain_len_cache: HashMap::new(),
            chain_cache_valid: false,
            displayable_cache: Vec::new(),
            displayable_cache_valid: false,
            guides_cache: Vec::new(),
            guides_cache_valid: false,
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
        assert_eq!(tree.compact_display_name(0, 2), "src/utils/helpers/");
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
        tree.config.compact_folders = false;
        assert_eq!(tree.compact_chain_len(0), 0);
    }

    #[test]
    fn compact_chain_on_file_returns_zero() {
        let tree = tree_from_compact(&[("a.txt", NodeKind::File, 0, false)]);
        assert_eq!(tree.compact_chain_len(0), 0);
    }

    #[test]
    fn compact_chain_stops_when_sibling_after_subtree() {
        // src/ (expanded) → utils/ (expanded, has children) + other/ (sibling)
        let tree = tree_from_compact(&[
            ("src", NodeKind::Directory, 0, true),
            ("utils", NodeKind::Directory, 1, true),
            ("helpers", NodeKind::Directory, 2, true),
            ("format.rs", NodeKind::File, 3, false),
            ("other", NodeKind::Directory, 1, true),
            ("stuff.rs", NodeKind::File, 2, false),
        ]);
        // src has two children (utils and other), so no compaction
        assert_eq!(tree.compact_chain_len(0), 0);
    }

    // ── Displayable & guides cache tests ───────────────────────────────

    #[test]
    fn displayable_cache_invalidated_on_expand() {
        // Build a tree with a dir (not expanded) and a file, compact_folders on
        let mut tree = tree_from_compact(&[
            ("src", NodeKind::Directory, 0, false),
            ("readme.txt", NodeKind::File, 0, false),
        ]);

        // Warm cache
        let indices_before = tree.cached_displayable_indices().to_vec();
        assert_eq!(indices_before, vec![0, 1]);
        assert!(tree.displayable_cache_valid);

        // Simulate expand by manually marking expanded and inserting a child,
        // then calling invalidate via collapse/expand path.
        // We can't call expand() because it hits the filesystem; instead
        // we manually mutate and call invalidate_chain_cache.
        tree.nodes[0].is_expanded = true;
        tree.nodes[0].children_loaded = true;
        let child = TreeNode::new(PathBuf::from("main.rs"), NodeKind::File, 1);
        tree.nodes.insert(1, child);
        tree.invalidate_chain_cache();

        // Cache should be invalid now
        assert!(!tree.displayable_cache_valid);

        // Getting cached indices should rebuild
        let indices_after = tree.cached_displayable_indices().to_vec();
        assert_eq!(indices_after, vec![0, 1, 2]);
        assert!(tree.displayable_cache_valid);
    }

    #[test]
    fn guides_cache_invalidated_on_collapse() {
        // Build a tree with expanded dir + children
        let mut tree = tree_from_compact(&[
            ("src", NodeKind::Directory, 0, true),
            ("main.rs", NodeKind::File, 1, false),
            ("lib.rs", NodeKind::File, 1, false),
            ("readme.txt", NodeKind::File, 0, false),
        ]);

        // Warm guides cache
        let guides_before = tree.cached_guides().to_vec();
        assert!(!guides_before.is_empty());
        assert!(tree.guides_cache_valid);

        // Collapse src — removes children, invalidates caches
        tree.collapse(0);

        // Cache should be invalid
        assert!(!tree.guides_cache_valid);

        // Rebuild should reflect new structure
        let guides_after = tree.cached_guides().to_vec();
        // After collapse: ["src", "readme.txt"] — 2 nodes
        assert_eq!(guides_after.len(), 2);
        assert!(tree.guides_cache_valid);
    }

    #[test]
    fn cached_displayable_matches_fresh_computation() {
        // Build a tree with compact chain: src/ → utils/ → format.rs, plus a sibling
        let mut tree = tree_from_compact(&[
            ("src", NodeKind::Directory, 0, true),
            ("utils", NodeKind::Directory, 1, true),
            ("format.rs", NodeKind::File, 2, false),
            ("readme.txt", NodeKind::File, 0, false),
        ]);

        // Manually compute expected displayable indices:
        // Index 0: src (chain_len=1, skips utils at index 1)
        // Index 2: format.rs (chain_len=0)
        // Index 3: readme.txt (chain_len=0)
        let cached = tree.cached_displayable_indices().to_vec();
        assert_eq!(cached, vec![0, 2, 3]);

        // build_displayable_indices should return the same
        let fresh = tree.build_displayable_indices();
        assert_eq!(cached, fresh);
    }

    // ── snapshot_refresh tests ─────────────────────────────────────────

    #[test]
    fn snapshot_refresh_preserves_expanded_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("a/b")).unwrap();
        std::fs::write(tmp.path().join("a/b/c.txt"), "c").unwrap();
        std::fs::write(tmp.path().join("d.txt"), "d").unwrap();

        let config = TreeConfig::default();
        let mut tree = FileTree::new(tmp.path().to_path_buf(), config.clone());
        let a_idx = tree.nodes.iter().position(|n| n.name == "a").unwrap();
        tree.expand(a_idx);

        let expanded: HashSet<PathBuf> = tree
            .nodes
            .iter()
            .filter(|n| n.is_dir() && n.is_expanded)
            .map(|n| n.path.clone())
            .collect();

        let refreshed = FileTree::snapshot_refresh(tmp.path().to_path_buf(), config, &expanded);

        let a = refreshed.nodes.iter().find(|n| n.name == "a").unwrap();
        assert!(a.is_expanded);
        assert!(refreshed.nodes.iter().any(|n| n.name == "b"));
    }

    #[test]
    fn snapshot_refresh_equivalent_to_refresh() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("x")).unwrap();
        std::fs::write(tmp.path().join("x/y.txt"), "y").unwrap();
        std::fs::write(tmp.path().join("z.txt"), "z").unwrap();

        let config = TreeConfig::default();
        let mut original = FileTree::new(tmp.path().to_path_buf(), config.clone());
        // expand "x"
        let x_idx = original.nodes.iter().position(|n| n.name == "x").unwrap();
        original.expand(x_idx);

        let expanded: HashSet<PathBuf> = original
            .nodes
            .iter()
            .filter(|n| n.is_dir() && n.is_expanded)
            .map(|n| n.path.clone())
            .collect();

        let snapshot = FileTree::snapshot_refresh(tmp.path().to_path_buf(), config, &expanded);

        // Same node count
        assert_eq!(original.nodes.len(), snapshot.nodes.len());
    }
}
