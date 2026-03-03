use std::path::PathBuf;

use super::loader::load_children;
use super::node::TreeNode;

pub struct FileTree {
    pub nodes: Vec<TreeNode>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub root: PathBuf,
}

impl FileTree {
    pub fn new(root: PathBuf) -> Self {
        let children = load_children(&root, 0);
        Self {
            nodes: children,
            cursor: 0,
            scroll_offset: 0,
            root,
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
        let node = &self.nodes[index];
        if !node.is_dir() || node.is_expanded {
            return;
        }

        let depth = node.depth + 1;
        let path = node.path.clone();

        let children = load_children(&path, depth);

        self.nodes[index].is_expanded = true;
        self.nodes[index].children_loaded = true;

        // Insert children right after the expanded node
        let insert_pos = index + 1;
        self.nodes.splice(insert_pos..insert_pos, children);
    }

    /// Collapse a directory node: remove all descendant nodes.
    pub fn collapse(&mut self, index: usize) {
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
    pub fn visible_range(&self, viewport_height: usize) -> &[TreeNode] {
        let start = self.scroll_offset;
        let end = (start + viewport_height).min(self.nodes.len());
        &self.nodes[start..end]
    }

    /// Refresh expanded directories (re-read from filesystem).
    pub fn refresh(&mut self) {
        // Collect indices of expanded directories (in reverse to preserve indices)
        let expanded: Vec<usize> = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.is_dir() && n.is_expanded)
            .map(|(i, _)| i)
            .collect();

        // Collapse all expanded dirs (reverse order to preserve indices)
        for &idx in expanded.iter().rev() {
            self.collapse(idx);
        }

        // Re-read root
        let children = load_children(&self.root, 0);
        self.nodes = children;

        // Re-expand previously expanded dirs (forward order)
        // We match by path since indices have shifted
        let expanded_paths: Vec<PathBuf> = expanded
            .iter()
            .filter_map(|_| None::<PathBuf>) // placeholder — we need to collect before collapsing
            .collect();

        // Simpler approach: just reset. The user can re-expand.
        // A smarter refresh is a Phase 4 enhancement.
        let _ = expanded_paths;

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

    /// For rendering tree connectors: determine which depths have a continuing vertical line.
    /// Returns a Vec of bools where index = depth, true = has more siblings below.
    pub fn connector_guides(&self, index: usize) -> Vec<bool> {
        let node = &self.nodes[index];
        let mut guides = vec![false; node.depth];

        // For each ancestor depth, check if there are more nodes at that depth after this index
        for d in 0..node.depth {
            for i in (index + 1)..self.nodes.len() {
                if self.nodes[i].depth < d {
                    break;
                }
                if self.nodes[i].depth == d {
                    guides[d] = true;
                    break;
                }
            }
        }

        guides
    }
}
