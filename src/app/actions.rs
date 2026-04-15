use super::*;

impl App {
    pub(super) fn handle_action(
        &mut self,
        action: &Action,
        preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
        search_tx: &mpsc::Sender<SearchBatch>,
    ) -> PostAction {
        let mut post = PostAction::None;
        match *action {
            Action::Quit => {
                if self.ui.input_mode == InputMode::Normal {
                    self.should_quit = true;
                } else {
                    self.ui.input_mode = InputMode::Normal;
                    self.ui.context_menu = None;
                    self.ui.input_dialog = None;
                    self.ui.picker_state = None;
                }
            }

            // Tree actions
            Action::CursorUp
            | Action::CursorDown
            | Action::CursorLeft
            | Action::CursorRight
            | Action::Toggle
            | Action::ScrollUp(_)
            | Action::ScrollDown(_)
            | Action::GotoTop
            | Action::GotoBottom => {
                let before_path = self.tree.selected().map(|n| n.path.clone());
                self.handle_tree_action(action);
                let after_path = self.tree.selected().map(|n| n.path.clone());
                if self.preview.visible && before_path != after_path {
                    self.trigger_preview_load(preview_tx);
                }
            }
            Action::Refresh => {
                self.full_refresh_sync(preview_tx);
            }

            // Preview actions
            Action::PreviewScrollUp(_) | Action::PreviewScrollDown(_) | Action::SwitchFocus => {
                self.handle_preview_action(action);
            }
            Action::TogglePreview => {
                self.preview.visible = !self.preview.visible;
                if self.preview.visible {
                    self.trigger_preview_load(preview_tx);
                } else {
                    self.focus = FocusPane::Tree;
                }
            }
            Action::ToggleRender => {
                self.preview.state.render_markdown = !self.preview.state.render_markdown;
                self.preview.state.cached_mtime = None;
                if self.preview.visible {
                    self.trigger_preview_load(preview_tx);
                }
            }

            // Separator drag
            Action::SeparatorDragStart => {
                self.dragging_separator = true;
            }
            Action::DragUpdate(col, row) => {
                if self.dragging_separator {
                    if self.main_area_width > 0 {
                        let ratio = 1.0 - (f32::from(col) / f32::from(self.main_area_width));
                        self.config.preview.split_ratio = ratio.clamp(0.2, 0.8);
                    }
                } else if self.preview.area_x.is_some_and(|px| col >= px) {
                    self.handle_selection_action(&Action::SelectionUpdate(col, row));
                }
            }

            // Selection actions
            Action::SelectionStart(_, _)
            | Action::SelectionUpdate(_, _)
            | Action::CopySelection
            | Action::ClearSelection => {
                self.dragging_separator = false;
                self.handle_selection_action(action);
            }

            // Click routing
            Action::ClickRow(row) => {
                self.dragging_separator = false;
                self.handle_click_row(row, preview_tx);
            }

            Action::DragEnd => {
                self.dragging_separator = false;
            }

            Action::Hover(col, row) => {
                self.update_hover(col, row);
            }

            // Right-click context menu
            Action::RightClick(col, row) => {
                self.open_context_menu(col, row);
            }

            // Context menu actions
            Action::MenuClose => {
                self.ui.context_menu = None;
                self.ui.input_mode = InputMode::Normal;
            }
            Action::MenuUp => {
                if let Some(ref mut menu) = self.ui.context_menu {
                    menu.move_up();
                }
            }
            Action::MenuDown => {
                if let Some(ref mut menu) = self.ui.context_menu {
                    menu.move_down();
                }
            }
            Action::MenuSelect => {
                // Resolve actual action from selected menu item
                if let Some(menu) = self.ui.context_menu.take() {
                    if let Some(menu_action) = menu.selected_action().cloned() {
                        self.ui.input_mode = InputMode::Normal;
                        post = self.execute_menu_action(&menu_action, menu.node_idx, preview_tx);
                    }
                }
            }
            // File operations (keyboard shortcuts)
            Action::NewFile => {
                let dir = self.current_dir();
                self.ui.input_dialog = Some(InputDialogState::for_new_file(dir));
                self.ui.input_mode = InputMode::Dialog;
            }
            Action::NewDir => {
                let dir = self.current_dir();
                self.ui.input_dialog = Some(InputDialogState::for_new_dir(dir));
                self.ui.input_mode = InputMode::Dialog;
            }
            Action::RenameNode => {
                if let Some(node) = self.tree.selected() {
                    self.ui.input_dialog = Some(InputDialogState::for_rename(
                        node.path.clone(),
                        node.name.clone(),
                    ));
                    self.ui.input_mode = InputMode::Dialog;
                }
            }
            Action::DeleteNode => {
                if let Some(node) = self.tree.selected() {
                    self.ui.input_dialog = Some(InputDialogState::for_delete(
                        node.path.clone(),
                        node.name.clone(),
                        self.config.general.use_trash,
                    ));
                    self.ui.input_mode = InputMode::Dialog;
                }
            }

            // Dialog actions
            Action::DialogChar(ch) => {
                if let Some(ref mut dialog) = self.ui.input_dialog {
                    dialog.insert_char(ch);
                }
            }
            Action::DialogBackspace => {
                if let Some(ref mut dialog) = self.ui.input_dialog {
                    dialog.delete_char();
                }
            }
            Action::DialogLeft => {
                if let Some(ref mut dialog) = self.ui.input_dialog {
                    dialog.move_left();
                }
            }
            Action::DialogRight => {
                if let Some(ref mut dialog) = self.ui.input_dialog {
                    dialog.move_right();
                }
            }
            Action::DialogConfirm => {
                self.confirm_dialog(preview_tx);
            }
            Action::DialogCancel => {
                self.ui.input_dialog = None;
                self.ui.input_mode = InputMode::Normal;
            }

            // Search actions -- Find mode
            Action::StartFind => {
                self.search_state = SearchState::new(SearchMode::Find);
                self.search_state.origin_cursor = self.tree.cursor;
                self.search_state.origin_scroll_offset = self.tree.scroll_offset;
                self.ui.input_mode = InputMode::Search;
            }
            // Search actions -- Filter mode
            Action::StartFilter => {
                self.search_state = SearchState::new(SearchMode::Filter);
                self.search_state.origin_cursor = self.tree.cursor;
                self.search_state.origin_scroll_offset = self.tree.scroll_offset;
                self.ui.input_mode = InputMode::Search;
            }
            Action::SearchChar(ch) => {
                self.search_state.insert_char(ch);
                match self.search_state.mode {
                    SearchMode::Find => self.update_find_matches(),
                    SearchMode::Filter => self.update_filter_view(),
                    SearchMode::Global => {}
                }
            }
            Action::SearchBackspace => {
                self.search_state.delete_char();
                match self.search_state.mode {
                    SearchMode::Find => self.update_find_matches(),
                    SearchMode::Filter => self.update_filter_view(),
                    SearchMode::Global => {}
                }
            }
            Action::SearchLeft => {
                self.search_state.move_left();
            }
            Action::SearchRight => {
                self.search_state.move_right();
            }
            Action::SearchConfirm => {
                match self.search_state.mode {
                    SearchMode::Find => {
                        // Exit search, clear highlights, cursor stays
                        self.ui.input_mode = InputMode::Normal;
                        self.search_state.clear();
                    }
                    SearchMode::Filter => {
                        // Exit input but keep filter active
                        self.ui.input_mode = InputMode::Normal;
                    }
                    SearchMode::Global => {}
                }
            }
            Action::SearchCancel => {
                // Restore cursor and clear all search state
                self.tree.cursor = self.search_state.origin_cursor;
                self.tree.scroll_offset = self.search_state.origin_scroll_offset;
                self.ui.input_mode = InputMode::Normal;
                self.search_state.clear();
            }
            Action::SearchNext => {
                self.search_navigate_next();
            }
            Action::SearchPrev => {
                self.search_navigate_prev();
            }
            // Global search actions
            Action::StartGlobalSearch => {
                let last_id = self.search_state.request_id;
                self.search_state = SearchState::new(SearchMode::Global);
                self.search_state.request_id = last_id;
                self.search_state.global_search_type = GlobalSearchType::FileName;
                self.ui.input_mode = InputMode::GlobalSearch;
            }
            Action::StartGlobalSearchContent => {
                let last_id = self.search_state.request_id;
                self.search_state = SearchState::new(SearchMode::Global);
                self.search_state.request_id = last_id;
                self.search_state.global_search_type = GlobalSearchType::Content;
                self.ui.input_mode = InputMode::GlobalSearch;
            }
            Action::GlobalSearchChar(ch) => {
                self.search_state.insert_char(ch);
                self.spawn_global_search(search_tx);
            }
            Action::GlobalSearchBackspace => {
                self.search_state.delete_char();
                if self.search_state.query.is_empty() {
                    self.abort_global_search_task(true);
                    self.search_state.clear();
                } else {
                    self.spawn_global_search(search_tx);
                }
            }
            Action::GlobalSearchUp => {
                if self.search_state.global_selected > 0 {
                    self.search_state.global_selected -= 1;
                    if self.search_state.global_selected < self.search_state.global_scroll_offset {
                        self.search_state.global_scroll_offset = self.search_state.global_selected;
                    }
                }
            }
            Action::GlobalSearchDown => {
                let upper = if self.search_state.global_search_type == GlobalSearchType::Content {
                    self.search_state.visible_item_count()
                } else {
                    self.search_state.global_results.len()
                };
                if upper > 0 && self.search_state.global_selected + 1 < upper {
                    self.search_state.global_selected += 1;
                    let visible = self.search_state.global_visible_height;
                    if visible > 0
                        && self.search_state.global_selected
                            >= self.search_state.global_scroll_offset + visible
                    {
                        self.search_state.global_scroll_offset =
                            self.search_state.global_selected - visible + 1;
                    }
                }
            }
            Action::GlobalSearchConfirm => {
                if self.search_state.global_search_type == GlobalSearchType::Content {
                    post = self.handle_content_search_confirm();
                } else if let Some(result) = self
                    .search_state
                    .global_results
                    .get(self.search_state.global_selected)
                    .cloned()
                {
                    self.close_global_search_overlay();
                    post = self.search_open_action(result.path, None);
                }
            }
            Action::GlobalSearchCancel => {
                self.close_global_search_overlay();
            }
            Action::GlobalSearchGoto => {
                if self.search_state.global_search_type == GlobalSearchType::Content {
                    self.handle_content_search_goto(preview_tx);
                } else if let Some(result) = self
                    .search_state
                    .global_results
                    .get(self.search_state.global_selected)
                    .cloned()
                {
                    self.close_global_search_overlay();
                    self.tree.navigate_to_path(&result.path);
                    self.reapply_git();
                    self.trigger_preview_load(preview_tx);
                }
            }

            // Open file in editor
            Action::OpenInEditor => {
                if let Some(node) = self.tree.selected() {
                    if !node.is_dir() {
                        post = PostAction::OpenEditor(node.path.clone(), None);
                    }
                }
            }
            Action::OpenExternally => {
                if let Some(node) = self.tree.selected() {
                    if !node.is_dir() {
                        let path = node.path.clone();
                        self.open_externally(&path);
                    }
                }
            }
            // Collapse all directories
            Action::CollapseAll => {
                self.tree.collapse_all();
                self.reapply_git();
                self.refresh_search_state();
                if self.preview.visible {
                    self.trigger_preview_load(preview_tx);
                }
            }
            Action::DoubleClick(row) => {
                // Cursor already set by the first ClickRow. For files, open externally.
                // For directories, do nothing -- the first click already toggled.
                let row_idx = row as usize;
                if row_idx < self.tree.rendered_indices.len() {
                    let idx = self.tree.rendered_indices[row_idx];
                    if idx < self.tree.len() && !self.tree.nodes[idx].is_dir() {
                        let path = self.tree.nodes[idx].path.clone();
                        self.open_externally(&path);
                    }
                }
            }
            // Branch picker
            Action::OpenBranchPicker => {
                if self.git.is_some() {
                    self.open_branch_picker();
                }
            }
            Action::PickerChar(ch) => {
                if let Some(ref mut picker) = self.ui.picker_state {
                    picker.insert_char(ch);
                }
            }
            Action::PickerBackspace => {
                if let Some(ref mut picker) = self.ui.picker_state {
                    picker.delete_char();
                }
            }
            Action::PickerUp => {
                if let Some(ref mut picker) = self.ui.picker_state {
                    picker.move_up();
                }
            }
            Action::PickerDown => {
                if let Some(ref mut picker) = self.ui.picker_state {
                    picker.move_down();
                }
            }
            Action::PickerConfirm => {
                self.confirm_picker();
            }
            Action::PickerCancel => {
                self.ui.picker_state = None;
                self.ui.input_mode = InputMode::Normal;
            }

            Action::EnterKey => {
                let selected_is_dir = self
                    .tree
                    .selected()
                    .is_some_and(crate::tree::node::TreeNode::is_dir);
                if selected_is_dir {
                    let idx = self.tree.cursor;
                    self.tree.toggle(idx);
                    self.reapply_git();
                    self.refresh_search_state();
                } else if let Some(path) = self.tree.selected().map(|n| n.path.clone()) {
                    post = PostAction::OpenEditor(path, None);
                }
            }

            Action::Paste(ref text) => {
                match self.ui.input_mode {
                    InputMode::Normal | InputMode::ContextMenu => {
                        // Ignore paste in non-input modes -- safety guard
                    }
                    InputMode::Search => {
                        self.search_state.insert_str(text);
                        match self.search_state.mode {
                            SearchMode::Find => self.update_find_matches(),
                            SearchMode::Filter => self.update_filter_view(),
                            SearchMode::Global => {}
                        }
                    }
                    InputMode::Dialog => {
                        if let Some(ref mut dialog) = self.ui.input_dialog {
                            dialog.insert_str(text);
                        }
                    }
                    InputMode::Picker => {
                        if let Some(ref mut picker) = self.ui.picker_state {
                            picker.insert_str(text);
                        }
                    }
                    InputMode::GlobalSearch => {
                        self.search_state.insert_str(text);
                        self.spawn_global_search(search_tx);
                    }
                }
            }

            Action::None => {}
        }
        post
    }
}
