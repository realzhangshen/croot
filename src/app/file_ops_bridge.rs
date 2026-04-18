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
                        let dir = if node.is_dir() {
                            &node.path
                        } else {
                            node.path.parent().unwrap_or(&node.path)
                        };
                        let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
                    }
                }
            }
            MenuAction::NewFile => {
                let dir = self.dir_for_node(node_idx);
                self.ui.input_dialog = Some(InputDialogState::for_new_file(dir));
                self.ui.input_mode = InputMode::Dialog;
            }
            MenuAction::NewDir => {
                let dir = self.dir_for_node(node_idx);
                self.ui.input_dialog = Some(InputDialogState::for_new_dir(dir));
                self.ui.input_mode = InputMode::Dialog;
            }
            MenuAction::Rename => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    self.ui.input_dialog = Some(InputDialogState::for_rename(
                        node.path.clone(),
                        node.name.clone(),
                    ));
                    self.ui.input_mode = InputMode::Dialog;
                }
            }
            MenuAction::Delete => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    self.ui.input_dialog = Some(InputDialogState::for_delete(
                        node.path.clone(),
                        node.name.clone(),
                        self.config.general.use_trash,
                    ));
                    self.ui.input_mode = InputMode::Dialog;
                }
            }
            MenuAction::TogglePreview => {
                self.preview.visible = !self.preview.visible;
                if self.preview.visible {
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
                let last_id = self.search_state.request_id;
                self.search_state = SearchState::new(SearchMode::Global);
                self.search_state.request_id = last_id;
                self.search_state.global_search_type = GlobalSearchType::Unified;
                self.ui.input_mode = InputMode::GlobalSearch;
            }
            MenuAction::Separator => {} // inert -- separators are not selectable
        }

        // File-modifying actions refresh via confirm_dialog; skip to avoid a double trigger.
        if !matches!(
            action,
            MenuAction::NewFile | MenuAction::NewDir | MenuAction::Rename | MenuAction::Delete
        ) && self.preview.visible
        {
            self.trigger_preview_load(preview_tx);
        }
        PostAction::None
    }

    pub(super) fn dir_for_node(&self, node_idx: usize) -> PathBuf {
        if let Some(node) = self.tree.nodes.get(node_idx) {
            file_ops::dir_for_path(&node.path, node.is_dir(), &self.root)
        } else {
            self.root.clone()
        }
    }

    pub(super) fn open_context_menu(&mut self, col: u16, row: u16) {
        // Menu only opens over the tree area (exclude preview + separator).
        if self
            .preview
            .area_x
            .is_some_and(|px| col >= px.saturating_sub(1))
        {
            return;
        }
        if row < self.tree_area_y || row >= self.tree_area_y + self.tree_area_height {
            return;
        }
        let relative_row = (row - self.tree_area_y) as usize;
        let menu = if relative_row >= self.tree.rendered_indices.len() {
            // Clicks below the last item target the workspace root menu.
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
