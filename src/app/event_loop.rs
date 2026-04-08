use super::*;

impl App {
    pub async fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> anyhow::Result<()>
    where
        B::Error: Send + Sync + 'static,
    {
        // Image result channel type -- always defined so tokio::select! compiles without #[cfg]
        #[cfg(feature = "image-preview")]
        type ImageResult = (PathBuf, String, ratatui_image::thread::ThreadProtocol);
        #[cfg(not(feature = "image-preview"))]
        type ImageResult = ();

        // Set up image resize worker thread (must happen before EventStream)
        #[cfg(feature = "image-preview")]
        {
            let (resize_tx, resize_rx) =
                std::sync::mpsc::channel::<ratatui_image::thread::ResizeRequest>();
            let (response_tx, response_rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                while let Ok(request) = resize_rx.recv() {
                    let _ = response_tx.send(request.resize_encode());
                }
            });

            self.preview.resize_tx = Some(resize_tx);
            self.preview.resize_response_rx = Some(response_rx);
        }

        let mut reader = EventStream::new();

        let (image_tx, mut image_rx) = mpsc::channel::<ImageResult>(4);
        let _ = &image_tx; // suppress unused warning when feature is off

        // Set up file watcher with 100ms debounce
        let (fs_tx, mut fs_rx) = mpsc::channel::<()>(4);
        let watcher_result = crate::watcher::setup_watcher(&self.root, fs_tx);
        if let Some(err) = watcher_result.error {
            self.show_error(err);
        }
        let _watcher = watcher_result.debouncer;
        let mut watcher_active = true;

        // Channel for receiving loaded preview results
        let (preview_tx, mut preview_rx) = mpsc::channel::<(u64, PathBuf, LoadedPreview)>(4);

        // Channel for receiving background refresh results
        let (refresh_tx, mut refresh_rx) = mpsc::channel::<RefreshResult>(2);

        // Channel for receiving global search results (streaming batches)
        let (search_tx, mut search_rx) = mpsc::channel::<SearchBatch>(16);

        // Trigger initial preview load if auto_preview is on
        if self.preview.visible {
            self.trigger_preview_load(&preview_tx);
        }

        let mut post_action = PostAction::None;

