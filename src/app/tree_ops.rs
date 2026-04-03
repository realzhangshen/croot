use super::*;

impl App {
    pub(super) fn handle_tree_action(&mut self, action: &Action) {
        match action {
            Action::CursorUp => {
                if self.focus == FocusPane::Preview {
                    self.preview_state.scroll_up(1);
                } else {
                    self.tree.cursor_up();
                }
            }
            Action::CursorDown => {
                if self.focus == FocusPane::Preview {
                    self.preview_state.scroll_down(1);
                } else {
                    self.tree.cursor_down();
                }
            }
            Action::CursorLeft => {
                if self.focus == FocusPane::Tree {
                    self.tree.cursor_left();
                }
            }
            Action::CursorRight => {
                if self.focus == FocusPane::Tree {
                    self.tree.cursor_right();
                    self.reapply_git();
                    self.refresh_search_state();
                }
            }
            Action::Toggle => {
                let idx = self.tree.cursor;
                self.tree.toggle(idx);
                self.reapply_git();
                self.refresh_search_state();
            }
            Action::ScrollUp(n) => {
                for _ in 0..*n {
                    self.tree.cursor_up();
                }
            }
            Action::ScrollDown(n) => {
                for _ in 0..*n {
                    self.tree.cursor_down();
                }
            }
            Action::GotoTop => {
                if self.focus == FocusPane::Preview {
                    self.preview_state.scroll_offset = 0;
                } else {
                    self.tree.cursor = 0;
                }
            }
            Action::GotoBottom => {
                if self.focus == FocusPane::Preview {
                    self.preview_state.scroll_offset =
                        self.preview_state.total_lines.saturating_sub(1);
                } else if !self.tree.is_empty() {
                    // Use rendered_indices (visible nodes) when available;
                    // fall back to the full displayable indices list.
                    let last = self
                        .tree
                        .rendered_indices
                        .last()
                        .copied()
                        .unwrap_or_else(|| {
                            let indices = self.tree.build_displayable_indices();
                            indices.last().copied().unwrap_or(self.tree.len() - 1)
                        });
                    self.tree.cursor = last;
                }
            }
            _ => {}
        }
    }

    pub(super) fn handle_click_row(
        &mut self,
        row: u16,
        preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) {
        self.focus = FocusPane::Tree;
        self.preview_state.selection.clear();
        let row_idx = row as usize;
        let idx = if row_idx < self.tree.rendered_indices.len() {
            self.tree.rendered_indices[row_idx]
        } else {
            return;
        };
        if idx < self.tree.len() {
            let already_selected = self.tree.cursor == idx;
            self.tree.cursor = idx;
            if self.tree.nodes[idx].is_dir() {
                self.tree.toggle(idx);
                self.reapply_git();
                self.refresh_search_state();
                if self.preview_visible {
                    self.trigger_preview_load(preview_tx);
                }
            } else if already_selected && self.preview_visible {
                self.preview_visible = false;
                self.focus = FocusPane::Tree;
            } else {
                self.preview_visible = true;
                self.trigger_preview_load(preview_tx);
            }
        }
    }

    pub(super) fn update_hover(&mut self, col: u16, row: u16) {
        if self.preview_area_x.is_some_and(|px| col >= px) {
            self.hover_row = None;
            return;
        }
        if row >= self.tree_area_y && row < self.tree_area_y + self.tree_area_height {
            let relative_row = (row - self.tree_area_y) as usize;
            if relative_row < self.tree.rendered_indices.len() {
                self.hover_row = Some(relative_row);
            } else {
                self.hover_row = None;
            }
        } else {
            self.hover_row = None;
        }
    }
}
