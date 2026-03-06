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
use crate::input::handler::{
    handle_key, handle_key_dialog, handle_key_menu, handle_key_search, Action, InputMode,
};
use crate::input::mouse::handle_mouse;
use crate::layout::{self, FocusPane, PreviewLayout};
use crate::preview::loader::{load_preview, LoadedPreview};
use crate::preview::state::{PreviewKind, PreviewState};
use crate::render::colors;
use crate::render::context_menu::{ContextMenuState, ContextMenuWidget, MenuAction};
use crate::render::input_dialog::{DialogKind, InputDialogState, InputDialogWidget};
use crate::render::preview_view::PreviewView;
use crate::render::search_bar::{fuzzy_match, SearchBar, SearchState};
use crate::render::status_bar::{HyperlinkRegion, StatusBar};
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
    hover_row: Option<usize>,
    // UI overlay state
    input_mode: InputMode,
    context_menu: Option<ContextMenuState>,
    input_dialog: Option<InputDialogState>,
    // Search state
    search_state: SearchState,
    /// Filtered node indices when search is active. Empty = no filter.
    search_filtered: Vec<usize>,
    // Hyperlink regions for post-render OSC 8 emission
    hyperlink_regions: Vec<HyperlinkRegion>,
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
            hover_row: None,
            input_mode: InputMode::Normal,
            context_menu: None,
            input_dialog: None,
            search_state: SearchState::new(),
            search_filtered: Vec::new(),
            hyperlink_regions: Vec::new(),
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
            self.emit_osc8_hyperlinks()?;

            tokio::select! {
                event = reader.next() => {
                    match event {
                        Some(Ok(Event::Key(key))) => {
                            let action = match self.input_mode {
                                InputMode::Normal => {
                                    let has_selection = self.preview_state.selection.is_active();
                                    let action = handle_key(key, self.preview_visible, has_selection);
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
                                InputMode::ContextMenu => handle_key_menu(key),
                                InputMode::Dialog => handle_key_dialog(key),
                                InputMode::Search => handle_key_search(key),
                            };
                            self.handle_action(action, &preview_tx).await;
                        }
                        Some(Ok(Event::Mouse(mouse))) => {
                            if self.input_mode == InputMode::ContextMenu {
                                self.handle_context_menu_mouse(mouse);
                            } else if self.input_mode == InputMode::Normal {
                                let action = handle_mouse(mouse, self.tree_area_y, self.tree_area_height, self.preview_area_x);
                                self.handle_action(action, &preview_tx).await;
                            }
                        }
                        Some(Ok(Event::Resize(_, _))) => {
                            self.context_menu = None;
                            self.input_mode = InputMode::Normal;
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
                        watcher_active = false;
                        continue;
                    }
                    self.tree.refresh();
                    if let Some(ref mut git) = self.git {
                        git.refresh();
                    }
                    self.reapply_git();
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

    fn emit_osc8_hyperlinks(&self) -> anyhow::Result<()> {
        use std::io::Write;
        let mut stdout = std::io::stdout();
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
            crossterm::queue!(
                stdout,
                crossterm::style::SetAttribute(crossterm::style::Attribute::Reset)
            )?;
        }
        stdout.flush()?;
        Ok(())
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        let size = frame.area();

        let show_search_bar = self.input_mode == InputMode::Search
            || !self.search_state.is_empty();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if show_search_bar {
                vec![
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]
            } else {
                vec![Constraint::Min(1), Constraint::Length(1)]
            })
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

            TreeView {
                config: &self.config.tree,
                hover_row: self.hover_row,
                filter_indices: &self.search_filtered,
            }
            .render(tree_area, frame.buffer_mut(), &mut self.tree);

            let sep_style = Style::default().fg(colors::TREE_LINE);
            for y in separator_area.y..separator_area.y + separator_area.height {
                frame
                    .buffer_mut()
                    .set_string(separator_area.x, y, "│", sep_style);
            }

            self.preview_content_width = preview_width;
            self.preview_area_x = Some(preview_area.x);

            let content_area_y = preview_area.y + 1;
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

            PreviewView {
                config: &self.config.preview,
                focused: self.focus == FocusPane::Preview,
            }
            .render(preview_area, frame.buffer_mut(), &mut self.preview_state);
        } else {
            self.preview_area_x = None;
            self.preview_layout = None;
            self.tree_area_height = main_area.height;

            TreeView {
                config: &self.config.tree,
                hover_row: self.hover_row,
                filter_indices: &self.search_filtered,
            }
            .render(main_area, frame.buffer_mut(), &mut self.tree);
        }

        let root_name = self.root.file_name().map_or_else(
            || self.root.to_string_lossy().into_owned(),
            |n| n.to_string_lossy().into_owned(),
        );
        let root_path = self.root.to_string_lossy().into_owned();

        let selected_rel = self.tree.selected().and_then(|n| {
            if n.is_dir() {
                None
            } else {
                n.path
                    .strip_prefix(&self.root)
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned())
            }
        });
        let selected_abs = self.tree.selected().and_then(|n| {
            if n.is_dir() {
                None
            } else {
                Some(n.path.to_string_lossy().into_owned())
            }
        });

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
            root_path: &root_path,
            cmux_status: cmux_indicator,
            selected_path: selected_rel.as_deref(),
            selected_abs_path: selected_abs.as_deref(),
        };
        self.hyperlink_regions = status_bar.hyperlink_regions(status_area);
        status_bar.render(status_area, frame.buffer_mut());

        // Search bar (shown when in search mode or filter is active)
        if show_search_bar {
            let search_area = chunks[2];
            let search_bar = SearchBar {
                state: &self.search_state,
            };
            search_bar.render(search_area, frame.buffer_mut());
        }

        // Render overlays (context menu / input dialog)
        if let Some(ref menu) = self.context_menu {
            let widget = ContextMenuWidget { state: menu };
            widget.render(size, frame.buffer_mut());
        }

        if let Some(ref dialog) = self.input_dialog {
            let widget = InputDialogWidget { state: dialog };
            widget.render(size, frame.buffer_mut());
        }
    }

    async fn handle_action(
        &mut self,
        action: Action,
        preview_tx: &mpsc::Sender<(PathBuf, LoadedPreview)>,
    ) {
        match action {
            Action::Quit => {
                if self.input_mode == InputMode::Normal {
                    self.should_quit = true;
                } else {
                    self.input_mode = InputMode::Normal;
                    self.context_menu = None;
                    self.input_dialog = None;
                }
            }

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

            Action::Hover(col, row) => {
                self.update_hover(col, row);
            }

            // Right-click context menu
            Action::RightClick(col, row) => {
                self.open_context_menu(col, row);
            }

            // Context menu actions
            Action::MenuClose => {
                self.context_menu = None;
                self.input_mode = InputMode::Normal;
            }
            Action::MenuUp => {
                if let Some(ref mut menu) = self.context_menu {
                    menu.move_up();
                }
            }
            Action::MenuDown => {
                if let Some(ref mut menu) = self.context_menu {
                    menu.move_down();
                }
            }
            Action::MenuSelect(ref _placeholder) => {
                // Resolve actual action from selected menu item
                if let Some(menu) = self.context_menu.take() {
                    let menu_action = menu.selected_action().clone();
                    self.input_mode = InputMode::Normal;
                    self.execute_menu_action(&menu_action, menu.node_idx, preview_tx)
                        .await;
                }
            }
            // File operations (keyboard shortcuts)
            Action::NewFile => self.start_new_file(),
            Action::NewDir => self.start_new_dir(),
            Action::RenameNode => self.start_rename(),
            Action::DeleteNode => self.start_delete(),

            // Dialog actions
            Action::DialogChar(ch) => {
                if let Some(ref mut dialog) = self.input_dialog {
                    dialog.insert_char(ch);
                }
            }
            Action::DialogBackspace => {
                if let Some(ref mut dialog) = self.input_dialog {
                    dialog.delete_char();
                }
            }
            Action::DialogLeft => {
                if let Some(ref mut dialog) = self.input_dialog {
                    dialog.move_left();
                }
            }
            Action::DialogRight => {
                if let Some(ref mut dialog) = self.input_dialog {
                    dialog.move_right();
                }
            }
            Action::DialogConfirm => {
                self.confirm_dialog();
            }
            Action::DialogCancel => {
                self.input_dialog = None;
                self.input_mode = InputMode::Normal;
            }

            // Multi-select
            Action::ToggleSelect => {
                self.tree.toggle_select();
                self.tree.cursor_down();
            }
            Action::ClearSelect => {
                self.tree.clear_selection();
            }
            Action::DeleteSelected => {
                self.delete_selected();
            }

            // Search actions
            Action::StartSearch => {
                self.input_mode = InputMode::Search;
                self.search_state.clear();
                self.search_filtered.clear();
            }
            Action::SearchChar(ch) => {
                self.search_state.insert_char(ch);
                self.update_search_filter();
            }
            Action::SearchBackspace => {
                self.search_state.delete_char();
                self.update_search_filter();
            }
            Action::SearchLeft => {
                self.search_state.move_left();
            }
            Action::SearchRight => {
                self.search_state.move_right();
            }
            Action::SearchConfirm => {
                self.input_mode = InputMode::Normal;
                // Keep the filter active
            }
            Action::SearchCancel => {
                self.input_mode = InputMode::Normal;
                self.search_state.clear();
                self.search_filtered.clear();
            }
            Action::SearchNext => {
                self.search_navigate_next();
            }
            Action::SearchPrev => {
                self.search_navigate_prev();
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

        if node.is_dir() {
            self.preview_state.clear();
            return;
        }

        let path = node.path.clone();

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

        if let Some(handle) = self.preview_debounce_handle.take() {
            handle.abort();
        }

        self.preview_state.kind = PreviewKind::Loading;

        let tx = preview_tx.clone();
        let delay = Duration::from_millis(self.config.preview.preview_delay_ms);
        let max_file_size_kb = self.config.preview.max_file_size_kb;
        let syntax_highlight = self.config.preview.syntax_highlight;
        let render_markdown = self.preview_state.render_markdown;
        let preview_width = self.preview_content_width as usize;

        self.preview_debounce_handle = Some(tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            let path_for_send = path.clone();
            let loaded = tokio::task::spawn_blocking(move || {
                load_preview(
                    &path,
                    max_file_size_kb,
                    syntax_highlight,
                    render_markdown,
                    preview_width,
                )
            })
            .await;

            if let Ok(loaded) = loaded {
                let _ = tx.send((path_for_send, loaded)).await;
            }
        }));
    }

    fn update_hover(&mut self, col: u16, row: u16) {
        if self.preview_area_x.is_some_and(|px| col >= px) {
            self.hover_row = None;
            return;
        }
        if row >= self.tree_area_y && row < self.tree_area_y + self.tree_area_height {
            let relative_row = (row - self.tree_area_y) as usize;
            if relative_row < self.tree.rendered_indices.len() {
                self.hover_row = Some(relative_row);
            } else {
                self.hover_row = None;
            }
        } else {
            self.hover_row = None;
        }
    }

    // ── Context menu ────────────────────────────────────────────────────

    fn open_context_menu(&mut self, col: u16, row: u16) {
        // Exclude preview pane and separator
        if self.preview_area_x.is_some_and(|px| col >= px.saturating_sub(1)) {
            return;
        }
        if row < self.tree_area_y || row >= self.tree_area_y + self.tree_area_height {
            return;
        }
        let relative_row = (row - self.tree_area_y) as usize;
        let menu = if relative_row >= self.tree.rendered_indices.len() {
            // Empty space below tree items → workspace root menu
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
                ContextMenuState::new_for_file(col, row, node_idx)
            }
        };

        self.context_menu = Some(menu);
        self.input_mode = InputMode::ContextMenu;
    }

    fn handle_context_menu_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(ref menu) = self.context_menu {
                    let tw = self.main_area_width;
                    let th = self.tree_area_y + self.tree_area_height + 1;
                    if menu.contains(mouse.column, mouse.row, tw, th) {
                        if let Some(idx) = menu.row_to_item(mouse.row, tw, th) {
                            let menu_action = menu.items[idx].action.clone();
                            let node_idx = menu.node_idx;
                            self.context_menu = None;
                            self.input_mode = InputMode::Normal;
                            // Can't call async from here, so store for later
                            // Actually we can use a simpler approach: match synchronously
                            self.execute_menu_action_sync(&menu_action, node_idx);
                        }
                    } else {
                        self.context_menu = None;
                        self.input_mode = InputMode::Normal;
                    }
                }
            }
            MouseEventKind::Moved => {
                if let Some(ref mut menu) = self.context_menu {
                    let tw = self.main_area_width;
                    let th = self.tree_area_y + self.tree_area_height + 1;
                    if let Some(idx) = menu.row_to_item(mouse.row, tw, th) {
                        menu.selected = idx;
                    }
                }
            }
            _ => {
                // Any other click closes the menu
                if matches!(mouse.kind, MouseEventKind::Down(_)) {
                    self.context_menu = None;
                    self.input_mode = InputMode::Normal;
                }
            }
        }
    }

    async fn execute_menu_action(
        &mut self,
        action: &MenuAction,
        node_idx: usize,
        preview_tx: &mpsc::Sender<(PathBuf, LoadedPreview)>,
    ) {
        match action {
            MenuAction::CopyPath => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    let rel = node
                        .path
                        .strip_prefix(&self.root)
                        .unwrap_or(&node.path)
                        .to_string_lossy()
                        .into_owned();
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(rel);
                    }
                }
            }
            MenuAction::CopyAbsPath => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    let abs = node.path.to_string_lossy().into_owned();
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(abs);
                    }
                }
            }
            MenuAction::RevealInFinder => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    let _ = std::process::Command::new("open")
                        .arg("-R")
                        .arg(&node.path)
                        .spawn();
                }
            }
            MenuAction::NewFile => self.start_new_file_at(node_idx),
            MenuAction::NewDir => self.start_new_dir_at(node_idx),
            MenuAction::Rename => self.start_rename_at(node_idx),
            MenuAction::Delete => self.start_delete_at(node_idx),
        }

        // Refresh preview after menu actions that modify files
        if matches!(action, MenuAction::NewFile | MenuAction::NewDir | MenuAction::Rename | MenuAction::Delete) {
            // Refresh handled in confirm_dialog
        } else if self.preview_visible {
            self.trigger_preview_load(preview_tx);
        }
    }

    /// Synchronous version for mouse click handler (non-async context).
    fn execute_menu_action_sync(&mut self, action: &MenuAction, node_idx: usize) {
        match action {
            MenuAction::CopyPath => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    let rel = node
                        .path
                        .strip_prefix(&self.root)
                        .unwrap_or(&node.path)
                        .to_string_lossy()
                        .into_owned();
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(rel);
                    }
                }
            }
            MenuAction::CopyAbsPath => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    let abs = node.path.to_string_lossy().into_owned();
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(abs);
                    }
                }
            }
            MenuAction::RevealInFinder => {
                if let Some(node) = self.tree.nodes.get(node_idx) {
                    let _ = std::process::Command::new("open")
                        .arg("-R")
                        .arg(&node.path)
                        .spawn();
                }
            }
            MenuAction::NewFile => self.start_new_file_at(node_idx),
            MenuAction::NewDir => self.start_new_dir_at(node_idx),
            MenuAction::Rename => self.start_rename_at(node_idx),
            MenuAction::Delete => self.start_delete_at(node_idx),
        }
    }

    // ── File operations ─────────────────────────────────────────────────

    fn start_new_file(&mut self) {
        let dir = self.current_dir();
        self.input_dialog = Some(InputDialogState::new(
            DialogKind::NewFile,
            dir,
            String::new(),
        ));
        self.input_mode = InputMode::Dialog;
    }

    fn start_new_dir(&mut self) {
        let dir = self.current_dir();
        self.input_dialog = Some(InputDialogState::new(
            DialogKind::NewDir,
            dir,
            String::new(),
        ));
        self.input_mode = InputMode::Dialog;
    }

    fn start_rename(&mut self) {
        if let Some(node) = self.tree.selected() {
            let name = node.name.clone();
            let path = node.path.clone();
            self.input_dialog = Some(InputDialogState::new(DialogKind::Rename, path, name));
            self.input_mode = InputMode::Dialog;
        }
    }

    fn start_delete(&mut self) {
        if let Some(node) = self.tree.selected() {
            let name = node.name.clone();
            let path = node.path.clone();
            self.input_dialog = Some(InputDialogState::new(
                DialogKind::ConfirmDelete,
                path,
                name,
            ));
            self.input_mode = InputMode::Dialog;
        }
    }

    fn start_new_file_at(&mut self, node_idx: usize) {
        let dir = self.dir_for_node(node_idx);
        self.input_dialog = Some(InputDialogState::new(
            DialogKind::NewFile,
            dir,
            String::new(),
        ));
        self.input_mode = InputMode::Dialog;
    }

    fn start_new_dir_at(&mut self, node_idx: usize) {
        let dir = self.dir_for_node(node_idx);
        self.input_dialog = Some(InputDialogState::new(
            DialogKind::NewDir,
            dir,
            String::new(),
        ));
        self.input_mode = InputMode::Dialog;
    }

    fn start_rename_at(&mut self, node_idx: usize) {
        if let Some(node) = self.tree.nodes.get(node_idx) {
            let name = node.name.clone();
            let path = node.path.clone();
            self.input_dialog = Some(InputDialogState::new(DialogKind::Rename, path, name));
            self.input_mode = InputMode::Dialog;
        }
    }

    fn start_delete_at(&mut self, node_idx: usize) {
        if let Some(node) = self.tree.nodes.get(node_idx) {
            let name = node.name.clone();
            let path = node.path.clone();
            self.input_dialog = Some(InputDialogState::new(
                DialogKind::ConfirmDelete,
                path,
                name,
            ));
            self.input_mode = InputMode::Dialog;
        }
    }

    fn confirm_dialog(&mut self) {
        let Some(dialog) = self.input_dialog.take() else {
            return;
        };
        self.input_mode = InputMode::Normal;

        match dialog.kind {
            DialogKind::NewFile => {
                if !dialog.input.is_empty() {
                    let new_path = dialog.context_path.join(&dialog.input);
                    // Create parent dirs if needed
                    if let Some(parent) = new_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::File::create(&new_path);
                }
            }
            DialogKind::NewDir => {
                if !dialog.input.is_empty() {
                    let new_path = dialog.context_path.join(&dialog.input);
                    let _ = std::fs::create_dir_all(&new_path);
                }
            }
            DialogKind::Rename => {
                if !dialog.input.is_empty() && dialog.input != dialog.target_name {
                    if let Some(parent) = dialog.context_path.parent() {
                        let new_path = parent.join(&dialog.input);
                        let _ = std::fs::rename(&dialog.context_path, &new_path);
                    }
                }
            }
            DialogKind::ConfirmDelete => {
                if self.tree.selected_set.is_empty() {
                    let path = &dialog.context_path;
                    if path.is_dir() {
                        let _ = std::fs::remove_dir_all(path);
                    } else {
                        let _ = std::fs::remove_file(path);
                    }
                } else {
                    let paths = self.tree.selected_paths();
                    for path in &paths {
                        if path.is_dir() {
                            let _ = std::fs::remove_dir_all(path);
                        } else {
                            let _ = std::fs::remove_file(path);
                        }
                    }
                    self.tree.clear_selection();
                }
            }
        }

        // Refresh tree after any file operation
        self.tree.refresh();
        if let Some(ref mut git) = self.git {
            git.refresh();
        }
        self.reapply_git();
    }

    /// Get the directory context for the currently selected node.
    fn current_dir(&self) -> PathBuf {
        if let Some(node) = self.tree.selected() {
            if node.is_dir() {
                node.path.clone()
            } else {
                node.path.parent().unwrap_or(&self.root).to_path_buf()
            }
        } else {
            self.root.clone()
        }
    }

    /// Get the directory for a given node (node itself if dir, or its parent).
    fn dir_for_node(&self, node_idx: usize) -> PathBuf {
        if let Some(node) = self.tree.nodes.get(node_idx) {
            if node.is_dir() {
                node.path.clone()
            } else {
                node.path.parent().unwrap_or(&self.root).to_path_buf()
            }
        } else {
            self.root.clone()
        }
    }

    // ── Batch operations ──────────────────────────────────────────────────

    fn delete_selected(&mut self) {
        if self.tree.selected_set.is_empty() {
            self.start_delete();
            return;
        }

        let paths = self.tree.selected_paths();
        let count = paths.len();
        let name = format!("{count} items");

        // Use the first path as context
        let context = paths.first().cloned().unwrap_or_else(|| self.root.clone());
        self.input_dialog = Some(InputDialogState::new(
            DialogKind::ConfirmDelete,
            context,
            name,
        ));
        self.input_mode = InputMode::Dialog;
    }

    // ── Search ───────────────────────────────────────────────────────────

    fn update_search_filter(&mut self) {
        if self.search_state.query.is_empty() {
            self.search_filtered.clear();
            self.search_state.match_count = 0;
            return;
        }

        self.search_filtered = self
            .tree
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                // Match against the node name or its relative path
                let rel_path = node
                    .path
                    .strip_prefix(&self.root)
                    .unwrap_or(&node.path)
                    .to_string_lossy();
                fuzzy_match(&self.search_state.query, &rel_path)
                    || fuzzy_match(&self.search_state.query, &node.name)
            })
            .map(|(i, _)| i)
            .collect();

        self.search_state.match_count = self.search_filtered.len();

        // Move cursor to first match if current cursor isn't in results
        if !self.search_filtered.is_empty()
            && !self.search_filtered.contains(&self.tree.cursor)
        {
            self.tree.cursor = self.search_filtered[0];
        }
    }

    fn search_navigate_next(&mut self) {
        if self.search_filtered.is_empty() {
            return;
        }
        // Find the next filtered index after current cursor
        let next = self
            .search_filtered
            .iter()
            .find(|&&idx| idx > self.tree.cursor)
            .or_else(|| self.search_filtered.first());
        if let Some(&idx) = next {
            self.tree.cursor = idx;
        }
    }

    fn search_navigate_prev(&mut self) {
        if self.search_filtered.is_empty() {
            return;
        }
        let prev = self
            .search_filtered
            .iter()
            .rev()
            .find(|&&idx| idx < self.tree.cursor)
            .or_else(|| self.search_filtered.last());
        if let Some(&idx) = prev {
            self.tree.cursor = idx;
        }
    }

    // ── Utility ─────────────────────────────────────────────────────────

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
