use super::*;

impl App {
    pub(super) fn apply_pending_preview_navigation(&mut self, path: &std::path::Path) {
        if let Some((target_path, line)) = self.preview.pending_line.take() {
            if target_path == path {
                self.preview.state.scroll_to_line(line);
            }
        }

        if let Some(pending) = self.preview.pending_highlight.take() {
            if pending.path == path {
                self.preview
                    .state
                    .set_search_highlight(pending.line, &pending.query);
            }
        }
    }

    pub(super) fn handle_preview_action(&mut self, action: &Action) {
        #[cfg(feature = "image-preview")]
        if self.preview.state.kind == PreviewKind::Image {
            // Only allow focus switching for image previews
            if let Action::SwitchFocus = action {
                self.focus = match self.focus {
                    FocusPane::Tree => FocusPane::Preview,
                    FocusPane::Preview => FocusPane::Tree,
                };
            }
            return;
        }
        match action {
            Action::PreviewScrollUp(n) => self.preview.state.scroll_up(*n as usize),
            Action::PreviewScrollDown(n) => self.preview.state.scroll_down(*n as usize),
            Action::SwitchFocus => {
                self.focus = match self.focus {
                    FocusPane::Tree => FocusPane::Preview,
                    FocusPane::Preview => FocusPane::Tree,
                };
            }
            _ => {}
        }
    }

    pub(super) fn handle_selection_action(&mut self, action: &Action) {
        #[cfg(feature = "image-preview")]
        if self.preview.state.kind == PreviewKind::Image {
            return;
        }
        match action {
            Action::SelectionStart(col, row) => {
                self.focus = FocusPane::Preview;
                if let Some(pos) = self.screen_to_content(*col, *row) {
                    self.preview.state.selection.anchor = Some(pos);
                    self.preview.state.selection.cursor = Some(pos);
                } else {
                    self.preview.state.selection.clear();
                }
            }
            Action::SelectionUpdate(col, row) => {
                if self.preview.state.selection.anchor.is_some() {
                    if let Some(pos) = self.screen_to_content(*col, *row) {
                        self.preview.state.selection.cursor = Some(pos);
                    }
                }
            }
            Action::CopySelection => {
                if let Some(text) = self.preview.state.extract_selected_text() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(text);
                    }
                }
                self.preview.state.selection.clear();
            }
            Action::ClearSelection => {
                self.preview.state.selection.clear();
            }
            _ => {}
        }
    }

    /// Schedule a debounced preview load for the currently selected node.
    pub(super) fn trigger_preview_load(
        &mut self,
        preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) {
        let Some(node) = self.tree.selected() else {
            if let Some(handle) = self.preview.debounce_handle.take() {
                handle.abort();
            }
            self.preview.generation = self.preview.generation.wrapping_add(1);
            self.preview.pending_line = None;
            self.preview.pending_highlight = None;
            self.preview.state.clear();
            return;
        };

        let path = node.path.clone();
        let node_git_status = node.git_status;

        // The diff hint participates in the cache key (see below). Deriving
        // it up front also lets us bypass Repository::discover + canonicalize
        // for clean / ignored / untracked / staged-added files.
        let git_diff_hint = if self.config.preview.show_git_diff {
            crate::git::diff::GitDiffHint::from_status(node_git_status)
        } else {
            crate::git::diff::GitDiffHint::Skip
        };

        // path+mtime alone is insufficient: a file previewed while git status
        // was stale (Clean before a background refresh landed Modified) would
        // keep the stale diff gutter since mtime doesn't change when only git
        // state changes. Including the hint in the cache key fixes that.
        if self.preview.state.current_path.as_ref() == Some(&path)
            && self.preview.state.kind != PreviewKind::Loading
            && self.preview.state.cached_diff_hint == Some(git_diff_hint)
            && (self.preview.state.kind != PreviewKind::Rendered
                || self.preview.state.cached_render_width
                    == Some(self.preview.content_width as usize))
        {
            let current_mtime = std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok());
            if current_mtime == self.preview.state.cached_mtime {
                self.apply_pending_preview_navigation(&path);
                return;
            }
        }

        if let Some(handle) = self.preview.debounce_handle.take() {
            handle.abort();
        }

        self.preview.generation += 1;
        self.preview.state.kind = PreviewKind::Loading;

        let generation = self.preview.generation;
        let tx = preview_tx.clone();
        let delay = Duration::from_millis(self.config.preview.preview_delay_ms);
        let max_file_size_kb = self.config.preview.max_file_size_kb;
        let syntax_highlight = self.config.syntax_enabled();
        let render_markdown = self.preview.state.render_markdown;
        let preview_width = self.preview.content_width as usize;
        let image_preview = self.config.preview.image_preview;
        // Reuse the already-discovered workdir to skip per-preview Repository::discover.
        let repo_root = self.git.as_ref().map(|g| g.repo_root().to_path_buf());

        self.preview.debounce_handle = Some(tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            let path_for_send = path.clone();
            let loaded = tokio::task::spawn_blocking(move || {
                load_preview(
                    &path,
                    &PreviewRequest {
                        max_file_size_kb,
                        syntax_highlight,
                        render_markdown,
                        preview_width,
                        image_preview,
                        repo_root: repo_root.as_deref(),
                        git_diff_hint,
                    },
                )
            })
            .await;

            if let Ok(loaded) = loaded {
                let _ = tx.send((generation, path_for_send, loaded)).await;
            }
        }));
    }

    pub(super) fn screen_to_content(
        &self,
        screen_col: u16,
        screen_row: u16,
    ) -> Option<crate::preview::state::ContentPos> {
        let pl = self.preview.layout?;
        layout::screen_to_content(pl, self.preview.state.scroll_offset, screen_col, screen_row)
    }
}
