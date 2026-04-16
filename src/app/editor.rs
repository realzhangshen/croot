use super::*;

impl App {
    /// Resolve the editor command: config -> $VISUAL -> $EDITOR -> "vi".
    pub(super) fn resolve_editor(&self) -> String {
        crate::config::resolve_editor(&self.config)
    }

    fn resolve_open_command(&self, path: &std::path::Path) -> String {
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        for rule in &self.config.open.rules {
            if let Ok(glob) = globset::Glob::new(&rule.pattern) {
                let matcher = glob.compile_matcher();
                if matcher.is_match(file_name.as_ref()) {
                    return rule.command.clone();
                }
            }
        }
        self.config.open.default.clone()
    }

    pub(super) fn open_externally(&mut self, path: &std::path::Path) {
        let command_str = self.resolve_open_command(path);
        let parts = match shell_words::split(&command_str) {
            Ok(p) if !p.is_empty() => p,
            _ => return,
        };
        let (cmd, args) = match parts.split_first() {
            Some(pair) => pair,
            None => return,
        };
        if let Err(e) = std::process::Command::new(cmd)
            .args(args)
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            self.show_error(format!("Failed to open '{cmd}': {e}"));
        }
    }

    /// Open `path` in `editor.external` (no TUI suspend), falling back to
    /// `open_externally` when the config does not set one.
    pub(super) fn open_in_external_editor(&mut self, path: &std::path::Path, line: Option<usize>) {
        let Some(ext_cmd) = crate::config::resolve_external_editor(&self.config) else {
            self.open_externally(path);
            return;
        };
        let argv = build_external_editor_argv(&ext_cmd, path, line);
        let Some((cmd, args)) = argv.split_first() else {
            return;
        };
        if let Err(e) = std::process::Command::new(cmd)
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            self.show_error(format!("Failed to open external editor '{cmd}': {e}"));
        }
    }

    /// Suspend the terminal, spawn the editor, then resume.
    pub(super) fn open_editor_suspend<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        path: &std::path::Path,
        line: Option<usize>,
    ) -> anyhow::Result<()>
    where
        B::Error: Send + Sync + 'static,
    {
        let mut stdout = std::io::stdout();
        if self.enhanced_keyboard {
            let _ = crossterm::execute!(stdout, PopKeyboardEnhancementFlags);
        }
        if self.mouse_enabled {
            let _ = crossterm::execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        } else {
            let _ = crossterm::execute!(stdout, LeaveAlternateScreen);
        }
        let _ = crossterm::terminal::disable_raw_mode();

        let editor_str = self.resolve_editor();
        let mut parts =
            shell_words::split(&editor_str).unwrap_or_else(|_| vec![editor_str.clone()]);
        if let Some(n) = line {
            parts.push(format!("+{n}"));
        }
        let cmd = parts.first().map_or("vi", |s| s.as_str());
        let status = std::process::Command::new(cmd)
            .args(&parts[1..])
            .arg(path)
            .status();

        // Defer reporting until after the TUI is restored and the message is visible.
        let editor_error = match status {
            Err(e) => Some(format!("Failed to open editor '{editor_str}': {e}")),
            Ok(s) if !s.success() => Some(format!(
                "Editor '{editor_str}' exited with {}",
                s.code().map_or("signal".to_string(), |c| c.to_string())
            )),
            _ => None,
        };

        let _ = crossterm::terminal::enable_raw_mode();
        let mut stdout = std::io::stdout();
        if self.mouse_enabled {
            let _ = crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture);
        } else {
            let _ = crossterm::execute!(stdout, EnterAlternateScreen);
        }
        if self.enhanced_keyboard {
            let _ = crossterm::execute!(
                stdout,
                PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
            );
        }
        terminal.clear()?;

        if let Some(msg) = editor_error {
            self.show_error(msg);
        }

        Ok(())
    }

    /// Fully refresh tree/git/search/preview in the caller's thread.
    ///
    /// Delegates to [`RefreshCoordinator::start_sync`] so any in-flight
    /// background refresh becomes stale and any queued follow-up is dropped.
    /// Used when the result must be available immediately (e.g. after editor
    /// suspend, manual refresh, file operations).
    pub(super) fn full_refresh_sync(
        &mut self,
        preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) {
        self.refresh.start_sync();

        self.tree.refresh();
        if let Some(ref mut git) = self.git {
            git.refresh();
        }
        self.reapply_git();
        self.refresh_search_state();
        if self.preview.visible {
            self.trigger_preview_load(preview_tx);
        }
    }

    /// Spawn a blocking task to rebuild tree + git state; the event loop
    /// applies the result from `refresh_tx` next tick.
    ///
    /// Coalescing is handled by [`RefreshCoordinator::try_start_background`]:
    /// if another refresh is already running, the call only records the need
    /// for a follow-up and returns.
    pub(super) fn background_refresh(&mut self, refresh_tx: &mpsc::Sender<RefreshResult>) {
        let Some(generation) = self.refresh.try_start_background() else {
            return;
        };

        let root = self.root.clone();
        let config = self.tree.config.clone();
        let expanded_paths: std::collections::HashSet<PathBuf> = self
            .tree
            .nodes
            .iter()
            .filter(|n| n.is_dir() && n.is_expanded)
            .map(|n| n.path.clone())
            .collect();

        let tx = refresh_tx.clone();

        tokio::task::spawn_blocking(move || {
            let tree = crate::tree::forest::FileTree::snapshot_refresh(
                root.clone(),
                config,
                &expanded_paths,
            );
            let git = crate::git::status::GitState::snapshot_refresh(&root);
            let _ = tx.blocking_send(RefreshResult {
                generation,
                tree,
                git,
            });
        });
    }

    /// Refresh tree, git, and preview after returning from a suspend-mode editor.
    pub(super) fn refresh_after_editor(
        &mut self,
        preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) {
        self.full_refresh_sync(preview_tx);
    }
}
