use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
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
use crate::preview::dispatcher::preview_command;
use crate::preview::loader::{load_preview, LoadedPreview};
use crate::preview::state::{ContentPos, FocusPane, PreviewKind, PreviewState};
use crate::render::colors;
use crate::render::preview_view::PreviewView;
use crate::render::status_bar::StatusBar;
use crate::render::tree_view::TreeView;
use crate::tree::forest::FileTree;
use crate::tree::node::NodeKind;

/// Cached layout coordinates of the preview content area (set during draw).
#[derive(Debug, Clone, Copy)]
struct PreviewLayout {
    /// Screen x where content text starts (after gutter).
    x: u16,
    /// Screen y where content starts (after header).
    y: u16,
    /// Height of the content area (excluding header).
    height: u16,
}

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
}

impl App {
    pub fn new(root: PathBuf) -> anyhow::Result<Self> {
        let config = Config::load();
        let mut tree = FileTree::new(
            root.clone(),
            config.tree.show_hidden,
            config.tree.dirs_first,
            config.tree.exclude.clone(),
            config.tree.show_ignored,
            config.tree.compact_folders,
            config.tree.show_size,
            config.tree.show_modified,
        );
        let git = GitState::load(&root);
        let cmux = CmuxBridge::detect(config.cmux.split_direction.clone());

        if let Some(ref git) = git {
            apply_git_statuses(&mut tree, git);
        }

        let preview_visible = config.preview.auto_preview;

        Ok(Self {
            tree,
            git,
            cmux,
            config,
            root,
            should_quit: false,
            tree_area_y: 0,
            tree_area_height: 0,
            preview_state: PreviewState::new(),
            preview_visible,
            focus: FocusPane::Tree,
            preview_debounce_handle: None,
            preview_area_x: None,
            preview_layout: None,
        })
    }

    pub async fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> anyhow::Result<()> {
        let mut reader = EventStream::new();

        // Set up file watcher with 100ms debounce
        let (fs_tx, mut fs_rx) = mpsc::channel::<()>(1);
        let _watcher = setup_watcher(&self.root, fs_tx);
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
                        Some(Ok(Event::Resize(_, _))) => {}
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
                show_size: self.config.tree.show_size,
                show_modified: self.config.tree.show_modified,
            }
            .render(tree_area, frame.buffer_mut(), &mut self.tree);

