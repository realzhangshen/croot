use super::*;

impl App {
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
            return;
        };

        let path = node.path.clone();
        let node_git_status = node.git_status;

        // Derive the diff hint up front so it can participate in the cache
        // key. When show_git_diff is off we always skip; otherwise the status
        // decides between Skip / AllAdded / Compute. This lets us bypass
        // Repository::discover + canonicalize for clean, ignored, untracked,
        // and staged-added files.
        let git_diff_hint = if self.config.preview.show_git_diff {
            crate::git::diff::GitDiffHint::from_status(node_git_status)
        } else {
            crate::git::diff::GitDiffHint::Skip
        };

        // Short-circuit only when path+mtime+hint all match. Including the
        // hint in the cache key is load-bearing: without it, a file that was
        // previewed while git status was stale (e.g. Clean before a background
        // refresh landed Modified) would be stuck with its outdated diff
        // gutter for the rest of the session, because mtime alone doesn't
        // change when only the git state changed.
        if self.preview.state.current_path.as_ref() == Some(&path)
            && self.preview.state.kind != PreviewKind::Loading
            && self.preview.state.cached_diff_hint == Some(git_diff_hint)
        {
            let current_mtime = std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok());
            if current_mtime == self.preview.state.cached_mtime {
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

        self.preview.debounce_handle = Some(tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            let path_for_send = path.clone();
            let loaded = tokio::task::spawn_blocking(move || {
                load_preview(
                    &path,
                    max_file_size_kb,
                    syntax_highlight,
                    render_markdown,
                    preview_width,
                    image_preview,
                    git_diff_hint,
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
