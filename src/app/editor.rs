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

    /// Open a file in a configured external editor (background, no TUI suspend).
    /// Uses `editor.external` config with `file:line` syntax; falls back to `open_externally()`.
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
        // Leave alternate screen
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

        // Resolve editor and split into command + args (e.g. "code --wait")
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

        // Capture error to show after terminal restore (when the TUI is visible again)
        let editor_error = match status {
            Err(e) => Some(format!("Failed to open editor '{editor_str}': {e}")),
            Ok(s) if !s.success() => Some(format!(
                "Editor '{editor_str}' exited with {}",
                s.code().map_or("signal".to_string(), |c| c.to_string())
            )),
            _ => None,
        };

        // Restore terminal
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

        // Show editor error after terminal is restored so the message is visible
        if let Some(msg) = editor_error {
            self.show_error(msg);
        }

        Ok(())
    }

    /// Full refresh: tree -> git -> search -> preview.
    /// Consolidates the refresh sequence that was previously duplicated across 5+ call sites.
    pub(super) fn full_refresh(
        &mut self,
        preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) {
        self.tree.refresh();
        if let Some(ref mut git) = self.git {
            git.refresh();
        }
        self.reapply_git();
        self.refresh_search_state();
        if self.preview_visible {
            self.trigger_preview_load(preview_tx);
        }
    }

    /// Refresh tree, git, and preview after returning from a suspend-mode editor.
    pub(super) fn refresh_after_editor(
        &mut self,
        preview_tx: &mpsc::Sender<(u64, PathBuf, LoadedPreview)>,
    ) {
        self.full_refresh(preview_tx);
    }
}