            // Render separator
            let sep_style = Style::default().fg(colors::TREE_LINE);
            for y in separator_area.y..separator_area.y + separator_area.height {
                frame
                    .buffer_mut()
                    .set_string(separator_area.x, y, "│", sep_style);
            }

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
                show_line_numbers: self.config.preview.show_line_numbers,
                focused: self.focus == FocusPane::Preview,
            }
            .render(preview_area, frame.buffer_mut(), &mut self.preview_state);
        } else {
            // No preview — full width tree
            self.preview_area_x = None;
            self.preview_layout = None;
            self.tree_area_height = main_area.height;

            TreeView {
                show_size: self.config.tree.show_size,
                show_modified: self.config.tree.show_modified,
            }
            .render(main_area, frame.buffer_mut(), &mut self.tree);
        }

        let root_name = self.root.file_name().map_or_else(
            || self.root.to_string_lossy().into_owned(),
            |n| n.to_string_lossy().into_owned(),
        );

        let (file_count, dir_count) = self.count_visible();
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
        let mut cursor_moved = false;

        match action {
            Action::Quit => self.should_quit = true,
            Action::CursorUp => {
                if self.focus == FocusPane::Preview {
                    self.preview_state.scroll_up(1);
                } else {
                    self.tree.cursor_up();
                    cursor_moved = true;
                }
            }
            Action::CursorDown => {
                if self.focus == FocusPane::Preview {
                    self.preview_state.scroll_down(1);
                } else {
                    self.tree.cursor_down();
                    cursor_moved = true;
                }
            }
            Action::CursorLeft => {
                if self.focus == FocusPane::Tree {
                    self.tree.cursor_left();
                    cursor_moved = true;
                }
            }
            Action::CursorRight => {
                if self.focus == FocusPane::Tree {
                    self.tree.cursor_right();
                    self.reapply_git();
                    cursor_moved = true;
                }
            }
            Action::Toggle => {
                let idx = self.tree.cursor;
                self.tree.toggle(idx);
                self.reapply_git();
                cursor_moved = true;
            }
            Action::Open => {
                self.open_selected().await;
                cursor_moved = true;
            }
            Action::Refresh => {
                self.tree.refresh();
                if let Some(ref mut git) = self.git {
                    git.refresh();
                }
                self.reapply_git();
                cursor_moved = true;
            }
            Action::ScrollUp(n) => {
                for _ in 0..n {
                    self.tree.cursor_up();
                }
                cursor_moved = true;
            }
            Action::ScrollDown(n) => {
                for _ in 0..n {
                    self.tree.cursor_down();
                }
                cursor_moved = true;
            }
            Action::GotoTop => {
                if self.focus == FocusPane::Preview {
                    self.preview_state.scroll_offset = 0;
                } else {
                    self.tree.cursor = 0;
                    cursor_moved = true;
                }
            }
            Action::GotoBottom => {
                if self.focus == FocusPane::Preview {
                    self.preview_state.scroll_offset =
                        self.preview_state.total_lines.saturating_sub(1);
                } else if !self.tree.is_empty() {
                    self.tree.cursor = self.tree.len() - 1;
                    cursor_moved = true;
                }
            }
            Action::SelectionStart(col, row) => {
                self.focus = FocusPane::Preview;
                if let Some(pos) = self.screen_to_content(col, row) {
                    self.preview_state.selection.anchor = Some(pos);
                    self.preview_state.selection.cursor = Some(pos);
                } else {
                    self.preview_state.selection.clear();
                }
            }
            Action::SelectionUpdate(col, row) => {
                if self.preview_state.selection.anchor.is_some() {
                    if let Some(pos) = self.screen_to_content(col, row) {
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
            Action::ClickRow(row) => {
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
                        // Clicked a file — open preview
                        self.preview_visible = true;
                        self.trigger_preview_load(preview_tx);
                    }
                }
            }
            Action::TogglePreview => {
                self.preview_visible = !self.preview_visible;
                if self.preview_visible {
                    self.trigger_preview_load(preview_tx);
                } else {
                    self.focus = FocusPane::Tree;
                }
            }
            Action::SwitchFocus => {
                self.focus = match self.focus {
                    FocusPane::Tree => FocusPane::Preview,
                    FocusPane::Preview => FocusPane::Tree,
                };
            }
            Action::PreviewScrollUp(n) => {
                self.preview_state.scroll_up(n as usize);
            }
            Action::PreviewScrollDown(n) => {
                self.preview_state.scroll_down(n as usize);
            }
            Action::None => {}
        }

        let _ = cursor_moved; // cursor movement no longer triggers preview
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

        // Skip if already showing this file
        if self.preview_state.current_path.as_ref() == Some(&path)
            && self.preview_state.kind != PreviewKind::Loading
        {
            return;
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

        self.preview_debounce_handle = Some(tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            let path_for_send = path.clone();
            let loaded = tokio::task::spawn_blocking(move || {
                load_preview(&path, max_file_size_kb, syntax_highlight, is_light)
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
    fn screen_to_content(&self, screen_col: u16, screen_row: u16) -> Option<ContentPos> {
        let layout = self.preview_layout?;

        // Check bounds
        if screen_row < layout.y || screen_row >= layout.y + layout.height || screen_col < layout.x
        {
            return None;
        }

        let row_in_content = (screen_row - layout.y) as usize;
        let line = self.preview_state.scroll_offset + row_in_content;
        let col = (screen_col - layout.x) as usize;

        Some(ContentPos { line, col })
    }

    fn reapply_git(&mut self) {
        if let Some(ref git) = self.git {
            apply_git_statuses(&mut self.tree, git);
        }
    }

    fn count_visible(&self) -> (usize, usize) {
        let files = self
            .tree
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::File)
            .count();
        let dirs = self
            .tree
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Directory)
            .count();
        (files, dirs)
    }
}

fn apply_git_statuses(tree: &mut FileTree, git: &GitState) {
    for node in &mut tree.nodes {
        node.git_status = git.status_for(&node.path, node.is_dir());
    }
}

/// Set up a file system watcher that sends a signal on changes (100ms debounce).
fn setup_watcher(
    root: &Path,
    tx: mpsc::Sender<()>,
) -> Option<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>> {
    let debouncer = new_debouncer(
        Duration::from_millis(100),
        move |events: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            if let Ok(events) = events {
                let has_real_change = events.iter().any(|e| e.kind == DebouncedEventKind::Any);
                if has_real_change {
                    let _ = tx.try_send(());
                }
            }
        },
    );

    match debouncer {
        Ok(mut d) => {
            if let Err(e) = d.watcher().watch(root, notify::RecursiveMode::Recursive) {
                eprintln!("croot: failed to watch {}: {e}", root.display());
                return None;
            }
            Some(d)
        }
        Err(e) => {
            eprintln!("croot: failed to initialize file watcher: {e}");
            None
        }
    }
}
