use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{StatefulWidget, Widget},
    Terminal,
};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::cmux::bridge::CmuxBridge;
use crate::config::Config;
use crate::git::status::GitState;
use crate::input::handler::{handle_key, Action};
use crate::input::mouse::handle_mouse;
use crate::layout::{self, FocusPane, PreviewLayout};
use crate::preview::dispatcher::preview_command;
use crate::preview::loader::{load_preview, LoadedPreview};
use crate::preview::state::{PreviewKind, PreviewState};
use crate::render::colors;
use crate::render::preview_view::PreviewView;
use crate::render::status_bar::StatusBar;
use crate::render::tree_view::TreeView;
use crate::tree::forest::FileTree;

pub struct App {
    pub tree: FileTree,
    pub git: Option<GitState>,
    pub cmux: Option<CmuxBridge>,
    pub config: Config,
    pub root: PathBuf,
    pub should_quit: bool,
    tree_area_y: u16,
    tree_area_height: u16,
    // Preview panel state
    pub preview_state: PreviewState,
    pub preview_visible: bool,
    pub focus: FocusPane,
    preview_debounce_handle: Option<JoinHandle<()>>,
    preview_area_x: Option<u16>,
    preview_layout: Option<PreviewLayout>,
    preview_content_width: u16,
    dragging_separator: bool,
    main_area_width: u16,
}

impl App {
    pub fn new(root: PathBuf) -> anyhow::Result<Self> {
        let config = Config::load();
        let mut tree = FileTree::new(root.clone(), config.tree.clone());
        let git = GitState::load(&root);
        let cmux = CmuxBridge::detect(config.cmux.split_direction.clone());

        if let Some(ref git) = git {
            git.apply_to_nodes(&mut tree.nodes);
        }

        let preview_visible = config.preview.auto_preview;
        let render_markdown = config.preview.render_markdown;

        Ok(Self {
            tree,
            git,
            cmux,
            config,
            root,
            should_quit: false,
            tree_area_y: 0,
            tree_area_height: 0,
            preview_state: {
                let mut ps = PreviewState::new();
                ps.render_markdown = render_markdown;
                ps
            },
            preview_visible,
            focus: FocusPane::Tree,
            preview_debounce_handle: None,
            preview_area_x: None,
            preview_layout: None,
            preview_content_width: 80,
            dragging_separator: false,
            main_area_width: 0,
        })
    }

