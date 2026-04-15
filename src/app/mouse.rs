use super::*;

impl App {
    pub(super) fn handle_context_menu_mouse(
        &mut self,
        mouse: crossterm::event::MouseEvent,
        preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) -> PostAction {
        use crossterm::event::{MouseButton, MouseEventKind};

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(ref menu) = self.ui.context_menu {
                    let tw = self.main_area_width;
                    let th = self.tree_area_y + self.tree_area_height + 1;
                    if menu.contains(mouse.column, mouse.row, tw, th) {
                        if let Some(idx) = menu.row_to_item(mouse.row, tw, th) {
                            let menu_action = menu.items[idx].action.clone();
                            let node_idx = menu.node_idx;
                            self.ui.context_menu = None;
                            self.ui.input_mode = InputMode::Normal;
                            return self.execute_menu_action(&menu_action, node_idx, preview_tx);
                        }
                    } else {
                        self.ui.context_menu = None;
                        self.ui.input_mode = InputMode::Normal;
                    }
                }
            }
            MouseEventKind::Moved => {
                if let Some(ref mut menu) = self.ui.context_menu {
                    let tw = self.main_area_width;
                    let th = self.tree_area_y + self.tree_area_height + 1;
                    if let Some(idx) = menu.row_to_item(mouse.row, tw, th) {
                        menu.selected = idx;
                    }
                }
            }
            _ => {
                // Any other click closes the menu
                if matches!(mouse.kind, MouseEventKind::Down(_)) {
                    self.ui.context_menu = None;
                    self.ui.input_mode = InputMode::Normal;
                }
            }
        }
        PostAction::None
    }

    pub(super) fn handle_picker_mouse(
        &mut self,
        mouse: crossterm::event::MouseEvent,
    ) -> PostAction {
        use crossterm::event::{MouseButton, MouseEventKind};

        let area = self.last_terminal_area;

        let layout = match self.ui.picker_state.as_ref().and_then(|p| p.layout(area)) {
            Some(l) => l,
            None => return PostAction::None,
        };

        let dialog = layout.dialog_rect;
        let inside = mouse.column >= dialog.x
            && mouse.column < dialog.x + dialog.width
            && mouse.row >= dialog.y
            && mouse.row < dialog.y + dialog.height;

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if inside {
                    if let Some(picker) = self.ui.picker_state.as_mut() {
                        if let Some(idx) = picker.row_to_filtered_idx(&layout, mouse.row) {
                            picker.selected = idx;
                            self.confirm_picker();
                        }
                    }
                } else {
                    // Click outside closes picker
                    self.ui.picker_state = None;
                    self.ui.input_mode = InputMode::Normal;
                }
            }
            MouseEventKind::Moved => {
                if inside {
                    if let Some(picker) = self.ui.picker_state.as_mut() {
                        if let Some(idx) = picker.row_to_filtered_idx(&layout, mouse.row) {
                            picker.selected = idx;
                        }
                    }
                }
            }
            MouseEventKind::ScrollUp => {
                if inside {
                    if let Some(picker) = self.ui.picker_state.as_mut() {
                        picker.move_up();
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                if inside {
                    if let Some(picker) = self.ui.picker_state.as_mut() {
                        picker.move_down();
                    }
                }
            }
            _ => {
                if matches!(mouse.kind, MouseEventKind::Down(_)) {
                    self.ui.picker_state = None;
                    self.ui.input_mode = InputMode::Normal;
                }
            }
        }
        PostAction::None
    }

    pub(super) fn handle_dialog_mouse(
        &mut self,
        mouse: crossterm::event::MouseEvent,
        preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) -> PostAction {
        use crossterm::event::{MouseButton, MouseEventKind};

        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return PostAction::None;
        }

        // Check if click is outside the dialog area
        let area = self.last_terminal_area;

        if let Some(ref dialog) = self.ui.input_dialog {
            let dialog_rect = crate::render::input_dialog::input_dialog_rect(area, &dialog.kind);

            let inside = mouse.column >= dialog_rect.x
                && mouse.column < dialog_rect.x + dialog_rect.width
                && mouse.row >= dialog_rect.y
                && mouse.row < dialog_rect.y + dialog_rect.height;

            if inside {
                let (confirm_rect, cancel_rect) = dialog.button_positions(area);
                if mouse.column >= confirm_rect.x
                    && mouse.column < confirm_rect.x + confirm_rect.width
                    && mouse.row == confirm_rect.y
                {
                    self.confirm_dialog(preview_tx);
                    return PostAction::None;
                }
                if mouse.column >= cancel_rect.x
                    && mouse.column < cancel_rect.x + cancel_rect.width
                    && mouse.row == cancel_rect.y
                {
                    self.ui.input_dialog = None;
                    self.ui.input_mode = InputMode::Normal;
                    return PostAction::None;
                }
            } else {
                self.ui.input_dialog = None;
                self.ui.input_mode = InputMode::Normal;
            }
        }
        PostAction::None
    }

    pub(super) fn handle_global_search_mouse(
        &mut self,
        mouse: crossterm::event::MouseEvent,
        _preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) -> PostAction {
        use crossterm::event::{MouseButton, MouseEventKind};

        // Only respond to left-click; ignore hover, scroll, drag, etc.
        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return PostAction::None;
        }

        // Compute overlay rect (shared with GlobalSearchOverlay::render)
        let area = self.last_terminal_area;
        let overlay = crate::render::global_search::global_search_rect(area);

        let inside = mouse.column >= overlay.x
            && mouse.column < overlay.x + overlay.width
            && mouse.row >= overlay.y
            && mouse.row < overlay.y + overlay.height;

        if !inside {
            // Click outside overlay -> cancel
            self.close_global_search_overlay();
            return PostAction::None;
        }

        // Click inside the results area -> select + confirm that result
        let results_y = overlay.y + 3;
        let results_end_y = overlay.y + overlay.height.saturating_sub(2);
        if mouse.row >= results_y && mouse.row < results_end_y {
            let scroll = self.search_state.global_scroll_offset;
            let clicked_index = scroll + (mouse.row - results_y) as usize;

            if self.search_state.global_search_type == GlobalSearchType::Content {
                if clicked_index < self.search_state.visible_item_count() {
                    self.search_state.global_selected = clicked_index;
                    return self.handle_content_search_confirm();
                }
            } else if clicked_index < self.search_state.global_results.len() {
                self.search_state.global_selected = clicked_index;
                if let Some(result) = self
                    .search_state
                    .global_results
                    .get(self.search_state.global_selected)
                    .cloned()
                {
                    self.close_global_search_overlay();
                    return self.search_open_action(result.path, None);
                }
            }
        }

        // Click on input area or border -> no-op
        PostAction::None
    }

    pub(super) fn handle_status_bar_click(
        &mut self,
        col: u16,
        _preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) -> PostAction {
        // Check if click is on the branch name region
        if let Some((start, end)) = self.status_bar_branch_region {
            if col >= start && col < end && self.git.is_some() {
                self.open_branch_picker();
            }
        }
        PostAction::None
    }

    pub(super) fn handle_search_bar_click(
        &mut self,
        col: u16,
        _preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) -> PostAction {
        use crate::render::search_bar::SearchBar;

        // Check if click is on the [x] close button
        if self.search_bar_y.is_some() {
            let area_width = self.main_area_width;
            let close_x = SearchBar::close_button_x(0, area_width);
            if col >= close_x {
                // Close button clicked -> cancel search
                self.ui.input_mode = InputMode::Normal;
                self.search_state.clear();
                return PostAction::None;
            }
        }

        // Click elsewhere on search bar -> focus search (preserve query)
        self.ui.input_mode = InputMode::Search;
        PostAction::None
    }
}