        loop {
            // Poll for completed image resize results (non-blocking)
            #[cfg(feature = "image-preview")]
            if let Some(ref rx) = self.preview.resize_response_rx {
                while let Ok(result) = rx.try_recv() {
                    if let Ok(response) = result {
                        if let Some(ref mut thread_proto) = self.preview.state.image_state {
                            thread_proto.update_resized_protocol(response);
                        }
                    }
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
            if self.ui.input_mode == InputMode::Normal {
                self.emit_osc8_hyperlinks()?;
            }

            // When an error message is displayed, set a tick to auto-dismiss it
            // even if no user events occur.
            let has_error = self.ui.error_message.is_some();

            tokio::select! {
                () = tokio::time::sleep(Duration::from_secs(1)), if has_error => {
                    // Tick: re-draw will check if error should be dismissed
                    continue;
                }
                event = reader.next() => {
                    match event {
                        Some(Ok(Event::Key(key))) => {
                            let action = match self.ui.input_mode {
                                InputMode::Normal => {
                                    let has_selection = self.preview.state.selection.is_active();
                                    let action = handle_key(key, self.preview.visible, has_selection, &self.keybinding_map);
                                    if self.focus == FocusPane::Preview {
                                        match action {
                                            Action::ScrollUp(n) => Action::PreviewScrollUp(n),
                                            Action::ScrollDown(n) => Action::PreviewScrollDown(n),
                                            a => a,
                                        }
                                    } else {
                                        action
                                    }
                                }
                                InputMode::ContextMenu => {
                                    handle_key_menu(key, &self.keybinding_map)
                                }
                                InputMode::Dialog => handle_key_dialog(key),
                                InputMode::Search => handle_key_search(key),
                                InputMode::Picker => handle_key_picker(key),
                                InputMode::GlobalSearch => handle_key_global_search(key),
                            };
                            post_action = self.handle_action(&action, &preview_tx, &search_tx);
                        }
                        Some(Ok(Event::Paste(text))) => {
                            let clean: String = text.chars().filter(|c| !c.is_control()).collect();
                            if !clean.is_empty() {
                                post_action = self.handle_action(&Action::Paste(clean), &preview_tx, &search_tx);
                            }
                        }
                        Some(Ok(Event::Mouse(mouse))) if self.mouse_enabled => {
                            use crossterm::event::{MouseButton, MouseEventKind};

                            if self.ui.input_mode == InputMode::ContextMenu {
                                post_action =
                                    self.handle_context_menu_mouse(mouse, &preview_tx);
                            } else if self.ui.input_mode == InputMode::Picker {
                                post_action = self.handle_picker_mouse(mouse);
                            } else if self.ui.input_mode == InputMode::Dialog {
                                // R5: Click outside dialog cancels it
                                post_action = self.handle_dialog_mouse(mouse, &preview_tx);
                            } else if self.ui.input_mode == InputMode::GlobalSearch {
                                post_action = self.handle_global_search_mouse(mouse, &preview_tx);
                            } else {
                                // Route by area priority: status > search > tree/preview
                                let is_left_down = matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left));

                                if mouse.row == self.status_bar_y && is_left_down {
                                    post_action = self.handle_status_bar_click(mouse.column, &preview_tx);
                                } else if self.search_bar_y.is_some_and(|y| mouse.row == y) && is_left_down {
                                    post_action = self.handle_search_bar_click(mouse.column, &preview_tx);
                                } else {
                                    let action = handle_mouse(mouse, self.tree_area_y, self.tree_area_height, self.preview.area_x, &mut self.click_tracker);
                                    post_action = self.handle_action(&action, &preview_tx, &search_tx);
                                }
                            }
                        }
                        Some(Ok(Event::Resize(_, _))) => {
                            self.ui.context_menu = None;
                            self.ui.picker_state = None;
                            self.ui.input_mode = InputMode::Normal;
                            if self.preview.visible {
                                self.trigger_preview_load(&preview_tx);
                            }
                        }
                        Some(Err(_)) | None => break,
                        _ => {}
                    }
                }
                result = fs_rx.recv(), if watcher_active => {
                    if result.is_none() {
                        watcher_active = false;
                        continue;
                    }
                    self.background_refresh(&refresh_tx);
                }
                result = search_rx.recv() => {
                    if let Some(batch) = result {
                        if batch.generation == self.search_state.request_id {
                            if !batch.results.is_empty() {
                                if self.search_state.global_search_type == GlobalSearchType::Content {
                                    // Merge new results into existing grouped results
                                    let new_groups = group_search_results(batch.results);
                                    for ng in new_groups {
                                        if let Some(existing) = self.search_state.grouped_results.iter_mut().find(|g| g.path == ng.path) {
                                            existing.matches.extend(ng.matches);
                                        } else {
                                            self.search_state.grouped_results.push(ng);
                                        }
                                    }
                                    self.search_state.global_results.clear();
                                } else {
                                    self.search_state.global_results.extend(batch.results);
                                }
                            }
                            if batch.is_final {
                                self.search_state.global_error = batch.error;
                                self.search_state.global_loading = false;
                            }
                        }
                    }
                }
                result = preview_rx.recv() => {
                    if let Some((gen, path, loaded)) = result {
                        // Discard stale preview results from older generations
                        if gen != self.preview.generation {
                            continue;
                        }
                        #[allow(unused_mut)]
                        let mut handled = false;
                        #[cfg(feature = "image-preview")]
                        if loaded.kind == PreviewKind::Image {
                            handled = true;
                            // Decode image and create ThreadProtocol in background
                            if let (Some(picker), Some(resize_tx)) =
                                (self.preview.image_picker.clone(), self.preview.resize_tx.clone())
                            {
                                let file_info = loaded.file_info.clone();
                                let tx = image_tx.clone();
                                let path_clone = path.clone();
                                let preview_tx_clone = preview_tx.clone();
                                let gen_for_error = gen;
                                tokio::task::spawn_blocking(move || {
                                    match crate::preview::image::load_image(&path_clone, &picker) {
                                        Ok(proto) => {
                                            let thread_proto =
                                                ratatui_image::thread::ThreadProtocol::new(
                                                    resize_tx,
                                                    Some(proto),
                                                );
                                            let _ = tx.blocking_send((path_clone, file_info, thread_proto));
                                        }
                                        Err(e) => {
                                            let _ = preview_tx_clone.blocking_send((
                                                gen_for_error,
                                                path_clone,
                                                LoadedPreview {
                                                    kind: PreviewKind::Error(e),
                                                    content: Vec::new(),
                                                    file_info,
                                                    line_diffs: None,
                                                    // Image decode errors have no diff gutter.
                                                    git_diff_hint:
                                                        crate::git::diff::GitDiffHint::Skip,
                                                },
                                            ));
                                        }
                                    }
                                });
                            }
                        }
                        if !handled {
                            // Staleness check: only apply if still viewing this path
                            let still_selected = self.tree.selected()
                                .is_some_and(|n| n.path == path);
                            if still_selected {
                                self.preview.state.apply(path, loaded.kind, loaded.content, loaded.file_info, loaded.line_diffs, loaded.git_diff_hint);
                                // Apply pending line scroll from content search confirm
                                if let Some((ref target_path, line)) = self.preview.pending_line {
                                    if self.preview.state.current_path.as_ref() == Some(target_path) {
                                        self.preview.state.scroll_to_line(line);
                                    }
                                }
                                self.preview.pending_line = None;
                            }
                        }
                    }
                }
                result = image_rx.recv() => {
                    #[cfg(feature = "image-preview")]
                    if let Some((path, file_info, thread_proto)) = result {
                        // Staleness check: only apply if still viewing this path
                        let still_selected = self.tree.selected()
                            .is_some_and(|n| n.path == path);
                        if still_selected {
                            self.preview.state.apply_image(path, file_info, thread_proto);
                        }
                    }
                    #[cfg(not(feature = "image-preview"))]
                    let _ = result;
                }
                result = async {
                    match self.branch_switch_rx.as_mut() {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    self.branch_switch_rx = None;
                    if let Some(result) = result {
                        if result.success {
                            self.background_refresh(&refresh_tx);
                        } else {
                            let branches = crate::git::branches::list_branches(&result.repo_root);
                            let mut restored_picker = PickerState::new_branch(&branches);
                            restored_picker.error_message = Some(result.stderr);
                            self.ui.picker_state = Some(restored_picker);
                            self.ui.input_mode = InputMode::Picker;
                        }
                    }
                }
                result = refresh_rx.recv() => {
                    if let Some(refresh) = result {
                        // Clear in-flight regardless of staleness and learn
                        // whether a coalesced follow-up was queued.
                        let should_follow_up = self.refresh.finish_background();

                        if self.refresh.is_current(refresh.generation) {
                            // Capture cursor path at APPLY time (not request time)
                            let cursor_path = self.tree.selected().map(|n| n.path.clone());

                            self.tree = refresh.tree;
                            self.git = refresh.git;

                            // Restore cursor by path
                            if let Some(ref path) = cursor_path {
                                if let Some(idx) = self.tree.nodes.iter().position(|n| n.path == *path) {
                                    self.tree.cursor = idx;
                                }
                            }
                            self.tree.cursor = self.tree.cursor.min(self.tree.nodes.len().saturating_sub(1));

                            self.reapply_git();
                            self.refresh_search_state();
                            if self.preview.visible {
                                self.trigger_preview_load(&preview_tx);
                            }
                        }

                        // Spawn the coalesced follow-up so any events that
                        // arrived while the previous refresh was running are
                        // captured in a single catch-up snapshot.
                        if should_follow_up {
                            self.background_refresh(&refresh_tx);
                        }
                    }
                }
            }

            // Process post-actions that require terminal access
            match std::mem::replace(&mut post_action, PostAction::None) {
                PostAction::OpenEditor(path, line) => {
                    // Auto-detect: try cmux first, fall back to suspend
                    let opened_in_cmux = if let Some(ref cmux) = self.cmux {
                        let editor = self.resolve_editor();
                        match cmux.open_in_editor(&editor, &path, line) {
                            Ok(()) => true,
                            Err(e) => {
                                self.show_error(format!(
                                    "cmux failed, falling back to suspend: {e}"
                                ));
                                false
                            }
                        }
                    } else {
                        false
                    };
                    if !opened_in_cmux {
                        self.open_editor_suspend(terminal, &path, line)?;
                        self.refresh_after_editor(&preview_tx);
                        reader = EventStream::new();
                    }
                }
                PostAction::OpenEditorSuspend(path, line) => {
                    self.open_editor_suspend(terminal, &path, line)?;
                    self.refresh_after_editor(&preview_tx);
                    reader = EventStream::new();
                }
                PostAction::OpenEditorCmux(path, line) => {
                    if let Some(ref cmux) = self.cmux {
                        let editor = self.resolve_editor();
                        if let Err(e) = cmux.open_in_editor(&editor, &path, line) {
                            self.show_error(format!("cmux failed: {e}"));
                        }
                    }
                }
                PostAction::OpenExternalEditor(path, line) => {
                    self.open_in_external_editor(&path, line);
                }
                PostAction::None => {}
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    pub(super) fn emit_osc8_hyperlinks(&self) -> anyhow::Result<()> {
        use std::io::Write;
        if self.hyperlink_regions.is_empty() {
            return Ok(());
        }
        let mut stdout = std::io::stdout();
        // Save cursor position to avoid disturbing ratatui's cursor state
        crossterm::queue!(stdout, crossterm::cursor::SavePosition)?;
        for region in &self.hyperlink_regions {
            crossterm::queue!(stdout, crossterm::cursor::MoveTo(region.x, region.y))?;
            crossterm::queue!(
                stdout,
                crossterm::style::SetAttribute(crossterm::style::Attribute::Reverse)
            )?;
            write!(
                stdout,
                "\x1b]8;;{}\x07{}\x1b]8;;\x07",
                region.url, region.text
            )?;
            // Undo only the Reverse attribute, not all attributes
            crossterm::queue!(
                stdout,
                crossterm::style::SetAttribute(crossterm::style::Attribute::NoReverse)
            )?;
        }
        // Restore cursor position
        crossterm::queue!(stdout, crossterm::cursor::RestorePosition)?;
        stdout.flush()?;
        Ok(())
    }
}
