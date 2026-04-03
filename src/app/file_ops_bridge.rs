use super::*;

impl App {
    pub(super) fn confirm_dialog(
        &mut self,
        preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) {
        let Some(dialog) = self.ui.input_dialog.take() else {
            return;
        };
        self.ui.input_mode = InputMode::Normal;

        let result = file_ops::execute_dialog(
            &dialog.kind,
            &dialog.input,
            &dialog.target_name,
            &dialog.context_path,
            &self.root,
            dialog.use_trash,
        );
        match result {
            file_ops::FileOpResult::Ok => {
                self.full_refresh_sync(preview_tx);
            }
            file_ops::FileOpResult::Error(msg) => {
                self.show_error(msg);
            }
            file_ops::FileOpResult::Noop => {}
        }
    }

    pub(super) fn execute_menu_action(
        &mut self,
        action: &MenuAction,
        node_idx: usize,
        preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) -> PostAction {
        match action {
            MenuAction::OpenInEditor => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    if !node.is_dir() {
                        return PostAction::OpenEditorSuspend(node.path.clone(), None);
                    }
                }
            }
            MenuAction::OpenInCmuxTab => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    if !node.is_dir() {
                        return PostAction::OpenEditorCmux(node.path.clone(), None);
                    }
                }
            }
            MenuAction::CopyPath => {
                let text = if let Some(node) = self.tree.nodes.get(node_idx) {
                    node.path
                        .strip_prefix(&self.root)
                        .unwrap_or(&node.path)
                        .to_string_lossy()
                        .into_owned()
                } else {
                    String::new()
                };
                if !text.is_empty() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(text);
                    }
                }
            }
            MenuAction::CopyAbsPath => {
                let text = if let Some(node) = self.tree.nodes.get(node_idx) {
                    node.path.to_string_lossy().into_owned()
                } else {
                    String::new()
                };
                if !text.is_empty() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(text);
                    }
                }
            }
            MenuAction::OpenExternally => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    if !node.is_dir() {
                        self.open_externally(&node.path.clone());
                    }
                }
            }
            MenuAction::RevealInFinder => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    #[cfg(target_os = "macos")]
                    {
                        let _ = std::process::Command::new("open")
                            .arg("-R")
                            .arg(&node.path)
                            .spawn();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        // Open the parent directory in the default file manager
                        let dir = if node.is_dir() {
                            &node.path
                        } else {
                            node.path.parent().unwrap_or(&node.path)
                        };
                        let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
                    }
                }
            }
            MenuAction::NewFile => self.start_new_file_at(node_idx),
            MenuAction::NewDir => self.start_new_dir_at(node_idx),
            MenuAction::Rename => self.start_rename_at(node_idx),
            MenuAction::Delete => self.start_delete_at(node_idx),
            MenuAction::TogglePreview => {
                self.preview_visible = !self.preview_visible;
                if self.preview_visible {
                    self.trigger_preview_load(preview_tx);
                } else {
                    self.focus = FocusPane::Tree;
                }
            }
            MenuAction::Refresh => {
                self.full_refresh_sync(preview_tx);
            }
            MenuAction::CollapseAll => {
                self.tree.collapse_all();
                self.reapply_git();
                self.refresh_search_state();
            }
            MenuAction::StartFind => {
                self.search_state = SearchState::new(SearchMode::Find);
                self.search_state.origin_cursor = self.tree.cursor;
                self.search_state.origin_scroll_offset = self.tree.scroll_offset;
                self.ui.input_mode = InputMode::Search;
            }
            MenuAction::Separator => {} // inert -- should not reach here
        }

        // Refresh preview after menu actions that modify files
        if matches!(
            action,
            MenuAction::NewFile | MenuAction::NewDir | MenuAction::Rename | MenuAction::Delete
        ) {
            // Refresh handled in confirm_dialog
        } else if self.preview_visible {
            self.trigger_preview_load(preview_tx);
        }
        PostAction::None
    }

    pub(super) fn start_new_file(&mut self) {
        let dir = self.current_dir();
        self.ui.input_dialog = Some(InputDialogState::new(
            DialogKind::NewFile,
            dir,
            String::new(),
        ));
        self.ui.input_mode = InputMode::Dialog;
    }

    pub(super) fn start_new_dir(&mut self) {
        let dir = self.current_dir();
        self.ui.input_dialog = Some(InputDialogState::new(
            DialogKind::NewDir,
            dir,
            String::new(),
        ));
        self.ui.input_mode = InputMode::Dialog;
    }

    pub(super) fn start_rename(&mut self) {
        if let Some(node) = self.tree.selected() {
            let name = node.name.clone();
            let path = node.path.clone();
            self.ui.input_dialog = Some(InputDialogState::new(DialogKind::Rename, path, name));
            self.ui.input_mode = InputMode::Dialog;
        }
    }

    pub(super) fn start_delete(&mut self) {
        if let Some(node) = self.tree.selected() {
            let name = node.name.clone();
            let path = node.path.clone();
            let mut dialog = InputDialogState::new(DialogKind::ConfirmDelete, path, name);
            dialog.use_trash = self.config.general.use_trash;
            self.ui.input_dialog = Some(dialog);
            self.ui.input_mode = InputMode::Dialog;
        }
    }

    fn start_new_file_at(&mut self, node_idx: usize) {
        let dir = self.dir_for_node(node_idx);
        self.ui.input_dialog = Some(InputDialogState::new(
            DialogKind::NewFile,
            dir,
            String::new(),
        ));
        self.ui.input_mode = InputMode::Dialog;
    }

    fn start_new_dir_at(&mut self, node_idx: usize) {
        let dir = self.dir_for_node(node_idx);
        self.ui.input_dialog = Some(InputDialogState::new(
            DialogKind::NewDir,
            dir,
            String::new(),
        ));
        self.ui.input_mode = InputMode::Dialog;
    }

    fn start_rename_at(&mut self, node_idx: usize) {
        if let Some(node) = self.tree.nodes.get(node_idx) {
            let name = node.name.clone();
            let path = node.path.clone();
            self.ui.input_dialog = Some(InputDialogState::new(DialogKind::Rename, path, name));
            self.ui.input_mode = InputMode::Dialog;
        }
    }

    fn start_delete_at(&mut self, node_idx: usize) {
        if let Some(node) = self.tree.nodes.get(node_idx) {
            let name = node.name.clone();
            let path = node.path.clone();
            let mut dialog = InputDialogState::new(DialogKind::ConfirmDelete, path, name);
            dialog.use_trash = self.config.general.use_trash;
            self.ui.input_dialog = Some(dialog);
            self.ui.input_mode = InputMode::Dialog;
        }
    }

    /// Get the directory for a given node (node itself if dir, or its parent).
    pub(super) fn dir_for_node(&self, node_idx: usize) -> PathBuf {
        if let Some(node) = self.tree.nodes.get(node_idx) {
            file_ops::dir_for_path(&node.path, node.is_dir(), &self.root)
        } else {
            self.root.clone()
        }
    }

    pub(super) fn open_context_menu(&mut self, col: u16, row: u16) {
        // Exclude preview pane and separator
        if self
            .preview_area_x
            .is_some_and(|px| col >= px.saturating_sub(1))
        {
            return;
        }
        if row < self.tree_area_y || row >= self.tree_area_y + self.tree_area_height {
            return;
        }
        let relative_row = (row - self.tree_area_y) as usize;
        let menu = if relative_row >= self.tree.rendered_indices.len() {
            // Empty space below tree items -> workspace root menu
            ContextMenuState::new_for_workspace(col, row, self.tree.len())
        } else {
            let node_idx = self.tree.rendered_indices[relative_row];
            if node_idx >= self.tree.len() {
                return;
            }
            self.tree.cursor = node_idx;
            if self.tree.nodes[node_idx].is_dir() {
                ContextMenuState::new_for_dir(col, row, node_idx)
            } else {
                ContextMenuState::new_for_file(col, row, node_idx, self.cmux.is_some())
            }
        };

        self.ui.context_menu = Some(menu);
        self.ui.input_mode = InputMode::ContextMenu;
    }
}