    pub async fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> anyhow::Result<()> {
        let mut reader = EventStream::new();

        // Set up file watcher with 100ms debounce
        let (fs_tx, mut fs_rx) = mpsc::channel::<()>(1);
        let _watcher = crate::watcher::setup_watcher(&self.root, fs_tx);
        let mut watcher_active = true;

        // Channel for receiving loaded preview results
        let (preview_tx, mut preview_rx) = mpsc::channel::<(PathBuf, LoadedPreview)>(4);

        // Trigger initial preview load if auto_preview is on
        if self.preview_visible {
            self.trigger_preview_load(&preview_tx);
        }

        loop {
            terminal.draw(|frame| self.draw(frame))?;

            tokio::select! {
                event = reader.next() => {
                    match event {
                        Some(Ok(Event::Key(key))) => {
                            let has_selection = self.preview_state.selection.is_active();
                            let action = handle_key(key, self.preview_visible, has_selection);
                            // Keyboard scroll should target whichever pane has focus.
                            // The mouse handler already routes by position, so we only
                            // transform here (at the keyboard entry point).
                            let action = if self.focus == FocusPane::Preview {
                                match action {
                                    Action::ScrollUp(n) => Action::PreviewScrollUp(n),
                                    Action::ScrollDown(n) => Action::PreviewScrollDown(n),
                                    a => a,
                                }
                            } else {
                                action
                            };
                            self.handle_action(action, &preview_tx).await;
                        }
                        Some(Ok(Event::Mouse(mouse))) => {
                            let action = handle_mouse(mouse, self.tree_area_y, self.tree_area_height, self.preview_area_x);
                            self.handle_action(action, &preview_tx).await;
                        }
                        Some(Ok(Event::Resize(_, _))) => {
                            if self.preview_visible {
                                self.trigger_preview_load(&preview_tx);
                            }
                        }
                        Some(Err(_)) | None => break,
                        _ => {}
                    }
                }
                result = fs_rx.recv(), if watcher_active => {
                    if result.is_none() {
                        // Sender dropped (watcher failed to init). Disable this arm.
                        watcher_active = false;
                        continue;
                    }
                    // File system change detected — refresh tree structure and git status
                    self.tree.refresh();
                    if let Some(ref mut git) = self.git {
                        git.refresh();
                    }
                    self.reapply_git();
                    // Re-trigger preview in case the current file changed
                    if self.preview_visible {
                        self.trigger_preview_load(&preview_tx);
                    }
                }
                result = preview_rx.recv() => {
                    if let Some((path, loaded)) = result {
                        self.preview_state.apply(path, loaded.kind, loaded.content, loaded.file_info);
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        // Cleanup
        if self.config.preview.close_on_exit {
            if let Some(ref mut cmux) = self.cmux {
                cmux.close_preview().await;
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        let size = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(size);

        let main_area = chunks[0];
        let status_area = chunks[1];

        self.tree_area_y = main_area.y;
        self.main_area_width = main_area.width;

        if self.preview_visible && main_area.width > 20 {
            // Split horizontally: tree | separator | preview
            let ratio = self.config.preview.split_ratio.clamp(0.2, 0.8);
            let tree_width = (f32::from(main_area.width) * (1.0 - ratio)) as u16;
            let separator_width: u16 = 1;
            let preview_width = main_area.width.saturating_sub(tree_width + separator_width);

            let tree_area = ratatui::layout::Rect {
                x: main_area.x,
                y: main_area.y,
                width: tree_width,
                height: main_area.height,
            };
            let separator_area = ratatui::layout::Rect {
                x: main_area.x + tree_width,
                y: main_area.y,
                width: separator_width,
                height: main_area.height,
            };
            let preview_area = ratatui::layout::Rect {
                x: main_area.x + tree_width + separator_width,
                y: main_area.y,
                width: preview_width,
                height: main_area.height,
            };

            self.tree_area_height = tree_area.height;

            // Render tree
            TreeView {
                config: &self.config.tree,
            }
            .render(tree_area, frame.buffer_mut(), &mut self.tree);

            // Render separator
            let sep_style = Style::default().fg(colors::TREE_LINE);
            for y in separator_area.y..separator_area.y + separator_area.height {
                frame
                    .buffer_mut()
                    .set_string(separator_area.x, y, "│", sep_style);
            }

            self.preview_content_width = preview_width;
            self.preview_area_x = Some(preview_area.x);

            // Compute and cache preview layout for coordinate mapping
            let content_area_y = preview_area.y + 1; // skip header
            let content_area_height = preview_area.height.saturating_sub(1);
            let gutter_width = if self.config.preview.show_line_numbers
                && self.preview_state.kind == PreviewKind::Text
            {
                let digits = if self.preview_state.total_lines == 0 {
                    1
                } else {
                    (self.preview_state.total_lines as f64).log10().floor() as u16 + 1
                };
                digits + 1
            } else {
                0
            };
            self.preview_layout = Some(PreviewLayout {
                x: preview_area.x + gutter_width,
                y: content_area_y,
                height: content_area_height,
            });

            // Render preview
            PreviewView {
                config: &self.config.preview,
                focused: self.focus == FocusPane::Preview,
            }
            .render(preview_area, frame.buffer_mut(), &mut self.preview_state);
        } else {
            // No preview — full width tree
            self.preview_area_x = None;
            self.preview_layout = None;
            self.tree_area_height = main_area.height;

            TreeView {
                config: &self.config.tree,
            }
            .render(main_area, frame.buffer_mut(), &mut self.tree);
        }

        let root_name = self.root.file_name().map_or_else(
            || self.root.to_string_lossy().into_owned(),
            |n| n.to_string_lossy().into_owned(),
        );

        let file_count = self.tree.file_count;
        let dir_count = self.tree.dir_count;
        let branch = self
            .git
            .as_ref()
            .and_then(|g| g.branch())
            .map(std::string::ToString::to_string);
        let cmux_indicator = if self.cmux.is_some() {
            Some("cmux")
        } else {
            None
        };

        let status_bar = StatusBar {
            branch: branch.as_deref(),
            file_count,
            dir_count,
            root_name: &root_name,
            cmux_status: cmux_indicator,
        };
        status_bar.render(status_area, frame.buffer_mut());
    }

    async fn handle_action(
        &mut self,
        action: Action,
        preview_tx: &mpsc::Sender<(PathBuf, LoadedPreview)>,
    ) {
        match action {
            Action::Quit => self.should_quit = true,

            // Tree actions
            Action::CursorUp
            | Action::CursorDown
            | Action::CursorLeft
            | Action::CursorRight
            | Action::Toggle
            | Action::Refresh
            | Action::ScrollUp(_)
            | Action::ScrollDown(_)
            | Action::GotoTop
            | Action::GotoBottom => {
                self.handle_tree_action(&action);
            }

            // Actions requiring async
            Action::Open => self.open_selected().await,

            // Preview actions
            Action::PreviewScrollUp(_) | Action::PreviewScrollDown(_) | Action::SwitchFocus => {
                self.handle_preview_action(&action);
            }
            Action::TogglePreview => {
                self.preview_visible = !self.preview_visible;
                if self.preview_visible {
                    self.trigger_preview_load(preview_tx);
                } else {
                    self.focus = FocusPane::Tree;
                }
            }
            Action::ToggleRender => {
                self.preview_state.render_markdown = !self.preview_state.render_markdown;
                // Force reload by clearing cached mtime
                self.preview_state.cached_mtime = None;
                if self.preview_visible {
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
                        let ratio = 1.0 - (col as f32 / self.main_area_width as f32);
                        self.config.preview.split_ratio = ratio.clamp(0.2, 0.8);
                    }
                } else if self.preview_area_x.is_some_and(|px| col >= px) {
                    self.handle_selection_action(&Action::SelectionUpdate(col, row));
                }
            }

            // Selection actions
            Action::SelectionStart(_, _)
            | Action::SelectionUpdate(_, _)
            | Action::CopySelection
            | Action::ClearSelection => {
                self.dragging_separator = false;
                self.handle_selection_action(&action);
            }

            // Click routing
            Action::ClickRow(row) => {
                self.dragging_separator = false;
                self.handle_click_row(row, preview_tx);
            }

            Action::None => {}
        }
    }

    fn handle_tree_action(&mut self, action: &Action) {
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
                }
            }
            Action::Toggle => {
                let idx = self.tree.cursor;
                self.tree.toggle(idx);
                self.reapply_git();
            }
            Action::Refresh => {
                self.tree.refresh();
                if let Some(ref mut git) = self.git {
                    git.refresh();
                }
                self.reapply_git();
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
                    self.tree.cursor = self.tree.len() - 1;
                }
            }
            _ => {}
        }
    }

    fn handle_preview_action(&mut self, action: &Action) {
        match action {
            Action::PreviewScrollUp(n) => self.preview_state.scroll_up(*n as usize),
            Action::PreviewScrollDown(n) => self.preview_state.scroll_down(*n as usize),
            Action::SwitchFocus => {
                self.focus = match self.focus {
                    FocusPane::Tree => FocusPane::Preview,
                    FocusPane::Preview => FocusPane::Tree,
                };
            }
            _ => {}
        }
    }

    fn handle_selection_action(&mut self, action: &Action) {
        match action {
            Action::SelectionStart(col, row) => {
                self.focus = FocusPane::Preview;
                if let Some(pos) = self.screen_to_content(*col, *row) {
                    self.preview_state.selection.anchor = Some(pos);
                    self.preview_state.selection.cursor = Some(pos);
                } else {
                    self.preview_state.selection.clear();
                }
            }
            Action::SelectionUpdate(col, row) => {
                if self.preview_state.selection.anchor.is_some() {
                    if let Some(pos) = self.screen_to_content(*col, *row) {
                        self.preview_state.selection.cursor = Some(pos);
                    }
                }
            }
            Action::CopySelection => {
                if let Some(text) = self.preview_state.extract_selected_text() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(text);
                    }
                }
                self.preview_state.selection.clear();
            }
            Action::ClearSelection => {
                self.preview_state.selection.clear();
            }
            _ => {}
        }
    }

    fn handle_click_row(&mut self, row: u16, preview_tx: &mpsc::Sender<(PathBuf, LoadedPreview)>) {
        self.focus = FocusPane::Tree;
        self.preview_state.selection.clear();
        let row_idx = row as usize;
        let idx = if row_idx < self.tree.rendered_indices.len() {
            self.tree.rendered_indices[row_idx]
        } else {
            return;
        };
        if idx < self.tree.len() {
            self.tree.cursor = idx;
            if self.tree.nodes[idx].is_dir() {
                self.tree.toggle(idx);
                self.reapply_git();
            } else {
                self.preview_visible = true;
                self.trigger_preview_load(preview_tx);
            }
        }
    }

    /// Schedule a debounced preview load for the currently selected file.
    fn trigger_preview_load(&mut self, preview_tx: &mpsc::Sender<(PathBuf, LoadedPreview)>) {
        let Some(node) = self.tree.selected() else {
            return;
        };

        // Don't preview directories — show empty/hint state
        if node.is_dir() {
            self.preview_state.clear();
            return;
        }

        let path = node.path.clone();

        // Skip if already showing this file and it hasn't changed (mtime check)
        if self.preview_state.current_path.as_ref() == Some(&path)
            && self.preview_state.kind != PreviewKind::Loading
        {
            let current_mtime = std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok());
            if current_mtime == self.preview_state.cached_mtime {
                return;
            }
        }

        // Cancel previous debounce
        if let Some(handle) = self.preview_debounce_handle.take() {
            handle.abort();
        }

        self.preview_state.kind = PreviewKind::Loading;

        let tx = preview_tx.clone();
        let delay = Duration::from_millis(self.config.preview.preview_delay_ms);
        let max_file_size_kb = self.config.preview.max_file_size_kb;
        let syntax_highlight = self.config.preview.syntax_highlight;
        let is_light = colors::is_light();
        let render_markdown = self.preview_state.render_markdown;
        let preview_width = self.preview_content_width as usize;

        self.preview_debounce_handle = Some(tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            let path_for_send = path.clone();
            let loaded = tokio::task::spawn_blocking(move || {
                load_preview(&path, max_file_size_kb, syntax_highlight, is_light, render_markdown, preview_width)
            })
            .await;

            if let Ok(loaded) = loaded {
                let _ = tx.send((path_for_send, loaded)).await;
            }
        }));
    }

    async fn open_selected(&mut self) {
        let Some(node) = self.tree.selected() else {
            return;
        };

        if node.is_dir() {
            let idx = self.tree.cursor;
            self.tree.toggle(idx);
            self.reapply_git();
            return;
        }

        let path = node.path.clone();
        let cmd = preview_command(&path);

        if let Some(ref mut cmux) = self.cmux {
            let _ = cmux.send_to_preview(&cmd).await;
        }
    }

    /// Map screen coordinates to content-space coordinates using the cached preview layout.
    fn screen_to_content(
        &self,
        screen_col: u16,
        screen_row: u16,
    ) -> Option<crate::preview::state::ContentPos> {
        let pl = self.preview_layout?;
        layout::screen_to_content(pl, self.preview_state.scroll_offset, screen_col, screen_row)
    }

    fn reapply_git(&mut self) {
        if let Some(ref git) = self.git {
            git.apply_to_nodes(&mut self.tree.nodes);
        }
    }
}
