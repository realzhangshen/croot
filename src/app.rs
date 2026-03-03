use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{StatefulWidget, Widget},
    Terminal,
};
use tokio::sync::mpsc;

use crate::cmux::bridge::CmuxBridge;
use crate::config::Config;
use crate::git::status::GitState;
use crate::input::handler::{handle_key, Action};
use crate::input::mouse::handle_mouse;
use crate::preview::dispatcher::preview_command;
use crate::render::status_bar::StatusBar;
use crate::render::tree_view::TreeView;
use crate::tree::forest::FileTree;
use crate::tree::node::NodeKind;

pub struct App {
    pub tree: FileTree,
    pub git: Option<GitState>,
    pub cmux: Option<CmuxBridge>,
    #[allow(dead_code)] // preview/cmux config not yet consumed
    pub config: Config,
    pub root: PathBuf,
    pub should_quit: bool,
    tree_area_y: u16,
    tree_area_height: u16,
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
        );
        let git = GitState::load(&root);
        let cmux = CmuxBridge::detect();

        if let Some(ref git) = git {
            apply_git_statuses(&mut tree, git);
        }

        Ok(Self {
            tree,
            git,
            cmux,
            config,
            root,
            should_quit: false,
            tree_area_y: 0,
            tree_area_height: 0,
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

        loop {
            terminal.draw(|frame| self.draw(frame))?;

            tokio::select! {
                event = reader.next() => {
                    match event {
                        Some(Ok(Event::Key(key))) => {
                            let action = handle_key(key);
                            self.handle_action(action).await;
                        }
                        Some(Ok(Event::Mouse(mouse))) => {
                            let action = handle_mouse(mouse, self.tree_area_y, self.tree_area_height);
                            self.handle_action(action).await;
                        }
                        Some(Ok(Event::Resize(_, _))) => {}
                        Some(Err(_)) | None => break,
                        _ => {}
                    }
                }
                _ = fs_rx.recv() => {
                    // File system change detected — refresh git and expanded dirs
                    if let Some(ref mut git) = self.git {
                        git.refresh();
                    }
                    self.reapply_git();
                }
            }

            if self.should_quit {
                break;
            }
        }

        // Cleanup
        if let Some(ref mut cmux) = self.cmux {
            cmux.close_preview().await;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        let size = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(size);

        let tree_area = chunks[0];
        let status_area = chunks[1];

        self.tree_area_y = tree_area.y;
        self.tree_area_height = tree_area.height;

        TreeView.render(tree_area, frame.buffer_mut(), &mut self.tree);

        let root_name = self
            .root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.root.to_string_lossy().into_owned());

        let (file_count, dir_count) = self.count_visible();
        let branch = self.git.as_ref().and_then(|g| g.branch()).map(|s| s.to_string());
        let cmux_indicator = if self.cmux.is_some() { Some("cmux") } else { None };

        let status_bar = StatusBar {
            branch: branch.as_deref(),
            file_count,
            dir_count,
            root_name: &root_name,
            cmux_status: cmux_indicator,
        };
        status_bar.render(status_area, frame.buffer_mut());
    }

    async fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::CursorUp => self.tree.cursor_up(),
            Action::CursorDown => self.tree.cursor_down(),
            Action::CursorLeft => self.tree.cursor_left(),
            Action::CursorRight => {
                self.tree.cursor_right();
                self.reapply_git();
            }
            Action::Toggle => {
                let idx = self.tree.cursor;
                self.tree.toggle(idx);
                self.reapply_git();
            }
            Action::Open => self.open_selected().await,
            Action::Refresh => {
                self.tree.refresh();
                if let Some(ref mut git) = self.git {
                    git.refresh();
                }
                self.reapply_git();
            }
            Action::ScrollUp(n) => {
                for _ in 0..n {
                    self.tree.cursor_up();
                }
            }
            Action::ScrollDown(n) => {
                for _ in 0..n {
                    self.tree.cursor_down();
                }
            }
            Action::GotoTop => self.tree.cursor = 0,
            Action::GotoBottom => {
                if !self.tree.is_empty() {
                    self.tree.cursor = self.tree.len() - 1;
                }
            }
            Action::ClickRow(row) => {
                let idx = self.tree.scroll_offset + row as usize;
                if idx < self.tree.len() {
                    self.tree.cursor = idx;
                    if self.tree.nodes[idx].is_dir() {
                        self.tree.toggle(idx);
                        self.reapply_git();
                    }
                }
            }
            Action::None => {}
        }
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
                let has_real_change = events
                    .iter()
                    .any(|e| e.kind == DebouncedEventKind::Any);
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
